# P1-C1 Performance Benchmark Suite

Complete performance validation infrastructure for the HybridHeadlessLauncher with spider-chrome integration.

## Quick Start

```bash
# Run all benchmarks (interactive)
./scripts/run_all_benchmarks.sh

# Run Criterion benchmarks only
cargo bench --bench hybrid_launcher_benchmark

# Run specific load test
./scripts/load_test_1k.sh   # 1,000 sessions
./scripts/load_test_5k.sh   # 5,000 sessions
./scripts/load_test_10k.sh  # 10,000 sessions (stress test)
```

## Benchmark Structure

### 📊 Criterion Benchmarks (`/benches/hybrid_launcher_benchmark.rs`)

Automated statistical benchmarks using Criterion.rs:

1. **Session Lifecycle** - Creation/destruction with different stealth levels
2. **Page Load** - Performance across static, SPA, and heavy JS pages
3. **Stealth Overhead** - Impact of security features on performance
4. **Memory Profiling** - Resource usage and leak detection
5. **Concurrent Load** - Scaling from 10 to 1,000 concurrent sessions
6. **CDP Commands** - Chrome DevTools Protocol execution speed
7. **Pool Management** - Browser pool efficiency
8. **Content Generation** - Screenshot/PDF/HTML extraction speed
9. **Error Recovery** - Retry and resilience mechanisms

### 🚀 Load Test Scripts (`/scripts/`)

Comprehensive load testing at production scale:

- **load_test_1k.sh** - 1,000 sessions (production baseline)
- **load_test_5k.sh** - 5,000 sessions (heavy load)
- **load_test_10k.sh** - 10,000 sessions (stress test, graceful degradation)
- **run_all_benchmarks.sh** - Complete suite execution

### ⚙️ Configuration (`/benchmarks/baseline-config.toml`)

Performance baselines and acceptance thresholds:

- Target response times for all operations
- Memory and CPU limits
- Success rate requirements
- Regression detection thresholds
- System requirements for each test level

### 📖 Documentation (`/docs/benchmarks/`)

Detailed guides and references:

- **P1-C1-PERFORMANCE-VALIDATION.md** - Complete validation guide
- Usage instructions
- Expected results and baselines
- Troubleshooting guide

## Running Benchmarks

### Criterion Benchmarks

```bash
# Run all criterion benchmarks
cargo bench --bench hybrid_launcher_benchmark

# Run specific benchmark group
cargo bench --bench hybrid_launcher_benchmark -- session_lifecycle
cargo bench --bench hybrid_launcher_benchmark -- page_load
cargo bench --bench hybrid_launcher_benchmark -- stealth_overhead

# Generate HTML reports
cargo bench --bench hybrid_launcher_benchmark
open target/criterion/report/index.html
```

### Load Tests

```bash
# 1K sessions (recommended starting point)
./scripts/load_test_1k.sh

# 5K sessions (requires significant resources)
# Minimum: 64 CPU cores, 256GB RAM
./scripts/load_test_5k.sh

# 10K sessions (extreme stress test)
# Minimum: 128 CPU cores, 512GB RAM
./scripts/load_test_10k.sh
```

### Baseline Management

```bash
# Save current results as baseline
cargo bench --bench hybrid_launcher_benchmark -- --save-baseline p1c1-baseline

# Compare against baseline
cargo bench --bench hybrid_launcher_benchmark -- --baseline p1c1-baseline

# Compare two baselines
critcmp p1c1-baseline p1c1-v2
```

## Performance Targets

### Session Creation (by Stealth Level)

| Level   | Target  | P95     | P99     |
|---------|---------|---------|---------|
| Minimal | <50ms   | 45ms    | 48ms    |
| Medium  | <75ms   | 72ms    | 74ms    |
| High    | <100ms  | 95ms    | 98ms    |

### Page Load Performance

| Type      | Target  | P95     | P99     |
|-----------|---------|---------|---------|
| Static    | <500ms  | 456ms   | 489ms   |
| SPA       | <1.5s   | 1.4s    | 1.5s    |
| Heavy JS  | <3s     | 2.8s    | 2.9s    |

### Concurrent Load

| Sessions | Success Rate | Avg Response | Memory | CPU  |
|----------|--------------|--------------|--------|------|
| 1K       | >99%         | <2s          | <60GB  | <80% |
| 5K       | >98%         | <3s          | <250GB | <90% |
| 10K      | >95%         | <5s          | <500GB | <95% |

## System Requirements

### Minimum (1K sessions)

- **CPU:** 16+ cores
- **RAM:** 64GB
- **Storage:** 100GB SSD
- **Network:** 1Gbps

### Recommended for 5K

- **CPU:** 64+ cores
- **RAM:** 256GB
- **Storage:** 500GB SSD
- **Network:** 10Gbps

### Recommended for 10K

- **CPU:** 128+ cores
- **RAM:** 512GB
- **Storage:** 1TB SSD
- **Network:** 10Gbps

## Results

### Location

- **Criterion Reports:** `target/criterion/`
- **Load Test Results:** `benchmarks/results/`
- **Baselines:** `target/criterion/[baseline-name]/`

### Interpreting Results

**Criterion Output:**
```
session_lifecycle/minimal_stealth
                        time:   [35.234 ms 35.891 ms 36.112 ms]
                        change: [-2.3% +0.1% +1.8%] (p = 0.89 > 0.05)
                        No change in performance detected.
```

**Load Test Output:**
```
Success Rate: 99.5% (Target: >99%) ✓
Avg Response Time: 1.847s (Target: <2s) ✓
Peak Memory: 52.3GB (Target: <60GB) ✓
Avg CPU: 67.4% (Target: <80%) ✓
```

## Troubleshooting

### High Memory Usage

Reduce concurrent workers in load test scripts:
```bash
# Edit load_test_*.sh
CONCURRENT_WORKERS=25  # Reduced from 50
```

### Failed Benchmarks

Increase measurement time for more stable results:
```rust
// In benchmark file
group.measurement_time(Duration::from_secs(60));
```

### Inconsistent Results

1. Close other applications
2. Disable CPU frequency scaling
3. Increase sample size
4. Run multiple times and average

## CI Integration

### GitHub Actions Example

```yaml
- name: Run Performance Benchmarks
  run: |
    cargo bench --bench hybrid_launcher_benchmark -- --save-baseline current
    critcmp baseline current --threshold 10

- name: Upload Results
  uses: actions/upload-artifact@v3
  with:
    name: criterion-results
    path: target/criterion/
```

## Validation Checklist

P1-C1 acceptance requires:

- [ ] All session benchmarks within target times
- [ ] All page load benchmarks meet thresholds
- [ ] 1K load test passes (>99% success)
- [ ] 5K load test passes (>98% success)
- [ ] 10K stress test passes (>95% success, graceful degradation)
- [ ] No memory leaks detected
- [ ] Circuit breakers function correctly
- [ ] Auto-scaling responds appropriately
- [ ] Error recovery completes within SLA

## Files Created

```
/benches/
  ├── hybrid_launcher_benchmark.rs      # Main benchmark suite
  └── Cargo.toml                         # Benchmark dependencies

/benchmarks/
  ├── baseline-config.toml               # Performance baselines
  ├── README.md                          # This file
  └── results/                           # Test results (gitignored)

/scripts/
  ├── load_test_1k.sh                    # 1K session load test
  ├── load_test_5k.sh                    # 5K session load test
  ├── load_test_10k.sh                   # 10K stress test
  └── run_all_benchmarks.sh              # Complete suite runner

/docs/benchmarks/
  └── P1-C1-PERFORMANCE-VALIDATION.md    # Detailed guide
```

## Next Steps

1. **Run Initial Baseline:**
   ```bash
   ./scripts/run_all_benchmarks.sh
   ```

2. **Review Results:**
   ```bash
   open target/criterion/report/index.html
   cat benchmarks/results/performance_report_*.md
   ```

3. **Establish Monitoring:**
   - Set up alerts based on baseline thresholds
   - Configure CI to run benchmarks on PR

4. **Document Performance:**
   - Update roadmap with actual numbers
   - Add performance section to main README

## Support

For issues or questions:

1. Check `/docs/benchmarks/P1-C1-PERFORMANCE-VALIDATION.md`
2. Review existing benchmark code in `/tests/integration/spider_chrome_benchmarks.rs`
3. See Criterion.rs docs: https://bheisler.github.io/criterion.rs/book/

---

**P1-C1 Performance Validation Suite** - Ready for Week 2 validation ✅
