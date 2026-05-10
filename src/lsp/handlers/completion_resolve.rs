use crate::wal::completions::get_all_completions_ref;
use crate::wal::docs::get_doc;
use anyhow::Result;
use lsp_server::{Connection, Request, Response};
use lsp_types::CompletionItem;

pub fn handle(connection: &Connection, req: Request) -> Result<()> {
    let item: CompletionItem = serde_json::from_value(req.params)?;
    let resolved = resolve_completion_item(&item);

    let resp = Response::new_ok(req.id, resolved);
    connection.sender.send(lsp_server::Message::Response(resp))?;
    Ok(())
}

fn resolve_completion_item(item: &CompletionItem) -> CompletionItem {
    if item.documentation.is_some() {
        return item.clone();
    }

    let label = &item.label;
    let doc = get_doc(label);
    if let Some(doc) = doc {
        return CompletionItem {
            documentation: Some(lsp_types::Documentation::String(format!(
                "**{}**\n\n{}\n\n```wal\n{}\n```",
                doc.name, doc.description, doc.signature
            ))),
            ..item.clone()
        };
    }

    if let Some(detail) = &item.detail {
        return CompletionItem {
            documentation: Some(lsp_types::Documentation::String(detail.clone())),
            ..item.clone()
        };
    }

    for builtin in get_all_completions_ref() {
        if builtin.label == *label {
            if let Some(detail) = &builtin.detail {
                return CompletionItem {
                    documentation: Some(lsp_types::Documentation::String(detail.clone())),
                    ..item.clone()
                };
            }
        }
    }

    item.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_adds_doc_for_known_symbol() {
        let item = CompletionItem {
            label: "+".to_string(),
            detail: Some("addition".to_string()),
            documentation: None,
            ..Default::default()
        };
        let resolved = resolve_completion_item(&item);
        assert!(resolved.documentation.is_some());
    }

    #[test]
    fn test_resolve_skips_when_already_has_doc() {
        let item = CompletionItem {
            label: "+".to_string(),
            detail: Some("addition".to_string()),
            documentation: Some(lsp_types::Documentation::String("existing".to_string())),
            ..Default::default()
        };
        let resolved = resolve_completion_item(&item);
        assert_eq!(
            resolved.documentation,
            Some(lsp_types::Documentation::String("existing".to_string()))
        );
    }

    #[test]
    fn test_resolve_falls_back_to_detail() {
        let item = CompletionItem {
            label: "zzz-nonexistent".to_string(),
            detail: Some("fallback detail".to_string()),
            documentation: None,
            ..Default::default()
        };
        let resolved = resolve_completion_item(&item);
        assert!(resolved.documentation.is_some());
    }

    #[test]
    fn test_resolve_unknown_symbol_returns_same() {
        let item = CompletionItem {
            label: "totally-unknown-symbol".to_string(),
            detail: None,
            documentation: None,
            ..Default::default()
        };
        let resolved = resolve_completion_item(&item);
        assert!(resolved.documentation.is_none());
    }
}
