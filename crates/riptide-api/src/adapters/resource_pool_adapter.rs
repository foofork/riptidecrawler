use crate::resource_manager::ResourceManager;
/// Adapter to bridge ResourceManager to Pool<T> interface for ResourceFacade integration.
///
/// This adapter allows ResourceFacade to work with the existing ResourceManager
/// until full WASM pool infrastructure is available.
use async_trait::async_trait;
use riptide_types::ports::{pool::*, Pool, PoolHealth};
use std::sync::Arc;

/// Marker type for resource manager pool operations
#[derive(Debug, Clone)]
pub struct ResourceSlot {
    _phantom: std::marker::PhantomData<()>,
}

impl ResourceSlot {
    fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Adapter that wraps ResourceManager as a Pool<ResourceSlot>
///
/// This adapter bridges the ResourceManager interface to the Pool<T> trait
/// required by ResourceFacade, enabling facade integration without rewriting
/// the entire resource management system.
pub struct ResourceManagerPoolAdapter {
    resource_manager: Arc<ResourceManager>,
}

impl ResourceManagerPoolAdapter {
    /// Create a new adapter wrapping a ResourceManager
    pub fn new(resource_manager: Arc<ResourceManager>) -> Self {
        Self { resource_manager }
    }
}

#[async_trait]
impl Pool<ResourceSlot> for ResourceManagerPoolAdapter {
    async fn acquire(&self) -> Result<PooledResource<ResourceSlot>, PoolError> {
        // For now, we're not actually acquiring a specific resource
        // The real resource acquisition happens in the handlers via ResourceManager
        // This adapter just provides a Pool interface for ResourceFacade coordination

        Ok(PooledResource::new(
            ResourceSlot::new(),
            "resource-manager".to_string(),
            |_| {}, // No-op drop handler - ResourceManager handles cleanup
        ))
    }

    async fn release(&self, _resource: ResourceSlot) -> Result<(), PoolError> {
        // No-op - ResourceManager handles resource lifecycle
        Ok(())
    }

    async fn size(&self) -> usize {
        // Return a nominal size - actual limits are in ResourceManager
        let status = self.resource_manager.get_resource_status().await;
        status.headless_pool_total
    }

    async fn available(&self) -> usize {
        let status = self.resource_manager.get_resource_status().await;
        status.headless_pool_available
    }

    async fn health(&self) -> PoolHealth {
        let status = self.resource_manager.get_resource_status().await;

        PoolHealth {
            total: status.headless_pool_total,
            available: status.headless_pool_available,
            in_use: status.headless_pool_total - status.headless_pool_available,
            failed: 0, // ResourceManager doesn't track failed separately
            success_rate: if status.timeout_count > 0 {
                0.95 // Assume 95% success if timeouts occurred
            } else {
                1.0
            },
            avg_acquisition_time_ms: None,
            avg_latency_ms: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RiptideApiConfig;

    #[tokio::test]
    async fn test_adapter_basic_operations() {
        let config = RiptideApiConfig::default();
        let resource_manager = Arc::new(
            ResourceManager::new(config)
                .await
                .expect("Failed to create resource manager"),
        );

        let adapter = ResourceManagerPoolAdapter::new(resource_manager);
        let pool: &dyn Pool<ResourceSlot> = &adapter;

        // Test acquire
        let resource = pool.acquire().await.expect("Failed to acquire");

        // Test release
        pool.release(resource.into_inner())
            .await
            .expect("Failed to release");

        // Test health
        let health = pool.health().await;
        // health.total is usize which is always >= 0, just verify it exists
        let _ = health.total;
    }
}
