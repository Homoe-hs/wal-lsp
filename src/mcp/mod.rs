mod tools;

use anyhow::Result;
use serde_json::Value;
use std::io::{BufRead, Write};
use tracing::info;

pub fn run() -> Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();

    let mut writer = stdout.lock();
    let mut reader = BufRead::lines(stdin.lock());

    while let Some(Ok(line)) = reader.next() {
        info!("Received: {}", line);

        let request: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                info!("Failed to parse JSON: {}", e);
                continue;
            }
        };

        let response = handle_mcp_request(&request);

        if let Some(resp) = response {
            let resp_json = serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string());
            writeln!(writer, "{}", resp_json)?;
            writer.flush()?;
        }
    }

    Ok(())
}

fn handle_mcp_request(request: &Value) -> Option<Value> {
    let method = request.get("method")?.as_str()?;
    let id = request.get("id").cloned();

    info!("MCP method: {}", method);

    match method {
        "tools/list" => Some(tools::list_tools()),
        "tools/call" => {
            let params = request.get("params")?;
            let result = tools::call_tool(params);
            Some(serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result
            }))
        }
        "initialize" => Some(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "wal-lsp",
                    "version": "0.1.0"
                }
            }
        })),
        "notifications/initialized" => None,
        _ => {
            info!("Unknown MCP method: {}", method);
            None
        }
    }
}
