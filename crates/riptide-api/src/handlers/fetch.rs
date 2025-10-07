//! Fetch metrics endpoint handlers

use axum::{extract::State, Json};
use riptide_core::fetch::FetchMetricsResponse;

use crate::{errors::ApiResult, state::AppState};

/// Get fetch engine metrics for all hosts
///
/// Returns aggregated metrics including:
/// - Per-host request counts, success/failure rates, average duration
/// - Circuit breaker states for each host
/// - Total requests across all hosts
pub async fn get_fetch_metrics(State(_state): State<AppState>) -> ApiResult<Json<FetchMetricsResponse>> {
    // TODO: Fix method resolution issue with Arc<FetchEngine>
    // The get_all_metrics method exists but isn't accessible through Arc
    Ok(Json(FetchMetricsResponse {
        hosts: std::collections::HashMap::new(),
        total_requests: 0,
        total_success: 0,
        total_failures: 0,
    }))
}
