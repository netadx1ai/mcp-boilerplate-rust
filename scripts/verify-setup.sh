#!/bin/bash

# MCP Boilerplate Rust - Setup Verification Script
# Verifies that the project is set up correctly

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== MCP Boilerplate Rust - Setup Verification ===${NC}\n"

ERRORS=0
WARNINGS=0

# Check Rust installation
echo -n "Checking Rust installation... "
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    echo -e "${GREEN}✓${NC} $RUST_VERSION"
else
    echo -e "${RED}✗ Rust not found${NC}"
    echo "  Install from: https://rustup.rs/"
    ERRORS=$((ERRORS + 1))
fi

# Check Cargo
echo -n "Checking Cargo... "
if command -v cargo &> /dev/null; then
    CARGO_VERSION=$(cargo --version)
    echo -e "${GREEN}✓${NC} $CARGO_VERSION"
else
    echo -e "${RED}✗ Cargo not found${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Check project structure
echo -n "Checking project structure... "
if [ -f "Cargo.toml" ] && [ -d "src" ] && [ -f "src/main.rs" ]; then
    echo -e "${GREEN}✓${NC} Project structure valid"
else
    echo -e "${RED}✗ Invalid project structure${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Check source files
echo -n "Checking source files... "
if [ -f "src/main.rs" ] && [ -f "src/types.rs" ] && [ -f "src/tools/echo.rs" ]; then
    echo -e "${GREEN}✓${NC} All source files present"
else
    echo -e "${RED}✗ Missing source files${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Check .env file
echo -n "Checking .env file... "
if [ -f ".env" ]; then
    echo -e "${GREEN}✓${NC} .env file exists"
else
    echo -e "${YELLOW}⚠${NC} .env file not found"
    if [ -f ".env.example" ]; then
        echo "  Run: cp .env.example .env"
    fi
    WARNINGS=$((WARNINGS + 1))
fi

# Check dependencies
echo -n "Checking dependencies... "
if cargo fetch &> /dev/null; then
    echo -e "${GREEN}✓${NC} Dependencies fetched"
else
    echo -e "${RED}✗ Failed to fetch dependencies${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Try to build
echo -n "Building project... "
if cargo check &> /dev/null; then
    echo -e "${GREEN}✓${NC} Project builds successfully"
else
    echo -e "${RED}✗ Build failed${NC}"
    echo "  Run 'cargo check' for details"
    ERRORS=$((ERRORS + 1))
fi

# Check optional tools
echo -n "Checking curl... "
if command -v curl &> /dev/null; then
    echo -e "${GREEN}✓${NC} curl available"
else
    echo -e "${YELLOW}⚠${NC} curl not found (optional)"
    WARNINGS=$((WARNINGS + 1))
fi

echo -n "Checking jq... "
if command -v jq &> /dev/null; then
    echo -e "${GREEN}✓${NC} jq available"
else
    echo -e "${YELLOW}⚠${NC} jq not found (optional, for test.sh)"
    WARNINGS=$((WARNINGS + 1))
fi

echo -n "Checking make... "
if command -v make &> /dev/null; then
    echo -e "${GREEN}✓${NC} make available"
else
    echo -e "${YELLOW}⚠${NC} make not found (optional)"
    WARNINGS=$((WARNINGS + 1))
fi

echo -n "Checking docker... "
if command -v docker &> /dev/null; then
    echo -e "${GREEN}✓${NC} docker available"
else
    echo -e "${YELLOW}⚠${NC} docker not found (optional)"
    WARNINGS=$((WARNINGS + 1))
fi

# Summary
echo ""
echo -e "${BLUE}=== Summary ===${NC}"
echo ""

if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed!${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Run: cargo run"
    echo "  2. Test: curl http://localhost:8025/health"
    echo "  3. Read: QUICKSTART.md"
    exit 0
elif [ $ERRORS -eq 0 ]; then
    echo -e "${YELLOW}⚠ Setup complete with $WARNINGS warning(s)${NC}"
    echo ""
    echo "You can proceed, but some optional features may not work."
    echo ""
    echo "Next steps:"
    echo "  1. Run: cargo run"
    echo "  2. Test: curl http://localhost:8025/health"
    exit 0
else
    echo -e "${RED}✗ Setup incomplete: $ERRORS error(s), $WARNINGS warning(s)${NC}"
    echo ""
    echo "Please fix the errors above before proceeding."
    exit 1
fi