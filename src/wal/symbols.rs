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
    let root = tree.root_node();
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
        // First symbol: "add" from (define add ...)
        assert_eq!(symbols[0].name, "add");
        // Second symbol: "greeting" from (define greeting ...)
        // The inner (fn ...) is also extracted as a nested symbol (name="fn")
        assert!(symbols.iter().any(|s| s.name == "greeting"));
    }
}
