//! Tracing subscriber bring-up.
//!
//! Always writes to **stderr**: under the stdio transport, stdout carries
//! MCP JSON-RPC frames and any non-protocol byte on stdout corrupts the stream.

use anyhow::Result;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::config::LogFormat;

fn env_filter() -> EnvFilter {
    EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap_or_else(|_| EnvFilter::new("info"))
}

/// Install a tracing subscriber for the chosen log format.
///
/// Always writes to stderr. Subsequent calls in the same process are a
/// no-op rather than a panic — `set_global_default` rejects the second
/// install, and we swallow that.
pub fn init(format: LogFormat) -> Result<()> {
    let filter = env_filter();

    let registry = tracing_subscriber::registry().with(filter);

    let res = match format {
        LogFormat::Pretty => {
            let layer = fmt::layer().with_writer(std::io::stderr).with_target(true);
            registry.with(layer).try_init()
        }
        LogFormat::Json => {
            let layer = fmt::layer()
                .json()
                .flatten_event(true)
                .with_current_span(true)
                .with_span_list(false)
                .with_writer(std::io::stderr);
            registry.with(layer).try_init()
        }
    };

    // try_init returns Err if a global subscriber is already installed.
    // For our purposes (tests, double-init), that's not fatal.
    let _ = res;
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn init_is_idempotent() {
        // Two back-to-back calls must not panic — even if the global
        // dispatcher is already set.
        init(LogFormat::Pretty).unwrap();
        init(LogFormat::Json).unwrap();
    }
}
