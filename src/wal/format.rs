use crate::wal::parser::WalParser;
use tree_sitter::Node;

const TAB_WIDTH: usize = 4;

#[allow(dead_code)]
pub fn format_document(source: &str) -> String {
    let mut parser = WalParser::new();
    let tree = parser.parse_with_errors(source);
    let root = tree.root_node();

    let mut output = String::new();
    format_node(root, source, 0, &mut output);
    output
}

fn format_node(node: Node, source: &str, indent: usize, output: &mut String) {
    let kind = node.kind();

    match kind {
        "program" | "source_file" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                format_node(child, source, indent, output);
            }
        }
        "list" | "sexpr" => {
            output.push('(');
            let mut cursor = node.walk();
            let children: Vec<Node> = node.children(&mut cursor).collect();

            if !children.is_empty() {
                let first = &children[0];
                let first_text = get_node_text(*first, source);
                if let Some(text) = first_text {
                    output.push_str(&text);
                }

                if children.len() > 1 {
                    let rest = &children[1..];
                    if rest.iter().any(|c| c.kind() == "list" || c.kind() == "sexpr") {
                        for child in rest {
                            output.push('\n');
                            append_tabs(output, indent + 1);
                            format_node(*child, source, indent + 1, output);
                        }
                    } else {
                        for child in rest {
                            output.push(' ');
                            let child_text = get_node_text(*child, source).unwrap_or_default();
                            output.push_str(&child_text);
                        }
                    }
                }
            }

            output.push(')');
        }
        _ => {
            let text = get_node_text(node, source).unwrap_or_default();
            output.push_str(&text);
        }
    }
}

fn get_node_text(node: Node, source: &str) -> Option<String> {
    source.get(node.byte_range()).map(|s| {
        let trimmed = s.trim();
        if trimmed.starts_with('(') || trimmed.starts_with('[') || trimmed.starts_with('{') {
            trimmed.strip_prefix(['(', '[', '{']).unwrap_or(trimmed).strip_suffix([')', ']', '}']).unwrap_or(trimmed).trim().to_string()
        } else {
            trimmed.to_string()
        }
    })
}

fn append_tabs(output: &mut String, count: usize) {
    for _ in 0..count {
        for _ in 0..TAB_WIDTH {
            output.push(' ');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple() {
        let input = "(define x 1)";
        let output = format_document(input);
        assert!(output.contains("(define"));
    }

    #[test]
    fn test_format_nested() {
        let input = "(define add (fn [x y] (+ x y)))";
        let output = format_document(input);
        assert!(output.contains("(define"));
        assert!(output.contains("(fn"));
    }

    #[test]
    fn test_format_multiline() {
        let input = "(do (define x 1) (define y 2) (+ x y))";
        let output = format_document(input);
        assert!(output.contains("(define"));
    }
}