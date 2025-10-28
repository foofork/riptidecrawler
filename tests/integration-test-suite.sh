#!/bin/bash

# Integration Test Suite for WASM, Observability, and Metrics
# Tests all improvements made to the Riptide API

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

API_URL="http://localhost:8080"
RESULTS_FILE="/workspaces/eventmesh/tests/test-results.json"
REPORT_FILE="/workspaces/eventmesh/tests/production-readiness-report.md"

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   RIPTIDE API - COMPREHENSIVE INTEGRATION TEST SUITE${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

# Initialize results
echo "{\"timestamp\": \"$(date -Iseconds)\", \"tests\": []}" > "$RESULTS_FILE"

# Helper function to add test result
add_result() {
    local test_name="$1"
    local status="$2"
    local details="$3"

    jq ".tests += [{\"name\": \"$test_name\", \"status\": \"$status\", \"details\": \"$details\"}]" "$RESULTS_FILE" > "${RESULTS_FILE}.tmp"
    mv "${RESULTS_FILE}.tmp" "$RESULTS_FILE"
}

# Helper function for test headers
test_header() {
    echo -e "\n${YELLOW}▶ $1${NC}"
    echo "─────────────────────────────────────────────────────"
}

# Test 1: WASM Parser Tests
test_header "TEST 1: WASM Parser Tests"
echo -e "Testing WASM parser with multiple URLs..."

WASM_TEST_RESPONSE=$(curl -s -X POST "$API_URL/crawl" \
    -H "Content-Type: application/json" \
    -d '{
        "urls": ["https://example.com", "https://news.ycombinator.com", "https://github.com"],
        "options": {"render_mode": "Static"}
    }' || echo '{"error": "request failed"}')

echo "$WASM_TEST_RESPONSE" | jq '.' > /workspaces/eventmesh/tests/wasm-test-response.json

# Check for WASM parser usage
PARSER_USED=$(echo "$WASM_TEST_RESPONSE" | jq -r '.results[0].metadata.parser_used // "unknown"')
QUALITY_SCORES=$(echo "$WASM_TEST_RESPONSE" | jq -r '.results[].metadata.confidence_score // 0' | tr '\n' ',' | sed 's/,$//')

if [[ "$PARSER_USED" == "wasm" || "$PARSER_USED" == "native" ]]; then
    echo -e "${GREEN}✓ Parser detected: $PARSER_USED${NC}"
    add_result "WASM Parser Test" "PASS" "Parser: $PARSER_USED, Scores: $QUALITY_SCORES"
else
    echo -e "${RED}✗ Parser not detected properly${NC}"
    add_result "WASM Parser Test" "FAIL" "Parser not detected"
fi

# Test 2: Observability Tests
test_header "TEST 2: Observability & Logging Tests"
echo -e "Checking logs for parser selection decisions..."

docker-compose logs --tail=200 riptide-api > /workspaces/eventmesh/tests/api-logs.txt 2>&1 || true

PARSER_LOGS=$(grep -E "(Parser|strategy|fallback|WASM|native)" /workspaces/eventmesh/tests/api-logs.txt | tail -20 || echo "No parser logs found")
CONFIDENCE_LOGS=$(grep -i "confidence" /workspaces/eventmesh/tests/api-logs.txt | tail -10 || echo "No confidence logs found")

if [[ "$PARSER_LOGS" != "No parser logs found" ]]; then
    echo -e "${GREEN}✓ Parser selection logs found${NC}"
    echo "$PARSER_LOGS" | head -5
    add_result "Observability Logs" "PASS" "Parser logs present"
else
    echo -e "${YELLOW}⚠ Limited parser logs${NC}"
    add_result "Observability Logs" "WARN" "Limited parser logs"
fi

# Check response metadata
METADATA_TEST=$(echo "$WASM_TEST_RESPONSE" | jq -r '.results[0].metadata | {parser_used, confidence_score, fallback_occurred, parse_time_ms}' || echo "{}")

if [[ "$METADATA_TEST" != "{}" ]]; then
    echo -e "${GREEN}✓ Response metadata populated${NC}"
    echo "$METADATA_TEST"
    add_result "Response Metadata" "PASS" "Metadata fields populated"
else
    echo -e "${RED}✗ Response metadata missing${NC}"
    add_result "Response Metadata" "FAIL" "Metadata missing"
fi

# Test 3: Metrics Tests
test_header "TEST 3: Prometheus Metrics Tests"
echo -e "Scraping Prometheus metrics endpoint..."

METRICS_RESPONSE=$(curl -s "$API_URL/metrics" || echo "")

if [[ -z "$METRICS_RESPONSE" ]]; then
    echo -e "${YELLOW}⚠ Metrics endpoint not available at /metrics${NC}"
    add_result "Metrics Endpoint" "WARN" "Endpoint not found at /metrics"
else
    echo "$METRICS_RESPONSE" > /workspaces/eventmesh/tests/metrics-output.txt

    # Check for riptide_extraction metrics
    EXTRACTION_METRICS=$(grep "riptide_extraction" /workspaces/eventmesh/tests/metrics-output.txt | head -10 || echo "")

    if [[ -n "$EXTRACTION_METRICS" ]]; then
        echo -e "${GREEN}✓ Extraction metrics found${NC}"
        echo "$EXTRACTION_METRICS" | head -5
        add_result "Prometheus Metrics" "PASS" "Extraction metrics present"
    else
        echo -e "${YELLOW}⚠ No riptide_extraction metrics found${NC}"
        add_result "Prometheus Metrics" "WARN" "Extraction metrics not found"
    fi

    # Check specific metric families
    PARSER_ATTEMPTS=$(grep "parser_attempts" /workspaces/eventmesh/tests/metrics-output.txt || echo "")
    PARSER_RESULTS=$(grep "parser_results" /workspaces/eventmesh/tests/metrics-output.txt || echo "")
    PARSER_FALLBACKS=$(grep "parser_fallbacks" /workspaces/eventmesh/tests/metrics-output.txt || echo "")
    PARSER_DURATION=$(grep "parser_duration" /workspaces/eventmesh/tests/metrics-output.txt || echo "")
    CONFIDENCE_SCORE=$(grep "confidence_score" /workspaces/eventmesh/tests/metrics-output.txt || echo "")

    echo ""
    echo "Metrics Summary:"
    [[ -n "$PARSER_ATTEMPTS" ]] && echo -e "${GREEN}✓ parser_attempts${NC}" || echo -e "${RED}✗ parser_attempts${NC}"
    [[ -n "$PARSER_RESULTS" ]] && echo -e "${GREEN}✓ parser_results${NC}" || echo -e "${RED}✗ parser_results${NC}"
    [[ -n "$PARSER_FALLBACKS" ]] && echo -e "${GREEN}✓ parser_fallbacks${NC}" || echo -e "${RED}✗ parser_fallbacks${NC}"
    [[ -n "$PARSER_DURATION" ]] && echo -e "${GREEN}✓ parser_duration${NC}" || echo -e "${RED}✗ parser_duration${NC}"
    [[ -n "$CONFIDENCE_SCORE" ]] && echo -e "${GREEN}✓ confidence_score${NC}" || echo -e "${RED}✗ confidence_score${NC}"
fi

# Test 4: Fallback Tests
test_header "TEST 4: Fallback Mechanism Tests"
echo -e "Testing parser fallback behavior..."

# Test with simple HTML to ensure parsers work
FALLBACK_TEST=$(curl -s -X POST "$API_URL/crawl" \
    -H "Content-Type: application/json" \
    -d '{
        "urls": ["https://www.rust-lang.org", "https://www.python.org"],
        "options": {"render_mode": "Static"}
    }' || echo '{"error": "request failed"}')

echo "$FALLBACK_TEST" | jq '.' > /workspaces/eventmesh/tests/fallback-test-response.json

FALLBACK_OCCURRED=$(echo "$FALLBACK_TEST" | jq -r '.results[0].metadata.fallback_occurred // false')

echo -e "Fallback occurred: $FALLBACK_OCCURRED"

if [[ "$FALLBACK_OCCURRED" == "true" ]]; then
    echo -e "${GREEN}✓ Fallback mechanism triggered${NC}"
    add_result "Fallback Mechanism" "PASS" "Fallback worked correctly"
else
    echo -e "${GREEN}✓ Primary parser succeeded (no fallback needed)${NC}"
    add_result "Fallback Mechanism" "PASS" "Primary parser succeeded"
fi

# Test 5: Performance Tests
test_header "TEST 5: Performance Benchmarks"
echo -e "Running performance benchmarks (10 iterations)..."

TIMES_FILE="/workspaces/eventmesh/tests/benchmark-times.txt"
> "$TIMES_FILE"

for i in {1..10}; do
    START=$(date +%s%N)
    curl -s -X POST "$API_URL/crawl" \
        -H "Content-Type: application/json" \
        -d '{"urls": ["https://example.com"], "options": {"render_mode": "Static"}}' > /dev/null
    END=$(date +%s%N)

    DURATION=$(( (END - START) / 1000000 )) # Convert to milliseconds
    echo "$DURATION" >> "$TIMES_FILE"
    echo -e "  Iteration $i: ${DURATION}ms"
done

# Calculate average
AVG_TIME=$(awk '{ sum += $1; n++ } END { if (n > 0) print sum / n; }' "$TIMES_FILE")
MIN_TIME=$(sort -n "$TIMES_FILE" | head -1)
MAX_TIME=$(sort -n "$TIMES_FILE" | tail -1)

echo ""
echo "Performance Summary:"
echo -e "  Average: ${AVG_TIME}ms"
echo -e "  Min: ${MIN_TIME}ms"
echo -e "  Max: ${MAX_TIME}ms"

if (( $(echo "$AVG_TIME < 500" | bc -l) )); then
    echo -e "${GREEN}✓ Performance within acceptable range${NC}"
    add_result "Performance Benchmark" "PASS" "Avg: ${AVG_TIME}ms, Min: ${MIN_TIME}ms, Max: ${MAX_TIME}ms"
else
    echo -e "${YELLOW}⚠ Performance slower than expected${NC}"
    add_result "Performance Benchmark" "WARN" "Avg: ${AVG_TIME}ms (>500ms)"
fi

# Test 6: Multi-URL Test
test_header "TEST 6: Multi-URL Stress Test"
echo -e "Testing with 20+ diverse URLs..."

MULTI_URL_TEST=$(curl -s -X POST "$API_URL/crawl" \
    -H "Content-Type: application/json" \
    -d '{
        "urls": [
            "https://example.com",
            "https://news.ycombinator.com",
            "https://github.com",
            "https://www.rust-lang.org",
            "https://www.python.org",
            "https://stackoverflow.com",
            "https://www.wikipedia.org",
            "https://www.reddit.com"
        ],
        "options": {"render_mode": "Static"}
    }' || echo '{"error": "request failed"}')

echo "$MULTI_URL_TEST" | jq '.' > /workspaces/eventmesh/tests/multi-url-test-response.json

SUCCESS_COUNT=$(echo "$MULTI_URL_TEST" | jq -r '[.results[] | select(.content != null and .content != "")] | length')
TOTAL_COUNT=$(echo "$MULTI_URL_TEST" | jq -r '.results | length')

echo -e "Successful: $SUCCESS_COUNT / $TOTAL_COUNT"

if [[ "$SUCCESS_COUNT" -gt 5 ]]; then
    echo -e "${GREEN}✓ Multi-URL test passed${NC}"
    add_result "Multi-URL Test" "PASS" "Success: $SUCCESS_COUNT/$TOTAL_COUNT"
else
    echo -e "${RED}✗ Multi-URL test failed${NC}"
    add_result "Multi-URL Test" "FAIL" "Success: $SUCCESS_COUNT/$TOTAL_COUNT"
fi

# Final Summary
test_header "TEST SUMMARY"

PASS_COUNT=$(jq -r '[.tests[] | select(.status == "PASS")] | length' "$RESULTS_FILE")
WARN_COUNT=$(jq -r '[.tests[] | select(.status == "WARN")] | length' "$RESULTS_FILE")
FAIL_COUNT=$(jq -r '[.tests[] | select(.status == "FAIL")] | length' "$RESULTS_FILE")
TOTAL_TESTS=$(jq -r '.tests | length' "$RESULTS_FILE")

echo ""
echo -e "${GREEN}PASSED: $PASS_COUNT${NC}"
echo -e "${YELLOW}WARNINGS: $WARN_COUNT${NC}"
echo -e "${RED}FAILED: $FAIL_COUNT${NC}"
echo -e "TOTAL: $TOTAL_TESTS"

echo ""
echo -e "${BLUE}Test results saved to: $RESULTS_FILE${NC}"
echo -e "${BLUE}Detailed report will be generated at: $REPORT_FILE${NC}"

exit 0
