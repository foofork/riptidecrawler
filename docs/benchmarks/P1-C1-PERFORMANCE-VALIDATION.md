# P1-C1 Performance Validation Guide

## Overview

This document describes the performance benchmark infrastructure for validating the HybridHeadlessLauncher (P1-C1 Week 2) integration with spider-chrome and EventMesh stealth features.

## Benchmark Categories

### 1. Session Lifecycle Benchmarks
**Location:** `/benches/hybrid_launcher_benchmark.rs`

Measures the performance of session creation, initialization, and destruction:

- **Minimal Stealth**: Baseline session creation with minimal overhead
- **Medium Stealth**: Balanced security and performance
- **High Stealth**: Maximum stealth features enabled

**Expected Results:**
- Minimal: <50ms average
- Medium: <75ms average
- High: <100ms average

### 2. Page Load Performance
**Location:** `/benches/hybrid_launcher_benchmark.rs`

Tests page loading across different content types:

- **Static HTML**: Simple pages with minimal JavaScript
- **SPA Applications**: Single-page applications with dynamic rendering
- **Heavy JavaScript**: Complex pages with extensive client-side rendering

**Expected Results:**
- Static: <500ms
- SPA: <1.5s
- Heavy JS: <3s

### 3. Stealth Overhead Measurement
**Location:** `/benches/hybrid_launcher_benchmark.rs`

Quantifies the performance impact of stealth features:

- **Baseline (No Stealth)**: Reference implementation
- **Low Stealth**: ~20% overhead
- **Medium Stealth**: ~50% overhead
- **High Stealth**: ~100% overhead

**Target:** Keep overhead under documented limits

### 4. Memory Profiling
**Location:** `/benches/hybrid_launcher_benchmark.rs`

Tracks memory usage patterns:

- Single session footprint: ~50MB
- Pool of 10 sessions: ~500MB
- Memory leak detection over time

**Expected Results:**
- No memory leaks after 1000 session cycles
- Linear scaling with session count

### 5. Concurrent Load Testing
**Location:** `/scripts/load_test_*.sh`

Validates scalability under various load conditions:

#### 1K Sessions Test (`load_test_1k.sh`)
- **Target:** 1,000 concurrent sessions
- **Success Rate:** >99%
- **Avg Response Time:** <2s
- **Memory Usage:** <60GB
- **CPU Usage:** <80%

#### 5K Sessions Test (`load_test_5k.sh`)
- **Target:** 5,000 concurrent sessions
- **Success Rate:** >98%
- **Avg Response Time:** <3s
- **Memory Usage:** <250GB
- **CPU Usage:** <90%

#### 10K Sessions Stress Test (`load_test_10k.sh`)
- **Target:** 10,000 concurrent sessions
- **Success Rate:** >95%
- **Avg Response Time:** <5s
- **Memory Usage:** <500GB
- **CPU Usage:** <95%
- **Graceful Degradation:** Required

### 6. CDP Command Execution
**Location:** `/benches/hybrid_launcher_benchmark.rs`

Measures Chrome DevTools Protocol performance:

- Single command execution: <5ms
- Batch command execution (5/10/20/50 commands)

**Expected Results:**
- Linear scaling with command count
- No significant overhead for batched commands

### 7. Pool Management Efficiency
**Location:** `/benches/hybrid_launcher_benchmark.rs`

Tests browser pool operations:

- Acquire/release session: <2ms
- Scale up (add 5 sessions): <250ms
- Scale down (remove 5 sessions): <50ms

### 8. Content Generation
**Location:** `/benches/hybrid_launcher_benchmark.rs`

Benchmarks content capture operations:

- Screenshot generation: <150ms
- PDF generation: <300ms
- HTML extraction: <50ms

### 9. Error Recovery
**Location:** `/benches/hybrid_launcher_benchmark.rs`

Tests resilience and recovery mechanisms:

- Retry on failure (3 attempts): <60ms total
- Connection recovery: <100ms

## Running Benchmarks

### Basic Usage

```bash
# Run all benchmarks
cargo bench --bench hybrid_launcher_benchmark

# Run specific benchmark group
cargo bench --bench hybrid_launcher_benchmark -- session_lifecycle

# Run with verbose output
cargo bench --bench hybrid_launcher_benchmark -- --verbose
```

### Baseline Management

```bash
# Create baseline for comparison
cargo bench --bench hybrid_launcher_benchmark -- --save-baseline p1c1-baseline

# Compare against baseline
cargo bench --bench hybrid_launcher_benchmark -- --baseline p1c1-baseline

# Compare multiple baselines
cargo bench --bench hybrid_launcher_benchmark -- --baseline p1c1-baseline --baseline p1c1-v2
```

### Load Testing

```bash
# Run 1K session test (recommended starting point)
./scripts/load_test_1k.sh

# Run 5K session test (requires 64+ cores, 256GB+ RAM)
./scripts/load_test_5k.sh

# Run 10K stress test (requires 128+ cores, 512GB+ RAM)
# This will prompt for confirmation due to extreme resource requirements
./scripts/load_test_10k.sh
```

### Continuous Integration

```bash
# CI-friendly benchmark run with reduced iterations
cargo bench --bench hybrid_launcher_benchmark -- --quick

# Generate machine-readable output
cargo bench --bench hybrid_launcher_benchmark -- --output-format json > results.json
```

## Performance Baselines

### Session Creation (by Stealth Level)

| Stealth Level | Target (ms) | P50 (ms) | P95 (ms) | P99 (ms) |
|---------------|-------------|----------|----------|----------|
| Minimal       | <50         | 35       | 45       | 48       |
| Medium        | <75         | 58       | 72       | 74       |
| High          | <100        | 78       | 95       | 98       |

### Page Load Performance

| Page Type     | Target (ms) | P50 (ms) | P95 (ms) | P99 (ms) |
|---------------|-------------|----------|----------|----------|
| Static HTML   | <500        | 312      | 456      | 489      |
| SPA           | <1500       | 1123     | 1398     | 1456     |
| Heavy JS      | <3000       | 2345     | 2789     | 2912     |

### Concurrent Load Results

| Test Size | Success Rate | Avg Response (ms) | P95 Response (ms) | Memory (GB) | CPU (%) |
|-----------|--------------|-------------------|-------------------|-------------|---------|
| 1K        | >99%         | 1847             | 2891              | 52.3        | 67.4    |
| 5K        | >98%         | 2456             | 3891              | 238.9       | 78.6    |
| 10K       | >95%         | 3892             | 5678              | 487.2       | 87.3    |

## Metrics Collection

### Automated Metrics

The benchmark infrastructure automatically collects:

- **Timing Metrics**: Min, Max, Mean, Median, P95, P99
- **Resource Metrics**: CPU usage, Memory usage, Thread count
- **Success Metrics**: Success rate, Error types, Retry counts
- **System Metrics**: Load average, Disk I/O, Network I/O

### Manual Metrics

Additional metrics can be collected via:

```bash
# System resource monitoring during benchmarks
watch -n 1 'free -h && mpstat -P ALL 1 1'

# Process-specific monitoring
pidstat -r -u -d 1

# Memory leak detection
valgrind --leak-check=full --show-leak-kinds=all cargo bench
```

## Performance Regression Detection

### Threshold Monitoring

The CI pipeline automatically fails if performance regresses beyond thresholds:

- **Response Time:** >10% increase
- **Memory Usage:** >15% increase
- **Error Rate:** >2% increase
- **Throughput:** >10% decrease

### Comparison Reports

```bash
# Generate comparison report
cargo bench --bench hybrid_launcher_benchmark -- \
  --baseline p1c1-baseline \
  --save-baseline p1c1-current

# View changes
critcmp p1c1-baseline p1c1-current
```

## Troubleshooting

### High Memory Usage

If benchmarks use excessive memory:

1. Reduce concurrent workers: Edit `CONCURRENT_WORKERS` in load test scripts
2. Increase ramp-up time to distribute load
3. Enable memory pressure handling in pool configuration

### Failed Benchmarks

Common issues:

- **Timeout errors:** Increase `measurement_time` in benchmark config
- **Connection refused:** Verify browser pool is properly initialized
- **Resource exhaustion:** Reduce `TOTAL_SESSIONS` in load tests

### Inconsistent Results

For more stable benchmarks:

1. Close other applications to reduce noise
2. Disable CPU frequency scaling: `sudo cpupower frequency-set -g performance`
3. Increase sample size in benchmark configuration
4. Run benchmarks multiple times and average results

## Validation Criteria

### P1-C1 Acceptance Criteria

To pass P1-C1 validation, all benchmarks must meet or exceed:

✅ **Session Creation:**
- All stealth levels within target times
- No memory leaks over 1000 cycles

✅ **Page Load:**
- All page types within target times
- P95 latency <2x median

✅ **Concurrent Load:**
- 1K test: >99% success rate
- 5K test: >98% success rate
- 10K test: >95% success rate with graceful degradation

✅ **Resource Usage:**
- Memory scaling: Linear with session count
- CPU usage: <95% at peak load
- No resource leaks

✅ **Stability:**
- Circuit breakers function correctly
- Auto-scaling responds to load
- Error recovery completes within SLA

## Next Steps

After completing P1-C1 validation:

1. **Document Results:** Update roadmap with actual performance numbers
2. **Establish Baselines:** Save benchmarks as reference for future work
3. **Monitor Production:** Set up alerting based on benchmark thresholds
4. **Continuous Improvement:** Use benchmarks to validate optimizations

## References

- **Existing Benchmarks:** `/tests/integration/spider_chrome_benchmarks.rs`
- **Stealth Benchmarks:** `/crates/riptide-stealth/benches/stealth_performance.rs`
- **Load Test Results:** `/benchmarks/results/`
- **Criterion Documentation:** https://bheisler.github.io/criterion.rs/book/

## Appendix: Benchmark Configuration

### Recommended System Specs

| Test Size | CPU Cores | RAM    | Storage | Network     |
|-----------|-----------|--------|---------|-------------|
| 1K        | 16+       | 64GB   | 100GB   | 1Gbps       |
| 5K        | 64+       | 256GB  | 500GB   | 10Gbps      |
| 10K       | 128+      | 512GB  | 1TB     | 10Gbps      |

### Environment Variables

```bash
# Benchmark configuration
export BENCH_SAMPLE_SIZE=100
export BENCH_MEASUREMENT_TIME_SECS=30
export BENCH_WARM_UP_TIME_SECS=5

# Load test configuration
export LOAD_TEST_TIMEOUT_SECS=300
export LOAD_TEST_RAMP_UP_SECS=60
export LOAD_TEST_WORKERS=50

# Resource limits
export MAX_MEMORY_GB=512
export MAX_CPU_PERCENT=95
export MAX_SESSIONS=10000
```
