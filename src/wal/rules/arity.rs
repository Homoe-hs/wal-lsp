use lsp_types::{Diagnostic, DiagnosticSeverity};
use tree_sitter::Node;
use crate::wal::rules::{LintContext, LintSeverity, Rule, RuleDescriptor, get_form_name, get_args};

/// 已知符号 arity 表: (name, expected_args)
const KNOWN_ARITIES: &[(&str, usize)] = &[
    ("define", 2), ("set", 2), ("set!", 2),
    ("if", 3), ("fn", 2), ("defmacro", 3),
    ("/", 2), ("**", 2), ("mod", 2),
    ("floor", 1), ("ceil", 1), ("round", 1), ("abs", 1),
    ("quote", 1), ("quasiquote", 1), ("unquote", 1),
    ("eval", 1), ("parse", 1),
    ("first", 1), ("second", 1), ("last", 1), ("rest", 1),
    ("length", 1), ("average", 1), ("sum", 1),
    ("map", 2), ("fold", 3), ("zip", 2),
    ("max", 1), ("min", 1), ("in", 2),
    ("seta", 3), ("geta", 2), ("dela", 2), ("mapa", 2),
    ("geta/default", 3),
    ("not", 1), ("null?", 1), ("signal?", 1),
    ("atom?", 1), ("symbol?", 1), ("string?", 1), ("int?", 1), ("list?", 1),
    ("convert/bin", 2),
    ("exit", 1), ("import", 1), ("require", 1), ("eval-file", 1),
    ("load", 1), ("unload", 1),
    ("get", 1), ("slice", 3), ("reval", 2),
    ("step", 1), ("alias", 2), ("unalias", 1),
    ("find", 1), ("count", 1),
    ("in-groups", 2), ("in-scope", 2), ("in-scopes", 2),
    ("resolve-group", 1), ("all-scopes", 0),
    ("!", 1), (">", 2), ("<", 2), (">=", 2), ("<=", 2),
];

pub struct ArityRule;

impl ArityRule {
    fn desc_ref() -> &'static RuleDescriptor {
        &RuleDescriptor {
            id: "arity-check",
            name: "Check function arity",
            description: "Verifies that known functions receive the correct number of arguments",
            default_enabled: true,
            default_severity: LintSeverity::Error,
        }
    }
}

impl Rule for ArityRule {
    fn descriptor(&self) -> &RuleDescriptor { Self::desc_ref() }

    fn check(&self, node: Node, ctx: &LintContext) -> Vec<Diagnostic> {
        if node.kind() != "list" { return vec![]; }
        let (fn_name, bracket) = match get_form_name(node, ctx.source) {
            Some(v) => v,
            None => return vec![],
        };
        // 只检查 () 括号形式
        if bracket != "(" { return vec![]; }
        let args = get_args(node);
        let arg_count = args.len();
        for &(name, expected) in KNOWN_ARITIES {
            if name == fn_name.as_str() && arg_count != expected {
                let pos = node.start_position();
                let range = ctx.range_from(pos, fn_name.len());
                return vec![Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("'{}' expects {} argument(s), got {}", name, expected, arg_count),
                    ..Default::default()
                }];
            }
        }
        vec![]
    }
}
