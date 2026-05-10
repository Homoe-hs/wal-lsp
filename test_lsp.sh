#!/usr/bin/env bash
# WAL LSP Integration Tests
# Usage: bash test_lsp.sh [path-to-wal-lsp]
set -euo pipefail

LSP="${1:-wal-lsp}"
PASS=0
FAIL=0

msg() {
    local body="$1"
    printf 'Content-Length: %d\r\n\r\n%s' "${#body}" "$body"
}


check() {
    local name="$1" expected="$2" actual="$3"
    if printf '%s' "$actual" | grep -Fq -- "$expected"; then
        PASS=$((PASS + 1))
        echo "  ✅ $name"
    else
        FAIL=$((FAIL + 1))
        echo "  ❌ $name"
        echo "     want: $expected"
        echo "     got:  $(echo "$actual" | head -c 150)"
    fi
}

# Start server
coproc LSP { "$LSP" 2>/dev/null; }
if [ -z "${LSP_PID:-}" ]; then
    echo "Failed to start $LSP" >&2
    exit 1
fi

send() { msg "$1" >&"${LSP[1]}"; }
recv() {
    local len="" line
    while IFS= read -r line; do
        line="${line%%[$'\r']*}"
        [ -z "$line" ] && break
        [[ "$line" =~ Content-Length:\ ([0-9]+) ]] && len="${BASH_REMATCH[1]}"
    done <&"${LSP[0]}"
    if [ -n "$len" ]; then
        head -c "$len" <&"${LSP[0]}" 2>/dev/null
    fi
}

echo "=== WAL LSP Test Suite ==="
echo "Server: $($LSP --version 2>/dev/null)"
echo ""

# 1. Initialize
send '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"capabilities":{},"rootUri":null}}'
r=$(recv)
check "1. Initialize" '"result"' "$r"
check "   completion"  '"completionProvider"' "$r"
check "   diagnostic"  '"diagnosticProvider"' "$r"
check "   foldingRange"  '"foldingRangeProvider"' "$r"
check "   codeAction"  '"codeActionProvider"' "$r"
check "   signatureHelp"  '"signatureHelpProvider"' "$r"
check "   highlight"  '"documentHighlightProvider"' "$r"
check "   references"  '"referencesProvider"' "$r"
check "   symbol"  '"workspaceSymbolProvider"' "$r"
check "   formatting"  '"documentFormattingProvider"' "$r"

# 2. initialized (no response)
send '{"jsonrpc":"2.0","method":"initialized","params":{}}'

# 3. didOpen + diagnostics
echo ""
send '{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///test.wal","languageId":"wal","version":1,"text":"(define x 42)\n(+ x 1)\n(unknown-fn 2)"}}}'
r=$(recv)
check "2. Diagnostics push" 'publishDiagnostics' "$r"
check "   has unknown-fn" 'unknown-fn' "$r"

# 4. Pull diagnostics
echo ""
send '{"jsonrpc":"2.0","id":2,"method":"textDocument/diagnostic","params":{"textDocument":{"uri":"file:///test.wal"}}}'
r=$(recv)
check "3. Pull diagnostics" '"kind"' "$r"

# 5. Completion
echo ""
send '{"jsonrpc":"2.0","id":3,"method":"textDocument/completion","params":{"textDocument":{"uri":"file:///test.wal"},"position":{"line":0,"character":1}}}'
r=$(recv)
check "4. Completion" '"items"' "$r"
check "   load fn" '"load"' "$r"

# 6. Hover
echo ""
send '{"jsonrpc":"2.0","id":4,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///test.wal"},"position":{"line":0,"character":1}}}'
r=$(recv)
check "5. Hover" '"contents"' "$r"

# 7. Goto-definition
echo ""
send '{"jsonrpc":"2.0","id":5,"method":"textDocument/definition","params":{"textDocument":{"uri":"file:///test.wal"},"position":{"line":0,"character":8}}}'
r=$(recv)
check "6. Goto-def" '"uri"' "$r"

# 8. Document Symbols
echo ""
send '{"jsonrpc":"2.0","id":6,"method":"textDocument/documentSymbol","params":{"textDocument":{"uri":"file:///test.wal"}}}'
r=$(recv)
check "7. Document Symbols" '"name"' "$r"

# 9. Folding Range
echo ""
send '{"jsonrpc":"2.0","id":7,"method":"textDocument/foldingRange","params":{"textDocument":{"uri":"file:///test.wal"}}}'
r=$(recv)
check "8. Folding Range" '[' "$r"

# 10. Formatting
echo ""
send '{"jsonrpc":"2.0","id":8,"method":"textDocument/formatting","params":{"textDocument":{"uri":"file:///test.wal"},"options":{"tabSize":2,"insertSpaces":true}}}'
r=$(recv)
check "9. Formatting" '[' "$r"

# 11. Document Highlight
echo ""
send '{"jsonrpc":"2.0","id":9,"method":"textDocument/documentHighlight","params":{"textDocument":{"uri":"file:///test.wal"},"position":{"line":0,"character":8}}}'
r=$(recv)
check "10. Highlight" '[' "$r"

# 12. Signature Help
echo ""
send '{"jsonrpc":"2.0","id":10,"method":"textDocument/signatureHelp","params":{"textDocument":{"uri":"file:///test.wal"},"position":{"line":0,"character":7}}}'
r=$(recv)
check "11. Signature Help" '"signatures"' "$r"

# 13. References
echo ""
send '{"jsonrpc":"2.0","id":11,"method":"textDocument/references","params":{"textDocument":{"uri":"file:///test.wal"},"position":{"line":0,"character":8},"context":{"includeDeclaration":true}}}'
r=$(recv)
check "12. References" '[' "$r"

# 14. Code Action
echo ""
send '{"jsonrpc":"2.0","id":12,"method":"textDocument/codeAction","params":{"textDocument":{"uri":"file:///test.wal"},"range":{"start":{"line":2,"character":0},"end":{"line":2,"character":14}},"context":{"diagnostics":[{"range":{"start":{"line":2,"character":0},"end":{"line":2,"character":14}},"severity":2,"source":"wal-lsp:unknown-symbol","message":"Unknown function","code":"unknown-symbol"}]}}}'
r=$(recv)
check "13. Code Action" '"title"' "$r"
check "   disable" 'disable' "$r"

# 15. Unknown method returns proper error
echo ""
send '{"jsonrpc":"2.0","id":99,"method":"textDocument/unknownMethod","params":{}}'
r=$(recv)
check "14. Unknown method" '-32601' "$r"

# 16. Shutdown
echo ""
send '{"jsonrpc":"2.0","id":100,"method":"shutdown","params":null}'
r=$(recv)
check "15. Shutdown" '"id":100' "$r"

# 17. Exit
send '{"jsonrpc":"2.0","method":"exit","params":null}'

echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="
exit $FAIL
