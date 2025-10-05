// TODO: Streaming pipeline infrastructure prepared but routes not yet activated
#![allow(dead_code)]

//! Pipeline orchestration for streaming operations.
//!
//! This module provides high-level orchestration logic for coordinating
//! streaming operations across different protocols and managing the
//! interaction between processors, buffers, and output streams.

use super::buffer::{BackpressureHandler, BufferManager};
use super::config::StreamConfig;
use super::error::{StreamingError, StreamingResult};
use super::processor::StreamProcessor;
use crate::models::*;
use crate::pipeline::PipelineOrchestrator;
use crate::state::AppState;
use serde::Serialize;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// High-level streaming pipeline orchestrator
pub struct StreamingPipeline {
    app: AppState,
    config: StreamConfig,
    buffer_manager: Arc<BufferManager>,
    request_id: String,
}

impl StreamingPipeline {
    /// Create a new streaming pipeline
    pub fn new(app: AppState, request_id: Option<String>) -> Self {
        Self {
            app,
            config: StreamConfig::from_env(),
            buffer_manager: Arc::new(BufferManager::new()),
            request_id: request_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        }
    }

    /// Execute crawl streaming pipeline
    pub async fn execute_crawl_stream<T, F>(
        &self,
        body: CrawlBody,
        sender_fn: F,
    ) -> StreamingResult<StreamExecutionSummary>
    where
        T: Send + 'static,
        F: Fn(&StreamEvent) -> Result<T, StreamingError> + Send + Sync + 'static,
    {
        let start_time = Instant::now();
        let options = body.options.unwrap_or_default();

        info!(
            request_id = %self.request_id,
            url_count = body.urls.len(),
            "Starting crawl streaming pipeline"
        );

        // Create pipeline components
        let pipeline = PipelineOrchestrator::new(self.app.clone(), options);
        let mut processor =
            StreamProcessor::new(pipeline, self.request_id.clone(), body.urls.len());
        let buffer = self.buffer_manager.get_buffer(&self.request_id).await;
        let mut backpressure_handler = BackpressureHandler::new(self.request_id.clone(), buffer);

        // Send initial metadata
        let metadata_event = StreamEvent::Metadata(StreamMetadata {
            total_urls: body.urls.len(),
            request_id: self.request_id.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            stream_type: "crawl".to_string(),
        });

        if let Err(e) = self
            .send_event(&metadata_event, &sender_fn, &mut backpressure_handler)
            .await
        {
            warn!(request_id = %self.request_id, error = %e, "Failed to send metadata event");
        }

        // Process URLs concurrently
        let mut result_rx = processor.process_urls_concurrent(body.urls.clone()).await?;

        // Stream results as they arrive
        while let Some(processed_result) = result_rx.recv().await {
            let index = processed_result.index;
            let url = processed_result.url.clone();

            // Convert to crawl result
            let crawl_result = processor.convert_to_crawl_result(processed_result);

            // Create stream result event
            let result_event = StreamEvent::Result(Box::new(StreamResultData {
                index,
                result: crawl_result,
                progress: processor.create_progress(),
            }));

            // Send result with backpressure handling
            if let Err(e) = self
                .send_event(&result_event, &sender_fn, &mut backpressure_handler)
                .await
            {
                debug!(
                    request_id = %self.request_id,
                    error = %e,
                    "Client disconnected or send error, stopping stream"
                );
                break;
            }

            // Send periodic progress updates for long operations
            if processor.should_send_progress_update(5) {
                let progress_event =
                    StreamEvent::Progress(processor.create_operation_progress(Some(url)));
                if let Err(e) = self
                    .send_event(&progress_event, &sender_fn, &mut backpressure_handler)
                    .await
                {
                    debug!(request_id = %self.request_id, error = %e, "Failed to send progress update");
                }
            }
        }

        // Send final summary
        let summary = processor.create_summary();
        let summary_event = StreamEvent::Summary(summary.clone());

        if let Err(e) = self
            .send_event(&summary_event, &sender_fn, &mut backpressure_handler)
            .await
        {
            warn!(request_id = %self.request_id, error = %e, "Failed to send summary event");
        }

        // Log completion
        processor.log_completion();

        // Record comprehensive metrics
        self.app.metrics.record_http_request(
            "POST",
            "/crawl/stream",
            200,
            start_time.elapsed().as_secs_f64(),
        );

        // Record streaming-specific metrics
        info!(
            request_id = %self.request_id,
            total_urls = body.urls.len(),
            successful = summary.successful,
            failed = summary.failed,
            cache_hits = summary.from_cache,
            throughput = summary.throughput_per_second,
            total_duration_ms = start_time.elapsed().as_millis(),
            backpressure_events = backpressure_handler.metrics().dropped_messages,
            "Crawl streaming pipeline completed with metrics"
        );

        // Clean up buffer
        self.buffer_manager.remove_buffer(&self.request_id).await;

        Ok(StreamExecutionSummary {
            request_id: self.request_id.clone(),
            total_urls: body.urls.len(),
            successful: summary.successful,
            failed: summary.failed,
            from_cache: summary.from_cache,
            total_duration_ms: start_time.elapsed().as_millis() as u64,
            throughput: summary.throughput_per_second,
            backpressure_events: backpressure_handler.metrics().dropped_messages,
        })
    }

    /// Execute deep search streaming pipeline
    pub async fn execute_deepsearch_stream<T, F>(
        &self,
        body: DeepSearchBody,
        sender_fn: F,
    ) -> StreamingResult<StreamExecutionSummary>
    where
        T: Send + 'static,
        F: Fn(&StreamEvent) -> Result<T, StreamingError> + Send + Sync + 'static,
    {
        let start_time = Instant::now();
        let limit = body.limit.unwrap_or(10).min(50);
        let include_content = body.include_content.unwrap_or(true);
        let mut total_urls_found = 0;

        info!(
            request_id = %self.request_id,
            query = %body.query,
            limit = limit,
            include_content = include_content,
            "Starting deep search streaming pipeline"
        );

        // Check for Serper API key
        let serper_api_key = std::env::var("SERPER_API_KEY").map_err(|_| {
            StreamingError::invalid_request("SERPER_API_KEY environment variable not set")
        })?;

        let buffer = self.buffer_manager.get_buffer(&self.request_id).await;
        let mut backpressure_handler = BackpressureHandler::new(self.request_id.clone(), buffer);

        // Send initial metadata
        let metadata_event = StreamEvent::Metadata(StreamMetadata {
            total_urls: 0, // Unknown until search completes
            request_id: self.request_id.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            stream_type: "deepsearch".to_string(),
        });

        if let Err(e) = self
            .send_event(&metadata_event, &sender_fn, &mut backpressure_handler)
            .await
        {
            warn!(request_id = %self.request_id, error = %e, "Failed to send metadata event");
        }

        // Perform web search
        let search_results = self
            .perform_web_search(&body.query, limit, &serper_api_key)
            .await?;

        // Send search metadata
        let search_metadata_event = StreamEvent::SearchMetadata(DeepSearchMetadata {
            query: body.query.clone(),
            urls_found: search_results.len(),
            search_time_ms: start_time.elapsed().as_millis() as u64,
        });

        if let Err(e) = self
            .send_event(
                &search_metadata_event,
                &sender_fn,
                &mut backpressure_handler,
            )
            .await
        {
            warn!(request_id = %self.request_id, error = %e, "Failed to send search metadata");
        }

        let mut summary = StreamExecutionSummary {
            request_id: self.request_id.clone(),
            total_urls: search_results.len(),
            successful: 0,
            failed: 0,
            from_cache: 0,
            total_duration_ms: 0,
            throughput: 0.0,
            backpressure_events: 0,
        };

        if !include_content || search_results.is_empty() {
            // Send search results without content extraction
            let search_results_len = search_results.len();
            total_urls_found = search_results.len();
            for (index, result) in search_results.clone().into_iter().enumerate() {
                let search_result_event =
                    StreamEvent::SearchResult(Box::new(DeepSearchResultData {
                        index,
                        search_result: result,
                        crawl_result: None,
                    }));

                if let Err(e) = self
                    .send_event(&search_result_event, &sender_fn, &mut backpressure_handler)
                    .await
                {
                    debug!(request_id = %self.request_id, error = %e, "Client disconnected during search results");
                    break;
                }
            }
            summary.successful = search_results_len;
        } else {
            // Extract URLs and crawl with streaming
            let urls: Vec<String> = search_results.iter().map(|r| r.url.clone()).collect();
            let crawl_options = body.crawl_options.unwrap_or_default();
            let pipeline = PipelineOrchestrator::new(self.app.clone(), crawl_options);
            let processor = StreamProcessor::new(pipeline, self.request_id.clone(), urls.len());

            // Process URLs concurrently
            let mut result_rx = processor.process_urls_concurrent(urls).await?;

            // Stream results as they arrive
            while let Some(processed_result) = result_rx.recv().await {
                let index = processed_result.index;
                let search_result = search_results[index].clone();

                let crawl_result = match processed_result.result {
                    Ok(pipeline_result) => Some(CrawlResult {
                        url: pipeline_result.document.url.clone(),
                        status: pipeline_result.http_status,
                        from_cache: pipeline_result.from_cache,
                        gate_decision: pipeline_result.gate_decision,
                        quality_score: pipeline_result.quality_score,
                        processing_time_ms: pipeline_result.processing_time_ms,
                        document: Some(pipeline_result.document),
                        error: None,
                        cache_key: pipeline_result.cache_key,
                    }),
                    Err(_) => None,
                };

                let search_result_event =
                    StreamEvent::SearchResult(Box::new(DeepSearchResultData {
                        index,
                        search_result,
                        crawl_result,
                    }));

                if let Err(e) = self
                    .send_event(&search_result_event, &sender_fn, &mut backpressure_handler)
                    .await
                {
                    debug!(request_id = %self.request_id, error = %e, "Client disconnected during deep search");
                    break;
                }
            }

            let final_summary = processor.create_summary();
            summary.successful = final_summary.successful;
            summary.failed = final_summary.failed;
            summary.from_cache = final_summary.from_cache;
            summary.throughput = final_summary.throughput_per_second;
        }

        // Send final summary
        summary.total_duration_ms = start_time.elapsed().as_millis() as u64;
        summary.backpressure_events = backpressure_handler.metrics().dropped_messages;

        let summary_event = StreamEvent::DeepSearchSummary(DeepSearchSummary {
            query: body.query.clone(),
            total_urls_found,
            total_processing_time_ms: summary.total_duration_ms,
            status: "completed".to_string(),
        });

        if let Err(e) = self
            .send_event(&summary_event, &sender_fn, &mut backpressure_handler)
            .await
        {
            warn!(request_id = %self.request_id, error = %e, "Failed to send final summary");
        }

        // Record comprehensive metrics for deep search
        self.app.metrics.record_http_request(
            "POST",
            "/deepsearch/stream",
            200,
            start_time.elapsed().as_secs_f64(),
        );

        info!(
            request_id = %self.request_id,
            query = %body.query,
            urls_found = search_results.len(),
            successful = summary.successful,
            failed = summary.failed,
            cache_hits = summary.from_cache,
            include_content = include_content,
            throughput = summary.throughput,
            total_duration_ms = summary.total_duration_ms,
            backpressure_events = summary.backpressure_events,
            "Deep search streaming pipeline completed with comprehensive metrics"
        );

        // Clean up buffer
        self.buffer_manager.remove_buffer(&self.request_id).await;

        Ok(summary)
    }

    /// Send stream event with backpressure handling
    async fn send_event<T, F>(
        &self,
        event: &StreamEvent,
        sender_fn: &F,
        backpressure_handler: &mut BackpressureHandler,
    ) -> StreamingResult<()>
    where
        T: Send + 'static,
        F: Fn(&StreamEvent) -> Result<T, StreamingError>,
    {
        // Check for backpressure
        if backpressure_handler.should_drop_message(0).await {
            warn!(
                request_id = %self.request_id,
                event_type = ?std::mem::discriminant(event),
                "Dropping event due to backpressure"
            );
            return Ok(());
        }

        let send_start = Instant::now();

        // Send event using provided function
        sender_fn(event).map_err(|e| {
            error!(
                request_id = %self.request_id,
                error = %e,
                "Failed to send stream event"
            );
            e
        })?;

        let send_duration = send_start.elapsed();
        backpressure_handler.record_send_time(send_duration).await?;

        Ok(())
    }

    /// Perform web search using the Serper API
    async fn perform_web_search(
        &self,
        query: &str,
        limit: u32,
        api_key: &str,
    ) -> StreamingResult<Vec<SearchResult>> {
        let search_request = serde_json::json!({
            "q": query,
            "num": limit,
            "gl": "us",
            "hl": "en"
        });

        let response = self
            .app
            .http_client
            .post("https://google.serper.dev/search")
            .header("X-API-KEY", api_key)
            .header("Content-Type", "application/json")
            .json(&search_request)
            .send()
            .await
            .map_err(|e| StreamingError::connection(format!("Search request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(StreamingError::connection(format!(
                "Search API returned status: {}",
                response.status()
            )));
        }

        let search_response: serde_json::Value = response.json().await.map_err(|e| {
            StreamingError::connection(format!("Failed to parse search response: {}", e))
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

    /// Get pipeline configuration
    pub fn config(&self) -> &StreamConfig {
        &self.config
    }

    /// Get request ID
    pub fn request_id(&self) -> &str {
        &self.request_id
    }
}

/// Stream event types for unified event handling
#[derive(Debug, Clone)]
pub enum StreamEvent {
    Metadata(StreamMetadata),
    Result(Box<StreamResultData>),
    Progress(super::processor::OperationProgress),
    Summary(super::processor::StreamSummary),
    SearchMetadata(DeepSearchMetadata),
    SearchResult(Box<DeepSearchResultData>),
    DeepSearchSummary(DeepSearchSummary),
    Error(StreamErrorData),
}

/// Metadata sent at the beginning of a stream
#[derive(Debug, Clone, Serialize)]
pub struct StreamMetadata {
    pub total_urls: usize,
    pub request_id: String,
    pub timestamp: String,
    pub stream_type: String,
}

/// Individual streaming result with progress information
#[derive(Debug, Clone, Serialize)]
pub struct StreamResultData {
    pub index: usize,
    pub result: CrawlResult,
    pub progress: super::processor::StreamProgress,
}

/// Metadata for deep search operations
#[derive(Debug, Clone, Serialize)]
pub struct DeepSearchMetadata {
    pub query: String,
    pub urls_found: usize,
    pub search_time_ms: u64,
}

/// Individual deep search result with optional crawl data
#[derive(Debug, Clone, Serialize)]
pub struct DeepSearchResultData {
    pub index: usize,
    pub search_result: SearchResult,
    pub crawl_result: Option<CrawlResult>,
}

/// Final summary for deep search operations
#[derive(Debug, Clone, Serialize)]
pub struct DeepSearchSummary {
    pub query: String,
    pub total_urls_found: usize,
    pub total_processing_time_ms: u64,
    pub status: String,
}

/// Error event data
#[derive(Debug, Clone, Serialize)]
pub struct StreamErrorData {
    pub error_type: String,
    pub message: String,
    pub retryable: bool,
    pub timestamp: String,
}

/// Summary of stream execution
#[derive(Debug, Clone)]
pub struct StreamExecutionSummary {
    pub request_id: String,
    pub total_urls: usize,
    pub successful: usize,
    pub failed: usize,
    pub from_cache: usize,
    pub total_duration_ms: u64,
    pub throughput: f64,
    pub backpressure_events: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CrawlOptions;

    #[tokio::test]
    async fn test_streaming_pipeline_creation() {
        let app = AppState::new().await.expect("Failed to create AppState");
        let pipeline = StreamingPipeline::new(app, Some("test-123".to_string()));
        assert_eq!(pipeline.request_id(), "test-123");
    }

    #[test]
    fn test_stream_event_metadata() {
        let metadata = StreamMetadata {
            total_urls: 5,
            request_id: "test-123".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            stream_type: "crawl".to_string(),
        };

        let event = StreamEvent::Metadata(metadata);
        assert!(matches!(event, StreamEvent::Metadata(_)));
    }

    #[test]
    fn test_stream_execution_summary() {
        let summary = StreamExecutionSummary {
            request_id: "test-123".to_string(),
            total_urls: 10,
            successful: 8,
            failed: 2,
            from_cache: 3,
            total_duration_ms: 5000,
            throughput: 2.0,
            backpressure_events: 1,
        };

        assert_eq!(summary.total_urls, 10);
        assert_eq!(summary.successful, 8);
        assert_eq!(summary.failed, 2);
        assert_eq!(summary.throughput, 2.0);
    }

    #[test]
    fn test_deep_search_metadata() {
        let metadata = DeepSearchMetadata {
            query: "test query".to_string(),
            urls_found: 5,
            search_time_ms: 1000,
        };

        assert_eq!(metadata.query, "test query");
        assert_eq!(metadata.urls_found, 5);
        assert_eq!(metadata.search_time_ms, 1000);
    }
}
