#!/bin/bash
# Comprehensive Extraction Test Suite
# Tests multiple websites with different extraction strategies

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_URLS=(
    "https://www.rust-lang.org/"
    "https://developer.mozilla.org/en-US/"
    "https://en.wikipedia.org/wiki/Web_scraping"
    "https://news.ycombinator.com/"
)

STRATEGIES=("auto" "multi")
INCLUDE_HTML_OPTIONS=("true" "false")

# Results directory
RESULTS_DIR="$(pwd)/test-results"
mkdir -p "$RESULTS_DIR"

RESULTS_FILE="$RESULTS_DIR/extraction_results.json"
SUMMARY_FILE="$RESULTS_DIR/extraction_summary.txt"

# Initialize results array
echo "[" > "$RESULTS_FILE"
first_result=true

# Function to run a single test
run_test() {
    local url="$1"
    local strategy="$2"
    local include_html="$3"

    echo -e "${BLUE}Testing: $url${NC}"
    echo -e "  Strategy: ${YELLOW}$strategy${NC}, Include HTML: ${YELLOW}$include_html${NC}"

    # Build the command - use --direct to avoid API calls
    local cmd="cargo run --bin riptide -- extract '$url' --strategy $strategy --direct"

    # Run the extraction and capture output
    local output
    if ! output=$(eval "$cmd" 2>&1); then
        echo -e "  ${RED}✗ Failed${NC}"
        return 1
    fi

    # Parse the JSON output
    local json_output=$(echo "$output" | tail -1)

    # Validate JSON
    if ! echo "$json_output" | jq empty 2>/dev/null; then
        echo -e "  ${RED}✗ Invalid JSON output${NC}"
        return 1
    fi

    # Extract fields
    local strategy_used=$(echo "$json_output" | jq -r '.strategy_used // "unknown"')
    local page_size=$(echo "$json_output" | jq -r '.metadata.page_size_bytes // 0')
    local extracted_chars=$(echo "$json_output" | jq -r '.content | length')
    local has_raw_html=$(echo "$json_output" | jq 'has("raw_html")')
    local raw_html_size=0

    if [ "$has_raw_html" = "true" ]; then
        raw_html_size=$(echo "$json_output" | jq -r '.raw_html | length')
    fi

    # Calculate coverage
    local coverage=0
    if [ "$page_size" -gt 0 ]; then
        coverage=$(echo "scale=2; ($extracted_chars * 100) / $page_size" | bc)
    fi

    # Validate strategy
    local strategy_match="true"
    if [ "$strategy" != "auto" ] && [ "$strategy_used" != "$strategy" ]; then
        strategy_match="false"
        echo -e "  ${YELLOW}⚠ Strategy mismatch: requested=$strategy, used=$strategy_used${NC}"
    fi

    # Create result object
    local result=$(jq -n \
        --arg url "$url" \
        --arg strategy_requested "$strategy" \
        --arg strategy_used "$strategy_used" \
        --argjson page_size "$page_size" \
        --argjson extracted_chars "$extracted_chars" \
        --arg coverage "$coverage" \
        --argjson has_raw_html "$has_raw_html" \
        --argjson raw_html_size "$raw_html_size" \
        --arg strategy_match "$strategy_match" \
        --arg include_html "$include_html" \
        '{
            url: $url,
            strategy_requested: $strategy_requested,
            strategy_used: $strategy_used,
            page_size_bytes: $page_size,
            extracted_chars: $extracted_chars,
            coverage_percent: $coverage,
            has_raw_html: $has_raw_html,
            raw_html_size: $raw_html_size,
            strategy_match: $strategy_match,
            include_html: $include_html
        }')

    # Append to results file
    if [ "$first_result" = true ]; then
        first_result=false
    else
        echo "," >> "$RESULTS_FILE"
    fi
    echo "$result" >> "$RESULTS_FILE"

    echo -e "  ${GREEN}✓ Success${NC} - Extracted: ${extracted_chars} chars, Coverage: ${coverage}%, Strategy: ${strategy_used}"

    return 0
}

# Main test execution
echo -e "${GREEN}=== Comprehensive Extraction Test Suite ===${NC}\n"

total_tests=0
passed_tests=0
failed_tests=0

# Test each combination
for url in "${TEST_URLS[@]}"; do
    for strategy in "${STRATEGIES[@]}"; do
        for include_html in "${INCLUDE_HTML_OPTIONS[@]}"; do
            total_tests=$((total_tests + 1))

            if run_test "$url" "$strategy" "$include_html"; then
                passed_tests=$((passed_tests + 1))
            else
                failed_tests=$((failed_tests + 1))
            fi

            echo "" # Blank line between tests

            # Small delay to avoid rate limiting
            sleep 1
        done
    done
done

# Close JSON array
echo "" >> "$RESULTS_FILE"
echo "]" >> "$RESULTS_FILE"

# Generate summary
echo -e "${GREEN}=== Test Summary ===${NC}" | tee "$SUMMARY_FILE"
echo "" | tee -a "$SUMMARY_FILE"
echo "Total Tests: $total_tests" | tee -a "$SUMMARY_FILE"
echo "Passed: $passed_tests" | tee -a "$SUMMARY_FILE"
echo "Failed: $failed_tests" | tee -a "$SUMMARY_FILE"
echo "" | tee -a "$SUMMARY_FILE"

# Analyze results
if [ -f "$RESULTS_FILE" ]; then
    echo -e "${BLUE}=== Extraction Quality Analysis ===${NC}" | tee -a "$SUMMARY_FILE"
    echo "" | tee -a "$SUMMARY_FILE"

    # Average coverage by strategy
    echo "Average Coverage by Strategy:" | tee -a "$SUMMARY_FILE"
    for strategy in "${STRATEGIES[@]}"; do
        avg_coverage=$(jq -r --arg strategy "$strategy" \
            '[.[] | select(.strategy_requested == $strategy) | .coverage_percent | tonumber] | add / length' \
            "$RESULTS_FILE" 2>/dev/null || echo "0")
        echo "  $strategy: ${avg_coverage}%" | tee -a "$SUMMARY_FILE"
    done
    echo "" | tee -a "$SUMMARY_FILE"

    # Average extraction by URL
    echo "Average Extraction by Site:" | tee -a "$SUMMARY_FILE"
    for url in "${TEST_URLS[@]}"; do
        site_name=$(echo "$url" | sed 's|https://||' | sed 's|/.*||')
        avg_chars=$(jq -r --arg url "$url" \
            '[.[] | select(.url == $url) | .extracted_chars] | add / length' \
            "$RESULTS_FILE" 2>/dev/null || echo "0")
        echo "  $site_name: ${avg_chars} chars (avg)" | tee -a "$SUMMARY_FILE"
    done
    echo "" | tee -a "$SUMMARY_FILE"

    # Strategy validation
    echo "Strategy Validation:" | tee -a "$SUMMARY_FILE"
    mismatches=$(jq -r '[.[] | select(.strategy_match == "false")] | length' "$RESULTS_FILE")
    if [ "$mismatches" -eq 0 ]; then
        echo -e "  ${GREEN}✓ All strategies matched as expected${NC}" | tee -a "$SUMMARY_FILE"
    else
        echo -e "  ${RED}✗ $mismatches strategy mismatches found${NC}" | tee -a "$SUMMARY_FILE"
        jq -r '.[] | select(.strategy_match == "false") | "    \(.url): requested=\(.strategy_requested), used=\(.strategy_used)"' \
            "$RESULTS_FILE" | tee -a "$SUMMARY_FILE"
    fi
    echo "" | tee -a "$SUMMARY_FILE"

    # HTML inclusion validation
    echo "HTML Inclusion Validation:" | tee -a "$SUMMARY_FILE"
    with_html=$(jq -r '[.[] | select(.include_html == "true")] | length' "$RESULTS_FILE")
    has_html=$(jq -r '[.[] | select(.has_raw_html == true)] | length' "$RESULTS_FILE")
    echo "  Requested with HTML: $with_html" | tee -a "$SUMMARY_FILE"
    echo "  Actually has HTML: $has_html" | tee -a "$SUMMARY_FILE"
    if [ "$with_html" -eq "$has_html" ]; then
        echo -e "  ${GREEN}✓ HTML inclusion working correctly${NC}" | tee -a "$SUMMARY_FILE"
    else
        echo -e "  ${YELLOW}⚠ HTML inclusion mismatch${NC}" | tee -a "$SUMMARY_FILE"
    fi
    echo "" | tee -a "$SUMMARY_FILE"
fi

# Store results in memory coordination
if command -v npx &> /dev/null; then
    echo -e "${BLUE}Storing results in memory coordination...${NC}"

    # Create summary for memory
    memory_summary=$(jq -n \
        --argjson total "$total_tests" \
        --argjson passed "$passed_tests" \
        --argjson failed "$failed_tests" \
        --arg timestamp "$(date -Iseconds)" \
        --slurpfile results "$RESULTS_FILE" \
        '{
            timestamp: $timestamp,
            total_tests: $total,
            passed_tests: $passed,
            failed_tests: $failed,
            results: $results[0]
        }')

    # Store in memory
    npx claude-flow@alpha hooks memory-store \
        --key "swarm/tester/extraction-results" \
        --value "$memory_summary" \
        --namespace "coordination" 2>/dev/null || echo "Note: Memory storage skipped (claude-flow not available)"
fi

echo -e "\n${GREEN}=== Test Complete ===${NC}"
echo "Results saved to: $RESULTS_FILE"
echo "Summary saved to: $SUMMARY_FILE"

if [ "$failed_tests" -gt 0 ]; then
    exit 1
fi

exit 0
