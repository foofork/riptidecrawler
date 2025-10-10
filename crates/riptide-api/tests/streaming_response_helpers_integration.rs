//! Integration tests for streaming response helpers.
//!
//! Tests that verify the response helpers are properly wired into
//! the streaming endpoints and produce correct formats.

use axum::body::to_bytes;
use riptide_api::streaming::response_helpers::{
    CompletionHelper, KeepAliveHelper, ProgressHelper, StreamingErrorResponse,
    StreamingResponseBuilder, StreamingResponseType,
};
use serde_json::{json, Value};

#[tokio::test]
async fn test_ndjson_error_response_format() {
    let error = json!({
        "error": "validation_error",
        "message": "Invalid request parameters",
        "retryable": false
    });

    let response = StreamingErrorResponse::ndjson(error);

    // Verify status code
    assert_eq!(response.status(), 500);

    // Verify headers
    let headers = response.headers();
    assert_eq!(headers.get("content-type").unwrap(), "application/x-ndjson");
    assert_eq!(headers.get("cache-control").unwrap(), "no-cache");
    assert_eq!(headers.get("connection").unwrap(), "keep-alive");

    // Verify body format
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // NDJSON must end with newline
    assert!(body_str.ends_with('\n'), "NDJSON must end with newline");

    // Must be valid JSON
    let parsed: Value = serde_json::from_str(body_str.trim()).unwrap();
    assert_eq!(parsed["error"], "validation_error");
    assert_eq!(parsed["message"], "Invalid request parameters");
    assert_eq!(parsed["retryable"], false);
}

#[tokio::test]
async fn test_sse_error_response_format() {
    let error = json!({
        "error": "processing_error",
        "message": "Failed to process request",
        "retryable": true
    });

    let response = StreamingErrorResponse::sse(error);

    // Verify status code
    assert_eq!(response.status(), 500);

    // Verify headers
    let headers = response.headers();
    assert_eq!(headers.get("content-type").unwrap(), "text/event-stream");
    assert_eq!(headers.get("cache-control").unwrap(), "no-cache");
    assert_eq!(headers.get("connection").unwrap(), "keep-alive");

    // Verify body format
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // SSE event format verification
    assert!(
        body_str.starts_with("event: error"),
        "SSE must start with event type"
    );
    assert!(body_str.contains("data:"), "SSE must contain data field");
    assert!(
        body_str.ends_with("\n\n"),
        "SSE event must end with double newline"
    );

    // Verify data payload
    let data_line = body_str
        .lines()
        .find(|line| line.starts_with("data:"))
        .unwrap();
    let data_json = data_line.trim_start_matches("data:").trim();
    let parsed: Value = serde_json::from_str(data_json).unwrap();
    assert_eq!(parsed["error"], "processing_error");
}

#[tokio::test]
async fn test_json_error_response_format() {
    let error = json!({
        "error": "internal_error",
        "message": "Server error",
        "retryable": false
    });

    let response = StreamingErrorResponse::json(error);

    // Verify status code
    assert_eq!(response.status(), 500);

    // Verify headers
    let headers = response.headers();
    assert_eq!(headers.get("content-type").unwrap(), "application/json");
    assert_eq!(headers.get("connection").unwrap(), "close");

    // Verify body format
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Must be valid JSON (no trailing newline required)
    let parsed: Value = serde_json::from_str(&body_str).unwrap();
    assert_eq!(parsed["error"], "internal_error");
}

#[test]
fn test_ndjson_keep_alive_format() {
    let keepalive = KeepAliveHelper::ndjson_message();

    // Must end with newline
    assert!(keepalive.ends_with('\n'));

    // Must be valid JSON
    let parsed: Value = serde_json::from_str(keepalive.trim()).unwrap();
    assert_eq!(parsed["type"], "keep-alive");
    assert!(parsed["timestamp"].is_string());

    // Timestamp should be valid RFC3339
    let timestamp = parsed["timestamp"].as_str().unwrap();
    assert!(chrono::DateTime::parse_from_rfc3339(timestamp).is_ok());
}

#[test]
fn test_sse_keep_alive_format() {
    let keepalive = KeepAliveHelper::sse_message();

    // SSE comment format
    assert!(keepalive.starts_with(':'));
    assert!(keepalive.ends_with("\n\n"));

    // Should contain timestamp
    assert!(keepalive.contains("keep-alive"));

    // Verify timestamp format by extracting and parsing
    let parts: Vec<&str> = keepalive.split_whitespace().collect();
    assert!(parts.len() >= 2);
    let timestamp = parts[2];
    assert!(chrono::DateTime::parse_from_rfc3339(timestamp).is_ok());
}

#[test]
fn test_progress_message_format() {
    let progress_data = json!({
        "current": 5,
        "total": 10,
        "percent": 50.0
    });

    let ndjson_progress = ProgressHelper::ndjson_message(&progress_data);

    // NDJSON format
    assert!(ndjson_progress.ends_with('\n'));
    let parsed: Value = serde_json::from_str(ndjson_progress.trim()).unwrap();
    assert_eq!(parsed["type"], "progress");
    assert_eq!(parsed["data"]["current"], 5);
    assert_eq!(parsed["data"]["total"], 10);
    assert!(parsed["timestamp"].is_string());

    let sse_progress = ProgressHelper::sse_message(&progress_data);

    // SSE format
    assert!(sse_progress.starts_with("event: progress"));
    assert!(sse_progress.contains("data:"));
    assert!(sse_progress.ends_with("\n\n"));
}

#[test]
fn test_completion_message_format() {
    let summary = json!({
        "total": 10,
        "successful": 9,
        "failed": 1,
        "cache_hit_rate": 0.8
    });

    let ndjson_completion = CompletionHelper::ndjson_message(&summary);

    // NDJSON format
    assert!(ndjson_completion.ends_with('\n'));
    let parsed: Value = serde_json::from_str(ndjson_completion.trim()).unwrap();
    assert_eq!(parsed["type"], "completion");
    assert_eq!(parsed["summary"]["total"], 10);
    assert_eq!(parsed["summary"]["successful"], 9);
    assert_eq!(parsed["summary"]["failed"], 1);

    let sse_completion = CompletionHelper::sse_message(&summary);

    // SSE format
    assert!(sse_completion.starts_with("event: completion"));
    assert!(sse_completion.ends_with("\n\n"));
}

#[tokio::test]
async fn test_streaming_builder_ndjson() {
    let (tx, rx) = tokio::sync::mpsc::channel(10);

    tokio::spawn(async move {
        for i in 0..3 {
            let _ = tx
                .send(json!({"index": i, "value": format!("item_{}", i)}))
                .await;
        }
    });

    let response = riptide_api::streaming::response_helpers::stream_from_receiver(
        rx,
        StreamingResponseType::Ndjson,
    );

    // Verify response properties
    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/x-ndjson"
    );
}

#[test]
fn test_response_type_properties() {
    // NDJSON
    let ndjson = StreamingResponseType::Ndjson;
    assert_eq!(ndjson.content_type(), "application/x-ndjson");
    assert_eq!(ndjson.buffer_size(), 256);
    assert!(ndjson.supports_keep_alive());

    // SSE
    let sse = StreamingResponseType::Sse;
    assert_eq!(sse.content_type(), "text/event-stream");
    assert_eq!(sse.buffer_size(), 128);
    assert!(sse.supports_keep_alive());

    // JSON
    let json = StreamingResponseType::Json;
    assert_eq!(json.content_type(), "application/json");
    assert_eq!(json.buffer_size(), 64);
    assert!(!json.supports_keep_alive());
}

#[tokio::test]
async fn test_error_response_consistency_across_types() {
    let error = json!({
        "error": "test_error",
        "message": "Test error message",
        "code": "TEST_001"
    });

    // All three response types should preserve error structure
    let ndjson_resp = StreamingErrorResponse::ndjson(&error);
    let sse_resp = StreamingErrorResponse::sse(&error);
    let json_resp = StreamingErrorResponse::json(&error);

    // Extract and verify NDJSON
    let ndjson_body = to_bytes(ndjson_resp.into_body(), usize::MAX).await.unwrap();
    let ndjson_str = String::from_utf8(ndjson_body.to_vec()).unwrap();
    let ndjson_parsed: Value = serde_json::from_str(ndjson_str.trim()).unwrap();
    assert_eq!(ndjson_parsed["error"], "test_error");
    assert_eq!(ndjson_parsed["code"], "TEST_001");

    // Extract and verify SSE
    let sse_body = to_bytes(sse_resp.into_body(), usize::MAX).await.unwrap();
    let sse_str = String::from_utf8(sse_body.to_vec()).unwrap();
    let data_line = sse_str
        .lines()
        .find(|line| line.starts_with("data:"))
        .unwrap();
    let sse_data = data_line.trim_start_matches("data:").trim();
    let sse_parsed: Value = serde_json::from_str(sse_data).unwrap();
    assert_eq!(sse_parsed["error"], "test_error");
    assert_eq!(sse_parsed["code"], "TEST_001");

    // Extract and verify JSON
    let json_body = to_bytes(json_resp.into_body(), usize::MAX).await.unwrap();
    let json_str = String::from_utf8(json_body.to_vec()).unwrap();
    let json_parsed: Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(json_parsed["error"], "test_error");
    assert_eq!(json_parsed["code"], "TEST_001");
}

#[test]
fn test_helper_for_type_dispatching() {
    let test_data = json!({"test": "data"});

    // KeepAlive
    let ndjson_ka = KeepAliveHelper::for_type(StreamingResponseType::Ndjson);
    assert!(ndjson_ka.contains("keep-alive"));
    assert!(ndjson_ka.ends_with('\n'));

    let sse_ka = KeepAliveHelper::for_type(StreamingResponseType::Sse);
    assert!(sse_ka.starts_with(':'));
    assert!(sse_ka.ends_with("\n\n"));

    let json_ka = KeepAliveHelper::for_type(StreamingResponseType::Json);
    assert!(json_ka.is_empty());

    // Progress
    let ndjson_prog = ProgressHelper::for_type(StreamingResponseType::Ndjson, &test_data);
    assert!(ndjson_prog.contains("progress"));

    let sse_prog = ProgressHelper::for_type(StreamingResponseType::Sse, &test_data);
    assert!(sse_prog.starts_with("event: progress"));

    // Completion
    let ndjson_comp = CompletionHelper::for_type(StreamingResponseType::Ndjson, &test_data);
    assert!(ndjson_comp.contains("completion"));

    let sse_comp = CompletionHelper::for_type(StreamingResponseType::Sse, &test_data);
    assert!(sse_comp.starts_with("event: completion"));
}
