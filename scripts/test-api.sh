#!/bin/bash
# Quick API testing script for development mode

echo "🧪 Testing RipTide API Endpoints..."
echo ""

# Test 1: Health Check
echo "1️⃣  Health Check:"
curl -s http://localhost:8080/api/v1/health | jq -r '.status'
echo ""

# Test 2: Tables Extraction
echo "2️⃣  Table Extraction:"
curl -s -X POST http://localhost:8080/api/v1/tables/extract \
  -H "Content-Type: application/json" \
  -d '{"html_content":"<table><tr><th>Product</th><th>Price</th></tr><tr><td>Widget</td><td>$10</td></tr></table>"}' \
  | jq -r '"Extracted " + (.total_tables|tostring) + " tables"'
echo ""

# Test 3: Search
echo "3️⃣  Search API:"
curl -s "http://localhost:8080/api/v1/search?q=rust+programming" \
  | jq -r '"Found " + (.total_results|tostring) + " results (provider: " + .provider_used + ")"'
echo ""

# Test 4: Authentication Bypass Verification
echo "4️⃣  Auth Bypass Test (should succeed without API key):"
if curl -s http://localhost:8080/api/v1/tables/extract \
  -H "Content-Type: application/json" \
  -d '{"html_content":"<table><tr><td>Test</td></tr></table>"}' \
  | grep -q "total_tables"; then
    echo "✅ Authentication bypass working - no API key required"
else
    echo "❌ Authentication bypass failed - API key might be required"
fi
echo ""

echo "✅ All tests completed!"
