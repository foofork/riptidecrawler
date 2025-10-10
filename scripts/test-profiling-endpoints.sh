#!/bin/bash
# Test script for profiling API endpoints
# Usage: ./scripts/test-profiling-endpoints.sh [BASE_URL]

set -e

BASE_URL="${1:-http://localhost:8080}"
TIMESTAMP=$(date +%s)
OUTPUT_DIR="/tmp/riptide-profiling-test-${TIMESTAMP}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "======================================"
echo "RipTide Profiling Endpoints Test"
echo "======================================"
echo "Base URL: ${BASE_URL}"
echo "Output Directory: ${OUTPUT_DIR}"
echo ""

# Create output directory
mkdir -p "${OUTPUT_DIR}"

# Test function with error handling
test_endpoint() {
    local method=$1
    local endpoint=$2
    local name=$3

    echo -n "Testing ${name}... "

    local output_file="${OUTPUT_DIR}/${name// /_}.json"
    local status_code

    if [ "$method" = "GET" ]; then
        status_code=$(curl -s -o "${output_file}" -w "%{http_code}" "${BASE_URL}${endpoint}")
    else
        status_code=$(curl -s -X POST -o "${output_file}" -w "%{http_code}" "${BASE_URL}${endpoint}")
    fi

    if [ "$status_code" = "200" ]; then
        # Validate JSON
        if jq empty "${output_file}" 2>/dev/null; then
            echo -e "${GREEN}✓ PASS${NC} (HTTP $status_code)"
            return 0
        else
            echo -e "${RED}✗ FAIL${NC} (Invalid JSON)"
            return 1
        fi
    else
        echo -e "${RED}✗ FAIL${NC} (HTTP $status_code)"
        cat "${output_file}"
        return 1
    fi
}

# Test health endpoint first
echo "=== Pre-flight Health Check ==="
test_endpoint "GET" "/healthz" "Health Check" || {
    echo -e "${RED}ERROR: API server is not healthy!${NC}"
    exit 1
}
echo ""

# Test all profiling endpoints
echo "=== Profiling Endpoints ==="

# 1. Memory Profile
test_endpoint "GET" "/api/profiling/memory" "Memory Profile"
if [ -f "${OUTPUT_DIR}/Memory_Profile.json" ]; then
    echo "  RSS: $(jq -r '.rss_mb' "${OUTPUT_DIR}/Memory_Profile.json")MB"
    echo "  Heap: $(jq -r '.heap_mb' "${OUTPUT_DIR}/Memory_Profile.json")MB"
    echo "  Status: $(jq -r '.threshold_status' "${OUTPUT_DIR}/Memory_Profile.json")"
fi
echo ""

# 2. CPU Profile
test_endpoint "GET" "/api/profiling/cpu" "CPU Profile"
if [ -f "${OUTPUT_DIR}/CPU_Profile.json" ]; then
    echo "  CPU Usage: $(jq -r '.cpu_usage_percent' "${OUTPUT_DIR}/CPU_Profile.json")%"
    echo "  Available: $(jq -r '.available' "${OUTPUT_DIR}/CPU_Profile.json")"
fi
echo ""

# 3. Bottleneck Analysis
test_endpoint "GET" "/api/profiling/bottlenecks" "Bottleneck Analysis"
if [ -f "${OUTPUT_DIR}/Bottleneck_Analysis.json" ]; then
    echo "  Hotspots: $(jq -r '.hotspots | length' "${OUTPUT_DIR}/Bottleneck_Analysis.json")"
    echo "  CPU Bound: $(jq -r '.cpu_bound_percent' "${OUTPUT_DIR}/Bottleneck_Analysis.json")%"
    echo "  IO Bound: $(jq -r '.io_bound_percent' "${OUTPUT_DIR}/Bottleneck_Analysis.json")%"
fi
echo ""

# 4. Allocation Metrics
test_endpoint "GET" "/api/profiling/allocations" "Allocation Metrics"
if [ -f "${OUTPUT_DIR}/Allocation_Metrics.json" ]; then
    echo "  Efficiency: $(jq -r '.efficiency_score' "${OUTPUT_DIR}/Allocation_Metrics.json")"
    echo "  Fragmentation: $(jq -r '.fragmentation_percent' "${OUTPUT_DIR}/Allocation_Metrics.json")%"
    echo "  Top Allocators: $(jq -r '.top_allocators | length' "${OUTPUT_DIR}/Allocation_Metrics.json")"
fi
echo ""

# 5. Leak Detection (POST)
test_endpoint "POST" "/api/profiling/leak-detection" "Leak Detection"
if [ -f "${OUTPUT_DIR}/Leak_Detection.json" ]; then
    echo "  Potential Leaks: $(jq -r '.potential_leaks | length' "${OUTPUT_DIR}/Leak_Detection.json")"
    echo "  Growth Rate: $(jq -r '.growth_rate_mb_per_hour' "${OUTPUT_DIR}/Leak_Detection.json")MB/hour"
    if [ "$(jq -r '.highest_risk_component' "${OUTPUT_DIR}/Leak_Detection.json")" != "null" ]; then
        echo -e "  ${YELLOW}⚠ Highest Risk: $(jq -r '.highest_risk_component' "${OUTPUT_DIR}/Leak_Detection.json")${NC}"
    fi
fi
echo ""

# 6. Heap Snapshot (POST)
test_endpoint "POST" "/api/profiling/snapshot" "Heap Snapshot"
if [ -f "${OUTPUT_DIR}/Heap_Snapshot.json" ]; then
    echo "  Snapshot ID: $(jq -r '.snapshot_id' "${OUTPUT_DIR}/Heap_Snapshot.json")"
    echo "  Size: $(jq -r '.size_bytes' "${OUTPUT_DIR}/Heap_Snapshot.json") bytes"
    echo "  Status: $(jq -r '.status' "${OUTPUT_DIR}/Heap_Snapshot.json")"
fi
echo ""

# Performance test - rapid successive calls
echo "=== Performance Test ==="
echo -n "Testing rapid successive calls (10 requests)... "
start_time=$(date +%s%N)
for i in {1..10}; do
    curl -s "${BASE_URL}/api/profiling/memory" > /dev/null
done
end_time=$(date +%s%N)
duration_ms=$(( (end_time - start_time) / 1000000 ))
avg_ms=$(( duration_ms / 10 ))

if [ $avg_ms -lt 100 ]; then
    echo -e "${GREEN}✓ PASS${NC} (avg ${avg_ms}ms per request)"
else
    echo -e "${YELLOW}⚠ SLOW${NC} (avg ${avg_ms}ms per request, target <100ms)"
fi
echo ""

# Summary
echo "======================================"
echo "Test Summary"
echo "======================================"
echo "Results saved to: ${OUTPUT_DIR}"
echo ""
echo "View detailed results:"
echo "  ls -lh ${OUTPUT_DIR}"
echo "  jq . ${OUTPUT_DIR}/*.json"
echo ""

# Check for warnings or critical issues
echo "=== System Health Summary ==="
if [ -f "${OUTPUT_DIR}/Memory_Profile.json" ]; then
    threshold_status=$(jq -r '.threshold_status' "${OUTPUT_DIR}/Memory_Profile.json")
    rss_mb=$(jq -r '.rss_mb' "${OUTPUT_DIR}/Memory_Profile.json")

    case "$threshold_status" in
        "critical")
            echo -e "${RED}⚠ CRITICAL: Memory usage ${rss_mb}MB exceeds safe limits${NC}"
            ;;
        "warning")
            echo -e "${YELLOW}⚠ WARNING: Memory usage ${rss_mb}MB approaching limits${NC}"
            ;;
        "normal")
            echo -e "${GREEN}✓ Normal: Memory usage ${rss_mb}MB within safe limits${NC}"
            ;;
    esac
fi

if [ -f "${OUTPUT_DIR}/Leak_Detection.json" ]; then
    leak_count=$(jq -r '.potential_leaks | length' "${OUTPUT_DIR}/Leak_Detection.json")
    if [ "$leak_count" -gt 0 ]; then
        echo -e "${YELLOW}⚠ Detected $leak_count potential memory leak(s)${NC}"
    else
        echo -e "${GREEN}✓ No memory leaks detected${NC}"
    fi
fi

echo ""
echo "======================================"
echo "Testing Complete!"
echo "======================================"
