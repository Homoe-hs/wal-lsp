use crate::lsp::WORKSPACE;
use crate::wal::symbols::extract_symbols;
use anyhow::Result;
use lsp_server::{Connection, Request, Response};
use lsp_types::DocumentSymbolResponse;
use tracing::info;

pub fn handle(connection: &Connection, req: Request) -> Result<()> {
    let params: lsp_types::DocumentSymbolParams = serde_json::from_value(req.params)?;

    info!(
        "Document symbols requested for {:?}",
        params.text_document.uri
    );

    let document_text = {
        let ws = WORKSPACE.read().unwrap();
        ws.get_document(&params.text_document.uri).map(|d| d.text.clone())
    };

    let result: DocumentSymbolResponse = if let Some(text) = document_text {
        let symbols = extract_symbols(&text);
        let doc_symbols: Vec<lsp_types::DocumentSymbol> = symbols
            .into_iter()
            .map(|s| s.to_document_symbol())
            .collect();
        DocumentSymbolResponse::Nested(doc_symbols)
    } else {
        DocumentSymbolResponse::Nested(vec![])
    };

    let resp = Response::new_ok(req.id, result);
    connection
        .sender
        .send(lsp_server::Message::Response(resp))?;

    Ok(())
}
