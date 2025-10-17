#!/bin/bash
# Load Testing Script for EventMesh/RipTide
# Tests API performance under various load conditions

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
API_PORT=${API_PORT:-8080}
TEST_TYPE=${1:-basic}
RESULTS_DIR="tests/load/results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

echo -e "${GREEN}EventMesh Load Testing Suite${NC}"
echo "========================================"
echo "Test Type: $TEST_TYPE"
echo "Timestamp: $TIMESTAMP"
echo ""

# Create results directory
mkdir -p "$RESULTS_DIR"

# Function to check if API is running
check_api() {
    echo -e "${YELLOW}Checking if API is running on port $API_PORT...${NC}"

    if curl -s -f "http://localhost:$API_PORT/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ API is running${NC}"
        return 0
    else
        echo -e "${RED}✗ API is not running${NC}"
        return 1
    fi
}

# Function to start API if needed
start_api() {
    echo -e "${YELLOW}Starting EventMesh API...${NC}"

    # Build release version
    cargo build --release 2>&1 | tee "$RESULTS_DIR/build_$TIMESTAMP.log"

    # Start API in background
    cargo run --release -- serve --port $API_PORT > "$RESULTS_DIR/api_$TIMESTAMP.log" 2>&1 &
    API_PID=$!

    echo "API PID: $API_PID"

    # Wait for API to be ready
    echo "Waiting for API to start..."
    for i in {1..30}; do
        if check_api; then
            echo -e "${GREEN}✓ API started successfully${NC}"
            return 0
        fi
        sleep 1
    done

    echo -e "${RED}✗ Failed to start API${NC}"
    return 1
}

# Function to stop API
stop_api() {
    if [ ! -z "$API_PID" ]; then
        echo -e "${YELLOW}Stopping API (PID: $API_PID)...${NC}"
        kill $API_PID 2>/dev/null || true
        wait $API_PID 2>/dev/null || true
        echo -e "${GREEN}✓ API stopped${NC}"
    fi
}

# Function to run load test
run_load_test() {
    local test_file=$1
    local test_name=$2

    echo ""
    echo -e "${YELLOW}Running $test_name...${NC}"
    echo "----------------------------------------"

    if [ ! -f "$test_file" ]; then
        echo -e "${RED}✗ Test file not found: $test_file${NC}"
        return 1
    fi

    # Check if drill is installed
    if ! command -v drill &> /dev/null; then
        echo -e "${YELLOW}Installing drill...${NC}"
        cargo install drill
    fi

    # Run drill with benchmark
    drill --benchmark "$test_file" --stats --quiet \
        > "$RESULTS_DIR/${test_name}_$TIMESTAMP.txt" 2>&1

    local exit_code=$?

    if [ $exit_code -eq 0 ]; then
        echo -e "${GREEN}✓ $test_name completed${NC}"

        # Show summary
        echo ""
        echo "Results Summary:"
        tail -n 20 "$RESULTS_DIR/${test_name}_$TIMESTAMP.txt"
    else
        echo -e "${RED}✗ $test_name failed${NC}"
        return 1
    fi
}

# Function to collect system metrics
collect_metrics() {
    echo ""
    echo -e "${YELLOW}Collecting system metrics...${NC}"

    {
        echo "=== System Metrics ==="
        echo "Timestamp: $(date)"
        echo ""
        echo "=== CPU Usage ==="
        top -bn1 | grep "Cpu(s)" || true
        echo ""
        echo "=== Memory Usage ==="
        free -h || true
        echo ""
        echo "=== Disk Usage ==="
        df -h || true
        echo ""
    } > "$RESULTS_DIR/metrics_$TIMESTAMP.txt"

    echo -e "${GREEN}✓ Metrics collected${NC}"
}

# Function to generate report
generate_report() {
    local report_file="$RESULTS_DIR/report_$TIMESTAMP.md"

    echo ""
    echo -e "${YELLOW}Generating report...${NC}"

    {
        echo "# Load Test Report"
        echo ""
        echo "**Timestamp:** $TIMESTAMP"
        echo "**Test Type:** $TEST_TYPE"
        echo ""
        echo "## Test Results"
        echo ""

        for result in "$RESULTS_DIR"/*_$TIMESTAMP.txt; do
            if [ -f "$result" ]; then
                test_name=$(basename "$result" "_$TIMESTAMP.txt")
                echo "### $test_name"
                echo ""
                echo '```'
                tail -n 30 "$result"
                echo '```'
                echo ""
            fi
        done

        echo "## System Metrics"
        echo ""
        echo '```'
        cat "$RESULTS_DIR/metrics_$TIMESTAMP.txt" 2>/dev/null || echo "No metrics collected"
        echo '```'

    } > "$report_file"

    echo -e "${GREEN}✓ Report generated: $report_file${NC}"
}

# Main execution
main() {
    local should_stop_api=false

    # Check if API is already running
    if ! check_api; then
        if start_api; then
            should_stop_api=true
        else
            echo -e "${RED}Failed to start API. Exiting.${NC}"
            exit 1
        fi
    fi

    # Collect initial metrics
    collect_metrics

    # Run appropriate test
    case "$TEST_TYPE" in
        basic)
            run_load_test "tests/load/basic_load_test.yml" "basic_load_test"
            ;;
        stress)
            run_load_test "tests/load/stress_test.yml" "stress_test"
            ;;
        all)
            run_load_test "tests/load/basic_load_test.yml" "basic_load_test"
            run_load_test "tests/load/stress_test.yml" "stress_test"
            ;;
        *)
            echo -e "${RED}Unknown test type: $TEST_TYPE${NC}"
            echo "Valid options: basic, stress, all"
            exit 1
            ;;
    esac

    # Collect final metrics
    collect_metrics

    # Generate report
    generate_report

    # Stop API if we started it
    if [ "$should_stop_api" = true ]; then
        stop_api
    fi

    echo ""
    echo -e "${GREEN}Load testing complete!${NC}"
    echo "Results saved to: $RESULTS_DIR"
}

# Trap to ensure API is stopped on exit
trap 'stop_api' EXIT INT TERM

# Run main function
main
