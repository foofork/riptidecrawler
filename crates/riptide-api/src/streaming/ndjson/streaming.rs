//! Core NDJSON streaming logic and handler implementation.

// TODO(P2): Streaming infrastructure - will be activated when routes are added
// STATUS: NDJSON streaming implementation ready
// PLAN: Activate with streaming routes (see streaming/config.rs:1)
// EFFORT: Part of streaming feature implementation
// PRIORITY: Future feature
// BLOCKER: Same as streaming/config.rs
#![allow(dead_code)]
//!
//! This module contains the main streaming handler struct and its implementation methods.

use super::helpers::{orchestrate_crawl_stream_optimized, orchestrate_deepsearch_stream_optimized};
use crate::models::{CrawlBody, DeepSearchBody};
use crate::state::AppState;
use crate::streaming::buffer::{BackpressureHandler, BufferManager};
use crate::streaming::config::StreamConfig;
use crate::streaming::error::{StreamingError, StreamingResult};
use axum::body::Body;
use axum::http::StatusCode;
use axum::response::Response;
use bytes::Bytes;
use futures_util::StreamExt;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio_stream::wrappers::ReceiverStream;
use tracing::error;

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
        config.buffer.max_size = buffer_limit.clamp(256, 2048); // Respect limit but stay reasonable
        config.buffer.default_size = (buffer_limit / 4).clamp(128, 512); // Quarter of limit
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
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/x-ndjson")
            .header("Transfer-Encoding", "chunked")
            .header("Cache-Control", "no-cache")
            .header("Access-Control-Allow-Origin", "*")
            .header("X-Request-ID", &self.request_id)
            .body(Body::from_stream(
                ReceiverStream::new(rx).map(Ok::<_, std::io::Error>),
            ))
            .map_err(|e| StreamingError::channel(e.to_string()))
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
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/x-ndjson")
            .header("Transfer-Encoding", "chunked")
            .header("Cache-Control", "no-cache")
            .header("Access-Control-Allow-Origin", "*")
            .header("X-Request-ID", &self.request_id)
            .body(Body::from_stream(
                ReceiverStream::new(rx).map(Ok::<_, std::io::Error>),
            ))
            .map_err(|e| StreamingError::channel(e.to_string()))
    }
}
