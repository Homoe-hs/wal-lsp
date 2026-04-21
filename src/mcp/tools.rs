use serde_json::{json, Value};
use std::process::Command;
use tracing::info;

pub fn list_tools() -> Value {
    json!({
        "tools": [
            {
                "name": "wal_parse",
                "description": "Parse WAL code and return the AST with structure",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "WAL code to parse"
                        }
                    },
                    "required": ["code"]
                }
            },
            {
                "name": "wal_analyze",
                "description": "Analyze WAL code and return diagnostics, symbols, and errors",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "WAL code to analyze"
                        }
                    },
                    "required": ["code"]
                }
            },
            {
                "name": "wal_execute",
                "description": "Execute WAL code and return the result",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "WAL code to execute"
                        },
                        "args": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Arguments to pass to the WAL script"
                        }
                    },
                    "required": ["code"]
                }
            },
            {
                "name": "wal_complete",
                "description": "Get completion suggestions for WAL code",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "Partial WAL code for completions"
                        }
                    },
                    "required": ["code"]
                }
            },
            {
                "name": "wal_symbols",
                "description": "Get document symbol hierarchy (define, fn, defsig, defmacro)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "WAL code to extract symbols from"
                        }
                    },
                    "required": ["code"]
                }
            },
            {
                "name": "wal_format",
                "description": "Format WAL code",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "WAL code to format"
                        }
                    },
                    "required": ["code"]
                }
            }
        ]
    })
}

pub fn call_tool(params: &Value) -> Value {
    let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
    let arguments = params.get("arguments");

    info!("Calling tool: {} with args: {:?}", name, arguments);

    match name {
        "wal_parse" => {
            let code = arguments
                .and_then(|a| a.get("code"))
                .and_then(|c| c.as_str())
                .unwrap_or("");
            wal_parse(code)
        }
        "wal_analyze" => {
            let code = arguments
                .and_then(|a| a.get("code"))
                .and_then(|c| c.as_str())
                .unwrap_or("");
            wal_analyze(code)
        }
        "wal_execute" => {
            let code = arguments
                .and_then(|a| a.get("code"))
                .and_then(|c| c.as_str())
                .unwrap_or("");
            wal_execute(code)
        }
        "wal_complete" => {
            let code = arguments
                .and_then(|a| a.get("code"))
                .and_then(|c| c.as_str())
                .unwrap_or("");
            wal_complete(code)
        }
        "wal_symbols" => {
            let code = arguments
                .and_then(|a| a.get("code"))
                .and_then(|c| c.as_str())
                .unwrap_or("");
            wal_symbols(code)
        }
        "wal_format" => {
            let code = arguments
                .and_then(|a| a.get("code"))
                .and_then(|c| c.as_str())
                .unwrap_or("");
            wal_format(code)
        }
        _ => {
            json!({
                "content": [{
                    "type": "text",
                    "text": format!("Unknown tool: {}", name)
                }],
                "isError": true
            })
        }
    }
}

fn wal_parse(code: &str) -> Value {
    use crate::wal::parser::WalParser;

    let mut parser = WalParser::new();
    match parser.parse(code) {
        Ok(tree) => {
            let root = tree.root_node();
            let has_error = root.has_error();
            let node_kind = root.kind();
            let node_text = code.get(root.byte_range()).unwrap_or(code);

            json!({
                "content": [{
                    "type": "text",
                    "text": json!({
                        "success": true,
                        "ast": {
                            "rootKind": node_kind,
                            "hasError": has_error,
                            "text": node_text
                        }
                    }).to_string()
                }]
            })
        }
        Err(e) => {
            json!({
                "content": [{
                    "type": "text",
                    "text": json!({
                        "success": false,
                        "error": e
                    }).to_string()
                }],
                "isError": true
            })
        }
    }
}

fn wal_analyze(code: &str) -> Value {
    use crate::wal::parser::WalParser;
    use crate::wal::symbols::extract_symbols;

    let mut parser = WalParser::new();
    let tree = parser.parse_with_errors(code);
    let root = tree.root_node();
    let has_errors = root.has_error();

    let symbols = extract_symbols(code);
    let symbol_count = symbols.len();

    let errors: Vec<Value> = if has_errors {
        vec![json!({
            "code": "SYNTAX_ERROR",
            "message": "Syntax error in code",
            "severity": "error"
        })]
    } else {
        vec![]
    };

    json!({
        "content": [{
            "type": "text",
            "text": json!({
                "success": true,
                "hasErrors": has_errors,
                "symbols": symbols.iter().map(|s| {
                    json!({
                        "name": s.name,
                        "kind": kind_name(s.kind),
                        "detail": s.detail
                    })
                }).collect::<Vec<_>>(),
                "symbolCount": symbol_count,
                "errors": errors
            }).to_string()
        }]
    })
}

fn kind_name(kind: lsp_types::SymbolKind) -> &'static str {
    match kind {
        lsp_types::SymbolKind::FUNCTION => "function",
        lsp_types::SymbolKind::VARIABLE => "variable",
        lsp_types::SymbolKind::METHOD => "method",
        lsp_types::SymbolKind::FIELD => "field",
        lsp_types::SymbolKind::CLASS => "class",
        lsp_types::SymbolKind::MODULE => "module",
        lsp_types::SymbolKind::NAMESPACE => "namespace",
        _ => "unknown",
    }
}

fn wal_execute(code: &str) -> Value {
    let output = Command::new("wal").args(["-c", code]).output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() {
                json!({
                    "content": [{
                        "type": "text",
                        "text": json!({
                            "success": true,
                            "output": stdout.to_string(),
                            "exitCode": output.status.code()
                        }).to_string()
                    }]
                })
            } else {
                json!({
                    "content": [{
                        "type": "text",
                        "text": json!({
                            "success": false,
                            "error": stdout.to_string(),
                            "stderr": stderr.to_string(),
                            "exitCode": output.status.code()
                        }).to_string()
                    }],
                    "isError": true
                })
            }
        }
        Err(e) => {
            json!({
                "content": [{
                    "type": "text",
                    "text": json!({
                        "success": false,
                        "error": format!("Failed to execute WAL: {}. Make sure 'wal' is installed and in PATH.", e)
                    }).to_string()
                }],
                "isError": true
            })
        }
    }
}

fn wal_complete(_code: &str) -> Value {
    use crate::wal::completions::get_all_completions;

    let completions = get_all_completions();

    let items: Vec<Value> = completions
        .iter()
        .map(|c| {
            json!({
                "label": c.label,
                "kind": kind_str(&c.kind),
                "detail": c.detail
            })
        })
        .collect();

    json!({
        "content": [{
            "type": "text",
            "text": json!({
                "completions": items,
                "count": items.len()
            }).to_string()
        }]
    })
}

fn kind_str(kind: &crate::wal::completions::CompletionKind) -> &'static str {
    match kind {
        crate::wal::completions::CompletionKind::Keyword => "keyword",
        crate::wal::completions::CompletionKind::Function => "function",
        crate::wal::completions::CompletionKind::Operator => "operator",
        crate::wal::completions::CompletionKind::Variable => "variable",
        crate::wal::completions::CompletionKind::Signal => "signal",
    }
}

fn wal_symbols(code: &str) -> Value {
    use crate::wal::symbols::extract_symbols;

    let symbols = extract_symbols(code);

    let result: Vec<Value> = symbols
        .iter()
        .map(|s| {
            json!({
                "name": s.name,
                "kind": kind_name(s.kind),
                "detail": s.detail,
                "range": {
                    "start": {
                        "line": s.range.start.line,
                        "character": s.range.start.character
                    },
                    "end": {
                        "line": s.range.end.line,
                        "character": s.range.end.character
                    }
                },
                "children": s.children.iter().map(|c| {
                    json!({
                        "name": c.name,
                        "kind": kind_name(c.kind),
                        "detail": c.detail
                    })
                }).collect::<Vec<_>>()
            })
        })
        .collect();

    json!({
        "content": [{
            "type": "text",
            "text": json!({
                "symbols": result,
                "count": result.len()
            }).to_string()
        }]
    })
}

fn wal_format(code: &str) -> Value {
    let formatted = crate::wal::format::format_document(code);
    json!({
        "content": [{
            "type": "text",
            "text": json!({
                "formatted": formatted,
                "changed": formatted != code
            }).to_string()
        }]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tools() {
        let tools = list_tools();
        assert!(tools.get("tools").is_some());
        let tools_array = tools.get("tools").unwrap().as_array().unwrap();
        assert_eq!(tools_array.len(), 6);

        let tool_names: Vec<&str> = tools_array
            .iter()
            .map(|t| t.get("name").unwrap().as_str().unwrap())
            .collect();
        assert!(tool_names.contains(&"wal_parse"));
        assert!(tool_names.contains(&"wal_analyze"));
        assert!(tool_names.contains(&"wal_execute"));
        assert!(tool_names.contains(&"wal_complete"));
        assert!(tool_names.contains(&"wal_symbols"));
        assert!(tool_names.contains(&"wal_format"));
    }

    #[test]
    fn test_wal_parse_invalid() {
        let result = wal_parse("(+ 1");
        let content = result.get("content").unwrap().as_array().unwrap();
        let text = content[0].get("text").unwrap().as_str().unwrap();
        assert!(
            text.contains("hasError") || text.contains("has_error"),
            "Expected hasError in: {}",
            text
        );
    }

    #[test]
    fn test_wal_analyze() {
        let result = wal_analyze("(define x 1)");
        let content = result.get("content").unwrap().as_array().unwrap();
        let text = content[0].get("text").unwrap().as_str().unwrap();
        assert!(text.contains("symbols"));
    }

    #[test]
    fn test_wal_complete() {
        let result = wal_complete("(+ ");
        let content = result.get("content").unwrap().as_array().unwrap();
        let text = content[0].get("text").unwrap().as_str().unwrap();
        assert!(text.contains("completions"));
    }

    #[test]
    fn test_wal_symbols() {
        let result = wal_symbols("(define x 1)");
        let content = result.get("content").unwrap().as_array().unwrap();
        let text = content[0].get("text").unwrap().as_str().unwrap();
        assert!(text.contains("symbols"));
    }

    #[test]
    fn test_wal_format() {
        let result = wal_format("(define x 1)");
        let content = result.get("content").unwrap().as_array().unwrap();
        let text = content[0].get("text").unwrap().as_str().unwrap();
        assert!(text.contains("formatted"));
    }

    #[test]
    fn test_call_tool_unknown() {
        let params = serde_json::json!({
            "name": "unknown_tool",
            "arguments": {}
        });
        let result = call_tool(&params);
        assert!(result.get("isError").is_some());
        let is_error = result.get("isError").unwrap().as_bool().unwrap();
        assert!(is_error);
    }

    #[test]
    fn test_call_tool_wal_parse() {
        let params = serde_json::json!({
            "name": "wal_parse",
            "arguments": {"code": "(+ 1 2)"}
        });
        let result = call_tool(&params);
        assert!(result.get("content").is_some());
    }

    #[test]
    fn test_call_tool_wal_symbols() {
        let params = serde_json::json!({
            "name": "wal_symbols",
            "arguments": {"code": "(define x 1)"}
        });
        let result = call_tool(&params);
        let content = result.get("content").unwrap().as_array().unwrap();
        let text = content[0].get("text").unwrap().as_str().unwrap();
        assert!(text.contains("symbols") || text.contains("count"));
    }
}
