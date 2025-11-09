//! Generic pool interface for managing pooled resources
//!
//! This port defines a backend-agnostic interface for resource pooling,
//! enabling different pool implementations (WASM, Browser, LLM, etc.)
//! to share a common abstraction.
//!
//! # Architecture
//!
//! Following hexagonal architecture principles:
//! ```text
//! Domain Layer (riptide-types)
//!     ↓ defines Pool<T> trait
//! Infrastructure Layer (riptide-pool, riptide-browser, riptide-intelligence)
//!     ↓ implements concrete pools
//! Application Layer (riptide-facade, riptide-api)
//!     ↓ uses Pool<T> abstraction
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_types::ports::{Pool, PoolHealth};
//!
//! async fn use_pool<T>(pool: &dyn Pool<T>) -> Result<()> {
//!     // Acquire resource from pool
//!     let resource = pool.acquire().await?;
//!
//!     // Use resource...
//!
//!     // Release back to pool
//!     pool.release(resource).await?;
//!
//!     // Check pool health
//!     let health = pool.health().await;
//!     println!("Pool: {} available, {} in use", health.available, health.in_use);
//!
//!     Ok(())
//! }
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;
use thiserror::Error;

/// Generic pool interface for managing pooled resources
///
/// This trait provides a common abstraction for different pool implementations:
/// - WASM instance pools (riptide-pool)
/// - Browser session pools (riptide-browser)
/// - LLM client pools (riptide-intelligence)
///
/// # Type Parameters
///
/// * `T` - The type of resource being pooled
///
/// # Thread Safety
///
/// All methods are async and implementations must be `Send + Sync`.
#[async_trait]
pub trait Pool<T>: Send + Sync {
    /// Acquire a resource from the pool
    ///
    /// This method will:
    /// - Return an available resource if one exists
    /// - Create a new resource if pool is not at capacity
    /// - Wait until a resource becomes available (up to configured timeout)
    ///
    /// # Returns
    ///
    /// A `PooledResource<T>` which auto-releases on drop
    ///
    /// # Errors
    ///
    /// - `PoolError::Exhausted` if no resources available and pool at capacity
    /// - `PoolError::CreationFailed` if resource creation fails
    /// - `PoolError::Timeout` if acquisition times out
    async fn acquire(&self) -> Result<PooledResource<T>, PoolError>;

    /// Release a resource back to the pool
    ///
    /// The pool will perform health checks before accepting the resource.
    /// Unhealthy resources will be discarded.
    ///
    /// # Arguments
    ///
    /// * `resource` - The resource to return to the pool
    ///
    /// # Errors
    ///
    /// - `PoolError::ValidationFailed` if resource fails health check
    async fn release(&self, resource: T) -> Result<(), PoolError>;

    /// Get current pool size (total instances)
    async fn size(&self) -> usize;

    /// Get number of available resources
    async fn available(&self) -> usize;

    /// Get number of resources currently in use
    async fn in_use(&self) -> usize {
        self.size().await - self.available().await
    }

    /// Get comprehensive pool health metrics
    async fn health(&self) -> PoolHealth;

    /// Get pool statistics for monitoring
    async fn stats(&self) -> PoolStats {
        let health = self.health().await;
        PoolStats {
            total: health.total,
            available: health.available,
            in_use: health.in_use,
            failed: health.failed,
            utilization: if health.total > 0 {
                health.in_use as f64 / health.total as f64
            } else {
                0.0
            },
            success_rate: health.success_rate,
        }
    }
}

/// RAII wrapper for pooled resources
///
/// Automatically releases the resource back to the pool when dropped.
/// This ensures resources are always returned even if code panics.
pub struct PooledResource<T> {
    resource: Option<T>,
    pool_id: String,
    release_fn: Option<Box<dyn FnOnce(T) + Send>>,
}

impl<T> PooledResource<T> {
    /// Create a new pooled resource
    ///
    /// # Arguments
    ///
    /// * `resource` - The underlying resource
    /// * `pool_id` - Identifier for the pool (for logging/metrics)
    /// * `release_fn` - Callback to release the resource
    pub fn new<F>(resource: T, pool_id: String, release_fn: F) -> Self
    where
        F: FnOnce(T) + Send + 'static,
    {
        Self {
            resource: Some(resource),
            pool_id,
            release_fn: Some(Box::new(release_fn)),
        }
    }

    /// Get a reference to the underlying resource
    pub fn get(&self) -> &T {
        self.resource
            .as_ref()
            .expect("PooledResource already consumed")
    }

    /// Get a mutable reference to the underlying resource
    pub fn get_mut(&mut self) -> &mut T {
        self.resource
            .as_mut()
            .expect("PooledResource already consumed")
    }

    /// Consume the wrapper and return the underlying resource
    ///
    /// **Warning**: This prevents automatic release on drop.
    /// Caller is responsible for manually releasing the resource.
    pub fn into_inner(mut self) -> T {
        self.resource
            .take()
            .expect("PooledResource already consumed")
    }

    /// Get the pool identifier
    pub fn pool_id(&self) -> &str {
        &self.pool_id
    }
}

impl<T> Drop for PooledResource<T> {
    fn drop(&mut self) {
        if let (Some(resource), Some(release_fn)) = (self.resource.take(), self.release_fn.take()) {
            // Execute release callback
            release_fn(resource);
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for PooledResource<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PooledResource")
            .field("pool_id", &self.pool_id)
            .field("resource", &self.resource)
            .finish()
    }
}

/// Pool health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolHealth {
    /// Total number of instances in the pool
    pub total: usize,
    /// Number of available instances
    pub available: usize,
    /// Number of instances currently in use
    pub in_use: usize,
    /// Number of failed/unhealthy instances
    pub failed: usize,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Average acquisition time in milliseconds
    pub avg_acquisition_time_ms: Option<u64>,
    /// Average operation latency in milliseconds
    pub avg_latency_ms: Option<u64>,
}

impl PoolHealth {
    /// Create a new health snapshot
    pub fn new(
        total: usize,
        available: usize,
        in_use: usize,
        failed: usize,
        success_rate: f64,
    ) -> Self {
        Self {
            total,
            available,
            in_use,
            failed,
            success_rate,
            avg_acquisition_time_ms: None,
            avg_latency_ms: None,
        }
    }

    /// Check if pool is healthy
    pub fn is_healthy(&self) -> bool {
        self.success_rate >= 0.95 && self.available > 0
    }

    /// Check if pool is exhausted
    pub fn is_exhausted(&self) -> bool {
        self.available == 0 && self.in_use >= self.total
    }

    /// Get utilization percentage (0.0 to 100.0)
    pub fn utilization(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.in_use as f64 / self.total as f64) * 100.0
        }
    }
}

/// Pool statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    /// Total number of instances
    pub total: usize,
    /// Number of available instances
    pub available: usize,
    /// Number of instances in use
    pub in_use: usize,
    /// Number of failed instances
    pub failed: usize,
    /// Pool utilization (0.0 to 1.0)
    pub utilization: f64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
}

impl PoolStats {
    /// Check if pool is at capacity
    pub fn is_at_capacity(&self) -> bool {
        self.available == 0
    }

    /// Check if pool is underutilized
    pub fn is_underutilized(&self, threshold: f64) -> bool {
        self.utilization < threshold
    }
}

/// Pool error types
#[derive(Debug, Error, Clone)]
pub enum PoolError {
    /// Pool is exhausted (no resources available)
    #[error("Pool exhausted: no resources available")]
    Exhausted,

    /// Resource creation failed
    #[error("Resource creation failed: {0}")]
    CreationFailed(String),

    /// Resource validation failed
    #[error("Resource validation failed: {0}")]
    ValidationFailed(String),

    /// Operation timed out
    #[error("Pool operation timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    /// Resource is unhealthy
    #[error("Resource unhealthy: {reason}")]
    Unhealthy { reason: String },

    /// Pool is shutting down
    #[error("Pool is shutting down")]
    ShuttingDown,

    /// Generic pool error
    #[error("Pool error: {0}")]
    Other(String),
}

impl PoolError {
    /// Create a creation failed error
    pub fn creation_failed(msg: impl Into<String>) -> Self {
        Self::CreationFailed(msg.into())
    }

    /// Create a validation failed error
    pub fn validation_failed(msg: impl Into<String>) -> Self {
        Self::ValidationFailed(msg.into())
    }

    /// Create a timeout error
    pub fn timeout(duration: Duration) -> Self {
        Self::Timeout {
            timeout_ms: duration.as_millis() as u64,
        }
    }

    /// Create an unhealthy error
    pub fn unhealthy(reason: impl Into<String>) -> Self {
        Self::Unhealthy {
            reason: reason.into(),
        }
    }

    /// Create a generic error
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Exhausted | Self::Timeout { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_health_is_healthy() {
        let health = PoolHealth::new(10, 5, 3, 0, 0.98);
        assert!(health.is_healthy());

        let unhealthy = PoolHealth::new(10, 5, 3, 2, 0.90);
        assert!(!unhealthy.is_healthy());

        let exhausted = PoolHealth::new(10, 0, 10, 0, 0.98);
        assert!(!exhausted.is_healthy());
    }

    #[test]
    fn test_pool_health_is_exhausted() {
        let exhausted = PoolHealth::new(10, 0, 10, 0, 0.98);
        assert!(exhausted.is_exhausted());

        let not_exhausted = PoolHealth::new(10, 2, 8, 0, 0.98);
        assert!(!not_exhausted.is_exhausted());
    }

    #[test]
    fn test_pool_health_utilization() {
        let health = PoolHealth::new(10, 3, 7, 0, 0.98);
        assert!((health.utilization() - 70.0).abs() < 0.01);

        let empty = PoolHealth::new(0, 0, 0, 0, 1.0);
        assert_eq!(empty.utilization(), 0.0);
    }

    #[test]
    fn test_pool_stats() {
        let stats = PoolStats {
            total: 10,
            available: 3,
            in_use: 7,
            failed: 0,
            utilization: 0.7,
            success_rate: 0.95,
        };

        assert!(!stats.is_at_capacity());
        assert!(!stats.is_underutilized(0.5));
        assert!(stats.is_underutilized(0.8));
    }

    #[test]
    fn test_pool_error_is_retryable() {
        assert!(PoolError::Exhausted.is_retryable());
        assert!(PoolError::timeout(Duration::from_secs(5)).is_retryable());
        assert!(!PoolError::creation_failed("test").is_retryable());
        assert!(!PoolError::ShuttingDown.is_retryable());
    }
}
