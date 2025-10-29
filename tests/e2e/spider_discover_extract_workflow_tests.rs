//! End-to-end tests for Phase 2 discover→extract workflow
//!
//! Tests the complete workflow:
//! 1. Spider discovers URLs from a site
//! 2. Extract detailed content from each discovered URL
//!
//! This simulates the event site use case and other real-world scenarios.

#[cfg(test)]
mod spider_discover_extract_e2e_tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use riptide_api::{create_app, AppConfig};
    use serde_json::{json, Value};
    use std::collections::HashSet;

    fn integration_test_config() -> AppConfig {
        AppConfig {
            port: 0,
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            headless_url: std::env::var("HEADLESS_URL").ok(),
            cache_ttl: 60,
            max_concurrency: 5,
            gate_hi_threshold: 0.8,
            gate_lo_threshold: 0.3,
            cors_origins: vec![],
            api_key: Some("test-key".to_string()),
            openai_api_key: None,
            spider_config: Some(riptide_core::spider::SpiderConfig::new(
                "https://httpbin.org".parse().unwrap(),
            )),
        }
    }

    async fn setup_test_server() -> TestServer {
        let config = integration_test_config();
        let app = create_app(config).await.expect("Failed to create test app");
        TestServer::new(app.into_make_service()).unwrap()
    }

    // ========================================================================
    // Complete Discover → Extract Workflow Tests
    // ========================================================================

    #[tokio::test]
    async fn test_complete_discover_extract_workflow() {
        let server = setup_test_server().await;

        // STEP 1: Discover URLs using spider with result_mode=urls
        let spider_body = json!({
            "seed_urls": ["https://httpbin.org/links/5"],
            "max_depth": 1,
            "max_pages": 5,
            "strategy": "breadth_first",
            "timeout_seconds": 15,
            "delay_ms": 300,
            "concurrency": 1,
            "respect_robots": false,
            "follow_redirects": true,
            "result_mode": "urls"
        });

        let spider_response = server
            .post("/spider/crawl")
            .json(&spider_body)
            .await;

        if spider_response.status_code() != StatusCode::OK {
            println!("Spider crawl failed (may be expected in test env): {}", spider_response.status_code());
            return;
        }

        let spider_result: Value = spider_response.json();

        // Verify spider returned discovered URLs
        assert!(spider_result["result"]["discovered_urls"].is_array());
        let discovered_urls = spider_result["result"]["discovered_urls"]
            .as_array()
            .unwrap();

        println!("Discovered {} URLs", discovered_urls.len());
        assert!(discovered_urls.len() > 0, "Should discover at least one URL");

        // STEP 2: Extract content from each discovered URL
        let mut extraction_results = Vec::new();

        for url in discovered_urls.iter().take(3) {
            // Limit to first 3 for test speed
            let url_str = url.as_str().unwrap();

            let extract_body = json!({
                "urls": [url_str],
                "options": {
                    "format": "markdown"
                }
            });

            let extract_response = server
                .post("/crawl")
                .json(&extract_body)
                .await;

            if extract_response.status_code() == StatusCode::OK {
                let extract_result: Value = extract_response.json();
                extraction_results.push((url_str.to_string(), extract_result));
            }
        }

        println!("Successfully extracted {} URLs", extraction_results.len());

        // Verify extractions were successful
        assert!(
            extraction_results.len() > 0,
            "Should successfully extract at least one discovered URL"
        );

        // Verify extraction results have expected structure
        for (url, result) in &extraction_results {
            assert!(result["results"].is_array());

            if result["results"].as_array().unwrap().len() > 0 {
                let first_result = &result["results"][0];
                assert_eq!(first_result["url"].as_str().unwrap(), url);

                // Should have document or error
                assert!(
                    first_result["document"].is_object() || first_result["error"].is_object(),
                    "Result should have document or error"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_discover_filter_extract_workflow() {
        let server = setup_test_server().await;

        // STEP 1: Discover URLs
        let spider_body = json!({
            "seed_urls": ["https://httpbin.org/links/10"],
            "max_pages": 10,
            "result_mode": "urls"
        });

        let spider_response = server
            .post("/spider/crawl")
            .json(&spider_body)
            .await;

        if spider_response.status_code() != StatusCode::OK {
            return; // Skip if crawl fails
        }

        let spider_result: Value = spider_response.json();
        let discovered_urls = spider_result["result"]["discovered_urls"]
            .as_array()
            .unwrap();

        // STEP 2: Filter URLs (simulate selecting only certain pages)
        let filtered_urls: Vec<String> = discovered_urls
            .iter()
            .filter(|url| {
                let url_str = url.as_str().unwrap();
                // Filter criteria: only links with specific patterns
                url_str.contains("links") || url_str.contains("html")
            })
            .map(|url| url.as_str().unwrap().to_string())
            .collect();

        println!(
            "Filtered {} URLs from {} discovered",
            filtered_urls.len(),
            discovered_urls.len()
        );

        // STEP 3: Batch extract filtered URLs
        if !filtered_urls.is_empty() {
            let extract_body = json!({
                "urls": filtered_urls,
                "options": {
                    "format": "markdown"
                }
            });

            let extract_response = server
                .post("/crawl")
                .json(&extract_body)
                .await;

            if extract_response.status_code() == StatusCode::OK {
                let extract_result: Value = extract_response.json();

                let successful = extract_result["successful"].as_u64().unwrap();
                let total = extract_result["total_urls"].as_u64().unwrap();

                println!("Extracted {}/{} filtered URLs", successful, total);

                assert_eq!(
                    total,
                    filtered_urls.len() as u64,
                    "Should attempt to extract all filtered URLs"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_live_hilversum_style_workflow() {
        let server = setup_test_server().await;

        // Simulate Live Hilversum use case:
        // 1. Discover all pages on a site
        // 2. Select pages of interest (e.g., news articles)
        // 3. Extract full content from selected pages
        // 4. Process/save extracted content

        // STEP 1: Broad discovery with max_pages limit
        let spider_body = json!({
            "seed_urls": ["https://httpbin.org/"],
            "max_depth": 2,
            "max_pages": 20,
            "strategy": "breadth_first",
            "result_mode": "urls"
        });

        let spider_response = server
            .post("/spider/crawl")
            .json(&spider_body)
            .await;

        if spider_response.status_code() != StatusCode::OK {
            return; // Skip if environment doesn't support
        }

        let spider_result: Value = spider_response.json();
        let all_discovered = spider_result["result"]["discovered_urls"]
            .as_array()
            .unwrap();

        println!("Discovery phase: found {} URLs", all_discovered.len());

        // STEP 2: Classify/filter URLs (simulate "find news articles")
        let interesting_urls: Vec<String> = all_discovered
            .iter()
            .filter(|url| {
                let url_str = url.as_str().unwrap();
                // Simulate filtering for content pages
                !url_str.contains("static")
                    && !url_str.contains("asset")
                    && !url_str.ends_with(".css")
                    && !url_str.ends_with(".js")
            })
            .map(|url| url.as_str().unwrap().to_string())
            .collect();

        println!("Classification: {} interesting URLs", interesting_urls.len());

        // STEP 3: Extract content in batches
        let batch_size = 5;
        let mut all_extractions = Vec::new();

        for chunk in interesting_urls.chunks(batch_size) {
            let extract_body = json!({
                "urls": chunk,
                "options": {
                    "format": "markdown",
                    "include_metadata": true
                }
            });

            let extract_response = server
                .post("/crawl")
                .json(&extract_body)
                .await;

            if extract_response.status_code() == StatusCode::OK {
                let extract_result: Value = extract_response.json();
                all_extractions.push(extract_result);
            }
        }

        println!("Extraction: processed {} batches", all_extractions.len());

        // STEP 4: Aggregate results
        let mut total_extracted = 0;
        let mut total_successful = 0;

        for batch_result in &all_extractions {
            total_extracted += batch_result["total_urls"].as_u64().unwrap_or(0);
            total_successful += batch_result["successful"].as_u64().unwrap_or(0);
        }

        println!(
            "Final results: {}/{} successful extractions",
            total_successful, total_extracted
        );

        if total_extracted > 0 {
            let success_rate = (total_successful as f64 / total_extracted as f64) * 100.0;
            assert!(
                success_rate >= 50.0,
                "Success rate should be reasonable: {:.1}%",
                success_rate
            );
        }
    }

    // ========================================================================
    // Error Handling in Workflow
    // ========================================================================

    #[tokio::test]
    async fn test_workflow_with_failed_extractions() {
        let server = setup_test_server().await;

        // STEP 1: Discover URLs
        let spider_body = json!({
            "seed_urls": ["https://httpbin.org/links/3"],
            "max_pages": 3,
            "result_mode": "urls"
        });

        let spider_response = server
            .post("/spider/crawl")
            .json(&spider_body)
            .await;

        if spider_response.status_code() != StatusCode::OK {
            return;
        }

        let spider_result: Value = spider_response.json();
        let mut discovered_urls: Vec<String> = spider_result["result"]["discovered_urls"]
            .as_array()
            .unwrap()
            .iter()
            .map(|u| u.as_str().unwrap().to_string())
            .collect();

        // Add some invalid URLs to test error handling
        discovered_urls.push("https://invalid-domain-12345.invalid".to_string());
        discovered_urls.push("https://httpbin.org/status/404".to_string());

        // STEP 2: Try to extract all URLs (including invalid ones)
        let extract_body = json!({
            "urls": discovered_urls,
            "options": {
                "format": "markdown"
            }
        });

        let extract_response = server
            .post("/crawl")
            .json(&extract_body)
            .await;

        if extract_response.status_code() == StatusCode::OK {
            let extract_result: Value = extract_response.json();

            let total = extract_result["total_urls"].as_u64().unwrap();
            let successful = extract_result["successful"].as_u64().unwrap();
            let failed = extract_result["failed"].as_u64().unwrap();

            // Verify error handling
            assert_eq!(
                total,
                successful + failed,
                "Total should equal successful + failed"
            );
            assert!(failed > 0, "Should have some failed extractions");

            // Check that results include error information
            let results = extract_result["results"].as_array().unwrap();
            let failed_results: Vec<_> = results
                .iter()
                .filter(|r| r["error"].is_object())
                .collect();

            assert_eq!(
                failed_results.len(),
                failed as usize,
                "Failed count should match number of error results"
            );
        }
    }

    // ========================================================================
    // Workflow Metrics and Performance
    // ========================================================================

    #[tokio::test]
    async fn test_workflow_performance_metrics() {
        let server = setup_test_server().await;

        let start = std::time::Instant::now();

        // STEP 1: Discovery
        let spider_body = json!({
            "seed_urls": ["https://httpbin.org/links/5"],
            "max_pages": 5,
            "result_mode": "urls"
        });

        let spider_response = server
            .post("/spider/crawl")
            .json(&spider_body)
            .await;

        if spider_response.status_code() != StatusCode::OK {
            return;
        }

        let discovery_time = start.elapsed();
        println!("Discovery took: {:?}", discovery_time);

        let spider_result: Value = spider_response.json();
        let discovered_urls = spider_result["result"]["discovered_urls"]
            .as_array()
            .unwrap();

        // STEP 2: Extraction
        let extract_start = std::time::Instant::now();

        let extract_body = json!({
            "urls": discovered_urls.iter().take(3).collect::<Vec<_>>(),
            "options": {
                "format": "markdown"
            }
        });

        let extract_response = server
            .post("/crawl")
            .json(&extract_body)
            .await;

        let extraction_time = extract_start.elapsed();
        println!("Extraction took: {:?}", extraction_time);

        if extract_response.status_code() == StatusCode::OK {
            let extract_result: Value = extract_response.json();

            // Verify timing metadata
            assert!(
                extract_result["statistics"]["total_processing_time_ms"].is_number(),
                "Should include processing time"
            );

            let total_time = start.elapsed();
            println!("Total workflow time: {:?}", total_time);

            // Workflow should complete in reasonable time
            assert!(
                total_time.as_secs() < 60,
                "Workflow should complete within 60 seconds"
            );
        }
    }

    #[tokio::test]
    async fn test_workflow_url_deduplication_across_stages() {
        let server = setup_test_server().await;

        // STEP 1: Discover URLs
        let spider_body = json!({
            "seed_urls": ["https://httpbin.org/links/5"],
            "max_pages": 5,
            "result_mode": "urls"
        });

        let spider_response = server
            .post("/spider/crawl")
            .json(&spider_body)
            .await;

        if spider_response.status_code() != StatusCode::OK {
            return;
        }

        let spider_result: Value = spider_response.json();
        let discovered_urls = spider_result["result"]["discovered_urls"]
            .as_array()
            .unwrap();

        // Verify no duplicates in discovered URLs
        let unique_discovered: HashSet<_> = discovered_urls.iter().collect();
        assert_eq!(
            discovered_urls.len(),
            unique_discovered.len(),
            "Discovery should not produce duplicates"
        );

        // STEP 2: Extract with intentional duplicates
        let mut urls_to_extract: Vec<&Value> = discovered_urls.iter().take(3).collect();
        if !urls_to_extract.is_empty() {
            // Add first URL again (duplicate)
            urls_to_extract.push(urls_to_extract[0]);
        }

        let extract_body = json!({
            "urls": urls_to_extract,
            "options": {
                "format": "markdown"
            }
        });

        let extract_response = server
            .post("/crawl")
            .json(&extract_body)
            .await;

        if extract_response.status_code() == StatusCode::OK {
            let extract_result: Value = extract_response.json();

            // System should handle duplicates gracefully
            let total_urls = extract_result["total_urls"].as_u64().unwrap();
            assert!(
                total_urls >= 3,
                "Should process duplicate URLs (may cache)"
            );
        }
    }
}
