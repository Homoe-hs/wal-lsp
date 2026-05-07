use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

fn send_msg(stdin: &mut impl Write, msg: &str) {
    let header = format!("Content-Length: {}\r\n\r\n", msg.len());
    stdin.write_all(header.as_bytes()).unwrap();
    stdin.write_all(msg.as_bytes()).unwrap();
    stdin.flush().unwrap();
}

fn recv_msg<R: BufRead>(reader: &mut R) -> Option<String> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut header_line = String::new();
        match reader.read_line(&mut header_line) {
            Ok(0) => return None,
            Ok(_) => {}
            Err(_) => return None,
        }
        let trimmed = header_line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(value) = trimmed.strip_prefix("Content-Length:") {
            let value = value.trim();
            content_length = value.parse().ok();
        }
    }
    let len = content_length.expect("No Content-Length header found");
    let mut body = vec![0u8; len];
    reader.read_exact(&mut body).unwrap();
    Some(String::from_utf8(body).unwrap())
}

fn find_binary() -> std::path::PathBuf {
    let mut p = std::env::current_exe().unwrap();
    p.pop();
    p.pop();
    p.push("wal-lsp");
    if !p.exists() {
        p = std::env::current_dir().unwrap().join("target/debug/wal-lsp");
    }
    p
}

struct LspSession {
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
    child: Child,
}

impl LspSession {
    fn new() -> Self {
        let binary = find_binary();
        let mut child = Command::new(&binary)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap_or_else(|_| panic!("Failed to start {:?}", binary));

        let stdin = child.stdin.take().unwrap();
        let reader = BufReader::new(child.stdout.take().unwrap());
        Self {
            stdin,
            reader,
            child,
        }
    }

    fn initialize(&mut self) -> serde_json::Value {
        send_msg(
            &mut self.stdin,
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"capabilities":{},"rootUri":null}}"#,
        );
        let resp = recv_msg(&mut self.reader).expect("Should receive initialize response");
        let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(
            parsed["result"].is_object(),
            "Should have result, got: {:?}",
            parsed
        );

        send_msg(
            &mut self.stdin,
            r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#,
        );
        parsed
    }

    fn send(&mut self, msg: &str) {
        send_msg(&mut self.stdin, msg);
    }

    fn recv(&mut self) -> Option<String> {
        recv_msg(&mut self.reader)
    }

    #[allow(dead_code)]
    fn recv_json(&mut self) -> Option<serde_json::Value> {
        self.recv()
            .and_then(|s| serde_json::from_str(&s).ok())
    }

    fn shutdown_and_exit(mut self) {
        send_msg(
            &mut self.stdin,
            r#"{"jsonrpc":"2.0","id":99,"method":"shutdown","params":null}"#,
        );
        let resp = self.recv().expect("Should receive shutdown response");
        let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(parsed["id"], 99);

        send_msg(
            &mut self.stdin,
            r#"{"jsonrpc":"2.0","method":"exit","params":null}"#,
        );
        drop(self.stdin);
        let status = self.child.wait().expect("Wait for exit");
        assert!(
            status.success(),
            "Server should exit cleanly, got: {}",
            status
        );
    }
}

#[test]
fn test_lsp_initialize_handshake() {
    let mut session = LspSession::new();
    session.initialize();

    session.send(
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///test.wal","languageId":"wal","version":1,"text":"(+ 1 2)"}}}"#,
    );
    let _diag = session.recv();

    session.shutdown_and_exit();
}

#[test]
fn test_lsp_completion_returns_results() {
    let mut session = LspSession::new();
    session.initialize();

    session.send(
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///test.wal","languageId":"wal","version":1,"text":"(+ 1 2)"}}}"#,
    );
    let _ = session.recv();

    send_msg(
        &mut session.stdin,
        r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/completion","params":{"textDocument":{"uri":"file:///test.wal"},"position":{"line":0,"character":2}}}"#,
    );
    let resp = session.recv().expect("Should get completion response");
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();
    assert_eq!(parsed["id"], 2);

    session.shutdown_and_exit();
}

#[test]
fn test_lsp_hover_returns_documentation() {
    let mut session = LspSession::new();
    session.initialize();

    session.send(
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///test.wal","languageId":"wal","version":1,"text":"(+ 1 2)"}}}"#,
    );
    let _ = session.recv();

    send_msg(
        &mut session.stdin,
        r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///test.wal"},"position":{"line":0,"character":1}}}"#,
    );
    let resp = session.recv().expect("Should get hover response");
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();
    assert_eq!(parsed["id"], 2);
    if let Some(result) = parsed["result"].as_object() {
        assert!(result.contains_key("contents"), "Hover should have contents");
    }

    session.shutdown_and_exit();
}

#[test]
fn test_lsp_diagnostics_on_open() {
    let mut session = LspSession::new();
    session.initialize();

    send_msg(
        &mut session.stdin,
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///bad.wal","languageId":"wal","version":1,"text":"(define x)"}}}"#,
    );
    let notify = session.recv().expect("Should receive diagnostics");
    let parsed: serde_json::Value = serde_json::from_str(&notify).unwrap();
    assert_eq!(parsed["method"], "textDocument/publishDiagnostics");
    let diags = parsed["params"]["diagnostics"].as_array().unwrap();
    assert!(!diags.is_empty(), "Bad code should produce diagnostics");

    session.shutdown_and_exit();
}

#[test]
fn test_lsp_unknown_method_does_not_crash() {
    let mut session = LspSession::new();
    session.initialize();

    send_msg(
        &mut session.stdin,
        r#"{"jsonrpc":"2.0","id":100,"method":"textDocument/unknownMethod","params":{}}"#,
    );

    session.send(
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///test.wal","languageId":"wal","version":1,"text":"(+ 1 2)"}}}"#,
    );
    let _ = session.recv();

    session.shutdown_and_exit();
}

#[test]
fn test_lsp_goto_definition() {
    let mut session = LspSession::new();
    session.initialize();

    send_msg(
        &mut session.stdin,
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///def.wal","languageId":"wal","version":1,"text":"(define my-var 42)\n(+ my-var 1)"}}}"#,
    );
    let _ = session.recv();

    send_msg(
        &mut session.stdin,
        r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/definition","params":{"textDocument":{"uri":"file:///def.wal"},"position":{"line":1,"character":4}}}"#,
    );
    let resp = session.recv().expect("Should get definition response");
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();
    assert_eq!(parsed["id"], 2);

    session.shutdown_and_exit();
}

#[test]
fn test_lsp_document_symbols() {
    let mut session = LspSession::new();
    session.initialize();

    send_msg(
        &mut session.stdin,
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///sym.wal","languageId":"wal","version":1,"text":"(define x 42)\n(defun add [a b] (+ a b))"}}}"#,
    );
    let _ = session.recv();

    send_msg(
        &mut session.stdin,
        r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/documentSymbol","params":{"textDocument":{"uri":"file:///sym.wal"}}}"#,
    );
    let resp = session.recv().expect("Should get symbols response");
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();
    assert_eq!(parsed["id"], 2);

    session.shutdown_and_exit();
}
