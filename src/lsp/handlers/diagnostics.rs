use crate::lsp::WORKSPACE;
use crate::wal::parser::WalParser;
use crate::wal::symbols::extract_symbols;
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
        // State
        ("define", 2), ("set", 2), ("set!", 2),
        // Control flow
        ("if", 3),
        ("fn", 2),
        ("defmacro", 3),
        // Math
        ("/", 2), ("**", 2), ("mod", 2),
        ("floor", 1), ("ceil", 1), ("round", 1),
        // Quote
        ("quote", 1), ("quasiquote", 1), ("unquote", 1),
        ("eval", 1), ("parse", 1),
        // List — accessors
        ("first", 1), ("second", 1), ("last", 1), ("rest", 1),
        ("length", 1), ("average", 1), ("sum", 1),
        // List — transform
        ("map", 2), ("fold", 3), ("zip", 2),
        ("max", 1), ("min", 1),
        ("in", 2),
        // Array
        ("seta", 3), ("geta", 2), ("dela", 2), ("mapa", 2),
        ("geta/default", 3),
        // Type
        ("not", 1),
        ("atom?", 1), ("symbol?", 1), ("string?", 1), ("int?", 1), ("list?", 1),
        ("convert/bin", 2),
        // IO
        ("exit", 1),
        ("import", 1), ("require", 1), ("eval-file", 1),
        ("load", 1), ("unload", 1),
        // Signal
        ("get", 1), ("slice", 3), ("reval", 2),
        // Waveform
        ("step", 1), ("alias", 2), ("unalias", 1),
        ("find", 1), ("count", 1),
        // Scope/group
        ("in-groups", 2), ("in-scope", 2), ("in-scopes", 2),
        ("resolve-group", 1), ("all-scopes", 0),
        // Comparison
        ("!", 1), (">", 2), ("<", 2), (">=", 2), ("<=", 2),
        // List aggregations
        ("max", 1), ("min", 1),
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

    // 0. Extract user-defined symbols (define/defun/defsig/defmacro) for skip-list
    let user_symbols: HashSet<String> = extract_symbols(text)
        .into_iter()
        .map(|s| s.name)
        .collect();

    // 1. Syntax errors (from tree-sitter ERROR nodes)
    if tree.root_node().has_error() {
        let mut cursor = tree.walk();
        collect_syntax_errors(&mut cursor, text, &mut diagnostics);
    }

    // 2. Semantic errors (unknown functions, wrong arity)
    collect_semantic_errors(tree.root_node(), text, &user_symbols, &mut diagnostics);

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

        let error_node_text = source.get(node.byte_range()).unwrap_or("");
        // Skip ERROR nodes that are just comments swallowed by error recovery
        if !error_node_text.trim_start().starts_with(";;") {
            let range = Range::new(
                Position::new(start.row as u32, start.column as u32),
                Position::new(end.row as u32, end.column as u32),
            );
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
    user_symbols: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let kind = node.kind();

    if kind == "list" {
        validate_list_node(node, source, user_symbols, diagnostics);
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_semantic_errors(child, source, user_symbols, diagnostics);
    }
}

/// Get the first atom text from a list node, handling bracket type
fn get_form_info(node: tree_sitter::Node, source: &str) -> Option<(String, String, tree_sitter::Point)> {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();
    
    // Determine bracket type: (, [, or {
    let bracket = children.first().map(|c| {
        source.get(c.byte_range()).map(|s| s.trim().to_string()).unwrap_or_default()
    }).unwrap_or_default();
    
    let sexpr_list = children.iter().find(|c| c.kind() == "sexpr_list")?;
    let sl_children: Vec<tree_sitter::Node> = {
        let mut c = sexpr_list.walk();
        sexpr_list.children(&mut c)
            .filter(|child| child.kind() == "sexpr")
            .collect()
    };
    
    if sl_children.is_empty() {
        return None;
    }
    
    let fn_sexpr = sl_children[0];
    let fn_pos = fn_sexpr.start_position();
    let fn_text = {
        let mut fc = fn_sexpr.walk();
        let atom_text = fn_sexpr.children(&mut fc)
            .find(|a| a.kind() == "atom")
            .and_then(|a| source.get(a.byte_range()).map(|s| s.trim().to_string()));
        atom_text
    }?;
    
    Some((fn_text, bracket, fn_pos))
}

/// Extract the raw sub-sexprs from a list (for arity counting etc.)
fn get_sub_sexprs(node: tree_sitter::Node) -> Vec<tree_sitter::Node> {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();
    let sexpr_list = match children.iter().find(|c| c.kind() == "sexpr_list") {
        Some(sl) => sl,
        None => return vec![],
    };
    let mut c = sexpr_list.walk();
    sexpr_list.children(&mut c)
        .filter(|child| child.kind() == "sexpr")
        .collect()
}

/// Check if a list node uses ( ) brackets (function call syntax) vs [ ] or { }
// (removed unused is_paren_list)

fn validate_list_node(node: tree_sitter::Node, source: &str, user_symbols: &HashSet<String>, diagnostics: &mut Vec<Diagnostic>) {
    let form_info = match get_form_info(node, source) {
        Some(info) => info,
        None => return,
    };
    let (fn_name, bracket, fn_pos) = form_info;
    let is_top_level = node.parent().map_or(false, |p| {
        p.kind() == "sexpr" && p.parent().map_or(false, |gp| gp.kind() == "program")
    });
    let is_fn_call_form = bracket == "(";
    
    let sub_sexprs = get_sub_sexprs(node);
    let arg_count = sub_sexprs.len().saturating_sub(1);

    // ---- Structural validation for known forms (all levels) ----
    validate_form_structure(&fn_name, &sub_sexprs, &fn_pos, node, source, diagnostics);

    // ---- Known-symbol check (top-level only, to avoid false positives) ----
    if is_top_level && is_fn_call_form {
        if !KNOWN_SYMBOLS.contains(fn_name.as_str()) && !user_symbols.contains(fn_name.as_str()) {
            let range = range_from_point(fn_pos, fn_name.len());
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
    }

    // ---- Arity check (top-level + nested, for ( ) lists only) ----
    if is_fn_call_form {
        for &(name, expected_arity) in KNOWN_ARITIES.iter() {
            if name == fn_name.as_str() && arg_count != expected_arity {
                let range = range_from_point(fn_pos, fn_name.len());
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

fn range_from_point(pos: tree_sitter::Point, len: usize) -> Range {
    Range::new(
        Position::new(pos.row as u32, pos.column as u32),
        Position::new(pos.row as u32, (pos.column + len) as u32),
    )
}

/// Validate the structure of known special forms
fn validate_form_structure(
    fn_name: &str,
    sub_sexprs: &[tree_sitter::Node],
    fn_pos: &tree_sitter::Point,
    _node: tree_sitter::Node,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match fn_name {
        "let" => validate_let_form(sub_sexprs, fn_pos, source, diagnostics),
        "case" => validate_case_form(sub_sexprs, fn_pos, diagnostics),
        "defun" | "fn" => validate_fn_params(sub_sexprs, fn_name, source, diagnostics),
        _ => {}
    }
}

/// Validate let bindings: each binding is [id expr] where id must be an atom
fn validate_let_form(
    sub_sexprs: &[tree_sitter::Node],
    fn_pos: &tree_sitter::Point,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if sub_sexprs.len() < 2 {
        let range = range_from_point(*fn_pos, "let".len());
        diagnostics.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            message: "let expects at least one binding pair and a body".to_string(),
            ..Default::default()
        });
        return;
    }
    // First sub-sexpr after "let" should be the binding list [...]
    let binding_node = sub_sexprs[1];
    let bindings = get_bracket_contents(binding_node);
    
    for (i, binding) in bindings.iter().enumerate() {
        // Each binding [id expr] must have at least 2 elements
        if binding.len() < 2 {
            let pos = binding.first().map(|n| n.start_position()).unwrap_or(*fn_pos);
            let range = range_from_point(pos, 1);
            diagnostics.push(Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                message: format!("let binding #{} is missing a value", i + 1),
                ..Default::default()
            });
            continue;
        }
        // First element must be a valid identifier (atom)
        if !is_atom_like(binding[0], source) {
            let pos = binding[0].start_position();
            let text = source.get(binding[0].byte_range()).unwrap_or("?").trim().to_string();
            let range = range_from_point(pos, text.len());
            diagnostics.push(Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                message: format!("let binding #{} id must be a symbol, got '{}'", i + 1, text),
                ..Default::default()
            });
        }
    }
}

/// Validate case clauses: each is [value expr+]  
fn validate_case_form(
    sub_sexprs: &[tree_sitter::Node],
    fn_pos: &tree_sitter::Point,
    diagnostics: &mut Vec<Diagnostic>,
) {
    // Clauses start from sub_sexprs[1] onward (after "case" and the key)
    if sub_sexprs.len() < 3 {
        let range = range_from_point(*fn_pos, "case".len());
        diagnostics.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            message: "case expects a key and at least one clause".to_string(),
            ..Default::default()
        });
        return;
    }
    for (i, clause) in sub_sexprs.iter().enumerate().skip(2) {
        let contents = get_bracket_contents(*clause);
        if contents.is_empty() {
            let pos = clause.start_position();
            let range = range_from_point(pos, 1);
            diagnostics.push(Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                message: format!("case clause #{} is empty", i - 1),
                ..Default::default()
            });
        }
    }
}

/// Validate fn/defun parameter list: all elements must be atoms
fn validate_fn_params(
    sub_sexprs: &[tree_sitter::Node],
    fn_name: &str,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if sub_sexprs.len() < 2 {
        return; // arity check handles this
    }
    let params_node = sub_sexprs[1];
    let params = get_bracket_contents(params_node);
    
    for (i, param_group) in params.iter().enumerate() {
        // Each param is the first element of its group
        let param = match param_group.first() {
            Some(p) => *p,
            None => continue,
        };
        if !is_atom_like(param, source) {
            let pos = param.start_position();
            let text = source.get(param.byte_range()).unwrap_or("?").trim().to_string();
            let range = range_from_point(pos, text.len());
            diagnostics.push(Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                message: format!("{} parameter #{} must be a symbol, got '{}'", fn_name, i + 1, text),
                ..Default::default()
            });
        }
    }
}

/// Extract the contents of a [...] bracketed list as groups of sub-sexprs
/// For [a b c]: returns [[a], [b], [c]] (each atom is a separate group)
/// For [[a 1] [b 2]]: returns [[a, 1], [b, 2]] (each inner pair is a group)
/// Accepts either a list node or a sexpr wrapping a list node.
fn get_bracket_contents<'a>(node: tree_sitter::Node<'a>) -> Vec<Vec<tree_sitter::Node<'a>>> {
    // Unwrap sexpr → list if needed
    let list_node = if node.kind() == "sexpr" {
        let mut c = node.walk();
        let result = node.children(&mut c).find(|ch| ch.kind() == "list");
        result
    } else if node.kind() == "list" {
        Some(node)
    } else {
        None
    };
    let list_node = match list_node {
        Some(n) => n,
        None => return vec![],
    };
    
    let inner = get_sub_sexprs(list_node);
    
    // Check if contents are nested brackets (e.g., [[a 1] [b 2]])
    // Only check the first element — if it's a bracketed list, we have nested bindings
    let has_nested = inner.first().map_or(false, |s| {
        let c = &mut s.walk();
        let result = s.children(c).any(|ch| ch.kind() == "list");
        result
    });
    
    if has_nested {
        // Each outer sexpr wraps a [...] list; unwrap and extract its contents
        inner.iter().map(|s| {
            let mut c = s.walk();
            let list = s.children(&mut c).find(|ch| ch.kind() == "list");
            match list {
                Some(l) => get_sub_sexprs(l),
                None => vec![],
            }
        }).collect()
    } else {
        // Flat list; treat the whole thing as one group
        vec![inner]
    }
}

/// Check if a node represents an atom-like value (symbol, number, string, bool, operator)
fn is_atom_like(node: tree_sitter::Node, source: &str) -> bool {
    let mut c = node.walk();
    node.children(&mut c).any(|ch| {
        matches!(ch.kind(), "atom" | "symbol" | "base_symbol" | "scoped_symbol" 
            | "grouped_symbol" | "operator" | "int" | "float" | "string" | "bool")
    }) || source.get(node.byte_range()).map_or(false, |s| {
        let t = s.trim();
        !t.starts_with('(') && !t.starts_with('[') && !t.starts_with('{')
    })
}
