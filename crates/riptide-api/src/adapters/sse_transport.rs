//! Server-Sent Events (SSE) transport adapter implementing StreamingTransport trait.
//!
//! This adapter provides SSE protocol handling for the streaming facade,
//! focusing on event formatting, reconnection support, and keepalive.

use async_trait::async_trait;
use axum::response::sse::Event;
use riptide_types::ports::streaming::{
    DeepSearchMetadata, DeepSearchResultData, StreamErrorData, StreamEvent, StreamMetadata,
    StreamProgress, StreamResult, StreamSummary, StreamingTransport,
};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tracing::{debug, warn};

use crate::streaming::error::{StreamingError, StreamingResult};

/// SSE transport adapter
///
/// Handles SSE-specific protocol details:
/// - Event formatting (event:, data:, id:, retry:)
/// - Automatic keepalive (comment lines)
/// - Last-Event-ID reconnection support
/// - Event streaming over HTTP
pub struct SseTransport {
    /// SSE event sender channel
    sender: Arc<Mutex<mpsc::Sender<Result<Event, Infallible>>>>,

    /// Session identifier for logging and metrics
    session_id: String,

    /// Event counter for Last-Event-ID support
    event_counter: Arc<Mutex<usize>>,

    /// Message counter for metrics
    message_count: Arc<Mutex<usize>>,

    /// Connection start time
    connected_at: Instant,

    /// Retry interval for client reconnection (milliseconds)
    retry_interval_ms: u32,
}

impl SseTransport {
    /// Create a new SSE transport
    ///
    /// # Arguments
    ///
    /// * `sender` - Channel sender for SSE events
    /// * `session_id` - Unique session identifier
    /// * `retry_interval_ms` - Client reconnection retry interval
    pub fn new(
        sender: mpsc::Sender<Result<Event, Infallible>>,
        session_id: String,
        retry_interval_ms: Option<u32>,
    ) -> Self {
        Self {
            sender: Arc::new(Mutex::new(sender)),
            session_id,
            event_counter: Arc::new(Mutex::new(0)),
            message_count: Arc::new(Mutex::new(0)),
            connected_at: Instant::now(),
            retry_interval_ms: retry_interval_ms.unwrap_or(5000),
        }
    }

    /// Send an SSE event with proper formatting
    async fn send_sse_event(
        &self,
        event_type: &str,
        data: &serde_json::Value,
        include_id: bool,
    ) -> StreamingResult<()> {
        let sender = self.sender.lock().await;

        // Check if channel is closed (client disconnected)
        if sender.is_closed() {
            return Err(StreamingError::client_disconnected("SSE channel closed"));
        }

        // Serialize event data
        let data_str = serde_json::to_string(data)
            .map_err(StreamingError::from)?;

        // Build SSE event
        let mut event = Event::default()
            .event(event_type)
            .data(data_str);

        // Add event ID for reconnection support
        if include_id {
            let mut counter = self.event_counter.lock().await;
            *counter += 1;
            event = event.id(counter.to_string());
        }

        // Add retry interval for important events
        if matches!(event_type, "metadata" | "summary" | "error") {
            event = event.retry(Duration::from_millis(self.retry_interval_ms as u64));
        }

        // Send event
        sender.send(Ok(event)).await.map_err(|_| {
            StreamingError::channel("Failed to send SSE event, client disconnected")
        })?;

        // Update message count
        let mut count = self.message_count.lock().await;
        *count += 1;

        Ok(())
    }

    /// Get current message count
    pub async fn message_count(&self) -> usize {
        *self.message_count.lock().await
    }

    /// Get current event counter
    pub async fn event_counter(&self) -> usize {
        *self.event_counter.lock().await
    }

    /// Get connection duration
    pub fn connection_duration(&self) -> Duration {
        self.connected_at.elapsed()
    }

    /// Check if client is still connected
    pub async fn is_connected(&self) -> bool {
        !self.sender.lock().await.is_closed()
    }
}

#[async_trait]
impl StreamingTransport for SseTransport {
    type Message = serde_json::Value;
    type Error = StreamingError;

    async fn send_event(&mut self, event: StreamEvent) -> Result<(), Self::Error> {
        let (event_type, data, include_id) = match event {
            StreamEvent::Metadata(metadata) => {
                ("metadata", serde_json::to_value(metadata).unwrap(), false)
            }
            StreamEvent::Result(result) => {
                ("result", serde_json::to_value(result).unwrap(), true)
            }
            StreamEvent::Progress(progress) => {
                ("progress", serde_json::to_value(progress).unwrap(), false)
            }
            StreamEvent::Summary(summary) => {
                ("summary", serde_json::to_value(summary).unwrap(), false)
            }
            StreamEvent::SearchMetadata(metadata) => {
                ("search_metadata", serde_json::to_value(metadata).unwrap(), false)
            }
            StreamEvent::SearchResult(result) => {
                ("search_result", serde_json::to_value(result).unwrap(), true)
            }
            StreamEvent::Error(error) => {
                ("error", serde_json::to_value(error).unwrap(), false)
            }
        };

        let session_id = self.session_id.clone();
        debug!(
            session_id = %session_id,
            event_type = event_type,
            "Sending SSE event"
        );

        self.send_sse_event(event_type, &data, include_id).await
    }

    async fn send_metadata(&mut self, metadata: StreamMetadata) -> Result<(), Self::Error> {
        self.send_event(StreamEvent::Metadata(metadata)).await
    }

    async fn send_result(&mut self, result: StreamResult) -> Result<(), Self::Error> {
        self.send_event(StreamEvent::Result(Box::new(result))).await
    }

    async fn send_error(&mut self, error: StreamErrorData) -> Result<(), Self::Error> {
        self.send_event(StreamEvent::Error(error)).await
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        let session_id = self.session_id.clone();
        debug!(session_id = %session_id, "Closing SSE connection");

        // Send a final "done" event (optional, for graceful closure)
        let done_event = serde_json::json!({
            "status": "complete",
            "message": "Stream ended"
        });

        if let Err(e) = self.send_sse_event("done", &done_event, false).await {
            let session_id = self.session_id.clone();
            let error_msg = e.to_string();
            warn!(
                session_id = %session_id,
                error = %error_msg,
                "Error sending final done event"
            );
        }

        let session_id = self.session_id.clone();
        let duration_ms = self.connected_at.elapsed().as_millis();
        let message_count = self.message_count().await;
        let event_count = self.event_counter().await;
        debug!(
            session_id = %session_id,
            duration_ms = duration_ms,
            message_count = message_count,
            event_count = event_count,
            "SSE connection closed"
        );

        Ok(())
    }

    fn protocol_name(&self) -> &'static str {
        "sse"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_stream::wrappers::ReceiverStream;

    #[test]
    fn test_protocol_name() {
        let (tx, _rx) = mpsc::channel(10);
        let transport = SseTransport::new(tx, "test-session".to_string(), Some(3000));
        assert_eq!(transport.protocol_name(), "sse");
    }

    #[tokio::test]
    async fn test_message_count() {
        let (tx, _rx) = mpsc::channel(10);
        let transport = SseTransport::new(tx, "test-session".to_string(), None);
        assert_eq!(transport.message_count().await, 0);
    }

    #[tokio::test]
    async fn test_event_counter() {
        let (tx, _rx) = mpsc::channel(10);
        let transport = SseTransport::new(tx, "test-session".to_string(), None);
        assert_eq!(transport.event_counter().await, 0);
    }

    #[test]
    fn test_connection_duration() {
        let (tx, _rx) = mpsc::channel(10);
        let transport = SseTransport::new(tx, "test-session".to_string(), None);
        let duration = transport.connection_duration();
        assert!(duration.as_millis() >= 0);
    }

    #[tokio::test]
    async fn test_is_connected() {
        let (tx, _rx) = mpsc::channel(10);
        let transport = SseTransport::new(tx, "test-session".to_string(), None);
        assert!(transport.is_connected().await);
    }

    #[tokio::test]
    async fn test_send_metadata() {
        let (tx, mut rx) = mpsc::channel(10);
        let mut transport = SseTransport::new(tx, "test-session".to_string(), None);

        let metadata = StreamMetadata {
            request_id: "req-123".to_string(),
            session_id: "session-456".to_string(),
            total_urls: 5,
            config: riptide_types::ports::streaming::StreamConfig::default(),
            started_at: chrono::Utc::now(),
        };

        let result = transport.send_metadata(metadata).await;
        assert!(result.is_ok());

        // Verify event was sent
        let event = rx.recv().await;
        assert!(event.is_some());
    }

    #[tokio::test]
    async fn test_send_progress() {
        let (tx, mut rx) = mpsc::channel(10);
        let mut transport = SseTransport::new(tx, "test-session".to_string(), None);

        let progress = StreamProgress {
            completed: 5,
            total: 10,
            percentage: 50.0,
            rate: 2.5,
            eta_seconds: Some(2),
        };

        let result = transport.send_event(StreamEvent::Progress(progress)).await;
        assert!(result.is_ok());

        // Verify event was sent
        let event = rx.recv().await;
        assert!(event.is_some());
        assert_eq!(transport.message_count().await, 1);
    }

    #[tokio::test]
    async fn test_send_result_with_id() {
        let (tx, mut rx) = mpsc::channel(10);
        let mut transport = SseTransport::new(tx, "test-session".to_string(), None);

        let result = StreamResult {
            url: "https://example.com".to_string(),
            result: serde_json::json!({"status": "success"}),
            processed_at: chrono::Utc::now(),
            duration_ms: 100,
        };

        let send_result = transport.send_result(result).await;
        assert!(send_result.is_ok());

        // Verify event was sent with ID
        let event = rx.recv().await;
        assert!(event.is_some());
        assert_eq!(transport.event_counter().await, 1);
    }

    #[tokio::test]
    async fn test_send_error() {
        let (tx, mut rx) = mpsc::channel(10);
        let mut transport = SseTransport::new(tx, "test-session".to_string(), None);

        let error = StreamErrorData {
            code: "ERR_001".to_string(),
            message: "Test error".to_string(),
            context: None,
            occurred_at: chrono::Utc::now(),
        };

        let result = transport.send_error(error).await;
        assert!(result.is_ok());

        // Verify event was sent
        let event = rx.recv().await;
        assert!(event.is_some());
    }

    #[tokio::test]
    async fn test_close() {
        let (tx, mut rx) = mpsc::channel(10);
        let mut transport = SseTransport::new(tx, "test-session".to_string(), None);

        let result = transport.close().await;
        assert!(result.is_ok());

        // Verify "done" event was sent
        let event = rx.recv().await;
        assert!(event.is_some());
    }

    #[tokio::test]
    async fn test_disconnected_client() {
        let (tx, rx) = mpsc::channel(10);
        let mut transport = SseTransport::new(tx, "test-session".to_string(), None);

        // Drop receiver to simulate disconnection
        drop(rx);

        assert!(!transport.is_connected().await);

        let metadata = StreamMetadata {
            request_id: "req-123".to_string(),
            session_id: "session-456".to_string(),
            total_urls: 5,
            config: riptide_types::ports::streaming::StreamConfig::default(),
            started_at: chrono::Utc::now(),
        };

        // Should fail with client disconnected error
        let result = transport.send_metadata(metadata).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(StreamingError::ClientDisconnected { .. })));
    }
}
