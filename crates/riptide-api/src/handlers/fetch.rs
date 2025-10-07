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
    // TODO(P0): Fix method resolution issue with Arc<FetchEngine>
    // ISSUE: The get_all_metrics method exists but isn't accessible through Arc
    // PLAN: Update FetchEngine to expose metrics through Arc wrapper
    // IMPLEMENTATION:
    //   1. Check FetchEngine trait bounds - ensure methods are public
    //   2. Add metrics accessor method that works with Arc
    //   3. Update state.fetch_engine usage to call new accessor
    //   4. Populate FetchMetricsResponse with real host metrics
    // DEPENDENCIES: FetchEngine implementation in riptide-core
    // EFFORT: Low (1-2 hours)
    // BLOCKER: None - this is a simple API surface issue
    Ok(Json(FetchMetricsResponse {
        hosts: std::collections::HashMap::new(),
        total_requests: 0,
        total_success: 0,
        total_failures: 0,
    }))
}
