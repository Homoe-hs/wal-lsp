use lsp_types::{Diagnostic, DiagnosticSeverity};
use std::sync::LazyLock;
use tree_sitter::Node;
use crate::wal::rules::{LintContext, LintSeverity, Rule, RuleDescriptor, get_form_name, get_args};

pub struct StructureRule;

static STRUCTURE_DESC: LazyLock<RuleDescriptor> = LazyLock::new(|| RuleDescriptor {
    id: "form-structure",
    name: "Validate form structure",
    description: "Checks structures of special forms: let, case, fn, defun",
    default_enabled: true,
    default_severity: LintSeverity::Error,
});

impl Rule for StructureRule {
    fn descriptor(&self) -> &RuleDescriptor { &STRUCTURE_DESC }
    fn check(&self, node: Node, ctx: &LintContext) -> Vec<Diagnostic> {
        if node.kind() != "list" { return vec![]; }
        let (fn_name, _) = match get_form_name(node, ctx.source) {
            Some(v) => v,
            None => return vec![],
        };
        match fn_name.as_str() {
            "let" => check_let(node, ctx),
            "case" => check_case(node, ctx),
            "defun" | "fn" => check_fn_params(node, ctx, &fn_name),
            _ => vec![],
        }
    }
}

fn get_bracket_contents<'a>(node: Node<'a>, _source: &str) -> Vec<Vec<Node<'a>>> {
    let list_node = if node.kind() == "sexpr" {
        let mut c = node.walk();
        let children: Vec<Node> = node.children(&mut c).collect();
        children.into_iter().find(|ch| ch.kind() == "list")
    } else if node.kind() == "list" {
        Some(node)
    } else { None };
    let list_node = match list_node { Some(n) => n, None => return vec![] };
    // Get inner sexprs of the bracket list
    let mut lc = list_node.walk();
    let children: Vec<_> = list_node.children(&mut lc).collect();
    let sexpr_list = match children.iter().find(|c| c.kind() == "sexpr_list") {
        Some(sl) => *sl, None => return vec![]
    };
    let mut sc = sexpr_list.walk();
    let inner: Vec<Node> = sexpr_list.children(&mut sc)
        .filter(|ch| ch.kind() == "sexpr").collect();
    if inner.is_empty() { return vec![]; }
    // Check if first element is itself a bracket list (e.g. [[a 1] [b 2]])
    {
        let mut fc = inner[0].walk();
        let has_nested = inner[0].children(&mut fc).any(|ch| ch.kind() == "list");
        if !has_nested { return vec![inner]; }
    }
    // Nested case
    inner.iter().map(|s| {
        let mut c = s.walk();
        let list = s.children(&mut c).find(|ch| ch.kind() == "list");
        match list {
            Some(l) => {
                let mut lc2 = l.walk();
                let children2: Vec<Node> = l.children(&mut lc2).collect();
                match children2.iter().find(|ch| ch.kind() == "sexpr_list") {
                    Some(sl2) => {
                        let mut sc2 = sl2.walk();
                        sl2.children(&mut sc2).filter(|ch| ch.kind() == "sexpr").collect()
                    }
                    None => vec![],
                }
            }
            None => vec![],
        }
    }).collect()
}

fn is_atom_like(node: Node, source: &str) -> bool {
    let mut c = node.walk();
    node.children(&mut c).any(|ch| {
        matches!(ch.kind(), "atom" | "symbol" | "base_symbol" | "scoped_symbol"
            | "grouped_symbol" | "operator" | "int" | "float" | "string" | "bool")
    }) || source.get(node.byte_range()).map_or(false, |s| {
        let t = s.trim();
        !t.starts_with('(') && !t.starts_with('[') && !t.starts_with('{')
    })
}

fn check_let(node: Node, ctx: &LintContext) -> Vec<Diagnostic> {
    let args = get_args(node);
    if args.len() < 1 {
        let range = ctx.range_from(node.start_position(), "let".len());
        return vec![Diagnostic {
            range, severity: Some(DiagnosticSeverity::ERROR),
            message: "let expects at least one binding pair and a body".to_string(),
            ..Default::default()
        }];
    }
    let binding_node = args[0];
    let bindings = get_bracket_contents(binding_node, ctx.source);
    if bindings.is_empty() || bindings[0].is_empty() {
        let pos = binding_node.start_position();
        let range = ctx.range_from(pos, 1);
        return vec![Diagnostic {
            range, severity: Some(DiagnosticSeverity::ERROR),
            message: "let expects a binding list in brackets [...]".to_string(),
            ..Default::default()
        }];
    }
    let mut diags = Vec::new();
    for (i, binding) in bindings.iter().enumerate() {
        if binding.len() < 2 {
            let pos = binding.first().map(|n| n.start_position()).unwrap_or(node.start_position());
            let range = ctx.range_from(pos, 1);
            diags.push(Diagnostic {
                range, severity: Some(DiagnosticSeverity::ERROR),
                message: format!("let binding #{} is missing a value", i + 1),
                ..Default::default()
            });
            continue;
        }
        if !is_atom_like(binding[0], ctx.source) {
            let text = ctx.node_text(binding[0]);
            let range = ctx.range_from(binding[0].start_position(), text.len());
            diags.push(Diagnostic {
                range, severity: Some(DiagnosticSeverity::ERROR),
                message: format!("let binding #{} id must be a symbol, got '{}'", i + 1, text),
                ..Default::default()
            });
        }
    }
    diags
}

fn check_case(node: Node, ctx: &LintContext) -> Vec<Diagnostic> {
    let args = get_args(node);
    if args.len() < 2 {
        let range = ctx.range_from(node.start_position(), "case".len());
        return vec![Diagnostic {
            range, severity: Some(DiagnosticSeverity::ERROR),
            message: "case expects a key and at least one clause".to_string(),
            ..Default::default()
        }];
    }
    let mut diags = Vec::new();
    let mut seen_default = false;
    for (i, clause) in args.iter().enumerate().skip(1) {
        let contents = get_bracket_contents(*clause, ctx.source);
        if contents.is_empty() || contents[0].is_empty() {
            let pos = clause.start_position();
            let range = ctx.range_from(pos, 1);
            diags.push(Diagnostic {
                range, severity: Some(DiagnosticSeverity::ERROR),
                message: format!("case clause #{} is empty", i),
                ..Default::default()
            });
            continue;
        }
        let first_text = contents[0].first().and_then(|n| ctx.source.get(n.byte_range()).map(|s| s.trim().to_string()));
        if first_text.as_deref() == Some("default") { seen_default = true; }
        if seen_default && first_text.as_deref() != Some("default") {
            let pos = clause.start_position();
            let range = ctx.range_from(pos, 1);
            diags.push(Diagnostic {
                range, severity: Some(DiagnosticSeverity::WARNING),
                message: format!("case clause #{} is unreachable (after default)", i),
                ..Default::default()
            });
        }
    }
    diags
}

fn check_fn_params(node: Node, ctx: &LintContext, fn_name: &str) -> Vec<Diagnostic> {
    let args = get_args(node);
    if args.is_empty() { return vec![]; }
    let params_node = args[0];
    // 变参: 单符号参数跳过
    let mut pc = params_node.walk();
    let has_bracket = params_node.children(&mut pc).any(|ch| ch.kind() == "list");
    if !has_bracket { return vec![]; }
    let params = get_bracket_contents(params_node, ctx.source);
    let mut diags = Vec::new();
    for (i, group) in params.iter().enumerate() {
        let param = match group.first() { Some(p) => *p, None => continue };
        if !is_atom_like(param, ctx.source) {
            let text = ctx.node_text(param);
            let range = ctx.range_from(param.start_position(), text.len());
            diags.push(Diagnostic {
                range, severity: Some(DiagnosticSeverity::ERROR),
                message: format!("{} parameter #{} must be a symbol, got '{}'", fn_name, i + 1, text),
                ..Default::default()
            });
        }
    }
    diags
}
