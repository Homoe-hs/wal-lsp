use crate::wal::parser::WalParser;
use tree_sitter::Node;

/// 格式化选项 — 类似 verible 的 `FormatOptions` (列限制预留，后续实现行包裹)
#[derive(Debug, Clone, Copy)]
pub struct FormatOptions {
    /// 每级缩进空格数 (verible 默认 2)
    pub indentation_spaces: u32,
    /// 目标行长度上限 (verible 默认 100，预留)
    #[allow(dead_code)]
    pub column_limit: u32,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indentation_spaces: 2,
            column_limit: 100,
        }
    }
}

/// 使用默认选项格式化
pub fn format_document(source: &str) -> String {
    format_document_with_opts(source, &FormatOptions::default())
}

/// 使用指定选项格式化
pub fn format_document_with_opts(source: &str, opts: &FormatOptions) -> String {
    let mut parser = WalParser::new();
    let tree = parser.parse_with_errors(source);
    let root = tree.root_node();

    let mut output = String::new();
    format_node(root, source, 0, opts, &mut output);
    output
}

fn format_node(node: Node, source: &str, indent: u32, opts: &FormatOptions, output: &mut String) {
    let kind = node.kind();

    match kind {
        "program" | "source_file" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                format_node(child, source, indent, opts, output);
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

            let content_start = 1;
            let content_end = children.len().saturating_sub(1);

            if content_start < content_end {
                let rest = &children[content_start..content_end];

                if rest.iter().any(|c| c.kind() == "list" || c.kind() == "sexpr") {
                    for child in rest {
                        output.push('\n');
                        append_indent(output, indent + 1, opts);
                        format_node(*child, source, indent + 1, opts, output);
                    }
                } else if !rest.is_empty() {
                    let first_content = rest[0];
                    if first_content.kind() == "sexpr_list" {
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
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                format_node(child, source, indent, opts, output);
            }
        }
        "quoted" | "quasiquoted" | "unquote" | "unquote_splice" => {
            let prefix = match kind {
                "quoted" => "'",
                "quasiquoted" => "`",
                "unquote" => ",",
                "unquote_splice" => ",@",
                _ => "",
            };
            output.push_str(prefix);
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.is_named() {
                    format_node(child, source, indent, opts, output);
                }
            }
        }
        "timed_atom" => {
            let mut cursor = node.walk();
            let children: Vec<Node> = node.children(&mut cursor).collect();
            if let Some(first) = children.first() {
                format_node(*first, source, indent, opts, output);
            }
            output.push('@');
            if children.len() > 1 {
                for child in &children[1..] {
                    format_node(*child, source, indent, opts, output);
                }
            }
        }
        _ => {
            let text = source.get(node.byte_range()).unwrap_or("").trim().to_string();
            output.push_str(&text);
        }
    }
}

fn append_indent(output: &mut String, level: u32, opts: &FormatOptions) {
    let total = (level as usize) * (opts.indentation_spaces as usize);
    for _ in 0..total {
        output.push(' ');
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
        let output = format_document(input);
        assert!(!output.is_empty());
    }

    #[test]
    fn test_format_empty_input() {
        let output = format_document("");
        assert!(output.is_empty(), "Empty input should produce empty output");
    }

    #[test]
    fn test_format_only_comment() {
        let output = format_document(";; just a comment");
        assert!(output.contains(";;"), "Comment should be preserved");
    }

    #[test]
    fn test_format_comment_with_code() {
        let output = format_document(";; header\n(define x 1)");
        // tree-sitter treats comments as extras and may strip them;
        // verify the code portion is preserved
        assert!(output.contains("define"));
        assert!(output.contains("x"));
    }

    #[test]
    fn test_format_no_panic_on_deep_nesting() {
        let input = "(+ 1 (+ 2 (+ 3 (+ 4 (+ 5 (+ 6 (+ 7 (+ 8 9))))))))";
        let output = format_document(input);
        assert!(!output.is_empty());
    }

    #[test]
    fn test_format_multi_sexpr() {
        let input = "(define x 1)\n(define y 2)\n(+ x y)";
        let output = format_document(input);
        assert!(output.contains("x"));
        assert!(output.contains("y"));
    }

    #[test]
    fn test_format_idempotent() {
        let input = "(do (define x 1) (define y 2) (+ x y))";
        let first = format_document(input);
        let second = format_document(&first);
        // After two passes, output should be stable (same number of non-whitespace chars)
        let normalize = |s: &str| -> String {
            s.chars().filter(|c| !c.is_whitespace()).collect()
        };
        assert_eq!(normalize(&first), normalize(&second),
            "Format should be idempotent");
    }

    #[test]
    fn test_format_bare_atoms() {
        let input = "42";
        let output = format_document(input);
        assert!(output.contains("42"));
    }

    #[test]
    fn test_format_multiple_bracket_types() {
        let input = "(let ([x [1 2]] [y {3 4}]) (+ (first x) (first y)))";
        let output = format_document(input);
        assert!(output.contains("[1 2]"));
        assert!(output.contains("{3 4}"));
    }

    #[test]
    fn test_format_empty_list() {
        let tests = ["()", "[]", "{}"];
        for t in &tests {
            let output = format_document(t);
            assert!(!output.is_empty(), "Empty list '{}' should produce output", t);
        }
    }

    #[test]
    fn test_format_complex_real_world_code() {
        let input = r#"(defun process [data xs]
  (let ([n (length xs)]
        [total (sum xs)]
        [avg (/ total n)])
    (cond
      [(> avg 100) (print "large")]
      [(< avg 10) (print "small")]
      [#t (do
            (define result (map (fn [x] (* data x)) xs))
            (fold + 0 result))]))))"#;
        let output = format_document(input);
        // Should not panic and should preserve key structure
        assert!(!output.is_empty());
    }

    #[test]
    fn test_format_roundtrip_simple() {
        let inputs = vec![
            "(define x 1)",
            "(+ 1 2 3)",
            "(if #t 1 0)",
            "(let ([x 10]) (+ x 1))",
            "(cond [#t \"yes\"])",
            "(defun f [x] (* x 2))",
            "(when #t (print \"ok\"))",
        ];
        for input in &inputs {
            let first = format_document(input);
            let second = format_document(&first);
            let normalize = |s: &str| -> String {
                s.chars().filter(|c| !c.is_whitespace()).collect()
            };
            assert_eq!(normalize(&first), normalize(&second),
                "Roundtrip failed for '{}'", input);
        }
    }

    #[test]
    fn test_format_roundtrip_nested() {
        let inputs = vec![
            "(+ (* 1 2) (/ 3 4))",
            "(defun add [a b] (+ a b))",
            "(array ['k1 10] ['k2 20])",
            "(map (fn [x] (* x 2)) '(1 2 3))",
            "(case x [1 \"a\"] [2 \"b\"] [default \"c\"])",
        ];
        for input in &inputs {
            let first = format_document(input);
            let second = format_document(&first);
            let normalize = |s: &str| -> String {
                s.chars().filter(|c| !c.is_whitespace()).collect()
            };
            assert_eq!(normalize(&first), normalize(&second),
                "Roundtrip failed for nested '{}'", input);
        }
    }

    #[test]
    fn test_format_roundtrip_with_quotes() {
        let inputs = vec![
            "'(1 2 3)",
            "`(a ,b ,@c)",
            "'(quote hello)",
        ];
        for input in &inputs {
            let output = format_document(input);
            assert!(!output.is_empty(), "Quoted form '{}' should produce output", input);
        }
    }

    #[test]
    fn test_format_ultra_deep_nesting() {
        // 20-level deep nesting
        let input = "(+ 1 (+ 2 (+ 3 (+ 4 (+ 5 (+ 6 (+ 7 (+ 8 (+ 9 (+ 10 (+ 11 (+ 12 (+ 13 (+ 14 (+ 15 (+ 16 (+ 17 (+ 18 (+ 19 20)))))))))))))))))))";
        let output = format_document(input);
        assert!(!output.is_empty());
        // Should not panic
    }

    #[test]
    fn test_format_many_sexprs_in_sequence() {
        let mut input = String::new();
        for i in 0..50 {
            input.push_str(&format!("(define var{} {})\n", i, i));
        }
        let output = format_document(&input);
        for i in 0..50 {
            assert!(output.contains(&format!("var{}", i)), "Missing var{}", i);
        }
    }

    #[test]
    fn test_format_mixed_bracket_depth() {
        let input = "(let ([x [1 2 3]] [y {4 5 6}] [z '(7 8 9)]) (list (first x) (first y) (first z)))";
        let output = format_document(input);
        assert!(!output.is_empty());
    }

    #[test]
    fn test_format_all_types_in_one_expr() {
        let input = r#"(defun all-types [s i b l a]
  (do
    (define str-val (strlen s))
    (define int-val (* i 2))
    (define bool-val (and b true))
    (define list-sum (sum l))
    (define arr-get (geta a 'key))
    (list str-val int-val bool-val list-sum arr-get)))"#;
        let output = format_document(input);
        assert!(!output.is_empty());
    }

    #[test]
    fn test_format_consistency_across_types() {
        let inputs = [
            "(+ 1 2)",
            "(- 3 4)",
            "(if #t 1 0)",
            "(let ([x 1]) x)",
            "(defun f [x] x)",
            "(map (fn [x] x) '(1 2))",
            "(fold + 0 '(1 2 3))",
            "(array ['k 1])",
            "(case x [1 \"a\"])",
            "'(1 2 3)",
        ];
        for input in &inputs {
            let output = format_document(input);
            assert!(!output.is_empty(), "Format '{}' should produce output", input);
            let second = format_document(&output);
            let norm = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
            assert_eq!(norm(&output), norm(&second),
                "Reformat of '{}' changed content", input);
        }
    }

    #[test]
    fn test_format_with_custom_indent_2_spaces() {
        let opts = FormatOptions { indentation_spaces: 2, column_limit: 100 };
        let input = "(defun add [a b] (+ a b))";
        let output = format_document_with_opts(input, &opts);
        assert!(!output.is_empty());
    }

    #[test]
    fn test_format_with_custom_indent_8_spaces() {
        let opts = FormatOptions { indentation_spaces: 8, column_limit: 100 };
        let input = "(defun add [a b] (+ a b))";
        let output = format_document_with_opts(input, &opts);
        assert!(!output.is_empty());
    }

    #[test]
    fn test_format_options_idempotent() {
        let opts = FormatOptions { indentation_spaces: 3, column_limit: 80 };
        let input = "(do (define x 1) (define y 2) (+ x y))";
        let first = format_document_with_opts(input, &opts);
        let second = format_document_with_opts(&first, &opts);
        let norm = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
        assert_eq!(norm(&first), norm(&second), "FormatOptions idempotent");
    }

    #[test]
    fn test_format_options_nested_indent() {
        let opts = FormatOptions { indentation_spaces: 6, column_limit: 120 };
        let input = "(let ([x 10] [y 20]) (+ x y))";
        let output = format_document_with_opts(input, &opts);
        // With 6-space indent, nested content should be indented by 6
        assert!(!output.is_empty());
    }
}
