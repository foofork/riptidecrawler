use crate::errors::ApiError;
use crate::metrics::ErrorType;
use crate::models::*;
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use riptide_spider::{CrawlingStrategy, ScoringConfig, SpiderConfig};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

use super::shared::{spider::parse_seed_urls, MetricsRecorder};

/// Spider crawl endpoint for deep crawling operations.
///
/// This endpoint uses the Spider engine to perform deep crawling with:
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

    // Check if spider is enabled
    let spider = state.spider.as_ref().ok_or_else(|| ApiError::ConfigError {
        message: "Spider engine is not enabled".to_string(),
    })?;

    // Parse and validate seed URLs using shared utility
    let seed_urls = parse_seed_urls(&body.seed_urls)?;

    // Create a temporary spider config based on request parameters
    let mut spider_config = if let Some(base_config) = &state.config.spider_config {
        base_config.clone()
    } else {
        // Use defaults if no base config
        SpiderConfig::new(seed_urls[0].clone())
    };

    // Override config with request parameters
    if let Some(max_depth) = body.max_depth {
        spider_config.max_depth = Some(max_depth);
    }
    if let Some(max_pages) = body.max_pages {
        spider_config.max_pages = Some(max_pages);
    }
    if let Some(timeout_seconds) = body.timeout_seconds {
        spider_config.timeout = Duration::from_secs(timeout_seconds);
    }
    if let Some(delay_ms) = body.delay_ms {
        spider_config.delay = Duration::from_millis(delay_ms);
    }
    if let Some(concurrency) = body.concurrency {
        spider_config.concurrency = concurrency;
    }
    if let Some(respect_robots) = body.respect_robots {
        spider_config.respect_robots = respect_robots;
    }
    if let Some(follow_redirects) = body.follow_redirects {
        spider_config.follow_redirects = follow_redirects;
    }

    // Set strategy if provided
    if let Some(strategy_str) = &body.strategy {
        let _ = match strategy_str.as_str() {
            "breadth_first" => CrawlingStrategy::BreadthFirst,
            "depth_first" => CrawlingStrategy::DepthFirst,
            "best_first" => CrawlingStrategy::BestFirst {
                scoring_config: ScoringConfig::default(),
            },
            _ => {
                warn!("Unknown strategy '{}', using default", strategy_str);
                CrawlingStrategy::BreadthFirst
            }
        };
        spider_config.strategy = riptide_spider::types::StrategyConfig {
            default_strategy: "breadth_first".to_string(),
            scoring: ScoringConfig::default(),
            enable_adaptive: true,
            adaptive_criteria: Default::default(),
        };
    }

    debug!("Spider configuration prepared: {:?}", spider_config);

    // Create metrics recorder
    let metrics = MetricsRecorder::new(&state);

    // Record spider crawl start
    state.metrics.record_spider_crawl_start();

    // Perform the crawl
    let crawl_result = spider.crawl(seed_urls).await.map_err(|e| {
        // Record failed spider crawl
        metrics.record_spider_crawl_failure();
        ApiError::internal(format!("Spider crawl failed: {}", e))
    })?;

    // Record successful spider crawl completion
    metrics.record_spider_crawl(
        crawl_result.pages_crawled,
        crawl_result.pages_failed,
        crawl_result.duration,
    );

    // Update frontier size metrics
    let frontier_stats = spider.get_frontier_stats().await;
    metrics.update_frontier_size(frontier_stats.total_requests);

    // Get current state and performance metrics
    let crawl_state = spider.get_crawl_state().await;
    let performance_metrics = spider.get_performance_metrics().await;

    // Build API response
    let api_result = SpiderApiResult {
        pages_crawled: crawl_result.pages_crawled,
        pages_failed: crawl_result.pages_failed,
        duration_seconds: crawl_result.duration.as_secs_f64(),
        stop_reason: crawl_result.stop_reason.clone(),
        domains: crawl_result.domains,
    };

    let response = SpiderCrawlResponse {
        result: api_result,
        state: crawl_state,
        performance: performance_metrics,
    };

    info!(
        pages_crawled = crawl_result.pages_crawled,
        pages_failed = crawl_result.pages_failed,
        duration_ms = start_time.elapsed().as_millis(),
        stop_reason = %crawl_result.stop_reason,
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
    // Check if spider is enabled
    let spider = state.spider.as_ref().ok_or_else(|| ApiError::ConfigError {
        message: "Spider engine is not enabled".to_string(),
    })?;

    let include_metrics = body.include_metrics.unwrap_or(false);

    // Get current state
    let crawl_state = spider.get_crawl_state().await;

    // Get performance metrics if requested
    let performance = if include_metrics {
        Some(spider.get_performance_metrics().await)
    } else {
        None
    };

    // Get frontier stats if requested
    let frontier_stats = if include_metrics {
        Some(spider.get_frontier_stats().await)
    } else {
        None
    };

    // Get adaptive stop stats if requested
    let adaptive_stop_stats = if include_metrics {
        Some(spider.get_adaptive_stop_stats().await)
    } else {
        None
    };

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
    // Check if spider is enabled
    let spider = state.spider.as_ref().ok_or_else(|| ApiError::ConfigError {
        message: "Spider engine is not enabled".to_string(),
    })?;

    match body.action.as_str() {
        "stop" => {
            spider.stop().await;
            info!("Spider stop requested");
            Ok((
                StatusCode::OK,
                Json(serde_json::json!({"status": "stopped"})),
            ))
        }
        "reset" => match spider.reset().await {
            Ok(_) => {
                info!("Spider reset completed");
                Ok((StatusCode::OK, Json(serde_json::json!({"status": "reset"}))))
            }
            Err(e) => {
                state.metrics.record_error(ErrorType::Http);
                Err(ApiError::internal(format!("Spider reset failed: {}", e)))
            }
        },
        _ => {
            state.metrics.record_error(ErrorType::Http);
            Err(ApiError::validation(format!(
                "Unknown action: {}",
                body.action
            )))
        }
    }
}
