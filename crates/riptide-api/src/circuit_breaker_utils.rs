//! Circuit breaker utility functions for wrapping critical operations
//!
//! This module provides helper functions to wrap extraction, PDF processing,
//! and headless service calls with circuit breaker protection.

use crate::errors::{ApiError, ApiResult};
use riptide_core::circuit_breaker::{CircuitBreakerState, CircuitBreakerError};
use riptide_core::component::PerformanceMetrics;
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
///
/// # Returns
/// Result from the operation, or circuit breaker error if circuit is open
pub async fn with_circuit_breaker<F, T, E>(
    circuit_breaker: &Arc<Mutex<CircuitBreakerState>>,
    performance_metrics: &Arc<Mutex<PerformanceMetrics>>,
    operation_name: &str,
    operation: F,
) -> Result<T, ApiError>
where
    F: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    // Check if circuit breaker allows the request
    {
        let cb = circuit_breaker.lock().await;
        if !cb.can_execute() {
            let failure_rate = cb.failure_rate();
            warn!(
                operation = operation_name,
                state = ?cb.state(),
                failure_rate = failure_rate,
                "Circuit breaker is OPEN, rejecting request"
            );
            return Err(ApiError::service_unavailable(format!(
                "Circuit breaker is OPEN for {}. Failure rate: {}%, Please try again later.",
                operation_name, failure_rate
            )));
        }
    }

    let start = Instant::now();

    // Execute the operation
    match operation.await {
        Ok(result) => {
            let duration_ms = start.elapsed().as_millis() as u64;

            // Record success
            {
                let mut cb = circuit_breaker.lock().await;
                cb.record_success();

                let mut metrics = performance_metrics.lock().await;
                metrics.record_request(duration_ms, true);
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

            // Record failure
            {
                let mut cb = circuit_breaker.lock().await;
                cb.record_failure();

                let mut metrics = performance_metrics.lock().await;
                metrics.record_request(duration_ms, false);
            }

            error!(
                operation = operation_name,
                duration_ms = duration_ms,
                error = %err,
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
        )
        .await;

        assert!(result.is_err());
    }
}
