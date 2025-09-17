#!/bin/bash

# MCP Boilerplate Rust - Quick Setup Script
# This script provides convenient access to all setup and configuration tools

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Project info
PROJECT_NAME="MCP Boilerplate Rust"
PROJECT_DESC="Model Context Protocol implementation with AI image generation"

echo -e "${CYAN}🚀 $PROJECT_NAME - Setup Script${NC}"
echo -e "${CYAN}================================================${NC}"
echo -e "📝 $PROJECT_DESC"
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
        if bash "$script_path"; then
            echo -e "${GREEN}✅ $script_name completed successfully${NC}"
            return 0
        else
            echo -e "${RED}❌ $script_name failed${NC}"
            return 1
        fi
    else
        echo -e "${YELLOW}⚠️  $script_name not found at: $script_path${NC}"
        return 1
    fi
}

# Main menu function
show_menu() {
    echo -e "${CYAN}📋 Available Setup Options:${NC}"
    echo
    echo "  1) 🔑 Setup Google Gemini API Environment"
    echo "  2) 🧪 Run Verification Tests"
    echo "  3) 🚀 Run End-to-End Tests"
    echo "  4) 🖼️  Test Image Generation Server"
    echo "  5) 🎨 Generate Sample Image (Quick Test)"
    echo "  6) 🔨 Build All Components"
    echo "  7) 🧹 Clean Build Artifacts"
    echo "  8) 📚 Show Project Documentation"
    echo "  9) 🏁 Complete Setup (All Steps)"
    echo "  0) ❌ Exit"
    echo
}

# Individual setup functions
setup_gemini() {
    echo -e "${BLUE}🔑 Setting up Google Gemini API Environment...${NC}"
    run_script "scripts/shell/setup/setup_gemini_env.sh" "Gemini Environment Setup"
}

run_verification() {
    echo -e "${BLUE}🧪 Running verification tests...${NC}"
    run_script "scripts/shell/verification/verify_gemini_fix.sh" "Gemini Fix Verification"
}

run_e2e_tests() {
    echo -e "${BLUE}🚀 Running end-to-end tests...${NC}"
    run_script "scripts/shell/testing/run_e2e_tests.sh" "End-to-End Tests"
}

test_image_server() {
    echo -e "${BLUE}🖼️  Testing image generation server...${NC}"
    run_script "scripts/shell/testing/test_image_generation_server.sh" "Image Generation Server Tests"
}

generate_sample_image() {
    echo -e "${BLUE}🎨 Generating sample image...${NC}"
    if [[ -f "generate_image.py" ]]; then
        echo "Generating a test image with prompt: 'A beautiful mountain landscape at sunset'"
        python3 generate_image.py "A beautiful mountain landscape at sunset"
    else
        echo -e "${RED}❌ generate_image.py not found${NC}"
        return 1
    fi
}

build_all() {
    echo -e "${BLUE}🔨 Building all components...${NC}"
    echo "Building workspace..."
    if cargo build --workspace; then
        echo -e "${GREEN}✅ Build completed successfully${NC}"
        
        echo "Building documentation..."
        if cargo doc --workspace --no-deps; then
            echo -e "${GREEN}✅ Documentation built successfully${NC}"
        else
            echo -e "${YELLOW}⚠️  Documentation build had issues${NC}"
        fi
    else
        echo -e "${RED}❌ Build failed${NC}"
        return 1
    fi
}

clean_artifacts() {
    echo -e "${BLUE}🧹 Cleaning build artifacts...${NC}"
    if cargo clean; then
        echo -e "${GREEN}✅ Build artifacts cleaned${NC}"
    else
        echo -e "${RED}❌ Clean failed${NC}"
        return 1
    fi
}

show_documentation() {
    echo -e "${BLUE}📚 Project Documentation${NC}"
    echo "=========================="
    echo
    echo "📁 Key Files:"
    echo "  • README.md - Main project documentation"
    echo "  • API.md - API reference"
    echo "  • PROJECT_STRUCTURE.md - Project organization"
    echo "  • scripts/README.md - Scripts documentation"
    echo
    echo "🎨 Image Generation:"
    echo "  • python3 generate_image.py 'your prompt' - Generate images"
    echo "  • scripts/python/clients/ - Client tools"
    echo "  • generated_images/ - Output directory"
    echo
    echo "🛠️  Development:"
    echo "  • cargo build --workspace - Build all components"
    echo "  • cargo test --workspace - Run all tests"
    echo "  • cargo run --bin image-generation-server -- --help"
    echo
    echo "🔧 Configuration:"
    echo "  • Set GEMINI_API_KEY environment variable"
    echo "  • Use scripts/shell/setup/setup_gemini_env.sh for guided setup"
    echo
}

complete_setup() {
    echo -e "${CYAN}🏁 Running Complete Setup...${NC}"
    echo "This will run all setup steps in sequence"
    echo
    
    local failed_steps=()
    
    # Step 1: Build
    echo -e "${BLUE}Step 1/4: Building project...${NC}"
    if ! build_all; then
        failed_steps+=("Build")
    fi
    echo
    
    # Step 2: Gemini setup
    echo -e "${BLUE}Step 2/4: Setting up Gemini environment...${NC}"
    if ! setup_gemini; then
        failed_steps+=("Gemini Setup")
    fi
    echo
    
    # Step 3: Verification
    echo -e "${BLUE}Step 3/4: Running verification tests...${NC}"
    if ! run_verification; then
        failed_steps+=("Verification")
    fi
    echo
    
    # Step 4: Sample image
    echo -e "${BLUE}Step 4/4: Generating sample image...${NC}"
    if ! generate_sample_image; then
        failed_steps+=("Sample Image")
    fi
    echo
    
    # Summary
    if [[ ${#failed_steps[@]} -eq 0 ]]; then
        echo -e "${GREEN}🎉 Complete setup finished successfully!${NC}"
        echo
        echo -e "${CYAN}🚀 You're ready to use MCP Boilerplate Rust!${NC}"
        echo
        echo "Quick start commands:"
        echo "  python3 generate_image.py 'your prompt'"
        echo "  cargo run --bin image-generation-server"
        echo
    else
        echo -e "${YELLOW}⚠️  Setup completed with some issues:${NC}"
        for step in "${failed_steps[@]}"; do
            echo "  • $step failed"
        done
        echo
        echo "You can retry individual steps from the main menu."
    fi
}

# Handle command line arguments
if [[ $# -gt 0 ]]; then
    case "$1" in
        "gemini"|"setup-gemini")
            setup_gemini
            exit $?
            ;;
        "verify"|"verification")
            run_verification
            exit $?
            ;;
        "test"|"e2e")
            run_e2e_tests
            exit $?
            ;;
        "image"|"image-test")
            test_image_server
            exit $?
            ;;
        "generate"|"sample")
            generate_sample_image
            exit $?
            ;;
        "build")
            build_all
            exit $?
            ;;
        "clean")
            clean_artifacts
            exit $?
            ;;
        "docs"|"documentation")
            show_documentation
            exit $?
            ;;
        "all"|"complete")
            complete_setup
            exit $?
            ;;
        "help"|"--help"|"-h")
            echo "Usage: $0 [command]"
            echo
            echo "Commands:"
            echo "  gemini      - Setup Google Gemini API environment"
            echo "  verify      - Run verification tests"
            echo "  test        - Run end-to-end tests"
            echo "  image       - Test image generation server"
            echo "  generate    - Generate sample image"
            echo "  build       - Build all components"
            echo "  clean       - Clean build artifacts"
            echo "  docs        - Show documentation"
            echo "  all         - Run complete setup"
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
            setup_gemini
            ;;
        2)
            run_verification
            ;;
        3)
            run_e2e_tests
            ;;
        4)
            test_image_server
            ;;
        5)
            generate_sample_image
            ;;
        6)
            build_all
            ;;
        7)
            clean_artifacts
            ;;
        8)
            show_documentation
            ;;
        9)
            complete_setup
            ;;
        0)
            echo -e "${CYAN}👋 Goodbye!${NC}"
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