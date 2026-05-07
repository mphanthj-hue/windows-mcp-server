set shell := ["bash", "-cu"]

# Run on stdio (default transport)
run:
    cargo run

# Run on streamable-HTTP, loopback:8080
run-http:
    cargo run -- --transport http

# Run all tests
test:
    cargo test --all-targets

# Lint with clippy, warnings as errors
lint:
    cargo clippy --all-targets -- -D warnings

# Format
fmt:
    cargo fmt --all

# Same gates CI runs
ci:
    cargo fmt --all -- --check
    cargo clippy --all-targets -- -D warnings
    cargo test --all-targets
