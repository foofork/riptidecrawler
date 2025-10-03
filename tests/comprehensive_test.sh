#!/bin/bash

# COMPREHENSIVE Real-World Testing Suite for RipTide API
# Tests ALL major features: strategies, streaming, PDF, tables, spider, stealth

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configuration
SERVER_URL="http://localhost:3000"
OUTPUT_DIR="/workspaces/eventmesh/tests/output"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
TEST_LOG="$OUTPUT_DIR/comprehensive_${TIMESTAMP}.csv"
RESULTS_DIR="$OUTPUT_DIR/results_${TIMESTAMP}"

mkdir -p "$RESULTS_DIR"

# Global counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

echo -e "${CYAN}╔═══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║           RipTide COMPREHENSIVE Real-World Test Suite                 ║${NC}"
echo -e "${CYAN}║                    $(date)                       ║${NC}"
echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Initialize CSV log
echo "Category,Test Name,URL,HTTP Code,Duration(s),Size(bytes),Status,Details" > "$TEST_LOG"

# Test function with comprehensive metrics
test_endpoint() {
    local category="$1"
    local test_name="$2"
    local method="${3:-POST}"
    local endpoint="$4"
    local data="$5"
    local url_tested="${6:-N/A}"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    echo -e "\n${YELLOW}[$TOTAL_TESTS] Testing: ${test_name}${NC}"
    echo -e "   Category: ${category}"
    echo -e "   Endpoint: ${method} ${endpoint}"
    [ "$url_tested" != "N/A" ] && echo -e "   URL: ${url_tested}"
    echo "   ─────────────────────────────────────────────────────────"

    local output_file="$RESULTS_DIR/${category}_${test_name//[ \/]/_}.json"
    local start_time=$(date +%s.%N)

    # Make request
    if [ "$method" = "GET" ]; then
        local http_code=$(curl -s -w "%{http_code}" -o "$output_file" \
            -X GET "${SERVER_URL}${endpoint}" 2>&1)
    else
        local http_code=$(curl -s -w "%{http_code}" -o "$output_file" \
            -X POST "${SERVER_URL}${endpoint}" \
            -H "Content-Type: application/json" \
            -d "$data" 2>&1)
    fi

    local end_time=$(date +%s.%N)
    local duration=$(echo "$end_time - $start_time" | bc 2>/dev/null || echo "0")

    # Analyze response
    local file_size=$(stat -f%z "$output_file" 2>/dev/null || stat -c%s "$output_file" 2>/dev/null || echo 0)

    if [[ "$http_code" =~ ^2[0-9]{2}$ ]]; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
        echo -e "   ${GREEN}✓ PASSED${NC} (${duration}s, ${file_size} bytes)"

        # Show content preview
        if [ -f "$output_file" ] && [ -s "$output_file" ]; then
            echo -e "   ${BLUE}Preview:${NC}"
            head -n 5 "$output_file" | sed 's/^/      /' | cut -c1-100

            # Extract interesting metrics
            local detail="OK"
            if grep -q "\"success\"" "$output_file" 2>/dev/null; then
                detail="success:true"
            fi
            echo "$category,$test_name,$url_tested,$http_code,$duration,$file_size,PASSED,$detail" >> "$TEST_LOG"
        fi
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
        echo -e "   ${RED}✗ FAILED${NC} HTTP $http_code"
        echo -e "   ${RED}Error:${NC}"
        head -n 10 "$output_file" 2>/dev/null | sed 's/^/      /' || echo "      (no output)"
        echo "$category,$test_name,$url_tested,$http_code,$duration,$file_size,FAILED,http_$http_code" >> "$TEST_LOG"
    fi
}

# Check server health
echo -e "${CYAN}═══ Server Health Check ═══${NC}"
if ! curl -s "${SERVER_URL}/health" > /dev/null 2>&1; then
    echo -e "${RED}✗ Server not responding at ${SERVER_URL}${NC}"
    echo -e "${YELLOW}Please start the server first:${NC}"
    echo -e "  cd /workspaces/eventmesh && ./target/release/riptide-api"
    exit 1
fi
echo -e "${GREEN}✓ Server is healthy${NC}"

#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# TEST SUITE 1: BASIC CRAWLING
#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo -e "\n${CYAN}╔═══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  TEST SUITE 1: Basic Crawling & Extraction                            ║${NC}"
echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════════════╝${NC}"

# Wikipedia - Clean structured content
test_endpoint "Crawling" "Wikipedia scrape" "POST" "/scrape" \
    '{"url": "https://en.wikipedia.org/wiki/Web_scraping", "format": "markdown"}' \
    "https://en.wikipedia.org/wiki/Web_scraping"

# GitHub - Dynamic JavaScript-heavy
test_endpoint "Crawling" "GitHub repo scrape" "POST" "/scrape" \
    '{"url": "https://github.com/rust-lang/rust", "format": "markdown"}' \
    "https://github.com/rust-lang/rust"

# Hacker News - Simple static HTML
test_endpoint "Crawling" "Hacker News" "POST" "/scrape" \
    '{"url": "https://news.ycombinator.com/", "format": "markdown"}' \
    "https://news.ycombinator.com/"

#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# TEST SUITE 2: ADVANCED EXTRACTION STRATEGIES
#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo -e "\n${CYAN}╔═══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  TEST SUITE 2: Advanced Extraction Strategies                         ║${NC}"
echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════════════╝${NC}"

# CSS Strategy
test_endpoint "Strategies" "CSS extraction (MDN)" "POST" "/strategies/extract" \
    '{"url": "https://developer.mozilla.org/en-US/docs/Web/HTML", "strategy": "css", "selectors": {"title": "h1", "content": "article"}}' \
    "https://developer.mozilla.org/en-US/docs/Web/HTML"

# TREK Strategy (Table extraction)
test_endpoint "Strategies" "TREK table extraction" "POST" "/strategies/extract" \
    '{"url": "https://en.wikipedia.org/wiki/List_of_countries_by_population", "strategy": "trek"}' \
    "https://en.wikipedia.org/wiki/List_of_countries_by_population"

#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# TEST SUITE 3: TABLE EXTRACTION
#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo -e "\n${CYAN}╔═══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  TEST SUITE 3: Table Extraction & Export                              ║${NC}"
echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════════════╝${NC}"

# Extract tables
test_endpoint "Tables" "Wikipedia country table" "POST" "/api/v1/tables/extract" \
    '{"url": "https://en.wikipedia.org/wiki/List_of_countries_by_population", "format": "json"}' \
    "https://en.wikipedia.org/wiki/List_of_countries_by_population"

# Export as CSV
test_endpoint "Tables" "Export table as CSV" "POST" "/api/v1/tables/export" \
    '{"url": "https://en.wikipedia.org/wiki/List_of_countries_by_area", "format": "csv"}' \
    "https://en.wikipedia.org/wiki/List_of_countries_by_area"

#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# TEST SUITE 4: HEADLESS RENDERING
#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo -e "\n${CYAN}╔═══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  TEST SUITE 4: Headless Browser Rendering                             ║${NC}"
echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════════════╝${NC}"

# Render JavaScript-heavy site
test_endpoint "Rendering" "Headless render (React site)" "POST" "/render" \
    '{"url": "https://www.rust-lang.org/", "wait_for": "networkidle", "format": "markdown"}' \
    "https://www.rust-lang.org/"

#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# TEST SUITE 5: DEEP CRAWLING (SPIDER)
#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo -e "\n${CYAN}╔═══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  TEST SUITE 5: Deep Crawling (Spider)                                 ║${NC}"
echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════════════╝${NC}"

# Spider crawl with depth limit
test_endpoint "Spider" "Deep crawl (depth=2)" "POST" "/spider/crawl" \
    '{"seed_url": "https://www.rust-lang.org/", "max_depth": 2, "max_pages": 5, "respect_robots": true}' \
    "https://www.rust-lang.org/"

#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# TEST SUITE 6: STEALTH & ANTI-BOT
#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo -e "\n${CYAN}╔═══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  TEST SUITE 6: Stealth & Anti-Bot Detection                           ║${NC}"
echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════════════╝${NC}"

# Check stealth configuration
test_endpoint "Stealth" "Get stealth config" "GET" "/stealth/config" "" ""

# Test with stealth enabled
test_endpoint "Stealth" "Scrape with stealth" "POST" "/scrape" \
    '{"url": "https://httpbin.org/headers", "stealth": true, "format": "markdown"}' \
    "https://httpbin.org/headers"

#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# TEST SUITE 7: MONITORING & HEALTH
#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo -e "\n${CYAN}╔═══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  TEST SUITE 7: Monitoring & Health Checks                             ║${NC}"
echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════════════╝${NC}"

# Health check
test_endpoint "Health" "Health check" "GET" "/health" "" ""

# Prometheus metrics
test_endpoint "Health" "Prometheus metrics" "GET" "/metrics" "" ""

#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# FINAL SUMMARY
#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo -e "\n${CYAN}╔═══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║                        TEST SUMMARY & RESULTS                          ║${NC}"
echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════════════╝${NC}"

echo -e "\n${BLUE}Test Execution Summary:${NC}"
echo -e "  Total Tests:    ${TOTAL_TESTS}"
echo -e "  ${GREEN}Passed:         ${PASSED_TESTS}${NC}"
echo -e "  ${RED}Failed:         ${FAILED_TESTS}${NC}"

if [ $TOTAL_TESTS -gt 0 ]; then
    SUCCESS_RATE=$(echo "scale=1; $PASSED_TESTS * 100 / $TOTAL_TESTS" | bc)
    echo -e "  Success Rate:   ${SUCCESS_RATE}%"
fi

# Category breakdown
echo -e "\n${BLUE}Results by Category:${NC}"
tail -n +2 "$TEST_LOG" | awk -F',' '{
    cat[$1]++;
    if ($7 == "PASSED") passed[$1]++;
} END {
    for (c in cat) {
        p = (passed[c] ? passed[c] : 0);
        printf "  %-15s %d/%d passed\n", c":", p, cat[c];
    }
}' | sort

# Performance stats
echo -e "\n${BLUE}Performance Statistics:${NC}"
tail -n +2 "$TEST_LOG" | grep "PASSED" | awk -F',' '{
    sum += $5; count++;
    if ($5 > max) max = $5;
    if (min == 0 || $5 < min) min = $5;
    size_sum += $6;
} END {
    if (count > 0) {
        printf "  Avg Duration:   %.2fs\n", sum/count;
        printf "  Min Duration:   %.2fs\n", min;
        printf "  Max Duration:   %.2fs\n", max;
        printf "  Total Data:     %.2f MB\n", size_sum/1024/1024;
        printf "  Avg Size:       %.2f KB\n", size_sum/count/1024;
    }
}'

echo -e "\n${BLUE}Detailed Results:${NC}"
echo ""
column -t -s',' "$TEST_LOG" | head -30

echo -e "\n${BLUE}Files Generated:${NC}"
echo -e "  CSV Log:        ${TEST_LOG}"
echo -e "  Results Dir:    ${RESULTS_DIR}"
echo -e "  Total Files:    $(ls -1 "$RESULTS_DIR" | wc -l)"

# Show sample output files
echo -e "\n${BLUE}Sample Output Files:${NC}"
ls -lh "$RESULTS_DIR" | head -10 | tail -5 | awk '{printf "  %s  %10s  %s\n", $9, $5, $9}'

# Exit status
echo ""
if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                     ✓ ALL TESTS PASSED!                               ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════════════╝${NC}"
    exit 0
else
    echo -e "${RED}╔═══════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║                   ✗ SOME TESTS FAILED                                 ║${NC}"
    echo -e "${RED}╚═══════════════════════════════════════════════════════════════════════╝${NC}"
    exit 1
fi
