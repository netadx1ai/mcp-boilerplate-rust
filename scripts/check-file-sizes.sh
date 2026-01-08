#!/bin/bash

# Check File Sizes - Enforce 500 line limit per file
# Run this before commits to ensure code organization standards

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

MAX_LINES=500
WARN_LINES=400

echo -e "${GREEN}Checking file sizes...${NC}\n"

VIOLATIONS=0
WARNINGS=0

# Check all Rust source files
while IFS= read -r file; do
    lines=$(wc -l < "$file")
    
    if [ "$lines" -gt "$MAX_LINES" ]; then
        echo -e "${RED}✗ VIOLATION: $file has $lines lines (max: $MAX_LINES)${NC}"
        VIOLATIONS=$((VIOLATIONS + 1))
    elif [ "$lines" -gt "$WARN_LINES" ]; then
        echo -e "${YELLOW}⚠ WARNING: $file has $lines lines (approaching limit of $MAX_LINES)${NC}"
        WARNINGS=$((WARNINGS + 1))
    fi
done < <(find src -name "*.rs" -type f)

echo ""

if [ "$VIOLATIONS" -gt 0 ]; then
    echo -e "${RED}Found $VIOLATIONS file(s) exceeding $MAX_LINES lines!${NC}"
    echo -e "${RED}Please split large files into modules.${NC}"
    echo -e "See docs/CODE_ORGANIZATION.md for guidance."
    exit 1
elif [ "$WARNINGS" -gt 0 ]; then
    echo -e "${YELLOW}Found $WARNINGS file(s) approaching size limit.${NC}"
    echo -e "Consider splitting files over $WARN_LINES lines."
    echo -e "${GREEN}Check passed with warnings.${NC}"
    exit 0
else
    echo -e "${GREEN}✓ All files under size limits!${NC}"
    exit 0
fi