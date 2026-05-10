use std::collections::HashSet;
use lsp_types::{Diagnostic, DiagnosticSeverity};
use tree_sitter::Node;
use crate::wal::rules::{LintContext, LintSeverity, Rule, RuleDescriptor, get_form_name};

/// 所有已知 WAL 符号
const KNOWN_SYMBOLS: &[&str] = &[
    "+", "-", "*", "/", "**", "floor", "ceil", "round", "mod", "sum",
    "!", "not", "=", "!=", ">", "<", ">=", "<=", "&&", "||", "and", "or",
    "bor", "band", "bxor",
    "print", "printf", "set", "set!", "define", "let", "if", "case",
    "when", "unless", "cond", "while", "do", "exit",
    "fn", "defmacro", "macroexpand", "gensym", "type", "alias", "unalias",
    "quote", "quasiquote", "unquote", "unquote-splice",
    "eval", "parse", "rel_eval", "slice", "get", "call", "import",
    "list", "first", "second", "last", "rest", "in", "map",
    "max", "min", "fold", "length", "average", "zip",

    "defined?", "atom?", "symbol?", "string?", "int?", "list?",
    "null?", "signal?",
    "convert/bin", "string->int", "bits->sint",
    "symbol->string", "string->symbol", "int->string",
    "string-append",
    "abs", "reval",
    "load", "unload", "step", "eval-file",
    "require", "repl", "loaded-traces",
    "signals", "index", "max-index", "ts", "trace-name", "trace-file",
    "find", "find/g", "whenever", "fold/signal", "signal-width", "sample-at",
    "trim-trace", "count", "timeframe",
    "all-scopes", "scoped", "resolve-scope", "set-scope", "unset-scope",
    "groups", "in-group", "in-groups", "resolve-group",
    "in-scope", "in-scopes",
    "array", "seta", "geta", "geta/default", "dela", "mapa",
    "defsig", "new-trace", "dump-trace",
    "defun",
    "car", "cdr", "cadr", "caar", "cddr",
    "inc", "dec",
    "dowhile", "until",
    "rising", "falling",
    "range",
    "SIGNALS", "INDEX", "MAX-INDEX", "CS", "CG",
    "LOCAL-SIGNALS", "LOCAL-SCOPES", "SCOPES", "VIRTUAL-SIGNALS",
    "TRACE-FILE", "TRACE-NAME", "TS",
    "true", "false", "nil",
];

pub struct UnknownSymbolRule;

impl Rule for UnknownSymbolRule {
    fn descriptor(&self) -> &RuleDescriptor {
        &RuleDescriptor {
            id: "unknown-symbol",
            name: "Unknown function or operator",
            description: "Warns when calling an undefined or unknown function at top level",
            default_enabled: true,
            default_severity: LintSeverity::Warning,
        }
    }

    fn check(&self, node: Node, ctx: &LintContext) -> Vec<Diagnostic> {
        if node.kind() != "list" { return vec![]; }
        let (fn_name, bracket) = match get_form_name(node, ctx.source) {
            Some(v) => v,
            None => return vec![],
        };
        if bracket != "(" { return vec![]; }

        // 只检查顶层
        let is_top_level = node.parent().map_or(false, |p| {
            p.kind() == "sexpr" && p.parent().map_or(false, |gp| gp.kind() == "program")
        });
        if !is_top_level { return vec![]; }

        // 跳过数字字面量
        let is_number = fn_name.parse::<i64>().is_ok() || fn_name.parse::<f64>().is_ok();
        if is_number { return vec![]; }

        // 跳过特殊前缀
        if fn_name.starts_with('\'') || fn_name.starts_with('`') || fn_name.starts_with(',') { return vec![]; }

        // 检查是否已知或用户定义
        let known: HashSet<&str> = HashSet::from_iter(KNOWN_SYMBOLS.iter().copied());
        if known.contains(fn_name.as_str()) { return vec![]; }
        if ctx.user_symbols.contains(&fn_name) { return vec![]; }

        let pos = node.start_position();
        let range = ctx.range_from(pos, fn_name.len());
        vec![Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::WARNING),
            message: format!("Unknown function or operator: '{}'", fn_name),
            ..Default::default()
        }]
    }
}
