//! Spider handlers using SpiderFacade
//!
//! This module provides HTTP handlers for deep crawling operations using
//! the riptide-facade SpiderFacade for simplified spider engine access.
#![allow(dead_code)]
use crate::context::ApplicationContext;
use crate::errors::ApiError;
use crate::models::*;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use riptide_types::ResultMode;
use serde::Deserialize;

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
    State(_state): State<ApplicationContext>,
    Query(_query): Query<SpiderCrawlQuery>,
    Json(body): Json<SpiderCrawlBody>,
) -> Result<impl IntoResponse, ApiError> {
    // Acquire resources via ResourceFacade (Phase 5 - Handler Integration)
    let tenant_id = "spider-crawl"; // TODO: Extract from request context in Phase 6
    let _resource_slot = acquire_spider_resources(&_state, tenant_id).await?;

    // Get spider facade
    let spider_facade = _state
        .spider_facade
        .as_ref()
        .ok_or_else(|| ApiError::ConfigError {
            message: "SpiderFacade not initialized".to_string(),
        })?;

    // Parse seed URLs
    let seed_urls: Vec<url::Url> = body
        .seed_urls
        .iter()
        .map(|s| url::Url::parse(s))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| ApiError::InvalidUrl {
            url: "".to_string(),
            message: format!("Invalid seed URL: {}", e),
        })?;

    // Execute crawl via facade
    let summary = spider_facade
        .crawl(seed_urls)
        .await
        .map_err(|e| ApiError::InternalError {
            message: format!("Spider crawl failed: {}", e),
        })?;

    Ok(Json(summary))
}

/// Get spider status and metrics
pub async fn spider_status(
    State(_state): State<ApplicationContext>,
    Json(_body): Json<SpiderStatusRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Get spider facade
    let spider_facade = _state
        .spider_facade
        .as_ref()
        .ok_or_else(|| ApiError::ConfigError {
            message: "SpiderFacade not initialized".to_string(),
        })?;

    // Get status from facade
    let (state, performance) =
        spider_facade
            .get_status()
            .await
            .map_err(|e| ApiError::InternalError {
                message: format!("Failed to get spider status: {}", e),
            })?;

    // Build response
    let response = SpiderStatusResponse {
        state,
        performance,
        frontier_stats: None,
        adaptive_stop_stats: None,
    };

    Ok(Json(response))
}

/// Spider control endpoint for start/stop/reset operations
pub async fn spider_control(
    State(_state): State<ApplicationContext>,
    Json(_body): Json<SpiderControlRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Get spider facade
    let spider_facade = _state
        .spider_facade
        .as_ref()
        .ok_or_else(|| ApiError::ConfigError {
            message: "SpiderFacade not initialized".to_string(),
        })?;

    // Execute control action via facade
    let message =
        spider_facade
            .control(&_body.action)
            .await
            .map_err(|e| ApiError::ValidationError {
                message: e.to_string(),
            })?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": message
    })))
}

/// Acquire resources for spider operations via ResourceFacade
///
/// Coordinates rate limiting, memory pressure, and pool capacity through
/// the ResourceFacade layer (Phase 5 integration).
async fn acquire_spider_resources(
    state: &ApplicationContext,
    tenant_id: &str,
) -> Result<crate::adapters::ResourceSlot, ApiError> {
    use riptide_facade::facades::ResourceResult as FacadeResult;

    // Acquire WASM slot through facade (handles all resource coordination)
    match state.resource_facade.acquire_wasm_slot(tenant_id).await {
        Ok(FacadeResult::Success(slot)) => Ok(slot),
        Ok(FacadeResult::RateLimited { retry_after }) => Err(ApiError::RateLimitExceeded {
            retry_after: retry_after.as_secs(),
        }),
        Ok(FacadeResult::MemoryPressure) => Err(ApiError::InternalError {
            message: "System under memory pressure".to_string(),
        }),
        Ok(FacadeResult::ResourceExhausted) => Err(ApiError::InternalError {
            message: "Spider resources exhausted".to_string(),
        }),
        Ok(FacadeResult::Timeout) => Err(ApiError::TimeoutError {
            operation: "Resource acquisition".to_string(),
            message: "Timeout acquiring spider resources".to_string(),
        }),
        Err(e) => Err(ApiError::InternalError {
            message: format!("Resource facade error: {}", e),
        }),
    }
}
