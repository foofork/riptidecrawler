#!/bin/bash
# Test RipTide CLI in local mode (without API server)
# This tests the extraction capabilities directly

set -e

CLI_BIN="${CLI_BIN:-target/x86_64-unknown-linux-gnu/release/riptide}"
RESULTS_DIR="${RESULTS_DIR:-test-results/cli-local-tests}"
mkdir -p "$RESULTS_DIR"

echo "🧪 Testing RipTide CLI - Local Mode"
echo "===================================="
echo ""
echo "CLI Binary: $CLI_BIN"
echo "Results: $RESULTS_DIR"
echo ""

# Test URLs - simple and safe
declare -a URLS=(
    "https://example.com"
    "https://en.wikipedia.org/wiki/WebAssembly"
    "https://www.rust-lang.org/"
)

PASSED=0
FAILED=0
TOTAL=${#URLS[@]}

for i in "${!URLS[@]}"; do
    url="${URLS[$i]}"
    test_num=$((i + 1))
    out_file="$RESULTS_DIR/test-$test_num.json"

    echo "[$test_num/$TOTAL] Testing: $url"

    # Try extract command with different options
    if timeout 30s "$CLI_BIN" extract \
        --url "$url" \
        --output json \
        --verbose \
        > "$out_file" 2>&1; then

        # Check if output is valid JSON
        if jq empty "$out_file" 2>/dev/null; then
            echo "  ✅ PASS - Valid JSON output"
            PASSED=$((PASSED + 1))
        else
            echo "  ⚠️  WARN - Invalid JSON"
            cat "$out_file"
            FAILED=$((FAILED + 1))
        fi
    else
        EXIT_CODE=$?
        echo "  ❌ FAIL - Exit code: $EXIT_CODE"
        echo "  Output:"
        cat "$out_file" | head -20
        FAILED=$((FAILED + 1))
    fi
    echo ""
done

echo "===================================="
echo "Results: $PASSED passed, $FAILED failed out of $TOTAL"
echo "===================================="

if [ $FAILED -gt 0 ]; then
    exit 1
fi
