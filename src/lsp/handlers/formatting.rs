use crate::lsp::WORKSPACE;
use crate::wal::format::format_document_with_opts;
use anyhow::Result;
use lsp_server::{Connection, Request, Response};
use lsp_types::{DocumentFormattingParams, Position, Range, TextEdit};

pub fn handle(connection: &Connection, req: Request) -> Result<()> {
    let params: DocumentFormattingParams = serde_json::from_value(req.params)?;
    let uri = &params.text_document.uri;

    let text = {
        let ws = WORKSPACE.read().unwrap_or_else(|e| e.into_inner());
        ws.get_document(uri)
            .map(|d| d.text.clone())
            .unwrap_or_default()
    };

    if text.is_empty() {
        let resp = Response::new_ok(req.id, serde_json::Value::Null);
        connection.sender.send(lsp_server::Message::Response(resp))?;
        return Ok(());
    }

    let opts = crate::lsp::FORMAT_OPTS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let formatted = format_document_with_opts(&text, &opts);

    let edit = TextEdit {
        range: Range::new(
            Position::new(0, 0),
            Position::new(text.lines().count() as u32, 0),
        ),
        new_text: formatted,
    };

    let resp = Response::new_ok(req.id, vec![edit]);
    connection.sender.send(lsp_server::Message::Response(resp))?;

    Ok(())
}
