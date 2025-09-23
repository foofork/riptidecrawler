pub mod render;
pub mod sessions;

use crate::errors::{ApiError, ApiResult};
use crate::models::*;
use crate::pipeline::PipelineOrchestrator;
use crate::state::AppState;
use crate::validation::{validate_crawl_request, validate_deepsearch_request};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::time::Instant;
use tracing::{debug, info};

// Re-export render handler
pub use render::render;

/// Application startup time for uptime calculation
pub static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

/// Initialize startup time tracking
pub fn init_startup_time() {
    START_TIME.set(Instant::now()).ok();
}

/// Comprehensive health check endpoint with dependency validation.
///
/// Returns detailed health information including:
/// - Overall application health status
/// - Individual dependency health (Redis, WASM extractor, HTTP client)
/// - System metrics and uptime
/// - Version information
///
/// This endpoint is suitable for load balancer health checks and monitoring systems.
pub async fn health(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();
    debug!("Starting health check");

    // Perform comprehensive health check
    let health_status = state.health_check().await;

    // Calculate uptime
    let uptime = START_TIME
        .get()
        .map(|start| start.elapsed().as_secs())
        .unwrap_or(0);

    // Get current timestamp
    let timestamp = chrono::Utc::now().to_rfc3339();

    // Build dependency status
    let dependencies = DependencyStatus {
        redis: health_status.redis.into(),
        extractor: health_status.extractor.into(),
        http_client: health_status.http_client.into(),
        headless_service: state.config.headless_url.as_ref().map(|_| {
            // TODO: Add actual headless service health check
            ServiceHealth {
                status: "unknown".to_string(),
                message: Some("Health check not implemented".to_string()),
                response_time_ms: None,
                last_check: timestamp.clone(),
            }
        }),
    };

    // TODO: Implement actual system metrics collection
    let metrics = Some(SystemMetrics {
        memory_usage_bytes: 0,    // Placeholder
        active_connections: 0,    // Placeholder
        total_requests: 0,        // Placeholder
        requests_per_second: 0.0, // Placeholder
        avg_response_time_ms: start_time.elapsed().as_millis() as f64,
    });

    let overall_status = if health_status.healthy {
        "healthy"
    } else {
        "unhealthy"
    };

    let response = HealthResponse {
        status: overall_status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp,
        uptime,
        dependencies,
        metrics,
    };

    info!(
        status = overall_status,
        uptime_seconds = uptime,
        check_time_ms = start_time.elapsed().as_millis(),
        "Health check completed"
    );

    // Return appropriate HTTP status based on health
    let status_code = if health_status.healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    Ok((status_code, Json(response)))
}

/// Batch crawl endpoint for processing multiple URLs concurrently.
///
/// This endpoint processes multiple URLs through the complete fetch->gate->extract pipeline:
/// 1. Validates input URLs and options
/// 2. Creates a pipeline orchestrator with appropriate settings
/// 3. Executes crawling concurrently while respecting rate limits
/// 4. Returns comprehensive results with statistics
///
/// Supports various crawl options including caching strategies, concurrency limits,
/// and extraction modes.
pub async fn crawl(
    State(state): State<AppState>,
    Json(body): Json<CrawlBody>,
) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();

    info!(
        url_count = body.urls.len(),
        cache_mode = body.options.as_ref().map(|o| &o.cache_mode),
        "Received crawl request"
    );

    // Validate the request
    validate_crawl_request(&body)?;

    // Use provided options or defaults
    let options = body.options.unwrap_or_default();

    debug!(
        concurrency = options.concurrency,
        cache_mode = %options.cache_mode,
        "Using crawl options"
    );

    // Create pipeline orchestrator
    let pipeline = PipelineOrchestrator::new(state.clone(), options);

    // Execute batch crawling
    let (pipeline_results, stats) = pipeline.execute_batch(&body.urls).await;

    // Convert pipeline results to API response format
    let mut crawl_results = Vec::with_capacity(body.urls.len());
    let mut from_cache_count = 0;

    for (index, pipeline_result) in pipeline_results.into_iter().enumerate() {
        let url = &body.urls[index];

        match pipeline_result {
            Some(result) => {
                if result.from_cache {
                    from_cache_count += 1;
                }

                crawl_results.push(CrawlResult {
                    url: url.clone(),
                    status: result.http_status,
                    from_cache: result.from_cache,
                    gate_decision: result.gate_decision,
                    quality_score: result.quality_score,
                    processing_time_ms: result.processing_time_ms,
                    document: Some(result.document),
                    error: None,
                    cache_key: result.cache_key,
                });
            }
            None => {
                crawl_results.push(CrawlResult {
                    url: url.clone(),
                    status: 0, // Unknown status for failed requests
                    from_cache: false,
                    gate_decision: "failed".to_string(),
                    quality_score: 0.0,
                    processing_time_ms: 0,
                    document: None,
                    error: Some(ErrorInfo {
                        error_type: "pipeline_error".to_string(),
                        message: "Failed to process URL".to_string(),
                        retryable: true,
                    }),
                    cache_key: "".to_string(),
                });
            }
        }
    }

    // Calculate cache hit rate
    let cache_hit_rate = if !body.urls.is_empty() {
        from_cache_count as f64 / body.urls.len() as f64
    } else {
        0.0
    };

    // Build response statistics
    let statistics = CrawlStatistics {
        total_processing_time_ms: stats.total_processing_time_ms,
        avg_processing_time_ms: stats.avg_processing_time_ms,
        gate_decisions: GateDecisionBreakdown {
            raw: stats.gate_decisions.raw,
            probes_first: stats.gate_decisions.probes_first,
            headless: stats.gate_decisions.headless,
            cached: from_cache_count,
        },
        cache_hit_rate,
    };

    let response = CrawlResponse {
        total_urls: body.urls.len(),
        successful: stats.successful_extractions,
        failed: stats.failed_extractions,
        from_cache: from_cache_count,
        results: crawl_results,
        statistics,
    };

    info!(
        total_urls = body.urls.len(),
        successful = stats.successful_extractions,
        failed = stats.failed_extractions,
        cache_hits = from_cache_count,
        total_time_ms = start_time.elapsed().as_millis(),
        "Crawl request completed"
    );

    // Record metrics for crawl request
    state
        .metrics
        .record_http_request("POST", "/crawl", 200, start_time.elapsed().as_secs_f64());

    Ok(Json(response))
}

/// Prometheus metrics endpoint.
///
/// Returns metrics in Prometheus exposition format for scraping by monitoring systems.
pub async fn metrics(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let registry = &state.metrics.registry;
    let encoder = prometheus::TextEncoder::new();

    match encoder.encode_to_string(&registry.gather()) {
        Ok(metrics_output) => Ok((
            StatusCode::OK,
            [("Content-Type", "text/plain; version=0.0.4")],
            metrics_output,
        )),
        Err(e) => {
            tracing::error!("Failed to encode metrics: {}", e);
            Err(ApiError::dependency(
                "prometheus",
                "Failed to encode metrics",
            ))
        }
    }
}

/// Deep search endpoint using Serper.dev API for web search and content extraction.
///
/// This endpoint:
/// 1. Validates the search query and parameters
/// 2. Performs a web search using Serper.dev API
/// 3. Extracts URLs from search results
/// 4. Crawls the discovered URLs using the standard pipeline
/// 5. Returns combined search and content results
///
/// Requires SERPER_API_KEY environment variable to be set.
pub async fn deepsearch(
    State(state): State<AppState>,
    Json(body): Json<DeepSearchBody>,
) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();

    info!(
        query = %body.query,
        limit = body.limit,
        "Received deep search request"
    );

    // Validate the request
    validate_deepsearch_request(&body)?;

    // Get search parameters
    let limit = body.limit.unwrap_or(10).min(50); // Cap at 50 results
    let include_content = body.include_content.unwrap_or(true);

    // Check for Serper API key
    let serper_api_key = std::env::var("SERPER_API_KEY").map_err(|_| ApiError::ConfigError {
        message: "SERPER_API_KEY environment variable not set".to_string(),
    })?;

    debug!(
        limit = limit,
        include_content = include_content,
        "Performing web search"
    );

    // Perform web search using Serper.dev
    let search_results = perform_web_search(&state, &body.query, limit, &serper_api_key).await?;

    info!(
        query = %body.query,
        results_found = search_results.len(),
        "Web search completed"
    );

    // Extract URLs for crawling
    let urls: Vec<String> = search_results.iter().map(|r| r.url.clone()).collect();

    // If content extraction is requested, crawl the URLs
    let mut final_results = Vec::new();
    if include_content && !urls.is_empty() {
        let crawl_options = body.crawl_options.unwrap_or_default();
        let pipeline = PipelineOrchestrator::new(state.clone(), crawl_options);

        debug!(
            url_count = urls.len(),
            "Starting content extraction for search results"
        );

        let (pipeline_results, _) = pipeline.execute_batch(&urls).await;

        // Combine search results with crawled content
        for (index, search_result) in search_results.into_iter().enumerate() {
            let content = pipeline_results
                .get(index)
                .and_then(|r| r.as_ref().map(|pr| pr.document.clone()));

            let crawl_result = pipeline_results.get(index).and_then(|r| {
                r.as_ref().map(|pr| CrawlResult {
                    url: pr.document.url.clone(),
                    status: pr.http_status,
                    from_cache: pr.from_cache,
                    gate_decision: pr.gate_decision.clone(),
                    quality_score: pr.quality_score,
                    processing_time_ms: pr.processing_time_ms,
                    document: Some(pr.document.clone()),
                    error: None,
                    cache_key: pr.cache_key.clone(),
                })
            });

            final_results.push(SearchResult {
                url: search_result.url,
                rank: search_result.rank,
                search_title: search_result.search_title,
                search_snippet: search_result.search_snippet,
                content,
                crawl_result,
            });
        }
    } else {
        // Return search results without content
        final_results = search_results;
    }

    let processing_time_ms = start_time.elapsed().as_millis() as u64;

    let query_clone = body.query.clone();
    let response = DeepSearchResponse {
        query: body.query,
        urls_found: urls.len(),
        urls_crawled: if include_content { urls.len() } else { 0 },
        results: final_results,
        status: "completed".to_string(),
        processing_time_ms,
    };

    info!(
        query = %query_clone,
        urls_found = urls.len(),
        processing_time_ms = processing_time_ms,
        "Deep search completed"
    );

    // Record metrics for deepsearch request
    state.metrics.record_http_request(
        "POST",
        "/deepsearch",
        200,
        start_time.elapsed().as_secs_f64(),
    );

    Ok(Json(response))
}

/// Perform web search using Serper.dev API.
async fn perform_web_search(
    state: &AppState,
    query: &str,
    limit: u32,
    api_key: &str,
) -> ApiResult<Vec<SearchResult>> {
    let search_request = serde_json::json!({
        "q": query,
        "num": limit,
        "gl": "us",
        "hl": "en"
    });

    let response = state
        .http_client
        .post("https://google.serper.dev/search")
        .header("X-API-KEY", api_key)
        .header("Content-Type", "application/json")
        .json(&search_request)
        .send()
        .await
        .map_err(|e| ApiError::dependency("serper_api", format!("Search request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(ApiError::dependency(
            "serper_api",
            format!("Search API returned status: {}", response.status()),
        ));
    }

    let search_response: serde_json::Value = response.json().await.map_err(|e| {
        ApiError::dependency("serper_api", format!("Failed to parse response: {}", e))
    })?;

    // Parse search results
    let mut results = Vec::new();
    if let Some(organic) = search_response.get("organic").and_then(|o| o.as_array()) {
        for (index, result) in organic.iter().enumerate() {
            if let (Some(link), Some(title)) = (
                result.get("link").and_then(|l| l.as_str()),
                result.get("title").and_then(|t| t.as_str()),
            ) {
                results.push(SearchResult {
                    url: link.to_string(),
                    rank: (index + 1) as u32,
                    search_title: Some(title.to_string()),
                    search_snippet: result
                        .get("snippet")
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string()),
                    content: None,
                    crawl_result: None,
                });
            }
        }
    }

    Ok(results)
}

/// 404 handler for unknown endpoints.
pub async fn not_found() -> impl IntoResponse {
    let error_response = serde_json::json!({
        "error": {
            "type": "not_found",
            "message": "The requested endpoint was not found",
            "retryable": false,
            "status": 404
        }
    });

    (StatusCode::NOT_FOUND, Json(error_response))
}
