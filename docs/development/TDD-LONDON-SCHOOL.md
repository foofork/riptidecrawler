# TDD London School Guide for RipTide

## Overview

This guide documents the Test-Driven Development (TDD) approach used in RipTide, following the London School (mockist) methodology.

## Core Principles

### RED-GREEN-REFACTOR Cycle

The fundamental TDD workflow consists of three phases:

#### 1. RED Phase - Write Failing Test First

**Goal**: Define expected behavior before implementation exists

```rust
#[tokio::test]
async fn test_redis_pool_reuses_connections() {
    // ARRANGE: Setup (test will fail - nothing implemented yet)
    let pool = RedisPool::new("redis://localhost:6379", RedisConfig::default())
        .await
        .expect("Failed to create pool");

    // ACT: Perform action
    let conn1 = pool.get().await.expect("Failed to get connection 1");
    let conn2 = pool.get().await.expect("Failed to get connection 2");

    // ASSERT: Verify behavior
    assert!(Arc::ptr_eq(&conn1.inner(), &conn2.inner()),
        "Connections should share same underlying pool");
}
```

**Characteristics of RED Phase**:
- Test **must fail** (compilation error or runtime failure)
- Describes desired behavior, not implementation
- Uses interfaces that don't exist yet
- Clear, descriptive test name

#### 2. GREEN Phase - Make Test Pass

**Goal**: Implement minimal code to make test pass

```rust
// crates/riptide-utils/src/redis.rs

use redis::aio::ConnectionManager;
use std::sync::Arc;

pub struct RedisPool {
    manager: Arc<ConnectionManager>,
    config: RedisConfig,
}

impl RedisPool {
    pub async fn new(url: &str, config: RedisConfig) -> Result<Self> {
        let client = redis::Client::open(url)?;
        let manager = ConnectionManager::new(client).await?;

        Ok(Self {
            manager: Arc::new(manager),
            config,
        })
    }

    pub async fn get(&self) -> Result<ConnectionManager> {
        // Clone Arc - shares underlying connection
        Ok(self.manager.as_ref().clone())
    }
}
```

**Characteristics of GREEN Phase**:
- **Simplest implementation** that makes test pass
- Don't optimize or add features not tested
- Test now **passes**
- Code may be rough/duplicated (that's okay!)

#### 3. REFACTOR Phase - Improve Code Quality

**Goal**: Clean up implementation while keeping tests green

```rust
impl RedisPool {
    // Refactor: Add health checks without breaking existing test
    pub async fn new(url: &str, config: RedisConfig) -> Result<Self> {
        let client = redis::Client::open(url)?;
        let manager = ConnectionManager::new(client).await?;

        let pool = Self {
            manager: Arc::new(manager),
            config,
        };

        // Improved: Start health check background task
        pool.start_health_checks();

        Ok(pool)
    }

    fn start_health_checks(&self) {
        let manager = self.manager.clone();
        let interval = self.config.health_check_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            loop {
                interval.tick().await;
                if let Err(e) = redis::cmd("PING")
                    .query_async::<_, ()>(&mut manager.as_ref().clone())
                    .await
                {
                    tracing::warn!("Redis health check failed: {}", e);
                }
            }
        });
    }
}
```

**Characteristics of REFACTOR Phase**:
- Tests **stay green** throughout
- Extract duplication
- Improve naming
- Add error handling
- Optimize performance
- Document complex logic

---

## London School vs. Chicago School

### London School (Mockist) - **Used in RipTide**

**Philosophy**: Test object interactions and behavior

**Approach**:
- **Mock all dependencies** (database, HTTP, file system)
- Test in **complete isolation**
- Focus on **messages between objects**
- Test **behavior, not state**

**Example**:

```rust
#[tokio::test]
async fn test_spider_delegates_to_extractor() {
    // ARRANGE: Mock all dependencies
    let mut mock_fetcher = MockFetcher::new();
    let mut mock_extractor = MockExtractor::new();

    // EXPECT: Define expected interactions
    mock_fetcher.expect_fetch()
        .with(eq("https://example.com"))
        .times(1)
        .returning(|_| Ok("<html>...".to_string()));

    mock_extractor.expect_extract()
        .with(eq("<html>..."))
        .times(1)
        .returning(|_| Ok(Document { title: "Test" }));

    // ACT: Execute
    let spider = Spider::new(mock_fetcher, mock_extractor);
    let result = spider.crawl("https://example.com").await;

    // ASSERT: Mocks verify interactions happened
    assert!(result.is_ok());
    // Mock expectations automatically verified on drop
}
```

**Advantages**:
- Fast tests (no real I/O)
- Tests stay focused (one class at a time)
- Forces good design (loose coupling)
- Pinpoints failures quickly

**Disadvantages**:
- More setup code (mocks)
- Can be brittle if implementation changes
- Need integration tests to verify real behavior

### Chicago School (Classicist)

**Philosophy**: Test final state after operations

**Approach**:
- Use **real dependencies** where possible
- Test **observable outcomes**
- Verify **state changes**
- Fewer mocks, more integration

**Example**:

```rust
#[tokio::test]
async fn test_spider_crawls_real_site() {
    // Use REAL HTTP client and fetcher
    let spider = Spider::new_with_real_http();

    // Test against real site (or test fixture server)
    let result = spider.crawl("https://example.com").await;

    // Verify final state
    assert!(result.is_ok());
    assert_eq!(result.unwrap().urls.len(), 10);
}
```

**Why RipTide Uses London School**:
1. **Fast tests**: No waiting for HTTP, Redis, disk I/O
2. **Reliable**: No flaky network failures in unit tests
3. **Isolated**: Test one component without dependencies
4. **CI-friendly**: Tests run in <10 minutes without Docker

---

## Test Structure

### Anatomy of a Good Test

```rust
#[tokio::test]
async fn test_retry_exponential_backoff() {
    // ARRANGE: Setup test environment
    let policy = RetryPolicy {
        max_attempts: 3,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(30),
        backoff_factor: 2.0,
    };

    let attempt_times = Arc::new(Mutex::new(Vec::new()));
    let times_clone = attempt_times.clone();

    // ACT: Execute operation under test
    let start = Instant::now();

    let result = policy.execute(|| async {
        let mut times = times_clone.lock().unwrap();
        times.push(start.elapsed());

        if times.len() < 3 {
            Err(TestError::Transient)
        } else {
            Ok("success".to_string())
        }
    }).await;

    // ASSERT: Verify expected behavior
    assert!(result.is_ok(), "Should succeed after retries");

    let times = attempt_times.lock().unwrap();
    assert_eq!(times.len(), 3, "Should make 3 attempts");

    // Verify exponential backoff delays
    let delay1 = times[1].as_millis() - times[0].as_millis();
    let delay2 = times[2].as_millis() - times[1].as_millis();

    assert!(delay1 >= 90 && delay1 <= 150, "First delay ~100ms");
    assert!(delay2 >= 180 && delay2 <= 250, "Second delay ~200ms");

    let ratio = delay2 as f64 / delay1 as f64;
    assert!(ratio >= 1.8 && ratio <= 2.2, "Exponential growth ~2x");
}
```

### Test Naming Convention

**Pattern**: `test_<component>_<behavior>_<context>`

**Examples**:
- `test_redis_pool_reuses_connections`
- `test_retry_exponential_backoff`
- `test_rate_limiter_blocks_requests_exceeding_quota`
- `test_http_client_timeout_behavior`
- `test_secrets_redacted_in_debug_output`

**Good Names**:
- ✅ `test_retry_stops_after_max_attempts`
- ✅ `test_rate_limiter_replenishes_quota_over_time`
- ✅ `test_redis_pool_health_check_failure_handling`

**Bad Names**:
- ❌ `test_retry` (too vague)
- ❌ `test_1` (no meaning)
- ❌ `test_redis_works` (not specific)

---

## Mocking Strategies

### Using mockall Crate

```rust
use mockall::predicate::*;
use mockall::mock;

// Define trait to mock
#[async_trait]
pub trait HttpFetcher: Send + Sync {
    async fn fetch(&self, url: &str) -> Result<String>;
}

// Generate mock implementation
mock! {
    pub HttpFetcher {}

    #[async_trait]
    impl HttpFetcher for HttpFetcher {
        async fn fetch(&self, url: &str) -> Result<String>;
    }
}

// Use in tests
#[tokio::test]
async fn test_with_mock() {
    let mut mock = MockHttpFetcher::new();

    mock.expect_fetch()
        .with(eq("https://example.com"))
        .times(1)
        .returning(|_| Ok("<html>content</html>".to_string()));

    let result = mock.fetch("https://example.com").await;
    assert!(result.is_ok());
}
```

### Manual Mocks for Simple Cases

```rust
pub struct FakeRedisClient {
    data: Arc<Mutex<HashMap<String, String>>>,
}

impl FakeRedisClient {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn set(&self, key: String, value: String) -> Result<()> {
        self.data.lock().unwrap().insert(key, value);
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        Ok(self.data.lock().unwrap().get(key).cloned())
    }
}
```

---

## Test Coverage Requirements

### Minimum Coverage Targets

- **Statements**: >80%
- **Branches**: >75%
- **Functions**: >80%
- **Lines**: >80%

### Measuring Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage for specific package
cargo tarpaulin -p riptide-utils --out Html

# Run coverage for all Phase 0 tests
cargo tarpaulin --test 'phase0_*' --out Html

# View HTML report
open tarpaulin-report.html
```

### What to Test

**Must Test**:
- ✅ Public API methods
- ✅ Error handling paths
- ✅ Edge cases and boundary conditions
- ✅ Retry logic and backoff
- ✅ Concurrent access (thread safety)
- ✅ Resource cleanup (Drop, shutdown)

**Can Skip**:
- ❌ Getters/setters with no logic
- ❌ Trivial delegators
- ❌ Generated code (serde derives)
- ❌ Private implementation details (test behavior, not implementation)

---

## Test Organization

### Directory Structure

```
tests/
├── phase0/
│   ├── README.md
│   ├── unit/
│   │   ├── test_redis_pool.rs
│   │   ├── test_http_client.rs
│   │   ├── test_retry_policy.rs
│   │   ├── test_rate_limiter.rs
│   │   └── test_config_secrets.rs
│   ├── integration/
│   │   └── phase0_integration_tests.rs
│   └── fixtures/
│       ├── http_fixtures.rs
│       └── golden/
│           ├── sitemap.xml
│           ├── events.ics
│           └── event_jsonld.html
```

### Test File Organization

```rust
// tests/phase0/unit/test_redis_pool.rs

// 1. Imports
use std::sync::Arc;
use std::time::Duration;

// 2. Test module
#[cfg(test)]
mod redis_pool_tests {
    use super::*;

    // 3. Unit tests (RED phase)
    #[tokio::test]
    async fn test_redis_pool_reuses_connections() { ... }

    #[tokio::test]
    async fn test_redis_pool_health_checks() { ... }

    #[tokio::test]
    async fn test_redis_pool_retry_logic() { ... }
}

// 4. Mock implementations
#[cfg(test)]
mod mocks {
    pub struct MockRedisClient { ... }
}

// 5. Test helpers
#[cfg(test)]
mod test_helpers {
    pub fn create_test_pool() -> RedisPool { ... }
}

// 6. Implementation checklist (documentation)
/// Implementation Checklist (GREEN Phase)
///
/// 1. RedisPool struct with Arc<ConnectionManager>
/// 2. new() method with health checks
/// 3. get() method returning cloned Arc
/// ...
```

---

## Common Testing Patterns

### 1. Test Async Operations

```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### 2. Test Concurrent Access

```rust
#[tokio::test]
async fn test_concurrent_access() {
    let shared = Arc::new(SimpleRateLimiter::new(100));

    let handles: Vec<_> = (0..200)
        .map(|_| {
            let limiter = shared.clone();
            tokio::spawn(async move {
                limiter.check()
            })
        })
        .collect();

    let results = futures::future::join_all(handles).await;

    let allowed = results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(allowed, 100);
}
```

### 3. Test Error Handling

```rust
#[tokio::test]
async fn test_error_handling() {
    let result = fallible_operation().await;

    assert!(result.is_err(), "Should fail");

    let error = result.unwrap_err();
    assert_eq!(error.to_string(), "Expected error message");
}
```

### 4. Test Timeouts

```rust
#[tokio::test]
async fn test_timeout() {
    let start = Instant::now();

    let result = timeout(
        Duration::from_millis(100),
        slow_operation()
    ).await;

    assert!(result.is_err(), "Should timeout");
    assert!(start.elapsed() < Duration::from_millis(150));
}
```

### 5. Test Retries with Backoff

```rust
#[tokio::test]
async fn test_retry_with_backoff() {
    let attempt_times = Arc::new(Mutex::new(Vec::new()));
    let times = attempt_times.clone();

    let start = Instant::now();

    retry_policy.execute(|| async {
        times.lock().unwrap().push(start.elapsed());
        // Fail first N-1 attempts, succeed last
    }).await;

    let times = attempt_times.lock().unwrap();

    // Verify exponential delays
    for i in 1..times.len() {
        let delay = times[i] - times[i-1];
        // Assert delay grows exponentially
    }
}
```

---

## Integration Testing with Fixtures

### Using Wiremock for HTTP

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};

#[tokio::test]
async fn test_http_integration() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Setup expectations
    Mock::given(wiremock::matchers::path("/api/data"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({ "status": "ok" })))
        .mount(&mock_server)
        .await;

    // Test with real HTTP client
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/data", mock_server.uri()))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}
```

### Golden Test Fixtures

```rust
#[test]
fn test_ics_parsing() {
    // Load recorded fixture
    let ics_content = include_str!("../fixtures/golden/events.ics");

    // Parse
    let events = parse_ics(ics_content).unwrap();

    // Verify against known good output
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].title, "Phase 0 Planning Meeting");
}
```

---

## Continuous Integration

### Running Tests in CI

```yaml
# .github/workflows/test.yml
test:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4

    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable

    - name: Run Phase 0 tests
      run: cargo test --test 'phase0_*'

    - name: Check coverage
      run: |
        cargo tarpaulin --test 'phase0_*' --out Xml
        if [ $(grep -oP 'line-rate="\K[0-9.]+' cobertura.xml | head -1 | awk '{print ($1 < 0.8)}') -eq 1 ]; then
          echo "Coverage below 80%"
          exit 1
        fi
```

---

## Best Practices

### DO

✅ Write test **before** implementation (RED phase)
✅ Test **one behavior** per test
✅ Use **descriptive test names**
✅ Mock **all external dependencies**
✅ Test **error paths** and edge cases
✅ Measure and maintain **>80% coverage**
✅ Keep tests **fast** (<100ms for unit tests)
✅ Make tests **deterministic** (no flaky tests)

### DON'T

❌ Skip RED phase (write tests after code)
❌ Test implementation details
❌ Use real HTTP/Redis/disk in unit tests
❌ Have tests depend on each other
❌ Leave commented-out tests
❌ Test getters/setters with no logic
❌ Mock what you don't own (use adapters)
❌ Ignore failing tests

---

## Troubleshooting

### Test is Flaky

**Problem**: Test passes sometimes, fails others

**Solutions**:
- Use deterministic fixtures (not real network)
- Add timeouts to async operations
- Fix race conditions with proper synchronization
- Avoid `sleep()` - use mocks with controlled timing

### Test is Slow

**Problem**: Unit test takes >100ms

**Solutions**:
- Mock external I/O (HTTP, Redis, disk)
- Use smaller datasets in tests
- Reduce retry/timeout durations for tests
- Move to integration tests if truly needs real I/O

### Mock Expectations Fail

**Problem**: Mock was called wrong number of times

**Solutions**:
- Verify test logic (are you calling it right?)
- Use `.times(1..)` for "at least once"
- Check async/await (ensure futures complete)
- Debug with `.with(any())` to see what's being called

---

## Phase 0 Test Examples

See comprehensive examples in:
- `tests/phase0/unit/test_redis_pool.rs`
- `tests/phase0/unit/test_retry_policy.rs`
- `tests/phase0/unit/test_rate_limiter.rs`
- `tests/phase0/unit/test_http_client.rs`
- `tests/phase0/unit/test_config_secrets.rs`
- `tests/phase0/integration/phase0_integration_tests.rs`

---

## References

- [London vs Chicago School](https://softwareengineering.stackexchange.com/questions/123627/what-are-the-london-and-chicago-schools-of-tdd)
- [mockall crate](https://docs.rs/mockall/)
- [wiremock crate](https://docs.rs/wiremock/)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
