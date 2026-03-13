// capiba-mcp v0.1 — servidor MCP usando o SDK oficial rmcp

mod flags;
mod prompts;
mod resources;
mod server;
mod tools;

use anyhow::Result;
use rmcp::{transport::io::stdio, ServiceExt};
use server::CapibaMcp;

// ─── Main ─────────────────────────────────────────────────────────────────────
// LCOV_EXCL_START — não testável sem servidor MCP ativo

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Verificar flags de ajuda e versão antes de iniciar o servidor
    if flags::check_help_version_flags(&args).is_some() {
        return Ok(());
    }

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let service = CapibaMcp::new()
        .serve(stdio())
        .await
        .inspect_err(|e| eprintln!("error: {}", e))?;

    service.waiting().await?;
    Ok(())
}
// LCOV_EXCL_STOP
