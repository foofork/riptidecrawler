# Test Categorization Matrix

**Version**: 1.0
**Date**: 2025-10-21
**Purpose**: Decision framework for categorizing tests

## Quick Decision Tree

```
Is this test testing a single function/module in isolation?
├─ YES → Unit Test (tests/unit/)
└─ NO → Continue...
    │
    Is this test using multiple real components together?
    ├─ YES → Continue...
    │   │
    │   Does it test a complete user workflow end-to-end?
    │   ├─ YES → E2E Test (tests/e2e/)
    │   └─ NO → Integration Test (tests/integration/)
    │
    └─ NO → Special Category Tests
        │
        ├─ Performance testing? → Performance (tests/performance/)
        ├─ Chaos/failure testing? → Chaos (tests/chaos/)
        ├─ Security testing? → Security (tests/security/)
        ├─ Golden file testing? → Regression (tests/regression/)
        ├─ Component-specific suite? → Component (tests/component/{name}/)
        ├─ Monitoring/health? → Monitoring (tests/monitoring/)
        └─ Legacy/deprecated? → Archive (tests/archive/)
```

## Detailed Category Criteria

### Unit Tests (`tests/unit/`)

**Purpose**: Test individual units of code in complete isolation

**Characteristics**:
- ✅ Tests a single function, struct, or module
- ✅ All dependencies are mocked
- ✅ No I/O operations (file, network, database)
- ✅ Executes in < 10ms typically
- ✅ Deterministic (same input → same output)
- ✅ No external state or configuration
- ✅ Can run in parallel without conflicts

**Examples**:
```rust
// ✅ Unit Test - Tests single function with mocks
#[test]
fn test_rate_limiter_blocks_overflow() {
    let config = RateLimiterConfig { limit: 10, window: Duration::from_secs(1) };
    let limiter = RateLimiter::new(config);

    for _ in 0..10 {
        assert!(limiter.allow().is_ok());
    }
    assert!(limiter.allow().is_err());
}

// ✅ Unit Test - Pure logic testing
#[test]
fn test_url_parser_extracts_domain() {
    let url = "https://example.com/path?query=1";
    assert_eq!(extract_domain(url), "example.com");
}
```

**Files That Belong Here**:
- `rate_limiter_tests.rs`
- `memory_manager_tests.rs`
- `circuit_breaker_test.rs`
- `buffer_backpressure_tests.rs`
- `ndjson_format_compliance_tests.rs`

**Decision Rules**:
- If you're mocking ALL dependencies → Unit Test
- If test fails due to external factors → NOT Unit Test
- If test requires network/filesystem → NOT Unit Test

---

### Integration Tests (`tests/integration/`)

**Purpose**: Test how multiple components work together

**Characteristics**:
- ✅ Tests 2+ components with real implementations
- ✅ May use some mocks for external services
- ✅ Limited I/O (controlled test databases, mock HTTP)
- ✅ Executes in < 1s typically
- ✅ Tests component boundaries and contracts
- ✅ May require setup/teardown
- ✅ Tests data flow between components

**Examples**:
```rust
// ✅ Integration Test - Tests spider + extraction together
#[tokio::test]
async fn test_spider_extraction_pipeline_integration() {
    let spider = Spider::new(config);
    let extractor = Extractor::new();

    let html = spider.fetch("https://example.com").await?;
    let doc = extractor.extract(&html)?;

    assert!(doc.title.is_some());
}

// ✅ Integration Test - Tests persistence layer
#[tokio::test]
async fn test_session_persistence_across_components() {
    let db = TestDatabase::new().await;
    let session_mgr = SessionManager::new(&db);

    let session = session_mgr.create().await?;
    drop(session_mgr);

    let new_mgr = SessionManager::new(&db);
    let restored = new_mgr.get(session.id).await?;
    assert_eq!(restored.id, session.id);
}
```

**Files That Belong Here**:
- `spider_integration_tests.rs`
- `worker_integration_tests.rs`
- `contract_tests.rs`
- `session_persistence_tests.rs`
- `browser_pool_tests.rs`

**Decision Rules**:
- If testing component A calling component B → Integration Test
- If testing API contracts between modules → Integration Test
- If testing data persistence and retrieval → Integration Test

---

### E2E Tests (`tests/e2e/`)

**Purpose**: Test complete user workflows from start to finish

**Characteristics**:
- ✅ Tests entire application flow
- ✅ Uses real implementations (minimal mocks)
- ✅ Real I/O operations (may use test environments)
- ✅ Executes in > 1s (often several seconds)
- ✅ Tests from user's perspective
- ✅ May require external services or mock servers
- ✅ Tests acceptance criteria

**Examples**:
```rust
// ✅ E2E Test - Complete workflow
#[tokio::test]
async fn test_complete_extraction_workflow() {
    // Start with CLI input
    let output = Command::new("riptide")
        .args(&["extract", "https://example.com"])
        .output()
        .await?;

    // Verify complete workflow
    assert!(output.status.success());
    let result: ExtractionResult = serde_json::from_slice(&output.stdout)?;
    assert!(result.title.is_some());
    assert!(result.content.len() > 0);
}

// ✅ E2E Test - Real-world scenario
#[tokio::test]
async fn test_cli_extract_with_real_website() {
    let app = TestApp::spawn().await;

    let response = app
        .client
        .post("/api/extract")
        .json(&json!({ "url": "https://news.ycombinator.com" }))
        .send()
        .await?;

    assert_eq!(response.status(), 200);
    let doc: ExtractedDoc = response.json().await?;
    assert!(doc.is_valid());
}
```

**Files That Belong Here**:
- `end_to_end_workflow_tests.rs`
- `real_world_tests.rs`
- `cli_e2e_tests.rs`
- `e2e_api.rs`

**Decision Rules**:
- If test simulates user interaction → E2E Test
- If test exercises multiple layers (CLI → API → DB) → E2E Test
- If test requires real external services → E2E Test

---

### Performance Tests (`tests/performance/`)

**Purpose**: Verify performance characteristics and SLOs

**Characteristics**:
- ✅ Measures throughput, latency, resource usage
- ✅ Validates Service Level Objectives (SLOs)
- ✅ Statistical analysis of results
- ✅ Benchmark comparisons
- ✅ Load and stress testing
- ✅ May take significant time to run

**Examples**:
```rust
// ✅ Performance Test - Throughput
#[bench]
fn bench_extraction_throughput(b: &mut Bencher) {
    let extractor = Extractor::new();
    let html = load_sample_html();

    b.iter(|| {
        extractor.extract(&html)
    });
}

// ✅ Performance Test - SLO validation
#[tokio::test]
async fn test_ttfb_under_500ms_slo() {
    let start = Instant::now();
    let response = client.get("https://example.com").send().await?;
    let ttfb = start.elapsed();

    assert!(ttfb < Duration::from_millis(500), "TTFB SLO violated: {:?}", ttfb);
}
```

**Files That Belong Here**:
- `phase1_performance_tests.rs`
- `cli_performance_tests.rs`
- `benchmarks/*`
- `load/*`

**Decision Rules**:
- If test measures timing → Performance Test
- If test validates SLO → Performance Test
- If test uses Criterion → Performance Test

---

### Chaos Tests (`tests/chaos/`)

**Purpose**: Verify system resilience under adverse conditions

**Characteristics**:
- ✅ Injects failures and errors
- ✅ Tests error handling and recovery
- ✅ Validates graceful degradation
- ✅ Tests system invariants under stress
- ✅ May intentionally crash components
- ✅ Tests timeout and retry logic

**Examples**:
```rust
// ✅ Chaos Test - Network failure
#[tokio::test]
async fn test_resilience_to_network_timeout() {
    let mut mock_server = MockServer::start().await;
    mock_server
        .mock_timeout(Duration::from_secs(10))
        .mount()
        .await;

    let client = ReliableClient::new();
    let result = client.get(mock_server.url()).await;

    // Should handle timeout gracefully
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Timeout));
}

// ✅ Chaos Test - Resource exhaustion
#[tokio::test]
async fn test_graceful_degradation_under_memory_pressure() {
    let manager = ResourceManager::new(Config {
        max_memory: 100_000, // Tiny limit
    });

    let results = Vec::new();
    for _ in 0..1000 {
        match manager.allocate(1000).await {
            Ok(resource) => results.push(resource),
            Err(e) => {
                // Should fail gracefully, not panic
                assert!(matches!(e, Error::ResourceExhausted));
                break;
            }
        }
    }
}
```

**Files That Belong Here**:
- `error_resilience_tests.rs`
- `network_chaos_tests.rs`
- `resource_chaos_tests.rs`

**Decision Rules**:
- If test intentionally injects failures → Chaos Test
- If test verifies error recovery → Chaos Test
- If test checks system under extreme conditions → Chaos Test

---

### Security Tests (`tests/security/`)

**Purpose**: Verify security measures and protections

**Characteristics**:
- ✅ Tests authentication and authorization
- ✅ Validates input sanitization
- ✅ Tests for common vulnerabilities
- ✅ Verifies data protection
- ✅ Tests stealth and privacy features

**Examples**:
```rust
// ✅ Security Test - Input validation
#[test]
fn test_api_rejects_sql_injection_attempts() {
    let malicious_input = "'; DROP TABLE users; --";
    let result = validate_input(malicious_input);
    assert!(result.is_err());
}

// ✅ Security Test - Stealth mode
#[test]
fn test_stealth_mode_masks_user_agent() {
    let config = StealthConfig::enabled();
    let client = HttpClient::new(config);
    let headers = client.default_headers();

    assert!(!headers.get("User-Agent").unwrap().contains("riptide"));
}
```

**Files That Belong Here**:
- `stealth_tests.rs`
- `input_validation_tests.rs`
- `auth_tests.rs`

---

### Regression Tests (`tests/regression/`)

**Purpose**: Prevent regressions using golden files and baselines

**Characteristics**:
- ✅ Compares against known-good outputs
- ✅ Golden file testing
- ✅ Baseline performance comparisons
- ✅ Detects unintended behavior changes
- ✅ Version compatibility testing

**Examples**:
```rust
// ✅ Regression Test - Golden file
#[test]
fn test_extraction_matches_golden_output() {
    let html = load_test_html("article.html");
    let doc = extract(&html)?;

    let golden = load_golden("article.json");
    assert_eq_json!(doc, golden);
}

// ✅ Regression Test - Baseline performance
#[test]
fn test_performance_within_baseline() {
    let result = benchmark_extraction();
    let baseline = load_baseline("extraction_v1.0.0");

    assert!(result.median < baseline.median * 1.1, "Performance regression");
}
```

**Files That Belong Here**:
- `golden/*`
- `baseline_update_tests.rs`
- `adaptive_timeout_tests.rs`

---

### Component Tests (`tests/component/{name}/`)

**Purpose**: Component-specific test suites (mix of unit, integration, E2E)

**Characteristics**:
- ✅ All tests related to one component
- ✅ May include multiple test types
- ✅ Component-specific fixtures
- ✅ Self-contained test suite

**Files That Belong Here**:
- `component/cli/*` - All CLI tests
- `component/wasm/*` - All WASM tests
- `component/api/*` - All API tests

**Decision Rules**:
- If test is specific to one component AND doesn't fit other categories clearly → Component Test

---

### Monitoring Tests (`tests/monitoring/`)

**Purpose**: Test monitoring, metrics, and health checks

**Characteristics**:
- ✅ Health check validation
- ✅ Metrics collection testing
- ✅ Monitoring system tests
- ✅ Alerting logic tests

**Files That Belong Here**:
- `monitoring/health/*`
- `monitoring/metrics/*`
- `cache_key_tests.rs`

---

### Archive Tests (`tests/archive/`)

**Purpose**: Deprecated or temporary tests kept for reference

**Characteristics**:
- ✅ Phase-based tests (phase3, phase4)
- ✅ Temporary debugging tests
- ✅ Legacy test code
- ✅ Migration artifacts

**Files That Belong Here**:
- `archive/phase3/*`
- `archive/phase4/*`
- `archive/week3/*`
- `tdd_demo_test.rs`
- `quick_circuit_test.rs`

---

## File-by-File Categorization Guide

### Currently in `tests/` Root

| File | Category | Destination |
|------|----------|-------------|
| `integration_test.rs` | Integration | `tests/integration/` (analyze first) |
| `integration_headless_cdp.rs` | Integration | `tests/integration/headless_cdp_tests.rs` |
| `integration_pipeline_orchestration.rs` | Integration | `tests/integration/pipeline_orchestration_tests.rs` |
| `integration_fetch_reliability.rs` | Integration | `tests/integration/fetch_reliability_tests.rs` |
| `wasm_component_tests.rs` | Component | `tests/component/wasm/component_tests.rs` |
| `wasm_component_guard_test.rs` | Component | `tests/component/wasm/component_guard_test.rs` |
| `e2e_tests.rs` | E2E | `tests/e2e/` |
| `real_world_tests.rs` | E2E | `tests/e2e/` |
| `tdd_demo_test.rs` | Archive | `tests/archive/` |
| `golden_test_cli.rs` | Regression | `tests/regression/golden/cli_tests.rs` |
| `cli_tables_test.rs` | Component | `tests/component/cli/tables_test.rs` |
| `error_handling_comprehensive.rs` | Chaos | `tests/chaos/` OR `tests/integration/` |
| `fix_topic_chunker.rs` | Archive | `tests/archive/` |
| `quick_circuit_test.rs` | Archive | `tests/archive/` |

### Currently in Subdirectories

#### `tests/unit/*` → Keep in place ✅
All files already properly categorized

#### `tests/integration/*` → Review and categorize
- `contract_tests.rs` → Keep ✅
- `spider_integration_tests.rs` → Keep ✅
- `worker_integration_tests.rs` → Keep ✅
- Others → Review individually

#### `tests/e2e/*` → Keep in place ✅
Already properly categorized

#### `tests/cli/*` → Move to `tests/component/cli/`
All CLI-specific tests should be grouped together

#### `tests/wasm/*` → Move to `tests/component/wasm/`
All WASM-specific tests should be grouped together

#### `tests/phase3/*`, `tests/phase4/*`, `tests/week3/*` → Archive
Phase-based organization is temporal, not logical

#### `tests/performance/*` → Keep in place ✅
Already properly categorized

#### `tests/chaos/*` → Keep in place ✅
Already properly categorized

## Decision Flowchart Summary

```
START: I need to categorize a test file

Q1: Is this a single unit test in isolation with all mocks?
    YES → tests/unit/
    NO → Continue to Q2

Q2: Does this test multiple components with real implementations?
    YES → Continue to Q3
    NO → Continue to Q5

Q3: Does this test a complete end-to-end user workflow?
    YES → tests/e2e/
    NO → Continue to Q4

Q4: Does this test component interactions/contracts?
    YES → tests/integration/
    NO → Continue to Q5

Q5: What is the primary purpose?
    Performance/Benchmarks → tests/performance/
    Chaos/Resilience → tests/chaos/
    Security → tests/security/
    Golden Files/Regression → tests/regression/
    Health/Metrics → tests/monitoring/
    Component-specific suite → tests/component/{name}/
    Legacy/Temporary → tests/archive/

Q6: Still unsure?
    → Default to tests/integration/
    → Add comment explaining categorization
    → Discuss with team
```

## Summary Table

| Category | Scope | Speed | I/O | Mocks | Examples |
|----------|-------|-------|-----|-------|----------|
| Unit | Single function | < 10ms | None | All | `rate_limiter_tests.rs` |
| Integration | 2+ components | < 1s | Limited | Some | `spider_integration_tests.rs` |
| E2E | Complete workflow | > 1s | Real | Minimal | `end_to_end_workflow_tests.rs` |
| Performance | Benchmarks | Varies | Varies | Varies | `phase1_performance_tests.rs` |
| Chaos | Resilience | Varies | Varies | Some | `error_resilience_tests.rs` |
| Security | Protection | Varies | Some | Some | `stealth_tests.rs` |
| Regression | Golden files | Varies | Some | Some | `golden/*` |
| Component | Component suite | Mixed | Mixed | Mixed | `component/cli/*` |
| Monitoring | Health/metrics | Fast | Some | Some | `monitoring/health/*` |
| Archive | Deprecated | N/A | N/A | N/A | `archive/phase3/*` |

---

**When in doubt**: Ask yourself "What is the PRIMARY purpose of this test?" and use that to guide categorization.
