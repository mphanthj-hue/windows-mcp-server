//! Server bring-up: builds the handler and dispatches to the chosen
//! transport.

pub mod handler;

pub use handler::McpServerHandler;

use anyhow::Result;

use crate::config::Config;

/// Build the handler and serve until the transport completes (graceful
/// shutdown on SIGINT, or transport-fatal error).
pub async fn run(config: Config) -> Result<()> {
    let handler = McpServerHandler::new();
    crate::transport::serve(handler, config).await
}
