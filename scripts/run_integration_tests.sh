#!/bin/bash
# Integration Test Runner for Riptide Crawler
# Runs ignored tests that require infrastructure (Chrome, Redis, PostgreSQL)

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test result counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Log function
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Check if Chrome/Chromium is installed
check_chrome() {
    log "Checking for Chrome/Chromium..."

    if command -v google-chrome &> /dev/null; then
        CHROME_PATH=$(command -v google-chrome)
        success "Found Chrome at: $CHROME_PATH"
        return 0
    elif command -v chromium &> /dev/null; then
        CHROME_PATH=$(command -v chromium)
        success "Found Chromium at: $CHROME_PATH"
        return 0
    elif command -v chromium-browser &> /dev/null; then
        CHROME_PATH=$(command -v chromium-browser)
        success "Found Chromium at: $CHROME_PATH"
        return 0
    else
        error "Chrome/Chromium not found. Please install:"
        echo "  Ubuntu/Debian: sudo apt install chromium-browser"
        echo "  macOS: brew install --cask google-chrome"
        return 1
    fi
}

# Check if Redis is running
check_redis() {
    log "Checking for Redis..."

    if command -v redis-cli &> /dev/null && redis-cli ping &> /dev/null; then
        success "Redis is running"
        return 0
    else
        warn "Redis not running. Redis tests will be skipped."
        echo "  To install: sudo apt install redis-server  OR  brew install redis"
        echo "  To start: sudo systemctl start redis  OR  brew services start redis"
        return 1
    fi
}

# Check if PostgreSQL is running
check_postgres() {
    log "Checking for PostgreSQL..."

    if command -v psql &> /dev/null && pg_isready &> /dev/null; then
        success "PostgreSQL is running"
        return 0
    else
        warn "PostgreSQL not running. Database tests will be skipped."
        echo "  To install: sudo apt install postgresql  OR  brew install postgresql"
        echo "  To start: sudo systemctl start postgresql  OR  brew services start postgresql"
        return 1
    fi
}

# Check disk space (need at least 5GB for Chrome instances)
check_disk_space() {
    log "Checking disk space..."

    available=$(df / | awk 'END{print $4}')
    required=5000000 # 5GB in KB

    if [ "$available" -lt "$required" ]; then
        error "Insufficient disk space. Available: $(($available / 1024))MB, Required: 5000MB"
        echo "  Run: cargo clean"
        return 1
    fi

    success "Sufficient disk space: $(($available / 1024))MB available"
    return 0
}

# Run browser integration tests
run_browser_tests() {
    log "Running browser integration tests..."
    TOTAL_TESTS=$((TOTAL_TESTS + 5))

    if cargo test -p riptide-facade --test browser_facade_integration -- --ignored --nocapture 2>&1 | tee /tmp/browser_tests.log; then
        PASSED_TESTS=$((PASSED_TESTS + 5))
        success "Browser facade tests passed"
    else
        FAILED_TESTS=$((FAILED_TESTS + 5))
        error "Browser facade tests failed. See /tmp/browser_tests.log"
    fi
}

# Run headless browser tests
run_headless_tests() {
    log "Running headless browser tests..."
    TOTAL_TESTS=$((TOTAL_TESTS + 3))

    if cargo test -p riptide-headless -- --ignored --nocapture 2>&1 | tee /tmp/headless_tests.log; then
        PASSED_TESTS=$((PASSED_TESTS + 3))
        success "Headless browser tests passed"
    else
        FAILED_TESTS=$((FAILED_TESTS + 3))
        error "Headless browser tests failed. See /tmp/headless_tests.log"
    fi
}

# Run stealth integration tests
run_stealth_tests() {
    log "Running stealth integration tests..."
    TOTAL_TESTS=$((TOTAL_TESTS + 2))

    if cargo test -p riptide-stealth -- --ignored --nocapture 2>&1 | tee /tmp/stealth_tests.log; then
        PASSED_TESTS=$((PASSED_TESTS + 2))
        success "Stealth tests passed"
    else
        FAILED_TESTS=$((FAILED_TESTS + 2))
        error "Stealth tests failed. See /tmp/stealth_tests.log"
    fi
}

# Run Redis integration tests
run_redis_tests() {
    log "Running Redis integration tests..."
    TOTAL_TESTS=$((TOTAL_TESTS + 15))

    if cargo test -p riptide-cache -- --ignored --nocapture 2>&1 | tee /tmp/redis_tests.log; then
        PASSED_TESTS=$((PASSED_TESTS + 15))
        success "Redis tests passed"
    else
        FAILED_TESTS=$((FAILED_TESTS + 15))
        error "Redis tests failed. See /tmp/redis_tests.log"
    fi
}

# Run PostgreSQL integration tests
run_postgres_tests() {
    log "Running PostgreSQL integration tests..."
    TOTAL_TESTS=$((TOTAL_TESTS + 8))

    if cargo test -p riptide-persistence -- --ignored --nocapture 2>&1 | tee /tmp/postgres_tests.log; then
        PASSED_TESTS=$((PASSED_TESTS + 8))
        success "PostgreSQL tests passed"
    else
        FAILED_TESTS=$((FAILED_TESTS + 8))
        error "PostgreSQL tests failed. See /tmp/postgres_tests.log"
    fi
}

# Run live website crawl tests
run_live_crawl_tests() {
    log "Running live website crawl tests..."
    TOTAL_TESTS=$((TOTAL_TESTS + 3))

    # Test against safe, stable websites
    export TEST_URLS="https://example.com,https://httpbin.org/html,https://quotes.toscrape.com"

    if cargo test -p riptide-facade test_live_crawl -- --ignored --nocapture 2>&1 | tee /tmp/live_crawl_tests.log; then
        PASSED_TESTS=$((PASSED_TESTS + 3))
        success "Live crawl tests passed"
    else
        FAILED_TESTS=$((FAILED_TESTS + 3))
        error "Live crawl tests failed. See /tmp/live_crawl_tests.log"
    fi
}

# Print test summary
print_summary() {
    echo ""
    echo "======================================================================"
    echo "                    INTEGRATION TEST SUMMARY"
    echo "======================================================================"
    echo ""
    echo -e "Total Tests:   ${BLUE}$TOTAL_TESTS${NC}"
    echo -e "Passed:        ${GREEN}$PASSED_TESTS${NC}"
    echo -e "Failed:        ${RED}$FAILED_TESTS${NC}"
    echo -e "Skipped:       ${YELLOW}$SKIPPED_TESTS${NC}"
    echo ""

    if [ $FAILED_TESTS -eq 0 ]; then
        success "ALL INTEGRATION TESTS PASSED! ✓"
        return 0
    else
        error "SOME TESTS FAILED. Check logs in /tmp/"
        return 1
    fi
}

# Main execution
main() {
    log "==================================================================="
    log "        Riptide Crawler Integration Test Suite"
    log "==================================================================="
    echo ""

    # Infrastructure checks
    CHROME_AVAILABLE=false
    REDIS_AVAILABLE=false
    POSTGRES_AVAILABLE=false

    check_disk_space || exit 1
    check_chrome && CHROME_AVAILABLE=true || SKIPPED_TESTS=$((SKIPPED_TESTS + 10))
    check_redis && REDIS_AVAILABLE=true || SKIPPED_TESTS=$((SKIPPED_TESTS + 15))
    check_postgres && POSTGRES_AVAILABLE=true || SKIPPED_TESTS=$((SKIPPED_TESTS + 8))

    echo ""
    log "==================================================================="
    log "                    Running Test Suites"
    log "==================================================================="
    echo ""

    # Run tests based on available infrastructure
    if [ "$CHROME_AVAILABLE" = true ]; then
        run_browser_tests
        run_headless_tests
        run_stealth_tests
        run_live_crawl_tests
    else
        warn "Skipping browser tests (Chrome not available)"
    fi

    if [ "$REDIS_AVAILABLE" = true ]; then
        run_redis_tests
    else
        warn "Skipping Redis tests (Redis not available)"
    fi

    if [ "$POSTGRES_AVAILABLE" = true ]; then
        run_postgres_tests
    else
        warn "Skipping PostgreSQL tests (PostgreSQL not available)"
    fi

    # Print summary and exit
    print_summary
}

# Parse command line arguments
QUICK_MODE=false
while [[ $# -gt 0 ]]; do
    case $1 in
        --quick)
            QUICK_MODE=true
            shift
            ;;
        --browser-only)
            log "Running browser tests only"
            check_chrome && run_browser_tests && run_headless_tests
            exit $?
            ;;
        --redis-only)
            log "Running Redis tests only"
            check_redis && run_redis_tests
            exit $?
            ;;
        --postgres-only)
            log "Running PostgreSQL tests only"
            check_postgres && run_postgres_tests
            exit $?
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --quick          Run quick smoke tests only"
            echo "  --browser-only   Run only browser integration tests"
            echo "  --redis-only     Run only Redis integration tests"
            echo "  --postgres-only  Run only PostgreSQL integration tests"
            echo "  --help           Show this help message"
            echo ""
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Run main
main
