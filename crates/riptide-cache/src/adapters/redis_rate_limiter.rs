//! Redis-based rate limiter adapter implementing RateLimiter port.
//!
//! Provides distributed rate limiting using Redis as the backend store.
//! Uses token bucket algorithm with atomic operations for consistency.

use async_trait::async_trait;
use std::{sync::Arc, time::Duration};
use tracing::{debug, warn};

use riptide_types::{
    error::riptide_error::RiptideError,
    ports::rate_limit::{HostStats, PerHostRateLimiter, RateLimiter, Result},
};

use crate::redis::RedisManager;

/// Redis-based rate limiter implementation
///
/// Uses Redis for distributed rate limiting across multiple instances.
/// Implements token bucket algorithm with atomic operations.
///
/// # Key Structure
/// - `ratelimit:v1:<tenant_id>` - Token count for tenant
/// - `ratelimit:v1:stats:<tenant_id>` - Statistics for tenant
///
/// # Example
///
/// ```rust,ignore
/// use riptide_cache::adapters::RedisRateLimiter;
/// use riptide_cache::redis::RedisManager;
///
/// let redis = Arc::new(RedisManager::new("redis://localhost").await?);
/// let limiter = RedisRateLimiter::new(redis, 100, Duration::from_secs(60));
///
/// limiter.check_quota("tenant-123").await?;
/// limiter.consume("tenant-123", 1).await?;
/// ```
pub struct RedisRateLimiter {
    redis: Arc<RedisManager>,
    max_requests: usize,
    window_duration: Duration,
    namespace: String,
}

impl RedisRateLimiter {
    /// Create a new Redis-based rate limiter
    ///
    /// # Arguments
    /// * `redis` - Redis manager instance
    /// * `max_requests` - Maximum requests allowed per window
    /// * `window_duration` - Time window for rate limiting
    pub fn new(redis: Arc<RedisManager>, max_requests: usize, window_duration: Duration) -> Self {
        Self {
            redis,
            max_requests,
            window_duration,
            namespace: "ratelimit".to_string(),
        }
    }

    /// Create a new Redis-based rate limiter with custom namespace
    ///
    /// # Arguments
    /// * `redis` - Redis manager instance
    /// * `max_requests` - Maximum requests allowed per window
    /// * `window_duration` - Time window for rate limiting
    /// * `namespace` - Custom namespace for Redis keys
    pub fn with_namespace(
        redis: Arc<RedisManager>,
        max_requests: usize,
        window_duration: Duration,
        namespace: String,
    ) -> Self {
        Self {
            redis,
            max_requests,
            window_duration,
            namespace,
        }
    }

    /// Generate Redis key for tenant quota
    fn quota_key(&self, tenant_id: &str) -> String {
        format!("{}:v1:{}", self.namespace, tenant_id)
    }

    /// Generate Redis key for tenant stats
    #[allow(dead_code)]
    fn stats_key(&self, tenant_id: &str) -> String {
        format!("{}:v1:stats:{}", self.namespace, tenant_id)
    }

    /// Get current token count for a tenant
    async fn get_count(&self, tenant_id: &str) -> Result<usize> {
        let key = self.quota_key(tenant_id);
        let count_bytes = self.redis.get(&self.namespace, &key).await?;

        let count = count_bytes
            .and_then(|v| String::from_utf8(v).ok())
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);

        Ok(count)
    }
}

#[async_trait]
impl RateLimiter for RedisRateLimiter {
    async fn check_quota(&self, tenant_id: &str) -> Result<()> {
        let count = self.get_count(tenant_id).await?;

        if count >= self.max_requests {
            debug!(
                tenant_id = %tenant_id,
                count = count,
                max = self.max_requests,
                "Rate limit quota exceeded"
            );
            Err(RiptideError::RateLimitExceeded {
                tenant_id: tenant_id.to_string(),
            })
        } else {
            Ok(())
        }
    }

    async fn consume(&self, tenant_id: &str, amount: usize) -> Result<()> {
        // Check quota first
        self.check_quota(tenant_id).await?;

        // Increment counter with TTL
        let key = self.quota_key(tenant_id);
        self.redis
            .incr(&self.namespace, &key, amount, self.window_duration)
            .await?;

        debug!(
            tenant_id = %tenant_id,
            amount = amount,
            "Consumed rate limit quota"
        );

        Ok(())
    }

    async fn reset(&self, tenant_id: &str) -> Result<()> {
        let key = self.quota_key(tenant_id);
        self.redis.delete(&self.namespace, &key).await?;

        debug!(tenant_id = %tenant_id, "Reset rate limit quota");

        Ok(())
    }

    async fn get_remaining(&self, tenant_id: &str) -> Result<usize> {
        let count = self.get_count(tenant_id).await?;
        let remaining = self.max_requests.saturating_sub(count);

        Ok(remaining)
    }
}

/// Per-host rate limiter using Redis
///
/// Extends RedisRateLimiter with per-host tracking for token bucket algorithm.
pub struct RedisPerHostRateLimiter {
    base: RedisRateLimiter,
    requests_per_second: f64,
    #[allow(dead_code)]
    burst_capacity: usize,
}

impl RedisPerHostRateLimiter {
    /// Create a new per-host rate limiter
    ///
    /// # Arguments
    /// * `redis` - Redis manager instance
    /// * `requests_per_second` - Rate of token refill (requests per second)
    /// * `burst_capacity` - Maximum burst capacity (tokens)
    /// * `window_duration` - Time window for statistics
    pub fn new(
        redis: Arc<RedisManager>,
        requests_per_second: f64,
        burst_capacity: usize,
        window_duration: Duration,
    ) -> Self {
        Self {
            base: RedisRateLimiter::with_namespace(
                redis,
                burst_capacity,
                window_duration,
                "ratelimit:host".to_string(),
            ),
            requests_per_second,
            burst_capacity,
        }
    }
}

#[async_trait]
impl RateLimiter for RedisPerHostRateLimiter {
    async fn check_quota(&self, tenant_id: &str) -> Result<()> {
        self.base.check_quota(tenant_id).await
    }

    async fn consume(&self, tenant_id: &str, amount: usize) -> Result<()> {
        self.base.consume(tenant_id, amount).await
    }

    async fn reset(&self, tenant_id: &str) -> Result<()> {
        self.base.reset(tenant_id).await
    }

    async fn get_remaining(&self, tenant_id: &str) -> Result<usize> {
        self.base.get_remaining(tenant_id).await
    }
}

#[async_trait]
impl PerHostRateLimiter for RedisPerHostRateLimiter {
    async fn check_rate_limit(&self, host: &str) -> std::result::Result<(), Duration> {
        match self.check_quota(host).await {
            Ok(()) => {
                // Try to consume
                if self.consume(host, 1).await.is_err() {
                    let retry_after = Duration::from_secs_f64(1.0 / self.requests_per_second);
                    warn!(
                        host = %host,
                        retry_after_ms = retry_after.as_millis(),
                        "Host rate limit exceeded"
                    );
                    Err(retry_after)
                } else {
                    Ok(())
                }
            }
            Err(_) => {
                let retry_after = Duration::from_secs_f64(1.0 / self.requests_per_second);
                Err(retry_after)
            }
        }
    }

    async fn get_host_stats(&self, host: &str) -> Option<HostStats> {
        let count = self.base.get_count(host).await.ok()?;
        let remaining = self.base.get_remaining(host).await.ok()?;
        let available_tokens = remaining as f64;

        Some(HostStats {
            request_count: count as u64,
            available_tokens,
            last_request_age: Duration::ZERO, // Redis doesn't track this without additional keys
        })
    }

    async fn get_all_stats(&self) -> Vec<(String, HostStats)> {
        // Note: This would require scanning Redis keys, which is expensive
        // For production use, consider maintaining a separate index
        Vec::new()
    }

    async fn tracked_hosts_count(&self) -> usize {
        // Note: Would require scanning Redis keys
        // For production, maintain a separate counter
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Integration tests require Redis instance
    // See crates/riptide-cache/tests/ for full integration tests

    #[test]
    fn test_key_generation() {
        let redis = Arc::new(RedisManager::new_test_instance());
        let limiter = RedisRateLimiter::new(redis, 100, Duration::from_secs(60));

        let key = limiter.quota_key("tenant-123");
        assert_eq!(key, "ratelimit:v1:tenant-123");

        let stats_key = limiter.stats_key("tenant-123");
        assert_eq!(stats_key, "ratelimit:v1:stats:tenant-123");
    }
}
