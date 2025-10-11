#!/usr/bin/env bash
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(cd "$TEST_DIR/../.." && pwd)"
RESULTS_DIR="$TEST_DIR/results"
LOGS_DIR="$TEST_DIR/logs"
BINARY="$PROJECT_ROOT/target/release/eventmesh-cli"

# Default values
TEST_URLS="$TEST_DIR/test-urls.json"
METHODS=("jina" "playwright" "selenium" "puppeteer" "firecrawl" "r2r")
TIMEOUT=30
RETRY_ATTEMPTS=3
PARALLEL_JOBS=4

# Create directories
mkdir -p "$RESULTS_DIR" "$LOGS_DIR"

# Function to print colored output
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if binary exists
check_binary() {
    if [[ ! -f "$BINARY" ]]; then
        log_error "Binary not found at $BINARY"
        log_info "Building project..."
        cd "$PROJECT_ROOT" && cargo build --release
        if [[ ! -f "$BINARY" ]]; then
            log_error "Failed to build binary"
            exit 1
        fi
    fi
    log_success "Binary found: $BINARY"
}

# Function to test a single URL with a method
test_url() {
    local method=$1
    local url=$2
    local test_id=$3
    local attempt=1
    local max_attempts=$RETRY_ATTEMPTS

    while [[ $attempt -le $max_attempts ]]; do
        local output_file="$LOGS_DIR/${test_id}_${method}_attempt${attempt}.log"
        local result_file="$RESULTS_DIR/${test_id}_${method}_attempt${attempt}.json"
        local start_time=$(date +%s%3N)

        if timeout "$TIMEOUT" "$BINARY" extract \
            --method "$method" \
            --url "$url" \
            --output "$result_file" \
            > "$output_file" 2>&1; then

            local end_time=$(date +%s%3N)
            local duration=$((end_time - start_time))

            # Extract content length
            local content_length=0
            if [[ -f "$result_file" ]]; then
                content_length=$(wc -c < "$result_file" | tr -d ' ')
            fi

            log_success "✓ $test_id [$method] - ${duration}ms, ${content_length} bytes"

            # Create success metadata
            cat > "${result_file}.meta.json" <<EOF
{
  "test_id": "$test_id",
  "method": "$method",
  "url": "$url",
  "success": true,
  "duration_ms": $duration,
  "content_length": $content_length,
  "attempt": $attempt,
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
}
EOF
            return 0
        else
            local end_time=$(date +%s%3N)
            local duration=$((end_time - start_time))

            log_warning "✗ $test_id [$method] failed (attempt $attempt/$max_attempts)"

            # Create failure metadata
            local error_msg=$(tail -n 5 "$output_file" | tr '\n' ' ')
            cat > "${result_file}.meta.json" <<EOF
{
  "test_id": "$test_id",
  "method": "$method",
  "url": "$url",
  "success": false,
  "duration_ms": $duration,
  "content_length": 0,
  "attempt": $attempt,
  "error": "$error_msg",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
}
EOF

            if [[ $attempt -lt $max_attempts ]]; then
                log_info "Retrying in 2 seconds..."
                sleep 2
            fi
            ((attempt++))
        fi
    done

    log_error "✗ $test_id [$method] failed after $max_attempts attempts"
    return 1
}

# Function to run tests for all URLs
run_all_tests() {
    local session_id="session-$(date +%s)"
    local session_file="$RESULTS_DIR/${session_id}.json"

    log_info "Starting test session: $session_id"
    log_info "Test URLs: $TEST_URLS"
    log_info "Methods: ${METHODS[*]}"
    log_info "Timeout: ${TIMEOUT}s per test"
    log_info "Retry attempts: $RETRY_ATTEMPTS"
    log_info "Parallel jobs: $PARALLEL_JOBS"

    # Parse test URLs
    local urls=$(jq -r '.test_urls[] | "\(.id)|\(.url)"' "$TEST_URLS")
    local total_tests=0
    local successful_tests=0
    local failed_tests=0

    # Count total tests
    local url_count=$(echo "$urls" | wc -l)
    local method_count=${#METHODS[@]}
    total_tests=$((url_count * method_count))

    log_info "Total tests to run: $total_tests"

    # Create session header
    cat > "$session_file" <<EOF
{
  "session_id": "$session_id",
  "start_time": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "total_tests": $total_tests,
  "methods": $(jq -c -n '$ARGS.positional' --args "${METHODS[@]}"),
  "results": [
EOF

    # Run tests
    local first_result=true
    while IFS='|' read -r test_id url; do
        log_info "Testing URL: $test_id"

        for method in "${METHODS[@]}"; do
            if test_url "$method" "$url" "$test_id"; then
                ((successful_tests++))
            else
                ((failed_tests++))
            fi

            # Append result to session file
            local result_meta="${RESULTS_DIR}/${test_id}_${method}_attempt${RETRY_ATTEMPTS}.json.meta.json"
            if [[ -f "$result_meta" ]]; then
                if [[ "$first_result" == "true" ]]; then
                    first_result=false
                else
                    echo "," >> "$session_file"
                fi
                cat "$result_meta" >> "$session_file"
            fi
        done
    done <<< "$urls"

    # Close session file
    cat >> "$session_file" <<EOF
  ],
  "end_time": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "successful_tests": $successful_tests,
  "failed_tests": $failed_tests,
  "success_rate": $(echo "scale=2; $successful_tests * 100 / $total_tests" | bc)
}
EOF

    log_success "Test session complete!"
    log_info "Results saved to: $session_file"
    log_info "Total tests: $total_tests"
    log_success "Successful: $successful_tests ($(echo "scale=1; $successful_tests * 100 / $total_tests" | bc)%)"
    log_error "Failed: $failed_tests ($(echo "scale=1; $failed_tests * 100 / $total_tests" | bc)%)"
}

# Function to generate summary report
generate_report() {
    local session_file=$1

    if [[ ! -f "$session_file" ]]; then
        log_error "Session file not found: $session_file"
        return 1
    fi

    log_info "Generating summary report..."

    local session_id=$(jq -r '.session_id' "$session_file")
    local report_file="$RESULTS_DIR/${session_id}_report.txt"

    {
        echo "================================"
        echo "Test Session Report"
        echo "================================"
        echo ""
        echo "Session ID: $(jq -r '.session_id' "$session_file")"
        echo "Start Time: $(jq -r '.start_time' "$session_file")"
        echo "End Time: $(jq -r '.end_time' "$session_file")"
        echo ""
        echo "Summary:"
        echo "  Total Tests: $(jq -r '.total_tests' "$session_file")"
        echo "  Successful:  $(jq -r '.successful_tests' "$session_file")"
        echo "  Failed:      $(jq -r '.failed_tests' "$session_file")"
        echo "  Success Rate: $(jq -r '.success_rate' "$session_file")%"
        echo ""
        echo "Method Statistics:"

        # Calculate per-method statistics
        for method in "${METHODS[@]}"; do
            local method_success=$(jq "[.results[] | select(.method == \"$method\" and .success == true)] | length" "$session_file")
            local method_total=$(jq "[.results[] | select(.method == \"$method\")] | length" "$session_file")
            local method_rate=$(echo "scale=1; $method_success * 100 / $method_total" | bc)
            local method_avg_duration=$(jq "[.results[] | select(.method == \"$method\" and .success == true) | .duration_ms] | add / length" "$session_file")

            echo "  $method:"
            echo "    Success: $method_success/$method_total ($method_rate%)"
            echo "    Avg Duration: ${method_avg_duration}ms"
        done

        echo ""
        echo "Failed Tests:"
        jq -r '.results[] | select(.success == false) | "  \(.test_id) [\(.method)]: \(.error)"' "$session_file"

    } > "$report_file"

    cat "$report_file"
    log_success "Report saved to: $report_file"
}

# Parse command line arguments
COMMAND="${1:-run}"

case "$COMMAND" in
    run)
        check_binary
        run_all_tests
        ;;
    report)
        SESSION_FILE="${2:-$(ls -t "$RESULTS_DIR"/session-*.json | head -n1)}"
        generate_report "$SESSION_FILE"
        ;;
    clean)
        log_warning "Cleaning up results and logs..."
        rm -rf "$RESULTS_DIR"/* "$LOGS_DIR"/*
        log_success "Cleanup complete"
        ;;
    *)
        echo "Usage: $0 {run|report [session-file]|clean}"
        echo ""
        echo "Commands:"
        echo "  run           - Run all extraction tests"
        echo "  report [file] - Generate report from session file"
        echo "  clean         - Clean up results and logs"
        exit 1
        ;;
esac
