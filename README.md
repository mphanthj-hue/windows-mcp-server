# rust-mcp-server-template

A forkable starting point for [Model Context Protocol](https://modelcontextprotocol.io)
servers in Rust. Single binary, two transports (stdio + streamable-HTTP),
one example tool, real tests, real CI. Built on the
[`rmcp`](https://crates.io/crates/rmcp) SDK.

## Quickstart

```bash
# stdio (default — suitable for Claude Desktop / Claude Code / any local MCP client)
cargo run

# streamable-HTTP, loopback:8080
cargo run -- --transport http
```

## Using this template

Click **Use this template** on GitHub to create a new repo from this
scaffold, or run:

```bash
gh repo create my-mcp-server --template shoehn/rust-mcp-server-template --public
```

After creating your repo:

1. Rename the crate in `Cargo.toml` (`[package].name`, `repository`).
2. Rename `McpServerHandler` in `src/server/handler.rs` (and its re-exports in `src/server/mod.rs` and `src/lib.rs`) to match your server's domain.
3. Update this README.
4. Replace the `ping` example tool in `src/tools/` with your real tools.

## Adding a tool

1. Create `src/tools/<name>.rs` with `Args` + `Output` types deriving
   `Deserialize+JsonSchema` / `Serialize+JsonSchema`.
2. Add a `pub mod <name>;` line in `src/tools/mod.rs`.
3. Add a `#[tool(name = "...", description = "...")]` method on
   your handler struct in `src/server/handler.rs` that delegates to the
   pure handler in your new file.

The `#[tool_router]` macro picks it up automatically — no other wiring.

## CLI

| Flag                        | Env var          | Default          |
| --------------------------- | ---------------- | ---------------- |
| `--transport stdio\|http`   | `MCP_TRANSPORT`  | `stdio`          |
| `--bind <addr>`             | `MCP_BIND`       | `127.0.0.1:8080` |
| `--log-format pretty\|json` | `MCP_LOG_FORMAT` | `pretty`         |

## Development

```bash
just test    # cargo test --all-targets
just lint    # cargo clippy --all-targets -- -D warnings
just fmt     # cargo fmt --all
just ci      # fmt --check + clippy + test
```

## License

[AGPL-3.0-or-later](LICENSE)
