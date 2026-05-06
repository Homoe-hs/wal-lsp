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

    get_hover_for_word(&word)
}

/// Resolve hover info for a known word (no workspace dependency)
pub fn get_hover_for_word(word: &str) -> Option<Hover> {
    if word.is_empty() {
        return None;
    }

    if let Some(doc) = get_doc(word) {
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

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Rich docs (get_doc) ----

    #[test]
    fn test_hover_rich_doc_for_plus() {
        let hover = get_hover_for_word("+").expect("+ should have hover");
        let text = hover_text(&hover);
        assert!(text.contains("**+**"), "Should contain bold name");
        assert!(text.contains("Addition"), "Should contain description");
        assert!(text.contains("(+ expr*)"), "Should contain signature");
    }

    #[test]
    fn test_hover_rich_doc_for_defun() {
        let hover = get_hover_for_word("defun").expect("defun should have hover");
        let text = hover_text(&hover);
        assert!(text.contains("**defun**"));
        assert!(text.contains("(defun name args body+)"));
    }

    #[test]
    fn test_hover_rich_doc_for_map() {
        let hover = get_hover_for_word("map").expect("map should have hover");
        let text = hover_text(&hover);
        assert!(text.contains("**map**"));
        assert!(text.contains("map"));
    }

    #[test]
    fn test_hover_rich_doc_for_load() {
        let hover = get_hover_for_word("load").expect("load should have hover");
        let text = hover_text(&hover);
        assert!(text.contains("load"));
    }

    #[test]
    fn test_hover_rich_doc_for_if() {
        let hover = get_hover_for_word("if").expect("if should have hover");
        let text = hover_text(&hover);
        assert!(text.contains("if"));
    }

    // ---- Fallback chain (BUILTIN_FUNCTIONS / SPECIAL_FORMS / MACROS / OPERATORS) ----

    #[test]
    fn test_hover_fallback_for_print() {
        // print has rich doc, should come from get_doc
        let hover = get_hover_for_word("print").expect("print should have hover");
        let text = hover_text(&hover);
        assert!(text.contains("print"));
        assert!(text.contains("Print all arguments")); // from rich doc
    }

    #[test]
    fn test_hover_fallback_for_floor() {
        // floor is in BUILTIN_FUNCTIONS but maybe not in rich docs
        let hover = get_hover_for_word("floor");
        // Should have some hover info (from completions fallback if not in rich docs)
        assert!(hover.is_some(), "floor should have hover info (rich or fallback)");
    }

    #[test]
    fn test_hover_fallback_for_define() {
        let hover = get_hover_for_word("define").expect("define should have hover");
        let text = hover_text(&hover);
        assert!(text.contains("define"));
    }

    #[test]
    fn test_hover_fallback_for_timeframe() {
        let hover = get_hover_for_word("timeframe").expect("timeframe should have hover");
        let text = hover_text(&hover);
        assert!(text.contains("timeframe"));
    }

    #[test]
    fn test_hover_for_operator_comparison() {
        for op in &[">", "<", ">=", "<=", "=", "!="] {
            let hover = get_hover_for_word(op);
            assert!(hover.is_some(), "Operator '{}' should have hover info", op);
        }
    }

    // ---- Unknown symbols ----

    #[test]
    fn test_hover_unknown_symbol_returns_none() {
        assert!(get_hover_for_word("zzz-nonexistent-42").is_none());
    }

    #[test]
    fn test_hover_empty_string_returns_none() {
        assert!(get_hover_for_word("").is_none());
    }

    #[test]
    fn test_hover_for_nil_returns_none_or_fallback() {
        // nil might not have rich docs; check it doesn't panic
        let _ = get_hover_for_word("nil");
    }

    // ---- helpers ----

    fn hover_text(hover: &Hover) -> String {
        match &hover.contents {
            HoverContents::Scalar(MarkedString::String(s)) => s.clone(),
            _ => String::new(),
        }
    }

    #[test]
    fn test_hover_all_rich_doc_functions() {
        // All functions that have rich docs in docs.rs
        let expected = [
            "+", "-", "*", "/", "**",
            "!", "&&", "||", "=", "!=", ">", "<", ">=", "<=",
            "define", "let", "set!", "fn", "if",
            "cond", "case", "when", "unless", "do",
            "defun", "print", "printf",
            "load", "unload", "step", "alias", "unalias",
            "whenever", "find", "count", "timeframe",
            "get", "slice", "reval",
            "groups", "in-groups", "resolve-group",
            "in-scopes", "all-scopes",
            "list", "first", "second", "last", "rest", "in",
            "map", "fold", "zip", "min", "max", "sum", "average", "length",
            "array", "seta", "geta", "geta/default", "dela", "mapa",
            "convert/bin", "atom?", "symbol?", "string?", "int?", "list?",
            "exit", "eval-file", "while", "signal?",
        ];
        for name in &expected {
            let hover = get_hover_for_word(name);
            assert!(hover.is_some(),
                "Function '{}' should have hover info (from docs or fallback)", name);
            let text = hover_text(&hover.unwrap());
            assert!(!text.is_empty(),
                "Hover text for '{}' should not be empty", name);
        }
    }

    #[test]
    fn test_hover_all_completion_items() {
        use crate::wal::completions::{
            BUILTIN_FUNCTIONS, MACROS, OPERATORS, SPECIAL_FORMS,
        };
        let mut missing = Vec::new();
        for (name, _) in OPERATORS {
            if get_hover_for_word(name).is_none() { missing.push(*name); }
        }
        for (name, _) in SPECIAL_FORMS {
            if get_hover_for_word(name).is_none() { missing.push(*name); }
        }
        for (name, _) in BUILTIN_FUNCTIONS {
            if get_hover_for_word(name).is_none() { missing.push(*name); }
        }
        for (name, _) in MACROS {
            if get_hover_for_word(name).is_none() { missing.push(*name); }
        }
        assert!(missing.is_empty(),
            "These completion items lack hover info: {:?}", missing);
    }

    #[test]
    fn test_hover_doc_contains_signature() {
        let sig_checks = [
            ("+", "(+ expr*)"),
            ("define", "(define id expr)"),
            ("let", "(let ([id expr]+) body)"),
            ("defun", "(defun name args body+)"),
            ("if", "(if cond then else)"),
            ("load", "(load file [id])"),
            ("map", "(map f xs)"),
        ];
        for (name, sig_fragment) in &sig_checks {
            let hover = get_hover_for_word(name)
                .unwrap_or_else(|| panic!("{} should have hover", name));
            let text = hover_text(&hover);
            assert!(text.contains(sig_fragment),
                "Hover for '{}' should contain '{}', got:\n{}", name, sig_fragment, text);
        }
    }
}
