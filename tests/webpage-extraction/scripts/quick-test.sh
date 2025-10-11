#!/usr/bin/env bash
set -euo pipefail

# Quick test script for development - tests a subset of URLs quickly

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(cd "$TEST_DIR/../.." && pwd)"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }

# Quick test with just a few URLs and methods
QUICK_URLS=(
    "https://example.com"
    "https://developer.mozilla.org/en-US/docs/Web/JavaScript"
    "https://www.wikipedia.org"
)

QUICK_METHODS=("jina" "playwright")

BINARY="$PROJECT_ROOT/target/debug/eventmesh-cli"

log_info "Quick Test Mode"
log_info "Testing ${#QUICK_URLS[@]} URLs with ${#QUICK_METHODS[@]} methods"

# Build if needed
if [[ ! -f "$BINARY" ]]; then
    log_info "Building debug binary..."
    cd "$PROJECT_ROOT" && cargo build
fi

# Run quick tests
mkdir -p "$TEST_DIR/results" "$TEST_DIR/logs"

SUCCESS=0
TOTAL=0

for url in "${QUICK_URLS[@]}"; do
    for method in "${QUICK_METHODS[@]}"; do
        TOTAL=$((TOTAL + 1))
        TEST_ID="quick-$(echo "$url" | md5sum | cut -c1-8)"

        log_info "Testing: $url [$method]"

        if timeout 15 "$BINARY" extract \
            --method "$method" \
            --url "$url" \
            > "$TEST_DIR/logs/${TEST_ID}_${method}.log" 2>&1; then
            SUCCESS=$((SUCCESS + 1))
            log_success "✓ Passed"
        else
            echo "   ✗ Failed (see logs/${TEST_ID}_${method}.log)"
        fi
    done
done

echo ""
log_success "Quick test complete: $SUCCESS/$TOTAL passed"
echo ""
echo "To run full test suite:"
echo "  cd $TEST_DIR/scripts"
echo "  ./run-all-tests.sh run"
