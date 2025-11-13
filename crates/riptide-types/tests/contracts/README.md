# Contract Tests for Riptide Port Traits

This directory contains comprehensive contract tests for all port traits defined in `riptide-types`. Contract tests serve as executable specifications that validate any implementation adheres to the expected behavior.

## Purpose

Contract tests ensure that trait implementations:
- **Behave Correctly**: All operations work as documented
- **Handle Errors Gracefully**: Edge cases and errors are handled properly
- **Maintain Consistency**: State remains consistent across operations
- **Are Thread-Safe**: Operations are safe under concurrent access
- **Meet Performance Expectations**: Operations complete in reasonable time

## Available Contract Test Suites

### 1. CacheStorage Contract (`cache_storage_contract.rs`)

Tests for the `CacheStorage` trait covering:
- Basic get/set/delete operations
- TTL expiration behavior
- Batch operations (mget/mset)
- Atomic operations (incr)
- Large values and binary data
- Concurrent access patterns
- Health checks

**Usage:**
```rust
use riptide_types::tests::contracts::cache_storage_contract;

#[tokio::test]
async fn test_my_cache() {
    let cache = MyCache::new();
    cache_storage_contract::run_all_tests(&cache).await.unwrap();
}
```

### 2. SessionStorage Contract (`session_storage_contract.rs`)

Tests for the `SessionStorage` trait covering:
- CRUD operations
- Session expiration and cleanup
- Multi-tenancy isolation
- User-based filtering
- Metadata handling
- Concurrent session operations

**Usage:**
```rust
use riptide_types::tests::contracts::session_storage_contract;

#[tokio::test]
async fn test_my_session_storage() {
    let storage = MySessionStorage::new();
    session_storage_contract::run_all_tests(&storage).await.unwrap();
}
```

### 3. Coordination Contract (`coordination_contract.rs`)

Tests for the `CacheSync` trait covering:
- Set/delete notifications
- Pattern-based invalidation
- High-frequency operations
- Concurrent notifications
- Error handling and recovery

**Usage:**
```rust
use riptide_types::tests::contracts::coordination_contract;

#[tokio::test]
async fn test_my_coordinator() {
    let coordinator = MyCoordinator::new();
    coordination_contract::run_all_tests(&coordinator).await.unwrap();
}
```

## Running Contract Tests

### Run all contract tests
```bash
cargo test -p riptide-types --test '*'
```

### Run specific contract test suite
```bash
cargo test -p riptide-types cache_storage_contract
cargo test -p riptide-types session_storage_contract
cargo test -p riptide-types coordination_contract
```

### Run with detailed output
```bash
cargo test -p riptide-types -- --nocapture --test-threads=1
```

## Using Contract Tests for New Implementations

### Step 1: Implement the Trait

```rust
use riptide_types::ports::CacheStorage;
use async_trait::async_trait;

pub struct MyCache {
    // Your implementation
}

#[async_trait]
impl CacheStorage for MyCache {
    async fn get(&self, key: &str) -> RiptideResult<Option<Vec<u8>>> {
        // Your implementation
    }

    // ... other methods
}
```

### Step 2: Create Test Module

Create a test file in your crate (e.g., `tests/my_cache_tests.rs`):

```rust
use my_crate::MyCache;
use riptide_types::tests::contracts::cache_storage_contract;

#[tokio::test]
async fn test_cache_contract() {
    let cache = MyCache::new();

    // Run all contract tests
    cache_storage_contract::run_all_tests(&cache).await.unwrap();
}

#[tokio::test]
async fn test_cache_specific_behavior() {
    let cache = MyCache::new();

    // Run individual tests
    cache_storage_contract::test_basic_operations(&cache).await.unwrap();
    cache_storage_contract::test_ttl_expiration(&cache).await.unwrap();

    // Add your own specific tests
    // ...
}
```

### Step 3: Run Tests

```bash
cargo test -p my-crate
```

## Test Structure

Each contract test follows this pattern:

1. **Setup**: Create test data with unique identifiers
2. **Execute**: Perform the operation being tested
3. **Assert**: Verify the expected behavior
4. **Cleanup**: Remove test data to avoid interference

Example:
```rust
pub async fn test_basic_operations<C: CacheStorage>(cache: &C) -> RiptideResult<()> {
    // Setup
    let key = "test_key_unique_id";
    let value = b"test_value";

    // Execute & Assert
    cache.set(key, value, None).await?;
    let retrieved = cache.get(key).await?;
    assert_eq!(retrieved.unwrap(), value);

    // Cleanup
    cache.delete(key).await?;

    Ok(())
}
```

## Best Practices

### For Test Writers

1. **Use Unique Keys**: Avoid key collisions by including test name or UUID
2. **Clean Up**: Always delete test data after tests
3. **Test Edge Cases**: Include boundary conditions, empty values, large values
4. **Test Concurrency**: Validate thread-safety with concurrent operations
5. **Document Expectations**: Clearly state what each test validates

### For Implementation Writers

1. **Run All Tests**: Use `run_all_tests()` to ensure full compliance
2. **Check Performance**: Tests should complete quickly (< 1s each)
3. **Handle Errors**: Return meaningful errors, not panics
4. **Support Concurrency**: Ensure thread-safe implementations
5. **Document Limitations**: If your implementation has known limitations, document them

## Integration Tests vs Contract Tests

**Contract Tests** (this directory):
- Validate trait behavior
- Reusable across implementations
- Focus on interface compliance
- Located in `riptide-types/tests/contracts/`

**Integration Tests** (implementation crates):
- Test specific implementations
- Include infrastructure setup (Redis, databases)
- Test interaction with real systems
- Located in implementation crates (e.g., `riptide-persistence/tests/`)

## Adding New Contract Tests

When adding a new port trait:

1. Create `new_trait_contract.rs` in this directory
2. Implement test functions covering all trait methods
3. Include `run_all_tests()` convenience function
4. Add self-tests with a mock implementation
5. Update `mod.rs` to export the new module
6. Add documentation to this README
7. Update examples showing usage

## Troubleshooting

### Tests Fail Due to Timing Issues

Some tests (especially TTL tests) may be sensitive to timing. Consider:
- Using longer TTLs (e.g., 2 seconds instead of 1)
- Adding small delays before assertions
- Adjusting sleep durations for CI environments

### Tests Fail Intermittently

Intermittent failures often indicate:
- Race conditions in concurrent tests
- Insufficient cleanup between tests
- External dependencies (use mocks in contract tests)

### Performance Issues

If tests are slow:
- Check if implementation has O(n) operations where O(1) expected
- Reduce test data sizes while maintaining coverage
- Use simpler test fixtures

## Contributing

When contributing new contract tests:
1. Ensure tests are deterministic
2. Include both positive and negative test cases
3. Test edge cases and boundary conditions
4. Add documentation explaining what each test validates
5. Run tests locally before submitting PR

## Questions?

- Check existing contract tests for patterns
- Review trait documentation in `src/ports/`
- Open an issue for clarification
- Consult the main project README
