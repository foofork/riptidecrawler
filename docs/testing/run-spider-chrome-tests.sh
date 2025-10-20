#!/bin/bash
# Spider-Chrome Test Suite Runner
# Runs tests sequentially with proper cleanup and browser resource management

set -e

RESULTS_DIR="docs/testing/results"
mkdir -p "$RESULTS_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

echo "🧪 Spider-Chrome Test Suite"
echo "================================"
echo "Results will be saved to: $RESULTS_DIR/"
echo ""

# Function to run test with timing
run_test() {
    local name="$1"
    local cmd="$2"
    local log_file="$RESULTS_DIR/${name}.log"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    echo -e "${BLUE}▶ Running: $name${NC}"
    start_time=$(date +%s)

    if eval "$cmd" > "$log_file" 2>&1; then
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        echo -e "  ${GREEN}✓ PASS${NC} ($duration seconds)"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        echo -e "  ${RED}✗ FAIL${NC} ($duration seconds)"
        echo -e "  ${YELLOW}See: $log_file${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))

        # Show last 20 lines of error
        echo -e "  ${YELLOW}Last 20 lines of output:${NC}"
        tail -n 20 "$log_file" | sed 's/^/    /'
        echo ""

        return 1
    fi
}

# Cleanup function
cleanup() {
    echo ""
    echo "🧹 Cleaning up browser processes..."
    pkill -f "chromium|chrome" || true
    sleep 2
}

# Trap to ensure cleanup on exit
trap cleanup EXIT

# Phase 1: Type Checking (Fast)
echo -e "${BLUE}📦 Phase 1: Type Checking${NC}"
echo "================================"
run_test "typecheck-browser-abstraction" \
    "cargo check -p riptide-browser-abstraction --all-features"
run_test "typecheck-engine" \
    "cargo check -p riptide-engine --all-features"
run_test "typecheck-headless" \
    "cargo check -p riptide-headless --all-features"
run_test "typecheck-facade" \
    "cargo check -p riptide-facade --all-features"
echo ""

# Phase 2: Unit Tests (Fast, no browser)
echo -e "${BLUE}🔬 Phase 2: Unit Tests (No Browser Required)${NC}"
echo "================================"
run_test "unit-browser-abstraction" \
    "cargo test -p riptide-browser-abstraction --lib --all-features"
run_test "unit-engine" \
    "cargo test -p riptide-engine --lib --all-features"
run_test "unit-headless" \
    "cargo test -p riptide-headless --lib --all-features"
run_test "unit-types" \
    "cargo test -p riptide-types --lib --all-features"
echo ""

# Phase 3: Browser Abstraction Integration Tests
echo -e "${BLUE}🌐 Phase 3: Browser Abstraction Tests${NC}"
echo "================================"
echo "These tests validate spider-chrome type definitions and parameter structures."
echo ""
run_test "integration-spider-chrome-types" \
    "cargo test -p riptide-browser-abstraction --test spider_chrome_integration_tests -- --test-threads=1 --nocapture"
cleanup
sleep 2
echo ""

# Phase 4: Engine Pool Tests (Sequential, browser required)
echo -e "${BLUE}🏊 Phase 4: Engine Pool Tests${NC}"
echo "================================"
echo "These tests require browser instances and run sequentially."
echo ""
run_test "pool-lifecycle" \
    "cargo test -p riptide-engine --test browser_pool_lifecycle_tests -- --test-threads=1 --nocapture"
cleanup
sleep 2

run_test "pool-cdp" \
    "cargo test -p riptide-engine --test cdp_pool_tests -- --test-threads=1 --nocapture"
cleanup
sleep 2

run_test "pool-validation" \
    "cargo test -p riptide-engine --test cdp_pool_validation_tests -- --test-threads=1 --nocapture"
cleanup
sleep 2
echo ""

# Phase 5: Headless Launcher Tests (Sequential, browser required)
echo -e "${BLUE}🚀 Phase 5: Headless Launcher Tests${NC}"
echo "================================"
run_test "headless-basic" \
    "cargo test -p riptide-headless --test headless_tests -- --test-threads=1 --nocapture"
cleanup
sleep 2
echo ""

# Phase 6: Integration Tests (Sequential, browser required)
echo -e "${BLUE}🔗 Phase 6: Spider-Chrome Integration Tests${NC}"
echo "================================"
if [ -f "tests/integration/spider_chrome_tests.rs" ]; then
    run_test "integration-spider-chrome-full" \
        "cargo test --test spider_chrome_tests -- --test-threads=1 --nocapture"
    cleanup
    sleep 2
else
    echo -e "  ${YELLOW}⚠ spider_chrome_tests.rs not found, skipping${NC}"
fi
echo ""

# Phase 7: CLI Tests (if applicable)
echo -e "${BLUE}💻 Phase 7: CLI Tests${NC}"
echo "================================"
if [ -d "tests/cli" ] && [ -f "tests/cli/Cargo.toml" ]; then
    run_test "cli-tests" \
        "cd tests/cli && cargo test -- --test-threads=1 --nocapture"
    cleanup
    sleep 2
else
    echo -e "  ${YELLOW}⚠ CLI tests directory not found, skipping${NC}"
fi
echo ""

# Phase 8: E2E Tests (Long-running, optional)
echo -e "${BLUE}🎯 Phase 8: E2E Tests (Optional)${NC}"
echo "================================"
if [ "$RUN_E2E" = "true" ]; then
    run_test "e2e-complete-workflow" \
        "cargo test --test e2e_tests -- --test-threads=1 --nocapture"
    cleanup
else
    echo -e "  ${YELLOW}ℹ Skipping E2E tests (set RUN_E2E=true to enable)${NC}"
fi
echo ""

# Generate Summary
echo ""
echo "================================"
echo -e "${BLUE}📊 Test Summary${NC}"
echo "================================"
echo "Total tests run:    $TOTAL_TESTS"
echo -e "Passed:             ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed:             ${RED}$FAILED_TESTS${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    echo ""
    exit 0
else
    echo -e "${RED}✗ Some tests failed.${NC}"
    echo -e "Check logs in: ${YELLOW}$RESULTS_DIR/${NC}"
    echo ""
    exit 1
fi
