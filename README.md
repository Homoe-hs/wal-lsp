# WAL LSP Server

Language Server Protocol implementation for [WAL (Waveform Analysis Language)](https://wal-lang.org).

---

## Features

### LSP Capabilities

| 功能 | 说明 |
|------|------|
| **Diagnostics** | 实时语法错误 + 语义错误检测 |
| **Completion** | 125+ 内建函数/运算符/宏/特殊变量补全 |
| **Hover** | 悬停显示函数文档和签名 |
| **Go-to-Definition** | 跨文件符号跳转 |
| **Document Symbols** | 层级符号树（define / defun / defmacro） |
| **信号名补全** | 从 VCD 文件自动加载信号列表 |
| **增量同步** | `textDocument/didChange` 增量更新 |

### Semantic Error Checking

在 tree-sitter 语法检查的基础上，额外提供语义级诊断：

| 诊断类型 | 严重度 | 示例 |
|---------|--------|------|
| 未知函数/运算符 | ⚠️ WARNING | `(verilator-sim 1)` → Unknown function |
| 参数数量不匹配 | ❌ ERROR | `(define)` → expects 2 arguments, got 0 |
| 语法错误 | ❌ ERROR | `(define a (+ 1` → Syntax error |

**已知符号库**: 89 个 WAL 运算符/特式/宏/内建函数  
**精确 arity 检测**: 31 个函数，严格验证参数数量  

---

## Performance

### Stress Test Results

对 4,500,000 行 WAL 源码进行诊断测试（单机 8GB RAM）：

```
Lines        Diags        Time       Rate         Status
─────────────────────────────────────────────────────────
     5K         3.6K       0.1s    29,291/s      ✅
    10K         7.1K       0.3s    28,324/s      ✅
    50K        35.9K       1.4s    26,453/s      ✅
   100K        71.6K       2.7s    26,378/s      ✅
   500K       357.4K      14.5s    24,627/s      ✅
  1,000K      714.5K      29.9s    23,880/s      ✅
  2,000K    1,430.0K      56.2s    25,425/s      ✅
  3,000K    2,129.1K     104.1s    20,454/s      ✅
  4,000K    2,838.4K     157.8s    17,991/s      ✅
  4,500K    3,119.9K     186.6s    16,719/s      ✅
 ───────────────────────────────────────────────────────
  5,000K    — TIMEOUT —  120s+     —             💥
```

| 指标 | 值 |
|------|-----|
| 最大通过行数 | 4,500,000 |
| 最大诊断数 | 3,119,946 |
| 处理速率 | 16,700—29,300 d/s |
| 断点 | 5,000,000 行 (tree-sitter 解析极限) |

实际编辑中单个 `.wal` 文件极少超过 100K 行，性能绰绰有余。

---

## Installation

### Prerequisites

- Rust toolchain (1.70+)

### Build & Install

```bash
cd WAL-lsp
cargo build --release
cp target/release/wal-lsp ~/.local/bin/
```

### Verify

```bash
$ wal-lsp --help
$ wal-lsp --version
```

---

## Editor Configuration

### OpenCode

Add to `~/.config/opencode/opencode.json`:

```json
{
  "lsp": {
    "wal": {
      "command": ["/home/hesheng/.local/bin/wal-lsp"],
      "extensions": [".wal", ".rkt"]
    }
  }
}
```

### VS Code

```json
{
  "languages": [{
    "id": "wal",
    "extensions": [".wal"],
    "configuration": "./language-configuration.json"
  }]
}
```

### Neovim

```lua
vim.api.nvim_create_autocmd("FileType", {
  pattern = "wal",
  callback = function()
    vim.lsp.start({
      name = "wal-lsp",
      cmd = { "wal-lsp" },
      root_dir = vim.fs.dirname(vim.fs.find({ ".git" }, { upward = true })[1]),
    })
  end,
})
```

### Helix

```toml
# ~/.config/helix/languages.toml
[[language]]
name = "wal"
scope = "source.wal"
file-types = ["wal", "rkt"]
language-servers = ["wal-lsp"]

[language-server.wal-lsp]
command = "wal-lsp"
```

---

## Usage

```bash
# Start LSP server (stdio)
wal-lsp
```

The server communicates via JSON-RPC 2.0 over stdin/stdout using the LSP protocol.

### Test LSP directly

```bash
# Initialize
printf 'Content-Length: 108\r\n\r\n{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}' \
  | wal-lsp
```

---

## Architecture

```
wal-lsp/
├── src/
│   ├── main.rs              # LSP entry point (clap: --help/--version)
│   ├── workspace/           # Workspace management
│   │   ├── mod.rs           # Workspace, DocumentInfo, SymbolIndex
│   │   └── waveform.rs      # WaveformManager (VCD signal parsing)
│   ├── lsp/
│   │   ├── mod.rs           # LSP server (lsp-server, LazyLock workspace)
│   │   └── handlers/
│   │       ├── diagnostics.rs  # Syntax + semantic error checking (☆)
│   │       ├── completion.rs   # Keyword + signal completion
│   │       ├── hover.rs        # Function documentation
│   │       ├── goto.rs         # Cross-file go-to-definition
│   │       └── symbols.rs      # Document symbol tree
│   └── wal/                 # WAL language tools
│       ├── parser.rs        # Tree-sitter wrapper
│       ├── completions.rs   # 125+ builtin completion items
│       ├── docs.rs          # Function documentation (Lazy)
│       ├── symbols.rs       # Symbol extraction from AST
│       └── format.rs        # Code formatter
├── tree-sitter-wal/         # Tree-sitter WAL grammar
└── editors/                 # Editor configuration presets
```

### Key Design Decisions

| 决策 | 说明 |
|------|------|
| **Pure LSP** | 无 MCP 模式，单一职责 |
| **LazyLock 工作空间** | 单例 `Arc<RwLock<Workspace>>`，所有 handler 共享 |
| **RwLock 毒化保护** | 所有 `.read()`/`.write()` 使用 `unwrap_or_else(|e| e.into_inner())` |
| **语义检查器** | tree-sitter AST 遍历 + 已知符号/arity 对照表 |
| **顶层检测** | `is_toplevel` 检查避免递归体变量误报 |

---

## License

BSD-3-Clause
