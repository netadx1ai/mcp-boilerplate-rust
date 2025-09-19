#!/bin/bash

# Blog Generation Server E2E Test Script
# This script runs comprehensive End-to-End tests for the blog generation server
# Following the patterns established in Phase 2.1 & 2.2 of the E2E testing framework

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BLOG_SERVER_DIR="$PROJECT_ROOT/examples/blog-generation-server"
TEST_TIMEOUT=30
SERVER_STARTUP_WAIT=0.5

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Utility functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((TESTS_PASSED++))
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((TESTS_FAILED++))
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

run_test() {
    local test_name="$1"
    local test_command="$2"
    
    ((TESTS_RUN++))
    log_info "Running test: $test_name"
    
    if timeout $TEST_TIMEOUT bash -c "$test_command"; then
        log_success "$test_name"
        return 0
    else
        log_error "$test_name"
        return 1
    fi
}

# Test functions
test_compilation() {
    log_info "🔨 Testing blog generation server compilation..."
    
    cd "$BLOG_SERVER_DIR"
    if cargo build --bin blog-generation-server --quiet; then
        log_success "Blog server compilation"
        return 0
    else
        log_error "Blog server compilation failed"
        return 1
    fi
}

test_help_command() {
    log_info "📖 Testing help command..."
    
    cd "$BLOG_SERVER_DIR"
    local help_output
    if help_output=$(cargo run --bin blog-generation-server --quiet -- --help 2>&1); then
        if echo "$help_output" | grep -q "blog-generation-server" && \
           echo "$help_output" | grep -q "transport" && \
           echo "$help_output" | grep -q "port" && \
           echo "$help_output" | grep -q "delay"; then
            log_success "Help command shows expected content"
            return 0
        else
            log_error "Help command missing expected content"
            return 1
        fi
    else
        log_error "Help command failed to execute"
        return 1
    fi
}

test_version_command() {
    log_info "🏷️ Testing version command..."
    
    cd "$BLOG_SERVER_DIR"
    local version_output
    if version_output=$(cargo run --bin blog-generation-server --quiet -- --version 2>&1); then
        if echo "$version_output" | grep -q "0.1.0"; then
            log_success "Version command shows correct version"
            return 0
        else
            log_error "Version command shows unexpected version"
            return 1
        fi
    else
        log_error "Version command failed to execute"
        return 1
    fi
}

test_invalid_arguments() {
    log_info "🚫 Testing invalid argument handling..."
    
    cd "$BLOG_SERVER_DIR"
    
    # Test invalid transport
    if cargo run --bin blog-generation-server --quiet -- --transport invalid 2>/dev/null; then
        log_error "Invalid transport should have failed"
        return 1
    else
        log_success "Invalid transport properly rejected"
    fi
    
    # Test invalid port
    if cargo run --bin blog-generation-server --quiet -- --port -1 2>/dev/null; then
        log_error "Invalid port should have failed"
        return 1
    else
        log_success "Invalid port properly rejected"
    fi
    
    # Test unknown flag
    if cargo run --bin blog-generation-server --quiet -- --unknown-flag 2>/dev/null; then
        log_error "Unknown flag should have failed"
        return 1
    else
        log_success "Unknown flag properly rejected"
    fi
    
    return 0
}

test_stdio_transport() {
    log_info "📡 Testing STDIO transport mode..."
    
    cd "$BLOG_SERVER_DIR"
    
    # Start server in background with STDIO transport
    local server_pid
    if cargo run --bin blog-generation-server --quiet -- --transport stdio > /dev/null 2>&1 & 
    then
        server_pid=$!
        sleep $SERVER_STARTUP_WAIT
        
        # Check if process is still running
        if kill -0 "$server_pid" 2>/dev/null; then
            log_success "STDIO transport server started successfully"
            kill "$server_pid" 2>/dev/null || true
            wait "$server_pid" 2>/dev/null || true
            return 0
        else
            log_error "STDIO transport server failed to start"
            return 1
        fi
    else
        log_error "Failed to start STDIO transport server"
        return 1
    fi
}

test_http_transport() {
    log_info "🌐 Testing HTTP transport mode..."
    
    cd "$BLOG_SERVER_DIR"
    
    # Use a unique port to avoid conflicts
    local test_port=3020
    local server_pid
    
    if cargo run --bin blog-generation-server --quiet -- --transport http --port $test_port > /dev/null 2>&1 &
    then
        server_pid=$!
        sleep $SERVER_STARTUP_WAIT
        
        # Check if process is still running
        if kill -0 "$server_pid" 2>/dev/null; then
            log_success "HTTP transport server started successfully on port $test_port"
            kill "$server_pid" 2>/dev/null || true
            wait "$server_pid" 2>/dev/null || true
            return 0
        else
            log_error "HTTP transport server failed to start"
            return 1
        fi
    else
        log_error "Failed to start HTTP transport server"
        return 1
    fi
}

test_delay_parameter() {
    log_info "⏱️ Testing delay parameter configurations..."
    
    cd "$BLOG_SERVER_DIR"
    
    for delay in 0 1 2 5; do
        local server_pid
        if cargo run --bin blog-generation-server --quiet -- --delay $delay > /dev/null 2>&1 &
        then
            server_pid=$!
            sleep $SERVER_STARTUP_WAIT
            
            if kill -0 "$server_pid" 2>/dev/null; then
                log_success "Delay parameter $delay seconds works"
                kill "$server_pid" 2>/dev/null || true
                wait "$server_pid" 2>/dev/null || true
            else
                log_error "Delay parameter $delay seconds failed"
                return 1
            fi
        else
            log_error "Failed to start server with delay $delay"
            return 1
        fi
    done
    
    return 0
}

test_debug_mode() {
    log_info "🐛 Testing debug mode..."
    
    cd "$BLOG_SERVER_DIR"
    
    local server_pid
    if cargo run --bin blog-generation-server --quiet -- --debug > /dev/null 2>&1 &
    then
        server_pid=$!
        sleep $SERVER_STARTUP_WAIT
        
        if kill -0 "$server_pid" 2>/dev/null; then
            log_success "Debug mode works correctly"
            kill "$server_pid" 2>/dev/null || true
            wait "$server_pid" 2>/dev/null || true
            return 0
        else
            log_error "Debug mode failed to start"
            return 1
        fi
    else
        log_error "Failed to start server in debug mode"
        return 1
    fi
}

test_blog_tool_structure() {
    log_info "📝 Testing blog generation tool structure..."
    
    # This test validates that the create_blog_post tool is properly structured
    # In a full implementation, this would test actual MCP communication
    # For now, we verify the server compiles with the tool correctly
    
    cd "$BLOG_SERVER_DIR"
    
    # Check that the source code contains the expected tool structure
    if grep -q "CreateBlogPostTool" src/main.rs && \
       grep -q "create_blog_post" src/main.rs && \
       grep -q "topic" src/main.rs && \
       grep -q "style" src/main.rs && \
       grep -q "word_count" src/main.rs; then
        log_success "Blog generation tool structure is correct"
        return 0
    else
        log_error "Blog generation tool structure is incomplete"
        return 1
    fi
}

test_performance_characteristics() {
    log_info "⚡ Testing performance characteristics..."
    
    cd "$BLOG_SERVER_DIR"
    
    # Test compilation time
    local start_time=$(date +%s.%N)
    if cargo build --bin blog-generation-server --quiet; then
        local end_time=$(date +%s.%N)
        local compile_time=$(echo "$end_time - $start_time" | bc -l)
        
        log_info "Compilation time: ${compile_time}s"
        
        # Compilation should be reasonable (< 30 seconds)
        if (( $(echo "$compile_time < 30" | bc -l) )); then
            log_success "Compilation time is acceptable"
        else
            log_warning "Compilation time is high: ${compile_time}s"
        fi
    else
        log_error "Performance test compilation failed"
        return 1
    fi
    
    # Test startup time with minimal delay
    local startup_start=$(date +%s.%N)
    local server_pid
    if cargo run --bin blog-generation-server --quiet -- --delay 0 > /dev/null 2>&1 &
    then
        server_pid=$!
        sleep 0.1  # Brief wait to let server start
        
        if kill -0 "$server_pid" 2>/dev/null; then
            local startup_end=$(date +%s.%N)
            local startup_time=$(echo "$startup_end - $startup_start" | bc -l)
            
            log_info "Server startup time: ${startup_time}s"
            
            # Startup should be quick (< 2 seconds)
            if (( $(echo "$startup_time < 2" | bc -l) )); then
                log_success "Server startup time is acceptable"
            else
                log_warning "Server startup time is high: ${startup_time}s"
            fi
            
            kill "$server_pid" 2>/dev/null || true
            wait "$server_pid" 2>/dev/null || true
        else
            log_error "Server failed to start for performance test"
            return 1
        fi
    else
        log_error "Failed to start server for performance test"
        return 1
    fi
    
    return 0
}

# Main test execution
main() {
    log_info "🚀 Starting Blog Generation Server E2E Tests"
    log_info "Project root: $PROJECT_ROOT"
    log_info "Blog server directory: $BLOG_SERVER_DIR"
    echo
    
    # Check that we're in the right directory
    if [ ! -f "$BLOG_SERVER_DIR/Cargo.toml" ]; then
        log_error "Blog generation server not found at $BLOG_SERVER_DIR"
        exit 1
    fi
    
    # Run all tests
    test_compilation
    test_help_command
    test_version_command
    test_invalid_arguments
    test_stdio_transport
    test_http_transport
    test_delay_parameter
    test_debug_mode
    test_blog_tool_structure
    test_performance_characteristics
    
    # Print summary
    echo
    log_info "📊 Test Summary:"
    log_info "Tests run: $TESTS_RUN"
    log_success "Tests passed: $TESTS_PASSED"
    
    if [ $TESTS_FAILED -gt 0 ]; then
        log_error "Tests failed: $TESTS_FAILED"
        echo
        log_error "❌ Blog Generation Server E2E Tests FAILED"
        exit 1
    else
        echo
        log_success "✅ Blog Generation Server E2E Tests PASSED"
        log_info "🎉 All blog generation server tests completed successfully!"
        exit 0
    fi
}

# Check dependencies
check_dependencies() {
    if ! command -v cargo &> /dev/null; then
        log_error "cargo is required but not installed"
        exit 1
    fi
    
    if ! command -v timeout &> /dev/null; then
        log_error "timeout command is required but not installed"
        exit 1
    fi
    
    if ! command -v bc &> /dev/null; then
        log_warning "bc command not found - performance timing may be inaccurate"
    fi
}

# Script entry point
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    check_dependencies
    main "$@"
fi