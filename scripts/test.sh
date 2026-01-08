#!/bin/bash

# MCP Boilerplate Rust - Test Script
# Simple script to test all endpoints with curl

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
HOST=${HOST:-localhost}
PORT=${PORT:-8025}
BASE_URL="http://${HOST}:${PORT}"

echo -e "${GREEN}=== MCP Boilerplate Rust - Test Suite ===${NC}\n"

# Function to print test header
print_test() {
    echo -e "${BLUE}Test: $1${NC}"
}

# Function to print success
print_success() {
    echo -e "${GREEN}✓ Success${NC}\n"
}

# Function to print error
print_error() {
    echo -e "${RED}✗ Failed: $1${NC}\n"
}

# Test 1: Health Check
print_test "Health Check"
curl -s -X GET "${BASE_URL}/health" | jq '.' || print_error "Health check failed"
print_success

# Test 2: Root endpoint
print_test "Root Endpoint"
curl -s -X GET "${BASE_URL}/" | jq '.' || print_error "Root endpoint failed"
print_success

# Test 3: Echo Tool - Echo Action
print_test "Echo Tool - Echo Action"
curl -s -X POST "${BASE_URL}/tools/echo" \
  -H "Content-Type: application/json" \
  -d '{"action":"echo","message":"Hello from MCP Rust!"}' | jq '.' || print_error "Echo action failed"
print_success

# Test 4: Echo Tool - Ping Action
print_test "Echo Tool - Ping Action"
curl -s -X POST "${BASE_URL}/tools/echo" \
  -H "Content-Type: application/json" \
  -d '{"action":"ping"}' | jq '.' || print_error "Ping action failed"
print_success

# Test 5: Echo Tool - Info Action
print_test "Echo Tool - Info Action"
curl -s -X POST "${BASE_URL}/tools/echo" \
  -H "Content-Type: application/json" \
  -d '{"action":"info"}' | jq '.' || print_error "Info action failed"
print_success

# Test 6: Echo Tool - Invalid Action (should fail gracefully)
print_test "Echo Tool - Invalid Action (Expected Error)"
curl -s -X POST "${BASE_URL}/tools/echo" \
  -H "Content-Type: application/json" \
  -d '{"action":"invalid"}' | jq '.' || print_error "Invalid action test failed"
print_success

# Test 7: Echo Tool - Missing Message Parameter (should fail gracefully)
print_test "Echo Tool - Missing Parameter (Expected Error)"
curl -s -X POST "${BASE_URL}/tools/echo" \
  -H "Content-Type: application/json" \
  -d '{"action":"echo"}' | jq '.' || print_error "Missing parameter test failed"
print_success

echo -e "${GREEN}=== All Tests Completed ===${NC}"
echo ""
echo "Server: ${BASE_URL}"
echo ""
echo "Available endpoints:"
echo "  GET  ${BASE_URL}/health"
echo "  POST ${BASE_URL}/tools/echo"
echo ""
echo "Example commands:"
echo ""
echo "# Echo a message:"
echo "curl -X POST ${BASE_URL}/tools/echo \\"
echo "  -H 'Content-Type: application/json' \\"
echo "  -d '{\"action\":\"echo\",\"message\":\"Hello MCP!\"}'"
echo ""
echo "# Ping test:"
echo "curl -X POST ${BASE_URL}/tools/echo \\"
echo "  -H 'Content-Type: application/json' \\"
echo "  -d '{\"action\":\"ping\"}'"
echo ""
echo "# Get tool info:"
echo "curl -X POST ${BASE_URL}/tools/echo \\"
echo "  -H 'Content-Type: application/json' \\"
echo "  -d '{\"action\":\"info\"}'"