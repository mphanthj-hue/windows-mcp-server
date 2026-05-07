use std::net::SocketAddr;

use clap::{Parser, ValueEnum};

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum Transport {
    Stdio,
    Http,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum LogFormat {
    Pretty,
    Json,
}

/// Resolved server configuration after CLI/env merge.
#[derive(Clone, Debug)]
pub struct Config {
    pub transport: Transport,
    pub bind: SocketAddr,
    pub log_format: LogFormat,
}

#[derive(Parser, Debug)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    version,
    about = "Forkable Rust MCP server template (stdio + streamable-HTTP)."
)]
pub struct Cli {
    /// Transport to serve MCP over.
    #[arg(long, value_enum, env = "MCP_TRANSPORT", default_value_t = Transport::Stdio)]
    pub transport: Transport,

    /// HTTP bind address (only used when --transport=http).
    #[arg(long, env = "MCP_BIND", default_value = "127.0.0.1:8080")]
    pub bind: SocketAddr,

    /// Log output format.
    #[arg(long, value_enum, env = "MCP_LOG_FORMAT", default_value_t = LogFormat::Pretty)]
    pub log_format: LogFormat,
}

impl From<Cli> for Config {
    fn from(cli: Cli) -> Self {
        Self {
            transport: cli.transport,
            bind: cli.bind,
            log_format: cli.log_format,
        }
    }
}

impl Cli {
    /// Parse CLI from process args. Exits the process on parse failure.
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use clap::Parser;

    #[test]
    fn defaults_match_design() {
        let cli = Cli::try_parse_from(["rust-mcp-server-template"]).unwrap();
        let cfg = Config::from(cli);
        assert_eq!(cfg.transport, Transport::Stdio);
        assert_eq!(cfg.log_format, LogFormat::Pretty);
        assert_eq!(cfg.bind.to_string(), "127.0.0.1:8080");
    }

    #[test]
    fn cli_selects_http_transport() {
        let cli = Cli::try_parse_from(["rust-mcp-server-template", "--transport", "http"]).unwrap();
        assert_eq!(cli.transport, Transport::Http);
    }

    #[test]
    fn cli_overrides_bind() {
        let cli = Cli::try_parse_from([
            "rust-mcp-server-template",
            "--transport",
            "http",
            "--bind",
            "0.0.0.0:9000",
        ])
        .unwrap();
        assert_eq!(cli.bind.to_string(), "0.0.0.0:9000");
    }

    #[test]
    fn cli_selects_json_log_format() {
        let cli =
            Cli::try_parse_from(["rust-mcp-server-template", "--log-format", "json"]).unwrap();
        assert_eq!(cli.log_format, LogFormat::Json);
    }

    #[test]
    fn unknown_transport_is_rejected() {
        let err = Cli::try_parse_from(["rust-mcp-server-template", "--transport", "websocket"])
            .unwrap_err();
        // clap exits non-zero on parse error; this proves we never reach runtime.
        let rendered = err.to_string();
        assert!(
            rendered.contains("websocket") || rendered.contains("invalid value"),
            "expected error to mention invalid value, got: {rendered}"
        );
    }
}
