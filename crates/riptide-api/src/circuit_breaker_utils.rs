//! Circuit breaker utility functions for wrapping critical operations
//!
//! This module provides helper functions to wrap extraction, PDF processing,
//! and headless service calls with circuit breaker protection.

use crate::errors::{ApiError, ApiResult};
use riptide_core::circuit_breaker::{CircuitBreakerState, CircuitBreakerError};
use riptide_core::component::PerformanceMetrics;
use riptide_core::events::{BaseEvent, EventBus, EventSeverity};
use std::future::Future;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::{warn, error, info};

/// Wrap an async operation with circuit breaker protection
///
/// # Arguments
/// * `circuit_breaker` - The circuit breaker state
/// * `performance_metrics` - Performance metrics tracker
/// * `operation_name` - Name of the operation for logging
/// * `operation` - The async operation to execute
/// * `event_bus` - Optional event bus for circuit breaker state change events
///
/// # Returns
/// Result from the operation, or circuit breaker error if circuit is open
pub async fn with_circuit_breaker<F, T, E>(
    circuit_breaker: &Arc<Mutex<CircuitBreakerState>>,
    performance_metrics: &Arc<Mutex<PerformanceMetrics>>,
    operation_name: &str,
    operation: F,
    event_bus: Option<&Arc<EventBus>>,
) -> Result<T, ApiError>
where
    F: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    // Check if circuit breaker allows the request
    let (can_execute, current_state) = {
        let cb = circuit_breaker.lock().await;
        (cb.can_execute(), cb.state().clone())
    };

    if !can_execute {
        let cb = circuit_breaker.lock().await;
        let failure_rate = cb.failure_rate();
        warn!(
            operation = operation_name,
            state = ?cb.state(),
            failure_rate = failure_rate,
            "Circuit breaker is OPEN, rejecting request"
        );

        // Emit circuit breaker open event
        if let Some(bus) = event_bus {
            let mut event = BaseEvent::new(
                "circuit_breaker.open",
                "circuit_breaker_utils",
                EventSeverity::Warn,
            );
            event.add_metadata("operation", operation_name);
            event.add_metadata("failure_rate", &failure_rate.to_string());
            event.add_metadata("state", &format!("{:?}", cb.state()));
            let _ = bus.emit(event).await;
        }

        return Err(ApiError::service_unavailable(format!(
            "Circuit breaker is OPEN for {}. Failure rate: {}%, Please try again later.",
            operation_name, failure_rate
        )));
    }

    let start = Instant::now();

    // Execute the operation
    match operation.await {
        Ok(result) => {
            let duration_ms = start.elapsed().as_millis() as u64;

            // Record success and check for state transition
            let state_changed = {
                let mut cb = circuit_breaker.lock().await;
                let old_state = cb.state().clone();
                cb.record_success();
                let new_state = cb.state().clone();

                let mut metrics = performance_metrics.lock().await;
                metrics.record_request(duration_ms, true);

                old_state != new_state
            };

            // Emit state transition event if circuit recovered
            if state_changed {
                if let Some(bus) = event_bus {
                    let cb = circuit_breaker.lock().await;
                    let mut event = BaseEvent::new(
                        "circuit_breaker.state_change",
                        "circuit_breaker_utils",
                        EventSeverity::Info,
                    );
                    event.add_metadata("operation", operation_name);
                    event.add_metadata("new_state", &format!("{:?}", cb.state()));
                    event.add_metadata("duration_ms", &duration_ms.to_string());
                    let _ = bus.emit(event).await;
                }
            }

            info!(
                operation = operation_name,
                duration_ms = duration_ms,
                "Operation succeeded"
            );

            Ok(result)
        }
        Err(err) => {
            let duration_ms = start.elapsed().as_millis() as u64;

            // Record failure and check for state transition
            let (state_changed, failure_rate) = {
                let mut cb = circuit_breaker.lock().await;
                let old_state = cb.state().clone();
                cb.record_failure();
                let new_state = cb.state().clone();

                let mut metrics = performance_metrics.lock().await;
                metrics.record_request(duration_ms, false);

                (old_state != new_state, cb.failure_rate())
            };

            // Emit state transition event if circuit opened
            if state_changed {
                if let Some(bus) = event_bus {
                    let cb = circuit_breaker.lock().await;
                    let mut event = BaseEvent::new(
                        "circuit_breaker.state_change",
                        "circuit_breaker_utils",
                        EventSeverity::Error,
                    );
                    event.add_metadata("operation", operation_name);
                    event.add_metadata("new_state", &format!("{:?}", cb.state()));
                    event.add_metadata("failure_rate", &failure_rate.to_string());
                    event.add_metadata("duration_ms", &duration_ms.to_string());
                    let _ = bus.emit(event).await;
                }
            }

            error!(
                operation = operation_name,
                duration_ms = duration_ms,
                error = %err,
                failure_rate = failure_rate,
                "Operation failed"
            );

            Err(ApiError::extraction(format!(
                "{} failed: {}",
                operation_name, err
            )))
        }
    }
}

/// Wrap WASM extraction with circuit breaker
pub async fn extract_with_circuit_breaker<F, T, E>(
    circuit_breaker: &Arc<Mutex<CircuitBreakerState>>,
    performance_metrics: &Arc<Mutex<PerformanceMetrics>>,
    extractor_op: F,
) -> ApiResult<T>
where
    F: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    with_circuit_breaker(
        circuit_breaker,
        performance_metrics,
        "wasm_extraction",
        extractor_op,
        None,
    )
    .await
}

/// Wrap PDF processing with circuit breaker
pub async fn process_pdf_with_circuit_breaker<F, T, E>(
    circuit_breaker: &Arc<Mutex<CircuitBreakerState>>,
    performance_metrics: &Arc<Mutex<PerformanceMetrics>>,
    pdf_op: F,
) -> ApiResult<T>
where
    F: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    with_circuit_breaker(
        circuit_breaker,
        performance_metrics,
        "pdf_processing",
        pdf_op,
        None,
    )
    .await
}

/// Wrap headless service call with circuit breaker
pub async fn headless_extract_with_circuit_breaker<F, T, E>(
    circuit_breaker: &Arc<Mutex<CircuitBreakerState>>,
    performance_metrics: &Arc<Mutex<PerformanceMetrics>>,
    headless_op: F,
) -> ApiResult<T>
where
    F: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    with_circuit_breaker(
        circuit_breaker,
        performance_metrics,
        "headless_extraction",
        headless_op,
        None,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use riptide_core::circuit_breaker::CircuitBreakerConfig;

    #[tokio::test]
    async fn test_circuit_breaker_wrapper_success() {
        let cb = Arc::new(Mutex::new(CircuitBreakerState::new(
            CircuitBreakerConfig::default(),
        )));
        let metrics = Arc::new(Mutex::new(PerformanceMetrics::default()));

        let result = with_circuit_breaker(
            &cb,
            &metrics,
            "test_op",
            async { Ok::<_, String>("success".to_string()) },
            None,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_circuit_breaker_wrapper_failure() {
        let cb = Arc::new(Mutex::new(CircuitBreakerState::new(
            CircuitBreakerConfig::default(),
        )));
        let metrics = Arc::new(Mutex::new(PerformanceMetrics::default()));

        let result = with_circuit_breaker(
            &cb,
            &metrics,
            "test_op",
            async { Err::<String, _>("operation failed") },
            None,
        )
        .await;

        assert!(result.is_err());
    }
}
