//! Circuit breaker port definition
//!
//! This module defines the abstract circuit breaker interface for fault tolerance
//! and graceful degradation. The port enables dependency inversion for resilience
//! patterns across the system.
//!
//! # Architecture
//!
//! Circuit breakers implement the **Circuit Breaker Pattern** to prevent cascading
//! failures and allow systems to recover gracefully:
//!
//! - **Closed**: Normal operation, requests flow through
//! - **Open**: Failures detected, requests fail fast
//! - **HalfOpen**: Testing recovery, limited requests allowed
//!
//! # Design Goals
//!
//! - **Fault Tolerance**: Prevent cascading failures across services
//! - **Fast Failure**: Fail fast when downstream systems are unhealthy
//! - **Self-Healing**: Automatically test recovery and transition states
//! - **Testability**: Enable mock implementations for unit tests
//! - **Flexibility**: Support multiple backends (standard, LLM-specific, custom)
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_types::ports::CircuitBreaker;
//!
//! async fn example(circuit: &dyn CircuitBreaker) -> anyhow::Result<()> {
//!     // Try to acquire permission to execute
//!     match circuit.try_call().await {
//!         Ok(permit) => {
//!             // Execute operation
//!             match risky_operation().await {
//!                 Ok(result) => {
//!                     circuit.on_success().await;
//!                     Ok(result)
//!                 }
//!                 Err(e) => {
//!                     circuit.on_failure().await;
//!                     Err(e)
//!                 }
//!             }
//!         }
//!         Err(_) => {
//!             // Circuit is open, fail fast
//!             Err(anyhow::anyhow!("Circuit breaker is open"))
//!         }
//!     }
//! }
//! ```

use crate::error::Result as RiptideResult;
use async_trait::async_trait;
use std::fmt::Debug;
use std::time::Duration;

/// Circuit breaker state
///
/// Represents the current state of the circuit breaker and determines
/// how requests are handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed - requests pass through normally
    ///
    /// In this state:
    /// - All requests are allowed
    /// - Failures are counted
    /// - Transitions to Open when failure threshold is reached
    Closed,

    /// Circuit is open - requests are rejected immediately
    ///
    /// In this state:
    /// - All requests fail fast without execution
    /// - System waits for recovery timeout
    /// - Transitions to HalfOpen after timeout expires
    Open,

    /// Circuit is half-open - limited requests allowed to test recovery
    ///
    /// In this state:
    /// - Limited number of trial requests allowed
    /// - Success leads to transition to Closed
    /// - Failure leads to transition back to Open
    HalfOpen,
}

/// Circuit breaker configuration
///
/// Defines thresholds and timeouts for circuit breaker behavior.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit
    pub failure_threshold: u32,

    /// Time to wait before transitioning from Open to HalfOpen
    pub recovery_timeout: Duration,

    /// Maximum number of trial requests in HalfOpen state
    pub half_open_max_requests: u32,

    /// Optional: Success rate threshold for closing in HalfOpen
    pub success_rate_threshold: Option<f32>,

    /// Optional: Time window for counting failures
    pub failure_window: Option<Duration>,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            half_open_max_requests: 3,
            success_rate_threshold: Some(0.7),
            failure_window: Some(Duration::from_secs(60)),
        }
    }
}

/// Circuit breaker statistics for monitoring
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    /// Current state
    pub state: CircuitState,

    /// Total requests attempted
    pub total_requests: u64,

    /// Successful requests
    pub successful_requests: u64,

    /// Failed requests
    pub failed_requests: u64,

    /// Number of times circuit has opened
    pub circuit_opens: u64,

    /// Current failure count in window
    pub current_failures: u32,

    /// Success rate (0.0 - 1.0)
    pub success_rate: f32,
}

impl Default for CircuitBreakerStats {
    fn default() -> Self {
        Self {
            state: CircuitState::Closed,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            circuit_opens: 0,
            current_failures: 0,
            success_rate: 1.0,
        }
    }
}

/// Call permit granted by circuit breaker
///
/// Acts as a guard to ensure proper tracking of request outcomes.
/// Dropping without calling `on_success()` or `on_failure()` may
/// count as a failure depending on implementation.
pub trait CircuitBreakerPermit: Send + Sync {
    /// Mark the call as successful
    fn on_success(&self);

    /// Mark the call as failed
    fn on_failure(&self);
}

/// Backend-agnostic circuit breaker interface
///
/// Implementations must be thread-safe (`Send + Sync`) and support
/// asynchronous state transitions and permit acquisition.
///
/// # State Machine
///
/// ```text
/// ┌─────────┐
/// │ Closed  │────failures > threshold───┐
/// └─────────┘                            ▼
///      ▲                            ┌────────┐
///      │                            │  Open  │
///      │                            └────────┘
///      │                                 │
///      │                                 │ recovery timeout
///      │                                 ▼
///      │                          ┌────────────┐
///      └──success in trial────────│ HalfOpen   │
///                                 └────────────┘
///                                      │
///                                      │ failure in trial
///                                      └──────────────────▶ Open
/// ```
#[async_trait]
pub trait CircuitBreaker: Send + Sync + Debug {
    /// Get the current state of the circuit breaker
    ///
    /// # Returns
    ///
    /// Current circuit state (Closed, Open, or HalfOpen)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if circuit.state().await == CircuitState::Open {
    ///     println!("Circuit is open, failing fast");
    /// }
    /// ```
    async fn state(&self) -> CircuitState;

    /// Attempt to acquire permission to execute a call
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Permission granted, proceed with operation
    /// * `Err(_)` - Circuit is open, fail fast
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// match circuit.try_call().await {
    ///     Ok(()) => {
    ///         // Execute operation
    ///         let result = execute_operation().await;
    ///         if result.is_ok() {
    ///             circuit.on_success().await;
    ///         } else {
    ///             circuit.on_failure().await;
    ///         }
    ///     }
    ///     Err(_) => {
    ///         // Circuit open, handle gracefully
    ///     }
    /// }
    /// ```
    async fn try_call(&self) -> RiptideResult<()>;

    /// Record a successful call
    ///
    /// Updates internal state and may trigger state transitions:
    /// - In Closed: Resets failure counter
    /// - In HalfOpen: May transition to Closed if threshold met
    /// - In Open: No effect
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let result = execute_operation().await;
    /// if result.is_ok() {
    ///     circuit.on_success().await;
    /// }
    /// ```
    async fn on_success(&self);

    /// Record a failed call
    ///
    /// Updates internal state and may trigger state transitions:
    /// - In Closed: Increments failure counter, may open circuit
    /// - In HalfOpen: Immediately transitions to Open
    /// - In Open: No effect
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let result = execute_operation().await;
    /// if result.is_err() {
    ///     circuit.on_failure().await;
    /// }
    /// ```
    async fn on_failure(&self);

    /// Get current statistics for monitoring
    ///
    /// # Returns
    ///
    /// Current circuit breaker statistics including request counts,
    /// success rate, and state transitions.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let stats = circuit.stats().await?;
    /// println!("Success rate: {:.2}%", stats.success_rate * 100.0);
    /// println!("Circuit opens: {}", stats.circuit_opens);
    /// ```
    async fn stats(&self) -> RiptideResult<CircuitBreakerStats>;

    /// Reset the circuit breaker to Closed state
    ///
    /// Clears all counters and resets to initial state. Useful for:
    /// - Administrative recovery
    /// - Testing scenarios
    /// - Manual intervention
    ///
    /// **Warning**: Use with caution in production. Manual resets can
    /// mask underlying issues.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Administrative reset after fixing downstream issue
    /// circuit.reset().await?;
    /// ```
    async fn reset(&self) -> RiptideResult<()>;

    /// Check if circuit is allowing requests
    ///
    /// Convenience method to check if requests would be accepted.
    ///
    /// # Returns
    ///
    /// * `true` - Circuit is Closed or HalfOpen with permits available
    /// * `false` - Circuit is Open or HalfOpen with no permits
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if circuit.is_call_permitted().await {
    ///     // Proceed with operation
    /// }
    /// ```
    async fn is_call_permitted(&self) -> bool {
        self.try_call().await.is_ok()
    }

    /// Get human-readable health status
    ///
    /// # Returns
    ///
    /// String describing circuit health for dashboards/logs
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// println!("Circuit health: {}", circuit.health_status().await);
    /// ```
    async fn health_status(&self) -> String {
        let stats = self.stats().await.unwrap_or_default();
        format!(
            "State: {:?}, Success Rate: {:.2}%, Opens: {}",
            stats.state,
            stats.success_rate * 100.0,
            stats.circuit_opens
        )
    }
}

/// Helper for executing operations with circuit breaker protection
///
/// Convenience function for wrapping operations with circuit breaker logic.
///
/// # Example
///
/// ```rust,ignore
/// use riptide_types::ports::with_circuit_breaker;
///
/// let result = with_circuit_breaker(&circuit, || async {
///     // Your operation here
///     fetch_data().await
/// }).await?;
/// ```
pub async fn with_circuit_breaker<F, Fut, T>(
    circuit: &dyn CircuitBreaker,
    operation: F,
) -> RiptideResult<T>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = RiptideResult<T>>,
{
    // Try to acquire permission
    circuit.try_call().await?;

    // Execute operation
    match operation().await {
        Ok(result) => {
            circuit.on_success().await;
            Ok(result)
        }
        Err(e) => {
            circuit.on_failure().await;
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_state_equality() {
        assert_eq!(CircuitState::Closed, CircuitState::Closed);
        assert_ne!(CircuitState::Closed, CircuitState::Open);
        assert_ne!(CircuitState::Open, CircuitState::HalfOpen);
    }

    #[test]
    fn test_default_config() {
        let config = CircuitBreakerConfig::default();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.recovery_timeout, Duration::from_secs(30));
        assert_eq!(config.half_open_max_requests, 3);
        assert!(config.success_rate_threshold.is_some());
    }

    #[test]
    fn test_default_stats() {
        let stats = CircuitBreakerStats::default();
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.success_rate, 1.0);
    }
}
