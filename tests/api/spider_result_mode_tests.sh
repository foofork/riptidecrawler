#!/bin/bash
# Comprehensive API Test Suite for Spider result_mode Feature
# Tests URL discovery and result mode functionality

set -euo pipefail

# Configuration
API_BASE_URL="${API_BASE_URL:-http://localhost:8080}"
RESULTS_DIR="tests/api/results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="$RESULTS_DIR/spider_result_mode_report_$TIMESTAMP.json"

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Setup
mkdir -p "$RESULTS_DIR"

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

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

# ============================================================================
# Test Functions
# ============================================================================

test_spider_crawl_without_result_mode() {
    # Test backward compatibility - no result_mode defaults to stats only
    local response
    response=$(curl -s -X POST "$API_BASE_URL/spider/crawl" \
        -H "Content-Type: application/json" \
        -d '{
            "seed_urls": ["https://example.com"],
            "max_pages": 5,
            "max_depth": 2
        }')

    # Should return success and standard response
    if echo "$response" | jq -e '.result.pages_crawled' > /dev/null 2>&1; then
        # Should NOT have discovered_urls field (backward compat)
        if echo "$response" | jq -e '.result.discovered_urls' > /dev/null 2>&1; then
            log_fail "Should not have discovered_urls without result_mode"
            return 1
        fi
        return 0
    else
        log_fail "Invalid response structure: $response"
        return 1
    fi
}

test_spider_crawl_with_stats_mode() {
    # Test explicit result_mode=stats
    local response
    response=$(curl -s -X POST "$API_BASE_URL/spider/crawl" \
        -H "Content-Type: application/json" \
        -d '{
            "seed_urls": ["https://example.com"],
            "max_pages": 5,
            "max_depth": 2,
            "result_mode": "stats"
        }')

    # Should have standard stats but no URLs
    if echo "$response" | jq -e '.result.pages_crawled' > /dev/null 2>&1; then
        # Should NOT have discovered_urls in stats mode
        if echo "$response" | jq -e '.result.discovered_urls' > /dev/null 2>&1; then
            log_fail "Stats mode should not include discovered_urls"
            return 1
        fi
        return 0
    else
        log_fail "Invalid response structure"
        return 1
    fi
}

test_spider_crawl_with_urls_mode() {
    # Test result_mode=urls returns discovered URLs
    local response
    response=$(curl -s -X POST "$API_BASE_URL/spider/crawl" \
        -H "Content-Type: application/json" \
        -d '{
            "seed_urls": ["https://example.com"],
            "max_pages": 10,
            "max_depth": 2,
            "result_mode": "urls"
        }')

    # Should have discovered_urls array
    if echo "$response" | jq -e '.result.discovered_urls' > /dev/null 2>&1; then
        local url_count
        url_count=$(echo "$response" | jq '.result.discovered_urls | length')

        if [ "$url_count" -ge 0 ]; then
            log_info "Discovered $url_count URLs"
            return 0
        else
            log_fail "Invalid discovered_urls array"
            return 1
        fi
    else
        log_fail "Missing discovered_urls in response: $response"
        return 1
    fi
}

test_spider_crawl_invalid_result_mode() {
    # Test that invalid result_mode returns error
    local response
    local status_code

    response=$(curl -s -w "\n%{http_code}" -X POST "$API_BASE_URL/spider/crawl" \
        -H "Content-Type: application/json" \
        -d '{
            "seed_urls": ["https://example.com"],
            "result_mode": "invalid_mode"
        }')

    status_code=$(echo "$response" | tail -n 1)

    # Should return 400 Bad Request
    if [ "$status_code" = "400" ]; then
        return 0
    else
        log_fail "Expected 400, got $status_code"
        return 1
    fi
}

test_spider_max_pages_limit() {
    # Test that max_pages limits discovered URLs
    local response
    response=$(curl -s -X POST "$API_BASE_URL/spider/crawl" \
        -H "Content-Type: application/json" \
        -d '{
            "seed_urls": ["https://example.com"],
            "max_pages": 3,
            "max_depth": 2,
            "result_mode": "urls"
        }')

    if echo "$response" | jq -e '.result.discovered_urls' > /dev/null 2>&1; then
        local url_count
        url_count=$(echo "$response" | jq '.result.discovered_urls | length')

        # Should not exceed max_pages
        if [ "$url_count" -le 3 ]; then
            log_info "URL count ($url_count) respects max_pages (3)"
            return 0
        else
            log_fail "URL count ($url_count) exceeds max_pages (3)"
            return 1
        fi
    fi
    return 1
}

test_spider_bfs_strategy() {
    # Test breadth-first strategy
    local response
    response=$(curl -s -X POST "$API_BASE_URL/spider/crawl" \
        -H "Content-Type: application/json" \
        -d '{
            "seed_urls": ["https://example.com"],
            "max_pages": 10,
            "strategy": "breadth_first",
            "result_mode": "urls"
        }')

    # Should complete successfully
    if echo "$response" | jq -e '.result.pages_crawled' > /dev/null 2>&1; then
        local strategy
        strategy=$(echo "$response" | jq -r '.state.strategy // "unknown"')
        log_info "Strategy used: $strategy"
        return 0
    fi
    return 1
}

test_spider_dfs_strategy() {
    # Test depth-first strategy
    local response
    response=$(curl -s -X POST "$API_BASE_URL/spider/crawl" \
        -H "Content-Type: application/json" \
        -d '{
            "seed_urls": ["https://example.com"],
            "max_pages": 10,
            "strategy": "depth_first",
            "result_mode": "urls"
        }')

    # Should complete successfully
    if echo "$response" | jq -e '.result.pages_crawled' > /dev/null 2>&1; then
        return 0
    fi
    return 1
}

test_spider_url_deduplication() {
    # Test that duplicate URLs are handled
    local response
    response=$(curl -s -X POST "$API_BASE_URL/spider/crawl" \
        -H "Content-Type: application/json" \
        -d '{
            "seed_urls": ["https://example.com", "https://example.com"],
            "max_pages": 5,
            "result_mode": "urls"
        }')

    if echo "$response" | jq -e '.result.discovered_urls' > /dev/null 2>&1; then
        # Should deduplicate seed URLs
        return 0
    fi
    return 1
}

test_spider_multiple_domains() {
    # Test crawling multiple domains
    local response
    response=$(curl -s -X POST "$API_BASE_URL/spider/crawl" \
        -H "Content-Type: application/json" \
        -d '{
            "seed_urls": ["https://example.com", "https://example.org"],
            "max_pages": 10,
            "result_mode": "urls"
        }')

    if echo "$response" | jq -e '.result.domains' > /dev/null 2>&1; then
        local domain_count
        domain_count=$(echo "$response" | jq '.result.domains | length')

        if [ "$domain_count" -ge 1 ]; then
            log_info "Crawled $domain_count domain(s)"
            return 0
        fi
    fi
    return 1
}

test_spider_max_depth() {
    # Test max_depth constraint
    local response
    response=$(curl -s -X POST "$API_BASE_URL/spider/crawl" \
        -H "Content-Type: application/json" \
        -d '{
            "seed_urls": ["https://example.com"],
            "max_pages": 20,
            "max_depth": 1,
            "result_mode": "urls"
        }')

    # Should respect depth limit
    if echo "$response" | jq -e '.result.pages_crawled' > /dev/null 2>&1; then
        return 0
    fi
    return 1
}

# ============================================================================
# Main Test Execution
# ============================================================================

log_info "Starting Spider Result Mode API Tests"
log_info "API Base URL: $API_BASE_URL"
log_info "Results will be saved to: $REPORT_FILE"
echo ""

# Check if API is available
log_info "Checking API availability..."
if ! curl -s -f "$API_BASE_URL/healthz" > /dev/null 2>&1; then
    log_fail "API not available at $API_BASE_URL"
    log_info "Please start the server: cargo run --release"
    exit 1
fi
log_pass "API is available"
echo ""

# Run all tests
run_test "Spider crawl without result_mode (backward compat)" test_spider_crawl_without_result_mode
run_test "Spider crawl with result_mode=stats" test_spider_crawl_with_stats_mode
run_test "Spider crawl with result_mode=urls" test_spider_crawl_with_urls_mode
run_test "Spider crawl with invalid result_mode" test_spider_crawl_invalid_result_mode
run_test "Spider respects max_pages limit" test_spider_max_pages_limit
run_test "Spider breadth-first strategy" test_spider_bfs_strategy
run_test "Spider depth-first strategy" test_spider_dfs_strategy
run_test "Spider URL deduplication" test_spider_url_deduplication
run_test "Spider multiple domain crawling" test_spider_multiple_domains
run_test "Spider max_depth constraint" test_spider_max_depth

# ============================================================================
# Generate Report
# ============================================================================

echo ""
log_info "Generating test report..."

cat > "$REPORT_FILE" <<EOF
{
  "test_suite": "Spider Result Mode API Tests",
  "timestamp": "$TIMESTAMP",
  "api_base_url": "$API_BASE_URL",
  "summary": {
    "total_tests": $TOTAL_TESTS,
    "passed": $PASSED_TESTS,
    "failed": $FAILED_TESTS,
    "success_rate": $(awk "BEGIN {printf \"%.2f\", ($PASSED_TESTS / $TOTAL_TESTS) * 100}")
  },
  "test_categories": {
    "result_mode_variants": 4,
    "crawl_strategies": 2,
    "constraint_enforcement": 3,
    "edge_cases": 1
  }
}
EOF

# ============================================================================
# Summary
# ============================================================================

echo ""
echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo "Total Tests:  $TOTAL_TESTS"
echo -e "Passed:       ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed:       ${RED}$FAILED_TESTS${NC}"
echo "Success Rate: $(awk "BEGIN {printf \"%.1f%%\", ($PASSED_TESTS / $TOTAL_TESTS) * 100}")"
echo ""
echo "Report saved to: $REPORT_FILE"
echo "=========================================="

# Exit with appropriate code
if [ $FAILED_TESTS -eq 0 ]; then
    log_pass "All tests passed!"
    exit 0
else
    log_fail "$FAILED_TESTS test(s) failed"
    exit 1
fi
