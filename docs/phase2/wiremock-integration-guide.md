# Phase 2: Wiremock Integration Guide

## Executive Summary

RipTide currently has **12 files** with existing wiremock/mockito usage, but **293 files** contain network calls to external services (example.com, httpbin.org, reqwest::Client). This guide provides comprehensive patterns for wiremock integration across the codebase.

**Status**: Wiremock already available as dev-dependency in `crates/riptide-api/Cargo.toml` (v0.6).

---

## Existing Wiremock Implementation

### Current Usage Location
**File**: `/workspaces/eventmesh/tests/integration_fetch_reliability.rs`

**Pattern Demonstrated**:
```rust
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn test_circuit_breaker_full_lifecycle() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Configure mock response
    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(500))
        .expect(2)
        .mount(&mock_server)
        .await;

    let url = format!("{}/test", mock_server.uri());

    // Test code here...
}
```

### Key Features Demonstrated
1. ✅ **HTTP Method Matching**: `method("GET")`
2. ✅ **Path Matching**: `path("/test")`
3. ✅ **Response Templates**: `ResponseTemplate::new(500)`
4. ✅ **Call Counting**: `.expect(2)` verifies exact call count
5. ✅ **Dynamic URLs**: `mock_server.uri()` provides test server URL

---

## Network Call Analysis

### Files Requiring Network Mocking (293 Total)

#### High-Priority Test Files (20+ instances)

| File Path | Network Calls | Current Approach |
|-----------|---------------|------------------|
| `/workspaces/eventmesh/tests/api/complete_api_coverage_tests.rs` | httpbin.org/html (9x) | Real HTTP calls |
| `/workspaces/eventmesh/tests/chaos/edge_cases_tests.rs` | example.com, invalid URLs | Real HTTP calls |
| `/workspaces/eventmesh/tests/golden_tests.rs` | example.com (8x) | Real HTTP calls |
| `/workspaces/eventmesh/crates/riptide-api/tests/streaming_sse_ws_tests.rs` | reqwest::Client | Custom mocks |
| `/workspaces/eventmesh/crates/riptide-core/tests/integration_tests.rs` | Various URLs | Real HTTP calls |

#### Medium-Priority Files (10-20 instances)

| Category | File Count | Services Called |
|----------|------------|-----------------|
| Spider Tests | 8 | httpbin.org, example.com |
| HTML Extraction Tests | 12 | example.com, inline HTML |
| Intelligence Provider Tests | 6 | OpenAI, Azure, Google Vertex APIs |
| Search Provider Tests | 7 | Serper API, various search engines |

#### Low-Priority Files (<10 instances)

- Documentation examples: 25 files
- Benchmark tests: 15 files
- Configuration examples: 18 files

---

## Recommended Wiremock Patterns

### Pattern 1: Basic HTTP Mocking

**Use Case**: Replace `httpbin.org` and `example.com` calls

```rust
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::{method, path}};

#[tokio::test]
async fn test_basic_extraction() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/html"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string("<html><body>Test Content</body></html>"))
        .mount(&mock_server)
        .await;

    let url = format!("{}/html", mock_server.uri());
    let result = extract_content(&url).await;

    assert!(result.is_ok());
}
```

**Applicable Files**:
- `/workspaces/eventmesh/tests/api/complete_api_coverage_tests.rs` (9 instances)
- `/workspaces/eventmesh/tests/golden_tests.rs` (8 instances)
- `/workspaces/eventmesh/crates/riptide-html/tests/integration_tests.rs`

---

### Pattern 2: API Provider Mocking

**Use Case**: Mock AI providers (OpenAI, Azure, Google Vertex)

```rust
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::{method, path, header}};
use serde_json::json;

#[tokio::test]
async fn test_openai_provider() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("Authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "choices": [{
                    "message": {
                        "content": "Mocked AI response"
                    }
                }]
            })))
        .mount(&mock_server)
        .await;

    let provider = OpenAIProvider::new(mock_server.uri(), "test-key");
    let response = provider.generate("test prompt").await;

    assert_eq!(response.unwrap(), "Mocked AI response");
}
```

**Applicable Files**:
- `/workspaces/eventmesh/crates/riptide-intelligence/tests/provider_tests.rs`
- `/workspaces/eventmesh/crates/riptide-intelligence/tests/integration_tests.rs`

---

### Pattern 3: Search Provider Mocking

**Use Case**: Mock Serper API and search providers

```rust
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::{method, path, body_json}};
use serde_json::json;

#[tokio::test]
async fn test_serper_search() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/search"))
        .and(body_json(json!({"q": "rust programming"})))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "organic": [{
                    "title": "Rust Programming Language",
                    "link": "https://www.rust-lang.org",
                    "snippet": "Rust programming documentation"
                }]
            })))
        .mount(&mock_server)
        .await;

    let provider = SerperProvider::new(mock_server.uri(), "test-api-key");
    let results = provider.search("rust programming").await;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Rust Programming Language");
}
```

**Applicable Files**:
- `/workspaces/eventmesh/tests/unit/search_provider_test.rs`
- `/workspaces/eventmesh/tests/unit/serper_provider_test.rs`
- `/workspaces/eventmesh/crates/riptide-search/tests/integration_tests.rs`

---

### Pattern 4: Error Simulation

**Use Case**: Test circuit breakers, retry logic, error handling

```rust
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::{method, path}};
use std::time::Duration;

#[tokio::test]
async fn test_retry_logic() {
    let mock_server = MockServer::start().await;

    // First two calls return 503 (retryable)
    Mock::given(method("GET"))
        .and(path("/api"))
        .respond_with(ResponseTemplate::new(503))
        .expect(2)
        .mount(&mock_server)
        .await;

    // Third call succeeds
    Mock::given(method("GET"))
        .and(path("/api"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string("success"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let url = format!("{}/api", mock_server.uri());
    let result = client.get_with_retry(&url).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().status(), 200);
}
```

**Applicable Files**:
- `/workspaces/eventmesh/tests/integration_fetch_reliability.rs` (already using)
- `/workspaces/eventmesh/crates/riptide-search/src/circuit_breaker.rs`
- `/workspaces/eventmesh/tests/chaos/error_resilience_tests.rs`

---

### Pattern 5: Delayed Responses

**Use Case**: Test timeouts and performance

```rust
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::{method, path}};
use std::time::Duration;

#[tokio::test]
async fn test_timeout_handling() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/slow"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(10))  // Simulate slow response
                .set_body_string("delayed content")
        )
        .mount(&mock_server)
        .await;

    let url = format!("{}/slow", mock_server.uri());
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        client.get(&url)
    ).await;

    assert!(result.is_err()); // Should timeout
}
```

**Applicable Files**:
- `/workspaces/eventmesh/crates/riptide-api/tests/integration/test_edge_cases.rs`
- `/workspaces/eventmesh/tests/unit/ttfb_performance_tests.rs`

---

## Priority Migration Plan

### Phase 1: High-Impact Files (Week 1-2)

1. **tests/api/complete_api_coverage_tests.rs** (9 httpbin.org calls)
   - Replace all `https://httpbin.org/html` with wiremock
   - Estimated effort: 4 hours

2. **tests/golden_tests.rs** (8 example.com calls)
   - Replace all `https://example.com` with wiremock
   - Estimated effort: 3 hours

3. **crates/riptide-intelligence/tests/provider_tests.rs** (AI provider mocking)
   - Mock OpenAI, Azure, Google Vertex APIs
   - Estimated effort: 6 hours

### Phase 2: Medium-Impact Files (Week 3-4)

4. **Search Provider Tests** (7 files)
   - Mock Serper API and search providers
   - Estimated effort: 8 hours

5. **Spider Integration Tests** (8 files)
   - Replace httpbin.org with wiremock
   - Estimated effort: 6 hours

### Phase 3: Low-Impact Files (Week 5-6)

6. **Documentation Examples** (25 files)
   - Update examples to use wiremock patterns
   - Estimated effort: 10 hours

7. **Benchmark Tests** (15 files)
   - Add wiremock for network-dependent benchmarks
   - Estimated effort: 8 hours

---

## Test Helper Factory Pattern

### Recommended Implementation

Create `/workspaces/eventmesh/tests/common/wiremock_helpers.rs`:

```rust
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::{method, path, header}};
use serde_json::json;

pub struct WiremockTestFactory;

impl WiremockTestFactory {
    /// Create mock for httpbin.org/html
    pub async fn mock_httpbin_html(server: &MockServer) {
        Mock::given(method("GET"))
            .and(path("/html"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_string(include_str!("../fixtures/httpbin_html.html")))
            .mount(server)
            .await;
    }

    /// Create mock for example.com
    pub async fn mock_example_com(server: &MockServer, content: &str) {
        Mock::given(method("GET"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_string(content))
            .mount(server)
            .await;
    }

    /// Create mock for OpenAI API
    pub async fn mock_openai_api(server: &MockServer, response: &str) {
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(header("Authorization", "Bearer test-key"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({
                    "choices": [{"message": {"content": response}}]
                })))
            .mount(server)
            .await;
    }

    /// Create mock for Serper search API
    pub async fn mock_serper_search(server: &MockServer, results: Vec<(&str, &str, &str)>) {
        let organic: Vec<_> = results.iter().map(|(title, link, snippet)| {
            json!({
                "title": title,
                "link": link,
                "snippet": snippet
            })
        }).collect();

        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({"organic": organic})))
            .mount(server)
            .await;
    }

    /// Create error response (for circuit breaker testing)
    pub async fn mock_error_response(server: &MockServer, path: &str, status_code: u16, count: u64) {
        Mock::given(method("GET"))
            .and(path(path))
            .respond_with(ResponseTemplate::new(status_code))
            .expect(count)
            .mount(server)
            .await;
    }
}
```

---

## Integration Checklist

For each test file requiring wiremock integration:

- [ ] Identify all external HTTP calls
- [ ] Replace URLs with `mock_server.uri()`
- [ ] Configure appropriate mock responses
- [ ] Add `.expect(N)` for call count verification
- [ ] Update test assertions for mocked behavior
- [ ] Add error case mocks for resilience testing
- [ ] Document mock setup in test comments
- [ ] Run tests to verify mock behavior matches real API

---

## Benefits of Wiremock Integration

1. **Faster Test Execution**: No network I/O, tests run 10-100x faster
2. **Deterministic Results**: No flaky tests from network issues
3. **Offline Testing**: Tests work without internet connection
4. **Error Simulation**: Easy to test edge cases and error handling
5. **CI/CD Reliability**: No external service dependencies
6. **Cost Reduction**: No API quota consumption during testing

---

## Next Steps

1. Create test helper factory in `/workspaces/eventmesh/tests/common/wiremock_helpers.rs`
2. Start with Phase 1 high-impact files
3. Gradually migrate remaining test files
4. Update CI configuration to skip network-dependent tests
5. Document wiremock patterns in testing guide

---

**Status**: Ready for implementation
**Estimated Total Effort**: 45 hours across 3 phases
**Expected Impact**: 293 test files improved, 90%+ reduction in test execution time
