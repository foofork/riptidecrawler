//! Spider handlers using SpiderFacade
//!
//! This module provides HTTP handlers for deep crawling operations using
//! the riptide-facade SpiderFacade for simplified spider engine access.

use crate::dto::ResultMode;
use crate::errors::ApiError;
use crate::models::*;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::time::Instant;
use tracing::info;

// Unused imports removed - parse_seed_urls and MetricsRecorder not used yet

/// Default max content size in bytes (1MB)
#[allow(dead_code)]
const DEFAULT_MAX_CONTENT_BYTES: usize = 1_048_576;

/// Query parameters for spider crawl endpoint
#[derive(Debug, Deserialize)]
pub struct SpiderCrawlQuery {
    /// Result mode: stats (default), urls, pages, stream, or store
    #[serde(default)]
    pub result_mode: ResultMode,

    /// Include specific fields (comma-separated, e.g., "title,links,markdown")
    pub include: Option<String>,

    /// Exclude specific fields (comma-separated, e.g., "content")
    pub exclude: Option<String>,

    /// Maximum content size in bytes per page (default: 1MB)
    pub max_content_bytes: Option<usize>,
}

/// Spider crawl endpoint for deep crawling operations.
///
/// This endpoint uses the SpiderFacade to perform deep crawling with:
/// - Simplified facade API over the Spider engine
/// - Frontier-based URL queue management
/// - Multiple crawling strategies (BFS, DFS, Best-First)
/// - Adaptive stopping based on content analysis
/// - Budget controls and rate limiting
/// - Session persistence for authenticated crawling
///
/// Query Parameters:
/// - result_mode: "stats" (default) or "urls"
///   - stats: Returns statistics only (backward compatible)
///   - urls: Returns statistics plus discovered URLs list
#[tracing::instrument(
    name = "spider_crawl",
    skip_all,
    fields(
        http.method = "POST",
        http.route = "/spider/crawl",
        seed_count = body.seed_urls.len(),
        max_depth = body.max_depth,
        max_pages = body.max_pages,
        otel.status_code
    )
)]
pub async fn spider_crawl(
    State(_state): State<AppState>,
    Query(_query): Query<SpiderCrawlQuery>,
    Json(body): Json<SpiderCrawlBody>,
) -> Result<impl IntoResponse, ApiError> {
    let _start_time = Instant::now();

    info!(
        seed_count = body.seed_urls.len(),
        max_depth = body.max_depth,
        max_pages = body.max_pages,
        strategy = body.strategy.as_deref(),
        "Received spider crawl request"
    );

    // Facade temporarily unavailable during refactoring
    Err::<Json<()>, ApiError>(ApiError::internal(
        "Facade temporarily unavailable during refactoring",
    ))
}

/// Get spider status and metrics
pub async fn spider_status(
    State(_state): State<AppState>,
    Json(_body): Json<SpiderStatusRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Facade temporarily unavailable during refactoring
    Err::<Json<()>, ApiError>(ApiError::internal(
        "Facade temporarily unavailable during refactoring",
    ))
}

/// Spider control endpoint for start/stop/reset operations
pub async fn spider_control(
    State(_state): State<AppState>,
    Json(_body): Json<SpiderControlRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Facade temporarily unavailable during refactoring
    Err::<Json<()>, ApiError>(ApiError::internal(
        "Facade temporarily unavailable during refactoring",
    ))
}
