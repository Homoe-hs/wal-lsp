use crate::lsp::WORKSPACE;
use anyhow::Result;
use lsp_server::{Connection, Request, Response};
use lsp_types::{Location, WorkspaceSymbol, WorkspaceSymbolParams, WorkspaceSymbolResponse};
use tracing::info;

pub fn handle(connection: &Connection, req: Request) -> Result<()> {
    let params: WorkspaceSymbolParams = serde_json::from_value(req.params)?;

    info!("Workspace symbol requested: {:?}", params.query);

    let result = find_workspace_symbols(&params.query);

    let resp = Response::new_ok(req.id, result);
    connection
        .sender
        .send(lsp_server::Message::Response(resp))?;

    Ok(())
}

fn find_workspace_symbols(query: &str) -> WorkspaceSymbolResponse {
    let ws = WORKSPACE.read().unwrap_or_else(|e| e.into_inner());
    let query_lower = query.to_lowercase();

    let mut symbols: Vec<WorkspaceSymbol> = Vec::new();
    for (name, locations) in &ws.symbol_index.by_name {
        if !query.is_empty()
            && !name.to_lowercase().contains(&query_lower)
        {
            continue;
        }
        for loc in locations {
            symbols.push(WorkspaceSymbol {
                name: name.clone(),
                kind: loc.kind,
                tags: None,
                container_name: None,
                location: lsp_types::OneOf::Left(Location {
                    uri: loc.uri.clone(),
                    range: loc.range,
                }),
                data: None,
            });
        }
    }

    symbols.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    symbols.dedup_by(|a, b| a.name.to_lowercase() == b.name.to_lowercase() && a.location == b.location);

    WorkspaceSymbolResponse::Nested(symbols)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lsp::WORKSPACE;
    use std::str::FromStr;

    fn setup() {
        let mut ws = WORKSPACE.write().unwrap_or_else(|e| e.into_inner());
        ws.documents.clear();
        ws.symbol_index = crate::workspace::SymbolIndex::new();

        let u1 = lsp_types::Uri::from_str("file:///a.wal").unwrap();
        ws.open_document(u1, "(define foo 1)".to_string());
        let u2 = lsp_types::Uri::from_str("file:///b.wal").unwrap();
        ws.open_document(u2, "(define bar 2)".to_string());
    }

    fn unwrap_nested(response: WorkspaceSymbolResponse) -> Vec<WorkspaceSymbol> {
        match response {
            WorkspaceSymbolResponse::Nested(s) => s,
            _ => panic!("Expected Nested variant"),
        }
    }

    #[test]
    fn test_find_all() {
        setup();
        let result = unwrap_nested(find_workspace_symbols(""));
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_find_by_query() {
        setup();
        let result = unwrap_nested(find_workspace_symbols("foo"));
        assert_eq!(result.len(), 1);
        assert!(result.iter().any(|s| s.name == "foo"));
    }

    #[test]
    fn test_find_no_match() {
        setup();
        let result = unwrap_nested(find_workspace_symbols("zzz"));
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_case_insensitive() {
        setup();
        let result = unwrap_nested(find_workspace_symbols("FOO"));
        assert_eq!(result.len(), 1);
    }
}
