use crate::lsp::WORKSPACE;
use crate::wal::completions::{BUILTIN_FUNCTIONS, MACROS, OPERATORS, SPECIAL_FORMS};
use crate::wal::docs::get_doc;
use anyhow::Result;
use lsp_server::{Connection, Request, Response};
use lsp_types::{Hover, HoverContents, MarkedString};

pub fn handle(connection: &Connection, req: Request) -> Result<()> {
    let params: lsp_types::TextDocumentPositionParams = serde_json::from_value(req.params)?;

    let result = get_hover_info(&params);

    let resp = if let Some(hover) = result {
        Response::new_ok(req.id, hover)
    } else {
        Response::new_ok(req.id, serde_json::Value::Null)
    };

    connection
        .sender
        .send(lsp_server::Message::Response(resp))?;

    Ok(())
}

fn get_hover_info(params: &lsp_types::TextDocumentPositionParams) -> Option<Hover> {
    let uri = &params.text_document.uri;
    let position = params.position;

    let word = {
        let ws = WORKSPACE.read().unwrap_or_else(|e| e.into_inner());
        ws.get_word_at_position(uri, position.line, position.character)?
    };

    if let Some(doc) = get_doc(&word) {
        return Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(format!(
                "**{}**\n\n{}\n\n```wal\n{}\n```",
                doc.name, doc.description, doc.signature
            ))),
            range: None,
        });
    }

    for (name, detail) in BUILTIN_FUNCTIONS
        .iter()
        .chain(SPECIAL_FORMS.iter())
        .chain(MACROS.iter())
        .chain(OPERATORS.iter()) {
        if *name == word {
            return Some(Hover {
                contents: HoverContents::Scalar(MarkedString::String(format!(
                    "**{}**\n\n{}",
                    name, detail
                ))),
                range: None,
            });
        }
    }

    None
}
