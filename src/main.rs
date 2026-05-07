mod lsp;
mod wal;
pub mod workspace;

use clap::Parser;
use anyhow::Result;
use tracing::info;

#[derive(Parser)]
#[command(name = "wal-lsp")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(
    about = "WAL Language Server Protocol implementation",
    long_about = "LSP server for WAL (Waveform Analysis Language).\n\n\
                  Provides intelligent editing features for .wal files:\n  \
                  - Code completion (builtins, signals, scopes, macros)\n  \
                  - Hover documentation for operators and functions\n  \
                  - Go-to-definition for variables and functions\n  \
                  - Document symbols and diagnostics\n  \
                  - Waveform signal name completion",
    after_help = "The server communicates via stdio using the Language Server Protocol.\n\
                  Configure your editor to use 'wal-lsp' for .wal files.\n\n\
                  EDITOR SETUP:\n  \
                  VS Code: install the WAL extension and set wal-lsp.path\n  \
                  Neovim: add to lspconfig with cmd = {'wal-lsp'}\n  \
                  Helix: add to languages.toml with command = 'wal-lsp'"
)]
struct Cli;

fn main() -> Result<()> {
    let _ = Cli::parse();

    tracing_subscriber::fmt::init();

    info!("Starting WAL LSP server");
    lsp::run()?;

    Ok(())
}
