//! WebSocket streaming implementation for real-time bidirectional communication.
//!
//! This module provides WebSocket endpoints for real-time streaming with
//! backpressure handling, connection management, and message routing.

use super::buffer::{BackpressureHandler, BufferManager};
use super::config::StreamConfig;
use super::error::{ClientType, ConnectionContext, StreamingError, StreamingResult};
// use super::metrics::WebSocketMetrics; // Unused
use crate::models::*;
use crate::pipeline::PipelineOrchestrator;
use crate::state::AppState;
use crate::validation::validate_crawl_request;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures::{sink::SinkExt, stream::SplitSink, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// WebSocket endpoint for bidirectional real-time communication
pub async fn crawl_websocket(
    ws: WebSocketUpgrade,
    State(app): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket(socket, app))
}

/// Handle WebSocket connection for real-time streaming
pub async fn handle_websocket(socket: WebSocket, app: AppState) {
    let session_id = Uuid::new_v4().to_string();
    let (sender, receiver) = socket.split();

    info!(session_id = %session_id, "WebSocket connection established");

    let connection_context = ConnectionContext {
        session_id: session_id.clone(),
        client_type: ClientType::WebSocket,
        connected_at: Instant::now(),
    };

    // Create WebSocket handler with proper state management
    let mut handler = WebSocketHandler::new(app, connection_context).await;

    // Handle the WebSocket session
    if let Err(e) = handler.handle_session(sender, receiver).await {
        error!(session_id = %session_id, error = %e, "WebSocket session error");
    }

    info!(session_id = %session_id, "WebSocket connection closed");
}

/// WebSocket session handler with connection management
pub struct WebSocketHandler {
    app: AppState,
    context: ConnectionContext,
    connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
    buffer_manager: Arc<BufferManager>,
    config: StreamConfig,
}

impl WebSocketHandler {
    /// Create a new WebSocket handler
    pub async fn new(app: AppState, context: ConnectionContext) -> Self {
        let buffer_manager = Arc::new(BufferManager::new());
        let config = StreamConfig::from_env();

        Self {
            app,
            context,
            connections: Arc::new(RwLock::new(HashMap::new())),
            buffer_manager,
            config,
        }
    }

    /// Handle a complete WebSocket session with ping/pong keepalive
    pub async fn handle_session(
        &mut self,
        mut sender: SplitSink<WebSocket, Message>,
        mut receiver: futures_util::stream::SplitStream<WebSocket>,
    ) -> StreamingResult<()> {
        // Register connection
        self.register_connection().await;

        // Send welcome message
        self.send_welcome_message(&mut sender).await?;

        // Spawn ping task for keepalive (30-second interval)
        let session_id_clone = self.context.session_id.clone();
        let ping_interval = self.config.websocket.ping_interval;
        let sender_clone = Arc::new(tokio::sync::Mutex::new(sender));
        let sender_for_ping = sender_clone.clone();

        let ping_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(ping_interval);
            loop {
                interval.tick().await;

                // Send ping with timestamp as payload
                let ping_data = format!("{}", chrono::Utc::now().timestamp_millis());
                let mut sender = sender_for_ping.lock().await;

                if let Err(e) = sender.send(Message::Ping(ping_data.into_bytes())).await {
                    debug!(session_id = %session_id_clone, error = %e, "Failed to send ping, connection likely closed");
                    break;
                }

                debug!(session_id = %session_id_clone, "Sent WebSocket ping");
            }
        });

        // Handle incoming messages
        loop {
            let msg = receiver.next().await;

            match msg {
                Some(Ok(Message::Text(text))) => {
                    let mut sender_guard = sender_clone.lock().await;
                    if let Err(e) = self.handle_text_message(text, &mut sender_guard).await {
                        error!(
                            session_id = %self.context.session_id,
                            error = %e,
                            "Error handling text message"
                        );
                    }
                }
                Some(Ok(Message::Binary(data))) => {
                    debug!(
                        session_id = %self.context.session_id,
                        size = data.len(),
                        "Received binary WebSocket frame"
                    );
                    // Binary frame support for efficient data transfer
                    // Can be used for compressed payloads or custom protocols
                }
                Some(Ok(Message::Close(_))) => {
                    info!(session_id = %self.context.session_id, "WebSocket connection closed by client");
                    break;
                }
                Some(Ok(Message::Ping(data))) => {
                    // Respond to ping with pong (echo the data)
                    let mut sender_guard = sender_clone.lock().await;
                    if let Err(e) = sender_guard.send(Message::Pong(data)).await {
                        warn!(
                            session_id = %self.context.session_id,
                            error = %e,
                            "Failed to send pong"
                        );
                        break;
                    }
                    self.update_connection_ping().await;
                    debug!(session_id = %self.context.session_id, "Received ping, sent pong");
                }
                Some(Ok(Message::Pong(_data))) => {
                    // Pong received in response to our ping
                    self.update_connection_ping().await;
                    debug!(session_id = %self.context.session_id, "Received pong response");
                }
                Some(Err(e)) => {
                    warn!(
                        session_id = %self.context.session_id,
                        error = %e,
                        "WebSocket error"
                    );
                    break;
                }
                None => {
                    debug!(session_id = %self.context.session_id, "WebSocket stream ended");
                    break;
                }
            }
        }

        // Cancel ping task
        ping_task.abort();

        // Clean up connection
        self.cleanup_connection().await;
        Ok(())
    }

    /// Register connection in the connection registry
    async fn register_connection(&self) {
        let mut connections = self.connections.write().await;
        connections.insert(
            self.context.session_id.clone(),
            ConnectionInfo {
                last_ping: Instant::now(),
                backpressure_count: 0,
                is_slow: false,
                message_count: 0,
                connection_start: self.context.connected_at,
            },
        );
    }

    /// Update connection ping time
    async fn update_connection_ping(&self) {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(&self.context.session_id) {
            conn.last_ping = Instant::now();
        }
    }

    /// Clean up connection
    async fn cleanup_connection(&self) {
        let mut connections = self.connections.write().await;
        connections.remove(&self.context.session_id);
        self.buffer_manager
            .remove_buffer(&self.context.session_id)
            .await;
    }

    /// Send welcome message to newly connected client
    async fn send_welcome_message(
        &self,
        sender: &mut SplitSink<WebSocket, Message>,
    ) -> StreamingResult<()> {
        let welcome_msg = WebSocketMessage {
            message_type: "welcome".to_string(),
            data: serde_json::json!({
                "session_id": self.context.session_id,
                "server_time": chrono::Utc::now().to_rfc3339(),
                "protocol_version": "1.0",
                "supported_operations": ["crawl", "ping", "status"]
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.send_message(sender, &welcome_msg).await
    }

    /// Handle incoming text message
    async fn handle_text_message(
        &self,
        text: String,
        sender: &mut SplitSink<WebSocket, Message>,
    ) -> StreamingResult<()> {
        let request: WebSocketRequest = serde_json::from_str(&text)
            .map_err(|e| StreamingError::invalid_request(format!("Invalid JSON: {}", e)))?;

        match request.request_type.as_str() {
            "crawl" => self.handle_crawl_request(request, sender).await,
            "ping" => self.handle_ping_request(sender).await,
            "status" => self.handle_status_request(sender).await,
            _ => {
                let error_msg = WebSocketMessage {
                    message_type: "error".to_string(),
                    data: serde_json::json!({
                        "error_type": "unknown_request",
                        "message": format!("Unknown request type: {}", request.request_type)
                    }),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                self.send_message(sender, &error_msg).await
            }
        }
    }

    /// Handle crawl request
    async fn handle_crawl_request(
        &self,
        request: WebSocketRequest,
        sender: &mut SplitSink<WebSocket, Message>,
    ) -> StreamingResult<()> {
        let crawl_body: CrawlBody = serde_json::from_value(request.data).map_err(|e| {
            StreamingError::invalid_request(format!("Invalid crawl request: {}", e))
        })?;

        // Validate the request
        if let Err(e) = validate_crawl_request(&crawl_body) {
            let error_msg = WebSocketMessage {
                message_type: "error".to_string(),
                data: serde_json::json!({
                    "error_type": "validation_error",
                    "message": e.to_string()
                }),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            return self.send_message(sender, &error_msg).await;
        }

        // Start streaming crawl
        self.stream_crawl_websocket(crawl_body, sender).await
    }

    /// Handle ping request
    async fn handle_ping_request(
        &self,
        sender: &mut SplitSink<WebSocket, Message>,
    ) -> StreamingResult<()> {
        let pong_msg = WebSocketMessage {
            message_type: "pong".to_string(),
            data: serde_json::json!({
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "session_id": self.context.session_id
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        self.send_message(sender, &pong_msg).await
    }

    /// Handle status request
    async fn handle_status_request(
        &self,
        sender: &mut SplitSink<WebSocket, Message>,
    ) -> StreamingResult<()> {
        let connections = self.connections.read().await;
        let conn_info = connections.get(&self.context.session_id);

        let status_msg = WebSocketMessage {
            message_type: "status".to_string(),
            data: serde_json::json!({
                "session_id": self.context.session_id,
                "connected_duration_ms": self.context.connected_at.elapsed().as_millis(),
                "is_healthy": conn_info.map(|c| !c.is_slow).unwrap_or(false),
                "message_count": conn_info.map(|c| c.message_count).unwrap_or(0),
                "backpressure_count": conn_info.map(|c| c.backpressure_count).unwrap_or(0)
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        self.send_message(sender, &status_msg).await
    }

    /// Stream crawl results over WebSocket with backpressure handling
    async fn stream_crawl_websocket(
        &self,
        body: CrawlBody,
        sender: &mut SplitSink<WebSocket, Message>,
    ) -> StreamingResult<()> {
        let start_time = Instant::now();
        let options = body.options.unwrap_or_default();

        // Get buffer and create backpressure handler
        let buffer = self
            .buffer_manager
            .get_buffer(&self.context.session_id)
            .await;
        let mut backpressure_handler =
            BackpressureHandler::new(self.context.session_id.clone(), buffer);

        // Send initial metadata
        let metadata_msg = WebSocketMessage {
            message_type: "metadata".to_string(),
            data: serde_json::json!({
                "total_urls": body.urls.len(),
                "session_id": self.context.session_id,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "stream_type": "crawl"
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.send_message_with_backpressure(sender, &metadata_msg, &mut backpressure_handler)
            .await?;

        // Create pipeline orchestrator
        let pipeline = PipelineOrchestrator::new(self.app.clone(), options);
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

        // Stream results with backpressure monitoring
        let mut completed_count = 0;
        let mut error_count = 0;

        while let Some((index, url, pipeline_result)) = result_rx.recv().await {
            // Check if connection is still healthy
            if !self.is_connection_healthy().await {
                warn!(session_id = %self.context.session_id, "Terminating stream due to unhealthy connection");
                break;
            }

            let crawl_result = match pipeline_result {
                Ok(result) => {
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

            let result_msg = WebSocketMessage {
                message_type: "result".to_string(),
                data: serde_json::json!({
                    "index": index,
                    "result": crawl_result,
                    "progress": {
                        "completed": completed_count + error_count,
                        "total": body.urls.len(),
                        "success_rate": if completed_count + error_count > 0 {
                            completed_count as f64 / (completed_count + error_count) as f64
                        } else { 0.0 }
                    }
                }),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };

            // Send with backpressure handling
            if let Err(e) = self
                .send_message_with_backpressure(sender, &result_msg, &mut backpressure_handler)
                .await
            {
                debug!(session_id = %self.context.session_id, error = %e, "Client disconnected or error sending");
                break;
            }
        }

        // Send final summary
        let summary_msg = WebSocketMessage {
            message_type: "summary".to_string(),
            data: serde_json::json!({
                "total_urls": body.urls.len(),
                "successful": completed_count,
                "failed": error_count,
                "total_processing_time_ms": start_time.elapsed().as_millis() as u64
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.send_message_with_backpressure(sender, &summary_msg, &mut backpressure_handler)
            .await?;

        info!(
            session_id = %self.context.session_id,
            total_urls = body.urls.len(),
            successful = completed_count,
            failed = error_count,
            total_time_ms = start_time.elapsed().as_millis(),
            "WebSocket crawl streaming completed"
        );

        Ok(())
    }

    /// Send WebSocket message with error handling
    async fn send_message(
        &self,
        sender: &mut SplitSink<WebSocket, Message>,
        message: &WebSocketMessage,
    ) -> StreamingResult<()> {
        let message_text = serde_json::to_string(message).map_err(StreamingError::from)?;

        sender
            .send(Message::Text(message_text))
            .await
            .map_err(|e| {
                StreamingError::connection(format!("Failed to send WebSocket message: {}", e))
            })?;

        // Update message count
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(&self.context.session_id) {
            conn.message_count += 1;
        }

        Ok(())
    }

    /// Send WebSocket message with backpressure handling
    async fn send_message_with_backpressure(
        &self,
        sender: &mut SplitSink<WebSocket, Message>,
        message: &WebSocketMessage,
        backpressure_handler: &mut BackpressureHandler,
    ) -> StreamingResult<()> {
        // Check if we should drop this message due to backpressure
        if backpressure_handler.should_drop_message(0).await {
            warn!(session_id = %self.context.session_id, "Dropping WebSocket message due to backpressure");
            return Ok(());
        }

        let send_start = Instant::now();
        self.send_message(sender, message).await?;
        let send_duration = send_start.elapsed();

        // Record send metrics
        backpressure_handler.record_send_time(send_duration).await?;

        // Update backpressure metrics in connection info
        if send_duration
            > Duration::from_millis(
                self.config.websocket.client_response_timeout.as_millis() as u64 / 10,
            )
        {
            let mut connections = self.connections.write().await;
            if let Some(conn) = connections.get_mut(&self.context.session_id) {
                conn.backpressure_count += 1;
                if conn.backpressure_count > 10 {
                    conn.is_slow = true;
                    warn!(session_id = %self.context.session_id, "Marking WebSocket connection as slow");
                }
            }
        }

        Ok(())
    }

    /// Check if connection is healthy
    async fn is_connection_healthy(&self) -> bool {
        let connections = self.connections.read().await;
        if let Some(conn) = connections.get(&self.context.session_id) {
            !conn.is_slow && conn.backpressure_count < 100
        } else {
            false
        }
    }
}

/// Connection information for backpressure management
#[derive(Debug)]
pub struct ConnectionInfo {
    pub last_ping: Instant,
    pub backpressure_count: usize,
    pub is_slow: bool,
    pub message_count: usize,
    pub connection_start: Instant,
}

/// WebSocket request message structure
#[derive(Deserialize, Debug)]
pub struct WebSocketRequest {
    pub request_type: String,
    pub data: serde_json::Value,
}

/// WebSocket response message structure
#[derive(Serialize, Debug)]
pub struct WebSocketMessage {
    pub message_type: String,
    pub data: serde_json::Value,
    pub timestamp: String,
}

// Note: WebSocketMetrics is now imported from super::metrics
// The previous duplicate implementation (52 lines) has been removed
// and replaced with the shared StreamingMetrics from metrics.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_request_deserialization() {
        let json = r#"{"request_type": "crawl", "data": {"urls": ["https://example.com"]}}"#;
        let request: WebSocketRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.request_type, "crawl");
    }

    #[test]
    fn test_websocket_message_serialization() {
        let message = WebSocketMessage {
            message_type: "result".to_string(),
            data: serde_json::json!({"status": "success"}),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("\"message_type\":\"result\""));
    }

    #[test]
    fn test_connection_info() {
        let info = ConnectionInfo {
            last_ping: Instant::now(),
            backpressure_count: 5,
            is_slow: false,
            message_count: 100,
            connection_start: Instant::now(),
        };

        assert_eq!(info.backpressure_count, 5);
        assert!(!info.is_slow);
        assert_eq!(info.message_count, 100);
    }

    #[test]
    fn test_websocket_metrics() {
        let mut metrics = WebSocketMetrics::default();

        metrics.record_connection();
        assert_eq!(metrics.active_connections, 1);
        assert_eq!(metrics.total_connections, 1);

        metrics.record_item_sent();
        metrics.record_item_received();
        assert_eq!(metrics.total_items_sent, 1);
        assert_eq!(metrics.total_items_received, 1);

        metrics.record_disconnection(Duration::from_secs(60));
        assert_eq!(metrics.active_connections, 0);
        assert_eq!(metrics.average_connection_duration_ms, 60000.0);
    }

    #[test]
    fn test_health_ratio() {
        let mut metrics = WebSocketMetrics::default();

        // Perfect health
        metrics.total_connections = 10;
        metrics.error_count = 0;
        assert_eq!(metrics.health_ratio(), 1.0);

        // Some errors
        metrics.error_count = 2;
        assert_eq!(metrics.health_ratio(), 0.8);

        // No connections
        metrics.total_connections = 0;
        assert_eq!(metrics.health_ratio(), 1.0);
    }
}
