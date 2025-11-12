# Contract Tests Architecture

## Overview

Contract tests in Riptide are reusable test suites that validate trait implementations adhere to expected behaviors. They serve as executable specifications that ensure consistent behavior across different implementations of port traits.

## Purpose

Contract tests provide several key benefits:

1. **Consistency**: Ensure all implementations of a trait behave identically
2. **Documentation**: Serve as executable specification for trait behavior
3. **Quality Assurance**: Catch implementation bugs early
4. **Portability**: Enable easy swapping of implementations (e.g., Redis → Memcached)
5. **Test Reuse**: Write tests once, use across all implementations

## Architecture

### Hexagonal Architecture Integration

```
┌─────────────────────────────────────────────────────┐
│                Application Core                      │
│  ┌──────────────────────────────────────────────┐  │
│  │         Domain Logic (Riptide Core)          │  │
│  └──────────────────────────────────────────────┘  │
│                        │                             │
│           ┌────────────┴────────────┐               │
│           │                         │               │
│    ┌──────▼──────┐          ┌──────▼──────┐       │
│    │   Ports     │          │   Ports     │       │
│    │  (Traits)   │          │  (Traits)   │       │
│    └──────┬──────┘          └──────┬──────┘       │
└───────────┼──────────────────────────┼─────────────┘
            │                          │
   ┌────────▼────────┐        ┌────────▼────────┐
   │   Adapters      │        │   Adapters      │
   │ (Redis, etc)    │        │ (Postgres, etc) │
   └────────┬────────┘        └────────┬────────┘
            │                           │
   ┌────────▼────────┐        ┌────────▼────────┐
   │ Contract Tests  │        │ Contract Tests  │
   │   Validate      │        │   Validate      │
   │ Implementation  │        │ Implementation  │
   └─────────────────┘        └─────────────────┘
```

### Test Organization

```
crates/riptide-types/
├── src/
│   └── ports/              # Port trait definitions
│       ├── cache.rs        # CacheStorage trait
│       ├── session.rs      # SessionStorage trait
│       └── ...
└── tests/
    ├── contracts/          # Contract test modules
    │   ├── mod.rs          # Exports and documentation
    │   ├── README.md       # Contract tests guide
    │   ├── cache_storage_contract.rs
    │   ├── session_storage_contract.rs
    │   └── coordination_contract.rs
    ├── cache_storage_contract_tests.rs     # Integration tests
    ├── session_storage_contract_tests.rs   # Integration tests
    └── coordination_contract_tests.rs      # Integration tests
```

## Available Contract Test Suites

### 1. CacheStorage Contract

**Location**: `riptide-types/tests/contracts/cache_storage_contract.rs`

**Tests**:
- Basic CRUD operations (get, set, delete, exists)
- TTL expiration behavior
- Batch operations (mget, mset, delete_many)
- Atomic operations (incr)
- Binary data handling
- Large value storage
- Concurrent access patterns
- Health checks

**Example Usage**:
```rust
use riptide_types::ports::CacheStorage;
use riptide_types::tests::contracts::cache_storage_contract;

#[tokio::test]
async fn test_redis_cache_contract() {
    let cache = RedisCache::new("redis://localhost").await.unwrap();
    cache_storage_contract::run_all_tests(&cache).await.unwrap();
}
```

### 2. SessionStorage Contract

**Location**: `riptide-types/tests/contracts/session_storage_contract.rs`

**Tests**:
- CRUD operations (create, read, update, delete)
- Session expiration and cleanup
- Multi-tenancy isolation
- User-based filtering
- Active/expired filtering
- Metadata handling
- Concurrent session operations

**Example Usage**:
```rust
use riptide_types::ports::SessionStorage;
use riptide_types::tests::contracts::session_storage_contract;

#[tokio::test]
async fn test_postgres_session_storage_contract() {
    let storage = PostgresSessionStorage::new(&pool).await;
    session_storage_contract::run_all_tests(&storage).await.unwrap();
}
```

### 3. Coordination Contract

**Location**: `riptide-types/tests/contracts/coordination_contract.rs`

**Tests**:
- Set/delete notifications
- Pattern-based invalidation
- High-frequency operations
- Concurrent notifications
- Error handling and recovery
- Notification ordering

**Example Usage**:
```rust
use riptide_types::tests::contracts::coordination_contract::CacheSync;
use riptide_types::tests::contracts::coordination_contract;

#[tokio::test]
async fn test_redis_pubsub_contract() {
    let coordinator = RedisPubSub::new("redis://localhost").await.unwrap();
    coordination_contract::run_all_tests(&coordinator).await.unwrap();
}
```

## Implementation Guide

### Step 1: Implement the Trait

```rust
use riptide_types::ports::CacheStorage;
use riptide_types::error::Result as RiptideResult;
use async_trait::async_trait;

pub struct MyCache {
    // Your implementation
}

#[async_trait]
impl CacheStorage for MyCache {
    async fn get(&self, key: &str) -> RiptideResult<Option<Vec<u8>>> {
        // Your implementation
    }

    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> RiptideResult<()> {
        // Your implementation
    }

    // ... other methods
}
```

### Step 2: Add Contract Tests

Create a test file in your implementation crate:

```rust
// In crates/my-adapter/tests/contract_tests.rs
use my_adapter::MyCache;
use riptide_types::tests::contracts::cache_storage_contract;

#[tokio::test]
async fn test_my_cache_contract() {
    let cache = MyCache::new();

    // Run all contract tests
    cache_storage_contract::run_all_tests(&cache).await.unwrap();
}

#[tokio::test]
async fn test_my_cache_specific_behavior() {
    let cache = MyCache::new();

    // Run specific tests
    cache_storage_contract::test_basic_operations(&cache).await.unwrap();
    cache_storage_contract::test_ttl_expiration(&cache).await.unwrap();

    // Add implementation-specific tests
    // ...
}
```

### Step 3: Handle Test Dependencies

For tests requiring external services (Redis, Postgres):

```rust
#[tokio::test]
#[ignore = "requires Redis"]
async fn test_redis_cache_contract() {
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let cache = RedisCache::new(&redis_url).await.unwrap();
    cache_storage_contract::run_all_tests(&cache).await.unwrap();
}
```

Run with:
```bash
cargo test -- --ignored  # Run ignored tests
cargo test               # Run only non-ignored tests
```

## Contract Test Principles

### 1. Independence

Each test should be independent and not rely on state from other tests:

```rust
pub async fn test_basic_operations<C: CacheStorage>(cache: &C) -> RiptideResult<()> {
    let key = "test_key_unique";  // Unique key

    // Test
    cache.set(key, b"value", None).await?;
    // ... assertions

    // Cleanup
    cache.delete(key).await?;

    Ok(())
}
```

### 2. Comprehensive Coverage

Test all aspects of the trait:
- Happy path operations
- Edge cases (empty values, large values, special characters)
- Error conditions
- Concurrent access
- Resource cleanup

### 3. Clear Assertions

Use descriptive assertion messages:

```rust
assert!(
    retrieved.is_some(),
    "Value should be retrievable after set"
);
assert_eq!(
    retrieved.unwrap(),
    value,
    "Retrieved value should match original"
);
```

### 4. Performance Awareness

Keep tests fast:
- Use short TTLs in tests (1-2 seconds)
- Limit batch sizes to reasonable amounts
- Clean up resources promptly

## Testing Strategies

### Integration Testing Strategy

```
┌─────────────────────────────────────────────────┐
│         Test Pyramid for Port Traits           │
├─────────────────────────────────────────────────┤
│              E2E Tests                          │ <- System-wide scenarios
│           (Full Stack)                          │
├─────────────────────────────────────────────────┤
│         Integration Tests                       │ <- Real infrastructure
│       (Contract Tests +                         │    (Redis, Postgres, etc)
│        Real Adapters)                           │
├─────────────────────────────────────────────────┤
│           Unit Tests                            │ <- Logic validation
│      (Contract Tests +                          │    (In-memory mocks)
│       Mock Adapters)                            │
└─────────────────────────────────────────────────┘
```

### Test Execution Flow

1. **Development**: Run with in-memory implementations
   ```bash
   cargo test -p riptide-types
   ```

2. **Pre-commit**: Run adapter tests with real infrastructure
   ```bash
   cargo test -p riptide-persistence
   ```

3. **CI/CD**: Run all tests including integration
   ```bash
   cargo test --workspace
   ```

## Troubleshooting

### Common Issues

#### 1. TTL Tests Failing

**Problem**: TTL expiration tests fail intermittently

**Solution**:
- Use longer TTLs (2+ seconds)
- Add buffer time for clock skew
- Check system clock synchronization

```rust
// Instead of 1 second
cache.set(key, value, Some(Duration::from_secs(2))).await?;
tokio::time::sleep(Duration::from_millis(2100)).await;
```

#### 2. Concurrent Test Failures

**Problem**: Concurrent tests produce inconsistent results

**Solution**:
- Ensure thread-safe implementation
- Use unique keys per test
- Add synchronization where needed

```rust
let key = format!("test_key_{}", uuid::Uuid::new_v4());
```

#### 3. Resource Leaks

**Problem**: Tests leave data behind

**Solution**:
- Always clean up in tests
- Use unique prefixes for test data
- Implement Drop handlers if needed

```rust
let key = format!("test:{}:data", test_name);
// ... test
cache.delete(&key).await?;  // Cleanup
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Contract Tests

on: [push, pull_request]

jobs:
  contract-tests:
    runs-on: ubuntu-latest

    services:
      redis:
        image: redis:7-alpine
        ports:
          - 6379:6379

      postgres:
        image: postgres:16-alpine
        env:
          POSTGRES_PASSWORD: test
        ports:
          - 5432:5432

    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Run contract tests (in-memory)
        run: cargo test -p riptide-types

      - name: Run contract tests (Redis)
        env:
          REDIS_URL: redis://localhost:6379
        run: cargo test -p riptide-persistence -- --ignored

      - name: Run contract tests (Postgres)
        env:
          DATABASE_URL: postgres://postgres:test@localhost:5432/test
        run: cargo test -p riptide-persistence -- --ignored
```

## Best Practices

### 1. Test Organization

✅ **Do**: Organize tests by trait behavior
```rust
// cache_storage_contract.rs
pub async fn test_basic_operations(...) { }
pub async fn test_ttl_expiration(...) { }
pub async fn test_batch_operations(...) { }
```

❌ **Don't**: Mix implementation-specific tests
```rust
// cache_storage_contract.rs
pub async fn test_redis_connection(...) { }  // Wrong!
```

### 2. Error Handling

✅ **Do**: Test error paths
```rust
let result = cache.incr("non_numeric", 1).await;
assert!(result.is_err(), "Should error on non-numeric value");
```

❌ **Don't**: Ignore potential errors
```rust
cache.set(key, value, None).await.unwrap();  // Might panic
```

### 3. Documentation

✅ **Do**: Document what each test validates
```rust
/// Test TTL expiration behavior
///
/// Validates:
/// - Keys with TTL expire after duration
/// - Expired keys return None
/// - exists returns false for expired keys
pub async fn test_ttl_expiration<C: CacheStorage>(cache: &C) -> RiptideResult<()> {
```

❌ **Don't**: Leave tests undocumented
```rust
pub async fn test_stuff<C: CacheStorage>(cache: &C) -> RiptideResult<()> {
```

### 4. Performance

✅ **Do**: Keep tests fast
```rust
// Use small data sizes
let data = vec![0u8; 1024];  // 1KB

// Use short timeouts
tokio::time::timeout(Duration::from_secs(5), test_fn()).await?;
```

❌ **Don't**: Use production-scale data
```rust
// Avoid large data in tests
let data = vec![0u8; 100_000_000];  // 100MB - too large!
```

## Future Enhancements

Planned improvements to the contract test framework:

1. **Property-Based Testing**: Add quickcheck/proptest integration
2. **Fuzz Testing**: Add fuzzing for cache keys and values
3. **Performance Benchmarks**: Add criterion benchmarks to contracts
4. **Chaos Testing**: Add fault injection for error path testing
5. **Compliance Reports**: Generate compliance reports for implementations

## References

- [Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/)
- [Contract Testing](https://martinfowler.com/bliki/ContractTest.html)
- [Test Pyramid](https://martinfowler.com/articles/practical-test-pyramid.html)
- [Rust Testing Best Practices](https://doc.rust-lang.org/book/ch11-00-testing.html)
