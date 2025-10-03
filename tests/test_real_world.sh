#!/bin/bash

# Real-world testing script for EventMesh self-hosted version
# Tests scraping of various real-world websites and logs all results

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SERVER_URL="http://localhost:3000"
OUTPUT_DIR="/workspaces/eventmesh/tests/output"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
TEST_LOG="$OUTPUT_DIR/test_run_${TIMESTAMP}.log"

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     EventMesh Real-World Testing Suite                    ║${NC}"
echo -e "${BLUE}║     $(date)                             ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Function to test a URL
test_url() {
    local url="$1"
    local name="$2"
    local output_file="$OUTPUT_DIR/${name}_${TIMESTAMP}.json"

    echo -e "${YELLOW}Testing: ${name}${NC}"
    echo -e "URL: ${url}"
    echo "-----------------------------------------------------------"

    # Start timing
    local start_time=$(date +%s.%N)

    # Make the request
    local http_code=$(curl -s -w "%{http_code}" -o "$output_file" \
        -X POST "${SERVER_URL}/scrape" \
        -H "Content-Type: application/json" \
        -d "{\"url\": \"${url}\", \"format\": \"markdown\"}")

    # End timing
    local end_time=$(date +%s.%N)
    local duration=$(echo "$end_time - $start_time" | bc)

    # Check response
    if [ "$http_code" -eq 200 ]; then
        local file_size=$(stat -f%z "$output_file" 2>/dev/null || stat -c%s "$output_file" 2>/dev/null)
        local line_count=$(wc -l < "$output_file")

        echo -e "${GREEN}✓ SUCCESS${NC}"
        echo -e "  HTTP Status: ${http_code}"
        echo -e "  Duration: ${duration}s"
        echo -e "  Output Size: ${file_size} bytes"
        echo -e "  Lines: ${line_count}"
        echo -e "  Saved to: ${output_file}"

        # Show preview of scraped content
        echo -e "\n${BLUE}Content Preview (first 20 lines):${NC}"
        head -n 20 "$output_file" | sed 's/^/  /'

        # Log success
        echo "$name,$url,$http_code,$duration,$file_size,$line_count,SUCCESS" >> "$TEST_LOG"

        return 0
    else
        echo -e "${RED}✗ FAILED${NC}"
        echo -e "  HTTP Status: ${http_code}"
        echo -e "  Duration: ${duration}s"

        # Show error response
        echo -e "\n${RED}Error Response:${NC}"
        cat "$output_file" | sed 's/^/  /'

        # Log failure
        echo "$name,$url,$http_code,$duration,0,0,FAILED" >> "$TEST_LOG"

        return 1
    fi

    echo ""
}

# Wait for server to be ready
echo -e "${YELLOW}Checking if server is ready...${NC}"
for i in {1..30}; do
    if curl -s "${SERVER_URL}/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Server is ready!${NC}"
        echo ""
        break
    fi
    echo -n "."
    sleep 1
done

# Initialize log file with headers
echo "Test Name,URL,HTTP Code,Duration (s),Size (bytes),Lines,Status" > "$TEST_LOG"

# Test suite
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Running Test Suite${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Test 1: Wikipedia (clean, structured content)
test_url "https://en.wikipedia.org/wiki/Web_scraping" "wikipedia"
sleep 2

# Test 2: GitHub (JavaScript-heavy, dynamic content)
test_url "https://github.com/rust-lang/rust" "github"
sleep 2

# Test 3: Hacker News (simple, static content)
test_url "https://news.ycombinator.com/" "hackernews"
sleep 2

# Test 4: Rust Official Site (modern web framework)
test_url "https://www.rust-lang.org/" "rust_official"
sleep 2

# Test 5: MDN Web Docs (technical documentation)
test_url "https://developer.mozilla.org/en-US/docs/Web/HTML" "mdn"
sleep 2

# Summary
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Test Summary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Count results
total_tests=$(tail -n +2 "$TEST_LOG" | wc -l)
successful_tests=$(tail -n +2 "$TEST_LOG" | grep -c "SUCCESS" || echo 0)
failed_tests=$(tail -n +2 "$TEST_LOG" | grep -c "FAILED" || echo 0)

echo -e "Total Tests: ${total_tests}"
echo -e "${GREEN}Successful: ${successful_tests}${NC}"
echo -e "${RED}Failed: ${failed_tests}${NC}"
echo ""

# Calculate average duration for successful tests
if [ "$successful_tests" -gt 0 ]; then
    avg_duration=$(tail -n +2 "$TEST_LOG" | grep "SUCCESS" | awk -F',' '{sum+=$4; count++} END {printf "%.2f", sum/count}')
    total_size=$(tail -n +2 "$TEST_LOG" | grep "SUCCESS" | awk -F',' '{sum+=$5} END {print sum}')

    echo -e "Average Duration: ${avg_duration}s"
    echo -e "Total Data Scraped: ${total_size} bytes ($(echo "scale=2; $total_size/1024/1024" | bc) MB)"
fi

echo ""
echo -e "${BLUE}Results Summary:${NC}"
echo ""
column -t -s ',' "$TEST_LOG"

echo ""
echo -e "${BLUE}All results saved to: ${OUTPUT_DIR}${NC}"
echo -e "${BLUE}Test log: ${TEST_LOG}${NC}"
echo ""

# Exit with appropriate status
if [ "$failed_tests" -gt 0 ]; then
    exit 1
else
    exit 0
fi
