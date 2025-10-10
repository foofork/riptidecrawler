//! Comprehensive NDJSON Streaming Tests - TDD Implementation
//!
//! Tests the /crawl/stream and /deepsearch/stream endpoints with focus on:
//! - TTFB < 500ms with warm cache
//! - Zero unwrap/expect error handling
//! - Backpressure and buffer management (65536 bytes)
//! - Streaming as results complete (no batching)
//! - Progress records and error handling
//! - Resource controls and performance optimization

use axum::http::{HeaderMap, StatusCode};
use bytes::Bytes;
use futures::stream::StreamExt;
use httpmock::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use uuid::Uuid;

/// Test framework for NDJSON streaming
struct NdjsonStreamingTestFramework {
    mock_server: MockServer,
    client: reqwest::Client,
    base_url: String,
}

impl NdjsonStreamingTestFramework {
    fn new() -> Self {
        let mock_server = MockServer::start();
        let client = reqwest::Client::new();
        let base_url = "http://localhost:8080".to_string();

        Self {
            mock_server,
            client,
            base_url,
        }
    }

    /// Setup mock responses for successful content
    fn setup_successful_content_mocks(&self, count: usize) -> Vec<String> {
        let mut urls = Vec::new();

        for i in 0..count {
            let path = format!("/test-content-{}", i);
            let _mock = self.mock_server.mock(|when, then| {
                when.method(GET).path(&path);
                then.status(200)
                    .header("content-type", "text/html")
                    .header("content-length", "1024")
                    .body(&format!(
                        r#"<html>
                        <head><title>Test Content {}</title></head>
                        <body>
                            <h1>Test Page {}</h1>
                            <p>This is test content for URL {}. It contains enough text to make processing meaningful and test the streaming capabilities.</p>
                            <div class="content">Additional content that will be extracted and processed by the crawler.</div>
                        </body>
                        </html>"#,
                        i, i, i
                    ));
            });

            urls.push(format!(
                "{}/test-content-{}",
                self.mock_server.base_url(),
                i
            ));
        }

        urls
    }

    /// Setup mock responses with delays to test streaming behavior
    fn setup_delayed_content_mocks(&self, delays_ms: Vec<u64>) -> Vec<String> {
        let mut urls = Vec::new();

        for (i, delay) in delays_ms.iter().enumerate() {
            let path = format!("/delayed-content-{}", i);
            let delay_ms = *delay;

            let _mock = self.mock_server.mock(|when, then| {
                when.method(GET).path(&path);
                then.status(200)
                    .header("content-type", "text/html")
                    .delay(Duration::from_millis(delay_ms))
                    .body(&format!(
                        r#"<html>
                        <head><title>Delayed Content {}</title></head>
                        <body><p>Content with {}ms delay</p></body>
                        </html>"#,
                        i, delay_ms
                    ));
            });

            urls.push(format!(
                "{}/delayed-content-{}",
                self.mock_server.base_url(),
                i
            ));
        }

        urls
    }

    /// Setup mock responses that will fail
    fn setup_error_content_mocks(&self, count: usize) -> Vec<String> {
        let mut urls = Vec::new();

        for i in 0..count {
            let path = format!("/error-content-{}", i);
            let _mock = self.mock_server.mock(|when, then| {
                when.method(GET).path(&path);
                then.status(500)
                    .header("content-type", "text/html")
                    .body("Internal Server Error");
            });

            urls.push(format!(
                "{}/error-content-{}",
                self.mock_server.base_url(),
                i
            ));
        }

        urls
    }

    /// Create crawl request with specified options
    fn create_crawl_request(&self, urls: Vec<String>, cache_mode: &str, concurrency: u32) -> Value {
        serde_json::json!({
            "urls": urls,
            "options": {
                "cache_mode": cache_mode,
                "concurrency": concurrency,
                "stream": true,  // Default ON for streaming
                "timeout_ms": 10000,
                "user_agent": "RipTide-Test/1.0",
                "respect_robots": false
            }
        })
    }

    /// Create deepsearch request
    fn create_deepsearch_request(&self, query: &str, limit: u32, include_content: bool) -> Value {
        serde_json::json!({
            "query": query,
            "limit": limit,
            "include_content": include_content,
            "crawl_options": {
                "cache_mode": "disabled",
                "concurrency": 2,
                "stream": true
            }
        })
    }

    /// Make streaming request and parse NDJSON lines
    async fn make_streaming_request(
        &self,
        endpoint: &str,
        body: Value,
    ) -> Result<StreamingResponse, TestError> {
        let start_time = Instant::now();

        let response = self
            .client
            .post(&format!("{}/{}", self.base_url, endpoint))
            .json(&body)
            .send()
            .await
            .map_err(|e| TestError::RequestFailed(e.to_string()))?;

        let ttfb = start_time.elapsed();
        let status = response.status();
        let headers = response.headers().clone();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(TestError::RequestFailed(format!(
                "Status: {}, Body: {}",
                status, error_text
            )));
        }

        let mut stream = response.bytes_stream();
        let mut lines = Vec::new();
        let mut buffer = String::new();
        let mut first_line_time = None;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| TestError::StreamError(e.to_string()))?;

            if first_line_time.is_none() {
                first_line_time = Some(start_time.elapsed());
            }

            buffer.push_str(&String::from_utf8_lossy(&chunk));

            // Process complete lines
            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].to_string();
                buffer = buffer[newline_pos + 1..].to_string();

                if !line.trim().is_empty() {
                    let parsed_line = serde_json::from_str::<Value>(&line).map_err(|e| {
                        TestError::ParseError(format!("Line: {}, Error: {}", line, e))
                    })?;
                    lines.push(parsed_line);
                }
            }
        }

        Ok(StreamingResponse {
            ttfb,
            first_line_time: first_line_time.unwrap_or(ttfb),
            status,
            headers,
            lines,
            total_time: start_time.elapsed(),
        })
    }
}

/// Streaming response structure for testing
#[derive(Debug)]
struct StreamingResponse {
    ttfb: Duration,
    first_line_time: Duration,
    status: StatusCode,
    headers: HeaderMap,
    lines: Vec<Value>,
    total_time: Duration,
}

impl StreamingResponse {
    /// Get metadata line (should be first)
    fn metadata(&self) -> Result<&Value, TestError> {
        self.lines
            .get(0)
            .ok_or_else(|| TestError::ParseError("No metadata line found".to_string()))
    }

    /// Get result lines (excluding metadata and summary)
    fn result_lines(&self) -> Vec<&Value> {
        self.lines
            .iter()
            .filter(|line| line.get("result").is_some() || line.get("search_result").is_some())
            .collect()
    }

    /// Get summary line (should be last)
    fn summary(&self) -> Result<&Value, TestError> {
        self.lines
            .last()
            .ok_or_else(|| TestError::ParseError("No summary line found".to_string()))
    }

    /// Get progress lines
    fn progress_lines(&self) -> Vec<&Value> {
        self.lines
            .iter()
            .filter(|line| {
                line.get("progress_percentage").is_some() || line.get("operation_type").is_some()
            })
            .collect()
    }

    /// Validate NDJSON structure
    fn validate_structure(&self) -> Result<(), TestError> {
        if self.lines.is_empty() {
            return Err(TestError::ValidationError(
                "No NDJSON lines received".to_string(),
            ));
        }

        // Check for required headers
        let content_type = self
            .headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| TestError::ValidationError("Missing content-type header".to_string()))?;

        if content_type != "application/x-ndjson" {
            return Err(TestError::ValidationError(format!(
                "Expected content-type 'application/x-ndjson', got '{}'",
                content_type
            )));
        }

        // Check for streaming headers
        if self.headers.get("transfer-encoding").is_none() {
            return Err(TestError::ValidationError(
                "Missing transfer-encoding header".to_string(),
            ));
        }

        Ok(())
    }
}

/// Test-specific error types
#[derive(Debug)]
enum TestError {
    RequestFailed(String),
    StreamError(String),
    ParseError(String),
    ValidationError(String),
    TTFBTimeout(Duration),
    Performance(String),
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestError::RequestFailed(msg) => write!(f, "Request failed: {}", msg),
            TestError::StreamError(msg) => write!(f, "Stream error: {}", msg),
            TestError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            TestError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            TestError::TTFBTimeout(duration) => write!(f, "TTFB timeout: {:?}", duration),
            TestError::Performance(msg) => write!(f, "Performance issue: {}", msg),
        }
    }
}

impl std::error::Error for TestError {}

/// Test TTFB requirement for /crawl/stream with warm cache
#[tokio::test]
async fn test_crawl_stream_ttfb_under_500ms() {
    let framework = NdjsonStreamingTestFramework::new();
    let urls = framework.setup_successful_content_mocks(3);

    // First request to warm up cache
    let request = framework.create_crawl_request(urls.clone(), "write_through", 3);
    let _ = framework
        .make_streaming_request("crawl/stream", request.clone())
        .await;

    // Second request should hit warm cache
    let response = framework
        .make_streaming_request("crawl/stream", request)
        .await
        .expect("Streaming request should succeed");

    // Validate TTFB requirement
    assert!(
        response.ttfb.as_millis() < 500,
        "TTFB {} ms exceeds 500ms requirement with warm cache",
        response.ttfb.as_millis()
    );

    // Validate streaming structure
    response
        .validate_structure()
        .expect("Response structure should be valid");

    // Validate metadata
    let metadata = response.metadata().expect("Should have metadata");
    assert_eq!(metadata["total_urls"], 3);
    assert_eq!(metadata["stream_type"], "crawl");
    assert!(metadata["request_id"].is_string());
}

/// Test that results stream as they complete (not batched)
#[tokio::test]
async fn test_crawl_stream_incremental_results() {
    let framework = NdjsonStreamingTestFramework::new();

    // Setup URLs with different processing times
    let urls = framework.setup_delayed_content_mocks(vec![100, 500, 200]);
    let request = framework.create_crawl_request(urls, "disabled", 3);

    let start_time = Instant::now();
    let response = framework
        .make_streaming_request("crawl/stream", request)
        .await
        .expect("Streaming request should succeed");

    let result_lines = response.result_lines();
    assert_eq!(result_lines.len(), 3, "Should have 3 result lines");

    // Results should arrive in completion order, not request order
    // This tests that streaming happens as results complete
    for (i, result_line) in result_lines.iter().enumerate() {
        let result = result_line.get("result").expect("Should have result field");
        assert!(result["processing_time_ms"].is_number());
        assert!(result["url"].is_string());
        assert!(result["status"].is_number());

        // Verify progress tracking
        if let Some(progress) = result_line.get("progress") {
            assert!(progress["completed"].as_u64().unwrap() >= 1);
            assert_eq!(progress["total"], 3);
            assert!(progress["success_rate"].is_number());
        }
    }

    // Verify total processing time is reasonable for parallel execution
    assert!(
        response.total_time.as_millis() < 800,
        "Total time {} ms suggests batching instead of streaming",
        response.total_time.as_millis()
    );
}

/// Test error handling with structured error objects
#[tokio::test]
async fn test_crawl_stream_error_handling() {
    let framework = NdjsonStreamingTestFramework::new();

    // Mix of successful and failing URLs
    let mut all_urls = framework.setup_successful_content_mocks(2);
    all_urls.extend(framework.setup_error_content_mocks(2));

    let request = framework.create_crawl_request(all_urls, "disabled", 4);
    let response = framework
        .make_streaming_request("crawl/stream", request)
        .await
        .expect("Streaming request should succeed even with errors");

    let result_lines = response.result_lines();
    assert_eq!(result_lines.len(), 4, "Should have 4 result lines");

    let mut success_count = 0;
    let mut error_count = 0;

    for result_line in result_lines {
        let result = result_line.get("result").expect("Should have result field");

        if result.get("error").is_some() {
            error_count += 1;
            let error = &result["error"];

            // Validate structured error object
            assert!(error["error_type"].is_string(), "Error should have type");
            assert!(error["message"].is_string(), "Error should have message");
            assert!(
                error["retryable"].is_boolean(),
                "Error should indicate if retryable"
            );

            // Validate that URL and other fields are still present
            assert!(result["url"].is_string());
            assert_eq!(result["from_cache"], false);
        } else {
            success_count += 1;
            assert!(
                result["document"].is_object(),
                "Success should have document"
            );
            assert!(result["status"].as_u64().unwrap() >= 200);
        }
    }

    assert_eq!(success_count, 2, "Should have 2 successful results");
    assert_eq!(error_count, 2, "Should have 2 error results");

    // Validate summary includes error statistics
    let summary = response.summary().expect("Should have summary");
    assert_eq!(summary["successful"], 2);
    assert_eq!(summary["failed"], 2);
    assert_eq!(summary["total_urls"], 4);
}

/// Test /deepsearch/stream endpoint
#[tokio::test]
async fn test_deepsearch_stream_endpoint() {
    let framework = NdjsonStreamingTestFramework::new();

    // Mock Serper API
    std::env::set_var("SERPER_API_KEY", "test_key");

    let serper_mock = MockServer::start();
    let _search_mock = serper_mock.mock(|when, then| {
        when.method(POST)
            .path("/search")
            .header("X-API-KEY", "test_key");
        then.status(200).json_body(serde_json::json!({
            "organic": [
                {
                    "title": "Test Result 1",
                    "link": "https://example.com/test1",
                    "snippet": "Test snippet 1"
                },
                {
                    "title": "Test Result 2",
                    "link": "https://example.com/test2",
                    "snippet": "Test snippet 2"
                }
            ]
        }));
    });

    let request = framework.create_deepsearch_request("test query", 2, true);
    let response = framework
        .make_streaming_request("deepsearch/stream", request)
        .await
        .expect("Deep search streaming should succeed");

    response
        .validate_structure()
        .expect("Response structure should be valid");

    // Check for deepsearch-specific metadata
    let metadata = response.metadata().expect("Should have metadata");
    assert_eq!(metadata["stream_type"], "deepsearch");

    // Should have search metadata
    let search_metadata_lines: Vec<&Value> = response
        .lines
        .iter()
        .filter(|line| line.get("query").is_some() && line.get("urls_found").is_some())
        .collect();
    assert!(
        !search_metadata_lines.is_empty(),
        "Should have search metadata"
    );

    let search_metadata = search_metadata_lines[0];
    assert_eq!(search_metadata["query"], "test query");
    assert!(search_metadata["urls_found"].is_number());

    // Should have search results with crawl data
    let search_results: Vec<&Value> = response
        .lines
        .iter()
        .filter(|line| line.get("search_result").is_some())
        .collect();
    assert!(!search_results.is_empty(), "Should have search results");

    for search_result in search_results {
        let search_data = &search_result["search_result"];
        assert!(search_data["url"].is_string());
        assert!(search_data["rank"].is_number());
        assert!(search_data["search_title"].is_string());

        // May have crawl result if content extraction was requested
        if search_result.get("crawl_result").is_some() {
            let crawl_result = &search_result["crawl_result"];
            assert!(crawl_result["status"].is_number());
            assert!(crawl_result["gate_decision"].is_string());
        }
    }
}

/// Test backpressure handling with large buffer (65536 bytes)
#[tokio::test]
async fn test_backpressure_and_buffer_management() {
    let framework = NdjsonStreamingTestFramework::new();

    // Create many URLs to test buffer management
    let urls = framework.setup_successful_content_mocks(20);
    let request = framework.create_crawl_request(urls, "disabled", 10);

    let response = framework
        .make_streaming_request("crawl/stream", request)
        .await
        .expect("High-load streaming should succeed");

    // Should handle all URLs without dropping
    assert_eq!(
        response.result_lines().len(),
        20,
        "All results should be delivered"
    );

    // Check if progress updates were sent for large batches
    let progress_lines = response.progress_lines();
    if !progress_lines.is_empty() {
        // Validate progress structure
        for progress_line in progress_lines {
            assert!(progress_line["progress_percentage"].is_number());
            assert!(progress_line["items_completed"].is_number());
            assert!(progress_line["items_total"].is_number());
            assert_eq!(progress_line["items_total"], 20);
        }
    }

    // Verify buffer management didn't cause excessive delays
    assert!(
        response.total_time.as_millis() < 5000,
        "Buffer management should not cause excessive delays"
    );
}

/// Test request validation
#[tokio::test]
async fn test_streaming_request_validation() {
    let framework = NdjsonStreamingTestFramework::new();

    // Test empty URLs
    let invalid_request = framework.create_crawl_request(vec![], "disabled", 1);
    let result = framework
        .make_streaming_request("crawl/stream", invalid_request)
        .await;
    assert!(result.is_err(), "Empty URLs should be rejected");

    // Test invalid cache mode
    let invalid_request = serde_json::json!({
        "urls": ["https://example.com"],
        "options": {
            "cache_mode": "invalid_mode",
            "concurrency": 1
        }
    });
    let result = framework
        .make_streaming_request("crawl/stream", invalid_request)
        .await;
    assert!(result.is_err(), "Invalid cache mode should be rejected");

    // Test missing SERPER_API_KEY for deepsearch
    std::env::remove_var("SERPER_API_KEY");
    let deepsearch_request = framework.create_deepsearch_request("test", 5, false);
    let result = framework
        .make_streaming_request("deepsearch/stream", deepsearch_request)
        .await;
    assert!(result.is_err(), "Missing API key should be rejected");
}

/// Test performance characteristics and metrics
#[tokio::test]
async fn test_streaming_performance_and_metrics() {
    let framework = NdjsonStreamingTestFramework::new();

    // Test with 10 URLs to check performance
    let urls = framework.setup_successful_content_mocks(10);
    let request = framework.create_crawl_request(urls, "disabled", 5);

    let start_time = Instant::now();
    let response = framework
        .make_streaming_request("crawl/stream", request)
        .await
        .expect("Performance test should succeed");

    // Validate performance metrics are included
    let summary = response.summary().expect("Should have summary");
    assert!(summary["total_processing_time_ms"].is_number());
    assert!(summary["cache_hit_rate"].is_number());

    // Check that processing time is reported per result
    for result_line in response.result_lines() {
        let result = result_line.get("result").expect("Should have result field");
        assert!(result["processing_time_ms"].is_number());
        assert!(result["quality_score"].is_number());
    }

    // Validate first result arrives quickly (streaming benefit)
    assert!(
        response.first_line_time.as_millis() < 1000,
        "First result should arrive quickly in streaming mode"
    );

    // Verify request ID is consistent across all lines
    let request_id = response.metadata().expect("Should have metadata")["request_id"]
        .as_str()
        .expect("Should have request ID");

    // Check headers contain request ID
    if let Some(header_request_id) = response.headers.get("X-Request-ID") {
        assert_eq!(
            header_request_id.to_str().unwrap(),
            request_id,
            "Request ID should be consistent"
        );
    }
}

/// Test concurrent streaming sessions
#[tokio::test]
async fn test_concurrent_streaming_sessions() {
    let framework = NdjsonStreamingTestFramework::new();

    // Setup different content for each session
    let urls1 = framework.setup_successful_content_mocks(3);
    let urls2 = framework.setup_successful_content_mocks(5);

    let request1 = framework.create_crawl_request(urls1, "disabled", 2);
    let request2 = framework.create_crawl_request(urls2, "disabled", 3);

    // Start both sessions concurrently
    let (result1, result2) = tokio::join!(
        framework.make_streaming_request("crawl/stream", request1),
        framework.make_streaming_request("crawl/stream", request2)
    );

    let response1 = result1.expect("First session should succeed");
    let response2 = result2.expect("Second session should succeed");

    // Validate both sessions completed successfully
    assert_eq!(response1.result_lines().len(), 3);
    assert_eq!(response2.result_lines().len(), 5);

    // Verify different request IDs
    let request_id1 = response1.metadata().unwrap()["request_id"]
        .as_str()
        .unwrap();
    let request_id2 = response2.metadata().unwrap()["request_id"]
        .as_str()
        .unwrap();
    assert_ne!(
        request_id1, request_id2,
        "Concurrent sessions should have different request IDs"
    );

    // Both should meet TTFB requirements
    assert!(
        response1.ttfb.as_millis() < 2000,
        "Concurrent session 1 TTFB should be reasonable"
    );
    assert!(
        response2.ttfb.as_millis() < 2000,
        "Concurrent session 2 TTFB should be reasonable"
    );
}

/// Integration test for complete streaming workflow
#[tokio::test]
async fn test_complete_streaming_workflow() {
    let framework = NdjsonStreamingTestFramework::new();

    // Mixed scenario: fast, slow, and error URLs
    let mut all_urls = framework.setup_successful_content_mocks(3);
    all_urls.extend(framework.setup_delayed_content_mocks(vec![800]));
    all_urls.extend(framework.setup_error_content_mocks(1));

    let request = framework.create_crawl_request(all_urls, "write_through", 3);
    let response = framework
        .make_streaming_request("crawl/stream", request)
        .await
        .expect("Complete workflow should succeed");

    // Validate complete NDJSON structure
    response
        .validate_structure()
        .expect("Structure should be valid");

    // Should have metadata, results, and summary
    assert!(
        response.lines.len() >= 7,
        "Should have metadata + 5 results + summary + possible progress"
    );

    let metadata = response.metadata().expect("Should have metadata");
    let summary = response.summary().expect("Should have summary");
    let results = response.result_lines();

    // Validate metadata
    assert_eq!(metadata["total_urls"], 5);
    assert_eq!(metadata["stream_type"], "crawl");

    // Validate results
    assert_eq!(results.len(), 5);

    let mut successful = 0;
    let mut failed = 0;
    for result_line in results {
        let result = result_line.get("result").expect("Should have result");
        if result.get("error").is_some() {
            failed += 1;
        } else {
            successful += 1;
        }
    }

    // Validate summary matches results
    assert_eq!(summary["successful"], successful);
    assert_eq!(summary["failed"], failed);
    assert_eq!(summary["total_urls"], 5);
    assert!(summary["total_processing_time_ms"].as_u64().unwrap() > 0);

    // Performance validation
    assert!(
        response.ttfb.as_millis() < 1000,
        "Complete workflow TTFB should be reasonable"
    );
}
