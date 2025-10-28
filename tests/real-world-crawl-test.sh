#!/bin/bash
# ============================================================================
# Real-World Crawl Testing Suite
# ============================================================================
# Tests actual URL crawling with all extraction methods
# ============================================================================

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

API_URL="http://localhost:8080"
TEST_LOG="/tmp/crawl-test-$(date +%Y%m%d-%H%M%S).log"

echo -e "${BLUE}============================================================================${NC}"
echo -e "${BLUE}Real-World Crawl Testing Suite${NC}"
echo -e "${BLUE}============================================================================${NC}"
echo ""

# Test URLs covering different scenarios
declare -A TEST_URLS=(
    ["simple-html"]="https://example.com"
    ["news-site"]="https://www.bbc.com/news"
    ["github"]="https://github.com/rust-lang/rust"
    ["documentation"]="https://doc.rust-lang.org/book/"
    ["wikipedia"]="https://en.wikipedia.org/wiki/Web_scraping"
)

TESTS_PASSED=0
TESTS_FAILED=0

test_crawl() {
    local name=$1
    local url=$2
    local method=$3

    echo -e "${BLUE}[TEST]${NC} $name - $url (method: $method)"

    local payload=$(cat <<EOF
{
    "urls": ["$url"],
    "cache_mode": "bypass"
}
EOF
)

    local start_time=$(date +%s)
    local response=$(curl -s -X POST "$API_URL/crawl" \
        -H "Content-Type: application/json" \
        -d "$payload" 2>&1)
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    echo "$response" >> "$TEST_LOG"

    # Check for successful response
    if echo "$response" | jq -e '.results[0].success' >/dev/null 2>&1; then
        local content_length=$(echo "$response" | jq -r '.results[0].content.text' | wc -c)
        local links_count=$(echo "$response" | jq -r '.results[0].links | length')

        echo -e "${GREEN}[PASS]${NC} Success! Duration: ${duration}s, Content: ${content_length} chars, Links: ${links_count}"
        ((TESTS_PASSED++))
        return 0
    else
        local error=$(echo "$response" | jq -r '.error.message // .results[0].error // "Unknown error"')
        echo -e "${RED}[FAIL]${NC} Error: $error"
        ((TESTS_FAILED++))
        return 1
    fi
}

test_extraction_methods() {
    local url=$1
    local name=$2

    echo ""
    echo -e "${YELLOW}Testing: $name${NC}"
    echo -e "${YELLOW}URL: $url${NC}"
    echo ""

    # Test with default (WASM first)
    test_crawl "$name [WASM]" "$url" "wasm" || true

    sleep 2
}

# Test 1: Simple HTML (should work with WASM)
test_extraction_methods "${TEST_URLS[simple-html]}" "Simple HTML Page"

# Test 2: News site (complex HTML)
test_extraction_methods "${TEST_URLS[news-site]}" "News Website"

# Test 3: GitHub (SPA elements)
test_extraction_methods "${TEST_URLS[github]}" "GitHub Repository"

# Test 4: Documentation site
test_extraction_methods "${TEST_URLS[documentation]}" "Rust Documentation"

# Test 5: Wikipedia (rich content)
test_extraction_methods "${TEST_URLS[wikipedia]}" "Wikipedia Page"

# Test 6: Batch crawl
echo ""
echo -e "${BLUE}[TEST]${NC} Batch crawl (multiple URLs)"
BATCH_PAYLOAD=$(cat <<EOF
{
    "urls": [
        "https://example.com",
        "https://www.rust-lang.org",
        "https://github.com"
    ],
    "cache_mode": "bypass"
}
EOF
)

BATCH_RESPONSE=$(curl -s -X POST "$API_URL/crawl" \
    -H "Content-Type: application/json" \
    -d "$BATCH_PAYLOAD" 2>&1)

BATCH_SUCCESS=$(echo "$BATCH_RESPONSE" | jq '[.results[].success] | map(select(. == true)) | length')
BATCH_TOTAL=$(echo "$BATCH_RESPONSE" | jq '.results | length')

if [ "$BATCH_SUCCESS" -gt 0 ]; then
    echo -e "${GREEN}[PASS]${NC} Batch crawl: $BATCH_SUCCESS/$BATCH_TOTAL successful"
    ((TESTS_PASSED++))
else
    echo -e "${RED}[FAIL]${NC} Batch crawl: $BATCH_SUCCESS/$BATCH_TOTAL successful"
    ((TESTS_FAILED++))
fi

# Summary
echo ""
echo -e "${BLUE}============================================================================${NC}"
echo -e "${BLUE}Test Summary${NC}"
echo -e "${BLUE}============================================================================${NC}"
echo -e "Total Tests: $((TESTS_PASSED + TESTS_FAILED))"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo -e "Log file: $TEST_LOG"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
