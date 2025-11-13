#!/bin/bash

# Comprehensive Test Suite for RiptideCrawler
# Tests both local and Docker builds with identical payloads

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test results directory
RESULTS_DIR="/tmp/riptide-tests-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$RESULTS_DIR"

echo "📁 Test results will be saved to: $RESULTS_DIR"

# Function to run a test
run_test() {
    local name=$1
    local endpoint=$2
    local payload=$3
    local output_file=$4

    echo -e "${YELLOW}Running: $name${NC}"

    if curl -X POST "$endpoint" \
        -H "Content-Type: application/json" \
        -d "$payload" \
        --max-time 30 \
        --silent \
        --show-error \
        --output "$output_file" 2>&1; then

        local size=$(stat -f%z "$output_file" 2>/dev/null || stat -c%s "$output_file" 2>/dev/null)
        if [ "$size" -gt 100 ]; then
            echo -e "${GREEN}✅ PASS${NC} - Size: ${size} bytes"
            return 0
        else
            echo -e "${RED}❌ FAIL${NC} - Empty or too small response (${size} bytes)"
            return 1
        fi
    else
        echo -e "${RED}❌ FAIL${NC} - Request failed"
        return 1
    fi
}

# Test 1: Example.com extraction
TEST1_PAYLOAD='{"url": "https://example.com"}'

# Test 2: Wikipedia Rust page
TEST2_PAYLOAD='{"url": "https://en.wikipedia.org/wiki/Rust_(programming_language)"}'

# Test 3: Batch crawl
TEST3_PAYLOAD='{"urls": ["https://example.com", "https://example.org"]}'

# Local tests
echo -e "\n${YELLOW}=== LOCAL BUILD TESTS ===${NC}\n"

run_test "Local: Example.com" \
    "http://localhost:8080/extract" \
    "$TEST1_PAYLOAD" \
    "$RESULTS_DIR/local-example.json"

run_test "Local: Wikipedia Rust" \
    "http://localhost:8080/extract" \
    "$TEST2_PAYLOAD" \
    "$RESULTS_DIR/local-wiki-rust.json"

run_test "Local: Batch crawl" \
    "http://localhost:8080/crawl" \
    "$TEST3_PAYLOAD" \
    "$RESULTS_DIR/local-batch.json"

# Docker tests
echo -e "\n${YELLOW}=== DOCKER BUILD TESTS ===${NC}\n"

run_test "Docker: Example.com" \
    "http://localhost:8080/extract" \
    "$TEST1_PAYLOAD" \
    "$RESULTS_DIR/docker-example.json"

run_test "Docker: Wikipedia Rust" \
    "http://localhost:8080/extract" \
    "$TEST2_PAYLOAD" \
    "$RESULTS_DIR/docker-wiki-rust.json"

run_test "Docker: Batch crawl" \
    "http://localhost:8080/crawl" \
    "$TEST3_PAYLOAD" \
    "$RESULTS_DIR/docker-batch.json"

# Comparison analysis
echo -e "\n${YELLOW}=== COMPARISON ANALYSIS ===${NC}\n"

compare_files() {
    local test_name=$1
    local local_file="$RESULTS_DIR/local-$2"
    local docker_file="$RESULTS_DIR/docker-$2"

    if [ ! -f "$local_file" ] || [ ! -f "$docker_file" ]; then
        echo -e "${RED}❌ $test_name: Missing file(s)${NC}"
        return 1
    fi

    local local_size=$(stat -f%z "$local_file" 2>/dev/null || stat -c%s "$local_file")
    local docker_size=$(stat -f%z "$docker_file" 2>/dev/null || stat -c%s "$docker_file")
    local local_hash=$(md5sum "$local_file" | awk '{print $1}')
    local docker_hash=$(md5sum "$docker_file" | awk '{print $1}')

    echo "📊 $test_name:"
    echo "   Local:  ${local_size} bytes (md5: ${local_hash:0:8}...)"
    echo "   Docker: ${docker_size} bytes (md5: ${docker_hash:0:8}...)"

    if [ "$local_hash" == "$docker_hash" ]; then
        echo -e "   ${GREEN}✅ IDENTICAL${NC}"
    else
        local size_diff=$((docker_size - local_size))
        echo -e "   ${YELLOW}⚠️  DIFFERENT${NC} (size diff: $size_diff bytes)"
    fi
}

compare_files "Example.com" "example.json"
compare_files "Wikipedia Rust" "wiki-rust.json"
compare_files "Batch crawl" "batch.json"

# Content validation
echo -e "\n${YELLOW}=== CONTENT VALIDATION ===${NC}\n"

validate_json() {
    local file=$1
    local name=$2

    if jq empty "$file" 2>/dev/null; then
        local has_content=$(jq -r 'if .content or .results then "yes" else "no" end' "$file")
        local has_markdown=$(jq -r 'if .markdown then "yes" else "no" end' "$file")

        echo "📄 $name:"
        echo "   Valid JSON: ✅"
        echo "   Has content: $has_content"
        echo "   Has markdown: $has_markdown"
    else
        echo -e "📄 $name: ${RED}❌ Invalid JSON${NC}"
    fi
}

for f in "$RESULTS_DIR"/*.json; do
    if [ -f "$f" ]; then
        validate_json "$f" "$(basename "$f")"
    fi
done

# Summary
echo -e "\n${YELLOW}=== TEST SUMMARY ===${NC}\n"
echo "Results directory: $RESULTS_DIR"
echo "Total test files: $(ls -1 "$RESULTS_DIR"/*.json 2>/dev/null | wc -l)"
echo -e "\nTo inspect results:"
echo "  ls -lh $RESULTS_DIR/"
echo "  jq . $RESULTS_DIR/local-example.json | head -50"
