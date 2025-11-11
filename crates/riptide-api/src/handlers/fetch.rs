//! Fetch metrics endpoint handlers
//!
//! ## Facade Pattern Usage
//!
//! This handler uses `state.fetch_engine` directly for advanced metrics and per-host circuit breakers.
//!
//! For simple HTTP operations without metrics, consider using `state.scraper_facade` which provides:
//! - `scraper_facade.fetch_html(url)` - Fetch HTML content as string
//! - `scraper_facade.fetch_bytes(url)` - Fetch raw bytes
//!
//! Example: See `/handlers/render/processors.rs` for facade usage in PDF and static rendering.

use axum::{extract::State, Json};
use riptide_fetch::FetchMetricsResponse;

use crate::{errors::ApiResult, context::ApplicationContext};

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
    State(state): State<ApplicationContext>,
) -> ApiResult<Json<FetchMetricsResponse>> {
    // Using fetch_engine directly (facade alternative: state.scraper_facade for simple operations)
    let metrics = state.fetch_engine.get_all_metrics().await;
    Ok(Json(metrics))
}
