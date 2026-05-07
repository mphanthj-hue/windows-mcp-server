//! `ServerHandler` — the rmcp service implementation that owns the
//! template's tool set.
//!
//! Adding a new tool: drop a file in `src/tools/`, declare it in
//! `tools/mod.rs`, then add a `#[tool]` method here that delegates to
//! it. The router macro picks it up automatically — no other wiring.

use rmcp::{
    ServerHandler,
    handler::server::{
        router::tool::ToolRouter,
        wrapper::{Json, Parameters},
    },
    model::{Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};

use crate::tools::ping::{PingArgs, PingOutput, ping_inner};

#[derive(Clone, Default)]
pub struct McpServerHandler {
    // Read by the `#[tool_handler]` macro's generated impl; the lint
    // pass can't see through that, hence the allow.
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl McpServerHandler {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    /// Smoke-test echo tool. Returns `{ "echo": message ?? "pong" }`.
    #[tool(
        name = "ping",
        description = "Smoke-test tool that echoes the optional `message` argument, or returns \"pong\" if none was provided."
    )]
    #[tracing::instrument(skip_all)]
    pub fn ping(&self, Parameters(args): Parameters<PingArgs>) -> Json<PingOutput> {
        Json(ping_inner(args))
    }
}

#[tool_handler]
impl ServerHandler for McpServerHandler {
    fn get_info(&self) -> ServerInfo {
        // `ServerInfo` and `Implementation` are `#[non_exhaustive]`, so
        // we start from `Default::default()` and assign individual fields
        // — struct expressions (even with `..base`) aren't allowed for
        // these from outside `rmcp`.
        let mut server_info_struct = Implementation::default();
        server_info_struct.name = env!("CARGO_PKG_NAME").to_string();
        server_info_struct.version = env!("CARGO_PKG_VERSION").to_string();

        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.server_info = server_info_struct;
        info.instructions = Some(
            "Forkable Rust MCP server template. The example `ping` tool is a smoke-test; \
             replace it (or add new tools alongside) under `src/tools/`."
                .into(),
        );
        info
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn advertises_ping_tool() {
        let handler = McpServerHandler::new();
        let tools = handler.tool_router.list_all();

        let ping = tools
            .iter()
            .find(|t| t.name == "ping")
            .expect("ping tool must be advertised");

        let desc = ping
            .description
            .as_ref()
            .expect("ping tool must have a description");
        assert!(!desc.is_empty(), "ping description must be non-empty");
    }

    #[test]
    fn ping_route_is_registered() {
        let handler = McpServerHandler::new();
        assert!(handler.tool_router.has_route("ping"));
    }
}
