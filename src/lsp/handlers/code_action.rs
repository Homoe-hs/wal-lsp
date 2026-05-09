use crate::lsp::WORKSPACE;
use anyhow::Result;
use lsp_server::{Connection, Request, Response};
use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, TextEdit,
};
use std::collections::HashMap;
use tracing::info;

pub fn handle(connection: &Connection, req: Request) -> Result<()> {
    let params: CodeActionParams = serde_json::from_value(req.params)?;
    let uri = params.text_document.uri.clone();
    let range = params.range;

    info!("Code action requested for {:?} at {:?}", uri, range);

    let actions = get_code_actions(&uri, &params.context);

    let resp = Response::new_ok(req.id, actions);
    connection
        .sender
        .send(lsp_server::Message::Response(resp))?;

    Ok(())
}

fn get_code_actions(
    uri: &lsp_types::Uri,
    context: &lsp_types::CodeActionContext,
) -> Vec<CodeActionOrCommand> {
    let _ws = WORKSPACE.read().unwrap_or_else(|e| e.into_inner());
    let mut actions = Vec::new();

    for diag in &context.diagnostics {
        let rule_id = diag
            .source
            .as_deref()
            .and_then(|s| s.strip_prefix("wal-lsp:"));

        if let Some(rule) = rule_id {
            let line = diag.range.start.line;
            let title = format!("disable {}", rule);

            let edit = TextEdit {
                range: lsp_types::Range::new(
                    lsp_types::Position::new(line, 0),
                    lsp_types::Position::new(line, 0),
                ),
                new_text: format!(";; lint_off {}\n", rule),
            };

            let mut changes = HashMap::new();
            changes.insert(uri.clone(), vec![edit]);

            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title,
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: Some(vec![diag.clone()]),
                edit: Some(lsp_types::WorkspaceEdit {
                    changes: Some(changes),
                    document_changes: None,
                    change_annotations: None,
                }),
                ..Default::default()
            }));
        }
    }

    actions
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{
        CodeActionContext, Diagnostic, DiagnosticSeverity, Position, Range,
    };
    use std::str::FromStr;

    fn make_diag(line: u32, rule: &str) -> Diagnostic {
        Diagnostic {
            range: Range::new(Position::new(line, 0), Position::new(line, 5)),
            severity: Some(DiagnosticSeverity::WARNING),
            source: Some(format!("wal-lsp:{}", rule).into()),
            message: format!("test message for {}", rule),
            ..Default::default()
        }
    }

    #[test]
    fn test_code_action_for_unknown_symbol() {
        let uri = lsp_types::Uri::from_str("file:///test.wal").unwrap();
        let context = CodeActionContext {
            diagnostics: vec![make_diag(0, "unknown-symbol")],
            only: None,
            trigger_kind: None,
        };
        let actions = get_code_actions(&uri, &context);
        assert!(!actions.is_empty());
        let has_disable = actions.iter().any(|a| match a {
            CodeActionOrCommand::CodeAction(ca) => ca.title.contains("unknown-symbol"),
            _ => false,
        });
        assert!(has_disable, "Should have disable rule action");
    }

    #[test]
    fn test_code_action_no_diags() {
        let uri = lsp_types::Uri::from_str("file:///test.wal").unwrap();
        let context = CodeActionContext {
            diagnostics: vec![],
            only: None,
            trigger_kind: None,
        };
        let actions = get_code_actions(&uri, &context);
        assert!(actions.is_empty());
    }
}
