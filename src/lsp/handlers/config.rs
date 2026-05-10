use crate::config::{CONFIG, LspConfig};
use crate::lsp::FORMAT_OPTS;
use anyhow::Result;
use lsp_server::{Connection, Notification};
use tracing::info;

pub fn handle_did_change_configuration(connection: &Connection, notif: Notification) -> Result<()> {
    let params = notif
        .extract::<lsp_types::DidChangeConfigurationParams>("workspace/didChangeConfiguration")
        .map_err(|e| anyhow::anyhow!("Failed to extract params: {:?}", e))?;

    info!("Configuration changed");

    let settings = params.settings;

    let wal_settings = settings
        .as_object()
        .and_then(|obj| obj.get("wal-lsp"))
        .or_else(|| {
            settings.as_object().and_then(|obj| obj.get("wal"))
        });

    let new_config = wal_settings
        .and_then(|v| serde_json::from_value::<LspConfig>(v.clone()).ok())
        .unwrap_or_default();

    {
        let mut config = CONFIG.write().unwrap_or_else(|e| e.into_inner());
        *config = new_config.clone();
    }

    if let Some(spaces) = new_config.format.indentation_spaces {
        let mut opts = FORMAT_OPTS.lock().unwrap_or_else(|e| e.into_inner());
        opts.indentation_spaces = spaces;
    }

    let docs_to_check: Vec<(lsp_types::Uri, String, i32)> = {
        let ws = crate::lsp::WORKSPACE.read().unwrap_or_else(|e| e.into_inner());
        ws.documents
            .iter()
            .filter_map(|(u, d)| {
                d.tree.as_ref().map(|_| (u.clone(), d.text.clone(), d.version))
            })
            .collect()
    };

    for (uri, text, version) in &docs_to_check {
        let tree = {
            let ws = crate::lsp::WORKSPACE.read().unwrap_or_else(|e| e.into_inner());
            ws.get_document(uri).and_then(|d| d.tree.clone())
        };
        let diagnostics = match tree {
            Some(t) => crate::lsp::handlers::diagnostics::analyze_document_from_tree(text, &t),
            None => continue,
        };
        let params = lsp_types::PublishDiagnosticsParams {
            uri: uri.clone(),
            diagnostics,
            version: Some(*version),
        };
        let notification =
            Notification::new("textDocument/publishDiagnostics".to_string(), params);
        if connection.sender.send(lsp_server::Message::Notification(notification)).is_err() {
            break;
        }
    }

    Ok(())
}
