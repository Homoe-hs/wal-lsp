use crate::lsp::WORKSPACE;
use anyhow::Result;
use lsp_server::{Connection, Request, Response};
use lsp_types::{DocumentHighlight, DocumentHighlightKind, DocumentHighlightParams};
use tracing::info;

pub fn handle(connection: &Connection, req: Request) -> Result<()> {
    let params: DocumentHighlightParams = serde_json::from_value(req.params)?;
    let uri = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    info!("Document highlight requested for {:?} at {:?}", uri, position);

    let highlights = find_highlights(&uri, position);

    let resp = Response::new_ok(req.id, highlights);
    connection
        .sender
        .send(lsp_server::Message::Response(resp))?;

    Ok(())
}

fn find_highlights(uri: &lsp_types::Uri, position: lsp_types::Position) -> Vec<DocumentHighlight> {
    let ws = WORKSPACE.read().unwrap_or_else(|e| e.into_inner());
    let doc = match ws.get_document(uri) {
        Some(d) => d,
        None => return vec![],
    };

    let word = match ws.get_word_at_position(uri, position.line, position.character) {
        Some(w) => w,
        None => return vec![],
    };

    if word.is_empty() {
        return vec![];
    }

    let mut highlights = Vec::new();

    for (line_idx, line) in doc.text.lines().enumerate() {
        let mut col = 0;
        while col < line.len() {
            if let Some((end, _)) = find_word_at(line, col, &word) {
                highlights.push(DocumentHighlight {
                    range: lsp_types::Range::new(
                        lsp_types::Position::new(line_idx as u32, col as u32),
                        lsp_types::Position::new(line_idx as u32, end as u32),
                    ),
                    kind: Some(DocumentHighlightKind::READ),
                });
                col = end;
            } else {
                break;
            }
        }
    }

    highlights
}

fn find_word_at(line: &str, start: usize, word: &str) -> Option<(usize, bool)> {
    let mut current_start = start;
    loop {
        if current_start >= line.len() {
            return None;
        }
        let line_rest = &line[current_start..];
        let byte_pos = match line_rest.find(word) {
            Some(p) => p,
            None => return None,
        };
        let actual_start = current_start + byte_pos;
        let actual_end = actual_start + word.len();

        if actual_end > line.len() {
            return None;
        }

        let before_ok = actual_start == 0
            || !line[..actual_start]
                .chars()
                .last()
                .map_or(false, |c| crate::workspace::is_wal_word_char(c));

        let after_ok = actual_end >= line.len()
            || !line[actual_end..]
                .chars()
                .next()
                .map_or(false, |c| crate::workspace::is_wal_word_char(c));

        if before_ok && after_ok {
            return Some((actual_end, true));
        }

        let ch = line[actual_start..].chars().next().unwrap_or(' ');
        current_start = actual_start + ch.len_utf8();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_word_at_simple() {
        assert_eq!(find_word_at("(define x 42)", 0, "define"), Some((7, true)));
    }

    #[test]
    fn test_find_word_at_no_match() {
        assert_eq!(find_word_at("(+ 1 2)", 0, "xxx"), None);
    }

    #[test]
    fn test_find_word_at_not_subword() {
        // "define" inside "undefined" should not match
        assert_eq!(find_word_at("undefined", 0, "define"), None);
    }

    #[test]
    fn test_find_word_at_after_initial() {
        let result = find_word_at("(define x (define y 2))", 8, "define");
        assert!(result.is_some());
    }

    #[test]
    fn test_find_word_boundary_hyphen() {
        let result = find_word_at("(eval-file x)", 0, "eval-file");
        assert_eq!(result, Some((10, true)));
    }
}
