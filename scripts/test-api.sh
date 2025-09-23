#!/bin/bash

# RipTide API Testing Script with Serper Integration
# This script tests the key endpoints with the Serper API

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

API_HOST="${API_HOST:-http://localhost:8080}"

echo -e "${BLUE}=== RipTide API Testing with Serper ===${NC}"
echo -e "${BLUE}API Host: $API_HOST${NC}"
echo ""

# Function to check endpoint
test_endpoint() {
    local method=$1
    local endpoint=$2
    local data=$3
    local description=$4

    echo -e "${YELLOW}Testing: $description${NC}"
    echo -e "Endpoint: $method $endpoint"

    if [ "$method" == "GET" ]; then
        response=$(curl -s -w "\n%{http_code}" "$API_HOST$endpoint" 2>/dev/null)
    else
        response=$(curl -s -w "\n%{http_code}" -X "$method" \
            -H "Content-Type: application/json" \
            -d "$data" \
            "$API_HOST$endpoint" 2>/dev/null)
    fi

    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)

    if [ "$http_code" -eq 200 ] || [ "$http_code" -eq 201 ]; then
        echo -e "${GREEN}✓ Success (HTTP $http_code)${NC}"
        echo "$body" | jq '.' 2>/dev/null || echo "$body"
    else
        echo -e "${RED}✗ Failed (HTTP $http_code)${NC}"
        echo "$body"
    fi
    echo ""
}

# 1. Test Health Check
test_endpoint "GET" "/healthz" "" "Health Check Endpoint"

# 2. Test Metrics (if available)
echo -e "${YELLOW}Testing: Metrics Endpoint${NC}"
echo -e "Endpoint: GET /metrics"
curl -s "$API_HOST/metrics" | head -10
echo -e "${GREEN}✓ Metrics endpoint checked${NC}"
echo ""

# 3. Test Basic Crawl
test_endpoint "POST" "/crawl" \
    '{"urls": ["https://example.com"], "options": {"cache_mode": "bypass"}}' \
    "Basic Crawl Test"

# 4. Test Deep Search with Serper
echo -e "${BLUE}=== Testing Serper Integration ===${NC}"
test_endpoint "POST" "/deepsearch" \
    '{"query": "Rust programming language", "limit": 3, "include_content": false}' \
    "Deep Search (Serper API) - Metadata Only"

# 5. Test Deep Search with Content Extraction
test_endpoint "POST" "/deepsearch" \
    '{"query": "web scraping best practices", "limit": 2, "include_content": true, "crawl_options": {"cache_mode": "read_through"}}' \
    "Deep Search with Content Extraction"

# 6. Test Batch Crawl
test_endpoint "POST" "/crawl" \
    '{
        "urls": [
            "https://httpbin.org/html",
            "https://httpbin.org/json"
        ],
        "options": {
            "concurrency": 2,
            "cache_mode": "read_through"
        }
    }' \
    "Batch Crawl Test"

echo -e "${BLUE}=== Testing Complete ===${NC}"
echo -e "${GREEN}All tests executed. Please review the results above.${NC}"