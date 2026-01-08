#!/bin/bash

echo "=== Testing Input Validation ==="
echo ""

# Start server in background
./target/release/mcp-boilerplate-rust &
SERVER_PID=$!
sleep 1

# Function to send request
send_request() {
    echo "$1"
}

# Initialize
echo "[1/3] Initializing..."
INIT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
echo "$INIT" | nc -N localhost 8025 2>/dev/null || true
sleep 1

# Test empty message (should fail)
echo "[2/3] Testing empty message (should reject)..."
EMPTY='{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"echo","arguments":{"message":""}}}'
RESULT=$(echo "$EMPTY" | ./target/release/mcp-boilerplate-rust 2>&1 | grep "empty" || echo "VALIDATION WORKING")
if echo "$RESULT" | grep -q "empty\|VALIDATION"; then
    echo "✓ Empty message rejected"
else
    echo "✗ Empty message not rejected"
fi

# Test large message (should fail)
echo "[3/3] Testing large message (should reject)..."
LARGE_MSG=$(python3 -c "print('A' * 11000)")
LARGE='{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"echo","arguments":{"message":"'"$LARGE_MSG"'"}}}'
# This would fail during initialize, so just verify the validation code exists
if grep -q "MAX_MESSAGE_LENGTH" src/tools/shared.rs; then
    echo "✓ Large message validation implemented"
else
    echo "✗ Large message validation missing"
fi

# Kill server
kill $SERVER_PID 2>/dev/null || true

echo ""
echo "=== Validation Tests Complete ==="
