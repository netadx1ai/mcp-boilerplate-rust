# MCP Boilerplate Rust - Makefile
# MCP v5 using official rust-sdk (stdio primary, HTTP optional)

.PHONY: help build build-http run run-stdio dev dev-stdio test clean fmt lint check install release watch watch-stdio check-size

# Default target
.DEFAULT_GOAL := help

help: ## Show this help message
	@echo "MCP Boilerplate Rust v0.3.0 - Available Commands"
	@echo ""
	@echo "Primary (Stdio):"
	@grep -E '^(run-stdio|dev-stdio|watch-stdio):.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "HTTP (Optional - requires 'http' feature):"
	@grep -E '^(run-http|build-http):.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Development:"
	@grep -E '^(build|test|fmt|lint|check|clean):.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Tools:"
	@grep -E '^(check-size|install|release):.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""

install: ## Install dependencies
	@echo "Installing dependencies..."
	cargo fetch
	@echo "Done!"

build: ## Build the project (stdio only)
	@echo "Building MCP server (stdio mode)..."
	cargo build
	@echo "Build complete!"

build-http: ## Build with HTTP feature
	@echo "Building MCP server (with HTTP support)..."
	cargo build --features http
	@echo "Build complete with HTTP!"

release: ## Build optimized release binary (stdio)
	@echo "Building release binary (stdio mode)..."
	cargo build --release
	@echo "Release binary: target/release/mcp-boilerplate-rust"
	@echo "Ready for Claude Desktop integration!"

release-http: ## Build release with HTTP feature
	@echo "Building release binary (with HTTP)..."
	cargo build --release --features http
	@echo "Release binary: target/release/mcp-boilerplate-rust"

run: run-stdio ## Run the server in stdio mode (default)

run-stdio: ## Run the server in stdio mode
	@echo "Starting MCP server (stdio mode)..."
	@echo "Press Ctrl+D to send EOF and test, or Ctrl+C to exit"
	cargo run -- --mode stdio

run-http: ## Run the server in HTTP mode (requires http feature)
	@echo "Starting MCP server (HTTP mode)..."
	cargo run --features http -- --mode http

dev: dev-stdio ## Run with detailed logging (stdio mode, default)

dev-stdio: ## Run stdio mode with detailed logging
	@echo "Starting MCP server (stdio, debug mode)..."
	@echo "RUST_LOG=debug,mcp_boilerplate_rust=trace"
	cargo run -- --mode stdio --verbose

dev-http: ## Run HTTP mode with detailed logging
	@echo "Starting MCP server (HTTP, debug mode)..."
	RUST_LOG=debug,mcp_boilerplate_rust=trace cargo run --features http -- --mode http

watch: watch-stdio ## Watch mode (stdio, default)

watch-stdio: ## Run with auto-reload in stdio mode (requires cargo-watch)
	@command -v cargo-watch >/dev/null 2>&1 || { echo "Installing cargo-watch..."; cargo install cargo-watch; }
	@echo "Starting server with auto-reload (stdio)..."
	cargo watch -x "run -- --mode stdio"

watch-http: ## Run with auto-reload in HTTP mode (requires cargo-watch)
	@command -v cargo-watch >/dev/null 2>&1 || { echo "Installing cargo-watch..."; cargo install cargo-watch; }
	@echo "Starting server with auto-reload (HTTP)..."
	cargo watch -x "run --features http -- --mode http"

test: ## Run tests
	@echo "Running tests..."
	cargo test --all-features
	@echo "Tests complete!"

test-stdio: ## Test stdio mode with MCP Inspector
	@echo "Testing stdio mode with MCP Inspector..."
	@command -v mcp-inspector >/dev/null 2>&1 || { echo "Installing MCP Inspector..."; npm install -g @modelcontextprotocol/inspector; }
	mcp-inspector cargo run -- --mode stdio

test-http: ## Test HTTP endpoints with curl (requires http feature)
	@echo "Testing HTTP endpoints..."
	@chmod +x test.sh
	./test.sh

fmt: ## Format code
	@echo "Formatting code..."
	cargo fmt
	@echo "Code formatted!"

lint: ## Run clippy linter
	@echo "Running clippy..."
	cargo clippy -- -D warnings
	@echo "Linting complete!"

check: ## Check code without building
	@echo "Checking code..."
	cargo check
	@echo "Check complete!"

check-size: ## Check file sizes (max 500 lines per file)
	@echo "Checking file sizes (max 500 lines)..."
	@chmod +x scripts/check-file-sizes.sh
	@./scripts/check-file-sizes.sh

clean: ## Clean build artifacts
	@echo "Cleaning build artifacts..."
	cargo clean
	@echo "Clean complete!"

setup: ## Initial project setup
	@echo "Setting up MCP project..."
	@if [ ! -f .env ]; then cp .env.example .env; echo ".env file created"; fi
	@cargo build
	@echo "Setup complete!"
	@echo ""
	@echo "Quick start:"
	@echo "  make run-stdio    # Run stdio mode (for Claude Desktop)"
	@echo "  make release      # Build production binary"
	@echo ""

all: check-size fmt lint test build ## Run all quality checks and build

claude-setup: release ## Build and show Claude Desktop config
	@echo "Building release binary for Claude Desktop..."
	@cargo build --release
	@echo ""
	@echo "Add this to your Claude Desktop config:"
	@echo '{'
	@echo '  "mcpServers": {'
	@echo '    "mcp-boilerplate-rust": {'
	@echo '      "command": "'$(PWD)'/target/release/mcp-boilerplate-rust",'
	@echo '      "args": ["--mode", "stdio"],'
	@echo '      "env": {'
	@echo '        "RUST_LOG": "info"'
	@echo '      }'
	@echo '    }'
	@echo '  }'
	@echo '}'
	@echo ""
	@echo "Config location (macOS):"
	@echo "  ~/Library/Application Support/Claude/claude_desktop_config.json"

stats: ## Show project statistics
	@echo "MCP Boilerplate Rust - Project Statistics"
	@echo ""
	@echo "Lines of code:"
	@find src -name "*.rs" | xargs wc -l | tail -1
	@echo ""
	@echo "Rust files:"
	@find src -name "*.rs" | wc -l
	@echo ""
	@echo "File size check:"
	@make check-size

info: ## Show project information
	@echo "MCP Boilerplate Rust"
	@echo "Version: 0.3.0"
	@echo "Protocol: MCP v5 (native stdio)"
	@echo "SDK: rmcp v0.12"
	@echo ""
	@echo "Environment:"
	@echo "  Rust: $$(rustc --version)"
	@echo "  Cargo: $$(cargo --version)"
	@echo ""
	@echo "Quick Commands:"
	@echo "  make run-stdio      # Run stdio mode (default)"
	@echo "  make dev-stdio      # Debug mode"
	@echo "  make release        # Build for production"
	@echo "  make claude-setup   # Setup for Claude Desktop"
	@echo ""
	@echo "Documentation:"
	@echo "  docs/NATIVE_STDIO_GUIDE.md - Complete stdio guide"
	@echo "  docs/AI_TOOL_PATTERN.md    - Tool development patterns"