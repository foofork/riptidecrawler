//! Deep Search Streaming Tests - Advanced NDJSON Testing
//!
//! Comprehensive tests for the /deepsearch/stream endpoint focusing on:
//! - Search integration with streaming
//! - Content extraction streaming
//! - Error handling in search context
//! - Performance with search + crawl operations

use futures::stream::StreamExt;
use httpmock::prelude::*;
use httpmock::Mock;
use reqwest::Client;
use serde_json::Value;
use std::time::{Duration, Instant};

/// Test framework specifically for deep search streaming
struct DeepSearchStreamingTestFramework {
    serper_mock_server: MockServer,
    content_mock_server: MockServer,
    client: Client,
    api_base_url: String,
}

impl DeepSearchStreamingTestFramework {
    fn new() -> Self {
        let serper_mock_server = MockServer::start();
        let content_mock_server = MockServer::start();
        let client = Client::new();
        let api_base_url = "http://localhost:8080".to_string();

        // Set mock Serper API URL (would need to be configurable in actual implementation)
        std::env::set_var("SERPER_API_KEY", "test_api_key_123");

        Self {
            serper_mock_server,
            content_mock_server,
            client,
            api_base_url,
        }
    }

    /// Setup mock Serper API response
    fn setup_serper_mock(&self, query: &str, results: Vec<SearchResultData>) -> Mock<'_> {
        let organic_results: Vec<Value> = results
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "title": r.title,
                    "link": r.url,
                    "snippet": r.snippet,
                    "position": r.position
                })
            })
            .collect();

        self.serper_mock_server.mock(|when, then| {
            when.method(POST)
                .path("/search")
                .header("X-API-KEY", "test_api_key_123")
                .json_body_partial(format!(r#"{{"q":"{}"}}"#, query));
            then.status(200).json_body(serde_json::json!({
                "organic": organic_results,
                "searchParameters": {
                    "q": query,
                    "gl": "us",
                    "hl": "en"
                }
            }));
        })
    }

    /// Setup mock content servers for search results
    fn setup_content_mocks(&self, urls: &[&str]) -> Vec<Mock<'_>> {
        urls.iter().map(|url| {
            let path = url.trim_start_matches(&self.content_mock_server.base_url());
            self.content_mock_server.mock(|when, then| {
                when.method(GET).path(path);
                then.status(200)
                    .header("content-type", "text/html")
                    .body(format!(
                        r#"<html>
                        <head><title>Content for {}</title></head>
                        <body>
                            <h1>Main Content</h1>
                            <p>This is the main content from {}. It contains relevant information for the search query.</p>
                            <div class="article-content">
                                <p>Additional detailed content that provides value to users searching for information.</p>
                                <ul>
                                    <li>Point 1: Relevant information</li>
                                    <li>Point 2: More relevant information</li>
                                    <li>Point 3: Additional context</li>
                                </ul>
                            </div>
                        </body>
                        </html>"#,
                        url, url
                    ));
            })
        }).collect()
    }

    /// Setup failing content mocks
    fn setup_failing_content_mocks(&self, urls: &[&str]) -> Vec<Mock<'_>> {
        urls.iter()
            .map(|url| {
                let path = url.trim_start_matches(&self.content_mock_server.base_url());
                self.content_mock_server.mock(|when, then| {
                    when.method(GET).path(path);
                    then.status(404)
                        .header("content-type", "text/html")
                        .body("<html><body><h1>404 Not Found</h1></body></html>");
                })
            })
            .collect()
    }

    /// Create deepsearch request
    fn create_request(
        &self,
        query: &str,
        limit: u32,
        include_content: bool,
        concurrency: Option<u32>,
    ) -> Value {
        let mut request = serde_json::json!({
            "query": query,
            "limit": limit,
            "include_content": include_content
        });

        if let Some(conc) = concurrency {
            request["crawl_options"] = serde_json::json!({
                "cache_mode": "disabled",
                "concurrency": conc,
                "stream": true,
                "timeout_ms": 30000
            });
        }

        request
    }

    /// Make streaming request and parse response
    async fn make_streaming_request(
        &self,
        request: Value,
    ) -> Result<DeepSearchStreamingResponse, String> {
        let start_time = Instant::now();

        let response = self
            .client
            .post(format!("{}/deepsearch/stream", self.api_base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let ttfb = start_time.elapsed();
        let status = response.status();
        let headers = response.headers().clone();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("HTTP {}: {}", status, error_text));
        }

        let mut stream = response.bytes_stream();
        let mut lines = Vec::new();
        let mut buffer = String::new();
        let mut first_line_time = None;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| format!("Stream error: {}", e))?;

            if first_line_time.is_none() {
                first_line_time = Some(start_time.elapsed());
            }

            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].to_string();
                buffer = buffer[newline_pos + 1..].to_string();

                if !line.trim().is_empty() {
                    let parsed_line = serde_json::from_str::<Value>(&line)
                        .map_err(|e| format!("Parse error for line '{}': {}", line, e))?;
                    lines.push(parsed_line);
                }
            }
        }

        Ok(DeepSearchStreamingResponse {
            ttfb,
            first_line_time: first_line_time.unwrap_or(ttfb),
            status,
            headers,
            lines,
            total_time: start_time.elapsed(),
        })
    }
}

/// Search result data for mocking
#[derive(Debug, Clone)]
struct SearchResultData {
    title: String,
    url: String,
    snippet: String,
    position: u32,
}

impl SearchResultData {
    fn new(title: &str, url: &str, snippet: &str, position: u32) -> Self {
        Self {
            title: title.to_string(),
            url: url.to_string(),
            snippet: snippet.to_string(),
            position,
        }
    }
}

/// Deep search streaming response
#[derive(Debug)]
struct DeepSearchStreamingResponse {
    ttfb: Duration,
    first_line_time: Duration,
    #[allow(dead_code)]
    status: reqwest::StatusCode,
    headers: reqwest::header::HeaderMap,
    lines: Vec<Value>,
    total_time: Duration,
}

impl DeepSearchStreamingResponse {
    fn stream_metadata(&self) -> Option<&Value> {
        self.lines
            .iter()
            .find(|line| line.get("stream_type").is_some())
    }

    fn search_metadata(&self) -> Option<&Value> {
        self.lines
            .iter()
            .find(|line| line.get("query").is_some() && line.get("urls_found").is_some())
    }

    fn search_results(&self) -> Vec<&Value> {
        self.lines
            .iter()
            .filter(|line| line.get("search_result").is_some())
            .collect()
    }

    fn summary(&self) -> Option<&Value> {
        self.lines
            .iter()
            .find(|line| line.get("query").is_some() && line.get("status").is_some())
    }

    fn validate_headers(&self) -> Result<(), String> {
        let content_type = self
            .headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| "Missing content-type header".to_string())?;

        if content_type != "application/x-ndjson" {
            return Err(format!(
                "Expected content-type 'application/x-ndjson', got '{}'",
                content_type
            ));
        }

        if self.headers.get("transfer-encoding").is_none() {
            return Err("Missing transfer-encoding header".to_string());
        }

        Ok(())
    }
}

/// Test basic deep search streaming functionality
#[tokio::test]
async fn test_deepsearch_basic_streaming() {
    let framework = DeepSearchStreamingTestFramework::new();

    // Setup search results
    let search_results = vec![
        SearchResultData::new(
            "Test Article 1",
            &format!("{}/article1", framework.content_mock_server.base_url()),
            "This is a test article about streaming",
            1,
        ),
        SearchResultData::new(
            "Test Article 2",
            &format!("{}/article2", framework.content_mock_server.base_url()),
            "Another test article with relevant content",
            2,
        ),
    ];

    // Setup mock servers - must keep mocks alive for duration of test
    let _serper_mock_guard =
        framework.setup_serper_mock("streaming technology", search_results.clone());
    let _content_mocks_guard = framework.setup_content_mocks(&[
        &format!("{}/article1", framework.content_mock_server.base_url()),
        &format!("{}/article2", framework.content_mock_server.base_url()),
    ]);

    let request = framework.create_request("streaming technology", 2, true, Some(2));
    let response = framework
        .make_streaming_request(request)
        .await
        .expect("Basic deep search streaming should succeed");

    // Validate response structure
    response
        .validate_headers()
        .expect("Headers should be valid");
    assert!(!response.lines.is_empty(), "Should have response lines");

    // Check stream metadata
    let stream_metadata = response
        .stream_metadata()
        .expect("Should have stream metadata");
    assert_eq!(stream_metadata["stream_type"], "deepsearch");
    assert!(stream_metadata["request_id"].is_string());

    // Check search metadata
    let search_metadata = response
        .search_metadata()
        .expect("Should have search metadata");
    assert_eq!(search_metadata["query"], "streaming technology");
    assert_eq!(search_metadata["urls_found"], 2);
    assert!(search_metadata["search_time_ms"].is_number());

    // Check search results
    let search_results = response.search_results();
    assert_eq!(search_results.len(), 2, "Should have 2 search results");

    for result in search_results.iter() {
        let search_result = &result["search_result"];
        assert!(search_result["url"].is_string());
        assert!(search_result["rank"].is_number());
        assert!(search_result["search_title"].is_string());
        assert!(search_result["search_snippet"].is_string());

        // Since include_content=true, should have crawl results
        let crawl_result = result["crawl_result"]
            .as_object()
            .expect("Should have crawl result when include_content=true");
        assert!(crawl_result["status"].is_number());
        assert!(crawl_result["gate_decision"].is_string());
        assert!(crawl_result["quality_score"].is_number());
        assert!(crawl_result["document"].is_object());
    }

    // Check summary
    let summary = response.summary().expect("Should have summary");
    assert_eq!(summary["query"], "streaming technology");
    assert_eq!(summary["total_urls_found"], 2);
    assert_eq!(summary["status"], "completed");
    assert!(summary["total_processing_time_ms"].is_number());
}

/// Test deep search without content extraction
#[tokio::test]
async fn test_deepsearch_without_content_extraction() {
    let framework = DeepSearchStreamingTestFramework::new();

    let search_results = vec![SearchResultData::new(
        "Fast Result",
        "https://example.com/fast",
        "Quick result without crawling",
        1,
    )];

    // Setup mock server - must keep mock alive for duration of test
    let _serper_mock_guard = framework.setup_serper_mock("fast search", search_results);

    let request = framework.create_request("fast search", 1, false, None);
    let response = framework
        .make_streaming_request(request)
        .await
        .expect("Search without content should succeed");

    // Should complete quickly since no crawling
    assert!(
        response.total_time.as_millis() < 2000,
        "Search without content should be fast: {}ms",
        response.total_time.as_millis()
    );

    let search_results = response.search_results();
    assert_eq!(search_results.len(), 1);

    // Should not have crawl results
    let result = &search_results[0];
    assert!(
        result["crawl_result"].is_null() || result.get("crawl_result").is_none(),
        "Should not have crawl result when include_content=false"
    );

    // But should have search data
    let search_result = &result["search_result"];
    assert_eq!(search_result["url"], "https://example.com/fast");
    assert_eq!(search_result["search_title"], "Fast Result");
}

/// Test TTFB performance for deepsearch streaming
#[tokio::test]
async fn test_deepsearch_ttfb_performance() {
    let framework = DeepSearchStreamingTestFramework::new();

    let search_results = vec![SearchResultData::new(
        "Cached Content",
        &format!("{}/cached", framework.content_mock_server.base_url()),
        "This should be fast",
        1,
    )];

    // Setup mock servers - must keep mocks alive for duration of test
    let _serper_mock_guard = framework.setup_serper_mock("cached query", search_results);
    let _content_mocks_guard = framework.setup_content_mocks(&[&format!(
        "{}/cached",
        framework.content_mock_server.base_url()
    )]);

    // First request to potentially warm cache
    let request = framework.create_request("cached query", 1, true, Some(1));
    let _ = framework.make_streaming_request(request.clone()).await;

    // Second request should be faster
    let response = framework
        .make_streaming_request(request)
        .await
        .expect("Cached deep search should succeed");

    // TTFB should be reasonable for deepsearch (more lenient than crawl due to external API)
    assert!(
        response.ttfb.as_millis() < 2000,
        "Deep search TTFB {}ms should be reasonable",
        response.ttfb.as_millis()
    );

    // First result should arrive quickly after search completes
    assert!(
        response.first_line_time.as_millis() < 3000,
        "First line should arrive within 3s for deep search"
    );
}

/// Test error handling in deep search streaming
#[tokio::test]
async fn test_deepsearch_error_handling() {
    let framework = DeepSearchStreamingTestFramework::new();

    // Mix of working and failing URLs
    let search_results = vec![
        SearchResultData::new(
            "Working Article",
            &format!("{}/working", framework.content_mock_server.base_url()),
            "This article works fine",
            1,
        ),
        SearchResultData::new(
            "Broken Article",
            &format!("{}/broken", framework.content_mock_server.base_url()),
            "This article will fail",
            2,
        ),
    ];

    // Setup mock servers - must keep mocks alive for duration of test
    let _serper_mock_guard = framework.setup_serper_mock("mixed results", search_results);
    let _working_mock_guard = framework.setup_content_mocks(&[&format!(
        "{}/working",
        framework.content_mock_server.base_url()
    )]);
    let _failing_mock_guard = framework.setup_failing_content_mocks(&[&format!(
        "{}/broken",
        framework.content_mock_server.base_url()
    )]);

    let request = framework.create_request("mixed results", 2, true, Some(2));
    let response = framework
        .make_streaming_request(request)
        .await
        .expect("Mixed results should succeed overall");

    let search_results = response.search_results();
    assert_eq!(search_results.len(), 2);

    let mut successful_crawls = 0;
    let mut failed_crawls = 0;

    for result in search_results {
        // All should have search results
        assert!(result["search_result"].is_object());

        // Check crawl results
        if let Some(crawl_result) = result.get("crawl_result") {
            if crawl_result.is_null() {
                failed_crawls += 1;
            } else if crawl_result["error"].is_object() {
                failed_crawls += 1;
                // Validate error structure
                let error = &crawl_result["error"];
                assert!(error["error_type"].is_string());
                assert!(error["message"].is_string());
                assert!(error["retryable"].is_boolean());
            } else {
                successful_crawls += 1;
                assert!(crawl_result["document"].is_object());
            }
        }
    }

    assert_eq!(successful_crawls, 1, "Should have 1 successful crawl");
    assert_eq!(failed_crawls, 1, "Should have 1 failed crawl");
}

/// Test missing API key error handling
#[tokio::test]
async fn test_deepsearch_missing_api_key() {
    let framework = DeepSearchStreamingTestFramework::new();

    // Remove API key
    std::env::remove_var("SERPER_API_KEY");

    let request = framework.create_request("test query", 1, false, None);
    let result = framework.make_streaming_request(request).await;

    // Should fail with meaningful error
    assert!(result.is_err(), "Should fail without API key");
    let error = result.unwrap_err();
    assert!(
        error.contains("SERPER_API_KEY") || error.contains("API") || error.contains("key"),
        "Error should mention API key issue: {}",
        error
    );

    // Restore API key for other tests
    std::env::set_var("SERPER_API_KEY", "test_api_key_123");
}

/// Test large result set handling
#[tokio::test]
async fn test_deepsearch_large_result_set() {
    let framework = DeepSearchStreamingTestFramework::new();

    // Create many search results
    let search_results: Vec<SearchResultData> = (1..=10)
        .map(|i| {
            SearchResultData::new(
                &format!("Article {}", i),
                &format!("{}/article{}", framework.content_mock_server.base_url(), i),
                &format!("Content snippet for article {}", i),
                i as u32,
            )
        })
        .collect();

    let _serper_mock = framework.setup_serper_mock("large query", search_results);

    // Setup some working and some failing content
    let working_urls: Vec<String> = (1..=5)
        .map(|i| format!("{}/article{}", framework.content_mock_server.base_url(), i))
        .collect();
    let failing_urls: Vec<String> = (6..=10)
        .map(|i| format!("{}/article{}", framework.content_mock_server.base_url(), i))
        .collect();

    let working_refs: Vec<&str> = working_urls.iter().map(|s| s.as_str()).collect();
    let failing_refs: Vec<&str> = failing_urls.iter().map(|s| s.as_str()).collect();

    // Setup mock servers - must keep mocks alive for duration of test
    let _working_mocks_guard = framework.setup_content_mocks(&working_refs);
    let _failing_mocks_guard = framework.setup_failing_content_mocks(&failing_refs);

    let request = framework.create_request("large query", 10, true, Some(5));
    let response = framework
        .make_streaming_request(request)
        .await
        .expect("Large result set should succeed");

    // Should handle all results
    let search_results = response.search_results();
    assert_eq!(search_results.len(), 10, "Should stream all search results");

    // Results should arrive as they complete (streaming benefit)
    // Total time should be reasonable for parallel processing
    assert!(
        response.total_time.as_millis() < 10000,
        "Large result set should complete in reasonable time with streaming"
    );

    // Check search metadata
    let search_metadata = response.search_metadata().unwrap();
    assert_eq!(search_metadata["urls_found"], 10);

    // Summary should reflect mixed results
    let summary = response.summary().unwrap();
    assert_eq!(summary["total_urls_found"], 10);
}

/// Test streaming with search API failure
#[tokio::test]
async fn test_deepsearch_search_api_failure() {
    let framework = DeepSearchStreamingTestFramework::new();

    // Mock failing search API
    let _failing_mock = framework.serper_mock_server.mock(|when, then| {
        when.method(POST)
            .path("/search")
            .header("X-API-KEY", "test_api_key_123");
        then.status(500).body("Internal Server Error");
    });

    let request = framework.create_request("failing query", 5, true, Some(2));
    let result = framework.make_streaming_request(request).await;

    // Should fail gracefully
    assert!(result.is_err(), "Should fail when search API fails");
    let error = result.unwrap_err();
    assert!(
        error.contains("500") || error.contains("search") || error.contains("API"),
        "Error should indicate search API failure: {}",
        error
    );
}

/// Test concurrent deep search sessions
#[tokio::test]
async fn test_concurrent_deepsearch_sessions() {
    let framework = DeepSearchStreamingTestFramework::new();

    // Setup different search results for each session
    let search_results1 = vec![
        SearchResultData::new("Result 1A", "https://example.com/1a", "Content 1A", 1),
        SearchResultData::new("Result 1B", "https://example.com/1b", "Content 1B", 2),
    ];
    let search_results2 = vec![SearchResultData::new(
        "Result 2A",
        "https://example.com/2a",
        "Content 2A",
        1,
    )];

    // Setup mock servers - must keep mocks alive for duration of test
    let _serper_mock1_guard = framework.setup_serper_mock("query one", search_results1);
    let _serper_mock2_guard = framework.setup_serper_mock("query two", search_results2);

    let request1 = framework.create_request("query one", 2, false, None);
    let request2 = framework.create_request("query two", 1, false, None);

    // Run concurrently
    let (result1, result2) = tokio::join!(
        framework.make_streaming_request(request1),
        framework.make_streaming_request(request2)
    );

    let response1 = result1.expect("First session should succeed");
    let response2 = result2.expect("Second session should succeed");

    // Validate both sessions
    assert_eq!(response1.search_results().len(), 2);
    assert_eq!(response2.search_results().len(), 1);

    // Check different request IDs
    let req_id1 = response1.stream_metadata().unwrap()["request_id"]
        .as_str()
        .unwrap();
    let req_id2 = response2.stream_metadata().unwrap()["request_id"]
        .as_str()
        .unwrap();
    assert_ne!(
        req_id1, req_id2,
        "Concurrent sessions should have different request IDs"
    );

    // Both should complete in reasonable time
    assert!(response1.total_time.as_millis() < 5000);
    assert!(response2.total_time.as_millis() < 5000);
}

/// Test rate limiting and backpressure in deep search
#[tokio::test]
async fn test_deepsearch_rate_limiting() {
    let framework = DeepSearchStreamingTestFramework::new();

    // Create large number of search results to test limits
    let search_results: Vec<SearchResultData> = (1..=20)
        .map(|i| {
            SearchResultData::new(
                &format!("Rate Test {}", i),
                &format!("{}/rate{}", framework.content_mock_server.base_url(), i),
                &format!("Rate limiting test content {}", i),
                i as u32,
            )
        })
        .collect();

    // Setup mock server - must keep mock alive for duration of test
    let _serper_mock_guard = framework.setup_serper_mock("rate limit test", search_results);

    // Setup content with some fast, some slow responses
    let urls: Vec<String> = (1..=20)
        .map(|i| format!("{}/rate{}", framework.content_mock_server.base_url(), i))
        .collect();

    for (i, url) in urls.iter().enumerate() {
        let path = url.trim_start_matches(&framework.content_mock_server.base_url());
        let delay = if i % 3 == 0 { 500 } else { 100 }; // Some slower responses

        let _mock = framework.content_mock_server.mock(|when, then| {
            when.method(GET).path(path);
            then.status(200)
                .delay(Duration::from_millis(delay))
                .header("content-type", "text/html")
                .body(format!("<html><body><h1>Content {}</h1></body></html>", i));
        });
    }

    let request = framework.create_request("rate limit test", 20, true, Some(5)); // Limited concurrency
    let response = framework
        .make_streaming_request(request)
        .await
        .expect("Rate limiting test should succeed");

    // Should handle all results despite rate limiting
    let search_results = response.search_results();
    assert_eq!(
        search_results.len(),
        20,
        "All results should be processed despite rate limiting"
    );

    // Processing should respect concurrency limits (not too fast)
    // With 20 URLs and concurrency 5, should take reasonable time
    assert!(
        response.total_time.as_millis() > 500, // At least some serialization
        "Should show evidence of rate limiting/concurrency control"
    );
    assert!(
        response.total_time.as_millis() < 15000, // But not too slow
        "Should still complete in reasonable time"
    );

    // Summary should be accurate
    let summary = response.summary().unwrap();
    assert_eq!(summary["total_urls_found"], 20);
}
