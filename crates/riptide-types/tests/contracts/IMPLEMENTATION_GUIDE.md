# Contract Tests Implementation Guide

## Quick Start

This guide helps you implement and test new port trait adapters using contract tests.

## Step-by-Step Guide

### 1. Implement Your Adapter

Choose the trait you're implementing and create your adapter:

```rust
// In your adapter crate (e.g., crates/my-adapter/src/lib.rs)
use riptide_types::ports::CacheStorage;
use riptide_types::error::Result as RiptideResult;
use async_trait::async_trait;
use std::time::Duration;

pub struct MyCache {
    // Your internal state
    connection: YourConnectionType,
}

impl MyCache {
    pub async fn new(config: &str) -> Result<Self, YourError> {
        // Initialize your cache
        Ok(Self {
            connection: YourConnectionType::connect(config).await?,
        })
    }
}

#[async_trait]
impl CacheStorage for MyCache {
    async fn get(&self, key: &str) -> RiptideResult<Option<Vec<u8>>> {
        // Your implementation
        todo!()
    }

    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> RiptideResult<()> {
        // Your implementation
        todo!()
    }

    async fn delete(&self, key: &str) -> RiptideResult<()> {
        // Your implementation
        todo!()
    }

    async fn exists(&self, key: &str) -> RiptideResult<bool> {
        // Your implementation
        todo!()
    }

    // Implement remaining methods or use trait defaults
}
```

### 2. Add Contract Tests

Create a test file in your adapter crate:

```rust
// In crates/my-adapter/tests/cache_contract_tests.rs
use my_adapter::MyCache;
use riptide_types::tests::contracts::cache_storage_contract;

// Helper to setup test cache
async fn setup_test_cache() -> MyCache {
    let config = std::env::var("MY_CACHE_URL")
        .unwrap_or_else(|_| "default://localhost".to_string());

    MyCache::new(&config).await.expect("Failed to create test cache")
}

// Run all contract tests
#[tokio::test]
#[ignore = "requires external service"]
async fn test_cache_all_contracts() {
    let cache = setup_test_cache().await;
    cache_storage_contract::run_all_tests(&cache).await.unwrap();
}

// Run individual contract tests for debugging
#[tokio::test]
#[ignore = "requires external service"]
async fn test_cache_basic_operations() {
    let cache = setup_test_cache().await;
    cache_storage_contract::test_basic_operations(&cache).await.unwrap();
}

#[tokio::test]
#[ignore = "requires external service"]
async fn test_cache_ttl_expiration() {
    let cache = setup_test_cache().await;
    cache_storage_contract::test_ttl_expiration(&cache).await.unwrap();
}

// Add implementation-specific tests
#[tokio::test]
async fn test_my_specific_feature() {
    let cache = setup_test_cache().await;
    // Test implementation-specific behaviors
}
```

### 3. Run Tests During Development

```bash
# Run unit tests (without external dependencies)
cargo test -p my-adapter

# Run integration tests (with external services)
cargo test -p my-adapter -- --ignored

# Run specific test
cargo test -p my-adapter test_cache_basic_operations -- --ignored

# Run with output for debugging
cargo test -p my-adapter -- --ignored --nocapture
```

### 4. Add to CI/CD

Update your CI configuration to run contract tests:

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Run unit tests
        run: cargo test -p my-adapter

  integration-tests:
    runs-on: ubuntu-latest
    services:
      # Add your service (Redis, Postgres, etc)
      cache-service:
        image: your-cache:latest
        ports:
          - 6379:6379

    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Run contract tests
        env:
          MY_CACHE_URL: your://localhost:6379
        run: cargo test -p my-adapter -- --ignored
```

## Available Contract Test Suites

### CacheStorage Contracts

**Import**: `use riptide_types::tests::contracts::cache_storage_contract;`

**Available Tests**:
- `test_basic_operations` - get, set, delete, exists
- `test_ttl_expiration` - TTL behavior and expiration
- `test_batch_operations` - mget, mset
- `test_delete_many` - Batch deletion
- `test_expire` - Setting TTL on existing keys
- `test_ttl_query` - Querying remaining TTL
- `test_incr` - Atomic increment operations
- `test_health_check` - Health check behavior
- `test_large_values` - Handling large data
- `test_binary_data` - Binary data integrity
- `test_concurrent_operations` - Thread safety
- `test_empty_values` - Empty value handling
- `run_all_tests` - Run all tests

### SessionStorage Contracts

**Import**: `use riptide_types::tests::contracts::session_storage_contract;`

**Available Tests**:
- `test_crud_operations` - Create, read, update, delete
- `test_expiration` - Session expiration and cleanup
- `test_multi_tenancy` - Tenant isolation
- `test_user_filtering` - Filter by user ID
- `test_active_filtering` - Filter active/expired
- `test_metadata` - Metadata handling
- `test_concurrent_operations` - Thread safety
- `test_empty_list` - Empty result handling
- `test_cleanup_no_expired` - Cleanup with no expired sessions
- `run_all_tests` - Run all tests

### Coordination Contracts

**Import**: `use riptide_types::tests::contracts::coordination_contract;`

**Available Tests**:
- `test_basic_notifications` - Set/delete notifications
- `test_pattern_invalidation` - Pattern-based invalidation
- `test_high_frequency` - Rapid notifications
- `test_concurrent_notifications` - Concurrent access
- `test_error_handling` - Error recovery
- `test_notification_ordering` - Ordering guarantees
- `test_mixed_operations` - Mixed operation types
- `test_large_batch` - Large batch operations
- `run_all_tests` - Run all tests

## Common Patterns

### Pattern 1: Test with Docker Compose

```yaml
# docker-compose.test.yml
version: '3.8'

services:
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"

  tests:
    build: .
    depends_on:
      - redis
    environment:
      - REDIS_URL=redis://redis:6379
    command: cargo test -- --ignored
```

Run with:
```bash
docker-compose -f docker-compose.test.yml up --abort-on-container-exit
```

### Pattern 2: Test with Testcontainers

```rust
use testcontainers::{clients, images};

async fn setup_test_cache() -> MyCache {
    let docker = clients::Cli::default();
    let redis = docker.run(images::redis::Redis::default());
    let port = redis.get_host_port_ipv4(6379);

    let url = format!("redis://localhost:{}", port);
    MyCache::new(&url).await.expect("Failed to create cache")
}

#[tokio::test]
async fn test_with_testcontainers() {
    let cache = setup_test_cache().await;
    cache_storage_contract::run_all_tests(&cache).await.unwrap();
}
```

### Pattern 3: Test with In-Memory Mock

For development without external dependencies:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MemoryCache {
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MemoryCache {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl CacheStorage for MemoryCache {
    async fn get(&self, key: &str) -> RiptideResult<Option<Vec<u8>>> {
        let data = self.data.read().await;
        Ok(data.get(key).cloned())
    }

    // ... implement other methods
}

#[tokio::test]
async fn test_memory_cache() {
    let cache = MemoryCache::new();
    cache_storage_contract::test_basic_operations(&cache).await.unwrap();
}
```

## Debugging Failing Tests

### Test Fails: "Key not found"

**Cause**: Set operation didn't persist or cleanup from previous test

**Solution**:
```rust
// Use unique keys
let key = format!("test_{}_{}", test_name, uuid::Uuid::new_v4());

// Clean up before and after
cache.delete(&key).await?;  // Clean before
// ... test
cache.delete(&key).await?;  // Clean after
```

### Test Fails: "TTL test timing out"

**Cause**: TTL not expiring or clock skew

**Solution**:
```rust
// Use longer TTLs and waits
cache.set(key, value, Some(Duration::from_secs(2))).await?;
tokio::time::sleep(Duration::from_millis(2100)).await;  // Wait slightly longer

// Check if TTL is supported
if cache.ttl(key).await?.is_none() {
    // TTL query not supported, skip or use alternative test
}
```

### Test Fails: "Concurrent test race condition"

**Cause**: Non-thread-safe implementation

**Solution**:
```rust
// Ensure your implementation uses proper synchronization
use tokio::sync::RwLock;

pub struct MyCache {
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}
```

### Test Fails: "Connection refused"

**Cause**: External service not running

**Solution**:
```bash
# Start service
docker run -d -p 6379:6379 redis:7-alpine

# Or mark test as ignored
#[tokio::test]
#[ignore = "requires Redis"]
async fn test_redis_cache() {
    // ...
}
```

## Best Practices

### ✅ Do

1. **Use unique keys**: Prevent test interference
   ```rust
   let key = format!("test_{}_{}", test_name, uuid::Uuid::new_v4());
   ```

2. **Clean up resources**: Always delete test data
   ```rust
   // At end of test
   cache.delete(&key).await?;
   ```

3. **Handle optional features**: Check support before testing
   ```rust
   if cache.ttl(key).await?.is_some() {
       // TTL supported, run TTL tests
   }
   ```

4. **Use environment variables**: Make tests configurable
   ```rust
   let url = std::env::var("CACHE_URL")
       .unwrap_or_else(|_| "default://localhost".to_string());
   ```

5. **Add timeouts**: Prevent hanging tests
   ```rust
   tokio::time::timeout(
       Duration::from_secs(5),
       cache.get(key)
   ).await??;
   ```

### ❌ Don't

1. **Don't use hardcoded data**: Use generated test data
   ```rust
   // Bad
   cache.set("key", b"value", None).await?;

   // Good
   let key = format!("test_{}", uuid::Uuid::new_v4());
   cache.set(&key, b"value", None).await?;
   ```

2. **Don't ignore cleanup**: Always clean up
   ```rust
   // Bad
   cache.set(key, value, None).await?;
   // Test ends without cleanup

   // Good
   cache.set(key, value, None).await?;
   // ... test
   cache.delete(key).await?;
   ```

3. **Don't test external service details**: Focus on trait contract
   ```rust
   // Bad - testing Redis-specific features
   let redis_info = cache.get_redis_info().await?;

   // Good - testing trait contract
   let exists = cache.exists(key).await?;
   ```

4. **Don't use production data**: Use test data only
   ```rust
   // Bad
   let prod_url = "redis://prod.example.com";

   // Good
   let test_url = "redis://localhost:6379";
   ```

## Checklist

Before submitting your implementation, verify:

- [ ] All contract tests pass
- [ ] Implementation is thread-safe (concurrent tests pass)
- [ ] Resources are properly cleaned up
- [ ] Error cases are handled gracefully
- [ ] Tests are marked with `#[ignore]` if they need external services
- [ ] CI/CD configuration includes contract tests
- [ ] Documentation explains any implementation-specific behavior
- [ ] Performance meets reasonable expectations

## Getting Help

- Check existing implementations in `crates/riptide-persistence/`
- Review contract test source in `crates/riptide-types/tests/contracts/`
- See architecture docs in `/workspaces/riptidecrawler/docs/architecture/CONTRACT_TESTS.md`
- Open an issue if you find gaps in contract tests

## Contributing

If you find missing test cases in the contract tests:

1. Add the test case to the appropriate contract module
2. Ensure the test documents what it validates
3. Test against an in-memory implementation
4. Submit a PR with the enhancement
