use crate::lsp::WORKSPACE;
use crate::wal::parser::WalParser;
use anyhow::Result;
use lsp_server::{Connection, Notification};
use lsp_types::{Diagnostic, DiagnosticSeverity, PublishDiagnosticsParams, Position, Range};
use once_cell::sync::Lazy;
use std::collections::HashSet;
use tracing::info;

/// Set of all known WAL operators, special forms, builtins, and macros
static KNOWN_SYMBOLS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut set = HashSet::new();

    // From wal-rust operator.rs — all 82 implemented operators
    // Math
    set.insert("+"); set.insert("-"); set.insert("*"); set.insert("/"); set.insert("**");
    set.insert("floor"); set.insert("ceil"); set.insert("round"); set.insert("mod"); set.insert("sum");
    // Logical
    set.insert("!"); set.insert("not"); set.insert("="); set.insert("!=");
    set.insert(">"); set.insert("<"); set.insert(">="); set.insert("<=");
    set.insert("&&"); set.insert("||"); set.insert("and"); set.insert("or");
    // Bitwise
    set.insert("bor"); set.insert("band"); set.insert("bxor");
    // Control flow
    set.insert("print"); set.insert("printf"); set.insert("set"); set.insert("set!");
    set.insert("define"); set.insert("let"); set.insert("if"); set.insert("case");
    set.insert("when"); set.insert("unless"); set.insert("cond");
    set.insert("while"); set.insert("do"); set.insert("exit");
    set.insert("fn"); set.insert("defmacro"); set.insert("macroexpand"); set.insert("gensym");
    set.insert("type"); set.insert("alias"); set.insert("unalias");
    // Special forms
    set.insert("quote"); set.insert("quasiquote"); set.insert("unquote");
    set.insert("eval"); set.insert("parse"); set.insert("rel_eval");
    set.insert("slice"); set.insert("get"); set.insert("call"); set.insert("import");
    // List
    set.insert("list"); set.insert("first"); set.insert("second"); set.insert("last");
    set.insert("rest"); set.insert("in"); set.insert("map");
    set.insert("max"); set.insert("min"); set.insert("fold");
    set.insert("length"); set.insert("average"); set.insert("zip");
    // Type checks
    set.insert("defined?"); set.insert("atom?"); set.insert("symbol?");
    set.insert("string?"); set.insert("int?"); set.insert("list?");
    set.insert("convert/bin");
    set.insert("string->int"); set.insert("bits->sint");
    set.insert("symbol->string"); set.insert("string->symbol"); set.insert("int->string");
    // Signal operations
    set.insert("load"); set.insert("unload"); set.insert("step"); set.insert("eval-file");
    set.insert("require"); set.insert("repl"); set.insert("loaded-traces");
    set.insert("signal?"); set.insert("signals"); set.insert("index"); set.insert("max-index");
    set.insert("ts"); set.insert("trace-name"); set.insert("trace-file");
    set.insert("find"); set.insert("find/g"); set.insert("whenever");
    set.insert("fold/signal"); set.insert("signal-width"); set.insert("sample-at");
    set.insert("trim-trace"); set.insert("count"); set.insert("timeframe");
    // Scope/group
    set.insert("all-scopes"); set.insert("scoped"); set.insert("resolve-scope");
    set.insert("set-scope"); set.insert("unset-scope"); set.insert("groups");
    set.insert("in-group"); set.insert("in-groups"); set.insert("resolve-group");
    set.insert("in-scope"); set.insert("in-scopes");
    // Array
    set.insert("array"); set.insert("seta"); set.insert("geta");
    set.insert("geta/default"); set.insert("dela"); set.insert("mapa");
    // Virtual
    set.insert("defsig"); set.insert("new-trace"); set.insert("dump-trace");
    // Waveform
    set.insert("step"); set.insert("step-until"); set.insert("always");
    // Macros
    set.insert("defun"); set.insert("defunm"); set.insert("for/list");
    set.insert("car"); set.insert("cdr"); set.insert("cadr"); set.insert("caar"); set.insert("cddr");
    set.insert("inc"); set.insert("dec"); set.insert("inc-define");
    set.insert("dowhile"); set.insert("until");
    // Special variables (already in completions, but for completeness)
    set.insert("SIGNALS"); set.insert("INDEX"); set.insert("MAX-INDEX"); set.insert("CS");
    set.insert("LOCAL-SIGNALS"); set.insert("VIRTUAL-SIGNALS");
    set.insert("TRACE-FILE"); set.insert("TRACE-NAME"); set.insert("TS");
    set.insert("SCOPES"); set.insert("LOCAL-SCOPES"); set.insert("CG");
    // String ops
    set.insert("concat"); set.insert("strlen");
    // Partition/filter
    set.insert("partition"); set.insert("filter"); set.insert("sort"); set.insert("reverse");
    set.insert("append");
    // Others
    set.insert("signal?"); set.insert("abs"); set.insert("signed"); set.insert("reval");
    set.insert("null?"); set.insert("list?");
    // Boolean literals (self-evaluating)
    set.insert("true"); set.insert("false"); set.insert("nil");

    set
});

/// Known symbol arities: (name, arg_count)
static KNOWN_ARITIES: Lazy<Vec<(&'static str, usize)>> = Lazy::new(|| {
    vec![
        ("define", 2), ("set", 2), ("set!", 2),
        ("if", 3), ("fn", 2),
        ("defmacro", 3),
        ("/", 2), ("**", 2), ("mod", 2),
        ("quote", 1), ("quasiquote", 1), ("unquote", 1),
        ("eval", 1), ("parse", 1),
        ("first", 1), ("second", 1), ("last", 1), ("rest", 1),
        ("length", 1), ("average", 1), ("sum", 1),
        ("floor", 1), ("ceil", 1), ("round", 1),
        ("not", 1),
        ("exit", 0),
        ("import", 1), ("require", 1), ("eval-file", 1), ("load", 1),
        ("get", 1),
    ]
});

pub fn handle_did_open(connection: &Connection, notif: Notification) -> Result<()> {
    let params = notif
        .extract::<lsp_types::DidOpenTextDocumentParams>("textDocument/didOpen")
        .map_err(|e| anyhow::anyhow!("Failed to extract params: {:?}", e))?;

    let uri = params.text_document.uri.clone();
    let text = params.text_document.text.clone();

    info!("Document opened: {:?}", uri);

    {
        let mut ws = WORKSPACE.write().unwrap_or_else(|e| e.into_inner());
        ws.open_document(uri.clone(), text.clone());
    }

    let diagnostics = analyze_document(&text);

    let params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version: None,
    };

    let notification = Notification::new("textDocument/publishDiagnostics".to_string(), params);
    connection
        .sender
        .send(lsp_server::Message::Notification(notification))?;

    Ok(())
}

pub fn handle_did_change(connection: &Connection, notif: Notification) -> Result<()> {
    let params = notif
        .extract::<lsp_types::DidChangeTextDocumentParams>("textDocument/didChange")
        .map_err(|e| anyhow::anyhow!("Failed to extract params: {:?}", e))?;

    let uri = params.text_document.uri.clone();
    let text = params
        .content_changes
        .get(0)
        .map(|c| c.text.clone())
        .unwrap_or_default();

    info!("Document changed: {:?}", uri);

    {
        let mut ws = WORKSPACE.write().unwrap_or_else(|e| e.into_inner());
        ws.update_document(&uri, text.clone());
    }

    let diagnostics = analyze_document(&text);

    let params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version: None,
    };

    let notification = Notification::new("textDocument/publishDiagnostics".to_string(), params);
    connection
        .sender
        .send(lsp_server::Message::Notification(notification))?;

    Ok(())
}

fn analyze_document(text: &str) -> Vec<Diagnostic> {
    let mut parser = WalParser::new();
    let tree = parser.parse_with_errors(text);

    let mut diagnostics = Vec::new();

    // 1. Syntax errors (from tree-sitter ERROR nodes)
    if tree.root_node().has_error() {
        let mut cursor = tree.walk();
        collect_syntax_errors(&mut cursor, text, &mut diagnostics);
    }

    // 2. Semantic errors (unknown functions, wrong arity)
    collect_semantic_errors(tree.root_node(), text, &mut diagnostics);

    diagnostics
}

fn collect_syntax_errors(
    cursor: &mut tree_sitter::TreeCursor,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let node = cursor.node();

    if node.kind() == "ERROR" {
        let start = node.start_position();
        let end = node.end_position();

        let range = Range::new(
            Position::new(start.row as u32, start.column as u32),
            Position::new(end.row as u32, end.column as u32),
        );

        let error_node_text = source.get(node.byte_range()).unwrap_or("");
        // Truncate long error messages
        let truncated = if error_node_text.len() > 80 {
            format!("{}...", &error_node_text[..77])
        } else {
            error_node_text.to_string()
        };

        diagnostics.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            message: format!("Syntax error: {}", truncated),
            ..Default::default()
        });
    }

    if cursor.goto_first_child() {
        loop {
            collect_syntax_errors(cursor, source, diagnostics);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}

fn collect_semantic_errors(
    node: tree_sitter::Node,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let kind = node.kind();

    // Only process list nodes at the top level (direct children of program/sexpr_list)
    // Skip nested lists inside function bodies and bindings
    if kind == "list" {
        // Check if this is a top-level expression (parent is sexpr, grandparent is sexpr_list or program)
        let is_toplevel = node.parent().map_or(false, |p| {
            p.kind() == "sexpr" && p.parent().map_or(false, |gp| {
                gp.kind() == "program"
            })
        });

        if is_toplevel {
            let mut cursor = node.walk();
            let children: Vec<_> = node.children(&mut cursor).collect();

            let sexpr_list = children.iter().find(|c| c.kind() == "sexpr_list");
            let sub_sexprs: Vec<tree_sitter::Node> = sexpr_list
                .map(|sl| {
                    let mut c = sl.walk();
                    sl.children(&mut c).collect()
                })
                .unwrap_or_default();

            if !sub_sexprs.is_empty() {
                let fn_sexpr = sub_sexprs[0];
                let fn_pos = fn_sexpr.start_position();
                let fn_info: Option<(String, tree_sitter::Point)> = {
                    let atoms: Vec<tree_sitter::Node> = {
                        let mut fc = fn_sexpr.walk();
                        fn_sexpr.children(&mut fc).collect()
                    };
                    atoms.iter()
                        .find(|a| a.kind() == "atom")
                        .and_then(|a| source.get(a.byte_range()).map(|s| (s.trim().to_string(), a.start_position())))
                };

                let arg_count = sub_sexprs.iter()
                    .filter(|c| c.kind() == "sexpr")
                    .count()
                    .saturating_sub(1);

                if let Some((ref fn_name, pos)) = fn_info {
                    if !KNOWN_SYMBOLS.contains(fn_name.as_str()) {
                        let range = Range::new(
                            Position::new(pos.row as u32, pos.column as u32),
                            Position::new(pos.row as u32, (pos.column + fn_name.len()) as u32),
                        );
                        if !fn_name.starts_with('\'') && !fn_name.starts_with('`')
                            && !fn_name.starts_with('#') && !fn_name.starts_with('~')
                            && !fn_name.starts_with(";;")
                        {
                            diagnostics.push(Diagnostic {
                                range,
                                severity: Some(DiagnosticSeverity::WARNING),
                                message: format!("Unknown function or operator: '{}'", fn_name),
                                ..Default::default()
                            });
                        }
                    }

                    for &(name, expected_arity) in KNOWN_ARITIES.iter() {
                        if name == fn_name.as_str() && arg_count != expected_arity {
                            let range = Range::new(
                                Position::new(fn_pos.row as u32, fn_pos.column as u32),
                                Position::new(fn_pos.row as u32, (fn_pos.column + fn_name.len()) as u32),
                            );
                            diagnostics.push(Diagnostic {
                                range,
                                severity: Some(DiagnosticSeverity::ERROR),
                                message: format!(
                                    "'{}' expects {} argument(s), got {}",
                                    fn_name, expected_arity, arg_count
                                ),
                                ..Default::default()
                            });
                        }
                    }
                }
            }
        }
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_semantic_errors(child, source, diagnostics);
    }
}
