//! Server-Sent Events (SSE) streaming implementation.
//!
//! This module provides SSE endpoints for real-time streaming with automatic
//! keep-alive, reconnection handling, and proper event formatting.

use super::buffer::{BackpressureHandler, BufferManager};
use super::config::StreamConfig;
use super::error::{StreamingError, StreamingResult};
use super::response_helpers::StreamingErrorResponse;
use crate::models::*;
use crate::pipeline::PipelineOrchestrator;
use crate::state::AppState;
use crate::validation::validate_crawl_request;
use axum::extract::{Json, State};
use axum::response::sse::{Event, KeepAlive};
use axum::response::{IntoResponse, Response, Sse};
use serde::Serialize;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// SSE (Server-Sent Events) endpoint for real-time crawl progress
pub async fn crawl_sse(State(app): State<AppState>, Json(body): Json<CrawlBody>) -> Response {
    let start_time = Instant::now();
    let request_id = Uuid::new_v4().to_string();

    info!(
        request_id = %request_id,
        url_count = body.urls.len(),
        cache_mode = body.options.as_ref().map(|o| &o.cache_mode),
        "Received SSE crawl request"
    );

    // Validate the request early to fail fast
    if let Err(e) = validate_crawl_request(&body) {
        let error_json = serde_json::json!({
            "error": {
                "type": "validation_error",
                "message": e.to_string(),
                "retryable": false
            }
        });
        return StreamingErrorResponse::sse(error_json).into_response();
    }

    // Create SSE handler
    let sse_handler = SseStreamingHandler::new(app.clone(), request_id.clone());

    match sse_handler.handle_crawl_stream(body, start_time).await {
        Ok(response) => response.into_response(),
        Err(e) => {
            let error_json = serde_json::json!({
                "error": {
                    "type": "streaming_error",
                    "message": e.to_string(),
                    "retryable": true
                }
            });
            StreamingErrorResponse::sse(error_json).into_response()
        }
    }
}

/// SSE streaming handler with connection management
pub struct SseStreamingHandler {
    app: AppState,
    request_id: String,
    buffer_manager: Arc<BufferManager>,
    config: StreamConfig,
}

impl SseStreamingHandler {
    /// Create a new SSE streaming handler
    pub fn new(app: AppState, request_id: String) -> Self {
        Self {
            app,
            request_id,
            buffer_manager: Arc::new(BufferManager::new()),
            config: StreamConfig::from_env(),
        }
    }

    /// Handle SSE crawl streaming
    pub async fn handle_crawl_stream(
        &self,
        body: CrawlBody,
        start_time: Instant,
    ) -> StreamingResult<impl IntoResponse> {
        let buffer = self.buffer_manager.get_buffer(&self.request_id).await;
        let (tx, rx) = buffer.create_channel::<Result<Event, Infallible>>().await;

        // Clone necessary data for the spawned task
        let app_clone = self.app.clone();
        let body_clone = body.clone();
        let request_id = self.request_id.clone();
        let buffer_clone = buffer.clone();
        let config = self.config.clone();

        // Spawn the SSE orchestration task
        tokio::spawn(async move {
            let mut backpressure_handler =
                BackpressureHandler::new(request_id.clone(), buffer_clone);

            let request_id_clone = request_id.clone();
            if let Err(e) = orchestrate_crawl_sse(
                app_clone,
                body_clone,
                tx,
                start_time,
                request_id,
                &mut backpressure_handler,
                config,
            )
            .await
            {
                error!(request_id = %request_id_clone, error = %e, "SSE orchestration error");
            }
        });

        // Return SSE response with keep-alive heartbeat (30s interval)
        // The heartbeat sends ":heartbeat\n" comments to keep connection alive
        Ok(Sse::new(ReceiverStream::new(rx))
            .keep_alive(
                KeepAlive::new()
                    .interval(Duration::from_secs(30)) // 30-second heartbeat interval
                    .text(":heartbeat"), // SSE comment format for heartbeat
            )
            .into_response())
    }
}

/// Orchestrate SSE crawl operation with proper event formatting
async fn orchestrate_crawl_sse(
    app: AppState,
    body: CrawlBody,
    tx: mpsc::Sender<Result<Event, Infallible>>,
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
        "Starting SSE crawl orchestration"
    );

    // Send initial metadata event
    let metadata = SseMetadata {
        total_urls: body.urls.len(),
        request_id: request_id.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        stream_type: "crawl".to_string(),
        retry_interval_ms: config.sse.retry_interval.as_millis() as u32,
    };

    send_sse_event(&tx, "metadata", &metadata, None, backpressure_handler).await?;

    // Create pipeline orchestrator
    let pipeline = PipelineOrchestrator::new(app.clone(), options);
    let (result_tx, mut result_rx) = mpsc::channel(body.urls.len());

    // Spawn individual URL processing tasks
    for (index, url) in body.urls.iter().enumerate() {
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

    drop(result_tx);

    // Stream results via SSE
    let mut completed_count = 0;
    let mut error_count = 0;
    let mut cache_hits = 0;

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

        // Send progress event
        let progress = SseProgress {
            completed: completed_count + error_count,
            total: body.urls.len(),
            success_rate: if completed_count + error_count > 0 {
                completed_count as f64 / (completed_count + error_count) as f64
            } else {
                0.0
            },
            cache_hit_rate: if completed_count > 0 {
                cache_hits as f64 / completed_count as f64
            } else {
                0.0
            },
        };

        send_sse_event(
            &tx,
            "progress",
            &progress,
            Some(index),
            backpressure_handler,
        )
        .await?;

        // Send result event
        let result_data = SseResult {
            index,
            result: crawl_result,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
        };

        send_sse_event(
            &tx,
            "result",
            &result_data,
            Some(index),
            backpressure_handler,
        )
        .await?;

        // Send periodic status updates for long operations
        if body.urls.len() > 20 && (completed_count + error_count) % 10 == 0 {
            let status = SseStatus {
                phase: "processing".to_string(),
                items_processed: completed_count + error_count,
                items_remaining: body.urls.len() - (completed_count + error_count),
                estimated_completion: estimate_completion(
                    start_time,
                    completed_count + error_count,
                    body.urls.len(),
                ),
                throughput_per_minute: calculate_throughput(
                    start_time,
                    completed_count + error_count,
                ),
            };

            send_sse_event(&tx, "status", &status, None, backpressure_handler).await?;
        }

        // Check if client is still connected (will fail if disconnected)
        if tx.is_closed() {
            debug!(request_id = %request_id, "SSE client disconnected");
            break;
        }
    }

    // Send completion event
    let completion = SseCompletion {
        total_urls: body.urls.len(),
        successful: completed_count,
        failed: error_count,
        from_cache: cache_hits,
        total_processing_time_ms: start_time.elapsed().as_millis() as u64,
        cache_hit_rate: if !body.urls.is_empty() {
            cache_hits as f64 / body.urls.len() as f64
        } else {
            0.0
        },
        average_processing_time_ms: if completed_count > 0 {
            start_time.elapsed().as_millis() as f64 / completed_count as f64
        } else {
            0.0
        },
    };

    send_sse_event(&tx, "complete", &completion, None, backpressure_handler).await?;

    info!(
        request_id = %request_id,
        total_urls = body.urls.len(),
        successful = completed_count,
        failed = error_count,
        cache_hits = cache_hits,
        total_time_ms = start_time.elapsed().as_millis(),
        "SSE crawl streaming completed"
    );

    // Record metrics
    app.metrics.record_http_request(
        "POST",
        "/crawl/sse",
        200,
        start_time.elapsed().as_secs_f64(),
    );

    Ok(())
}

/// Send an SSE event with proper formatting and backpressure handling
///
/// SSE events support reconnection via Last-Event-ID header:
/// - Events with IDs can be resumed after disconnection
/// - Client sends Last-Event-ID header on reconnect
/// - Server resumes from ID + 1
async fn send_sse_event<T: Serialize>(
    tx: &mpsc::Sender<Result<Event, Infallible>>,
    event_type: &str,
    data: &T,
    id: Option<usize>,
    backpressure_handler: &mut BackpressureHandler,
) -> StreamingResult<()> {
    // Check for backpressure
    if backpressure_handler
        .should_drop_message(tx.capacity())
        .await
    {
        warn!("Dropping SSE event due to backpressure: {}", event_type);
        return Ok(());
    }

    let send_start = Instant::now();

    let data_str = serde_json::to_string(data).map_err(StreamingError::from)?;

    // Build SSE event with proper formatting:
    // - event: <type>
    // - data: <json>
    // - id: <number> (for reconnection support)
    // - retry: <ms> (client reconnect interval)
    let mut event = Event::default().event(event_type).data(data_str);

    // Add event ID for Last-Event-ID reconnection support
    // Client can resume from last received ID after disconnection
    if let Some(id) = id {
        event = event.id(id.to_string());
    }

    // Add retry information for certain event types
    // Tells client how long to wait before reconnecting
    if matches!(event_type, "metadata" | "complete") {
        event = event.retry(Duration::from_secs(5)); // 5-second retry interval
    }

    tx.send(Ok(event))
        .await
        .map_err(|_| StreamingError::channel("Failed to send SSE event"))?;

    let send_duration = send_start.elapsed();
    backpressure_handler.record_send_time(send_duration).await?;

    Ok(())
}

/// Calculate processing throughput
fn calculate_throughput(start_time: Instant, processed: usize) -> f64 {
    let elapsed_minutes = start_time.elapsed().as_secs_f64() / 60.0;
    if elapsed_minutes > 0.0 {
        processed as f64 / elapsed_minutes
    } else {
        0.0
    }
}

/// Estimate completion time
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

// SSE-specific data models

/// SSE metadata event
#[derive(Serialize, Debug)]
struct SseMetadata {
    total_urls: usize,
    request_id: String,
    timestamp: String,
    stream_type: String,
    retry_interval_ms: u32,
}

/// SSE progress event
#[derive(Serialize, Debug)]
struct SseProgress {
    completed: usize,
    total: usize,
    success_rate: f64,
    cache_hit_rate: f64,
}

/// SSE result event
#[derive(Serialize, Debug)]
struct SseResult {
    index: usize,
    result: CrawlResult,
    processing_time_ms: u64,
}

/// SSE status event
#[derive(Serialize, Debug)]
struct SseStatus {
    phase: String,
    items_processed: usize,
    items_remaining: usize,
    estimated_completion: Option<String>,
    throughput_per_minute: f64,
}

/// SSE completion event
#[derive(Serialize, Debug)]
struct SseCompletion {
    total_urls: usize,
    successful: usize,
    failed: usize,
    from_cache: usize,
    total_processing_time_ms: u64,
    cache_hit_rate: f64,
    average_processing_time_ms: f64,
}

// Note: SseMetrics is now imported from super::metrics
// The previous duplicate implementation (62 lines) has been removed
// and replaced with the shared StreamingMetrics from metrics.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming::metrics::SseMetrics;

    #[test]
    fn test_sse_metadata_serialization() {
        let metadata = SseMetadata {
            total_urls: 5,
            request_id: "test-123".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            stream_type: "crawl".to_string(),
            retry_interval_ms: 5000,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"total_urls\":5"));
        assert!(json.contains("\"retry_interval_ms\":5000"));
    }

    #[test]
    fn test_sse_progress_calculation() {
        let progress = SseProgress {
            completed: 8,
            total: 10,
            success_rate: 0.875,
            cache_hit_rate: 0.25,
        };

        assert_eq!(progress.completed, 8);
        assert_eq!(progress.total, 10);
        assert_eq!(progress.success_rate, 0.875);
        assert_eq!(progress.cache_hit_rate, 0.25);
    }

    #[test]
    fn test_calculate_throughput() {
        let start_time = Instant::now() - Duration::from_secs(120); // 2 minutes ago
        let throughput = calculate_throughput(start_time, 60);
        // Use approximate comparison for floating point (30 items per minute)
        assert!(
            (throughput - 30.0).abs() < 0.01,
            "Expected ~30.0, got {}",
            throughput
        );
    }

    #[test]
    fn test_estimate_completion() {
        let start_time = Instant::now() - Duration::from_secs(60);
        let result = estimate_completion(start_time, 3, 10);
        assert!(result.is_some());

        // Test edge cases
        assert!(estimate_completion(start_time, 0, 10).is_none());
        assert!(estimate_completion(start_time, 10, 0).is_none());
    }

    #[test]
    fn test_sse_metrics() {
        let mut metrics = SseMetrics::default();

        metrics.record_connection();
        assert_eq!(metrics.active_connections, 1);
        assert_eq!(metrics.total_connections, 1);

        metrics.record_item_sent(); // was record_event_sent
        metrics.record_item_sent();
        metrics.record_item_dropped(); // was record_event_dropped
        assert_eq!(metrics.total_items_sent, 2); // was total_events_sent
        assert_eq!(metrics.items_dropped, 1); // was events_dropped
        assert_eq!(metrics.delivery_ratio(), 2.0 / 3.0);

        metrics.record_reconnection();
        assert_eq!(metrics.reconnection_rate(), 1.0);

        metrics.record_disconnection(Duration::from_millis(30000));
        assert_eq!(metrics.active_connections, 0);
        assert_eq!(metrics.average_connection_duration_ms, 30000.0);
    }
}
