# Phase 3: P1 Testing & Validation (Weeks 7-9)

**Duration**: 3 weeks
**Goal**: Achieve 90%+ test coverage and eliminate 44 ignored tests
**Key Outcomes**:
- 12 untested facades now have comprehensive tests
- 44 ignored tests fixed and re-enabled
- Integration tests for all resilience patterns
- Test coverage: 61% → 90%+

---

## Phase Overview

Phase 3 eliminates the testing debt identified in Week 0:

1. **Sprint 7**: Test 12 untested facades (one per day)
2. **Sprint 8**: Fix and re-enable 44 ignored tests
3. **Sprint 9**: Integration testing for resilience patterns

**Testing Philosophy**: "Testing was shifted left" - unit tests were written concurrently with Sprint 2-6 work. This phase focuses on:
- Facades that existed before refactoring (untested legacy)
- Ignored tests from previous sprints
- End-to-end integration scenarios

---

## Sprint 7: Untested Facade Testing (Week 7)

**Goal**: Bring 12 legacy untested facades to 90%+ coverage

### Untested Facades (from Week 0 audit)

| Facade | LOC | Complexity | Test Priority | Days |
|--------|-----|------------|---------------|------|
| RateLimitingFacade | 234 | Medium | P0 | 0.5 |
| RetryFacade | 189 | Medium | P0 | 0.5 |
| BatchFacade | 312 | High | P0 | 1.0 |
| TransformFacade | 267 | Medium | P1 | 0.5 |
| FilterFacade | 198 | Low | P1 | 0.5 |
| AggregateFacade | 423 | High | P1 | 1.0 |
| ValidationFacade | 145 | Low | P1 | 0.5 |
| AuthorizationFacade | 289 | Medium | P0 | 0.5 |
| AuditFacade | 176 | Low | P1 | 0.5 |
| SchedulerFacade | 534 | High | P1 | 1.0 |
| NotificationFacade | 223 | Medium | P2 | 0.5 |
| ReportingFacade | 367 | Medium | P2 | 0.5 |

**Total**: 5 days (12 facades)

### Testing Strategy

**Test Structure** (London School TDD):
```rust
// crates/riptide-facade/tests/{facade}_tests.rs

use mockall::predicate::*;
use riptide_facade::{[Facade]};
use riptide_types::*;

mod [facade]_tests {
    use super::*;

    // Helper: Create test context with mocked ports
    fn create_test_context() -> Arc<ApplicationContext> {
        let mut browser_mock = MockBrowserDriver::new();
        browser_mock.expect_acquire()
            .returning(|| Ok(MockBrowser::new()));

        ApplicationContext::builder()
            .with_browser_driver(Arc::new(browser_mock))
            // ... mock all required ports
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_[happy_path]() {
        let context = create_test_context();
        let facade = [Facade]::new(context);

        let result = facade.[method](...).await;

        assert!(result.is_ok());
        // ... assertions
    }

    #[tokio::test]
    async fn test_[error_case]() {
        let context = create_test_context();
        let facade = [Facade]::new(context);

        let result = facade.[method_that_fails](...).await;

        assert!(result.is_err());
        // ... error assertions
    }

    #[tokio::test]
    async fn test_[edge_case]() {
        // ... edge case testing
    }

    // Property-based tests
    #[quickcheck_async::tokio]
    async fn prop_[invariant](input: ArbitraryInput) {
        // ... property test
    }
}
```

### Day 1-2: P0 Facades (RateLimiting, Retry, Authorization, Batch)

#### Task 7.1: RateLimitingFacade Tests
`crates/riptide-facade/tests/rate_limiting_facade_tests.rs`:

```rust
#[tokio::test]
async fn test_rate_limit_enforced() {
    let context = create_test_context();
    let facade = RateLimitingFacade::new(context);

    // Configure rate limit: 2 requests per second
    facade.configure("test-key", 2, Duration::from_secs(1)).await.unwrap();

    // First 2 requests should succeed
    assert!(facade.check("test-key").await.unwrap());
    assert!(facade.check("test-key").await.unwrap());

    // Third request should be denied
    assert!(!facade.check("test-key").await.unwrap());
}

#[tokio::test]
async fn test_rate_limit_reset() {
    let context = create_test_context();
    let facade = RateLimitingFacade::new(context);

    facade.configure("test-key", 1, Duration::from_secs(1)).await.unwrap();

    // Exhaust limit
    assert!(facade.check("test-key").await.unwrap());
    assert!(!facade.check("test-key").await.unwrap());

    // Wait for reset
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Should be allowed again
    assert!(facade.check("test-key").await.unwrap());
}

#[tokio::test]
async fn test_rate_limit_per_key_isolation() {
    let context = create_test_context();
    let facade = RateLimitingFacade::new(context);

    facade.configure("key1", 1, Duration::from_secs(1)).await.unwrap();
    facade.configure("key2", 1, Duration::from_secs(1)).await.unwrap();

    // Exhaust key1
    assert!(facade.check("key1").await.unwrap());
    assert!(!facade.check("key1").await.unwrap());

    // key2 should still work (isolation)
    assert!(facade.check("key2").await.unwrap());
}

// Property test: rate limit never allows more than configured
#[quickcheck_async::tokio]
async fn prop_rate_limit_never_exceeds(limit: u32, duration_secs: u64) {
    let context = create_test_context();
    let facade = RateLimitingFacade::new(context);

    let limit = limit.clamp(1, 100);
    let duration = Duration::from_secs(duration_secs.clamp(1, 60));

    facade.configure("prop-key", limit, duration).await.unwrap();

    let mut allowed = 0;
    for _ in 0..limit * 2 {
        if facade.check("prop-key").await.unwrap() {
            allowed += 1;
        }
    }

    assert!(allowed <= limit, "Rate limiter allowed {} requests, limit was {}", allowed, limit);
}
```

**Acceptance Criteria**:
- [ ] 90%+ line coverage
- [ ] Edge cases: reset, isolation, overflow
- [ ] Property test for invariant
- [ ] All tests pass

#### Task 7.2: RetryFacade Tests
```rust
#[tokio::test]
async fn test_retry_succeeds_eventually() {
    let mut attempt = 0;
    let operation = || {
        attempt += 1;
        if attempt < 3 {
            async { Err("Transient failure".into()) }
        } else {
            async { Ok("Success") }
        }
    };

    let context = create_test_context();
    let facade = RetryFacade::new(context);

    let result = facade.retry(operation, 5, Duration::from_millis(10)).await;

    assert!(result.is_ok());
    assert_eq!(attempt, 3);
}

#[tokio::test]
async fn test_retry_exhausts_attempts() {
    let operation = || async { Err("Permanent failure".into()) };

    let context = create_test_context();
    let facade = RetryFacade::new(context);

    let result = facade.retry(operation, 3, Duration::from_millis(10)).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_retry_exponential_backoff() {
    let start = std::time::Instant::now();
    let operation = || async { Err("Failure".into()) };

    let context = create_test_context();
    let facade = RetryFacade::new(context);

    let _result = facade.retry_with_backoff(
        operation,
        3,
        Duration::from_millis(100),
        2.0, // exponential factor
    ).await;

    let elapsed = start.elapsed();

    // Expected: 100ms + 200ms + 400ms = 700ms
    assert!(elapsed >= Duration::from_millis(700));
    assert!(elapsed < Duration::from_millis(800));
}
```

**Acceptance Criteria**:
- [ ] Retry logic tested
- [ ] Exponential backoff validated
- [ ] Timeout scenarios covered
- [ ] 90%+ coverage

#### Task 7.3: AuthorizationFacade Tests
```rust
#[tokio::test]
async fn test_authorized_user_can_access_resource() {
    let context = create_test_context();
    let facade = AuthorizationFacade::new(context);

    // Grant permission
    facade.grant("user123", "document:456", "read").await.unwrap();

    // Check authorization
    let allowed = facade.authorize("user123", "document:456", "read").await.unwrap();

    assert!(allowed);
}

#[tokio::test]
async fn test_unauthorized_user_denied() {
    let context = create_test_context();
    let facade = AuthorizationFacade::new(context);

    // No permission granted
    let allowed = facade.authorize("user123", "document:456", "write").await.unwrap();

    assert!(!allowed);
}

#[tokio::test]
async fn test_revoke_permission() {
    let context = create_test_context();
    let facade = AuthorizationFacade::new(context);

    // Grant then revoke
    facade.grant("user123", "document:456", "read").await.unwrap();
    facade.revoke("user123", "document:456", "read").await.unwrap();

    let allowed = facade.authorize("user123", "document:456", "read").await.unwrap();

    assert!(!allowed);
}
```

**Acceptance Criteria**:
- [ ] RBAC permission checks tested
- [ ] Grant/revoke functionality validated
- [ ] Permission inheritance scenarios
- [ ] 90%+ coverage

#### Task 7.4: BatchFacade Tests
```rust
#[tokio::test]
async fn test_batch_processing_all_succeed() {
    let context = create_test_context();
    let facade = BatchFacade::new(context);

    let items = vec!["item1", "item2", "item3"];
    let operation = |item: &str| async move { Ok(format!("Processed: {}", item)) };

    let results = facade.batch(items, operation, 10).await;

    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.is_ok()));
}

#[tokio::test]
async fn test_batch_processing_partial_failure() {
    let context = create_test_context();
    let facade = BatchFacade::new(context);

    let items = vec!["item1", "fail", "item3"];
    let operation = |item: &str| async move {
        if item == "fail" {
            Err("Intentional failure".into())
        } else {
            Ok(format!("Processed: {}", item))
        }
    };

    let results = facade.batch(items, operation, 10).await;

    assert_eq!(results.len(), 3);
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 2);
    assert_eq!(results.iter().filter(|r| r.is_err()).count(), 1);
}

#[tokio::test]
async fn test_batch_concurrency_limit() {
    let context = create_test_context();
    let facade = BatchFacade::new(context);

    let concurrent = Arc::new(AtomicUsize::new(0));
    let max_concurrent = Arc::new(AtomicUsize::new(0));

    let items: Vec<u32> = (0..100).collect();
    let operation = |_item: u32| {
        let concurrent = concurrent.clone();
        let max_concurrent = max_concurrent.clone();

        async move {
            let current = concurrent.fetch_add(1, Ordering::SeqCst) + 1;
            max_concurrent.fetch_max(current, Ordering::SeqCst);

            tokio::time::sleep(Duration::from_millis(10)).await;

            concurrent.fetch_sub(1, Ordering::SeqCst);

            Ok(())
        }
    };

    let _results = facade.batch(items, operation, 10).await;

    let max = max_concurrent.load(Ordering::SeqCst);
    assert!(max <= 10, "Max concurrent was {}, limit was 10", max);
}
```

**Acceptance Criteria**:
- [ ] Batch processing tested
- [ ] Concurrency limits enforced
- [ ] Partial failure handling validated
- [ ] 90%+ coverage

### Day 3-4: P1 Facades (Transform, Filter, Aggregate, Validation, Audit)

#### Task 7.5-7.9: Similar Testing Pattern
For each P1 facade:
- [ ] Happy path tests
- [ ] Error cases
- [ ] Edge cases (empty input, null, overflow)
- [ ] Property tests where applicable
- [ ] 90%+ coverage

**Aggregate patterns tested**:
- TransformFacade: data transformation correctness
- FilterFacade: predicate logic, empty results
- AggregateFacade: aggregation functions (sum, avg, count)
- ValidationFacade: schema validation, coercion
- AuditFacade: event logging, retention

### Day 5: P2 Facades (Scheduler, Notification, Reporting)

#### Task 7.10: SchedulerFacade Tests (Complex)
```rust
#[tokio::test]
async fn test_cron_schedule_execution() {
    let context = create_test_context();
    let facade = SchedulerFacade::new(context);

    let executed = Arc::new(AtomicBool::new(false));
    let executed_clone = executed.clone();

    let task = move || {
        executed_clone.store(true, Ordering::SeqCst);
        async { Ok(()) }
    };

    // Schedule for every second
    facade.schedule("* * * * * *", task).await.unwrap();

    // Wait for execution
    tokio::time::sleep(Duration::from_secs(2)).await;

    assert!(executed.load(Ordering::SeqCst));
}

#[tokio::test]
async fn test_scheduler_cancellation() {
    let context = create_test_context();
    let facade = SchedulerFacade::new(context);

    let task_id = facade.schedule("* * * * * *", || async { Ok(()) }).await.unwrap();

    // Cancel task
    facade.cancel(task_id).await.unwrap();

    // Verify not executing
    tokio::time::sleep(Duration::from_secs(2)).await;

    let status = facade.status(task_id).await.unwrap();
    assert_eq!(status, TaskStatus::Cancelled);
}
```

**Acceptance Criteria**:
- [ ] Cron parsing tested
- [ ] Task execution validated
- [ ] Cancellation works
- [ ] 90%+ coverage

### Sprint 7 Quality Gates

**All 6 gates must pass** ✅

**Additional Sprint 7 Validation**:
- [ ] 12 untested facades now have 90%+ coverage
- [ ] Total workspace coverage: 61% → 78%
- [ ] All property tests pass (quickcheck)

---

## Sprint 8: Fix Ignored Tests (Week 8)

**Goal**: Re-enable and fix 44 ignored tests from Week 0 baseline

### Ignored Test Inventory (from Week 0)

**Categorized by reason**:

| Reason | Count | Crates |
|--------|-------|--------|
| Flaky (timing issues) | 12 | riptide-browser (3), riptide-facade (9) |
| Missing infrastructure | 8 | riptide-facade (8) |
| Incomplete implementation | 15 | riptide-browser (2), riptide-facade (13) |
| External service dependency | 9 | riptide-facade (9) |

### Day 1-2: Fix Flaky Tests (12 tests)

#### Task 8.1: Browser Pool Timing Issues
**Problem**: Tests fail intermittently due to browser startup timing

```rust
// BEFORE (flaky)
#[tokio::test]
#[ignore] // Flaky: browser might not be ready
async fn test_browser_checkout_checkin() {
    let pool = ChromiumPool::new(config).await.unwrap();
    let browser = pool.checkout().await.unwrap();
    pool.checkin(browser).await.unwrap();
}

// AFTER (deterministic)
#[tokio::test]
async fn test_browser_checkout_checkin() {
    let pool = ChromiumPool::new(config).await.unwrap();

    // Wait for pool to be ready
    pool.wait_ready(Duration::from_secs(30)).await.unwrap();

    let browser = pool.checkout().await.unwrap();
    assert!(browser.is_connected().await);

    pool.checkin(browser).await.unwrap();

    // Verify pool health
    assert_eq!(pool.available(), 1);
}
```

**Acceptance Criteria**:
- [ ] All 12 flaky tests fixed
- [ ] Tests run 100 times without failure
- [ ] Proper timeouts and retries added
- [ ] All re-enabled (no `#[ignore]`)

### Day 3: Fix Infrastructure-Dependent Tests (8 tests)

#### Task 8.2: Missing Adapter Mocks
**Problem**: Tests require real Redis/Postgres, not available in CI

```rust
// BEFORE (requires real Redis)
#[tokio::test]
#[ignore] // No Redis in CI
async fn test_cache_facade_integration() {
    let redis = connect_redis("redis://localhost").await.unwrap();
    let facade = CacheFacade::new(redis);
    // ...
}

// AFTER (mocked)
#[tokio::test]
async fn test_cache_facade_integration() {
    let mock_cache = MockCacheStorage::new();
    mock_cache.expect_get()
        .returning(|_| Ok(Some("cached value".into())));

    let context = ApplicationContext::builder()
        .with_cache_storage(Arc::new(mock_cache))
        .build()
        .unwrap();

    let facade = CacheFacade::new(context);

    let result = facade.get("key").await.unwrap();
    assert_eq!(result, Some("cached value".to_string()));
}
```

**Acceptance Criteria**:
- [ ] All 8 tests use mocks instead of real infrastructure
- [ ] Tests pass in CI
- [ ] All re-enabled

### Day 4: Complete Incomplete Implementations (15 tests)

#### Task 8.3: Finish TODO Tests
**Problem**: Tests marked `#[ignore]` with "TODO: implement" comments

```rust
// BEFORE (incomplete)
#[tokio::test]
#[ignore] // TODO: implement extraction logic
async fn test_pdf_extraction() {
    // Not implemented
}

// AFTER (complete)
#[tokio::test]
async fn test_pdf_extraction() {
    let context = create_test_context();
    let facade = ExtractorFacade::new(context);

    let pdf_content = include_bytes!("../fixtures/sample.pdf");
    let result = facade.extract_pdf(pdf_content).await.unwrap();

    assert!(!result.text.is_empty());
    assert!(result.metadata.page_count > 0);
}
```

**Acceptance Criteria**:
- [ ] All 15 incomplete tests finished
- [ ] Implementations match test expectations
- [ ] All re-enabled

### Day 5: Fix External Service Dependencies (9 tests)

#### Task 8.4: Mock External Services
**Problem**: Tests require live URLs or external APIs

```rust
// BEFORE (requires internet)
#[tokio::test]
#[ignore] // Requires external URL
async fn test_crawl_live_url() {
    let facade = CrawlFacade::new(context);
    let result = facade.crawl("https://example.com").await.unwrap();
    assert!(!result.html.is_empty());
}

// AFTER (mocked HTTP)
#[tokio::test]
async fn test_crawl_live_url() {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::path;

    // Setup mock server
    let mock_server = MockServer::start().await;
    Mock::given(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<html>Test</html>"))
        .mount(&mock_server)
        .await;

    let context = create_test_context();
    let facade = CrawlFacade::new(context);

    let result = facade.crawl(&mock_server.uri()).await.unwrap();

    assert_eq!(result.html, "<html>Test</html>");
}
```

**Acceptance Criteria**:
- [ ] All 9 tests use wiremock or similar
- [ ] No internet required
- [ ] Tests deterministic
- [ ] All re-enabled

### Sprint 8 Quality Gates

**All 6 gates must pass** ✅

**Additional Sprint 8 Validation**:
- [ ] Ignored tests: 44 → 0 (100% reduction!)
- [ ] Total workspace coverage: 78% → 85%
- [ ] CI test time reduced (no external dependencies)

---

## Sprint 9: Integration & Resilience Testing (Week 9)

**Goal**: End-to-end integration tests for all resilience patterns

### Integration Test Suite Structure

```
crates/riptide-api/tests/
├── integration/
│   ├── resilience/
│   │   ├── circuit_breaker_tests.rs
│   │   ├── rate_limiting_tests.rs
│   │   ├── retry_tests.rs
│   │   └── idempotency_tests.rs
│   ├── flows/
│   │   ├── crawl_flow_tests.rs
│   │   ├── extract_flow_tests.rs
│   │   └── search_flow_tests.rs
│   └── performance/
│       ├── throughput_tests.rs
│       └── latency_tests.rs
```

### Day 1-2: Resilience Pattern Integration Tests

#### Task 9.1: Circuit Breaker Integration Test
`crates/riptide-api/tests/integration/resilience/circuit_breaker_tests.rs`:

```rust
#[tokio::test]
async fn test_circuit_breaker_protects_crawl_facade() {
    // Start test server with controlled failures
    let mut server = mockito::Server::new();
    let url = server.url();

    // Configure to fail 5 times, then succeed
    let mut failure_count = 0;
    let mock = server.mock("GET", "/")
        .with_status_code_fn(move || {
            failure_count += 1;
            if failure_count <= 5 { 500 } else { 200 }
        })
        .create();

    // Initialize context with circuit breaker
    let context = ApplicationContext::builder()
        .with_circuit_breaker(Arc::new(CircuitBreakerAdapter::new(
            3, // failure threshold
            Duration::from_secs(5), // timeout
        )))
        .with_http_client(Arc::new(ReqwestHttpAdapter::new()))
        // ... other ports
        .build()
        .unwrap();

    let facade = CrawlFacade::new(context);

    // First 3 requests should fail normally
    for i in 0..3 {
        let result = facade.crawl(&url).await;
        assert!(result.is_err(), "Request {} should fail", i);
    }

    // Circuit should open (4th request should fail immediately)
    let circuit_state = facade.context.circuit_breaker.state();
    assert_eq!(circuit_state, CircuitState::Open);

    // Request should fail fast (no HTTP call)
    let start = std::time::Instant::now();
    let result = facade.crawl(&url).await;
    let elapsed = start.elapsed();

    assert!(result.is_err());
    assert!(elapsed < Duration::from_millis(100), "Should fail fast, took {:?}", elapsed);

    // Wait for half-open transition
    tokio::time::sleep(Duration::from_secs(6)).await;

    assert_eq!(facade.context.circuit_breaker.state(), CircuitState::HalfOpen);

    // Next request succeeds, circuit closes
    let result = facade.crawl(&url).await;
    assert!(result.is_ok());
    assert_eq!(facade.context.circuit_breaker.state(), CircuitState::Closed);
}
```

**Acceptance Criteria**:
- [ ] Circuit opens after threshold failures
- [ ] Requests fail fast when open
- [ ] Half-open transition works
- [ ] Circuit closes on success

#### Task 9.2: Rate Limiting Integration Test
```rust
#[tokio::test]
async fn test_rate_limiting_protects_api() {
    let context = ApplicationContext::builder()
        .with_rate_limiter(Arc::new(TokenBucketRateLimiter::new(
            Quota::per_second(nonzero!(5u32)), // 5 requests/second
        )))
        .build()
        .unwrap();

    let facade = CrawlFacade::new(context);

    // Fire 10 requests rapidly
    let start = std::time::Instant::now();
    for i in 0..10 {
        facade.crawl("https://example.com").await.unwrap();
    }
    let elapsed = start.elapsed();

    // Should take at least 1 second (5 + 5 with 1 second spacing)
    assert!(elapsed >= Duration::from_secs(1), "Rate limiting not working, took {:?}", elapsed);
    assert!(elapsed < Duration::from_secs(3), "Too slow, took {:?}", elapsed);
}
```

**Acceptance Criteria**:
- [ ] Rate limiting enforced
- [ ] Throughput matches configured quota
- [ ] No requests dropped

#### Task 9.3: Idempotency Integration Test
```rust
#[tokio::test]
async fn test_idempotency_prevents_duplicate_crawls() {
    let context = ApplicationContext::builder()
        .with_idempotency_store(Arc::new(RedisIdempotencyAdapter::new("redis://localhost")?))
        .build()
        .unwrap();

    let facade = CrawlFacade::new(context);

    // First crawl
    let result1 = facade.crawl_with_id("id-123", "https://example.com").await.unwrap();

    // Duplicate crawl (should return cached result)
    let start = std::time::Instant::now();
    let result2 = facade.crawl_with_id("id-123", "https://example.com").await.unwrap();
    let elapsed = start.elapsed();

    // Results should be identical
    assert_eq!(result1.html, result2.html);

    // Should be much faster (cached)
    assert!(elapsed < Duration::from_millis(100), "Should be cached, took {:?}", elapsed);
}
```

**Acceptance Criteria**:
- [ ] Duplicate operations detected
- [ ] Cached results returned
- [ ] Significant performance improvement

### Day 3-4: End-to-End Flow Tests

#### Task 9.4: Complete Crawl Flow Test
```rust
#[tokio::test]
async fn test_complete_crawl_extract_search_flow() {
    // Initialize full context
    let context = create_production_context().await;

    // 1. Crawl URL
    let crawl_facade = CrawlFacade::new(context.clone());
    let crawl_result = crawl_facade.crawl("https://example.com").await.unwrap();
    assert!(!crawl_result.html.is_empty());

    // 2. Extract content
    let extractor_facade = ExtractorFacade::new(context.clone());
    let extract_result = extractor_facade.extract(&crawl_result.html).await.unwrap();
    assert!(!extract_result.text.is_empty());

    // 3. Index in search
    let search_facade = SearchFacade::new(context.clone());
    search_facade.index("doc-1", &extract_result).await.unwrap();

    // 4. Search for content
    let search_results = search_facade.search(&extract_result.text[..20], None).await.unwrap();
    assert!(search_results.total > 0);

    // 5. Generate PDF report
    let pdf_facade = PdfFacade::new(context.clone());
    let pdf = pdf_facade.generate_report(&search_results).await.unwrap();
    assert!(!pdf.is_empty());
}
```

**Acceptance Criteria**:
- [ ] Full flow completes end-to-end
- [ ] All facades integrate correctly
- [ ] Data flows between facades
- [ ] No errors in happy path

### Day 5: Performance & Load Tests

#### Task 9.5: Throughput Test
```rust
#[tokio::test]
async fn test_api_throughput_under_load() {
    let context = create_production_context().await;
    let facade = CrawlFacade::new(context);

    // Spawn 100 concurrent requests
    let tasks: Vec<_> = (0..100)
        .map(|i| {
            let facade = facade.clone();
            tokio::spawn(async move {
                facade.crawl(&format!("https://example.com/page/{}", i)).await
            })
        })
        .collect();

    // Wait for all to complete
    let start = std::time::Instant::now();
    let results = futures::future::join_all(tasks).await;
    let elapsed = start.elapsed();

    // Verify results
    let successes = results.iter().filter(|r| r.is_ok()).count();

    println!("Throughput: {} req/sec", successes as f64 / elapsed.as_secs_f64());

    assert!(successes >= 95, "At least 95% success rate");
    assert!(elapsed < Duration::from_secs(30), "Should complete in <30s");
}
```

**Acceptance Criteria**:
- [ ] Handles 100 concurrent requests
- [ ] >95% success rate under load
- [ ] Latency acceptable (<300ms p95)

### Sprint 9 Quality Gates

**All 6 gates must pass** ✅

**Additional Sprint 9 Validation**:
- [ ] All resilience patterns tested end-to-end
- [ ] Complete flows validated
- [ ] Performance benchmarks established
- [ ] Total workspace coverage: 85% → 90%+

---

## Phase 3 Success Metrics

| Metric | Phase 2 End | Sprint 9 Target | Actual |
|--------|-------------|-----------------|--------|
| Test Coverage | 61% | 90% | ___ |
| Ignored Tests | 44 | 0 | ___ |
| Untested Facades | 12 | 0 | ___ |
| Integration Tests | 0 | 25+ | ___ |

---

## Phase 3 Deliverables

- [ ] 12 untested facades now have 90%+ coverage
- [ ] 44 ignored tests fixed and re-enabled
- [ ] 25+ integration tests for resilience patterns
- [ ] End-to-end flow tests for crawl/extract/search
- [ ] Performance benchmarks established
- [ ] Test coverage: 61% → 90%+
- [ ] All quality gates passed for Sprints 7-9

---

## Next: Phase 4 (Weeks 10-16)

**Goal**: Remove AppState wrapper, flip feature flags, production hardening
**See**: [ROADMAP-PHASE-4.md](ROADMAP-PHASE-4.md)

---

**Status**: Ready for Sprint 7 kickoff
**Owner**: QA Engineering Team
**Duration**: 3 weeks
