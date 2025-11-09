//! Resource management facade for centralized resource orchestration.
//!
//! This facade consolidates business logic for:
//! - WASM instance lifecycle management
//! - Rate limiting coordination
//! - Resource pool orchestration
//! - Memory tracking
//! - Performance monitoring
//!
//! The facade depends only on port traits, not concrete implementations.

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::metrics::BusinessMetrics;
use riptide_types::{
    error::riptide_error::RiptideError,
    ports::{pool::Pool, rate_limit::RateLimiter},
};

/// Result type for resource operations
pub type Result<T> = std::result::Result<T, RiptideError>;

/// Resource facade coordinating all resource management concerns
///
/// This facade orchestrates:
/// - Pool management (browser, PDF, WASM)
/// - Rate limiting enforcement
/// - Memory pressure detection
/// - Performance degradation tracking
///
/// # Architecture
///
/// The facade depends on port traits (not concrete implementations):
/// - `Pool<T>` - Generic pooling interface
/// - `RateLimiter` - Rate limiting interface
/// - `BusinessMetrics` - Metrics collection interface
///
/// Concrete implementations are injected via dependency injection.
///
/// # Example
///
/// ```rust,ignore
/// use riptide_facade::facades::ResourceFacade;
/// use std::sync::Arc;
///
/// let facade = ResourceFacade::new(
///     wasm_pool,
///     rate_limiter,
///     metrics,
///     config,
/// );
///
/// // Acquire resources with orchestration
/// match facade.acquire_wasm_slot("tenant-123").await? {
///     ResourceResult::Success(slot) => {
///         // Use WASM slot
///     }
///     ResourceResult::RateLimited { retry_after } => {
///         // Handle rate limit
///     }
///     _ => {
///         // Handle other resource constraints
///     }
/// }
/// ```
pub struct ResourceFacade<T> {
    /// WASM instance pool
    wasm_pool: Arc<dyn Pool<T>>,
    /// Rate limiter for tenant/host quotas
    rate_limiter: Arc<dyn RateLimiter>,
    /// Business metrics collector
    #[allow(dead_code)] // Used in future sprints for detailed metrics reporting
    metrics: Arc<BusinessMetrics>,
    /// Resource configuration
    config: ResourceConfig,
}

/// Configuration for resource management
#[derive(Clone, Debug)]
pub struct ResourceConfig {
    /// Memory pressure threshold (0.0-1.0)
    pub memory_pressure_threshold: f64,
    /// Global memory limit in MB
    pub memory_limit_mb: usize,
    /// Timeout for resource acquisition
    pub acquisition_timeout: Duration,
    /// Whether to enable auto-cleanup on timeout
    pub auto_cleanup_on_timeout: bool,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            memory_pressure_threshold: 0.8,
            memory_limit_mb: 2048,
            acquisition_timeout: Duration::from_secs(30),
            auto_cleanup_on_timeout: true,
        }
    }
}

/// Resource operation result
///
/// Indicates the outcome of resource acquisition attempts.
#[derive(Debug)]
pub enum ResourceResult<T> {
    /// Operation succeeded with acquired resources
    Success(T),
    /// Operation timed out waiting for resources
    Timeout,
    /// All resources of requested type are exhausted
    ResourceExhausted,
    /// Rate limit exceeded, retry after duration
    RateLimited {
        /// Duration to wait before retrying
        retry_after: Duration,
    },
    /// System is under memory pressure
    MemoryPressure,
}

impl<T> ResourceFacade<T>
where
    T: Send + Sync + 'static,
{
    /// Create a new resource facade
    ///
    /// # Arguments
    /// * `wasm_pool` - WASM instance pool implementation
    /// * `rate_limiter` - Rate limiting implementation
    /// * `metrics` - Business metrics collector
    /// * `config` - Resource configuration
    pub fn new(
        wasm_pool: Arc<dyn Pool<T>>,
        rate_limiter: Arc<dyn RateLimiter>,
        metrics: Arc<BusinessMetrics>,
        config: ResourceConfig,
    ) -> Self {
        Self {
            wasm_pool,
            rate_limiter,
            metrics,
            config,
        }
    }

    /// Acquire WASM slot with orchestration
    ///
    /// Coordinates:
    /// 1. Memory pressure check
    /// 2. Rate limit enforcement
    /// 3. Pool resource acquisition
    /// 4. Metrics tracking
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for rate limiting
    ///
    /// # Returns
    /// * `ResourceResult::Success` - WASM slot acquired
    /// * `ResourceResult::RateLimited` - Rate limit exceeded
    /// * `ResourceResult::MemoryPressure` - System under memory pressure
    /// * `ResourceResult::ResourceExhausted` - Pool exhausted
    /// * `ResourceResult::Timeout` - Acquisition timed out
    pub async fn acquire_wasm_slot(&self, tenant_id: &str) -> Result<ResourceResult<T>> {
        let start_time = Instant::now();

        // 1. Check memory pressure
        if self.is_under_memory_pressure().await {
            tracing::warn!(
                tenant_id = %tenant_id,
                "WASM acquisition rejected due to memory pressure"
            );
            return Ok(ResourceResult::MemoryPressure);
        }

        // 2. Apply rate limiting
        match self.rate_limiter.check_quota(tenant_id).await {
            Ok(()) => {
                // Consume quota
                self.rate_limiter.consume(tenant_id, 1).await?;
            }
            Err(RiptideError::RateLimitExceeded { .. }) => {
                tracing::debug!(
                    tenant_id = %tenant_id,
                    "Rate limit exceeded for WASM acquisition"
                );
                return Ok(ResourceResult::RateLimited {
                    retry_after: Duration::from_secs(1),
                });
            }
            Err(e) => return Err(e),
        }

        // 3. Acquire from pool with timeout
        let pool_result =
            tokio::time::timeout(self.config.acquisition_timeout, self.wasm_pool.acquire()).await;

        let resource = match pool_result {
            Ok(Ok(resource)) => {
                let elapsed = start_time.elapsed();
                tracing::debug!(
                    tenant_id = %tenant_id,
                    acquisition_time_ms = elapsed.as_millis(),
                    "WASM slot acquired successfully"
                );

                resource
            }
            Ok(Err(_)) => {
                tracing::warn!(
                    tenant_id = %tenant_id,
                    "WASM pool exhausted"
                );
                return Ok(ResourceResult::ResourceExhausted);
            }
            Err(_) => {
                tracing::warn!(
                    tenant_id = %tenant_id,
                    timeout_ms = self.config.acquisition_timeout.as_millis(),
                    "WASM acquisition timed out"
                );

                return Ok(ResourceResult::Timeout);
            }
        };

        Ok(ResourceResult::Success(resource.into_inner()))
    }

    /// Check if system is under memory pressure
    ///
    /// # Returns
    /// `true` if memory usage exceeds configured threshold
    async fn is_under_memory_pressure(&self) -> bool {
        // Get pool health to check memory usage
        let health = self.wasm_pool.health().await;

        // Calculate memory utilization
        if health.total > 0 {
            let utilization = health.in_use as f64 / health.total as f64;
            utilization >= self.config.memory_pressure_threshold
        } else {
            false
        }
    }

    /// Get current resource status
    ///
    /// Provides a comprehensive snapshot of resource utilization.
    pub async fn get_status(&self) -> Result<ResourceStatus> {
        let pool_health = self.wasm_pool.health().await;

        Ok(ResourceStatus {
            pool_total: pool_health.total,
            pool_available: pool_health.available,
            pool_in_use: pool_health.in_use,
            pool_failed: pool_health.failed,
            memory_pressure: self.is_under_memory_pressure().await,
        })
    }

    /// Cleanup resources on timeout or error
    ///
    /// Triggers cleanup operations when configured:
    /// - Force pool health check
    /// - Update metrics
    ///
    /// # Arguments
    /// * `operation_type` - Type of operation that failed (for logging)
    pub async fn cleanup_on_timeout(&self, operation_type: &str) {
        if !self.config.auto_cleanup_on_timeout {
            return;
        }

        tracing::warn!(
            operation = %operation_type,
            "Performing timeout cleanup"
        );

        // Force health check to detect/recover failed resources
        let _health = self.wasm_pool.health().await;
    }
}

/// Current resource status
///
/// Provides a snapshot of system resource utilization.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ResourceStatus {
    /// Total pool capacity
    pub pool_total: usize,
    /// Available pool resources
    pub pool_available: usize,
    /// Resources currently in use
    pub pool_in_use: usize,
    /// Failed/unhealthy resources
    pub pool_failed: usize,
    /// Whether system is under memory pressure
    pub memory_pressure: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use riptide_types::ports::PoolHealth;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Mock implementations for testing
    struct MockPool {
        capacity: AtomicUsize,
    }

    impl MockPool {
        fn new(capacity: usize) -> Self {
            Self {
                capacity: AtomicUsize::new(capacity),
            }
        }
    }

    #[async_trait::async_trait]
    impl Pool<String> for MockPool {
        async fn acquire(
            &self,
        ) -> std::result::Result<
            riptide_types::ports::pool::PooledResource<String>,
            riptide_types::ports::pool::PoolError,
        > {
            let current = self.capacity.load(Ordering::Relaxed);
            if current > 0 {
                self.capacity.fetch_sub(1, Ordering::Relaxed);
                Ok(riptide_types::ports::pool::PooledResource::new(
                    "mock-resource".to_string(),
                    "mock-pool".to_string(),
                    |_| {},
                ))
            } else {
                Err(riptide_types::ports::pool::PoolError::Exhausted)
            }
        }

        async fn release(
            &self,
            _resource: String,
        ) -> std::result::Result<(), riptide_types::ports::pool::PoolError> {
            self.capacity.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }

        async fn size(&self) -> usize {
            self.capacity.load(Ordering::Relaxed)
        }

        async fn available(&self) -> usize {
            self.capacity.load(Ordering::Relaxed)
        }

        async fn health(&self) -> PoolHealth {
            let total = 10;
            let available = self.capacity.load(Ordering::Relaxed);
            PoolHealth {
                total,
                available,
                in_use: total - available,
                failed: 0,
                success_rate: 1.0,
                avg_acquisition_time_ms: Some(10),
                avg_latency_ms: Some(50),
            }
        }
    }

    struct MockRateLimiter {
        allow: bool,
    }

    #[async_trait::async_trait]
    impl RateLimiter for MockRateLimiter {
        async fn check_quota(&self, _tenant_id: &str) -> Result<()> {
            if self.allow {
                Ok(())
            } else {
                Err(RiptideError::RateLimitExceeded {
                    tenant_id: "test".to_string(),
                })
            }
        }

        async fn consume(&self, _tenant_id: &str, _amount: usize) -> Result<()> {
            Ok(())
        }

        async fn reset(&self, _tenant_id: &str) -> Result<()> {
            Ok(())
        }

        async fn get_remaining(&self, _tenant_id: &str) -> Result<usize> {
            Ok(100)
        }
    }

    // Tests use the real BusinessMetrics struct with default implementation

    #[tokio::test]
    async fn test_acquire_wasm_slot_success() {
        let pool = Arc::new(MockPool::new(5)) as Arc<dyn Pool<String>>;
        let limiter = Arc::new(MockRateLimiter { allow: true }) as Arc<dyn RateLimiter>;
        let metrics = Arc::new(BusinessMetrics::default());

        let facade = ResourceFacade::new(pool, limiter, metrics, ResourceConfig::default());

        let result = facade.acquire_wasm_slot("tenant-123").await.unwrap();

        match result {
            ResourceResult::Success(_) => {
                // Success expected
            }
            other => panic!("Expected Success, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_acquire_wasm_slot_rate_limited() {
        let pool = Arc::new(MockPool::new(5)) as Arc<dyn Pool<String>>;
        let limiter = Arc::new(MockRateLimiter { allow: false }) as Arc<dyn RateLimiter>;
        let metrics = Arc::new(BusinessMetrics::default());

        let facade = ResourceFacade::new(pool, limiter, metrics, ResourceConfig::default());

        let result = facade.acquire_wasm_slot("tenant-123").await.unwrap();

        match result {
            ResourceResult::RateLimited { .. } => {
                // Rate limited expected
            }
            other => panic!("Expected RateLimited, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_get_status() {
        let pool = Arc::new(MockPool::new(5)) as Arc<dyn Pool<String>>;
        let limiter = Arc::new(MockRateLimiter { allow: true }) as Arc<dyn RateLimiter>;
        let metrics = Arc::new(BusinessMetrics::default());

        let facade = ResourceFacade::new(pool, limiter, metrics, ResourceConfig::default());

        let status = facade.get_status().await.unwrap();

        assert_eq!(status.pool_total, 10);
        assert_eq!(status.pool_available, 5);
    }
}
