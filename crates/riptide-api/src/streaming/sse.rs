//! Server-Sent Events (SSE) streaming implementation.
//!
//! This module provides SSE endpoints for real-time streaming with automatic
//! keep-alive, reconnection handling, and proper event formatting.

use super::buffer::{BackpressureHandler, BufferManager};
use super::config::StreamConfig;
use super::error::{StreamingError, StreamingResult};
use crate::errors::ApiError;
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
pub async fn crawl_sse(
    State(app): State<AppState>,
    Json(body): Json<CrawlBody>,
) -> Response {
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
        return create_sse_error_response(e).into_response();
    }

    // Create SSE handler
    let sse_handler = SseStreamingHandler::new(app.clone(), request_id.clone());

    match sse_handler.handle_crawl_stream(body, start_time).await {
        Ok(response) => response.into_response(),
        Err(e) => create_sse_error_response(ApiError::from(e)).into_response(),
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

        // Return SSE response with keep-alive
        Ok(Sse::new(ReceiverStream::new(rx))
            .keep_alive(
                KeepAlive::new()
                    .interval(self.config.sse.keep_alive_interval)
                    .text("keep-alive"),
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

    let mut event = Event::default().event(event_type).data(data_str);

    if let Some(id) = id {
        event = event.id(id.to_string());
    }

    // Add retry information for certain event types
    if matches!(event_type, "metadata" | "complete") {
        event = event.retry(Duration::from_secs(5));
    }

    tx.send(Ok(event))
        .await
        .map_err(|_| StreamingError::channel("Failed to send SSE event"))?;

    let send_duration = send_start.elapsed();
    backpressure_handler.record_send_time(send_duration).await?;

    Ok(())
}

/// Create an SSE error response
fn create_sse_error_response(error: ApiError) -> impl IntoResponse {
    let error_data = serde_json::json!({
        "error": {
            "type": "validation_error",
            "message": error.to_string(),
            "retryable": false
        }
    });

    let error_event = Event::default().event("error").data(error_data.to_string());

    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(1);

    // Send error event and close
    tokio::spawn(async move {
        let _ = tx.send(Ok(error_event)).await;
    });

    Sse::new(ReceiverStream::new(rx)).into_response()
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

/// SSE connection metrics
#[derive(Debug, Default)]
pub struct SseMetrics {
    pub active_connections: usize,
    pub total_connections: usize,
    pub total_events_sent: usize,
    pub events_dropped: usize,
    pub average_connection_duration_ms: f64,
    pub reconnection_count: usize,
}

impl SseMetrics {
    /// Record a new SSE connection
    pub fn record_connection(&mut self) {
        self.active_connections += 1;
        self.total_connections += 1;
    }

    /// Record SSE connection closure
    pub fn record_disconnection(&mut self, duration: Duration) {
        self.active_connections = self.active_connections.saturating_sub(1);

        // Update average duration
        let total_duration =
            self.average_connection_duration_ms * (self.total_connections - 1) as f64;
        self.average_connection_duration_ms =
            (total_duration + duration.as_millis() as f64) / self.total_connections as f64;
    }

    /// Record event sent
    pub fn record_event_sent(&mut self) {
        self.total_events_sent += 1;
    }

    /// Record event dropped
    pub fn record_event_dropped(&mut self) {
        self.events_dropped += 1;
    }

    /// Record client reconnection
    pub fn record_reconnection(&mut self) {
        self.reconnection_count += 1;
    }

    /// Get event delivery ratio
    pub fn delivery_ratio(&self) -> f64 {
        let total_events = self.total_events_sent + self.events_dropped;
        if total_events == 0 {
            1.0
        } else {
            self.total_events_sent as f64 / total_events as f64
        }
    }

    /// Get reconnection rate
    pub fn reconnection_rate(&self) -> f64 {
        if self.total_connections == 0 {
            0.0
        } else {
            self.reconnection_count as f64 / self.total_connections as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(throughput, 30.0); // 30 items per minute
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

        metrics.record_event_sent();
        metrics.record_event_sent();
        metrics.record_event_dropped();
        assert_eq!(metrics.total_events_sent, 2);
        assert_eq!(metrics.events_dropped, 1);
        assert_eq!(metrics.delivery_ratio(), 2.0 / 3.0);

        metrics.record_reconnection();
        assert_eq!(metrics.reconnection_rate(), 1.0);

        metrics.record_disconnection(Duration::from_millis(30000));
        assert_eq!(metrics.active_connections, 0);
        assert_eq!(metrics.average_connection_duration_ms, 30000.0);
    }
}
