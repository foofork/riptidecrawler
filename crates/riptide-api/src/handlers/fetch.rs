//! Fetch metrics endpoint handlers
//!
//! Note: For simple HTTP operations, consider using state.scraper_facade
//! which provides a simplified interface via riptide-facade.

use axum::{extract::State, Json};
use riptide_fetch::FetchMetricsResponse;

use crate::{errors::ApiResult, state::AppState};

/// Get fetch engine metrics for all hosts
///
/// Returns aggregated metrics including:
/// - Per-host request counts, success/failure rates, average duration
/// - Circuit breaker states for each host
/// - Total requests across all hosts
///
/// Note: Basic FetchEngine doesn't track per-host metrics.
/// For detailed metrics, consider using PerHostFetchEngine.
/// For simple HTTP operations, consider using state.scraper_facade.
pub async fn get_fetch_metrics(
    State(state): State<AppState>,
) -> ApiResult<Json<FetchMetricsResponse>> {
    // Using fetch_engine directly (facade alternative: state.scraper_facade for simple operations)
    let metrics = state.fetch_engine.get_all_metrics().await;
    Ok(Json(metrics))
}
