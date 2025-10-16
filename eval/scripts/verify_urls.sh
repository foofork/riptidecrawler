#!/bin/bash

# URL Verification Script
# Checks that all URLs in test suites are accessible

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "==================================="
echo "RipTide URL Verification"
echo "==================================="
echo ""

TOTAL=0
SUCCESS=0
FAILED=0
SUITES_DIR="suites"

# Function to check URL
check_url() {
    local url=$1
    local name=$2
    local suite=$3

    TOTAL=$((TOTAL + 1))

    printf "%-50s " "$name"

    # Use curl to check if URL is accessible
    HTTP_CODE=$(curl -o /dev/null -s -w "%{http_code}" --max-time 10 -L "$url" 2>/dev/null || echo "000")

    if [[ "$HTTP_CODE" =~ ^(200|301|302|303|307|308)$ ]]; then
        echo -e "${GREEN}✓${NC} [$HTTP_CODE]"
        SUCCESS=$((SUCCESS + 1))
        return 0
    else
        echo -e "${RED}✗${NC} [$HTTP_CODE] - $url"
        FAILED=$((FAILED + 1))
        echo "$suite,$name,$url,$HTTP_CODE,FAILED" >> results/url_verification.csv
        return 1
    fi
}

# Create results directory
mkdir -p results

# Initialize CSV output
echo "Suite,Name,URL,HTTP_Code,Status" > results/url_verification.csv

# Process all suite files
for suite_file in $SUITES_DIR/*.yml; do
    if [ -f "$suite_file" ]; then
        suite_name=$(basename "$suite_file" .yml)
        echo ""
        echo -e "${YELLOW}Suite: $suite_name${NC}"
        echo "-----------------------------------"

        # Parse YAML and extract URLs
        while IFS= read -r line; do
            if [[ "$line" =~ ^[[:space:]]*-[[:space:]]*name:[[:space:]]*\"(.+)\"$ ]]; then
                current_name="${BASH_REMATCH[1]}"
            elif [[ "$line" =~ ^[[:space:]]*url:[[:space:]]*\"(.+)\"$ ]]; then
                current_url="${BASH_REMATCH[1]}"
                if [ ! -z "${current_name:-}" ] && [ ! -z "${current_url:-}" ]; then
                    if check_url "$current_url" "$current_name" "$suite_name"; then
                        echo "$suite_name,$current_name,$current_url,200,SUCCESS" >> results/url_verification.csv
                    fi
                    current_name=""
                    current_url=""
                fi
            fi
        done < "$suite_file"
    fi
done

echo ""
echo "==================================="
echo "Verification Summary"
echo "==================================="
echo -e "Total URLs: $TOTAL"
echo -e "Successful: ${GREEN}$SUCCESS${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"
echo ""

# Calculate success rate
if [ $TOTAL -gt 0 ]; then
    SUCCESS_RATE=$((SUCCESS * 100 / TOTAL))
    echo "Success Rate: $SUCCESS_RATE%"
    echo ""
    echo "Results saved to: results/url_verification.csv"
fi

if [ $FAILED -gt 0 ]; then
    echo ""
    echo -e "${RED}Some URLs are not accessible!${NC}"
    exit 1
else
    echo ""
    echo -e "${GREEN}All URLs verified successfully!${NC}"
fi