# RipTide Performance Benchmark Suite Guide

## 📊 Overview

This comprehensive guide covers all performance benchmarks for the RipTide project, focusing on P1-C1 (HybridHeadlessLauncher) and P1-C3 (Facade APIs) validation.

## 🎯 Quick Start

```bash
# Run ALL benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench hybrid_launcher_benchmark
cargo bench --bench facade_benchmark
cargo bench --bench performance_benchmarks
cargo bench --bench wasm_performance

# Run specific benchmark group
cargo bench --bench hybrid_launcher_benchmark -- session_lifecycle
cargo bench --bench facade_benchmark -- browser_facade
```

## 📁 Benchmark Structure

```
benches/
├── Cargo.toml                          # Benchmark configuration
├── hybrid_launcher_benchmark.rs        # P1-C1: HybridHeadlessLauncher benchmarks
├── facade_benchmark.rs                 # P1-C3: Facade API benchmarks
├── performance_benchmarks.rs           # General performance benchmarks
└── wasm_performance.rs                 # WASM-specific benchmarks
```

## 🧪 Benchmark Suites

### 1. HybridHeadlessLauncher Benchmarks (P1-C1)

**File:** `benches/hybrid_launcher_benchmark.rs`

**Categories:**
- **Session Lifecycle**: Creation, initialization, destruction
- **Page Load Performance**: Static HTML, SPA, heavy JavaScript
- **Stealth Overhead**: None, Low, Medium, High presets
- **Memory Profiling**: Single session, pool allocation
- **Concurrent Load**: 10, 50, 100, 500, 1K sessions
- **CDP Commands**: Single and batch execution
- **Pool Management**: Acquire/release, scaling
- **Content Generation**: Screenshots, PDFs, HTML extraction
- **Error Recovery**: Retry logic, connection recovery

**Usage Examples:**

```bash
# Run all HybridHeadlessLauncher benchmarks
cargo bench --bench hybrid_launcher_benchmark

# Run specific category
cargo bench --bench hybrid_launcher_benchmark -- session_lifecycle
cargo bench --bench hybrid_launcher_benchmark -- stealth_overhead
cargo bench --bench hybrid_launcher_benchmark -- memory_profiling
cargo bench --bench hybrid_launcher_benchmark -- concurrent_load

# Create baseline for comparison
cargo bench --bench hybrid_launcher_benchmark -- --save-baseline p1c1-baseline

# Compare against baseline
cargo bench --bench hybrid_launcher_benchmark -- --baseline p1c1-baseline

# Run with verbose output
cargo bench --bench hybrid_launcher_benchmark -- --verbose

# Generate detailed HTML report
cargo bench --bench hybrid_launcher_benchmark -- --plotting-backend plotters
```

**Key Metrics:**
- Session creation time: Target < 100ms
- Stealth overhead: Target < 50% increase
- Memory per session: Target < 100MB
- Concurrent sessions: Target 1K+ simultaneous
- Pool acquisition: Target < 5ms

### 2. Facade API Benchmarks (P1-C3)

**File:** `benches/facade_benchmark.rs`

**Categories:**

#### BrowserFacade Benchmarks
- Lifecycle: Create, navigate, get content
- Stealth configurations: None, Low, Medium, High
- Capture: Screenshots (full/viewport), PDF generation

#### ExtractionFacade Benchmarks
- Parsing: Simple/complex HTML, CSS selectors, XPath
- Pattern extraction: JSON-LD, microdata, tables, links
- Scalability: 1KB, 10KB, 100KB, 1MB, 10MB content

#### ScraperFacade Benchmarks
- Crawling: Single page, multi-depth (2, 3, 5, 10 levels)
- Concurrency: 1, 5, 10, 25, 50 parallel crawls
- Rate limiting: None, 10 RPS, 5 RPS, 1 RPS

#### Combined Workflows
- Navigate → Extract → Transform
- Crawl → Extract → Store
- Multi-page extraction

#### Error Handling & Cleanup
- Successful operations (baseline)
- Retry logic overhead
- Timeout handling
- Resource cleanup

**Usage Examples:**

```bash
# Run all facade benchmarks
cargo bench --bench facade_benchmark

# Run specific facade category
cargo bench --bench facade_benchmark -- browser_facade
cargo bench --bench facade_benchmark -- extraction_facade
cargo bench --bench facade_benchmark -- scraper_facade
cargo bench --bench facade_benchmark -- combined

# Run specific benchmark
cargo bench --bench facade_benchmark -- browser_facade/lifecycle
cargo bench --bench facade_benchmark -- extraction_facade/scalability
cargo bench --bench facade_benchmark -- scraper_facade/concurrency

# Baseline and comparison
cargo bench --bench facade_benchmark -- --save-baseline p1c3-baseline
cargo bench --bench facade_benchmark -- --baseline p1c3-baseline
```

**Key Metrics:**
- BrowserFacade navigation: Target < 300ms
- ExtractionFacade parsing: Target < 100ms for 100KB
- ScraperFacade throughput: Target 10+ pages/sec
- Combined workflow: Target < 500ms end-to-end

### 3. General Performance Benchmarks

**File:** `benches/performance_benchmarks.rs`

General RipTide performance benchmarks (existing suite).

```bash
cargo bench --bench performance_benchmarks
```

### 4. WASM Performance Benchmarks

**File:** `benches/wasm_performance.rs`

WASM-specific performance validation.

```bash
cargo bench --bench wasm_performance
```

## 📈 Interpreting Results

### Criterion Output

```
session_lifecycle/create_session_minimal_stealth
                        time:   [48.123 ms 49.456 ms 50.789 ms]
                        change: [-5.2% -3.1% -1.0%] (p = 0.01 < 0.05)
                        Performance has improved.
```

**Understanding the metrics:**
- **time**: [lower_bound median upper_bound]
- **change**: Performance delta vs. previous run
- **p-value**: Statistical significance (< 0.05 is significant)

### HTML Reports

Generated in `target/criterion/<benchmark_name>/report/index.html`

```bash
# View reports
open target/criterion/session_lifecycle/report/index.html
open target/criterion/browser_facade/report/index.html
```

## 🎛️ Advanced Usage

### Custom Sample Sizes

```bash
# More samples for stable results (slower)
cargo bench --bench hybrid_launcher_benchmark -- --sample-size 100

# Fewer samples for quick feedback (faster)
cargo bench --bench hybrid_launcher_benchmark -- --sample-size 10
```

### Measurement Time

```bash
# Longer measurement for accurate results
cargo bench --bench facade_benchmark -- --measurement-time 60

# Shorter for quick iteration
cargo bench --bench facade_benchmark -- --measurement-time 10
```

### Filtering Benchmarks

```bash
# Run all benchmarks containing "stealth"
cargo bench -- stealth

# Run all benchmarks containing "concurrent" or "concurrency"
cargo bench -- concurr

# Run specific benchmark by exact name
cargo bench --bench hybrid_launcher_benchmark -- "session_lifecycle/create_session_minimal_stealth"
```

### Baseline Management

```bash
# Save baseline
cargo bench -- --save-baseline my-baseline

# Compare against baseline
cargo bench -- --baseline my-baseline

# List available baselines
ls target/criterion/*/base/

# Delete baseline
rm -rf target/criterion/*/my-baseline/
```

### Export Formats

```bash
# Generate CSV output
cargo bench -- --output-format csv > benchmark_results.csv

# Generate JSON output
cargo bench -- --output-format json > benchmark_results.json
```

## 🔍 Performance Targets

### P1-C1 HybridHeadlessLauncher

| Metric | Target | Baseline | Status |
|--------|--------|----------|--------|
| Session creation (no stealth) | < 50ms | TBD | 🟡 |
| Session creation (high stealth) | < 100ms | TBD | 🟡 |
| Stealth overhead | < 50% | TBD | 🟡 |
| Memory per session | < 100MB | TBD | 🟡 |
| Concurrent sessions (1K) | < 5s setup | TBD | 🟡 |
| Pool acquisition | < 5ms | TBD | 🟡 |
| Screenshot (viewport) | < 150ms | TBD | 🟡 |
| PDF generation | < 400ms | TBD | 🟡 |

### P1-C3 Facade APIs

| Metric | Target | Baseline | Status |
|--------|--------|----------|--------|
| BrowserFacade creation | < 100ms | TBD | 🟡 |
| BrowserFacade navigation | < 300ms | TBD | 🟡 |
| ExtractionFacade parse (100KB) | < 100ms | TBD | 🟡 |
| ExtractionFacade parse (1MB) | < 500ms | TBD | 🟡 |
| ScraperFacade (single page) | < 500ms | TBD | 🟡 |
| ScraperFacade (50 concurrent) | < 10s | TBD | 🟡 |
| Combined workflow | < 500ms | TBD | 🟡 |

## 📊 Continuous Integration

### GitHub Actions Integration

```yaml
# .github/workflows/benchmarks.yml
name: Performance Benchmarks

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run benchmarks
        run: cargo bench --all

      - name: Store benchmark results
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: target/criterion/results.json
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true
```

### Running in CI

```bash
# Fast CI profile
cargo bench --profile ci

# Reduced samples for speed
cargo bench -- --sample-size 10 --measurement-time 5
```

## 🛠️ Troubleshooting

### Benchmarks Running Too Slowly

```bash
# Reduce sample size and measurement time
cargo bench -- --sample-size 10 --measurement-time 10

# Run specific quick benchmarks
cargo bench --bench hybrid_launcher_benchmark -- session_lifecycle
```

### Inconsistent Results

```bash
# Increase sample size
cargo bench -- --sample-size 200 --measurement-time 60

# Reduce system noise
sudo nice -n -20 cargo bench

# Disable CPU frequency scaling (Linux)
sudo cpupower frequency-set --governor performance
```

### Memory Issues

```bash
# Reduce concurrent benchmarks
cargo bench --bench hybrid_launcher_benchmark -- concurrent_load -- --sample-size 5

# Monitor memory during benchmarks
watch -n 1 'ps aux | grep criterion'
```

## 📚 Resources

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [RipTide P1-C1 Roadmap](/docs/hive/p1-c1-completion-plan.md)
- [RipTide Architecture](/docs/architecture/)

## 🚀 Next Steps

1. **Establish Baselines**
   ```bash
   cargo bench -- --save-baseline p1c1-initial
   cargo bench -- --save-baseline p1c3-initial
   ```

2. **Run Regular Comparisons**
   ```bash
   cargo bench -- --baseline p1c1-initial
   cargo bench -- --baseline p1c3-initial
   ```

3. **Monitor Performance Trends**
   - Track metrics over time
   - Identify regressions early
   - Optimize hot paths

4. **Integrate with CI/CD**
   - Automated benchmark runs
   - Performance regression detection
   - Historical trend analysis

## 📞 Support

For benchmark-related questions or issues:
- Open an issue: GitHub Issues
- Documentation: `/docs/hive/`
- Roadmap: `/docs/hive/p1-c1-completion-plan.md`

---

**Last Updated:** 2025-10-19
**Version:** 1.0.0
**Status:** ✅ Complete
