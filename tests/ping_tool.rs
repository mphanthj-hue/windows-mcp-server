//! Integration test: round-trip the `ping` tool through an in-process
//! MCP client/server pair, using a `tokio::io::duplex` pipe as the
//! transport. This is the test forkers should copy when adding their
//! first real tool.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use rmcp::{
    ServiceExt,
    model::{CallToolRequestParams, RawContent},
};
use windows_mcp_server::McpServerHandler;
use tokio::io::duplex;

#[tokio::test]
async fn round_trip_ping_call() {
    // 64KB buffer per side is plenty for a smoke-test handshake + one tool call.
    let (server_io, client_io) = duplex(64 * 1024);

    // Server side: spin up the real handler on the duplex pipe.
    let server_task = tokio::spawn(async move {
        let handler = McpServerHandler::new();
        let running = handler
            .serve(server_io)
            .await
            .expect("server initialize should succeed");
        running
            .waiting()
            .await
            .expect("server should terminate without join error")
    });

    // Client side: empty `()` client (no client-side handlers needed).
    let client = ().serve(client_io).await.expect("client initialize should succeed");

    // 1. list_tools sees ping
    let listed = client
        .list_tools(None)
        .await
        .expect("list_tools should succeed");
    let names: Vec<&str> = listed.tools.iter().map(|t| t.name.as_ref()).collect();
    assert!(
        names.contains(&"ping"),
        "ping tool should be advertised; saw {names:?}"
    );

    // 2. call_tool("ping", { "message": "hello" })
    let args = serde_json::json!({ "message": "hello" })
        .as_object()
        .unwrap()
        .clone();
    let res = client
        .call_tool(CallToolRequestParams::new("ping").with_arguments(args))
        .await
        .expect("call_tool should succeed");

    assert!(
        !res.is_error.unwrap_or(false),
        "tool result should not be an error: {res:?}"
    );

    // The handler returns `Json<PingOutput>`, which rmcp serializes into
    // the `content` array as a text block whose `text` is the JSON
    // payload, plus `structured_content` for clients that prefer it.
    let text = res
        .content
        .iter()
        .find_map(|c| match &c.raw {
            RawContent::Text(t) => Some(t.text.clone()),
            _ => None,
        })
        .expect("expected at least one text content block");

    assert!(
        text.contains("hello"),
        "echoed text should contain the input message; got: {text}"
    );

    // Tear down: cancel the client, which closes the duplex pipe and
    // unblocks the server.
    client.cancel().await.ok();
    let _ = server_task.await;
}
