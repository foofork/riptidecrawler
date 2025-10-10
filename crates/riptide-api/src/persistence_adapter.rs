/*!
# Persistence Adapter for riptide-api Integration

This module provides a high-level adapter interface for integrating the
riptide-persistence layer into riptide-api. It wraps PersistentCacheManager,
TenantManager, and StateManager with convenient methods for API handlers.

## Architecture

The adapter follows the Facade pattern to simplify persistence operations:
- Unified error handling with ApiError conversion
- Tenant-aware cache operations
- Performance tracking and metrics
- Graceful degradation support

## Example Usage

```rust
use persistence_adapter::PersistenceAdapter;

let adapter = PersistenceAdapter::new(config).await?;

// Tenant-aware caching
let cached_data: Option<SearchResult> = adapter
    .get_cached("search_results", Some("tenant-123"))
    .await?;

adapter.set_cached(
    "search_results",
    &search_results,
    Duration::from_secs(3600),
    Some("tenant-123")
).await?;
```
*/

use anyhow::Result;
use riptide_persistence::{
    BillingPlan, CacheStats, Checkpoint, PersistenceConfig, PersistentCacheManager,
    StateManager, TenantConfig, TenantManager, TenantOwner, TenantSummary,
};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// High-level persistence adapter for riptide-api integration
///
/// This adapter provides a unified interface for:
/// - Multi-tenant cache operations
/// - State management (hot reload, checkpoints)
/// - Tenant provisioning and billing
/// - Performance monitoring
#[derive(Clone)]
pub struct PersistenceAdapter {
    /// Cache manager for high-performance caching
    cache_manager: Arc<PersistentCacheManager>,

    /// Tenant manager for multi-tenancy support
    tenant_manager: Arc<TenantManager>,

    /// State manager for configuration and checkpoints
    state_manager: Arc<StateManager>,

    /// Configuration for persistence layer
    config: PersistenceConfig,
}

impl PersistenceAdapter {
    /// Initialize a new persistence adapter with configuration
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection URL
    /// * `config` - Persistence configuration
    ///
    /// # Returns
    ///
    /// A configured persistence adapter ready for use
    ///
    /// # Errors
    ///
    /// Returns error if Redis connection fails or managers cannot initialize
    pub async fn new(redis_url: &str, config: PersistenceConfig) -> Result<Self> {
        tracing::info!("Initializing PersistenceAdapter with Redis: {}", redis_url);

        // Initialize cache manager
        let cache_manager = PersistentCacheManager::new(redis_url, config.cache.clone())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to initialize cache manager: {}", e))?;

        tracing::info!("PersistentCacheManager initialized successfully");

        // Initialize tenant manager
        let tenant_manager = TenantManager::new(
            redis_url,
            config.tenant.clone(),
            Arc::new(tokio::sync::RwLock::new(
                riptide_persistence::tenant::TenantMetrics::default(),
            )),
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize tenant manager: {}", e))?;

        tracing::info!("TenantManager initialized successfully");

        // Initialize state manager
        let state_manager = StateManager::new(redis_url, config.state.clone())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to initialize state manager: {}", e))?;

        tracing::info!("StateManager initialized successfully");

        Ok(Self {
            cache_manager: Arc::new(cache_manager),
            tenant_manager: Arc::new(tenant_manager),
            state_manager: Arc::new(state_manager),
            config,
        })
    }

    /// Get cached value with tenant isolation
    ///
    /// # Type Parameters
    ///
    /// * `T` - Type to deserialize cached value into
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    /// * `tenant_id` - Optional tenant ID for isolation (None = default tenant)
    ///
    /// # Returns
    ///
    /// Cached value if exists and valid, None otherwise
    pub async fn get_cached<T: DeserializeOwned>(
        &self,
        key: &str,
        tenant_id: Option<&str>,
    ) -> Result<Option<T>> {
        self.cache_manager
            .get(key, tenant_id)
            .await
            .map_err(|e| anyhow::anyhow!("Cache get failed: {}", e))
    }

    /// Set cached value with TTL and tenant isolation
    ///
    /// # Type Parameters
    ///
    /// * `T` - Type to serialize into cache
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    /// * `value` - Value to cache
    /// * `ttl` - Time-to-live duration
    /// * `tenant_id` - Optional tenant ID for isolation
    pub async fn set_cached<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
        tenant_id: Option<&str>,
    ) -> Result<()> {
        self.cache_manager
            .set(key, value, tenant_id, Some(ttl), None)
            .await
            .map_err(|e| anyhow::anyhow!("Cache set failed: {}", e))
    }

    /// Invalidate cached entry
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key to invalidate
    /// * `tenant_id` - Optional tenant ID for isolation
    pub async fn invalidate(&self, key: &str, tenant_id: Option<&str>) -> Result<()> {
        self.cache_manager
            .delete(key, tenant_id)
            .await
            .map_err(|e| anyhow::anyhow!("Cache invalidation failed: {}", e))
    }

    /// Warm cache with batch of keys
    ///
    /// # Arguments
    ///
    /// * `keys` - Vector of cache keys to warm
    ///
    /// # Returns
    ///
    /// Number of entries successfully warmed
    pub async fn warm_cache(&self, keys: Vec<String>) -> Result<usize> {
        self.cache_manager
            .warm_cache(keys)
            .await
            .map_err(|e| anyhow::anyhow!("Cache warming failed: {}", e))
    }

    /// Get cache statistics
    ///
    /// # Returns
    ///
    /// Current cache statistics including hit rate, size, etc.
    pub async fn get_cache_stats(&self) -> Result<CacheStats> {
        self.cache_manager
            .get_stats()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get cache stats: {}", e))
    }

    /// Create a new tenant with configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Tenant configuration
    /// * `owner` - Tenant owner information
    /// * `billing_plan` - Billing plan for tenant
    ///
    /// # Returns
    ///
    /// Generated tenant ID
    pub async fn create_tenant(
        &self,
        config: TenantConfig,
        owner: TenantOwner,
        billing_plan: BillingPlan,
    ) -> Result<String> {
        self.tenant_manager
            .create_tenant(config, owner, billing_plan)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create tenant: {}", e))
    }

    /// Get tenant information
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - Tenant ID to query
    ///
    /// # Returns
    ///
    /// Tenant summary if found
    pub async fn get_tenant(&self, tenant_id: &str) -> Result<Option<TenantSummary>> {
        self.tenant_manager
            .get_tenant(tenant_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get tenant: {}", e))
    }

    /// List all tenants
    ///
    /// # Returns
    ///
    /// Vector of all tenant summaries
    pub async fn list_tenants(&self) -> Result<Vec<TenantSummary>> {
        self.tenant_manager
            .list_tenants()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list tenants: {}", e))
    }

    /// Update tenant configuration
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - Tenant ID to update
    /// * `config` - New tenant configuration
    pub async fn update_tenant(&self, tenant_id: &str, config: TenantConfig) -> Result<()> {
        self.tenant_manager
            .update_tenant(tenant_id, config)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to update tenant: {}", e))
    }

    /// Delete tenant and all associated data
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - Tenant ID to delete
    pub async fn delete_tenant(&self, tenant_id: &str) -> Result<()> {
        self.tenant_manager
            .delete_tenant(tenant_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete tenant: {}", e))
    }

    /// Check tenant quota
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - Tenant ID to check
    /// * `resource` - Resource type (e.g., "operations_per_minute")
    /// * `amount` - Amount to check/consume
    ///
    /// # Returns
    ///
    /// Ok if quota check passes, error otherwise
    pub async fn check_quota(&self, tenant_id: &str, resource: &str, amount: u64) -> Result<()> {
        self.tenant_manager
            .check_quota(tenant_id, resource, amount)
            .await
            .map_err(|e| anyhow::anyhow!("Quota check failed: {}", e))
    }

    /// Get tenant resource usage statistics
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - Tenant ID to query
    ///
    /// # Returns
    ///
    /// Resource usage statistics
    pub async fn get_tenant_usage(
        &self,
        tenant_id: &str,
    ) -> Result<riptide_persistence::ResourceUsage> {
        self.tenant_manager
            .get_resource_usage(tenant_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get tenant usage: {}", e))
    }

    /// Get tenant billing information
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - Tenant ID to query
    ///
    /// # Returns
    ///
    /// Billing information including plan and usage
    pub async fn get_tenant_billing(
        &self,
        tenant_id: &str,
    ) -> Result<riptide_persistence::tenant::BillingInfo> {
        self.tenant_manager
            .get_billing_info(tenant_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get billing info: {}", e))
    }

    /// Reload configuration from disk
    ///
    /// # Returns
    ///
    /// Ok if reload successful
    pub async fn reload_config(&self) -> Result<()> {
        self.state_manager
            .reload_config()
            .await
            .map_err(|e| anyhow::anyhow!("Config reload failed: {}", e))
    }

    /// Create checkpoint of current state
    ///
    /// # Returns
    ///
    /// Checkpoint identifier
    pub async fn create_checkpoint(&self) -> Result<Checkpoint> {
        self.state_manager
            .create_checkpoint()
            .await
            .map_err(|e| anyhow::anyhow!("Checkpoint creation failed: {}", e))
    }

    /// Restore state from checkpoint
    ///
    /// # Arguments
    ///
    /// * `checkpoint_id` - Checkpoint ID to restore
    pub async fn restore_checkpoint(&self, checkpoint_id: &str) -> Result<()> {
        self.state_manager
            .restore_checkpoint(checkpoint_id)
            .await
            .map_err(|e| anyhow::anyhow!("Checkpoint restore failed: {}", e))
    }

    /// Get list of available checkpoints
    ///
    /// # Returns
    ///
    /// Vector of checkpoint information
    pub async fn list_checkpoints(&self) -> Result<Vec<Checkpoint>> {
        self.state_manager
            .list_checkpoints()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list checkpoints: {}", e))
    }

    /// Get current state snapshot
    ///
    /// # Returns
    ///
    /// Current state including configuration and metrics
    pub async fn get_state_snapshot(
        &self,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let mut snapshot = HashMap::new();

        // Add cache stats
        if let Ok(stats) = self.get_cache_stats().await {
            snapshot.insert("cache_stats".to_string(), serde_json::to_value(stats)?);
        }

        // Add tenant count
        if let Ok(tenants) = self.list_tenants().await {
            snapshot.insert(
                "tenant_count".to_string(),
                serde_json::to_value(tenants.len())?,
            );
        }

        // Add configuration
        snapshot.insert(
            "config".to_string(),
            serde_json::to_value(&self.config)?,
        );

        Ok(snapshot)
    }

    /// Check if persistence layer is healthy
    ///
    /// # Returns
    ///
    /// true if all components are operational
    pub async fn health_check(&self) -> bool {
        // Check cache manager
        if self.cache_manager.get_stats().await.is_err() {
            return false;
        }

        // Check tenant manager
        if self.tenant_manager.list_tenants().await.is_err() {
            return false;
        }

        // Check state manager
        if self.state_manager.list_checkpoints().await.is_err() {
            return false;
        }

        true
    }

    /// Get reference to cache manager (for advanced operations)
    pub fn cache_manager(&self) -> &Arc<PersistentCacheManager> {
        &self.cache_manager
    }

    /// Get reference to tenant manager (for advanced operations)
    pub fn tenant_manager(&self) -> &Arc<TenantManager> {
        &self.tenant_manager
    }

    /// Get reference to state manager (for advanced operations)
    pub fn state_manager(&self) -> &Arc<StateManager> {
        &self.state_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_persistence_adapter_creation() {
        // This test requires a running Redis instance
        // Skip in CI environments without Redis
        if std::env::var("SKIP_REDIS_TESTS").is_ok() {
            return;
        }

        let config = PersistenceConfig::default();
        let adapter = PersistenceAdapter::new("redis://localhost:6379", config).await;

        assert!(
            adapter.is_ok(),
            "PersistenceAdapter should initialize successfully"
        );
    }
}
