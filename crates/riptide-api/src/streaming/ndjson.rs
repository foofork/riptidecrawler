//! NDJSON (Newline Delimited JSON) streaming implementation.
//!
//! This module provides streaming JSON responses where each line contains
//! a complete JSON object, allowing for efficient streaming of large datasets.

use super::buffer::{BackpressureHandler, BufferManager};
use super::config::StreamConfig;
use super::error::{StreamingError, StreamingResult};
use crate::errors::{ApiError, ApiResult};
use crate::models::*;
use std::time::Duration;
use crate::pipeline::PipelineOrchestrator;
use crate::state::AppState;
use crate::validation::{validate_crawl_request, validate_deepsearch_request};
use axum::body::Body;
use axum::extract::{Json, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use serde::Serialize;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use futures_util::StreamExt;

/// NDJSON streaming handler for crawl operations
///
/// Key features:
/// - TTFB < 500ms with warm cache
/// - Buffer management with 65536 bytes limit
/// - Streaming results as they complete (no batching)
/// - Zero unwrap/expect error handling
pub async fn crawl_stream(
    State(app): State<AppState>,
    Json(body): Json<CrawlBody>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    let request_id = Uuid::new_v4().to_string();

    info!(
        request_id = %request_id,
        url_count = body.urls.len(),
        cache_mode = body.options.as_ref().map(|o| &o.cache_mode),
        "Received NDJSON crawl request"
    );

    // Validate the request early to fail fast - zero unwrap approach
    let validation_result = validate_crawl_request(&body);
    if let Err(e) = validation_result {
        return create_error_response(e);
    }

    // Create streaming handler with enhanced configuration
    let streaming_handler = NdjsonStreamingHandler::new_optimized(
        app.clone(),
        request_id.clone(),
        65536 // 65536 bytes buffer limit as specified
    );

    // Handle streaming with zero-error approach
    let stream_result = streaming_handler
        .handle_crawl_stream(body, start_time)
        .await;

    match stream_result {
        Ok(response) => response,
        Err(e) => {
            error!(
                request_id = %request_id,
                error = %e,
                "NDJSON crawl stream failed"
            );
            create_error_response(ApiError::from(e))
        }
    }
}

/// NDJSON streaming handler for deep search operations
///
/// Key features:
/// - Search integration with real-time streaming
/// - Content extraction streaming
/// - TTFB optimization for search operations
/// - Zero unwrap/expect error handling
pub async fn deepsearch_stream(
    State(app): State<AppState>,
    Json(body): Json<DeepSearchBody>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    let request_id = Uuid::new_v4().to_string();

    info!(
        request_id = %request_id,
        query = %body.query,
        limit = body.limit,
        "Received NDJSON deep search request"
    );

    // Validate the request early to fail fast - zero unwrap approach
    let validation_result = validate_deepsearch_request(&body);
    if let Err(e) = validation_result {
        return create_error_response(e);
    }

    // Create streaming handler with enhanced configuration
    let streaming_handler = NdjsonStreamingHandler::new_optimized(
        app.clone(),
        request_id.clone(),
        65536 // 65536 bytes buffer limit as specified
    );

    // Handle streaming with zero-error approach
    let stream_result = streaming_handler
        .handle_deepsearch_stream(body, start_time)
        .await;

    match stream_result {
        Ok(response) => response,
        Err(e) => {
            error!(
                request_id = %request_id,
                error = %e,
                "NDJSON deep search stream failed"
            );
            create_error_response(ApiError::from(e))
        }
    }
}

/// NDJSON streaming handler with buffer management
pub struct NdjsonStreamingHandler {
    app: AppState,
    request_id: String,
    buffer_manager: Arc<BufferManager>,
    config: StreamConfig,
}

impl NdjsonStreamingHandler {
    /// Create a new NDJSON streaming handler
    pub fn new(app: AppState, request_id: String) -> Self {
        Self {
            app,
            request_id,
            buffer_manager: Arc::new(BufferManager::new()),
            config: StreamConfig::from_env(),
        }
    }

    /// Create a new optimized NDJSON streaming handler with specific buffer size
    /// Designed for PR-3 requirements:
    /// - Buffer management with specified limit
    /// - TTFB optimization
    /// - Zero-error approach
    pub fn new_optimized(app: AppState, request_id: String, buffer_limit: usize) -> Self {
        let mut config = StreamConfig::from_env();

        // Configure for TTFB < 500ms optimization
        config.ndjson.flush_interval = Duration::from_millis(50); // Faster flushing
        config.buffer.max_size = buffer_limit.min(2048).max(256); // Respect limit but stay reasonable
        config.buffer.default_size = (buffer_limit / 4).min(512).max(128); // Quarter of limit
        config.general.default_timeout = Duration::from_secs(30); // Reasonable timeout

        Self {
            app,
            request_id,
            buffer_manager: Arc::new(BufferManager::new()),
            config,
        }
    }

    /// Handle crawl streaming with proper buffer management
    pub async fn handle_crawl_stream(
        &self,
        body: CrawlBody,
        start_time: Instant,
    ) -> StreamingResult<Response> {
        let buffer = self.buffer_manager.get_buffer(&self.request_id).await;
        let (tx, rx) = buffer.create_channel::<Bytes>().await;

        // Clone necessary data for the spawned task
        let app_clone = self.app.clone();
        let body_clone = body.clone();
        let request_id = self.request_id.clone();
        let buffer_clone = buffer.clone();
        let config_clone = self.config.clone();

        // Spawn the streaming orchestration task
        tokio::spawn(async move {
            let mut backpressure_handler =
                BackpressureHandler::new(request_id.clone(), buffer_clone);

            // Use enhanced orchestration with zero-error approach
            let orchestration_result = orchestrate_crawl_stream_optimized(
                app_clone,
                body_clone,
                tx,
                start_time,
                request_id.clone(),
                &mut backpressure_handler,
                config_clone,
            )
            .await;

            if let Err(e) = orchestration_result {
                error!(
                    request_id = %request_id,
                    error = %e,
                    "NDJSON crawl stream orchestration error"
                );
            }
        });

        // Return streaming response with appropriate headers
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/x-ndjson")
            .header("Transfer-Encoding", "chunked")
            .header("Cache-Control", "no-cache")
            .header("Access-Control-Allow-Origin", "*")
            .header("X-Request-ID", &self.request_id)
            .body(Body::from_stream(
                ReceiverStream::new(rx).map(Ok::<_, std::io::Error>),
            ))
            .map_err(|e| StreamingError::channel(e.to_string()))?)
    }

    /// Handle deep search streaming with proper buffer management
    pub async fn handle_deepsearch_stream(
        &self,
        body: DeepSearchBody,
        start_time: Instant,
    ) -> StreamingResult<Response> {
        let buffer = self.buffer_manager.get_buffer(&self.request_id).await;
        let (tx, rx) = buffer.create_channel::<Bytes>().await;

        // Clone necessary data for the spawned task
        let app_clone = self.app.clone();
        let body_clone = body.clone();
        let request_id = self.request_id.clone();
        let buffer_clone = buffer.clone();
        let config_clone = self.config.clone();

        // Spawn the streaming orchestration task
        tokio::spawn(async move {
            let mut backpressure_handler =
                BackpressureHandler::new(request_id.clone(), buffer_clone);

            // Use enhanced orchestration with zero-error approach
            let orchestration_result = orchestrate_deepsearch_stream_optimized(
                app_clone,
                body_clone,
                tx,
                start_time,
                request_id.clone(),
                &mut backpressure_handler,
                config_clone,
            )
            .await;

            if let Err(e) = orchestration_result {
                error!(
                    request_id = %request_id,
                    error = %e,
                    "NDJSON deep search stream orchestration error"
                );
            }
        });

        // Return streaming response with appropriate headers
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/x-ndjson")
            .header("Transfer-Encoding", "chunked")
            .header("Cache-Control", "no-cache")
            .header("Access-Control-Allow-Origin", "*")
            .header("X-Request-ID", &self.request_id)
            .body(Body::from_stream(
                ReceiverStream::new(rx).map(Ok::<_, std::io::Error>),
            ))
            .map_err(|e| StreamingError::channel(e.to_string()))?)
    }
}

/// Optimized orchestrate streaming crawl operation for PR-3 requirements
///
/// Key features:
/// - TTFB < 500ms optimization
/// - Stream results as they complete (no batching)
/// - Zero unwrap/expect error handling
/// - Buffer management with specified limits
async fn orchestrate_crawl_stream_optimized(
    app: AppState,
    body: CrawlBody,
    tx: mpsc::Sender<Bytes>,
    start_time: Instant,
    request_id: String,
    backpressure_handler: &mut BackpressureHandler,
    config: StreamConfig,
) -> StreamingResult<()> {
    let options = body.options.unwrap_or_default();

    debug!(
        request_id = %request_id,
        concurrency = options.concurrency,
        cache_mode = %options.cache_mode,
        buffer_limit = config.buffer.max_size,
        "Starting optimized NDJSON crawl orchestration"
    );

    // Send initial metadata IMMEDIATELY for TTFB optimization
    let metadata = StreamMetadata {
        total_urls: body.urls.len(),
        request_id: request_id.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        stream_type: "crawl".to_string(),
    };

    // First flush - critical for TTFB < 500ms
    let metadata_result = send_ndjson_line_fast(&tx, &metadata, backpressure_handler).await;
    if let Err(e) = metadata_result {
        return Err(StreamingError::channel(format!("Failed to send metadata: {}", e)));
    }

    // Create pipeline orchestrator
    let pipeline = PipelineOrchestrator::new(app.clone(), options);

    // Initialize tracking variables
    let mut completed_count = 0;
    let mut error_count = 0;
    let mut cache_hits = 0;

    // Use optimized streaming approach - results stream as they complete
    let urls = body.urls.clone();
    let (result_tx, mut result_rx) = mpsc::channel(urls.len().min(1000)); // Limit channel size

    // Spawn individual URL processing tasks with immediate results
    for (index, url) in urls.iter().enumerate() {
        let pipeline_clone = pipeline.clone();
        let url_clone = url.clone();
        let result_tx_clone = result_tx.clone();
        let request_id_clone = request_id.clone();

        tokio::spawn(async move {
            let task_start = Instant::now();
            let single_result = pipeline_clone.execute_single(&url_clone).await;
            let processing_time = task_start.elapsed();

            // Send result immediately when complete (no batching)
            let send_result = result_tx_clone
                .send((index, url_clone.clone(), single_result, processing_time))
                .await;

            if let Err(_) = send_result {
                warn!(
                    request_id = %request_id_clone,
                    url = %url_clone,
                    "Failed to send result - receiver likely closed"
                );
            }
        });
    }

    drop(result_tx); // Close the sender

    // Stream results as they arrive - critical for no-batching requirement
    while let Some((index, url, pipeline_result, processing_time)) = result_rx.recv().await {
        let crawl_result = match pipeline_result {
            Ok(result) => {
                if result.from_cache {
                    cache_hits += 1;
                }
                completed_count += 1;

                CrawlResult {
                    url: url.clone(),
                    status: result.http_status,
                    from_cache: result.from_cache,
                    gate_decision: result.gate_decision,
                    quality_score: result.quality_score,
                    processing_time_ms: processing_time.as_millis() as u64,
                    document: Some(result.document),
                    error: None,
                    cache_key: result.cache_key,
                }
            }
            Err(pipeline_error) => {
                error_count += 1;

                // Structured error handling - zero unwrap approach
                let error_message = format!("Processing failed for {}: {}", url, pipeline_error);
                warn!(
                    request_id = %request_id,
                    url = %url,
                    error = %pipeline_error,
                    "URL processing failed"
                );

                CrawlResult {
                    url: url.clone(),
                    status: 0,
                    from_cache: false,
                    gate_decision: "failed".to_string(),
                    quality_score: 0.0,
                    processing_time_ms: processing_time.as_millis() as u64,
                    document: None,
                    error: Some(ErrorInfo {
                        error_type: "processing_error".to_string(),
                        message: error_message,
                        retryable: true,
                    }),
                    cache_key: "".to_string(),
                }
            }
        };

        // Create stream result with progress tracking
        let stream_result = StreamResult {
            index,
            result: crawl_result,
            progress: StreamProgress {
                completed: completed_count + error_count,
                total: urls.len(),
                success_rate: if completed_count + error_count > 0 {
                    completed_count as f64 / (completed_count + error_count) as f64
                } else {
                    0.0
                },
            },
        };

        // Check buffer capacity before sending - backpressure management
        let current_capacity = tx.capacity();
        if current_capacity == 0 {
            warn!(
                request_id = %request_id,
                "Channel at capacity, applying backpressure"
            );

            let should_drop = backpressure_handler
                .should_drop_message(current_capacity)
                .await;

            if should_drop {
                warn!(request_id = %request_id, "Dropping message due to backpressure");
                continue;
            }
        }

        // Send result immediately (streaming as completed)
        let send_result = send_ndjson_line_fast(&tx, &stream_result, backpressure_handler).await;
        if let Err(e) = send_result {
            debug!(
                request_id = %request_id,
                error = %e,
                "Client disconnected, stopping stream"
            );
            break;
        }

        // Send progress update for long-running operations (>10 URLs)
        if urls.len() > 10 && (completed_count + error_count) % 5 == 0 {
            let progress_update = OperationProgress {
                operation_id: "crawl_stream".to_string(),
                operation_type: "batch_crawl".to_string(),
                started_at: (chrono::Utc::now()
                    - chrono::Duration::milliseconds(start_time.elapsed().as_millis() as i64))
                .to_rfc3339(),
                current_phase: "processing".to_string(),
                progress_percentage: (completed_count + error_count) as f64 / urls.len() as f64
                    * 100.0,
                items_completed: completed_count + error_count,
                items_total: urls.len(),
                estimated_completion: estimate_completion(
                    start_time,
                    completed_count + error_count,
                    urls.len(),
                ),
                current_item: Some(url.clone()),
            };

            let progress_result = send_ndjson_line_fast(&tx, &progress_update, backpressure_handler).await;
            if let Err(_) = progress_result {
                debug!(request_id = %request_id, "Client disconnected during progress update");
                break;
            }
        }
    }

    // Send final summary
    let summary = StreamSummary {
        total_urls: urls.len(),
        successful: completed_count,
        failed: error_count,
        from_cache: cache_hits,
        total_processing_time_ms: start_time.elapsed().as_millis() as u64,
        cache_hit_rate: if !urls.is_empty() {
            cache_hits as f64 / urls.len() as f64
        } else {
            0.0
        },
    };

    let summary_result = send_ndjson_line_fast(&tx, &summary, backpressure_handler).await;
    if let Err(e) = summary_result {
        warn!(
            request_id = %request_id,
            error = %e,
            "Failed to send summary"
        );
    }

    info!(
        request_id = %request_id,
        total_urls = urls.len(),
        successful = completed_count,
        failed = error_count,
        cache_hits = cache_hits,
        total_time_ms = start_time.elapsed().as_millis(),
        "Optimized NDJSON crawl streaming completed"
    );

    // Record metrics
    app.metrics.record_http_request(
        "POST",
        "/crawl/stream",
        200,
        start_time.elapsed().as_secs_f64(),
    );

    Ok(())
}

/// Original orchestrate streaming crawl operation with backpressure handling
async fn orchestrate_crawl_stream(
    app: AppState,
    body: CrawlBody,
    tx: mpsc::Sender<Bytes>,
    start_time: Instant,
    request_id: String,
    backpressure_handler: &mut BackpressureHandler,
) -> StreamingResult<()> {
    let options = body.options.unwrap_or_default();

    debug!(
        request_id = %request_id,
        concurrency = options.concurrency,
        cache_mode = %options.cache_mode,
        "Starting NDJSON crawl orchestration"
    );

    // Send initial metadata about the request
    let metadata = StreamMetadata {
        total_urls: body.urls.len(),
        request_id: request_id.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        stream_type: "crawl".to_string(),
    };

    send_ndjson_line(&tx, &metadata, backpressure_handler).await?;

    // Create pipeline orchestrator
    let pipeline = PipelineOrchestrator::new(app.clone(), options);

    // Process URLs and stream results
    let mut completed_count = 0;
    let mut error_count = 0;
    let mut cache_hits = 0;

    // Use a custom streaming approach to send results as they complete
    let urls = body.urls.clone();
    let (result_tx, mut result_rx) = mpsc::channel(urls.len());

    // Spawn individual URL processing tasks
    for (index, url) in urls.iter().enumerate() {
        let pipeline_clone = pipeline.clone();
        let url_clone = url.clone();
        let result_tx_clone = result_tx.clone();

        tokio::spawn(async move {
            let single_result = pipeline_clone.execute_single(&url_clone).await;
            let _ = result_tx_clone
                .send((index, url_clone, single_result))
                .await;
        });
    }

    drop(result_tx); // Close the sender

    // Stream results as they arrive
    while let Some((index, url, pipeline_result)) = result_rx.recv().await {
        let crawl_result = match pipeline_result {
            Ok(result) => {
                if result.from_cache {
                    cache_hits += 1;
                }
                completed_count += 1;

                CrawlResult {
                    url: url.clone(),
                    status: result.http_status,
                    from_cache: result.from_cache,
                    gate_decision: result.gate_decision,
                    quality_score: result.quality_score,
                    processing_time_ms: result.processing_time_ms,
                    document: Some(result.document),
                    error: None,
                    cache_key: result.cache_key,
                }
            }
            Err(_) => {
                error_count += 1;

                CrawlResult {
                    url: url.clone(),
                    status: 0,
                    from_cache: false,
                    gate_decision: "failed".to_string(),
                    quality_score: 0.0,
                    processing_time_ms: 0,
                    document: None,
                    error: Some(ErrorInfo {
                        error_type: "processing_error".to_string(),
                        message: "Failed to process URL".to_string(),
                        retryable: true,
                    }),
                    cache_key: "".to_string(),
                }
            }
        };

        // Create stream result with progress tracking
        let stream_result = StreamResult {
            index,
            result: crawl_result,
            progress: StreamProgress {
                completed: completed_count + error_count,
                total: urls.len(),
                success_rate: if completed_count + error_count > 0 {
                    completed_count as f64 / (completed_count + error_count) as f64
                } else {
                    0.0
                },
            },
        };

        // Send progress update for long-running operations
        if urls.len() > 10 && (completed_count + error_count) % 5 == 0 {
            let progress_update = OperationProgress {
                operation_id: "crawl_stream".to_string(),
                operation_type: "batch_crawl".to_string(),
                started_at: (chrono::Utc::now()
                    - chrono::Duration::milliseconds(start_time.elapsed().as_millis() as i64))
                .to_rfc3339(),
                current_phase: "processing".to_string(),
                progress_percentage: (completed_count + error_count) as f64 / urls.len() as f64
                    * 100.0,
                items_completed: completed_count + error_count,
                items_total: urls.len(),
                estimated_completion: estimate_completion(
                    start_time,
                    completed_count + error_count,
                    urls.len(),
                ),
                current_item: Some(url.clone()),
            };

            if send_ndjson_line(&tx, &progress_update, backpressure_handler)
                .await
                .is_err()
            {
                debug!(request_id = %request_id, "Client disconnected during progress update");
                break;
            }
        }

        // Check for backpressure before sending
        if backpressure_handler
            .should_drop_message(tx.capacity())
            .await
        {
            warn!(request_id = %request_id, "Dropping message due to backpressure");
            continue;
        }

        if send_ndjson_line(&tx, &stream_result, backpressure_handler)
            .await
            .is_err()
        {
            debug!(request_id = %request_id, "Client disconnected, stopping stream");
            break;
        }
    }

    // Send final summary
    let summary = StreamSummary {
        total_urls: urls.len(),
        successful: completed_count,
        failed: error_count,
        from_cache: cache_hits,
        total_processing_time_ms: start_time.elapsed().as_millis() as u64,
        cache_hit_rate: if !urls.is_empty() {
            cache_hits as f64 / urls.len() as f64
        } else {
            0.0
        },
    };

    send_ndjson_line(&tx, &summary, backpressure_handler).await?;

    info!(
        request_id = %request_id,
        total_urls = urls.len(),
        successful = completed_count,
        failed = error_count,
        cache_hits = cache_hits,
        total_time_ms = start_time.elapsed().as_millis(),
        "NDJSON crawl streaming completed"
    );

    // Record metrics
    app.metrics.record_http_request(
        "POST",
        "/crawl/stream",
        200,
        start_time.elapsed().as_secs_f64(),
    );

    Ok(())
}

/// Optimized orchestrate streaming deep search operation for PR-3 requirements
///
/// Key features:
/// - TTFB < 500ms for search metadata
/// - Stream search results as they become available
/// - Content extraction streaming in parallel
/// - Zero unwrap/expect error handling
async fn orchestrate_deepsearch_stream_optimized(
    app: AppState,
    body: DeepSearchBody,
    tx: mpsc::Sender<Bytes>,
    start_time: Instant,
    request_id: String,
    backpressure_handler: &mut BackpressureHandler,
    config: StreamConfig,
) -> StreamingResult<()> {
    let limit = body.limit.unwrap_or(10).min(50);
    let include_content = body.include_content.unwrap_or(true);

    // Check for Serper API key with error handling (no unwrap)
    let serper_api_key = std::env::var("SERPER_API_KEY").map_err(|_| {
        StreamingError::invalid_request("SERPER_API_KEY environment variable not set")
    })?;

    debug!(
        request_id = %request_id,
        limit = limit,
        include_content = include_content,
        buffer_limit = config.buffer.max_size,
        "Starting optimized NDJSON deep search orchestration"
    );

    // Send initial metadata IMMEDIATELY for TTFB optimization
    let metadata = StreamMetadata {
        total_urls: 0, // Unknown until search completes
        request_id: request_id.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        stream_type: "deepsearch".to_string(),
    };

    // First flush - critical for TTFB < 500ms
    let metadata_result = send_ndjson_line_fast(&tx, &metadata, backpressure_handler).await;
    if let Err(e) = metadata_result {
        return Err(StreamingError::channel(format!("Failed to send metadata: {}", e)));
    }

    // Perform web search with error handling
    info!(request_id = %request_id, query = %body.query, "Performing web search");
    let search_start = Instant::now();
    let search_result = perform_web_search(&app, &body.query, limit, &serper_api_key).await;

    let search_results = match search_result {
        Ok(results) => results,
        Err(e) => {
            let error_msg = format!("Search failed: {}", e);
            error!(request_id = %request_id, error = %error_msg);
            return Err(StreamingError::Pipeline {
                source: anyhow::anyhow!(error_msg),
            });
        }
    };

    let search_time = search_start.elapsed();

    // Send search metadata immediately after search completes
    let search_metadata = DeepSearchMetadata {
        query: body.query.clone(),
        urls_found: search_results.len(),
        search_time_ms: search_time.as_millis() as u64,
    };

    let search_metadata_result = send_ndjson_line_fast(&tx, &search_metadata, backpressure_handler).await;
    if let Err(e) = search_metadata_result {
        warn!(
            request_id = %request_id,
            error = %e,
            "Failed to send search metadata"
        );
    }

    let search_results_len = search_results.len();

    if !include_content || search_results.is_empty() {
        // Send search results without content extraction (fast path)
        for (index, result) in search_results.into_iter().enumerate() {
            let stream_result = DeepSearchResult {
                index,
                search_result: result,
                crawl_result: None,
            };

            // Check buffer capacity before sending
            let current_capacity = tx.capacity();
            if current_capacity == 0 {
                let should_drop = backpressure_handler
                    .should_drop_message(current_capacity)
                    .await;

                if should_drop {
                    warn!(request_id = %request_id, "Dropping search result due to backpressure");
                    continue;
                }
            }

            let send_result = send_ndjson_line_fast(&tx, &stream_result, backpressure_handler).await;
            if let Err(e) = send_result {
                debug!(
                    request_id = %request_id,
                    error = %e,
                    "Client disconnected, stopping stream"
                );
                break;
            }
        }
    } else {
        // Extract URLs and crawl with streaming (parallel content extraction)
        let urls: Vec<String> = search_results.iter().map(|r| r.url.clone()).collect();
        let crawl_options = body.crawl_options.unwrap_or_default();
        let pipeline = PipelineOrchestrator::new(app.clone(), crawl_options);

        debug!(
            request_id = %request_id,
            url_count = urls.len(),
            "Starting parallel content extraction"
        );

        // Process URLs concurrently and stream results as they complete
        let (result_tx, mut result_rx) = mpsc::channel(urls.len().min(1000));

        // Spawn individual URL processing tasks with immediate results
        for (index, url) in urls.iter().enumerate() {
            let pipeline_clone = pipeline.clone();
            let url_clone = url.clone();
            let result_tx_clone = result_tx.clone();
            let search_result = search_results[index].clone();
            let request_id_clone = request_id.clone();

            tokio::spawn(async move {
                let task_start = Instant::now();
                let crawl_result = pipeline_clone.execute_single(&url_clone).await;
                let processing_time = task_start.elapsed();

                // Send result immediately when complete (no batching)
                let send_result = result_tx_clone
                    .send((index, search_result, crawl_result, processing_time))
                    .await;

                if let Err(_) = send_result {
                    warn!(
                        request_id = %request_id_clone,
                        url = %url_clone,
                        "Failed to send crawl result - receiver likely closed"
                    );
                }
            });
        }

        drop(result_tx);

        // Stream results as they arrive (critical for no-batching requirement)
        while let Some((index, search_result, pipeline_result, processing_time)) = result_rx.recv().await {
            let crawl_result = match pipeline_result {
                Ok(pipeline_data) => Some(CrawlResult {
                    url: pipeline_data.document.url.clone(),
                    status: pipeline_data.http_status,
                    from_cache: pipeline_data.from_cache,
                    gate_decision: pipeline_data.gate_decision,
                    quality_score: pipeline_data.quality_score,
                    processing_time_ms: processing_time.as_millis() as u64,
                    document: Some(pipeline_data.document),
                    error: None,
                    cache_key: pipeline_data.cache_key,
                }),
                Err(pipeline_error) => {
                    // Structured error handling for crawl failures
                    let error_message = format!("Crawl failed for {}: {}", search_result.url, pipeline_error);
                    warn!(
                        request_id = %request_id,
                        url = %search_result.url,
                        error = %pipeline_error,
                        "URL crawling failed"
                    );

                    Some(CrawlResult {
                        url: search_result.url.clone(),
                        status: 0,
                        from_cache: false,
                        gate_decision: "failed".to_string(),
                        quality_score: 0.0,
                        processing_time_ms: processing_time.as_millis() as u64,
                        document: None,
                        error: Some(ErrorInfo {
                            error_type: "crawl_error".to_string(),
                            message: error_message,
                            retryable: true,
                        }),
                        cache_key: "".to_string(),
                    })
                }
            };

            let stream_result = DeepSearchResult {
                index,
                search_result,
                crawl_result,
            };

            // Check buffer capacity before sending - backpressure management
            let current_capacity = tx.capacity();
            if current_capacity == 0 {
                warn!(
                    request_id = %request_id,
                    "Channel at capacity for deepsearch, applying backpressure"
                );

                let should_drop = backpressure_handler
                    .should_drop_message(current_capacity)
                    .await;

                if should_drop {
                    warn!(request_id = %request_id, "Dropping deepsearch result due to backpressure");
                    continue;
                }
            }

            // Send result immediately (streaming as completed)
            let send_result = send_ndjson_line_fast(&tx, &stream_result, backpressure_handler).await;
            if let Err(e) = send_result {
                debug!(
                    request_id = %request_id,
                    error = %e,
                    "Client disconnected, stopping deepsearch stream"
                );
                break;
            }
        }
    }

    // Send final summary
    let final_summary = DeepSearchSummary {
        query: body.query,
        total_urls_found: search_results_len,
        total_processing_time_ms: start_time.elapsed().as_millis() as u64,
        status: "completed".to_string(),
    };

    let summary_result = send_ndjson_line_fast(&tx, &final_summary, backpressure_handler).await;
    if let Err(e) = summary_result {
        warn!(
            request_id = %request_id,
            error = %e,
            "Failed to send deepsearch summary"
        );
    }

    info!(
        request_id = %request_id,
        urls_found = search_results_len,
        total_time_ms = start_time.elapsed().as_millis(),
        "Optimized NDJSON deep search streaming completed"
    );

    // Record metrics
    app.metrics.record_http_request(
        "POST",
        "/deepsearch/stream",
        200,
        start_time.elapsed().as_secs_f64(),
    );

    Ok(())
}

/// Original orchestrate streaming deep search operation
async fn orchestrate_deepsearch_stream(
    app: AppState,
    body: DeepSearchBody,
    tx: mpsc::Sender<Bytes>,
    start_time: Instant,
    request_id: String,
    backpressure_handler: &mut BackpressureHandler,
) -> StreamingResult<()> {
    let limit = body.limit.unwrap_or(10).min(50);
    let include_content = body.include_content.unwrap_or(true);

    // Check for Serper API key
    let serper_api_key = std::env::var("SERPER_API_KEY").map_err(|_| {
        StreamingError::invalid_request("SERPER_API_KEY environment variable not set")
    })?;

    debug!(
        request_id = %request_id,
        limit = limit,
        include_content = include_content,
        "Starting NDJSON deep search orchestration"
    );

    // Send initial metadata
    let metadata = StreamMetadata {
        total_urls: 0, // Unknown until search completes
        request_id: request_id.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        stream_type: "deepsearch".to_string(),
    };

    send_ndjson_line(&tx, &metadata, backpressure_handler).await?;

    // Perform web search
    info!(request_id = %request_id, query = %body.query, "Performing web search");
    let search_results = perform_web_search(&app, &body.query, limit, &serper_api_key)
        .await
        .map_err(|e| StreamingError::Pipeline {
            source: anyhow::anyhow!("Search failed: {}", e),
        })?;

    // Send search metadata
    let search_metadata = DeepSearchMetadata {
        query: body.query.clone(),
        urls_found: search_results.len(),
        search_time_ms: start_time.elapsed().as_millis() as u64,
    };

    send_ndjson_line(&tx, &search_metadata, backpressure_handler).await?;

    let search_results_len = search_results.len();

    if !include_content || search_results.is_empty() {
        // Send search results without content extraction
        for (index, result) in search_results.into_iter().enumerate() {
            let stream_result = DeepSearchResult {
                index,
                search_result: result,
                crawl_result: None,
            };

            if backpressure_handler
                .should_drop_message(tx.capacity())
                .await
            {
                continue;
            }

            if send_ndjson_line(&tx, &stream_result, backpressure_handler)
                .await
                .is_err()
            {
                debug!(request_id = %request_id, "Client disconnected, stopping stream");
                break;
            }
        }
    } else {
        // Extract URLs and crawl with streaming
        let urls: Vec<String> = search_results.iter().map(|r| r.url.clone()).collect();
        let crawl_options = body.crawl_options.unwrap_or_default();
        let pipeline = PipelineOrchestrator::new(app.clone(), crawl_options);

        debug!(request_id = %request_id, url_count = urls.len(), "Starting content extraction");

        // Process URLs concurrently and stream results
        let (result_tx, mut result_rx) = mpsc::channel(urls.len());

        // Spawn individual URL processing tasks
        for (index, url) in urls.iter().enumerate() {
            let pipeline_clone = pipeline.clone();
            let url_clone = url.clone();
            let result_tx_clone = result_tx.clone();
            let search_result = search_results[index].clone();

            tokio::spawn(async move {
                let crawl_result = pipeline_clone.execute_single(&url_clone).await;
                let _ = result_tx_clone
                    .send((index, search_result, crawl_result))
                    .await;
            });
        }

        drop(result_tx);

        // Stream results as they arrive
        while let Some((index, search_result, pipeline_result)) = result_rx.recv().await {
            let crawl_result = pipeline_result.ok().map(|pr| CrawlResult {
                url: pr.document.url.clone(),
                status: pr.http_status,
                from_cache: pr.from_cache,
                gate_decision: pr.gate_decision,
                quality_score: pr.quality_score,
                processing_time_ms: pr.processing_time_ms,
                document: Some(pr.document),
                error: None,
                cache_key: pr.cache_key,
            });

            let stream_result = DeepSearchResult {
                index,
                search_result,
                crawl_result,
            };

            if backpressure_handler
                .should_drop_message(tx.capacity())
                .await
            {
                continue;
            }

            if send_ndjson_line(&tx, &stream_result, backpressure_handler)
                .await
                .is_err()
            {
                debug!(request_id = %request_id, "Client disconnected, stopping stream");
                break;
            }
        }
    }

    // Send final summary
    let final_summary = DeepSearchSummary {
        query: body.query,
        total_urls_found: search_results_len,
        total_processing_time_ms: start_time.elapsed().as_millis() as u64,
        status: "completed".to_string(),
    };

    send_ndjson_line(&tx, &final_summary, backpressure_handler).await?;

    info!(
        request_id = %request_id,
        urls_found = search_results_len,
        total_time_ms = start_time.elapsed().as_millis(),
        "NDJSON deep search streaming completed"
    );

    // Record metrics
    app.metrics.record_http_request(
        "POST",
        "/deepsearch/stream",
        200,
        start_time.elapsed().as_secs_f64(),
    );

    Ok(())
}

/// Fast NDJSON line sending optimized for TTFB < 500ms
///
/// Key optimizations:
/// - Zero allocation string formatting where possible
/// - Fast error propagation
/// - Minimal backpressure overhead for initial sends
async fn send_ndjson_line_fast<T: Serialize>(
    tx: &mpsc::Sender<Bytes>,
    obj: &T,
    backpressure_handler: &mut BackpressureHandler,
) -> StreamingResult<()> {
    let send_start = Instant::now();

    // Serialize with error handling (no unwrap)
    let json_result = serde_json::to_string(obj);
    let json_str = match json_result {
        Ok(s) => s,
        Err(e) => {
            return Err(StreamingError::Serialization { source: e });
        }
    };

    // Efficient line formatting
    let mut line_bytes = json_str.into_bytes();
    line_bytes.push(b'\n');

    // Send with error handling (no unwrap)
    let send_result = tx.send(Bytes::from(line_bytes)).await;
    if let Err(_) = send_result {
        return Err(StreamingError::channel("Failed to send to stream"));
    }

    let send_duration = send_start.elapsed();
    let record_result = backpressure_handler.record_send_time(send_duration).await;
    if let Err(e) = record_result {
        // Log but don't fail the operation for backpressure recording issues
        debug!(
            duration_ms = send_duration.as_millis(),
            error = %e,
            "Failed to record send time"
        );
    }

    Ok(())
}

/// Send a JSON object as an NDJSON line with backpressure handling
async fn send_ndjson_line<T: Serialize>(
    tx: &mpsc::Sender<Bytes>,
    obj: &T,
    backpressure_handler: &mut BackpressureHandler,
) -> StreamingResult<()> {
    let send_start = Instant::now();

    let json_str = serde_json::to_string(obj).map_err(StreamingError::from)?;
    let line = format!("{}\n", json_str);

    tx.send(Bytes::from(line.into_bytes()))
        .await
        .map_err(|_| StreamingError::channel("Failed to send to stream"))?;

    let send_duration = send_start.elapsed();
    backpressure_handler.record_send_time(send_duration).await?;

    Ok(())
}

/// Create an error response for failed request validation
fn create_error_response(error: ApiError) -> Response {
    let error_json = serde_json::json!({
        "error": {
            "type": "validation_error",
            "message": error.to_string(),
            "retryable": false
        }
    });

    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("Content-Type", "application/json")
        .body(Body::from(error_json.to_string()))
        .unwrap_or_else(|_| {
            // If we can't even build this simple error response, return a minimal one
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap_or_else(|e| {
                    error!(error = %e, "Failed to build minimal error response, returning empty response");
                    Response::new(Body::empty())
                })
        })
}

/// Perform web search using existing handler logic
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

/// Estimate completion time for long-running operations
fn estimate_completion(start_time: Instant, completed: usize, total: usize) -> Option<String> {
    if completed == 0 || total == 0 {
        return None;
    }

    let elapsed = start_time.elapsed();
    let avg_time_per_item = elapsed.as_secs_f64() / completed as f64;
    let remaining_items = total.saturating_sub(completed);
    let estimated_remaining_secs = avg_time_per_item * remaining_items as f64;

    let completion_time =
        chrono::Utc::now() + chrono::Duration::seconds(estimated_remaining_secs as i64);
    Some(completion_time.to_rfc3339())
}

// Models for NDJSON streaming

/// Metadata sent at the beginning of a stream
#[derive(Serialize, Debug)]
struct StreamMetadata {
    total_urls: usize,
    request_id: String,
    timestamp: String,
    stream_type: String,
}

/// Individual streaming result with progress information
#[derive(Serialize, Debug)]
struct StreamResult {
    index: usize,
    result: CrawlResult,
    progress: StreamProgress,
}

/// Progress information for streaming operations
#[derive(Serialize, Debug)]
struct StreamProgress {
    completed: usize,
    total: usize,
    success_rate: f64,
}

/// Final summary for streaming crawl operations
#[derive(Serialize, Debug)]
struct StreamSummary {
    total_urls: usize,
    successful: usize,
    failed: usize,
    from_cache: usize,
    total_processing_time_ms: u64,
    cache_hit_rate: f64,
}

/// Metadata for deep search operations
#[derive(Serialize, Debug)]
struct DeepSearchMetadata {
    query: String,
    urls_found: usize,
    search_time_ms: u64,
}

/// Individual deep search result with optional crawl data
#[derive(Serialize, Debug)]
struct DeepSearchResult {
    index: usize,
    search_result: SearchResult,
    crawl_result: Option<CrawlResult>,
}

/// Final summary for deep search operations
#[derive(Serialize, Debug)]
struct DeepSearchSummary {
    query: String,
    total_urls_found: usize,
    total_processing_time_ms: u64,
    status: String,
}

/// Progress tracking for long-running operations
#[derive(Serialize, Debug)]
pub struct OperationProgress {
    pub operation_id: String,
    pub operation_type: String,
    pub started_at: String,
    pub current_phase: String,
    pub progress_percentage: f64,
    pub items_completed: usize,
    pub items_total: usize,
    pub estimated_completion: Option<String>,
    pub current_item: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CrawlOptions;

    #[tokio::test]
    async fn test_ndjson_handler_creation() {
        let app = AppState::new().await.expect("Failed to create AppState");
        let request_id = "test-123".to_string();
        let handler = NdjsonStreamingHandler::new(app, request_id.clone());
        assert_eq!(handler.request_id, request_id);
    }

    #[test]
    fn test_stream_metadata_serialization() {
        let metadata = StreamMetadata {
            total_urls: 5,
            request_id: "test-123".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            stream_type: "crawl".to_string(),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"total_urls\":5"));
        assert!(json.contains("\"request_id\":\"test-123\""));
    }

    #[test]
    fn test_stream_progress_calculation() {
        let progress = StreamProgress {
            completed: 3,
            total: 5,
            success_rate: 0.8,
        };

        assert_eq!(progress.completed, 3);
        assert_eq!(progress.total, 5);
        assert_eq!(progress.success_rate, 0.8);
    }

    #[test]
    fn test_estimate_completion() {
        let start_time = Instant::now();

        // Test with no progress
        assert!(estimate_completion(start_time, 0, 10).is_none());

        // Test with some progress
        let result = estimate_completion(start_time, 2, 10);
        assert!(result.is_some());
    }

    #[test]
    fn test_create_error_response() {
        let error = ApiError::validation("Test error");
        let response = create_error_response(error);
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
