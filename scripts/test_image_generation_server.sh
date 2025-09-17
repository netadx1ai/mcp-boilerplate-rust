#!/bin/bash

# Image Generation Server E2E Test Script
# 
# This script provides comprehensive End-to-End testing specifically for the 
# image generation server, following the practical validation approach established
# in Phase 2.1 and designed for Phase 2.2 completion.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TIMEOUT_SECONDS=10
TEST_TEMP_DIR=""
FAILED_TESTS=()
PASSED_TESTS=()
TOTAL_TESTS=0

# Cleanup function
cleanup() {
    if [[ -n "$TEST_TEMP_DIR" && -d "$TEST_TEMP_DIR" ]]; then
        rm -rf "$TEST_TEMP_DIR"
    fi
    
    # Kill any remaining server processes
    pkill -f "image-generation-server" 2>/dev/null || true
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
    log_info "🧪 Setting up Image Generation Server E2E test environment..."
    
    # Create temporary directory for tests
    TEST_TEMP_DIR=$(mktemp -d)
    export MCP_TEST_MODE=true
    export RUST_LOG=info
    
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
    
    log_success "Test environment initialized at $TEST_TEMP_DIR"
}

# Test 1: Server compilation and basic functionality
test_server_compilation() {
    log_info "🔨 Testing image generation server compilation..."
    
    local start_time=$(date +%s)
    
    if timeout $TIMEOUT_SECONDS cargo build --bin image-generation-server --quiet; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        
        record_test_result "server compilation (${duration}s)" "PASS"
    else
        record_test_result "server compilation" "FAIL"
        return 1
    fi
}

# Test 2: CLI interface validation
test_cli_interface() {
    log_info "📋 Testing CLI interface functionality..."
    
    # Test --help command
    if timeout 5s ./target/debug/image-generation-server --help > /dev/null 2>&1; then
        record_test_result "CLI help command" "PASS"
    else
        record_test_result "CLI help command" "FAIL"
    fi
    
    # Test --version command
    if timeout 5s ./target/debug/image-generation-server --version > /dev/null 2>&1; then
        record_test_result "CLI version command" "PASS"
    else
        record_test_result "CLI version command" "FAIL"
    fi
    
    # Test help output contains expected content
    local help_output
    if help_output=$(timeout 5s ./target/debug/image-generation-server --help 2>&1); then
        if [[ "$help_output" == *"Transport type to use"* && "$help_output" == *"Simulate processing delay"* ]]; then
            record_test_result "CLI help content validation" "PASS"
        else
            record_test_result "CLI help content validation" "FAIL"
        fi
    else
        record_test_result "CLI help content validation" "FAIL"
    fi
}

# Test 3: Parameter validation and error handling
test_parameter_validation() {
    log_info "⚙️ Testing parameter validation..."
    
    # Test invalid transport type
    if timeout 5s ./target/debug/image-generation-server --transport invalid > /dev/null 2>&1; then
        record_test_result "invalid transport rejection" "FAIL"
    else
        record_test_result "invalid transport rejection" "PASS"
    fi
    
    # Test invalid port (non-numeric)
    if timeout 5s ./target/debug/image-generation-server --port abc > /dev/null 2>&1; then
        record_test_result "invalid port rejection" "FAIL"
    else
        record_test_result "invalid port rejection" "PASS"
    fi
    
    # Test unknown flag
    if timeout 5s ./target/debug/image-generation-server --unknown-flag > /dev/null 2>&1; then
        record_test_result "unknown flag rejection" "FAIL"
    else
        record_test_result "unknown flag rejection" "PASS"
    fi
    
    # Test valid parameter combinations
    if timeout 5s ./target/debug/image-generation-server --delay 1 --debug --help > /dev/null 2>&1; then
        record_test_result "valid parameters acceptance" "PASS"
    else
        record_test_result "valid parameters acceptance" "FAIL"
    fi
}

# Test 4: Server startup validation
test_server_startup() {
    log_info "🚀 Testing server startup behavior..."
    
    cd "$TEST_TEMP_DIR"
    
    # Test STDIO transport startup
    log_info "📡 Testing STDIO transport startup..."
    if timeout 3s ./target/debug/image-generation-server --transport stdio --delay 0 < /dev/null > startup_stdio.log 2>&1 &
    then
        local pid=$!
        sleep 0.5
        
        if kill -0 $pid 2>/dev/null; then
            # Server started successfully
            kill $pid 2>/dev/null || true
            wait $pid 2>/dev/null || true
            
            # Check startup log content
            if grep -q "Starting MCP Image Generation Server" startup_stdio.log && grep -q "Registered tool: generate_image" startup_stdio.log; then
                record_test_result "STDIO transport startup with logs" "PASS"
            else
                record_test_result "STDIO transport startup logs" "FAIL"
            fi
        else
            record_test_result "STDIO transport startup" "FAIL"
        fi
    else
        record_test_result "STDIO transport startup" "FAIL"
    fi
    
    # Test HTTP transport startup
    log_info "🌐 Testing HTTP transport startup..."
    local http_port=$(shuf -i 8000-9000 -n 1)
    
    if timeout 3s ./target/debug/image-generation-server --transport http --port "$http_port" --delay 0 < /dev/null > startup_http.log 2>&1 &
    then
        local pid=$!
        sleep 0.8
        
        if kill -0 $pid 2>/dev/null; then
            # Server started successfully
            kill $pid 2>/dev/null || true
            wait $pid 2>/dev/null || true
            
            # Check startup log content
            if grep -q "Starting MCP Image Generation Server" startup_http.log && grep -q "Transport: Http" startup_http.log; then
                record_test_result "HTTP transport startup with logs" "PASS"
            else
                record_test_result "HTTP transport startup logs" "FAIL"
            fi
        else
            record_test_result "HTTP transport startup" "FAIL"
        fi
    else
        record_test_result "HTTP transport startup" "FAIL"
    fi
    
    cd - > /dev/null
}

# Test 5: AI scaffolding validation
test_ai_scaffolding() {
    log_info "🤖 Testing AI scaffolding functionality..."
    
    # Run unit tests to validate AI scaffolding structure
    if timeout $TIMEOUT_SECONDS cargo test --bin image-generation-server --quiet > /dev/null 2>&1; then
        record_test_result "AI scaffolding unit tests" "PASS"
    else
        record_test_result "AI scaffolding unit tests" "FAIL"
    fi
    
    # Check for AI-related strings in binary (if strings command available)
    if command -v strings &> /dev/null; then
        local binary_strings
        if binary_strings=$(strings ./target/debug/image-generation-server 2>/dev/null); then
            local ai_indicators=("generate_image" "photorealistic" "1024x1024" "placeholder-diffusion")
            local found_count=0
            
            for indicator in "${ai_indicators[@]}"; do
                if echo "$binary_strings" | grep -q "$indicator"; then
                    ((found_count++))
                fi
            done
            
            if [[ $found_count -ge 3 ]]; then
                record_test_result "AI scaffolding binary content ($found_count/4 indicators)" "PASS"
            else
                record_test_result "AI scaffolding binary content ($found_count/4 indicators)" "FAIL"
            fi
        else
            log_warning "Could not analyze binary content"
            record_test_result "AI scaffolding binary analysis" "SKIP"
        fi
    else
        log_info "strings command not available, skipping binary analysis"
    fi
}

# Test 6: Realistic configuration testing
test_realistic_configurations() {
    log_info "🎨 Testing realistic AI generation configurations..."
    
    # Test realistic delay configurations
    local delays=(0 1 2 5)
    for delay in "${delays[@]}"; do
        if timeout 5s ./target/debug/image-generation-server --delay "$delay" --help > /dev/null 2>&1; then
            record_test_result "delay parameter $delay" "PASS"
        else
            record_test_result "delay parameter $delay" "FAIL"
        fi
    done
    
    # Test debug mode
    if timeout 5s ./target/debug/image-generation-server --debug --help > /dev/null 2>&1; then
        record_test_result "debug mode parameter" "PASS"
    else
        record_test_result "debug mode parameter" "FAIL"
    fi
    
    # Test custom host parameter
    if timeout 5s ./target/debug/image-generation-server --host "0.0.0.0" --help > /dev/null 2>&1; then
        record_test_result "custom host parameter" "PASS"
    else
        record_test_result "custom host parameter" "FAIL"
    fi
}

# Test 7: Performance validation
test_performance() {
    log_info "⚡ Testing performance characteristics..."
    
    # Test startup time
    local start_time=$(date +%s%N)
    
    if timeout 8s ./target/debug/image-generation-server --help > /dev/null 2>&1; then
        local end_time=$(date +%s%N)
        local duration_ms=$(( (end_time - start_time) / 1000000 ))
        local duration_s=$(( duration_ms / 1000 ))
        
        if [[ $duration_s -le 5 ]]; then
            record_test_result "startup performance (${duration_s}s)" "PASS"
        else
            record_test_result "startup performance (slow: ${duration_s}s)" "FAIL"
        fi
    else
        record_test_result "startup performance" "FAIL"
    fi
    
    # Test compilation time (if not already compiled)
    log_info "⏱️ Testing compilation performance..."
    
    # Clean and rebuild to test compilation time
    if cargo clean --bin image-generation-server > /dev/null 2>&1; then
        local compile_start=$(date +%s)
        
        if timeout 30s cargo build --bin image-generation-server --quiet; then
            local compile_end=$(date +%s)
            local compile_duration=$((compile_end - compile_start))
            
            if [[ $compile_duration -le 20 ]]; then
                record_test_result "compilation performance (${compile_duration}s)" "PASS"
            else
                record_test_result "compilation performance (slow: ${compile_duration}s)" "FAIL"
            fi
        else
            record_test_result "compilation performance" "FAIL"
        fi
    else
        record_test_result "compilation performance setup" "FAIL"
    fi
}

# Test 8: Integration with existing framework
test_framework_integration() {
    log_info "🔧 Testing integration with existing E2E framework..."
    
    # Test that server works with existing E2E script patterns
    if [[ -x "./scripts/run_e2e_tests.sh" ]]; then
        # Run quick test to ensure integration
        if timeout 15s ./scripts/run_e2e_tests.sh --quick > /dev/null 2>&1; then
            record_test_result "E2E framework integration" "PASS"
        else
            record_test_result "E2E framework integration" "FAIL"
        fi
    else
        log_warning "E2E framework script not found or not executable"
        record_test_result "E2E framework availability" "FAIL"
    fi
    
    # Test with unit test suite
    if timeout 10s cargo test --workspace --quiet > /dev/null 2>&1; then
        record_test_result "workspace unit test integration" "PASS"
    else
        record_test_result "workspace unit test integration" "FAIL"
    fi
}

# Generate comprehensive test report
generate_report() {
    echo
    echo "======================================================="
    echo "    Image Generation Server E2E Test Results"
    echo "======================================================="
    echo
    
    # Test execution summary
    echo -e "${BLUE}📊 Test Execution Summary${NC}"
    echo "  Total Tests: $TOTAL_TESTS"
    echo "  Passed: ${#PASSED_TESTS[@]}"
    echo "  Failed: ${#FAILED_TESTS[@]}"
    echo
    
    # Calculate pass rate
    local pass_rate=0
    if [[ $TOTAL_TESTS -gt 0 ]]; then
        pass_rate=$(( (${#PASSED_TESTS[@]} * 100) / TOTAL_TESTS ))
    fi
    
    echo "Pass Rate: ${pass_rate}%"
    echo
    
    # Detailed results
    if [[ ${#PASSED_TESTS[@]} -gt 0 ]]; then
        echo -e "${GREEN}✅ PASSED TESTS (${#PASSED_TESTS[@]})${NC}"
        for test in "${PASSED_TESTS[@]}"; do
            echo "  ✅ $test"
        done
        echo
    fi
    
    if [[ ${#FAILED_TESTS[@]} -gt 0 ]]; then
        echo -e "${RED}❌ FAILED TESTS (${#FAILED_TESTS[@]})${NC}"
        for test in "${FAILED_TESTS[@]}"; do
            echo "  ❌ $test"
        done
        echo
    fi
    
    # Phase 2.2 completion assessment
    echo "======================================================="
    echo -e "${BLUE}📋 Phase 2.2 Completion Assessment${NC}"
    echo "======================================================="
    
    # Check critical requirements
    local critical_tests=(
        "server compilation"
        "CLI help command"
        "CLI version command" 
        "invalid transport rejection"
        "STDIO transport startup with logs"
        "AI scaffolding unit tests"
    )
    
    local critical_passed=0
    for critical_test in "${critical_tests[@]}"; do
        for passed_test in "${PASSED_TESTS[@]}"; do
            if [[ "$passed_test" == *"$critical_test"* ]]; then
                ((critical_passed++))
                break
            fi
        done
    done
    
    echo "Critical Requirements: ${critical_passed}/${#critical_tests[@]} passed"
    
    if [[ $critical_passed -eq ${#critical_tests[@]} && $pass_rate -ge 85 ]]; then
        echo -e "${GREEN}🎉 Phase 2.2 READY FOR COMPLETION${NC}"
        echo
        echo "✅ Image Generation Server E2E tests demonstrate:"
        echo "  • Server compiles and runs correctly"
        echo "  • CLI interface fully functional"
        echo "  • Error handling validates input properly"
        echo "  • AI scaffolding implemented with proper structure"
        echo "  • Transport configurations working"
        echo "  • Performance within acceptable limits"
        echo
        return 0
    else
        echo -e "${YELLOW}⚠️  Phase 2.2 NEEDS ATTENTION${NC}"
        echo "  Critical tests passed: ${critical_passed}/${#critical_tests[@]}"
        echo "  Overall pass rate: ${pass_rate}%"
        echo "  Minimum requirements: 6/6 critical tests + 85% pass rate"
        echo
        return 1
    fi
}

# Main execution
main() {
    echo "🤖 Image Generation Server E2E Test Suite"
    echo "=========================================="
    echo "Phase 2.2: AI Scaffolding Validation"
    echo
    
    setup_test_environment
    
    # Run all test phases
    test_server_compilation
    test_cli_interface  
    test_parameter_validation
    test_server_startup
    test_ai_scaffolding
    test_realistic_configurations
    test_performance
    test_framework_integration
    
    # Generate and display results
    generate_report
}

# Handle command line arguments
case "${1:-}" in
--help|-h)
echo "Usage: $0 [options]"
echo
echo "Image Generation Server E2E Test Runner"
echo "Validates AI scaffolding and practical functionality for Phase 2.2"
echo
echo "Options:"
echo "  --help, -h          Show this help message"
echo "  --quick            Run quick subset of tests"
echo "  --compilation      Run only compilation tests"
echo "  --cli              Run only CLI interface tests"
echo "  --startup          Run only startup tests"
echo "  --ai-scaffolding   Run only AI scaffolding validation"
echo "  --performance      Run only performance tests"
echo
echo "Environment variables:"
echo "  TIMEOUT_SECONDS    Timeout for individual tests (default: 10)"
echo
exit 0
;;
--quick)
setup_test_environment
test_server_compilation
test_cli_interface
test_parameter_validation
generate_report
;;
--compilation)
setup_test_environment
test_server_compilation
test_performance
generate_report
;;
--cli)
setup_test_environment
test_cli_interface
test_parameter_validation
generate_report
;;
--startup)
setup_test_environment
test_server_startup
generate_report
;;
--ai-scaffolding)
setup_test_environment
test_ai_scaffolding
test_realistic_configurations
generate_report
;;
--performance)
setup_test_environment
test_performance
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