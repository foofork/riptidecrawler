use crate::errors::ApiError;
use crate::facades::CrawlHandlerFacade;
use crate::models::{CrawlBody, CrawlResponse};
use crate::context::ApplicationContext;
use crate::telemetry_config::extract_trace_context;
use crate::validation::validate_crawl_request;
use axum::{extract::State, http::HeaderMap, Json};
use opentelemetry::trace::SpanKind;
use riptide_events::{BaseEvent, EventSeverity};
use std::time::Instant;
use tracing::{debug, info, warn, Span};

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
    State(state): State<ApplicationContext>,
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

    // Create facade for business logic
    let facade = CrawlHandlerFacade::new(state.clone());

    // Check if spider mode is requested and route accordingly
    let response = if options.use_spider.unwrap_or(false) {
        info!("Spider mode requested, routing to spider crawl");
        facade.crawl_spider_mode(&body.urls, &options).await?
    } else {
        // Execute batch crawl through facade
        facade.crawl_batch(&body.urls, options).await?
    };

    // Record success metrics in span (TELEM-003)
    let elapsed_ms = start_time.elapsed().as_millis() as u64;
    current_span.record("otel.status_code", "OK");
    current_span.record("http.status_code", 200);
    current_span.record("successful_count", response.successful);
    current_span.record("failed_count", response.failed);
    current_span.record("cache_hits", response.from_cache);
    current_span.record("cache_hit_rate", response.statistics.cache_hit_rate);
    current_span.record("total_time_ms", elapsed_ms);

    info!(
        total_urls = body.urls.len(),
        successful = response.successful,
        failed = response.failed,
        cache_hits = response.from_cache,
        total_time_ms = elapsed_ms,
        "Crawl request completed"
    );

    // Emit crawl completion event
    let mut complete_event =
        BaseEvent::new("crawl.completed", "api.crawl_handler", EventSeverity::Info);
    complete_event.add_metadata("total_urls", &body.urls.len().to_string());
    complete_event.add_metadata("successful", &response.successful.to_string());
    complete_event.add_metadata("failed", &response.failed.to_string());
    complete_event.add_metadata("cache_hits", &response.from_cache.to_string());
    complete_event.add_metadata("duration_ms", &start_time.elapsed().as_millis().to_string());
    complete_event.add_metadata(
        "cache_hit_rate",
        &format!("{:.2}", response.statistics.cache_hit_rate),
    );
    if let Err(e) = state.event_bus.emit(complete_event).await {
        warn!(error = %e, "Failed to emit crawl completion event");
    }

    // Record metrics for crawl request
    state.record_http_request("POST", "/crawl", 200, start_time.elapsed().as_secs_f64());

    Ok(Json(response))
}
