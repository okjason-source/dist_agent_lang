#!/bin/bash
# DAL Test Runner - Layer 3 Testing
# Runs *.test.dal files using the dist_agent_lang CLI

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧪 DAL Test Runner - Layer 3${NC}\n"

# Find all .test.dal files
TEST_FILES=$(find examples -name "*.test.dal" 2>/dev/null || echo "")

if [ -z "$TEST_FILES" ]; then
    echo -e "${YELLOW}⚠️  No .test.dal files found${NC}"
    exit 0
fi

# Count test files
TOTAL_FILES=$(echo "$TEST_FILES" | wc -l | tr -d ' ')
PASSED=0
FAILED=0

echo -e "${BLUE}Found $TOTAL_FILES test file(s)${NC}\n"

# Run each test file
for test_file in $TEST_FILES; do
    echo -e "${BLUE}──────────────────────────────────────${NC}"
    echo -e "${BLUE}Running: $test_file${NC}"
    echo -e "${BLUE}──────────────────────────────────────${NC}"
    
    # Try to run the test file
    if cargo run --release --quiet -- run "$test_file" 2>&1; then
        echo -e "${GREEN}✓ PASSED${NC}\n"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}✗ FAILED${NC}\n"
        FAILED=$((FAILED + 1))
    fi
done

# Summary
echo -e "${BLUE}══════════════════════════════════════${NC}"
echo -e "${BLUE}Test Summary${NC}"
echo -e "${BLUE}══════════════════════════════════════${NC}"
echo -e "Total test files: $TOTAL_FILES"
echo -e "${GREEN}Passed: $PASSED${NC}"
if [ $FAILED -gt 0 ]; then
    echo -e "${RED}Failed: $FAILED${NC}"
else
    echo -e "Failed: $FAILED"
fi
echo -e "${BLUE}══════════════════════════════════════${NC}\n"

# Exit with appropriate code
if [ $FAILED -gt 0 ]; then
    exit 1
else
    echo -e "${GREEN}🎉 All tests passed!${NC}"
    exit 0
fi
