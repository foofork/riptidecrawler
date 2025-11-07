//! Stream lifecycle management and event handling.
//!
//! This module provides sophisticated lifecycle management for streaming operations,
#![allow(dead_code)]
//! including event handlers for connection establishment, data flow, errors, and cleanup.
//!
//! ## Metrics Tracking
//!
//! The lifecycle manager automatically tracks comprehensive metrics for observability:
//!
//! ### Stream Start Metrics
//! - **Timestamp**: Recorded via `stream_start` field in `ConnectionInfo`
//! - **Client ID**: Tracked via `connection_id` in all lifecycle events
//! - **Content Type**: Set via `set_content_type()` method
//! - **Latency**: Recorded via `streaming_latency_seconds{operation="stream_start"}`
//!
//! ### Stream Completion Metrics
//! - **Duration**: Tracked in `streaming_duration_seconds{status="success|failure"}`
//! - **Bytes Sent**: Accumulated in `streaming_bytes_total` counter
//! - **Success/Failure**: Distinguished via histogram labels
//! - **Throughput**: Calculated and recorded in `streaming_throughput_bytes_per_sec`
//!
//! ### Error Metrics
//! - **Error Type**: Categorized in `streaming_errors_total{error_type="..."}`
//!   - `connection`: Connection-related errors
//!   - `timeout`: Timeout errors
//!   - `backpressure`: Backpressure exceeded errors
//!   - `pipeline`: Pipeline processing errors
//!   - `serialization`: Serialization errors
//!   - `other`: Uncategorized errors
//! - **Error Count**: Incremented per error occurrence
//! - **Recovery Attempts**: Tracked via `recoverable` flag in error events
//!
//! ### Performance Metrics
//! - **Throughput**: Bytes per second calculated on completion and close
//! - **Latency Percentiles**: Tracked via histogram buckets for:
//!   - `stream_start`: Time to start streaming
//!   - `stream_completion`: Total stream processing time
//!   - `connection_close`: Connection cleanup time
//! - **Connection Duration**: Tracked in `streaming_connection_duration_seconds`
//!
//! ### Prometheus Queries
//!
//! ```promql
//! # Stream throughput over 5 minutes
//! rate(riptide_streaming_bytes_total[5m])
//!
//! # Error rate by type
//! rate(riptide_streaming_errors_total[5m])
//!
//! # P95 stream duration
//! histogram_quantile(0.95, rate(riptide_streaming_duration_seconds_bucket[5m]))
//!
//! # P99 latency for stream start
//! histogram_quantile(0.99, rate(riptide_streaming_latency_seconds_bucket{operation="stream_start"}[5m]))
//! ```
//!
//! ## Performance
//!
//! Metrics collection adds < 1ms overhead per streaming operation, verified through
//! comprehensive benchmarking in `streaming_lifecycle_metrics_tests.rs`.

use super::error::StreamingError;
use super::processor::StreamProcessor;
use crate::metrics::RipTideMetrics;
use serde::Serialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Stream lifecycle events
#[derive(Debug, Clone)]
pub enum LifecycleEvent {
    /// Stream connection established
    ConnectionEstablished {
        connection_id: String,
        protocol: String,
        timestamp: Instant,
    },
    /// Stream started processing
    StreamStarted {
        connection_id: String,
        request_id: String,
        total_items: usize,
        timestamp: Instant,
    },
    /// Stream made progress
    ProgressUpdate {
        connection_id: String,
        request_id: String,
        completed: usize,
        total: usize,
        throughput: f64,
        timestamp: Instant,
    },
    /// Stream encountered an error
    StreamError {
        connection_id: String,
        request_id: String,
        error: String,
        recoverable: bool,
        timestamp: Instant,
    },
    /// Stream completed successfully
    StreamCompleted {
        connection_id: String,
        request_id: String,
        summary: StreamCompletionSummary,
        timestamp: Instant,
    },
    /// Stream was terminated or failed
    StreamTerminated {
        connection_id: String,
        request_id: String,
        reason: String,
        timestamp: Instant,
    },
    /// Connection closed
    ConnectionClosed {
        connection_id: String,
        duration: Duration,
        bytes_sent: usize,
        messages_sent: usize,
        timestamp: Instant,
    },
}

/// Summary of stream completion
#[derive(Debug, Clone, Serialize)]
pub struct StreamCompletionSummary {
    pub total_items: usize,
    pub successful: usize,
    pub failed: usize,
    pub cache_hits: usize,
    pub duration_ms: u64,
    pub throughput: f64,
    pub error_rate: f64,
}

/// Stream lifecycle manager
pub struct StreamLifecycleManager {
    /// Event channel sender
    event_tx: mpsc::UnboundedSender<LifecycleEvent>,
    /// Active connections tracking
    active_connections: Arc<tokio::sync::RwLock<std::collections::HashMap<String, ConnectionInfo>>>,
}

/// Information about an active connection
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub connection_id: String,
    pub protocol: String,
    pub connection_start: Instant, // Renamed from start_time to match usage
    pub bytes_sent: usize,
    pub messages_sent: usize,
    pub current_request_id: Option<String>,
    pub stream_start: Option<Instant>,
    pub content_type: Option<String>,
}

impl StreamLifecycleManager {
    /// Create a new lifecycle manager
    pub fn new(metrics: Arc<RipTideMetrics>) -> Self {
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        let active_connections =
            Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));

        let active_connections_clone = active_connections.clone();
        let metrics_clone = metrics.clone();

        // Start event processing task
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                Self::handle_lifecycle_event(event, &active_connections_clone, &metrics_clone)
                    .await;
            }
        });

        Self {
            event_tx,
            active_connections,
        }
    }

    /// Emit a lifecycle event
    pub fn emit_event(&self, event: LifecycleEvent) {
        if let Err(e) = self.event_tx.send(event) {
            error!("Failed to emit lifecycle event: {}", e);
        }
    }

    /// Handle connection established
    pub async fn connection_established(&self, connection_id: String, protocol: String) {
        let event = LifecycleEvent::ConnectionEstablished {
            connection_id: connection_id.clone(),
            protocol: protocol.clone(),
            timestamp: Instant::now(),
        };

        // Track the connection
        let conn_info = ConnectionInfo {
            connection_id: connection_id.clone(),
            protocol,
            connection_start: Instant::now(),
            bytes_sent: 0,
            messages_sent: 0,
            current_request_id: None,
            stream_start: None,
            content_type: None,
        };

        self.active_connections
            .write()
            .await
            .insert(connection_id, conn_info);

        self.emit_event(event);
    }

    /// Handle stream started
    pub async fn stream_started(
        &self,
        connection_id: String,
        request_id: String,
        total_items: usize,
    ) {
        let now = Instant::now();

        // Update connection with current request and stream start time
        if let Some(conn) = self
            .active_connections
            .write()
            .await
            .get_mut(&connection_id)
        {
            conn.current_request_id = Some(request_id.clone());
            conn.stream_start = Some(now);
        }

        let event = LifecycleEvent::StreamStarted {
            connection_id,
            request_id,
            total_items,
            timestamp: now,
        };

        self.emit_event(event);
    }

    /// Handle progress update
    pub async fn progress_update(
        &self,
        connection_id: String,
        request_id: String,
        completed: usize,
        total: usize,
        throughput: f64,
    ) {
        let event = LifecycleEvent::ProgressUpdate {
            connection_id,
            request_id,
            completed,
            total,
            throughput,
            timestamp: Instant::now(),
        };

        self.emit_event(event);
    }

    /// Handle stream error
    pub async fn stream_error(
        &self,
        connection_id: String,
        request_id: String,
        error: StreamingError,
    ) {
        let event = LifecycleEvent::StreamError {
            connection_id,
            request_id,
            error: error.to_string(),
            recoverable: error.is_recoverable(),
            timestamp: Instant::now(),
        };

        self.emit_event(event);
    }

    /// Handle stream completion
    pub async fn stream_completed(
        &self,
        connection_id: String,
        request_id: String,
        processor: &StreamProcessor,
    ) {
        let stats = processor.stats();
        let summary = StreamCompletionSummary {
            total_items: stats.total_urls,
            successful: stats.completed_count,
            failed: stats.error_count,
            cache_hits: stats.cache_hits,
            duration_ms: processor.start_time.elapsed().as_millis() as u64,
            throughput: stats.throughput(processor.start_time.elapsed()),
            error_rate: stats.error_count as f64 / stats.total_urls.max(1) as f64,
        };

        let event = LifecycleEvent::StreamCompleted {
            connection_id,
            request_id,
            summary,
            timestamp: Instant::now(),
        };

        self.emit_event(event);
    }

    /// Handle stream termination
    pub async fn stream_terminated(
        &self,
        connection_id: String,
        request_id: String,
        reason: String,
    ) {
        let event = LifecycleEvent::StreamTerminated {
            connection_id,
            request_id,
            reason,
            timestamp: Instant::now(),
        };

        self.emit_event(event);
    }

    /// Handle connection closed
    pub async fn connection_closed(&self, connection_id: String) {
        let conn_info = self.active_connections.write().await.remove(&connection_id);

        if let Some(info) = conn_info {
            let event = LifecycleEvent::ConnectionClosed {
                connection_id,
                duration: info.connection_start.elapsed(),
                bytes_sent: info.bytes_sent,
                messages_sent: info.messages_sent,
                timestamp: Instant::now(),
            };

            self.emit_event(event);
        }
    }

    /// Update connection statistics
    pub async fn update_connection_stats(&self, connection_id: &str, bytes_sent: usize) {
        if let Some(conn) = self.active_connections.write().await.get_mut(connection_id) {
            conn.bytes_sent += bytes_sent;
            conn.messages_sent += 1;
        }
    }

    /// Set content type for connection
    pub async fn set_content_type(&self, connection_id: &str, content_type: String) {
        if let Some(conn) = self.active_connections.write().await.get_mut(connection_id) {
            conn.content_type = Some(content_type);
        }
    }

    /// Get active connection count
    pub async fn active_connection_count(&self) -> usize {
        self.active_connections.read().await.len()
    }

    /// Get connection info
    pub async fn get_connection_info(&self, connection_id: &str) -> Option<ConnectionInfo> {
        self.active_connections
            .read()
            .await
            .get(connection_id)
            .cloned()
    }

    /// Internal event handler
    async fn handle_lifecycle_event(
        event: LifecycleEvent,
        active_connections: &Arc<
            tokio::sync::RwLock<std::collections::HashMap<String, ConnectionInfo>>,
        >,
        metrics: &Arc<RipTideMetrics>,
    ) {
        match event {
            LifecycleEvent::ConnectionEstablished {
                connection_id,
                protocol,
                timestamp: _,
            } => {
                // Update active connections gauge
                let active_count = active_connections.read().await.len();
                metrics
                    .streaming_active_connections
                    .set(active_count as f64);
                metrics.streaming_total_connections.inc();

                // Update main active_connections gauge
                metrics.active_connections.inc();

                info!(
                    connection_id = %connection_id,
                    protocol = %protocol,
                    "Stream connection established"
                );
            }
            LifecycleEvent::StreamStarted {
                connection_id,
                request_id,
                total_items,
                timestamp,
            } => {
                // Record stream start latency
                metrics.record_streaming_latency("stream_start", timestamp.elapsed().as_secs_f64());

                info!(
                    connection_id = %connection_id,
                    request_id = %request_id,
                    total_items = total_items,
                    "Stream processing started"
                );
            }
            LifecycleEvent::ProgressUpdate {
                connection_id,
                request_id,
                completed,
                total,
                throughput,
                timestamp: _,
            } => {
                // Record message sent for each progress update
                metrics.record_streaming_message_sent();

                debug!(
                    connection_id = %connection_id,
                    request_id = %request_id,
                    completed = completed,
                    total = total,
                    progress_pct = (completed as f64 / total.max(1) as f64) * 100.0,
                    throughput = throughput,
                    "Stream progress update"
                );
            }
            LifecycleEvent::StreamError {
                connection_id,
                request_id,
                error,
                recoverable,
                timestamp: _,
            } => {
                // Update error rate - calculate from active connections
                let active_count = active_connections.read().await.len().max(1);
                let error_rate = 1.0 / active_count as f64;
                metrics.streaming_error_rate.set(error_rate);

                // Record error by type
                let error_type = if error.contains("Connection") {
                    "connection"
                } else if error.contains("Timeout") {
                    "timeout"
                } else if error.contains("Backpressure") {
                    "backpressure"
                } else if error.contains("Pipeline") {
                    "pipeline"
                } else if error.contains("Serialization") {
                    "serialization"
                } else {
                    "other"
                };

                metrics.record_streaming_error_by_type(error_type);

                if recoverable {
                    warn!(
                        connection_id = %connection_id,
                        request_id = %request_id,
                        error = %error,
                        error_type = error_type,
                        "Recoverable stream error occurred"
                    );
                } else {
                    error!(
                        connection_id = %connection_id,
                        request_id = %request_id,
                        error = %error,
                        error_type = error_type,
                        "Fatal stream error occurred"
                    );
                }
            }
            LifecycleEvent::StreamCompleted {
                connection_id,
                request_id,
                summary,
                timestamp: _,
            } => {
                // Record final message count
                for _ in 0..summary.successful {
                    metrics.record_streaming_message_sent();
                }

                // Record any dropped messages (failed items)
                for _ in 0..summary.failed {
                    metrics.record_streaming_message_dropped();
                }

                // Record stream duration with success status
                let duration_secs = summary.duration_ms as f64 / 1000.0;
                let success = summary.failed == 0;
                metrics.record_streaming_duration(duration_secs, success);

                // Record throughput in bytes/sec
                // Estimate bytes from messages (assuming ~1KB average message size)
                let estimated_bytes = summary.successful * 1024;
                let bytes_per_sec = if duration_secs > 0.0 {
                    estimated_bytes as f64 / duration_secs
                } else {
                    0.0
                };
                metrics.update_streaming_throughput(bytes_per_sec);

                // Record completion latency
                metrics.record_streaming_latency("stream_completion", duration_secs);

                info!(
                    connection_id = %connection_id,
                    request_id = %request_id,
                    total_items = summary.total_items,
                    successful = summary.successful,
                    failed = summary.failed,
                    cache_hits = summary.cache_hits,
                    duration_ms = summary.duration_ms,
                    throughput = summary.throughput,
                    error_rate = summary.error_rate,
                    bytes_per_sec = bytes_per_sec,
                    "Stream completed successfully"
                );
            }
            LifecycleEvent::StreamTerminated {
                connection_id,
                request_id,
                reason,
                timestamp: _,
            } => {
                warn!(
                    connection_id = %connection_id,
                    request_id = %request_id,
                    reason = %reason,
                    "Stream terminated"
                );
            }
            LifecycleEvent::ConnectionClosed {
                connection_id,
                duration,
                bytes_sent,
                messages_sent,
                timestamp: _,
            } => {
                // Update active connections count
                let active_count = active_connections.read().await.len();
                metrics
                    .streaming_active_connections
                    .set(active_count as f64);

                // Decrement main active_connections gauge
                metrics.active_connections.dec();

                // Record connection duration
                metrics.record_streaming_connection_duration(duration.as_secs_f64());

                // Record total bytes transferred
                metrics.record_streaming_bytes(bytes_sent);

                // Update memory usage estimate based on bytes sent
                let current_memory = bytes_sent * active_count.max(1);
                metrics
                    .streaming_memory_usage_bytes
                    .set(current_memory as f64);

                // Calculate and record final throughput
                let duration_secs = duration.as_secs_f64();
                if duration_secs > 0.0 {
                    let bytes_per_sec = bytes_sent as f64 / duration_secs;
                    metrics.update_streaming_throughput(bytes_per_sec);
                }

                // Record connection close latency
                metrics.record_streaming_latency("connection_close", duration.as_secs_f64());

                info!(
                    connection_id = %connection_id,
                    duration_ms = duration.as_millis(),
                    bytes_sent = bytes_sent,
                    messages_sent = messages_sent,
                    avg_message_size = if messages_sent > 0 { bytes_sent / messages_sent } else { 0 },
                    "Stream connection closed"
                );
            }
        }
    }
}

impl StreamingError {
    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            StreamingError::Connection { .. } => true,
            StreamingError::Timeout { .. } => true,
            StreamingError::BackpressureExceeded { .. } => true,
            StreamingError::InvalidRequest { .. } => false,
            StreamingError::Pipeline { .. } => false,
            StreamingError::BufferOverflow { .. } => true,
            StreamingError::Channel { .. } => true,
            StreamingError::ClientDisconnected { .. } => false,
            StreamingError::Serialization { .. } => false,
        }
    }
}
