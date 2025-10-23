//! Timeout profile types and statistics

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::debug;

/// Timeout configuration limits
pub const MIN_TIMEOUT_SECS: u64 = 5;
pub const MAX_TIMEOUT_SECS: u64 = 60;
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
pub const BACKOFF_MULTIPLIER: f64 = 1.5;
pub const SUCCESS_REDUCTION: f64 = 0.9;

/// Timeout profile for a specific domain
///
/// Tracks historical performance and adapts timeout values based on
/// success/failure patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutProfile {
    /// Domain name
    pub domain: String,
    /// Current timeout in seconds
    pub timeout_secs: u64,
    /// Total requests made
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests (timeouts)
    pub failed_requests: u64,
    /// Consecutive successes
    pub consecutive_successes: u32,
    /// Consecutive failures
    pub consecutive_failures: u32,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Last update timestamp
    pub last_updated: u64,
}

impl TimeoutProfile {
    /// Create a new timeout profile for a domain
    pub(crate) fn new(domain: String) -> Self {
        Self {
            domain,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            consecutive_successes: 0,
            consecutive_failures: 0,
            avg_response_time_ms: 0.0,
            last_updated: Self::current_timestamp(),
        }
    }

    /// Update profile after successful request
    pub(crate) fn record_success(&mut self, response_time: Duration) {
        self.total_requests += 1;
        self.successful_requests += 1;
        self.consecutive_successes += 1;
        self.consecutive_failures = 0;
        self.last_updated = Self::current_timestamp();

        // Update average response time using exponential moving average
        let response_ms = response_time.as_millis() as f64;
        if self.avg_response_time_ms == 0.0 {
            self.avg_response_time_ms = response_ms;
        } else {
            // Exponential moving average (80% old, 20% new)
            self.avg_response_time_ms = 0.8 * self.avg_response_time_ms + 0.2 * response_ms;
        }

        // Reduce timeout after 3 consecutive successes
        if self.consecutive_successes >= 3 {
            let new_timeout = (self.timeout_secs as f64 * SUCCESS_REDUCTION) as u64;
            self.timeout_secs = new_timeout.max(MIN_TIMEOUT_SECS);
            self.consecutive_successes = 0;

            debug!(
                domain = &self.domain,
                new_timeout = self.timeout_secs,
                "Reduced timeout after consecutive successes"
            );
        }
    }

    /// Update profile after timeout failure
    pub(crate) fn record_timeout(&mut self) {
        self.total_requests += 1;
        self.failed_requests += 1;
        self.consecutive_failures += 1;
        self.consecutive_successes = 0;
        self.last_updated = Self::current_timestamp();

        // Increase timeout with exponential backoff
        let new_timeout = (self.timeout_secs as f64 * BACKOFF_MULTIPLIER) as u64;
        self.timeout_secs = new_timeout.min(MAX_TIMEOUT_SECS);

        tracing::warn!(
            domain = &self.domain,
            new_timeout = self.timeout_secs,
            consecutive_failures = self.consecutive_failures,
            "Increased timeout after failure"
        );
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.successful_requests as f64 / self.total_requests as f64) * 100.0
    }

    /// Get current Unix timestamp in seconds
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

/// Timeout statistics aggregated across all profiles
#[derive(Debug, Clone)]
pub struct TimeoutStats {
    /// Total number of domains tracked
    pub total_domains: usize,
    /// Total requests across all domains
    pub total_requests: u64,
    /// Total successful requests
    pub total_successes: u64,
    /// Total failed requests
    pub total_failures: u64,
    /// Average timeout across all domains
    pub avg_timeout_secs: f64,
    /// Average success rate across all domains
    pub avg_success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_creation() {
        let profile = TimeoutProfile::new("example.com".to_string());
        assert_eq!(profile.domain, "example.com");
        assert_eq!(profile.timeout_secs, DEFAULT_TIMEOUT_SECS);
        assert_eq!(profile.total_requests, 0);
    }

    #[test]
    fn test_record_success() {
        let mut profile = TimeoutProfile::new("example.com".to_string());
        profile.record_success(Duration::from_millis(500));

        assert_eq!(profile.successful_requests, 1);
        assert_eq!(profile.failed_requests, 0);
        assert_eq!(profile.consecutive_successes, 1);
        assert_eq!(profile.avg_response_time_ms, 500.0);
    }

    #[test]
    fn test_record_timeout() {
        let mut profile = TimeoutProfile::new("slow-site.com".to_string());
        let initial_timeout = profile.timeout_secs;

        profile.record_timeout();

        assert_eq!(profile.failed_requests, 1);
        assert_eq!(profile.consecutive_failures, 1);
        assert!(profile.timeout_secs > initial_timeout);
    }

    #[test]
    fn test_adaptive_timeout_reduction() {
        let mut profile = TimeoutProfile::new("fast-site.com".to_string());
        let initial_timeout = profile.timeout_secs;

        // Record 4 consecutive successes (should trigger reduction after 3)
        for _ in 0..4 {
            profile.record_success(Duration::from_millis(100));
        }

        assert!(profile.timeout_secs < initial_timeout);
        assert!(profile.timeout_secs >= MIN_TIMEOUT_SECS);
    }

    #[test]
    fn test_timeout_bounds() {
        let mut profile = TimeoutProfile::new("example.com".to_string());

        // Test max timeout
        profile.timeout_secs = MAX_TIMEOUT_SECS - 1;
        profile.record_timeout();
        assert_eq!(profile.timeout_secs, MAX_TIMEOUT_SECS);

        // Test min timeout
        profile.timeout_secs = MIN_TIMEOUT_SECS + 1;
        for _ in 0..10 {
            profile.record_success(Duration::from_millis(50));
        }
        assert!(profile.timeout_secs >= MIN_TIMEOUT_SECS);
    }

    #[test]
    fn test_success_rate() {
        let mut profile = TimeoutProfile::new("example.com".to_string());

        profile.record_success(Duration::from_secs(1));
        profile.record_success(Duration::from_secs(1));
        profile.record_timeout();

        // Use approximate comparison for floating point
        let rate = profile.success_rate();
        assert!((rate - 66.67).abs() < 0.1, "Expected ~66.67%, got {}", rate);
    }

    #[test]
    fn test_exponential_moving_average() {
        let mut profile = TimeoutProfile::new("example.com".to_string());

        profile.record_success(Duration::from_millis(1000));
        assert_eq!(profile.avg_response_time_ms, 1000.0);

        profile.record_success(Duration::from_millis(500));
        // 0.8 * 1000 + 0.2 * 500 = 900
        assert_eq!(profile.avg_response_time_ms, 900.0);
    }
}
