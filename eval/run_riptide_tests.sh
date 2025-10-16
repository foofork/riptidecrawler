#!/bin/bash

# RipTide CLI Test Runner
# Executes riptide CLI on all test URLs and generates CSV output

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
RESULTS_DIR="eval/results/$(date +%Y%m%d_%H%M%S)"
RIPTIDE_CMD="cargo run --bin riptide --"
TIMEOUT=30

echo "==================================="
echo "RipTide CLI Test Runner"
echo "==================================="
echo ""

# Create results directory
mkdir -p "$RESULTS_DIR"

# Initialize CSV output
CSV_FILE="$RESULTS_DIR/extraction_results.csv"
echo "Suite,Test_Name,URL,Type,Command,Success,Content_Length,Title_Extracted,Time_ms,Error" > "$CSV_FILE"

# Statistics
TOTAL=0
SUCCESS=0
FAILED=0

# Function to run extraction test
run_extraction_test() {
    local suite=$1
    local name=$2
    local url=$3
    local type=$4

    TOTAL=$((TOTAL + 1))

    printf "%-50s " "$name"

    # Determine command based on type
    local cmd=""
    local output_file="$RESULTS_DIR/$(echo "$name" | tr ' /:' '_').json"

    case "$type" in
        pdf)
            # PDF extraction command
            cmd="$RIPTIDE_CMD pdf extract --url \"$url\" --tables --output json"
            ;;
        product)
            # Product page with schema extraction
            cmd="$RIPTIDE_CMD extract --url \"$url\" --engine auto --strategy auto --output json --metadata"
            ;;
        listing|events_listing|venue_listing|aggregator_listing)
            # Listing pages - extract links and summaries
            cmd="$RIPTIDE_CMD extract --url \"$url\" --engine raw --strategy css --output json"
            ;;
        article|docs|reference)
            # Standard content extraction
            cmd="$RIPTIDE_CMD extract --url \"$url\" --engine auto --strategy auto --output json"
            ;;
        *)
            # Default extraction
            cmd="$RIPTIDE_CMD extract --url \"$url\" --engine auto --strategy auto --output json"
            ;;
    esac

    # Run extraction with timeout
    local start_time=$(date +%s%3N)
    local success="false"
    local content_length=0
    local title_extracted="false"
    local error_msg=""

    if timeout $TIMEOUT bash -c "$cmd" > "$output_file" 2>&1; then
        local end_time=$(date +%s%3N)
        local duration=$((end_time - start_time))

        # Check if output has content
        if [ -s "$output_file" ]; then
            # Extract metrics from JSON output
            content_length=$(jq -r '.content.content // .content // "" | length' "$output_file" 2>/dev/null || echo "0")

            if [ "$content_length" -gt 100 ]; then
                success="true"
                SUCCESS=$((SUCCESS + 1))
                echo -e "${GREEN}✓${NC} [${content_length} chars, ${duration}ms]"

                # Check if title was extracted
                local title=$(jq -r '.content.title // .title // ""' "$output_file" 2>/dev/null || echo "")
                if [ ! -z "$title" ]; then
                    title_extracted="true"
                fi
            else
                FAILED=$((FAILED + 1))
                error_msg="No content extracted"
                echo -e "${YELLOW}⚠${NC} [No content]"
            fi
        else
            FAILED=$((FAILED + 1))
            error_msg="Empty output file"
            echo -e "${RED}✗${NC} [Empty output]"
        fi

        # Write to CSV
        echo "$suite,\"$name\",\"$url\",$type,\"${cmd//\"/\\\"}\",\"$success\",$content_length,$title_extracted,$duration,\"$error_msg\"" >> "$CSV_FILE"
    else
        FAILED=$((FAILED + 1))
        error_msg="Command timeout or error"
        echo -e "${RED}✗${NC} [Timeout/Error]"
        echo "$suite,\"$name\",\"$url\",$type,\"${cmd//\"/\\\"}\",false,0,false,0,\"$error_msg\"" >> "$CSV_FILE"
    fi
}

# Process test suites
echo "Checking if riptide CLI is available..."
if ! $RIPTIDE_CMD --help &>/dev/null; then
    echo -e "${YELLOW}Building riptide CLI...${NC}"
    cargo build --bin riptide 2>&1 | tail -5
fi

# Process each suite
for suite_file in eval/suites/*.yml; do
    if [ -f "$suite_file" ]; then
        suite_name=$(basename "$suite_file" .yml)
        echo ""
        echo -e "${BLUE}Suite: $suite_name${NC}"
        echo "-----------------------------------"

        # Parse YAML and run tests
        current_name=""
        current_url=""
        current_type=""

        while IFS= read -r line; do
            if [[ "$line" =~ ^[[:space:]]*-[[:space:]]*name:[[:space:]]*\"(.+)\"$ ]]; then
                current_name="${BASH_REMATCH[1]}"
            elif [[ "$line" =~ ^[[:space:]]*url:[[:space:]]*\"(.+)\"$ ]]; then
                current_url="${BASH_REMATCH[1]}"
            elif [[ "$line" =~ ^[[:space:]]*type:[[:space:]]*(.+)$ ]]; then
                current_type="${BASH_REMATCH[1]}"

                # Run test when we have all three values
                if [ ! -z "${current_name:-}" ] && [ ! -z "${current_url:-}" ] && [ ! -z "${current_type:-}" ]; then
                    run_extraction_test "$suite_name" "$current_name" "$current_url" "$current_type"
                    current_name=""
                    current_url=""
                    current_type=""
                fi
            fi
        done < "$suite_file"
    fi
done

# Generate summary CSV
SUMMARY_CSV="$RESULTS_DIR/summary.csv"
echo "Metric,Value" > "$SUMMARY_CSV"
echo "Total Tests,$TOTAL" >> "$SUMMARY_CSV"
echo "Successful,$SUCCESS" >> "$SUMMARY_CSV"
echo "Failed,$FAILED" >> "$SUMMARY_CSV"
echo "Success Rate,$((SUCCESS * 100 / TOTAL))%" >> "$SUMMARY_CSV"

# Generate per-suite summary
SUITE_SUMMARY="$RESULTS_DIR/suite_summary.csv"
echo "Suite,Total,Success,Failed,Success_Rate" > "$SUITE_SUMMARY"

for suite in 00_static_docs 10_news_articles 20_product_pages 30_listings 40_tables_pdfs 50_events_hilversum_music; do
    suite_total=$(grep "^$suite," "$CSV_FILE" | wc -l)
    suite_success=$(grep "^$suite,.*,\"true\"," "$CSV_FILE" | wc -l)
    suite_failed=$((suite_total - suite_success))

    if [ $suite_total -gt 0 ]; then
        suite_rate=$((suite_success * 100 / suite_total))
        echo "$suite,$suite_total,$suite_success,$suite_failed,$suite_rate%" >> "$SUITE_SUMMARY"
    fi
done

# Print summary
echo ""
echo "==================================="
echo "Test Execution Summary"
echo "==================================="
echo -e "Total Tests:    $TOTAL"
echo -e "Successful:     ${GREEN}$SUCCESS${NC}"
echo -e "Failed:         ${RED}$FAILED${NC}"

if [ $TOTAL -gt 0 ]; then
    SUCCESS_RATE=$((SUCCESS * 100 / TOTAL))
    echo -e "Success Rate:   $SUCCESS_RATE%"
fi

echo ""
echo "Results saved to:"
echo "  - Detailed:     $CSV_FILE"
echo "  - Summary:      $SUMMARY_CSV"
echo "  - Per-Suite:    $SUITE_SUMMARY"
echo "  - JSON Output:  $RESULTS_DIR/*.json"

# Check against spec requirements
echo ""
echo "==================================="
echo "Performance vs Specification"
echo "==================================="

if [ -f "$SUITE_SUMMARY" ]; then
    echo ""
    echo "Target success rates from spec:"
    echo "  - Static docs:  >90%"
    echo "  - News sites:   >85%"
    echo "  - E-commerce:   >70%"
    echo ""
    echo "Actual results:"
    while IFS=, read -r suite total success failed rate; do
        if [ "$suite" != "Suite" ]; then
            case "$suite" in
                "00_static_docs")
                    echo -n "  - Static docs:  $rate"
                    [ "${rate%\%}" -ge 90 ] && echo -e " ${GREEN}✓${NC}" || echo -e " ${RED}✗${NC}"
                    ;;
                "10_news_articles")
                    echo -n "  - News sites:   $rate"
                    [ "${rate%\%}" -ge 85 ] && echo -e " ${GREEN}✓${NC}" || echo -e " ${RED}✗${NC}"
                    ;;
                "20_product_pages")
                    echo -n "  - E-commerce:   $rate"
                    [ "${rate%\%}" -ge 70 ] && echo -e " ${GREEN}✓${NC}" || echo -e " ${RED}✗${NC}"
                    ;;
            esac
        fi
    done < "$SUITE_SUMMARY"
fi

echo ""