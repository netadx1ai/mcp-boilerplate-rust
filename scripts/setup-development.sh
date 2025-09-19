#!/bin/bash

# MCP Boilerplate Rust - Development Environment Setup
# Official SDK Pivot Edition
#
# This script sets up the development environment for building production-ready
# MCP servers using the official RMCP SDK.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Project info
PROJECT_NAME="MCP Boilerplate Rust"
REQUIRED_RUST_VERSION="1.75.0"
RMCP_VERSION="0.6.3"

echo -e "${BLUE}🚀 ${PROJECT_NAME} - Development Setup${NC}"
echo -e "${BLUE}   Official RMCP SDK v${RMCP_VERSION} Edition${NC}"
echo ""

# Function to print status
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Check if running in project directory
if [[ ! -f "Cargo.toml" ]]; then
    print_error "Must run from project root directory (where Cargo.toml exists)"
    exit 1
fi

# Check Rust installation
check_rust() {
    print_info "Checking Rust installation..."
    
    if ! command -v rustc &> /dev/null; then
        print_error "Rust is not installed. Install from https://rustup.rs/"
        exit 1
    fi
    
    RUST_VERSION=$(rustc --version | awk '{print $2}')
    print_status "Rust $RUST_VERSION detected"
    
    # Check minimum version (simplified check)
    if [[ "$(printf '%s\n' "$REQUIRED_RUST_VERSION" "$RUST_VERSION" | sort -V | head -n1)" != "$REQUIRED_RUST_VERSION" ]]; then
        print_warning "Rust $REQUIRED_RUST_VERSION+ recommended (found $RUST_VERSION)"
    fi
}

# Check Cargo tools
check_cargo_tools() {
    print_info "Checking development tools..."
    
    # Required tools
    TOOLS=(
        "cargo-watch"
        "cargo-audit" 
        "cargo-deny"
        "cargo-nextest"
    )
    
    MISSING_TOOLS=()
    
    for tool in "${TOOLS[@]}"; do
        if ! cargo "$tool" --version &> /dev/null; then
            MISSING_TOOLS+=("$tool")
        else
            print_status "$tool installed"
        fi
    done
    
    if [[ ${#MISSING_TOOLS[@]} -gt 0 ]]; then
        print_info "Installing missing tools: ${MISSING_TOOLS[*]}"
        for tool in "${MISSING_TOOLS[@]}"; do
            cargo install "$tool"
            print_status "Installed $tool"
        done
    fi
}

# Check system dependencies
check_system_deps() {
    print_info "Checking system dependencies..."
    
    # Check for essential tools
    TOOLS=(
        "git"
        "curl" 
        "docker"
    )
    
    for tool in "${TOOLS[@]}"; do
        if command -v "$tool" &> /dev/null; then
            print_status "$tool available"
        else
            print_warning "$tool not found (recommended for full development experience)"
        fi
    done
}

# Validate official SDK integration
validate_sdk() {
    print_info "Validating official RMCP SDK integration..."
    
    # Check if rmcp is in dependencies
    if grep -q "rmcp.*0\.6\.3" Cargo.toml; then
        print_status "Official RMCP SDK v0.6.3 configured"
    else
        print_warning "RMCP SDK dependency not found or wrong version"
        print_info "Adding official RMCP SDK dependency..."
        
        # Backup Cargo.toml
        cp Cargo.toml Cargo.toml.backup
        
        # Note: In practice, this would need more sophisticated TOML editing
        print_info "Please manually verify RMCP SDK dependency in Cargo.toml"
    fi
}

# Setup project structure
setup_structure() {
    print_info "Verifying project structure..."
    
    # Required directories
    DIRS=(
        "servers"
        "templates" 
        "deployment"
        "examples"
        "docs"
        "tests/integration"
        "tests/performance"
        "tests/security"
        "tests/compliance"
        "scripts"
    )
    
    for dir in "${DIRS[@]}"; do
        if [[ ! -d "$dir" ]]; then
            mkdir -p "$dir"
            print_status "Created $dir/"
        else
            print_status "$dir/ exists"
        fi
    done
}

# Build and test
build_and_test() {
    print_info "Building project with official SDK..."
    
    # Clean previous builds
    cargo clean
    
    # Check project compiles
    if cargo check --workspace --all-targets; then
        print_status "Project compiles successfully"
    else
        print_error "Compilation failed - check dependencies"
        return 1
    fi
    
    # Run basic tests if any exist
    if cargo test --workspace --no-run &> /dev/null; then
        print_info "Running basic tests..."
        if cargo test --workspace; then
            print_status "Tests passed"
        else
            print_warning "Some tests failed"
        fi
    else
        print_info "No tests to run (expected for new project)"
    fi
}

# Create development configuration
create_dev_config() {
    print_info "Creating development configuration..."
    
    # Create .env.example if it doesn't exist
    if [[ ! -f ".env.example" ]]; then
        cat > .env.example << 'EOF'
# MCP Server Development Configuration
# Copy to .env and customize for your environment

# Server Configuration
RUST_LOG=info
RUST_BACKTRACE=1

# Development Mode
DEVELOPMENT_MODE=true

# External API Keys (for production servers)
# NEWS_API_KEY=your_news_api_key_here
# DATABASE_URL=postgresql://user:pass@localhost/dbname

# Security
# JWT_SECRET=your_jwt_secret_here
# RATE_LIMIT_PER_MINUTE=60

# Monitoring
# PROMETHEUS_ENDPOINT=http://localhost:9090
# JAEGER_ENDPOINT=http://localhost:14268/api/traces
EOF
        print_status "Created .env.example"
    fi
    
    # Create justfile for common tasks
    if [[ ! -f "justfile" ]]; then
        cat > justfile << 'EOF'
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
EOF
        print_status "Created justfile"
    fi
}

# Create README for scripts directory
create_scripts_readme() {
    if [[ ! -f "scripts/README.md" ]]; then
        cat > scripts/README.md << 'EOF'
# Development Scripts

This directory contains scripts for development, testing, and deployment.

## Available Scripts

- `setup-development.sh` - Initial development environment setup
- `test-all-servers.sh` - Test all MCP servers
- `benchmark-servers.sh` - Performance benchmarking
- `validate-security.sh` - Security validation
- `deploy-production.sh` - Production deployment

## Usage

Make sure scripts are executable:

```bash
chmod +x scripts/*.sh
```

Run setup:

```bash
./scripts/setup-development.sh
```

## Requirements

- Bash 4.0+
- Rust 1.75+
- Docker (for deployment scripts)
- Git
EOF
        print_status "Created scripts/README.md"
    fi
}

# Main setup function
main() {
    echo -e "${BLUE}Starting development environment setup...${NC}"
    echo ""
    
    check_rust
    check_system_deps
    check_cargo_tools
    validate_sdk
    setup_structure
    create_dev_config
    create_scripts_readme
    
    print_info "Building project..."
    if build_and_test; then
        echo ""
        echo -e "${GREEN}🎉 Development environment setup complete!${NC}"
        echo ""
        echo -e "${BLUE}Next steps:${NC}"
        echo "  1. Copy .env.example to .env and customize"
        echo "  2. Install just: cargo install just"
        echo "  3. Run: just check"
        echo "  4. Start developing: just watch"
        echo ""
        echo -e "${BLUE}Available commands:${NC}"
        echo "  just build          # Build all servers"
        echo "  just test           # Run all tests"  
        echo "  just check          # Format, lint, and test"
        echo "  just run-fs         # Run filesystem server"
        echo "  just docs           # Generate documentation"
        echo ""
        print_status "Ready to build production MCP servers! 🚀"
    else
        echo ""
        print_error "Setup completed with issues. Check the output above."
        exit 1
    fi
}

# Handle command line arguments
case "${1:-setup}" in
    "setup"|"")
        main
        ;;
    "check")
        check_rust
        check_system_deps
        check_cargo_tools
        ;;
    "build")
        build_and_test
        ;;
    "clean")
        print_info "Cleaning project..."
        cargo clean
        rm -rf target/
        print_status "Clean complete"
        ;;
    "help"|"-h"|"--help")
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  setup (default)  - Full development environment setup"
        echo "  check           - Check system requirements only"
        echo "  build           - Build and test project"
        echo "  clean           - Clean build artifacts"
        echo "  help            - Show this help"
        ;;
    *)
        print_error "Unknown command: $1"
        echo "Run '$0 help' for usage information"
        exit 1
        ;;
esac