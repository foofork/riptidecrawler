//! Comprehensive wireup and activation tests for RipTide API
//!
//! Tests based on WEEK_1_ACTION_PLAN.md requirements:
//! - Health check endpoint
//! - Simple extraction (example.com)
//! - Real URL extraction (20+ diverse sites)
//! - Error handling and edge cases
//! - Metrics validation

use crate::common::{assert_error_contains, with_timeout};
use crate::config::TestConfig;
use reqwest;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;

/// Test URLs based on WEEK_1_ACTION_PLAN.md diverse site requirements
const TEST_URLS: &[(&str, &str)] = &[
    // Simple static
    ("https://example.com", "simple_static"),
    // News sites
    ("https://www.bbc.com/news/technology", "news_bbc"),
    ("https://techcrunch.com/latest", "news_techcrunch"),
    // Blogs
    ("https://martinfowler.com/articles/", "blog_fowler"),
    // Documentation
    ("https://docs.rust-lang.org/book/", "docs_rust"),
    // E-commerce
    ("https://www.amazon.com/dp/B08N5WRWNW", "ecommerce_amazon"),
    // Social
    ("https://dev.to/", "social_devto"),
    // Complex SPAs
    ("https://github.com/trending", "spa_github"),
    // Additional diverse sites
    ("https://www.wikipedia.org/", "reference_wiki"),
    ("https://www.reddit.com/r/programming/", "social_reddit"),
    ("https://stackoverflow.com/questions", "qa_stackoverflow"),
    ("https://news.ycombinator.com/", "aggregator_hn"),
    ("https://www.youtube.com/", "video_youtube"),
    ("https://twitter.com/", "social_twitter"),
    ("https://www.linkedin.com/", "social_linkedin"),
    ("https://medium.com/", "blog_medium"),
    ("https://www.nytimes.com/", "news_nyt"),
    ("https://www.theguardian.com/", "news_guardian"),
    ("https://www.cnn.com/", "news_cnn"),
    ("https://www.washingtonpost.com/", "news_wapo"),
];

/// Helper to start API server for tests
async fn start_test_api_server() -> Result<(String, tokio::process::Child), String> {
    use tokio::process::Command;

    let port = 8080;
    let base_url = format!("http://localhost:{}", port);

    // Start the API server
    let child = Command::new("cargo")
        .args(&["run", "--release", "--bin", "riptide-api"])
        .env("RUST_LOG", "info")
        .spawn()
        .map_err(|e| format!("Failed to start API server: {}", e))?;

    // Wait for server to be ready
    for _ in 0..30 {
        sleep(Duration::from_secs(1)).await;
        if let Ok(response) = reqwest::get(&format!("{}/healthz", base_url)).await {
            if response.status().is_success() {
                return Ok((base_url, child));
            }
        }
    }

    Err("API server failed to start within timeout".to_string())
}

/// Helper to stop API server
async fn stop_test_api_server(mut child: tokio::process::Child) {
    let _ = child.kill().await;
}

#[tokio::test]
async fn test_health_check_endpoint() {
    let (base_url, child) = start_test_api_server()
        .await
        .expect("Failed to start API server");

    let result = with_timeout(TestConfig::SHORT_TIMEOUT, async {
        let client = reqwest::Client::new();
        let response = client
            .get(&format!("{}/healthz", base_url))
            .send()
            .await
            .map_err(|e| format!("Health check request failed: {}", e))?;

        let status = response.status();
        let body: Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse health response: {}", e))?;

        // Validate health check response
        assert!(status.is_success(), "Health check should return 2xx status");
        assert!(
            body.get("status").is_some(),
            "Health response should have status field"
        );
        assert!(
            body.get("version").is_some(),
            "Health response should have version field"
        );

        Ok(())
    })
    .await;

    stop_test_api_server(child).await;

    assert!(
        result.is_ok(),
        "Health check test should pass: {:?}",
        result
    );
}

#[tokio::test]
async fn test_simple_extraction_example_com() {
    let (base_url, child) = start_test_api_server()
        .await
        .expect("Failed to start API server");

    let result = with_timeout(TestConfig::DEFAULT_TIMEOUT, async {
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/extract", base_url))
            .header("Content-Type", "application/json")
            .json(&json!({
                "url": "https://example.com"
            }))
            .send()
            .await
            .map_err(|e| format!("Extraction request failed: {}", e))?;

        let status = response.status();
        let body: Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse extraction response: {}", e))?;

        // Validate extraction response
        assert!(
            status.is_success(),
            "Extraction should return 2xx status, got: {}",
            status
        );
        assert!(
            body.get("success").and_then(|v| v.as_bool()).unwrap_or(false),
            "Extraction should be successful"
        );
        assert!(
            body.get("content").is_some(),
            "Extraction should return content"
        );

        let content = body["content"].as_str().unwrap_or("");
        assert!(
            !content.is_empty(),
            "Extracted content should not be empty"
        );
        assert!(
            content.len() > 50,
            "Extracted content should have meaningful length"
        );

        Ok(())
    })
    .await;

    stop_test_api_server(child).await;

    assert!(
        result.is_ok(),
        "Simple extraction test should pass: {:?}",
        result
    );
}

#[tokio::test]
#[ignore] // Run with --ignored flag for full test suite
async fn test_real_url_extractions() {
    let (base_url, child) = start_test_api_server()
        .await
        .expect("Failed to start API server");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client");

    let mut success_count = 0;
    let mut error_count = 0;
    let mut results = Vec::new();

    // Test each URL from our diverse set
    for (url, category) in TEST_URLS {
        let result = with_timeout(TestConfig::DEFAULT_TIMEOUT, async {
            let response = client
                .post(&format!("{}/extract", base_url))
                .header("Content-Type", "application/json")
                .json(&json!({
                    "url": url
                }))
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status = resp.status();
                    let body: Value = resp.json().await.unwrap_or_else(|_| json!({}));

                    if status.is_success()
                        && body.get("success").and_then(|v| v.as_bool()).unwrap_or(false)
                    {
                        Ok(format!("✓ {}: {} - Success", category, url))
                    } else {
                        Err(format!(
                            "✗ {}: {} - Failed with status {}",
                            category, url, status
                        ))
                    }
                }
                Err(e) => Err(format!("✗ {}: {} - Error: {}", category, url, e)),
            }
        })
        .await;

        match result {
            Ok(Ok(msg)) => {
                success_count += 1;
                results.push(msg);
            }
            Ok(Err(msg)) | Err(msg) => {
                error_count += 1;
                results.push(msg);
            }
        }

        // Small delay between requests to avoid rate limiting
        sleep(Duration::from_millis(500)).await;
    }

    stop_test_api_server(child).await;

    // Print results
    println!("\n=== Real URL Extraction Results ===");
    for result in &results {
        println!("{}", result);
    }
    println!(
        "\nTotal: {} URLs, Success: {}, Errors: {}",
        TEST_URLS.len(),
        success_count,
        error_count
    );
    println!(
        "Success Rate: {:.1}%",
        (success_count as f64 / TEST_URLS.len() as f64) * 100.0
    );

    // Week 1 success criteria: 90% success rate
    let success_rate = success_count as f64 / TEST_URLS.len() as f64;
    assert!(
        success_rate >= 0.90,
        "Success rate {:.1}% should be at least 90%",
        success_rate * 100.0
    );
}

#[tokio::test]
async fn test_error_handling_invalid_url() {
    let (base_url, child) = start_test_api_server()
        .await
        .expect("Failed to start API server");

    let result = with_timeout(TestConfig::SHORT_TIMEOUT, async {
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/extract", base_url))
            .header("Content-Type", "application/json")
            .json(&json!({
                "url": "not-a-valid-url"
            }))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let status = response.status();
        let body: Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        // Should return error response
        assert!(
            status.is_client_error() || !body.get("success").and_then(|v| v.as_bool()).unwrap_or(true),
            "Invalid URL should return error"
        );

        Ok(())
    })
    .await;

    stop_test_api_server(child).await;

    assert!(
        result.is_ok(),
        "Error handling test should pass: {:?}",
        result
    );
}

#[tokio::test]
async fn test_error_handling_missing_url() {
    let (base_url, child) = start_test_api_server()
        .await
        .expect("Failed to start API server");

    let result = with_timeout(TestConfig::SHORT_TIMEOUT, async {
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/extract", base_url))
            .header("Content-Type", "application/json")
            .json(&json!({}))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let status = response.status();

        // Should return 400 Bad Request
        assert!(
            status.is_client_error(),
            "Missing URL should return 4xx status, got: {}",
            status
        );

        Ok(())
    })
    .await;

    stop_test_api_server(child).await;

    assert!(
        result.is_ok(),
        "Missing URL test should pass: {:?}",
        result
    );
}

#[tokio::test]
async fn test_error_handling_timeout() {
    let (base_url, child) = start_test_api_server()
        .await
        .expect("Failed to start API server");

    let result = with_timeout(Duration::from_secs(5), async {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .expect("Failed to create client");

        // Use a URL that might timeout (very slow responding)
        let response = client
            .post(&format!("{}/extract", base_url))
            .header("Content-Type", "application/json")
            .json(&json!({
                "url": "https://httpbin.org/delay/10"
            }))
            .send()
            .await;

        // Either timeout error or extraction failure is acceptable
        match response {
            Ok(resp) => {
                let body: Value = resp.json().await.unwrap_or_else(|_| json!({}));
                let success = body.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                // It's OK if extraction fails gracefully
                assert!(
                    !success,
                    "Slow URL should either timeout or fail gracefully"
                );
            }
            Err(_) => {
                // Timeout error is expected
            }
        }

        Ok(())
    })
    .await;

    stop_test_api_server(child).await;

    // Test passes if we handled timeout gracefully (either way)
    assert!(
        result.is_ok(),
        "Timeout handling test should pass: {:?}",
        result
    );
}

#[tokio::test]
async fn test_metrics_endpoint() {
    let (base_url, child) = start_test_api_server()
        .await
        .expect("Failed to start API server");

    let result = with_timeout(TestConfig::SHORT_TIMEOUT, async {
        let client = reqwest::Client::new();

        // First, do an extraction to generate metrics
        let _ = client
            .post(&format!("{}/extract", base_url))
            .header("Content-Type", "application/json")
            .json(&json!({
                "url": "https://example.com"
            }))
            .send()
            .await;

        sleep(Duration::from_millis(500)).await;

        // Then check metrics endpoint
        let response = client
            .get(&format!("{}/metrics", base_url))
            .send()
            .await
            .map_err(|e| format!("Metrics request failed: {}", e))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read metrics response: {}", e))?;

        // Validate metrics endpoint
        assert!(
            status.is_success(),
            "Metrics endpoint should return 2xx status"
        );
        assert!(!body.is_empty(), "Metrics should not be empty");

        // Check for expected metric names (Prometheus format)
        assert!(
            body.contains("riptide") || body.contains("http_requests") || body.contains("memory"),
            "Metrics should contain expected metric names"
        );

        Ok(())
    })
    .await;

    stop_test_api_server(child).await;

    assert!(
        result.is_ok(),
        "Metrics endpoint test should pass: {:?}",
        result
    );
}

#[tokio::test]
async fn test_concurrent_extractions() {
    let (base_url, child) = start_test_api_server()
        .await
        .expect("Failed to start API server");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client");

    // Test concurrent extractions
    let mut handles = Vec::new();
    let test_urls = vec![
        "https://example.com",
        "https://www.rust-lang.org/",
        "https://github.com/",
    ];

    for url in test_urls {
        let base_url = base_url.clone();
        let client = client.clone();
        let url = url.to_string();

        let handle = tokio::spawn(async move {
            client
                .post(&format!("{}/extract", base_url))
                .header("Content-Type", "application/json")
                .json(&json!({
                    "url": url
                }))
                .send()
                .await
        });

        handles.push(handle);
    }

    // Wait for all requests to complete
    let mut success_count = 0;
    for handle in handles {
        if let Ok(Ok(response)) = handle.await {
            if response.status().is_success() {
                success_count += 1;
            }
        }
    }

    stop_test_api_server(child).await;

    // At least 2 out of 3 should succeed in concurrent scenario
    assert!(
        success_count >= 2,
        "At least 2/3 concurrent extractions should succeed, got: {}",
        success_count
    );
}

#[tokio::test]
async fn test_extraction_quality() {
    let (base_url, child) = start_test_api_server()
        .await
        .expect("Failed to start API server");

    let result = with_timeout(TestConfig::DEFAULT_TIMEOUT, async {
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/extract", base_url))
            .header("Content-Type", "application/json")
            .json(&json!({
                "url": "https://example.com"
            }))
            .send()
            .await
            .map_err(|e| format!("Extraction request failed: {}", e))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse extraction response: {}", e))?;

        let success = body.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
        assert!(success, "Extraction should be successful");

        // Validate extraction quality
        if let Some(content) = body.get("content").and_then(|v| v.as_str()) {
            assert!(!content.is_empty(), "Content should not be empty");
            assert!(
                content.len() >= 50,
                "Content should have meaningful length: {} bytes",
                content.len()
            );

            // Content should not be just HTML tags
            let non_html_chars = content
                .chars()
                .filter(|c| !c.is_whitespace() && *c != '<' && *c != '>')
                .count();
            assert!(
                non_html_chars > 30,
                "Content should contain actual text, not just HTML"
            );
        }

        // Check for metadata
        if let Some(title) = body.get("title") {
            if let Some(title_str) = title.as_str() {
                assert!(!title_str.is_empty(), "Title should not be empty if present");
            }
        }

        Ok(())
    })
    .await;

    stop_test_api_server(child).await;

    assert!(
        result.is_ok(),
        "Extraction quality test should pass: {:?}",
        result
    );
}

#[tokio::test]
async fn test_no_crashes_or_panics() {
    let (base_url, child) = start_test_api_server()
        .await
        .expect("Failed to start API server");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client");

    // Send various edge case requests
    let edge_cases = vec![
        json!({"url": ""}),
        json!({"url": "javascript:alert('xss')"}),
        json!({"url": "file:///etc/passwd"}),
        json!({"url": "https://localhost:1"}),
        json!({"url": "a".repeat(10000)}),
        json!({"url": "https://example.com", "extra": "field"}),
    ];

    for payload in edge_cases {
        let _ = client
            .post(&format!("{}/extract", base_url))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await;

        // Small delay between requests
        sleep(Duration::from_millis(100)).await;
    }

    // Check that server is still responsive
    let health_response = client
        .get(&format!("{}/healthz", base_url))
        .send()
        .await
        .expect("Server should still be responsive after edge cases");

    stop_test_api_server(child).await;

    assert!(
        health_response.status().is_success(),
        "Server should remain healthy after edge case testing"
    );
}
