mod handlers;

use crate::workspace::{create_workspace, SharedWorkspace};
use anyhow::Result;
use lsp_server::{Connection, Message, Notification, Request, Response};
use serde_json::to_value;
use tracing::{error, info};

pub static WORKSPACE: std::sync::LazyLock<SharedWorkspace> =
    std::sync::LazyLock::new(create_workspace);

/// 全局格式化选项 (可通过 CLI 或 LSP 初始化参数配置)
pub static FORMAT_OPTS: std::sync::LazyLock<std::sync::Mutex<crate::wal::format::FormatOptions>> =
    std::sync::LazyLock::new(|| {
        std::sync::Mutex::new(crate::wal::format::FormatOptions::default())
    });

pub fn run() -> Result<()> {
    let (connection, _io_threads) = Connection::stdio();

    info!("WAL LSP server starting...");

    let (id, init_params) = match connection.initialize_start() {
        Ok(v) => v,
        Err(e) => {
            error!("Initialization failed: {}. Ensure correct LSP Content-Length header.", e);
            return Err(anyhow::anyhow!("Failed to initialize: {}", e));
        }
    };

    info!("Init params: {:?}", init_params);

    let server_capabilities = to_value(lsp_types::ServerCapabilities {
        text_document_sync: Some(lsp_types::TextDocumentSyncCapability::Kind(
            lsp_types::TextDocumentSyncKind::FULL,
        )),
        completion_provider: Some(lsp_types::CompletionOptions {
            resolve_provider: Some(true),
            trigger_characters: Some(vec![
                "(".to_string(),
                " ".to_string(),
                "~".to_string(),
                "#".to_string(),
            ]),
            ..Default::default()
        }),
        hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
        definition_provider: Some(lsp_types::OneOf::Left(true)),
        references_provider: Some(lsp_types::OneOf::Left(true)),
        document_symbol_provider: Some(lsp_types::OneOf::Left(true)),
        workspace_symbol_provider: Some(lsp_types::OneOf::Left(true)),
        document_formatting_provider: Some(lsp_types::OneOf::Left(true)),
        ..Default::default()
    })
    .map_err(|e| anyhow::anyhow!("Failed to serialize capabilities: {}", e))?;

    connection
        .initialize_finish(id, server_capabilities)
        .map_err(|e| anyhow::anyhow!("Failed to finish initialization: {}", e))?;

    info!("LSP server initialized");

    loop {
        let msg = match connection.receiver.recv() {
            Ok(m) => m,
            Err(e) => {
                error!("Receiver error: {}", e);
                break;
            }
        };
        match msg {
            Message::Request(req) => {
                info!("Received request: {:?}", req.method);
                if let Err(e) = handle_request(&connection, req) {
                    error!("Error handling request: {}", e);
                }
            }
            Message::Notification(notif) => {
                info!("Received notification: {:?}", notif.method);
                if notif.method == "exit" {
                    break;
                }
                if let Err(e) = handle_notification(&connection, notif) {
                    error!("Error handling notification: {}", e);
                }
            }
            Message::Response(_) => {}
        }
    }

    Ok(())
}

fn handle_request(connection: &Connection, req: Request) -> Result<()> {
    let id = req.id.clone();
    let result = match req.method.as_str() {
        "textDocument/completion" => handlers::completion::handle(connection, req),
        "completionItem/resolve" => handlers::completion_resolve::handle(connection, req),
        "textDocument/hover" => handlers::hover::handle(connection, req),
        "textDocument/definition" => handlers::goto::handle(connection, req),
        "textDocument/references" => handlers::references::handle(connection, req),
        "textDocument/documentSymbol" => handlers::symbols::handle(connection, req),
        "workspace/symbol" => handlers::workspace_symbol::handle(connection, req),
        "textDocument/formatting" => handlers::formatting::handle(connection, req),
        "shutdown" => {
            info!("Received shutdown request");
            let resp = Response::new_ok(req.id, serde_json::Value::Null);
            Ok(connection.sender.send(Message::Response(resp))?)
        }
        _ => {
            info!("Unhandled request method: {}", req.method);
            Err(anyhow::anyhow!("Method not found: {}", req.method))
        }
    };
    if let Err(e) = result {
        let resp = Response::new_err(id, -32601, format!("{}", e));
        connection.sender.send(Message::Response(resp))?;
    }
    Ok(())
}

fn handle_notification(connection: &Connection, notif: Notification) -> Result<()> {
    match notif.method.as_str() {
        "textDocument/didOpen" => handlers::diagnostics::handle_did_open(connection, notif),
        "textDocument/didChange" => handlers::diagnostics::handle_did_change(connection, notif),
        "textDocument/didClose" => handlers::diagnostics::handle_did_close(connection, notif),
        "workspace/didChangeConfiguration" => {
            handlers::config::handle_did_change_configuration(connection, notif)
        }
        _ => {
            info!("Unhandled notification: {}", notif.method);
            Ok(())
        }
    }
}
