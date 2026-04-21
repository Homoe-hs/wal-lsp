mod lsp;
mod mcp;
mod wal;
pub mod workspace;

use anyhow::Result;
use clap::Parser;
use tracing::info;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, help = "Run as MCP server instead of LSP")]
    mcp: bool,
    #[arg(long, help = "Run as LSP server")]
    lsp: bool,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    if args.mcp {
        info!("Starting WAL MCP server");
        mcp::run()?;
    } else {
        info!("Starting WAL LSP server");
        lsp::run()?;
    }

    Ok(())
}
