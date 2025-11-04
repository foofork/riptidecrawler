//! Circuit breaker implementation for fault tolerance
//!
//! Prevents cascading failures by opening the circuit after repeated failures
//! and allowing gradual recovery through half-open state.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, warn};

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are rejected
    Open {
        /// Time when circuit was opened
        opened_at: Instant,
        /// Number of failures that triggered opening
        failure_count: u32,
    },
    /// Circuit is half-open, testing if service recovered
    HalfOpen {
        /// Number of successful requests in half-open state
        success_count: u32,
    },
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,
    /// Duration to wait before transitioning from Open to HalfOpen
    pub timeout: Duration,
    /// Number of successful requests in HalfOpen before closing
    pub success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout: Duration::from_secs(60),
            success_threshold: 2,
        }
    }
}

/// Circuit breaker for protecting services from cascading failures
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<Mutex<CircuitBreakerState>>,
}

#[derive(Debug)]
struct CircuitBreakerState {
    circuit_state: CircuitState,
    failure_count: u32,
    success_count: u32,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with default config
    pub fn new() -> Self {
        Self::with_config(CircuitBreakerConfig::default())
    }

    /// Create a new circuit breaker with custom config
    pub fn with_config(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(CircuitBreakerState {
                circuit_state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
            })),
        }
    }

    /// Check if circuit allows requests
    pub async fn is_available(&self) -> bool {
        let mut state = self.state.lock().await;

        match &state.circuit_state {
            CircuitState::Closed => true,
            CircuitState::HalfOpen { .. } => true,
            CircuitState::Open { opened_at, .. } => {
                // Check if timeout has elapsed
                if opened_at.elapsed() >= self.config.timeout {
                    debug!("Circuit breaker transitioning to HalfOpen");
                    state.circuit_state = CircuitState::HalfOpen { success_count: 0 };
                    state.success_count = 0;
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Record a successful request
    pub async fn record_success(&self) {
        let mut state = self.state.lock().await;

        match &state.circuit_state {
            CircuitState::Closed => {
                // Reset failure count on success
                state.failure_count = 0;
            }
            CircuitState::HalfOpen { .. } => {
                state.success_count += 1;

                if state.success_count >= self.config.success_threshold {
                    debug!(
                        "Circuit breaker closing after {} successes",
                        state.success_count
                    );
                    state.circuit_state = CircuitState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                }
            }
            CircuitState::Open { .. } => {
                // Should not receive success in Open state
                warn!("Received success in Open state - this should not happen");
            }
        }
    }

    /// Record a failed request
    pub async fn record_failure(&self) {
        let mut state = self.state.lock().await;

        match &state.circuit_state {
            CircuitState::Closed => {
                state.failure_count += 1;

                if state.failure_count >= self.config.failure_threshold {
                    warn!(
                        "Circuit breaker opening after {} failures",
                        state.failure_count
                    );
                    state.circuit_state = CircuitState::Open {
                        opened_at: Instant::now(),
                        failure_count: state.failure_count,
                    };
                }
            }
            CircuitState::HalfOpen { .. } => {
                warn!("Circuit breaker reopening after failure in HalfOpen state");
                state.circuit_state = CircuitState::Open {
                    opened_at: Instant::now(),
                    failure_count: state.failure_count + 1,
                };
                state.failure_count += 1;
                state.success_count = 0;
            }
            CircuitState::Open { .. } => {
                // Already open, just increment counter
                state.failure_count += 1;
            }
        }
    }

    /// Get current circuit state
    pub async fn get_state(&self) -> CircuitState {
        let state = self.state.lock().await;
        state.circuit_state.clone()
    }

    /// Get failure count
    pub async fn get_failure_count(&self) -> u32 {
        let state = self.state.lock().await;
        state.failure_count
    }

    /// Reset circuit breaker to closed state
    pub async fn reset(&self) {
        let mut state = self.state.lock().await;
        debug!("Circuit breaker manually reset to Closed");
        state.circuit_state = CircuitState::Closed;
        state.failure_count = 0;
        state.success_count = 0;
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            timeout: Duration::from_secs(60),
            success_threshold: 2,
        };
        let cb = CircuitBreaker::with_config(config);

        // Initially closed
        assert!(cb.is_available().await);
        assert_eq!(cb.get_state().await, CircuitState::Closed);

        // Record failures
        cb.record_failure().await;
        cb.record_failure().await;
        assert!(cb.is_available().await); // Still closed

        cb.record_failure().await; // Threshold reached
        assert!(!cb.is_available().await); // Now open
        assert!(matches!(cb.get_state().await, CircuitState::Open { .. }));
    }

    #[tokio::test]
    async fn test_circuit_breaker_transitions_to_half_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout: Duration::from_millis(100),
            success_threshold: 2,
        };
        let cb = CircuitBreaker::with_config(config);

        // Open the circuit
        cb.record_failure().await;
        cb.record_failure().await;
        assert!(!cb.is_available().await);

        // Wait for timeout
        sleep(Duration::from_millis(150)).await;

        // Should transition to HalfOpen
        assert!(cb.is_available().await);
        assert!(matches!(
            cb.get_state().await,
            CircuitState::HalfOpen { .. }
        ));
    }

    #[tokio::test]
    async fn test_circuit_breaker_closes_after_successes() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout: Duration::from_millis(100),
            success_threshold: 2,
        };
        let cb = CircuitBreaker::with_config(config);

        // Open the circuit
        cb.record_failure().await;
        cb.record_failure().await;

        // Wait and transition to HalfOpen
        sleep(Duration::from_millis(150)).await;
        assert!(cb.is_available().await);

        // Record successes
        cb.record_success().await;
        cb.record_success().await;

        // Should close
        assert_eq!(cb.get_state().await, CircuitState::Closed);
        assert_eq!(cb.get_failure_count().await, 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_reopens_on_half_open_failure() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout: Duration::from_millis(100),
            success_threshold: 2,
        };
        let cb = CircuitBreaker::with_config(config);

        // Open the circuit
        cb.record_failure().await;
        cb.record_failure().await;

        // Wait and transition to HalfOpen
        sleep(Duration::from_millis(150)).await;
        assert!(cb.is_available().await);

        // Record failure in HalfOpen
        cb.record_failure().await;

        // Should reopen
        assert!(!cb.is_available().await);
        assert!(matches!(cb.get_state().await, CircuitState::Open { .. }));
    }

    #[tokio::test]
    async fn test_circuit_breaker_reset() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout: Duration::from_secs(60),
            success_threshold: 2,
        };
        let cb = CircuitBreaker::with_config(config);

        // Open the circuit
        cb.record_failure().await;
        cb.record_failure().await;
        assert!(!cb.is_available().await);

        // Reset
        cb.reset().await;

        // Should be closed
        assert!(cb.is_available().await);
        assert_eq!(cb.get_state().await, CircuitState::Closed);
        assert_eq!(cb.get_failure_count().await, 0);
    }

    #[tokio::test]
    async fn test_success_resets_failure_count_when_closed() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            timeout: Duration::from_secs(60),
            success_threshold: 2,
        };
        let cb = CircuitBreaker::with_config(config);

        // Record some failures
        cb.record_failure().await;
        cb.record_failure().await;
        assert_eq!(cb.get_failure_count().await, 2);

        // Record success
        cb.record_success().await;

        // Failure count should reset
        assert_eq!(cb.get_failure_count().await, 0);
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }
}
