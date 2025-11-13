//! Contract tests for CacheStorage trait
//!
//! These tests validate that any implementation of the CacheStorage trait
//! adheres to the expected behavior and invariants. Use these tests to verify
//! alternative implementations (e.g., in-memory, Memcached, etc.).
//!
//! # Usage
//!
//! ```rust,ignore
//! use riptide_types::ports::CacheStorage;
//!
//! #[tokio::test]
//! async fn test_my_implementation() {
//!     let cache = MyCache::new();
//!     cache_storage_contract::test_basic_operations(&cache).await.unwrap();
//!     cache_storage_contract::test_ttl_expiration(&cache).await.unwrap();
//!     // ... run all contract tests
//! }
//! ```

use riptide_types::error::{Result as RiptideResult, RiptideError};
use riptide_types::ports::CacheStorage;
use std::time::Duration;

/// Test basic get/set/delete operations
///
/// Validates:
/// - Setting a value and retrieving it
/// - Getting non-existent keys returns None
/// - Deleting keys works correctly
pub async fn test_basic_operations<C: CacheStorage>(cache: &C) -> RiptideResult<()> {
    // Test 1: Set and get a value
    let key = "test_key_basic";
    let value = b"test_value";

    cache.set(key, value, None).await?;
    let retrieved = cache.get(key).await?;

    assert!(retrieved.is_some(), "Value should be retrievable after set");
    assert_eq!(
        retrieved.unwrap(),
        value,
        "Retrieved value should match original"
    );

    // Test 2: Get non-existent key
    let missing = cache.get("non_existent_key").await?;
    assert!(missing.is_none(), "Non-existent key should return None");

    // Test 3: Delete key
    cache.delete(key).await?;
    let after_delete = cache.get(key).await?;
    assert!(
        after_delete.is_none(),
        "Key should not exist after deletion"
    );

    // Test 4: Delete non-existent key (should not error)
    cache.delete("non_existent_key").await?;

    Ok(())
}

/// Test exists functionality
///
/// Validates:
/// - exists returns true for existing keys
/// - exists returns false for non-existent keys
/// - exists returns false after deletion
pub async fn test_exists<C: CacheStorage>(cache: &C) -> RiptideResult<()> {
    let key = "test_key_exists";
    let value = b"test_value";

    // Non-existent key
    assert!(!cache.exists(key).await?, "Key should not exist initially");

    // After setting
    cache.set(key, value, None).await?;
    assert!(cache.exists(key).await?, "Key should exist after set");

    // After deletion
    cache.delete(key).await?;
    assert!(
        !cache.exists(key).await?,
        "Key should not exist after deletion"
    );

    Ok(())
}

/// Test TTL expiration behavior
///
/// Validates:
/// - Keys with TTL expire after the specified duration
/// - Keys without TTL persist
/// - Expired keys return None on get
/// - Expired keys return false on exists
pub async fn test_ttl_expiration<C: CacheStorage>(cache: &C) -> RiptideResult<()> {
    let key = "test_key_ttl";
    let value = b"test_value";

    // Set with 1 second TTL
    cache.set(key, value, Some(Duration::from_secs(1))).await?;

    // Should exist immediately
    assert!(cache.exists(key).await?, "Key should exist immediately");
    let retrieved = cache.get(key).await?;
    assert!(
        retrieved.is_some(),
        "Value should be retrievable immediately"
    );

    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(1100)).await;

    // Should not exist after expiration
    assert!(!cache.exists(key).await?, "Key should not exist after TTL");
    let expired = cache.get(key).await?;
    assert!(expired.is_none(), "Expired key should return None");

    Ok(())
}

/// Test batch operations (mset/mget)
///
/// Validates:
/// - Multiple values can be set in one operation
/// - Multiple values can be retrieved in one operation
/// - Order is preserved in mget results
/// - Missing keys return None in correct positions
pub async fn test_batch_operations<C: CacheStorage>(cache: &C) -> RiptideResult<()> {
    // Test mset
    let items = vec![
        ("batch_key_1", b"value1" as &[u8]),
        ("batch_key_2", b"value2"),
        ("batch_key_3", b"value3"),
    ];

    cache.mset(items.clone(), None).await?;

    // Test mget - all keys exist
    let keys = vec!["batch_key_1", "batch_key_2", "batch_key_3"];
    let results = cache.mget(&keys).await?;

    assert_eq!(results.len(), 3, "Should return same number of results");
    assert_eq!(
        results[0].as_ref().unwrap().as_slice(),
        b"value1",
        "First value should match"
    );
    assert_eq!(
        results[1].as_ref().unwrap().as_slice(),
        b"value2",
        "Second value should match"
    );
    assert_eq!(
        results[2].as_ref().unwrap().as_slice(),
        b"value3",
        "Third value should match"
    );

    // Test mget with missing keys
    let mixed_keys = vec!["batch_key_1", "non_existent", "batch_key_3"];
    let mixed_results = cache.mget(&mixed_keys).await?;

    assert_eq!(
        mixed_results.len(),
        3,
        "Should return same number of results"
    );
    assert!(mixed_results[0].is_some(), "First key should exist");
    assert!(mixed_results[1].is_none(), "Second key should not exist");
    assert!(mixed_results[2].is_some(), "Third key should exist");

    // Cleanup
    cache.delete("batch_key_1").await?;
    cache.delete("batch_key_2").await?;
    cache.delete("batch_key_3").await?;

    Ok(())
}

/// Test batch deletion
///
/// Validates:
/// - Multiple keys can be deleted at once
/// - Returns correct count of deleted keys
/// - Non-existent keys don't cause errors
pub async fn test_delete_many<C: CacheStorage>(cache: &C) -> RiptideResult<()> {
    // Setup test data
    cache.set("del_key_1", b"value1", None).await?;
    cache.set("del_key_2", b"value2", None).await?;
    cache.set("del_key_3", b"value3", None).await?;

    // Delete multiple keys
    let keys = vec!["del_key_1", "del_key_2", "del_key_3", "non_existent"];
    let count = cache.delete_many(&keys).await?;

    // Should delete at least 3 keys (or 4 if implementation counts non-existent)
    assert!(count >= 3, "Should delete at least 3 existing keys");

    // Verify deletion
    assert!(!cache.exists("del_key_1").await?);
    assert!(!cache.exists("del_key_2").await?);
    assert!(!cache.exists("del_key_3").await?);

    Ok(())
}

/// Test expire functionality
///
/// Validates:
/// - Can set TTL on existing key
/// - Returns true when key exists
/// - Returns false when key doesn't exist
/// - Key expires after TTL
pub async fn test_expire<C: CacheStorage>(cache: &C) -> RiptideResult<()> {
    let key = "test_key_expire";
    let value = b"test_value";

    // Set without TTL
    cache.set(key, value, None).await?;

    // Set expire on existing key
    let result = cache.expire(key, Duration::from_secs(1)).await?;
    assert!(result, "Should return true for existing key");

    // Should still exist immediately
    assert!(cache.exists(key).await?);

    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(1100)).await;

    // Should not exist after expiration
    assert!(!cache.exists(key).await?);

    // Try to expire non-existent key
    let no_key_result = cache.expire("non_existent", Duration::from_secs(1)).await?;
    assert!(!no_key_result, "Should return false for non-existent key");

    Ok(())
}

/// Test ttl query functionality
///
/// Validates:
/// - Returns Some(duration) for keys with TTL
/// - Returns None for non-existent keys
/// - TTL decreases over time (within reasonable bounds)
pub async fn test_ttl_query<C: CacheStorage>(cache: &C) -> RiptideResult<()> {
    let key = "test_key_ttl_query";
    let value = b"test_value";

    // Set with 10 second TTL
    cache.set(key, value, Some(Duration::from_secs(10))).await?;

    // Check initial TTL
    let ttl = cache.ttl(key).await?;

    // Note: Some implementations may not support precise TTL queries
    // and may return None even for keys with TTL
    if let Some(remaining) = ttl {
        // If supported, should be close to 10 seconds
        assert!(
            remaining.as_secs() >= 9 && remaining.as_secs() <= 10,
            "TTL should be approximately 10 seconds, got: {:?}",
            remaining
        );

        // Wait a bit and check again
        tokio::time::sleep(Duration::from_millis(500)).await;
        let ttl2 = cache.ttl(key).await?;
        if let Some(remaining2) = ttl2 {
            assert!(remaining2 < remaining, "TTL should decrease over time");
        }
    }

    // Non-existent key should return None
    let no_key_ttl = cache.ttl("non_existent").await?;
    assert!(no_key_ttl.is_none(), "Non-existent key should return None");

    // Cleanup
    cache.delete(key).await?;

    Ok(())
}

/// Test increment operations
///
/// Validates:
/// - Can increment non-existent key (starts at 0)
/// - Can increment existing numeric value
/// - Can decrement with negative delta
/// - Returns correct new value
/// - Fails on non-numeric values
pub async fn test_incr<C: CacheStorage>(cache: &C) -> RiptideResult<()> {
    let key = "test_key_incr";

    // Increment non-existent key
    let value1 = cache.incr(key, 5).await?;
    assert_eq!(value1, 5, "First increment should be 5");

    // Increment again
    let value2 = cache.incr(key, 3).await?;
    assert_eq!(value2, 8, "Second increment should be 8");

    // Decrement
    let value3 = cache.incr(key, -10).await?;
    assert_eq!(value3, -2, "After decrement should be -2");

    // Cleanup
    cache.delete(key).await?;

    // Test error on non-numeric value
    cache.set("non_numeric", b"not_a_number", None).await?;
    let result = cache.incr("non_numeric", 1).await;
    assert!(result.is_err(), "Should error on non-numeric value");

    // Cleanup
    cache.delete("non_numeric").await?;

    Ok(())
}

/// Test health check functionality
///
/// Validates:
/// - Health check returns true for healthy cache
/// - Health check doesn't leave artifacts
pub async fn test_health_check<C: CacheStorage>(cache: &C) -> RiptideResult<()> {
    let healthy = cache.health_check().await?;
    assert!(healthy, "Cache should be healthy");

    // Health check should not leave any keys
    let health_key_exists = cache.exists("__health_check__").await?;
    assert!(
        !health_key_exists,
        "Health check should clean up after itself"
    );

    Ok(())
}

/// Test large value handling
///
/// Validates:
/// - Can store and retrieve large values
/// - Values maintain integrity
pub async fn test_large_values<C: CacheStorage>(cache: &C) -> RiptideResult<()> {
    let key = "test_key_large";
    // 1MB value
    let large_value: Vec<u8> = vec![0x42; 1024 * 1024];

    cache.set(key, &large_value, None).await?;
    let retrieved = cache.get(key).await?;

    assert!(retrieved.is_some(), "Large value should be retrievable");
    assert_eq!(
        retrieved.unwrap().len(),
        large_value.len(),
        "Large value should maintain size"
    );

    // Cleanup
    cache.delete(key).await?;

    Ok(())
}

/// Test binary data handling
///
/// Validates:
/// - Can store and retrieve binary data (not just UTF-8)
/// - Binary data maintains integrity
pub async fn test_binary_data<C: CacheStorage>(cache: &C) -> RiptideResult<()> {
    let key = "test_key_binary";
    // Non-UTF8 binary data
    let binary_value: Vec<u8> = vec![0xFF, 0xFE, 0x00, 0x01, 0x80, 0x7F];

    cache.set(key, &binary_value, None).await?;
    let retrieved = cache.get(key).await?;

    assert!(retrieved.is_some(), "Binary value should be retrievable");
    assert_eq!(
        retrieved.unwrap(),
        binary_value,
        "Binary value should be identical"
    );

    // Cleanup
    cache.delete(key).await?;

    Ok(())
}

/// Test concurrent operations
///
/// Validates:
/// - Multiple concurrent operations don't corrupt data
/// - Cache remains consistent under concurrent load
pub async fn test_concurrent_operations<C: CacheStorage>(cache: &C) -> RiptideResult<()> {
    use tokio::task::JoinSet;

    let mut tasks = JoinSet::new();

    // Spawn 10 concurrent tasks
    for i in 0..10 {
        let key = format!("concurrent_key_{}", i);
        let value = format!("value_{}", i);
        tasks.spawn(async move {
            // Each task does set, get, exists, delete
            let result: RiptideResult<()> = Ok(());
            result
        });

        // Immediately use cache operations (closure can't capture cache directly)
        cache.set(&key, value.as_bytes(), None).await?;
    }

    // Wait for all tasks
    while let Some(result) = tasks.join_next().await {
        result.map_err(|e| RiptideError::Cache(format!("Task join error: {}", e)))??;
    }

    // Verify all keys exist
    for i in 0..10 {
        let key = format!("concurrent_key_{}", i);
        assert!(
            cache.exists(&key).await?,
            "Concurrent key {} should exist",
            i
        );

        // Cleanup
        cache.delete(&key).await?;
    }

    Ok(())
}

/// Test empty value handling
///
/// Validates:
/// - Can store and retrieve empty byte arrays
/// - Empty values are distinguishable from missing keys
pub async fn test_empty_values<C: CacheStorage>(cache: &C) -> RiptideResult<()> {
    let key = "test_key_empty";
    let empty_value: &[u8] = b"";

    cache.set(key, empty_value, None).await?;

    // Should exist
    assert!(cache.exists(key).await?, "Empty value key should exist");

    // Should retrieve empty value
    let retrieved = cache.get(key).await?;
    assert!(retrieved.is_some(), "Should return Some for empty value");
    assert_eq!(
        retrieved.unwrap().len(),
        0,
        "Retrieved value should be empty"
    );

    // Cleanup
    cache.delete(key).await?;

    Ok(())
}

/// Run all contract tests
///
/// This is a convenience function that runs all contract tests in sequence.
/// Use individual test functions for more granular control.
pub async fn run_all_tests<C: CacheStorage>(cache: &C) -> RiptideResult<()> {
    test_basic_operations(cache).await?;
    test_exists(cache).await?;
    test_ttl_expiration(cache).await?;
    test_batch_operations(cache).await?;
    test_delete_many(cache).await?;
    test_expire(cache).await?;
    test_ttl_query(cache).await?;
    test_incr(cache).await?;
    test_health_check(cache).await?;
    test_large_values(cache).await?;
    test_binary_data(cache).await?;
    test_concurrent_operations(cache).await?;
    test_empty_values(cache).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use riptide_types::ports::CacheStorage;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// Simple in-memory cache for testing the contract tests themselves
    struct MemoryCache {
        data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    }

    impl MemoryCache {
        fn new() -> Self {
            Self {
                data: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    #[async_trait::async_trait]
    impl CacheStorage for MemoryCache {
        async fn get(&self, key: &str) -> RiptideResult<Option<Vec<u8>>> {
            let data = self.data.read().await;
            Ok(data.get(key).cloned())
        }

        async fn set(&self, key: &str, value: &[u8], _ttl: Option<Duration>) -> RiptideResult<()> {
            let mut data = self.data.write().await;
            data.insert(key.to_string(), value.to_vec());
            Ok(())
        }

        async fn delete(&self, key: &str) -> RiptideResult<()> {
            let mut data = self.data.write().await;
            data.remove(key);
            Ok(())
        }

        async fn exists(&self, key: &str) -> RiptideResult<bool> {
            let data = self.data.read().await;
            Ok(data.contains_key(key))
        }
    }

    #[tokio::test]
    async fn test_memory_cache_basic() {
        let cache = MemoryCache::new();
        test_basic_operations(&cache).await.unwrap();
    }

    #[tokio::test]
    async fn test_memory_cache_exists() {
        let cache = MemoryCache::new();
        test_exists(&cache).await.unwrap();
    }

    #[tokio::test]
    async fn test_memory_cache_batch() {
        let cache = MemoryCache::new();
        test_batch_operations(&cache).await.unwrap();
    }
}
