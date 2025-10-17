#!/bin/bash
# Performance Analysis Script for Phase 3 Direct Execution
# This script profiles the extraction pipeline and identifies bottlenecks

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_DIR="$PROJECT_ROOT/docs/hive-mind/performance-results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== RipTide Phase 3 Performance Analysis ===${NC}"
echo "Timestamp: $TIMESTAMP"
echo "Results Directory: $RESULTS_DIR"
echo ""

# Create results directory
mkdir -p "$RESULTS_DIR"

# Function to log with timestamp
log() {
    echo -e "${GREEN}[$(date +%H:%M:%S)]${NC} $1"
}

error() {
    echo -e "${RED}[$(date +%H:%M:%S)] ERROR:${NC} $1"
}

warning() {
    echo -e "${YELLOW}[$(date +%H:%M:%S)] WARNING:${NC} $1"
}

# 1. Build with performance profiling enabled
log "Building RipTide with performance features..."
cd "$PROJECT_ROOT"
cargo build --release --features "memory-profiling,bottleneck-analysis" 2>&1 | tee "$RESULTS_DIR/build_${TIMESTAMP}.log"

if [ $? -ne 0 ]; then
    error "Build failed. Check logs at $RESULTS_DIR/build_${TIMESTAMP}.log"
    exit 1
fi

# 2. Run performance benchmarks
log "Running performance benchmarks..."
if cargo test --release --features "memory-profiling,bottleneck-analysis" --package riptide-performance -- --nocapture > "$RESULTS_DIR/benchmarks_${TIMESTAMP}.log" 2>&1; then
    log "Benchmarks completed successfully"
else
    warning "Some benchmarks failed. Check logs for details."
fi

# 3. Profile extraction pipeline with different engines
log "Profiling extraction engines..."

# Test URLs for profiling
TEST_URLS=(
    "https://example.com"
    "https://github.com"
    "https://docs.rs"
)

for url in "${TEST_URLS[@]}"; do
    url_safe=$(echo "$url" | sed 's/[^a-zA-Z0-9]/_/g')
    log "Profiling: $url"

    # Profile WASM engine
    time ./target/release/riptide extract "$url" --engine wasm --output-dir /tmp/riptide_profile 2>&1 | tee "$RESULTS_DIR/profile_wasm_${url_safe}_${TIMESTAMP}.log" || warning "WASM extraction failed for $url"

    # Profile headless engine (if available)
    if command -v google-chrome &> /dev/null || command -v chromium &> /dev/null; then
        time ./target/release/riptide extract "$url" --engine headless --output-dir /tmp/riptide_profile 2>&1 | tee "$RESULTS_DIR/profile_headless_${url_safe}_${TIMESTAMP}.log" || warning "Headless extraction failed for $url"
    else
        warning "Chrome/Chromium not found. Skipping headless profiling."
    fi

    # Profile stealth engine
    time ./target/release/riptide extract "$url" --engine stealth --output-dir /tmp/riptide_profile 2>&1 | tee "$RESULTS_DIR/profile_stealth_${url_safe}_${TIMESTAMP}.log" || warning "Stealth extraction failed for $url"
done

# 4. Memory profiling
log "Running memory profiling..."
valgrind --tool=massif --massif-out-file="$RESULTS_DIR/massif_${TIMESTAMP}.out" ./target/release/riptide extract "https://example.com" --engine wasm 2>&1 | tee "$RESULTS_DIR/valgrind_${TIMESTAMP}.log" || warning "Valgrind profiling failed"

# Generate massif visualization
if [ -f "$RESULTS_DIR/massif_${TIMESTAMP}.out" ]; then
    ms_print "$RESULTS_DIR/massif_${TIMESTAMP}.out" > "$RESULTS_DIR/massif_report_${TIMESTAMP}.txt"
    log "Memory profile saved to massif_report_${TIMESTAMP}.txt"
fi

# 5. CPU profiling with perf (Linux only)
if command -v perf &> /dev/null; then
    log "Running CPU profiling with perf..."
    perf record -g -o "$RESULTS_DIR/perf_${TIMESTAMP}.data" -- ./target/release/riptide extract "https://example.com" --engine wasm 2>&1 | tee "$RESULTS_DIR/perf_${TIMESTAMP}.log" || warning "Perf profiling failed"

    # Generate perf report
    if [ -f "$RESULTS_DIR/perf_${TIMESTAMP}.data" ]; then
        perf report -i "$RESULTS_DIR/perf_${TIMESTAMP}.data" --stdio > "$RESULTS_DIR/perf_report_${TIMESTAMP}.txt"
        log "CPU profile saved to perf_report_${TIMESTAMP}.txt"
    fi
else
    warning "perf not found. Skipping CPU profiling."
fi

# 6. Collect system resource metrics
log "Collecting system resource metrics..."
cat > "$RESULTS_DIR/system_info_${TIMESTAMP}.txt" << EOF
=== System Information ===
Hostname: $(hostname)
OS: $(uname -s -r)
CPU: $(lscpu | grep "Model name" | cut -d: -f2 | xargs)
CPU Cores: $(nproc)
Total Memory: $(free -h | grep Mem | awk '{print $2}')
Available Memory: $(free -h | grep Mem | awk '{print $7}')
Disk Space: $(df -h / | tail -1 | awk '{print $4}')

=== Rust Toolchain ===
$(rustc --version)
$(cargo --version)

=== Build Profile ===
Profile: release
Features: memory-profiling, bottleneck-analysis
EOF

# 7. Generate summary report
log "Generating performance analysis summary..."
cat > "$RESULTS_DIR/ANALYSIS_SUMMARY_${TIMESTAMP}.md" << 'EOF'
# Phase 3 Performance Analysis Report

## Executive Summary

This report contains comprehensive performance analysis of the RipTide Phase 3 direct execution implementation.

## Test Environment

See `system_info_*.txt` for detailed system specifications.

## Performance Metrics

### Extraction Engine Comparison

| Engine    | Avg Time | Memory Peak | CPU Usage | Success Rate |
|-----------|----------|-------------|-----------|--------------|
| WASM      | TBD      | TBD         | TBD       | TBD          |
| Headless  | TBD      | TBD         | TBD       | TBD          |
| Stealth   | TBD      | TBD         | TBD       | TBD          |

### Memory Analysis

- **Peak Memory Usage**: TBD MB
- **Average Memory Usage**: TBD MB
- **Memory Growth Rate**: TBD MB/s
- **Potential Leaks**: TBD

### CPU Profiling

Top hotspots identified:
1. TBD
2. TBD
3. TBD

### Bottlenecks Identified

#### Critical (P0)
- TBD

#### High Priority (P1)
- TBD

#### Medium Priority (P2)
- TBD

## Optimization Recommendations

### Immediate Actions (P0)
1. TBD

### Short-term Improvements (P1)
1. TBD

### Long-term Optimizations (P2)
1. TBD

## Performance Baselines

These baselines will be used for regression testing:

- **WASM Extraction**: TBD ms/page
- **Headless Extraction**: TBD ms/page
- **Stealth Extraction**: TBD ms/page
- **Memory Footprint**: TBD MB
- **Throughput**: TBD pages/second

## Flamegraphs and Visualizations

See accompanying files:
- `massif_report_*.txt` - Memory profiling
- `perf_report_*.txt` - CPU profiling
- Profile logs in `profile_*.log` files

## Next Steps

1. Implement P0 optimizations
2. Create automated performance regression tests
3. Set up continuous performance monitoring
4. Optimize identified hot paths

---

**Analysis Date**: $(date +%Y-%m-%d)
**Analyst**: Performance Analyzer Agent (Hive Mind)
EOF

log "Summary report created at ANALYSIS_SUMMARY_${TIMESTAMP}.md"

# 8. Extract timing information from logs
log "Extracting timing metrics..."
python3 << 'PYTHON_SCRIPT' > "$RESULTS_DIR/timing_analysis_${TIMESTAMP}.json" || warning "Timing analysis script failed"
import json
import re
import glob
import os

results_dir = os.environ.get('RESULTS_DIR', 'docs/hive-mind/performance-results')
timestamp = os.environ.get('TIMESTAMP', 'unknown')

timing_data = {
    "timestamp": timestamp,
    "engines": {}
}

# Parse timing from profile logs
profile_logs = glob.glob(f"{results_dir}/profile_*_{timestamp}.log")
for log_file in profile_logs:
    engine = "unknown"
    if "wasm" in log_file:
        engine = "wasm"
    elif "headless" in log_file:
        engine = "headless"
    elif "stealth" in log_file:
        engine = "stealth"

    if engine not in timing_data["engines"]:
        timing_data["engines"][engine] = []

    try:
        with open(log_file, 'r') as f:
            content = f.read()
            # Look for timing patterns (real, user, sys from 'time' command)
            real_match = re.search(r'real\s+(\d+)m([\d.]+)s', content)
            if real_match:
                minutes = int(real_match.group(1))
                seconds = float(real_match.group(2))
                total_seconds = minutes * 60 + seconds
                timing_data["engines"][engine].append({
                    "file": os.path.basename(log_file),
                    "time_seconds": total_seconds
                })
    except Exception as e:
        print(f"Error processing {log_file}: {e}", file=sys.stderr)

print(json.dumps(timing_data, indent=2))
PYTHON_SCRIPT

log "Timing analysis saved to timing_analysis_${TIMESTAMP}.json"

# 9. Final summary
echo ""
echo -e "${GREEN}=== Performance Analysis Complete ===${NC}"
echo ""
echo "Results saved to: $RESULTS_DIR"
echo ""
echo "Key files:"
echo "  - ANALYSIS_SUMMARY_${TIMESTAMP}.md (main report)"
echo "  - timing_analysis_${TIMESTAMP}.json (timing metrics)"
echo "  - massif_report_${TIMESTAMP}.txt (memory profiling)"
echo "  - perf_report_${TIMESTAMP}.txt (CPU profiling)"
echo "  - profile_*.log (detailed extraction logs)"
echo ""
log "Review the analysis summary and implement recommended optimizations."
echo ""
