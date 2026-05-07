# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Principles

1. Don't assume. Don't hide confusion. Surface tradeoffs.
2. Minimum code that solves the problem. Nothing speculative.
3. Touch only what you must. Clean up only your own mess.
4. Define success criteria. Loop until verified.

## Project intent

This is a **forkable Rust MCP server template**. It ships with two
transports, one example tool (`ping`), real tests, and CI. Fork this
repo to start a new MCP server.

The crate uses Rust edition **2024** and depends on:
- `rmcp` 1.6.x with `server`, `macros`, `transport-io`, `transport-streamable-http-server` — the MCP server SDK.
- `tokio` (multi-threaded runtime, `signal`, `net`, `io-std`).
- `clap` (derive + env) for CLI, `schemars` 1.x for tool schemas, `tracing` + `tracing-subscriber` for observability, `serde`/`serde_json` for payloads, `anyhow` for application-boundary errors.
- `hyper` + `hyper-util` + `tower-service` for streamable-HTTP.

When extending this project, prefer the `rmcp` server abstractions
over hand-rolling JSON-RPC. Use Context7
(`mcp__context7__resolve-library-id` → `query-docs` against the
`rmcp` library id) to pull current API docs before adding handlers —
the crate is young and APIs shift between minor versions.

## Architecture

```
src/
  main.rs               # parse CLI, init tracing, call server::run
  lib.rs                # re-exports for tests and downstream consumers
  config.rs             # Config + Cli (clap) + Transport / LogFormat enums
  observability.rs      # tracing-subscriber init; stderr only
  server/
    mod.rs              # run(config) -> dispatches to transport::serve
    handler.rs          # ServerHandler impl + #[tool_router] + #[tool_handler]
  tools/
    mod.rs              # tool re-exports + map_err helper (anyhow → ErrorData)
    ping.rs             # the example tool — copy as the template for new tools
  transport/
    mod.rs              # Transport enum dispatch
    stdio.rs            # rmcp::transport::io::stdio() bring-up
    http.rs             # StreamableHttpService + LocalSessionManager + hyper-util
tests/
  ping_tool.rs          # in-process round-trip via tokio::io::duplex
```

Conventions:

- **Tracing → stderr only.** Stdout is reserved for MCP JSON-RPC frames under stdio.
- **Error mapping at one site.** Tool handlers convert `anyhow::Error` → `rmcp::ErrorData` exclusively through `tools::map_err`, which preserves the full `.context(...)` chain.
- **Adding a tool.** Create `src/tools/<name>.rs` with `Args` + `Output` types; declare it from `tools/mod.rs`; add a `#[tool(...)]` method on the handler that delegates to the pure handler in the new file.
- **Tests live in three layers.** Unit tests in `src/tools/<name>.rs` and `src/server/handler.rs`, an integration test in `tests/ping_tool.rs` that drives a round-trip via `tokio::io::duplex`. Forkers should copy this pattern when adding a real tool.

## Commands

```bash
cargo build              # debug build
cargo run                # stdio (default transport)
cargo run -- --transport http   # streamable-HTTP on 127.0.0.1:8080
cargo clippy --all-targets -- -D warnings   # CI gate
cargo fmt --all -- --check                  # CI gate
cargo test --all-targets                    # CI gate

# Or via the justfile:
just run | run-http | test | lint | fmt | ci
```

CI runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` on every push and PR (`.github/workflows/ci.yml`).
