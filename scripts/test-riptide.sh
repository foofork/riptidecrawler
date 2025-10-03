#!/bin/bash

# RipTide API Test Script
# This script runs basic API tests to verify the installation

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

API_URL="${RIPTIDE_API_URL:-http://localhost:8080}"

echo -e "${BLUE}🧪 RipTide API Test Suite${NC}"
echo "Testing API at: $API_URL"
echo "================================"
echo ""

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run test
run_test() {
    local test_name="$1"
    local command="$2"
    local expected_status="${3:-200}"

    echo -ne "Testing ${test_name}... "

    response=$(eval "$command" 2>&1)
    status=$?

    if [ $status -eq 0 ]; then
        echo -e "${GREEN}✅ PASS${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        if [ -n "$response" ]; then
            echo "$response" | jq '.' 2>/dev/null || echo "$response"
        fi
    else
        echo -e "${RED}❌ FAIL${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        echo -e "${RED}Error: $response${NC}"
    fi
    echo ""
}

# Test 1: Health Check
run_test "Health Check" \
    "curl -sf $API_URL/healthz"

# Test 2: Metrics Endpoint
run_test "Prometheus Metrics" \
    "curl -sf $API_URL/metrics | head -n 5"

# Test 3: Basic Crawl
run_test "Basic Crawl (example.com)" \
    "curl -sf -X POST $API_URL/crawl \
        -H 'Content-Type: application/json' \
        -d '{\"urls\":[\"https://example.com\"],\"options\":{\"concurrency\":1}}'"

# Test 4: Session List
run_test "List Sessions" \
    "curl -sf $API_URL/sessions"

# Test 5: Worker Status
run_test "Worker Status" \
    "curl -sf $API_URL/workers/status"

# Test 6: Monitoring Health Score
run_test "Health Score" \
    "curl -sf $API_URL/monitoring/health-score"

# Test 7: Pipeline Phases
run_test "Pipeline Phases" \
    "curl -sf $API_URL/pipeline/phases"

# Test 8: Strategies Info
run_test "Extraction Strategies" \
    "curl -sf $API_URL/strategies/info"

echo "================================"
echo -e "${BLUE}📊 Test Results${NC}"
echo ""
echo -e "  ${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "  ${RED}Failed: $TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed!${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠️  Some tests failed. Check the output above.${NC}"
    exit 1
fi
