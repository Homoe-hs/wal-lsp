use crate::lsp::WORKSPACE;
use crate::wal::completions::{get_all_completions, CompletionKind as WalCompletionKind};
use anyhow::Result;
use lsp_server::{Connection, Request, Response};
use lsp_types::{CompletionItem, CompletionList, CompletionParams, CompletionResponse};
use tracing::info;

pub fn handle(connection: &Connection, req: Request) -> Result<()> {
    let params: CompletionParams = serde_json::from_value(req.params)?;
    let uri = params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;

    info!("Completion requested for {:?} at {:?}", uri, position);

    let prefix = {
        let ws = WORKSPACE.read().unwrap_or_else(|e| e.into_inner());
        ws.get_document(&uri)
            .map(|doc| {
                let lines: Vec<&str> = doc.text.lines().collect();
                if let Some(line) = lines.get(position.line as usize) {
                    extract_prefix(line, position.character as usize)
                } else {
                    String::new()
                }
            })
            .unwrap_or_default()
    };

    let mut items: Vec<CompletionItem> = get_all_completions()
        .into_iter()
        .filter(|c| prefix.is_empty() || c.label.starts_with(&prefix))
        .map(|c| CompletionItem {
            label: c.label,
            kind: Some(match c.kind {
                WalCompletionKind::Keyword => lsp_types::CompletionItemKind::KEYWORD,
                WalCompletionKind::Function => lsp_types::CompletionItemKind::FUNCTION,
                WalCompletionKind::Operator => lsp_types::CompletionItemKind::OPERATOR,
                WalCompletionKind::Variable => lsp_types::CompletionItemKind::VARIABLE,
                WalCompletionKind::Signal => lsp_types::CompletionItemKind::VARIABLE,
            }),
            detail: c.detail,
            documentation: c.documentation.map(|d| lsp_types::Documentation::String(d)),
            ..Default::default()
        })
        .collect();

    if let Some(signal_items) = get_signal_completions(&uri, position) {
        items.extend(signal_items);
    }

    let result = CompletionResponse::List(CompletionList {
        items,
        is_incomplete: false,
    });

    let resp = Response::new_ok(req.id, result);
    connection
        .sender
        .send(lsp_server::Message::Response(resp))?;

    Ok(())
}

fn get_signal_completions(uri: &lsp_types::Uri, position: lsp_types::Position) -> Option<Vec<CompletionItem>> {
    let ws = WORKSPACE.read().unwrap_or_else(|e| e.into_inner());
    let doc = ws.get_document(uri)?;

    let lines: Vec<&str> = doc.text.lines().collect();
    let line_str = lines.get(position.line as usize)?;

    let cursor_col = position.character as usize;
    if cursor_col > line_str.len() {
        return None;
    }

    let prefix = extract_signal_prefix(line_str, cursor_col);
    if prefix.is_empty() {
        return None;
    }

    let signals = ws.waveform_manager.find(&prefix);
    if signals.is_empty() {
        return None;
    }

    Some(
        signals
            .into_iter()
            .map(|name| CompletionItem {
                label: name,
                kind: Some(lsp_types::CompletionItemKind::VARIABLE),
                detail: Some("signal".to_string()),
                documentation: None,
                ..Default::default()
            })
            .collect(),
    )
}

fn extract_prefix(line: &str, cursor_col: usize) -> String {
    let before = &line[..cursor_col.min(line.len())];
    let mut end = before.len();
    while end > 0 {
        let ch = before[..end].chars().last().unwrap();
        if ch.is_alphanumeric() || "+-*/=!><.%?_|&^~#".contains(ch) {
            end -= ch.len_utf8();
        } else {
            break;
        }
    }
    let prefix = before[end..].to_string();
    // Don't filter when prefix is purely operator characters (e.g., just typed "+")
    let has_alpha = prefix.chars().any(|c| c.is_alphanumeric());
    if !has_alpha && !prefix.is_empty() {
        String::new()
    } else {
        prefix
    }
}

fn extract_signal_prefix(line: &str, cursor_pos: usize) -> String {
    let before_cursor = &line[..cursor_pos.min(line.len())];

    let mut end = before_cursor.len();
    while end > 0 {
        let ch = before_cursor[..end].chars().last().unwrap();
        if ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == '.' || ch == '/' {
            end -= 1;
        } else {
            break;
        }
    }

    before_cursor[end..].to_string()
}
