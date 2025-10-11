//! Common streaming response helpers and utilities.
//!
//! This module provides reusable response handling patterns for different
//! streaming protocols (NDJSON, SSE, WebSocket) to reduce code duplication.
//!
//! # Response Format Specifications
//!
//! ## NDJSON (Newline Delimited JSON)
//!
//! Each line contains a complete JSON object followed by a newline (`\n`):
//! ```text
//! {"type":"metadata","request_id":"123","total_urls":5}
//! {"type":"result","index":0,"data":{"url":"https://example.com","status":200}}
//! {"type":"progress","completed":1,"total":5}
//! {"type":"completion","summary":{"total":5,"successful":5}}
//! ```
//!
//! Content-Type: `application/x-ndjson`
//!
//! ## SSE (Server-Sent Events)
//!
//! Events follow the SSE protocol with event type, data, and optional ID:
//! ```text
//! event: metadata
//! data: {"request_id":"123","total_urls":5}
//! id: 0
//!
//! event: result
//! data: {"index":0,"url":"https://example.com","status":200}
//! id: 1
//!
//! event: progress
//! data: {"completed":1,"total":5}
//!
//! event: completion
//! data: {"summary":{"total":5,"successful":5}}
//!
//! ```
//!
//! Content-Type: `text/event-stream`
//!
//! Keep-alive comments (sent every 30 seconds):
//! ```text
//! : keep-alive 2024-01-01T00:00:00Z
//!
//! ```
//!
//! ## JSON (Single Response)
//!
//! Single JSON object containing all results:
//! ```json
//! [
//!   {"index":0,"url":"https://example.com","status":200},
//!   {"index":1,"url":"https://example.org","status":200}
//! ]
//! ```
//!
//! Content-Type: `application/json`
//!
//! # Error Response Formats
//!
//! All error responses follow a consistent structure across protocols:
//!
//! ## NDJSON Error
//! ```json
//! {"error":"validation_error","message":"Invalid request","retryable":false}
//! ```
//!
//! ## SSE Error
//! ```text
//! event: error
//! data: {"error":"processing_error","message":"Failed to process","retryable":true}
//!
//! ```
//!
//! ## JSON Error
//! ```json
//! {"error":"internal_error","message":"Server error occurred","retryable":false}
//! ```
//!
//! # Usage Examples
//!
//! ## Creating an NDJSON Streaming Response
//!
//! ```rust,no_run
//! use riptide_api::streaming::response_helpers::{StreamingResponseBuilder, StreamingResponseType};
//! use futures_util::stream;
//! use serde_json::json;
//!
//! let data_stream = stream::iter(vec![
//!     json!({"index": 0, "value": "first"}),
//!     json!({"index": 1, "value": "second"}),
//! ]);
//!
//! let response = StreamingResponseBuilder::new(StreamingResponseType::Ndjson)
//!     .build(data_stream);
//! ```
//!
//! ## Creating an SSE Response
//!
//! ```rust,no_run
//! use riptide_api::streaming::response_helpers::{StreamingResponseBuilder, StreamingResponseType};
//! use futures_util::stream;
//! use serde_json::json;
//!
//! let events = stream::iter(vec![
//!     json!({"event": "start", "timestamp": "2024-01-01T00:00:00Z"}),
//!     json!({"event": "data", "value": 42}),
//! ]);
//!
//! let response = StreamingResponseBuilder::new(StreamingResponseType::Sse)
//!     .build(events);
//! ```
//!
//! ## Creating Error Responses
//!
//! ```rust,no_run
//! use riptide_api::streaming::response_helpers::StreamingErrorResponse;
//! use serde_json::json;
//!
//! let error = json!({
//!     "error": "validation_failed",
//!     "message": "Invalid input parameters",
//!     "retryable": false
//! });
//!
//! // NDJSON error
//! let ndjson_error = StreamingErrorResponse::ndjson(&error);
//!
//! // SSE error
//! let sse_error = StreamingErrorResponse::sse(&error);
//!
//! // JSON error
//! let json_error = StreamingErrorResponse::json(&error);
//! ```
//!
//! ## Using Helper Messages
//!
//! ```rust,no_run
//! use riptide_api::streaming::response_helpers::{KeepAliveHelper, ProgressHelper, CompletionHelper};
//! use serde_json::json;
//!
//! // Keep-alive message
//! let keepalive = KeepAliveHelper::ndjson_message();
//!
//! // Progress update
//! let progress = ProgressHelper::ndjson_message(json!({
//!     "current": 5,
//!     "total": 10,
//!     "percent": 50.0
//! }));
//!
//! // Completion message
//! let completion = CompletionHelper::ndjson_message(json!({
//!     "total": 10,
//!     "successful": 9,
//!     "failed": 1
//! }));
//! ```

use axum::{
    http::{HeaderMap, HeaderValue, StatusCode},
    response::Response,
};
use futures_util::{stream::Stream, StreamExt};
use serde::Serialize;
use serde_json::json;
use std::convert::Infallible;
use tokio_stream::wrappers::ReceiverStream;

/// Common streaming response types
#[derive(Debug, Clone)]
pub enum StreamingResponseType {
    /// NDJSON (Newline Delimited JSON) streaming
    Ndjson,
    /// Server-Sent Events
    Sse,
    /// JSON streaming (for WebSocket-like responses)
    Json,
}

impl StreamingResponseType {
    /// Get the content type for this response type
    pub fn content_type(&self) -> &'static str {
        match self {
            StreamingResponseType::Ndjson => "application/x-ndjson",
            StreamingResponseType::Sse => "text/event-stream",
            StreamingResponseType::Json => "application/json",
        }
    }

    /// Get recommended headers for this response type
    pub fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        headers.insert(
            "content-type",
            HeaderValue::from_static(self.content_type()),
        );

        headers.insert("cache-control", HeaderValue::from_static("no-cache"));

        match self {
            StreamingResponseType::Ndjson => {
                headers.insert("x-accel-buffering", HeaderValue::from_static("no"));
                headers.insert("connection", HeaderValue::from_static("keep-alive"));
            }
            StreamingResponseType::Sse => {
                headers.insert("connection", HeaderValue::from_static("keep-alive"));
                headers.insert("access-control-allow-origin", HeaderValue::from_static("*"));
                headers.insert(
                    "access-control-allow-headers",
                    HeaderValue::from_static("cache-control"),
                );
            }
            StreamingResponseType::Json => {
                headers.insert("connection", HeaderValue::from_static("close"));
            }
        }

        headers
    }

    /// Check if this response type supports keep-alive
    pub fn supports_keep_alive(&self) -> bool {
        matches!(
            self,
            StreamingResponseType::Ndjson | StreamingResponseType::Sse
        )
    }

    /// Get recommended buffer size for this response type
    pub fn buffer_size(&self) -> usize {
        match self {
            StreamingResponseType::Ndjson => 256,
            StreamingResponseType::Sse => 128,
            StreamingResponseType::Json => 64,
        }
    }
}

/// Common streaming response builder
pub struct StreamingResponseBuilder {
    response_type: StreamingResponseType,
    status: StatusCode,
    headers: HeaderMap,
    compression: bool,
}

impl StreamingResponseBuilder {
    /// Create a new response builder
    pub fn new(response_type: StreamingResponseType) -> Self {
        let headers = response_type.headers();
        Self {
            response_type,
            status: StatusCode::OK,
            headers,
            compression: false,
        }
    }

    /// Set the HTTP status code
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Add custom headers
    #[allow(dead_code)]
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        for (key, value) in headers {
            if let Some(key) = key {
                self.headers.insert(key, value);
            }
        }
        self
    }

    /// Add a single header
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: axum::http::header::IntoHeaderName,
        V: TryInto<HeaderValue>,
    {
        if let Ok(value) = value.try_into() {
            self.headers.insert(key, value);
        }
        self
    }

    /// Enable compression (if supported by the client)
    pub fn with_compression(mut self) -> Self {
        self.compression = true;
        self
    }

    /// Build the streaming response
    pub fn build<S, T>(self, stream: S) -> Response
    where
        S: Stream<Item = T> + Send + 'static,
        T: Serialize + Send + 'static,
    {
        match self.response_type {
            StreamingResponseType::Ndjson => self.build_ndjson_response(stream),
            StreamingResponseType::Sse => self.build_sse_response(stream),
            StreamingResponseType::Json => self.build_json_response(stream),
        }
    }

    /// Build NDJSON streaming response
    fn build_ndjson_response<S, T>(self, stream: S) -> Response
    where
        S: Stream<Item = T> + Send + 'static,
        T: Serialize + Send,
    {
        let stream = stream.map(|item| match serde_json::to_string(&item) {
            Ok(json) => Ok::<String, Infallible>(format!("{}\n", json)),
            Err(e) => {
                tracing::error!("Failed to serialize item to NDJSON: {}", e);
                Ok(format!(
                    "{{\"error\":\"serialization_failed\",\"message\":\"{}\"}}\n",
                    e
                ))
            }
        });

        let body = axum::body::Body::from_stream(stream);

        let mut response = Response::builder().status(self.status).body(body).unwrap();

        *response.headers_mut() = self.headers;

        if self.compression {
            // Note: In practice, you'd apply compression middleware
            response
                .headers_mut()
                .insert("vary", HeaderValue::from_static("accept-encoding"));
        }

        response
    }

    /// Build SSE streaming response
    fn build_sse_response<S, T>(self, stream: S) -> Response
    where
        S: Stream<Item = T> + Send + 'static,
        T: Serialize + Send,
    {
        let stream = stream.map(|item| {
            match serde_json::to_string(&item) {
                Ok(json) => Ok::<String, Infallible>(format!("data: {}\n\n", json)),
                Err(e) => {
                    tracing::error!("Failed to serialize item for SSE: {}", e);
                    Ok(format!("event: error\ndata: {{\"error\":\"serialization_failed\",\"message\":\"{}\"}}\n\n", e))
                }
            }
        });

        let body = axum::body::Body::from_stream(stream);

        let mut response = Response::builder().status(self.status).body(body).unwrap();

        *response.headers_mut() = self.headers;
        response
    }

    /// Build JSON streaming response (for single large responses)
    fn build_json_response<S, T>(self, stream: S) -> Response
    where
        S: Stream<Item = T> + Send + 'static,
        T: Serialize + Send + 'static,
    {
        // For JSON responses, we collect all items into a single JSON array
        let stream = stream.collect::<Vec<_>>();
        let body = axum::body::Body::from_stream(futures_util::stream::once(async move {
            match serde_json::to_string(&stream.await) {
                Ok(json) => Ok::<String, Infallible>(json),
                Err(e) => {
                    tracing::error!("Failed to serialize items to JSON: {}", e);
                    Ok(format!(
                        "{{\"error\":\"serialization_failed\",\"message\":\"{}\"}}",
                        e
                    ))
                }
            }
        }));

        let mut response = Response::builder().status(self.status).body(body).unwrap();

        *response.headers_mut() = self.headers;
        response
    }
}

/// Helper for creating error responses in streaming format
pub struct StreamingErrorResponse;

impl StreamingErrorResponse {
    /// Create an NDJSON error response
    pub fn ndjson(error: impl Serialize) -> Response {
        let error_json = match serde_json::to_string(&error) {
            Ok(json) => format!("{}\n", json),
            Err(e) => format!(
                "{{\"error\":\"serialization_failed\",\"message\":\"{}\"}}\n",
                e
            ),
        };

        let headers = StreamingResponseType::Ndjson.headers();
        let mut response = Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(axum::body::Body::from(error_json))
            .unwrap();

        *response.headers_mut() = headers;
        response
    }

    /// Create an SSE error response
    pub fn sse(error: impl Serialize) -> Response {
        let error_sse = match serde_json::to_string(&error) {
            Ok(json) => format!("event: error\ndata: {}\n\n", json),
            Err(e) => format!(
                "event: error\ndata: {{\"error\":\"serialization_failed\",\"message\":\"{}\"}}\n\n",
                e
            ),
        };

        let headers = StreamingResponseType::Sse.headers();
        let mut response = Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(axum::body::Body::from(error_sse))
            .unwrap();

        *response.headers_mut() = headers;
        response
    }

    /// Create a JSON error response
    pub fn json(error: impl Serialize) -> Response {
        match serde_json::to_string(&error) {
            Ok(json) => {
                let headers = StreamingResponseType::Json.headers();
                let mut response = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(axum::body::Body::from(json))
                    .unwrap();

                *response.headers_mut() = headers;
                response
            }
            Err(e) => {
                let fallback = format!(
                    "{{\"error\":\"serialization_failed\",\"message\":\"{}\"}}",
                    e
                );
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(fallback))
                    .unwrap()
            }
        }
    }

    /// Create error response based on response type
    pub fn for_type(response_type: StreamingResponseType, error: impl Serialize) -> Response {
        match response_type {
            StreamingResponseType::Ndjson => Self::ndjson(error),
            StreamingResponseType::Sse => Self::sse(error),
            StreamingResponseType::Json => Self::json(error),
        }
    }
}

/// Helper for creating keep-alive messages
pub struct KeepAliveHelper;

impl KeepAliveHelper {
    /// Create NDJSON keep-alive message
    pub fn ndjson_message() -> String {
        format!(
            "{{\"type\":\"keep-alive\",\"timestamp\":\"{}\"}}\n",
            chrono::Utc::now().to_rfc3339()
        )
    }

    /// Create SSE keep-alive message
    pub fn sse_message() -> String {
        format!(": keep-alive {}\n\n", chrono::Utc::now().to_rfc3339())
    }

    /// Create keep-alive message for specific type
    pub fn for_type(response_type: StreamingResponseType) -> String {
        match response_type {
            StreamingResponseType::Ndjson => Self::ndjson_message(),
            StreamingResponseType::Sse => Self::sse_message(),
            StreamingResponseType::Json => "".to_string(), // JSON doesn't need keep-alive
        }
    }
}

/// Helper for creating completion messages
pub struct CompletionHelper;

impl CompletionHelper {
    /// Create NDJSON completion message
    pub fn ndjson_message(summary: impl Serialize) -> String {
        match serde_json::to_string(&json!({
            "type": "completion",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "summary": summary
        })) {
            Ok(json) => format!("{}\n", json),
            Err(e) => format!("{{\"type\":\"completion\",\"error\":\"{}\"}}\n", e),
        }
    }

    /// Create SSE completion message
    pub fn sse_message(summary: impl Serialize) -> String {
        match serde_json::to_string(&summary) {
            Ok(json) => format!("event: completion\ndata: {}\n\n", json),
            Err(e) => format!("event: error\ndata: {{\"error\":\"completion_serialization_failed\",\"message\":\"{}\"}}\n\n", e),
        }
    }

    /// Create completion message for specific type
    pub fn for_type(response_type: StreamingResponseType, summary: impl Serialize) -> String {
        match response_type {
            StreamingResponseType::Ndjson => Self::ndjson_message(summary),
            StreamingResponseType::Sse => Self::sse_message(summary),
            StreamingResponseType::Json => "".to_string(), // JSON sends everything at once
        }
    }
}

/// Helper for creating progress messages
#[allow(dead_code)]
pub struct ProgressHelper;

#[allow(dead_code)]
impl ProgressHelper {
    /// Create NDJSON progress message
    pub fn ndjson_message(progress: impl Serialize) -> String {
        match serde_json::to_string(&json!({
            "type": "progress",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "data": progress
        })) {
            Ok(json) => format!("{}\n", json),
            Err(e) => format!("{{\"type\":\"progress\",\"error\":\"{}\"}}\n", e),
        }
    }

    /// Create SSE progress message
    pub fn sse_message(progress: impl Serialize) -> String {
        match serde_json::to_string(&progress) {
            Ok(json) => format!("event: progress\ndata: {}\n\n", json),
            Err(e) => format!("event: error\ndata: {{\"error\":\"progress_serialization_failed\",\"message\":\"{}\"}}\n\n", e),
        }
    }

    /// Create progress message for specific type
    pub fn for_type(response_type: StreamingResponseType, progress: impl Serialize) -> String {
        match response_type {
            StreamingResponseType::Ndjson => Self::ndjson_message(progress),
            StreamingResponseType::Sse => Self::sse_message(progress),
            StreamingResponseType::Json => "".to_string(), // JSON doesn't send individual progress
        }
    }
}

/// Utility for creating streaming responses from channels
#[allow(dead_code)]
pub fn stream_from_receiver<T>(
    receiver: tokio::sync::mpsc::Receiver<T>,
    response_type: StreamingResponseType,
) -> Response
where
    T: Serialize + Send + 'static,
{
    let stream = ReceiverStream::new(receiver);
    StreamingResponseBuilder::new(response_type).build(stream)
}

/// Utility for creating streaming responses with error handling
#[allow(dead_code)]
pub fn safe_stream_response<S, T>(stream: S, response_type: StreamingResponseType) -> Response
where
    S: Stream<Item = Result<T, Box<dyn std::error::Error + Send + Sync>>> + Send + 'static,
    T: Serialize + Send + 'static + Default,
{
    let safe_stream = stream.map(|result: Result<T, _>| result.unwrap_or_default());

    StreamingResponseBuilder::new(response_type).build(safe_stream)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use serde_json::Value;

    #[test]
    fn test_streaming_response_types() {
        assert_eq!(
            StreamingResponseType::Ndjson.content_type(),
            "application/x-ndjson"
        );
        assert_eq!(
            StreamingResponseType::Sse.content_type(),
            "text/event-stream"
        );
        assert_eq!(
            StreamingResponseType::Json.content_type(),
            "application/json"
        );

        assert!(StreamingResponseType::Ndjson.supports_keep_alive());
        assert!(StreamingResponseType::Sse.supports_keep_alive());
        assert!(!StreamingResponseType::Json.supports_keep_alive());
    }

    #[test]
    fn test_buffer_sizes() {
        assert_eq!(StreamingResponseType::Ndjson.buffer_size(), 256);
        assert_eq!(StreamingResponseType::Sse.buffer_size(), 128);
        assert_eq!(StreamingResponseType::Json.buffer_size(), 64);
    }

    #[test]
    fn test_response_type_headers() {
        let ndjson_headers = StreamingResponseType::Ndjson.headers();
        assert_eq!(
            ndjson_headers.get("content-type").unwrap(),
            "application/x-ndjson"
        );
        assert_eq!(ndjson_headers.get("cache-control").unwrap(), "no-cache");
        assert_eq!(ndjson_headers.get("connection").unwrap(), "keep-alive");

        let sse_headers = StreamingResponseType::Sse.headers();
        assert_eq!(
            sse_headers.get("content-type").unwrap(),
            "text/event-stream"
        );
        assert_eq!(sse_headers.get("connection").unwrap(), "keep-alive");

        let json_headers = StreamingResponseType::Json.headers();
        assert_eq!(
            json_headers.get("content-type").unwrap(),
            "application/json"
        );
        assert_eq!(json_headers.get("connection").unwrap(), "close");
    }

    #[test]
    fn test_keep_alive_helpers() {
        let ndjson_msg = KeepAliveHelper::ndjson_message();
        assert!(ndjson_msg.contains("keep-alive"));
        assert!(ndjson_msg.ends_with('\n'));
        let parsed: Value = serde_json::from_str(ndjson_msg.trim()).unwrap();
        assert_eq!(parsed["type"], "keep-alive");
        assert!(parsed["timestamp"].is_string());

        let sse_msg = KeepAliveHelper::sse_message();
        assert!(sse_msg.starts_with(": keep-alive"));
        assert!(sse_msg.ends_with("\n\n"));
    }

    #[test]
    fn test_keep_alive_for_type() {
        let ndjson = KeepAliveHelper::for_type(StreamingResponseType::Ndjson);
        assert!(ndjson.contains("keep-alive"));

        let sse = KeepAliveHelper::for_type(StreamingResponseType::Sse);
        assert!(sse.starts_with(":"));

        let json = KeepAliveHelper::for_type(StreamingResponseType::Json);
        assert!(json.is_empty());
    }

    #[test]
    fn test_completion_helpers() {
        let summary = json!({ "total": 5, "processed": 5 });

        let ndjson_msg = CompletionHelper::ndjson_message(&summary);
        assert!(ndjson_msg.contains("completion"));
        assert!(ndjson_msg.ends_with('\n'));
        let parsed: Value = serde_json::from_str(ndjson_msg.trim()).unwrap();
        assert_eq!(parsed["type"], "completion");
        assert_eq!(parsed["summary"]["total"], 5);

        let sse_msg = CompletionHelper::sse_message(&summary);
        assert!(sse_msg.starts_with("event: completion"));
        assert!(sse_msg.contains("data:"));
        assert!(sse_msg.ends_with("\n\n"));
    }

    #[test]
    fn test_completion_for_type() {
        let summary = json!({ "items": 10 });

        let ndjson = CompletionHelper::for_type(StreamingResponseType::Ndjson, &summary);
        assert!(ndjson.contains("completion"));

        let sse = CompletionHelper::for_type(StreamingResponseType::Sse, &summary);
        assert!(sse.starts_with("event: completion"));

        let json = CompletionHelper::for_type(StreamingResponseType::Json, &summary);
        assert!(json.is_empty());
    }

    #[test]
    fn test_progress_helpers() {
        let progress = json!({ "current": 3, "total": 10 });

        let ndjson_msg = ProgressHelper::ndjson_message(&progress);
        assert!(ndjson_msg.contains("progress"));
        assert!(ndjson_msg.ends_with('\n'));
        let parsed: Value = serde_json::from_str(ndjson_msg.trim()).unwrap();
        assert_eq!(parsed["type"], "progress");
        assert_eq!(parsed["data"]["current"], 3);

        let sse_msg = ProgressHelper::sse_message(&progress);
        assert!(sse_msg.starts_with("event: progress"));
        assert!(sse_msg.contains("data:"));
        assert!(sse_msg.ends_with("\n\n"));
    }

    #[test]
    fn test_progress_for_type() {
        let progress = json!({ "percent": 50 });

        let ndjson = ProgressHelper::for_type(StreamingResponseType::Ndjson, &progress);
        assert!(ndjson.contains("progress"));

        let sse = ProgressHelper::for_type(StreamingResponseType::Sse, &progress);
        assert!(sse.starts_with("event: progress"));

        let json = ProgressHelper::for_type(StreamingResponseType::Json, &progress);
        assert!(json.is_empty());
    }

    #[tokio::test]
    async fn test_streaming_error_response_ndjson() {
        let error = json!({
            "error": "test_error",
            "message": "This is a test error"
        });

        let response = StreamingErrorResponse::ndjson(error);

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/x-ndjson"
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.ends_with('\n'));

        let parsed: Value = serde_json::from_str(body_str.trim()).unwrap();
        assert_eq!(parsed["error"], "test_error");
    }

    #[tokio::test]
    async fn test_streaming_error_response_sse() {
        let error = json!({
            "error": "sse_error",
            "message": "SSE error message"
        });

        let response = StreamingErrorResponse::sse(error);

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/event-stream"
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.starts_with("event: error"));
        assert!(body_str.contains("data:"));
        assert!(body_str.ends_with("\n\n"));
    }

    #[tokio::test]
    async fn test_streaming_error_response_json() {
        let error = json!({
            "error": "json_error",
            "details": "Error details"
        });

        let response = StreamingErrorResponse::json(error);

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        let parsed: Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(parsed["error"], "json_error");
    }

    #[tokio::test]
    async fn test_streaming_error_response_for_type() {
        let error = json!({"error": "generic_error"});

        let ndjson_resp = StreamingErrorResponse::for_type(StreamingResponseType::Ndjson, &error);
        assert_eq!(
            ndjson_resp.headers().get("content-type").unwrap(),
            "application/x-ndjson"
        );

        let sse_resp = StreamingErrorResponse::for_type(StreamingResponseType::Sse, &error);
        assert_eq!(
            sse_resp.headers().get("content-type").unwrap(),
            "text/event-stream"
        );

        let json_resp = StreamingErrorResponse::for_type(StreamingResponseType::Json, &error);
        assert_eq!(
            json_resp.headers().get("content-type").unwrap(),
            "application/json"
        );
    }

    #[test]
    fn test_streaming_response_builder() {
        let builder = StreamingResponseBuilder::new(StreamingResponseType::Ndjson);
        assert_eq!(builder.status, StatusCode::OK);
        assert!(!builder.compression);
    }

    #[test]
    fn test_streaming_response_builder_with_status() {
        let builder = StreamingResponseBuilder::new(StreamingResponseType::Ndjson)
            .status(StatusCode::ACCEPTED);
        assert_eq!(builder.status, StatusCode::ACCEPTED);
    }

    #[test]
    fn test_streaming_response_builder_with_compression() {
        let builder = StreamingResponseBuilder::new(StreamingResponseType::Sse).with_compression();
        assert!(builder.compression);
    }

    #[test]
    fn test_streaming_response_builder_with_header() {
        let builder = StreamingResponseBuilder::new(StreamingResponseType::Json)
            .header("x-custom-header", "custom-value");

        assert!(builder.headers.contains_key("x-custom-header"));
    }

    #[tokio::test]
    async fn test_stream_from_receiver() {
        let (tx, rx) = tokio::sync::mpsc::channel(10);

        // Spawn task to send test data
        tokio::spawn(async move {
            tx.send(json!({"id": 1, "name": "test1"})).await.unwrap();
            tx.send(json!({"id": 2, "name": "test2"})).await.unwrap();
        });

        let response = stream_from_receiver(rx, StreamingResponseType::Ndjson);

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/x-ndjson"
        );
    }

    #[test]
    fn test_message_formatting_consistency() {
        // Test that all message types have consistent structure
        let test_data = json!({"test": "value", "count": 42});

        // Test NDJSON formatting
        let ndjson_progress = ProgressHelper::ndjson_message(&test_data);
        assert!(ndjson_progress.contains("\"type\":\"progress\""));
        assert!(ndjson_progress.contains("\"timestamp\":"));
        assert!(ndjson_progress.ends_with('\n'));

        let ndjson_completion = CompletionHelper::ndjson_message(&test_data);
        assert!(ndjson_completion.contains("\"type\":\"completion\""));
        assert!(ndjson_completion.contains("\"timestamp\":"));
        assert!(ndjson_completion.ends_with('\n'));

        // Test SSE formatting
        let sse_progress = ProgressHelper::sse_message(&test_data);
        assert!(sse_progress.starts_with("event: progress\n"));
        assert!(sse_progress.contains("data:"));
        assert!(sse_progress.ends_with("\n\n"));

        let sse_completion = CompletionHelper::sse_message(&test_data);
        assert!(sse_completion.starts_with("event: completion\n"));
        assert!(sse_completion.contains("data:"));
        assert!(sse_completion.ends_with("\n\n"));
    }
}
