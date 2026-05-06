use crate::lsp::WORKSPACE;
use anyhow::Result;
use lsp_server::{Connection, Request, Response};
use lsp_types::{GotoDefinitionResponse, Location, TextDocumentPositionParams};
use tracing::info;

pub fn handle(connection: &Connection, req: Request) -> Result<()> {
    let params: TextDocumentPositionParams = serde_json::from_value(req.params)?;

    info!("Goto definition requested for {:?}", params.position);

    let result = find_definition(&params);

    let resp = Response::new_ok(req.id, result);
    connection
        .sender
        .send(lsp_server::Message::Response(resp))?;

    Ok(())
}

fn find_definition(params: &TextDocumentPositionParams) -> Option<GotoDefinitionResponse> {
    let uri = &params.text_document.uri;
    let line = params.position.line;
    let character = params.position.character;

    let word = {
        let ws = WORKSPACE.read().unwrap_or_else(|e| e.into_inner());
        ws.get_word_at_position(uri, line, character)?
    };

    goto_symbol(&word)
}

/// Resolve goto definition for a symbol name (no workspace position dependency)
pub fn goto_symbol(name: &str) -> Option<GotoDefinitionResponse> {
    let ws = WORKSPACE.read().unwrap_or_else(|e| e.into_inner());
    let locations = ws.symbol_index.find(name);

    if locations.is_empty() {
        return None;
    }

    let lsp_locations: Vec<Location> = locations
        .into_iter()
        .map(|loc| Location {
            uri: loc.uri,
            range: loc.range,
        })
        .collect();

    Some(GotoDefinitionResponse::Array(lsp_locations))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lsp::WORKSPACE;
    use std::str::FromStr;

    fn setup_workspace_with_symbols() {
        let mut ws = WORKSPACE.write().unwrap_or_else(|e| e.into_inner());
        // Clear previous state from other tests
        ws.documents.clear();
        ws.symbol_index = crate::workspace::SymbolIndex::new();

        let uri = lsp_types::Uri::from_str("file:///test-goto.wal").unwrap();
        let source = "(define target-var 42)\n(defun target-fn [x] (* x 2))";
        ws.open_document(uri.clone(), source.to_string());
    }

    #[test]
    fn test_goto_defined_variable() {
        setup_workspace_with_symbols();
        let result = goto_symbol("target-var");
        assert!(result.is_some(), "Should find 'target-var'");
        match result.unwrap() {
            GotoDefinitionResponse::Array(locations) => {
                assert_eq!(locations.len(), 1);
            }
            _ => panic!("Expected Array response"),
        }
    }

    #[test]
    fn test_goto_defined_function() {
        setup_workspace_with_symbols();
        let result = goto_symbol("target-fn");
        assert!(result.is_some(), "Should find 'target-fn'");
    }

    #[test]
    fn test_goto_undefined_symbol() {
        setup_workspace_with_symbols();
        let result = goto_symbol("nonexistent-symbol");
        assert!(result.is_none(), "Should not find undefined symbol");
    }

    #[test]
    fn test_goto_finds_correct_uri() {
        setup_workspace_with_symbols();
        let result = goto_symbol("target-var").expect("Should find");
        match result {
            GotoDefinitionResponse::Array(locations) => {
                assert_eq!(locations.len(), 1);
                assert!(locations[0].uri.to_string().contains("test-goto.wal"));
            }
            _ => panic!("Expected Array"),
        }
    }
}
