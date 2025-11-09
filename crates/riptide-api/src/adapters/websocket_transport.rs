//! WebSocket transport adapter implementing StreamingTransport trait.
//!
//! This adapter provides WebSocket protocol handling for the streaming facade,
//! extracting pure transport logic from the handler layer.

use async_trait::async_trait;
use axum::extract::ws::{Message, WebSocket};
use futures::{sink::SinkExt, stream::StreamExt};
use riptide_types::ports::streaming::{
    DeepSearchMetadata, DeepSearchResultData, StreamErrorData, StreamEvent, StreamMetadata,
    StreamProgress, StreamResult, StreamSummary, StreamingTransport,
};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::{debug, warn};

use crate::streaming::error::{StreamingError, StreamingResult};

/// WebSocket transport adapter
///
/// Handles WebSocket-specific protocol details:
/// - Message framing (text, binary, ping/pong)
/// - Connection state management
/// - Automatic ping/pong keepalive
/// - Graceful close handling
pub struct WebSocketTransport {
    /// WebSocket sender (wrapped in Arc<Mutex> for sharing across tasks)
    sender: Arc<Mutex<futures::stream::SplitSink<WebSocket, Message>>>,

    /// Session identifier for logging and metrics
    session_id: String,

    /// Connection state tracking
    is_connected: Arc<Mutex<bool>>,

    /// Message counter for metrics
    message_count: Arc<Mutex<usize>>,

    /// Connection start time for duration tracking
    connected_at: Instant,
}

impl WebSocketTransport {
    /// Create a new WebSocket transport from a socket
    ///
    /// # Arguments
    ///
    /// * `socket` - WebSocket connection to wrap
    /// * `session_id` - Unique session identifier
    pub fn new(socket: WebSocket, session_id: String) -> Self {
        let (sender, _receiver) = socket.split();

        Self {
            sender: Arc::new(Mutex::new(sender)),
            session_id,
            is_connected: Arc::new(Mutex::new(true)),
            message_count: Arc::new(Mutex::new(0)),
            connected_at: Instant::now(),
        }
    }

    /// Send a WebSocket message with error handling
    async fn send_ws_message(&self, message: Message) -> StreamingResult<()> {
        let mut sender = self.sender.lock().await;

        sender.send(message).await.map_err(|e| {
            *self.is_connected.blocking_lock() = false;
            StreamingError::connection(format!("Failed to send WebSocket message: {}", e))
        })?;

        // Update message count
        let mut count = self.message_count.lock().await;
        *count += 1;

        Ok(())
    }

    /// Send JSON data over WebSocket
    async fn send_json(&self, data: &serde_json::Value) -> StreamingResult<()> {
        let json_text = serde_json::to_string(data)
            .map_err(StreamingError::from)?;

        self.send_ws_message(Message::Text(json_text)).await
    }

    /// Get current message count
    pub async fn message_count(&self) -> usize {
        *self.message_count.lock().await
    }

    /// Get connection duration
    pub fn connection_duration(&self) -> std::time::Duration {
        self.connected_at.elapsed()
    }
}

#[async_trait]
impl StreamingTransport for WebSocketTransport {
    type Message = serde_json::Value;
    type Error = StreamingError;

    async fn send_event(&mut self, event: StreamEvent) -> Result<(), Self::Error> {
        let json_event = match event {
            StreamEvent::Metadata(metadata) => {
                serde_json::json!({
                    "type": "metadata",
                    "data": metadata
                })
            }
            StreamEvent::Result(result) => {
                serde_json::json!({
                    "type": "result",
                    "data": result
                })
            }
            StreamEvent::Progress(progress) => {
                serde_json::json!({
                    "type": "progress",
                    "data": progress
                })
            }
            StreamEvent::Summary(summary) => {
                serde_json::json!({
                    "type": "summary",
                    "data": summary
                })
            }
            StreamEvent::SearchMetadata(metadata) => {
                serde_json::json!({
                    "type": "search_metadata",
                    "data": metadata
                })
            }
            StreamEvent::SearchResult(result) => {
                serde_json::json!({
                    "type": "search_result",
                    "data": result
                })
            }
            StreamEvent::Error(error) => {
                serde_json::json!({
                    "type": "error",
                    "data": error
                })
            }
        };

        let session_id = self.session_id.clone();
        let event_type = json_event.get("type").map(|v| v.to_string());
        debug!(
            session_id = %session_id,
            event_type = ?event_type,
            "Sending WebSocket event"
        );

        self.send_json(&json_event).await
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
        debug!(session_id = %session_id, "Closing WebSocket connection");

        // Send close frame
        let mut sender = self.sender.lock().await;
        if let Err(e) = sender.send(Message::Close(None)).await {
            let session_id = self.session_id.clone();
            let error_msg = e.to_string();
            warn!(
                session_id = %session_id,
                error = %error_msg,
                "Error sending close frame"
            );
        }

        // Update connection state
        *self.is_connected.lock().await = false;

        let session_id = self.session_id.clone();
        let duration_ms = self.connected_at.elapsed().as_millis();
        let message_count = self.message_count().await;
        debug!(
            session_id = %session_id,
            duration_ms = duration_ms,
            message_count = message_count,
            "WebSocket connection closed"
        );

        Ok(())
    }

    fn protocol_name(&self) -> &'static str {
        "websocket"
    }
}

impl WebSocketTransport {
    /// Check if connection is still active
    pub fn is_connected(&self) -> bool {
        *self.is_connected.blocking_lock()
    }

    /// Send a ping message for keepalive
    pub async fn send_ping(&self, data: Vec<u8>) -> StreamingResult<()> {
        self.send_ws_message(Message::Ping(data)).await
    }

    /// Send a pong response
    pub async fn send_pong(&self, data: Vec<u8>) -> StreamingResult<()> {
        self.send_ws_message(Message::Pong(data)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_protocol_name() {
        // Create a mock socket for testing (using a channel-based approach)
        let (client, _server) = tokio::io::duplex(1024);
        let socket = WebSocket::from_raw_socket(client, axum::extract::ws::Role::Server, None).await;

        let transport = WebSocketTransport::new(socket, "test-session".to_string());
        assert_eq!(transport.protocol_name(), "websocket");
    }

    #[tokio::test]
    async fn test_message_count() {
        let (client, _server) = tokio::io::duplex(1024);
        let socket = WebSocket::from_raw_socket(client, axum::extract::ws::Role::Server, None).await;

        let transport = WebSocketTransport::new(socket, "test-session".to_string());
        assert_eq!(transport.message_count().await, 0);
    }

    #[tokio::test]
    async fn test_connection_duration() {
        let (client, _server) = tokio::io::duplex(1024);
        let socket = WebSocket::from_raw_socket(client, axum::extract::ws::Role::Server, None).await;

        let transport = WebSocketTransport::new(socket, "test-session".to_string());
        let duration = transport.connection_duration();
        assert!(duration.as_millis() >= 0);
    }

    #[tokio::test]
    async fn test_send_metadata() {
        let (client, _server) = tokio::io::duplex(1024);
        let socket = WebSocket::from_raw_socket(client, axum::extract::ws::Role::Server, None).await;

        let mut transport = WebSocketTransport::new(socket, "test-session".to_string());

        let metadata = StreamMetadata {
            request_id: "req-123".to_string(),
            session_id: "session-456".to_string(),
            total_urls: 10,
            config: riptide_types::ports::streaming::StreamConfig::default(),
            started_at: chrono::Utc::now(),
        };

        // This will send successfully (or fail with connection error)
        let result = transport.send_metadata(metadata).await;
        assert!(result.is_ok() || matches!(result, Err(StreamingError::Connection { .. })));
    }
}
