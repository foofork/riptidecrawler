#!/usr/bin/env bash
# ============================================================================
# RipTide Docker Modes Test Suite
# ============================================================================
# Tests all three docker-compose deployment modes:
#   1. Minimal (zero dependencies)
#   2. Simple (with Redis)
#   3. Distributed (full production)
#
# Usage:
#   ./scripts/test-docker-modes.sh [mode]
#
# Modes:
#   minimal       - Test minimal mode only
#   simple        - Test simple mode only
#   distributed   - Test distributed mode only
#   all           - Test all modes (default)
# ============================================================================

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_URL="https://example.com"
API_PORT="${RIPTIDE_API_PORT:-8080}"
API_BASE="http://localhost:${API_PORT}"
TIMEOUT=60

# Test results
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# ============================================================================
# Helper Functions
# ============================================================================

log() {
    echo -e "${BLUE}[$(date +'%H:%M:%S')]${NC} $*"
}

success() {
    echo -e "${GREEN}✅ $*${NC}"
    ((TESTS_PASSED++))
}

error() {
    echo -e "${RED}❌ $*${NC}"
    ((TESTS_FAILED++))
}

warning() {
    echo -e "${YELLOW}⚠️  $*${NC}"
}

section() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}$*${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
}

# Wait for service to be healthy
wait_for_health() {
    local service=$1
    local max_wait=$2
    local compose_file=$3

    log "Waiting for $service to be healthy (max ${max_wait}s)..."

    local elapsed=0
    while [ $elapsed -lt $max_wait ]; do
        if docker-compose -f "$compose_file" ps | grep -q "$service.*healthy"; then
            success "$service is healthy"
            return 0
        fi
        sleep 2
        ((elapsed+=2))
    done

    error "$service failed to become healthy after ${max_wait}s"
    docker-compose -f "$compose_file" logs "$service" | tail -20
    return 1
}

# Test HTTP endpoint
test_endpoint() {
    local name=$1
    local url=$2
    local expected_status=$3

    ((TESTS_TOTAL++))

    log "Testing: $name"

    local status
    status=$(curl -s -o /dev/null -w "%{http_code}" "$url" || echo "000")

    if [ "$status" = "$expected_status" ]; then
        success "$name returned $status"
        return 0
    else
        error "$name returned $status (expected $expected_status)"
        return 1
    fi
}

# Test extraction endpoint
test_extraction() {
    local name=$1
    local test_url=$2

    ((TESTS_TOTAL++))

    log "Testing: $name"

    local response
    response=$(curl -s "${API_BASE}/extract?url=${test_url}" || echo "")

    if echo "$response" | grep -q "example"; then
        success "$name extracted content successfully"
        return 0
    else
        error "$name failed to extract content"
        echo "Response: $response" | head -5
        return 1
    fi
}

# Test cache behavior
test_cache() {
    local name=$1
    local should_persist=$2
    local compose_file=$3

    ((TESTS_TOTAL++))

    log "Testing: $name cache behavior"

    # First request
    local time1
    time1=$(curl -s -w "%{time_total}" -o /dev/null "${API_BASE}/extract?url=${TEST_URL}")

    # Second request (should be cached)
    local time2
    time2=$(curl -s -w "%{time_total}" -o /dev/null "${API_BASE}/extract?url=${TEST_URL}")

    # Restart service
    log "Restarting service to test cache persistence..."
    docker-compose -f "$compose_file" restart riptide-api >/dev/null 2>&1
    sleep 5

    # Third request (after restart)
    local time3
    time3=$(curl -s -w "%{time_total}" -o /dev/null "${API_BASE}/extract?url=${TEST_URL}")

    if [ "$should_persist" = "true" ]; then
        # Cache should persist
        if (( $(echo "$time3 < 0.5" | bc -l) )); then
            success "$name cache persisted after restart ($time3s)"
            return 0
        else
            error "$name cache did not persist ($time3s)"
            return 1
        fi
    else
        # Cache should NOT persist
        if (( $(echo "$time3 > 0.5" | bc -l) )); then
            success "$name cache cleared as expected ($time3s)"
            return 0
        else
            warning "$name cache may have persisted unexpectedly ($time3s)"
            return 0  # Not a hard failure for minimal mode
        fi
    fi
}

# ============================================================================
# Mode Test Functions
# ============================================================================

test_minimal_mode() {
    section "Testing Minimal Mode (Zero Dependencies)"

    local compose_file="docker-compose.minimal.yml"

    log "Starting minimal mode..."
    docker-compose -f "$compose_file" up -d

    sleep 10

    # Check container count
    ((TESTS_TOTAL++))
    local container_count
    container_count=$(docker-compose -f "$compose_file" ps -q | wc -l)
    if [ "$container_count" -eq 1 ]; then
        success "Minimal mode has exactly 1 container"
    else
        error "Minimal mode has $container_count containers (expected 1)"
    fi

    # Wait for API
    if ! wait_for_health "riptide-api" 30 "$compose_file"; then
        error "Minimal mode API failed to start"
        docker-compose -f "$compose_file" down
        return 1
    fi

    # Test health endpoint
    test_endpoint "Health check" "${API_BASE}/health" "200"

    # Test extraction
    test_extraction "Static content extraction" "$TEST_URL"

    # Test cache behavior (should NOT persist)
    test_cache "Minimal mode" "false" "$compose_file"

    # Check memory usage
    ((TESTS_TOTAL++))
    local memory_mb
    memory_mb=$(docker stats --no-stream --format "{{.MemUsage}}" riptide-minimal | cut -d'/' -f1 | sed 's/MiB//')
    log "Memory usage: ${memory_mb}MB"
    if (( $(echo "$memory_mb < 600" | bc -l) )); then
        success "Memory usage within expected range (${memory_mb}MB < 600MB)"
    else
        warning "Memory usage higher than expected (${memory_mb}MB)"
    fi

    # Cleanup
    log "Stopping minimal mode..."
    docker-compose -f "$compose_file" down -v

    section "Minimal Mode Tests Complete"
}

test_simple_mode() {
    section "Testing Simple Mode (API + Redis)"

    local compose_file="docker-compose.simple.yml"

    log "Starting simple mode..."
    docker-compose -f "$compose_file" up -d

    sleep 15

    # Check container count
    ((TESTS_TOTAL++))
    local container_count
    container_count=$(docker-compose -f "$compose_file" ps -q | wc -l)
    if [ "$container_count" -eq 2 ]; then
        success "Simple mode has exactly 2 containers"
    else
        error "Simple mode has $container_count containers (expected 2)"
    fi

    # Wait for services
    if ! wait_for_health "redis" 20 "$compose_file"; then
        error "Simple mode Redis failed to start"
        docker-compose -f "$compose_file" down
        return 1
    fi

    if ! wait_for_health "riptide-api" 30 "$compose_file"; then
        error "Simple mode API failed to start"
        docker-compose -f "$compose_file" down
        return 1
    fi

    # Test Redis connectivity
    ((TESTS_TOTAL++))
    if docker-compose -f "$compose_file" exec -T redis redis-cli ping | grep -q "PONG"; then
        success "Redis is responding"
    else
        error "Redis is not responding"
    fi

    # Test health endpoint
    test_endpoint "Health check" "${API_BASE}/health" "200"

    # Test extraction
    test_extraction "Static content extraction" "$TEST_URL"

    # Test cache behavior (SHOULD persist)
    test_cache "Simple mode" "true" "$compose_file"

    # Check Redis cache entries
    ((TESTS_TOTAL++))
    local cache_entries
    cache_entries=$(docker-compose -f "$compose_file" exec -T redis redis-cli DBSIZE | cut -d':' -f2 | tr -d '\r')
    log "Redis cache entries: $cache_entries"
    if [ "$cache_entries" -gt 0 ]; then
        success "Redis contains cached entries ($cache_entries)"
    else
        warning "Redis has no cached entries"
    fi

    # Check memory usage
    ((TESTS_TOTAL++))
    local total_memory
    total_memory=$(docker stats --no-stream --format "{{.Container}}\t{{.MemUsage}}" | grep "riptide-simple" | awk '{print $2}' | cut -d'/' -f1 | sed 's/MiB//' | awk '{sum+=$1} END {print sum}')
    log "Total memory usage: ${total_memory}MB"
    if (( $(echo "$total_memory < 800" | bc -l) )); then
        success "Memory usage within expected range (${total_memory}MB < 800MB)"
    else
        warning "Memory usage higher than expected (${total_memory}MB)"
    fi

    # Cleanup
    log "Stopping simple mode..."
    docker-compose -f "$compose_file" down -v

    section "Simple Mode Tests Complete"
}

test_distributed_mode() {
    section "Testing Distributed Mode (Full Production)"

    local compose_file="docker-compose.yml"

    log "Starting distributed mode..."
    docker-compose up -d

    sleep 40

    # Check container count
    ((TESTS_TOTAL++))
    local container_count
    container_count=$(docker-compose ps -q | wc -l)
    if [ "$container_count" -ge 3 ]; then
        success "Distributed mode has $container_count containers (expected ≥3)"
    else
        error "Distributed mode has $container_count containers (expected ≥3)"
    fi

    # Wait for services
    if ! wait_for_health "redis" 20 "$compose_file"; then
        error "Distributed mode Redis failed to start"
        docker-compose down
        return 1
    fi

    if ! wait_for_health "riptide-api" 40 "$compose_file"; then
        error "Distributed mode API failed to start"
        docker-compose down
        return 1
    fi

    # Test health endpoint
    test_endpoint "Health check" "${API_BASE}/health" "200"

    # Test extraction
    test_extraction "Static content extraction" "$TEST_URL"

    # Test cache behavior (SHOULD persist)
    test_cache "Distributed mode" "true" "$compose_file"

    # Test browser service (if available)
    ((TESTS_TOTAL++))
    if docker-compose ps | grep -q "riptide-headless"; then
        if docker-compose ps | grep "riptide-headless" | grep -q "Up"; then
            success "Headless browser service is running"
        else
            error "Headless browser service is not running"
        fi
    else
        warning "Headless browser service not found (may be optional)"
    fi

    # Check memory usage
    ((TESTS_TOTAL++))
    local total_memory
    total_memory=$(docker stats --no-stream --format "{{.MemUsage}}" | cut -d'/' -f1 | sed 's/MiB//' | awk '{sum+=$1} END {print sum}')
    log "Total memory usage: ${total_memory}MB"
    if (( $(echo "$total_memory < 2000" | bc -l) )); then
        success "Memory usage within expected range (${total_memory}MB < 2000MB)"
    else
        warning "Memory usage higher than expected (${total_memory}MB)"
    fi

    # Cleanup
    log "Stopping distributed mode..."
    docker-compose down -v

    section "Distributed Mode Tests Complete"
}

# ============================================================================
# Main Script
# ============================================================================

main() {
    local mode="${1:-all}"

    section "RipTide Docker Modes Test Suite"
    log "Testing mode: $mode"
    log "API base URL: $API_BASE"
    log "Test URL: $TEST_URL"

    # Ensure Docker is running
    if ! docker info >/dev/null 2>&1; then
        error "Docker is not running"
        exit 1
    fi

    # Run tests based on mode
    case "$mode" in
        minimal)
            test_minimal_mode
            ;;
        simple)
            test_simple_mode
            ;;
        distributed)
            test_distributed_mode
            ;;
        all)
            test_minimal_mode
            echo ""
            test_simple_mode
            echo ""
            test_distributed_mode
            ;;
        *)
            error "Invalid mode: $mode"
            echo "Usage: $0 [minimal|simple|distributed|all]"
            exit 1
            ;;
    esac

    # Print summary
    section "Test Results"
    echo ""
    echo "Total Tests:  $TESTS_TOTAL"
    echo -e "${GREEN}Passed:       $TESTS_PASSED${NC}"
    echo -e "${RED}Failed:       $TESTS_FAILED${NC}"
    echo ""

    if [ $TESTS_FAILED -eq 0 ]; then
        success "All tests passed! 🎉"
        exit 0
    else
        error "Some tests failed"
        exit 1
    fi
}

# Run main function with arguments
main "$@"
