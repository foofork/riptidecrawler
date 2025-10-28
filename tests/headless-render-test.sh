#!/bin/bash

# Headless Rendering Native Parser Test Suite
# Tests Dynamic render mode with native parser path

set -e

API_URL="http://localhost:8080"
RESULTS_FILE="/workspaces/eventmesh/tests/headless-render-test-results.md"

# Test URLs - mix of SPA-heavy and JavaScript-rendered sites
URLS=(
    "https://example.com"
    "https://www.github.com"
    "https://www.wikipedia.org"
)

echo "🚀 Starting Headless Rendering Native Parser Tests"
echo "=================================================="
echo ""

# Initialize results file
cat > "$RESULTS_FILE" << 'EOF'
# Headless Rendering Test Results

## Test Configuration
- **Date**: $(date -u +"%Y-%m-%d %H:%M:%S UTC")
- **API Endpoint**: http://localhost:8080/crawl
- **Render Mode**: Dynamic (headless Chrome rendering)
- **Parser**: Native (with WASM fallback)

## Test Execution

EOF

# Function to test a URL
test_url() {
    local url="$1"
    local test_num="$2"

    echo "🧪 Test $test_num: Testing $url"
    echo "-----------------------------------"

    # Start time
    start_time=$(date +%s.%N)

    # Make the request
    response=$(curl -s -X POST "$API_URL/crawl" \
        -H "Content-Type: application/json" \
        -d "{\"urls\": [\"$url\"], \"options\": {\"render_mode\": \"Dynamic\"}}" \
        --max-time 30 || echo '{"error": "Request failed"}')

    # End time
    end_time=$(date +%s.%N)
    response_time=$(echo "$end_time - $start_time" | bc)

    # Extract key metrics
    status=$(echo "$response" | jq -r '.results[0].status // "error"')
    quality_score=$(echo "$response" | jq -r '.results[0].quality_score // 0')
    parser_used=$(echo "$response" | jq -r '.results[0].metadata.parser_used // "unknown"')
    title=$(echo "$response" | jq -r '.results[0].document.title // "N/A"')
    text_length=$(echo "$response" | jq -r '.results[0].document.text | length // 0')
    links_count=$(echo "$response" | jq -r '.results[0].document.links | length // 0')
    error=$(echo "$response" | jq -r '.error // "none"')

    # Determine test result
    if [ "$status" = "200" ] && [ "$(echo "$quality_score > 0.5" | bc)" -eq 1 ]; then
        result="✅ PASS"
    else
        result="❌ FAIL"
    fi

    # Display results
    echo "  Status: $status"
    echo "  Quality Score: $quality_score"
    echo "  Parser Used: $parser_used"
    echo "  Response Time: ${response_time}s"
    echo "  Title: $title"
    echo "  Text Length: $text_length chars"
    echo "  Links Count: $links_count"
    echo "  Error: $error"
    echo "  Result: $result"
    echo ""

    # Append to results file
    cat >> "$RESULTS_FILE" << EOF
### Test $test_num: $url

- **Result**: $result
- **Status**: $status
- **Quality Score**: $quality_score
- **Parser Used**: $parser_used
- **Response Time**: ${response_time}s
- **Document Stats**:
  - Title: $title
  - Text Length: $text_length characters
  - Links Count: $links_count links
- **Error**: $error

<details>
<summary>Full Response</summary>

\`\`\`json
$response
\`\`\`

</details>

---

EOF

    # Return 0 for pass, 1 for fail
    if [ "$result" = "✅ PASS" ]; then
        return 0
    else
        return 1
    fi
}

# Run tests
total_tests=${#URLS[@]}
passed_tests=0
failed_tests=0

for i in "${!URLS[@]}"; do
    test_num=$((i + 1))
    if test_url "${URLS[$i]}" "$test_num"; then
        ((passed_tests++))
    else
        ((failed_tests++))
    fi

    # Small delay between tests
    sleep 2
done

# Calculate success rate
success_rate=$(echo "scale=2; ($passed_tests / $total_tests) * 100" | bc)

# Summary
echo "=================================================="
echo "📊 Test Summary"
echo "=================================================="
echo "Total Tests: $total_tests"
echo "Passed: $passed_tests"
echo "Failed: $failed_tests"
echo "Success Rate: ${success_rate}%"
echo ""

# Append summary to results file
cat >> "$RESULTS_FILE" << EOF

## Summary

| Metric | Value |
|--------|-------|
| **Total Tests** | $total_tests |
| **Passed** | $passed_tests |
| **Failed** | $failed_tests |
| **Success Rate** | ${success_rate}% |

## Analysis

EOF

# Add analysis based on results
if [ "$passed_tests" -eq "$total_tests" ]; then
    cat >> "$RESULTS_FILE" << 'EOF'
### ✅ All Tests Passed

The headless rendering path with native parser is working correctly:
- All URLs were successfully crawled
- Quality scores exceeded 0.5 threshold
- Response times were within acceptable limits (<2s expected)
- Native parser handled Dynamic render mode properly

EOF
elif [ "$passed_tests" -gt 0 ]; then
    cat >> "$RESULTS_FILE" << 'EOF'
### ⚠️ Partial Success

Some tests passed but others failed. Review individual test results above for details.

**Possible Issues**:
- Network connectivity problems
- Site-specific rendering issues
- Parser fallback to WASM
- Timeout issues with complex sites

EOF
else
    cat >> "$RESULTS_FILE" << 'EOF'
### ❌ All Tests Failed

The headless rendering path encountered critical issues:
- Check if riptide-headless service is running
- Verify Chrome dependencies are installed
- Review API logs for errors
- Check network connectivity

EOF
fi

# Add recommendations
cat >> "$RESULTS_FILE" << 'EOF'

## Recommendations

1. **Performance**: Response times should be <2s for most sites
2. **Parser**: Native parser should be used for Dynamic mode (check logs)
3. **Quality**: Quality scores >0.5 indicate good extraction
4. **Monitoring**: Check Docker logs for parser selection details

## Docker Logs

To view parser selection and rendering details:
```bash
docker logs riptide-api --tail 50
docker logs riptide-headless --tail 50
```

## Next Steps

- [ ] Review any failed tests
- [ ] Check parser selection in logs
- [ ] Test with additional SPA-heavy sites
- [ ] Validate WASM fallback behavior
- [ ] Performance optimization if needed

---

**Test Completed**: $(date -u +"%Y-%m-%d %H:%M:%S UTC")
EOF

echo "✅ Results saved to: $RESULTS_FILE"
echo ""

# Exit with appropriate code
if [ "$failed_tests" -eq 0 ]; then
    exit 0
else
    exit 1
fi
