#!/bin/bash
# RipTide CLI Command Test Script
# Tests all CLI commands with spider-chrome integration
# Created: 2025-10-20

set -e  # Exit on error
set -u  # Exit on undefined variable

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Test output directory
TEST_OUTPUT_DIR="/tmp/riptide-cli-tests"
mkdir -p "$TEST_OUTPUT_DIR"

# Logging function
log() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((TESTS_PASSED++))
}

failure() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((TESTS_FAILED++))
}

warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Run a test command
run_test() {
    local test_name="$1"
    local command="$2"
    local expect_success="${3:-true}"

    ((TESTS_RUN++))
    log "Running test: $test_name"

    local output_file="${TEST_OUTPUT_DIR}/${test_name// /_}.log"

    if eval "$command" > "$output_file" 2>&1; then
        if [ "$expect_success" = "true" ]; then
            success "$test_name"
            return 0
        else
            failure "$test_name - Expected failure but succeeded"
            return 1
        fi
    else
        if [ "$expect_success" = "false" ]; then
            success "$test_name - Failed as expected"
            return 0
        else
            failure "$test_name - Exit code: $?"
            cat "$output_file"
            return 1
        fi
    fi
}

# Check if cargo is available
check_dependencies() {
    log "Checking dependencies..."

    if ! command -v cargo &> /dev/null; then
        failure "cargo not found. Please install Rust toolchain."
        exit 1
    fi

    if ! command -v curl &> /dev/null; then
        warning "curl not found. Some tests may be skipped."
    fi

    success "Dependencies check passed"
}

# Build the CLI
build_cli() {
    log "Building riptide CLI..."
    if cargo build --bin riptide 2>&1 | tee "${TEST_OUTPUT_DIR}/build.log"; then
        success "Build completed"
    else
        failure "Build failed"
        cat "${TEST_OUTPUT_DIR}/build.log"
        exit 1
    fi
}

# Test 1: Help command
test_help() {
    log "=== Testing Help Commands ==="
    run_test "Help: Main help" "cargo run --bin riptide -- --help"
    run_test "Help: Extract help" "cargo run --bin riptide -- extract --help"
    run_test "Help: Crawl help" "cargo run --bin riptide -- crawl --help"
    run_test "Help: Render help" "cargo run --bin riptide -- render --help"
}

# Test 2: Health check
test_health() {
    log "=== Testing Health Check ==="
    run_test "Health: Check API health" "cargo run --bin riptide -- --direct health" "false"
}

# Test 3: Extract command (direct mode)
test_extract() {
    log "=== Testing Extract Command ==="

    # Create test HTML file
    cat > "${TEST_OUTPUT_DIR}/test.html" <<EOF
<!DOCTYPE html>
<html>
<head><title>Test Page</title></head>
<body>
    <h1>Test Heading</h1>
    <p>Test paragraph content.</p>
    <div class="content">Main content here</div>
</body>
</html>
EOF

    run_test "Extract: From file" \
        "cargo run --bin riptide -- --direct extract --input-file ${TEST_OUTPUT_DIR}/test.html --file ${TEST_OUTPUT_DIR}/extract-output.json"

    run_test "Extract: With CSS selector" \
        "cargo run --bin riptide -- --direct extract --input-file ${TEST_OUTPUT_DIR}/test.html --selector 'h1' --file ${TEST_OUTPUT_DIR}/extract-selector.json"

    run_test "Extract: With metadata" \
        "cargo run --bin riptide -- --direct extract --input-file ${TEST_OUTPUT_DIR}/test.html --metadata --file ${TEST_OUTPUT_DIR}/extract-metadata.json"

    run_test "Extract: Invalid file" \
        "cargo run --bin riptide -- --direct extract --input-file /nonexistent/file.html" "false"
}

# Test 4: WASM commands
test_wasm() {
    log "=== Testing WASM Commands ==="
    run_test "WASM: Info" "cargo run --bin riptide -- --direct wasm info"
    run_test "WASM: Health" "cargo run --bin riptide -- --direct wasm health"
}

# Test 5: Cache commands
test_cache() {
    log "=== Testing Cache Commands ==="
    run_test "Cache: Status" "cargo run --bin riptide -- --direct cache status"
    run_test "Cache: Stats" "cargo run --bin riptide -- --direct cache stats"
    run_test "Cache: Validate" "cargo run --bin riptide -- --direct cache validate"
}

# Test 6: Stealth commands
test_stealth() {
    log "=== Testing Stealth Commands ==="
    run_test "Stealth: Info" "cargo run --bin riptide -- --direct stealth info"

    run_test "Stealth: Configure low" \
        "cargo run --bin riptide -- --direct stealth configure --preset low --output ${TEST_OUTPUT_DIR}/stealth-low.json"

    run_test "Stealth: Generate medium" \
        "cargo run --bin riptide -- --direct stealth generate --level medium --output ${TEST_OUTPUT_DIR}/stealth-inject.js"
}

# Test 7: System check
test_system_check() {
    log "=== Testing System Check ==="
    run_test "SystemCheck: Basic check" "cargo run --bin riptide -- --direct system-check"
    run_test "SystemCheck: Verbose" "cargo run --bin riptide -- --direct system-check --verbose"
}

# Test 8: Validate command
test_validate() {
    log "=== Testing Validate Command ==="
    run_test "Validate: Config validation" "cargo run --bin riptide -- --direct validate"
}

# Test 9: Tables extraction
test_tables() {
    log "=== Testing Tables Extraction ==="

    # Create test HTML with table
    cat > "${TEST_OUTPUT_DIR}/test-table.html" <<EOF
<!DOCTYPE html>
<html>
<body>
    <table>
        <thead>
            <tr><th>Name</th><th>Age</th></tr>
        </thead>
        <tbody>
            <tr><td>Alice</td><td>30</td></tr>
            <tr><td>Bob</td><td>25</td></tr>
        </tbody>
    </table>
</body>
</html>
EOF

    run_test "Tables: Extract as JSON" \
        "cargo run --bin riptide -- --direct tables --file ${TEST_OUTPUT_DIR}/test-table.html --format json --output ${TEST_OUTPUT_DIR}/table-output.json"

    run_test "Tables: Extract as CSV" \
        "cargo run --bin riptide -- --direct tables --file ${TEST_OUTPUT_DIR}/test-table.html --format csv --output ${TEST_OUTPUT_DIR}/table-output.csv"

    run_test "Tables: Extract as Markdown" \
        "cargo run --bin riptide -- --direct tables --file ${TEST_OUTPUT_DIR}/test-table.html --format markdown --output ${TEST_OUTPUT_DIR}/table-output.md"
}

# Test 10: Session commands
test_session() {
    log "=== Testing Session Commands ==="
    run_test "Session: List sessions" "cargo run --bin riptide -- --direct session list"
}

# Test 11: Job-local commands
test_job_local() {
    log "=== Testing Job-Local Commands ==="
    run_test "JobLocal: List jobs" "cargo run --bin riptide -- --direct job-local list"
}

# Test 12: Error handling
test_error_handling() {
    log "=== Testing Error Handling ==="

    run_test "Error: Invalid command" "cargo run --bin riptide -- invalid-command" "false"
    run_test "Error: Missing required argument" "cargo run --bin riptide -- extract" "false"
    run_test "Error: Invalid URL format" "cargo run --bin riptide -- --direct extract --url 'not-a-url'" "false"
}

# Test 13: Different output formats
test_output_formats() {
    log "=== Testing Output Formats ==="

    run_test "Output: JSON format" \
        "cargo run --bin riptide -- --direct --output json extract --input-file ${TEST_OUTPUT_DIR}/test.html"

    run_test "Output: Text format" \
        "cargo run --bin riptide -- --direct --output text extract --input-file ${TEST_OUTPUT_DIR}/test.html"
}

# Test 14: Extraction engines
test_extraction_engines() {
    log "=== Testing Extraction Engines ==="

    run_test "Engine: Auto engine" \
        "cargo run --bin riptide -- --direct extract --input-file ${TEST_OUTPUT_DIR}/test.html --engine auto"

    run_test "Engine: Raw engine" \
        "cargo run --bin riptide -- --direct extract --input-file ${TEST_OUTPUT_DIR}/test.html --engine raw"

    run_test "Engine: WASM engine" \
        "cargo run --bin riptide -- --direct extract --input-file ${TEST_OUTPUT_DIR}/test.html --engine wasm"
}

# Test 15: Extraction methods
test_extraction_methods() {
    log "=== Testing Extraction Methods ==="

    run_test "Method: Auto method" \
        "cargo run --bin riptide -- --direct extract --input-file ${TEST_OUTPUT_DIR}/test.html --method auto"

    run_test "Method: CSS method" \
        "cargo run --bin riptide -- --direct extract --input-file ${TEST_OUTPUT_DIR}/test.html --method css"

    run_test "Method: WASM method" \
        "cargo run --bin riptide -- --direct extract --input-file ${TEST_OUTPUT_DIR}/test.html --method wasm"
}

# Test 16: Version check
test_version() {
    log "=== Testing Version ==="
    run_test "Version: Check version" "cargo run --bin riptide -- --version"
}

# Test 17: Metrics commands
test_metrics() {
    log "=== Testing Metrics Commands ==="
    run_test "Metrics: Show metrics" "cargo run --bin riptide -- --direct metrics show" "false"
}

# Test 18: Schema commands
test_schema() {
    log "=== Testing Schema Commands ==="
    run_test "Schema: List schemas" "cargo run --bin riptide -- --direct schema list" "false"
}

# Generate summary report
generate_report() {
    log "=== Test Summary ==="
    echo ""
    echo "================================"
    echo "  RipTide CLI Test Results"
    echo "================================"
    echo "Total Tests Run:    $TESTS_RUN"
    echo -e "Tests Passed:       ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Tests Failed:       ${RED}$TESTS_FAILED${NC}"
    echo "================================"

    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}✓ All tests passed!${NC}"
        return 0
    else
        echo -e "${RED}✗ Some tests failed${NC}"
        echo ""
        echo "Check logs in: $TEST_OUTPUT_DIR"
        return 1
    fi
}

# Main execution
main() {
    log "Starting RipTide CLI Test Suite"
    log "Test output directory: $TEST_OUTPUT_DIR"
    echo ""

    check_dependencies
    build_cli

    echo ""

    # Run all test suites
    test_version
    test_help
    test_health
    test_extract
    test_wasm
    test_cache
    test_stealth
    test_system_check
    test_validate
    test_tables
    test_session
    test_job_local
    test_error_handling
    test_output_formats
    test_extraction_engines
    test_extraction_methods
    test_metrics
    test_schema

    echo ""
    generate_report
}

# Run main function
main "$@"
