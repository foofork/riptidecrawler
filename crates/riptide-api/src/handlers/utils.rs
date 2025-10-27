use crate::errors::ApiError;
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse};

/// Prometheus metrics endpoint.
///
/// Returns metrics in Prometheus exposition format for scraping by monitoring systems.
/// Includes GlobalStreamingMetrics from the streaming module.
pub async fn metrics(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    // Update streaming metrics before gathering all metrics
    let streaming_metrics = state.streaming.metrics().await;
    state.metrics.update_streaming_metrics(&streaming_metrics);

    let registry = &state.metrics.registry;
    let encoder = prometheus::TextEncoder::new();

    match encoder.encode_to_string(&registry.gather()) {
        Ok(metrics_output) => Ok((
            StatusCode::OK,
            [("Content-Type", "text/plain; version=0.0.4")],
            metrics_output,
        )),
        Err(e) => {
            tracing::error!("Failed to encode metrics: {}", e);
            Err(ApiError::dependency(
                "prometheus",
                "Failed to encode metrics",
            ))
        }
    }
}

/// 404 handler for unknown endpoints.
pub async fn not_found() -> impl IntoResponse {
    ApiError::not_found("endpoint").into_response()
}
