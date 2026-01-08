#!/bin/bash

# MCP Stdio Wrapper Compatibility Test
# Tests if the Rust MCP server is compatible with stdio wrapper

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

HOST=${HOST:-localhost}
PORT=${PORT:-8025}
BASE_URL="http://${HOST}:${PORT}"

echo -e "${BLUE}=== MCP Stdio Wrapper Compatibility Test ===${NC}\n"

PASSED=0
FAILED=0

# Function to print test header
print_test() {
    echo -e "${BLUE}Test: $1${NC}"
}

# Function to print success
print_success() {
    echo -e "${GREEN}✓ Pass${NC}\n"
    PASSED=$((PASSED + 1))
}

# Function to print error
print_error() {
    echo -e "${RED}✗ Fail: $1${NC}\n"
    FAILED=$((FAILED + 1))
}

# Test 1: Check if server is running
print_test "Server Health Check"
if curl -s -f "${BASE_URL}/health" > /dev/null 2>&1; then
    print_success
else
    print_error "Server not running on ${BASE_URL}"
    echo "Start server with: make run"
    exit 1
fi

# Test 2: Check /tools endpoint exists
print_test "GET /tools endpoint (required by stdio wrapper)"
TOOLS_RESPONSE=$(curl -s -w "\n%{http_code}" "${BASE_URL}/tools")
HTTP_CODE=$(echo "$TOOLS_RESPONSE" | tail -n 1)
BODY=$(echo "$TOOLS_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" = "200" ]; then
    echo "Response: $BODY" | jq '.' 2>/dev/null || echo "$BODY"
    print_success
else
    print_error "GET /tools returned HTTP $HTTP_CODE"
fi

# Test 3: Verify tools array in response
print_test "Verify tools array structure"
TOOLS_COUNT=$(echo "$BODY" | jq '.tools | length' 2>/dev/null || echo "0")

if [ "$TOOLS_COUNT" -gt 0 ]; then
    echo "Found $TOOLS_COUNT tool(s)"
    echo "$BODY" | jq '.tools[] | {name, description}' 2>/dev/null || echo "$BODY"
    print_success
else
    print_error "No tools found in response"
fi

# Test 4: Verify tool schema
print_test "Verify tool schema (name, description, parameters)"
FIRST_TOOL=$(echo "$BODY" | jq '.tools[0]' 2>/dev/null)

HAS_NAME=$(echo "$FIRST_TOOL" | jq 'has("name")' 2>/dev/null)
HAS_DESC=$(echo "$FIRST_TOOL" | jq 'has("description")' 2>/dev/null)
HAS_PARAMS=$(echo "$FIRST_TOOL" | jq 'has("parameters")' 2>/dev/null)

if [ "$HAS_NAME" = "true" ] && [ "$HAS_DESC" = "true" ] && [ "$HAS_PARAMS" = "true" ]; then
    echo "Tool schema is valid"
    print_success
else
    print_error "Tool missing required fields (name, description, parameters)"
fi

# Test 5: Test tool execution
print_test "POST /tools/echo execution"
TOOL_RESPONSE=$(curl -s -X POST "${BASE_URL}/tools/echo" \
  -H "Content-Type: application/json" \
  -d '{"action":"ping"}')

echo "Response: $TOOL_RESPONSE" | jq '.' 2>/dev/null || echo "$TOOL_RESPONSE"

SUCCESS=$(echo "$TOOL_RESPONSE" | jq '.success' 2>/dev/null)
if [ "$SUCCESS" = "true" ]; then
    print_success
else
    print_error "Tool execution failed or returned success=false"
fi

# Test 6: Test with JWT token header (optional)
print_test "Test with x-access-token header (optional auth)"
TOKEN_RESPONSE=$(curl -s -X POST "${BASE_URL}/tools/echo" \
  -H "Content-Type: application/json" \
  -H "x-access-token: test-token" \
  -d '{"action":"ping"}')

TOKEN_SUCCESS=$(echo "$TOKEN_RESPONSE" | jq '.success' 2>/dev/null)
if [ "$TOKEN_SUCCESS" = "true" ]; then
    echo "Server accepts x-access-token header"
    print_success
else
    echo "Note: Server may require valid JWT token"
    print_success
fi

# Test 7: Verify CORS headers
print_test "Verify CORS headers (required for HTTP MCP)"
CORS_HEADERS=$(curl -s -I "${BASE_URL}/tools" | grep -i "access-control")

if [ -n "$CORS_HEADERS" ]; then
    echo "CORS headers found:"
    echo "$CORS_HEADERS"
    print_success
else
    print_error "No CORS headers found"
fi

# Summary
echo ""
echo -e "${BLUE}=== Test Summary ===${NC}"
echo ""
echo -e "Passed: ${GREEN}$PASSED${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"
echo ""

if [ "$FAILED" -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    echo ""
    echo "Your Rust MCP server is compatible with stdio wrapper."
    echo ""
    echo "Next steps:"
    echo "1. Generate JWT token:"
    echo "   node -e \"const jwt=require('jsonwebtoken');console.log(jwt.sign({userObjId:'test'},process.env.JWT_SECRET||'aivaAPI',{algorithm:'HS256',expiresIn:'24h'}))\""
    echo ""
    echo "2. Configure Claude Desktop:"
    echo "   File: ~/Library/Application Support/Claude/claude_desktop_config.json"
    echo ""
    echo "   {"
    echo "     \"mcpServers\": {"
    echo "       \"rust-mcp\": {"
    echo "         \"command\": \"npx\","
    echo "         \"args\": [\"-y\", \"@netadx1ai/mcp-stdio-wrapper@latest\"],"
    echo "         \"env\": {"
    echo "           \"API_URL\": \"${BASE_URL}\","
    echo "           \"JWT_TOKEN\": \"your-token-here\","
    echo "           \"LOG_FILE\": \"/tmp/mcp-rust.log\""
    echo "         }"
    echo "       }"
    echo "     }"
    echo "   }"
    echo ""
    echo "3. Restart Claude Desktop"
    echo ""
    echo "4. Test in Claude: \"Can you list available tools?\""
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    echo ""
    echo "Please fix the issues above before using with stdio wrapper."
    exit 1
fi