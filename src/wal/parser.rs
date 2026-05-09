use tree_sitter::{Parser, Tree};

pub struct WalParser {
    parser: Parser,
    language_set: bool,
}

pub static WAL_PARSER: std::sync::LazyLock<std::sync::Mutex<WalParser>> =
    std::sync::LazyLock::new(|| {
        std::sync::Mutex::new(WalParser::new_inner())
    });

impl WalParser {
    fn new_inner() -> Self {
        let mut parser = Parser::new();
        let language_set = parser.set_language(&crate::wal::language()).is_ok();
        WalParser { parser, language_set }
    }

    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::new_inner()
    }

    #[allow(dead_code)]
    pub fn parse(&mut self, source: &str) -> Result<Tree, String> {
        self.parser
            .parse(source, None)
            .ok_or_else(|| "Parse failed".to_string())
    }

    pub fn parse_with_errors(&mut self, source: &str) -> Tree {
        if !self.language_set {
            // Create a fallback parser with language set
            let mut fallback = Parser::new();
            let _ = fallback.set_language(&crate::wal::language());
            if let Some(tree) = fallback.parse(source, None) {
                return tree;
            }
        }
        self.parser.parse(source, None).unwrap_or_else(|| {
            let mut parser = Parser::new();
            let _ = parser.set_language(&crate::wal::language());
            parser.parse(source, None).unwrap_or_else(|| {
                // Absolute fallback: empty parse (always succeeds)
                parser.parse("", None).unwrap()
            })
        })
    }

    pub fn parse_incremental(&mut self, source: &str, old_tree: Option<&Tree>) -> Tree {
        self.parser.parse(source, old_tree).unwrap_or_else(|| {
            self.parse_with_errors(source)
        })
    }
}

impl Default for WalParser {
    fn default() -> Self {
        Self::new_inner()
    }
}

#[allow(dead_code)]
pub fn get_node_text(node: tree_sitter::Node, source: &str) -> Option<String> {
    source.get(node.byte_range()).map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let mut parser = WalParser::new();
        let tree = parser.parse("(+ 1 2)").unwrap();
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_with_comment() {
        let mut parser = WalParser::new();
        let tree = parser.parse(";; comment\n(+ 1 2)").unwrap();
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_defun() {
        let mut parser = WalParser::new();
        let source = r#"
(define add (fn [x y] (+ x y)))
"#;
        let tree = parser.parse(source).unwrap();
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_incremental() {
        let mut parser = WalParser::new();
        let tree1 = parser.parse_incremental("(+ 1 2)", None);
        assert!(!tree1.root_node().has_error());

        let tree2 = parser.parse_incremental("(+ 1 2 3)", Some(&tree1));
        assert!(!tree2.root_node().has_error());
    }

    #[test]
    fn test_parse_incremental_from_empty() {
        let mut parser = WalParser::new();
        let tree = parser.parse_incremental("(define x 1)", None);
        assert!(!tree.root_node().has_error());
    }
}
