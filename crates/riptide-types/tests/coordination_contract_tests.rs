//! Integration tests for coordination_contract module
//!
//! These tests validate that the contract test suite itself works correctly
//! by running it against a simple mock implementation.

mod contracts;

use contracts::coordination_contract::{self, CacheSync};
use riptide_types::error::Result as RiptideResult;
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

    #[allow(dead_code)]
    async fn clear_notifications(&self) {
        self.notifications.write().await.clear();
    }
}

#[async_trait::async_trait]
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
    coordination_contract::test_basic_notifications(&coordinator)
        .await
        .unwrap();

    let notifications = coordinator.get_notifications().await;
    assert!(
        notifications.len() >= 6,
        "Should have at least 6 notifications"
    );
}

#[tokio::test]
async fn test_mock_pattern_invalidation() {
    let coordinator = MockCoordinator::new();
    coordination_contract::test_pattern_invalidation(&coordinator)
        .await
        .unwrap();

    let notifications = coordinator.get_notifications().await;
    assert!(
        notifications.len() >= 4,
        "Should have at least 4 invalidations"
    );
}

#[tokio::test]
async fn test_mock_high_frequency() {
    let coordinator = MockCoordinator::new();
    coordination_contract::test_high_frequency(&coordinator)
        .await
        .unwrap();

    let notifications = coordinator.get_notifications().await;
    assert!(
        notifications.len() >= 200,
        "Should have at least 200 rapid notifications"
    );
}

#[tokio::test]
async fn test_mock_concurrent() {
    let coordinator = MockCoordinator::new();
    coordination_contract::test_concurrent_notifications(&coordinator)
        .await
        .unwrap();

    let notifications = coordinator.get_notifications().await;
    assert!(
        notifications.len() >= 10,
        "Should have concurrent notifications"
    );
}

#[tokio::test]
async fn test_mock_error_handling() {
    let coordinator = MockCoordinator::new();
    coordination_contract::test_error_handling(&coordinator)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_mock_notification_ordering() {
    let coordinator = MockCoordinator::new();
    coordination_contract::test_notification_ordering(&coordinator)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_mock_mixed_operations() {
    let coordinator = MockCoordinator::new();
    coordination_contract::test_mixed_operations(&coordinator)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_mock_large_batch() {
    let coordinator = MockCoordinator::new();
    coordination_contract::test_large_batch(&coordinator)
        .await
        .unwrap();

    let notifications = coordinator.get_notifications().await;
    assert!(
        notifications.len() >= 1000,
        "Should have at least 1000 batch notifications"
    );
}

#[tokio::test]
async fn test_mock_all_contracts() {
    let coordinator = MockCoordinator::new();
    coordination_contract::run_all_tests(&coordinator)
        .await
        .unwrap();
}
