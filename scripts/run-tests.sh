#!/bin/bash
# Comprehensive test runner for dist_agent_lang

set -e

echo "🧪 Running Comprehensive Test Suite for dist_agent_lang"
echo "======================================================"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run tests and count results
run_test_suite() {
    local suite_name=$1
    local test_command=$2
    
    echo -e "${YELLOW}Running $suite_name...${NC}"
    
    if eval "$test_command" 2>&1 | tee /tmp/test_output.log; then
        local passed=$(grep -c "test result: ok" /tmp/test_output.log || echo "0")
        local failed=$(grep -c "test result: FAILED" /tmp/test_output.log || echo "0")
        
        if [ "$failed" -eq 0 ] && [ "$passed" -gt 0 ]; then
            echo -e "${GREEN}✅ $suite_name: PASSED${NC}"
            PASSED_TESTS=$((PASSED_TESTS + passed))
        else
            echo -e "${RED}❌ $suite_name: FAILED${NC}"
            FAILED_TESTS=$((FAILED_TESTS + failed))
        fi
        TOTAL_TESTS=$((TOTAL_TESTS + passed + failed))
    else
        echo -e "${RED}❌ $suite_name: ERROR${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    echo ""
}

# 1. Library Tests
run_test_suite "Library Tests" "cargo test --lib --all-features"

# 2. Integration Tests
if [ -d "tests/integration" ]; then
    run_test_suite "Integration Tests" "cargo test --test '*' --all-features"
fi

# 3. Security Audit
echo -e "${YELLOW}Running Security Audit...${NC}"
if cargo audit 2>&1 | tee /tmp/audit_output.log; then
    echo -e "${GREEN}✅ Security Audit: PASSED${NC}"
else
    echo -e "${YELLOW}⚠️  Security Audit: Warnings (check output)${NC}"
fi
echo ""

# 4. Code Quality Check
echo -e "${YELLOW}Running Clippy (Code Quality)...${NC}"
if cargo clippy --all-features -- -D warnings 2>&1 | tee /tmp/clippy_output.log; then
    echo -e "${GREEN}✅ Clippy: PASSED${NC}"
else
    local warnings=$(grep -c "warning:" /tmp/clippy_output.log || echo "0")
    echo -e "${YELLOW}⚠️  Clippy: $warnings warnings (non-blocking)${NC}"
fi
echo ""

# 5. Compilation Check
echo -e "${YELLOW}Checking Compilation...${NC}"
if cargo build --release --all-features 2>&1 | tee /tmp/build_output.log; then
    echo -e "${GREEN}✅ Compilation: SUCCESS${NC}"
else
    echo -e "${RED}❌ Compilation: FAILED${NC}"
    exit 1
fi
echo ""

# Summary
echo "======================================================"
echo -e "${YELLOW}Test Summary:${NC}"
echo "  Total Tests: $TOTAL_TESTS"
echo -e "  ${GREEN}Passed: $PASSED_TESTS${NC}"
if [ $FAILED_TESTS -gt 0 ]; then
    echo -e "  ${RED}Failed: $FAILED_TESTS${NC}"
else
    echo -e "  ${GREEN}Failed: 0${NC}"
fi
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed${NC}"
    exit 1
fi

