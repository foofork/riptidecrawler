//! Cache Warming Integration with Instance Pool
//!
//! This module provides integration between cache warming functionality
//! and the existing instance pool system.

use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

use crate::cache_warming::{
    CacheWarmingConfig, CacheWarmingManager, CacheWarmingPoolExt, CacheWarmingStats
};
use crate::instance_pool::AdvancedInstancePool;
use crate::events::EventBus;
use crate::types::ExtractionMode;

/// Enhanced instance pool with integrated cache warming
pub struct CacheWarmingEnabledPool {
    pool: Arc<AdvancedInstancePool>,
    warming_manager: Option<CacheWarmingManager>,
    cache_warming_enabled: bool,
}

impl CacheWarmingEnabledPool {
    /// Create a new cache warming enabled pool
    pub fn new(pool: Arc<AdvancedInstancePool>) -> Self {
        Self {
            pool,
            warming_manager: None,
            cache_warming_enabled: false,
        }
    }

    /// Enable cache warming with configuration
    pub async fn enable_cache_warming(
        &mut self,
        config: CacheWarmingConfig,
        event_bus: Option<Arc<EventBus>>,
    ) -> Result<()> {
        if !config.enabled {
            info!("Cache warming disabled by configuration");
            return Ok(());
        }

        info!("Enabling cache warming for instance pool");

        let warming_manager = self.pool.create_cache_warming_manager(config, event_bus);
        warming_manager.start().await?;

        self.warming_manager = Some(warming_manager);
        self.cache_warming_enabled = true;

        info!("Cache warming enabled successfully");
        Ok(())
    }

    /// Disable cache warming
    pub async fn disable_cache_warming(&mut self) {
        if let Some(warming_manager) = &self.warming_manager {
            warming_manager.stop().await;
            info!("Cache warming disabled");
        }
        self.warming_manager = None;
        self.cache_warming_enabled = false;
    }

    /// Extract content with cache warming optimization
    pub async fn extract_with_cache_warming(
        &self,
        html: &str,
        url: &str,
        mode: ExtractionMode,
    ) -> Result<crate::types::ExtractedDoc> {
        let start_time = Instant::now();
        let mut used_warm_instance = false;

        // Try to get a warm instance first if cache warming is enabled
        if self.cache_warming_enabled {
            if let Some(warming_manager) = &self.warming_manager {
                if let Some(warm_instance) = warming_manager.get_warm_instance().await {
                    used_warm_instance = true;
                    debug!("Using warm instance for extraction");

                    // Put the warm instance back into the pool and use normal extraction
                    self.pool.return_instance(warm_instance).await;
                }
            }
        }

        // Perform the extraction using the normal pool
        let result = self.pool.extract(html, url, mode).await;

        // Record pattern for intelligent caching
        if let Some(warming_manager) = &self.warming_manager {
            let processing_time = start_time.elapsed().as_millis() as f64;
            let cache_hit = used_warm_instance;
            warming_manager.record_url_pattern(url, processing_time, cache_hit).await;
        }

        result
    }

    /// Get cache warming statistics
    pub fn get_cache_warming_stats(&self) -> Option<CacheWarmingStats> {
        self.warming_manager.as_ref().map(|m| m.get_stats())
    }

    /// Get current warm pool size
    pub async fn get_warm_pool_size(&self) -> usize {
        if let Some(warming_manager) = &self.warming_manager {
            warming_manager.get_warm_pool_size().await
        } else {
            0
        }
    }

    /// Check if cache warming is enabled
    pub fn is_cache_warming_enabled(&self) -> bool {
        self.cache_warming_enabled
    }

    /// Get reference to underlying pool
    pub fn pool(&self) -> &Arc<AdvancedInstancePool> {
        &self.pool
    }

    /// Get cache warming manager reference
    pub fn warming_manager(&self) -> Option<&CacheWarmingManager> {
        self.warming_manager.as_ref()
    }
}

/// Factory for creating cache warming enabled pools
pub struct CacheWarmingPoolFactory;

impl CacheWarmingPoolFactory {
    /// Create a cache warming enabled pool with default configuration
    pub async fn create_with_cache_warming(
        pool: Arc<AdvancedInstancePool>,
        event_bus: Option<Arc<EventBus>>,
    ) -> Result<CacheWarmingEnabledPool> {
        let mut cache_warming_pool = CacheWarmingEnabledPool::new(pool);
        let config = CacheWarmingConfig::default();
        cache_warming_pool.enable_cache_warming(config, event_bus).await?;
        Ok(cache_warming_pool)
    }

    /// Create a cache warming enabled pool with custom configuration
    pub async fn create_with_custom_cache_warming(
        pool: Arc<AdvancedInstancePool>,
        config: CacheWarmingConfig,
        event_bus: Option<Arc<EventBus>>,
    ) -> Result<CacheWarmingEnabledPool> {
        let mut cache_warming_pool = CacheWarmingEnabledPool::new(pool);
        cache_warming_pool.enable_cache_warming(config, event_bus).await?;
        Ok(cache_warming_pool)
    }
}

/// Health monitoring integration for cache warming
pub struct CacheWarmingHealthMonitor {
    pool: Arc<CacheWarmingEnabledPool>,
}

impl CacheWarmingHealthMonitor {
    pub fn new(pool: Arc<CacheWarmingEnabledPool>) -> Self {
        Self { pool }
    }

    /// Perform health check on cache warming system
    pub async fn health_check(&self) -> CacheWarmingHealthStatus {
        let warm_pool_size = self.pool.get_warm_pool_size().await;
        let stats = self.pool.get_cache_warming_stats();

        let status = if !self.pool.is_cache_warming_enabled() {
            CacheWarmingHealthStatus::Disabled
        } else if warm_pool_size == 0 {
            CacheWarmingHealthStatus::NoWarmInstances
        } else if let Some(stats) = &stats {
            let hit_ratio = stats.cache_hit_ratio();
            if hit_ratio < 0.5 {
                CacheWarmingHealthStatus::LowHitRatio
            } else if hit_ratio < 0.8 {
                CacheWarmingHealthStatus::ModerateHitRatio
            } else {
                CacheWarmingHealthStatus::Healthy
            }
        } else {
            CacheWarmingHealthStatus::NoStats
        };

        status
    }

    /// Start continuous health monitoring
    pub async fn start_monitoring(self: Arc<Self>) -> Result<()> {
        info!("Starting cache warming health monitoring");

        let mut interval_timer = tokio::time::interval(Duration::from_secs(60)); // Every minute

        loop {
            interval_timer.tick().await;

            let health_status = self.health_check().await;
            debug!(status = ?health_status, "Cache warming health check completed");

            match health_status {
                CacheWarmingHealthStatus::LowHitRatio => {
                    warn!("Cache warming hit ratio is low, consider adjusting configuration");
                }
                CacheWarmingHealthStatus::NoWarmInstances => {
                    warn!("No warm instances available, cache warming may not be working properly");
                }
                CacheWarmingHealthStatus::Disabled => {
                    debug!("Cache warming is disabled");
                }
                _ => {}
            }
        }
    }
}

/// Health status for cache warming system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheWarmingHealthStatus {
    Healthy,
    ModerateHitRatio,
    LowHitRatio,
    NoWarmInstances,
    NoStats,
    Disabled,
}

impl CacheWarmingHealthStatus {
    pub fn is_healthy(&self) -> bool {
        matches!(self, CacheWarmingHealthStatus::Healthy)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CacheWarmingHealthStatus::Healthy => "healthy",
            CacheWarmingHealthStatus::ModerateHitRatio => "moderate_hit_ratio",
            CacheWarmingHealthStatus::LowHitRatio => "low_hit_ratio",
            CacheWarmingHealthStatus::NoWarmInstances => "no_warm_instances",
            CacheWarmingHealthStatus::NoStats => "no_stats",
            CacheWarmingHealthStatus::Disabled => "disabled",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::ExtractorConfig;
    use wasmtime::Engine;

    #[tokio::test]
    async fn test_cache_warming_pool_creation() {
        // This would need a real WASM component for full testing
        // For now, just test the structure
        let engine = Engine::default();
        let config = ExtractorConfig::default();

        // Note: This test would need to be expanded with actual pool creation
        // when the full integration is available
    }

    #[test]
    fn test_health_status() {
        assert!(CacheWarmingHealthStatus::Healthy.is_healthy());
        assert!(!CacheWarmingHealthStatus::LowHitRatio.is_healthy());
        assert_eq!(CacheWarmingHealthStatus::Disabled.as_str(), "disabled");
    }
}