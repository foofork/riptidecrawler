#!/bin/bash

################################################################################
# CLI URL Test Suite - Real-World Validation
################################################################################
# Purpose: Comprehensive testing of eventmesh CLI with diverse real-world URLs
# Usage: ./cli-url-tests.sh [--verbose] [--skip-slow]
################################################################################

set -euo pipefail

# Color codes for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m' # No Color

# Configuration
readonly CLI_BIN="${CLI_BIN:-cargo run --release --}"
readonly TIMEOUT="${TIMEOUT:-30}"
readonly OUTPUT_DIR="${OUTPUT_DIR:-/tmp/cli-url-tests}"
readonly RESULTS_FILE="${OUTPUT_DIR}/test-results.json"
readonly LOG_FILE="${OUTPUT_DIR}/test.log"

# Counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Flags
VERBOSE=false
SKIP_SLOW=false

################################################################################
# Helper Functions
################################################################################

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $*" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $*" | tee -a "$LOG_FILE"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*" | tee -a "$LOG_FILE"
}

log_skip() {
    echo -e "${YELLOW}[SKIP]${NC} $*" | tee -a "$LOG_FILE"
}

setup_test_env() {
    log_info "Setting up test environment..."
    mkdir -p "$OUTPUT_DIR"
    echo "[]" > "$RESULTS_FILE"
    > "$LOG_FILE"
    log_info "Output directory: $OUTPUT_DIR"
    log_info "CLI binary: $CLI_BIN"
    log_info "Timeout: ${TIMEOUT}s"
}

cleanup_test_env() {
    log_info "Test run complete!"
    log_info "Results saved to: $RESULTS_FILE"
    log_info "Logs saved to: $LOG_FILE"
}

record_result() {
    local test_name="$1"
    local status="$2"
    local duration="$3"
    local details="$4"

    local result=$(cat <<EOF
{
  "test": "$test_name",
  "status": "$status",
  "duration": $duration,
  "details": "$details",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
)

    # Append to results file (simple JSON array approach)
    if [ "$VERBOSE" = true ]; then
        log_info "Result: $result"
    fi
}

################################################################################
# Test Execution Function
################################################################################

run_test() {
    local category="$1"
    local test_name="$2"
    local url="$3"
    shift 3
    local cli_args=("$@")

    ((TOTAL_TESTS++))

    log_info "[$TOTAL_TESTS] Testing: $test_name ($category)"
    log_info "    URL: $url"
    log_info "    Args: ${cli_args[*]}"

    local output_file="${OUTPUT_DIR}/test-${TOTAL_TESTS}.out"
    local start_time=$(date +%s)

    # Run CLI command with timeout
    set +e
    if timeout "$TIMEOUT" $CLI_BIN extract "$url" "${cli_args[@]}" > "$output_file" 2>&1; then
        local exit_code=0
    else
        local exit_code=$?
    fi
    set -e

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    # Validate output
    if [ $exit_code -eq 124 ]; then
        log_error "Test timed out after ${TIMEOUT}s"
        ((FAILED_TESTS++))
        record_result "$test_name" "timeout" "$duration" "Exceeded ${TIMEOUT}s timeout"
        return 1
    elif [ $exit_code -ne 0 ]; then
        log_error "CLI exited with code $exit_code"
        ((FAILED_TESTS++))
        record_result "$test_name" "error" "$duration" "Exit code: $exit_code"
        if [ "$VERBOSE" = true ]; then
            cat "$output_file"
        fi
        return 1
    fi

    # Check output validity
    if [ ! -s "$output_file" ]; then
        log_error "Empty output produced"
        ((FAILED_TESTS++))
        record_result "$test_name" "failed" "$duration" "Empty output"
        return 1
    fi

    # Validate JSON output if -o json was used
    for arg in "${cli_args[@]}"; do
        if [[ "$arg" == "json" ]] && [[ "${cli_args[*]}" =~ "-o" ]]; then
            if ! jq empty "$output_file" 2>/dev/null; then
                log_error "Invalid JSON output"
                ((FAILED_TESTS++))
                record_result "$test_name" "failed" "$duration" "Invalid JSON"
                return 1
            fi
            break
        fi
    done

    log_success "Test passed in ${duration}s"
    ((PASSED_TESTS++))
    record_result "$test_name" "passed" "$duration" "Success"

    if [ "$VERBOSE" = true ]; then
        head -n 20 "$output_file"
    fi

    return 0
}

################################################################################
# Test Categories
################################################################################

test_simple_static_sites() {
    log_info "=== Testing Simple Static Sites ==="

    run_test "static" "Example.com Basic" \
        "https://example.com" \
        --method wasm

    run_test "static" "Example.com with Metadata" \
        "https://example.com" \
        --method wasm --metadata

    run_test "static" "Example.com JSON Output" \
        "https://example.com" \
        --method wasm -o json

    run_test "static" "Wikipedia Main Page" \
        "https://en.wikipedia.org/wiki/Main_Page" \
        --method wasm --metadata

    run_test "static" "Wikipedia Article" \
        "https://en.wikipedia.org/wiki/Rust_(programming_language)" \
        --method wasm --show-confidence -o json
}

test_news_sites() {
    log_info "=== Testing News Sites ==="

    run_test "news" "BBC News Homepage" \
        "https://www.bbc.com/news" \
        --method wasm --metadata

    run_test "news" "Reuters World News" \
        "https://www.reuters.com/world/" \
        --method wasm -o json

    run_test "news" "The Guardian" \
        "https://www.theguardian.com/international" \
        --method wasm --metadata

    if [ "$SKIP_SLOW" = false ]; then
        run_test "news" "CNN Homepage" \
            "https://www.cnn.com" \
            --method wasm --show-confidence
    else
        log_skip "CNN test (slow)"
        ((SKIPPED_TESTS++))
    fi
}

test_ecommerce_sites() {
    log_info "=== Testing E-commerce Sites ==="

    # Note: Using example product pages that are typically stable
    run_test "ecommerce" "Amazon Product Page" \
        "https://www.amazon.com/dp/B08N5WRWNW" \
        --method wasm --metadata -o json

    run_test "ecommerce" "eBay Listing" \
        "https://www.ebay.com/itm/274837394774" \
        --method wasm --show-confidence

    if [ "$SKIP_SLOW" = false ]; then
        run_test "ecommerce" "Etsy Product" \
            "https://www.etsy.com/listing/1234567890" \
            --method wasm --metadata || true
    else
        log_skip "Etsy test (slow)"
        ((SKIPPED_TESTS++))
    fi
}

test_tech_documentation() {
    log_info "=== Testing Technical Documentation ==="

    run_test "techdocs" "Rust Docs - std" \
        "https://doc.rust-lang.org/std/" \
        --method wasm --metadata

    run_test "techdocs" "docs.rs Package" \
        "https://docs.rs/tokio/latest/tokio/" \
        --method wasm -o json

    run_test "techdocs" "MDN JavaScript" \
        "https://developer.mozilla.org/en-US/docs/Web/JavaScript" \
        --method wasm --show-confidence

    run_test "techdocs" "GitHub Docs" \
        "https://docs.github.com/en/get-started" \
        --method wasm --metadata -o json
}

test_spa_dynamic_content() {
    log_info "=== Testing SPAs and Dynamic Content ==="

    run_test "spa" "React Documentation" \
        "https://react.dev/" \
        --method wasm --metadata

    run_test "spa" "Vue.js Guide" \
        "https://vuejs.org/guide/introduction.html" \
        --method wasm -o json

    if [ "$SKIP_SLOW" = false ]; then
        run_test "spa" "Next.js Docs" \
            "https://nextjs.org/docs" \
            --method wasm --show-confidence

        run_test "spa" "Svelte Tutorial" \
            "https://svelte.dev/tutorial" \
            --method wasm --metadata
    else
        log_skip "Next.js and Svelte tests (slow)"
        ((SKIPPED_TESTS+=2))
    fi
}

test_social_media() {
    log_info "=== Testing Social Media ==="

    # Note: These may have restrictions or require authentication
    run_test "social" "GitHub Public Profile" \
        "https://github.com/torvalds" \
        --method wasm --metadata || true

    run_test "social" "Reddit Public Page" \
        "https://www.reddit.com/r/rust/" \
        --method wasm -o json || true

    log_warn "Social media tests may fail due to anti-scraping measures"
}

test_blogs_content() {
    log_info "=== Testing Blogs and Content Sites ==="

    run_test "blog" "Medium Article" \
        "https://medium.com/@example/article" \
        --method wasm --metadata || true

    run_test "blog" "Dev.to Post" \
        "https://dev.to/t/rust" \
        --method wasm -o json

    run_test "blog" "Hacker News" \
        "https://news.ycombinator.com/" \
        --method wasm --show-confidence
}

test_edge_cases() {
    log_info "=== Testing Edge Cases ==="

    run_test "edge" "Very Long URL" \
        "https://example.com/very/long/path/that/goes/on/and/on/to/test/url/handling" \
        --method wasm || true

    run_test "edge" "URL with Query Params" \
        "https://example.com/page?param1=value1&param2=value2&param3=value3" \
        --method wasm --metadata

    run_test "edge" "URL with Fragment" \
        "https://example.com/page#section-heading" \
        --method wasm -o json

    # Test invalid URLs (should fail gracefully)
    log_info "Testing invalid URL handling..."
    set +e
    timeout "$TIMEOUT" $CLI_BIN extract "not-a-valid-url" --method wasm > /dev/null 2>&1
    if [ $? -ne 0 ]; then
        log_success "Invalid URL rejected correctly"
        ((PASSED_TESTS++))
    else
        log_error "Invalid URL not rejected"
        ((FAILED_TESTS++))
    fi
    set -e
    ((TOTAL_TESTS++))
}

################################################################################
# Main Execution
################################################################################

print_summary() {
    echo ""
    echo "========================================================================"
    echo "                         TEST SUMMARY"
    echo "========================================================================"
    echo "Total Tests:   $TOTAL_TESTS"
    echo -e "Passed:        ${GREEN}$PASSED_TESTS${NC}"
    echo -e "Failed:        ${RED}$FAILED_TESTS${NC}"
    echo -e "Skipped:       ${YELLOW}$SKIPPED_TESTS${NC}"

    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "\n${GREEN}✓ All tests passed!${NC}"
        echo "========================================================================"
        return 0
    else
        echo -e "\n${RED}✗ Some tests failed${NC}"
        echo "========================================================================"
        return 1
    fi
}

main() {
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --verbose)
                VERBOSE=true
                shift
                ;;
            --skip-slow)
                SKIP_SLOW=true
                shift
                ;;
            -h|--help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --verbose      Show detailed output for each test"
                echo "  --skip-slow    Skip slow-running tests"
                echo "  -h, --help     Show this help message"
                echo ""
                echo "Environment variables:"
                echo "  CLI_BIN        Path to CLI binary (default: cargo run --release --)"
                echo "  TIMEOUT        Timeout per test in seconds (default: 30)"
                echo "  OUTPUT_DIR     Directory for test outputs (default: /tmp/cli-url-tests)"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done

    # Setup
    setup_test_env

    # Run test suites
    test_simple_static_sites
    test_news_sites
    test_ecommerce_sites
    test_tech_documentation
    test_spa_dynamic_content
    test_social_media
    test_blogs_content
    test_edge_cases

    # Cleanup and summary
    cleanup_test_env
    print_summary
}

# Trap errors and interrupts
trap 'log_error "Script interrupted"; exit 130' INT TERM

# Run main function
main "$@"
