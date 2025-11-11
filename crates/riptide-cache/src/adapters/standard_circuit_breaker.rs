//! Standard circuit breaker adapter
//!
//! Adapts the lock-free circuit breaker from `riptide-utils` to implement
//! the `CircuitBreaker` port trait from `riptide-types`.
//!
//! # Architecture
//!
//! This adapter wraps `riptide_utils::circuit_breaker::CircuitBreaker` to provide
//! a uniform interface for dependency injection across the system.
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_cache::adapters::StandardCircuitBreakerAdapter;
//! use riptide_types::ports::{CircuitBreaker, CircuitBreakerConfig};
//!
//! let config = CircuitBreakerConfig::default();
//! let adapter: Arc<dyn CircuitBreaker> = StandardCircuitBreakerAdapter::new(config);
//!
//! // Use through port trait
//! adapter.try_call().await?;
//! ```

use async_trait::async_trait;
use riptide_types::error::{Result as RiptideResult, RiptideError};
use riptide_types::ports::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerStats, CircuitState,
};
use riptide_utils::circuit_breaker::{
    CircuitBreaker as UtilsCircuitBreaker, Config as UtilsConfig, RealClock, State as UtilsState,
};
use std::fmt;
use std::sync::Arc;

/// Standard circuit breaker adapter wrapping riptide-utils implementation
///
/// Provides thread-safe, lock-free circuit breaker functionality through
/// the `CircuitBreaker` port trait interface.
pub struct StandardCircuitBreakerAdapter {
    inner: Arc<UtilsCircuitBreaker>,
}

impl StandardCircuitBreakerAdapter {
    /// Create a new standard circuit breaker adapter
    ///
    /// # Arguments
    ///
    /// * `config` - Circuit breaker configuration
    ///
    /// # Returns
    ///
    /// Arc-wrapped adapter implementing the `CircuitBreaker` trait
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let config = CircuitBreakerConfig::default();
    /// let adapter = StandardCircuitBreakerAdapter::new(config);
    /// ```
    pub fn new(config: CircuitBreakerConfig) -> Arc<Self> {
        let utils_config = UtilsConfig {
            failure_threshold: config.failure_threshold,
            open_cooldown_ms: config.recovery_timeout.as_millis() as u64,
            half_open_max_in_flight: config.half_open_max_requests,
        };

        let inner = UtilsCircuitBreaker::new(utils_config, Arc::new(RealClock));

        Arc::new(Self { inner })
    }

    /// Create adapter with custom clock (for testing)
    #[cfg(test)]
    pub fn with_clock(
        config: CircuitBreakerConfig,
        clock: Arc<dyn riptide_utils::circuit_breaker::Clock>,
    ) -> Arc<Self> {
        let utils_config = UtilsConfig {
            failure_threshold: config.failure_threshold,
            open_cooldown_ms: config.recovery_timeout.as_millis() as u64,
            half_open_max_in_flight: config.half_open_max_requests,
        };

        let inner = UtilsCircuitBreaker::new(utils_config, clock);

        Arc::new(Self { inner })
    }

    /// Convert internal state to port trait state
    fn convert_state(state: UtilsState) -> CircuitState {
        match state {
            UtilsState::Closed => CircuitState::Closed,
            UtilsState::Open => CircuitState::Open,
            UtilsState::HalfOpen => CircuitState::HalfOpen,
        }
    }
}

impl fmt::Debug for StandardCircuitBreakerAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StandardCircuitBreakerAdapter")
            .field("state", &self.inner.state())
            .finish()
    }
}

#[async_trait]
impl CircuitBreaker for StandardCircuitBreakerAdapter {
    async fn state(&self) -> CircuitState {
        Self::convert_state(self.inner.state())
    }

    async fn try_call(&self) -> RiptideResult<()> {
        match self.inner.try_acquire() {
            Ok(_permit) => Ok(()),
            Err(_) => Err(RiptideError::CircuitBreakerOpen(
                "Circuit breaker is open, rejecting request".to_string(),
            )),
        }
    }

    async fn on_success(&self) {
        self.inner.on_success();
    }

    async fn on_failure(&self) {
        self.inner.on_failure();
    }

    async fn stats(&self) -> RiptideResult<CircuitBreakerStats> {
        // Note: The riptide-utils circuit breaker doesn't expose detailed stats
        // We provide basic stats based on state
        let state = Self::convert_state(self.inner.state());

        Ok(CircuitBreakerStats {
            state,
            // Note: These counters are not exposed by riptide-utils implementation
            // In a production system, you'd want to track these in the adapter
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            circuit_opens: 0,
            current_failures: 0,
            success_rate: if state == CircuitState::Closed {
                1.0
            } else {
                0.0
            },
        })
    }

    async fn reset(&self) -> RiptideResult<()> {
        // Note: riptide-utils circuit breaker doesn't have a reset method
        // This would require internal state access or a new method
        // For now, we return an error
        Err(RiptideError::NotImplemented(
            "Reset not supported by standard circuit breaker".to_string(),
        ))
    }

    async fn is_call_permitted(&self) -> bool {
        self.try_call().await.is_ok()
    }

    async fn health_status(&self) -> String {
        let state = Self::convert_state(self.inner.state());
        format!("State: {:?}", state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_adapter_creation() {
        let config = CircuitBreakerConfig::default();
        let adapter = StandardCircuitBreakerAdapter::new(config);

        // Initially closed
        assert_eq!(adapter.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_state_transitions() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: Duration::from_secs(1),
            half_open_max_requests: 2,
            success_rate_threshold: None,
            failure_window: None,
        };

        let adapter = StandardCircuitBreakerAdapter::new(config);

        // Initially closed
        assert_eq!(adapter.state().await, CircuitState::Closed);

        // Trigger failures to open circuit
        adapter.on_failure().await;
        adapter.on_failure().await;
        adapter.on_failure().await;

        // Should be open now
        assert_eq!(adapter.state().await, CircuitState::Open);

        // Try call should fail
        assert!(adapter.try_call().await.is_err());
    }

    #[tokio::test]
    async fn test_call_permitted() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout: Duration::from_secs(1),
            half_open_max_requests: 1,
            success_rate_threshold: None,
            failure_window: None,
        };

        let adapter = StandardCircuitBreakerAdapter::new(config);

        // Initially permitted
        assert!(adapter.is_call_permitted().await);

        // Trigger failures
        adapter.on_failure().await;
        adapter.on_failure().await;

        // Should not be permitted
        assert!(!adapter.is_call_permitted().await);
    }

    #[tokio::test]
    async fn test_health_status() {
        let config = CircuitBreakerConfig::default();
        let adapter = StandardCircuitBreakerAdapter::new(config);

        let status = adapter.health_status().await;
        assert!(status.contains("Closed"));
    }

    #[tokio::test]
    async fn test_stats() {
        let config = CircuitBreakerConfig::default();
        let adapter = StandardCircuitBreakerAdapter::new(config);

        let stats = adapter.stats().await.unwrap();
        assert_eq!(stats.state, CircuitState::Closed);
    }
}
