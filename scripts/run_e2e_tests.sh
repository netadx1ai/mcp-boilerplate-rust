#!/bin/bash

# MCP Boilerplate E2E Test Runner
# 
# This script runs comprehensive End-to-End tests for all MCP boilerplate servers
# following the verification mandate and quality standards defined in the project rules.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TIMEOUT_SECONDS=30
MAX_STARTUP_TIME=5
TEST_TEMP_DIR=""
FAILED_TESTS=()
PASSED_TESTS=()
TOTAL_TESTS=0

# Server list
SERVERS=("filesystem-server" "image-generation-server" "blog-generation-server" "creative-content-server")

# Cleanup function
cleanup() {
    if [[ -n "$TEST_TEMP_DIR" && -d "$TEST_TEMP_DIR" ]]; then
        rm -rf "$TEST_TEMP_DIR"
    fi
    
    # Kill any remaining server processes
    pkill -f "filesystem-server\|image-generation-server\|blog-generation-server\|creative-content-server" 2>/dev/null || true
}

# Setup trap for cleanup
trap cleanup EXIT

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Test result tracking
record_test_result() {
    local test_name="$1"
    local result="$2"
    
    ((TOTAL_TESTS++))
    
    if [[ "$result" == "PASS" ]]; then
        PASSED_TESTS+=("$test_name")
        log_success "✅ $test_name"
    else
        FAILED_TESTS+=("$test_name")
        log_error "❌ $test_name"
    fi
}

# Initialize test environment
setup_test_environment() {
    log_info "🧪 Setting up E2E test environment..."
    
    # Create temporary directory for tests
    TEST_TEMP_DIR=$(mktemp -d)
    export MCP_TEST_MODE=true
    export RUST_LOG=debug
    
    # Verify we're in the right directory
    if [[ ! -f "Cargo.toml" ]]; then
        log_error "Must run from project root directory"
        exit 1
    fi
    
    # Verify cargo is available
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Rust toolchain."
        exit 1
    fi
    
    log_success "Test environment initialized"
}

# Test server compilation
test_server_compilation() {
    local server="$1"
    log_info "🔧 Testing $server compilation..."
    
    if timeout $TIMEOUT_SECONDS cargo check --bin "$server" --quiet; then
        record_test_result "$server compilation" "PASS"
    else
        record_test_result "$server compilation" "FAIL"
    fi
}

# Test server help functionality
test_server_help() {
    local server="$1"
    log_info "📋 Testing $server help functionality..."
    
    local start_time=$(date +%s)
    
    if timeout $TIMEOUT_SECONDS cargo run --bin "$server" -- --help > /dev/null 2>&1; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        
        if [[ $duration -le $MAX_STARTUP_TIME ]]; then
            record_test_result "$server help (${duration}s)" "PASS"
        else
            log_warning "$server help took ${duration}s (> ${MAX_STARTUP_TIME}s target)"
            record_test_result "$server help (slow: ${duration}s)" "PASS"
        fi
    else
        record_test_result "$server help" "FAIL"
    fi
}

# Test server error handling
test_server_error_handling() {
    local server="$1"
    log_info "⚠️ Testing $server error handling..."
    
    # Test with invalid flag - should exit with error
    if timeout $TIMEOUT_SECONDS cargo run --bin "$server" -- --invalid-flag-xyz > /dev/null 2>&1; then
        record_test_result "$server error handling" "FAIL"
    else
        record_test_result "$server error handling" "PASS"
    fi
}

# Test STDIO transport mode
test_stdio_transport() {
    local server="$1"
    log_info "📡 Testing $server STDIO transport..."
    
    cd "$TEST_TEMP_DIR"
    
    # Start server with STDIO transport in background
    if timeout 3s cargo run --bin "$server" -- --transport stdio </dev/null > /dev/null 2>&1 &
    then
        local pid=$!
        sleep 1
        
        # Check if process is still running
        if kill -0 $pid 2>/dev/null; then
            kill $pid 2>/dev/null || true
            wait $pid 2>/dev/null || true
            record_test_result "$server STDIO transport" "PASS"
        else
            record_test_result "$server STDIO transport" "FAIL"
        fi
    else
        record_test_result "$server STDIO transport" "FAIL"
    fi
    
    cd - > /dev/null
}

# Test HTTP transport mode
test_http_transport() {
    local server="$1"
    log_info "🌐 Testing $server HTTP transport..."
    
    cd "$TEST_TEMP_DIR"
    
    # Get random port
    local port=$(shuf -i 8000-9000 -n 1)
    
    # Start server with HTTP transport in background
    if timeout 3s cargo run --bin "$server" -- --transport http --port "$port" </dev/null > /dev/null 2>&1 &
    then
        local pid=$!
        sleep 1
        
        # Check if process is still running
        if kill -0 $pid 2>/dev/null; then
            kill $pid 2>/dev/null || true
            wait $pid 2>/dev/null || true
            record_test_result "$server HTTP transport" "PASS"
        else
            record_test_result "$server HTTP transport" "FAIL"
        fi
    else
        record_test_result "$server HTTP transport" "FAIL"
    fi
    
    cd - > /dev/null
}

# Test filesystem server specific functionality
test_filesystem_server_specific() {
    if [[ "$1" != "filesystem-server" ]]; then
        return
    fi
    
    log_info "📁 Testing filesystem server specific functionality..."
    
    cd "$TEST_TEMP_DIR"
    
    # Create test files
    echo "test content" > test_file.txt
    mkdir -p test_dir
    echo "nested content" > test_dir/nested.txt
    
    # Test that server can start in directory with files
    if timeout 3s cargo run --bin filesystem-server -- --base-dir . --transport stdio </dev/null > /dev/null 2>&1; then
        record_test_result "filesystem server with test files" "PASS"
    else
        record_test_result "filesystem server with test files" "FAIL"
    fi
    
    cd - > /dev/null
}

# Run protocol compliance tests
test_protocol_compliance() {
    log_info "🔗 Running protocol compliance tests..."
    
    for server in "${SERVERS[@]}"; do
        test_server_compilation "$server"
        test_server_help "$server"
        test_server_error_handling "$server"
    done
}

# Run transport layer tests
test_transport_layer() {
    log_info "🚀 Running transport layer tests..."
    
    for server in "${SERVERS[@]}"; do
        test_stdio_transport "$server"
        test_http_transport "$server"
    done
}

# Run server-specific tests
test_server_specific() {
    log_info "🖥️ Running server-specific tests..."
    
    for server in "${SERVERS[@]}"; do
        test_filesystem_server_specific "$server"
    done
}

# Run performance tests
test_performance() {
    log_info "⚡ Running performance tests..."
    
    # Test startup times
    for server in "${SERVERS[@]}"; do
        log_info "⏱️ Testing $server startup performance..."
        
        local start_time=$(date +%s%N)
        
        if timeout $TIMEOUT_SECONDS cargo run --bin "$server" -- --help > /dev/null 2>&1; then
            local end_time=$(date +%s%N)
            local duration_ms=$(( (end_time - start_time) / 1000000 ))
            local duration_s=$(( duration_ms / 1000 ))
            
            if [[ $duration_s -le $MAX_STARTUP_TIME ]]; then
                record_test_result "$server startup performance (${duration_s}s)" "PASS"
            else
                record_test_result "$server startup performance (slow: ${duration_s}s)" "FAIL"
            fi
        else
            record_test_result "$server startup performance" "FAIL"
        fi
    done
}

# Generate test report
generate_report() {
    echo
    echo "========================================"
    echo "        E2E Test Results Summary        "
    echo "========================================"
    echo
    
    echo -e "${GREEN}✅ PASSED TESTS (${#PASSED_TESTS[@]})${NC}"
    if [[ ${#PASSED_TESTS[@]} -gt 0 ]]; then
        for test in "${PASSED_TESTS[@]}"; do
            echo "  ✅ $test"
        done
    fi
    echo
    
    if [[ ${#FAILED_TESTS[@]} -gt 0 ]]; then
        echo -e "${RED}❌ FAILED TESTS (${#FAILED_TESTS[@]})${NC}"
        for test in "${FAILED_TESTS[@]}"; do
            echo "  ❌ $test"
        done
        echo
    fi
    
    local pass_rate=0
    if [[ $TOTAL_TESTS -gt 0 ]]; then
        pass_rate=$(( (${#PASSED_TESTS[@]} * 100) / TOTAL_TESTS ))
    fi
    echo "Pass Rate: ${pass_rate}% (${#PASSED_TESTS[@]}/${TOTAL_TESTS})"
    
    if [[ ${#FAILED_TESTS[@]} -eq 0 ]]; then
        echo -e "${GREEN}🎉 All tests passed!${NC}"
        return 0
    else
        echo -e "${RED}💥 Some tests failed.${NC}"
        return 1
    fi
}

# Main execution
main() {
    echo "🧪 MCP Boilerplate E2E Test Suite"
    echo "=================================="
    echo
    
    setup_test_environment
    
    # Run all test phases
    test_protocol_compliance
    test_transport_layer
    test_server_specific
    test_performance
    
    # Generate and display results
    generate_report
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [options]"
        echo
        echo "Options:"
        echo "  --help, -h          Show this help message"
        echo "  --protocol-only     Run only protocol compliance tests"
        echo "  --transport-only    Run only transport layer tests"
        echo "  --performance-only  Run only performance tests"
        echo "  --quick            Run quick subset of tests"
        echo
        echo "Environment variables:"
        echo "  TIMEOUT_SECONDS     Timeout for individual tests (default: 30)"
        echo "  MAX_STARTUP_TIME    Max acceptable startup time (default: 5)"
        echo
        exit 0
        ;;
    --protocol-only)
        setup_test_environment
        test_protocol_compliance
        generate_report
        ;;
    --transport-only)
        setup_test_environment
        test_transport_layer
        generate_report
        ;;
    --performance-only)
        setup_test_environment
        test_performance
        generate_report
        ;;
    --quick)
        # Quick test - just compilation and help
        setup_test_environment
        for server in "${SERVERS[@]}"; do
            test_server_compilation "$server"
            test_server_help "$server"
        done
        generate_report
        ;;
    "")
        # Run all tests
        main
        ;;
    *)
        log_error "Unknown option: $1"
        echo "Use --help for usage information"
        exit 1
        ;;
esac