use futures::stream::StreamExt;
use httpmock::prelude::*;
use serde_json::Value;
use std::time::Instant;
use tokio_test;

/// Test NDJSON streaming endpoints for RipTide API.
///
/// This test module verifies that:
/// 1. Streaming endpoints return proper NDJSON format
/// 2. Results are streamed as they become available (not batched)
/// 3. TTFB (Time to First Byte) is under 500ms
/// 4. Stream metadata and summaries are properly formatted
/// 5. Error handling works correctly in streaming mode

#[tokio::test]
async fn test_ndjson_crawl_streaming() {
    // Setup mock HTTP server
    let server = MockServer::start();

    // Mock successful HTTP responses
    let mock_response_1 = server.mock(|when, then| {
        when.method(GET).path("/test1");
        then.status(200)
            .header("content-type", "text/html")
            .body("<html><head><title>Test 1</title></head><body><p>Content 1</p></body></html>");
    });

    let mock_response_2 = server.mock(|when, then| {
        when.method(GET).path("/test2");
        then.status(200)
            .header("content-type", "text/html")
            .body("<html><head><title>Test 2</title></head><body><p>Content 2</p></body></html>");
    });

    // Prepare test URLs
    let test_urls = vec![
        format!("{}/test1", server.base_url()),
        format!("{}/test2", server.base_url()),
    ];

    // Create request body
    let request_body = serde_json::json!({
        "urls": test_urls,
        "options": {
            "cache_mode": "disabled",
            "concurrency": 2
        }
    });

    // Setup test client
    let client = reqwest::Client::new();

    // Record start time for TTFB measurement
    let start_time = Instant::now();

    // Make streaming request
    let response = client
        .post("http://localhost:8080/crawl/stream")
        .json(&request_body)
        .send()
        .await
        .expect("Failed to send request");

    // Verify response headers
    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/x-ndjson"
    );
    assert_eq!(
        response.headers().get("transfer-encoding").unwrap(),
        "chunked"
    );

    // Process streaming response
    let mut stream = response.bytes_stream();
    let mut lines = Vec::new();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let ttfb = start_time.elapsed();
        assert!(
            ttfb.as_millis() < 500,
            "TTFB {} ms exceeds 500ms requirement",
            ttfb.as_millis()
        );

        let chunk = chunk.expect("Failed to read chunk");
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        // Process complete lines
        while let Some(newline_pos) = buffer.find('\n') {
            let line = buffer[..newline_pos].to_string();
            buffer = buffer[newline_pos + 1..].to_string();

            if !line.trim().is_empty() {
                lines.push(line);
            }
        }
    }

    // Verify we received expected number of lines
    // Expected: metadata + 2 results + summary = 4 lines
    assert!(
        lines.len() >= 4,
        "Expected at least 4 NDJSON lines, got {}",
        lines.len()
    );

    // Parse and verify first line (metadata)
    let metadata: Value = serde_json::from_str(&lines[0]).expect("Failed to parse metadata line");

    assert_eq!(metadata["total_urls"], 2);
    assert_eq!(metadata["stream_type"], "crawl");
    assert!(metadata["request_id"].is_string());
    assert!(metadata["timestamp"].is_string());

    // Verify individual results (should be lines 1 and 2)
    let mut result_count = 0;
    let mut successful_results = 0;

    for line in &lines[1..lines.len() - 1] {
        if let Ok(result) = serde_json::from_str::<Value>(line) {
            if result.get("result").is_some() {
                result_count += 1;

                // Verify result structure
                let result_obj = &result["result"];
                assert!(result_obj["url"].is_string());
                assert!(result_obj["status"].is_number());
                assert!(result_obj["gate_decision"].is_string());
                assert!(result_obj["quality_score"].is_number());
                assert!(result_obj["processing_time_ms"].is_number());

                if result_obj["status"].as_u64().unwrap() == 200 {
                    successful_results += 1;
                    assert!(result_obj["document"].is_object());
                } else {
                    assert!(result_obj["error"].is_object());
                }

                // Verify progress information
                let progress = &result["progress"];
                assert!(progress["completed"].is_number());
                assert_eq!(progress["total"], 2);
                assert!(progress["success_rate"].is_number());
            }
        }
    }

    assert_eq!(result_count, 2, "Expected 2 result lines");

    // Parse and verify last line (summary)
    let summary: Value =
        serde_json::from_str(lines.last().unwrap()).expect("Failed to parse summary line");

    assert_eq!(summary["total_urls"], 2);
    assert!(summary["successful"].is_number());
    assert!(summary["failed"].is_number());
    assert!(summary["total_processing_time_ms"].is_number());
    assert!(summary["cache_hit_rate"].is_number());

    // Verify mocks were called
    mock_response_1.assert();
    mock_response_2.assert();
}

#[tokio::test]
async fn test_ndjson_deepsearch_streaming() {
    // Setup environment for Serper API
    std::env::set_var("SERPER_API_KEY", "test_key");

    // Mock Serper API response
    let serper_server = MockServer::start();
    let _serper_mock = serper_server.mock(|when, then| {
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

    // Mock the target websites
    let content_server = MockServer::start();
    let _content_mock1 = content_server.mock(|when, then| {
        when.method(GET).path("/test1");
        then.status(200)
            .header("content-type", "text/html")
            .body("<html><head><title>Test 1</title></head><body><p>Content 1</p></body></html>");
    });

    let _content_mock2 = content_server.mock(|when, then| {
        when.method(GET).path("/test2");
        then.status(200)
            .header("content-type", "text/html")
            .body("<html><head><title>Test 2</title></head><body><p>Content 2</p></body></html>");
    });

    // Create request body
    let request_body = serde_json::json!({
        "query": "test search",
        "limit": 2,
        "include_content": true,
        "crawl_options": {
            "cache_mode": "disabled"
        }
    });

    // Setup test client
    let client = reqwest::Client::new();

    // Record start time for TTFB measurement
    let start_time = Instant::now();

    // Make streaming request
    let response = client
        .post("http://localhost:8080/deepsearch/stream")
        .json(&request_body)
        .send()
        .await
        .expect("Failed to send request");

    // Verify response headers
    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/x-ndjson"
    );

    // Process streaming response
    let mut stream = response.bytes_stream();
    let mut lines = Vec::new();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let ttfb = start_time.elapsed();
        assert!(
            ttfb.as_millis() < 500,
            "TTFB {} ms exceeds 500ms requirement",
            ttfb.as_millis()
        );

        let chunk = chunk.expect("Failed to read chunk");
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        // Process complete lines
        while let Some(newline_pos) = buffer.find('\n') {
            let line = buffer[..newline_pos].to_string();
            buffer = buffer[newline_pos + 1..].to_string();

            if !line.trim().is_empty() {
                lines.push(line);
            }
        }
    }

    // Verify we received expected lines
    // Expected: metadata + search_metadata + 2 results + summary = 5+ lines
    assert!(
        lines.len() >= 5,
        "Expected at least 5 NDJSON lines, got {}",
        lines.len()
    );

    // Parse and verify first line (stream metadata)
    let metadata: Value = serde_json::from_str(&lines[0]).expect("Failed to parse metadata line");

    assert_eq!(metadata["stream_type"], "deepsearch");
    assert!(metadata["request_id"].is_string());

    // Find and verify search metadata
    let mut found_search_metadata = false;
    for line in &lines[1..] {
        if let Ok(parsed) = serde_json::from_str::<Value>(line) {
            if parsed.get("query").is_some() && parsed.get("urls_found").is_some() {
                assert_eq!(parsed["query"], "test search");
                assert!(parsed["urls_found"].is_number());
                found_search_metadata = true;
                break;
            }
        }
    }
    assert!(found_search_metadata, "Search metadata not found");

    // Verify at least one search result with crawl data
    let mut found_search_result = false;
    for line in &lines[1..lines.len() - 1] {
        if let Ok(parsed) = serde_json::from_str::<Value>(line) {
            if parsed.get("search_result").is_some() {
                let search_result = &parsed["search_result"];
                assert!(search_result["url"].is_string());
                assert!(search_result["rank"].is_number());
                assert!(search_result["search_title"].is_string());

                // If content extraction was requested, verify crawl result
                if parsed["crawl_result"].is_object() {
                    let crawl_result = &parsed["crawl_result"];
                    assert!(crawl_result["status"].is_number());
                    assert!(crawl_result["gate_decision"].is_string());
                }

                found_search_result = true;
            }
        }
    }
    assert!(found_search_result, "Search result not found");

    // Parse and verify summary
    let summary: Value =
        serde_json::from_str(lines.last().unwrap()).expect("Failed to parse summary line");

    assert_eq!(summary["query"], "test search");
    assert!(summary["total_urls_found"].is_number());
    assert!(summary["total_processing_time_ms"].is_number());
    assert_eq!(summary["status"], "completed");
}

#[tokio::test]
async fn test_streaming_error_handling() {
    // Test streaming with invalid URLs to verify error handling
    let request_body = serde_json::json!({
        "urls": [
            "invalid-url",
            "http://nonexistent.example.com"
        ],
        "options": {
            "cache_mode": "disabled"
        }
    });

    let client = reqwest::Client::new();

    let response = client
        .post("http://localhost:8080/crawl/stream")
        .json(&request_body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    // Process streaming response
    let mut stream = response.bytes_stream();
    let mut lines = Vec::new();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.expect("Failed to read chunk");
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(newline_pos) = buffer.find('\n') {
            let line = buffer[..newline_pos].to_string();
            buffer = buffer[newline_pos + 1..].to_string();

            if !line.trim().is_empty() {
                lines.push(line);
            }
        }
    }

    // Verify error results are properly formatted
    let mut error_count = 0;
    for line in &lines[1..lines.len() - 1] {
        if let Ok(result) = serde_json::from_str::<Value>(line) {
            if let Some(result_obj) = result.get("result") {
                if result_obj["error"].is_object() {
                    error_count += 1;
                    let error = &result_obj["error"];
                    assert!(error["error_type"].is_string());
                    assert!(error["message"].is_string());
                    assert!(error["retryable"].is_boolean());
                }
            }
        }
    }

    assert!(error_count > 0, "Expected at least one error result");
}

#[tokio::test]
async fn test_streaming_request_validation() {
    // Test that invalid requests are rejected early
    let invalid_request = serde_json::json!({
        "urls": [] // Empty URLs array should be invalid
    });

    let client = reqwest::Client::new();

    let response = client
        .post("http://localhost:8080/crawl/stream")
        .json(&invalid_request)
        .send()
        .await
        .expect("Failed to send request");

    // Should return 400 Bad Request for validation errors
    assert_eq!(response.status(), 400);

    let response_text = response.text().await.expect("Failed to read response");
    let response_json: Value =
        serde_json::from_str(&response_text).expect("Failed to parse error response");

    assert_eq!(response_json["error"]["type"], "validation_error");
    assert_eq!(response_json["error"]["retryable"], false);
}

#[tokio::test]
async fn test_streaming_large_batch_performance() {
    // Test streaming with larger batch to verify performance characteristics
    let server = MockServer::start();

    // Create multiple mock endpoints
    let mut urls = Vec::new();
    for i in 0..10 {
        let _mock = server.mock(|when, then| {
            when.method(GET).path(&format!("/test{}", i));
            then.status(200)
                .header("content-type", "text/html")
                .body(&format!(
                "<html><head><title>Test {}</title></head><body><p>Content {}</p></body></html>",
                i, i
            ));
        });

        urls.push(format!("{}/test{}", server.base_url(), i));
    }

    let request_body = serde_json::json!({
        "urls": urls,
        "options": {
            "cache_mode": "disabled",
            "concurrency": 4
        }
    });

    let client = reqwest::Client::new();
    let start_time = Instant::now();

    let response = client
        .post("http://localhost:8080/crawl/stream")
        .json(&request_body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    // Process stream and count results
    let mut stream = response.bytes_stream();
    let mut result_count = 0;
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let first_result_time = start_time.elapsed();
        let chunk = chunk.expect("Failed to read chunk");
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(newline_pos) = buffer.find('\n') {
            let line = buffer[..newline_pos].to_string();
            buffer = buffer[newline_pos + 1..].to_string();

            if !line.trim().is_empty() {
                if let Ok(parsed) = serde_json::from_str::<Value>(&line) {
                    if parsed.get("result").is_some() {
                        result_count += 1;
                    }
                }
            }
        }
    }

    // Verify performance characteristics
    assert_eq!(result_count, 10, "Expected 10 results");

    // First result should arrive quickly (streaming benefit)
    let first_result_duration = start_time.elapsed();
    assert!(
        first_result_duration.as_millis() < 2000,
        "First result took {} ms, expected < 2000ms for streaming benefit",
        first_result_duration.as_millis()
    );

    let total_time = start_time.elapsed();
    println!(
        "Total streaming time for 10 URLs: {} ms",
        total_time.as_millis()
    );
}
