#!/bin/bash

set -e

echo "=== MCP HTTP Server Test ==="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

# Build with HTTP feature
echo "[1/5] Building with HTTP feature..."
cargo build --release --features http --quiet
echo -e "${GREEN}✓${NC} Build complete"
echo ""

# Start server in background
echo "[2/5] Starting HTTP server..."
./target/release/mcp-boilerplate-rust --mode http &
SERVER_PID=$!
sleep 2

# Function to cleanup
cleanup() {
    echo ""
    echo "Stopping server..."
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
}
trap cleanup EXIT

# Test health endpoint
echo "[3/5] Testing /health endpoint..."
HEALTH_RESPONSE=$(curl -s http://localhost:8025/health)
if echo "$HEALTH_RESPONSE" | grep -q "healthy"; then
    echo -e "${GREEN}✓${NC} Health check passed"
    echo "  Response: $(echo $HEALTH_RESPONSE | jq -c '.')"
else
    echo -e "${RED}✗${NC} Health check failed"
    exit 1
fi
echo ""

# Test tools list
echo "[4/5] Testing /tools endpoint..."
TOOLS_RESPONSE=$(curl -s http://localhost:8025/tools)
TOOL_COUNT=$(echo "$TOOLS_RESPONSE" | jq '.tools | length')
if [ "$TOOL_COUNT" -eq 3 ]; then
    echo -e "${GREEN}✓${NC} Tools list passed (found $TOOL_COUNT tools)"
    echo "  Tools: $(echo $TOOLS_RESPONSE | jq -r '.tools[].name' | tr '\n' ', ' | sed 's/,$//')"
else
    echo -e "${RED}✗${NC} Expected 3 tools, found $TOOL_COUNT"
    exit 1
fi
echo ""

# Test echo tool
echo "[5/5] Testing /tools/echo endpoint..."
ECHO_RESPONSE=$(curl -s -X POST http://localhost:8025/tools/echo \
    -H "Content-Type: application/json" \
    -d '{"message":"Hello HTTP MCP"}')

if echo "$ECHO_RESPONSE" | grep -q "Hello HTTP MCP"; then
    echo -e "${GREEN}✓${NC} Echo tool passed"
    echo "  Response: $(echo $ECHO_RESPONSE | jq -c '.content[0].text')"
else
    echo -e "${RED}✗${NC} Echo tool failed"
    echo "  Response: $ECHO_RESPONSE"
    exit 1
fi
echo ""

echo "=== All HTTP Tests Passed ==="
echo ""
echo "HTTP Server is ready!"
echo "  Health:    http://localhost:8025/health"
echo "  Tools:     http://localhost:8025/tools"
echo "  Echo:      curl -X POST http://localhost:8025/tools/echo -H 'Content-Type: application/json' -d '{\"message\":\"test\"}'"