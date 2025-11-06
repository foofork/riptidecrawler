use crate::errors::ApiError;
use crate::models::{
    CrawlBody, CrawlResponse, CrawlResult, CrawlStatistics, ErrorInfo, GateDecisionBreakdown,
};
use crate::pipeline::PipelineOrchestrator;
use crate::pipeline_enhanced::EnhancedPipelineOrchestrator;
use crate::state::AppState;
use crate::telemetry_config::extract_trace_context;
use crate::validation::validate_crawl_request;
use axum::{extract::State, http::HeaderMap, Json};
use opentelemetry::trace::SpanKind;
use riptide_events::{BaseEvent, EventSeverity};
// use riptide_spider::SpiderConfig; // Unused
use std::time::Instant;
use tracing::{debug, info, warn, Span};

use super::chunking::apply_content_chunking;
use super::shared::{MetricsRecorder, SpiderConfigBuilder};

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
        enhanced_pipeline = state.config.enhanced_pipeline_config.enable_enhanced_pipeline,
        "Using crawl options"
    );

    // Create pipeline orchestrator - use enhanced pipeline if enabled
    let (pipeline_results, stats) = if state
        .config
        .enhanced_pipeline_config
        .enable_enhanced_pipeline
    {
        info!("Using enhanced pipeline orchestrator with detailed phase timing");
        let enhanced_pipeline = EnhancedPipelineOrchestrator::new(state.clone(), options.clone());
        let (results, enhanced_stats) = enhanced_pipeline.execute_batch_enhanced(&body.urls).await;

        // Convert enhanced stats to standard stats for compatibility
        let standard_stats = crate::pipeline::PipelineStats {
            total_processed: enhanced_stats.total_urls,
            cache_hits: enhanced_stats.cache_hits,
            successful_extractions: enhanced_stats.successful,
            failed_extractions: enhanced_stats.failed,
            gate_decisions: enhanced_stats.gate_decisions,
            avg_processing_time_ms: enhanced_stats.avg_processing_time_ms,
            total_processing_time_ms: enhanced_stats.total_duration_ms,
        };

        // Convert enhanced results to standard pipeline results
        let standard_results: Vec<Option<crate::pipeline::PipelineResult>> = results
            .into_iter()
            .map(|opt_result| {
                opt_result.map(|enhanced_result| crate::pipeline::PipelineResult {
                    document: enhanced_result.document.unwrap_or_else(|| {
                        riptide_types::ExtractedDoc {
                            url: enhanced_result.url.clone(),
                            title: None,
                            text: String::new(),
                            quality_score: None,
                            links: Vec::new(),
                            byline: None,
                            published_iso: None,
                            markdown: None,
                            media: Vec::new(),
                            parser_metadata: None,
                            language: None,
                            reading_time: None,
                            word_count: None,
                            categories: Vec::new(),
                            site_name: None,
                            description: None,
                            html: None,
                        }
                    }),
                    from_cache: enhanced_result.cache_hit,
                    gate_decision: enhanced_result.gate_decision,
                    quality_score: enhanced_result.quality_score,
                    processing_time_ms: enhanced_result.total_duration_ms,
                    cache_key: format!("riptide:v1:enhanced:{}", enhanced_result.url),
                    http_status: 200,
                })
            })
            .collect();

        (standard_results, standard_stats)
    } else {
        info!("Using standard pipeline orchestrator");
        let pipeline = PipelineOrchestrator::new(state.clone(), options.clone());
        pipeline.execute_batch(&body.urls).await
    };

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
                    apply_content_chunking(doc.clone(), chunking_config)
                        .await
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
    let mut complete_event =
        BaseEvent::new("crawl.completed", "api.crawl_handler", EventSeverity::Info);
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
    _options: &riptide_types::config::CrawlOptions,
) -> Result<Json<CrawlResponse>, ApiError> {
    use super::shared::spider::parse_seed_urls;

    // Phase 2C.2: Restore spider crawl using SpiderFacade (restored after circular dependency fix)
    let spider_facade = state
        .spider_facade
        .as_ref()
        .ok_or_else(|| ApiError::ConfigError {
            message:
                "SpiderFacade not initialized. Spider functionality requires the 'spider' feature."
                    .to_string(),
        })?;

    // Parse and validate URLs using shared utility
    let seed_urls = parse_seed_urls(urls)?;

    debug!(
        seed_count = seed_urls.len(),
        "Using spider crawl via SpiderFacade"
    );

    // Create metrics recorder
    let metrics = MetricsRecorder::new(state);

    // Perform the crawl using SpiderFacade
    let spider_result =
        spider_facade
            .crawl(seed_urls)
            .await
            .map_err(|e| ApiError::InternalError {
                message: format!("Spider crawl failed: {}", e),
            })?;

    // Record successful spider crawl completion
    metrics.record_spider_crawl(
        spider_result.pages_crawled,
        spider_result.pages_failed,
        std::time::Duration::from_secs_f64(spider_result.duration_secs),
    );

    // Convert spider result to standard crawl response format
    let mut crawl_results = Vec::new();

    // Create results for discovered URLs
    // Note: Spider returns summary data, not individual page documents
    for (index, discovered_url) in spider_result.discovered_urls.iter().enumerate() {
        let is_successful = index < spider_result.pages_crawled as usize;
        crawl_results.push(CrawlResult {
            url: discovered_url.clone(),
            status: if is_successful { 200 } else { 0 },
            from_cache: false,
            gate_decision: "spider_crawl".to_string(),
            quality_score: if is_successful { 0.8 } else { 0.0 },
            processing_time_ms: (spider_result.duration_secs * 1000.0) as u64
                / spider_result.pages_crawled.max(1),
            document: None, // Spider summary doesn't include full documents
            error: if !is_successful {
                Some(ErrorInfo {
                    error_type: "spider_crawl_failed".to_string(),
                    message: "Page failed during spider crawl".to_string(),
                    retryable: true,
                })
            } else {
                None
            },
            cache_key: format!("spider:v1:{}", index),
        });
    }

    let statistics = CrawlStatistics {
        total_processing_time_ms: (spider_result.duration_secs * 1000.0) as u64,
        avg_processing_time_ms: if !spider_result.discovered_urls.is_empty() {
            (spider_result.duration_secs * 1000.0) / spider_result.discovered_urls.len() as f64
        } else {
            0.0
        },
        gate_decisions: GateDecisionBreakdown {
            raw: spider_result.pages_crawled as usize,
            probes_first: 0,
            headless: 0,
            cached: 0,
        },
        cache_hit_rate: 0.0,
    };

    let response = CrawlResponse {
        total_urls: spider_result.discovered_urls.len(),
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

    return Ok(Json(response));

    // ===== UNREACHABLE CODE BELOW - Kept for future reference during refactoring =====
    #[allow(unreachable_code)]
    {
        // Parse and validate URLs using shared utility
        let seed_urls = parse_seed_urls(urls)?;

        // Build spider config using shared builder
        let _spider_config =
            SpiderConfigBuilder::new(state, seed_urls[0].clone()).from_crawl_options(options);

        debug!("Using spider crawl via SpiderFacade with provided options");

        // Create metrics recorder
        let metrics = MetricsRecorder::new(state);

        // Perform the crawl using SpiderFacade (requires owned Vec<Url>)
        // REMOVED: let spider_result = spider_facade.crawl(seed_urls).await...

        // Placeholder struct for unreachable code to compile
        #[allow(dead_code)]
        struct PlaceholderSpiderResult {
            pages_crawled: u64,
            pages_failed: u64,
            duration_secs: f64,
            domains: Vec<String>,
            stop_reason: String,
        }
        let spider_result = PlaceholderSpiderResult {
            pages_crawled: 0,
            pages_failed: 0,
            duration_secs: 0.0,
            domains: Vec::new(),
            stop_reason: String::new(),
        };

        // Record successful spider crawl completion
        metrics.record_spider_crawl(
            spider_result.pages_crawled,
            spider_result.pages_failed,
            std::time::Duration::from_secs_f64(spider_result.duration_secs),
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
                processing_time_ms: (spider_result.duration_secs * 1000.0) as u64
                    / urls.len() as u64,
                document: None, // Spider would need to return actual documents
                error: None,
                cache_key: format!("spider_{}", index),
            });
        }

        let statistics = CrawlStatistics {
            total_processing_time_ms: (spider_result.duration_secs * 1000.0) as u64,
            avg_processing_time_ms: (spider_result.duration_secs * 1000.0) / urls.len() as f64,
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
    } // End of unreachable block
}
