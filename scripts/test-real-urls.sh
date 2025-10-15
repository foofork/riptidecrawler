#!/bin/bash
# Test RipTide with real-world URLs

CLI_BIN="target/x86_64-unknown-linux-gnu/release/riptide"
API_URL="http://localhost:8080"
RESULTS_DIR="test-results/real-world-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$RESULTS_DIR"

echo "🌊 RipTide Real-World URL Testing"
echo "================================="
echo "Results: $RESULTS_DIR"
echo ""

# Test URLs - diverse types
declare -a TEST_CASES=(
    "https://example.com|Simple static HTML"
    "https://www.rust-lang.org/|Rust homepage"
    "https://en.wikipedia.org/wiki/WebAssembly|Wikipedia article"
    "https://news.ycombinator.com/|Hacker News"
    "https://github.com/trending|GitHub trending"
    "https://www.bbc.com/news|BBC News"
    "https://docs.rust-lang.org/book/|Rust Book"
    "https://developer.mozilla.org/en-US/docs/Web/JavaScript|MDN JavaScript"
)

TOTAL=${#TEST_CASES[@]}
PASSED=0
FAILED=0

for i in "${!TEST_CASES[@]}"; do
    IFS='|' read -r url description <<< "${TEST_CASES[$i]}"
    test_num=$((i + 1))

    echo "[$test_num/$TOTAL] Testing: $description"
    echo "  URL: $url"

    # Test via CLI
    echo -n "  CLI: "
    if timeout 30s "$CLI_BIN" extract --url "$url" 2>/dev/null | head -5 > "$RESULTS_DIR/cli-$test_num.txt"; then
        echo "✅"

        # Test via API
        echo -n "  API: "
        if curl -s -X POST "$API_URL/api/v1/extract" \
            -H "Content-Type: application/json" \
            -d "{\"url\": \"$url\"}" \
            -m 30 > "$RESULTS_DIR/api-$test_num.json" 2>/dev/null; then

            # Check if valid JSON
            if jq -e '.content' "$RESULTS_DIR/api-$test_num.json" > /dev/null 2>&1; then
                # Extract metrics
                QUALITY=$(jq -r '.quality_score // 0' "$RESULTS_DIR/api-$test_num.json")
                TIME=$(jq -r '.extraction_time_ms // 0' "$RESULTS_DIR/api-$test_num.json")
                WORDS=$(jq -r '.metadata.word_count // 0' "$RESULTS_DIR/api-$test_num.json")
                STRATEGY=$(jq -r '.strategy_used // "unknown"' "$RESULTS_DIR/api-$test_num.json")

                echo "✅ (quality: $QUALITY, time: ${TIME}ms, words: $WORDS, strategy: $STRATEGY)"
                PASSED=$((PASSED + 1))
            else
                echo "❌ Invalid JSON"
                FAILED=$((FAILED + 1))
            fi
        else
            echo "❌ Request failed"
            FAILED=$((FAILED + 1))
        fi
    else
        echo "❌ CLI failed"
        FAILED=$((FAILED + 1))
    fi
    echo ""
done

# Summary
echo "================================="
echo "📊 Test Summary"
echo "================================="
echo "✅ Passed: $PASSED/$TOTAL"
echo "❌ Failed: $FAILED/$TOTAL"
echo "📈 Success Rate: $(( PASSED * 100 / TOTAL ))%"
echo ""

# Save summary
cat > "$RESULTS_DIR/summary.json" <<EOF
{
  "timestamp": "$(date -Iseconds)",
  "total_tests": $TOTAL,
  "passed": $PASSED,
  "failed": $FAILED,
  "success_rate": $(( PASSED * 100 / TOTAL ))
}
EOF

echo "Results saved to: $RESULTS_DIR"