//! Stdio transport bring-up.
//!
//! `rmcp::transport::io::stdio()` returns a `(stdin, stdout)` pair that
//! `ServiceExt::serve_with_ct` accepts directly. Stdout is reserved for
//! the MCP JSON-RPC stream — we never write logs there (see
//! `observability::init`, which hard-codes stderr).

use anyhow::{Context, Result};
use rmcp::ServiceExt;
use rmcp::transport::io::stdio;
use tokio_util::sync::CancellationToken;

use crate::server::McpServerHandler;

pub async fn serve(handler: McpServerHandler) -> Result<()> {
    let ct = CancellationToken::new();
    let shutdown_ct = ct.clone();

    // Cancel on Ctrl-C; the running service tears down cleanly.
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            tracing::info!("ctrl-c received, shutting down stdio transport");
            shutdown_ct.cancel();
        }
    });

    tracing::info!("starting stdio transport");

    let running = handler
        .serve_with_ct(stdio(), ct)
        .await
        .context("stdio transport")?;
    let reason = running.waiting().await.context("stdio transport")?;
    tracing::info!(?reason, "stdio transport terminated");
    Ok(())
}
