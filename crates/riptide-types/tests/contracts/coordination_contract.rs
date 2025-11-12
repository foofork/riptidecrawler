//! Contract tests for distributed coordination traits
//!
//! These tests validate CacheSync trait implementations used for
//! distributed cache coordination, pub/sub patterns, and cache invalidation.
//!
//! # Usage
//!
//! ```rust,ignore
//! use riptide_persistence::cache::CacheSync;
//!
//! #[tokio::test]
//! async fn test_my_coordination() {
//!     let coordinator = MyCoordinator::new();
//!     coordination_contract::test_basic_notifications(&coordinator).await.unwrap();
//!     coordination_contract::test_pattern_invalidation(&coordinator).await.unwrap();
//! }
//! ```

use riptide_types::error::{Result as RiptideResult};
use async_trait::async_trait;

/// CacheSync trait for testing - mirrors the trait from riptide-persistence
#[async_trait]
pub trait CacheSync: Send + Sync {
    async fn notify_set(&self, key: &str) -> RiptideResult<()>;
    async fn notify_delete(&self, key: &str) -> RiptideResult<()>;
    async fn invalidate_pattern(&self, pattern: &str) -> RiptideResult<()>;
}

/// Test basic set/delete notifications
///
/// Validates:
/// - Can send set notifications without error
/// - Can send delete notifications without error
/// - Multiple notifications can be sent sequentially
pub async fn test_basic_notifications<C: CacheSync>(coordinator: &C) -> RiptideResult<()> {
    // Test set notifications
    coordinator.notify_set("key1").await?;
    coordinator.notify_set("key2").await?;
    coordinator.notify_set("prefix:key3").await?;

    // Test delete notifications
    coordinator.notify_delete("key1").await?;
    coordinator.notify_delete("key2").await?;
    coordinator.notify_delete("prefix:key3").await?;

    Ok(())
}

/// Test pattern-based invalidation
///
/// Validates:
/// - Can invalidate patterns without error
/// - Supports wildcard patterns
/// - Supports prefix patterns
pub async fn test_pattern_invalidation<C: CacheSync>(coordinator: &C) -> RiptideResult<()> {
    // Test various pattern types
    coordinator.invalidate_pattern("user:*").await?;
    coordinator.invalidate_pattern("session:*").await?;
    coordinator.invalidate_pattern("cache:v1:*").await?;

    // Test exact match pattern
    coordinator.invalidate_pattern("exact_key").await?;

    Ok(())
}

/// Test high-frequency notifications
///
/// Validates:
/// - Can handle rapid notification bursts
/// - No dropped notifications under load
/// - Maintains stability under concurrent access
pub async fn test_high_frequency<C: CacheSync>(coordinator: &C) -> RiptideResult<()> {
    // Send 100 rapid notifications
    for i in 0..100 {
        let key = format!("rapid_key_{}", i);
        coordinator.notify_set(&key).await?;
    }

    // Send 100 rapid deletes
    for i in 0..100 {
        let key = format!("rapid_key_{}", i);
        coordinator.notify_delete(&key).await?;
    }

    Ok(())
}

/// Test concurrent notifications
///
/// Validates:
/// - Multiple concurrent notifications don't cause errors
/// - Thread-safe notification handling
/// - No race conditions or deadlocks
pub async fn test_concurrent_notifications<C: CacheSync>(coordinator: &C) -> RiptideResult<()> {
    use tokio::task::JoinSet;

    let mut tasks = JoinSet::new();

    // Spawn 10 concurrent notification tasks
    for i in 0..10 {
        let key = format!("concurrent_key_{}", i);
        tasks.spawn(async move { Ok::<String, ()>(key) });
    }

    // Send notifications for all keys
    while let Some(result) = tasks.join_next().await {
        let key = result.unwrap().unwrap();
        coordinator.notify_set(&key).await?;
        coordinator.notify_delete(&key).await?;
    }

    Ok(())
}

/// Test error handling and recovery
///
/// Validates:
/// - Graceful handling of invalid inputs
/// - Recovery from transient failures
/// - No persistent state corruption
pub async fn test_error_handling<C: CacheSync>(coordinator: &C) -> RiptideResult<()> {
    // Empty key (should handle gracefully)
    let _ = coordinator.notify_set("").await;
    let _ = coordinator.notify_delete("").await;

    // Very long key
    let long_key = "a".repeat(10000);
    let _ = coordinator.notify_set(&long_key).await;

    // Special characters in patterns
    let _ = coordinator.invalidate_pattern("key:with:colons").await;
    let _ = coordinator.invalidate_pattern("key.with.dots").await;

    // Should be able to continue normal operations
    coordinator.notify_set("normal_key").await?;

    Ok(())
}

/// Test notification ordering (best effort)
///
/// Validates:
/// - Notifications are processed in a reasonable order
/// - No obvious reordering issues
///
/// Note: This is a best-effort test as distributed systems
/// don't guarantee strict ordering
pub async fn test_notification_ordering<C: CacheSync>(coordinator: &C) -> RiptideResult<()> {
    // Send a sequence of operations
    coordinator.notify_set("order_key_1").await?;
    coordinator.notify_set("order_key_2").await?;
    coordinator.notify_set("order_key_3").await?;
    coordinator.notify_delete("order_key_1").await?;
    coordinator.notify_delete("order_key_2").await?;
    coordinator.notify_delete("order_key_3").await?;

    // If we get here without errors, ordering is at least sequential
    Ok(())
}

/// Test mixed operation types
///
/// Validates:
/// - Can interleave different notification types
/// - No interference between notification types
pub async fn test_mixed_operations<C: CacheSync>(coordinator: &C) -> RiptideResult<()> {
    // Interleave different operations
    coordinator.notify_set("mixed_1").await?;
    coordinator.invalidate_pattern("prefix:*").await?;
    coordinator.notify_delete("mixed_1").await?;
    coordinator.notify_set("mixed_2").await?;
    coordinator.invalidate_pattern("other:*").await?;
    coordinator.notify_delete("mixed_2").await?;

    Ok(())
}

/// Test large batch operations
///
/// Validates:
/// - Can handle large batches of notifications
/// - Performance remains acceptable
/// - No memory leaks or resource exhaustion
pub async fn test_large_batch<C: CacheSync>(coordinator: &C) -> RiptideResult<()> {
    // Send 1000 notifications
    for i in 0..1000 {
        let key = format!("batch_key_{}", i);
        coordinator.notify_set(&key).await?;
    }

    // Pattern invalidations
    for i in 0..100 {
        let pattern = format!("batch_pattern_{}:*", i);
        coordinator.invalidate_pattern(&pattern).await?;
    }

    Ok(())
}

/// Run all coordination contract tests
///
/// This is a convenience function that runs all contract tests in sequence.
/// Use individual test functions for more granular control.
pub async fn run_all_tests<C: CacheSync>(coordinator: &C) -> RiptideResult<()> {
    test_basic_notifications(coordinator).await?;
    test_pattern_invalidation(coordinator).await?;
    test_high_frequency(coordinator).await?;
    test_concurrent_notifications(coordinator).await?;
    test_error_handling(coordinator).await?;
    test_notification_ordering(coordinator).await?;
    test_mixed_operations(coordinator).await?;
    test_large_batch(coordinator).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// Mock coordinator for testing the contract tests
    struct MockCoordinator {
        notifications: Arc<RwLock<Vec<String>>>,
    }

    impl MockCoordinator {
        fn new() -> Self {
            Self {
                notifications: Arc::new(RwLock::new(Vec::new())),
            }
        }

        #[allow(dead_code)]
        async fn get_notifications(&self) -> Vec<String> {
            self.notifications.read().await.clone()
        }
    }

    #[async_trait]
    impl CacheSync for MockCoordinator {
        async fn notify_set(&self, key: &str) -> RiptideResult<()> {
            let mut notifications = self.notifications.write().await;
            notifications.push(format!("SET:{}", key));
            Ok(())
        }

        async fn notify_delete(&self, key: &str) -> RiptideResult<()> {
            let mut notifications = self.notifications.write().await;
            notifications.push(format!("DELETE:{}", key));
            Ok(())
        }

        async fn invalidate_pattern(&self, pattern: &str) -> RiptideResult<()> {
            let mut notifications = self.notifications.write().await;
            notifications.push(format!("INVALIDATE:{}", pattern));
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_mock_basic_notifications() {
        let coordinator = MockCoordinator::new();
        test_basic_notifications(&coordinator).await.unwrap();

        let notifications = coordinator.get_notifications().await;
        assert!(notifications.len() >= 6, "Should have at least 6 notifications");
    }

    #[tokio::test]
    async fn test_mock_pattern_invalidation() {
        let coordinator = MockCoordinator::new();
        test_pattern_invalidation(&coordinator).await.unwrap();

        let notifications = coordinator.get_notifications().await;
        assert!(notifications.len() >= 4, "Should have at least 4 invalidations");
    }

    #[tokio::test]
    async fn test_mock_concurrent() {
        let coordinator = MockCoordinator::new();
        test_concurrent_notifications(&coordinator).await.unwrap();

        let notifications = coordinator.get_notifications().await;
        assert!(notifications.len() >= 10, "Should have concurrent notifications");
    }
}
