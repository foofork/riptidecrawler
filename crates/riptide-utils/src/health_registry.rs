//! In-memory health check registry implementation
//!
//! This module provides a simple, thread-safe health check registry
//! for managing multiple health checks.

use async_trait::async_trait;
use riptide_types::error::Result;
use riptide_types::ports::health::{HealthCheck, HealthRegistry, HealthStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory implementation of health check registry
pub struct InMemoryHealthRegistry {
    checks: Arc<RwLock<HashMap<String, Arc<dyn HealthCheck>>>>,
}

impl InMemoryHealthRegistry {
    /// Creates a new health registry
    pub fn new() -> Self {
        Self {
            checks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Returns the number of registered checks
    pub async fn len(&self) -> usize {
        self.checks.read().await.len()
    }

    /// Checks if the registry is empty
    pub async fn is_empty(&self) -> bool {
        self.checks.read().await.is_empty()
    }

    /// Gets a list of all registered check names
    pub async fn list_checks(&self) -> Vec<String> {
        self.checks.read().await.keys().cloned().collect()
    }
}

impl Default for InMemoryHealthRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HealthRegistry for InMemoryHealthRegistry {
    async fn register(&mut self, check: Arc<dyn HealthCheck>) {
        let name = check.name().to_string();
        self.checks.write().await.insert(name, check);
    }

    async fn unregister(&mut self, name: &str) -> bool {
        self.checks.write().await.remove(name).is_some()
    }

    async fn check_all(&self) -> HashMap<String, HealthStatus> {
        let checks = self.checks.read().await;
        let mut results = HashMap::new();

        for (name, check) in checks.iter() {
            let status = check
                .check()
                .await
                .unwrap_or_else(|e| HealthStatus::Unhealthy {
                    error: format!("Health check failed: {}", e),
                });
            results.insert(name.clone(), status);
        }

        results
    }
}

/// Simple health check implementation for testing
pub struct SimpleHealthCheck {
    name: String,
    description: String,
    check_fn: Arc<dyn Fn() -> HealthStatus + Send + Sync>,
}

impl SimpleHealthCheck {
    /// Creates a new simple health check
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        check_fn: impl Fn() -> HealthStatus + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            check_fn: Arc::new(check_fn),
        }
    }

    /// Creates a health check that always returns healthy
    pub fn always_healthy(name: impl Into<String>) -> Self {
        Self::new(name, "Always healthy", || HealthStatus::Healthy)
    }

    /// Creates a health check that always returns unhealthy
    pub fn always_unhealthy(name: impl Into<String>, error: String) -> Self {
        Self::new(name, "Always unhealthy", move || HealthStatus::Unhealthy {
            error: error.clone(),
        })
    }
}

#[async_trait]
impl HealthCheck for SimpleHealthCheck {
    async fn check(&self) -> Result<HealthStatus> {
        Ok((self.check_fn)())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        Some(&self.description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = InMemoryHealthRegistry::new();
        assert_eq!(registry.len().await, 0);
        assert!(registry.is_empty().await);
    }

    #[tokio::test]
    async fn test_register_health_check() {
        let mut registry = InMemoryHealthRegistry::new();
        let check = Arc::new(SimpleHealthCheck::always_healthy("test"));

        registry.register(check).await;

        assert_eq!(registry.len().await, 1);
        assert!(!registry.is_empty().await);
    }

    #[tokio::test]
    async fn test_unregister_health_check() {
        let mut registry = InMemoryHealthRegistry::new();
        let check = Arc::new(SimpleHealthCheck::always_healthy("test"));

        registry.register(check).await;
        assert_eq!(registry.len().await, 1);

        let removed = registry.unregister("test").await;
        assert!(removed);
        assert_eq!(registry.len().await, 0);
    }

    #[tokio::test]
    async fn test_check_all_healthy() {
        let mut registry = InMemoryHealthRegistry::new();
        registry
            .register(Arc::new(SimpleHealthCheck::always_healthy("check1")))
            .await;
        registry
            .register(Arc::new(SimpleHealthCheck::always_healthy("check2")))
            .await;

        let results = registry.check_all().await;

        assert_eq!(results.len(), 2);
        assert!(results.get("check1").unwrap().is_healthy());
        assert!(results.get("check2").unwrap().is_healthy());
    }

    #[tokio::test]
    async fn test_check_all_unhealthy() {
        let mut registry = InMemoryHealthRegistry::new();
        registry
            .register(Arc::new(SimpleHealthCheck::always_unhealthy(
                "check1",
                "Test error".to_string(),
            )))
            .await;

        let results = registry.check_all().await;

        assert_eq!(results.len(), 1);
        assert!(results.get("check1").unwrap().is_unhealthy());
    }

    #[tokio::test]
    async fn test_is_healthy() {
        let mut registry = InMemoryHealthRegistry::new();
        registry
            .register(Arc::new(SimpleHealthCheck::always_healthy("check1")))
            .await;
        registry
            .register(Arc::new(SimpleHealthCheck::always_healthy("check2")))
            .await;

        assert!(registry.is_healthy().await);
    }

    #[tokio::test]
    async fn test_overall_status_healthy() {
        let mut registry = InMemoryHealthRegistry::new();
        registry
            .register(Arc::new(SimpleHealthCheck::always_healthy("check1")))
            .await;

        let status = registry.overall_status().await;
        assert!(status.is_healthy());
    }

    #[tokio::test]
    async fn test_overall_status_unhealthy() {
        let mut registry = InMemoryHealthRegistry::new();
        registry
            .register(Arc::new(SimpleHealthCheck::always_unhealthy(
                "check1",
                "Failed".to_string(),
            )))
            .await;

        let status = registry.overall_status().await;
        assert!(status.is_unhealthy());
    }

    #[tokio::test]
    async fn test_list_checks() {
        let mut registry = InMemoryHealthRegistry::new();
        registry
            .register(Arc::new(SimpleHealthCheck::always_healthy("check1")))
            .await;
        registry
            .register(Arc::new(SimpleHealthCheck::always_healthy("check2")))
            .await;

        let checks = registry.list_checks().await;
        assert_eq!(checks.len(), 2);
        assert!(checks.contains(&"check1".to_string()));
        assert!(checks.contains(&"check2".to_string()));
    }
}
