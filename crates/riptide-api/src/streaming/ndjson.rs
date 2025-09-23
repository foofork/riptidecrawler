//! NDJSON (Newline Delimited JSON) streaming implementation.
//!
//! This module provides streaming JSON responses where each line contains
//! a complete JSON object, allowing for efficient streaming of large datasets.

use super::buffer::{BackpressureHandler, BufferManager};
use super::config::StreamConfig;
use super::error::{StreamingError, StreamingResult};
use crate::errors::{ApiError, ApiResult};
use crate::models::*;
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

/// NDJSON streaming handler for crawl operations
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

    // Validate the request early to fail fast
    if let Err(e) = validate_crawl_request(&body) {
        return create_error_response(e);
    }

    // Create streaming handler
    let streaming_handler = NdjsonStreamingHandler::new(app.clone(), request_id.clone());

    match streaming_handler
        .handle_crawl_stream(body, start_time)
        .await
    {
        Ok(response) => response,
        Err(e) => create_error_response(ApiError::from(e)),
    }
}

/// NDJSON streaming handler for deep search operations
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

    // Validate the request early to fail fast
    if let Err(e) = validate_deepsearch_request(&body) {
        return create_error_response(e);
    }

    // Create streaming handler
    let streaming_handler = NdjsonStreamingHandler::new(app.clone(), request_id.clone());

    match streaming_handler
        .handle_deepsearch_stream(body, start_time)
        .await
    {
        Ok(response) => response,
        Err(e) => create_error_response(ApiError::from(e)),
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

        // Spawn the streaming orchestration task
        tokio::spawn(async move {
            let mut backpressure_handler =
                BackpressureHandler::new(request_id.clone(), buffer_clone);

            if let Err(e) = orchestrate_crawl_stream(
                app_clone,
                body_clone,
                tx,
                start_time,
                request_id,
                &mut backpressure_handler,
            )
            .await
            {
                error!(request_id = %request_id, error = %e, "NDJSON crawl stream orchestration error");
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
            ))?)
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

        // Spawn the streaming orchestration task
        tokio::spawn(async move {
            let mut backpressure_handler =
                BackpressureHandler::new(request_id.clone(), buffer_clone);

            if let Err(e) = orchestrate_deepsearch_stream(
                app_clone,
                body_clone,
                tx,
                start_time,
                request_id,
                &mut backpressure_handler,
            )
            .await
            {
                error!(request_id = %request_id, error = %e, "NDJSON deep search stream orchestration error");
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
            ))?)
    }
}

/// Orchestrate streaming crawl operation with backpressure handling
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

/// Orchestrate streaming deep search operation
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
                .expect("Failed to build minimal error response")
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
