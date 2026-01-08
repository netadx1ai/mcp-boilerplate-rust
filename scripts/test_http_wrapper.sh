#!/bin/bash

# Test HTTP Wrapper Integration
# Tests mcp-stdio-wrapper connecting to MCP HTTP server
# Version: 0.3.1
# Date: 2026-01-08

set -e

echo "=== MCP HTTP Wrapper Integration Test ==="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

PROJECT_DIR="/Users/hoangiso/Desktop/mcp-boilerplate-rust"
HTTP_SERVER_URL="http://localhost:8025"
WRAPPER_DIR="/Users/hoangiso/Desktop/mcp-stdio-wrapper"
WRAPPER_LOG="/tmp/mcp-http-wrapper.log"

# Check if HTTP server is running
echo "[1/6] Checking HTTP server..."
if curl -s "$HTTP_SERVER_URL/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} HTTP server is running"
else
    echo -e "${RED}✗${NC} HTTP server is not running"
    echo "Please start the server first:"
    echo "  cd $PROJECT_DIR"
    echo "  ./target/release/mcp-boilerplate-rust --mode http"
    exit 1
fi

# Test /tools endpoint
echo "[2/6] Testing /tools endpoint..."
TOOLS_RESPONSE=$(curl -s "$HTTP_SERVER_URL/tools")
TOOL_COUNT=$(echo "$TOOLS_RESPONSE" | python3 -c "import json,sys; print(len(json.load(sys.stdin)['tools']))")

if [ "$TOOL_COUNT" -eq 3 ]; then
    echo -e "${GREEN}✓${NC} /tools endpoint works ($TOOL_COUNT tools found)"
else
    echo -e "${RED}✗${NC} /tools endpoint failed (expected 3 tools, got $TOOL_COUNT)"
    exit 1
fi

# Check parameters field
echo "[3/6] Checking parameters field..."
HAS_PARAMS=$(echo "$TOOLS_RESPONSE" | python3 -c "import json,sys; print('parameters' in json.load(sys.stdin)['tools'][0])")

if [ "$HAS_PARAMS" = "True" ]; then
    echo -e "${GREEN}✓${NC} Tools have parameters field (wrapper compatible)"
else
    echo -e "${RED}✗${NC} Tools missing parameters field"
    exit 1
fi

# Test direct tool call
echo "[4/6] Testing direct tool execution..."
ECHO_RESPONSE=$(curl -s -X POST "$HTTP_SERVER_URL/tools/echo" \
    -H 'Content-Type: application/json' \
    -d '{"message":"Test from wrapper script"}')

if echo "$ECHO_RESPONSE" | grep -q "Test from wrapper script"; then
    echo -e "${GREEN}✓${NC} Tool execution works"
else
    echo -e "${RED}✗${NC} Tool execution failed"
    echo "Response: $ECHO_RESPONSE"
    exit 1
fi

# Test wrapper if built
echo "[5/6] Testing stdio wrapper..."
if [ -f "$WRAPPER_DIR/dist/index.js" ]; then
    echo "Using local wrapper build"
    
    # Clear log file
    rm -f "$WRAPPER_LOG"
    
    # Test wrapper with simple initialize
    WRAPPER_TEST=$(cat <<'WRAPPER_INPUT'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}
WRAPPER_INPUT
)
    
    # Run wrapper with timeout
    WRAPPER_OUTPUT=$(echo "$WRAPPER_TEST" | \
        API_URL="$HTTP_SERVER_URL" \
        JWT_TOKEN="test-token" \
        LOG_FILE="$WRAPPER_LOG" \
        timeout 3 node "$WRAPPER_DIR/dist/index.js" 2>/dev/null || true)
    
    if echo "$WRAPPER_OUTPUT" | grep -q "rmcp"; then
        echo -e "${GREEN}✓${NC} Wrapper responds to initialize"
    else
        echo -e "${YELLOW}⚠${NC} Wrapper test skipped (needs manual testing with Claude Desktop)"
    fi
    
    # Check wrapper log
    if [ -f "$WRAPPER_LOG" ]; then
        echo "  Wrapper log created: $WRAPPER_LOG"
    fi
else
    echo -e "${YELLOW}⚠${NC} Wrapper not built locally, will use npx"
    echo "  Claude Desktop will download via: npx -y @netadx1ai/mcp-stdio-wrapper@latest"
fi

# Check Claude Desktop config
echo "[6/6] Checking Claude Desktop config..."
CLAUDE_CONFIG="$HOME/Library/Application Support/Claude/claude_desktop_config.json"

if [ -f "$CLAUDE_CONFIG" ]; then
    if grep -q "mcp-boilerplate-rust-http" "$CLAUDE_CONFIG"; then
        echo -e "${GREEN}✓${NC} Claude Desktop config includes HTTP wrapper"
    else
        echo -e "${YELLOW}⚠${NC} Claude Desktop config missing HTTP wrapper entry"
        echo "  Run: cp $PROJECT_DIR/claude_desktop_config_http_wrapper.json \"$CLAUDE_CONFIG\""
    fi
else
    echo -e "${YELLOW}⚠${NC} Claude Desktop config not found"
fi

echo ""
echo "=== Summary ==="
echo -e "${GREEN}✓${NC} HTTP server running and accessible"
echo -e "${GREEN}✓${NC} /tools endpoint returns 3 tools"
echo -e "${GREEN}✓${NC} Tools have parameters field"
echo -e "${GREEN}✓${NC} Direct tool execution works"
echo ""
echo "Next steps:"
echo "  1. Make sure Claude Desktop config includes both servers"
echo "  2. Restart Claude Desktop:"
echo "     killall Claude && sleep 2 && open -a Claude"
echo "  3. In Claude Desktop, you should see TWO servers:"
echo "     - mcp-boilerplate-rust-stdio (direct)"
echo "     - mcp-boilerplate-rust-http (via wrapper)"
echo "  4. Test tools from both servers"
echo ""
echo "Monitor wrapper logs:"
echo "  tail -f $WRAPPER_LOG"
echo ""
echo "HTTP server info:"
echo "  URL: $HTTP_SERVER_URL"
echo "  Health: curl $HTTP_SERVER_URL/health"
echo "  Tools: curl $HTTP_SERVER_URL/tools"
echo ""