//! Streaming domain ports for hexagonal architecture
//!
//! This module provides backend-agnostic streaming interfaces that enable:
//! - Protocol-agnostic business logic (NDJSON, SSE, WebSocket)
//! - Dependency inversion for transport layers
//! - Testing with mock implementations
//! - Clear separation between business logic and protocol details
//!
//! # Design Goals
//!
//! - **Protocol Independence**: Business logic doesn't know about HTTP, WebSocket, or SSE details
//! - **Testability**: Easy mocking and in-memory testing
//! - **Flexibility**: Support multiple transport implementations
//! - **Lifecycle Management**: Explicit stream state transitions
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────┐
//! │ Domain Layer (Ports)                     │
//! │  ├─ StreamingTransport                   │
//! │  ├─ StreamProcessor                      │
//! │  └─ StreamLifecycle                      │
//! └──────────────────────────────────────────┘
//!              ↑ implements          ↑ uses
//!              │                     │
//! ┌────────────┴──────────┐   ┌────┴──────────────┐
//! │ Infrastructure         │   │ Application       │
//! │ - WebSocketTransport   │   │ - StreamingFacade │
//! │ - SseTransport         │   │ - Business logic  │
//! │ - NdjsonTransport      │   │ - Orchestration   │
//! └────────────────────────┘   └───────────────────┘
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_types::ports::streaming::{StreamingTransport, StreamEvent};
//!
//! async fn send_result<T: StreamingTransport>(
//!     transport: &mut T,
//!     result: StreamResult,
//! ) -> Result<(), T::Error> {
//!     transport.send_event(StreamEvent::Result(Box::new(result))).await
//! }
//! ```

use crate::error::Result as RiptideResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Transport layer abstraction for streaming protocols
///
/// This trait enables protocol-agnostic streaming by abstracting over
/// WebSocket, SSE, and NDJSON transports. Implementations handle the
/// protocol-specific message formatting and delivery.
///
/// # Type Parameters
///
/// - `Message`: Serializable message type (typically `serde_json::Value`)
/// - `Error`: Transport-specific error type
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` for use in async contexts.
#[async_trait]
pub trait StreamingTransport: Send + Sync {
    /// Message type for this transport
    type Message: Serialize + for<'de> Deserialize<'de> + Send;

    /// Error type for this transport
    type Error: std::error::Error + Send + Sync;

    /// Send a stream event
    ///
    /// # Arguments
    ///
    /// * `event` - Event to send (metadata, result, progress, etc.)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Event sent successfully
    /// * `Err(_)` - Transport error (disconnection, serialization failure, etc.)
    async fn send_event(&mut self, event: StreamEvent) -> Result<(), Self::Error>;

    /// Send stream metadata
    ///
    /// # Arguments
    ///
    /// * `metadata` - Stream metadata (session info, request details)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Metadata sent successfully
    /// * `Err(_)` - Transport error
    async fn send_metadata(&mut self, metadata: StreamMetadata) -> Result<(), Self::Error>;

    /// Send a stream result
    ///
    /// # Arguments
    ///
    /// * `result` - Processing result data
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Result sent successfully
    /// * `Err(_)` - Transport error
    async fn send_result(&mut self, result: StreamResult) -> Result<(), Self::Error>;

    /// Send an error event
    ///
    /// # Arguments
    ///
    /// * `error` - Error information to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Error sent successfully
    /// * `Err(_)` - Transport error
    async fn send_error(&mut self, error: StreamErrorData) -> Result<(), Self::Error>;

    /// Close the transport connection
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Connection closed gracefully
    /// * `Err(_)` - Transport error during close
    async fn close(&mut self) -> Result<(), Self::Error>;

    /// Get the protocol name
    ///
    /// # Returns
    ///
    /// Protocol identifier (e.g., "websocket", "sse", "ndjson")
    fn protocol_name(&self) -> &'static str;
}

/// Business logic interface for stream processing
///
/// This trait encapsulates the domain logic for processing URLs and
/// generating streaming results. Implementations coordinate with the
/// pipeline layer to execute crawling and extraction operations.
#[async_trait]
pub trait StreamProcessor: Send + Sync {
    /// Process multiple URLs concurrently
    ///
    /// # Arguments
    ///
    /// * `urls` - URLs to process
    ///
    /// # Returns
    ///
    /// * `Ok(results)` - Processed results
    /// * `Err(_)` - Processing error
    async fn process_urls(&self, urls: Vec<String>) -> RiptideResult<Vec<ProcessedResult>>;

    /// Create progress report
    ///
    /// # Returns
    ///
    /// Current progress metrics
    async fn create_progress(&self) -> StreamProgress;

    /// Create stream summary
    ///
    /// # Returns
    ///
    /// Summary of streaming operation
    async fn create_summary(&self) -> StreamSummary;

    /// Check if progress should be sent
    ///
    /// # Arguments
    ///
    /// * `interval` - Progress reporting interval
    ///
    /// # Returns
    ///
    /// `true` if progress should be sent now
    fn should_send_progress(&self, interval: usize) -> bool;
}

/// Lifecycle management for streaming sessions
///
/// This trait provides hooks for tracking stream lifecycle events,
/// enabling metrics collection, logging, and event emission.
#[async_trait]
pub trait StreamLifecycle: Send + Sync {
    /// Called when a new connection is established
    ///
    /// # Arguments
    ///
    /// * `connection_id` - Unique connection identifier
    /// * `protocol` - Protocol name (websocket, sse, ndjson)
    async fn on_connection_established(&self, connection_id: String, protocol: String);

    /// Called when stream processing starts
    ///
    /// # Arguments
    ///
    /// * `connection_id` - Connection identifier
    /// * `request_id` - Request identifier
    /// * `total_items` - Total number of items to process
    async fn on_stream_started(
        &self,
        connection_id: String,
        request_id: String,
        total_items: usize,
    );

    /// Called on progress updates
    ///
    /// # Arguments
    ///
    /// * `connection_id` - Connection identifier
    /// * `completed` - Number of completed items
    /// * `total` - Total number of items
    async fn on_progress(&self, connection_id: String, completed: usize, total: usize);

    /// Called when an error occurs
    ///
    /// # Arguments
    ///
    /// * `connection_id` - Connection identifier
    /// * `error` - Error information
    async fn on_error(&self, connection_id: String, error: crate::error::RiptideError);

    /// Called when stream completes successfully
    ///
    /// # Arguments
    ///
    /// * `connection_id` - Connection identifier
    /// * `summary` - Completion summary
    async fn on_completed(&self, connection_id: String, summary: StreamCompletionSummary);

    /// Called when connection closes
    ///
    /// # Arguments
    ///
    /// * `connection_id` - Connection identifier
    async fn on_connection_closed(&self, connection_id: String);
}

/// Stream state machine
///
/// Represents the current state of a streaming session with
/// explicit state transitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum StreamState {
    /// Stream is idle (not started)
    Idle,

    /// Connection established
    Connected,

    /// Actively streaming with progress
    Streaming {
        /// Progress percentage (0.0 - 100.0)
        progress: u32,
    },

    /// Stream paused
    Paused {
        /// Item index where paused
        at: usize,
    },

    /// Stream completed successfully
    Completed {
        /// Summary information
        summary: StreamSummary,
    },

    /// Stream failed with error
    Failed {
        /// Error message
        error: String,
    },
}

/// Stream event types
///
/// Represents all possible events that can be sent over a stream.
/// Events are protocol-agnostic and get formatted by transport adapters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    /// Stream metadata event
    Metadata(StreamMetadata),

    /// Processing result event
    Result(Box<StreamResultData>),

    /// Progress update event
    Progress(StreamProgress),

    /// Summary event (completion)
    Summary(StreamSummary),

    /// Search metadata event
    SearchMetadata(DeepSearchMetadata),

    /// Search result event
    SearchResult(Box<DeepSearchResultData>),

    /// Error event
    Error(StreamErrorData),
}

/// Stream metadata
///
/// Sent at the beginning of a stream to provide context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMetadata {
    /// Unique request identifier
    pub request_id: String,

    /// Session identifier
    pub session_id: String,

    /// Total number of URLs to process
    pub total_urls: usize,

    /// Stream configuration
    pub config: StreamConfig,

    /// Timestamp when stream started
    pub started_at: chrono::DateTime<chrono::Utc>,
}

/// Stream result data
///
/// Represents a single processing result in the stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamResultData {
    /// URL that was processed
    pub url: String,

    /// Processing result
    pub result: serde_json::Value,

    /// Processing timestamp
    pub processed_at: chrono::DateTime<chrono::Utc>,

    /// Processing duration
    pub duration_ms: u64,
}

/// Stream progress information
///
/// Sent periodically to update clients on processing progress.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamProgress {
    /// Number of completed items
    pub completed: usize,

    /// Total number of items
    pub total: usize,

    /// Progress percentage (0.0 - 100.0)
    pub percentage: f64,

    /// Processing rate (items per second)
    pub rate: f64,

    /// Estimated time remaining
    pub eta_seconds: Option<u64>,
}

/// Stream summary
///
/// Sent when stream completes, providing final statistics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StreamSummary {
    /// Total items processed
    pub total_processed: usize,

    /// Number of successful items
    pub successful: usize,

    /// Number of failed items
    pub failed: usize,

    /// Total processing time
    pub duration_ms: u64,

    /// Average processing time per item
    pub avg_duration_ms: f64,

    /// Final statistics
    pub stats: serde_json::Value,
}

/// Deep search metadata
///
/// Metadata specific to deep search operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSearchMetadata {
    /// Search query
    pub query: String,

    /// Number of results expected
    pub total_results: usize,

    /// Search depth
    pub depth: u32,
}

/// Deep search result data
///
/// Result specific to deep search operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSearchResultData {
    /// Result URL
    pub url: String,

    /// Search score/relevance
    pub score: f64,

    /// Result content
    pub content: serde_json::Value,
}

/// Stream error data
///
/// Error information sent over the stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamErrorData {
    /// Error code
    pub code: String,

    /// Error message
    pub message: String,

    /// Error context (optional)
    pub context: Option<serde_json::Value>,

    /// Timestamp when error occurred
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

/// Stream configuration
///
/// Configuration settings for a streaming session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    /// Buffer size
    pub buffer_size: usize,

    /// Progress reporting interval
    pub progress_interval: usize,

    /// Enable compression
    pub compression: bool,
}

/// Stream completion summary
///
/// Summary information when stream completes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamCompletionSummary {
    /// Total items processed
    pub total: usize,

    /// Successful items
    pub successful: usize,

    /// Failed items
    pub failed: usize,

    /// Total duration
    pub duration: Duration,
}

/// Processed result
///
/// Result of processing a single item.
#[derive(Debug, Clone)]
pub struct ProcessedResult {
    /// Item identifier
    pub id: String,

    /// Processing success
    pub success: bool,

    /// Result data
    pub data: serde_json::Value,
}

/// Stream result wrapper
///
/// Generic result type for streaming operations.
pub type StreamResult = StreamResultData;

/// Stream metrics
///
/// Metrics collected during streaming operations.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StreamMetrics {
    /// Active connections
    pub active_connections: usize,

    /// Total messages sent
    pub total_messages_sent: usize,

    /// Total messages dropped
    pub total_messages_dropped: usize,

    /// Average latency (milliseconds)
    pub average_latency_ms: f64,

    /// Throughput (bytes per second)
    pub throughput_bytes_per_sec: f64,

    /// Error rate (0.0 - 1.0)
    pub error_rate: f64,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 256,
            progress_interval: 10,
            compression: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_state_transitions() {
        let idle = StreamState::Idle;
        assert!(matches!(idle, StreamState::Idle));

        let streaming = StreamState::Streaming { progress: 50 };
        assert!(matches!(streaming, StreamState::Streaming { progress: 50 }));

        let paused = StreamState::Paused { at: 10 };
        assert!(matches!(paused, StreamState::Paused { at: 10 }));
    }

    #[test]
    fn test_stream_event_serialization() {
        let metadata = StreamMetadata {
            request_id: "req-123".to_string(),
            session_id: "session-456".to_string(),
            total_urls: 100,
            config: StreamConfig::default(),
            started_at: chrono::Utc::now(),
        };

        let event = StreamEvent::Metadata(metadata);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("metadata"));
    }

    #[test]
    fn test_stream_progress_calculation() {
        let progress = StreamProgress {
            completed: 50,
            total: 100,
            percentage: 50.0,
            rate: 10.0,
            eta_seconds: Some(5),
        };

        assert_eq!(progress.percentage, 50.0);
        assert_eq!(progress.rate, 10.0);
    }

    #[test]
    fn test_stream_config_defaults() {
        let config = StreamConfig::default();
        assert_eq!(config.buffer_size, 256);
        assert_eq!(config.progress_interval, 10);
        assert!(!config.compression);
    }

    #[test]
    fn test_stream_metrics_defaults() {
        let metrics = StreamMetrics::default();
        assert_eq!(metrics.active_connections, 0);
        assert_eq!(metrics.total_messages_sent, 0);
        assert_eq!(metrics.error_rate, 0.0);
    }
}
