# CI/CD Baseline Quality Gates

## Overview

This document describes the baseline quality gates implemented for the Riptide project CI/CD pipeline. These gates ensure consistent quality, performance, and reliability across all changes.

## Quality Gates

### 1. Test Suite Baseline

**Threshold**: 100% test pass rate (after fixing 7 environmental failures)

**How to Run Locally**:
```bash
cargo test --all --lib
```

**CI Behavior**:
- Runs on every PR and push to main
- Fails if any tests fail
- Caches dependencies for faster execution

### 2. Coverage Baseline

**Threshold**: 75% code coverage

**How to Run Locally**:
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --workspace --exclude-files "tests/*" --out Html --output-dir ./coverage

# Open coverage report
open coverage/index.html
```

**CI Behavior**:
- Generates coverage report with cargo-tarpaulin
- Uploads to Codecov
- Fails if coverage drops below 75%

### 3. Performance Regression Check

**Threshold**: No more than 10% performance regression

**How to Run Locally**:
```bash
# Create baseline
./scripts/run-benchmarks.sh week2-start

# Make changes...

# Compare to baseline
cargo bench --bench performance_benches -- --baseline week2-start
```

**CI Behavior**:
- Runs all benchmark suites
- Compares against main branch baseline
- Fails if any benchmark regresses > 10%

### 4. Build Performance Baseline

**Threshold**: Build time < 60 seconds

**How to Run Locally**:
```bash
# Clean build
cargo clean

# Measure build time
time cargo build --all --release
```

**CI Behavior**:
- Measures full release build time
- Fails if build exceeds 60s
- Monitors for build time regressions

### 5. Clippy Quality Baseline

**Threshold**: Zero clippy warnings (deny warnings)

**How to Run Locally**:
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

**CI Behavior**:
- Runs clippy on all targets
- Treats warnings as errors
- Fails on any quality issues

## Scripts

### run-benchmarks.sh

Executes all benchmark suites and saves results to a named baseline.

**Usage**:
```bash
./scripts/run-benchmarks.sh [baseline-name]

# Examples
./scripts/run-benchmarks.sh week2-start
./scripts/run-benchmarks.sh before-optimization
./scripts/run-benchmarks.sh production-candidate
```

**Features**:
- Runs all 5 benchmark suites
- Saves results to `./benchmarks/` directory
- Creates criterion baseline for comparisons
- Generates summary report

### load-test-pool.sh

Performs load testing on the browser pool API.

**Usage**:
```bash
./scripts/load-test-pool.sh [num-browsers] [num-requests] [concurrency]

# Examples
./scripts/load-test-pool.sh 5 100 2      # Small test
./scripts/load-test-pool.sh 20 1000 5    # Default test
./scripts/load-test-pool.sh 50 10000 10  # Large test
```

**Features**:
- Tests browser pool under load
- Measures latency and throughput
- Monitors memory usage
- Supports both `hey` tool and curl fallback

### monitor-health.sh

Comprehensive health monitoring for CI/CD.

**Usage**:
```bash
./scripts/monitor-health.sh
```

**Features**:
- Runs build, tests, and clippy
- Collects comprehensive metrics
- Saves JSON metrics to `./metrics/` directory
- Determines system health status
- Exits with error code if unhealthy

**Health Statuses**:
- `healthy`: All checks passing, minimal warnings
- `warning`: Excessive warnings (>10)
- `degraded`: Tests failing
- `unhealthy`: Build or clippy errors

## CI/CD Pipeline Overview

### Workflow: baseline-check.yml

Runs on every PR and push to main branch.

**Jobs**:

1. **test-baseline** (2-3 min)
   - Runs full test suite
   - Enforces 100% pass rate

2. **coverage-baseline** (3-5 min)
   - Generates coverage report
   - Uploads to Codecov
   - Enforces 75% threshold

3. **benchmark-regression** (5-10 min)
   - Runs all benchmarks
   - Compares to baseline
   - Enforces 10% regression limit

4. **build-baseline** (2-3 min)
   - Measures build time
   - Enforces 60s threshold

5. **clippy-baseline** (1-2 min)
   - Runs clippy checks
   - Enforces zero warnings

**Total Pipeline Time**: ~13-23 minutes

## Troubleshooting

### Tests Failing in CI but Pass Locally

**Common Causes**:
- Environmental differences (Chrome binary location)
- Missing dependencies
- Timing/concurrency issues

**Solutions**:
```bash
# Run with same settings as CI
cargo test --all --lib

# Check for flaky tests
cargo test -- --test-threads=1

# Run specific test multiple times
cargo test test_name -- --ignored --test-threads=1
```

### Benchmark Regression False Positives

**Common Causes**:
- CI runner variance
- Background processes
- Cold vs warm cache

**Solutions**:
- Run benchmarks multiple times
- Compare median instead of single run
- Use stricter baseline (15% instead of 10%)

### Coverage Report Not Generating

**Common Causes**:
- Tarpaulin not installed
- Exclude patterns too broad
- Binary targets included

**Solutions**:
```bash
# Reinstall tarpaulin
cargo install cargo-tarpaulin --force

# Run with verbose output
cargo tarpaulin --workspace --verbose
```

### Build Time Exceeding Threshold

**Common Causes**:
- Cache not restored
- New dependencies added
- Increased crate sizes

**Solutions**:
```bash
# Check cache status
ls -lh target/

# Profile build
cargo build --timings

# Use faster linker (lld)
echo '[target.x86_64-unknown-linux-gnu]
linker = "lld"' > .cargo/config.toml
```

## Metrics Dashboard

### Key Metrics to Monitor

1. **Test Pass Rate**
   - Target: 100%
   - Alert: Any failure

2. **Code Coverage**
   - Target: ≥75%
   - Alert: <70%

3. **Build Time**
   - Target: <60s
   - Alert: >90s

4. **Benchmark Performance**
   - Browser pool throughput: >100 pages/sec
   - Latency p50: <100ms
   - Latency p99: <500ms
   - Memory usage: <500MB per instance

5. **Pipeline Success Rate**
   - Target: ≥95%
   - Alert: <90%

### Setting Up Metrics Collection

```bash
# Run health check daily
crontab -e
0 9 * * * cd /path/to/eventmesh && ./scripts/monitor-health.sh

# Aggregate metrics
cat metrics/health-*.json | jq -s '.' > metrics/aggregate.json
```

## Future Enhancements

1. **Automated Regression Analysis**
   - Download main branch baseline
   - Statistical comparison
   - Detailed regression reports

2. **Performance Trending**
   - Track metrics over time
   - Visualize trends
   - Predict future issues

3. **Flaky Test Detection**
   - Identify unstable tests
   - Quarantine flaky tests
   - Re-run failed tests

4. **Parallel Test Execution**
   - Split tests across runners
   - Reduce pipeline time
   - Improve resource utilization

5. **Custom Metrics Dashboard**
   - Grafana integration
   - Real-time monitoring
   - Alert configuration

## References

- [Cargo Bench Documentation](https://doc.rust-lang.org/cargo/commands/cargo-bench.html)
- [Criterion.rs User Guide](https://bheisler.github.io/criterion.rs/book/)
- [Tarpaulin Coverage Tool](https://github.com/xd009642/tarpaulin)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
