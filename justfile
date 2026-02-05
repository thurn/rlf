set positional-arguments

# RLF - Rust Localization Framework

review: check-format no-inline-tests check clippy test

# Check that no #[test] attributes exist in src/
# Exception: rlf-macros (proc-macro crates can't have external integration tests)
no-inline-tests:
    #!/usr/bin/env bash
    if grep -r '#\[test\]' crates/*/src/ 2>/dev/null | grep -v 'rlf-macros'; then
        echo "Error: #[test] found in src/ directories"
        exit 1
    else
        echo "No inline tests"
    fi

# Check all workspace crates (quiet on success)
check:
    #!/usr/bin/env bash
    output=$(cargo check --workspace --all-targets 2>&1)
    if [ $? -eq 0 ]; then
        echo "Check passed"
    else
        echo "$output"
        exit 1
    fi

# Check all workspace crates (verbose)
check-verbose:
    cargo check --workspace --all-targets

# Build all workspace crates (quiet on success)
build:
    #!/usr/bin/env bash
    output=$(cargo build --workspace --all-targets 2>&1)
    if [ $? -eq 0 ]; then
        echo "Build passed"
    else
        echo "$output"
        exit 1
    fi

# Build all workspace crates (verbose)
build-verbose:
    cargo build --workspace --all-targets

# Run all tests (quiet on success)
test:
    #!/usr/bin/env bash
    output=$(cargo test --workspace 2>&1)
    if [ $? -eq 0 ]; then
        echo "Tests passed"
    else
        echo "$output"
        exit 1
    fi

# Run all tests (verbose)
test-verbose:
    cargo test --workspace

# Run clippy lints (quiet on success)
clippy:
    #!/usr/bin/env bash
    output=$(cargo clippy --workspace --all-targets -- -D warnings -D clippy::all 2>&1)
    if [ $? -eq 0 ]; then
        echo "Clippy passed"
    else
        echo "$output"
        exit 1
    fi

# Run clippy lints (verbose)
clippy-verbose:
    cargo clippy --workspace --all-targets -- -D warnings -D clippy::all

# Format all code
fmt:
    cargo +nightly fmt --all

# Check formatting (quiet on success)
check-format:
    #!/usr/bin/env bash
    output=$(cargo +nightly fmt --all -- --check 2>&1)
    if [ $? -eq 0 ]; then
        echo "Format OK"
    else
        echo "$output"
        exit 1
    fi

# Check formatting (verbose)
check-format-verbose:
    cargo +nightly fmt --all -- --check

# Clean build artifacts
clean:
    cargo clean
