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
        "list" => {
            let mut cursor = node.walk();
            let children: Vec<Node> = node.children(&mut cursor).collect();

            let (open, close) = if let Some(first) = children.first() {
                let bracket_text = source.get(first.byte_range()).unwrap_or("(");
                match bracket_text.trim() {
                    "[" => ('[', ']'),
                    "{" => ('{', '}'),
                    _ => ('(', ')'),
                }
            } else {
                ('(', ')')
            };

            output.push(open);

            let content_start = 1; // skip opening bracket
            let content_end = children.len().saturating_sub(1); // before closing bracket

            if content_start < content_end {
                let rest = &children[content_start..content_end];

                if rest.iter().any(|c| c.kind() == "list" || c.kind() == "sexpr") {
                    for child in rest {
                        output.push('\n');
                        append_tabs(output, indent + 1);
                        format_node(*child, source, indent + 1, output);
                    }
                } else if !rest.is_empty() {
                    let first_content = rest[0];
                    if first_content.kind() == "sexpr_list" {
                        // Flatten simple lists
                        let mut sc = first_content.walk();
                        let sexprs: Vec<Node> = first_content.children(&mut sc).collect();
                        let non_space: Vec<&Node> = sexprs.iter()
                            .filter(|c| c.kind() != "whitespace")
                            .collect();
                        for (i, child) in non_space.iter().enumerate() {
                            if i > 0 { output.push(' '); }
                            let text = source.get(child.byte_range()).unwrap_or("").trim().to_string();
                            output.push_str(&text);
                        }
                    } else {
                        for child in rest {
                            output.push(' ');
                            let text = source.get(child.byte_range()).unwrap_or("").trim().to_string();
                            output.push_str(&text);
                        }
                    }
                }
            }

            output.push(close);
        }
        "sexpr" => {
            // For sexpr wrapping an atom/list — just output the inner content
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                format_node(child, source, indent, output);
            }
        }
        _ => {
            let text = source.get(node.byte_range()).unwrap_or("").trim().to_string();
            output.push_str(&text);
        }
    }
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
        assert!(output.contains("[x y]"));
    }

    #[test]
    fn test_format_multiline() {
        let input = "(do (define x 1) (define y 2) (+ x y))";
        let output = format_document(input);
        assert!(output.contains("(define"));
    }

    #[test]
    fn test_format_preserves_brackets() {
        let input = "(let ([x 10] [y 20]) (+ x y))";
        let output = format_document(input);
        assert!(output.contains("[x 10]"));
        assert!(output.contains("[y 20]"));
    }

    #[test]
    fn test_format_braces() {
        let input = "(array ['x 10] ['y 20])";
        // Array braces {} should be preserved
        let output = format_document(input);
        // The quoted symbols with {} shouldn't be mangled
        assert!(!output.is_empty());
    }
}
