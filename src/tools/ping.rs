//! Example "ping" tool — the smoke-test tool that ships with the template.
//!
//! Handles a single optional `message` and echoes it back. Forks copy
//! this file as the starting template for a real tool: input args derive
//! `Deserialize + JsonSchema`, output derives `Serialize + JsonSchema`,
//! and the `#[tool]` macro registers it with the router.

use rmcp::handler::server::wrapper::{Json, Parameters};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct PingArgs {
    /// Optional message to echo back. Absent → returns the default echo.
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct PingOutput {
    /// The echoed message, or `"pong"` if no message was supplied.
    pub echo: String,
}

/// The default echo when no message is supplied.
pub const DEFAULT_ECHO: &str = "pong";

/// Pure handler — separated from the `#[tool]`-annotated method so the
/// unit test can drive it directly without spinning up a router.
pub fn ping_inner(args: PingArgs) -> PingOutput {
    PingOutput {
        echo: args.message.unwrap_or_else(|| DEFAULT_ECHO.to_string()),
    }
}

/// Tool entry point used by the router.
#[tracing::instrument(skip_all)]
pub fn ping(Parameters(args): Parameters<PingArgs>) -> Json<PingOutput> {
    Json(ping_inner(args))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn echoes_provided_message() {
        let out = ping(Parameters(PingArgs {
            message: Some("hello".into()),
        }));
        assert_eq!(out.0.echo, "hello");
    }

    #[test]
    fn defaults_when_message_absent() {
        let out = ping(Parameters(PingArgs::default()));
        assert_eq!(out.0.echo, DEFAULT_ECHO);
    }
}
