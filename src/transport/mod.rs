//! Transport dispatch — selects between stdio and streamable-HTTP at runtime.

pub mod http;
pub mod stdio;

use anyhow::Result;

use crate::config::{Config, Transport};
use crate::server::McpServerHandler;

pub async fn serve(handler: McpServerHandler, config: Config) -> Result<()> {
    match config.transport {
        Transport::Stdio => stdio::serve(handler).await,
        Transport::Http => http::serve(handler, config.bind).await,
    }
}
