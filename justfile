# MCP Boilerplate Rust - Development Tasks

# Default recipe
default:
    @just --list

# Build all servers
build:
    cargo build --workspace --all-targets

# Build for production
build-release:
    cargo build --workspace --release

# Run all tests
test:
    cargo test --workspace

# Run tests with nextest (if available)
test-fast:
    cargo nextest run --workspace

# Run integration tests
test-integration:
    cargo test --test integration

# Run performance tests
test-performance:
    cargo test --test performance

# Run security tests
test-security:
    cargo test --test security

# Format code
fmt:
    cargo fmt --all

# Lint code
lint:
    cargo clippy --workspace --all-targets

# Check code quality
check: fmt lint test

# Generate documentation
docs:
    cargo doc --workspace --no-deps --open

# Watch for changes and rebuild
watch:
    cargo watch -x "check --workspace"

# Watch and run tests
watch-test:
    cargo watch -x "test --workspace"

# Audit dependencies
audit:
    cargo audit
    cargo deny check

# Run a specific server
run-server server:
    cargo run --bin {{server}}

# Run filesystem server (example)
run-fs:
    cargo run --bin filesystem-server

# Clean build artifacts
clean:
    cargo clean

# Setup development environment
setup:
    ./scripts/setup-development.sh

# Benchmark all servers
bench:
    cargo bench

# Check all (comprehensive)
check-all: fmt lint test audit docs
    @echo "✅ All checks passed!"
