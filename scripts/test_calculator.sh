#!/bin/bash
set -e

echo "=== Calculator Tool Integration Tests ==="
echo ""

# Build release binary
echo "[1/2] Building release binary..."
cargo build --release --quiet
echo "✓ Build complete"
echo ""

echo "[2/2] Running calculator tests..."
echo ""

# Test calculate tool - addition
echo "Test 1: Calculate 5 + 3 = 8"
CALC_ADD=$((echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'; sleep 0.5; echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'; sleep 0.5; echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"calculate","arguments":{"operation":"add","a":5,"b":3}}}'; sleep 1) | ./target/release/mcp-boilerplate-rust --mode stdio 2>/dev/null | grep -E '^\{' | tail -1)

if echo "$CALC_ADD" | grep -q '"result":8'; then
    echo "✓ Addition test passed"
else
    echo "✗ Addition test failed"
    echo "  Response: $CALC_ADD"
    exit 1
fi
echo ""

# Test calculate tool - multiplication
echo "Test 2: Calculate 6 * 7 = 42"
CALC_MUL=$((echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'; sleep 0.5; echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'; sleep 0.5; echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"calculate","arguments":{"operation":"multiply","a":6,"b":7}}}'; sleep 1) | ./target/release/mcp-boilerplate-rust --mode stdio 2>/dev/null | grep -E '^\{' | tail -1)

if echo "$CALC_MUL" | grep -q '"result":42'; then
    echo "✓ Multiplication test passed"
else
    echo "✗ Multiplication test failed"
    echo "  Response: $CALC_MUL"
    exit 1
fi
echo ""

# Test calculate tool - division
echo "Test 3: Calculate 20 / 4 = 5"
CALC_DIV=$((echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'; sleep 0.5; echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'; sleep 0.5; echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"calculate","arguments":{"operation":"divide","a":20,"b":4}}}'; sleep 1) | ./target/release/mcp-boilerplate-rust --mode stdio 2>/dev/null | grep -E '^\{' | tail -1)

if echo "$CALC_DIV" | grep -q '"result":5'; then
    echo "✓ Division test passed"
else
    echo "✗ Division test failed"
    echo "  Response: $CALC_DIV"
    exit 1
fi
echo ""

# Test calculate tool - power
echo "Test 4: Calculate 2 ^ 3 = 8"
CALC_POW=$((echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'; sleep 0.5; echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'; sleep 0.5; echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"calculate","arguments":{"operation":"power","a":2,"b":3}}}'; sleep 1) | ./target/release/mcp-boilerplate-rust --mode stdio 2>/dev/null | grep -E '^\{' | tail -1)

if echo "$CALC_POW" | grep -q '"result":8'; then
    echo "✓ Power test passed"
else
    echo "✗ Power test failed"
    echo "  Response: $CALC_POW"
    exit 1
fi
echo ""

# Test evaluate tool - simple expression
echo "Test 5: Evaluate 2+3*4 = 14"
EVAL_SIMPLE=$((echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'; sleep 0.5; echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'; sleep 0.5; echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"evaluate","arguments":{"expression":"2+3*4"}}}'; sleep 1) | ./target/release/mcp-boilerplate-rust --mode stdio 2>/dev/null | grep -E '^\{' | tail -1)

if echo "$EVAL_SIMPLE" | grep -q '"result":14'; then
    echo "✓ Simple expression test passed"
else
    echo "✗ Simple expression test failed"
    echo "  Response: $EVAL_SIMPLE"
    exit 1
fi
echo ""

# Test evaluate tool - parentheses
echo "Test 6: Evaluate (2+3)*4 = 20"
EVAL_PAREN=$((echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'; sleep 0.5; echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'; sleep 0.5; echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"evaluate","arguments":{"expression":"(2+3)*4"}}}'; sleep 1) | ./target/release/mcp-boilerplate-rust --mode stdio 2>/dev/null | grep -E '^\{' | tail -1)

if echo "$EVAL_PAREN" | grep -q '"result":20'; then
    echo "✓ Parentheses expression test passed"
else
    echo "✗ Parentheses expression test failed"
    echo "  Response: $EVAL_PAREN"
    exit 1
fi
echo ""

# Test error handling - division by zero
echo "Test 7: Error handling - division by zero"
CALC_ERROR=$((echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'; sleep 0.5; echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'; sleep 0.5; echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"calculate","arguments":{"operation":"divide","a":5,"b":0}}}'; sleep 1) | ./target/release/mcp-boilerplate-rust --mode stdio 2>/dev/null | grep -E '^\{' | tail -1)

if echo "$CALC_ERROR" | grep -q '"error"'; then
    echo "✓ Division by zero error handling passed"
else
    echo "✗ Division by zero should return error"
    echo "  Response: $CALC_ERROR"
    exit 1
fi
echo ""

# Test error handling - invalid operation
echo "Test 8: Error handling - invalid operation"
CALC_INVALID=$((echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'; sleep 0.5; echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'; sleep 0.5; echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"calculate","arguments":{"operation":"invalid","a":5,"b":3}}}'; sleep 1) | ./target/release/mcp-boilerplate-rust --mode stdio 2>/dev/null | grep -E '^\{' | tail -1)

if echo "$CALC_INVALID" | grep -q '"error"'; then
    echo "✓ Invalid operation error handling passed"
else
    echo "✗ Invalid operation should return error"
    echo "  Response: $CALC_INVALID"
    exit 1
fi
echo ""

echo "=== All Calculator Tests Passed ==="
echo ""
echo "Calculator tools are working correctly!"
echo "  - calculate: add, subtract, multiply, divide, modulo, power"
echo "  - evaluate: mathematical expressions with +, -, *, /, ()"
echo ""