use crate::lsp::WORKSPACE;
use crate::wal::parser::WAL_PARSER;
use anyhow::Result;
use lsp_server::{Connection, Request, Response};
use lsp_types::{FoldingRange, FoldingRangeParams};
use tracing::info;

pub fn handle(connection: &Connection, req: Request) -> Result<()> {
    let params: FoldingRangeParams = serde_json::from_value(req.params)?;
    let uri = params.text_document.uri;

    info!("Folding range requested for {:?}", uri);

    let ranges = get_folding_ranges(&uri);

    let resp = Response::new_ok(req.id, ranges);
    connection
        .sender
        .send(lsp_server::Message::Response(resp))?;

    Ok(())
}

fn get_folding_ranges(uri: &lsp_types::Uri) -> Vec<FoldingRange> {
    let ws = WORKSPACE.read().unwrap_or_else(|e| e.into_inner());
    let doc = match ws.get_document(uri) {
        Some(d) => d,
        None => return vec![],
    };

    let tree = match &doc.tree {
        Some(t) => t.clone(),
        None => {
            let mut parser = WAL_PARSER.lock().unwrap_or_else(|e| e.into_inner());
            parser.parse_incremental(&doc.text, None)
        }
    };

    let mut ranges = Vec::new();
    collect_folding_ranges(tree.root_node(), &doc.text, &mut ranges);
    ranges
}

fn collect_folding_ranges(
    node: tree_sitter::Node,
    source: &str,
    ranges: &mut Vec<FoldingRange>,
) {
    if node.kind() == "list" {
        let start_line = node.start_position().row as u32;
        let end_line = node.end_position().row as u32;
        if end_line > start_line {
            let text = source.get(node.byte_range()).unwrap_or("");
            let open_bracket = text.chars().next().unwrap_or('(');
            let folded_end = if end_line > start_line + 1 {
                end_line - 1
            } else {
                end_line
            };
            ranges.push(FoldingRange {
                start_line,
                start_character: Some(node.start_position().column as u32),
                end_line: folded_end,
                end_character: Some(node.end_position().column as u32),
                kind: if open_bracket == '[' {
                    Some(lsp_types::FoldingRangeKind::Region)
                } else {
                    None
                },
                collapsed_text: None,
            });
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_folding_ranges(child, source, ranges);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lsp::WORKSPACE;
    use std::str::FromStr;

    fn setup(source: &str) {
        let mut ws = WORKSPACE.write().unwrap_or_else(|e| e.into_inner());
        ws.documents.clear();
        ws.symbol_index = crate::workspace::SymbolIndex::new();
        let uri = lsp_types::Uri::from_str("file:///test.wal").unwrap();
        let mut parser = WAL_PARSER.lock().unwrap_or_else(|e| e.into_inner());
        let tree = parser.parse_incremental(source, None);
        ws.open_document_with_tree(uri, source.to_string(), tree);
    }

    fn get_ranges(source: &str) -> Vec<FoldingRange> {
        setup(source);
        let uri = lsp_types::Uri::from_str("file:///test.wal").unwrap();
        get_folding_ranges(&uri)
    }

    #[test]
    fn test_simple_no_fold() {
        let r = get_ranges("(+ 1 2)");
        assert!(r.is_empty(), "Single-line list should not fold");
    }

    #[test]
    fn test_multiline_defun() {
        let r = get_ranges("(defun add [a b]\n  (+ a b))");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].start_line, 0);
        assert_eq!(r[0].end_line, 1);
    }

    #[test]
    fn test_let_bindings() {
        let r = get_ranges("(let ([x 1]\n     [y 2])\n  (+ x y))");
        // The outer let is multiline, some inner [] might be single-line
        let multi: Vec<_> = r.iter().filter(|f| f.end_line > f.start_line).collect();
        assert!(!multi.is_empty());
    }

    #[test]
    fn test_deeply_nested() {
        let r = get_ranges("(defun fib [n]\n  (if (<= n 1)\n      1\n      (* n (fib (- n 1)))))");
        // Should have at least defun and if as foldable regions
        assert!(r.len() >= 2, "Deeply nested should have >=2 fold ranges");
    }
}
