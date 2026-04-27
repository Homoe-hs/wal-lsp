use crate::lsp::WORKSPACE;
use crate::wal::format::format_document;
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

    let formatted = format_document(&text);

    let line_count = text.lines().count() as u32;
    let last_line_len = text.lines().last().map(|l| l.len() as u32).unwrap_or(0);

    let edit = TextEdit {
        range: Range::new(
            Position::new(0, 0),
            Position::new(line_count.saturating_sub(1), last_line_len),
        ),
        new_text: formatted,
    };

    let resp = Response::new_ok(req.id, vec![edit]);
    connection.sender.send(lsp_server::Message::Response(resp))?;

    Ok(())
}
