#!/bin/bash

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "==================================="
echo "RipTide URL Verification"
echo "==================================="
echo ""

TOTAL=0
SUCCESS=0
FAILED=0

# Create results directory and CSV file
mkdir -p eval/results
echo "Suite,Name,URL,HTTP_Code,Status" > eval/results/url_verification.csv

# Function to check URL
check_url() {
    local url=$1
    local name=$2
    local suite=$3

    TOTAL=$((TOTAL + 1))
    printf "%-50s " "$name"

    HTTP_CODE=$(curl -o /dev/null -s -w "%{http_code}" --max-time 10 -L "$url" 2>/dev/null || echo "000")

    if [[ "$HTTP_CODE" =~ ^(200|301|302|303|307|308)$ ]]; then
        echo -e "${GREEN}✓${NC} [$HTTP_CODE]"
        SUCCESS=$((SUCCESS + 1))
        echo "$suite,$name,$url,$HTTP_CODE,SUCCESS" >> eval/results/url_verification.csv
        return 0
    else
        echo -e "${RED}✗${NC} [$HTTP_CODE]"
        FAILED=$((FAILED + 1))
        echo "$suite,$name,$url,$HTTP_CODE,FAILED" >> eval/results/url_verification.csv
        return 1
    fi
}

# Process all suite files
for suite_file in eval/suites/*.yml; do
    if [ -f "$suite_file" ]; then
        suite_name=$(basename "$suite_file" .yml)
        echo ""
        echo -e "${YELLOW}Suite: $suite_name${NC}"
        echo "-----------------------------------"

        # Parse YAML
        while IFS= read -r line; do
            if [[ "$line" =~ ^[[:space:]]*-[[:space:]]*name:[[:space:]]*\"(.+)\"$ ]]; then
                current_name="${BASH_REMATCH[1]}"
            elif [[ "$line" =~ ^[[:space:]]*url:[[:space:]]*\"(.+)\"$ ]]; then
                current_url="${BASH_REMATCH[1]}"
                if [ ! -z "${current_name:-}" ] && [ ! -z "${current_url:-}" ]; then
                    check_url "$current_url" "$current_name" "$suite_name" || true
                    current_name=""
                    current_url=""
                fi
            fi
        done < "$suite_file"
    fi
done

echo ""
echo "==================================="
echo "Summary"
echo "==================================="
echo -e "Total: $TOTAL"
echo -e "Success: ${GREEN}$SUCCESS${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"

if [ $TOTAL -gt 0 ]; then
    SUCCESS_RATE=$((SUCCESS * 100 / TOTAL))
    echo "Success Rate: $SUCCESS_RATE%"
fi

echo ""
echo "Results saved to: eval/results/url_verification.csv"
