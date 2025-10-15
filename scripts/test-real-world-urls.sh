#!/bin/bash
# Comprehensive real-world URL testing script for Riptide CLI
# Tests engine selection, extraction strategies, and content types

set -e

# Set API key for authentication
export RIPTIDE_API_KEY="test-key-123"

RIPTIDE_CLI="./target/x86_64-unknown-linux-gnu/release/riptide"
RESULTS_FILE="/tmp/riptide-test-results.json"
LOG_DIR="/tmp/riptide-test-logs"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Create log directory
mkdir -p "$LOG_DIR"

# Initialize results
echo '{
  "test_timestamp": "'$(date -u +%Y-%m-%dT%H:%M:%SZ)'",
  "tests": []
}' > "$RESULTS_FILE"

# Test counter
TEST_NUM=0
PASSED=0
FAILED=0

# Function to run a single test
run_test() {
    local test_name="$1"
    local url="$2"
    local engine="$3"
    local strategy="$4"
    local expect_success="$5"

    TEST_NUM=$((TEST_NUM + 1))
    local log_file="$LOG_DIR/test_${TEST_NUM}.log"

    echo -e "\n${BLUE}[Test $TEST_NUM]${NC} $test_name"
    echo "  URL: $url"
    echo "  Engine: $engine | Strategy: $strategy"

    # Build command
    local cmd="$RIPTIDE_CLI extract --url \"$url\""
    if [ "$engine" != "auto" ]; then
        cmd="$cmd --engine $engine"
    fi
    if [ "$strategy" != "auto" ]; then
        cmd="$cmd --method $strategy"
    fi

    # Run command
    local start_time=$(date +%s)
    if eval "$cmd" > "$log_file" 2>&1; then
        local exit_code=0
    else
        local exit_code=$?
    fi
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    # Check results
    local output=$(cat "$log_file")
    local output_size=$(wc -c < "$log_file")
    local has_content=false

    if [ $output_size -gt 100 ]; then
        has_content=true
    fi

    # Determine test status
    local status="FAILED"
    local reason=""

    if [ $exit_code -eq 0 ] && [ "$has_content" = true ]; then
        status="PASSED"
        PASSED=$((PASSED + 1))
        echo -e "  ${GREEN}✓ PASSED${NC} (${duration}s, ${output_size} bytes)"
    else
        FAILED=$((FAILED + 1))
        if [ $exit_code -ne 0 ]; then
            reason="Exit code: $exit_code"
        elif [ "$has_content" = false ]; then
            reason="No content extracted"
        fi
        echo -e "  ${RED}✗ FAILED${NC} - $reason"
    fi

    # Extract engine info from output
    local engine_used=$(grep -oP 'Using \K\w+(?= engine)' "$log_file" || echo "unknown")

    # Save test result
    local test_result=$(cat <<EOF
{
  "test_number": $TEST_NUM,
  "test_name": "$test_name",
  "url": "$url",
  "engine_requested": "$engine",
  "engine_used": "$engine_used",
  "strategy": "$strategy",
  "status": "$status",
  "exit_code": $exit_code,
  "duration_seconds": $duration,
  "output_size_bytes": $output_size,
  "has_content": $has_content,
  "reason": "$reason"
}
EOF
)

    # Append to results (this is a simplified version)
    echo "$test_result" >> "$LOG_DIR/results.jsonl"
}

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}Riptide CLI Real-World URL Tests${NC}"
echo -e "${BLUE}================================${NC}"

# Test Category 1: Static Content Sites
echo -e "\n${YELLOW}=== Category 1: Static Content ===${NC}"

run_test "Static: Example.com (auto)" \
    "https://example.com" "auto" "auto" true

run_test "Static: Example.com (raw)" \
    "https://example.com" "raw" "auto" true

run_test "Static: Example.com (wasm)" \
    "https://example.com" "wasm" "auto" true

run_test "Static: Rust-lang.org (auto)" \
    "https://www.rust-lang.org" "auto" "auto" true

run_test "Static: Rust-lang.org (css strategy)" \
    "https://www.rust-lang.org" "auto" "css" true

# Test Category 2: News Sites
echo -e "\n${YELLOW}=== Category 2: News Sites ===${NC}"

run_test "News: Hacker News (auto)" \
    "https://news.ycombinator.com" "auto" "auto" true

run_test "News: Hacker News (raw)" \
    "https://news.ycombinator.com" "raw" "auto" true

run_test "News: The Verge (auto)" \
    "https://www.theverge.com" "auto" "auto" true

run_test "News: The Verge (regex strategy)" \
    "https://www.theverge.com" "auto" "regex" true

# Test Category 3: Documentation
echo -e "\n${YELLOW}=== Category 3: Documentation ===${NC}"

run_test "Docs: Rust docs (auto)" \
    "https://doc.rust-lang.org/book/" "auto" "auto" true

run_test "Docs: Rust docs (wasm)" \
    "https://doc.rust-lang.org/book/" "wasm" "auto" true

run_test "Docs: Rust docs (css strategy)" \
    "https://doc.rust-lang.org/book/" "auto" "css" true

# Test Category 4: E-commerce (simplified)
echo -e "\n${YELLOW}=== Category 4: E-commerce ===${NC}"

run_test "E-commerce: Amazon homepage (auto)" \
    "https://www.amazon.com" "auto" "auto" true

run_test "E-commerce: Amazon (raw engine)" \
    "https://www.amazon.com" "raw" "auto" true

# Test Category 5: GitHub
echo -e "\n${YELLOW}=== Category 5: GitHub ===${NC}"

run_test "GitHub: Rust repo (auto)" \
    "https://github.com/rust-lang/rust" "auto" "auto" true

run_test "GitHub: Rust repo (wasm)" \
    "https://github.com/rust-lang/rust" "wasm" "auto" true

run_test "GitHub: Rust repo (css strategy)" \
    "https://github.com/rust-lang/rust" "auto" "css" true

# Test Category 6: Edge Cases
echo -e "\n${YELLOW}=== Category 6: Edge Cases ===${NC}"

run_test "Edge: Invalid URL" \
    "https://this-domain-definitely-does-not-exist-12345.com" "auto" "auto" false

run_test "Edge: HTTP (not HTTPS)" \
    "http://example.com" "auto" "auto" true

# Print summary
echo -e "\n${BLUE}================================${NC}"
echo -e "${BLUE}Test Summary${NC}"
echo -e "${BLUE}================================${NC}"
echo -e "Total Tests: $TEST_NUM"
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo -e "Success Rate: $(echo "scale=2; $PASSED * 100 / $TEST_NUM" | bc)%"

# Generate final summary JSON
cat > "$RESULTS_FILE" <<EOF
{
  "test_timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "total_tests": $TEST_NUM,
  "passed": $PASSED,
  "failed": $FAILED,
  "success_rate": "$(echo "scale=2; $PASSED * 100 / $TEST_NUM" | bc)%",
  "log_directory": "$LOG_DIR",
  "results_file": "$LOG_DIR/results.jsonl"
}
EOF

echo -e "\n${BLUE}Results saved to:${NC}"
echo -e "  Summary: $RESULTS_FILE"
echo -e "  Details: $LOG_DIR/results.jsonl"
echo -e "  Logs: $LOG_DIR/"

# Exit with failure if any tests failed
if [ $FAILED -gt 0 ]; then
    exit 1
fi
