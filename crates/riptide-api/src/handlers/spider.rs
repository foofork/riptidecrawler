//! Spider handlers using SpiderFacade
//!
//! This module provides HTTP handlers for deep crawling operations using
//! the riptide-facade SpiderFacade for simplified spider engine access.

use crate::errors::ApiError;
use crate::metrics::ErrorType;
use crate::models::*;
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::time::Instant;
use tracing::{debug, info};

use super::shared::{spider::parse_seed_urls, MetricsRecorder};

/// Spider crawl endpoint for deep crawling operations.
///
/// This endpoint uses the SpiderFacade to perform deep crawling with:
/// - Simplified facade API over the Spider engine
/// - Frontier-based URL queue management
/// - Multiple crawling strategies (BFS, DFS, Best-First)
/// - Adaptive stopping based on content analysis
/// - Budget controls and rate limiting
/// - Session persistence for authenticated crawling
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

    // Build API response from CrawlSummary
    let api_result = SpiderApiResult {
        pages_crawled: crawl_summary.pages_crawled,
        pages_failed: crawl_summary.pages_failed,
        duration_seconds: crawl_summary.duration_secs,
        stop_reason: crawl_summary.stop_reason.clone(),
        domains: crawl_summary.domains.clone(),
    };

    let response = SpiderCrawlResponse {
        result: api_result,
        state: crawl_state,
        performance: performance_metrics,
    };

    info!(
        pages_crawled = crawl_summary.pages_crawled,
        pages_failed = crawl_summary.pages_failed,
        duration_ms = start_time.elapsed().as_millis(),
        stop_reason = %crawl_summary.stop_reason,
        "Spider crawl completed"
    );

    // Record HTTP request metrics
    metrics.record_http_request("POST", "/spider/crawl", 200, start_time.elapsed());

    Ok(Json(response))
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
