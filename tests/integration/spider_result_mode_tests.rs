//! Integration tests for Phase 2 spider result_mode feature
//!
//! Tests:
//! - Backward compatibility with result_mode=stats
//! - New result_mode=urls functionality
//! - discovered_urls accumulation during crawl
//! - URL deduplication
//! - Max pages constraint with URL collection
//! - Different crawl strategies with URLs

#[cfg(test)]
mod spider_result_mode_integration_tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use riptide_api::models::SpiderCrawlBody;
    use riptide_api::{create_app, AppConfig};
    use serde_json::{json, Value};
    use std::collections::HashSet;

    /// Helper to create integration test configuration
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
    // Backward Compatibility Tests
    // ========================================================================

    #[tokio::test]
    async fn test_backward_compatibility_no_result_mode() {
        let server = setup_test_server().await;

        let crawl_body = SpiderCrawlBody {
            seed_urls: vec!["https://httpbin.org/html".to_string()],
            max_depth: Some(1),
            max_pages: Some(3),
            strategy: Some("breadth_first".to_string()),
            timeout_seconds: Some(10),
            delay_ms: Some(500),
            concurrency: Some(1),
            respect_robots: Some(false),
            follow_redirects: Some(true),
        };

        let response = server.post("/spider/crawl").json(&crawl_body).await;

        if response.status_code() == StatusCode::OK {
            let body: Value = response.json();

            // Should have standard result fields
            assert!(body["result"].is_object());
            assert!(body["result"]["pages_crawled"].is_number());
            assert!(body["result"]["pages_failed"].is_number());
            assert!(body["result"]["duration_seconds"].is_number());
            assert!(body["result"]["stop_reason"].is_string());

            // Should NOT have discovered_urls (backward compatible)
            assert!(
                body["result"]["discovered_urls"].is_null()
                    || !body["result"].as_object().unwrap().contains_key("discovered_urls"),
                "Default mode should not include discovered_urls"
            );
        }
    }

    #[tokio::test]
    async fn test_explicit_result_mode_stats() {
        let server = setup_test_server().await;

        let mut crawl_body = json!({
            "seed_urls": ["https://httpbin.org/html"],
            "max_depth": 1,
            "max_pages": 3,
            "strategy": "breadth_first",
            "timeout_seconds": 10,
            "delay_ms": 500,
            "concurrency": 1,
            "respect_robots": false,
            "follow_redirects": true,
            "result_mode": "stats"
        });

        let response = server
            .post("/spider/crawl")
            .json(&crawl_body)
            .await;

        if response.status_code() == StatusCode::OK {
            let body: Value = response.json();

            // Should have stats but NOT discovered_urls
            assert!(body["result"]["pages_crawled"].is_number());
            assert!(
                !body["result"].as_object().unwrap().contains_key("discovered_urls"),
                "Stats mode should not include discovered_urls"
            );
        }
    }

    // ========================================================================
    // Result Mode URLs Tests
    // ========================================================================

    #[tokio::test]
    async fn test_result_mode_urls_basic() {
        let server = setup_test_server().await;

        let crawl_body = json!({
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

        let response = server
            .post("/spider/crawl")
            .json(&crawl_body)
            .await;

        if response.status_code() == StatusCode::OK {
            let body: Value = response.json();

            // Should have all standard fields
            assert!(body["result"]["pages_crawled"].is_number());
            assert!(body["result"]["pages_failed"].is_number());
            assert!(body["result"]["duration_seconds"].is_number());

            // MUST have discovered_urls array
            assert!(
                body["result"]["discovered_urls"].is_array(),
                "URLs mode must include discovered_urls array"
            );

            let urls = body["result"]["discovered_urls"].as_array().unwrap();
            assert!(
                urls.len() > 0,
                "Should discover at least one URL from httpbin.org/links/5"
            );

            // Verify URLs are strings
            for url in urls {
                assert!(url.is_string(), "Each URL should be a string");
                let url_str = url.as_str().unwrap();
                assert!(
                    url_str.starts_with("http://") || url_str.starts_with("https://"),
                    "URL should have valid protocol: {}",
                    url_str
                );
            }
        }
    }

    #[tokio::test]
    async fn test_discovered_urls_accumulation() {
        let server = setup_test_server().await;

        let crawl_body = json!({
            "seed_urls": ["https://httpbin.org/links/10/0"],
            "max_depth": 2,
            "max_pages": 10,
            "strategy": "breadth_first",
            "timeout_seconds": 20,
            "delay_ms": 200,
            "concurrency": 2,
            "respect_robots": false,
            "follow_redirects": true,
            "result_mode": "urls"
        });

        let response = server
            .post("/spider/crawl")
            .json(&crawl_body)
            .await;

        if response.status_code() == StatusCode::OK {
            let body: Value = response.json();
            let urls = body["result"]["discovered_urls"].as_array().unwrap();

            // Should accumulate URLs during the crawl
            let pages_crawled = body["result"]["pages_crawled"].as_u64().unwrap();

            assert!(
                urls.len() as u64 <= pages_crawled || urls.len() <= 10,
                "Discovered URLs ({}) should not exceed pages crawled ({}) or max_pages (10)",
                urls.len(),
                pages_crawled
            );

            // Verify no duplicates
            let unique_urls: HashSet<_> = urls.iter().collect();
            assert_eq!(
                urls.len(),
                unique_urls.len(),
                "discovered_urls should not contain duplicates"
            );
        }
    }

    #[tokio::test]
    async fn test_max_pages_constraint_with_urls() {
        let server = setup_test_server().await;

        let max_pages = 5;
        let crawl_body = json!({
            "seed_urls": ["https://httpbin.org/links/20"],
            "max_depth": 2,
            "max_pages": max_pages,
            "strategy": "breadth_first",
            "timeout_seconds": 15,
            "delay_ms": 200,
            "concurrency": 1,
            "respect_robots": false,
            "follow_redirects": true,
            "result_mode": "urls"
        });

        let response = server
            .post("/spider/crawl")
            .json(&crawl_body)
            .await;

        if response.status_code() == StatusCode::OK {
            let body: Value = response.json();
            let urls = body["result"]["discovered_urls"].as_array().unwrap();
            let pages_crawled = body["result"]["pages_crawled"].as_u64().unwrap();

            // Should respect max_pages constraint
            assert!(
                pages_crawled <= max_pages,
                "pages_crawled ({}) should not exceed max_pages ({})",
                pages_crawled,
                max_pages
            );

            // discovered_urls might be <= max_pages (some pages may not have links)
            assert!(
                urls.len() <= max_pages as usize,
                "discovered_urls count ({}) should not exceed max_pages ({})",
                urls.len(),
                max_pages
            );

            // Stop reason should indicate max_pages reached
            let stop_reason = body["result"]["stop_reason"].as_str().unwrap();
            assert!(
                stop_reason.contains("pages") || stop_reason.contains("timeout"),
                "Stop reason should mention pages or timeout: {}",
                stop_reason
            );
        }
    }

    #[tokio::test]
    async fn test_url_deduplication() {
        let server = setup_test_server().await;

        // Use seed URLs that might lead to the same pages
        let crawl_body = json!({
            "seed_urls": [
                "https://httpbin.org/html",
                "https://httpbin.org/html",  // Exact duplicate
                "https://httpbin.org/html/", // Trailing slash
            ],
            "max_depth": 1,
            "max_pages": 5,
            "strategy": "breadth_first",
            "timeout_seconds": 10,
            "delay_ms": 300,
            "concurrency": 1,
            "respect_robots": false,
            "follow_redirects": true,
            "result_mode": "urls"
        });

        let response = server
            .post("/spider/crawl")
            .json(&crawl_body)
            .await;

        if response.status_code() == StatusCode::OK {
            let body: Value = response.json();
            let urls = body["result"]["discovered_urls"].as_array().unwrap();

            // Verify no duplicates in discovered URLs
            let unique_urls: HashSet<_> = urls.iter().collect();
            assert_eq!(
                urls.len(),
                unique_urls.len(),
                "Should deduplicate discovered URLs"
            );
        }
    }

    // ========================================================================
    // Strategy Tests with URLs
    // ========================================================================

    #[tokio::test]
    async fn test_breadth_first_strategy_with_urls() {
        let server = setup_test_server().await;

        let crawl_body = json!({
            "seed_urls": ["https://httpbin.org/links/3"],
            "max_depth": 2,
            "max_pages": 5,
            "strategy": "breadth_first",
            "timeout_seconds": 15,
            "delay_ms": 300,
            "concurrency": 1,
            "respect_robots": false,
            "follow_redirects": true,
            "result_mode": "urls"
        });

        let response = server
            .post("/spider/crawl")
            .json(&crawl_body)
            .await;

        if response.status_code() == StatusCode::OK {
            let body: Value = response.json();
            assert!(body["result"]["discovered_urls"].is_array());
        }
    }

    #[tokio::test]
    async fn test_depth_first_strategy_with_urls() {
        let server = setup_test_server().await;

        let crawl_body = json!({
            "seed_urls": ["https://httpbin.org/links/3"],
            "max_depth": 2,
            "max_pages": 5,
            "strategy": "depth_first",
            "timeout_seconds": 15,
            "delay_ms": 300,
            "concurrency": 1,
            "respect_robots": false,
            "follow_redirects": true,
            "result_mode": "urls"
        });

        let response = server
            .post("/spider/crawl")
            .json(&crawl_body)
            .await;

        if response.status_code() == StatusCode::OK {
            let body: Value = response.json();
            assert!(body["result"]["discovered_urls"].is_array());
        }
    }

    // ========================================================================
    // Edge Cases
    // ========================================================================

    #[tokio::test]
    async fn test_empty_discovered_urls() {
        let server = setup_test_server().await;

        // Use a page with no outgoing links
        let crawl_body = json!({
            "seed_urls": ["https://httpbin.org/status/200"],
            "max_depth": 1,
            "max_pages": 1,
            "strategy": "breadth_first",
            "timeout_seconds": 5,
            "delay_ms": 100,
            "concurrency": 1,
            "respect_robots": false,
            "follow_redirects": true,
            "result_mode": "urls"
        });

        let response = server
            .post("/spider/crawl")
            .json(&crawl_body)
            .await;

        if response.status_code() == StatusCode::OK {
            let body: Value = response.json();

            // Should have discovered_urls array, even if empty
            assert!(body["result"]["discovered_urls"].is_array());

            // May be empty or contain just the seed URL
            let urls = body["result"]["discovered_urls"].as_array().unwrap();
            assert!(
                urls.len() <= 1,
                "Page with no links should discover 0-1 URLs"
            );
        }
    }

    #[tokio::test]
    async fn test_invalid_result_mode() {
        let server = setup_test_server().await;

        let crawl_body = json!({
            "seed_urls": ["https://httpbin.org/html"],
            "max_pages": 1,
            "result_mode": "invalid_mode"
        });

        let response = server
            .post("/spider/crawl")
            .json(&crawl_body)
            .await;

        // Should return error for invalid result_mode
        assert!(
            response.status_code() == StatusCode::BAD_REQUEST
                || response.status_code() == StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid result_mode should return 400 or 422, got: {}",
            response.status_code()
        );
    }

    #[tokio::test]
    async fn test_discovered_urls_with_timeout() {
        let server = setup_test_server().await;

        let crawl_body = json!({
            "seed_urls": ["https://httpbin.org/delay/10"], // Will likely timeout
            "max_depth": 1,
            "max_pages": 5,
            "strategy": "breadth_first",
            "timeout_seconds": 2, // Short timeout
            "delay_ms": 100,
            "concurrency": 1,
            "respect_robots": false,
            "follow_redirects": true,
            "result_mode": "urls"
        });

        let response = server
            .post("/spider/crawl")
            .json(&crawl_body)
            .await;

        if response.status_code() == StatusCode::OK {
            let body: Value = response.json();

            // Should still have discovered_urls array (may be empty)
            assert!(body["result"]["discovered_urls"].is_array());

            // Stop reason should mention timeout
            let stop_reason = body["result"]["stop_reason"].as_str().unwrap();
            assert!(
                stop_reason.contains("timeout") || stop_reason.contains("failed"),
                "Should indicate timeout or failure: {}",
                stop_reason
            );
        }
    }

    // ========================================================================
    // Performance and Metrics
    // ========================================================================

    #[tokio::test]
    async fn test_urls_mode_includes_performance_metrics() {
        let server = setup_test_server().await;

        let crawl_body = json!({
            "seed_urls": ["https://httpbin.org/html"],
            "max_pages": 3,
            "result_mode": "urls"
        });

        let response = server
            .post("/spider/crawl")
            .json(&crawl_body)
            .await;

        if response.status_code() == StatusCode::OK {
            let body: Value = response.json();

            // Should include performance metrics
            assert!(body["performance"].is_object());
            assert!(body["performance"]["pages_per_second"].is_number());

            // Should include state
            assert!(body["state"].is_object());
            assert!(body["state"]["pages_crawled"].is_number());
        }
    }

    #[tokio::test]
    async fn test_discovered_urls_size_is_reasonable() {
        let server = setup_test_server().await;

        let max_pages = 20;
        let crawl_body = json!({
            "seed_urls": ["https://httpbin.org/links/10"],
            "max_depth": 2,
            "max_pages": max_pages,
            "result_mode": "urls"
        });

        let response = server
            .post("/spider/crawl")
            .json(&crawl_body)
            .await;

        if response.status_code() == StatusCode::OK {
            let body: Value = response.json();
            let json_str = serde_json::to_string(&body).unwrap();

            // Payload size should be reasonable (not gigabytes)
            assert!(
                json_str.len() < 1_000_000,
                "Response size should be < 1MB, got {} bytes",
                json_str.len()
            );

            let urls = body["result"]["discovered_urls"].as_array().unwrap();
            assert!(
                urls.len() <= max_pages as usize,
                "Should not exceed max_pages"
            );
        }
    }
}
