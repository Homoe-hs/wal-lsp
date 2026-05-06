use lsp_types::{DocumentSymbol, Range, SymbolKind};
use tree_sitter::Node;

#[allow(dead_code)]
#[derive(Debug)]
pub struct WalSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub range: Range,
    pub detail: Option<String>,
    pub children: Vec<WalSymbol>,
}

#[allow(dead_code)]
pub fn extract_symbols(source: &str) -> Vec<WalSymbol> {
    let tree = crate::wal::parser::WalParser::new().parse_with_errors(source);
    extract_symbols_from_node(tree.root_node(), source)
}

#[allow(dead_code)]
pub fn extract_symbols_from_node(root: tree_sitter::Node, source: &str) -> Vec<WalSymbol> {
    let mut symbols = Vec::new();
    extract_symbols_recursive(root, source, &mut symbols);
    symbols
}

#[allow(dead_code)]
fn extract_symbols_recursive(node: Node, source: &str, symbols: &mut Vec<WalSymbol>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        if kind == "list" || kind == "sexpr" {
            if let Some(symbol) = try_extract_definition(child, source) {
                symbols.push(symbol);
            }
            extract_symbols_recursive(child, source, symbols);
        } else {
            extract_symbols_recursive(child, source, symbols);
        }
    }
}

#[allow(dead_code)]
fn try_extract_definition(node: Node, source: &str) -> Option<WalSymbol> {
    if node.kind() != "list" {
        return None;
    }

    let mut cursor = node.walk();
    let children: Vec<Node> = node.children(&mut cursor).collect();

    // Find the sexpr_list inside this list node
    let sexpr_list = children.iter().find(|c| c.kind() == "sexpr_list")?;
    let sl_children: Vec<Node> = {
        let mut c = sexpr_list.walk();
        sexpr_list.children(&mut c)
            .filter(|child| child.kind() == "sexpr")
            .collect()
    };

    if sl_children.is_empty() {
        return None;
    }

    let first_text = get_sexpr_text(sl_children[0], source)?;

    let (sym_kind, use_second_as_name) = match first_text.as_str() {
        "define" => (SymbolKind::VARIABLE, true),
        "defun" => (SymbolKind::FUNCTION, true),
        "defmacro" => (SymbolKind::METHOD, true),
        "defsig" => (SymbolKind::VARIABLE, true),
        "fn" => (SymbolKind::FUNCTION, false),
        _ => return None,
    };

    let name = if use_second_as_name && sl_children.len() >= 2 {
        get_sexpr_text(sl_children[1], source).unwrap_or_else(|| first_text.clone())
    } else {
        first_text.clone()
    };

    let detail = if use_second_as_name && sl_children.len() >= 3 {
        let value_text = get_sexpr_text(sl_children[2], source);
        Some(format!("= {}", value_text.unwrap_or_default()))
    } else if !use_second_as_name && sl_children.len() >= 2 {
        let args_text = get_sexpr_text(sl_children[1], source);
        Some(format!("fn {}", args_text.unwrap_or_default()))
    } else {
        None
    };

    let range = node_to_range(node);

    Some(WalSymbol {
        name,
        kind: sym_kind,
        range,
        detail,
        children: Vec::new(),
    })
}

fn get_sexpr_text(sexpr: Node, source: &str) -> Option<String> {
    let mut cursor = sexpr.walk();
    let children: Vec<Node> = sexpr.children(&mut cursor).collect();

    for child in children {
        match child.kind() {
            "atom" | "symbol" | "base_symbol" | "scoped_symbol" | "grouped_symbol"
            | "string" | "int" | "float" | "bool" | "operator" => {
                return source
                    .get(child.byte_range())
                    .map(|s| s.trim().to_string());
            }
            _ => {}
        }
    }
    None
}

#[allow(dead_code)]
fn get_node_text(node: Node, source: &str) -> Option<String> {
    source.get(node.byte_range()).map(|s| {
        let trimmed = s.trim();
        trimmed
            .strip_prefix(['(', '[', '{'])
            .unwrap_or(trimmed)
            .strip_suffix([')', ']', '}'])
            .unwrap_or(trimmed)
            .trim()
            .to_string()
    })
}

#[allow(dead_code)]
fn node_to_range(node: Node) -> Range {
    let start = node.start_position();
    let end = node.end_position();
    Range {
        start: lsp_types::Position {
            line: start.row as u32,
            character: start.column as u32,
        },
        end: lsp_types::Position {
            line: end.row as u32,
            character: end.column as u32,
        },
    }
}

impl WalSymbol {
    #[allow(dead_code, deprecated)]
    pub fn to_document_symbol(&self) -> DocumentSymbol {
        DocumentSymbol {
            name: self.name.clone(),
            kind: self.kind,
            tags: None,
            detail: self.detail.clone(),
            deprecated: None,
            range: self.range,
            selection_range: self.range,
            children: Some(
                self.children
                    .iter()
                    .map(|c| c.to_document_symbol())
                    .collect(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_define() {
        let source = "(define pi 3.14159)";
        let symbols = extract_symbols(source);
        assert!(!symbols.is_empty());
        assert_eq!(symbols[0].name, "pi");
        assert_eq!(symbols[0].kind, SymbolKind::VARIABLE);
    }

    #[test]
    fn test_extract_fn() {
        let source = "(fn [x y] (+ x y))";
        let symbols = extract_symbols(source);
        assert!(!symbols.is_empty());
        assert_eq!(symbols[0].name, "fn");
    }

    #[test]
    fn test_extract_defun() {
        let source = "(defun add [a b] (+ a b))";
        let symbols = extract_symbols(source);
        assert!(!symbols.is_empty());
        assert_eq!(symbols[0].name, "add");
        assert_eq!(symbols[0].kind, SymbolKind::FUNCTION);
    }

    #[test]
    fn test_extract_complex() {
        let source = r#"
(define add (fn [x y] (+ x y)))
(define greeting "Hello")
"#;
        let symbols = extract_symbols(source);
        assert!(!symbols.is_empty());
        assert_eq!(symbols[0].name, "add");
        assert!(symbols.iter().any(|s| s.name == "greeting"));
    }

    #[test]
    fn test_extract_defsig() {
        let source = "(defsig my-signal [7:0])";
        let symbols = extract_symbols(source);
        assert!(!symbols.is_empty());
        assert_eq!(symbols[0].name, "my-signal");
        assert_eq!(symbols[0].kind, SymbolKind::VARIABLE);
    }

    #[test]
    fn test_extract_defmacro() {
        let source = "(defmacro my-when [cond & body] `(if ,cond (do ,@body)))";
        let symbols = extract_symbols(source);
        assert!(!symbols.is_empty());
        assert_eq!(symbols[0].name, "my-when");
        assert_eq!(symbols[0].kind, SymbolKind::METHOD);
    }

    #[test]
    fn test_extract_multiple_defines() {
        let source = "(define a 1)\n(define b 2)\n(define c 3)";
        let symbols = extract_symbols(source);
        assert_eq!(symbols.len(), 3);
        assert!(symbols.iter().any(|s| s.name == "a"));
        assert!(symbols.iter().any(|s| s.name == "b"));
        assert!(symbols.iter().any(|s| s.name == "c"));
    }

    #[test]
    fn test_extract_no_defines_in_plain_expression() {
        let source = "(+ 1 2 3)";
        let symbols = extract_symbols(source);
        assert!(symbols.is_empty(), "No define/defun/defmacro should be found");
    }

    #[test]
    fn test_extract_nested_defines() {
        let source = "(do (define x 1) (define y (+ x 2)))";
        let symbols = extract_symbols(source);
        assert_eq!(symbols.len(), 2);
        assert!(symbols.iter().any(|s| s.name == "x"));
        assert!(symbols.iter().any(|s| s.name == "y"));
    }

    #[test]
    fn test_extract_defuns() {
        let source = "(defun add [a b] (+ a b))\n(defun sub [a b] (- a b))";
        let symbols = extract_symbols(source);
        assert_eq!(symbols.len(), 2);
        assert!(symbols.iter().any(|s| s.name == "add"));
        assert!(symbols.iter().any(|s| s.name == "sub"));
    }

    #[test]
    fn test_extract_from_multi_line_defun() {
        let source = r#"(defun factorial [n]
  (if (<= n 1)
      1
      (* n (factorial (- n 1)))))"#;
        let symbols = extract_symbols(source);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "factorial");
        assert_eq!(symbols[0].kind, SymbolKind::FUNCTION);
    }

    #[test]
    fn test_extract_deeply_nested_define() {
        let source = "(do (do (define deep-var 42) (print deep-var)) (define outer 10))";
        let symbols = extract_symbols(source);
        assert!(!symbols.is_empty());
        assert!(symbols.iter().any(|s| s.name == "deep-var"));
        assert!(symbols.iter().any(|s| s.name == "outer"));
    }

    #[test]
    fn test_extract_fn_inside_define() {
        let source = "(define add (fn [x y] (+ x y)))";
        let symbols = extract_symbols(source);
        assert_eq!(symbols.len(), 2);
        assert!(symbols.iter().any(|s| s.name == "add" && s.kind == SymbolKind::VARIABLE));
        assert!(symbols.iter().any(|s| s.name == "fn" && s.kind == SymbolKind::FUNCTION));
    }

    #[test]
    fn test_extract_let_binding_not_symbol() {
        // let bindings are NOT extracted as symbols (only define/defun/defmacro/defsig)
        let source = "(let ([x 10] [y 20]) (+ x y))";
        let symbols = extract_symbols(source);
        assert_eq!(symbols.len(), 0, "let bindings should not be extracted as symbols");
    }

    #[test]
    fn test_extract_multiple_defsig() {
        let source = "(defsig a [7:0])\n(defsig b [3:0])\n(defsig c [15:0])";
        let symbols = extract_symbols(source);
        assert_eq!(symbols.len(), 3);
        assert!(symbols.iter().all(|s| s.kind == SymbolKind::VARIABLE));
    }

    #[test]
    fn test_extract_with_comments() {
        let source = ";; header comment\n(define x 1)\n;; inline\n(define y 2)\n";
        let symbols = extract_symbols(source);
        assert_eq!(symbols.len(), 2);
    }

    #[test]
    fn test_extract_defun_with_comment_body() {
        let source = r#"(defun greet [name]
  ;; print greeting
  (print "Hello" name)
  ;; return name
  name)"#;
        let symbols = extract_symbols(source);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "greet");
    }

    #[test]
    fn test_extract_complex_lambda_chain() {
        let source = r#"(define chain
  (fn [x]
    (fn [y]
      (fn [z]
        (+ x y z)))))"#;
        let symbols = extract_symbols(source);
        assert!(!symbols.is_empty());
        assert!(symbols.iter().any(|s| s.name == "chain"));
    }

    #[test]
    fn test_extract_defun_with_default_args() {
        let source = r#"(defun inc-define sym)"#;
        let symbols = extract_symbols(source);
        // inc-define is a builtin, but in this context it's being defined
        // Actually, tree-sitter sees "inc-define" as a symbol name
        assert!(!symbols.is_empty());
    }

    #[test]
    fn test_extract_mixed_definitions() {
        let source = r#"
(define state-counter 0)
(defun increment [] (set! state-counter (+ state-counter 1)))
(defmation [7:0])
(defsig virtual-clk (rising tb.clk))
(defmacro my-unless [cond & body] `(if (not ,cond) (do ,@body)))
"#;
        let symbols = extract_symbols(source);
        assert!(!symbols.is_empty());
        // defmation is not recognized — should not cause panic
        let _ = symbols.len();
    }

    #[test]
    fn test_extract_symbol_range_is_valid() {
        let source = "(define pi 3.14159)";
        let symbols = extract_symbols(source);
        assert_eq!(symbols.len(), 1);
        let range = symbols[0].range;
        assert!(range.start.line <= range.end.line);
        assert!(range.start.character <= range.end.character);
    }

    #[test]
    fn test_extract_huge_source() {
        let mut source = String::new();
        for i in 0..100 {
            source.push_str(&format!("(define var_{} {})\n", i, i));
        }
        let symbols = extract_symbols(&source);
        assert_eq!(symbols.len(), 100);
    }

    #[test]
    fn test_extract_empty_source() {
        let source = "";
        let symbols = extract_symbols(source);
        assert!(symbols.is_empty());
    }

    #[test]
    fn test_extract_source_with_only_comments() {
        let source = ";; just a comment\n;; another comment";
        let symbols = extract_symbols(source);
        assert!(symbols.is_empty());
    }
}
