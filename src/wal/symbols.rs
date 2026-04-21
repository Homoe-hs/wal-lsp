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
    let mut cursor = node.walk();
    let child_nodes: Vec<Node> = node.children(&mut cursor).collect();

    if child_nodes.is_empty() {
        return None;
    }

    let first = child_nodes.first()?;
    if first.kind() != "atom" && first.kind() != "symbol" && first.kind() != "base_symbol" {
        return None;
    }

    let first_text = get_node_text(*first, source)?;
    let kind = match first_text.as_str() {
        "define" => SymbolKind::VARIABLE,
        "fn" => SymbolKind::FUNCTION,
        "defsig" => SymbolKind::VARIABLE,
        "defmacro" => SymbolKind::METHOD,
        _ => return None,
    };

    let name = if child_nodes.len() >= 2 {
        get_node_text(child_nodes[1], source).unwrap_or_else(|| first_text.clone())
    } else {
        first_text.clone()
    };

    let range = node_to_range(node);

    let mut children = Vec::new();
    for (i, child) in child_nodes.iter().enumerate().skip(2) {
        if child.kind() == "list" || child.kind() == "sexpr" {
            if let Some(child_symbol) = try_extract_definition(*child, source) {
                children.push(child_symbol);
            }
        } else if i > 1 {
            let text = get_node_text(*child, source).unwrap_or_default();
            if !text.is_empty() && text != "[]" && text != "()" && text != "{}" {
                children.push(WalSymbol {
                    name: text,
                    kind: SymbolKind::FIELD,
                    range: node_to_range(*child),
                    detail: None,
                    children: Vec::new(),
                });
            }
        }
    }

    let detail = match first_text.as_str() {
        "define" if child_nodes.len() >= 3 => {
            let value_text = get_node_text(child_nodes[2], source);
            Some(format!("= {}", value_text.unwrap_or_default()))
        }
        "fn" if child_nodes.len() >= 3 => {
            let args_text = get_node_text(child_nodes[2], source);
            Some(format!("fn {}", args_text.unwrap_or_default()))
        }
        "defsig" if child_nodes.len() >= 3 => {
            let expr_text = get_node_text(child_nodes[2], source);
            Some(format!("defsig {}", expr_text.unwrap_or_default()))
        }
        "defmacro" if child_nodes.len() >= 3 => {
            let args_text = get_node_text(child_nodes[2], source);
            Some(format!("macro {}", args_text.unwrap_or_default()))
        }
        _ => None,
    };

    Some(WalSymbol {
        name,
        kind,
        range,
        detail,
        children,
    })
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
        assert_eq!(symbols[0].name, "define");
    }

    #[test]
    fn test_extract_fn() {
        let source = "(fn [x y] (+ x y))";
        let symbols = extract_symbols(source);
        assert!(!symbols.is_empty());
        assert_eq!(symbols[0].name, "fn");
    }

    #[test]
    fn test_extract_complex() {
        let source = r#"
(define add (fn [x y] (+ x y)))
(define greeting "Hello")
"#;
        let symbols = extract_symbols(source);
        assert!(symbols.len() >= 2);
    }
}
