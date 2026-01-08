#!/bin/bash
set -e

echo "=== MCP Boilerplate Rust - Protocol Test ==="
echo ""

# Build release binary
echo "[1/4] Building release binary..."
cargo build --release --quiet
echo "✓ Build complete"
echo ""

# Test 1: Initialize
echo "[2/4] Testing initialize..."
INIT_RESPONSE=$((echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'; sleep 0.5) | timeout 2 ./target/release/mcp-boilerplate-rust --mode stdio 2>/dev/null | grep -E '^\{' | head -1)

if echo "$INIT_RESPONSE" | grep -q '"protocolVersion":"2024-11-05"'; then
    echo "✓ Initialize successful"
    echo "  Response: $INIT_RESPONSE" | head -c 100
    echo "..."
else
    echo "✗ Initialize failed"
    echo "  Response: $INIT_RESPONSE"
    exit 1
fi
echo ""

# Test 2: List tools
echo "[3/4] Testing tools/list..."
LIST_RESPONSE=$((echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'; sleep 0.5; echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'; sleep 0.5; echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}'; sleep 1) | ./target/release/mcp-boilerplate-rust --mode stdio 2>/dev/null | grep -E '^\{' | tail -1)

if echo "$LIST_RESPONSE" | grep -q '"name":"echo"'; then
    echo "✓ Tools list successful"
    TOOL_COUNT=$(echo "$LIST_RESPONSE" | grep -o '"name":"' | wc -l)
    echo "  Found $TOOL_COUNT tools"
else
    echo "✗ Tools list failed"
    echo "  Response: $LIST_RESPONSE"
    exit 1
fi
echo ""

# Test 3: Call echo tool
echo "[4/4] Testing tools/call (echo)..."
CALL_RESPONSE=$((echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'; sleep 0.5; echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'; sleep 0.5; echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"echo","arguments":{"message":"Hello MCP"}}}'; sleep 1) | ./target/release/mcp-boilerplate-rust --mode stdio 2>/dev/null | grep -E '^\{' | tail -1)

if echo "$CALL_RESPONSE" | grep -q 'Hello MCP'; then
    echo "✓ Echo tool call successful"
    echo "  Response contains message"
else
    echo "✗ Echo tool call failed"
    echo "  Response: $CALL_RESPONSE"
    exit 1
fi
echo ""

echo "=== All Tests Passed ==="
echo ""
echo "Available tools:"
echo "$LIST_RESPONSE" | grep -o '"name":"[^"]*"' | sed 's/"name":"//g' | sed 's/"//g' | sed 's/^/  - /'
echo ""
echo "Server is ready for Claude Desktop integration!"