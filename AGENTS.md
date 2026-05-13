# wal-lsp — AGENTS.md

## Build

```bash
cargo build --release          # release binary (static, no runtime deps)
```

- `tree-sitter-wal/grammar.js` 修改后执行 `tree-sitter generate`
- `flate2` 用 `miniz_oxide`（纯 Rust），不依赖系统 zlib
- glibc 兼容性由构建环境决定（当前 CI: glibc 2.34+）

## Test

```bash
cargo test -- --test-threads=1   # 全局共享状态，必须单线程
bash test_lsp.sh                  # LSP 端到端 (27 items)
```

## LSP Protocol

- `lsp-server 0.7` / `lsp-types 0.96`，stdio JSON-RPC
- 全局单例：`WORKSPACE`、`WAL_PARSER`、`FORMAT_OPTS`、`CONFIG` = `LazyLock`
- `RwLock`/`Mutex` 中毒用 `.unwrap_or_else(|e| e.into_inner())`
- lock 顺序：`WORKSPACE → WAL_PARSER`（反序会死锁）

## Golden Alignment

- 操作符以 golden (Python `wal/ast_defs.py`) 为准
- wal-rust 扩展可保留，需标注（如 `string-append — wal-rust 扩展`）
- `KNOWN_ARITIES` 中变参（`body+`、`str...`）不设固定 arity
- grammar.js: `_comment: () => /;.*/`（单分号兼容 golden）

## Release

```bash
git tag v<version> && git push origin v<version>
gh release create v<version> --title "v<version>" --notes "..." \
  <binary>#wal-lsp-linux-x86_64 <archive>#wal-lsp-linux-x86_64.tar.gz
```
