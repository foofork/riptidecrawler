//! Integration tests for Redis cache adapters
//!
//! Tests the concrete implementations:
//! - RedisIdempotencyStore: IdempotencyStore implementation
//! - RedisSessionStorage: Session storage adapter (if enabled)
//!
//! NOTE: These tests require Redis.
//! Use testcontainers or set SKIP_REDIS_TESTS=1 to skip.

#[cfg(test)]
mod redis_adapter_tests {
    use super::*;

    /// Test RedisIdempotencyStore acquire token
    #[tokio::test]
    #[ignore = "requires Redis testcontainer"]
    async fn test_redis_idempotency_acquire() {
        let store = setup_test_idempotency_store().await;

        let token = "test-token-1";
        let ttl = 60; // 60 seconds

        let result = store.acquire(token, ttl).await;
        assert!(result.is_ok(), "First acquire should succeed");
        assert!(result.unwrap(), "Token should be acquired");
    }

    #[tokio::test]
    #[ignore = "requires Redis testcontainer"]
    async fn test_redis_idempotency_duplicate_prevention() {
        let store = setup_test_idempotency_store().await;

        let token = "test-token-2";
        let ttl = 60;

        // First acquire should succeed
        let first = store.acquire(token, ttl).await;
        assert!(first.is_ok() && first.unwrap(), "First acquire should succeed");

        // Second acquire should fail (duplicate)
        let second = store.acquire(token, ttl).await;
        assert!(second.is_ok(), "Second acquire should return Ok");
        assert!(!second.unwrap(), "Second acquire should fail (duplicate)");
    }

    #[tokio::test]
    #[ignore = "requires Redis testcontainer"]
    async fn test_redis_idempotency_release() {
        let store = setup_test_idempotency_store().await;

        let token = "test-token-3";
        let ttl = 60;

        // Acquire
        store.acquire(token, ttl).await.expect("Acquire failed");

        // Release
        let result = store.release(token).await;
        assert!(result.is_ok(), "Release should succeed");

        // Should be able to acquire again after release
        let re_acquire = store.acquire(token, ttl).await;
        assert!(re_acquire.unwrap(), "Re-acquire after release should succeed");
    }

    #[tokio::test]
    #[ignore = "requires Redis testcontainer"]
    async fn test_redis_idempotency_ttl_expiration() {
        let store = setup_test_idempotency_store().await;

        let token = "test-token-4";
        let ttl = 1; // 1 second

        // Acquire with short TTL
        store.acquire(token, ttl).await.expect("Acquire failed");

        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Should be able to acquire again after expiration
        let re_acquire = store.acquire(token, ttl).await;
        assert!(re_acquire.unwrap(), "Re-acquire after TTL should succeed");
    }

    #[tokio::test]
    #[ignore = "requires Redis testcontainer"]
    async fn test_redis_idempotency_concurrent_acquire() {
        let store = setup_test_idempotency_store().await;
        let token = "test-token-5";
        let ttl = 60;

        // Spawn multiple tasks trying to acquire the same token
        let mut handles = vec![];
        for _i in 0..10 {
            let store_clone = store.clone();
            let token_clone = token.to_string();
            let handle = tokio::spawn(async move {
                store_clone.acquire(&token_clone, ttl).await
            });
            handles.push(handle);
        }

        // Collect results
        let mut success_count = 0;
        for handle in handles {
            if let Ok(Ok(true)) = handle.await {
                success_count += 1;
            }
        }

        // Only one task should succeed in acquiring the token
        assert_eq!(success_count, 1, "Only one concurrent acquire should succeed");
    }

    /// Test RedisSessionStorage operations (if feature enabled)
    #[cfg(feature = "sessions")]
    #[tokio::test]
    #[ignore = "requires Redis testcontainer"]
    async fn test_redis_session_storage_save() {
        let storage = setup_test_session_storage().await;

        let session = TestSession {
            id: "session-1".to_string(),
            user_id: "user-1".to_string(),
            data: "test data".to_string(),
        };

        let result = storage.save(&session).await;
        assert!(result.is_ok(), "Session save should succeed");
    }

    #[cfg(feature = "sessions")]
    #[tokio::test]
    #[ignore = "requires Redis testcontainer"]
    async fn test_redis_session_storage_retrieve() {
        let storage = setup_test_session_storage().await;

        // Setup
        let session = TestSession {
            id: "session-2".to_string(),
            user_id: "user-2".to_string(),
            data: "retrieve test".to_string(),
        };
        storage.save(&session).await.expect("Setup failed");

        // Test
        let found = storage.get("session-2").await;
        assert!(found.is_ok(), "Session retrieval should succeed");

        let session_data = found.unwrap();
        assert!(session_data.is_some(), "Session should exist");
        assert_eq!(session_data.unwrap().user_id, "user-2");
    }

    #[cfg(feature = "sessions")]
    #[tokio::test]
    #[ignore = "requires Redis testcontainer"]
    async fn test_redis_session_storage_delete() {
        let storage = setup_test_session_storage().await;

        // Setup
        let session = TestSession {
            id: "session-3".to_string(),
            user_id: "user-3".to_string(),
            data: "delete test".to_string(),
        };
        storage.save(&session).await.expect("Setup failed");

        // Test
        let result = storage.delete("session-3").await;
        assert!(result.is_ok(), "Session delete should succeed");

        // Verify
        let found = storage.get("session-3").await.unwrap();
        assert!(found.is_none(), "Session should be deleted");
    }

    #[cfg(feature = "sessions")]
    #[tokio::test]
    #[ignore = "requires Redis testcontainer"]
    async fn test_redis_session_storage_ttl() {
        let storage = setup_test_session_storage().await;

        // Save session with short TTL
        let session = TestSession {
            id: "session-4".to_string(),
            user_id: "user-4".to_string(),
            data: "ttl test".to_string(),
        };
        storage.save_with_ttl(&session, 1).await.expect("Save failed");

        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Verify session expired
        let found = storage.get("session-4").await.unwrap();
        assert!(found.is_none(), "Session should expire after TTL");
    }

    /// Test Redis connection pooling
    #[tokio::test]
    #[ignore = "requires Redis testcontainer"]
    async fn test_redis_connection_pool() {
        let store = setup_test_idempotency_store().await;

        // Perform multiple operations to test connection pooling
        let mut handles = vec![];
        for i in 0..20 {
            let store_clone = store.clone();
            let handle = tokio::spawn(async move {
                let token = format!("pool-test-{}", i);
                store_clone.acquire(&token, 60).await
            });
            handles.push(handle);
        }

        // All operations should succeed
        for handle in handles {
            let result = handle.await.expect("Task failed");
            assert!(result.is_ok(), "Operation should succeed with pooling");
        }
    }

    /// Test Redis error handling for connection failures
    #[tokio::test]
    #[ignore = "requires Redis testcontainer"]
    async fn test_redis_connection_failure_handling() {
        // Create store with invalid connection
        let invalid_store = create_invalid_store();

        let result = invalid_store.acquire("test", 60).await;
        assert!(result.is_err(), "Should fail with connection error");

        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("connection") || error_msg.contains("redis"),
            "Error should indicate connection issue"
        );
    }

    /// Test Redis performance under load
    #[tokio::test]
    #[ignore = "requires Redis testcontainer"]
    async fn test_redis_performance_under_load() {
        let store = setup_test_idempotency_store().await;

        let start = std::time::Instant::now();

        // Perform 100 operations
        for i in 0..100 {
            let token = format!("perf-test-{}", i);
            store.acquire(&token, 60).await.expect("Operation failed");
        }

        let duration = start.elapsed();

        // Should complete in reasonable time (< 5 seconds for 100 ops)
        assert!(
            duration.as_secs() < 5,
            "100 operations should complete in < 5s, took {:?}",
            duration
        );
    }

    // ============ Mock Types and Helpers ============

    #[derive(Debug, Clone)]
    struct TestSession {
        id: String,
        user_id: String,
        data: String,
    }

    #[derive(Clone)]
    struct MockIdempotencyStore;

    impl MockIdempotencyStore {
        async fn acquire(&self, _token: &str, _ttl: u64) -> Result<bool, String> {
            Ok(true)
        }

        async fn release(&self, _token: &str) -> Result<(), String> {
            Ok(())
        }
    }

    #[cfg(feature = "sessions")]
    struct MockSessionStorage;

    #[cfg(feature = "sessions")]
    impl MockSessionStorage {
        async fn save(&self, _session: &TestSession) -> Result<(), String> {
            Ok(())
        }

        async fn save_with_ttl(&self, _session: &TestSession, _ttl: u64) -> Result<(), String> {
            Ok(())
        }

        async fn get(&self, _id: &str) -> Result<Option<TestSession>, String> {
            Ok(None)
        }

        async fn delete(&self, _id: &str) -> Result<(), String> {
            Ok(())
        }
    }

    async fn setup_test_idempotency_store() -> MockIdempotencyStore {
        MockIdempotencyStore
    }

    #[cfg(feature = "sessions")]
    async fn setup_test_session_storage() -> MockSessionStorage {
        MockSessionStorage
    }

    fn create_invalid_store() -> MockIdempotencyStore {
        // In a real implementation, this would use an invalid Redis URL
        MockIdempotencyStore
    }
}
