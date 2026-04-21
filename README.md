# WAL LSP + MCP Server

A Language Server Protocol (LSP) and Model Context Protocol (MCP) implementation for [WAL (Waveform Analysis Language)](https://github.com/hesheng/WAL).

## Features

### LSP Features
- **Syntax highlighting** - via semantic tokens
- **Parse error diagnostics** - reports syntax errors in real-time
- **Completion** - built-in functions, keywords, operators, and **VCD signals**
- **Hover** - documentation for all built-in functions
- **Go to definition** - cross-file symbol navigation
- **Document symbols** - hierarchical symbol tree

### MCP Tools
| Tool | Description |
|------|-------------|
| `wal_parse` | Parse WAL code and return AST |
| `wal_analyze` | Analyze WAL code for diagnostics and symbols |
| `wal_execute` | Execute WAL code via `wal` command |
| `wal_complete` | Get completion suggestions |
| `wal_symbols` | Get document symbols |
| `wal_format` | Format WAL code with proper indentation |

### VCD Signal Completion
The LSP provides intelligent completion for VCD signals:
- **Auto-detection**: Parses `(load "file.vcd")` calls to load signal lists
- **Virtual signals**: Recognizes `(defsig name ...)` definitions
- **Prefix matching**: Fast O(k) lookup by signal prefix
- **Fuzzy matching**: Fallback fuzzy search for typo tolerance
- **Cross-document**: Signals from all open documents are available

Example:
```wal
(load "signals.vcd")
(defsig my-signal (= clk 1))

;; Completion works for both loaded signals and defsig definitions
(get tb.|)  ;; <- completion here shows all signals
```

### Tab Formatting
- Uses **tab indentation** (default 4 spaces equivalent)
- AST-based formatting for proper S-expression alignment
- Preserves nested list structure

## Installation

### Prerequisites

1. **Rust toolchain**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Tree-sitter CLI** (for grammar development)
   ```bash
   npm install -g tree-sitter-cli
   ```

3. **WAL runtime** (optional, for `wal_execute` tool)
   ```bash
   # Install wal command
   ```

### Build

```bash
cd WAL-lsp
cargo build --release
```

The binary will be at `target/release/wal-lsp`.

## Editor Configuration

### OpenCode

Add to your MCP configuration:
```json
{
  "mcpServers": {
    "wal-lsp": {
      "command": "/path/to/wal-lsp",
      "args": ["--mcp"]
    }
  }
}
```

### VS Code

1. Install the VS Code extension
2. The extension will auto-detect `.wal` files

### Neovim/Vim

```lua
-- ~/.config/nvim/lsp/wal.lua
local lspconfig = require('lspconfig')
lspconfig.wal_lsp.setup({
  cmd = {"/path/to/wal-lsp"},
  filetypes = {"wal"},
})
```

### Emacs

```elisp
(require 'lsp-mode')
(add-to-list 'lsp-language-id-configuration '(".*\\.wal\\'" . "wal"))
(lsp-register-client
 (make-lsp-client :server-id 'wal-lsp
                  :cmd '("wal-lsp")))
```

## Usage

### MCP Mode

```bash
# Start MCP server
wal-lsp --mcp

# Then send JSON-RPC messages via stdin
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | wal-lsp --mcp
```

### LSP Mode

```bash
# Start LSP server (uses stdio by default)
wal-lsp

# Or explicitly
wal-lsp --lsp
```

## Project Structure

```
WAL-lsp/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs              # Entry point (LSP/MCP dual mode)
│   ├── lib.rs               # Library exports
│   ├── workspace/           # Workspace management
│   │   ├── mod.rs          # Workspace, SymbolIndex, DocumentInfo
│   │   └── waveform.rs     # WaveformManager for VCD signals
│   ├── lsp/                 # LSP protocol implementation
│   │   ├── mod.rs          # LSP server setup
│   │   └── handlers/       # LSP request handlers
│   │       ├── completion.rs   # Completion (keywords + signals)
│   │       ├── diagnostics.rs  # Parse error reporting
│   │       ├── goto.rs         # Cross-file goto definition
│   │       ├── hover.rs        # Function documentation
│   │       └── symbols.rs     # Document symbols
│   ├── mcp/                # MCP protocol implementation
│   │   ├── mod.rs
│   │   └── tools.rs       # MCP tool definitions
│   └── wal/                # WAL language core
│       ├── mod.rs         # Module exports
│       ├── parser.rs      # Tree-sitter wrapper
│       ├── symbols.rs     # Symbol extraction from AST
│       ├── completions.rs # Built-in function data
│       ├── docs.rs        # Function documentation
│       ├── format.rs      # Code formatter (tab indent)
│       └── waveform.rs    # VCD/CSV header parsing
├── tree-sitter-wal/       # Tree-sitter grammar
│   ├── grammar.js
│   └── src/               # Generated parser
└── editors/               # Editor configurations
```

## Architecture

### Workspace Module
Centralized workspace management:
- `Workspace` - document cache, symbol index, waveform manager
- `DocumentInfo` - document text and version tracking
- `SymbolIndex` - global symbol index for cross-file navigation
- `WaveformManager` - VCD signal management

### Signal Completion Flow
```
1. User triggers completion (e.g., typing "tb.")
2. LSP extracts prefix from cursor position
3. Workspace looks up signals in WaveformManager
4. Prefix match → Fuzzy fallback
5. Returns completion items
```

## License

BSD-3-Clause (same as WAL project)
