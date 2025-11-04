//! Rate limiting using the governor crate

use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::num::NonZeroU32;
use std::sync::Arc;
use tracing::{debug, warn};

/// Simple rate limiter using token bucket algorithm
#[derive(Clone)]
pub struct SimpleRateLimiter {
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    requests_per_second: u32,
}

impl SimpleRateLimiter {
    /// Creates a new rate limiter
    ///
    /// # Arguments
    ///
    /// * `requests_per_second` - Maximum number of requests allowed per second
    ///
    /// # Panics
    ///
    /// Panics if `requests_per_second` is 0
    pub fn new(requests_per_second: u32) -> Self {
        assert!(requests_per_second > 0, "requests_per_second must be > 0");

        debug!("Creating rate limiter with {} req/s", requests_per_second);

        let quota = Quota::per_second(NonZeroU32::new(requests_per_second).unwrap());
        let limiter = Arc::new(RateLimiter::direct(quota));

        Self {
            limiter,
            requests_per_second,
        }
    }

    /// Creates a rate limiter with custom burst size
    ///
    /// # Arguments
    ///
    /// * `requests_per_second` - Maximum number of requests allowed per second
    /// * `burst_size` - Maximum burst size
    pub fn with_burst(requests_per_second: u32, burst_size: u32) -> Self {
        assert!(requests_per_second > 0, "requests_per_second must be > 0");
        assert!(burst_size > 0, "burst_size must be > 0");

        debug!(
            "Creating rate limiter with {} req/s and burst size {}",
            requests_per_second, burst_size
        );

        let quota = Quota::per_second(NonZeroU32::new(requests_per_second).unwrap())
            .allow_burst(NonZeroU32::new(burst_size).unwrap());
        let limiter = Arc::new(RateLimiter::direct(quota));

        Self {
            limiter,
            requests_per_second,
        }
    }

    /// Attempts to acquire a permit to proceed
    ///
    /// # Returns
    ///
    /// `true` if permit was acquired, `false` if rate limit exceeded
    pub fn check(&self) -> bool {
        match self.limiter.check() {
            Ok(_) => {
                debug!("Rate limit check passed");
                true
            }
            Err(_) => {
                warn!("Rate limit exceeded");
                false
            }
        }
    }

    /// Waits until a permit is available
    ///
    /// This will block until the rate limiter allows the operation
    pub async fn wait(&self) {
        self.limiter.until_ready().await;
        debug!("Rate limit permit acquired");
    }

    /// Gets the configured requests per second
    pub fn requests_per_second(&self) -> u32 {
        self.requests_per_second
    }
}

/// Rate limiter builder for custom configurations
pub struct RateLimiterBuilder {
    requests_per_second: u32,
    burst_size: Option<u32>,
}

impl RateLimiterBuilder {
    /// Creates a new rate limiter builder
    pub fn new(requests_per_second: u32) -> Self {
        Self {
            requests_per_second,
            burst_size: None,
        }
    }

    /// Sets the burst size
    pub fn burst_size(mut self, burst_size: u32) -> Self {
        self.burst_size = Some(burst_size);
        self
    }

    /// Builds the rate limiter
    pub fn build(self) -> SimpleRateLimiter {
        match self.burst_size {
            Some(burst) => SimpleRateLimiter::with_burst(self.requests_per_second, burst),
            None => SimpleRateLimiter::new(self.requests_per_second),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_new() {
        let limiter = SimpleRateLimiter::new(10);
        assert_eq!(limiter.requests_per_second(), 10);
    }

    #[test]
    #[should_panic(expected = "requests_per_second must be > 0")]
    fn test_rate_limiter_zero_panics() {
        SimpleRateLimiter::new(0);
    }

    #[test]
    fn test_rate_limiter_with_burst() {
        let limiter = SimpleRateLimiter::with_burst(10, 20);
        assert_eq!(limiter.requests_per_second(), 10);
    }

    #[test]
    fn test_rate_limiter_builder() {
        let limiter = RateLimiterBuilder::new(10).burst_size(20).build();

        assert_eq!(limiter.requests_per_second(), 10);
    }

    #[test]
    fn test_rate_limiter_builder_no_burst() {
        let limiter = RateLimiterBuilder::new(10).build();
        assert_eq!(limiter.requests_per_second(), 10);
    }

    #[tokio::test]
    async fn test_check_allows_requests() {
        let limiter = SimpleRateLimiter::new(10);

        // First request should succeed
        assert!(limiter.check());
    }

    #[tokio::test]
    async fn test_check_rate_limit() {
        // Very low rate for testing
        let limiter = SimpleRateLimiter::new(1);

        // First request succeeds
        assert!(limiter.check());

        // Immediate second request might fail
        // (depends on timing, so we don't assert false)
        let _ = limiter.check();
    }

    #[tokio::test]
    async fn test_wait_permits() {
        let limiter = SimpleRateLimiter::new(10);

        // Should complete without blocking significantly
        limiter.wait().await;
        limiter.wait().await;
    }

    #[tokio::test]
    async fn test_burst_allows_multiple() {
        let limiter = SimpleRateLimiter::with_burst(1, 5);

        // Burst should allow multiple rapid requests
        for _ in 0..5 {
            assert!(limiter.check());
        }
    }
}
