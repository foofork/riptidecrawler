//! Circuit Breaker Presets for Common Use Cases
//!
//! This module provides pre-configured circuit breaker settings optimized
//! for different types of external dependencies and operations.
//!
//! # Available Presets
//!
//! - `http_external()` - For external HTTP APIs and services
//! - `database()` - For database connections and queries
//! - `cache()` - For cache operations (Redis, Memcached, etc.)
//! - `internal_service()` - For internal microservices
//! - `aggressive()` - Fast failure detection for critical paths
//! - `permissive()` - Tolerant settings for flaky services
//!
//! # Example
//!
//! ```rust
//! use riptide_reliability::circuit::presets;
//! use riptide_types::reliability::circuit::{CircuitBreaker, RealClock};
//! use std::sync::Arc;
//!
//! // Use preset for external HTTP API
//! let config = presets::http_external();
//! let cb = CircuitBreaker::new(config, Arc::new(RealClock));
//! ```

use riptide_types::reliability::circuit::Config as CircuitConfig;
use std::time::Duration;

/// Circuit breaker presets for common use cases
pub mod presets {
    use super::*;

    /// Preset for external HTTP services and APIs
    ///
    /// Settings:
    /// - Failure threshold: 5 consecutive failures
    /// - Cooldown: 30 seconds
    /// - Half-open max requests: 3
    ///
    /// Optimized for:
    /// - Third-party HTTP APIs
    /// - External web services
    /// - Remote data sources
    pub fn http_external() -> CircuitConfig {
        CircuitConfig {
            failure_threshold: 5,
            open_cooldown_ms: 30_000, // 30 seconds
            half_open_max_in_flight: 3,
        }
    }

    /// Preset for database operations
    ///
    /// Settings:
    /// - Failure threshold: 3 consecutive failures
    /// - Cooldown: 60 seconds
    /// - Half-open max requests: 2
    ///
    /// Optimized for:
    /// - PostgreSQL, MySQL connections
    /// - Database query operations
    /// - Connection pool exhaustion
    pub fn database() -> CircuitConfig {
        CircuitConfig {
            failure_threshold: 3,
            open_cooldown_ms: 60_000, // 60 seconds
            half_open_max_in_flight: 2,
        }
    }

    /// Preset for cache operations
    ///
    /// Settings:
    /// - Failure threshold: 10 consecutive failures
    /// - Cooldown: 10 seconds
    /// - Half-open max requests: 5
    ///
    /// Optimized for:
    /// - Redis operations
    /// - Memcached operations
    /// - In-memory cache failures
    ///
    /// More permissive because cache failures should degrade gracefully
    pub fn cache() -> CircuitConfig {
        CircuitConfig {
            failure_threshold: 10,
            open_cooldown_ms: 10_000, // 10 seconds
            half_open_max_in_flight: 5,
        }
    }

    /// Preset for internal microservice calls
    ///
    /// Settings:
    /// - Failure threshold: 5 consecutive failures
    /// - Cooldown: 15 seconds
    /// - Half-open max requests: 3
    ///
    /// Optimized for:
    /// - Internal service-to-service communication
    /// - Kubernetes pod communication
    /// - Docker container communication
    pub fn internal_service() -> CircuitConfig {
        CircuitConfig {
            failure_threshold: 5,
            open_cooldown_ms: 15_000, // 15 seconds
            half_open_max_in_flight: 3,
        }
    }

    /// Aggressive preset for critical paths requiring fast failure detection
    ///
    /// Settings:
    /// - Failure threshold: 2 consecutive failures
    /// - Cooldown: 5 seconds
    /// - Half-open max requests: 1
    ///
    /// Optimized for:
    /// - Critical user-facing operations
    /// - Payment processing
    /// - Authentication services
    pub fn aggressive() -> CircuitConfig {
        CircuitConfig {
            failure_threshold: 2,
            open_cooldown_ms: 5_000, // 5 seconds
            half_open_max_in_flight: 1,
        }
    }

    /// Permissive preset for flaky but eventually consistent services
    ///
    /// Settings:
    /// - Failure threshold: 20 consecutive failures
    /// - Cooldown: 120 seconds
    /// - Half-open max requests: 10
    ///
    /// Optimized for:
    /// - Legacy systems with intermittent issues
    /// - Services with known flakiness
    /// - Non-critical background jobs
    pub fn permissive() -> CircuitConfig {
        CircuitConfig {
            failure_threshold: 20,
            open_cooldown_ms: 120_000, // 120 seconds
            half_open_max_in_flight: 10,
        }
    }

    /// Custom preset builder for specific requirements
    ///
    /// # Example
    ///
    /// ```rust
    /// use riptide_reliability::circuit::presets;
    ///
    /// let config = presets::custom()
    ///     .failure_threshold(7)
    ///     .cooldown_seconds(45)
    ///     .half_open_requests(4)
    ///     .build();
    /// ```
    pub fn custom() -> CircuitConfigBuilder {
        CircuitConfigBuilder::default()
    }
}

/// Builder for custom circuit breaker configurations
#[derive(Debug, Clone)]
pub struct CircuitConfigBuilder {
    failure_threshold: u32,
    open_cooldown_ms: u64,
    half_open_max_in_flight: u32,
}

impl Default for CircuitConfigBuilder {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            open_cooldown_ms: 30_000,
            half_open_max_in_flight: 3,
        }
    }
}

impl CircuitConfigBuilder {
    /// Set the number of failures before opening the circuit
    pub fn failure_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = threshold;
        self
    }

    /// Set the cooldown period in seconds
    pub fn cooldown_seconds(mut self, seconds: u64) -> Self {
        self.open_cooldown_ms = seconds * 1000;
        self
    }

    /// Set the cooldown period in milliseconds
    pub fn cooldown_ms(mut self, ms: u64) -> Self {
        self.open_cooldown_ms = ms;
        self
    }

    /// Set the cooldown period as Duration
    pub fn cooldown(mut self, duration: Duration) -> Self {
        self.open_cooldown_ms = duration.as_millis() as u64;
        self
    }

    /// Set the maximum number of requests allowed in half-open state
    pub fn half_open_requests(mut self, max_requests: u32) -> Self {
        self.half_open_max_in_flight = max_requests;
        self
    }

    /// Build the circuit breaker configuration
    pub fn build(self) -> CircuitConfig {
        CircuitConfig {
            failure_threshold: self.failure_threshold,
            open_cooldown_ms: self.open_cooldown_ms,
            half_open_max_in_flight: self.half_open_max_in_flight,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_external_preset() {
        let config = presets::http_external();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.open_cooldown_ms, 30_000);
        assert_eq!(config.half_open_max_in_flight, 3);
    }

    #[test]
    fn test_database_preset() {
        let config = presets::database();
        assert_eq!(config.failure_threshold, 3);
        assert_eq!(config.open_cooldown_ms, 60_000);
        assert_eq!(config.half_open_max_in_flight, 2);
    }

    #[test]
    fn test_cache_preset() {
        let config = presets::cache();
        assert_eq!(config.failure_threshold, 10);
        assert_eq!(config.open_cooldown_ms, 10_000);
        assert_eq!(config.half_open_max_in_flight, 5);
    }

    #[test]
    fn test_internal_service_preset() {
        let config = presets::internal_service();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.open_cooldown_ms, 15_000);
        assert_eq!(config.half_open_max_in_flight, 3);
    }

    #[test]
    fn test_aggressive_preset() {
        let config = presets::aggressive();
        assert_eq!(config.failure_threshold, 2);
        assert_eq!(config.open_cooldown_ms, 5_000);
        assert_eq!(config.half_open_max_in_flight, 1);
    }

    #[test]
    fn test_permissive_preset() {
        let config = presets::permissive();
        assert_eq!(config.failure_threshold, 20);
        assert_eq!(config.open_cooldown_ms, 120_000);
        assert_eq!(config.half_open_max_in_flight, 10);
    }

    #[test]
    fn test_custom_builder() {
        let config = presets::custom()
            .failure_threshold(7)
            .cooldown_seconds(45)
            .half_open_requests(4)
            .build();

        assert_eq!(config.failure_threshold, 7);
        assert_eq!(config.open_cooldown_ms, 45_000);
        assert_eq!(config.half_open_max_in_flight, 4);
    }

    #[test]
    fn test_custom_builder_with_duration() {
        let config = presets::custom().cooldown(Duration::from_secs(90)).build();

        assert_eq!(config.open_cooldown_ms, 90_000);
    }
}
