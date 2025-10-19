# Phase 4 Performance Benchmark Usage Guide

## Quick Start

### Run Full Validation Suite
```bash
# Using the convenience script (100 iterations)
./scripts/run-phase4-validation.sh

# Custom iterations and output path
./scripts/run-phase4-validation.sh 200 /tmp/my-results.json

# Direct cargo command
cargo run --release --bin phase4-validator -p riptide-performance -- \
  --iterations 100 \
  --output ./phase4-results.json
```

## Understanding the Benchmarks

### 1. Browser Pool Pre-warming
**What it measures**: Time to initialize browser instances
- **Baseline**: Cold start (no pool) - ~800-1000ms
- **Optimized**: Warm pool - ~200-300ms
- **Target**: 60-80% reduction

### 2. WASM AOT Compilation
**What it measures**: WASM module initialization time
- **Baseline**: JIT compilation - ~5000-6000μs
- **Optimized**: Cached AOT - ~1500-2000μs
- **Target**: 50-70% reduction

### 3. Adaptive Timeout
**What it measures**: Time wasted waiting for responses
- **Baseline**: Fixed 5000ms timeout - ~4100ms average waste
- **Optimized**: Adaptive timeout - ~500ms average waste
- **Target**: 30-50% reduction

### 4. Combined End-to-End
**What it measures**: Total extraction time with all optimizations
- **Baseline**: No optimizations - ~1200-1500ms
- **Optimized**: All enabled - ~400-600ms
- **Target**: 50-70% reduction

## Output Format

### JSON Results
```json
{
  "timestamp": "2025-10-17T08:30:00Z",
  "browser_pool": {
    "name": "Browser Pool Pre-warming",
    "baseline": {
      "mean": 850.0,
      "median": 850.0,
      "p95": 950.0,
      "p99": 1000.0,
      "std_dev": 100.0
    },
    "optimized": {
      "mean": 250.0,
      "median": 250.0,
      "p95": 280.0,
      "p99": 300.0,
      "std_dev": 30.0
    },
    "improvement": {
      "mean_reduction_pct": 70.6,
      "target_met": true
    }
  },
  "overall_verdict": {
    "all_passed": true
  }
}
```

### Console Output
```
🚀 Phase 4 Performance Validation
============================================================

🔍 Benchmarking browser pool pre-warming (100 iterations)...
  Progress: 10/100
  Progress: 20/100
  ...
  Progress: 100/100

📈 Browser Pool Pre-warming:
  Baseline:
    Mean: 850.00ms, Median: 850.00ms
    P95: 950.00ms, P99: 1000.00ms
  Optimized:
    Mean: 250.00ms, Median: 250.00ms
    P95: 280.00ms, P99: 300.00ms
  Improvement:
    Mean: 70.59%
    Target: 60-80%
    Status: ✅ PASSED

[... other benchmarks ...]

🎯 Overall Verdict:
  Browser Pool: ✅ PASSED
  WASM AOT: ✅ PASSED
  Adaptive Timeout: ✅ PASSED
  Combined: ✅ PASSED
  Memory: ✅ PASSED

  ALL PASSED: ✅ PASSED
============================================================
```

## Command Line Options

```bash
phase4-validator [OPTIONS]

OPTIONS:
  --iterations <N>    Number of iterations per benchmark (default: 100)
  --output <PATH>     Output JSON file path (default: ./phase4-results.json)
  --help              Show this help message
```

## Interpreting Results

### Statistical Metrics

- **Mean**: Average performance across all iterations
- **Median**: 50th percentile (middle value)
- **P95**: 95th percentile (95% of requests are faster)
- **P99**: 99th percentile (99% of requests are faster)
- **Std Dev**: Standard deviation (consistency measure)

### Performance Targets

Each benchmark has specific targets:
1. **Browser Pool**: 60-80% reduction (Lower bound: 60%, Upper: 80%)
2. **WASM AOT**: 50-70% reduction
3. **Adaptive Timeout**: 30-50% reduction
4. **Combined**: 50-70% overall reduction

A benchmark **PASSES** if the mean reduction falls within or exceeds the target range.

### Exit Codes

- **0**: All benchmarks passed ✅
- **1**: One or more benchmarks failed ❌

## CI/CD Integration

### GitHub Actions Example
```yaml
name: Performance Validation

on:
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 0 * * *'  # Daily at midnight

jobs:
  validate-performance:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Run Phase 4 Validation
        run: |
          ./scripts/run-phase4-validation.sh 100 results.json

      - name: Upload Results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: performance-results
          path: results.json

      - name: Check Regression
        run: |
          # Compare with baseline
          python scripts/check-regression.py results.json baseline.json
```

## Troubleshooting

### Benchmark Takes Too Long
- Reduce iterations: `--iterations 50`
- Run in release mode (already default in script)
- Check system resources (CPU, memory)

### Inconsistent Results
- High variance (std dev) indicates unstable environment
- Close other applications
- Run on dedicated hardware
- Increase iterations for better statistical significance

### Failed Benchmarks
- Check system resources (low memory, high CPU)
- Verify browser installation (for browser pool)
- Check network connectivity (for timeout tests)
- Review logs in `.swarm/memory.db`

## Advanced Usage

### Benchmark Individual Components
```rust
use riptide_performance::phase4_validation::benchmarks::Phase4BenchmarkSuite;

#[tokio::main]
async fn main() {
    let suite = Phase4BenchmarkSuite::new(100);

    // Run only browser pool benchmark
    let results = suite.benchmark_browser_pool().await;
    println!("{:#?}", results);
}
```

### Custom Statistical Analysis
```rust
use riptide_performance::phase4_validation::benchmarks::Statistics;

let measurements = vec![100.0, 200.0, 150.0, 180.0, 220.0];
let stats = Statistics::from_measurements(&measurements);

println!("Mean: {:.2}ms", stats.mean);
println!("P95: {:.2}ms", stats.p95);
```

## Performance Regression Detection

### Automated Monitoring
Set up alerts based on thresholds:
```bash
#!/bin/bash
# check-regression.sh

RESULTS="$1"
THRESHOLD=1.15  # 15% degradation threshold

P95=$(jq -r '.combined.optimized.p95' "$RESULTS")
BASELINE=550.0  # ms

if (( $(echo "$P95 > $BASELINE * $THRESHOLD" | bc -l) )); then
  echo "❌ Performance regression detected!"
  echo "P95: ${P95}ms exceeds baseline: ${BASELINE}ms by 15%"
  exit 1
fi

echo "✅ Performance within acceptable limits"
```

### Trend Analysis
Track performance over time:
```bash
# Store results with timestamp
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
./scripts/run-phase4-validation.sh 100 "results-${TIMESTAMP}.json"

# Analyze trend
python scripts/analyze-trend.py results-*.json
```

## Next Steps

After validation passes:
1. Review detailed report: `/workspaces/eventmesh/docs/hive-mind/phase4-performance-validation.md`
2. Merge changes to main branch
3. Deploy optimizations with feature flags
4. Monitor production metrics
5. Conduct Phase 5: Advanced features

## Support

For issues or questions:
- Review the detailed validation report
- Check logs: `.swarm/memory.db`
- Examine benchmark source: `crates/riptide-performance/src/phase4_validation/benchmarks.rs`
- Contact: Performance Analyzer Agent
