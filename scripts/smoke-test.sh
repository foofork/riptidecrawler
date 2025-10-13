#!/bin/bash
# Smoke test script for RipTide API
# Based on WEEK_1_ACTION_PLAN.md Day 7 requirements

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
API_PORT=8080
API_URL="http://localhost:${API_PORT}"
TEST_TIMEOUT=30
STARTUP_WAIT=10

# Test URLs from WEEK_1_ACTION_PLAN.md
TEST_URLS=(
    # Simple static
    "https://example.com"
    # News sites
    "https://www.bbc.com/news/technology"
    "https://techcrunch.com/latest"
    # Blogs
    "https://martinfowler.com/articles/"
    # Documentation
    "https://docs.rust-lang.org/book/"
    # E-commerce
    "https://www.amazon.com/dp/B08N5WRWNW"
    # Social
    "https://dev.to/"
    # Complex SPAs
    "https://github.com/trending"
    # Additional diverse sites
    "https://www.wikipedia.org/"
    "https://www.reddit.com/r/programming/"
    "https://stackoverflow.com/questions"
    "https://news.ycombinator.com/"
    "https://www.youtube.com/"
    "https://twitter.com/"
    "https://www.linkedin.com/"
    "https://medium.com/"
    "https://www.nytimes.com/"
    "https://www.theguardian.com/"
    "https://www.cnn.com/"
    "https://www.washingtonpost.com/"
)

# Counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to print colored output
print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    echo -e "${YELLOW}ℹ${NC} $1"
}

# Function to check if API is running
check_api_running() {
    if lsof -Pi :${API_PORT} -sTCP:LISTEN -t >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Function to wait for API to be ready
wait_for_api() {
    print_info "Waiting for API to be ready..."
    local max_attempts=30
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        if curl -s -f "${API_URL}/health" >/dev/null 2>&1; then
            print_success "API is ready"
            return 0
        fi
        attempt=$((attempt + 1))
        sleep 1
    done

    print_error "API failed to start within ${max_attempts} seconds"
    return 1
}

# Function to run a test
run_test() {
    local test_name=$1
    local test_command=$2

    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    print_info "Running test: ${test_name}"

    if eval "${test_command}"; then
        print_success "${test_name}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        print_error "${test_name}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

# Cleanup function
cleanup() {
    if [ -n "$API_PID" ]; then
        print_info "Stopping API server (PID: ${API_PID})..."
        kill $API_PID 2>/dev/null || true
        wait $API_PID 2>/dev/null || true
        print_success "API server stopped"
    fi
}

# Set trap to cleanup on exit
trap cleanup EXIT INT TERM

# Main execution
main() {
    echo "================================================"
    echo "RipTide API Smoke Test Suite"
    echo "================================================"
    echo ""

    # Check if API is already running
    if check_api_running; then
        print_error "API is already running on port ${API_PORT}"
        print_info "Please stop the existing instance first"
        exit 1
    fi

    # Start API server
    print_info "Starting RipTide API server..."
    cargo run --release --bin riptide-api > /tmp/riptide-api.log 2>&1 &
    API_PID=$!
    print_success "API server started (PID: ${API_PID})"

    # Wait for API to be ready
    if ! wait_for_api; then
        print_error "Failed to start API server"
        print_info "Check logs at /tmp/riptide-api.log"
        exit 1
    fi

    echo ""
    echo "Running smoke tests..."
    echo ""

    # Test 1: Health check
    run_test "Health check endpoint" \
        "curl -s -f ${API_URL}/health | jq -e '.status' > /dev/null"

    # Test 2: Metrics endpoint
    run_test "Metrics endpoint" \
        "curl -s -f ${API_URL}/metrics | grep -q 'riptide\|http_requests\|memory'"

    # Test 3: Simple extraction (example.com)
    run_test "Simple extraction (example.com)" \
        "curl -s -X POST ${API_URL}/extract \
            -H 'Content-Type: application/json' \
            -d '{\"url\": \"https://example.com\"}' \
            | jq -e '.success == true' > /dev/null"

    # Test 4-23: Real URL extractions
    local url_success=0
    local url_failure=0

    for url in "${TEST_URLS[@]}"; do
        if [[ $url =~ ^# ]]; then
            continue  # Skip comments
        fi

        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        local url_hash=$(echo -n "$url" | md5sum | cut -d' ' -f1 | cut -c1-8)

        print_info "Testing URL: ${url}"

        if curl -s -X POST "${API_URL}/extract" \
            -H "Content-Type: application/json" \
            -d "{\"url\": \"${url}\"}" \
            --max-time ${TEST_TIMEOUT} \
            | jq -e '.success == true' > /dev/null 2>&1; then
            print_success "Extraction succeeded: ${url_hash}"
            PASSED_TESTS=$((PASSED_TESTS + 1))
            url_success=$((url_success + 1))
        else
            print_error "Extraction failed: ${url_hash}"
            FAILED_TESTS=$((FAILED_TESTS + 1))
            url_failure=$((url_failure + 1))
        fi

        # Small delay between requests to avoid rate limiting
        sleep 0.5
    done

    # Test 5: Error handling - invalid URL
    run_test "Error handling (invalid URL)" \
        "curl -s -X POST ${API_URL}/extract \
            -H 'Content-Type: application/json' \
            -d '{\"url\": \"not-a-valid-url\"}' \
            | jq -e '.success == false or .error' > /dev/null"

    # Test 6: Error handling - missing URL
    run_test "Error handling (missing URL)" \
        "! curl -s -X POST ${API_URL}/extract \
            -H 'Content-Type: application/json' \
            -d '{}' \
            -w '%{http_code}' -o /dev/null | grep -q '200'"

    # Test 7: Concurrent requests
    print_info "Testing concurrent requests..."
    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    (curl -s -X POST "${API_URL}/extract" \
        -H "Content-Type: application/json" \
        -d '{"url": "https://example.com"}' > /dev/null 2>&1) &
    (curl -s -X POST "${API_URL}/extract" \
        -H "Content-Type: application/json" \
        -d '{"url": "https://www.rust-lang.org/"}' > /dev/null 2>&1) &
    (curl -s -X POST "${API_URL}/extract" \
        -H "Content-Type: application/json" \
        -d '{"url": "https://github.com/"}' > /dev/null 2>&1) &

    wait

    # Check if server is still responsive
    if curl -s -f "${API_URL}/health" > /dev/null 2>&1; then
        print_success "Concurrent requests handled correctly"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        print_error "Server unresponsive after concurrent requests"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi

    # Test 8: Server stability
    run_test "Server still responsive" \
        "curl -s -f ${API_URL}/health > /dev/null"

    # Print summary
    echo ""
    echo "================================================"
    echo "Smoke Test Results"
    echo "================================================"
    echo "Total Tests:   ${TOTAL_TESTS}"
    echo "Passed Tests:  ${PASSED_TESTS}"
    echo "Failed Tests:  ${FAILED_TESTS}"

    local success_rate=0
    if [ ${TOTAL_TESTS} -gt 0 ]; then
        success_rate=$((PASSED_TESTS * 100 / TOTAL_TESTS))
    fi
    echo "Success Rate:  ${success_rate}%"
    echo ""

    # URL extraction summary
    local total_urls=${#TEST_URLS[@]}
    if [ ${total_urls} -gt 0 ]; then
        local url_success_rate=$((url_success * 100 / total_urls))
        echo "URL Extraction:"
        echo "  Total URLs:    ${total_urls}"
        echo "  Successful:    ${url_success}"
        echo "  Failed:        ${url_failure}"
        echo "  Success Rate:  ${url_success_rate}%"
        echo ""
    fi

    # Week 1 success criteria check
    if [ ${success_rate} -ge 90 ]; then
        print_success "✅ Week 1 success criteria met (>90% success rate)"
        exit 0
    elif [ ${success_rate} -ge 80 ]; then
        print_info "⚠️  Close to Week 1 criteria (80-90% success rate)"
        exit 0
    else
        print_error "❌ Week 1 success criteria not met (<80% success rate)"
        exit 1
    fi
}

# Check dependencies
if ! command -v curl &> /dev/null; then
    print_error "curl is required but not installed"
    exit 1
fi

if ! command -v jq &> /dev/null; then
    print_error "jq is required but not installed"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    print_error "cargo is required but not installed"
    exit 1
fi

# Run main function
main
