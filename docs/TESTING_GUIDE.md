# RipTide Testing Guide - Comprehensive Testing Strategy

## 📋 Overview

RipTide RipTide maintains comprehensive testing across all components with 75% coverage targeting 80%. This consolidated guide covers all testing strategies, configuration, and quality metrics.

**Current Status**: ✅ **Production-ready** with comprehensive test suite validation

## 🧪 Testing Architecture

### Test Organization
```
tests/
├── unit/                    # Component-level unit tests
├── integration/             # End-to-end integration tests
├── golden/                  # Golden file validation tests
├── benchmarks/             # Performance benchmarks
├── wasm/                   # WASM-specific integration tests
└── fixtures/               # Test data and mock responses
```

### Test Categories

#### 1. Unit Tests
- **Coverage**: 75% current, targeting 80%
- **Focus**: Component isolation, error conditions, edge cases
- **Tools**: `cargo test`, property-based testing
- **Location**: Each crate's `tests/` directory

#### 2. Integration Tests
- **Coverage**: End-to-end API workflows
- **Focus**: Component interaction, real dependencies
- **Tools**: TestContainers for Redis, mock HTTP servers
- **Location**: `/tests/integration/`

#### 3. Golden Tests
- **Coverage**: Content extraction validation
- **Focus**: Regression testing, output consistency
- **Tools**: Offline fixtures, expected JSON outputs
- **Location**: `/tests/golden/`

#### 4. Performance Tests
- **Coverage**: Latency, throughput, resource usage
- **Focus**: p50/p95/p99 targets, memory limits
- **Tools**: `criterion`, custom benchmarks
- **Location**: `/benches/`

## 🔬 Test Strategies by Component

### WASM Extraction Testing
```rust
// tests/wasm/wasm_extractor_integration.rs
#[tokio::test]
async fn test_comprehensive_extraction() {
    let extractor = setup_wasm_extractor().await;

    // Test enhanced extraction features
    let result = extractor.extract(SAMPLE_HTML, ExtractionMode::Article).await?;

    // Validate new extraction capabilities
    assert!(!result.links.is_empty(), "Should extract links with rel attributes");
    assert!(result.language.is_some(), "Should detect content language");
    assert!(!result.categories.is_empty(), "Should categorize content");
    assert!(result.quality_score > 0.0, "Should calculate quality score");

    // Memory limit testing
    test_memory_limits(&extractor).await?;

    // Circuit breaker testing
    test_circuit_breaker(&extractor).await?;
}
```

### PDF Processing Testing
```rust
// tests/integration/pdf_processing.rs
#[tokio::test]
async fn test_pdf_pipeline_integration() {
    let processor = setup_pdf_processor().await;

    // Test with various PDF sizes
    let test_cases = vec![
        ("small.pdf", 1_000_000),      // 1MB
        ("medium.pdf", 10_000_000),    // 10MB
        ("large.pdf", 100_000_000),    // 100MB
    ];

    for (filename, expected_size) in test_cases {
        let pdf_data = load_test_pdf(filename);
        let mut progress_updates = vec![];

        let result = processor.process_with_progress(
            pdf_data,
            |update| {
                progress_updates.push(update);
                Ok(())
            }
        ).await?;

        // Validate results
        assert!(result.text.len() > 0, "Should extract text content");
        assert!(!progress_updates.is_empty(), "Should report progress");

        // Memory usage validation
        let memory_usage = get_current_memory_usage();
        assert!(memory_usage < 200_000_000, "Memory should stay under 200MB");
    }
}
```

### API Endpoint Testing
```rust
// tests/integration/api_endpoints.rs
#[tokio::test]
async fn test_crawl_endpoint_comprehensive() {
    let app = setup_test_app().await;

    let request_body = json!({
        "urls": [
            "https://example.com/article",
            "https://example.com/product",
            "https://example.com/news",
            "https://example.com/blog",
            "https://example.com/docs"
        ],
        "options": {
            "concurrency": 3,
            "cache_mode": "read_through",
            "extraction_mode": "article"
        }
    });

    let response = app
        .post("/crawl")
        .json(&request_body)
        .send()
        .await?;

    assert_eq!(response.status(), 200);

    let results: CrawlResponse = response.json().await?;
    assert_eq!(results.results.len(), 5);

    // Validate extraction quality
    for result in results.results {
        assert!(result.content.is_some(), "Should extract content");
        assert!(result.links.len() > 0, "Should extract links");
        assert!(result.metadata.is_some(), "Should extract metadata");
    }

    // Performance validation
    assert!(results.total_time_ms < 5000, "Should complete within 5s");
}
```

### Health and Monitoring Testing
```rust
// tests/integration/monitoring.rs
#[tokio::test]
async fn test_health_endpoints() {
    let app = setup_test_app().await;

    // Basic health check
    let health = app.get("/health").send().await?;
    assert_eq!(health.status(), 200);

    let health_data: HealthResponse = health.json().await?;
    assert_eq!(health_data.status, "healthy");
    assert!(health_data.components.contains_key("redis"));
    assert!(health_data.components.contains_key("wasm_extractor"));

    // Detailed metrics
    let metrics = app.get("/metrics").send().await?;
    assert_eq!(metrics.status(), 200);

    let metrics_text = metrics.text().await?;
    assert!(metrics_text.contains("riptide_wasm_memory_pages"));
    assert!(metrics_text.contains("http_requests_total"));
    assert!(metrics_text.contains("http_request_duration_seconds"));
}
```

## 🎯 Performance Testing

### Benchmark Configuration
```rust
// benches/api_performance.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_crawl_performance(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let app = rt.block_on(setup_test_app());

    c.bench_function("crawl_10_urls", |b| {
        b.iter(|| {
            rt.block_on(async {
                let response = app
                    .post("/crawl")
                    .json(&json!({
                        "urls": generate_test_urls(10),
                        "options": {"concurrency": 5}
                    }))
                    .send()
                    .await
                    .unwrap();

                assert_eq!(response.status(), 200);
            })
        })
    });
}

criterion_group!(benches, bench_crawl_performance);
criterion_main!(benches);
```

### Performance Targets
```yaml
Performance Benchmarks:
  api_latency:
    p50: "≤1.5s"        # 50th percentile for 10-URL mixed requests
    p95: "≤5.0s"        # 95th percentile SLA target
    p99: "≤8.0s"        # 99th percentile maximum

  throughput:
    concurrent_users: "≥50"    # Concurrent request handling
    requests_per_second: "≥20"  # Sustained throughput

  memory_usage:
    baseline: "50-100MB"       # Idle system memory
    peak_processing: "≤300MB"  # Maximum during operations
    pdf_worker: "≤200MB"       # Per-PDF processing limit

  wasm_performance:
    extraction_time: "≤100ms"  # WASM extraction latency
    cold_start: "≤15ms"        # After AOT cache warmup
    simd_improvement: "10-25%" # SIMD optimization gain
```

### Load Testing
```bash
#!/bin/bash
# scripts/load-test.sh

echo "Starting RipTide Load Testing..."

# Start application
cargo run --release --bin riptide-api &
APP_PID=$!
sleep 5

# Load test with different scenarios
echo "Testing concurrent crawl requests..."
for i in {1..50}; do
    curl -X POST http://localhost:3000/crawl \
        -H "Content-Type: application/json" \
        -d '{
            "urls": [
                "https://httpbin.org/html",
                "https://httpbin.org/json",
                "https://httpbin.org/xml"
            ],
            "options": {"concurrency": 3}
        }' &
done

wait

echo "Testing streaming endpoints..."
for i in {1..20}; do
    curl -X POST http://localhost:3000/deepsearch \
        -H "Content-Type: application/json" \
        -d '{"query": "test search", "limit": 5}' \
        --no-buffer &
done

wait

# Cleanup
kill $APP_PID
echo "Load testing complete"
```

## 🧰 Test Configuration

### Environment Setup
```bash
# Test environment variables
export RUST_TEST_THREADS=4
export RUST_BACKTRACE=1
export TEST_REDIS_URL="redis://localhost:6379/15"
export TEST_HEADLESS_URL="http://localhost:9124"
export TEST_LOG_LEVEL="debug"

# Test feature flags
export TEST_ENABLE_WASM=true
export TEST_ENABLE_PDF=true
export TEST_ENABLE_STREAMING=true
export TEST_ENABLE_SPIDER=true
```

### Test Data Management
```rust
// tests/common/mod.rs
pub struct TestDataManager {
    fixtures_dir: PathBuf,
    temp_dir: TempDir,
}

impl TestDataManager {
    pub fn load_html_fixture(&self, name: &str) -> String {
        let path = self.fixtures_dir.join("html").join(format!("{}.html", name));
        std::fs::read_to_string(path).expect("Failed to load HTML fixture")
    }

    pub fn load_expected_json(&self, name: &str) -> serde_json::Value {
        let path = self.fixtures_dir.join("expected").join(format!("{}.json", name));
        let content = std::fs::read_to_string(path).expect("Failed to load expected JSON");
        serde_json::from_str(&content).expect("Failed to parse expected JSON")
    }

    pub fn create_test_pdf(&self, size_mb: usize) -> Vec<u8> {
        // Generate test PDF of specified size
        generate_test_pdf_content(size_mb)
    }
}
```

### Golden Test Framework
```rust
// tests/golden/mod.rs
pub fn run_golden_test(test_name: &str, input: &str, extractor: &WasmExtractor) {
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(extractor.extract(input, ExtractionMode::Article))
        .expect("Extraction should succeed");

    let expected_path = format!("tests/golden/expected/{}.json", test_name);
    let expected: ExtractionResult = serde_json::from_str(
        &std::fs::read_to_string(expected_path).expect("Expected file should exist")
    ).expect("Expected JSON should be valid");

    // Compare key fields with tolerance for timestamps
    assert_eq!(result.title, expected.title, "Title should match");
    assert_eq!(result.content, expected.content, "Content should match");
    assert_eq!(result.links.len(), expected.links.len(), "Link count should match");

    // Quality score should be within 10% tolerance
    let score_diff = (result.quality_score - expected.quality_score).abs();
    assert!(score_diff < 0.1, "Quality score should be within tolerance");
}
```

## 📊 Coverage and Quality Metrics

### Coverage Reporting
```bash
# Generate coverage report using llvm-cov
cargo llvm-cov --html --open

# Coverage by component
cargo llvm-cov --package riptide-core
cargo llvm-cov --package riptide-api
cargo llvm-cov --package riptide-headless
cargo llvm-cov --package riptide-workers

# Integration test coverage
cargo llvm-cov --tests --package riptide-api
```

### Current Coverage Status
```yaml
Test Coverage by Component:
  riptide-core: "82%"        # Target: 85%
  riptide-api: "78%"         # Target: 80%
  riptide-headless: "71%"    # Target: 75%
  riptide-workers: "69%"     # Target: 75%
  wasm-extractor: "85%"      # Target: 90%

Overall Coverage: "75%"      # Target: 80%

Quality Metrics:
  panic_free_production: "94.3%"    # Target: 98%
  clippy_warnings: "218"            # Target: <100 (non-production)
  compilation_errors: "0"           # Target: 0 ✅
  test_execution_time: "45s"        # Target: <60s
```

### Test Quality Checklist
- [ ] ✅ All critical paths covered by tests
- [ ] ✅ Error conditions tested comprehensively
- [ ] ✅ Integration tests cover major workflows
- [ ] ✅ Performance tests validate SLA targets
- [ ] ✅ Golden tests prevent regressions
- [ ] ✅ WASM functionality thoroughly tested
- [ ] ✅ Circuit breaker behavior validated
- [ ] ✅ Memory limits enforced in tests
- [ ] ⚠️ Coverage target 80% (currently 75%)
- [ ] ⚠️ Remaining unwrap() calls in test code

## 🚀 Continuous Integration

### CI Test Pipeline
```yaml
# .github/workflows/test.yml
name: Test Suite
on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          components: llvm-tools-preview

      - name: Install dependencies
        run: |
          cargo install cargo-llvm-cov

      - name: Run tests with coverage
        run: |
          cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: lcov.info

  integration-tests:
    runs-on: ubuntu-latest
    services:
      redis:
        image: redis:7-alpine
        ports:
          - 6379:6379
    steps:
      - name: Run integration tests
        run: cargo test --test integration --all-features

  performance-tests:
    runs-on: ubuntu-latest
    steps:
      - name: Run benchmarks
        run: cargo bench --all-features

      - name: Performance regression check
        run: |
          # Compare against baseline performance metrics
          ./scripts/check-performance-regression.sh
```

### Quality Gates
```bash
#!/bin/bash
# scripts/quality-check.sh

echo "Running RipTide Quality Gates..."

# 1. Code compilation
echo "✓ Checking compilation..."
cargo check --all-features --all-targets
if [ $? -ne 0 ]; then
    echo "❌ Compilation failed"
    exit 1
fi

# 2. Test coverage
echo "✓ Checking test coverage..."
COVERAGE=$(cargo llvm-cov --summary-only | grep -o '[0-9]*\.[0-9]*%' | head -n1 | sed 's/%//')
if (( $(echo "$COVERAGE < 75.0" | bc -l) )); then
    echo "❌ Coverage $COVERAGE% below minimum 75%"
    exit 1
fi

# 3. Performance benchmarks
echo "✓ Running performance benchmarks..."
cargo bench --all-features > bench_results.txt
if ! grep -q "time.*1.5" bench_results.txt; then
    echo "❌ Performance regression detected"
    exit 1
fi

# 4. Memory leak detection
echo "✓ Checking for memory leaks..."
cargo test --test integration -- --ignored memory_leak_test
if [ $? -ne 0 ]; then
    echo "❌ Memory leak detected"
    exit 1
fi

echo "✅ All quality gates passed"
```

## 🔗 Related Documentation

### Source Files
- **Test Configuration**: This guide consolidates:
  - `test_configuration_guide.md` - Test setup and configuration
  - `test_analysis.md` - Coverage and quality analysis
  - `testing_strategy_comprehensive.md` - Testing methodology
  - `test-suite-overview.md` - Test architecture overview

### Implementation Files
- **Unit Tests**: Each crate's `src/` and `tests/` directories
- **Integration Tests**: `/tests/integration/`
- **Golden Tests**: `/tests/golden/`
- **Benchmarks**: `/benches/`
- **CI Configuration**: `.github/workflows/`

### Related Guides
- [Development Setup](development/getting-started.md) - Local testing environment
- [API Documentation](api/rest-api.md) - API testing examples
- [Performance Guide](api/performance.md) - Performance testing methodology
- [WASM Integration Guide](WASM_INTEGRATION_GUIDE.md) - WASM-specific testing

---

**Status**: ✅ **Comprehensive Testing Framework** - Production-ready with 75% coverage
**Last Updated**: 2025-09-25
**Maintained by**: Hive Mind Documentation Integration Coordinator