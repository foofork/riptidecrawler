//! Circuit Breaker Module
//!
//! This module provides circuit breaker functionality with deadlock-safe implementation.
//! The key improvement is the phase-based locking pattern that ensures no overlapping
//! locks are held across await points.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::warn;

// P2-F1 Day 3: Import from external crates (no longer from riptide-core)
#[cfg(feature = "events")]
use riptide_pool::PerformanceMetrics;
#[cfg(not(feature = "events"))]
use riptide_types::extracted::PerformanceMetrics; // Fallback if events feature not enabled

#[cfg(feature = "events")]
use riptide_events::{EventBus, PoolEvent, PoolOperation};

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

        // Update average processing time
        let new_time = result.duration.as_millis() as f64;
        m.avg_processing_time_ms = if m.total_extractions == 1 {
            new_time
        } else {
            (m.avg_processing_time_ms + new_time) / 2.0
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
