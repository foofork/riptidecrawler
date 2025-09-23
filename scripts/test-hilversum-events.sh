#!/bin/bash

# Test script for Serper API - Events in Hilversum Netherlands August 2025
# This script fetches 20 URLs for the specified query

source /workspaces/eventmesh/.env

if [ -z "$SERPER_API_KEY" ]; then
    echo "Error: SERPER_API_KEY not set"
    exit 1
fi

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

QUERY="events in hilversum netherlands august 2025"
NUM_RESULTS=20

echo -e "${BLUE}=== Serper API Test: Events in Hilversum Netherlands August 2025 ===${NC}"
echo -e "${YELLOW}Query: \"$QUERY\"${NC}"
echo -e "${YELLOW}Requesting $NUM_RESULTS URLs${NC}"
echo ""

# Call Serper API
response=$(curl -s -X POST https://google.serper.dev/search \
    -H "X-API-KEY: $SERPER_API_KEY" \
    -H "Content-Type: application/json" \
    -d "{
        \"q\": \"$QUERY\",
        \"num\": $NUM_RESULTS,
        \"gl\": \"nl\",
        \"hl\": \"en\"
    }")

# Check if response is valid
if ! echo "$response" | jq '.' >/dev/null 2>&1; then
    echo "❌ Serper API request failed"
    echo "Response: $response"
    exit 1
fi

# Save full response for analysis
echo "$response" > /workspaces/eventmesh/scripts/serper-hilversum-response.json
echo -e "${GREEN}✅ Serper API request successful!${NC}"
echo ""

# Extract and display URLs
echo -e "${BLUE}=== Extracted URLs ===${NC}"
urls=$(echo "$response" | jq -r '.organic[].link' 2>/dev/null)

if [ -z "$urls" ]; then
    echo "No organic results found. Checking for other result types..."
    # Try to get any URLs from the response
    urls=$(echo "$response" | jq -r '.. | .link? // empty' 2>/dev/null | head -20)
fi

count=0
echo "$urls" | while IFS= read -r url; do
    if [ ! -z "$url" ]; then
        count=$((count + 1))
        echo "$count. $url"
    fi
done

# Save URLs to file for batch processing
echo "$urls" > /workspaces/eventmesh/scripts/hilversum-urls.txt

echo ""
echo -e "${BLUE}=== Summary ===${NC}"
url_count=$(echo "$urls" | grep -c "^http")
echo -e "${GREEN}Total URLs extracted: $url_count${NC}"

# Display sample results with metadata
echo ""
echo -e "${BLUE}=== Sample Results (First 3) ===${NC}"
echo "$response" | jq '.organic[:3] | .[] | {
    title: .title,
    link: .link,
    snippet: .snippet,
    date: .date
}' 2>/dev/null || echo "$response" | jq '.organic[:3]' 2>/dev/null

# Create batch crawl request JSON for RipTide API
echo ""
echo -e "${BLUE}=== Creating RipTide Batch Request ===${NC}"

# Create JSON array of URLs
urls_json=$(echo "$urls" | jq -R . | jq -s .)

# Create the crawl request
cat > /workspaces/eventmesh/scripts/hilversum-crawl-request.json <<EOF
{
    "urls": $urls_json,
    "options": {
        "concurrency": 8,
        "cache_mode": "read_through",
        "timeout_ms": 20000
    }
}
EOF

echo -e "${GREEN}✅ Created crawl request: hilversum-crawl-request.json${NC}"
echo ""

# Test with RipTide API if it's running
echo -e "${BLUE}=== Testing RipTide API (if available) ===${NC}"
if curl -s -f http://localhost:8080/healthz > /dev/null 2>&1; then
    echo -e "${GREEN}✅ RipTide API is running${NC}"
    echo "Sending batch crawl request..."

    # Send the crawl request
    crawl_response=$(curl -s -X POST http://localhost:8080/crawl \
        -H "Content-Type: application/json" \
        -d @/workspaces/eventmesh/scripts/hilversum-crawl-request.json)

    if echo "$crawl_response" | jq '.' >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Crawl request processed${NC}"
        echo "$crawl_response" > /workspaces/eventmesh/scripts/hilversum-crawl-response.json

        # Show summary
        success_count=$(echo "$crawl_response" | jq '.results | map(select(.error == null)) | length' 2>/dev/null)
        error_count=$(echo "$crawl_response" | jq '.results | map(select(.error != null)) | length' 2>/dev/null)

        echo "Results: $success_count successful, $error_count errors"
    else
        echo "Crawl request failed: $crawl_response"
    fi
else
    echo -e "${YELLOW}⚠️  RipTide API is not running${NC}"
    echo "To process these URLs with RipTide:"
    echo "1. Start the API: cargo run --release --bin riptide-api"
    echo "2. Run: curl -X POST http://localhost:8080/crawl -H \"Content-Type: application/json\" -d @hilversum-crawl-request.json"
fi

echo ""
echo -e "${BLUE}=== Files Created ===${NC}"
echo "• serper-hilversum-response.json - Full Serper API response"
echo "• hilversum-urls.txt - List of extracted URLs"
echo "• hilversum-crawl-request.json - RipTide batch crawl request"
[ -f /workspaces/eventmesh/scripts/hilversum-crawl-response.json ] && echo "• hilversum-crawl-response.json - RipTide crawl results"

echo ""
echo -e "${GREEN}✅ Test complete!${NC}"