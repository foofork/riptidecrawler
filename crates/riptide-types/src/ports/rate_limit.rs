//! Rate limiting port for hexagonal architecture.
//!
//! Provides backend-agnostic trait for rate limiting functionality,
//! enabling dependency inversion and testability.

use async_trait::async_trait;
use std::time::Duration;

use crate::error::riptide_error::RiptideError;

/// Result type for rate limiting operations
pub type Result<T> = std::result::Result<T, RiptideError>;

/// Rate limiting port trait
///
/// Defines the interface for rate limiting implementations.
/// Concrete adapters (e.g., RedisRateLimiter, InMemoryRateLimiter)
/// implement this trait to provide actual rate limiting logic.
///
/// # Example
///
/// ```rust,ignore
/// use riptide_types::ports::RateLimiter;
///
/// async fn check_limit(limiter: &dyn RateLimiter, tenant_id: &str) -> Result<()> {
///     limiter.check_quota(tenant_id).await?;
///     limiter.consume(tenant_id, 1).await?;
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if a tenant has available quota without consuming it
    ///
    /// # Arguments
    /// * `tenant_id` - The tenant identifier to check quota for
    ///
    /// # Returns
    /// * `Ok(())` - Quota is available
    /// * `Err(RiptideError::RateLimitExceeded)` - Quota exceeded
    async fn check_quota(&self, tenant_id: &str) -> Result<()>;

    /// Consume quota for a tenant
    ///
    /// # Arguments
    /// * `tenant_id` - The tenant identifier
    /// * `amount` - Number of units to consume (typically 1 for single request)
    ///
    /// # Returns
    /// * `Ok(())` - Quota consumed successfully
    /// * `Err(RiptideError::RateLimitExceeded)` - Quota would be exceeded
    async fn consume(&self, tenant_id: &str, amount: usize) -> Result<()>;

    /// Reset quota for a tenant
    ///
    /// Typically used for testing or administrative purposes.
    ///
    /// # Arguments
    /// * `tenant_id` - The tenant identifier to reset quota for
    async fn reset(&self, tenant_id: &str) -> Result<()>;

    /// Get remaining quota for a tenant
    ///
    /// # Arguments
    /// * `tenant_id` - The tenant identifier
    ///
    /// # Returns
    /// Number of remaining quota units
    async fn get_remaining(&self, tenant_id: &str) -> Result<usize>;
}

/// Statistics for rate limiter
///
/// Provides insights into rate limiter behavior for monitoring
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RateLimitStats {
    /// Total number of quota checks performed
    pub total_checks: u64,
    /// Number of quota checks that succeeded
    pub successful_checks: u64,
    /// Number of quota checks that were rate limited
    pub rate_limited_checks: u64,
    /// Number of tenants currently being tracked
    pub tracked_tenants: usize,
}

/// Per-host rate limiting port trait
///
/// Extension of RateLimiter for per-host rate limiting with additional features:
/// - Token bucket algorithm support
/// - Jitter for thundering herd prevention
/// - Automatic cleanup of stale hosts
#[async_trait]
pub trait PerHostRateLimiter: RateLimiter {
    /// Check rate limit for a specific host
    ///
    /// Returns `Ok(())` if allowed, or an error with retry duration if rate limited.
    ///
    /// # Arguments
    /// * `host` - The hostname to check rate limit for
    ///
    /// # Returns
    /// * `Ok(())` - Request allowed
    /// * `Err(duration)` - Rate limited, contains suggested retry duration
    async fn check_rate_limit(&self, host: &str) -> std::result::Result<(), Duration>;

    /// Get statistics for a specific host
    ///
    /// # Arguments
    /// * `host` - The hostname to get stats for
    ///
    /// # Returns
    /// * `Some(HostStats)` - Statistics if host is tracked
    /// * `None` - Host has not made any requests yet
    async fn get_host_stats(&self, host: &str) -> Option<HostStats>;

    /// Get statistics for all tracked hosts
    ///
    /// # Returns
    /// Vector of (hostname, stats) tuples for all tracked hosts
    async fn get_all_stats(&self) -> Vec<(String, HostStats)>;

    /// Get total number of hosts being tracked
    async fn tracked_hosts_count(&self) -> usize;
}

/// Statistics for a specific host's rate limiting
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HostStats {
    /// Total requests from this host
    pub request_count: u64,
    /// Currently available tokens (for token bucket algorithm)
    pub available_tokens: f64,
    /// Time since last request
    pub last_request_age: Duration,
}
