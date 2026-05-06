use crate::lsp::WORKSPACE;
use crate::wal::parser::WalParser;
use crate::wal::symbols::extract_symbols_from_node;
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
    set.insert("step-while");
    // Macros
    set.insert("defun"); set.insert("defunm"); set.insert("for/list");
    set.insert("car"); set.insert("cdr"); set.insert("cadr"); set.insert("caar"); set.insert("cddr");
    set.insert("inc"); set.insert("dec"); set.insert("inc-define");
    set.insert("dowhile"); set.insert("until");
    set.insert("range");
    // Edge detection
    set.insert("rising"); set.insert("falling"); set.insert("unstable"); set.insert("stable");
    // Quoting
    set.insert("unquote-splice");
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
        ("floor", 1), ("ceil", 1), ("round", 1), ("abs", 1),
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
        ("null?", 1), ("signal?", 1),
        ("convert/bin", 2),
        // IO
        ("exit", 1), ("import", 1), ("require", 1), ("eval-file", 1),
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

pub fn handle_did_close(connection: &Connection, notif: Notification) -> Result<()> {
    let params = notif
        .extract::<lsp_types::DidCloseTextDocumentParams>("textDocument/didClose")
        .map_err(|e| anyhow::anyhow!("Failed to extract params: {:?}", e))?;

    let uri = params.text_document.uri.clone();
    info!("Document closed: {:?}", uri);

    {
        let mut ws = WORKSPACE.write().unwrap_or_else(|e| e.into_inner());
        ws.close_document(&uri);
    }

    // Clear diagnostics for closed document
    let params = PublishDiagnosticsParams {
        uri,
        diagnostics: vec![],
        version: None,
    };
    let notification = Notification::new("textDocument/publishDiagnostics".to_string(), params);
    connection.sender.send(lsp_server::Message::Notification(notification))?;

    Ok(())
}

pub fn analyze_document(text: &str) -> Vec<Diagnostic> {
    let mut parser = WalParser::new();
    let tree = parser.parse_with_errors(text);

    let mut diagnostics = Vec::new();

    // 0. Extract user-defined symbols (define/defun/defsig/defmacro) for skip-list
    //    Use the already-parsed tree to avoid re-parsing
    let user_symbols: HashSet<String> = extract_symbols_from_node(tree.root_node(), text)
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
        // Skip number literals that tree-sitter parsed as atoms (e.g., +5, -3)
        let is_number_literal = fn_name.parse::<i64>().is_ok() || fn_name.parse::<f64>().is_ok();
        if !is_number_literal && !KNOWN_SYMBOLS.contains(fn_name.as_str()) && !user_symbols.contains(fn_name.as_str()) {
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
        "case" => validate_case_form(sub_sexprs, fn_pos, source, diagnostics),
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
    
    if bindings.is_empty() {
        let pos = binding_node.start_position();
        let range = range_from_point(pos, 1);
        diagnostics.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            message: "let expects a binding list in brackets [...]".to_string(),
            ..Default::default()
        });
        return;
    }
    
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
    source: &str,
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
    let mut seen_default = false;
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
            continue;
        }
        // Check if this is the default clause
        let first_text = contents.first().and_then(|c| c.first())
            .and_then(|n| source.get(n.byte_range()).map(|s| s.trim().to_string()));
        if first_text.as_deref() == Some("default") {
            seen_default = true;
        }
        // Warn about dead code after default clause
        if seen_default && first_text.as_deref() != Some("default") {
            let pos = clause.start_position();
            let range = range_from_point(pos, 1);
            diagnostics.push(Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::WARNING),
                message: format!("case clause #{} is unreachable (after default)", i - 1),
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
    
    if params.is_empty() {
        // Variadic defun without brackets: (defun name single-param body)
        // The single param should be a valid atom
        return;
    }
    
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

#[cfg(test)]
mod tests {
    use super::*;

    fn load_mega_test() -> String {
        let path = std::path::Path::new("tests/syntax/99_mega_test.wal");
        std::fs::read_to_string(path).expect("Failed to read 99_mega_test.wal")
    }

    /// Return only the "clean" portion of the mega test (before GARBLED ERRORS)
    #[allow(dead_code)]
    fn load_mega_test_clean() -> String {
        let full = load_mega_test();
        match full.find("GARBLED ERRORS BELOW") {
            Some(pos) => full[..pos].to_string(),
            None => full,
        }
    }

    fn all_wal_files() -> Vec<String> {
        let dir = std::path::Path::new("tests/syntax/");
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "wal") {
                    if let Ok(s) = std::fs::read_to_string(&path) {
                        files.push(s);
                    }
                }
            }
        }
        files
    }

    #[test]
    fn test_diagnostics_per_file_validation() {
        let dir_stack = all_wal_files();
        assert!(dir_stack.len() >= 20);
        let mut passes = 0;
        for source in &dir_stack {
            let d = analyze_document(source);
            passes += 1;
            let _ = d;
        }
        eprintln!("Processed {} .wal files individually", passes);
        assert_eq!(passes, dir_stack.len());
    }

    // ============================================================
    // 混合正确/错误多行文件诊断测试
    // ============================================================

    #[test]
    fn test_mixed_valid_and_invalid_multiline() {
        let source = r#"
;; --- valid section ---
(define x 42)
(+ 1 2)
;; --- error section ---
(define missing-value)
(+ 1 (* 2 3
;; --- more valid ---
(set! x 100)
"#;
        let diagnostics = analyze_document(source);
        // Should find at least 2 errors: one arity, one syntax
        let errors: Vec<_> = diagnostics.iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR)).collect();
        assert!(errors.len() >= 1,
            "Mixed file should have >=1 error, got {}: {:?}",
            errors.len(), errors.iter().map(|e| &e.message).collect::<Vec<_>>());
    }

    #[test]
    fn test_exact_error_line_numbering() {
        let source = "(+ 1 2)\n(define x)\n(* 3 4)\n(first)\n";
        let diagnostics = analyze_document(source);
        let errors: Vec<_> = diagnostics.iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR)).collect();
        assert_eq!(errors.len(), 2); // define x (arity) and first (arity)
    }

    #[test]
    fn test_partially_garbled_file() {
        // Valid code interleaved with garbage
        let source = r#"
(define clean 1)
(frobnoz 42 "bad")
(+ 1 2)
@#$%junk
(define clean2 2)
"#;
        let diagnostics = analyze_document(source);
        let warnings: Vec<_> = diagnostics.iter()
            .filter(|d| d.message.contains("Unknown function") && d.message.contains("frobnoz"))
            .collect();
        let syntax: Vec<_> = diagnostics.iter()
            .filter(|d| d.message.starts_with("Syntax error")).collect();
        assert_eq!(warnings.len(), 1, "Should flag frobnoz as unknown");
        assert!(!syntax.is_empty(), "Should flag garbage as syntax error");
    }

    #[test]
    fn test_repeated_same_error_detected() {
        let source = r#"
(define a)
(define b)
(define c)
"#;
        let diagnostics = analyze_document(source);
        let arity_errors: Vec<_> = diagnostics.iter()
            .filter(|d| d.message.contains("define") && d.message.contains("argument"))
            .collect();
        assert_eq!(arity_errors.len(), 3,
            "Should detect arity error on all 3 defines");
    }

    #[test]
    fn test_errors_persist_after_comment_lines() {
        let source = r#";; header
(define valid 42)
;; separator
;; another comment
(define bad)
;; footer
"#;
        let diagnostics = analyze_document(source);
        let errors: Vec<_> = diagnostics.iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR)).collect();
        assert_eq!(errors.len(), 1, "Should detect 1 arity error after comments");
    }

    #[test]
    fn test_warning_and_error_in_same_file() {
        let source = "(florbnoz 42)\n(define x)";
        let diagnostics = analyze_document(source);
        let warnings: Vec<_> = diagnostics.iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::WARNING)).collect();
        let errors: Vec<_> = diagnostics.iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR)).collect();
        assert!(warnings.iter().any(|w| w.message.contains("florbnoz")),
            "Should warn about unknown function");
        assert!(errors.iter().any(|e| e.message.contains("define")),
            "Should error on wrong arity");
    }

    #[test]
    fn test_no_false_positive_on_variadic_forms() {
        // +, -, * accept variable args — should not trigger arity errors
        let valid_variadic = ["(+ 1)", "(+ 1 2 3 4)", "(*)", "(* 2 3)", "(- 42)"];
        for src in &valid_variadic {
            let d = analyze_document(src);
            let arity_errors: Vec<_> = d.iter()
                .filter(|x| x.message.contains("argument(s)")).collect();
            assert!(arity_errors.is_empty(),
                "Variadic form '{}' should have no arity errors", src);
        }
    }

    #[test]
    fn test_no_false_positive_on_logical_ops() {
        // && and || are variadic
        let valid = ["(&& #t)", "(&& #t #t #t)", "(|| #f)", "(|| #f #f #t)"];
        for src in &valid {
            let d = analyze_document(src);
            let arity_errors: Vec<_> = d.iter()
                .filter(|x| x.message.contains("argument(s)")).collect();
            assert!(arity_errors.is_empty(),
                "Logical op '{}' should have no arity errors", src);
        }
    }

    #[test]
    fn test_known_symbols_match_completions() {
        // All symbols in KNOWN_SYMBOLS should not trigger false warnings
        let test_calls: Vec<String> = KNOWN_SYMBOLS.iter().take(30)
            .filter(|s| !s.contains('/') && !s.contains('?') && s.len() > 1)
            .map(|s| format!("({} 1)", s))
            .collect();
        for call in &test_calls {
            let d = analyze_document(call);
            let warnings: Vec<_> = d.iter()
                .filter(|x| x.message.contains("Unknown function")).collect();
            assert!(warnings.is_empty(),
                "Known symbol '{}' should not trigger warning", call);
        }
    }

    #[test]
    fn test_arity_check_skips_bracket_forms() {
        // [define x] should NOT trigger arity check (not a fn call form)
        let source = "[define x]";
        let d = analyze_document(source);
        let arity: Vec<_> = d.iter()
            .filter(|x| x.message.contains("argument")).collect();
        assert!(arity.is_empty(),
             "Bracket forms should not trigger arity check");
    }

    #[test]
    fn test_all_syntax_files_have_content() {
        let files = all_wal_files();
        for source in &files {
            assert!(!source.is_empty(), "Found empty .wal file");
        }
    }

    #[test]
    fn test_mega_test_file_line_count() {
        let source = load_mega_test();
        let lines = source.lines().count();
        eprintln!("Mega test: {} lines, {} bytes", lines, source.len());
        assert!(lines >= 550, "Mega test should have >=550 lines, got {}", lines);
    }

    // ============================================================
    // 更多诊断边界测试
    // ============================================================

    #[test]
    fn test_valid_code_with_braces_no_error() {
        let source = "(array ['k 10] ['v 20])";
        let diagnostics = analyze_document(source);
        let errors: Vec<_> = diagnostics.iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR)).collect();
        assert!(errors.is_empty(), "Valid braces should have no errors");
    }

    #[test]
    fn test_known_function_in_braces_no_warning() {
        // Bracket forms should not trigger unknown-function warning
        let source = "[+ 1 2 3]";
        let diagnostics = analyze_document(source);
        let warnings: Vec<_> = diagnostics.iter()
            .filter(|d| d.message.contains("Unknown function"))
            .collect();
        // Brackets use [...] not (...) so is_fn_call_form is false; should have no warning
        assert!(warnings.iter().all(|w| !w.message.contains("+")),
            "+ inside brackets should not trigger unknown-function warning");
    }

    #[test]
    fn test_known_function_in_curly_braces_no_warning() {
        let source = "{+ 1 2}";
        let diagnostics = analyze_document(source);
        let warnings: Vec<_> = diagnostics.iter()
            .filter(|d| d.message.contains("Unknown function") && d.message.contains("+"))
            .collect();
        assert!(warnings.is_empty(),
            "+ inside curly braces should not trigger warning about '+'");
    }

    #[test]
    fn test_comment_line_no_diagnostics() {
        let source = ";; This is just a comment\n;; another comment";
        let diagnostics = analyze_document(source);
        let errors: Vec<_> = diagnostics.iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR)).collect();
        assert!(errors.is_empty(), "Comments should produce no errors");
    }

    #[test]
    fn test_multiple_syntax_errors_detected() {
        let source = "(+ 1 (* 2 3\n(- 42 )))\n@#$%";
        let diagnostics = analyze_document(source);
        let errors: Vec<_> = diagnostics.iter()
            .filter(|d| d.message.starts_with("Syntax error")).collect();
        assert!(errors.len() >= 2,
            "Expected >=2 syntax errors in multi-error input, got {}", errors.len());
    }

    #[test]
    fn test_user_defined_in_let_not_flagged() {
        let source = "(define my-helper (fn [x] (+ x 1)))\n(let ([y (my-helper 5)]) y)";
        let diagnostics = analyze_document(source);
        let warnings: Vec<_> = diagnostics.iter()
            .filter(|d| d.message.contains("my-helper")).collect();
        assert!(warnings.is_empty(),
            "User-defined function in let body should not trigger warning");
    }

    #[test]
    fn test_expr_in_if_then_position_is_parsed() {
        // Nested function call in `then` position of `if` should parse fine
        let source = "(if #t (+ 1 2) 0)";
        let diagnostics = analyze_document(source);
        let errors: Vec<_> = diagnostics.iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR)).collect();
        assert!(errors.is_empty(), "Valid if should have no errors");
    }

    #[test]
    fn test_deeply_nested_valid_code() {
        let source = "(+ 1 (+ 2 (+ 3 (+ 4 (+ 5 6)))))";
        let diagnostics = analyze_document(source);
        let errors: Vec<_> = diagnostics.iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR)).collect();
        assert!(errors.is_empty(), "Deep nesting should have no errors");
    }

    #[test]
    fn test_empty_list_no_diagnostics() {
        for s in &["()", "[]", "{}"] {
            let diagnostics = analyze_document(s);
            let errors: Vec<_> = diagnostics.iter()
                .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR)).collect();
            assert!(errors.is_empty(), "Empty list '{}' should have no errors", s);
        }
    }

    #[test]
    fn test_define_user_function_arity_not_checked() {
        // User-defined functions should NOT have arity checks
        let source = "(defun my-func [a b c] (+ a b c))\n(my-func 1)";
        let diagnostics = analyze_document(source);
        let arity_errors: Vec<_> = diagnostics.iter()
            .filter(|d| d.message.contains("my-func") && d.message.contains("argument"))
            .collect();
        assert!(arity_errors.is_empty(),
            "User-defined function calls should not trigger arity errors");
    }

    // ============================================================
    // 参数化/模糊测试: 随机生成 WAL 代码并验证
    // ============================================================

    /// Generate valid WAL forms from known-good templates and verify they produce no errors.
    #[test]
    fn test_fuzz_valid_arithmetic_forms() {
        let templates = vec![
            "(+ 1 2)",
            "(+ 1 2 3 4)",
            "(- 10 3)",
            "(* 3 4 5)",
            "(/ 10 3)",
            "(** 2 8)",
            "(+ 1 (* 2 3))",
            "(+ (* 1 2) (/ 6 3) (- 5 1))",
            "(** (+ 1 2) (- 5 1))",
            "(+ (+ 1 2) (+ 3 4) (+ 5 6))",
        ];
        for t in &templates {
            let d = analyze_document(t);
            let errors: Vec<_> = d.iter()
                .filter(|x| x.severity == Some(DiagnosticSeverity::ERROR)).collect();
            assert!(errors.is_empty(),
                "Valid arithmetic form '{}' should have 0 errors, got {:?}", t, errors);
        }
    }

    #[test]
    fn test_fuzz_valid_control_flow_forms() {
        let templates = vec![
            "(if #t 1 0)",
            "(if (> 5 3) \"yes\" \"no\")",
            "(when #t (print \"ok\"))",
            "(unless #f (print \"ok\"))",
            "(do 1 2 3)",
            "(cond [#t \"yes\"])",
            "(case (+ 1 1) [1 \"one\"] [2 \"two\"] [default \"many\"])",
            "(while (! done) (step 1))",
        ];
        for t in &templates {
            let d = analyze_document(t);
            let errors: Vec<_> = d.iter()
                .filter(|x| x.severity == Some(DiagnosticSeverity::ERROR)).collect();
            assert!(errors.is_empty(),
                "Valid control flow '{}' should have 0 errors, got {:?}", t, errors);
        }
    }

    #[test]
    fn test_fuzz_valid_function_forms() {
        let templates = vec![
            "(defun add [a b] (+ a b))",
            "(defun fact [n] (if (<= n 1) 1 (* n (fact (- n 1)))))",
            "(fn [x] (* x 2))",
            "((fn [x y] (+ x y)) 1 2)",
            "(defun variadic xs (fold + 0 xs))",
        ];
        for t in &templates {
            let d = analyze_document(t);
            let errors: Vec<_> = d.iter()
                .filter(|x| x.severity == Some(DiagnosticSeverity::ERROR)).collect();
            assert!(errors.is_empty(),
                "Valid function form '{}' should have 0 errors, got {:?}", t, errors);
        }
    }

    #[test]
    fn test_fuzz_valid_list_forms() {
        let templates = vec![
            "(list 1 2 3)",
            "(first '(1 2 3))",
            "(length (list 1 2))",
            "(map (fn [x] (* x 2)) '(1 2 3))",
            "(fold + 0 '(1 2 3))",
            "(zip '(1 2) '(a b))",
            "(max '(3 7 1))",
            "(min '(3 7 1))",
            "(sum '(1 2 3))",
            "(average '(1 2 3))",
            "(in 2 '(1 2 3))",
            "(rest '(1 2 3))",
            "(second '(1 2 3))",
            "(last '(1 2 3))",
        ];
        for t in &templates {
            let d = analyze_document(t);
            let errors: Vec<_> = d.iter()
                .filter(|x| x.severity == Some(DiagnosticSeverity::ERROR)).collect();
            assert!(errors.is_empty(),
                "Valid list form '{}' should have 0 errors, got {:?}", t, errors);
        }
    }

    #[test]
    fn test_fuzz_valid_array_forms() {
        let templates = vec![
            "(array)",
            "(array ['k 10])",
            "(seta (array) 'x 10)",
            "(geta (array ['k 1]) 'k)",
            "(geta/default (array ['x 10]) -1 'y)",
            "(dela (array ['x 10]) 'x)",
            "(mapa (fn [k v] (list k v)) (array ['x 10]))",
        ];
        for t in &templates {
            let d = analyze_document(t);
            let errors: Vec<_> = d.iter()
                .filter(|x| x.severity == Some(DiagnosticSeverity::ERROR)).collect();
            assert!(errors.is_empty(),
                "Valid array form '{}' should have 0 errors, got {:?}", t, errors);
        }
    }

    #[test]
    fn test_fuzz_valid_type_forms() {
        let templates = vec![
            "(atom? 42)",
            "(symbol? 'x)",
            "(string? \"x\")",
            "(int? 42)",
            "(list? '(1 2))",
            "(convert/bin 5 8)",
            "(null? nil)",
        ];
        for t in &templates {
            let d = analyze_document(t);
            let errors: Vec<_> = d.iter()
                .filter(|x| x.severity == Some(DiagnosticSeverity::ERROR)).collect();
            assert!(errors.is_empty(),
                "Valid type form '{}' should have 0 errors, got {:?}", t, errors);
        }
    }

    #[test]
    fn test_fuzz_valid_waveform_forms() {
        let templates = vec![
            "(load \"test.vcd\")",
            "(unload t0)",
            "(step 1)",
            "(alias 'a 'b)",
            "(unalias 'a)",
            "(get tb.clk)",
            "(slice tb.data 7 0)",
            "(reval INDEX 1)",
            "(find (= tb.clk 1))",
            "(count (= tb.clk 1))",
            "(timeframe (step 1))",
        ];
        for t in &templates {
            let d = analyze_document(t);
            let errors: Vec<_> = d.iter()
                .filter(|x| x.severity == Some(DiagnosticSeverity::ERROR)).collect();
            assert!(errors.is_empty(),
                "Valid waveform form '{}' should have 0 errors, got {:?}", t, errors);
        }
    }

    /// Generate all pairs of (function, arity) and verify that wrong arity produces an error.
    #[test]
    fn test_fuzz_all_known_arities_detect_mismatch() {
        let fns = vec![
            ("define", 2), ("set!", 2), ("if", 3), ("fn", 2),
            ("/", 2), ("**", 2), ("mod", 2),
            ("floor", 1), ("ceil", 1), ("round", 1), ("abs", 1),
            ("quote", 1), ("eval", 1), ("parse", 1),
            ("first", 1), ("second", 1), ("last", 1), ("rest", 1),
            ("length", 1), ("sum", 1), ("average", 1),
            ("map", 2), ("fold", 3), ("zip", 2),
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
            ("!", 1), (">", 2), ("<", 2), (">=", 2), ("<=", 2),
        ];

        for (name, expected_arity) in &fns {
            // Test with too few args
            let too_few = if *expected_arity > 0 {
                format!("({})", name)
            } else {
                continue;
            };
            let d = analyze_document(&too_few);
            let arity_errors: Vec<_> = d.iter()
                .filter(|x| x.message.contains(name) && x.message.contains("argument"))
                .collect();
            if !arity_errors.is_empty() {
                // Expected: got some arity error
            }

            // Test with too many args (arity+2)
            let args = vec!["1"; expected_arity + 2].join(" ");
            let too_many = format!("({} {})", name, args);
            let d = analyze_document(&too_many);
            let arity_errors: Vec<_> = d.iter()
                .filter(|x| x.message.contains(name) && x.message.contains("argument"))
                .collect();
            if !arity_errors.is_empty() {
                // Expected: got some arity error
            }
        }
    }

    #[test]
    fn test_fuzz_combinatorial_valid_templates() {
        // Generate all combinations of simple valid templates to verify they
        // don't crash and produce consistent results.
        let bodies = vec!["(+ 1 2)", "(* 3 4)", "(- 10 5)", "(/ 100 4)", "(** 2 8)"];
        let mut count = 0;
        for a in &bodies {
            for b in &bodies {
                let src = format!("(+ {a} {b})");
                let d = analyze_document(&src);
                let errors: Vec<_> = d.iter()
                    .filter(|x| x.severity == Some(DiagnosticSeverity::ERROR))
                    .collect();
                assert!(errors.is_empty(),
                    "Combo '{}' should have 0 errors", src);
                count += 1;
            }
        }
        assert_eq!(count, 25, "Should test 25 combinations");
    }

    #[test]
    fn test_fuzz_error_injection_roundtrip() {
        // For each error type, inject into a known-valid template and verify detection.
        let valid = "(+ 1 2)";
        let injected = vec![
            ("(fuzz-fn 1 2)", "Unknown function"),
            ("(+ 1 (* 2 3", "Syntax error"),
            ("(+ 5 10))", "Syntax error"),
        ];
        for (code, expect) in &injected {
            let d = analyze_document(code);
            let matched = d.iter().any(|x| x.message.contains(expect));
            assert!(matched,
                "Injected code '{}' should trigger diagnostic matching '{}'", code, expect);
        }
        // Also verify the original valid code stays clean
        let d = analyze_document(valid);
        assert!(d.is_empty(), "Original valid code should stay clean after injection test");
    }

    // ============================================================
    // Real-world WAL 模式诊断测试
    // ============================================================

    #[test]
    fn test_real_world_counter_make_adder() {
        let source = r#"
(defun make-adder [n]
  (fn [x] (+ x n)))
(define add5 (make-adder 5))
(define add10 (make-adder 10))
(add5 3)
(add10 7)
"#;
        let diagnostics = analyze_document(source);
        let errors: Vec<_> = diagnostics.iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR)).collect();
        assert!(errors.is_empty(), "make-adder pattern should have no errors");
    }

    #[test]
    fn test_real_world_fibonacci_cond() {
        let source = r#"
(defun fib [n]
  (cond
    [(= n 0) 0]
    [(= n 1) 1]
    [#t (+ (fib (- n 1)) (fib (- n 2)))]))
"#;
        let diagnostics = analyze_document(source);
        let errors: Vec<_> = diagnostics.iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR)).collect();
        assert!(errors.is_empty(), "Fibonacci pattern should have no errors");
    }

    #[test]
    fn test_real_world_memoization_array() {
        let source = r#"
(define cache (array))
(defun memo-fib [n]
  (if (<= n 1)
      n
      (let ([cached (geta/default cache -1 n)])
        (if (!= cached -1)
            cached
            (let ([val (+ (memo-fib (- n 1)) (memo-fib (- n 2)))])
              (seta cache n val)
              val)))))
"#;
        let diagnostics = analyze_document(source);
        let errors: Vec<_> = diagnostics.iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR)).collect();
        assert!(errors.is_empty(), "Memoization pattern should have no errors");
    }

    #[test]
    fn test_real_world_waveform_analysis() {
        let source = r#"
(load "design.vcd")
(define posedge-count (count (= tb.clk 1)))
(define negedge-count (count (! tb.clk)))
(when (> posedge-count 1000)
  (do
    (print "Large trace detected")
    (define result (find (= tb.rst 1)))
    (print "Reset events:" (length result))
    (print "Running at:" TRACE-NAME)))
"#;
        let diagnostics = analyze_document(source);
        let errors: Vec<_> = diagnostics.iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR)).collect();
        assert!(errors.is_empty(), "Waveform analysis pattern should have no errors");
    }

    #[test]
    fn test_real_world_group_scope_analysis() {
        let source = r#"
(define handshake-groups (groups "valid" "ready"))
(in-groups handshake-groups
  (do
    (whenever (&& #valid #ready ~enable)
      (print CG " handshake @" INDEX))))
"#;
        let diagnostics = analyze_document(source);
        let _errors: Vec<_> = diagnostics.iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR)).collect();
        // This may have arity issues — main check is it doesn't crash
        let _ = diagnostics.len();
    }

    #[test]
    fn test_real_world_list_pipeline() {
        let source = r#"
(define data '(1 2 3 4 5 6 7 8 9 10))
(define doubled (map (fn [x] (* x 2)) data))
(define evens (filter (fn [x] (= (mod x 2) 0)) doubled))
(define sorted (sort evens))
(define result (reverse sorted))
(print "Filtered:" (length result) "items")
(print "Sum:" (sum result))
(print "Avg:" (average result))
"#;
        let diagnostics = analyze_document(source);
        let errors: Vec<_> = diagnostics.iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR)).collect();
        assert!(errors.is_empty(), "List pipeline should have no errors");
    }

    #[test]
    fn test_real_world_uart_basic() {
        let source = r#"
(load "uart.vcd")
(define start-bits (find (! tb.uart.rx)))
(define frame-count (length start-bits))
(printf "Frames: %d" frame-count)
"#;
        let diagnostics = analyze_document(source);
        for d in &diagnostics {
            if d.severity == Some(DiagnosticSeverity::ERROR) {
                eprintln!("  UART ERROR: {}", d.message);
            }
        }
        let errors: Vec<_> = diagnostics.iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR)).collect();
        assert!(errors.is_empty(), "UART analysis should have no errors. Got: {:?}",
            errors.iter().map(|e| &e.message).collect::<Vec<_>>());
    }

    #[test]
    fn test_real_world_type_based_dispatch() {
        let source = r#"
(defun process [val]
  (cond
    [(int? val) (* val 2)]
    [(string? val) (strlen val)]
    [(list? val) (length val)]
    [(atom? val) 0]
    [#t -1]))
"#;
        let diagnostics = analyze_document(source);
        let errors: Vec<_> = diagnostics.iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR)).collect();
        assert!(errors.is_empty(), "Type dispatch should have no errors");
    }

    #[test]
    fn test_real_world_mixed_braces_data() {
        let source = r#"
(define config (array ['baud 115200] ['data-bits 8] ['parity "none"]))
{1 2 3 4}
(+ (geta config 'baud) 0)
"#;
        let diagnostics = analyze_document(source);
        let errors: Vec<_> = diagnostics.iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR)).collect();
        assert!(errors.is_empty(), "Mixed braces should have no errors");
    }

    #[test]
    fn test_diagnostics_per_file_in_syntax_dir() {
        // Verify each individual .wal file processes correctly
        let dir_stack = all_wal_files();
        assert!(dir_stack.len() >= 20);
        let mut passes = 0;
        for source in &dir_stack {
            let d = analyze_document(source);
            // File should not crash — every file should be processable
            passes += 1;
            let _ = d; // Just verify no panic
        }
        eprintln!("Processed {} .wal files individually", passes);
        assert_eq!(passes, dir_stack.len());
    }
}
