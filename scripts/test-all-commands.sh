#!/bin/bash
# Comprehensive CLI testing script for RipTide

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Test results array
declare -a FAILED_COMMANDS

# Binary location
RIPTIDE_BIN="./target/debug/riptide"

# Temp directory for test files
TEST_DIR="./tests/cli-test-tmp"
mkdir -p "$TEST_DIR"

# Helper functions
print_header() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}\n"
}

print_test() {
    echo -e "${YELLOW}[TEST]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((PASSED_TESTS++))
}

print_failure() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((FAILED_TESTS++))
    FAILED_COMMANDS+=("$1")
}

print_skip() {
    echo -e "${YELLOW}[SKIP]${NC} $1"
    ((SKIPPED_TESTS++))
}

run_test() {
    local test_name="$1"
    local command="$2"
    local expect_fail="${3:-false}"

    ((TOTAL_TESTS++))
    print_test "$test_name"

    if eval "$command" > /dev/null 2>&1; then
        if [ "$expect_fail" = "true" ]; then
            print_failure "$test_name (expected to fail but succeeded)"
        else
            print_success "$test_name"
        fi
    else
        if [ "$expect_fail" = "true" ]; then
            print_success "$test_name (expected failure)"
        else
            print_failure "$test_name"
        fi
    fi
}

run_help_test() {
    local test_name="$1"
    local command="$2"

    ((TOTAL_TESTS++))
    print_test "$test_name"

    if $RIPTIDE_BIN $command --help > /dev/null 2>&1; then
        print_success "$test_name"
    else
        print_failure "$test_name"
    fi
}

# Build the CLI first
print_header "Building RipTide CLI"
if cargo build --package riptide-cli; then
    print_success "Build successful"
else
    echo -e "${RED}Build failed, exiting${NC}"
    exit 1
fi

# Check if binary exists
if [ ! -f "$RIPTIDE_BIN" ]; then
    echo -e "${RED}Binary not found at $RIPTIDE_BIN${NC}"
    exit 1
fi

# ============================================================================
# HELP OUTPUT TESTS
# ============================================================================

print_header "Testing Help Output"

run_help_test "Main help" ""
run_help_test "Extract help" "extract"
run_help_test "Render help" "render"
run_help_test "Crawl help" "crawl"
run_help_test "Search help" "search"
run_help_test "Cache help" "cache"
run_help_test "WASM help" "wasm"
run_help_test "Stealth help" "stealth"
run_help_test "Domain help" "domain"
run_help_test "Health help" "health"
run_help_test "Metrics help" "metrics"
run_help_test "Validate help" "validate"
run_help_test "System check help" "system-check"
run_help_test "Tables help" "tables"
run_help_test "Schema help" "schema"
run_help_test "PDF help" "pdf"
run_help_test "Job help" "job"
run_help_test "Session help" "session"

# ============================================================================
# SCHEMA COMMANDS
# ============================================================================

print_header "Testing Schema Commands"

run_help_test "Schema learn help" "schema learn"
run_help_test "Schema test help" "schema test"
run_help_test "Schema diff help" "schema diff"
run_help_test "Schema push help" "schema push"
run_help_test "Schema list help" "schema list"
run_help_test "Schema show help" "schema show"
run_help_test "Schema rm help" "schema rm"

# ============================================================================
# DOMAIN COMMANDS
# ============================================================================

print_header "Testing Domain Commands"

run_help_test "Domain init help" "domain init"
run_help_test "Domain profile help" "domain profile"
run_help_test "Domain drift help" "domain drift"
run_help_test "Domain list help" "domain list"
run_help_test "Domain show help" "domain show"
run_help_test "Domain export help" "domain export"
run_help_test "Domain import help" "domain import"
run_help_test "Domain rm help" "domain rm"

# ============================================================================
# PDF COMMANDS
# ============================================================================

print_header "Testing PDF Commands"

run_help_test "PDF extract help" "pdf extract"
run_help_test "PDF to-md help" "pdf to-md"
run_help_test "PDF info help" "pdf info"
run_help_test "PDF stream help" "pdf stream"

# ============================================================================
# JOB COMMANDS
# ============================================================================

print_header "Testing Job Commands"

run_help_test "Job submit help" "job submit"
run_help_test "Job list help" "job list"
run_help_test "Job status help" "job status"
run_help_test "Job logs help" "job logs"
run_help_test "Job cancel help" "job cancel"
run_help_test "Job results help" "job results"
run_help_test "Job retry help" "job retry"
run_help_test "Job stats help" "job stats"

# ============================================================================
# SESSION COMMANDS
# ============================================================================

print_header "Testing Session Commands"

run_help_test "Session new help" "session new"
run_help_test "Session list help" "session list"
run_help_test "Session use help" "session use"
run_help_test "Session current help" "session current"
run_help_test "Session export help" "session export"
run_help_test "Session import help" "session import"
run_help_test "Session rm help" "session rm"
run_help_test "Session update help" "session update"
run_help_test "Session add-cookies help" "session add-cookies"
run_help_test "Session add-headers help" "session add-headers"
run_help_test "Session clone help" "session clone"
run_help_test "Session clear help" "session clear"
run_help_test "Session stats help" "session stats"

# ============================================================================
# CACHE COMMANDS
# ============================================================================

print_header "Testing Cache Commands"

run_help_test "Cache status help" "cache status"
run_help_test "Cache clear help" "cache clear"
run_help_test "Cache validate help" "cache validate"
run_help_test "Cache stats help" "cache stats"

# ============================================================================
# WASM COMMANDS
# ============================================================================

print_header "Testing WASM Commands"

run_help_test "WASM info help" "wasm info"
run_help_test "WASM benchmark help" "wasm benchmark"
run_help_test "WASM health help" "wasm health"

# ============================================================================
# STEALTH COMMANDS
# ============================================================================

print_header "Testing Stealth Commands"

run_help_test "Stealth configure help" "stealth configure"
run_help_test "Stealth test help" "stealth test"
run_help_test "Stealth info help" "stealth info"
run_help_test "Stealth generate help" "stealth generate"

# ============================================================================
# METRICS COMMANDS
# ============================================================================

print_header "Testing Metrics Commands"

run_help_test "Metrics show help" "metrics show"
run_help_test "Metrics export help" "metrics export"

# ============================================================================
# EXTRACT WITH FILE/STDIN
# ============================================================================

print_header "Testing Extract with File/Stdin Input"

# Create a test HTML file
cat > "$TEST_DIR/test.html" <<'EOF'
<!DOCTYPE html>
<html>
<head><title>Test Page</title></head>
<body>
    <article>
        <h1>Test Article</h1>
        <p>This is a test paragraph.</p>
    </article>
</body>
</html>
EOF

run_test "Extract from file with --local" \
    "$RIPTIDE_BIN extract --input-file $TEST_DIR/test.html --local --output text"

run_test "Extract from stdin with --local" \
    "cat $TEST_DIR/test.html | $RIPTIDE_BIN extract --stdin --local --output text"

# ============================================================================
# SESSION BASIC OPERATIONS
# ============================================================================

print_header "Testing Session Basic Operations"

SESSION_NAME="test-session-$$"

run_test "Session create" \
    "$RIPTIDE_BIN session new --name $SESSION_NAME"

run_test "Session list" \
    "$RIPTIDE_BIN session list"

run_test "Session current" \
    "$RIPTIDE_BIN session current"

run_test "Session stats" \
    "$RIPTIDE_BIN session stats --session $SESSION_NAME"

run_test "Session remove" \
    "$RIPTIDE_BIN session rm --session $SESSION_NAME"

# ============================================================================
# TABLES EXTRACTION
# ============================================================================

print_header "Testing Tables Extraction"

# Create a test HTML file with tables
cat > "$TEST_DIR/table-test.html" <<'EOF'
<!DOCTYPE html>
<html>
<head><title>Table Test</title></head>
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

run_test "Tables extract from file (markdown)" \
    "$RIPTIDE_BIN tables --file $TEST_DIR/table-test.html --format markdown"

run_test "Tables extract from file (csv)" \
    "$RIPTIDE_BIN tables --file $TEST_DIR/table-test.html --format csv"

run_test "Tables extract from file (json)" \
    "$RIPTIDE_BIN tables --file $TEST_DIR/table-test.html --format json"

run_test "Tables extract from stdin" \
    "cat $TEST_DIR/table-test.html | $RIPTIDE_BIN tables --stdin --format markdown"

# ============================================================================
# METRICS EXPORT
# ============================================================================

print_header "Testing Metrics Export (Note: Requires API server)"

print_skip "Metrics export JSON (requires running API server)"
print_skip "Metrics export Prometheus (requires running API server)"
print_skip "Metrics export CSV (requires running API server)"

# These would need a running server:
# run_test "Metrics export JSON" \
#     "$RIPTIDE_BIN metrics export --format json --output $TEST_DIR/metrics.json"
#
# run_test "Metrics export Prometheus" \
#     "$RIPTIDE_BIN metrics export --format prom --output $TEST_DIR/metrics.prom"
#
# run_test "Metrics export CSV" \
#     "$RIPTIDE_BIN metrics export --format csv --output $TEST_DIR/metrics.csv"

# ============================================================================
# VALIDATION TESTS
# ============================================================================

print_header "Testing Error Handling"

run_test "Extract with no input (should fail)" \
    "$RIPTIDE_BIN extract --local" \
    true

run_test "Extract with multiple inputs (should fail)" \
    "$RIPTIDE_BIN extract --url https://example.com --input-file $TEST_DIR/test.html --local" \
    true

run_test "Session use non-existent (should fail)" \
    "$RIPTIDE_BIN session use --session non-existent-session-$$" \
    true

# ============================================================================
# CLEANUP
# ============================================================================

print_header "Cleanup"

rm -rf "$TEST_DIR"
print_success "Test directory cleaned up"

# ============================================================================
# FINAL REPORT
# ============================================================================

print_header "Test Summary"

echo -e "Total Tests:   ${BLUE}$TOTAL_TESTS${NC}"
echo -e "Passed:        ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed:        ${RED}$FAILED_TESTS${NC}"
echo -e "Skipped:       ${YELLOW}$SKIPPED_TESTS${NC}"
echo ""

if [ $FAILED_TESTS -gt 0 ]; then
    echo -e "${RED}Failed Commands:${NC}"
    for cmd in "${FAILED_COMMANDS[@]}"; do
        echo -e "  ${RED}✗${NC} $cmd"
    done
    echo ""
    exit 1
else
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
fi
