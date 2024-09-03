# Set environment variable
export CARGO_MAKE_EXTEND_WORKSPACE_MAKEFILE := "true"

# Run rustfmt to check the code formatting without making changes
format:
    cargo fmt -- --check

# Clean up the project by removing the target directory
clean:
    cargo clean

# Run clippy to catch common mistakes and improve your Rust code
clippy:
    cargo clippy --all-targets --all-features -- -Dwarnings

# Execute all unit tests in the workspace
test:
    cargo llvm-cov nextest --features test_utils

# Run the entire CI pipeline including format, clippy, and test checks
run-ci-flow: format clippy test
    @echo "CI flow completed"
