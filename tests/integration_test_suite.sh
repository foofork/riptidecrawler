#!/bin/bash
#
# Comprehensive Integration Test Suite for RipTide
# Tests real servers with real URLs - no mocks
#

set -euo pipefail

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test URLs
declare -A TEST_URLS=(
    ["simple"]="http://example.com"
    ["complex"]="https://en.wikipedia.org/wiki/Rust_(programming_language)"
    ["ssr"]="https://news.ycombinator.com"
    ["spa"]="https://react.dev"
    ["framework"]="https://github.com"
)

# Results tracking
declare -A TEST_RESULTS
declare -A TEST_ERRORS
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Server URLs
API_URL="http://localhost:8080"
HEADLESS_URL="http://localhost:9123"

# Output directory
OUT_DIR="/workspaces/eventmesh/tests/integration_results"
mkdir -p "$OUT_DIR"

#
# Logging functions
#
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_test() {
    echo -e "${YELLOW}[TEST]${NC} $1"
}

#
# Test execution wrapper
#
run_test() {
    local test_name="$1"
    local test_command="$2"

    ((TESTS_RUN++))
    log_test "$test_name"

    local output_file="$OUT_DIR/${test_name//[: ]/_}.json"
    local error_file="$OUT_DIR/${test_name//[: ]/_}.error"

    if eval "$test_command" > "$output_file" 2> "$error_file"; then
        ((TESTS_PASSED++))
        TEST_RESULTS["$test_name"]="PASS"
        log_info "✓ PASSED: $test_name"
        return 0
    else
        ((TESTS_FAILED++))
        TEST_RESULTS["$test_name"]="FAIL"
        TEST_ERRORS["$test_name"]=$(cat "$error_file")
        log_error "✗ FAILED: $test_name"
        cat "$error_file"
        return 1
    fi
}

#
# Health checks
#
test_health_checks() {
    log_info "Running health checks..."

    run_test "Health: API Server" "curl -sf $API_URL/health"
    run_test "Health: Headless Server" "curl -sf $HEADLESS_URL/healthz"
    run_test "Health: Redis Connection" "redis-cli ping"
}

#
# Extraction tests
#
test_extraction() {
    log_info "Testing extraction engines..."

    for name in "${!TEST_URLS[@]}"; do
        local url="${TEST_URLS[$name]}"

        # Test raw extraction
        run_test "Extract Raw: $name" \
            "curl -sf -X POST $API_URL/api/v1/extract -H 'Content-Type: application/json' -d '{\"url\":\"$url\",\"engine\":\"raw\"}'"

        # Test WASM extraction
        run_test "Extract WASM: $name" \
            "curl -sf -X POST $API_URL/api/v1/extract -H 'Content-Type: application/json' -d '{\"url\":\"$url\",\"engine\":\"wasm\"}'"

        # Test headless extraction
        run_test "Extract Headless: $name" \
            "curl -sf -X POST $API_URL/api/v1/extract -H 'Content-Type: application/json' -d '{\"url\":\"$url\",\"engine\":\"headless\"}'"
    done
}

#
# Table extraction tests
#
test_tables() {
    log_info "Testing table extraction..."

    # Wikipedia has many tables
    run_test "Tables: Wikipedia" \
        "curl -sf -X POST $API_URL/api/v1/extract -H 'Content-Type: application/json' -d '{\"url\":\"${TEST_URLS[complex]}\",\"extract_tables\":true}'"

    # HN has simple tables
    run_test "Tables: HackerNews" \
        "curl -sf -X POST $API_URL/api/v1/extract -H 'Content-Type: application/json' -d '{\"url\":\"${TEST_URLS[ssr]}\",\"extract_tables\":true}'"
}

#
# Crawl tests
#
test_crawling() {
    log_info "Testing crawling with Spider..."

    run_test "Crawl: 2-level example.com" \
        "curl -sf -X POST $API_URL/api/v1/crawl -H 'Content-Type: application/json' -d '{\"url\":\"${TEST_URLS[simple]}\",\"max_depth\":2,\"max_pages\":10}'"
}

#
# Render tests
#
test_rendering() {
    log_info "Testing JavaScript rendering..."

    # SPA rendering
    run_test "Render: React SPA" \
        "curl -sf -X POST $HEADLESS_URL/render -H 'Content-Type: application/json' -d '{\"url\":\"${TEST_URLS[spa]}\",\"wait_for\":\"networkidle\"}'"

    # Screenshot
    run_test "Screenshot: GitHub" \
        "curl -sf -X POST $HEADLESS_URL/screenshot -H 'Content-Type: application/json' -d '{\"url\":\"${TEST_URLS[framework]}\",\"full_page\":false}'"
}

#
# Stealth tests
#
test_stealth() {
    log_info "Testing stealth and fingerprint evasion..."

    run_test "Stealth: Bot Detection" \
        "curl -sf -X POST $HEADLESS_URL/render -H 'Content-Type: application/json' -d '{\"url\":\"https://bot.sannysoft.com\",\"stealth\":true}'"
}

#
# Performance tests
#
test_performance() {
    log_info "Testing performance metrics..."

    run_test "Metrics: Prometheus" \
        "curl -sf $API_URL/metrics | grep -E 'riptide_|http_requests'"
}

#
# Generate test report
#
generate_report() {
    local report_file="$OUT_DIR/TEST_REPORT.md"

    cat > "$report_file" <<EOF
# RipTide Integration Test Report
**Generated:** $(date -u +"%Y-%m-%d %H:%M:%S UTC")

## Summary
- **Total Tests:** $TESTS_RUN
- **Passed:** $TESTS_PASSED
- **Failed:** $TESTS_FAILED
- **Success Rate:** $(awk "BEGIN {printf \"%.1f%%\", ($TESTS_PASSED/$TESTS_RUN)*100}")

## Test Environment
- Redis: $(redis-cli --version)
- API Server: $API_URL
- Headless Server: $HEADLESS_URL

## Test Results

EOF

    for test_name in "${!TEST_RESULTS[@]}"; do
        local result="${TEST_RESULTS[$test_name]}"
        echo "### $test_name" >> "$report_file"
        echo "**Status:** $result" >> "$report_file"

        if [ "$result" == "FAIL" ]; then
            echo "**Error:**" >> "$report_file"
            echo '```' >> "$report_file"
            echo "${TEST_ERRORS[$test_name]}" >> "$report_file"
            echo '```' >> "$report_file"
        fi

        echo "" >> "$report_file"
    done

    log_info "Test report generated: $report_file"
    cat "$report_file"
}

#
# Main test execution
#
main() {
    log_info "Starting comprehensive integration test suite..."
    log_info "Output directory: $OUT_DIR"

    # Run all test suites
    test_health_checks || true
    test_extraction || true
    test_tables || true
    test_crawling || true
    test_rendering || true
    test_stealth || true
    test_performance || true

    # Generate report
    generate_report

    # Exit with appropriate code
    if [ $TESTS_FAILED -gt 0 ]; then
        log_error "Integration tests completed with failures"
        exit 1
    else
        log_info "All integration tests passed!"
        exit 0
    fi
}

main "$@"
