//! Per-domain rate limiting with adaptive throttling and exponential backoff.
//!
//! This module implements intelligent rate limiting that:
//! - Tracks state per domain (isolated multi-target scraping)
//! - Adapts speed based on success/failure patterns
//! - Applies exponential backoff on 429/503 errors
//! - Speeds up on consecutive successes
//! - Integrates with StealthController for seamless operation

use crate::config::DomainTiming;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Per-domain rate limiter with adaptive throttling
///
/// Each domain maintains independent state with:
/// - Token bucket for smooth rate limiting
/// - Success/failure tracking for adaptive behavior
/// - Exponential backoff on errors
/// - Automatic cleanup of stale domains
pub struct RateLimiter {
    /// Per-domain rate limiting state
    domain_state: Arc<DashMap<String, DomainState>>,
    /// Default timing configuration
    default_timing: DomainTiming,
}

/// Rate limiting state for a specific domain
#[derive(Debug, Clone)]
struct DomainState {
    /// Available tokens for requests (fractional for smooth limiting)
    tokens: f64,
    /// Last time tokens were refilled
    last_refill: Instant,
    /// Total successful requests to this domain
    success_count: u64,
    /// Total failed requests to this domain
    failure_count: u64,
    /// Consecutive successful requests (resets on failure)
    consecutive_successes: u32,
    /// Consecutive failures (resets on success)
    consecutive_failures: u32,
    /// Current backoff delay (doubles on each failure)
    current_backoff: Duration,
    /// Last request timestamp
    last_request: Instant,
    /// Current delay multiplier (1.0 = normal, < 1.0 = faster, > 1.0 = slower)
    delay_multiplier: f64,
}

impl DomainState {
    /// Create new domain state with default timing
    fn new(timing: &DomainTiming) -> Self {
        Self {
            tokens: timing.burst_size as f64,
            last_refill: Instant::now(),
            success_count: 0,
            failure_count: 0,
            consecutive_successes: 0,
            consecutive_failures: 0,
            current_backoff: Duration::from_millis(timing.min_delay_ms),
            last_request: Instant::now(),
            delay_multiplier: 1.0,
        }
    }

    /// Record successful request and adapt speed
    fn record_success(&mut self, timing: &DomainTiming) {
        self.success_count += 1;
        self.consecutive_successes += 1;
        self.consecutive_failures = 0;

        // Reset backoff on success
        self.current_backoff = Duration::from_millis(timing.min_delay_ms);

        // Speed up after multiple consecutive successes
        if self.consecutive_successes >= 5 {
            self.delay_multiplier = (self.delay_multiplier * 0.9).max(0.5); // Speed up, but not more than 2x
            debug!(
                multiplier = self.delay_multiplier,
                consecutive = self.consecutive_successes,
                "Adaptive rate limiting: speeding up after consecutive successes"
            );
        }
    }

    /// Record failed request and apply exponential backoff
    fn record_failure(&mut self, timing: &DomainTiming, is_rate_limit_error: bool) {
        self.failure_count += 1;
        self.consecutive_failures += 1;
        self.consecutive_successes = 0;

        if is_rate_limit_error {
            // Exponential backoff for rate limit errors (429, 503)
            self.current_backoff =
                (self.current_backoff * 2).min(Duration::from_millis(timing.max_delay_ms));

            // Also slow down future requests
            self.delay_multiplier = (self.delay_multiplier * 1.5).min(3.0); // Slow down, but not more than 3x

            warn!(
                backoff_ms = self.current_backoff.as_millis(),
                multiplier = self.delay_multiplier,
                consecutive = self.consecutive_failures,
                "Rate limit detected: applying exponential backoff"
            );
        } else {
            // Regular failure - less aggressive backoff
            self.current_backoff =
                (self.current_backoff.mul_f32(1.2)).min(Duration::from_millis(timing.max_delay_ms));

            self.delay_multiplier = (self.delay_multiplier * 1.1).min(2.0);
        }
    }

    /// Get current delay with adaptive multiplier
    fn calculate_delay(&self, timing: &DomainTiming) -> Duration {
        let base_delay = Duration::from_millis((timing.min_delay_ms + timing.max_delay_ms) / 2);

        let adaptive_delay = base_delay.mul_f64(self.delay_multiplier);

        // Ensure delay is within configured bounds
        adaptive_delay
            .max(Duration::from_millis(timing.min_delay_ms))
            .min(Duration::from_millis(timing.max_delay_ms))
    }
}

impl RateLimiter {
    /// Create a new rate limiter with default timing
    pub fn new(default_timing: DomainTiming) -> Self {
        debug!(
            min_delay = default_timing.min_delay_ms,
            max_delay = default_timing.max_delay_ms,
            burst = default_timing.burst_size,
            rpm_limit = ?default_timing.rpm_limit,
            "Initializing per-domain rate limiter with adaptive throttling"
        );

        Self {
            domain_state: Arc::new(DashMap::new()),
            default_timing,
        }
    }

    /// Check if a request for the given domain is allowed
    ///
    /// Returns `Ok(delay)` if the request is allowed after the delay,
    /// or `Err(retry_after)` if rate limited.
    pub async fn check_rate_limit(
        &self,
        domain: &str,
        timing: Option<&DomainTiming>,
    ) -> Result<Duration, Duration> {
        let timing = timing.unwrap_or(&self.default_timing);
        let now = Instant::now();

        // Get or create domain state
        let mut entry = self
            .domain_state
            .entry(domain.to_string())
            .or_insert_with(|| DomainState::new(timing));

        let state = entry.value_mut();

        // Refill tokens based on time elapsed
        let time_passed = now.duration_since(state.last_refill).as_secs_f64();

        // Calculate tokens per second based on RPM limit
        let tokens_per_second = if let Some(rpm) = timing.rpm_limit {
            rpm as f64 / 60.0
        } else {
            // Default to burst_size requests per (max_delay * burst_size) window
            timing.burst_size as f64 / (timing.max_delay_ms as f64 / 1000.0)
        };

        let tokens_to_add = time_passed * tokens_per_second;
        state.tokens = (state.tokens + tokens_to_add).min(timing.burst_size as f64);
        state.last_refill = now;

        // Check if we have tokens available
        if state.tokens >= 1.0 {
            state.tokens -= 1.0;
            state.last_request = now;

            // Calculate delay with adaptive multiplier
            let delay = state.calculate_delay(timing);

            // If in backoff mode, use the larger of adaptive delay or current backoff
            let final_delay = delay.max(state.current_backoff);

            // Copy values we need after drop
            let tokens = state.tokens;
            let multiplier = state.delay_multiplier;

            drop(entry); // Release lock

            debug!(
                domain = %domain,
                delay_ms = final_delay.as_millis(),
                tokens = tokens,
                multiplier = multiplier,
                "Request allowed with adaptive delay"
            );

            Ok(final_delay)
        } else {
            // Rate limited - calculate retry delay
            let retry_after = Duration::from_secs_f64(1.0 / tokens_per_second);

            warn!(
                domain = %domain,
                retry_after_ms = retry_after.as_millis(),
                tokens = state.tokens,
                "Rate limit exceeded for domain"
            );

            drop(entry); // Release lock
            Err(retry_after)
        }
    }

    /// Record the result of a request for adaptive throttling
    ///
    /// Call this after each request completes to enable adaptive behavior.
    pub fn record_request_result(&self, domain: &str, success: bool, is_rate_limit_error: bool) {
        if let Some(mut entry) = self.domain_state.get_mut(domain) {
            if success {
                entry.record_success(&self.default_timing);
            } else {
                entry.record_failure(&self.default_timing, is_rate_limit_error);
            }
        }
    }

    /// Get statistics for a specific domain
    pub fn get_domain_stats(&self, domain: &str) -> Option<DomainStats> {
        self.domain_state.get(domain).map(|state| DomainStats {
            success_count: state.success_count,
            failure_count: state.failure_count,
            consecutive_successes: state.consecutive_successes,
            consecutive_failures: state.consecutive_failures,
            current_backoff_ms: state.current_backoff.as_millis() as u64,
            delay_multiplier: state.delay_multiplier,
            available_tokens: state.tokens,
            last_request_age: state.last_request.elapsed(),
        })
    }

    /// Get statistics for all tracked domains
    pub fn get_all_stats(&self) -> Vec<(String, DomainStats)> {
        self.domain_state
            .iter()
            .map(|entry| {
                (
                    entry.key().clone(),
                    DomainStats {
                        success_count: entry.value().success_count,
                        failure_count: entry.value().failure_count,
                        consecutive_successes: entry.value().consecutive_successes,
                        consecutive_failures: entry.value().consecutive_failures,
                        current_backoff_ms: entry.value().current_backoff.as_millis() as u64,
                        delay_multiplier: entry.value().delay_multiplier,
                        available_tokens: entry.value().tokens,
                        last_request_age: entry.value().last_request.elapsed(),
                    },
                )
            })
            .collect()
    }

    /// Clean up stale domain state (domains not accessed in over 1 hour)
    pub fn cleanup_stale_domains(&self) {
        let now = Instant::now();
        let stale_threshold = Duration::from_secs(3600); // 1 hour

        let stale_domains: Vec<String> = self
            .domain_state
            .iter()
            .filter(|entry| now.duration_since(entry.value().last_request) >= stale_threshold)
            .map(|entry| entry.key().clone())
            .collect();

        for domain in &stale_domains {
            self.domain_state.remove(domain);
        }

        if !stale_domains.is_empty() {
            debug!(
                removed = stale_domains.len(),
                remaining = self.domain_state.len(),
                "Cleaned up stale domain state"
            );
        }
    }

    /// Get total number of domains being tracked
    pub fn tracked_domains_count(&self) -> usize {
        self.domain_state.len()
    }
}

/// Statistics for a specific domain's rate limiting
#[derive(Debug, Clone)]
pub struct DomainStats {
    /// Total successful requests
    pub success_count: u64,
    /// Total failed requests
    pub failure_count: u64,
    /// Consecutive successful requests
    pub consecutive_successes: u32,
    /// Consecutive failures
    pub consecutive_failures: u32,
    /// Current backoff delay in milliseconds
    pub current_backoff_ms: u64,
    /// Current delay multiplier (1.0 = normal)
    pub delay_multiplier: f64,
    /// Currently available tokens
    pub available_tokens: f64,
    /// Time since last request
    pub last_request_age: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_timing() -> DomainTiming {
        DomainTiming {
            min_delay_ms: 100,
            max_delay_ms: 1000,
            rpm_limit: Some(60), // 60 requests per minute = 1 per second
            burst_size: 5,
        }
    }

    #[tokio::test]
    async fn test_rate_limiting_per_domain() {
        let limiter = RateLimiter::new(test_timing());
        let domain1 = "example.com";
        let domain2 = "other.com";

        // Both domains should have independent limits
        for _ in 0..5 {
            assert!(limiter.check_rate_limit(domain1, None).await.is_ok());
            assert!(limiter.check_rate_limit(domain2, None).await.is_ok());
        }

        // Both should be rate limited now
        assert!(limiter.check_rate_limit(domain1, None).await.is_err());
        assert!(limiter.check_rate_limit(domain2, None).await.is_err());
    }

    #[tokio::test]
    async fn test_adaptive_rate_limiting() {
        let limiter = RateLimiter::new(test_timing());
        let domain = "example.com";

        // Make successful requests
        for _ in 0..6 {
            let _ = limiter.check_rate_limit(domain, None).await;
            limiter.record_request_result(domain, true, false);
        }

        // Check that delay multiplier decreased (sped up)
        let stats = limiter.get_domain_stats(domain).unwrap();
        assert!(
            stats.delay_multiplier < 1.0,
            "Should speed up after successes"
        );
        assert_eq!(stats.consecutive_successes, 6);
    }

    #[tokio::test]
    async fn test_exponential_backoff() {
        let limiter = RateLimiter::new(test_timing());
        let domain = "example.com";

        // Record initial backoff
        let _ = limiter.check_rate_limit(domain, None).await;
        let initial_backoff = limiter.get_domain_stats(domain).unwrap().current_backoff_ms;

        // Record rate limit error
        limiter.record_request_result(domain, false, true);
        let after_one_error = limiter.get_domain_stats(domain).unwrap().current_backoff_ms;

        // Record another rate limit error
        limiter.record_request_result(domain, false, true);
        let after_two_errors = limiter.get_domain_stats(domain).unwrap().current_backoff_ms;

        // Backoff should increase exponentially
        assert!(after_one_error > initial_backoff);
        assert!(after_two_errors > after_one_error);
        assert_eq!(
            limiter
                .get_domain_stats(domain)
                .unwrap()
                .consecutive_failures,
            2
        );
    }

    #[tokio::test]
    async fn test_domain_isolation() {
        let limiter = RateLimiter::new(test_timing());
        let domain1 = "slow.com";
        let domain2 = "fast.com";

        // Make domain1 encounter rate limits
        for _ in 0..3 {
            let _ = limiter.check_rate_limit(domain1, None).await;
            limiter.record_request_result(domain1, false, true);
        }

        // domain1 should have high backoff
        let stats1 = limiter.get_domain_stats(domain1).unwrap();
        assert!(stats1.current_backoff_ms > 100);
        assert!(stats1.delay_multiplier > 1.0);

        // domain2 should be unaffected
        let _ = limiter.check_rate_limit(domain2, None).await;
        let stats2 = limiter.get_domain_stats(domain2).unwrap();
        assert_eq!(stats2.current_backoff_ms, 100); // Initial backoff
        assert_eq!(stats2.delay_multiplier, 1.0); // No slowdown
    }

    #[tokio::test]
    async fn test_tokens_refill_over_time() {
        let limiter = RateLimiter::new(test_timing());
        let domain = "example.com";

        // Exhaust tokens
        for _ in 0..5 {
            let _ = limiter.check_rate_limit(domain, None).await;
        }

        // Should be rate limited
        assert!(limiter.check_rate_limit(domain, None).await.is_err());

        // Wait for tokens to refill (1 RPS = 1 second per token)
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // Should work again
        assert!(limiter.check_rate_limit(domain, None).await.is_ok());
    }

    #[tokio::test]
    async fn test_cleanup_stale_domains() {
        let limiter = RateLimiter::new(test_timing());

        // Create some domain state
        let _ = limiter.check_rate_limit("example.com", None).await;
        let _ = limiter.check_rate_limit("other.com", None).await;

        assert_eq!(limiter.tracked_domains_count(), 2);

        // Cleanup won't remove fresh entries
        limiter.cleanup_stale_domains();
        assert_eq!(limiter.tracked_domains_count(), 2);
    }
}
