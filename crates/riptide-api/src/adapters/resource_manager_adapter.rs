//! ResourceManager adapter for hexagonal architecture trait abstraction
//!
//! This adapter wraps the concrete ResourceManager implementation to satisfy
//! the ResourceManagement port trait defined in riptide-types.
//!
//! # Purpose
//!
//! Enables dependency inversion by allowing ApplicationContext to depend on
//! Arc<dyn ResourceManagement> instead of the concrete Arc<ResourceManager>.
//!
//! # Architecture
//!
//! ```text
//! ApplicationContext (riptide-api)
//!     ↓ depends on trait
//! ResourceManagement trait (riptide-types/ports)
//!     ↑ implemented by
//! ResourceManagerAdapter (this file)
//!     ↓ wraps
//! ResourceManager (riptide-api/resource_manager)
//! ```

use crate::resource_manager::ResourceManager as ConcreteResourceManager;
use async_trait::async_trait;
use riptide_types::error::Result as RiptideResult;
use riptide_types::ports::resource::{ResourceManager, ResourceStatus};
use std::sync::Arc;

/// Adapter that bridges concrete ResourceManager to ResourceManager port trait
///
/// This adapter implements the dependency inversion principle by wrapping
/// the concrete ResourceManager in a trait-based interface, enabling:
/// - Testability via mock implementations
/// - Swappable resource management backends
/// - Clean hexagonal architecture boundaries
///
/// # Example
///
/// ```rust,ignore
/// use riptide_api::resource_manager::ResourceManager;
/// use riptide_api::adapters::ResourceManagerAdapter;
///
/// let concrete_manager = ResourceManager::new(config).await?;
/// let adapter: Arc<dyn ResourceManager> = ResourceManagerAdapter::new(Arc::new(concrete_manager));
/// ```
pub struct ResourceManagerAdapter {
    inner: Arc<ConcreteResourceManager>,
}

impl ResourceManagerAdapter {
    /// Create new adapter wrapping a concrete ResourceManager
    ///
    /// # Arguments
    ///
    /// * `manager` - Arc-wrapped concrete ResourceManager implementation
    ///
    /// # Returns
    ///
    /// An adapter that implements the ResourceManager port trait
    pub fn new(manager: Arc<ConcreteResourceManager>) -> Self {
        Self { inner: manager }
    }

    /// Create adapter as Arc for use as trait object
    ///
    /// # Arguments
    ///
    /// * `manager` - Arc-wrapped concrete ResourceManager implementation
    ///
    /// # Returns
    ///
    /// An Arc-wrapped adapter ready to use as Arc<dyn ResourceManager>
    pub fn new_arc(manager: Arc<ConcreteResourceManager>) -> Arc<Self> {
        Arc::new(Self { inner: manager })
    }
}

#[async_trait]
impl ResourceManager for ResourceManagerAdapter {
    /// Get current resource status
    ///
    /// Delegates to the inner ResourceManager and converts the concrete
    /// ResourceStatus type to the port trait's ResourceStatus type.
    async fn get_resource_status(&self) -> ResourceStatus {
        let concrete_status = self.inner.get_resource_status().await;

        // Map concrete ResourceStatus to port trait ResourceStatus
        let mut active_resources = std::collections::HashMap::new();
        active_resources.insert(
            "headless_pool".to_string(),
            concrete_status.headless_pool_total - concrete_status.headless_pool_available,
        );
        active_resources.insert(
            "pdf_processing".to_string(),
            concrete_status.pdf_total - concrete_status.pdf_available,
        );

        ResourceStatus {
            memory_usage: concrete_status.memory_usage_mb * 1024 * 1024, // MB to bytes
            memory_pressure: concrete_status.memory_pressure,
            degradation_score: concrete_status.degradation_score,
            active_resources,
        }
    }

    /// Request resource allocation
    ///
    /// Currently delegates to the inner ResourceManager's allocation logic.
    /// In future iterations, this could track allocations more explicitly.
    ///
    /// # Arguments
    ///
    /// * `resource_type` - Type of resource to allocate ("headless", "pdf", etc.)
    /// * `amount` - Amount of resources to allocate
    async fn allocate(&self, resource_type: &str, _amount: usize) -> RiptideResult<()> {
        // Check if resources are available
        let status = self.inner.get_resource_status().await;

        if status.memory_pressure {
            return Err(riptide_types::error::RiptideError::Custom(
                "Memory pressure detected - resource allocation denied".to_string(),
            ));
        }

        // Validate resource type is supported
        match resource_type {
            "headless" | "pdf" | "wasm" | "memory" => {
                // Resource manager tracks allocations internally via guards
                // This is a validation point for future explicit allocation tracking
                Ok(())
            }
            _ => Err(riptide_types::error::RiptideError::ValidationError(format!(
                "Unsupported resource type: {}",
                resource_type
            ))),
        }
    }

    /// Release allocated resources
    ///
    /// Currently a no-op as ResourceManager uses RAII guards for automatic cleanup.
    /// Future iterations may implement explicit resource tracking here.
    ///
    /// # Arguments
    ///
    /// * `resource_type` - Type of resource to release
    /// * `amount` - Amount of resources to release
    async fn release(&self, resource_type: &str, _amount: usize) -> RiptideResult<()> {
        // Validate resource type
        match resource_type {
            "headless" | "pdf" | "wasm" | "memory" => {
                // ResourceManager uses RAII guards (RenderResourceGuard, PdfResourceGuard)
                // that automatically release resources on drop
                Ok(())
            }
            _ => Err(riptide_types::error::RiptideError::ValidationError(format!(
                "Unsupported resource type: {}",
                resource_type
            ))),
        }
    }

    /// Check if resource manager is healthy
    ///
    /// Analyzes current status to determine health:
    /// - Memory pressure indicates unhealthy state
    /// - High degradation score (>0.7) indicates unhealthy state
    /// - Otherwise healthy
    async fn is_healthy(&self) -> bool {
        let status = self.inner.get_resource_status().await;

        // Healthy if:
        // 1. No memory pressure
        // 2. Degradation score below critical threshold (0.7)
        !status.memory_pressure && status.degradation_score < 0.7
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RiptideApiConfig;

    #[tokio::test]
    #[ignore = "Requires Chrome/Chromium to be installed - BrowserPool dependency"]
    async fn test_resource_manager_adapter_creation() {
        let config = RiptideApiConfig::default();
        let manager = ConcreteResourceManager::new(config).await.unwrap();
        let adapter = ResourceManagerAdapter::new(Arc::new(manager));

        // Verify we can use the trait interface
        let _trait_ref: &dyn ResourceManager = &adapter;
    }

    #[tokio::test]
    #[ignore = "Requires Chrome/Chromium to be installed - BrowserPool dependency"]
    async fn test_get_resource_status() {
        let config = RiptideApiConfig::default();
        let manager = ConcreteResourceManager::new(config).await.unwrap();
        let adapter = ResourceManagerAdapter::new(Arc::new(manager));

        let status = adapter.get_resource_status().await;

        // Verify status fields
        assert!(status.memory_usage >= 0);
        assert!(status.degradation_score >= 0.0);
        assert!(status.degradation_score <= 1.0);
        assert!(status.active_resources.contains_key("headless_pool"));
        assert!(status.active_resources.contains_key("pdf_processing"));
    }

    #[tokio::test]
    #[ignore = "Requires Chrome/Chromium to be installed - BrowserPool dependency"]
    async fn test_allocate_valid_resource_types() {
        let config = RiptideApiConfig::default();
        let manager = ConcreteResourceManager::new(config).await.unwrap();
        let adapter = ResourceManagerAdapter::new(Arc::new(manager));

        // Valid resource types should succeed
        assert!(adapter.allocate("headless", 1).await.is_ok());
        assert!(adapter.allocate("pdf", 1).await.is_ok());
        assert!(adapter.allocate("wasm", 1).await.is_ok());
        assert!(adapter.allocate("memory", 1).await.is_ok());
    }

    #[tokio::test]
    #[ignore = "Requires Chrome/Chromium to be installed - BrowserPool dependency"]
    async fn test_allocate_invalid_resource_type() {
        let config = RiptideApiConfig::default();
        let manager = ConcreteResourceManager::new(config).await.unwrap();
        let adapter = ResourceManagerAdapter::new(Arc::new(manager));

        // Invalid resource type should fail
        let result = adapter.allocate("invalid_type", 1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore = "Requires Chrome/Chromium to be installed - BrowserPool dependency"]
    async fn test_is_healthy() {
        let config = RiptideApiConfig::default();
        let manager = ConcreteResourceManager::new(config).await.unwrap();
        let adapter = ResourceManagerAdapter::new(Arc::new(manager));

        // Should be healthy with default config
        let healthy = adapter.is_healthy().await;
        assert!(healthy);
    }

    #[tokio::test]
    #[ignore = "Requires Chrome/Chromium to be installed - BrowserPool dependency"]
    async fn test_release_resources() {
        let config = RiptideApiConfig::default();
        let manager = ConcreteResourceManager::new(config).await.unwrap();
        let adapter = ResourceManagerAdapter::new(Arc::new(manager));

        // Release should succeed for valid types (even though it's a no-op)
        assert!(adapter.release("headless", 1).await.is_ok());
        assert!(adapter.release("pdf", 1).await.is_ok());
        assert!(adapter.release("wasm", 1).await.is_ok());
        assert!(adapter.release("memory", 1).await.is_ok());

        // Invalid type should fail
        assert!(adapter.release("invalid_type", 1).await.is_err());
    }
}
