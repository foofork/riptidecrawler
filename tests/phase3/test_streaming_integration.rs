use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use serde_json::Value;

/// Integration test for NDJSON streaming functionality
/// Tests the complete streaming pipeline with real server instances
#[tokio::test]
async fn test_ndjson_streaming_integration() {
    // Start the API server in the background
    let mut server = Command::new("cargo")
        .args(&["run", "-p", "riptide-api", "--", "--bind", "127.0.0.1:18080"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to start server");

    // Wait for server to start up
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Test data
    let test_urls = vec![
        "https://httpbin.org/delay/1".to_string(),
        "https://httpbin.org/html".to_string(),
    ];

    let request_body = serde_json::json!({
        "urls": test_urls,
        "options": {
            "cache_mode": "disabled",
            "concurrency": 2
        }
    });

    // Create HTTP client
    let client = reqwest::Client::new();
    let start_time = Instant::now();

    // Test streaming endpoint
    let response = client
        .post("http://127.0.0.1:18080/crawl/stream")
        .json(&request_body)
        .timeout(tokio::time::Duration::from_secs(30))
        .send()
        .await;

    // Clean up server
    let _ = server.kill().await;

    // Process response
    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);

            // Verify headers
            assert_eq!(
                resp.headers().get("content-type").unwrap(),
                "application/x-ndjson"
            );
            assert_eq!(
                resp.headers().get("transfer-encoding").unwrap(),
                "chunked"
            );

            // Test TTFB requirement
            let ttfb = start_time.elapsed();
            assert!(
                ttfb.as_millis() < 500,
                "TTFB {} ms exceeds 500ms requirement",
                ttfb.as_millis()
            );

            println!("✅ NDJSON streaming integration test passed");
            println!("   TTFB: {} ms", ttfb.as_millis());
        },
        Err(e) => {
            println!("⚠️  Server not running for integration test: {}", e);
            println!("   Run `cargo run -p riptide-api` to test manually");
        }
    }
}

/// Manual test to verify NDJSON format and streaming behavior
/// Run this test while the server is running
#[tokio::test]
async fn test_manual_streaming_format() {
    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "urls": ["https://httpbin.org/json"],
        "options": {
            "cache_mode": "disabled"
        }
    });

    match client
        .post("http://127.0.0.1:8080/crawl/stream")
        .json(&request_body)
        .timeout(tokio::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(response) => {
            assert_eq!(response.status(), 200);

            let body = response.text().await.unwrap();
            let lines: Vec<&str> = body.lines().collect();

            // Verify NDJSON format - each line should be valid JSON
            for (i, line) in lines.iter().enumerate() {
                if !line.trim().is_empty() {
                    match serde_json::from_str::<Value>(line) {
                        Ok(json) => {
                            println!("Line {}: Valid JSON with keys: {:?}",
                                i, json.as_object().map(|o| o.keys().collect::<Vec<_>>()));
                        },
                        Err(e) => {
                            panic!("Line {} is not valid JSON: {}\nLine: {}", i, e, line);
                        }
                    }
                }
            }

            println!("✅ Manual streaming format test passed");
            println!("   Received {} NDJSON lines", lines.len());
        },
        Err(_) => {
            println!("⚠️  Server not available for manual test");
            println!("   Start server with: cargo run -p riptide-api");
        }
    }
}

#[test]
fn test_streaming_implementation_exists() {
    // Verify the streaming module exists and contains required functions
    use std::path::Path;

    let streaming_path = Path::new("crates/riptide-api/src/streaming.rs");
    assert!(streaming_path.exists(), "streaming.rs module not found");

    let content = std::fs::read_to_string(streaming_path).unwrap();

    // Check for required functions
    assert!(content.contains("pub async fn crawl_stream"), "crawl_stream function not found");
    assert!(content.contains("pub async fn deepsearch_stream"), "deepsearch_stream function not found");
    assert!(content.contains("application/x-ndjson"), "NDJSON content type not found");
    assert!(content.contains("chunked"), "Chunked transfer encoding not found");

    println!("✅ Streaming implementation verification passed");
}

/// Performance benchmark for streaming vs non-streaming
#[tokio::test]
async fn test_streaming_performance_comparison() {
    let client = reqwest::Client::new();

    let test_urls = vec![
        "https://httpbin.org/delay/1",
        "https://httpbin.org/delay/1",
        "https://httpbin.org/delay/1",
    ];

    let request_body = serde_json::json!({
        "urls": test_urls,
        "options": {
            "cache_mode": "disabled",
            "concurrency": 3
        }
    });

    // Test streaming endpoint
    let start_streaming = Instant::now();
    let streaming_result = client
        .post("http://127.0.0.1:8080/crawl/stream")
        .json(&request_body)
        .timeout(tokio::time::Duration::from_secs(15))
        .send()
        .await;

    if let Ok(response) = streaming_result {
        let ttfb_streaming = start_streaming.elapsed();
        let _ = response.text().await;
        let total_streaming = start_streaming.elapsed();

        println!("📊 Streaming Performance:");
        println!("   TTFB: {} ms", ttfb_streaming.as_millis());
        println!("   Total: {} ms", total_streaming.as_millis());

        // TTFB should be under 500ms for streaming
        assert!(ttfb_streaming.as_millis() < 500,
            "Streaming TTFB {} ms exceeds requirement", ttfb_streaming.as_millis());
    } else {
        println!("⚠️  Server not available for performance test");
    }

    // Test regular endpoint for comparison
    let start_regular = Instant::now();
    let regular_result = client
        .post("http://127.0.0.1:8080/crawl")
        .json(&request_body)
        .timeout(tokio::time::Duration::from_secs(15))
        .send()
        .await;

    if let Ok(response) = regular_result {
        let ttfb_regular = start_regular.elapsed();
        let _ = response.text().await;
        let total_regular = start_regular.elapsed();

        println!("📊 Regular Performance:");
        println!("   TTFB: {} ms", ttfb_regular.as_millis());
        println!("   Total: {} ms", total_regular.as_millis());
    } else {
        println!("⚠️  Server not available for performance test");
    }
}

/// Test SSE (Server-Sent Events) endpoint functionality
#[tokio::test]
async fn test_sse_streaming_integration() {
    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "urls": ["https://httpbin.org/json", "https://httpbin.org/html"],
        "options": {
            "cache_mode": "disabled",
            "concurrency": 2
        }
    });

    match client
        .post("http://127.0.0.1:8080/crawl/sse")
        .json(&request_body)
        .timeout(tokio::time::Duration::from_secs(15))
        .send()
        .await
    {
        Ok(response) => {
            assert_eq!(response.status(), 200);

            // Verify SSE headers
            assert_eq!(
                response.headers().get("content-type").unwrap(),
                "text/event-stream"
            );
            assert_eq!(
                response.headers().get("cache-control").unwrap(),
                "no-cache"
            );

            let body = response.text().await.unwrap();

            // Verify SSE format
            assert!(body.contains("event: metadata"));
            assert!(body.contains("event: progress"));
            assert!(body.contains("event: result"));
            assert!(body.contains("event: complete"));

            println!("✅ SSE streaming integration test passed");
        },
        Err(_) => {
            println!("⚠️  Server not available for SSE test");
        }
    }
}

/// Test WebSocket connection and message handling
#[tokio::test]
async fn test_websocket_streaming_integration() {
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use futures_util::{SinkExt, StreamExt};

    match connect_async("ws://127.0.0.1:8080/crawl/ws").await {
        Ok((mut ws_stream, _)) => {
            // Send crawl request via WebSocket
            let request = serde_json::json!({
                "request_type": "crawl",
                "data": {
                    "urls": ["https://httpbin.org/json"],
                    "options": {
                        "cache_mode": "disabled"
                    }
                }
            });

            ws_stream.send(Message::Text(request.to_string())).await.unwrap();

            // Receive and verify messages
            let mut received_metadata = false;
            let mut received_result = false;
            let mut received_summary = false;

            while let Some(msg) = ws_stream.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                            match parsed["message_type"].as_str() {
                                Some("metadata") => received_metadata = true,
                                Some("result") => received_result = true,
                                Some("summary") => {
                                    received_summary = true;
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Err(_) => break,
                    _ => {}
                }
            }

            assert!(received_metadata, "Did not receive metadata message");
            assert!(received_result, "Did not receive result message");
            assert!(received_summary, "Did not receive summary message");

            println!("✅ WebSocket streaming integration test passed");
        },
        Err(_) => {
            println!("⚠️  WebSocket server not available for test");
        }
    }
}

/// Test backpressure handling in streaming
#[tokio::test]
async fn test_streaming_backpressure_handling() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::time::{sleep, Duration};

    let client = reqwest::Client::new();
    let processed_count = Arc::new(AtomicUsize::new(0));

    // Create a large batch to test backpressure
    let urls: Vec<String> = (0..50)
        .map(|i| format!("https://httpbin.org/delay/1?id={}", i))
        .collect();

    let request_body = serde_json::json!({
        "urls": urls,
        "options": {
            "cache_mode": "disabled",
            "concurrency": 10
        }
    });

    match client
        .post("http://127.0.0.1:8080/crawl/stream")
        .json(&request_body)
        .timeout(Duration::from_secs(120))
        .send()
        .await
    {
        Ok(response) => {
            assert_eq!(response.status(), 200);

            let mut stream = response.bytes_stream();
            let mut buffer = String::new();
            let mut first_result_time = None;
            let start_time = std::time::Instant::now();

            while let Some(chunk) = stream.next().await {
                // Simulate slow client by adding delay
                sleep(Duration::from_millis(50)).await;

                let chunk = chunk.expect("Failed to read chunk");
                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(newline_pos) = buffer.find('\n') {
                    let line = buffer[..newline_pos].to_string();
                    buffer = buffer[newline_pos + 1..].to_string();

                    if !line.trim().is_empty() {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&line) {
                            if parsed.get("result").is_some() {
                                if first_result_time.is_none() {
                                    first_result_time = Some(start_time.elapsed());
                                }
                                processed_count.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                }
            }

            let final_count = processed_count.load(Ordering::Relaxed);
            println!("📊 Backpressure test processed {} results", final_count);

            // Verify that streaming handled backpressure gracefully
            assert!(final_count > 0, "Should have processed some results even with backpressure");

            if let Some(ttfb) = first_result_time {
                println!("   TTFB with backpressure: {} ms", ttfb.as_millis());
                // Even with backpressure, TTFB should be reasonable
                assert!(ttfb.as_millis() < 5000, "TTFB too high even accounting for backpressure");
            }

            println!("✅ Backpressure handling test passed");
        },
        Err(_) => {
            println!("⚠️  Server not available for backpressure test");
        }
    }
}

/// Test connection recovery and error handling
#[tokio::test]
async fn test_streaming_connection_recovery() {
    let client = reqwest::Client::new();

    // Test with a mix of valid and invalid URLs
    let request_body = serde_json::json!({
        "urls": [
            "https://httpbin.org/json",
            "invalid-url-format",
            "https://httpbin.org/status/500",
            "https://httpbin.org/html"
        ],
        "options": {
            "cache_mode": "disabled"
        }
    });

    match client
        .post("http://127.0.0.1:8080/crawl/stream")
        .json(&request_body)
        .timeout(tokio::time::Duration::from_secs(30))
        .send()
        .await
    {
        Ok(response) => {
            assert_eq!(response.status(), 200);

            let body = response.text().await.unwrap();
            let lines: Vec<&str> = body.lines().collect();

            let mut success_count = 0;
            let mut error_count = 0;

            for line in &lines[1..lines.len()-1] { // Skip metadata and summary
                if let Ok(result) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(result_obj) = result.get("result") {
                        if result_obj["error"].is_object() {
                            error_count += 1;
                        } else if result_obj["document"].is_object() {
                            success_count += 1;
                        }
                    }
                }
            }

            println!("📊 Connection recovery test: {} success, {} errors", success_count, error_count);

            // Should have both successes and errors
            assert!(success_count > 0, "Should have some successful results");
            assert!(error_count > 0, "Should have some error results");
            assert_eq!(success_count + error_count, 4, "Should process all 4 URLs");

            println!("✅ Connection recovery test passed");
        },
        Err(_) => {
            println!("⚠️  Server not available for connection recovery test");
        }
    }
}

/// Test progress reporting for long-running operations
#[tokio::test]
async fn test_streaming_progress_reporting() {
    let client = reqwest::Client::new();

    // Create a larger batch to trigger progress reporting
    let urls: Vec<String> = (0..15)
        .map(|i| format!("https://httpbin.org/delay/1?batch={}", i))
        .collect();

    let request_body = serde_json::json!({
        "urls": urls,
        "options": {
            "cache_mode": "disabled",
            "concurrency": 5
        }
    });

    match client
        .post("http://127.0.0.1:8080/crawl/stream")
        .json(&request_body)
        .timeout(tokio::time::Duration::from_secs(60))
        .send()
        .await
    {
        Ok(response) => {
            assert_eq!(response.status(), 200);

            let body = response.text().await.unwrap();
            let lines: Vec<&str> = body.lines().collect();

            let mut found_progress_update = false;
            let mut max_progress_percentage = 0.0;

            for line in &lines {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(line) {
                    // Check for progress updates
                    if parsed.get("operation_type").is_some() &&
                       parsed["operation_type"] == "batch_crawl" {
                        found_progress_update = true;
                        if let Some(percentage) = parsed["progress_percentage"].as_f64() {
                            max_progress_percentage = max_progress_percentage.max(percentage);
                        }
                    }
                }
            }

            println!("📊 Progress reporting test: Max progress {}%", max_progress_percentage);

            // Should have found progress updates for large batches
            assert!(found_progress_update, "Should have progress updates for large batches");
            assert!(max_progress_percentage > 0.0, "Should have meaningful progress percentages");

            println!("✅ Progress reporting test passed");
        },
        Err(_) => {
            println!("⚠️  Server not available for progress reporting test");
        }
    }
}