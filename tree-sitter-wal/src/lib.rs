mod bindings;

use tree_sitter::Language;

pub use bindings::tree_sitter_wal;

pub fn language() -> Language {
    unsafe { bindings::language() }
}

pub fn parse(source: &str) -> Result<tree_sitter::Tree, String> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&language())
        .map_err(|e| format!("Failed to set language: {}", e))?;
    parser
        .parse(source, None)
        .ok_or_else(|| "parse failed".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_load_language() {
        let lang = language();
        assert!(lang.into_raw() != std::ptr::null());
    }

    #[test]
    fn can_parse_simple_expression() {
        let tree = parse("(+ 1 2)").unwrap();
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn can_parse_with_comments() {
        let tree = parse(";; comment\n(+ 1 2)").unwrap();
        assert!(!tree.root_node().has_error());
    }
}
