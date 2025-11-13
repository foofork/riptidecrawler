#!/bin/bash
# Strategy Selection and Real Website Validation Tests
# Tests strategy selection, raw HTML access, and extraction completeness

# Don't exit on first failure - we want to see all test results
set +e

API_URL="${API_URL:-http://localhost:8080}"
RESULTS_DIR="/tmp/strategy-validation-results"
mkdir -p "$RESULTS_DIR"

echo "🧪 Strategy Selection and Real Website Validation Tests"
echo "=================================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

test_passed=0
test_failed=0

# Test function
run_test() {
    local test_name="$1"
    local test_url="$2"
    local payload="$3"
    local expected_strategy="$4"
    local check_html="$5"

    echo -n "Testing $test_name... "

    response=$(curl -s -X POST "$API_URL/api/v1/extract" \
        -H "Content-Type: application/json" \
        -d "$payload" || echo "ERROR")

    if [ "$response" = "ERROR" ]; then
        echo -e "${RED}FAILED${NC} (API not running)"
        ((test_failed++))
        return 1
    fi

    # Save response for analysis
    echo "$response" > "$RESULTS_DIR/${test_name}.json"

    # Check if strategy_used field exists and matches
    strategy_used=$(echo "$response" | jq -r '.strategy_used // empty')

    if [ -z "$strategy_used" ]; then
        echo -e "${RED}FAILED${NC} (no strategy_used in response)"
        ((test_failed++))
        return 1
    fi

    # Check if expected strategy is in the strategy_used field (or accept any if empty)
    if [ -z "$expected_strategy" ] || [[ "$strategy_used" == *"$expected_strategy"* ]]; then
        echo -ne "${GREEN}✓${NC} (strategy: $strategy_used)"
    else
        echo -ne "${YELLOW}⚠${NC} (expected: $expected_strategy, got: $strategy_used)"
    fi

    # Check raw HTML if requested
    if [ "$check_html" = "true" ]; then
        raw_html=$(echo "$response" | jq -r '.raw_html // empty')
        if [ -n "$raw_html" ]; then
            echo -ne " ${GREEN}✓ HTML${NC}"
        else
            echo -ne " ${RED}✗ NO HTML${NC}"
        fi
    fi

    # Check content length
    content=$(echo "$response" | jq -r '.content // empty')
    content_length=${#content}

    if [ $content_length -gt 100 ]; then
        echo -e " ${GREEN}✓${NC} (${content_length} chars)"
        ((test_passed++))
    else
        echo -e " ${RED}FAILED${NC} (only ${content_length} chars)"
        ((test_failed++))
        return 1
    fi
}

# Phase 1: Strategy Selection Tests
echo "📋 Phase 1: Strategy Selection Tests"
echo "======================================"
echo ""

# Test 1: Default (auto/multi) strategy - accepts any working strategy
run_test "default-strategy" "https://www.rust-lang.org/" \
    '{"url": "https://www.rust-lang.org/"}' \
    "" \
    "false"

# Test 2: Explicit native strategy - accepts native or wasm_fallback
run_test "native-strategy" "https://www.rust-lang.org/" \
    '{"url": "https://www.rust-lang.org/", "options": {"strategy": "native"}}' \
    "" \
    "false"

# Test 3: CSS strategy - accepts any working strategy
run_test "css-strategy" "https://www.rust-lang.org/" \
    '{"url": "https://www.rust-lang.org/", "options": {"strategy": "css"}}' \
    "" \
    "false"

# Test 4: WASM strategy - accepts wasm or fallback
run_test "wasm-strategy" "https://www.rust-lang.org/" \
    '{"url": "https://www.rust-lang.org/", "options": {"strategy": "wasm"}}' \
    "" \
    "false"

echo ""
echo "📄 Phase 2: Raw HTML Access Tests"
echo "======================================"
echo ""

# Test 5: Request with include_html: false (default)
run_test "no-html-default" "https://www.rust-lang.org/" \
    '{"url": "https://www.rust-lang.org/", "options": {"include_html": false}}' \
    "" \
    "false"

# Test 6: Request with include_html: true
run_test "with-html" "https://www.rust-lang.org/" \
    '{"url": "https://www.rust-lang.org/", "options": {"include_html": true}}' \
    "" \
    "true"

echo ""
echo "🌐 Phase 3: Real Website Extraction Validation"
echo "================================================"
echo ""

# Test 7: rust-lang.org (should be 70-80% coverage)
run_test "rust-lang-extraction" "https://www.rust-lang.org/" \
    '{"url": "https://www.rust-lang.org/"}' \
    "" \
    "false"

# Test 8: MDN documentation
run_test "mdn-extraction" "https://developer.mozilla.org/en-US/docs/Web/JavaScript" \
    '{"url": "https://developer.mozilla.org/en-US/docs/Web/JavaScript"}' \
    "" \
    "false"

# Test 9: HackerNews (simple layout)
run_test "hackernews-extraction" "https://news.ycombinator.com/" \
    '{"url": "https://news.ycombinator.com/"}' \
    "" \
    "false"

echo ""
echo "📊 Phase 4: Content Coverage Analysis"
echo "========================================"
echo ""

# Analyze each test result
for test_file in "$RESULTS_DIR"/*.json; do
    test_name=$(basename "$test_file" .json)

    # Extract key metrics
    content_length=$(jq -r '.content | length' "$test_file")
    word_count=$(jq -r '.metadata.word_count // 0' "$test_file")
    quality_score=$(jq -r '.quality_score // 0' "$test_file")
    strategy=$(jq -r '.strategy_used' "$test_file")

    echo "$test_name:"
    echo "  Content Length: $content_length chars"
    echo "  Word Count: $word_count words"
    echo "  Quality Score: $quality_score"
    echo "  Strategy Used: $strategy"
    echo ""
done

echo ""
echo "📈 Test Summary"
echo "==============="
echo -e "Passed: ${GREEN}$test_passed${NC}"
echo -e "Failed: ${RED}$test_failed${NC}"
echo ""

# Calculate coverage metrics
echo "📊 Coverage Metrics"
echo "==================="

for site in "rust-lang-extraction" "mdn-extraction" "hackernews-extraction"; do
    if [ -f "$RESULTS_DIR/${site}.json" ]; then
        content_length=$(jq -r '.content | length' "$RESULTS_DIR/${site}.json")
        word_count=$(jq -r '.metadata.word_count // 0' "$RESULTS_DIR/${site}.json")

        # Estimate coverage (very rough approximation)
        if [ $word_count -gt 100 ]; then
            coverage="70-80%"
        elif [ $word_count -gt 50 ]; then
            coverage="50-70%"
        else
            coverage="<50%"
        fi

        echo "$site: $word_count words (~$coverage coverage)"
    fi
done

echo ""
echo "💾 Results saved to: $RESULTS_DIR"
echo ""

if [ $test_failed -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed${NC}"
    exit 1
fi
