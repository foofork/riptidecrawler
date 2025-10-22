# Testing Best Practices

**Version**: 1.0
**Date**: 2025-10-21
**For**: All contributors

## Core Principles

### 1. Test Behavior, Not Implementation

❌ **Bad** - Testing implementation details:
```rust
#[test]
fn test_cache_uses_hashmap() {
    let cache = Cache::new();
    // Don't access internal fields
    assert!(cache.internal_map.is_empty());  // ❌
}
```

✅ **Good** - Testing behavior:
```rust
#[test]
fn test_cache_stores_and_retrieves_values() {
    let mut cache = Cache::new();

    cache.set("key", "value");
    let result = cache.get("key");

    assert_eq!(result, Some("value"));
}
```

**Why**: Implementation details change; behavior contracts don't.

---

### 2. Arrange-Act-Assert (AAA) Pattern

✅ **Always structure tests this way**:
```rust
#[test]
fn test_feature() {
    // ARRANGE - Set up test data and preconditions
    let input = "test data";
    let expected = "expected result";
    let component = Component::new();

    // ACT - Execute the behavior being tested
    let result = component.process(input);

    // ASSERT - Verify the outcome
    assert_eq!(result, expected);
}
```

**Benefits**:
- Clear test structure
- Easy to understand
- Separates concerns

---

### 3. One Logical Assertion Per Test

❌ **Bad** - Testing multiple behaviors:
```rust
#[test]
fn test_everything() {
    let parser = Parser::new();

    // Too many unrelated assertions
    assert!(parser.parse("valid").is_ok());
    assert!(parser.parse("invalid").is_err());
    assert_eq!(parser.count(), 0);
    assert!(parser.supports_format("json"));
}
```

✅ **Good** - Focused tests:
```rust
#[test]
fn test_parser_accepts_valid_input() {
    let parser = Parser::new();
    assert!(parser.parse("valid").is_ok());
}

#[test]
fn test_parser_rejects_invalid_input() {
    let parser = Parser::new();
    assert!(parser.parse("invalid").is_err());
}

#[test]
fn test_parser_supports_json_format() {
    let parser = Parser::new();
    assert!(parser.supports_format("json"));
}
```

**Note**: Multiple assertions about the *same* behavior are OK:
```rust
#[test]
fn test_extraction_returns_complete_document() {
    let doc = extract(html);

    // These all verify the same behavior: "returns complete document"
    assert!(doc.title.is_some());
    assert!(!doc.content.is_empty());
    assert!(doc.url.is_some());
}
```

---

### 4. Tests Should Be Independent

❌ **Bad** - Tests depend on execution order:
```rust
static mut COUNTER: i32 = 0;

#[test]
fn test_first() {
    unsafe { COUNTER = 1; }
}

#[test]
fn test_second() {
    unsafe { assert_eq!(COUNTER, 1); }  // ❌ Depends on test_first
}
```

✅ **Good** - Each test is self-contained:
```rust
#[test]
fn test_first() {
    let counter = Counter::new();
    counter.set(1);
    assert_eq!(counter.get(), 1);
}

#[test]
fn test_second() {
    let counter = Counter::new();
    counter.set(2);
    assert_eq!(counter.get(), 2);
}
```

**Why**: Tests should run in any order, in parallel, and independently.

---

### 5. Use Descriptive Test Names

❌ **Bad** - Unclear test names:
```rust
#[test]
fn test1() { }

#[test]
fn test_stuff() { }

#[test]
fn it_works() { }
```

✅ **Good** - Self-documenting names:
```rust
#[test]
fn test_rate_limiter_blocks_requests_over_limit() { }

#[test]
fn test_extraction_handles_malformed_html_gracefully() { }

#[test]
fn test_cache_evicts_lru_entry_when_full() { }
```

**Pattern**: `test_{what}_{when}_{expected}`

---

### 6. Test Edge Cases and Boundaries

```rust
#[test]
fn test_rate_limiter_edge_cases() {
    let limiter = RateLimiter::new(10);

    // Boundary: exactly at limit
    for _ in 0..10 {
        assert!(limiter.allow().is_ok());
    }

    // Edge: one over limit
    assert!(limiter.allow().is_err());
}

#[test]
fn test_string_parsing_edge_cases() {
    // Empty string
    assert!(parse("").is_err());

    // Very long string
    let long_string = "a".repeat(10_000);
    assert!(parse(&long_string).is_ok());

    // Unicode
    assert!(parse("🦀 Rust").is_ok());

    // Null characters
    assert!(parse("test\0test").is_err());
}
```

**Common edge cases**:
- Empty collections
- Null/None values
- Zero/negative numbers
- Maximum values
- Boundary conditions
- Unicode/special characters

---

### 7. Test Error Paths Thoroughly

```rust
#[test]
fn test_http_client_handles_network_timeout() {
    let client = HttpClient::new();

    let result = client.get_with_timeout(
        "https://slow-server.com",
        Duration::from_millis(100)
    );

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Timeout));
}

#[test]
fn test_parser_provides_helpful_error_messages() {
    let parser = Parser::new();

    let result = parser.parse("invalid {");

    let error = result.unwrap_err();
    assert!(error.to_string().contains("line 1"));
    assert!(error.to_string().contains("unclosed brace"));
}
```

**Error testing checklist**:
- [ ] Test all error variants
- [ ] Verify error messages are helpful
- [ ] Test error recovery mechanisms
- [ ] Test error propagation
- [ ] Verify cleanup happens on error

---

### 8. Keep Tests Fast

✅ **Fast tests enable rapid development**:

```rust
// Unit tests: < 10ms
#[test]
fn test_fast_pure_function() {
    let result = pure_calculation(42);
    assert_eq!(result, 84);
}

// Integration tests: < 1s
#[tokio::test]
async fn test_component_integration() {
    let result = component_a
        .interact_with(component_b)
        .await;
    assert!(result.is_ok());
}

// E2E tests: < 5s
#[tokio::test]
async fn test_full_workflow() {
    let result = app.process_request(request).await;
    assert!(result.is_success());
}
```

**Speed optimization techniques**:
- Use mocks instead of real I/O
- Minimize setup/teardown
- Use in-memory databases for tests
- Parallelize independent tests
- Mock time-dependent operations

---

### 9. Use Test Fixtures and Builders

✅ **Builders for flexible test data**:
```rust
pub struct TestUserBuilder {
    name: String,
    email: String,
    role: Role,
}

impl TestUserBuilder {
    pub fn new() -> Self {
        Self {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            role: Role::User,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn admin(mut self) -> Self {
        self.role = Role::Admin;
        self
    }

    pub fn build(self) -> User {
        User {
            name: self.name,
            email: self.email,
            role: self.role,
        }
    }
}

// Usage
#[test]
fn test_admin_can_delete_users() {
    let admin = TestUserBuilder::new()
        .admin()
        .build();

    let user = TestUserBuilder::new().build();

    assert!(admin.can_delete(&user));
}
```

---

### 10. Mock External Dependencies

```rust
use mockall::*;

#[automock]
pub trait HttpClient {
    async fn get(&self, url: &str) -> Result<Response, Error>;
}

#[tokio::test]
async fn test_spider_retries_on_failure() {
    let mut mock_client = MockHttpClient::new();

    // First call fails
    mock_client
        .expect_get()
        .times(1)
        .returning(|_| Err(Error::NetworkFailure));

    // Second call succeeds
    mock_client
        .expect_get()
        .times(1)
        .returning(|_| Ok(Response::new("success")));

    let spider = Spider::new(mock_client);

    let result = spider.fetch_with_retry("https://example.com").await;

    assert!(result.is_ok());
}
```

**Mock benefits**:
- Fast tests (no real network calls)
- Deterministic (no flaky tests)
- Test error scenarios easily
- Verify interactions

---

### 11. Avoid Test Interdependence

❌ **Bad** - Tests that depend on each other:
```rust
#[test]
fn test_create_user() {
    DB.insert(user);  // ❌ Shared global state
}

#[test]
fn test_find_user() {
    let user = DB.find(id);  // ❌ Assumes test_create_user ran
    assert!(user.is_some());
}
```

✅ **Good** - Each test sets up its own state:
```rust
#[test]
fn test_create_user() {
    let db = TestDb::new();
    db.insert(user);
    assert!(db.find(user.id).is_some());
}

#[test]
fn test_find_user() {
    let db = TestDb::new();
    db.insert(user);  // Set up state for this test
    let found = db.find(user.id);
    assert!(found.is_some());
}
```

---

### 12. Document Non-Obvious Test Scenarios

```rust
/// Tests that the circuit breaker opens after the failure threshold is exceeded.
///
/// # Test Scenario
/// The circuit breaker is configured to open after 5 consecutive failures.
/// We simulate 5 failed requests, then verify the breaker is open.
///
/// # Regression
/// This test prevents regression of issue #123 where the circuit breaker
/// failed to open due to a race condition in the failure counter.
#[tokio::test]
async fn test_circuit_breaker_opens_after_threshold() {
    let breaker = CircuitBreaker::new(Config {
        failure_threshold: 5,
        timeout: Duration::from_secs(10),
    });

    // Simulate 5 failures
    for _ in 0..5 {
        let _ = breaker.call(|| async { Err(Error::Failure) }).await;
    }

    // Verify breaker is open
    assert_eq!(breaker.state(), State::Open);
}
```

---

### 13. Clean Up Resources

```rust
#[tokio::test]
async fn test_with_temp_file() {
    // Create temp file
    let temp_file = TempFile::new();

    // Test code
    write_to_file(&temp_file.path(), "data");

    // Cleanup happens automatically via Drop
}

// Implement Drop for automatic cleanup
impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
```

**Resource cleanup checklist**:
- [ ] Temporary files deleted
- [ ] Database connections closed
- [ ] Mock servers stopped
- [ ] Background tasks cancelled

---

### 14. Use Property-Based Testing for Invariants

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_url_parser_never_panics(url in "\\PC*") {
        // Property: parser should never panic, regardless of input
        let _ = parse_url(&url);
    }

    #[test]
    fn test_cache_size_invariant(
        ops in prop::collection::vec(any::<CacheOp>(), 0..100)
    ) {
        let mut cache = Cache::with_capacity(10);

        for op in ops {
            cache.execute(op);
        }

        // Property: cache never exceeds capacity
        assert!(cache.len() <= 10);
    }
}
```

---

### 15. Test Concurrent Behavior

```rust
#[tokio::test]
async fn test_rate_limiter_under_concurrent_load() {
    let limiter = Arc::new(RateLimiter::new(100));
    let mut handles = vec![];

    // Spawn 1000 concurrent requests
    for _ in 0..1000 {
        let limiter = limiter.clone();
        let handle = tokio::spawn(async move {
            limiter.allow().await
        });
        handles.push(handle);
    }

    // Collect results
    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // Exactly 100 should succeed
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 100);
    assert_eq!(results.iter().filter(|r| r.is_err()).count(), 900);
}
```

---

### 16. Avoid Flaky Tests

❌ **Common causes of flakiness**:
```rust
// ❌ Timing-dependent
#[test]
fn test_flaky_timing() {
    let start = Instant::now();
    slow_operation();
    assert!(start.elapsed() < Duration::from_millis(100));  // Flaky!
}

// ❌ Random data
#[test]
fn test_flaky_random() {
    let value = rand::random::<u32>();
    assert!(value > 100);  // Flaky!
}

// ❌ External dependencies
#[test]
fn test_flaky_network() {
    let result = fetch("https://example.com");  // Flaky!
    assert!(result.is_ok());
}
```

✅ **Solutions**:
```rust
// ✅ Use controlled time
#[test]
fn test_stable_timing() {
    let mut mock_time = MockTime::new();
    mock_time.set(Duration::from_secs(100));

    let result = operation_with_time(&mock_time);

    assert_eq!(result, expected);
}

// ✅ Use deterministic data
#[test]
fn test_stable_data() {
    let value = 42;  // Deterministic
    assert!(value > 0);
}

// ✅ Mock external dependencies
#[test]
fn test_stable_network() {
    let mock_client = MockHttpClient::new();
    mock_client.expect_get().returning(|_| Ok(response));

    let result = fetch_with_client(&mock_client);
    assert!(result.is_ok());
}
```

---

### 17. Test Performance Regressions

```rust
#[test]
fn test_no_performance_regression() {
    let input = generate_large_dataset();

    let start = Instant::now();
    process(input);
    let duration = start.elapsed();

    // Load baseline from previous runs
    let baseline = load_baseline("process_v1.0.0");

    // Allow 10% variance
    let threshold = baseline.mul_f32(1.1);

    assert!(
        duration < threshold,
        "Performance regression: {:?} exceeds baseline {:?}",
        duration,
        baseline
    );
}
```

---

### 18. Use Assertion Messages

❌ **Bad** - No context on failure:
```rust
#[test]
fn test_calculation() {
    assert_eq!(calculate(10), 20);  // Which calculation? Why 20?
}
```

✅ **Good** - Clear failure messages:
```rust
#[test]
fn test_calculation() {
    let input = 10;
    let result = calculate(input);
    let expected = 20;

    assert_eq!(
        result,
        expected,
        "calculate({}) should return {} (double the input), got {}",
        input,
        expected,
        result
    );
}
```

---

### 19. Group Related Tests

```rust
mod rate_limiter_tests {
    use super::*;

    mod configuration {
        #[test]
        fn test_accepts_valid_config() { }

        #[test]
        fn test_rejects_invalid_config() { }
    }

    mod behavior {
        #[test]
        fn test_allows_requests_under_limit() { }

        #[test]
        fn test_blocks_requests_over_limit() { }
    }

    mod edge_cases {
        #[test]
        fn test_zero_limit() { }

        #[test]
        fn test_max_limit() { }
    }
}
```

---

### 20. Review Test Coverage

```bash
# Generate coverage report
cargo tarpaulin --workspace --out Html

# Review uncovered lines
open tarpaulin-report.html

# Add tests for uncovered branches
```

**Coverage goals**:
- **Statements**: ≥80%
- **Branches**: ≥75%
- **Functions**: ≥80%
- **Lines**: ≥80%

---

## Summary Checklist

When writing a test, ask yourself:

- [ ] Does this test behavior, not implementation?
- [ ] Is it structured using AAA (Arrange-Act-Assert)?
- [ ] Does it test one logical thing?
- [ ] Is it independent of other tests?
- [ ] Does it have a descriptive name?
- [ ] Does it test edge cases and errors?
- [ ] Is it fast enough for its category?
- [ ] Does it use fixtures/builders for test data?
- [ ] Are external dependencies mocked?
- [ ] Is it deterministic (not flaky)?
- [ ] Does it clean up resources?
- [ ] Are assertions clear with good messages?
- [ ] Is it properly documented if non-obvious?

---

## Further Reading

- [London School TDD](https://github.com/mockito/mockito/wiki/Mockist-vs-Classicist-TDD)
- [Test Pyramid](https://martinfowler.com/bliki/TestPyramid.html)
- [Property-Based Testing](https://hypothesis.works/articles/what-is-property-based-testing/)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
