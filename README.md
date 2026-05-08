# WAL LSP Server

Language Server Protocol implementation for [WAL (Waveform Analysis Language)](https://wal-lang.org).

---

## Features

### LSP Capabilities

| 功能 | 说明 |
|------|------|
| **Diagnostics** | 语法错误 + 语义错误检测 (arity、未知符号、结构校验)，规则可配 severity/enabled |
| **Completion** | 140+ 内建补全 + VCD/CSV/FST 信号名补全 + `completionItem/resolve` 延迟加载文档 |
| **Hover** | 悬停显示函数文档、签名和示例 |
| **Go-to-Definition** | 跨文件符号跳转 |
| **Find References** | 跨文件查找符号引用 |
| **Document Symbols** | 符号树 (define / defun / defmacro / defsig / fn) |
| **Workspace Symbols** | 跨文件符号搜索 |
| **Configuration** | `workspace/didChangeConfiguration` — 规则 severity、启用状态、缩进空格数 |
| **信号名补全** | 从 VCD/CSV/FST 文件自动加载信号列表，支持前缀和模糊匹配 |
| **增量解析** | tree-sitter 增量解析 + 树缓存，避免每次全文重解析 |
| **波形自动加载** | 在 `(load ...)` 出现时自动加载 VCD/CSV/FST 文件 |

### Semantic Error Checking

在 tree-sitter 语法检查的基础上，通过可扩展规则引擎提供语义级诊断：

| 诊断类型 | 严重度 | 规则 ID | 示例 |
|---------|--------|---------|------|
| 未知函数/运算符 | ⚠️ WARNING | `unknown-symbol` | `(frobnoz 42)` → Unknown function |
| 参数数量不匹配 | ❌ ERROR | `arity-check` | `(define)` → expects 2 argument(s), got 0 |
| 语法错误 | ❌ ERROR | — | `(define a (+ 1` → Syntax error |
| 结构校验 (let/case/fn) | ❌ ERROR | `form-structure` | 绑定缺值、case clause 空、参数非符号 |

**已知符号库**: 130+ 个 WAL 运算符/特式/宏/内建函数  
**精确 arity 检测**: 50+ 个函数，严格验证参数数量  
**规则抑制**: 支持 `;; lint_off <rule-id>` / `;; lint_on <rule-id>` 注释级控制

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
 1,000K       714.5K      29.9s    23,880/s      ✅
 2,000K     1,430.0K      56.2s    25,425/s      ✅
 3,000K     2,129.1K     104.1s    20,454/s      ✅
 4,000K     2,838.4K     157.8s    17,991/s      ✅
 4,500K     3,119.9K     186.6s    16,719/s      ✅
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
printf 'Content-Length: 75\r\n\r\n{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}' \
  | wal-lsp
```

---

## Architecture

```
wal-lsp/
├── src/
│   ├── config.rs                # 全局配置 (LspConfig, RuleConfig, FormatConfig)
│   ├── main.rs                  # LSP entry point (clap: --help/--version)
│   ├── workspace/
│   │   ├── mod.rs               # Workspace, DocumentInfo, SymbolIndex (跨文件, 树缓存)
│   │   └── waveform.rs          # WaveformManager (VCD/CSV/FST 信号解析 & 查找)
│   ├── lsp/
│   │   ├── mod.rs               # LSP server (lsp-server, LazyLock workspace)
│   │   └── handlers/
│   │       ├── diagnostics.rs   # 语法 + 语义错误检测 (☆ 核心)
│   │       ├── completion.rs    # 140+ 内建补全 + 波形信号名补全
│   │       ├── completion_resolve.rs # completionItem/resolve 延迟加载
│   │       ├── config.rs        # workspace/didChangeConfiguration
│   │       ├── hover.rs         # 函数文档 (rich docs + fallback)
│   │       ├── goto.rs          # 跨文件 go-to-definition
│   │       ├── formatting.rs    # 代码格式化
│   │       ├── references.rs    # textDocument/references
│   │       ├── symbols.rs       # 文档符号树
│   │       └── workspace_symbol.rs # workspace/symbol
│   └── wal/                     # WAL 语言核心
│       ├── parser.rs            # Tree-sitter wrapper (全局单例 + 增量解析)
│       ├── completions.rs       # 140+ 补全项 (OPERATORS/SPECIAL_FORMS/BUILTIN_FUNCTIONS/MACROS)
│       ├── docs.rs              # 丰富函数文档 (签名+描述+示例, 60+ 项)
│       ├── symbols.rs           # AST → WalSymbol 提取 (define/defun/defmacro/defsig/fn)
│       ├── format.rs            # 格式化引擎 (可配缩进, tree-sitter AST 驱动)
│       ├── fst_reader.rs        # FST 二进制波形格式解析器
│       ├── waveform.rs          # VCD/CSV/FST 波形头解析
│       └── rules/               # 可扩展语义检查规则引擎
│           ├── mod.rs           # Rule trait, RuleRegistry, LintContext, 抑制解析
│           ├── arity.rs         # 参数数量检查 (50+ 已知函数)
│           ├── unknown_symbol.rs # 未知函数/符号警告 (130+ 已知符号)
│           └── structure.rs     # let/case/fn/defun 结构校验
├── tree-sitter-wal/             # Tree-sitter WAL 语法定义
│   ├── grammar.js               # BNF 文法 (sexpr, atom, list, quoted, scoped_symbol...)
│   └── src/parser.c             # 生成的 C 解析器
├── editors/                     # 编辑器配置预设
│   ├── vscode/settings.json
│   ├── vim/wal.lua
│   ├── emacs/wal-lsp.el
│   └── opencode/mcp.json
├── docs/wal-lang/               # WAL 语言参考文档 (8 篇)
├── tests/
│   ├── lsp_handshake.rs         # LSP 协议集成测试 (init/completion/hover/goto/diagnostics)
│   └── syntax/                  # 21 个 WAL 语法样本文件 (含 99_mega_test.wal)
└── Cargo.toml
```

### Key Design Decisions

| 决策 | 说明 |
|------|------|
| **Pure LSP** | 无 MCP 模式，单一职责 |
| **LazyLock 工作空间** | 单例 `Arc<RwLock<Workspace>>`，所有 handler 共享 |
| **RwLock 毒化保护** | 所有 `.read()`/`.write()` 使用 `unwrap_or_else(\|e\| e.into_inner())` |
| **规则引擎** | `Rule` trait + `RuleRegistry`，规则可独立注册/启用/抑制 |
| **规则抑制** | `;; lint_off arity-check` / `;; lint_on all` 注释级控制 |
| **全量同步 + 增量解析** | `TextDocumentSyncKind::FULL`，缓存旧树进行 tree-sitter 增量解析 |
| **波形信号补全** | 优先前缀匹配 → 模糊子串匹配，支持 VCD/CSV/FST |
| **波形自动加载** | `didOpen`/`didChange` 时自动扫描 `(load ...)` 和 `(defsig ...)` |
| **FST 解析** | 纯 Rust 实现，支持 LZ4 和 Zlib 压缩块，零外部依赖 |
| **顶层检测** | `is_toplevel` 检查避免递归体变量误报 |
| **格式化** | Tree-sitter AST 驱动，可配置缩进空格数 |
| **配置系统** | `didChangeConfiguration` — 规则 severity/enabled 覆盖 + 格式化选项 |

---

## License

BSD-3-Clause
