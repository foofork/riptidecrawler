//! Spider handlers using SpiderFacade
//!
//! This module provides HTTP handlers for deep crawling operations using
//! the riptide-facade SpiderFacade for simplified spider engine access.

use crate::dto::{CrawledPage, FieldFilter, ResultMode, SpiderResultPages};
use crate::errors::ApiError;
use crate::metrics::ErrorType;
use crate::models::*;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::time::Instant;
use tracing::{debug, info};

use super::shared::{spider::parse_seed_urls, MetricsRecorder};

/// Default max content size in bytes (1MB)
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
    skip(state, body),
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
    State(state): State<AppState>,
    Query(query): Query<SpiderCrawlQuery>,
    Json(body): Json<SpiderCrawlBody>,
) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();

    info!(
        seed_count = body.seed_urls.len(),
        max_depth = body.max_depth,
        max_pages = body.max_pages,
        strategy = body.strategy.as_deref(),
        "Received spider crawl request"
    );

    // Check if spider facade is enabled
    let spider_facade = state
        .spider_facade
        .as_ref()
        .ok_or_else(|| ApiError::ConfigError {
            message: "SpiderFacade is not enabled".to_string(),
        })?;

    // Parse and validate seed URLs using shared utility
    let seed_urls = parse_seed_urls(&body.seed_urls)?;

    debug!("Starting spider crawl with {} seed URLs", seed_urls.len());

    // Create metrics recorder
    let metrics = MetricsRecorder::new(&state);

    // Record spider crawl start
    state.metrics.record_spider_crawl_start();

    // Perform the crawl using SpiderFacade
    let crawl_summary = spider_facade.crawl(seed_urls).await.map_err(|e| {
        // Record failed spider crawl
        metrics.record_spider_crawl_failure();
        ApiError::internal(format!("Spider crawl failed: {}", e))
    })?;

    // Record successful spider crawl completion
    metrics.record_spider_crawl(
        crawl_summary.pages_crawled,
        crawl_summary.pages_failed,
        std::time::Duration::from_secs_f64(crawl_summary.duration_secs),
    );

    // Update frontier size metrics (using pages_crawled as proxy)
    metrics.update_frontier_size(crawl_summary.pages_crawled as usize);

    // Get current state and performance metrics from facade
    let crawl_state = spider_facade
        .get_state()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get crawl state: {}", e)))?;

    let performance_metrics = spider_facade
        .get_metrics()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get performance metrics: {}", e)))?;

    info!(
        pages_crawled = crawl_summary.pages_crawled,
        pages_failed = crawl_summary.pages_failed,
        duration_ms = start_time.elapsed().as_millis(),
        stop_reason = %crawl_summary.stop_reason,
        discovered_urls = crawl_summary.discovered_urls.len(),
        result_mode = ?query.result_mode,
        "Spider crawl completed"
    );

    // Record HTTP request metrics
    metrics.record_http_request("POST", "/spider/crawl", 200, start_time.elapsed());

    // Parse field filters
    let include_filter = query.include.as_ref().map(|s| FieldFilter::parse(s));
    let exclude_filter = query.exclude.as_ref().map(|s| FieldFilter::parse(s));
    let max_content_bytes = query.max_content_bytes.unwrap_or(DEFAULT_MAX_CONTENT_BYTES);

    // Build response based on result_mode
    match query.result_mode {
        ResultMode::Stats => {
            // Statistics only (backward compatible)
            let api_result = SpiderApiResult {
                pages_crawled: crawl_summary.pages_crawled,
                pages_failed: crawl_summary.pages_failed,
                duration_seconds: crawl_summary.duration_secs,
                stop_reason: crawl_summary.stop_reason.clone(),
                domains: crawl_summary.domains.clone(),
            };

            let response = SpiderCrawlResponseStats {
                result: api_result,
                state: crawl_state,
                performance: performance_metrics,
            };

            Ok(Json(response).into_response())
        }
        ResultMode::Urls => {
            // Statistics with discovered URLs
            let api_result = SpiderApiResultUrls {
                pages_crawled: crawl_summary.pages_crawled,
                pages_failed: crawl_summary.pages_failed,
                duration_seconds: crawl_summary.duration_secs,
                stop_reason: crawl_summary.stop_reason.clone(),
                domains: crawl_summary.domains.clone(),
                discovered_urls: crawl_summary.discovered_urls.clone(),
            };

            let response = SpiderCrawlResponseUrls {
                result: api_result,
                state: crawl_state,
                performance: performance_metrics,
            };

            Ok(Json(response).into_response())
        }
        ResultMode::Pages => {
            // Full page objects with content
            // Create CrawledPage objects from discovered URLs
            // Note: In a real implementation, this would need access to the actual
            // crawled page data. For now, we create placeholder pages from discovered URLs.
            let pages: Vec<CrawledPage> = crawl_summary
                .discovered_urls
                .iter()
                .map(|url| {
                    let mut page = CrawledPage::new(url.clone(), 0, 200);
                    // TODO: Populate with actual crawled data when available
                    // For now, we set basic metadata
                    page.final_url = Some(url.clone());
                    page.robots_obeyed = Some(true);
                    page
                })
                .collect();

            let mut result = SpiderResultPages {
                pages_crawled: crawl_summary.pages_crawled,
                pages_failed: crawl_summary.pages_failed,
                duration_seconds: crawl_summary.duration_secs,
                stop_reason: crawl_summary.stop_reason.clone(),
                domains: crawl_summary.domains.clone(),
                pages,
                api_version: "v1".to_string(),
            };

            // Apply field filtering and truncation
            result.apply_field_filter(include_filter.as_ref(), exclude_filter.as_ref());
            result.truncate_content(max_content_bytes);

            Ok(Json(result).into_response())
        }
        ResultMode::Stream | ResultMode::Store => {
            // Not yet implemented
            Err(ApiError::validation(format!(
                "Result mode '{:?}' is not yet implemented. Use 'stats', 'urls', or 'pages'.",
                query.result_mode
            )))
        }
    }
}

/// Get spider status and metrics
pub async fn spider_status(
    State(state): State<AppState>,
    Json(body): Json<SpiderStatusRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Check if spider facade is enabled
    let spider_facade = state
        .spider_facade
        .as_ref()
        .ok_or_else(|| ApiError::ConfigError {
            message: "SpiderFacade is not enabled".to_string(),
        })?;

    let include_metrics = body.include_metrics.unwrap_or(false);

    // Get current state from facade
    let crawl_state = spider_facade
        .get_state()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get crawl state: {}", e)))?;

    // Get performance metrics if requested
    let performance =
        if include_metrics {
            Some(spider_facade.get_metrics().await.map_err(|e| {
                ApiError::internal(format!("Failed to get performance metrics: {}", e))
            })?)
        } else {
            None
        };

    // Note: SpiderFacade doesn't expose frontier_stats and adaptive_stop_stats directly
    // These are internal implementation details. For now, we'll return None for these.
    let frontier_stats = None;
    let adaptive_stop_stats = None;

    let response = SpiderStatusResponse {
        state: crawl_state,
        performance,
        frontier_stats,
        adaptive_stop_stats,
    };

    Ok(Json(response))
}

/// Spider control endpoint for start/stop/reset operations
pub async fn spider_control(
    State(state): State<AppState>,
    Json(body): Json<SpiderControlRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Check if spider facade is enabled
    let spider_facade = state
        .spider_facade
        .as_ref()
        .ok_or_else(|| ApiError::ConfigError {
            message: "SpiderFacade is not enabled".to_string(),
        })?;

    match body.action.as_str() {
        "stop" => {
            spider_facade.stop().await.map_err(|e| {
                state.metrics.record_error(ErrorType::Http);
                ApiError::internal(format!("Spider stop failed: {}", e))
            })?;
            info!("Spider stop requested");
            Ok((
                StatusCode::OK,
                Json(serde_json::json!({"status": "stopped"})),
            ))
        }
        "reset" => {
            spider_facade.reset().await.map_err(|e| {
                state.metrics.record_error(ErrorType::Http);
                ApiError::internal(format!("Spider reset failed: {}", e))
            })?;
            info!("Spider reset completed");
            Ok((StatusCode::OK, Json(serde_json::json!({"status": "reset"}))))
        }
        _ => {
            state.metrics.record_error(ErrorType::Http);
            Err(ApiError::validation(format!(
                "Unknown action: {}",
                body.action
            )))
        }
    }
}
