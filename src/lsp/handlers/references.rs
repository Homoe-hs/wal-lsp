use crate::lsp::WORKSPACE;
use anyhow::Result;
use lsp_server::{Connection, Request, Response};
use lsp_types::{Location, ReferenceParams};
use tracing::info;

pub fn handle(connection: &Connection, req: Request) -> Result<()> {
    let params: ReferenceParams = serde_json::from_value(req.params)?;

    info!("References requested for {:?}", params);

    let result = find_references(&params);

    let resp = Response::new_ok(req.id, result);
    connection
        .sender
        .send(lsp_server::Message::Response(resp))?;

    Ok(())
}

fn find_references(params: &ReferenceParams) -> Vec<Location> {
    let uri = &params.text_document_position.text_document.uri;
    let line = params.text_document_position.position.line;
    let character = params.text_document_position.position.character;

    let word = {
        let ws = WORKSPACE.read().unwrap_or_else(|e| e.into_inner());
        match ws.get_word_at_position(uri, line, character) {
            Some(w) => w,
            None => return vec![],
        }
    };

    let ws = WORKSPACE.read().unwrap_or_else(|e| e.into_inner());
    let locations = ws.symbol_index.find(&word);
    locations
        .into_iter()
        .map(|loc| Location {
            uri: loc.uri,
            range: loc.range,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lsp::WORKSPACE;
    use std::str::FromStr;

    fn setup() {
        let mut ws = WORKSPACE.write().unwrap_or_else(|e| e.into_inner());
        ws.documents.clear();
        ws.symbol_index = crate::workspace::SymbolIndex::new();

        let uri = lsp_types::Uri::from_str("file:///test-refs.wal").unwrap();
        ws.open_document(uri, "(define target 42)".to_string());
    }

    #[test]
    fn test_find_references_known_symbol() {
        setup();
        let uri = lsp_types::Uri::from_str("file:///test-refs.wal").unwrap();
        let params = ReferenceParams {
            text_document_position: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri: uri.clone() },
                position: lsp_types::Position::new(0, 8),
            },
            context: lsp_types::ReferenceContext { include_declaration: true },
            work_done_progress_params: lsp_types::WorkDoneProgressParams::default(),
            partial_result_params: lsp_types::PartialResultParams::default(),
        };
        let result = find_references(&params);
        assert!(!result.is_empty());
        assert!(result.iter().any(|l| l.uri == uri));
    }

    #[test]
    fn test_find_references_unknown_symbol() {
        setup();
        let uri = lsp_types::Uri::from_str("file:///test-refs.wal").unwrap();
        let params = ReferenceParams {
            text_document_position: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri },
                position: lsp_types::Position::new(0, 1),
            },
            context: lsp_types::ReferenceContext { include_declaration: true },
            work_done_progress_params: lsp_types::WorkDoneProgressParams::default(),
            partial_result_params: lsp_types::PartialResultParams::default(),
        };
        let result = find_references(&params);
        assert!(result.is_empty());
    }
}
