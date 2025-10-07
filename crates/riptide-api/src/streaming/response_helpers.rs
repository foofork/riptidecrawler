//! Common streaming response helpers and utilities.
//!
//! This module provides reusable response handling patterns for different
//! streaming protocols (NDJSON, SSE, WebSocket) to reduce code duplication.

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

    /// Get recommended buffer size
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
#[allow(dead_code)] // Reserved for streaming API toolkit
pub struct CompletionHelper;

impl CompletionHelper {
    /// Create NDJSON completion message
    #[allow(dead_code)] // Reserved for streaming API toolkit
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
    #[allow(dead_code)] // Reserved for streaming API toolkit
    pub fn sse_message(summary: impl Serialize) -> String {
        match serde_json::to_string(&summary) {
            Ok(json) => format!("event: completion\ndata: {}\n\n", json),
            Err(e) => format!("event: error\ndata: {{\"error\":\"completion_serialization_failed\",\"message\":\"{}\"}}\n\n", e),
        }
    }

    /// Create completion message for specific type
    #[allow(dead_code)] // Reserved for streaming API toolkit
    pub fn for_type(response_type: StreamingResponseType, summary: impl Serialize) -> String {
        match response_type {
            StreamingResponseType::Ndjson => Self::ndjson_message(summary),
            StreamingResponseType::Sse => Self::sse_message(summary),
            StreamingResponseType::Json => "".to_string(), // JSON sends everything at once
        }
    }
}

/// Helper for creating progress messages
#[allow(dead_code)] // Reserved for streaming API toolkit
pub struct ProgressHelper;

impl ProgressHelper {
    /// Create NDJSON progress message
    #[allow(dead_code)] // Reserved for streaming API toolkit
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
    #[allow(dead_code)] // Reserved for streaming API toolkit
    pub fn sse_message(progress: impl Serialize) -> String {
        match serde_json::to_string(&progress) {
            Ok(json) => format!("event: progress\ndata: {}\n\n", json),
            Err(e) => format!("event: error\ndata: {{\"error\":\"progress_serialization_failed\",\"message\":\"{}\"}}\n\n", e),
        }
    }

    /// Create progress message for specific type
    #[allow(dead_code)] // Reserved for streaming API toolkit
    pub fn for_type(response_type: StreamingResponseType, progress: impl Serialize) -> String {
        match response_type {
            StreamingResponseType::Ndjson => Self::ndjson_message(progress),
            StreamingResponseType::Sse => Self::sse_message(progress),
            StreamingResponseType::Json => "".to_string(), // JSON doesn't send individual progress
        }
    }
}

/// Utility for creating streaming responses from channels
#[allow(dead_code)] // Reserved for streaming API toolkit
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
    #[allow(unused_imports)]
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
    fn test_keep_alive_helpers() {
        let ndjson_msg = KeepAliveHelper::ndjson_message();
        assert!(ndjson_msg.contains("keep-alive"));
        assert!(ndjson_msg.ends_with('\n'));

        let sse_msg = KeepAliveHelper::sse_message();
        assert!(sse_msg.starts_with(": keep-alive"));
        assert!(sse_msg.ends_with("\n\n"));
    }

    #[test]
    fn test_completion_helpers() {
        let summary = json!({ "total": 5, "processed": 5 });

        let ndjson_msg = CompletionHelper::ndjson_message(&summary);
        assert!(ndjson_msg.contains("completion"));
        assert!(ndjson_msg.ends_with('\n'));

        let sse_msg = CompletionHelper::sse_message(&summary);
        assert!(sse_msg.starts_with("event: completion"));
        assert!(sse_msg.contains("data:"));
        assert!(sse_msg.ends_with("\n\n"));
    }

    #[test]
    fn test_progress_helpers() {
        let progress = json!({ "current": 3, "total": 10 });

        let ndjson_msg = ProgressHelper::ndjson_message(&progress);
        assert!(ndjson_msg.contains("progress"));
        assert!(ndjson_msg.ends_with('\n'));

        let sse_msg = ProgressHelper::sse_message(&progress);
        assert!(sse_msg.starts_with("event: progress"));
        assert!(sse_msg.contains("data:"));
        assert!(sse_msg.ends_with("\n\n"));
    }
}
