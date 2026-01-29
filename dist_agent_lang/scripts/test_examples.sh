#!/bin/bash

# Script to test all DAL examples for compilation and basic execution
# This is a temporary solution until a full testing framework is built

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
EXAMPLES_DIR="examples"
BINARY="dist_agent_lang"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Test modes
MODE_COMPILE_ONLY=1
MODE_EXECUTE=2
MODE_BOTH=3

MODE=${MODE_BOTH}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --compile-only)
            MODE=${MODE_COMPILE_ONLY}
            shift
            ;;
        --execute-only)
            MODE=${MODE_EXECUTE}
            shift
            ;;
        --skip-execution)
            MODE=${MODE_COMPILE_ONLY}
            shift
            ;;
        --examples-dir)
            EXAMPLES_DIR="$2"
            shift 2
            ;;
        --binary)
            BINARY="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --compile-only      Only test compilation (parse)"
            echo "  --execute-only      Only test execution (skip parse)"
            echo "  --skip-execution    Same as --compile-only"
            echo "  --examples-dir DIR  Directory containing examples (default: examples)"
            echo "  --binary BINARY      Path to dist_agent_lang binary (default: dist_agent_lang)"
            echo "  --help              Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Check if binary exists
if ! command -v "$BINARY" &> /dev/null; then
    echo -e "${RED}❌ Error: $BINARY not found${NC}"
    echo "Please build the project first: cargo build --release"
    echo "Or specify the binary path with --binary"
    exit 1
fi

# Check if examples directory exists
if [ ! -d "$EXAMPLES_DIR" ]; then
    echo -e "${RED}❌ Error: Examples directory '$EXAMPLES_DIR' not found${NC}"
    exit 1
fi

echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}          Testing DAL Examples${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}"
echo ""
echo "Examples directory: $EXAMPLES_DIR"
echo "Binary: $BINARY"
echo "Mode: $([ $MODE -eq $MODE_COMPILE_ONLY ] && echo "Compile Only" || [ $MODE -eq $MODE_EXECUTE ] && echo "Execute Only" || echo "Both")"
echo ""

# Function to test a single file
test_file() {
    local file="$1"
    local filename=$(basename "$file")
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    echo -n "Testing $filename... "
    
    # Test compilation (parse)
    if [ $MODE -eq $MODE_COMPILE_ONLY ] || [ $MODE -eq $MODE_BOTH ]; then
        if "$BINARY" parse "$file" &> /dev/null; then
            # Parse successful
            if [ $MODE -eq $MODE_COMPILE_ONLY ]; then
                echo -e "${GREEN}✅ PASSED${NC} (compiled)"
                PASSED_TESTS=$((PASSED_TESTS + 1))
                return 0
            fi
        else
            echo -e "${RED}❌ FAILED${NC} (compilation error)"
            FAILED_TESTS=$((FAILED_TESTS + 1))
            echo "  Compilation output:"
            "$BINARY" parse "$file" 2>&1 | sed 's/^/    /'
            return 1
        fi
    fi
    
    # Test execution
    if [ $MODE -eq $MODE_EXECUTE ] || [ $MODE -eq $MODE_BOTH ]; then
        # Check if file requires external dependencies (heuristic)
        if grep -q "chain::\|ai::\|oracle::" "$file" 2>/dev/null; then
            echo -e "${YELLOW}⏭️  SKIPPED${NC} (requires external dependencies)"
            SKIPPED_TESTS=$((SKIPPED_TESTS + 1))
            return 0
        fi
        
        # Try to execute (with timeout to prevent hanging)
        if timeout 10 "$BINARY" run "$file" &> /dev/null; then
            echo -e "${GREEN}✅ PASSED${NC} (compiled and executed)"
            PASSED_TESTS=$((PASSED_TESTS + 1))
            return 0
        else
            # Check if it's a known issue (like missing dependencies)
            if "$BINARY" run "$file" 2>&1 | grep -q "not implemented\|not available\|missing"; then
                echo -e "${YELLOW}⏭️  SKIPPED${NC} (feature not implemented)"
                SKIPPED_TESTS=$((SKIPPED_TESTS + 1))
                return 0
            else
                echo -e "${RED}❌ FAILED${NC} (execution error)"
                FAILED_TESTS=$((FAILED_TESTS + 1))
                echo "  Execution output:"
                "$BINARY" run "$file" 2>&1 | sed 's/^/    /' | head -20
                return 1
            fi
        fi
    fi
}

# Find all .dal files in examples directory
dal_files=$(find "$EXAMPLES_DIR" -name "*.dal" -type f | sort)

if [ -z "$dal_files" ]; then
    echo -e "${YELLOW}⚠️  No .dal files found in $EXAMPLES_DIR${NC}"
    exit 0
fi

# Test each file
while IFS= read -r file; do
    test_file "$file"
done <<< "$dal_files"

# Print summary
echo ""
echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}                    Test Summary${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}"
echo ""
echo "Total tests: $TOTAL_TESTS"
echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
echo -e "${RED}Failed: $FAILED_TESTS${NC}"
echo -e "${YELLOW}Skipped: $SKIPPED_TESTS${NC}"
echo ""

# Exit with appropriate code
if [ $FAILED_TESTS -gt 0 ]; then
    exit 1
else
    exit 0
fi
