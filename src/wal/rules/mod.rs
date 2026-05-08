pub mod arity;
pub mod unknown_symbol;
pub mod structure;

use std::collections::{HashMap, HashSet};
use tree_sitter::Node;
use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

/// 规则严重度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintSeverity {
    Error,
    Warning,
    Info,
}

impl LintSeverity {
    pub fn to_lsp(&self) -> DiagnosticSeverity {
        match self {
            LintSeverity::Error => DiagnosticSeverity::ERROR,
            LintSeverity::Warning => DiagnosticSeverity::WARNING,
            LintSeverity::Info => DiagnosticSeverity::INFORMATION,
        }
    }
}

/// 规则描述
#[derive(Debug, Clone)]
pub struct RuleDescriptor {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub default_enabled: bool,
    pub default_severity: LintSeverity,
}

/// 规则的上下文信息
pub struct LintContext<'a> {
    pub source: &'a str,
    pub user_symbols: &'a HashSet<String>,
    pub line_suppressions: &'a HashMap<u32, Vec<String>>,
}

impl<'a> LintContext<'a> {
    pub fn range_from(&self, pos: tree_sitter::Point, len: usize) -> Range {
        Range::new(
            Position::new(pos.row as u32, pos.column as u32),
            Position::new(pos.row as u32, (pos.column + len) as u32),
        )
    }

    pub fn node_text(&self, node: Node) -> String {
        self.source.get(node.byte_range()).unwrap_or("").trim().to_string()
    }

    pub fn is_suppressed(&self, line: u32, rule_id: &str) -> bool {
        self.line_suppressions
            .get(&line)
            .map_or(false, |rules| rules.iter().any(|r| r == rule_id || r == "all"))
    }
}

/// 单条规则
pub trait Rule: Send + Sync {
    fn descriptor(&self) -> &RuleDescriptor;
    fn check(&self, node: Node, ctx: &LintContext) -> Vec<Diagnostic>;
}

/// 规则注册表
pub struct RuleRegistry {
    rules: Vec<Box<dyn Rule>>,
    enabled: HashMap<String, bool>,
}

impl RuleRegistry {
    pub fn new() -> Self {
        Self { rules: Vec::new(), enabled: HashMap::new() }
    }

    pub fn register(&mut self, rule: Box<dyn Rule>) {
        let id = rule.descriptor().id.to_string();
        let default = rule.descriptor().default_enabled;
        let enabled = crate::config::CONFIG
            .read()
            .map(|cfg| cfg.is_rule_enabled(&id, default))
            .unwrap_or(default);
        self.enabled.entry(id).or_insert(enabled);
        self.rules.push(rule);
    }

    pub fn is_enabled(&self, rule_id: &str) -> bool {
        self.enabled.get(rule_id).copied().unwrap_or(false)
    }

    pub fn check_node(&self, node: Node, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut all = Vec::new();
        for rule in &self.rules {
            let desc = rule.descriptor();
            if !self.is_enabled(desc.id) { continue; }
            if ctx.is_suppressed(node.start_position().row as u32, desc.id) { continue; }
            let mut diags = rule.check(node, ctx);
            for d in diags.iter_mut() {
                if d.source.is_none() {
                    d.source = Some(format!("wal-lsp:{}", desc.id).into());
                }
            }
            all.extend(diags);
        }
        all
    }

    pub fn check_all(&self, root: Node, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut results = Vec::new();
        self.collect_list_nodes(root, &mut results, ctx);
        results
    }

    fn collect_list_nodes(&self, node: Node, results: &mut Vec<Diagnostic>, ctx: &LintContext) {
        if node.kind() == "list" {
            results.extend(self.check_node(node, ctx));
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_list_nodes(child, results, ctx);
        }
    }
}

impl Default for RuleRegistry { fn default() -> Self { Self::new() } }

/// 解析 ;; lint_off / ;; lint_on 注释
/// - lint_off: 从下一行开始禁用指定规则 (或 "all")
/// - lint_on:  从下一行开始启用指定规则
pub fn parse_suppressions(source: &str) -> HashMap<u32, Vec<String>> {
    let mut suppressions: HashMap<u32, Vec<String>> = HashMap::new();
    let mut active: Vec<String> = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if !trimmed.starts_with(";;") {
            // 应用已激活的抑制到当前行
            for r in &active {
                suppressions.entry(line_num as u32).or_default().push(r.clone());
            }
            continue;
        }
        let comment = trimmed.trim_start_matches(";;").trim();

        if let Some(rest) = comment.strip_prefix("lint_off ") {
            let targets = rest.trim();
            for rule in targets.split(',') {
                let r = rule.trim().to_string();
                if !r.is_empty() { active.push(r); }
            }
            // 从下一行开始生效
            continue;
        }

        if comment == "lint_off" {
            active.push("all".to_string());
            continue;
        }

        // 应用已激活的抑制到当前行
        for r in &active {
            suppressions.entry(line_num as u32).or_default().push(r.clone());
        }

        if let Some(rest) = comment.strip_prefix("lint_on ") {
            let targets = rest.trim();
            for rule in targets.split(',') {
                let r = rule.trim().to_string();
                active.retain(|a| *a != r);
            }
        }
        if comment == "lint_on" { active.clear(); }
    }
    suppressions
}

/// 获取列表节点中的函数名和括号类型
pub fn get_form_name(node: Node, source: &str) -> Option<(String, String)> {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();
    let bracket = children.first().map(|c| {
        source.get(c.byte_range()).map(|s| s.trim().to_string()).unwrap_or_default()
    }).unwrap_or_default();
    let sexpr_list = children.iter().find(|c| c.kind() == "sexpr_list")?;
    let sl_children: Vec<Node> = {
        let mut c = sexpr_list.walk();
        sexpr_list.children(&mut c).filter(|child| child.kind() == "sexpr").collect()
    };
    if sl_children.is_empty() { return None; }
    let fn_sexpr = sl_children[0];
    let mut fc = fn_sexpr.walk();
    let fn_text = fn_sexpr.children(&mut fc)
        .find(|a| a.kind() == "atom")
        .and_then(|a| source.get(a.byte_range()).map(|s| s.trim().to_string()));
    fn_text.map(|t| (t, bracket))
}

/// 获取列表节点中的参数 (跳过第一个函数名)
pub fn get_args(node: Node) -> Vec<Node> {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();
    let sexpr_list = match children.iter().find(|c| c.kind() == "sexpr_list") {
        Some(sl) => sl, None => return vec![]
    };
    let mut c = sexpr_list.walk();
    sexpr_list.children(&mut c)
        .filter(|child| child.kind() == "sexpr")
        .skip(1).collect()
}
