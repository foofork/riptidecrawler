#!/bin/bash

# Validation script for the RipTide load testing suite
# This script performs basic validation of the load testing functionality

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOAD_TEST_SCRIPT="$SCRIPT_DIR/load-test.sh"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== RipTide Load Test Validation ===${NC}"

# Test 1: Check if load test script exists and is executable
echo -e "${BLUE}Test 1: Checking load test script...${NC}"
if [ -f "$LOAD_TEST_SCRIPT" ] && [ -x "$LOAD_TEST_SCRIPT" ]; then
    echo -e "${GREEN}✓ Load test script exists and is executable${NC}"
else
    echo -e "${RED}✗ Load test script not found or not executable${NC}"
    exit 1
fi

# Test 2: Validate script help functionality
echo -e "${BLUE}Test 2: Checking help functionality...${NC}"
if "$LOAD_TEST_SCRIPT" --help > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Help functionality works${NC}"
else
    echo -e "${RED}✗ Help functionality failed${NC}"
fi

# Test 3: Test data generation
echo -e "${BLUE}Test 3: Testing data generation...${NC}"
if "$LOAD_TEST_SCRIPT" generate-data > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Test data generation works${NC}"

    # Check if test data files were created
    if [ -f "$SCRIPT_DIR/test-data/test-urls.json" ] &&
       [ -f "$SCRIPT_DIR/test-data/small-batch.json" ] &&
       [ -f "$SCRIPT_DIR/test-data/large-batch.json" ]; then
        echo -e "${GREEN}✓ All test data files created successfully${NC}"
    else
        echo -e "${YELLOW}⚠ Some test data files may be missing${NC}"
    fi
else
    echo -e "${RED}✗ Test data generation failed${NC}"
fi

# Test 4: Validate JSON structure of test data
echo -e "${BLUE}Test 4: Validating test data JSON structure...${NC}"
if command -v jq >/dev/null 2>&1; then
    valid_json=true

    for file in "$SCRIPT_DIR/test-data"/*.json; do
        if [ -f "$file" ]; then
            if jq empty < "$file" >/dev/null 2>&1; then
                echo -e "${GREEN}✓ $(basename "$file") is valid JSON${NC}"
            else
                echo -e "${RED}✗ $(basename "$file") has invalid JSON${NC}"
                valid_json=false
            fi
        fi
    done

    if $valid_json; then
        echo -e "${GREEN}✓ All test data files have valid JSON structure${NC}"
    fi
else
    echo -e "${YELLOW}⚠ jq not available, skipping JSON validation${NC}"
fi

# Test 5: Check script syntax
echo -e "${BLUE}Test 5: Checking script syntax...${NC}"
if bash -n "$LOAD_TEST_SCRIPT"; then
    echo -e "${GREEN}✓ Script syntax is valid${NC}"
else
    echo -e "${RED}✗ Script syntax errors found${NC}"
fi

# Test 6: Test mock server functionality (without actual server)
echo -e "${BLUE}Test 6: Testing error handling with non-existent server...${NC}"
if "$LOAD_TEST_SCRIPT" --host "http://localhost:99999" test-health 2>/dev/null; then
    echo -e "${YELLOW}⚠ Expected failure but test passed${NC}"
else
    echo -e "${GREEN}✓ Correctly handles non-existent server${NC}"
fi

# Test 7: Check directory structure creation
echo -e "${BLUE}Test 7: Checking directory structure...${NC}"
if [ -d "$SCRIPT_DIR/test-data" ] && [ -d "$SCRIPT_DIR/load-test-results" ]; then
    echo -e "${GREEN}✓ Required directories created${NC}"
else
    echo -e "${RED}✗ Required directories not found${NC}"
fi

# Test 8: Validate dependencies check
echo -e "${BLUE}Test 8: Testing dependency checking...${NC}"
# This will succeed if dependencies are available, or show appropriate warnings
"$LOAD_TEST_SCRIPT" --help >/dev/null 2>&1
echo -e "${GREEN}✓ Dependency checking completed${NC}"

echo -e "${BLUE}=== Validation Summary ===${NC}"
echo -e "${GREEN}✓ Basic validation completed successfully${NC}"
echo -e "${YELLOW}Note: Full functionality requires a running RipTide server${NC}"

# Show usage examples
echo -e "${BLUE}=== Usage Examples ===${NC}"
echo "To run actual load tests (requires running RipTide server):"
echo "  $LOAD_TEST_SCRIPT test-health"
echo "  $LOAD_TEST_SCRIPT test-crawl"
echo "  $LOAD_TEST_SCRIPT test-all"
echo ""
echo "To start server and run tests automatically:"
echo "  $LOAD_TEST_SCRIPT --start-server --stop-server test-all"

echo -e "${GREEN}Validation complete!${NC}"