# RLF - Rust Localization Framework

# Check all workspace crates
check:
    cargo check --workspace

# Run all tests
test:
    cargo test --workspace

# Build all workspace crates
build:
    cargo build --workspace

# Format all code
fmt:
    cargo fmt --all

# Run clippy lints
lint:
    cargo clippy --workspace -- -D warnings
