use crate::lsp::WORKSPACE;
use crate::wal::parser::WalParser;
use anyhow::Result;
use lsp_server::{Connection, Notification};
use lsp_types::{Diagnostic, DiagnosticSeverity, PublishDiagnosticsParams};
use tracing::info;

pub fn handle_did_open(connection: &Connection, notif: Notification) -> Result<()> {
    let params = notif
        .extract::<lsp_types::DidOpenTextDocumentParams>("textDocument/didOpen")
        .map_err(|e| anyhow::anyhow!("Failed to extract params: {:?}", e))?;

    let uri = params.text_document.uri.clone();
    let text = params.text_document.text.clone();

    info!("Document opened: {:?}", uri);

    {
        let mut ws = WORKSPACE.write().unwrap();
        ws.open_document(uri.clone(), text.clone());
    }

    let diagnostics = analyze_document(&text);

    let params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version: None,
    };

    let notification = Notification::new("textDocument/publishDiagnostics".to_string(), params);
    connection
        .sender
        .send(lsp_server::Message::Notification(notification))?;

    Ok(())
}

pub fn handle_did_change(connection: &Connection, notif: Notification) -> Result<()> {
    let params = notif
        .extract::<lsp_types::DidChangeTextDocumentParams>("textDocument/didChange")
        .map_err(|e| anyhow::anyhow!("Failed to extract params: {:?}", e))?;

    let uri = params.text_document.uri.clone();
    let text = params
        .content_changes
        .get(0)
        .map(|c| c.text.clone())
        .unwrap_or_default();

    info!("Document changed: {:?}", uri);

    {
        let mut ws = WORKSPACE.write().unwrap();
        ws.update_document(&uri, text.clone());
    }

    let diagnostics = analyze_document(&text);

    let params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version: None,
    };

    let notification = Notification::new("textDocument/publishDiagnostics".to_string(), params);
    connection
        .sender
        .send(lsp_server::Message::Notification(notification))?;

    Ok(())
}

fn analyze_document(text: &str) -> Vec<Diagnostic> {
    let mut parser = WalParser::new();
    let tree = parser.parse_with_errors(text);

    let mut diagnostics = Vec::new();

    if tree.root_node().has_error() {
        let mut cursor = tree.walk();
        collect_errors(&mut cursor, text, &mut diagnostics);
    }

    diagnostics
}

fn collect_errors(
    cursor: &mut tree_sitter::TreeCursor,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let node = cursor.node();

    if node.kind() == "ERROR" {
        let start = node.start_position();
        let end = node.end_position();

        let range = lsp_types::Range::new(
            lsp_types::Position::new(start.row as u32, start.column as u32),
            lsp_types::Position::new(end.row as u32, end.column as u32),
        );

        let error_node_text = source.get(node.byte_range()).unwrap_or("");

        diagnostics.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            message: format!("Syntax error: {}", error_node_text),
            ..Default::default()
        });
    }

    if cursor.goto_first_child() {
        loop {
            collect_errors(cursor, source, diagnostics);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}
