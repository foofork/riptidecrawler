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
pub async fn get_fetch_metrics(State(state): State<AppState>) -> ApiResult<Json<FetchMetricsResponse>> {
    let metrics = state.fetch_engine.get_all_metrics().await;
    Ok(Json(metrics))
}
