#!/bin/bash

# SSE Integration Test Script
# Tests the SSE transport implementation

set -e

echo "========================================="
echo "MCP SSE Transport Integration Tests"
echo "========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test configuration
SERVER_HOST="127.0.0.1"
SERVER_PORT="8025"
BASE_URL="http://${SERVER_HOST}:${SERVER_PORT}"
SSE_URL="${BASE_URL}/sse"
HEALTH_URL="${BASE_URL}/health"
TOOLS_URL="${BASE_URL}/tools"
CALL_URL="${BASE_URL}/tools/call"

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Helper functions
print_test() {
    echo -e "${YELLOW}TEST ${TESTS_RUN}:${NC} $1"
}

print_pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    ((TESTS_PASSED++))
}

print_fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    ((TESTS_FAILED++))
}

# Check if server is running
check_server() {
    print_test "Checking if SSE server is running"
    ((TESTS_RUN++))
    
    if curl -s -f "${HEALTH_URL}" > /dev/null 2>&1; then
        print_pass "Server is running on ${BASE_URL}"
        return 0
    else
        print_fail "Server is not running. Start with: cargo run --features sse -- --mode sse"
        echo ""
        echo "To start the server, run:"
        echo "  cargo run --features sse -- --mode sse --bind ${SERVER_HOST}:${SERVER_PORT}"
        echo ""
        exit 1
    fi
}

# Test 1: Health check
test_health_check() {
    print_test "Health check endpoint"
    ((TESTS_RUN++))
    
    response=$(curl -s "${HEALTH_URL}")
    
    if echo "$response" | jq -e '.status == "healthy"' > /dev/null 2>&1; then
        print_pass "Health check returned healthy status"
        echo "  Response: $(echo $response | jq -c .)"
    else
        print_fail "Health check failed or returned invalid response"
        echo "  Response: $response"
    fi
}

# Test 2: List tools
test_list_tools() {
    print_test "List available tools"
    ((TESTS_RUN++))
    
    response=$(curl -s "${TOOLS_URL}")
    
    if echo "$response" | jq -e '.tools | length > 0' > /dev/null 2>&1; then
        tool_count=$(echo "$response" | jq '.tools | length')
        print_pass "Listed ${tool_count} tools"
        echo "$response" | jq -r '.tools[].name' | while read tool; do
            echo "    - $tool"
        done
    else
        print_fail "Failed to list tools"
        echo "  Response: $response"
    fi
}

# Test 3: Root endpoint
test_root_endpoint() {
    print_test "Root endpoint info"
    ((TESTS_RUN++))
    
    response=$(curl -s "${BASE_URL}/")
    
    if echo "$response" | jq -e '.service' > /dev/null 2>&1; then
        print_pass "Root endpoint returned server info"
        echo "  Service: $(echo $response | jq -r .service)"
        echo "  Version: $(echo $response | jq -r .version)"
        echo "  Transport: $(echo $response | jq -r .transport)"
    else
        print_fail "Root endpoint failed"
        echo "  Response: $response"
    fi
}

# Test 4: Call echo tool
test_call_echo() {
    print_test "Call echo tool"
    ((TESTS_RUN++))
    
    payload='{"name":"echo","arguments":{"message":"Test from SSE integration"}}'
    response=$(curl -s -X POST "${CALL_URL}" \
        -H "Content-Type: application/json" \
        -d "$payload")
    
    if echo "$response" | jq -e '.status == "accepted"' > /dev/null 2>&1; then
        request_id=$(echo "$response" | jq -r '.request_id')
        print_pass "Echo tool call accepted (Request ID: ${request_id})"
        echo "  Message: $(echo $response | jq -r .message)"
    else
        print_fail "Echo tool call failed"
        echo "  Response: $response"
    fi
}

# Test 5: Call ping tool
test_call_ping() {
    print_test "Call ping tool"
    ((TESTS_RUN++))
    
    payload='{"name":"ping","arguments":{}}'
    response=$(curl -s -X POST "${CALL_URL}" \
        -H "Content-Type: application/json" \
        -d "$payload")
    
    if echo "$response" | jq -e '.status == "accepted"' > /dev/null 2>&1; then
        request_id=$(echo "$response" | jq -r '.request_id')
        print_pass "Ping tool call accepted (Request ID: ${request_id})"
    else
        print_fail "Ping tool call failed"
        echo "  Response: $response"
    fi
}

# Test 6: Call info tool
test_call_info() {
    print_test "Call info tool"
    ((TESTS_RUN++))
    
    payload='{"name":"info","arguments":{}}'
    response=$(curl -s -X POST "${CALL_URL}" \
        -H "Content-Type: application/json" \
        -d "$payload")
    
    if echo "$response" | jq -e '.status == "accepted"' > /dev/null 2>&1; then
        request_id=$(echo "$response" | jq -r '.request_id')
        print_pass "Info tool call accepted (Request ID: ${request_id})"
    else
        print_fail "Info tool call failed"
        echo "  Response: $response"
    fi
}

# Test 7: SSE stream connectivity
test_sse_stream() {
    print_test "SSE stream connectivity"
    ((TESTS_RUN++))
    
    # Try to connect to SSE endpoint and receive at least one event
    timeout 5s curl -s -N "${SSE_URL}" | head -n 5 > /tmp/sse_test.log 2>&1 &
    sleep 2
    
    if [ -s /tmp/sse_test.log ]; then
        print_pass "SSE stream is accessible"
        echo "  First few lines:"
        head -n 3 /tmp/sse_test.log | while read line; do
            echo "    $line"
        done
    else
        print_fail "SSE stream not accessible or no data received"
    fi
    
    rm -f /tmp/sse_test.log
}

# Test 8: Client statistics
test_client_stats() {
    print_test "Client statistics tracking"
    ((TESTS_RUN++))
    
    response=$(curl -s "${HEALTH_URL}")
    
    if echo "$response" | jq -e '.clients' > /dev/null 2>&1; then
        connected=$(echo "$response" | jq -r '.clients.connected')
        print_pass "Client statistics available (Connected: ${connected})"
        
        if [ "$connected" -gt 0 ]; then
            echo "  Connected client IDs:"
            echo "$response" | jq -r '.clients.ids[]' | while read id; do
                echo "    - $id"
            done
        fi
    else
        print_fail "Client statistics not available"
        echo "  Response: $response"
    fi
}

# Test 9: Invalid tool call
test_invalid_tool() {
    print_test "Invalid tool call handling"
    ((TESTS_RUN++))
    
    payload='{"name":"nonexistent_tool","arguments":{}}'
    response=$(curl -s -X POST "${CALL_URL}" \
        -H "Content-Type: application/json" \
        -d "$payload")
    
    # Should still accept the call (error will be sent via SSE)
    if echo "$response" | jq -e '.status == "accepted"' > /dev/null 2>&1; then
        print_pass "Invalid tool call handled correctly"
        echo "  Response: $(echo $response | jq -c .)"
    else
        print_fail "Invalid tool call not handled properly"
        echo "  Response: $response"
    fi
}

# Test 10: CORS headers
test_cors() {
    print_test "CORS headers"
    ((TESTS_RUN++))
    
    headers=$(curl -s -I "${HEALTH_URL}")
    
    if echo "$headers" | grep -i "access-control-allow-origin" > /dev/null; then
        print_pass "CORS headers are present"
    else
        print_fail "CORS headers missing"
    fi
}

# Main test execution
main() {
    echo "Starting SSE integration tests..."
    echo "Target: ${BASE_URL}"
    echo ""
    
    # Check prerequisites
    if ! command -v curl &> /dev/null; then
        echo -e "${RED}ERROR:${NC} curl is required but not installed"
        exit 1
    fi
    
    if ! command -v jq &> /dev/null; then
        echo -e "${RED}ERROR:${NC} jq is required but not installed"
        echo "Install with: brew install jq (macOS) or apt-get install jq (Linux)"
        exit 1
    fi
    
    # Run tests
    check_server
    echo ""
    
    test_health_check
    echo ""
    
    test_root_endpoint
    echo ""
    
    test_list_tools
    echo ""
    
    test_call_echo
    echo ""
    
    test_call_ping
    echo ""
    
    test_call_info
    echo ""
    
    test_sse_stream
    echo ""
    
    test_client_stats
    echo ""
    
    test_invalid_tool
    echo ""
    
    test_cors
    echo ""
    
    # Summary
    echo "========================================="
    echo "Test Summary"
    echo "========================================="
    echo -e "Total Tests:  ${TESTS_RUN}"
    echo -e "${GREEN}Passed:       ${TESTS_PASSED}${NC}"
    
    if [ $TESTS_FAILED -gt 0 ]; then
        echo -e "${RED}Failed:       ${TESTS_FAILED}${NC}"
        echo ""
        echo -e "${RED}Some tests failed!${NC}"
        exit 1
    else
        echo -e "${RED}Failed:       ${TESTS_FAILED}${NC}"
        echo ""
        echo -e "${GREEN}All tests passed!${NC}"
        echo ""
        echo "SSE server is working correctly!"
        echo ""
        echo "Next steps:"
        echo "  1. Open examples/sse_client.html in a browser"
        echo "  2. Test the interactive client"
        echo "  3. Watch real-time SSE events"
    fi
}

# Run main function
main