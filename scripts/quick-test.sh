#!/usr/bin/env bash
# ============================================================================
# RipTide Quick Test Script
# ============================================================================
# Comprehensive automated testing with zero manual configuration.
#
# Usage:
#   ./scripts/quick-test.sh              # Run all tests
#   ./scripts/quick-test.sh --minimal    # Basic tests only
#   ./scripts/quick-test.sh --full       # Include browser tests
#   ./scripts/quick-test.sh --cleanup    # Cleanup only
#
# Requirements:
#   - Docker and Docker Compose
#   - curl (for API testing)
#   - jq (optional, for JSON formatting)
# ============================================================================

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
COMPOSE_FILE="docker-compose.test.yml"
API_URL="http://localhost:8080"
MAX_WAIT=60  # Maximum wait time for services (seconds)
TEST_RESULTS=()

# ============================================================================
# Helper Functions
# ============================================================================

log_info() {
    echo -e "${BLUE}ℹ ${NC}$1"
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

record_test() {
    local name=$1
    local result=$2
    TEST_RESULTS+=("$name:$result")
}

# ============================================================================
# Service Management
# ============================================================================

start_services() {
    log_info "Starting RipTide test environment..."

    # Check if already running
    if docker compose -f "$COMPOSE_FILE" ps | grep -q "Up"; then
        log_warning "Services already running. Stopping first..."
        docker compose -f "$COMPOSE_FILE" down
    fi

    # Start services
    docker compose -f "$COMPOSE_FILE" up -d

    log_info "Waiting for services to be ready..."
    wait_for_health
}

wait_for_health() {
    local elapsed=0
    local interval=2

    while [ $elapsed -lt $MAX_WAIT ]; do
        if curl -sf "$API_URL/health" > /dev/null 2>&1; then
            log_success "Services ready in ${elapsed}s"
            return 0
        fi
        sleep $interval
        elapsed=$((elapsed + interval))
        echo -n "."
    done

    echo ""
    log_error "Services failed to start within ${MAX_WAIT}s"
    docker compose -f "$COMPOSE_FILE" logs --tail=50
    return 1
}

stop_services() {
    log_info "Stopping services..."
    docker compose -f "$COMPOSE_FILE" down -v
    log_success "Services stopped and cleaned up"
}

# ============================================================================
# Test Functions
# ============================================================================

test_health_endpoint() {
    log_info "Testing health endpoint..."

    local response
    response=$(curl -sf "$API_URL/health" 2>&1) || {
        log_error "Health check failed"
        record_test "health_check" "FAIL"
        return 1
    }

    log_success "Health check passed"
    record_test "health_check" "PASS"
    return 0
}

test_basic_extraction() {
    log_info "Testing basic extraction (example.com)..."

    local response
    response=$(curl -sf -X POST "$API_URL/api/extract" \
        -H "Content-Type: application/json" \
        -d '{"url": "https://example.com"}' 2>&1) || {
        log_error "Basic extraction failed"
        echo "Response: $response"
        record_test "basic_extraction" "FAIL"
        return 1
    }

    # Check if response contains expected fields
    if echo "$response" | grep -q "Example Domain"; then
        log_success "Basic extraction passed"
        record_test "basic_extraction" "PASS"
        return 0
    else
        log_error "Basic extraction returned unexpected content"
        echo "Response: $response"
        record_test "basic_extraction" "FAIL"
        return 1
    fi
}

test_batch_crawl() {
    log_info "Testing batch crawl (multiple URLs)..."

    local response
    response=$(curl -sf -X POST "$API_URL/api/crawl" \
        -H "Content-Type: application/json" \
        -d '{
            "urls": [
                "https://example.com",
                "https://example.org"
            ],
            "use_spider": false
        }' 2>&1) || {
        log_error "Batch crawl failed"
        echo "Response: $response"
        record_test "batch_crawl" "FAIL"
        return 1
    }

    log_success "Batch crawl passed"
    record_test "batch_crawl" "PASS"
    return 0
}

test_spider_mode() {
    log_info "Testing spider mode (deep crawl)..."

    local response
    response=$(curl -sf -X POST "$API_URL/api/crawl" \
        -H "Content-Type: application/json" \
        -d '{
            "urls": ["https://example.com"],
            "use_spider": true,
            "max_depth": 2,
            "max_pages": 5
        }' 2>&1) || {
        log_error "Spider mode failed"
        echo "Response: $response"
        record_test "spider_mode" "FAIL"
        return 1
    }

    log_success "Spider mode passed"
    record_test "spider_mode" "PASS"
    return 0
}

test_metrics_endpoint() {
    log_info "Testing metrics endpoint..."

    local response
    response=$(curl -sf "$API_URL/metrics" 2>&1) || {
        log_error "Metrics endpoint failed"
        record_test "metrics" "FAIL"
        return 1
    }

    log_success "Metrics endpoint passed"
    record_test "metrics" "PASS"
    return 0
}

test_invalid_url() {
    log_info "Testing error handling (invalid URL)..."

    local response
    local http_code
    http_code=$(curl -sf -X POST "$API_URL/api/extract" \
        -H "Content-Type: application/json" \
        -d '{"url": "not-a-valid-url"}' \
        -w "%{http_code}" \
        -o /dev/null 2>&1)

    if [ "$http_code" -ge 400 ]; then
        log_success "Error handling passed (HTTP $http_code)"
        record_test "error_handling" "PASS"
        return 0
    else
        log_error "Error handling failed (expected 4xx, got $http_code)"
        record_test "error_handling" "FAIL"
        return 1
    fi
}

test_no_auth_required() {
    log_info "Testing that no authentication is required..."

    # This should succeed without any auth headers
    local response
    response=$(curl -sf -X POST "$API_URL/api/extract" \
        -H "Content-Type: application/json" \
        -d '{"url": "https://example.com"}' 2>&1) || {
        log_error "Request failed - auth may be required"
        record_test "no_auth" "FAIL"
        return 1
    }

    log_success "No authentication required (as expected)"
    record_test "no_auth" "PASS"
    return 0
}

# ============================================================================
# Test Suites
# ============================================================================

run_minimal_tests() {
    log_info "Running minimal test suite..."
    echo ""

    test_health_endpoint
    test_basic_extraction
    test_no_auth_required
}

run_standard_tests() {
    log_info "Running standard test suite..."
    echo ""

    test_health_endpoint
    test_basic_extraction
    test_no_auth_required
    test_batch_crawl
    test_spider_mode
    test_metrics_endpoint
    test_invalid_url
}

run_full_tests() {
    log_info "Running full test suite (including browser tests)..."
    echo ""

    # Start services with headless profile
    log_info "Starting with headless browser support..."
    docker compose -f "$COMPOSE_FILE" --profile headless up -d
    wait_for_health

    # Run all standard tests
    run_standard_tests

    # Additional browser-specific tests would go here
    log_warning "Browser-specific tests not yet implemented"
}

# ============================================================================
# Results Summary
# ============================================================================

print_summary() {
    local total=${#TEST_RESULTS[@]}
    local passed=0
    local failed=0

    echo ""
    echo "======================================================================"
    log_info "TEST SUMMARY"
    echo "======================================================================"
    echo ""

    for result in "${TEST_RESULTS[@]}"; do
        local name="${result%%:*}"
        local status="${result##*:}"

        if [ "$status" = "PASS" ]; then
            log_success "$name"
            ((passed++))
        else
            log_error "$name"
            ((failed++))
        fi
    done

    echo ""
    echo "======================================================================"
    echo -e "Total: $total | ${GREEN}Passed: $passed${NC} | ${RED}Failed: $failed${NC}"
    echo "======================================================================"
    echo ""

    if [ $failed -eq 0 ]; then
        log_success "All tests passed! 🎉"
        return 0
    else
        log_error "$failed test(s) failed"
        return 1
    fi
}

# ============================================================================
# Main Script
# ============================================================================

show_usage() {
    cat << EOF
RipTide Quick Test Script

Usage: $0 [OPTIONS]

Options:
    --minimal       Run minimal tests only (health, extraction, auth)
    --full          Run full test suite including browser tests
    --cleanup       Stop services and cleanup
    --help          Show this help message

Default: Runs standard test suite
EOF
}

main() {
    local mode="standard"

    # Parse arguments
    while [ $# -gt 0 ]; do
        case "$1" in
            --minimal)
                mode="minimal"
                shift
                ;;
            --full)
                mode="full"
                shift
                ;;
            --cleanup)
                stop_services
                exit 0
                ;;
            --help)
                show_usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done

    echo ""
    echo "======================================================================"
    echo "  RipTide Quick Test Script"
    echo "  Mode: $mode"
    echo "======================================================================"
    echo ""

    # Check prerequisites
    if ! command -v docker &> /dev/null; then
        log_error "Docker is required but not installed"
        exit 1
    fi

    if ! command -v curl &> /dev/null; then
        log_error "curl is required but not installed"
        exit 1
    fi

    # Start services
    start_services

    # Give services a moment to initialize
    sleep 2

    # Run tests based on mode
    case "$mode" in
        minimal)
            run_minimal_tests
            ;;
        full)
            run_full_tests
            ;;
        *)
            run_standard_tests
            ;;
    esac

    # Print summary
    local exit_code=0
    print_summary || exit_code=$?

    # Cleanup
    echo ""
    read -p "Stop services and cleanup? [Y/n] " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]] || [[ -z $REPLY ]]; then
        stop_services
    else
        log_info "Services left running. Stop with:"
        echo "  docker compose -f $COMPOSE_FILE down -v"
    fi

    exit $exit_code
}

# Run main function
main "$@"
