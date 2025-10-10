//! Comprehensive Integration Tests for Streaming Pipeline Validation
//!
//! This module provides thorough testing of the streaming pipeline including:
//! - TTFB < 500ms performance requirements
//! - NDJSON output format compliance
//! - Buffer management and backpressure handling
//! - Stream lifecycle management
//! - Error scenarios and recovery
//! - Performance benchmarks

use std::time::{Duration, Instant};
use tokio::time::timeout;
use serde_json::Value;
use bytes::Bytes;
use futures::stream::StreamExt;
use axum::http::{HeaderMap, StatusCode};
use httpmock::prelude::*;
use reqwest::Client;
use uuid::Uuid;

/// Comprehensive streaming test framework
pub struct StreamingValidationFramework {
    mock_server: MockServer,
    client: Client,
    api_base_url: String,
}

impl StreamingValidationFramework {
    pub fn new() -> Self {
        let mock_server = MockServer::start();
        let client = Client::new();
        let api_base_url = "http://localhost:8080".to_string();

        Self {
            mock_server,
            client,
            api_base_url,
        }
    }

    /// Setup mock responses with controllable timing for TTFB tests
    pub fn setup_timed_content_mocks(&self, count: usize, delay_ms: u64) -> Vec<String> {
        let mut urls = Vec::new();

        for i in 0..count {
            let path = format!("/timed-content-{}", i);
            let _mock = self.mock_server.mock(|when, then| {
                when.method(GET).path(&path);
                then.status(200)
                    .header("content-type", "text/html")
                    .header("content-length", "2048")
                    .delay(Duration::from_millis(delay_ms))
                    .body(&format!(
                        r#"<html>
                        <head><title>Timed Content {}</title></head>
                        <body>
                            <h1>Performance Test Content {}</h1>
                            <p>This content is designed to test streaming performance with {delay_ms}ms delay.</p>
                            <div class="main-content">
                                <p>Structured content for extraction testing. This should be processed efficiently.</p>
                                <ul>
                                    <li>Point 1: Performance validation</li>
                                    <li>Point 2: TTFB measurement</li>
                                    <li>Point 3: Streaming compliance</li>
                                </ul>
                            </div>
                            <footer>Generated at {timestamp}</footer>
                        </body>
                        </html>"#,
                        i, i, delay_ms=delay_ms, timestamp=chrono::Utc::now().to_rfc3339()
                    ));
            });

            urls.push(format!("{}/timed-content-{}", self.mock_server.base_url(), i));
        }

        urls
    }

    /// Setup mock responses that will randomly succeed or fail
    pub fn setup_mixed_reliability_mocks(&self, total_count: usize, success_ratio: f64) -> Vec<String> {
        let mut urls = Vec::new();
        let success_count = (total_count as f64 * success_ratio) as usize;

        for i in 0..total_count {
            let path = format!("/mixed-content-{}", i);
            let is_success = i < success_count;

            let _mock = self.mock_server.mock(|when, then| {
                when.method(GET).path(&path);
                if is_success {
                    then.status(200)
                        .header("content-type", "text/html")
                        .body(&format!(
                            r#"<html>
                            <head><title>Success Content {}</title></head>
                            <body>
                                <h1>Successful Response {}</h1>
                                <p>This request succeeded as part of reliability testing.</p>
                            </body>
                            </html>"#,
                            i, i
                        ));
                } else {
                    then.status(500)
                        .header("content-type", "text/plain")
                        .body("Internal Server Error - Simulated Failure");
                }
            });

            urls.push(format!("{}/mixed-content-{}", self.mock_server.base_url(), i));
        }

        urls
    }

    /// Make streaming request with detailed timing and validation
    pub async fn make_validated_streaming_request(
        &self,
        endpoint: &str,
        body: Value,
        expect_ttfb_under_ms: Option<u64>,
    ) -> Result<StreamingValidationResponse, StreamingError> {
        let start_time = Instant::now();

        let response = self.client
            .post(&format!("{}/{}", self.api_base_url, endpoint))
            .header("Accept", "application/x-ndjson")
            .header("User-Agent", "StreamingValidationTest/1.0")
            .json(&body)
            .send()
            .await
            .map_err(|e| StreamingError::Request(e.to_string()))?;

        let ttfb = start_time.elapsed();
        let status = response.status();
        let headers = response.headers().clone();

        // Validate TTFB if specified
        if let Some(max_ttfb_ms) = expect_ttfb_under_ms {
            if ttfb.as_millis() > max_ttfb_ms as u128 {
                return Err(StreamingError::Performance(format!(
                    "TTFB {}ms exceeds requirement of {}ms",
                    ttfb.as_millis(),
                    max_ttfb_ms
                )));
            }
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(StreamingError::Http(status, error_text));
        }

        // Stream and parse NDJSON
        let mut stream = response.bytes_stream();
        let mut lines = Vec::new();
        let mut buffer = String::new();
        let mut first_line_time = None;
        let mut line_timestamps = Vec::new();
        let mut total_bytes = 0;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result
                .map_err(|e| StreamingError::Stream(e.to_string()))?;

            total_bytes += chunk.len();

            if first_line_time.is_none() {
                first_line_time = Some(start_time.elapsed());
            }

            buffer.push_str(&String::from_utf8_lossy(&chunk));

            // Process complete lines
            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].trim();
                buffer = buffer[newline_pos + 1..].to_string();

                if !line.is_empty() {
                    let line_time = start_time.elapsed();
                    let parsed_line = serde_json::from_str::<Value>(line)
                        .map_err(|e| StreamingError::Parse(format!("Line: '{}', Error: {}", line, e)))?;
                    
                    lines.push(parsed_line);
                    line_timestamps.push(line_time);
                }
            }
        }

        Ok(StreamingValidationResponse {
            ttfb,
            first_line_time: first_line_time.unwrap_or(ttfb),
            status,
            headers,
            lines,
            line_timestamps,
            total_time: start_time.elapsed(),
            total_bytes,
        })
    }

    /// Create crawl request with performance testing options
    pub fn create_performance_crawl_request(&self, urls: Vec<String>, cache_mode: &str, concurrency: u32) -> Value {
        serde_json::json!({
            "urls": urls,
            "options": {
                "cache_mode": cache_mode,
                "concurrency": concurrency,
                "stream": true,
                "timeout_ms": 30000,
                "user_agent": "StreamingValidator/1.0",
                "respect_robots": false,
                "enable_performance_tracking": true
            }
        })
    }
}

/// Detailed streaming response with validation metrics
#[derive(Debug)]
pub struct StreamingValidationResponse {
    pub ttfb: Duration,
    pub first_line_time: Duration,
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub lines: Vec<Value>,
    pub line_timestamps: Vec<Duration>,
    pub total_time: Duration,
    pub total_bytes: usize,
}

impl StreamingValidationResponse {
    /// Validate NDJSON format compliance
    pub fn validate_ndjson_format(&self) -> Result<(), StreamingError> {
        // Check headers
        let content_type = self.headers.get("content-type")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| StreamingError::Format("Missing content-type header".to_string()))?;

        if content_type != "application/x-ndjson" {
            return Err(StreamingError::Format(
                format!("Expected content-type 'application/x-ndjson', got '{}'", content_type)
            ));
        }

        // Check for required streaming headers
        if self.headers.get("transfer-encoding").is_none() {
            return Err(StreamingError::Format("Missing transfer-encoding header".to_string()));
        }

        // Validate structure
        if self.lines.is_empty() {
            return Err(StreamingError::Format("No NDJSON lines received".to_string()));
        }

        // Check for metadata (first line)
        let metadata = self.lines.get(0)
            .ok_or_else(|| StreamingError::Format("No metadata line".to_string()))?;

        if !metadata.get("total_urls").is_some() || !metadata.get("request_id").is_some() {
            return Err(StreamingError::Format("Invalid metadata structure".to_string()));
        }

        // Check for summary (last line)
        let summary = self.lines.last()
            .ok_or_else(|| StreamingError::Format("No summary line".to_string()))?;

        if !summary.get("total_urls").is_some() {
            return Err(StreamingError::Format("Invalid summary structure".to_string()));
        }

        Ok(())
    }

    /// Validate streaming behavior (no batching)
    pub fn validate_streaming_behavior(&self) -> Result<(), StreamingError> {
        if self.line_timestamps.len() < 2 {
            return Ok(()); // Too few lines to validate streaming
        }

        // Check that lines arrive with reasonable intervals (not all at once)
        let mut gaps = Vec::new();
        for i in 1..self.line_timestamps.len() {
            let gap = self.line_timestamps[i].saturating_sub(self.line_timestamps[i - 1]);
            gaps.push(gap.as_millis());
        }

        // At least some gaps should be non-zero (streaming, not batched)
        let non_zero_gaps = gaps.iter().filter(|&&g| g > 0).count();
        if non_zero_gaps == 0 && self.lines.len() > 3 {
            return Err(StreamingError::Streaming(
                "All lines arrived simultaneously - indicates batching instead of streaming".to_string()
            ));
        }

        Ok(())
    }

    /// Validate buffer management (no excessive memory usage)
    pub fn validate_buffer_usage(&self, max_buffer_bytes: usize) -> Result<(), StreamingError> {
        if self.total_bytes > max_buffer_bytes {
            return Err(StreamingError::Buffer(format!(
                "Total bytes {} exceeded buffer limit of {}",
                self.total_bytes, max_buffer_bytes
            )));
        }
        Ok(())
    }

    /// Get result lines (excluding metadata/summary)
    pub fn result_lines(&self) -> Vec<&Value> {
        self.lines.iter()
            .filter(|line| line.get("result").is_some() || line.get("search_result").is_some())
            .collect()
    }

    /// Get error count from results
    pub fn error_count(&self) -> usize {
        self.result_lines().iter()
            .filter(|line| {
                if let Some(result) = line.get("result") {
                    result.get("error").is_some()
                } else {
                    false
                }
            })
            .count()
    }

    /// Get success count from results
    pub fn success_count(&self) -> usize {
        self.result_lines().len() - self.error_count()
    }

    /// Calculate throughput (items per second)
    pub fn throughput(&self) -> f64 {
        if self.total_time.as_secs_f64() == 0.0 {
            return 0.0;
        }
        self.result_lines().len() as f64 / self.total_time.as_secs_f64()
    }
}

/// Streaming-specific error types
#[derive(Debug)]
pub enum StreamingError {
    Request(String),
    Http(StatusCode, String),
    Stream(String),
    Parse(String),
    Format(String),
    Performance(String),
    Streaming(String),
    Buffer(String),
    Timeout,
}

impl std::fmt::Display for StreamingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamingError::Request(msg) => write!(f, "Request error: {}", msg),
            StreamingError::Http(status, body) => write!(f, "HTTP {} error: {}", status, body),
            StreamingError::Stream(msg) => write!(f, "Stream error: {}", msg),
            StreamingError::Parse(msg) => write!(f, "Parse error: {}", msg),
            StreamingError::Format(msg) => write!(f, "Format error: {}", msg),
            StreamingError::Performance(msg) => write!(f, "Performance error: {}", msg),
            StreamingError::Streaming(msg) => write!(f, "Streaming behavior error: {}", msg),
            StreamingError::Buffer(msg) => write!(f, "Buffer error: {}", msg),
            StreamingError::Timeout => write!(f, "Timeout error"),
        }
    }
}

impl std::error::Error for StreamingError {}

// ==================== INTEGRATION TESTS ====================

/// Test TTFB < 500ms requirement with warm cache
#[tokio::test]
async fn test_ttfb_500ms_requirement_warm_cache() {
    let framework = StreamingValidationFramework::new();
    let urls = framework.setup_timed_content_mocks(3, 50); // Fast responses

    // First request to warm cache
    let request = framework.create_performance_crawl_request(urls.clone(), "write_through", 3);
    let _ = framework.make_validated_streaming_request("crawl/stream", request.clone(), None).await;

    // Second request should hit warm cache and meet TTFB requirement
    let response = framework.make_validated_streaming_request(
        "crawl/stream",
        request,
        Some(500), // Enforce 500ms TTFB limit
    ).await.expect("TTFB test with warm cache should succeed");

    // Validate format compliance
    response.validate_ndjson_format().expect("NDJSON format should be valid");
    response.validate_streaming_behavior().expect("Should demonstrate streaming behavior");

    println!("TTFB with warm cache: {}ms", response.ttfb.as_millis());
    assert!(response.ttfb.as_millis() < 500, "TTFB requirement not met");
}

/// Test buffer management with 65536 bytes limit
#[tokio::test]
async fn test_buffer_management_65536_bytes() {
    let framework = StreamingValidationFramework::new();
    let urls = framework.setup_timed_content_mocks(20, 100); // Many URLs

    let request = framework.create_performance_crawl_request(urls, "disabled", 10);
    let response = framework.make_validated_streaming_request(
        "crawl/stream",
        request,
        None,
    ).await.expect("Buffer management test should succeed");

    // Validate buffer usage stays within limits
    response.validate_buffer_usage(65536).expect("Buffer usage should be within limits");
    response.validate_streaming_behavior().expect("Should stream incrementally");

    assert_eq!(response.result_lines().len(), 20, "All results should be delivered");
    println!("Total bytes: {}, Throughput: {:.2} items/sec", 
             response.total_bytes, response.throughput());
}

/// Test that results stream as they complete (no batching)
#[tokio::test]
async fn test_no_batching_streaming_behavior() {
    let framework = StreamingValidationFramework::new();
    
    // Create URLs with different processing times to test streaming
    let mut urls = Vec::new();
    for (i, delay) in [100, 300, 150, 250, 200].iter().enumerate() {
        let path = format!("/streaming-test-{}", i);
        let _mock = framework.mock_server.mock(|when, then| {
            when.method(GET).path(&path);
            then.status(200)
                .delay(Duration::from_millis(*delay))
                .header("content-type", "text/html")
                .body(&format!("<html><body><h1>Content {} ({}ms delay)</h1></body></html>", i, delay));
        });
        urls.push(format!("{}/streaming-test-{}", framework.mock_server.base_url(), i));
    }

    let request = framework.create_performance_crawl_request(urls, "disabled", 5);
    let response = framework.make_validated_streaming_request(
        "crawl/stream",
        request,
        None,
    ).await.expect("Streaming behavior test should succeed");

    response.validate_streaming_behavior().expect("Should demonstrate true streaming");
    response.validate_ndjson_format().expect("Format should be valid");

    // Results should arrive in order of completion, not request order
    assert_eq!(response.result_lines().len(), 5);
    println!("Line timestamps: {:?}", response.line_timestamps.iter()
             .map(|d| d.as_millis()).collect::<Vec<_>>());
}

/// Test error scenarios and recovery paths
#[tokio::test]
async fn test_error_scenarios_and_recovery() {
    let framework = StreamingValidationFramework::new();
    let urls = framework.setup_mixed_reliability_mocks(10, 0.6); // 60% success rate

    let request = framework.create_performance_crawl_request(urls, "disabled", 5);
    let response = framework.make_validated_streaming_request(
        "crawl/stream",
        request,
        None,
    ).await.expect("Error scenario test should succeed overall");

    response.validate_ndjson_format().expect("Format should be valid despite errors");

    let total_results = response.result_lines().len();
    let success_count = response.success_count();
    let error_count = response.error_count();

    assert_eq!(total_results, 10, "Should process all URLs despite errors");
    assert!(success_count >= 5, "Should have some successes");
    assert!(error_count >= 3, "Should have some errors");
    assert_eq!(success_count + error_count, total_results, "All results should be accounted for");

    // Validate error structure
    for line in response.result_lines() {
        if let Some(result) = line.get("result") {
            if let Some(error) = result.get("error") {
                assert!(error["error_type"].is_string(), "Error should have type");
                assert!(error["message"].is_string(), "Error should have message");
                assert!(error["retryable"].is_boolean(), "Error should indicate if retryable");
            }
        }
    }

    println!("Success: {}, Errors: {}, Total: {}", success_count, error_count, total_results);
}

/// Test stream lifecycle management
#[tokio::test]
async fn test_stream_lifecycle_management() {
    let framework = StreamingValidationFramework::new();
    let urls = framework.setup_timed_content_mocks(5, 200);

    let request = framework.create_performance_crawl_request(urls, "disabled", 3);
    let response = framework.make_validated_streaming_request(
        "crawl/stream",
        request,
        Some(1000),
    ).await.expect("Stream lifecycle test should succeed");

    // Validate complete lifecycle: start -> stream -> close
    assert!(!response.lines.is_empty(), "Should have lines");

    // Should have metadata (start)
    let metadata = response.lines.first().expect("Should have first line");
    assert!(metadata.get("total_urls").is_some(), "Should have metadata");
    assert!(metadata.get("request_id").is_some(), "Should have request ID");
    assert_eq!(metadata["stream_type"], "crawl");

    // Should have results (stream)
    let results = response.result_lines();
    assert_eq!(results.len(), 5, "Should have all results");

    // Should have summary (close)
    let summary = response.lines.last().expect("Should have last line");
    assert!(summary.get("total_urls").is_some(), "Should have summary");
    assert!(summary.get("successful").is_some(), "Should report success count");

    // Validate request ID consistency
    let request_id = metadata["request_id"].as_str().expect("Should have request ID");
    if let Some(header_request_id) = response.headers.get("X-Request-ID") {
        assert_eq!(header_request_id.to_str().unwrap(), request_id, 
                   "Request ID should be consistent across headers and content");
    }

    println!("Stream lifecycle validated with request ID: {}", request_id);
}

/// Test concurrent streaming sessions
#[tokio::test]
async fn test_concurrent_streaming_isolation() {
    let framework = StreamingValidationFramework::new();

    let urls1 = framework.setup_timed_content_mocks(3, 150);
    let urls2 = framework.setup_timed_content_mocks(4, 100);

    let request1 = framework.create_performance_crawl_request(urls1, "disabled", 2);
    let request2 = framework.create_performance_crawl_request(urls2, "disabled", 3);

    // Run concurrently
    let (result1, result2) = tokio::join!(
        framework.make_validated_streaming_request("crawl/stream", request1, Some(2000)),
        framework.make_validated_streaming_request("crawl/stream", request2, Some(2000))
    );

    let response1 = result1.expect("First concurrent stream should succeed");
    let response2 = result2.expect("Second concurrent stream should succeed");

    // Validate isolation
    assert_eq!(response1.result_lines().len(), 3);
    assert_eq!(response2.result_lines().len(), 4);

    // Different request IDs
    let req_id1 = response1.lines[0]["request_id"].as_str().unwrap();
    let req_id2 = response2.lines[0]["request_id"].as_str().unwrap();
    assert_ne!(req_id1, req_id2, "Concurrent streams should have different request IDs");

    // Both should meet performance requirements
    response1.validate_streaming_behavior().expect("Stream 1 should demonstrate streaming");
    response2.validate_streaming_behavior().expect("Stream 2 should demonstrate streaming");

    println!("Concurrent streams completed: {} ({}ms), {} ({}ms)",
             req_id1, response1.total_time.as_millis(),
             req_id2, response2.total_time.as_millis());
}

/// Test backpressure handling under high load
#[tokio::test]
async fn test_backpressure_handling_high_load() {
    let framework = StreamingValidationFramework::new();
    
    // Create high load scenario
    let urls = framework.setup_timed_content_mocks(50, 100);
    let request = framework.create_performance_crawl_request(urls, "disabled", 20);

    let start_time = Instant::now();
    let response = timeout(
        Duration::from_secs(30),
        framework.make_validated_streaming_request("crawl/stream", request, None)
    ).await
        .map_err(|_| StreamingError::Timeout)?
        .expect("High load test should succeed");

    let total_time = start_time.elapsed();

    // Should handle all URLs without dropping (good backpressure handling)
    assert_eq!(response.result_lines().len(), 50, "All URLs should be processed");
    
    // Should maintain reasonable throughput
    let throughput = response.throughput();
    assert!(throughput > 1.0, "Should maintain reasonable throughput: {:.2} items/sec", throughput);

    // Should stay within buffer limits
    response.validate_buffer_usage(65536).expect("Should manage buffer properly under load");

    println!("High load test: {} URLs in {}ms, throughput: {:.2} items/sec",
             response.result_lines().len(), total_time.as_millis(), throughput);
}

/// Performance benchmark test
#[tokio::test]
async fn test_streaming_performance_benchmark() {
    let framework = StreamingValidationFramework::new();
    let urls = framework.setup_timed_content_mocks(25, 80);

    let request = framework.create_performance_crawl_request(urls, "write_through", 10);
    
    let start_time = Instant::now();
    let response = framework.make_validated_streaming_request(
        "crawl/stream",
        request,
        Some(1000), // TTFB requirement
    ).await.expect("Performance benchmark should succeed");

    let total_time = start_time.elapsed();

    // Performance assertions
    assert!(response.ttfb.as_millis() < 1000, "TTFB should be under 1s");
    assert!(response.first_line_time.as_millis() < 1500, "First line should arrive quickly");
    
    let throughput = response.throughput();
    assert!(throughput > 2.0, "Should achieve reasonable throughput: {:.2} items/sec", throughput);

    // Validate all performance requirements
    response.validate_ndjson_format().expect("Format compliance");
    response.validate_streaming_behavior().expect("Streaming behavior");
    response.validate_buffer_usage(65536).expect("Buffer management");

    println!(
        "Performance Benchmark Results:\n" +
        "  TTFB: {}ms\n" +
        "  First Line: {}ms\n" +
        "  Total Time: {}ms\n" +
        "  Throughput: {:.2} items/sec\n" +
        "  Total Bytes: {}\n" +
        "  Success Rate: {:.1}%",
        response.ttfb.as_millis(),
        response.first_line_time.as_millis(),
        total_time.as_millis(),
        throughput,
        response.total_bytes,
        (response.success_count() as f64 / response.result_lines().len() as f64) * 100.0
    );
}
