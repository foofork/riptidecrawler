//! Per-host rate limiting with token bucket algorithm.
//!
//! Implements rate limiting on a per-host basis with:
//! - Token bucket algorithm for smooth rate limiting
//! - Configurable burst capacity
//! - Jitter to prevent thundering herd
//! - Automatic cleanup of stale host buckets
//! - Lock-free concurrent access via DashMap

use crate::config::ApiConfig;
use crate::resource_manager::metrics::ResourceMetrics;
use dashmap::DashMap;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use tracing::{debug, warn};

/// Per-host rate limiter with token bucket algorithm
///
/// Each host gets its own token bucket that refills at a configured rate.
/// This prevents any single host from overwhelming the system while allowing
/// burst traffic within configured limits.
///
/// Uses DashMap for lock-free concurrent access, eliminating contention
/// and improving throughput under high load.
pub struct PerHostRateLimiter {
    config: ApiConfig,
    host_buckets: Arc<DashMap<String, HostBucket>>,
    cleanup_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
    metrics: Arc<ResourceMetrics>,
}

/// Rate limiting bucket for a specific host
///
/// Uses token bucket algorithm:
/// - Tokens represent available request capacity
/// - Tokens refill at a constant rate (requests_per_second)
/// - Burst capacity allows temporary spikes
#[derive(Debug, Clone)]
struct HostBucket {
    /// Available tokens (fractional for smooth rate limiting)
    tokens: f64,
    /// Last time tokens were refilled
    last_refill: Instant,
    /// Total requests from this host
    request_count: u64,
    /// Last request timestamp (for cleanup)
    last_request: Instant,
}

impl PerHostRateLimiter {
    /// Create a new rate limiter
    ///
    /// Note: Call `start_cleanup_task()` after wrapping in Arc
    pub(crate) fn new(config: ApiConfig, metrics: Arc<ResourceMetrics>) -> Self {
        debug!(
            rps = config.rate_limiting.requests_per_second_per_host,
            burst = config.rate_limiting.burst_capacity_per_host,
            enabled = config.rate_limiting.enabled,
            "Initializing per-host rate limiter with lock-free DashMap"
        );

        Self {
            config,
            host_buckets: Arc::new(DashMap::new()),
            cleanup_task: Mutex::new(None),
            metrics,
        }
    }

    /// Start background cleanup task for stale host buckets
    ///
    /// Should be called once after the rate limiter is wrapped in Arc.
    /// Removes host buckets that haven't seen requests in over 1 hour.
    pub(crate) async fn start_cleanup_task(self: &Arc<Self>) {
        let rate_limiter = Arc::clone(self);

        let cleanup_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes

            loop {
                interval.tick().await;

                let buckets = &rate_limiter.host_buckets;
                let now = Instant::now();

                let count_before = buckets.len();

                // Collect stale host keys
                let stale_hosts: Vec<String> = buckets
                    .iter()
                    .filter(|entry| {
                        now.duration_since(entry.value().last_request) >= Duration::from_secs(3600)
                    })
                    .map(|entry| entry.key().clone())
                    .collect();

                // Remove stale hosts
                for host in &stale_hosts {
                    buckets.remove(host);
                }

                let count_after = buckets.len();

                if count_before != count_after {
                    debug!(
                        removed = count_before - count_after,
                        remaining = count_after,
                        "Rate limiter cleanup: removed stale host buckets"
                    );
                }
            }
        });

        let mut task = self.cleanup_task.lock().await;
        *task = Some(cleanup_handle);
    }

    /// Check if a request for the given host is allowed
    ///
    /// Returns `Ok(())` if the request is allowed, or an error with retry duration
    /// if rate limited.
    ///
    /// # Arguments
    /// * `host` - The hostname to check rate limit for
    ///
    /// # Returns
    /// * `Ok(())` - Request allowed
    /// * `Err(Duration)` - Rate limited, retry after duration
    pub(crate) async fn check_rate_limit(&self, host: &str) -> std::result::Result<(), Duration> {
        if !self.config.rate_limiting.enabled {
            return Ok(());
        }

        let now = Instant::now();
        let host_key = host.to_string();

        // Get or insert bucket using DashMap's entry API
        let mut entry = self
            .host_buckets
            .entry(host_key.clone())
            .or_insert_with(|| HostBucket {
                tokens: self.config.rate_limiting.requests_per_second_per_host,
                last_refill: now,
                request_count: 0,
                last_request: now,
            });

        let bucket = entry.value_mut();

        // Refill tokens based on time elapsed
        let time_passed = now.duration_since(bucket.last_refill).as_secs_f64();
        let tokens_to_add = time_passed * self.config.rate_limiting.requests_per_second_per_host;
        bucket.tokens = (bucket.tokens + tokens_to_add)
            .min(self.config.rate_limiting.burst_capacity_per_host as f64);
        bucket.last_refill = now;

        // Check if request can be served
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            bucket.request_count += 1;
            bucket.last_request = now;

            // Drop entry before sleeping to release the lock
            drop(entry);

            // Add jitter delay to prevent thundering herd
            let jitter_delay = self.config.calculate_jittered_delay();
            if jitter_delay > Duration::from_millis(1) {
                tokio::time::sleep(jitter_delay).await;
            }

            Ok(())
        } else {
            // Rate limited
            let request_count = bucket.request_count;
            drop(entry); // Release lock before incrementing metrics

            self.metrics
                .rate_limit_hits
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            let retry_after = Duration::from_secs_f64(
                1.0 / self.config.rate_limiting.requests_per_second_per_host,
            );

            warn!(
                host = %host,
                retry_after_ms = retry_after.as_millis(),
                requests = request_count,
                "Rate limit exceeded for host"
            );

            Err(retry_after)
        }
    }

    /// Get statistics for a specific host
    ///
    /// Returns `None` if the host hasn't made any requests yet.
    #[allow(dead_code)] // Reserved for future monitoring API
    pub async fn get_host_stats(&self, host: &str) -> Option<HostStats> {
        self.host_buckets.get(host).map(|bucket| HostStats {
            request_count: bucket.request_count,
            available_tokens: bucket.tokens,
            last_request_age: bucket.last_request.elapsed(),
        })
    }

    /// Get statistics for all hosts
    #[allow(dead_code)] // Reserved for future monitoring API
    pub async fn get_all_stats(&self) -> Vec<(String, HostStats)> {
        self.host_buckets
            .iter()
            .map(|entry| {
                (
                    entry.key().clone(),
                    HostStats {
                        request_count: entry.value().request_count,
                        available_tokens: entry.value().tokens,
                        last_request_age: entry.value().last_request.elapsed(),
                    },
                )
            })
            .collect()
    }

    /// Get total number of hosts being tracked
    #[allow(dead_code)] // Reserved for future monitoring API
    pub async fn tracked_hosts_count(&self) -> usize {
        self.host_buckets.len()
    }
}

/// Statistics for a specific host's rate limiting
#[derive(Debug, Clone)]
pub struct HostStats {
    /// Total requests from this host
    pub request_count: u64,
    /// Currently available tokens
    pub available_tokens: f64,
    /// Time since last request
    pub last_request_age: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RateLimitingConfig;

    fn test_config() -> ApiConfig {
        let mut config = ApiConfig::default();
        config.rate_limiting = RateLimitingConfig {
            enabled: true,
            requests_per_second_per_host: 2.0,
            burst_capacity_per_host: 5,
            jitter_factor: 0.0, // Disable jitter for predictable testing
            window_duration_secs: 60,
            cleanup_interval_secs: 300,
            max_tracked_hosts: 1000,
        };
        config
    }

    #[tokio::test]
    async fn test_rate_limiter_allows_requests() {
        let config = test_config();
        let metrics = Arc::new(ResourceMetrics::new());
        let limiter = Arc::new(PerHostRateLimiter::new(config, metrics));
        limiter.start_cleanup_task().await;

        let host = "example.com";

        // First request should succeed
        assert!(limiter.check_rate_limit(host).await.is_ok());

        // Check stats
        let stats = limiter.get_host_stats(host).await.unwrap();
        assert_eq!(stats.request_count, 1);
    }

    #[tokio::test]
    async fn test_rate_limiter_enforces_limits() {
        let config = test_config();
        let metrics = Arc::new(ResourceMetrics::new());
        let limiter = Arc::new(PerHostRateLimiter::new(config, metrics.clone()));
        limiter.start_cleanup_task().await;

        let host = "example.com";

        // Burst capacity is 5, so first 5 should succeed
        for _ in 0..5 {
            assert!(limiter.check_rate_limit(host).await.is_ok());
        }

        // Next request should be rate limited
        assert!(limiter.check_rate_limit(host).await.is_err());

        // Verify metrics
        assert!(
            metrics
                .rate_limit_hits
                .load(std::sync::atomic::Ordering::Relaxed)
                > 0
        );
    }

    #[tokio::test]
    async fn test_tokens_refill_over_time() {
        let config = test_config();
        let metrics = Arc::new(ResourceMetrics::new());
        let limiter = Arc::new(PerHostRateLimiter::new(config, metrics));
        limiter.start_cleanup_task().await;

        let host = "example.com";

        // Exhaust tokens
        for _ in 0..5 {
            let _ = limiter.check_rate_limit(host).await;
        }

        // Should be rate limited now
        assert!(limiter.check_rate_limit(host).await.is_err());

        // Wait for tokens to refill (2 RPS = 0.5s per token)
        tokio::time::sleep(Duration::from_millis(600)).await;

        // Should work again
        assert!(limiter.check_rate_limit(host).await.is_ok());
    }

    #[tokio::test]
    async fn test_separate_hosts_have_independent_limits() {
        let config = test_config();
        let metrics = Arc::new(ResourceMetrics::new());
        let limiter = Arc::new(PerHostRateLimiter::new(config, metrics));
        limiter.start_cleanup_task().await;

        let host1 = "example.com";
        let host2 = "other.com";

        // Exhaust tokens for host1
        for _ in 0..5 {
            let _ = limiter.check_rate_limit(host1).await;
        }

        // host1 should be rate limited
        assert!(limiter.check_rate_limit(host1).await.is_err());

        // host2 should still work
        assert!(limiter.check_rate_limit(host2).await.is_ok());
    }
}
