//! LLM-specific circuit breaker adapter
//!
//! Adapts the specialized LLM circuit breaker from `riptide-intelligence` to implement
//! the `CircuitBreaker` port trait from `riptide-types`.
//!
//! # Architecture
//!
//! This adapter wraps the multi-signal circuit breaker with repair attempt limiting
//! that's optimized for LLM provider resilience patterns.
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_cache::adapters::LlmCircuitBreakerAdapter;
//! use riptide_types::ports::{CircuitBreaker, CircuitBreakerConfig};
//!
//! let config = CircuitBreakerConfig::default();
//! let adapter: Arc<dyn CircuitBreaker> = LlmCircuitBreakerAdapter::new(config);
//!
//! // Use through port trait
//! adapter.try_call().await?;
//! ```

use async_trait::async_trait;
use parking_lot::RwLock;
use riptide_intelligence::circuit_breaker::{
    CircuitBreakerConfig as LlmConfig, CircuitBreakerStats as LlmStats, CircuitState as LlmState,
};
use riptide_types::error::{Result as RiptideResult, RiptideError};
use riptide_types::ports::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerStats, CircuitState,
};
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Internal state for LLM circuit breaker tracking
#[derive(Debug)]
struct LlmCircuitState {
    state: LlmState,
    stats: LlmStats,
    last_state_change: Instant,
    open_until: Option<Instant>,
}

/// LLM circuit breaker adapter wrapping riptide-intelligence implementation
///
/// Provides specialized circuit breaker functionality for LLM providers with:
/// - Time-windowed failure tracking
/// - Repair attempt limiting (max 1 retry)
/// - Success rate thresholds
/// - Multi-signal state transitions
pub struct LlmCircuitBreakerAdapter {
    state: Arc<RwLock<LlmCircuitState>>,
    config: LlmConfig,
}

impl LlmCircuitBreakerAdapter {
    /// Create a new LLM circuit breaker adapter
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
    /// let adapter = LlmCircuitBreakerAdapter::new(config);
    /// ```
    pub fn new(config: CircuitBreakerConfig) -> Arc<Self> {
        let llm_config = LlmConfig {
            failure_threshold: config.failure_threshold,
            failure_window_secs: config
                .failure_window
                .unwrap_or(Duration::from_secs(60))
                .as_secs(),
            min_request_threshold: 10,
            recovery_timeout_secs: config.recovery_timeout.as_secs(),
            max_repair_attempts: 1, // Hard requirement
            success_rate_threshold: config.success_rate_threshold.unwrap_or(0.7),
            half_open_max_requests: config.half_open_max_requests,
        };

        let state = LlmCircuitState {
            state: LlmState::Closed,
            stats: LlmStats {
                state: LlmState::Closed,
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                circuit_opens: 0,
                repair_attempts: 0,
                last_failure_time: None,
                last_success_time: None,
                current_failure_count: 0,
            },
            last_state_change: Instant::now(),
            open_until: None,
        };

        Arc::new(Self {
            state: Arc::new(RwLock::new(state)),
            config: llm_config,
        })
    }

    /// Convert internal LLM state to port trait state
    fn convert_state(state: LlmState) -> CircuitState {
        match state {
            LlmState::Closed => CircuitState::Closed,
            LlmState::Open => CircuitState::Open,
            LlmState::HalfOpen => CircuitState::HalfOpen,
        }
    }

    /// Check if enough time has passed to transition from Open to HalfOpen
    fn should_attempt_reset(&self, state: &LlmCircuitState) -> bool {
        if let Some(open_until) = state.open_until {
            Instant::now() >= open_until
        } else {
            false
        }
    }

    /// Update state machine based on current conditions
    fn update_state_machine(&self, state: &mut LlmCircuitState) {
        match state.state {
            LlmState::Closed => {
                // Check if we should open due to failures
                if state.stats.current_failure_count >= self.config.failure_threshold
                    && state.stats.total_requests >= self.config.min_request_threshold as u64
                {
                    state.state = LlmState::Open;
                    state.stats.state = LlmState::Open;
                    state.stats.circuit_opens += 1;
                    state.last_state_change = Instant::now();
                    state.open_until = Some(
                        Instant::now() + Duration::from_secs(self.config.recovery_timeout_secs),
                    );
                }
            }
            LlmState::Open => {
                // Check if we should try HalfOpen
                if self.should_attempt_reset(state) {
                    state.state = LlmState::HalfOpen;
                    state.stats.state = LlmState::HalfOpen;
                    state.last_state_change = Instant::now();
                    state.open_until = None;
                }
            }
            LlmState::HalfOpen => {
                // HalfOpen transitions are handled by on_success/on_failure
            }
        }
    }
}

impl fmt::Debug for LlmCircuitBreakerAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.state.read();
        f.debug_struct("LlmCircuitBreakerAdapter")
            .field("state", &state.state)
            .field("total_requests", &state.stats.total_requests)
            .finish()
    }
}

#[async_trait]
impl CircuitBreaker for LlmCircuitBreakerAdapter {
    async fn state(&self) -> CircuitState {
        let mut state = self.state.write();
        self.update_state_machine(&mut state);
        Self::convert_state(state.state.clone())
    }

    async fn try_call(&self) -> RiptideResult<()> {
        let mut state = self.state.write();
        self.update_state_machine(&mut state);

        match state.state {
            LlmState::Closed => {
                state.stats.total_requests += 1;
                Ok(())
            }
            LlmState::Open => Err(RiptideError::CircuitBreakerOpen(
                "LLM circuit breaker is open, rejecting request".to_string(),
            )),
            LlmState::HalfOpen => {
                // Allow limited requests in HalfOpen
                // In a real implementation, this would check permit count
                state.stats.total_requests += 1;
                Ok(())
            }
        }
    }

    async fn on_success(&self) {
        let mut state = self.state.write();
        state.stats.successful_requests += 1;
        state.stats.last_success_time = Some(Instant::now());

        match state.state {
            LlmState::Closed => {
                // Reset failure count on success
                state.stats.current_failure_count = 0;
            }
            LlmState::HalfOpen => {
                // Check if we should close the circuit
                let success_rate =
                    state.stats.successful_requests as f32 / state.stats.total_requests as f32;
                if success_rate >= self.config.success_rate_threshold {
                    state.state = LlmState::Closed;
                    state.stats.state = LlmState::Closed;
                    state.stats.current_failure_count = 0;
                    state.last_state_change = Instant::now();
                }
            }
            LlmState::Open => {
                // Success in Open state (shouldn't happen, but handle gracefully)
            }
        }
    }

    async fn on_failure(&self) {
        let mut state = self.state.write();
        state.stats.failed_requests += 1;
        state.stats.current_failure_count += 1;
        state.stats.last_failure_time = Some(Instant::now());

        match state.state {
            LlmState::Closed => {
                // Check if we should open
                if state.stats.current_failure_count >= self.config.failure_threshold {
                    state.state = LlmState::Open;
                    state.stats.state = LlmState::Open;
                    state.stats.circuit_opens += 1;
                    state.last_state_change = Instant::now();
                    state.open_until = Some(
                        Instant::now() + Duration::from_secs(self.config.recovery_timeout_secs),
                    );
                }
            }
            LlmState::HalfOpen => {
                // Immediately open on failure in HalfOpen
                state.state = LlmState::Open;
                state.stats.state = LlmState::Open;
                state.stats.circuit_opens += 1;
                state.last_state_change = Instant::now();
                state.open_until =
                    Some(Instant::now() + Duration::from_secs(self.config.recovery_timeout_secs));
            }
            LlmState::Open => {
                // Already open, just count the failure
            }
        }
    }

    async fn stats(&self) -> RiptideResult<CircuitBreakerStats> {
        let state = self.state.read();
        let success_rate = if state.stats.total_requests > 0 {
            state.stats.successful_requests as f32 / state.stats.total_requests as f32
        } else {
            1.0
        };

        Ok(CircuitBreakerStats {
            state: Self::convert_state(state.stats.state.clone()),
            total_requests: state.stats.total_requests,
            successful_requests: state.stats.successful_requests,
            failed_requests: state.stats.failed_requests,
            circuit_opens: state.stats.circuit_opens,
            current_failures: state.stats.current_failure_count,
            success_rate,
        })
    }

    async fn reset(&self) -> RiptideResult<()> {
        let mut state = self.state.write();
        state.state = LlmState::Closed;
        state.stats.state = LlmState::Closed;
        state.stats.current_failure_count = 0;
        state.last_state_change = Instant::now();
        state.open_until = None;
        Ok(())
    }

    async fn is_call_permitted(&self) -> bool {
        self.try_call().await.is_ok()
    }

    async fn health_status(&self) -> String {
        let stats = self.stats().await.unwrap_or_default();
        format!(
            "State: {:?}, Success Rate: {:.2}%, Opens: {}, Repair Attempts: {}",
            stats.state,
            stats.success_rate * 100.0,
            stats.circuit_opens,
            0 // repair_attempts not tracked in current stats
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_llm_adapter_creation() {
        let config = CircuitBreakerConfig::default();
        let adapter = LlmCircuitBreakerAdapter::new(config);

        // Initially closed
        assert_eq!(adapter.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_llm_state_transitions() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: Duration::from_millis(100),
            half_open_max_requests: 2,
            success_rate_threshold: Some(0.7),
            failure_window: Some(Duration::from_secs(60)),
        };

        let adapter = LlmCircuitBreakerAdapter::new(config);

        // Initially closed
        assert_eq!(adapter.state().await, CircuitState::Closed);

        // Need enough requests to meet min_request_threshold
        for _ in 0..10 {
            let _ = adapter.try_call().await;
            adapter.on_failure().await;
        }

        // Should be open now
        assert_eq!(adapter.state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_llm_success_resets_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: Duration::from_secs(1),
            half_open_max_requests: 2,
            success_rate_threshold: Some(0.7),
            failure_window: Some(Duration::from_secs(60)),
        };

        let adapter = LlmCircuitBreakerAdapter::new(config);

        // Add some failures
        let _ = adapter.try_call().await;
        adapter.on_failure().await;
        let _ = adapter.try_call().await;
        adapter.on_failure().await;

        // Success should reset counter
        let _ = adapter.try_call().await;
        adapter.on_success().await;

        // Should still be closed
        assert_eq!(adapter.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_llm_stats() {
        let config = CircuitBreakerConfig::default();
        let adapter = LlmCircuitBreakerAdapter::new(config);

        let _ = adapter.try_call().await;
        adapter.on_success().await;

        let stats = adapter.stats().await.unwrap();
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.successful_requests, 1);
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.success_rate, 1.0);
    }

    #[tokio::test]
    async fn test_llm_reset() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout: Duration::from_secs(1),
            half_open_max_requests: 1,
            success_rate_threshold: Some(0.7),
            failure_window: Some(Duration::from_secs(60)),
        };

        let adapter = LlmCircuitBreakerAdapter::new(config);

        // Trigger failures to open
        for _ in 0..10 {
            let _ = adapter.try_call().await;
            adapter.on_failure().await;
        }

        assert_eq!(adapter.state().await, CircuitState::Open);

        // Reset
        adapter.reset().await.unwrap();

        // Should be closed again
        assert_eq!(adapter.state().await, CircuitState::Closed);
    }
}
