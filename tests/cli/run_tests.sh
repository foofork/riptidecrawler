#!/bin/bash
# RipTide CLI-API Integration Test Runner
set -e

echo "🧪 RipTide CLI-API Integration Test Suite"
echo "=========================================="
echo ""

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Track results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

run_test_suite() {
    local suite_name=$1
    local test_pattern=$2

    echo -e "${YELLOW}Running: $suite_name${NC}"
    echo "----------------------------------------"

    if cargo test --test '*' "$test_pattern" -- --nocapture 2>&1 | tee /tmp/test_output.log; then
        local count=$(grep -c "test result: ok" /tmp/test_output.log || echo "0")
        PASSED_TESTS=$((PASSED_TESTS + count))
        echo -e "${GREEN}✓ $suite_name passed${NC}"
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
        echo -e "${RED}✗ $suite_name failed${NC}"
    fi
    echo ""
}

# Run individual test suites
echo "1. API Client Unit Tests"
run_test_suite "API Client Tests" "cli::api_client_tests"

echo "2. Fallback Logic Tests"
run_test_suite "Fallback Tests" "cli::fallback_tests"

echo "3. Integration Tests"
run_test_suite "Integration API Tests" "cli::integration_api_tests"

echo "4. Test Utilities"
run_test_suite "Test Utilities" "cli::test_utils::tests"

# Summary
echo ""
echo "=========================================="
echo "📊 Test Summary"
echo "=========================================="
echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed: ${RED}$FAILED_TESTS${NC}"
TOTAL_TESTS=$((PASSED_TESTS + FAILED_TESTS))
echo "Total: $TOTAL_TESTS"

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
