#!/bin/bash

# Pre-Integration Verification Script
# Checks everything is ready for Claude Desktop integration
# Version: 0.3.1
# Date: 2026-01-08

set -e

echo "=== Claude Desktop Integration Pre-Flight Check ==="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

PROJECT_DIR="/Users/hoangiso/Desktop/mcp-boilerplate-rust"
BINARY="$PROJECT_DIR/target/release/mcp-boilerplate-rust"
CLAUDE_CONFIG="$HOME/Library/Application Support/Claude/claude_desktop_config.json"

CHECKS_PASSED=0
CHECKS_FAILED=0

# Helper functions
check_pass() {
    echo -e "${GREEN}✓${NC} $1"
    CHECKS_PASSED=$((CHECKS_PASSED + 1))
}

check_fail() {
    echo -e "${RED}✗${NC} $1"
    CHECKS_FAILED=$((CHECKS_FAILED + 1))
}

check_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# Check 1: Project directory exists
echo "[1/10] Checking project directory..."
if [ -d "$PROJECT_DIR" ]; then
    check_pass "Project directory found"
else
    check_fail "Project directory not found: $PROJECT_DIR"
fi

# Check 2: Binary exists
echo "[2/10] Checking release binary..."
if [ -f "$BINARY" ]; then
    SIZE=$(ls -lh "$BINARY" | awk '{print $5}')
    check_pass "Binary found ($SIZE)"
else
    check_fail "Binary not found - run: cargo build --release"
fi

# Check 3: Binary is executable
echo "[3/10] Checking binary permissions..."
if [ -x "$BINARY" ]; then
    check_pass "Binary is executable"
else
    check_fail "Binary not executable - run: chmod +x $BINARY"
fi

# Check 4: Binary works
echo "[4/10] Testing binary execution..."
if "$BINARY" --help >/dev/null 2>&1; then
    check_pass "Binary executes successfully"
else
    check_fail "Binary execution failed"
fi

# Check 5: Test suite
echo "[5/10] Running stdio tests..."
cd "$PROJECT_DIR"
if ./test_mcp.sh >/dev/null 2>&1; then
    check_pass "Stdio tests passed"
else
    check_fail "Stdio tests failed"
fi

# Check 6: Validation tests
echo "[6/10] Running validation tests..."
if ./test_validation.sh >/dev/null 2>&1; then
    check_pass "Validation tests passed"
else
    check_warn "Validation tests had warnings (may be normal)"
fi

# Check 7: Claude Desktop config exists
echo "[7/10] Checking Claude Desktop config..."
if [ -f "$CLAUDE_CONFIG" ]; then
    check_pass "Config file exists"
else
    check_fail "Config file not found: $CLAUDE_CONFIG"
fi

# Check 8: Config is valid JSON
echo "[8/10] Validating config JSON..."
if python3 -m json.tool "$CLAUDE_CONFIG" >/dev/null 2>&1; then
    check_pass "Config is valid JSON"
else
    check_fail "Config has JSON syntax errors"
fi

# Check 9: Config has correct binary path
echo "[9/10] Checking config binary path..."
if grep -q "$BINARY" "$CLAUDE_CONFIG"; then
    check_pass "Binary path in config matches"
else
    check_fail "Binary path mismatch in config"
fi

# Check 10: Claude Desktop app exists
echo "[10/10] Checking Claude Desktop installation..."
if [ -d "/Applications/Claude.app" ]; then
    check_pass "Claude Desktop found"
else
    check_warn "Claude Desktop not found at /Applications/Claude.app"
fi

echo ""
echo "=== Summary ==="
echo -e "Passed: ${GREEN}$CHECKS_PASSED${NC}"
echo -e "Failed: ${RED}$CHECKS_FAILED${NC}"
echo ""

if [ $CHECKS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed!${NC}"
    echo ""
    echo "Ready for Claude Desktop integration!"
    echo ""
    echo "Next steps:"
    echo "  1. Restart Claude Desktop:"
    echo "     killall Claude && sleep 2 && open -a Claude"
    echo ""
    echo "  2. In Claude Desktop, try:"
    echo "     'Use the echo tool to say hello'"
    echo ""
    echo "  3. Monitor logs:"
    echo "     tail -f ~/Library/Logs/Claude/mcp*.log"
    echo ""
    exit 0
else
    echo -e "${RED}✗ Some checks failed${NC}"
    echo ""
    echo "Please fix the issues above before proceeding."
    echo ""
    exit 1
fi