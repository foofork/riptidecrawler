# RipTide Performance Benchmarks

Comprehensive performance benchmark suite for RipTide components.

## 📁 Benchmark Suites

### 1. `hybrid_launcher_benchmark.rs` (P1-C1 Validation)
**374 lines | 9 benchmark groups**

Validates HybridHeadlessLauncher performance with spider-chrome integration:
- ✅ Session lifecycle (create, destroy)
- ✅ Page load performance (static, SPA, heavy JS)
- ✅ Stealth overhead (None, Low, Medium, High)
- ✅ Memory profiling (single session, pool allocation)
- ✅ Concurrent load (10-1000 sessions)
- ✅ CDP command execution (single, batch)
- ✅ Pool management (acquire/release, scaling)
- ✅ Content generation (screenshots, PDFs)
- ✅ Error recovery (retry, connection recovery)

### 2. `facade_benchmark.rs` (P1-C3 Validation)
**548 lines | 12 benchmark groups**

Validates high-level facade API performance:

#### BrowserFacade (3 groups)
- ✅ Lifecycle: create, navigate, get content, execute JS
- ✅ Stealth: none, low, medium, high configurations
- ✅ Capture: screenshots (full/viewport), PDF generation

#### ExtractionFacade (3 groups)
- ✅ Parsing: simple/complex HTML, CSS selectors, XPath
- ✅ Patterns: JSON-LD, microdata, tables, links, metadata
- ✅ Scalability: 1KB → 10MB content sizes

#### ScraperFacade (3 groups)
- ✅ Crawling: single page, depth 2-10
- ✅ Concurrency: 1-50 parallel crawls
- ✅ Rate limiting: none, 10 RPS, 5 RPS, 1 RPS

#### Combined & Error Handling (3 groups)
- ✅ Workflows: navigate+extract+transform, crawl+extract+store
- ✅ Error handling: success, retry, timeout
- ✅ Resource cleanup: single facade, all facades

### 3. `performance_benchmarks.rs`
**370 lines**

General RipTide performance benchmarks (existing).

### 4. `wasm_performance.rs`
**392 lines**

WASM-specific performance validation (existing).

## 🚀 Quick Start

```bash
# Run ALL benchmarks
cargo bench

# Run specific suite
cargo bench --bench hybrid_launcher_benchmark
cargo bench --bench facade_benchmark

# Run specific group
cargo bench --bench facade_benchmark -- browser_facade
cargo bench --bench hybrid_launcher_benchmark -- stealth_overhead
```

## 📊 Performance Targets

### P1-C1 HybridHeadlessLauncher

| Metric | Target |
|--------|--------|
| Session creation (no stealth) | < 50ms |
| Session creation (high stealth) | < 100ms |
| Stealth overhead | < 50% |
| Memory per session | < 100MB |
| Concurrent sessions (1K) | < 5s |
| Pool acquisition | < 5ms |

### P1-C3 Facade APIs

| Metric | Target |
|--------|--------|
| BrowserFacade creation | < 100ms |
| BrowserFacade navigation | < 300ms |
| ExtractionFacade parse (100KB) | < 100ms |
| ScraperFacade (single page) | < 500ms |
| ScraperFacade (50 concurrent) | < 10s |
| Combined workflow | < 500ms |

## 📈 Advanced Usage

```bash
# Create baseline for comparison
cargo bench -- --save-baseline p1c1-baseline

# Compare against baseline
cargo bench -- --baseline p1c1-baseline

# Run with verbose output
cargo bench -- --verbose

# Filter benchmarks
cargo bench -- stealth
cargo bench -- concurrent

# Custom sample sizes
cargo bench -- --sample-size 100

# Export results
cargo bench -- --output-format csv > results.csv
```

## 📚 Documentation

See [/docs/hive/benchmark-guide.md](/docs/hive/benchmark-guide.md) for comprehensive documentation including:
- Detailed usage examples
- Interpreting results
- CI/CD integration
- Troubleshooting
- Performance monitoring

## 🔗 Related

- [P1-C1 Completion Plan](/docs/hive/p1-c1-completion-plan.md)
- [Validation Report](/docs/hive/validation-report.md)
- [Researcher Findings](/docs/hive/researcher-findings.md)

---

**Total:** 1,684 lines | 24+ benchmark groups | 100+ individual benchmarks
