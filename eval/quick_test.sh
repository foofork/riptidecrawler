#!/bin/bash

# Quick Test Script - Tests basic URL fetching capabilities
# This script tests what we can without the full CLI being operational

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "==================================="
echo "RipTide Quick URL Test"
echo "==================================="
echo ""

# Test using curl to verify URLs are accessible and have expected content

test_url() {
    local url=$1
    local name=$2
    local expected_content=$3

    echo -ne "Testing: $name... "

    # Fetch URL and check for expected content
    if curl -sL --max-time 10 "$url" | grep -q "$expected_content" 2>/dev/null; then
        echo -e "${GREEN}✓${NC} Content found"
        return 0
    else
        echo -e "${RED}✗${NC} Expected content not found"
        return 1
    fi
}

echo -e "${BLUE}Static Documentation Tests${NC}"
echo "-----------------------------------"
test_url "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Introduction" "MDN JavaScript Guide" "JavaScript"
test_url "https://doc.rust-lang.org/stable/book/ch01-01-installation.html" "Rust Book" "Installation"

echo ""
echo -e "${BLUE}News Article Tests${NC}"
echo "-----------------------------------"
test_url "https://www.reuters.com/graphics/WW2-ANNIVERSARY/CHINA-PARADE/zdvxkgybypx/" "Reuters China Tech" "China"
test_url "https://nos.nl/nieuws/tech" "NOS Tech News" "tech"

echo ""
echo -e "${BLUE}Product Page Tests${NC}"
echo "-----------------------------------"
test_url "https://www.coolblue.nl/en/product/947062/samsung-oled-4k-55s95d-2024.html" "Samsung OLED TV" "Samsung"

echo ""
echo -e "${BLUE}Listing Page Tests${NC}"
echo "-----------------------------------"
test_url "https://news.ycombinator.com/" "Hacker News" "Hacker News"
test_url "https://github.com/topics/rust" "GitHub Rust Topics" "Rust"

echo ""
echo -e "${BLUE}Event Page Tests${NC}"
echo "-----------------------------------"
test_url "https://www.livehilversum.com/en/events" "Live Hilversum Events" "events"

echo ""
echo "==================================="
echo -e "${GREEN}Quick tests completed!${NC}"
echo ""
echo "These tests verify that:"
echo "1. All URLs are accessible"
echo "2. Expected content is present"
echo "3. Pages load within timeout"
echo ""
echo "For full extraction testing, run:"
echo "  ./eval/run_tests.sh"
echo ""