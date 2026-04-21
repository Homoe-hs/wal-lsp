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
        let ws = WORKSPACE.read().unwrap();
        ws.get_word_at_position(uri, line, character)?
    };

    let ws = WORKSPACE.read().unwrap();
    let locations = ws.symbol_index.find(&word);

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
