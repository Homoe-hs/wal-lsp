use tree_sitter::{Parser, Tree};

#[allow(dead_code)]
pub struct WalParser {
    parser: Parser,
}

impl WalParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&crate::wal::language())
            .expect("Failed to set WAL language");
        WalParser { parser }
    }

    #[allow(dead_code)]
    pub fn parse(&mut self, source: &str) -> Result<Tree, String> {
        self.parser
            .parse(source, None)
            .ok_or_else(|| "Parse failed".to_string())
    }

    pub fn parse_with_errors(&mut self, source: &str) -> Tree {
        self.parser.parse(source, None).unwrap_or_else(|| {
            let mut parser = Parser::new();
            parser.set_language(&crate::wal::language()).unwrap();
            parser.parse(source, None).unwrap()
        })
    }
}

impl Default for WalParser {
    fn default() -> Self {
        Self::new()
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
}
