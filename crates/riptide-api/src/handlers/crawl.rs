use crate::errors::ApiError;
use crate::models::{
    CrawlBody, CrawlResponse, CrawlResult, CrawlStatistics, ErrorInfo, GateDecisionBreakdown,
};
use crate::pipeline::PipelineOrchestrator;
use crate::state::AppState;
use crate::telemetry_config::extract_trace_context;
use crate::validation::validate_crawl_request;
use axum::{extract::State, http::HeaderMap, Json};
use opentelemetry::trace::SpanKind;
use riptide_core::events::{BaseEvent, EventSeverity};
use riptide_core::spider::{CrawlingStrategy, ScoringConfig, SpiderConfig};
use std::time::Instant;
use tracing::{debug, info, warn, Span};
use url::Url;

use super::chunking::apply_content_chunking;

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
#[tracing::instrument(
    name = "crawl_handler",
    skip(state, body, headers),
    fields(
        http.method = "POST",
        http.route = "/crawl",
        url_count = body.urls.len(),
        cache_mode = ?body.options.as_ref().map(|o| &o.cache_mode),
        use_spider = ?body.options.as_ref().and_then(|o| o.use_spider),
        otel.kind = ?SpanKind::Server,
        otel.status_code
    )
)]
pub async fn crawl(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CrawlBody>,
) -> Result<Json<CrawlResponse>, ApiError> {
    let start_time = Instant::now();

    // Extract trace context from headers for distributed tracing (TELEM-004)
    if let Some(_parent_context) = extract_trace_context(&headers) {
        debug!("Trace context extracted from request headers");
    }

    // Record custom span attributes (TELEM-003)
    let current_span = Span::current();
    current_span.record("url_count", body.urls.len());
    if let Some(ref opts) = body.options {
        current_span.record("cache_mode", opts.cache_mode.as_str());
        current_span.record("concurrency", opts.concurrency);
        current_span.record("use_spider", opts.use_spider.unwrap_or(false));
    }

    info!(
        url_count = body.urls.len(),
        cache_mode = body.options.as_ref().map(|o| &o.cache_mode),
        "Received crawl request"
    );

    // Emit crawl start event
    let mut start_event = BaseEvent::new("crawl.started", "api.crawl_handler", EventSeverity::Info);
    start_event.add_metadata("url_count", &body.urls.len().to_string());
    if let Some(ref opts) = body.options {
        start_event.add_metadata("cache_mode", &opts.cache_mode.to_string());
        start_event.add_metadata("concurrency", &opts.concurrency.to_string());
    }
    if let Err(e) = state.event_bus.emit(start_event).await {
        warn!(error = %e, "Failed to emit crawl start event");
    }

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
    let pipeline = PipelineOrchestrator::new(state.clone(), options.clone());

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

                // Apply chunking if requested
                let document = if let Some(ref chunking_config) = options.chunking_config {
                    let doc = result.document;
                    apply_content_chunking(doc.clone(), chunking_config).await
                        .unwrap_or(doc)
                } else {
                    result.document
                };

                crawl_results.push(CrawlResult {
                    url: url.clone(),
                    status: result.http_status,
                    from_cache: result.from_cache,
                    gate_decision: result.gate_decision,
                    quality_score: result.quality_score,
                    processing_time_ms: result.processing_time_ms,
                    document: Some(document),
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

    // Record success metrics in span (TELEM-003)
    let elapsed_ms = start_time.elapsed().as_millis() as u64;
    current_span.record("otel.status_code", "OK");
    current_span.record("http.status_code", 200);
    current_span.record("successful_count", stats.successful_extractions);
    current_span.record("failed_count", stats.failed_extractions);
    current_span.record("cache_hits", from_cache_count);
    current_span.record("cache_hit_rate", cache_hit_rate);
    current_span.record("total_time_ms", elapsed_ms);

    info!(
        total_urls = body.urls.len(),
        successful = stats.successful_extractions,
        failed = stats.failed_extractions,
        cache_hits = from_cache_count,
        total_time_ms = elapsed_ms,
        "Crawl request completed"
    );

    // Emit crawl completion event
    let mut complete_event = BaseEvent::new("crawl.completed", "api.crawl_handler", EventSeverity::Info);
    complete_event.add_metadata("total_urls", &body.urls.len().to_string());
    complete_event.add_metadata("successful", &stats.successful_extractions.to_string());
    complete_event.add_metadata("failed", &stats.failed_extractions.to_string());
    complete_event.add_metadata("cache_hits", &from_cache_count.to_string());
    complete_event.add_metadata("duration_ms", &start_time.elapsed().as_millis().to_string());
    complete_event.add_metadata("cache_hit_rate", &format!("{:.2}", cache_hit_rate));
    if let Err(e) = state.event_bus.emit(complete_event).await {
        warn!(error = %e, "Failed to emit crawl completion event");
    }

    // Record metrics for crawl request
    state
        .metrics
        .record_http_request("POST", "/crawl", 200, start_time.elapsed().as_secs_f64());

    Ok(Json(response))
}

/// Handle spider crawl as part of regular crawl endpoint
pub(super) async fn handle_spider_crawl(
    state: &AppState,
    urls: &[String],
    options: &riptide_core::types::CrawlOptions,
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
        let _strategy = match strategy_str.as_str() {
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