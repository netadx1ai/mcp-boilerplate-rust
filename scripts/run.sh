#!/bin/bash

# MCP Boilerplate Rust - Run Script
# MCP v5 using official rust-sdk (stdio primary, HTTP optional)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}MCP Boilerplate Rust v0.3.0${NC}"

# Check if .env exists, if not copy from example
if [ ! -f .env ]; then
    echo -e "${YELLOW}No .env file found. Creating from .env.example...${NC}"
    if [ -f .env.example ]; then
        cp .env.example .env
        echo -e "${GREEN}.env file created. Please update it with your configuration.${NC}"
    else
        echo -e "${RED}Error: .env.example not found${NC}"
        exit 1
    fi
fi

# Load environment variables
export $(cat .env | grep -v '^#' | xargs)

# Set default values if not set
export HOST=${HOST:-0.0.0.0}
export PORT=${PORT:-8025}
export RUST_LOG=${RUST_LOG:-info,mcp_boilerplate_rust=debug}

echo -e "${GREEN}Configuration:${NC}"
echo "  Host: $HOST"
echo "  Port: $PORT"
echo "  Log Level: $RUST_LOG"
echo ""

# Check if running in development or production mode
MODE=${1:-dev}
SERVER_MODE=${2:-stdio}

# Validate server mode
if [ "$SERVER_MODE" != "http" ] && [ "$SERVER_MODE" != "stdio" ]; then
    echo -e "${RED}Invalid server mode: $SERVER_MODE${NC}"
    echo "Valid modes: stdio (default), http (requires http feature)"
    exit 1
fi

echo -e "${GREEN}Protocol: MCP v5 native ${SERVER_MODE}${NC}"
echo ""

# Build flags based on server mode
if [ "$SERVER_MODE" = "http" ]; then
    FEATURES="--features http"
    echo -e "${YELLOW}Note: HTTP mode requires 'http' feature${NC}"
else
    FEATURES=""
fi

if [ "$MODE" = "dev" ]; then
    echo -e "${GREEN}Running in DEVELOPMENT mode...${NC}"
    cargo run $FEATURES -- --mode $SERVER_MODE
elif [ "$MODE" = "prod" ]; then
    echo -e "${GREEN}Building and running in PRODUCTION mode...${NC}"
    cargo build --release $FEATURES
    ./target/release/mcp-boilerplate-rust --mode $SERVER_MODE
elif [ "$MODE" = "watch" ]; then
    echo -e "${GREEN}Running in WATCH mode (requires cargo-watch)...${NC}"
    if ! command -v cargo-watch &> /dev/null; then
        echo -e "${YELLOW}cargo-watch not found. Installing...${NC}"
        cargo install cargo-watch
    fi
    cargo watch -x "run $FEATURES -- --mode $SERVER_MODE"
else
    echo -e "${RED}Unknown mode: $MODE${NC}"
    echo "Usage: ./run.sh [dev|prod|watch] [stdio|http]"
    echo ""
    echo "Examples:"
    echo "  ./run.sh dev              # Dev mode with stdio (default)"
    echo "  ./run.sh dev stdio        # Dev mode with stdio protocol"
    echo "  ./run.sh dev http         # Dev mode with HTTP server"
    echo "  ./run.sh prod stdio       # Production with stdio (for Claude Desktop)"
    echo "  ./run.sh watch stdio      # Watch mode with stdio"
    echo ""
    echo "Default: stdio mode (native MCP protocol)"
    exit 1
fi