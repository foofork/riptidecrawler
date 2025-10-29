#!/bin/bash
# Professional API Test Suite for RipTide Extraction
# This is what production teams use instead of manual curl tests

set -euo pipefail

# Configuration
API_BASE_URL="${API_BASE_URL:-http://localhost:8080}"
RESULTS_DIR="tests/api/results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="$RESULTS_DIR/test_report_$TIMESTAMP.json"

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Setup
mkdir -p "$RESULTS_DIR"

# Helper functions
log_test() {
    echo -e "${YELLOW}[TEST]${NC} $1"
}

log_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((PASSED_TESTS++))
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((FAILED_TESTS++))
}

run_test() {
    ((TOTAL_TESTS++))
    local test_name="$1"
    local test_func="$2"

    log_test "$test_name"

    if $test_func; then
        log_pass "$test_name"
        return 0
    else
        log_fail "$test_name"
        return 1
    fi
}

# Test 1: Health Check
test_health_check() {
    local response=$(curl -s -w "%{http_code}" -o /tmp/health.json "$API_BASE_URL/healthz")
    local status_code="${response: -3}"

    if [[ "$status_code" == "200" ]]; then
        local health_status=$(jq -r '.status' /tmp/health.json 2>/dev/null || echo "error")
        [[ "$health_status" == "healthy" ]]
    else
        return 1
    fi
}

# Test 2: Single URL Extraction
test_single_url_extraction() {
    local response=$(curl -s -X POST "$API_BASE_URL/crawl" \
        -H 'Content-Type: application/json' \
        -d '{
            "urls": ["https://example.com"],
            "options": {"return_format": "markdown"}
        }')

    local successful=$(echo "$response" | jq -r '.successful // 0')
    [[ "$successful" -eq 1 ]]
}

# Test 3: Batch URL Extraction
test_batch_extraction() {
    local response=$(curl -s -X POST "$API_BASE_URL/crawl" \
        -H 'Content-Type: application/json' \
        -d '{
            "urls": [
                "https://example.com",
                "https://example.org",
                "https://www.iana.org"
            ],
            "options": {"return_format": "markdown"}
        }')

    local successful=$(echo "$response" | jq -r '.successful // 0')
    [[ "$successful" -ge 2 ]] # At least 2 should succeed
}

# Test 4: Cache Behavior
test_cache_behavior() {
    # First request (uncached)
    local response1=$(curl -s -X POST "$API_BASE_URL/crawl" \
        -H 'Content-Type: application/json' \
        -d '{
            "urls": ["https://example.com"],
            "options": {"cache_mode": "bypass"}
        }')

    local time1=$(echo "$response1" | jq -r '.results[0].processing_time_ms // 0')

    # Second request (cached)
    local response2=$(curl -s -X POST "$API_BASE_URL/crawl" \
        -H 'Content-Type: application/json' \
        -d '{
            "urls": ["https://example.com"],
            "options": {"cache_mode": "read_through"}
        }')

    local from_cache=$(echo "$response2" | jq -r '.results[0].from_cache // false')
    [[ "$from_cache" == "true" ]]
}

# Test 5: Error Handling - Invalid URL
test_invalid_url_handling() {
    local response=$(curl -s -X POST "$API_BASE_URL/crawl" \
        -H 'Content-Type: application/json' \
        -d '{
            "urls": ["not-a-valid-url"],
            "options": {"return_format": "markdown"}
        }')

    local failed=$(echo "$response" | jq -r '.failed // 0')
    [[ "$failed" -ge 1 ]]
}

# Test 6: Performance - Response Time
test_response_time() {
    local start=$(date +%s%N)

    curl -s -X POST "$API_BASE_URL/crawl" \
        -H 'Content-Type: application/json' \
        -d '{
            "urls": ["https://example.com"],
            "options": {"return_format": "markdown"}
        }' > /dev/null

    local end=$(date +%s%N)
    local duration=$(( (end - start) / 1000000 )) # Convert to ms

    [[ "$duration" -lt 5000 ]] # Should complete within 5 seconds
}

# Test 7: Content Quality - Gate Decisions
test_gate_decisions() {
    local response=$(curl -s -X POST "$API_BASE_URL/crawl" \
        -H 'Content-Type: application/json' \
        -d '{
            "urls": ["https://example.com"],
            "options": {"cache_mode": "bypass"}
        }')

    local gate_decision=$(echo "$response" | jq -r '.results[0].gate_decision // "unknown"')
    [[ "$gate_decision" != "unknown" ]]
}

# Test 8: Different Return Formats
test_return_formats() {
    local formats=("markdown" "html" "text")
    local all_passed=true

    for format in "${formats[@]}"; do
        local response=$(curl -s -X POST "$API_BASE_URL/crawl" \
            -H 'Content-Type: application/json' \
            -d "{
                \"urls\": [\"https://example.com\"],
                \"options\": {\"return_format\": \"$format\"}
            }")

        local successful=$(echo "$response" | jq -r '.successful // 0')
        if [[ "$successful" -ne 1 ]]; then
            all_passed=false
            break
        fi
    done

    $all_passed
}

# Test 9: Concurrent Requests
test_concurrent_requests() {
    local pids=()

    for i in {1..3}; do
        curl -s -X POST "$API_BASE_URL/crawl" \
            -H 'Content-Type: application/json' \
            -d "{
                \"urls\": [\"https://example.com\"],
                \"options\": {\"return_format\": \"markdown\"}
            }" > "$RESULTS_DIR/concurrent_$i.json" &
        pids+=($!)
    done

    # Wait for all requests
    for pid in "${pids[@]}"; do
        wait "$pid" || return 1
    done

    # Verify all succeeded
    local all_passed=true
    for i in {1..3}; do
        local successful=$(jq -r '.successful // 0' "$RESULTS_DIR/concurrent_$i.json")
        if [[ "$successful" -ne 1 ]]; then
            all_passed=false
            break
        fi
    done

    $all_passed
}

# Test 10: Extraction Content Validation
test_content_validation() {
    local response=$(curl -s -X POST "$API_BASE_URL/crawl" \
        -H 'Content-Type: application/json' \
        -d '{
            "urls": ["https://example.com"],
            "options": {"return_format": "markdown"}
        }')

    local title=$(echo "$response" | jq -r '.results[0].document.title // ""')
    local text=$(echo "$response" | jq -r '.results[0].document.text // ""')

    [[ -n "$title" ]] && [[ -n "$text" ]] && [[ ${#text} -gt 50 ]]
}

# Run all tests
echo "========================================"
echo "RipTide API Test Suite"
echo "Base URL: $API_BASE_URL"
echo "Started: $(date)"
echo "========================================"
echo ""

run_test "Health Check" test_health_check
run_test "Single URL Extraction" test_single_url_extraction
run_test "Batch URL Extraction" test_batch_extraction
run_test "Cache Behavior" test_cache_behavior
run_test "Invalid URL Handling" test_invalid_url_handling
run_test "Response Time < 5s" test_response_time
run_test "Gate Decision Logic" test_gate_decisions
run_test "Multiple Return Formats" test_return_formats
run_test "Concurrent Requests" test_concurrent_requests
run_test "Content Validation" test_content_validation

# Generate report
echo ""
echo "========================================"
echo "Test Summary"
echo "========================================"
echo "Total Tests:  $TOTAL_TESTS"
echo "Passed:       $PASSED_TESTS"
echo "Failed:       $FAILED_TESTS"
echo "Success Rate: $(awk "BEGIN {printf \"%.1f%%\", ($PASSED_TESTS/$TOTAL_TESTS)*100}")"
echo "========================================"

# Save JSON report
cat > "$REPORT_FILE" <<EOF
{
    "timestamp": "$TIMESTAMP",
    "api_base_url": "$API_BASE_URL",
    "total_tests": $TOTAL_TESTS,
    "passed": $PASSED_TESTS,
    "failed": $FAILED_TESTS,
    "success_rate": $(awk "BEGIN {printf \"%.2f\", ($PASSED_TESTS/$TOTAL_TESTS)*100}")
}
EOF

echo ""
echo "Report saved to: $REPORT_FILE"

# Exit with appropriate code
if [[ $FAILED_TESTS -eq 0 ]]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed.${NC}"
    exit 1
fi
