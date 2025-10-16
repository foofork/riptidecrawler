#!/bin/bash

# RipTide Real-World Test Runner
# Executes extraction tests on all URL suites

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SUITES_DIR="eval/suites"
RESULTS_DIR="eval/results/$(date +%Y%m%d_%H%M%S)"
RIPTIDE_BIN="cargo run --bin riptide-cli --"
TIMEOUT=30  # seconds per URL

# Create results directory
mkdir -p "$RESULTS_DIR"

echo "==================================="
echo "RipTide Real-World Test Runner"
echo "==================================="
echo ""
echo "Results will be saved to: $RESULTS_DIR"
echo ""

# Statistics
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Log file
LOG_FILE="$RESULTS_DIR/test_run.log"
SUMMARY_FILE="$RESULTS_DIR/summary.json"

# Function to test a single URL
test_url() {
    local url=$1
    local name=$2
    local type=$3
    local suite=$4

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    echo -ne "  Testing: $name... "

    # Create safe filename for results
    local safe_name=$(echo "$name" | tr ' /:' '_' | tr -d '"')
    local result_file="$RESULTS_DIR/${suite}_${safe_name}.json"

    # Run extraction based on type
    local cmd=""
    case "$type" in
        pdf)
            cmd="$RIPTIDE_BIN pdf extract --file <(curl -sL \"$url\") --tables --out \"$result_file\""
            ;;
        listing|events_listing|venue_listing|aggregator_listing)
            cmd="$RIPTIDE_BIN extract --url \"$url\" --engine raw --strategy auto --output json"
            ;;
        *)
            cmd="$RIPTIDE_BIN extract --url \"$url\" --engine auto --strategy auto --output json"
            ;;
    esac

    # Execute with timeout
    if timeout $TIMEOUT bash -c "$cmd" > "$result_file" 2>> "$LOG_FILE"; then
        # Check if output has content
        if [ -s "$result_file" ] && grep -q '"content"' "$result_file" 2>/dev/null; then
            echo -e "${GREEN}✓${NC}"
            PASSED_TESTS=$((PASSED_TESTS + 1))
            echo "{\"test\": \"$name\", \"suite\": \"$suite\", \"status\": \"passed\", \"url\": \"$url\"}" >> "$SUMMARY_FILE"
        else
            echo -e "${YELLOW}⚠${NC} (no content)"
            FAILED_TESTS=$((FAILED_TESTS + 1))
            echo "{\"test\": \"$name\", \"suite\": \"$suite\", \"status\": \"no_content\", \"url\": \"$url\"}" >> "$SUMMARY_FILE"
        fi
    else
        echo -e "${RED}✗${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        echo "{\"test\": \"$name\", \"suite\": \"$suite\", \"status\": \"failed\", \"url\": \"$url\"}" >> "$SUMMARY_FILE"
    fi
}

# Function to run a test suite
run_suite() {
    local suite_file=$1
    local suite_name=$(basename "$suite_file" .yml)

    echo ""
    echo -e "${BLUE}Suite: $suite_name${NC}"
    echo "-----------------------------------"

    # Parse YAML and run tests
    local current_name=""
    local current_url=""
    local current_type=""

    while IFS= read -r line; do
        if [[ "$line" =~ ^[[:space:]]*-[[:space:]]*name:[[:space:]]*\"(.+)\"$ ]]; then
            current_name="${BASH_REMATCH[1]}"
        elif [[ "$line" =~ ^[[:space:]]*url:[[:space:]]*\"(.+)\"$ ]]; then
            current_url="${BASH_REMATCH[1]}"
        elif [[ "$line" =~ ^[[:space:]]*type:[[:space:]]*(.+)$ ]]; then
            current_type="${BASH_REMATCH[1]}"
            # We have all three, run the test
            if [ ! -z "${current_name:-}" ] && [ ! -z "${current_url:-}" ] && [ ! -z "${current_type:-}" ]; then
                test_url "$current_url" "$current_name" "$current_type" "$suite_name"
                current_name=""
                current_url=""
                current_type=""
            fi
        fi
    done < "$suite_file"
}

# Initialize summary file
echo "[" > "$SUMMARY_FILE"

# Check if riptide CLI is available
echo "Checking RipTide CLI availability..."
if ! $RIPTIDE_BIN --version &>/dev/null; then
    echo -e "${RED}Error: RipTide CLI not found or not working${NC}"
    echo "Please ensure the CLI is built and accessible"
    exit 1
fi
echo -e "${GREEN}✓${NC} RipTide CLI is available"

# Process all suite files
for suite_file in $SUITES_DIR/*.yml; do
    if [ -f "$suite_file" ]; then
        run_suite "$suite_file"
    fi
done

# Close summary JSON
echo "]" >> "$SUMMARY_FILE.tmp"
# Fix JSON format (remove last comma and close array)
sed '$ s/,$//' "$SUMMARY_FILE" > "$SUMMARY_FILE.tmp" && echo "]" >> "$SUMMARY_FILE.tmp" && mv "$SUMMARY_FILE.tmp" "$SUMMARY_FILE"

# Generate HTML report
cat > "$RESULTS_DIR/report.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>RipTide Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        h1 { color: #333; }
        .summary { background: #f0f0f0; padding: 15px; border-radius: 5px; margin: 20px 0; }
        .passed { color: green; }
        .failed { color: red; }
        .no_content { color: orange; }
        table { width: 100%; border-collapse: collapse; }
        th, td { padding: 10px; text-align: left; border-bottom: 1px solid #ddd; }
        th { background-color: #4CAF50; color: white; }
    </style>
</head>
<body>
    <h1>RipTide Real-World Test Report</h1>
    <div class="summary">
        <h2>Summary</h2>
        <p>Total Tests: TOTAL_PLACEHOLDER</p>
        <p class="passed">Passed: PASSED_PLACEHOLDER</p>
        <p class="failed">Failed: FAILED_PLACEHOLDER</p>
        <p>Success Rate: RATE_PLACEHOLDER%</p>
    </div>
</body>
</html>
EOF

# Update HTML with actual values
SUCCESS_RATE=$((PASSED_TESTS * 100 / TOTAL_TESTS))
sed -i "s/TOTAL_PLACEHOLDER/$TOTAL_TESTS/g" "$RESULTS_DIR/report.html"
sed -i "s/PASSED_PLACEHOLDER/$PASSED_TESTS/g" "$RESULTS_DIR/report.html"
sed -i "s/FAILED_PLACEHOLDER/$FAILED_TESTS/g" "$RESULTS_DIR/report.html"
sed -i "s/RATE_PLACEHOLDER/$SUCCESS_RATE/g" "$RESULTS_DIR/report.html"

# Print summary
echo ""
echo "==================================="
echo "Test Execution Summary"
echo "==================================="
echo -e "Total Tests:  $TOTAL_TESTS"
echo -e "Passed:       ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed:       ${RED}$FAILED_TESTS${NC}"
echo -e "Skipped:      ${YELLOW}$SKIPPED_TESTS${NC}"
echo -e "Success Rate: $SUCCESS_RATE%"
echo ""
echo "Results saved to: $RESULTS_DIR"
echo "View report at:   $RESULTS_DIR/report.html"

# Exit with error if any tests failed
if [ $FAILED_TESTS -gt 0 ]; then
    exit 1
fi