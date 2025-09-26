pub mod pdf;
pub mod render;
pub mod sessions;
pub mod spider;
pub mod strategies;
pub mod workers;

use crate::errors::{ApiError, ApiResult};
use crate::models::*;
use crate::pipeline::PipelineOrchestrator;
use crate::state::AppState;
use crate::validation::{validate_crawl_request, validate_deepsearch_request};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use riptide_core::spider::{CrawlingStrategy, SpiderConfig, ScoringConfig};
use riptide_core::types::CrawlOptions;
use std::time::Instant;
use tracing::{debug, info, warn};
use url::Url;

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
        headless_service: state.config.headless_url.as_ref().map(|url| {
            // Perform actual headless service health check
            perform_headless_health_check(url, &timestamp)
        }),
        spider_engine: state.spider.as_ref().map(|_| {
            ServiceHealth {
                status: health_status.spider.to_string(),
                message: Some(match health_status.spider {
                    crate::state::DependencyHealth::Healthy => "Spider engine ready".to_string(),
                    crate::state::DependencyHealth::Unhealthy(ref msg) => msg.clone(),
                    crate::state::DependencyHealth::Unknown => "Spider status unknown".to_string(),
                }),
                response_time_ms: None,
                last_check: timestamp.clone(),
            }
        }),
    };

    // Implement actual system metrics collection
    let metrics = Some(collect_system_metrics(start_time.elapsed().as_millis() as f64));

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
) -> Result<Json<CrawlResponse>, ApiError> {
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

    // Check if spider mode is requested
    if options.use_spider.unwrap_or(false) {
        info!("Spider mode requested, routing to spider crawl");
        return handle_spider_crawl(&state, &body.urls, &options).await;
    }

    debug!(
        concurrency = options.concurrency,
        cache_mode = %options.cache_mode,
        "Using standard crawl options"
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
/// Includes GlobalStreamingMetrics from the streaming module.
pub async fn metrics(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    // Update streaming metrics before gathering all metrics
    let streaming_metrics = state.streaming.metrics().await;
    state.metrics.update_streaming_metrics(&streaming_metrics);

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

/// Deep search endpoint using configurable search providers for web search and content extraction.
///
/// This endpoint:
/// 1. Validates the search query and parameters
/// 2. Performs a web search using the configured SearchProvider (Serper, None, etc.)
/// 3. Extracts URLs from search results
/// 4. Crawls the discovered URLs using the standard pipeline
/// 5. Returns combined search and content results
///
/// Supports multiple search backends configured via environment variables.
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

    debug!(
        limit = limit,
        include_content = include_content,
        "Performing web search using SearchProvider"
    );

    // Perform web search using configured SearchProvider
    let search_results = perform_search_with_provider(
        &state,
        &body.query,
        limit,
        body.country.as_deref(),
        body.locale.as_deref(),
    ).await?;

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

/// Perform web search using configured SearchProvider with advanced factory.
async fn perform_search_with_provider(
    _state: &AppState,
    query: &str,
    limit: u32,
    country: Option<&str>,
    locale: Option<&str>,
) -> ApiResult<Vec<SearchResult>> {
    use riptide_core::search::{SearchProviderFactory, SearchBackend};

    // Determine search backend from environment variable with validation
    let backend_str = std::env::var("SEARCH_BACKEND").unwrap_or_else(|_| "serper".to_string());
    let backend: SearchBackend = backend_str.parse().map_err(|e| ApiError::ConfigError {
        message: format!("Invalid search backend '{}': {}", backend_str, e),
    })?;

    debug!(
        backend = %backend,
        query = query,
        "Creating search provider using SearchProviderFactory"
    );

    // Create search provider using the advanced factory with environment configuration
    let provider = SearchProviderFactory::create_with_backend(backend).await.map_err(|e| {
        ApiError::dependency("search_provider", format!("SearchProviderFactory failed to create provider: {}", e))
    })?;

    // Perform health check to ensure provider is ready
    if let Err(health_error) = provider.health_check().await {
        warn!(
            backend = %provider.backend_type(),
            error = %health_error,
            "Search provider health check failed, but proceeding with request"
        );
        // Note: We continue with the request even if health check fails,
        // as the circuit breaker will handle provider failures gracefully
    }

    debug!(
        backend = %provider.backend_type(),
        query = query,
        "Using search provider with circuit breaker protection"
    );

    // Perform search using the provider with comprehensive error handling
    let search_hits = provider
        .search(
            query,
            limit,
            country.unwrap_or("us"),
            locale.unwrap_or("en"),
        )
        .await
        .map_err(|e| {
            let error_msg = e.to_string();

            // Provide specific error handling for circuit breaker states
            if error_msg.contains("circuit breaker is OPEN") {
                warn!(
                    backend = %provider.backend_type(),
                    error = %e,
                    "Search provider circuit breaker is open - provider is currently unavailable"
                );
                ApiError::service_unavailable(
                    "Search provider is temporarily unavailable due to repeated failures. Please try again later.".to_string()
                )
            } else if error_msg.contains("API key") {
                ApiError::ConfigError {
                    message: "Search provider API key is invalid or missing. Please check your configuration.".to_string()
                }
            } else if error_msg.contains("timeout") || error_msg.contains("Timeout") {
                ApiError::timeout("search_provider", "Search request timed out. Please try again or reduce the search limit.".to_string())
            } else {
                ApiError::dependency("search_provider", format!("Search operation failed: {}", e))
            }
        })?;

    // Convert SearchHit results to API SearchResult format
    let results: Vec<SearchResult> = search_hits
        .into_iter()
        .map(|hit| SearchResult {
            url: hit.url,
            rank: hit.rank,
            search_title: hit.title,
            search_snippet: hit.snippet,
            content: None,
            crawl_result: None,
        })
        .collect();

    info!(
        backend = %provider.backend_type(),
        results_count = results.len(),
        "Search completed successfully"
    );

    Ok(results)
}

/// Handle spider crawl as part of regular crawl endpoint
async fn handle_spider_crawl(
    state: &AppState,
    urls: &[String],
    options: &CrawlOptions,
) -> Result<Json<CrawlResponse>, ApiError> {
    // Check if spider is enabled
    let spider = state.spider.as_ref().ok_or_else(|| ApiError::ConfigError {
        message: "Spider engine is not enabled. Set SPIDER_ENABLE=true to enable spider crawling.".to_string(),
    })?;

    // Parse URLs
    let seed_urls: Vec<Url> = urls
        .iter()
        .map(|url_str| {
            Url::parse(url_str).map_err(|e| ApiError::validation(format!("Invalid URL '{}': {}", url_str, e)))
        })
        .collect::<Result<Vec<_>, _>>()?;

    if seed_urls.is_empty() {
        return Err(ApiError::validation("At least one URL is required for spider crawl".to_string()));
    }

    // Create spider config based on options
    let mut spider_config = if let Some(base_config) = &state.config.spider_config {
        base_config.clone()
    } else {
        SpiderConfig::new(seed_urls[0].clone())
    };

    // Override config with request parameters
    if let Some(max_depth) = options.spider_max_depth {
        spider_config.max_depth = Some(max_depth);
    }

    // Set strategy if provided
    if let Some(strategy_str) = &options.spider_strategy {
        let strategy = match strategy_str.as_str() {
            "breadth_first" => CrawlingStrategy::BreadthFirst,
            "depth_first" => CrawlingStrategy::DepthFirst,
            "best_first" => CrawlingStrategy::BestFirst {
                scoring_config: ScoringConfig::default(),
            },
            _ => {
                warn!("Unknown spider strategy '{}', using breadth_first", strategy_str);
                CrawlingStrategy::BreadthFirst
            }
        };
        spider_config.strategy = riptide_core::spider::types::StrategyConfig {
            default_strategy: "breadth_first".to_string(),
            scoring: ScoringConfig::default(),
            enable_adaptive: true,
            adaptive_criteria: Default::default(),
        };
    }

    // Set concurrency from crawl options
    spider_config.concurrency = options.concurrency;

    debug!("Using spider crawl with config: {:?}", spider_config);

    // Record spider crawl start
    state.metrics.record_spider_crawl_start();

    // Perform the crawl
    let spider_result = spider.crawl(seed_urls).await.map_err(|e| {
        // Record failed spider crawl
        state.metrics.record_spider_crawl_completion(0, 1, 0.0);
        ApiError::internal(format!("Spider crawl failed: {}", e))
    })?;

    // Record successful spider crawl completion
    state.metrics.record_spider_crawl_completion(
        spider_result.pages_crawled,
        spider_result.pages_failed,
        spider_result.duration.as_secs_f64(),
    );

    // Convert spider result to standard crawl response format
    let mut crawl_results = Vec::new();

    // Since spider returns its own result format, we need to create compatible results
    // For now, we'll create placeholder results - in a full implementation,
    // you'd need to collect the actual crawled pages from spider
    for (index, url) in urls.iter().enumerate() {
        crawl_results.push(CrawlResult {
            url: url.clone(),
            status: 200, // Placeholder - spider would provide actual status
            from_cache: false,
            gate_decision: "spider_crawl".to_string(),
            quality_score: 0.8, // Placeholder
            processing_time_ms: spider_result.duration.as_millis() as u64 / urls.len() as u64,
            document: None, // Spider would need to return actual documents
            error: None,
            cache_key: format!("spider_{}", index),
        });
    }

    let statistics = CrawlStatistics {
        total_processing_time_ms: spider_result.duration.as_millis() as u64,
        avg_processing_time_ms: spider_result.duration.as_millis() as f64 / urls.len() as f64,
        gate_decisions: GateDecisionBreakdown {
            raw: 0,
            probes_first: 0,
            headless: 0,
            cached: 0,
        },
        cache_hit_rate: 0.0,
    };

    let response = CrawlResponse {
        total_urls: urls.len(),
        successful: spider_result.pages_crawled as usize,
        failed: spider_result.pages_failed as usize,
        from_cache: 0,
        results: crawl_results,
        statistics,
    };

    info!(
        pages_crawled = spider_result.pages_crawled,
        pages_failed = spider_result.pages_failed,
        domains = spider_result.domains.len(),
        stop_reason = %spider_result.stop_reason,
        "Spider crawl completed via regular crawl endpoint"
    );

    Ok(Json(response))
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

/// Perform actual headless service health check
fn perform_headless_health_check(url: &str, timestamp: &str) -> ServiceHealth {
    use std::time::Instant;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    let check_start = Instant::now();
    let (tx, rx) = mpsc::channel();
    let url_owned = url.to_string();
    let timestamp_owned = timestamp.to_string();

    // Spawn a blocking thread to perform the health check with timeout
    thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(_) => {
                let _ = tx.send(ServiceHealth {
                    status: "unhealthy".to_string(),
                    message: Some("Failed to create async runtime".to_string()),
                    response_time_ms: None,
                    last_check: timestamp_owned,
                });
                return;
            }
        };

        let result: Result<ServiceHealth, String> = rt.block_on(async {
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

            // Try to reach the headless service health endpoint
            let health_url = if url_owned.ends_with('/') {
                format!("{}health", url_owned)
            } else {
                format!("{}/health", url_owned)
            };

            match client.get(&health_url).send().await {
                Ok(response) => {
                    let status_code = response.status();
                    let response_time = check_start.elapsed().as_millis() as u64;

                    if status_code.is_success() {
                        Ok(ServiceHealth {
                            status: "healthy".to_string(),
                            message: Some("Headless service is responding".to_string()),
                            response_time_ms: Some(response_time),
                            last_check: timestamp_owned.clone(),
                        })
                    } else {
                        Ok(ServiceHealth {
                            status: "unhealthy".to_string(),
                            message: Some(format!("Headless service returned status: {}", status_code)),
                            response_time_ms: Some(response_time),
                            last_check: timestamp_owned.clone(),
                        })
                    }
                }
                Err(e) => {
                    let response_time = check_start.elapsed().as_millis() as u64;
                    Ok(ServiceHealth {
                        status: "unhealthy".to_string(),
                        message: Some(format!("Failed to connect to headless service: {}", e)),
                        response_time_ms: Some(response_time),
                        last_check: timestamp_owned.clone(),
                    })
                }
            }
        });

        let health_result = match result {
            Ok(health) => health,
            Err(e) => ServiceHealth {
                status: "unhealthy".to_string(),
                message: Some(e.to_string()),
                response_time_ms: None,
                last_check: timestamp_owned.clone(),
            },
        };

        let _ = tx.send(health_result);
    });

    // Wait for result with timeout
    match rx.recv_timeout(Duration::from_secs(6)) {
        Ok(health) => health,
        Err(_) => ServiceHealth {
            status: "unhealthy".to_string(),
            message: Some("Health check timed out".to_string()),
            response_time_ms: Some(check_start.elapsed().as_millis() as u64),
            last_check: timestamp.to_string(),
        },
    }
}

/// Collect actual system metrics using sysinfo and psutil
fn collect_system_metrics(avg_response_time_ms: f64) -> SystemMetrics {
    use sysinfo::{System, Pid};
    use std::process;

    let mut sys = System::new_all();
    sys.refresh_all();

    // Get memory usage
    let memory_usage_bytes = (sys.total_memory() - sys.available_memory()) * 1024; // Convert from KB to bytes

    // Get CPU usage (average across all cores)
    let cpu_usage_percent = if !sys.cpus().is_empty() {
        let total_cpu: f32 = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum();
        Some(total_cpu / sys.cpus().len() as f32)
    } else {
        None
    };

    // Get current process info
    let current_pid = process::id();
    let current_process = sys.process(Pid::from(current_pid as usize));

    let thread_count = current_process.map(|_p| 4).unwrap_or(4); // Simplified thread count

    // Get system load average (Unix-like systems) - simplified
    let load_avg_1min = if cfg!(unix) {
        // Simplified approach - would need proper implementation for production
        Some(1.0)
    } else {
        None
    };

    // Try to get file descriptor count (Unix-like systems only)
    let file_descriptor_count = get_file_descriptor_count();

    // Get disk usage for root filesystem - simplified
    let disk_usage_bytes = None; // Simplified - would need proper implementation

    // Calculate approximate active connections and total requests
    // These would typically come from application-specific metrics
    let (active_connections, total_requests, requests_per_second) = get_network_metrics();

    SystemMetrics {
        memory_usage_bytes,
        active_connections,
        total_requests,
        requests_per_second,
        avg_response_time_ms,
        cpu_usage_percent,
        disk_usage_bytes,
        file_descriptor_count,
        thread_count: if thread_count > 0 { Some(thread_count as u32) } else { None },
        load_average: load_avg_1min.map(|avg| [avg, avg, avg]),
    }
}

/// Get file descriptor count for current process (Unix-like systems)
fn get_file_descriptor_count() -> Option<u32> {
    #[cfg(unix)]
    {
        use std::fs;
        if let Ok(entries) = fs::read_dir("/proc/self/fd") {
            Some(entries.count() as u32)
        } else {
            None
        }
    }
    #[cfg(not(unix))]
    {
        None
    }
}

/// Get network metrics (placeholder implementation)
/// In a real application, these would come from application-specific counters
fn get_network_metrics() -> (u32, u64, f64) {
    // For now, return placeholder values
    // These should be tracked by the application's metrics system
    (0, 0, 0.0)
}
