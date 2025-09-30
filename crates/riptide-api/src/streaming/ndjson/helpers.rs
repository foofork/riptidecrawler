//! Helper functions and utilities for NDJSON streaming.
//!
//! This module contains orchestration functions, serialization helpers, and web search utilities.

use super::progress::{estimate_completion, OperationProgress};
use crate::errors::{ApiError, ApiResult};
use crate::models::*;
use crate::pipeline::PipelineOrchestrator;
use crate::state::AppState;
use crate::streaming::buffer::BackpressureHandler;
use crate::streaming::config::StreamConfig;
use crate::streaming::error::{StreamingError, StreamingResult};
use bytes::Bytes;
use serde::Serialize;
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Optimized orchestrate streaming crawl operation for PR-3 requirements
///
/// Key features:
/// - TTFB < 500ms optimization
/// - Stream results as they complete (no batching)
/// - Zero unwrap/expect error handling
/// - Buffer management with specified limits
pub async fn orchestrate_crawl_stream_optimized(
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

            if send_result.is_err() {
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

/// Optimized orchestrate streaming deep search operation for PR-3 requirements
///
/// Key features:
/// - TTFB < 500ms for search metadata
/// - Stream search results as they become available
/// - Content extraction streaming in parallel
/// - Zero unwrap/expect error handling
pub async fn orchestrate_deepsearch_stream_optimized(
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

                if send_result.is_err() {
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