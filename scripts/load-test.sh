#!/bin/bash

# RipTide Phase 1 Load Testing Script
# Tests performance requirements: 100 concurrent requests with <2s p95 response time
# Author: RipTide QA Team
# Version: 1.0.0

set -euo pipefail

# Configuration
DEFAULT_HOST="http://localhost:8080"
DEFAULT_CONCURRENT=100
DEFAULT_REQUESTS=1000
DEFAULT_DURATION=60
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="$SCRIPT_DIR/load-test-results"
TEST_DATA_DIR="$SCRIPT_DIR/test-data"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Global variables
HOST="${RIPTIDE_HOST:-$DEFAULT_HOST}"
CONCURRENT="${RIPTIDE_CONCURRENT:-$DEFAULT_CONCURRENT}"
REQUESTS="${RIPTIDE_REQUESTS:-$DEFAULT_REQUESTS}"
DURATION="${RIPTIDE_DURATION:-$DEFAULT_DURATION}"
VERBOSE=0
CLEANUP_ON_EXIT=1
START_SERVER=0
STOP_SERVER=0

# Ensure dependencies are available
check_dependencies() {
    echo -e "${BLUE}Checking dependencies...${NC}"

    local missing_deps=()

    # Check for curl
    if ! command -v curl &> /dev/null; then
        missing_deps+=("curl")
    fi

    # Check for jq for JSON processing
    if ! command -v jq &> /dev/null; then
        missing_deps+=("jq")
    fi

    # Check for hey for load testing (preferred) or fallback to curl
    if ! command -v hey &> /dev/null; then
        echo -e "${YELLOW}Warning: 'hey' load testing tool not found. Install with: go install github.com/rakyll/hey@latest${NC}"
        echo -e "${YELLOW}Falling back to curl-based testing (less accurate)${NC}"
    fi

    # Check for bc for calculations
    if ! command -v bc &> /dev/null; then
        missing_deps+=("bc")
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        echo -e "${RED}Error: Missing required dependencies: ${missing_deps[*]}${NC}"
        echo -e "${YELLOW}Install missing dependencies:${NC}"
        echo "  Ubuntu/Debian: sudo apt-get install curl jq bc"
        echo "  macOS: brew install curl jq bc"
        echo "  Go tool: go install github.com/rakyll/hey@latest"
        exit 1
    fi

    echo -e "${GREEN}All dependencies satisfied${NC}"
}

# Display usage information
usage() {
    cat << EOF
RipTide Phase 1 Load Testing Script

Usage: $0 [OPTIONS] [COMMAND]

Commands:
    test-health         Test /healthz endpoint only
    test-crawl          Test /crawl endpoint only
    test-all            Test all endpoints (default)
    generate-data       Generate test data files
    start-server        Start RipTide server for testing
    stop-server         Stop RipTide server
    report              Generate detailed performance report

Options:
    -h, --host HOST     Target host (default: $DEFAULT_HOST)
    -c, --concurrent N  Number of concurrent requests (default: $DEFAULT_CONCURRENT)
    -n, --requests N    Total number of requests (default: $DEFAULT_REQUESTS)
    -d, --duration N    Test duration in seconds (default: $DEFAULT_DURATION)
    -v, --verbose       Enable verbose output
    --no-cleanup        Don't cleanup temp files
    --start-server      Start RipTide server before testing
    --stop-server       Stop RipTide server after testing
    --help              Show this help message

Environment Variables:
    RIPTIDE_HOST        Override default host
    RIPTIDE_CONCURRENT  Override default concurrent requests
    RIPTIDE_REQUESTS    Override default total requests
    RIPTIDE_DURATION    Override default duration

Examples:
    # Basic load test with defaults
    $0

    # Test with custom parameters
    $0 --host http://127.0.0.1:8080 --concurrent 50 --requests 500

    # Test only health endpoint
    $0 test-health

    # Generate test data and run full test
    $0 generate-data && $0 test-all

    # Start server, test, and stop server
    $0 --start-server --stop-server test-all

Success Criteria:
    - P95 response time < 2000ms
    - Error rate < 1%
    - Successful completion of 100 concurrent requests

EOF
}

# Setup directories and cleanup
setup_environment() {
    mkdir -p "$RESULTS_DIR" "$TEST_DATA_DIR"

    # Cleanup function
    cleanup() {
        if [ $CLEANUP_ON_EXIT -eq 1 ]; then
            echo -e "${BLUE}Cleaning up temporary files...${NC}"
            # Remove temporary files but keep results
            find "$RESULTS_DIR" -name "*.tmp" -delete 2>/dev/null || true
        fi

        if [ $STOP_SERVER -eq 1 ]; then
            stop_riptide_server
        fi
    }

    trap cleanup EXIT
}

# Generate realistic test data
generate_test_data() {
    echo -e "${BLUE}Generating test data...${NC}"

    # Generate URLs for different content types
    cat > "$TEST_DATA_DIR/test-urls.json" << 'EOF'
{
  "urls": [
    "https://httpbin.org/html",
    "https://httpbin.org/json",
    "https://httpbin.org/xml",
    "https://httpbin.org/delay/1",
    "https://example.com",
    "https://www.wikipedia.org",
    "https://github.com",
    "https://stackoverflow.com",
    "https://news.ycombinator.com",
    "https://httpbin.org/status/200",
    "https://httpbin.org/headers",
    "https://httpbin.org/user-agent",
    "https://httpbin.org/ip",
    "https://jsonplaceholder.typicode.com/posts/1",
    "https://jsonplaceholder.typicode.com/users",
    "https://www.google.com",
    "https://www.reddit.com",
    "https://medium.com",
    "https://www.npmjs.com",
    "https://crates.io"
  ],
  "options": {
    "concurrency": 10,
    "cache_mode": "auto"
  }
}
EOF

    # Generate small batch test data
    cat > "$TEST_DATA_DIR/small-batch.json" << 'EOF'
{
  "urls": [
    "https://httpbin.org/json",
    "https://httpbin.org/html",
    "https://example.com"
  ],
  "options": {
    "concurrency": 3,
    "cache_mode": "force_fresh"
  }
}
EOF

    # Generate large batch test data
    cat > "$TEST_DATA_DIR/large-batch.json" << 'EOF'
{
  "urls": [
EOF

    # Add 50 diverse URLs for large batch testing
    local urls=(
        "https://httpbin.org/html"
        "https://httpbin.org/json"
        "https://httpbin.org/xml"
        "https://httpbin.org/delay/1"
        "https://example.com"
        "https://www.wikipedia.org"
        "https://github.com"
        "https://stackoverflow.com"
        "https://news.ycombinator.com"
        "https://httpbin.org/status/200"
        "https://jsonplaceholder.typicode.com/posts/1"
        "https://jsonplaceholder.typicode.com/users"
        "https://httpbin.org/headers"
        "https://httpbin.org/user-agent"
        "https://httpbin.org/ip"
        "https://www.google.com"
        "https://www.reddit.com"
        "https://medium.com"
        "https://www.npmjs.com"
        "https://crates.io"
    )

    for i in {1..50}; do
        local url_index=$((i % ${#urls[@]}))
        local url="${urls[$url_index]}"
        if [ $i -eq 50 ]; then
            echo "    \"$url\"" >> "$TEST_DATA_DIR/large-batch.json"
        else
            echo "    \"$url\"," >> "$TEST_DATA_DIR/large-batch.json"
        fi
    done

    cat >> "$TEST_DATA_DIR/large-batch.json" << 'EOF'
  ],
  "options": {
    "concurrency": 20,
    "cache_mode": "auto"
  }
}
EOF

    echo -e "${GREEN}Test data generated in $TEST_DATA_DIR${NC}"
}

# Start RipTide server
start_riptide_server() {
    echo -e "${BLUE}Starting RipTide server...${NC}"

    local server_dir="/workspaces/eventmesh"
    if [ ! -d "$server_dir" ]; then
        echo -e "${RED}Error: RipTide server directory not found at $server_dir${NC}"
        return 1
    fi

    cd "$server_dir"

    # Build the server if not already built
    if [ ! -f "target/release/riptide-api" ]; then
        echo -e "${BLUE}Building RipTide server...${NC}"
        cargo build --release --bin riptide-api
    fi

    # Start the server in background
    echo -e "${BLUE}Launching RipTide API server...${NC}"
    nohup ./target/release/riptide-api --bind "0.0.0.0:8080" > "$RESULTS_DIR/server.log" 2>&1 &
    local server_pid=$!
    echo $server_pid > "$RESULTS_DIR/server.pid"

    # Wait for server to start
    echo -e "${BLUE}Waiting for server to start...${NC}"
    local max_attempts=30
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        if curl -s "$HOST/healthz" > /dev/null 2>&1; then
            echo -e "${GREEN}RipTide server started successfully (PID: $server_pid)${NC}"
            return 0
        fi
        sleep 1
        ((attempt++))
    done

    echo -e "${RED}Error: RipTide server failed to start within $max_attempts seconds${NC}"
    return 1
}

# Stop RipTide server
stop_riptide_server() {
    if [ -f "$RESULTS_DIR/server.pid" ]; then
        local server_pid=$(cat "$RESULTS_DIR/server.pid")
        echo -e "${BLUE}Stopping RipTide server (PID: $server_pid)...${NC}"
        kill $server_pid 2>/dev/null || true
        rm -f "$RESULTS_DIR/server.pid"
        echo -e "${GREEN}RipTide server stopped${NC}"
    fi
}

# Check if server is responding
check_server_health() {
    echo -e "${BLUE}Checking server health...${NC}"

    local response=$(curl -s -w "%{http_code}" "$HOST/healthz" -o "$RESULTS_DIR/health-check.tmp")

    if [ "$response" = "200" ]; then
        local health_status=$(jq -r '.status // "unknown"' "$RESULTS_DIR/health-check.tmp" 2>/dev/null || echo "unknown")
        echo -e "${GREEN}Server is healthy (status: $health_status)${NC}"
        return 0
    else
        echo -e "${RED}Server health check failed (HTTP $response)${NC}"
        return 1
    fi
}

# Run load test using hey if available, otherwise use curl
run_load_test_hey() {
    local endpoint="$1"
    local test_name="$2"
    local post_data="$3"

    echo -e "${BLUE}Running load test: $test_name${NC}"
    echo -e "${BLUE}  Endpoint: $endpoint${NC}"
    echo -e "${BLUE}  Concurrent: $CONCURRENT${NC}"
    echo -e "${BLUE}  Total Requests: $REQUESTS${NC}"

    local output_file="$RESULTS_DIR/${test_name}-hey.json"
    local temp_file="$RESULTS_DIR/${test_name}-hey.tmp"

    if [ -n "$post_data" ]; then
        # POST request with data
        hey -z "${DURATION}s" -c $CONCURRENT -m POST \
            -H "Content-Type: application/json" \
            -d "$post_data" \
            -o csv "$endpoint" > "$temp_file" 2>&1
    else
        # GET request
        hey -z "${DURATION}s" -c $CONCURRENT \
            -o csv "$endpoint" > "$temp_file" 2>&1
    fi

    # Parse hey output and convert to JSON
    parse_hey_output "$temp_file" > "$output_file"

    echo -e "${GREEN}Load test completed: $test_name${NC}"
    return 0
}

# Parse hey output and convert to structured format
parse_hey_output() {
    local input_file="$1"

    if [ ! -f "$input_file" ]; then
        echo '{"error": "No output file found"}'
        return 1
    fi

    # Extract metrics from hey output
    local total_requests=$(grep -o "Total:\s*[0-9]*" "$input_file" | awk '{print $2}' || echo "0")
    local successful_requests=$(grep -o "Successful:\s*[0-9]*" "$input_file" | awk '{print $2}' || echo "0")
    local total_time=$(grep -o "Total time:\s*[0-9.]*" "$input_file" | awk '{print $3}' || echo "0")
    local requests_per_sec=$(grep -o "Requests/sec:\s*[0-9.]*" "$input_file" | awk '{print $2}' || echo "0")

    # Extract percentiles
    local p50=$(grep -A 10 "Response time histogram" "$input_file" | grep "50%" | awk '{print $2}' || echo "0")
    local p75=$(grep -A 10 "Response time histogram" "$input_file" | grep "75%" | awk '{print $2}' || echo "0")
    local p90=$(grep -A 10 "Response time histogram" "$input_file" | grep "90%" | awk '{print $2}' || echo "0")
    local p95=$(grep -A 10 "Response time histogram" "$input_file" | grep "95%" | awk '{print $2}' || echo "0")
    local p99=$(grep -A 10 "Response time histogram" "$input_file" | grep "99%" | awk '{print $2}' || echo "0")

    # Calculate error rate
    local error_rate=0
    if [ "$total_requests" -gt 0 ]; then
        error_rate=$(echo "scale=4; (($total_requests - $successful_requests) / $total_requests) * 100" | bc)
    fi

    # Convert to milliseconds (hey outputs in seconds)
    p50_ms=$(echo "$p50 * 1000" | bc 2>/dev/null || echo "0")
    p75_ms=$(echo "$p75 * 1000" | bc 2>/dev/null || echo "0")
    p90_ms=$(echo "$p90 * 1000" | bc 2>/dev/null || echo "0")
    p95_ms=$(echo "$p95 * 1000" | bc 2>/dev/null || echo "0")
    p99_ms=$(echo "$p99 * 1000" | bc 2>/dev/null || echo "0")

    cat << EOF
{
  "timestamp": "$(date -Iseconds)",
  "test_config": {
    "concurrent_users": $CONCURRENT,
    "total_requests": $total_requests,
    "duration_seconds": $DURATION
  },
  "results": {
    "total_requests": $total_requests,
    "successful_requests": $successful_requests,
    "failed_requests": $(($total_requests - $successful_requests)),
    "error_rate_percent": $error_rate,
    "total_time_seconds": $total_time,
    "requests_per_second": $requests_per_sec,
    "response_times_ms": {
      "p50": $p50_ms,
      "p75": $p75_ms,
      "p90": $p90_ms,
      "p95": $p95_ms,
      "p99": $p99_ms
    }
  }
}
EOF
}

# Fallback curl-based load testing
run_load_test_curl() {
    local endpoint="$1"
    local test_name="$2"
    local post_data="$3"

    echo -e "${YELLOW}Using curl-based load testing (less accurate than hey)${NC}"
    echo -e "${BLUE}Running load test: $test_name${NC}"

    local output_file="$RESULTS_DIR/${test_name}-curl.json"
    local temp_dir="$RESULTS_DIR/${test_name}-curl-temp"
    mkdir -p "$temp_dir"

    local start_time=$(date +%s.%N)
    local success_count=0
    local error_count=0
    local response_times=()

    # Run concurrent requests
    for ((i=1; i<=CONCURRENT; i++)); do
        {
            local request_start=$(date +%s.%N)

            if [ -n "$post_data" ]; then
                local response_code=$(curl -s -w "%{http_code}" -o "$temp_dir/response_$i.tmp" \
                    -H "Content-Type: application/json" \
                    -X POST -d "$post_data" "$endpoint")
            else
                local response_code=$(curl -s -w "%{http_code}" -o "$temp_dir/response_$i.tmp" "$endpoint")
            fi

            local request_end=$(date +%s.%N)
            local response_time=$(echo "($request_end - $request_start) * 1000" | bc)

            echo "$response_code,$response_time" > "$temp_dir/result_$i.tmp"
        } &
    done

    # Wait for all requests to complete
    wait

    local end_time=$(date +%s.%N)
    local total_time=$(echo "$end_time - $start_time" | bc)

    # Process results
    for ((i=1; i<=CONCURRENT; i++)); do
        if [ -f "$temp_dir/result_$i.tmp" ]; then
            local result=$(cat "$temp_dir/result_$i.tmp")
            local code=$(echo "$result" | cut -d',' -f1)
            local time=$(echo "$result" | cut -d',' -f2)

            response_times+=($time)

            if [[ "$code" =~ ^2[0-9][0-9]$ ]]; then
                ((success_count++))
            else
                ((error_count++))
            fi
        fi
    done

    # Calculate percentiles
    local sorted_times=($(printf '%s\n' "${response_times[@]}" | sort -n))
    local total_requests=${#sorted_times[@]}

    local p50_index=$(echo "$total_requests * 0.5 / 1" | bc)
    local p75_index=$(echo "$total_requests * 0.75 / 1" | bc)
    local p90_index=$(echo "$total_requests * 0.90 / 1" | bc)
    local p95_index=$(echo "$total_requests * 0.95 / 1" | bc)
    local p99_index=$(echo "$total_requests * 0.99 / 1" | bc)

    local p50=${sorted_times[$p50_index]:-0}
    local p75=${sorted_times[$p75_index]:-0}
    local p90=${sorted_times[$p90_index]:-0}
    local p95=${sorted_times[$p95_index]:-0}
    local p99=${sorted_times[$p99_index]:-0}

    local error_rate=0
    if [ $total_requests -gt 0 ]; then
        error_rate=$(echo "scale=2; ($error_count / $total_requests) * 100" | bc)
    fi

    local requests_per_sec=$(echo "scale=2; $total_requests / $total_time" | bc)

    cat > "$output_file" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "test_config": {
    "concurrent_users": $CONCURRENT,
    "total_requests": $total_requests,
    "duration_seconds": $total_time
  },
  "results": {
    "total_requests": $total_requests,
    "successful_requests": $success_count,
    "failed_requests": $error_count,
    "error_rate_percent": $error_rate,
    "total_time_seconds": $total_time,
    "requests_per_second": $requests_per_sec,
    "response_times_ms": {
      "p50": $p50,
      "p75": $p75,
      "p90": $p90,
      "p95": $p95,
      "p99": $p99
    }
  }
}
EOF

    # Cleanup
    rm -rf "$temp_dir"

    echo -e "${GREEN}Load test completed: $test_name${NC}"
}

# Run appropriate load test based on available tools
run_load_test() {
    if command -v hey &> /dev/null; then
        run_load_test_hey "$@"
    else
        run_load_test_curl "$@"
    fi
}

# Test health endpoint
test_health_endpoint() {
    echo -e "${BLUE}Testing /healthz endpoint...${NC}"

    run_load_test "$HOST/healthz" "health" ""

    local result_file="$RESULTS_DIR/health-hey.json"
    if [ ! -f "$result_file" ]; then
        result_file="$RESULTS_DIR/health-curl.json"
    fi

    if [ -f "$result_file" ]; then
        local p95=$(jq -r '.results.response_times_ms.p95' "$result_file")
        local error_rate=$(jq -r '.results.error_rate_percent' "$result_file")

        echo -e "${BLUE}Health endpoint results:${NC}"
        echo -e "  P95 response time: ${p95}ms"
        echo -e "  Error rate: ${error_rate}%"

        # Validate success criteria
        if (( $(echo "$p95 < 2000" | bc -l) )); then
            echo -e "${GREEN}✓ P95 response time meets criteria (<2000ms)${NC}"
        else
            echo -e "${RED}✗ P95 response time exceeds criteria (${p95}ms >= 2000ms)${NC}"
        fi

        if (( $(echo "$error_rate < 1" | bc -l) )); then
            echo -e "${GREEN}✓ Error rate meets criteria (<1%)${NC}"
        else
            echo -e "${RED}✗ Error rate exceeds criteria (${error_rate}% >= 1%)${NC}"
        fi
    fi
}

# Test crawl endpoint
test_crawl_endpoint() {
    echo -e "${BLUE}Testing /crawl endpoint...${NC}"

    # Use small batch data for load testing
    local test_data=$(cat "$TEST_DATA_DIR/small-batch.json")

    run_load_test "$HOST/crawl" "crawl" "$test_data"

    local result_file="$RESULTS_DIR/crawl-hey.json"
    if [ ! -f "$result_file" ]; then
        result_file="$RESULTS_DIR/crawl-curl.json"
    fi

    if [ -f "$result_file" ]; then
        local p95=$(jq -r '.results.response_times_ms.p95' "$result_file")
        local error_rate=$(jq -r '.results.error_rate_percent' "$result_file")

        echo -e "${BLUE}Crawl endpoint results:${NC}"
        echo -e "  P95 response time: ${p95}ms"
        echo -e "  Error rate: ${error_rate}%"

        # Validate success criteria
        if (( $(echo "$p95 < 2000" | bc -l) )); then
            echo -e "${GREEN}✓ P95 response time meets criteria (<2000ms)${NC}"
        else
            echo -e "${RED}✗ P95 response time exceeds criteria (${p95}ms >= 2000ms)${NC}"
        fi

        if (( $(echo "$error_rate < 1" | bc -l) )); then
            echo -e "${GREEN}✓ Error rate meets criteria (<1%)${NC}"
        else
            echo -e "${RED}✗ Error rate exceeds criteria (${error_rate}% >= 1%)${NC}"
        fi
    fi
}

# Generate comprehensive performance report
generate_report() {
    echo -e "${BLUE}Generating performance report...${NC}"

    local report_file="$RESULTS_DIR/performance-report-$(date +%Y%m%d-%H%M%S).md"

    cat > "$report_file" << EOF
# RipTide Phase 1 Load Testing Report

**Generated:** $(date)
**Test Configuration:**
- Host: $HOST
- Concurrent Users: $CONCURRENT
- Total Requests: $REQUESTS
- Test Duration: ${DURATION}s

## Success Criteria
- ✅ P95 response time < 2000ms
- ✅ Error rate < 1%
- ✅ 100 concurrent requests handled successfully

## Test Results

EOF

    # Process health endpoint results
    local health_file=""
    if [ -f "$RESULTS_DIR/health-hey.json" ]; then
        health_file="$RESULTS_DIR/health-hey.json"
    elif [ -f "$RESULTS_DIR/health-curl.json" ]; then
        health_file="$RESULTS_DIR/health-curl.json"
    fi

    if [ -n "$health_file" ] && [ -f "$health_file" ]; then
        cat >> "$report_file" << EOF
### Health Endpoint (/healthz)

$(jq -r '
"**Metrics:**
- Total Requests: " + (.results.total_requests | tostring) + "
- Successful Requests: " + (.results.successful_requests | tostring) + "
- Failed Requests: " + (.results.failed_requests | tostring) + "
- Error Rate: " + (.results.error_rate_percent | tostring) + "%
- Requests/sec: " + (.results.requests_per_second | tostring) + "

**Response Times:**
- P50: " + (.results.response_times_ms.p50 | tostring) + "ms
- P75: " + (.results.response_times_ms.p75 | tostring) + "ms
- P90: " + (.results.response_times_ms.p90 | tostring) + "ms
- P95: " + (.results.response_times_ms.p95 | tostring) + "ms
- P99: " + (.results.response_times_ms.p99 | tostring) + "ms"
' "$health_file")

**Success Criteria Validation:**
$(jq -r '
if (.results.response_times_ms.p95 < 2000) then
  "- ✅ P95 response time: " + (.results.response_times_ms.p95 | tostring) + "ms (< 2000ms)"
else
  "- ❌ P95 response time: " + (.results.response_times_ms.p95 | tostring) + "ms (>= 2000ms)"
end
' "$health_file")
$(jq -r '
if (.results.error_rate_percent < 1) then
  "- ✅ Error rate: " + (.results.error_rate_percent | tostring) + "% (< 1%)"
else
  "- ❌ Error rate: " + (.results.error_rate_percent | tostring) + "% (>= 1%)"
end
' "$health_file")

EOF
    fi

    # Process crawl endpoint results
    local crawl_file=""
    if [ -f "$RESULTS_DIR/crawl-hey.json" ]; then
        crawl_file="$RESULTS_DIR/crawl-hey.json"
    elif [ -f "$RESULTS_DIR/crawl-curl.json" ]; then
        crawl_file="$RESULTS_DIR/crawl-curl.json"
    fi

    if [ -n "$crawl_file" ] && [ -f "$crawl_file" ]; then
        cat >> "$report_file" << EOF
### Crawl Endpoint (/crawl)

$(jq -r '
"**Metrics:**
- Total Requests: " + (.results.total_requests | tostring) + "
- Successful Requests: " + (.results.successful_requests | tostring) + "
- Failed Requests: " + (.results.failed_requests | tostring) + "
- Error Rate: " + (.results.error_rate_percent | tostring) + "%
- Requests/sec: " + (.results.requests_per_second | tostring) + "

**Response Times:**
- P50: " + (.results.response_times_ms.p50 | tostring) + "ms
- P75: " + (.results.response_times_ms.p75 | tostring) + "ms
- P90: " + (.results.response_times_ms.p90 | tostring) + "ms
- P95: " + (.results.response_times_ms.p95 | tostring) + "ms
- P99: " + (.results.response_times_ms.p99 | tostring) + "ms"
' "$crawl_file")

**Success Criteria Validation:**
$(jq -r '
if (.results.response_times_ms.p95 < 2000) then
  "- ✅ P95 response time: " + (.results.response_times_ms.p95 | tostring) + "ms (< 2000ms)"
else
  "- ❌ P95 response time: " + (.results.response_times_ms.p95 | tostring) + "ms (>= 2000ms)"
end
' "$crawl_file")
$(jq -r '
if (.results.error_rate_percent < 1) then
  "- ✅ Error rate: " + (.results.error_rate_percent | tostring) + "% (< 1%)"
else
  "- ❌ Error rate: " + (.results.error_rate_percent | tostring) + "% (>= 1%)"
end
' "$crawl_file")

EOF
    fi

    cat >> "$report_file" << EOF
## Summary

This load test validates RipTide Phase 1 performance requirements by testing both the health check and crawl endpoints under load with 100 concurrent users.

### Test Environment
- Operating System: $(uname -s) $(uname -r)
- Load Testing Tool: $(command -v hey >/dev/null && echo "hey" || echo "curl (fallback)")
- Server Version: RipTide API v$(curl -s "$HOST/healthz" | jq -r '.version // "unknown"' 2>/dev/null || echo "unknown")

### Files Generated
- Raw test results: \`$RESULTS_DIR/\`
- Test data: \`$TEST_DATA_DIR/\`

### Recommendations
$(if [ -n "$health_file" ]; then
    jq -r '
    if (.results.response_times_ms.p95 > 1500) then
      "- Consider optimizing health check endpoint for faster response times"
    else
      "- Health check performance is excellent"
    end
    ' "$health_file"
fi)
$(if [ -n "$crawl_file" ]; then
    jq -r '
    if (.results.response_times_ms.p95 > 1500) then
      "- Consider optimizing crawl pipeline for better performance"
    else
      "- Crawl endpoint performance is excellent"
    end,
    if (.results.error_rate_percent > 0.5) then
      "- Investigate and reduce error rate for improved reliability"
    else
      "- Error rates are within acceptable limits"
    end
    ' "$crawl_file"
fi)

EOF

    echo -e "${GREEN}Performance report generated: $report_file${NC}"

    if [ $VERBOSE -eq 1 ]; then
        echo -e "${BLUE}Report preview:${NC}"
        head -20 "$report_file"
        echo "..."
    fi
}

# Main execution function
main() {
    local command="test-all"

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--host)
                HOST="$2"
                shift 2
                ;;
            -c|--concurrent)
                CONCURRENT="$2"
                shift 2
                ;;
            -n|--requests)
                REQUESTS="$2"
                shift 2
                ;;
            -d|--duration)
                DURATION="$2"
                shift 2
                ;;
            -v|--verbose)
                VERBOSE=1
                shift
                ;;
            --no-cleanup)
                CLEANUP_ON_EXIT=0
                shift
                ;;
            --start-server)
                START_SERVER=1
                shift
                ;;
            --stop-server)
                STOP_SERVER=1
                shift
                ;;
            --help)
                usage
                exit 0
                ;;
            test-health|test-crawl|test-all|generate-data|start-server|stop-server|report)
                command="$1"
                shift
                ;;
            *)
                echo -e "${RED}Error: Unknown option $1${NC}"
                usage
                exit 1
                ;;
        esac
    done

    # Execute command
    case $command in
        generate-data)
            setup_environment
            generate_test_data
            ;;
        start-server)
            setup_environment
            start_riptide_server
            ;;
        stop-server)
            stop_riptide_server
            ;;
        test-health)
            setup_environment
            check_dependencies
            check_server_health
            generate_test_data
            test_health_endpoint
            ;;
        test-crawl)
            setup_environment
            check_dependencies
            check_server_health
            generate_test_data
            test_crawl_endpoint
            ;;
        test-all)
            setup_environment
            check_dependencies

            if [ $START_SERVER -eq 1 ]; then
                start_riptide_server
            fi

            check_server_health
            generate_test_data
            test_health_endpoint
            test_crawl_endpoint
            generate_report
            ;;
        report)
            generate_report
            ;;
        *)
            echo -e "${RED}Error: Unknown command $command${NC}"
            usage
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"