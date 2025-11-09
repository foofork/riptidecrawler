//! Reliability configuration types for Riptide
//!
//! This module contains configuration structs for reliability patterns like
//! circuit breakers and retry logic. These types are shared across multiple
//! crates to avoid circular dependencies.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Circuit breaker configuration
///
/// Controls when the circuit breaker trips to prevent cascading failures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening the circuit
    pub failure_threshold: u32,
    /// Cooldown period in milliseconds before attempting to close the circuit
    pub open_cooldown_ms: u64,
    /// Maximum concurrent requests allowed in half-open state
    pub half_open_max_in_flight: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            open_cooldown_ms: 30_000,
            half_open_max_in_flight: 3,
        }
    }
}

/// Retry configuration with exponential backoff
///
/// Defines how retries should be performed for failed operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    #[serde(with = "serde_duration")]
    pub initial_delay: Duration,
    /// Maximum delay between retries
    #[serde(with = "serde_duration")]
    pub max_delay: Duration,
    /// Backoff multiplier (e.g., 2.0 for exponential backoff)
    pub backoff_multiplier: f64,
    /// Add jitter to prevent thundering herd
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Custom serde module for Duration serialization
mod serde_duration {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}
