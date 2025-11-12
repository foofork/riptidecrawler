//! Circuit Breaker Module
//!
//! This module provides circuit breaker functionality with deadlock-safe implementation.
//! The key improvement is the phase-based locking pattern that ensures no overlapping
//! locks are held across await points.
//!
//! ## Relationship to Canonical Circuit Breaker
//!
//! This is a **specialized wrapper** for extraction pool management. The canonical,
//! lock-free circuit breaker lives in `riptide-types::reliability::circuit` (which we re-export).
//!
//! **Why this specialized version exists:**
//! - Integrates with `riptide-events::EventBus` for pool lifecycle events
//! - Tracks `riptide-monitoring::PerformanceMetrics` for extraction metrics
//! - Phase-based locking pattern to prevent deadlocks across async boundaries
//! - Pool-specific state management coordinated with metrics
//!
//! **Design decision:** Kept as specialized due to tight integration with event bus and metrics.
//! Uses `Mutex` for coordination instead of the canonical's lock-free atomics.
//!
//! See `/docs/architecture/CIRCUIT_BREAKER_CONSOLIDATION_SUMMARY.md` for full analysis.

use async_trait::async_trait;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::warn;

// P2-F1 Day 3: Import from external crates (no longer from riptide-core)
use riptide_events::{EventBus, PoolEvent, PoolOperation};
use riptide_monitoring::PerformanceMetrics;

/// Circuit Breaker State
///
/// Represents the current state of the circuit breaker with failure tracking.
#[derive(Debug, Clone)]
pub enum CircuitBreakerState {
    /// Normal operation - tracking failures and successes
    Closed {
        failure_count: u64,
        success_count: u64,
        last_failure: Option<Instant>,
    },
    /// Circuit is open - rejecting requests
    Open {
        opened_at: Instant,
        failure_count: u64,
    },
    /// Testing if service has recovered
    HalfOpen {
        test_requests: u64,
        start_time: Instant,
    },
}

impl Default for CircuitBreakerState {
    fn default() -> Self {
        Self::Closed {
            failure_count: 0,
            success_count: 0,
            last_failure: None,
        }
    }
}

impl CircuitBreakerState {
    /// Check if the circuit breaker is currently open
    pub fn is_open(&self) -> bool {
        matches!(self, CircuitBreakerState::Open { .. })
    }

    /// Check if the circuit breaker is half-open (testing)
    pub fn is_half_open(&self) -> bool {
        matches!(self, CircuitBreakerState::HalfOpen { .. })
    }

    /// Check if the circuit breaker is closed (normal operation)
    pub fn is_closed(&self) -> bool {
        matches!(self, CircuitBreakerState::Closed { .. })
    }

    /// Get current state as port CircuitState
    fn to_port_state(&self) -> riptide_types::ports::CircuitState {
        match self {
            CircuitBreakerState::Closed { .. } => riptide_types::ports::CircuitState::Closed,
            CircuitBreakerState::Open { .. } => riptide_types::ports::CircuitState::Open,
            CircuitBreakerState::HalfOpen { .. } => riptide_types::ports::CircuitState::HalfOpen,
        }
    }
}

// Wrapper to implement CircuitBreaker trait (requires tokio::sync::Mutex wrapper)
// This struct wraps CircuitBreakerState to provide the async CircuitBreaker trait
#[derive(Debug)]
pub struct CircuitBreakerAdapter {
    state: Arc<Mutex<CircuitBreakerState>>,
}

impl CircuitBreakerAdapter {
    pub fn new(state: CircuitBreakerState) -> Self {
        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }

    pub fn from_arc(state: Arc<Mutex<CircuitBreakerState>>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl riptide_types::ports::CircuitBreaker for CircuitBreakerAdapter {
    async fn state(&self) -> riptide_types::ports::CircuitState {
        let state = self.state.lock().await;
        state.to_port_state()
    }

    async fn try_call(&self) -> riptide_types::error::Result<()> {
        let state = self.state.lock().await;
        if state.is_open() {
            Err(riptide_types::error::RiptideError::CircuitBreakerOpen(
                "Circuit breaker is open".to_string(),
            ))
        } else {
            Ok(())
        }
    }

    async fn on_success(&self) {
        let mut state = self.state.lock().await;
        match *state {
            CircuitBreakerState::Closed {
                ref mut success_count,
                ..
            } => {
                *success_count += 1;
            }
            CircuitBreakerState::HalfOpen { .. } => {
                // Transition to Closed on success in HalfOpen
                *state = CircuitBreakerState::Closed {
                    failure_count: 0,
                    success_count: 1,
                    last_failure: None,
                };
            }
            CircuitBreakerState::Open { .. } => {
                // No-op in Open state
            }
        }
    }

    async fn on_failure(&self) {
        let mut state = self.state.lock().await;
        let now = Instant::now();
        match *state {
            CircuitBreakerState::Closed {
                ref mut failure_count,
                ..
            } => {
                *failure_count += 1;
                // Open circuit if failures exceed threshold (default 5)
                if *failure_count >= 5 {
                    *state = CircuitBreakerState::Open {
                        opened_at: now,
                        failure_count: *failure_count,
                    };
                }
            }
            CircuitBreakerState::HalfOpen { .. } => {
                // Transition back to Open on failure
                *state = CircuitBreakerState::Open {
                    opened_at: now,
                    failure_count: 1,
                };
            }
            CircuitBreakerState::Open { .. } => {
                // Already open, no-op
            }
        }
    }

    async fn stats(&self) -> riptide_types::error::Result<riptide_types::ports::CircuitBreakerStats> {
        let state = self.state.lock().await;
        let (successful_requests, failed_requests, current_failures) = match *state {
            CircuitBreakerState::Closed {
                success_count,
                failure_count,
                ..
            } => (success_count, failure_count, failure_count as u32),
            CircuitBreakerState::Open { failure_count, .. } => (0, failure_count, failure_count as u32),
            CircuitBreakerState::HalfOpen { test_requests, .. } => (test_requests, 0, 0),
        };

        let total = successful_requests + failed_requests;
        let success_rate = if total > 0 {
            successful_requests as f32 / total as f32
        } else {
            1.0
        };

        Ok(riptide_types::ports::CircuitBreakerStats {
            state: state.to_port_state(),
            total_requests: total,
            successful_requests,
            failed_requests,
            circuit_opens: 0, // Not tracked in this implementation
            current_failures,
            success_rate,
        })
    }

    async fn reset(&self) -> riptide_types::error::Result<()> {
        let mut state = self.state.lock().await;
        *state = CircuitBreakerState::Closed {
            failure_count: 0,
            success_count: 0,
            last_failure: None,
        };
        Ok(())
    }
}

/// Parameters for recording extraction results
pub struct ExtractionResult {
    /// Pool identifier for events
    pub pool_id: String,
    /// Failure rate threshold percentage (0-100)
    pub failure_threshold: u8,
    /// Circuit breaker timeout duration in milliseconds
    pub timeout_duration: u64,
    /// Whether the extraction was successful
    pub success: bool,
    /// Duration of the extraction operation
    pub duration: Duration,
}

/// Record extraction result for circuit breaker with deadlock-safe implementation
///
/// This function uses a phase-based locking pattern to prevent deadlocks:
/// - Phase 1: Update metrics with scoped lock
/// - Phase 2: Update circuit breaker state with scoped lock
/// - Phase 3: Emit events with NO locks held
///
/// # Arguments
///
/// * `metrics` - Shared performance metrics
/// * `circuit_state` - Shared circuit breaker state
/// * `event_bus` - Optional event bus for emitting events
/// * `result` - Extraction result parameters
///
/// # Deadlock Prevention
///
/// The original implementation held multiple mutex guards across await points,
/// causing compilation errors and potential runtime deadlocks. This implementation
/// fixes that by:
///
/// 1. Using scoped blocks to ensure locks are dropped before the next phase
/// 2. Extracting all necessary data before dropping locks
/// 3. Performing event emission in spawned tasks with no locks held
pub async fn record_extraction_result(
    metrics: &Arc<Mutex<PerformanceMetrics>>,
    circuit_state: &Arc<Mutex<CircuitBreakerState>>,
    event_bus: &Option<Arc<EventBus>>,
    result: ExtractionResult,
) {
    // Phase 1: Update metrics in scoped block
    let (circuit_breaker_trips, failed_extractions, total_extractions) = {
        let mut m = metrics.lock().await;

        // Update basic metrics
        m.total_extractions += 1;
        if result.success {
            m.successful_extractions += 1;
        } else {
            m.failed_extractions += 1;
        }

        // Update average extraction time
        let new_time = result.duration.as_millis() as f64;
        m.avg_extraction_time_ms = if m.total_extractions == 1 {
            new_time
        } else {
            (m.avg_extraction_time_ms + new_time) / 2.0
        };

        // Extract data before dropping lock
        (
            m.circuit_breaker_trips,
            m.failed_extractions,
            m.total_extractions,
        )
    }; // Metrics lock dropped here

    // Phase 2: Update circuit breaker state in scoped block
    let (should_emit_trip_event, should_emit_reset_event, trip_metrics, successful_extractions) = {
        let mut state = circuit_state.lock().await;
        let mut should_emit_trip = false;
        let mut should_emit_reset = false;
        let mut trip_data = None;
        let mut success_count = 0;

        // Update circuit breaker state
        let new_state = match &*state {
            CircuitBreakerState::Closed {
                failure_count,
                success_count: sc,
                ..
            } => {
                let new_failure_count = if result.success { 0 } else { failure_count + 1 };
                let new_success_count = if result.success { sc + 1 } else { *sc };
                let total_requests = new_failure_count + new_success_count;

                if total_requests >= 10 {
                    let failure_rate = (new_failure_count as f64 / total_requests as f64) * 100.0;
                    if failure_rate >= result.failure_threshold as f64 {
                        // Need to update metrics again for circuit breaker trips
                        let new_trips = circuit_breaker_trips + 1;

                        warn!(
                            failure_rate = failure_rate,
                            threshold = result.failure_threshold,
                            "Circuit breaker opened due to high failure rate"
                        );

                        // Mark that we should emit event after locks are released
                        should_emit_trip = true;
                        trip_data = Some((
                            result.failure_threshold,
                            new_trips,
                            failed_extractions,
                            total_extractions,
                        ));

                        CircuitBreakerState::Open {
                            opened_at: Instant::now(),
                            failure_count: new_failure_count,
                        }
                    } else {
                        CircuitBreakerState::Closed {
                            failure_count: new_failure_count,
                            success_count: new_success_count,
                            last_failure: if result.success {
                                None
                            } else {
                                Some(Instant::now())
                            },
                        }
                    }
                } else {
                    CircuitBreakerState::Closed {
                        failure_count: new_failure_count,
                        success_count: new_success_count,
                        last_failure: if result.success {
                            None
                        } else {
                            Some(Instant::now())
                        },
                    }
                }
            }
            CircuitBreakerState::Open {
                opened_at,
                failure_count,
            } => {
                if opened_at.elapsed() >= Duration::from_millis(result.timeout_duration) {
                    tracing::info!("Circuit breaker transitioning to half-open");
                    CircuitBreakerState::HalfOpen {
                        test_requests: 0,
                        start_time: Instant::now(),
                    }
                } else {
                    CircuitBreakerState::Open {
                        opened_at: *opened_at,
                        failure_count: *failure_count,
                    }
                }
            }
            CircuitBreakerState::HalfOpen {
                test_requests,
                start_time,
            } => {
                if result.success {
                    tracing::info!("Circuit breaker closing after successful test request");

                    // Mark that we should emit reset event after locks are released
                    should_emit_reset = true;
                    success_count = 1; // Track successful extractions for event

                    CircuitBreakerState::Closed {
                        failure_count: 0,
                        success_count: 1,
                        last_failure: None,
                    }
                } else if *test_requests >= 3 {
                    warn!("Circuit breaker reopening after failed test requests");
                    CircuitBreakerState::Open {
                        opened_at: Instant::now(),
                        failure_count: 1,
                    }
                } else {
                    CircuitBreakerState::HalfOpen {
                        test_requests: test_requests + 1,
                        start_time: *start_time,
                    }
                }
            }
        };

        *state = new_state;

        (
            should_emit_trip,
            should_emit_reset,
            trip_data,
            success_count,
        )
    }; // Circuit state lock dropped here

    // Update metrics if circuit breaker tripped (needs separate lock acquisition)
    if should_emit_trip_event {
        let mut m = metrics.lock().await;
        m.circuit_breaker_trips += 1;
    } // Metrics lock dropped here

    // Phase 3: Emit events without holding any locks
    if should_emit_trip_event {
        if let Some((failure_threshold, total_trips, failed_extractions, total_extractions)) =
            trip_metrics
        {
            if let Some(event_bus) = event_bus {
                let event_bus = event_bus.clone();
                let pool_id_clone = result.pool_id.clone();

                tokio::spawn(async move {
                    let mut event = PoolEvent::new(
                        PoolOperation::CircuitBreakerTripped,
                        pool_id_clone,
                        "instance_pool",
                    );

                    event.add_metadata("failure_threshold", &failure_threshold.to_string());
                    event.add_metadata("total_trips", &total_trips.to_string());
                    event.add_metadata("failed_extractions", &failed_extractions.to_string());
                    event.add_metadata("total_extractions", &total_extractions.to_string());

                    if let Err(e) = event_bus.emit(event).await {
                        warn!(error = %e, "Failed to emit circuit breaker tripped event");
                    }
                });
            }
        }
    }

    if should_emit_reset_event {
        if let Some(event_bus) = event_bus {
            let event_bus = event_bus.clone();
            let pool_id_clone = result.pool_id;
            let total_trips = circuit_breaker_trips;

            tokio::spawn(async move {
                let mut event = PoolEvent::new(
                    PoolOperation::CircuitBreakerReset,
                    pool_id_clone,
                    "instance_pool",
                );

                event.add_metadata("total_trips", &total_trips.to_string());
                event.add_metadata(
                    "successful_extractions",
                    &successful_extractions.to_string(),
                );

                if let Err(e) = event_bus.emit(event).await {
                    warn!(error = %e, "Failed to emit circuit breaker reset event");
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_state_default() {
        let state = CircuitBreakerState::default();
        assert!(state.is_closed());
        assert!(!state.is_open());
        assert!(!state.is_half_open());
    }

    #[test]
    fn test_circuit_breaker_state_transitions() {
        let closed = CircuitBreakerState::Closed {
            failure_count: 0,
            success_count: 10,
            last_failure: None,
        };
        assert!(closed.is_closed());

        let open = CircuitBreakerState::Open {
            opened_at: Instant::now(),
            failure_count: 5,
        };
        assert!(open.is_open());

        let half_open = CircuitBreakerState::HalfOpen {
            test_requests: 0,
            start_time: Instant::now(),
        };
        assert!(half_open.is_half_open());
    }

    #[tokio::test]
    async fn test_record_extraction_result_success() {
        let metrics = Arc::new(Mutex::new(PerformanceMetrics::default()));
        let circuit_state = Arc::new(Mutex::new(CircuitBreakerState::default()));

        record_extraction_result(
            &metrics,
            &circuit_state,
            &None,
            ExtractionResult {
                pool_id: "test-pool".to_string(),
                failure_threshold: 50,
                timeout_duration: 5000,
                success: true,
                duration: Duration::from_millis(100),
            },
        )
        .await;

        let m = metrics.lock().await;
        assert_eq!(m.total_extractions, 1);
        assert_eq!(m.successful_extractions, 1);
        assert_eq!(m.failed_extractions, 0);
    }

    #[tokio::test]
    async fn test_record_extraction_result_failure() {
        let metrics = Arc::new(Mutex::new(PerformanceMetrics::default()));
        let circuit_state = Arc::new(Mutex::new(CircuitBreakerState::default()));

        record_extraction_result(
            &metrics,
            &circuit_state,
            &None,
            ExtractionResult {
                pool_id: "test-pool".to_string(),
                failure_threshold: 50,
                timeout_duration: 5000,
                success: false,
                duration: Duration::from_millis(100),
            },
        )
        .await;

        let m = metrics.lock().await;
        assert_eq!(m.total_extractions, 1);
        assert_eq!(m.successful_extractions, 0);
        assert_eq!(m.failed_extractions, 1);
    }
}
