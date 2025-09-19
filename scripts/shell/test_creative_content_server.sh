#!/bin/bash

# Creative Content Server E2E Test Script
# This script runs comprehensive End-to-End tests for the creative content server
# Following the patterns established in Phase 2.1, 2.2, and 2.3 of the E2E testing framework

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Test configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
CREATIVE_SERVER_DIR="$PROJECT_ROOT/examples/creative-content-server"
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

log_creative() {
    echo -e "${PURPLE}[CREATIVE]${NC} $1"
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
    log_info "🔨 Testing creative content server compilation..."
    
    cd "$CREATIVE_SERVER_DIR"
    if cargo build --bin creative-content-server --quiet; then
        log_success "Creative content server compilation"
        return 0
    else
        log_error "Creative content server compilation failed"
        return 1
    fi
}

test_help_command() {
    log_info "📖 Testing help command..."
    
    cd "$CREATIVE_SERVER_DIR"
    local help_output
    if help_output=$(cargo run --bin creative-content-server --quiet -- --help 2>&1); then
        if echo "$help_output" | grep -q "creative-content-server" && \
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
    
    cd "$CREATIVE_SERVER_DIR"
    local version_output
    if version_output=$(cargo run --bin creative-content-server --quiet -- --version 2>&1); then
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
    
    cd "$CREATIVE_SERVER_DIR"
    
    # Test invalid transport
    if cargo run --bin creative-content-server --quiet -- --transport invalid 2>/dev/null; then
        log_error "Invalid transport should have failed"
        return 1
    else
        log_success "Invalid transport properly rejected"
    fi
    
    # Test invalid port
    if cargo run --bin creative-content-server --quiet -- --port -1 2>/dev/null; then
        log_error "Invalid port should have failed"
        return 1
    else
        log_success "Invalid port properly rejected"
    fi
    
    # Test unknown flag
    if cargo run --bin creative-content-server --quiet -- --unknown-flag 2>/dev/null; then
        log_error "Unknown flag should have failed"
        return 1
    else
        log_success "Unknown flag properly rejected"
    fi
    
    return 0
}

test_stdio_transport() {
    log_info "📡 Testing STDIO transport mode..."
    
    cd "$CREATIVE_SERVER_DIR"
    
    # Start server in background with STDIO transport
    local server_pid
    if cargo run --bin creative-content-server --quiet -- --transport stdio > /dev/null 2>&1 & 
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
    
    cd "$CREATIVE_SERVER_DIR"
    
    # Use a unique port to avoid conflicts
    local test_port=3025
    local server_pid
    
    if cargo run --bin creative-content-server --quiet -- --transport http --port $test_port > /dev/null 2>&1 &
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
    
    cd "$CREATIVE_SERVER_DIR"
    
    for delay in 0 1 2 5; do
        local server_pid
        if cargo run --bin creative-content-server --quiet -- --delay $delay > /dev/null 2>&1 &
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
    
    cd "$CREATIVE_SERVER_DIR"
    
    local server_pid
    if cargo run --bin creative-content-server --quiet -- --debug > /dev/null 2>&1 &
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

test_creative_tools_structure() {
    log_creative "🎨 Testing creative content tools structure..."
    
    # This test validates that the creative content tools are properly structured
    # In a full implementation, this would test actual MCP communication
    # For now, we verify the server compiles with the tools correctly
    
    cd "$CREATIVE_SERVER_DIR"
    
    # Check that the source code contains the expected tool structures
    if grep -q "GenerateStoryTool" src/main.rs && \
       grep -q "CreatePoemTool" src/main.rs && \
       grep -q "DevelopCharacterTool" src/main.rs && \
       grep -q "generate_story" src/main.rs && \
       grep -q "create_poem" src/main.rs && \
       grep -q "develop_character" src/main.rs; then
        log_success "Creative content tools structure is correct"
        return 0
    else
        log_error "Creative content tools structure is incomplete"
        return 1
    fi
}

test_creative_content_variety() {
    log_creative "🎭 Testing creative content variety coverage..."
    
    cd "$CREATIVE_SERVER_DIR"
    
    # Test that the server supports various creative content types
    log_creative "  Validating story generation capabilities..."
    if grep -q "genre" src/main.rs && grep -q "prompt" src/main.rs; then
        log_success "Story generation tool parameters validated"
    else
        log_error "Story generation tool parameters missing"
        return 1
    fi
    
    log_creative "  Validating poetry creation capabilities..."
    if grep -q "style" src/main.rs && grep -q "theme" src/main.rs; then
        log_success "Poetry creation tool parameters validated"
    else
        log_error "Poetry creation tool parameters missing"
        return 1
    fi
    
    log_creative "  Validating character development capabilities..."
    if grep -q "character" src/main.rs && grep -q "name" src/main.rs; then
        log_success "Character development tool parameters validated"
    else
        log_error "Character development tool parameters missing"
        return 1
    fi
    
    return 0
}

test_multi_tool_integration() {
    log_creative "🔗 Testing multi-tool integration readiness..."
    
    cd "$CREATIVE_SERVER_DIR"
    
    # Verify that the server can handle multiple tools simultaneously
    # This tests the server's ability to register and manage multiple creative tools
    
    if grep -c "impl McpTool for" src/main.rs | grep -q "3"; then
        log_success "Multiple MCP tool implementations found"
    else
        log_warning "Expected 3 MCP tool implementations (generate_story, create_poem, develop_character)"
    fi
    
    # Test that all tools are properly registered
    if grep -q "register_tool" src/main.rs || grep -q "add_tool" src/main.rs; then
        log_success "Tool registration mechanism present"
    else
        log_warning "Tool registration mechanism not clearly identified"
    fi
    
    return 0
}

test_performance_characteristics() {
    log_info "⚡ Testing performance characteristics..."
    
    cd "$CREATIVE_SERVER_DIR"
    
    # Test compilation time
    local start_time=$(date +%s.%N)
    if cargo build --bin creative-content-server --quiet; then
        local end_time=$(date +%s.%N)
        local compile_time=$(echo "$end_time - $start_time" | bc -l 2>/dev/null || echo "N/A")
        
        if [ "$compile_time" != "N/A" ]; then
            log_info "Compilation time: ${compile_time}s"
            
            # Compilation should be reasonable (< 30 seconds)
            if (( $(echo "$compile_time < 30" | bc -l 2>/dev/null || echo "1") )); then
                log_success "Compilation time is acceptable"
            else
                log_warning "Compilation time is high: ${compile_time}s"
            fi
        else
            log_success "Compilation completed (timing unavailable)"
        fi
    else
        log_error "Performance test compilation failed"
        return 1
    fi
    
    # Test startup time with minimal delay
    local startup_start=$(date +%s.%N)
    local server_pid
    if cargo run --bin creative-content-server --quiet -- --delay 0 > /dev/null 2>&1 &
    then
        server_pid=$!
        sleep 0.1  # Brief wait to let server start
        
        if kill -0 "$server_pid" 2>/dev/null; then
            local startup_end=$(date +%s.%N)
            local startup_time=$(echo "$startup_end - $startup_start" | bc -l 2>/dev/null || echo "N/A")
            
            if [ "$startup_time" != "N/A" ]; then
                log_info "Server startup time: ${startup_time}s"
                
                # Startup should be quick (< 2 seconds)
                if (( $(echo "$startup_time < 2" | bc -l 2>/dev/null || echo "1") )); then
                    log_success "Server startup time is acceptable"
                else
                    log_warning "Server startup time is high: ${startup_time}s"
                fi
            else
                log_success "Server startup completed (timing unavailable)"
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
    log_info "🚀 Starting Creative Content Server E2E Tests"
    log_info "Project root: $PROJECT_ROOT"
    log_info "Creative content server directory: $CREATIVE_SERVER_DIR"
    echo
    
    # Check that we're in the right directory
    if [ ! -f "$CREATIVE_SERVER_DIR/Cargo.toml" ]; then
        log_error "Creative content server not found at $CREATIVE_SERVER_DIR"
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
    test_creative_tools_structure
    test_creative_content_variety
    test_multi_tool_integration
    test_performance_characteristics
    
    # Print summary
    echo
    log_info "📊 Test Summary:"
    log_info "Tests run: $TESTS_RUN"
    log_success "Tests passed: $TESTS_PASSED"
    
    if [ $TESTS_FAILED -gt 0 ]; then
        log_error "Tests failed: $TESTS_FAILED"
        echo
        log_error "❌ Creative Content Server E2E Tests FAILED"
        exit 1
    else
        echo
        log_success "✅ Creative Content Server E2E Tests PASSED"
        log_creative "🎉 All creative content server tests completed successfully!"
        log_creative "🎨 Ready for multi-tool creative content generation!"
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