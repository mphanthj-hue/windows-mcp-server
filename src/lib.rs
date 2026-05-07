//! Library surface for the `rust-mcp-server-template` crate.
//!
//! Re-exports everything `tests/` and downstream code may need so that
//! integration tests can drive the server without subprocess gymnastics.

pub mod config;
pub mod observability;
pub mod server;
pub mod tools;
pub mod transport;

pub use config::{Cli, Config, LogFormat, Transport};
pub use server::McpServerHandler;
