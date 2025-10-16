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
SUITES_DIR="eval/suites"

# Function to check URL
check_url() {
    local url=$1
    local name=$2

    TOTAL=$((TOTAL + 1))

    # Use curl to check if URL is accessible
    if curl -I -s -o /dev/null -w "%{http_code}" --max-time 10 "$url" | grep -q "^[23]"; then
        echo -e "${GREEN}✓${NC} $name"
        SUCCESS=$((SUCCESS + 1))
    else
        echo -e "${RED}✗${NC} $name - $url"
        FAILED=$((FAILED + 1))
    fi
}

# Process all suite files
for suite_file in $SUITES_DIR/*.yml; do
    if [ -f "$suite_file" ]; then
        suite_name=$(basename "$suite_file" .yml)
        echo ""
        echo -e "${YELLOW}Suite: $suite_name${NC}"
        echo "-----------------------------------"

        # Extract URLs and names from YAML (simple parsing)
        while IFS= read -r line; do
            if [[ "$line" =~ ^[[:space:]]*-[[:space:]]*name:[[:space:]]*\"(.+)\"$ ]]; then
                current_name="${BASH_REMATCH[1]}"
            elif [[ "$line" =~ ^[[:space:]]*url:[[:space:]]*\"(.+)\"$ ]]; then
                current_url="${BASH_REMATCH[1]}"
                if [ ! -z "${current_name:-}" ] && [ ! -z "${current_url:-}" ]; then
                    check_url "$current_url" "$current_name"
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

if [ $FAILED -gt 0 ]; then
    echo ""
    echo -e "${RED}Some URLs are not accessible!${NC}"
    exit 1
else
    echo ""
    echo -e "${GREEN}All URLs verified successfully!${NC}"
fi