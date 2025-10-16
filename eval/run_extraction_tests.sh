#!/bin/bash

# RipTide Extraction Test Runner
# Runs extraction tests on real-world URLs and generates CSV reports

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
RESULTS_DIR="eval/results/extraction_$(date +%Y%m%d_%H%M%S)"
RIPTIDE="./target/x86_64-unknown-linux-gnu/release/riptide"
TIMEOUT=30

echo "==================================="
echo "RipTide Extraction Test Runner"
echo "==================================="
echo ""
echo "Results directory: $RESULTS_DIR"
echo ""

# Create results directory
mkdir -p "$RESULTS_DIR"
mkdir -p "$RESULTS_DIR/json"

# Initialize CSV files
MAIN_CSV="$RESULTS_DIR/extraction_results.csv"
echo "Suite,Test_Name,URL,Type,Success,Content_Length,Title_Present,Text_Extracted,Time_ms,Error_Message" > "$MAIN_CSV"

SUMMARY_CSV="$RESULTS_DIR/summary.csv"
SUITE_CSV="$RESULTS_DIR/suite_performance.csv"
echo "Suite,Total,Success,Failed,Avg_Content_Length,Avg_Time_ms,Success_Rate" > "$SUITE_CSV"

# Statistics
TOTAL=0
SUCCESS=0
FAILED=0

# Function to run a single extraction test
run_test() {
    local suite=$1
    local name=$2
    local url=$3
    local type=$4

    TOTAL=$((TOTAL + 1))

    printf "[%d/%d] Testing: %-40s " "$TOTAL" "26" "${name:0:40}"

    # Create safe filename
    local safe_name=$(echo "$name" | tr ' /:' '_' | tr -d '"')
    local json_file="$RESULTS_DIR/json/${suite}_${safe_name}.json"

    # Start timer
    local start_time=$(date +%s%N)

    # Run extraction based on type
    local cmd=""
    case "$type" in
        pdf)
            cmd="extract --url \"$url\" --engine raw --local"
            ;;
        product)
            cmd="extract --url \"$url\" --engine auto --method css --local"
            ;;
        listing|events_listing|venue_listing|aggregator_listing)
            cmd="extract --url \"$url\" --engine raw --method css --local"
            ;;
        *)
            cmd="extract --url \"$url\" --engine auto --local"
            ;;
    esac

    # Execute extraction
    local success="false"
    local content_length=0
    local title_present="false"
    local text_extracted="false"
    local error_msg=""

    if timeout $TIMEOUT bash -c "$RIPTIDE $cmd" > "$json_file" 2>&1; then
        # Calculate time
        local end_time=$(date +%s%N)
        local duration=$(( (end_time - start_time) / 1000000 ))

        # Check if JSON is valid and extract metrics
        if [ -s "$json_file" ] && jq . "$json_file" >/dev/null 2>&1; then
            # Extract content metrics
            local content=$(jq -r '.content // .text // .markdown // "" | tostring' "$json_file" 2>/dev/null || echo "")
            content_length=${#content}

            # Check for title
            local title=$(jq -r '.title // .content.title // "" | tostring' "$json_file" 2>/dev/null || echo "")
            [ -n "$title" ] && title_present="true"

            # Check if meaningful content was extracted
            if [ "$content_length" -gt 100 ]; then
                success="true"
                text_extracted="true"
                SUCCESS=$((SUCCESS + 1))
                echo -e "${GREEN}✓${NC} [${content_length} chars, ${duration}ms]"
            else
                FAILED=$((FAILED + 1))
                error_msg="Insufficient content"
                echo -e "${YELLOW}⚠${NC} [Low content: ${content_length} chars]"
            fi
        else
            FAILED=$((FAILED + 1))
            error_msg="Invalid JSON output"
            echo -e "${RED}✗${NC} [Invalid JSON]"
        fi
    else
        # Command failed or timed out
        local end_time=$(date +%s%N)
        local duration=$(( (end_time - start_time) / 1000000 ))
        FAILED=$((FAILED + 1))
        error_msg="Command failed/timeout"
        echo -e "${RED}✗${NC} [Failed/Timeout]"
    fi

    # Write to CSV
    echo "$suite,\"$name\",\"$url\",$type,$success,$content_length,$title_present,$text_extracted,$duration,\"$error_msg\"" >> "$MAIN_CSV"
}

# Function to process a suite
process_suite() {
    local suite_file=$1
    local suite_name=$(basename "$suite_file" .yml)

    echo ""
    echo -e "${BLUE}Processing Suite: $suite_name${NC}"
    echo "----------------------------------------"

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

            # Run test when we have all values
            if [ -n "$current_name" ] && [ -n "$current_url" ] && [ -n "$current_type" ]; then
                run_test "$suite_name" "$current_name" "$current_url" "$current_type"
                current_name=""
                current_url=""
                current_type=""
            fi
        fi
    done < "$suite_file"
}

# Main execution
echo "Starting extraction tests..."
echo ""

# Check if riptide is available
echo "Checking RipTide CLI availability..."
if ! $RIPTIDE --version >/dev/null 2>&1; then
    echo -e "${YELLOW}Warning: RipTide CLI may not be fully built yet${NC}"
fi

# Process all test suites
for suite_file in eval/suites/*.yml; do
    if [ -f "$suite_file" ]; then
        process_suite "$suite_file"
    fi
done

# Generate summary statistics
echo ""
echo "==================================="
echo "Generating Summary Statistics"
echo "==================================="

# Overall summary
echo "Metric,Value" > "$SUMMARY_CSV"
echo "Total_Tests,$TOTAL" >> "$SUMMARY_CSV"
echo "Successful,$SUCCESS" >> "$SUMMARY_CSV"
echo "Failed,$FAILED" >> "$SUMMARY_CSV"

if [ $TOTAL -gt 0 ]; then
    SUCCESS_RATE=$((SUCCESS * 100 / TOTAL))
    echo "Success_Rate,$SUCCESS_RATE%" >> "$SUMMARY_CSV"
else
    echo "Success_Rate,0%" >> "$SUMMARY_CSV"
fi

# Per-suite analysis
for suite in 00_static_docs 10_news_articles 20_product_pages 30_listings 40_tables_pdfs 50_events_hilversum_music; do
    suite_lines=$(grep "^$suite," "$MAIN_CSV" 2>/dev/null || echo "")
    if [ -n "$suite_lines" ]; then
        suite_total=$(echo "$suite_lines" | wc -l)
        suite_success=$(echo "$suite_lines" | grep -c ",true," || echo "0")
        suite_failed=$((suite_total - suite_success))

        # Calculate averages
        avg_content=0
        avg_time=0
        if [ $suite_total -gt 0 ]; then
            avg_content=$(echo "$suite_lines" | awk -F',' '{sum+=$6; count++} END {if(count>0) print int(sum/count); else print 0}')
            avg_time=$(echo "$suite_lines" | awk -F',' '{sum+=$9; count++} END {if(count>0) print int(sum/count); else print 0}')
            suite_rate=$((suite_success * 100 / suite_total))
        else
            suite_rate=0
        fi

        echo "$suite,$suite_total,$suite_success,$suite_failed,$avg_content,$avg_time,$suite_rate%" >> "$SUITE_CSV"
    fi
done

# Print final summary
echo ""
echo "==================================="
echo "Test Execution Summary"
echo "==================================="
echo -e "Total Tests:     $TOTAL"
echo -e "Successful:      ${GREEN}$SUCCESS${NC}"
echo -e "Failed:          ${RED}$FAILED${NC}"

if [ $TOTAL -gt 0 ]; then
    echo -e "Success Rate:    $SUCCESS_RATE%"
fi

echo ""
echo "==================================="
echo "Performance vs Specification"
echo "==================================="
echo ""
echo "Target Success Rates:"
echo "  Static docs:   >90%"
echo "  News sites:    >85%"
echo "  E-commerce:    >70%"
echo ""
echo "Actual Results:"

# Check each suite against targets
while IFS=, read -r suite total success failed avg_content avg_time rate; do
    if [ "$suite" != "Suite" ]; then
        case "$suite" in
            "00_static_docs")
                printf "  Static docs:   %s" "$rate"
                rate_num=${rate%\%}
                [ "$rate_num" -ge 90 ] 2>/dev/null && echo -e " ${GREEN}✓ PASS${NC}" || echo -e " ${RED}✗ FAIL${NC}"
                ;;
            "10_news_articles")
                printf "  News sites:    %s" "$rate"
                rate_num=${rate%\%}
                [ "$rate_num" -ge 85 ] 2>/dev/null && echo -e " ${GREEN}✓ PASS${NC}" || echo -e " ${RED}✗ FAIL${NC}"
                ;;
            "20_product_pages")
                printf "  E-commerce:    %s" "$rate"
                rate_num=${rate%\%}
                [ "$rate_num" -ge 70 ] 2>/dev/null && echo -e " ${GREEN}✓ PASS${NC}" || echo -e " ${RED}✗ FAIL${NC}"
                ;;
        esac
    fi
done < "$SUITE_CSV"

echo ""
echo "==================================="
echo "Output Files"
echo "==================================="
echo "Main Results:     $MAIN_CSV"
echo "Summary:          $SUMMARY_CSV"
echo "Suite Analysis:   $SUITE_CSV"
echo "JSON Outputs:     $RESULTS_DIR/json/"
echo ""
echo "Test run complete!"