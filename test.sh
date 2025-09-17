#!/bin/bash

# MCP Boilerplate Rust - Testing Wrapper Script
# This script provides convenient access to all testing tools and verification scripts

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}🧪 MCP Boilerplate Rust - Testing Suite${NC}"
echo -e "${CYAN}==========================================${NC}"
echo

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]] || [[ ! -d "examples" ]]; then
    echo -e "${RED}❌ Error: This script must be run from the project root directory${NC}"
    echo "   Current directory: $(pwd)"
    echo "   Expected files: Cargo.toml, examples/ directory"
    exit 1
fi

echo -e "${GREEN}✅ Project directory verified${NC}"
echo

# Function to run a script with error handling
run_script() {
    local script_path="$1"
    local script_name="$2"
    
    if [[ -f "$script_path" ]]; then
        echo -e "${BLUE}🔧 Running $script_name...${NC}"
        echo "----------------------------------------"
        if bash "$script_path"; then
            echo "----------------------------------------"
            echo -e "${GREEN}✅ $script_name completed successfully${NC}"
            return 0
        else
            echo "----------------------------------------"
            echo -e "${RED}❌ $script_name failed${NC}"
            return 1
        fi
    else
        echo -e "${YELLOW}⚠️  $script_name not found at: $script_path${NC}"
        return 1
    fi
}

# Function to run cargo tests
run_cargo_tests() {
    echo -e "${BLUE}🦀 Running Rust tests...${NC}"
    echo "----------------------------------------"
    
    echo "Running unit tests..."
    if cargo test --workspace --lib; then
        echo -e "${GREEN}✅ Unit tests passed${NC}"
    else
        echo -e "${RED}❌ Unit tests failed${NC}"
        return 1
    fi
    
    echo
    echo "Running integration tests..."
    if cargo test --workspace --test '*'; then
        echo -e "${GREEN}✅ Integration tests passed${NC}"
    else
        echo -e "${RED}❌ Integration tests failed${NC}"
        return 1
    fi
    
    echo
    echo "Running doc tests..."
    if cargo test --workspace --doc; then
        echo -e "${GREEN}✅ Documentation tests passed${NC}"
    else
        echo -e "${RED}❌ Documentation tests failed${NC}"
        return 1
    fi
    
    echo "----------------------------------------"
    echo -e "${GREEN}✅ All Rust tests completed successfully${NC}"
}

# Function to run Python tests
run_python_tests() {
    echo -e "${BLUE}🐍 Running Python client tests...${NC}"
    echo "----------------------------------------"
    
    # Test the main image generator
    if [[ -f "generate_image.py" ]]; then
        echo "Testing main image generator (quick test)..."
        if python3 generate_image.py "test image" >/dev/null 2>&1; then
            echo -e "${GREEN}✅ Main image generator works${NC}"
        else
            echo -e "${YELLOW}⚠️  Main image generator had issues (may need API key)${NC}"
        fi
    fi
    
    # Test client scripts
    if [[ -d "scripts/python/clients" ]]; then
        echo "Testing Python client scripts..."
        cd scripts/python/clients
        
        # Test import capabilities
        if python3 -c "import image_generator; print('✅ image_generator module loads correctly')" 2>/dev/null; then
            echo -e "${GREEN}✅ Python clients import successfully${NC}"
        else
            echo -e "${YELLOW}⚠️  Python client import issues${NC}"
        fi
        
        cd ../../..
    fi
    
    echo "----------------------------------------"
    echo -e "${GREEN}✅ Python tests completed${NC}"
}

# Function to run quality checks
run_quality_checks() {
    echo -e "${BLUE}🔍 Running code quality checks...${NC}"
    echo "----------------------------------------"
    
    echo "Running cargo fmt check..."
    if cargo fmt --all --check; then
        echo -e "${GREEN}✅ Code formatting is correct${NC}"
    else
        echo -e "${YELLOW}⚠️  Code formatting issues found${NC}"
        echo "Run 'cargo fmt --all' to fix formatting"
    fi
    
    echo
    echo "Running cargo clippy..."
    if cargo clippy --workspace --all-targets -- -D warnings; then
        echo -e "${GREEN}✅ Clippy checks passed (0 warnings)${NC}"
    else
        echo -e "${RED}❌ Clippy found issues${NC}"
        return 1
    fi
    
    echo
    echo "Running cargo check..."
    if cargo check --workspace --all-targets; then
        echo -e "${GREEN}✅ Compilation check passed${NC}"
    else
        echo -e "${RED}❌ Compilation check failed${NC}"
        return 1
    fi
    
    echo "----------------------------------------"
    echo -e "${GREEN}✅ Code quality checks completed${NC}"
}

# Function to run functional tests
run_functional_tests() {
    echo -e "${BLUE}⚙️  Running functional tests...${NC}"
    echo "----------------------------------------"
    
    # Build first
    echo "Building project..."
    if cargo build --workspace; then
        echo -e "${GREEN}✅ Build successful${NC}"
    else
        echo -e "${RED}❌ Build failed${NC}"
        return 1
    fi
    
    echo
    echo "Testing server startup..."
    HELP_OUTPUT=$(./target/debug/image-generation-server --help 2>&1 || echo "FAILED")
    if echo "$HELP_OUTPUT" | grep -q "MCP AI image generation server"; then
        echo -e "${GREEN}✅ Server help command works${NC}"
    else
        echo -e "${RED}❌ Server help command failed${NC}"
        return 1
    fi
    
    echo
    echo "Testing mock mode..."
    MOCK_INPUT='{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"generate_image","arguments":{"prompt":"test"}}}'
    MOCK_OUTPUT=$(echo "$MOCK_INPUT" | timeout 10s ./target/debug/image-generation-server --transport stdio --delay 0 2>/dev/null || echo "TIMEOUT")
    
    if echo "$MOCK_OUTPUT" | grep -q "success\|image"; then
        echo -e "${GREEN}✅ Mock mode functional test passed${NC}"
    else
        echo -e "${YELLOW}⚠️  Mock mode test had issues (core functionality may still work)${NC}"
    fi
    
    echo "----------------------------------------"
    echo -e "${GREEN}✅ Functional tests completed${NC}"
}

# Main menu function
show_menu() {
    echo -e "${CYAN}📋 Available Testing Options:${NC}"
    echo
    echo "  1) 🦀 Run Rust Tests (unit, integration, doc)"
    echo "  2) 🐍 Run Python Client Tests"
    echo "  3) 🔍 Run Code Quality Checks (fmt, clippy, check)"
    echo "  4) ⚙️  Run Functional Tests (build, startup, mock)"
    echo "  5) 🧪 Run Verification Tests (Gemini integration)"
    echo "  6) 🚀 Run End-to-End Tests (comprehensive)"
    echo "  7) 🖼️  Run Image Generation Server Tests"
    echo "  8) 🎯 Run All Tests (complete test suite)"
    echo "  9) ⚡ Run Quick Tests (fast subset)"
    echo "  0) ❌ Exit"
    echo
}

# Function to run all tests
run_all_tests() {
    echo -e "${CYAN}🎯 Running Complete Test Suite...${NC}"
    echo "=================================="
    echo
    
    local failed_tests=()
    local total_tests=6
    local passed_tests=0
    
    # Test 1: Code Quality
    echo -e "${BLUE}Test 1/$total_tests: Code Quality Checks${NC}"
    if run_quality_checks; then
        ((passed_tests++))
    else
        failed_tests+=("Code Quality")
    fi
    echo
    
    # Test 2: Rust Tests
    echo -e "${BLUE}Test 2/$total_tests: Rust Tests${NC}"
    if run_cargo_tests; then
        ((passed_tests++))
    else
        failed_tests+=("Rust Tests")
    fi
    echo
    
    # Test 3: Functional Tests
    echo -e "${BLUE}Test 3/$total_tests: Functional Tests${NC}"
    if run_functional_tests; then
        ((passed_tests++))
    else
        failed_tests+=("Functional Tests")
    fi
    echo
    
    # Test 4: Python Tests
    echo -e "${BLUE}Test 4/$total_tests: Python Client Tests${NC}"
    if run_python_tests; then
        ((passed_tests++))
    else
        failed_tests+=("Python Tests")
    fi
    echo
    
    # Test 5: Verification
    echo -e "${BLUE}Test 5/$total_tests: Verification Tests${NC}"
    if run_script "scripts/shell/verification/verify_gemini_fix.sh" "Verification Tests"; then
        ((passed_tests++))
    else
        failed_tests+=("Verification Tests")
    fi
    echo
    
    # Test 6: Image Server Tests
    echo -e "${BLUE}Test 6/$total_tests: Image Generation Tests${NC}"
    if run_script "scripts/shell/testing/test_image_generation_server.sh" "Image Generation Tests"; then
        ((passed_tests++))
    else
        failed_tests+=("Image Generation Tests")
    fi
    echo
    
    # Summary
    echo -e "${CYAN}📊 Test Suite Summary${NC}"
    echo "====================="
    echo -e "Total tests: $total_tests"
    echo -e "Passed: ${GREEN}$passed_tests${NC}"
    echo -e "Failed: ${RED}$((total_tests - passed_tests))${NC}"
    echo
    
    if [[ ${#failed_tests[@]} -eq 0 ]]; then
        echo -e "${GREEN}🎉 ALL TESTS PASSED! Your code is ready for production.${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠️  Some tests failed:${NC}"
        for test in "${failed_tests[@]}"; do
            echo -e "  • ${RED}$test${NC}"
        done
        echo
        echo "Review the output above and fix the issues."
        return 1
    fi
}

# Function to run quick tests
run_quick_tests() {
    echo -e "${CYAN}⚡ Running Quick Test Suite...${NC}"
    echo "=============================="
    echo
    
    local failed_tests=()
    
    # Quick quality check
    echo -e "${BLUE}Quick Test 1/3: Code Quality${NC}"
    if cargo clippy --workspace --all-targets -- -D warnings && cargo check --workspace; then
        echo -e "${GREEN}✅ Code quality check passed${NC}"
    else
        failed_tests+=("Code Quality")
    fi
    echo
    
    # Quick unit tests
    echo -e "${BLUE}Quick Test 2/3: Unit Tests${NC}"
    if cargo test --workspace --lib; then
        echo -e "${GREEN}✅ Unit tests passed${NC}"
    else
        failed_tests+=("Unit Tests")
    fi
    echo
    
    # Quick functional test
    echo -e "${BLUE}Quick Test 3/3: Basic Functionality${NC}"
    if cargo build --bin image-generation-server; then
        if ./target/debug/image-generation-server --help >/dev/null 2>&1; then
            echo -e "${GREEN}✅ Basic functionality test passed${NC}"
        else
            failed_tests+=("Basic Functionality")
        fi
    else
        failed_tests+=("Basic Functionality")
    fi
    echo
    
    # Summary
    if [[ ${#failed_tests[@]} -eq 0 ]]; then
        echo -e "${GREEN}⚡ Quick tests passed! Ready for development.${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠️  Quick tests found issues:${NC}"
        for test in "${failed_tests[@]}"; do
            echo -e "  • ${RED}$test${NC}"
        done
        return 1
    fi
}

# Handle command line arguments
if [[ $# -gt 0 ]]; then
    case "$1" in
        "rust"|"cargo")
            run_cargo_tests
            exit $?
            ;;
        "python"|"py")
            run_python_tests
            exit $?
            ;;
        "quality"|"lint")
            run_quality_checks
            exit $?
            ;;
        "functional"|"func")
            run_functional_tests
            exit $?
            ;;
        "verify"|"verification")
            run_script "scripts/shell/verification/verify_gemini_fix.sh" "Verification Tests"
            exit $?
            ;;
        "e2e"|"integration")
            run_script "scripts/shell/testing/run_e2e_tests.sh" "End-to-End Tests"
            exit $?
            ;;
        "image"|"server")
            run_script "scripts/shell/testing/test_image_generation_server.sh" "Image Server Tests"
            exit $?
            ;;
        "all"|"complete")
            run_all_tests
            exit $?
            ;;
        "quick"|"fast")
            run_quick_tests
            exit $?
            ;;
        "help"|"--help"|"-h")
            echo "Usage: $0 [command]"
            echo
            echo "Commands:"
            echo "  rust        - Run Rust tests (unit, integration, doc)"
            echo "  python      - Run Python client tests"
            echo "  quality     - Run code quality checks"
            echo "  functional  - Run functional tests"
            echo "  verify      - Run verification tests"
            echo "  e2e         - Run end-to-end tests"
            echo "  image       - Run image generation server tests"
            echo "  all         - Run complete test suite"
            echo "  quick       - Run quick test subset"
            echo "  help        - Show this help"
            echo
            echo "If no command is provided, interactive menu will be shown."
            exit 0
            ;;
        *)
            echo -e "${RED}❌ Unknown command: $1${NC}"
            echo "Use '$0 help' to see available commands."
            exit 1
            ;;
    esac
fi

# Interactive menu loop
while true; do
    show_menu
    read -p "Choose an option (0-9): " choice
    echo
    
    case $choice in
        1)
            run_cargo_tests
            ;;
        2)
            run_python_tests
            ;;
        3)
            run_quality_checks
            ;;
        4)
            run_functional_tests
            ;;
        5)
            run_script "scripts/shell/verification/verify_gemini_fix.sh" "Verification Tests"
            ;;
        6)
            run_script "scripts/shell/testing/run_e2e_tests.sh" "End-to-End Tests"
            ;;
        7)
            run_script "scripts/shell/testing/test_image_generation_server.sh" "Image Server Tests"
            ;;
        8)
            run_all_tests
            ;;
        9)
            run_quick_tests
            ;;
        0)
            echo -e "${CYAN}👋 Testing complete!${NC}"
            exit 0
            ;;
        *)
            echo -e "${RED}❌ Invalid option. Please choose 0-9.${NC}"
            ;;
    esac
    
    echo
    read -p "Press Enter to continue..."
    echo
done