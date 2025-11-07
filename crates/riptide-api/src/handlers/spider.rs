//! Spider handlers using SpiderFacade
//!
//! This module provides HTTP handlers for deep crawling operations using
//! the riptide-facade SpiderFacade for simplified spider engine access.
#![allow(dead_code)]
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

    // Phase 2C.2: Call SpiderFacade (restored after circular dependency fix)
    let spider_facade = _state
        .spider_facade
        .as_ref()
        .ok_or_else(|| ApiError::ConfigError {
            message:
                "SpiderFacade not initialized. Spider functionality requires the 'spider' feature."
                    .to_string(),
        })?;

    // Parse seed URLs
    let seed_urls: Result<Vec<url::Url>, _> =
        body.seed_urls.iter().map(|s| url::Url::parse(s)).collect();

    let seed_urls = seed_urls.map_err(|e| ApiError::InvalidUrl {
        url: "".to_string(),
        message: format!("Invalid seed URL: {}", e),
    })?;

    // Call spider facade
    let summary = spider_facade
        .crawl(seed_urls)
        .await
        .map_err(|e| ApiError::InternalError {
            message: format!("Spider crawl failed: {}", e),
        })?;

    // Return response
    Ok(Json(summary))
}

/// Get spider status and metrics
pub async fn spider_status(
    State(_state): State<AppState>,
    Json(_body): Json<SpiderStatusRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Phase 2C.2: Call SpiderFacade (restored after circular dependency fix)
    let spider_facade = _state
        .spider_facade
        .as_ref()
        .ok_or_else(|| ApiError::ConfigError {
            message:
                "SpiderFacade not initialized. Spider functionality requires the 'spider' feature."
                    .to_string(),
        })?;

    // Get current state
    let state = spider_facade
        .get_state()
        .await
        .map_err(|e| ApiError::InternalError {
            message: format!("Failed to get spider state: {}", e),
        })?;

    // Build response
    let response = SpiderStatusResponse {
        state,
        performance: None,         // TODO: Get performance metrics from facade
        frontier_stats: None,      // TODO: Get frontier stats from facade
        adaptive_stop_stats: None, // TODO: Get adaptive stop stats from facade
    };

    Ok(Json(response))
}

/// Spider control endpoint for start/stop/reset operations
pub async fn spider_control(
    State(_state): State<AppState>,
    Json(_body): Json<SpiderControlRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Phase 2C.2: Call SpiderFacade (restored after circular dependency fix)
    let spider_facade = _state
        .spider_facade
        .as_ref()
        .ok_or_else(|| ApiError::ConfigError {
            message:
                "SpiderFacade not initialized. Spider functionality requires the 'spider' feature."
                    .to_string(),
        })?;

    // Execute action
    match _body.action.as_str() {
        "stop" => {
            spider_facade
                .stop()
                .await
                .map_err(|e| ApiError::InternalError {
                    message: format!("Failed to stop spider: {}", e),
                })?;
            Ok(Json(serde_json::json!({
                "success": true,
                "message": "Spider stopped successfully"
            })))
        }
        "reset" => {
            spider_facade
                .reset()
                .await
                .map_err(|e| ApiError::InternalError {
                    message: format!("Failed to reset spider: {}", e),
                })?;
            Ok(Json(serde_json::json!({
                "success": true,
                "message": "Spider reset successfully"
            })))
        }
        _ => Err(ApiError::ValidationError {
            message: format!(
                "Invalid action: '{}'. Must be 'stop' or 'reset'",
                _body.action
            ),
        }),
    }
}
