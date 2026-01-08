#!/bin/bash
set -e

echo "=== MCP Prompts & Resources Test ==="
echo ""

# Build release binary
echo "[1/7] Building release binary..."
cargo build --release --quiet
echo "✓ Build complete"
echo ""

# Test 1: Initialize
echo "[2/7] Testing initialize..."
INIT_RESPONSE=$((echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'; sleep 0.5) | timeout 2 ./target/release/mcp-boilerplate-rust --mode stdio 2>/dev/null | grep -E '^\{' | head -1)

if echo "$INIT_RESPONSE" | grep -q '"prompts"'; then
    echo "✓ Initialize successful - prompts capability enabled"
else
    echo "✗ Initialize failed - prompts capability missing"
    echo "  Response: $INIT_RESPONSE"
    exit 1
fi

if echo "$INIT_RESPONSE" | grep -q '"resources"'; then
    echo "✓ Initialize successful - resources capability enabled"
else
    echo "✗ Initialize failed - resources capability missing"
    exit 1
fi
echo ""

# Test 2: List prompts
echo "[3/7] Testing prompts/list..."
PROMPTS_RESPONSE=$((echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'; sleep 0.5; echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'; sleep 0.5; echo '{"jsonrpc":"2.0","id":2,"method":"prompts/list"}'; sleep 1) | ./target/release/mcp-boilerplate-rust --mode stdio 2>/dev/null | grep -E '^\{' | tail -1)

if echo "$PROMPTS_RESPONSE" | grep -q '"name":"code_review"'; then
    echo "✓ Prompts list successful"
    PROMPT_COUNT=$(echo "$PROMPTS_RESPONSE" | grep -o '"name":"' | wc -l)
    echo "  Found $PROMPT_COUNT prompts"
else
    echo "✗ Prompts list failed"
    echo "  Response: $PROMPTS_RESPONSE"
    exit 1
fi
echo ""

# Test 3: Get prompt with arguments
echo "[4/7] Testing prompts/get (code_review)..."
GET_PROMPT_RESPONSE=$((echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'; sleep 0.5; echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'; sleep 0.5; echo '{"jsonrpc":"2.0","id":3,"method":"prompts/get","params":{"name":"code_review","arguments":{"language":"rust","focus":"security"}}}'; sleep 1) | ./target/release/mcp-boilerplate-rust --mode stdio 2>/dev/null | grep -E '^\{' | tail -1)

if echo "$GET_PROMPT_RESPONSE" | grep -q 'rust'; then
    echo "✓ Prompt get successful"
    echo "  Response contains language parameter"
else
    echo "✗ Prompt get failed"
    echo "  Response: $GET_PROMPT_RESPONSE"
    exit 1
fi
echo ""

# Test 4: List resources
echo "[5/7] Testing resources/list..."
RESOURCES_RESPONSE=$((echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'; sleep 0.5; echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'; sleep 0.5; echo '{"jsonrpc":"2.0","id":4,"method":"resources/list"}'; sleep 1) | ./target/release/mcp-boilerplate-rust --mode stdio 2>/dev/null | grep -E '^\{' | tail -1)

if echo "$RESOURCES_RESPONSE" | grep -q '"uri":"config://server"'; then
    echo "✓ Resources list successful"
    RESOURCE_COUNT=$(echo "$RESOURCES_RESPONSE" | grep -o '"uri":"' | wc -l)
    echo "  Found $RESOURCE_COUNT resources"
else
    echo "✗ Resources list failed"
    echo "  Response: $RESOURCES_RESPONSE"
    exit 1
fi
echo ""

# Test 5: Read resource (config://server)
echo "[6/7] Testing resources/read (config://server)..."
READ_RESOURCE_RESPONSE=$((echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'; sleep 0.5; echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'; sleep 0.5; echo '{"jsonrpc":"2.0","id":5,"method":"resources/read","params":{"uri":"config://server"}}'; sleep 1) | ./target/release/mcp-boilerplate-rust --mode stdio 2>/dev/null | grep -E '^\{' | tail -1)

if echo "$READ_RESOURCE_RESPONSE" | grep -q 'mcp-boilerplate-rust'; then
    echo "✓ Resource read successful"
    echo "  Response contains server config"
else
    echo "✗ Resource read failed"
    echo "  Response: $READ_RESOURCE_RESPONSE"
    exit 1
fi
echo ""

# Test 6: Read resource (info://capabilities)
echo "[7/7] Testing resources/read (info://capabilities)..."
READ_CAPABILITIES_RESPONSE=$((echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'; sleep 0.5; echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'; sleep 0.5; echo '{"jsonrpc":"2.0","id":6,"method":"resources/read","params":{"uri":"info://capabilities"}}'; sleep 1) | ./target/release/mcp-boilerplate-rust --mode stdio 2>/dev/null | grep -E '^\{' | tail -1)

if echo "$READ_CAPABILITIES_RESPONSE" | grep -q 'capabilities'; then
    echo "✓ Capabilities resource read successful"
    echo "  Response contains capabilities info"
else
    echo "✗ Capabilities resource read failed"
    echo "  Response: $READ_CAPABILITIES_RESPONSE"
    exit 1
fi
echo ""

echo "=== All Prompts & Resources Tests Passed ==="
echo ""
echo "Available prompts:"
echo "$PROMPTS_RESPONSE" | grep -o '"name":"[^"]*"' | sed 's/"name":"//g' | sed 's/"//g' | sed 's/^/  - /'
echo ""
echo "Available resources:"
echo "$RESOURCES_RESPONSE" | grep -o '"uri":"[^"]*"' | sed 's/"uri":"//g' | sed 's/"//g' | sed 's/^/  - /'
echo ""
echo "Server is fully functional with all MCP capabilities!"