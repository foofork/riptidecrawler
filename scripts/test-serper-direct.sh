#!/bin/bash

# Direct Serper API test to verify the API key works

source /workspaces/riptide/.env

if [ -z "$SERPER_API_KEY" ]; then
    echo "Error: SERPER_API_KEY not set"
    exit 1
fi

echo "Testing Serper API directly with key: ${SERPER_API_KEY:0:10}..."
echo ""

# Test Serper search API
response=$(curl -s -X POST https://google.serper.dev/search \
    -H "X-API-KEY: $SERPER_API_KEY" \
    -H "Content-Type: application/json" \
    -d '{
        "q": "Rust programming language",
        "num": 3
    }')

# Check if response is valid
if echo "$response" | jq '.' >/dev/null 2>&1; then
    echo "✅ Serper API key is valid and working!"
    echo ""
    echo "Search results:"
    echo "$response" | jq '.organic[:2] | .[] | {title: .title, link: .link, snippet: .snippet}'
else
    echo "❌ Serper API request failed"
    echo "Response: $response"
fi