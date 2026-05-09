use crate::lsp::WORKSPACE;
use crate::wal::docs::get_doc;
use anyhow::Result;
use lsp_server::{Connection, Request, Response};
use lsp_types::{
    ParameterInformation, SignatureHelp, SignatureHelpParams, SignatureInformation,
};
use tracing::info;

pub fn handle(connection: &Connection, req: Request) -> Result<()> {
    let params: SignatureHelpParams = serde_json::from_value(req.params)?;
    let uri = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    info!("Signature help requested for {:?} at {:?}", uri, position);

    let result = get_signature_help(&uri, position);

    let resp = Response::new_ok(req.id, result);
    connection
        .sender
        .send(lsp_server::Message::Response(resp))?;

    Ok(())
}

fn get_signature_help(uri: &lsp_types::Uri, position: lsp_types::Position) -> Option<SignatureHelp> {
    let ws = WORKSPACE.read().unwrap_or_else(|e| e.into_inner());
    let doc = ws.get_document(uri)?;
    let tree = doc.tree.as_ref()?;

    // Walk the AST to find the enclosing list at the given position
    let cursor_pos = (position.line as usize, position.character as usize);
    let fn_name = find_function_name_at(tree.root_node(), &doc.text, cursor_pos)?;

    let doc_entry = get_doc(&fn_name)?;

    let params: Vec<ParameterInformation> = doc_entry
        .signature
        .split_whitespace()
        .skip(1)
        .filter(|p| p.starts_with('[') || p.starts_with('<') || p.starts_with(|c: char| c.is_lowercase()))
        .map(|p| {
            let name = p.trim_end_matches(&[',', ')', ']', '>']).to_string();
            ParameterInformation {
                label: lsp_types::ParameterLabel::Simple(name),
                documentation: None,
            }
        })
        .collect();

    Some(SignatureHelp {
        signatures: vec![SignatureInformation {
            label: doc_entry.signature.clone(),
            documentation: Some(lsp_types::Documentation::String(
                doc_entry.description.clone(),
            )),
            parameters: if params.is_empty() { None } else { Some(params) },
            active_parameter: None,
        }],
        active_signature: Some(0),
        active_parameter: None,
    })
}

fn find_function_name_at(
    node: tree_sitter::Node,
    source: &str,
    pos: (usize, usize),
) -> Option<String> {
    if node.kind() == "list" {
        let start = node.start_position();
        let end = node.end_position();
        let line = pos.0;
        let col = pos.1;
        if line >= start.row as usize
            && line <= end.row as usize
            && (line > start.row as usize || col >= start.column as usize)
            && (line < end.row as usize || col <= end.column as usize)
        {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "sexpr_list" {
                    let mut sc = child.walk();
                    for sexpr_child in child.children(&mut sc) {
                        if sexpr_child.kind() == "sexpr" {
                            let mut fc = sexpr_child.walk();
                            return sexpr_child
                                .children(&mut fc)
                                .find(|a| a.kind() == "atom")
                                .and_then(|a| source.get(a.byte_range()).map(|s| s.trim().to_string()));
                        }
                    }
                }
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(result) = find_function_name_at(child, source, pos) {
            return Some(result);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_fn_name_simple() {
        use crate::wal::parser::WAL_PARSER;
        let mut parser = WAL_PARSER.lock().unwrap_or_else(|e| e.into_inner());
        let tree = parser.parse_incremental("(+ 1 2)", None);
        let result = find_function_name_at(tree.root_node(), "(+ 1 2)", (0, 2));
        assert_eq!(result, Some("+".to_string()));
    }

    #[test]
    fn test_find_fn_name_nested() {
        use crate::wal::parser::WAL_PARSER;
        let mut parser = WAL_PARSER.lock().unwrap_or_else(|e| e.into_inner());
        let tree = parser.parse_incremental("(if (> x 0) x y)", None);
        let result = find_function_name_at(tree.root_node(), "(if (> x 0) x y)", (0, 5));
        assert_eq!(result, Some("if".to_string()));
    }

    #[test]
    fn test_get_signature_for_known_fn() {
        let result = get_signature_from_name("if");
        assert!(result.is_some());
        let sig = result.unwrap();
        assert!(!sig.signatures.is_empty());
        assert!(sig.signatures[0].label.contains("if"));
    }

    #[test]
    fn test_get_signature_for_unknown_fn() {
        let result = get_signature_from_name("nonexistent-fn");
        assert!(result.is_none());
    }

    fn get_signature_from_name(name: &str) -> Option<SignatureHelp> {
        let doc_entry = get_doc(name)?;
        Some(SignatureHelp {
            signatures: vec![SignatureInformation {
                label: doc_entry.signature.clone(),
                documentation: Some(lsp_types::Documentation::String(
                    doc_entry.description.clone(),
                )),
                parameters: None,
                active_parameter: None,
            }],
            active_signature: Some(0),
            active_parameter: None,
        })
    }
}
