# WAL LSP Server

Language Server Protocol implementation for [WAL (Waveform Analysis Language)](https://wal-lang.org).

---

## Features

### LSP Capabilities

| 功能 | 说明 |
|------|------|
| **Diagnostics** | 语法错误 + 语义错误检测 (arity、未知符号、结构校验)，规则可配 severity/enabled，支持推/拉双模式 |
| **Completion** | 167+ 内建补全 + VCD/CSV/FST 信号名补全 + `completionItem/resolve` 延迟加载 |
| **Signature Help** | 输入函数参数时显示签名、文档和参数列表 |
| **Hover** | 悬停显示函数文档、签名和示例 |
| **Go-to-Definition** | 跨文件符号跳转 |
| **Find References** | 跨文件查找符号引用 |
| **Document Highlight** | 光标所在符号高亮（含运算符 `+ - * / = ! > < & \| % ?`）|
| **Code Action** | 快速修复：一键 `;; lint_off rule-id` 抑制规则 |
| **Folding Range** | S-表达式代码折叠 |
| **Document Symbols** | 符号树 (define / defun / defmacro / defsig / fn) |
| **Workspace Symbols** | 跨文件符号搜索 |
| **Configuration** | `workspace/didChangeConfiguration` — 规则 severity、启用、缩进空格 |
| **信号名补全** | 从 VCD/CSV/FST 自动加载信号列表，前缀匹配 → 模糊匹配 |
| **波形自动加载** | `didOpen`/`didChange` 时自动从 `(load ...)` 加载波形文件 |
| **增量解析** | tree-sitter 增量解析 + 旧树缓存 |

### Semantic Error Checking

在 tree-sitter 语法检查基础上，通过可扩展规则引擎提供语义级诊断：

| 诊断类型 | 严重度 | 规则 ID | 示例 |
|---------|--------|---------|------|
| 未知函数/运算符 | ⚠️ WARNING | `unknown-symbol` | `(frobnoz 42)` → Unknown function |
| 参数数量不匹配 | ❌ ERROR | `arity-check` | `(define)` → expects 2 argument(s), got 0 |
| 语法错误 | ❌ ERROR | — | `(define a (+ 1` → Syntax error |
| 结构校验 (let/case/fn) | ❌ ERROR | `form-structure` | 绑定缺值、case clause 空 |

**已知符号库**: 170+ 个 WAL 运算符/特式/宏/内建函数  
**精确 arity 检测**: 84 个函数，严格验证参数数量  
**详尽 hover 文档**: 120+ 条（含签名、描述、示例）  
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

### Real-World Waveform Testing

基于真实硬件验证波形文件测试：

| 指标 | VCD | FST |
|------|-----|-----|
| 文件大小 | 8.0 GB | 650 MB |
| 信号数量 | 156,256 | 156,256 |
| Scope 数量 | 10,063 | 10,063 |
| 头解析时间 | ~0.3s | ~0.5s |
| 信号名补全 | 前缀匹配 < 1ms | 前缀匹配 < 1ms |

已验证的 WAL 操作（通过 LSP 完成信号名补全和诊断）：
- `(load "*.vcd")` / `(load "*.fst")` — 加载波形，自动索引信号名
- `(step [n])` — 步进仿真时间
- `(find cond)` — 查找满足条件的索引
- `(reval signal offset)` — 相对时间求值
- `(whenever cond body+)` — 条件循环
- `(get "signal.name")` — 字符串形式信号访问

---

## Installation

### Prerequisites

| 依赖 | 最低版本 | 说明 |
|------|---------|------|
| Rust | 1.80+ | `LazyLock` 等特性需求，推荐 1.95+ |
| Cargo | 1.80+ | 随 Rust 工具链一起安装 |
| C 编译器 | gcc / clang / msvc | tree-sitter 解析器编译需要 |

> 当前开发/测试环境使用 **Rust 1.95.0**，edition 2021。

### Key Dependencies

| crate | 版本 | 用途 |
|-------|------|------|
| `lsp-server` / `lsp-types` | 0.7 / 0.96 | LSP JSON-RPC 协议框架 |
| `tree-sitter` | 0.24 | 高性能增量解析引擎 |
| `tree-sitter-wal` | local | WAL 语法定义（本仓库子 crate）|
| `tokio` | 1 (full) | 异步运行时 |
| `serde` / `serde_json` | 1 | JSON 序列化 |
| `clap` | 4 | CLI 参数解析 (`--help` / `--version`) |
| `tracing` | 0.1 | 结构化日志 (stderr) |

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

### Run Tests

```bash
# 单元测试 + 集成测试 (260 项)
cargo test -- --test-threads=1

# LSP 协议端到端测试 (27 项)
bash test_lsp.sh
```

---

## Editor Configuration

### Prerequisites

`wal-lsp` must be installed in your `PATH`:

```bash
cp target/release/wal-lsp ~/.local/bin/
```

Verify:

```bash
wal-lsp --version
```

### OpenCode

Add to `~/.config/opencode/opencode.json`:

```json
{
  "lsp": {
    "wal": {
      "command": ["wal-lsp"],
      "extensions": [".wal", ".rkt"]
    }
  }
}
```

### VS Code

Create `.vscode/settings.json` in your project or add globally:

```json
{
  "wal-lsp.command": "wal-lsp",
  "languages": [{
    "id": "wal",
    "extensions": [".wal"],
    "configuration": "./language-configuration.json"
  }]
}
```

Or install the [WAL LSP extension](https://marketplace.visualstudio.com/items?itemName=wal.wal-lsp) (recommended).

### Neovim

```lua
-- Using built-in LSP client (no external plugin required)
vim.api.nvim_create_autocmd({ "BufNewFile", "BufRead" }, {
  pattern = "*.wal",
  callback = function()
    vim.bo.filetype = "wal"
  end,
})

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
│   ├── main.rs                  # LSP 入口 (clap: --help/--version)
│   ├── workspace/
│   │   ├── mod.rs               # Workspace, DocumentInfo, SymbolIndex (树缓存)
│   │   └── waveform.rs          # WaveformManager (VCD/CSV/FST 信号查找)
│   ├── lsp/
│   │   ├── mod.rs               # LSP 主循环 + 请求/通知分发
│   │   └── handlers/
│   │       ├── diagnostics.rs   # 诊断推/拉 + 语义检查 (☆)
│   │       ├── completion.rs    # 167+ 内建补全 + 信号名补全
│   │       ├── completion_resolve.rs # completionItem/resolve
│   │       ├── signature_help.rs # 函数签名提示
│   │       ├── hover.rs         # 悬停文档
│   │       ├── goto.rs          # 跨文件跳转定义
│   │       ├── references.rs    # 符号引用查找
│   │       ├── highlight.rs     # 符号高亮
│   │       ├── folding_range.rs # 代码折叠
│   │       ├── symbols.rs       # 文档符号树
│   │       ├── workspace_symbol.rs # 工作区符号搜索
│   │       ├── code_action.rs   # 快速修复
│   │       ├── formatting.rs    # 代码格式化
│   │       └── config.rs        # didChangeConfiguration
│   └── wal/                     # WAL 语言核心
│       ├── parser.rs            # Tree-sitter 单例 + 增量解析
│       ├── completions.rs       # 167 补全项
│       ├── docs.rs              # 120+ 函数文档 (签名+描述+示例)
│       ├── symbols.rs           # AST → WalSymbol 提取
│       ├── format.rs            # 格式化引擎
│       ├── fst_reader.rs        # FST 二进制波形解析器
│       ├── waveform.rs          # VCD/CSV/FST 头解析
│       └── rules/               # 语义检查规则引擎
│           ├── mod.rs           # Rule trait, Registry, LintContext, 抑制
│           ├── arity.rs         # 84 个函数参数数量检查
│           ├── unknown_symbol.rs # 未知符号警告 (170+ 已知)
│           └── structure.rs     # let/case/fn 结构校验
├── tree-sitter-wal/             # WAL tree-sitter 语法
│   ├── grammar.js               # BNF 文法
│   └── src/parser.c             # C 解析器
├── editors/                     # 编辑器配置
│   ├── vscode/settings.json
│   ├── vim/wal.lua
│   ├── emacs/wal-lsp.el
│   └── opencode/lsp.json
├── docs/wal-lang/               # WAL 语言参考 (8 篇)
├── tests/
│   ├── lsp_handshake.rs         # LSP 协议集成测试
│   └── syntax/                  # 21 个 .wal 语法样本
├── test_lsp.sh                  # 27 项 LSP 端到端测试
└── Cargo.toml
```

### Key Design Decisions

| 决策 | 说明 |
|------|------|
| **Pure LSP** | 无 MCP 模式，单一职责，通过 stdio JSON-RPC 通信 |
| **LazyLock 单例** | `WORKSPACE`、`WAL_PARSER`、`FORMAT_OPTS`、`CONFIG` 均使用 `LazyLock` |
| **RwLock 毒化保护** | 所有 `.read()`/`.write()` 使用 `unwrap_or_else(\|e\| e.into_inner())` |
| **规则引擎** | `Rule` trait + `RuleRegistry`，规则可独立注册/启用/抑制 |
| **规则抑制** | `;; lint_off rule-id` / `;; lint_on rule-id` 注释级控制 |
| **同步 + 增量解析** | `TextDocumentSyncKind::FULL`，缓存旧树增量解析 |
| **信号补全** | 前缀匹配 → 模糊匹配，支持 VCD/CSV/FST |
| **波形自动加载** | 自动扫描 `(load ...)` 和 `(defsig ...)` |
| **FST 解析** | 纯 Rust，LZ4 + Zlib 解压，零 `unsafe` |
| **语义检查** | 84 项 arity 验证 + 170+ 已知符号 + 结构校验 |
| **格式化** | Tree-sitter AST 驱动，可配置缩进 |
| **UTF-8 安全** | 全部代码零 panic 风险 (审计已确认) |
| **零 unsafe** | 整个代码库不含 `unsafe` 代码 |

---

## Related Projects

| 项目 | 说明 |
|------|------|
| [WAL Language](https://wal-lang.org) | WAL (Waveform Analysis Language) 官方文档 |
| [ics-jku/wal](https://github.com/ics-jku/wal) | WAL 语言参考实现 (Python/C++) |
| [Homoe-hs/wal-rust](https://github.com/Homoe-hs/wal-rust) | WAL 高性能 Rust 实现 (82 操作符, 并行 VCD 解析) |
| [tree-sitter-wal](https://github.com/tree-sitter/tree-sitter-wal) | WAL tree-sitter 语法定义 |
| [lsp-server](https://crates.io/crates/lsp-server) | Rust LSP 框架 (JSON-RPC over stdio) |
| [lsp-types](https://crates.io/crates/lsp-types) | LSP 协议类型定义 |
| [Microsoft Pyright](https://github.com/microsoft/pyright) | 参考架构 — LSP 协议实现的优秀范本 |

---

## License

BSD-3-Clause
