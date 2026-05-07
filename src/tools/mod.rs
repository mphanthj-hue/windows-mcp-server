//! Tool implementations.
//!
//! This is the module forkers will edit most. Each tool lives in its own
//! file; `mod.rs` re-exports them and provides the shared `map_err`
//! helper that turns an `anyhow::Error` (with all its `.context(...)`
//! layers) into the `rmcp::ErrorData` type expected at handler return
//! sites.

pub mod ping;

use rmcp::ErrorData;

/// Convert an `anyhow::Error` into the `rmcp` tool error type, preserving
/// the full `.context(...)` chain in the message.
///
/// `anyhow`'s `Display` only prints the outermost context. We use
/// `{:#}` (alternate form) so every layer surfaces in the message a
/// client will see, separated by `: `. This is the single canonical site
/// for `anyhow → ErrorData` mapping; tool handlers route through it
/// rather than constructing `ErrorData` ad-hoc.
pub fn map_err(err: &anyhow::Error) -> ErrorData {
    ErrorData::internal_error(format!("{err:#}"), None)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use anyhow::anyhow;

    #[test]
    fn preserves_context_chain() {
        let err: anyhow::Error = anyhow!("inner failure")
            .context("middle layer")
            .context("outer context");

        let mapped = map_err(&err);
        let msg = mapped.message.to_string();

        assert!(
            msg.contains("inner failure"),
            "expected inner cause in message, got: {msg}"
        );
        assert!(
            msg.contains("outer context"),
            "expected outermost context in message, got: {msg}"
        );
    }

    #[test]
    fn preserves_single_message() {
        let err = anyhow!("something failed");
        let mapped = map_err(&err);
        assert!(mapped.message.contains("something failed"));
    }
}
