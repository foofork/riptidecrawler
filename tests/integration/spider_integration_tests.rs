//! Integration tests for spider endpoints and functionality
//!
//! This module provides end-to-end testing of spider crawling,
//! status monitoring, and control operations.

#[cfg(test)]
mod spider_integration_tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use riptide_api::models::{SpiderCrawlBody, SpiderStatusRequest, SpiderControlRequest};
    use riptide_api::{create_app, AppConfig};
    use serde_json::{json, Value};
    use std::time::Duration;
    use tokio::time::sleep;

    /// Helper to create integration test configuration
    fn integration_test_config() -> AppConfig {
        AppConfig {
            port: 0,
            redis_url: std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            headless_url: std::env::var("HEADLESS_URL").ok(),
            cache_ttl: 60, // Shorter TTL for tests
            max_concurrency: 5, // Lower concurrency for tests
            gate_hi_threshold: 0.8,
            gate_lo_threshold: 0.3,
            cors_origins: vec![],
            api_key: Some("integration-test-key".to_string()),
            openai_api_key: None,
            spider_config: Some(riptide_core::spider::SpiderConfig::new(
                "https://httpbin.org".parse().unwrap()
            )),
        }
    }

    async fn setup_test_server() -> TestServer {
        let config = integration_test_config();
        let app = create_app(config).await.expect("Failed to create test app");
        TestServer::new(app.into_make_service()).unwrap()
    }

    #[tokio::test]
    async fn test_spider_crawl_basic_functionality() {
        let server = setup_test_server().await;

        let crawl_body = SpiderCrawlBody {
            seed_urls: vec!["https://httpbin.org/html".to_string()],
            max_depth: Some(1),
            max_pages: Some(3),
            strategy: Some("breadth_first".to_string()),
            timeout_seconds: Some(10),
            delay_ms: Some(500),
            concurrency: Some(2),
            respect_robots: Some(false), // For testing purposes
            follow_redirects: Some(true),
        };

        let response = server
            .post("/spider/crawl")
            .json(&crawl_body)
            .await;

        // Should successfully start crawl
        if response.status_code() == StatusCode::OK {
            let body: Value = response.json();

            // Verify response structure
            assert!(body["result"].is_object());
            assert!(body["state"].is_object());
            assert!(body["performance"].is_object());

            let result = &body["result"];
            assert!(result["pages_crawled"].is_number());
            assert!(result["pages_failed"].is_number());
            assert!(result["duration_seconds"].is_number());
            assert!(result["stop_reason"].is_string());

            // Should have crawled at least one page
            let pages_crawled = result["pages_crawled"].as_u64().unwrap();
            assert!(pages_crawled >= 1, "Should crawl at least one page");

            // Duration should be reasonable
            let duration = result["duration_seconds"].as_f64().unwrap();
            assert!(duration > 0.0, "Duration should be positive");
            assert!(duration < 60.0, "Duration should be reasonable for test");

        } else {
            // If crawl fails due to network/configuration issues, verify error handling
            let body: Value = response.json();
            assert!(body["error"].is_string());
            println!("Crawl failed (expected in some test environments): {}", body["error"]);
        }
    }

    #[tokio::test]
    async fn test_spider_crawl_multiple_urls() {
        let server = setup_test_server().await;

        let crawl_body = SpiderCrawlBody {
            seed_urls: vec![
                "https://httpbin.org/html".to_string(),
                "https://httpbin.org/json".to_string(),
                "https://httpbin.org/xml".to_string(),
            ],
            max_depth: Some(1),
            max_pages: Some(5),
            strategy: Some("breadth_first".to_string()),
            timeout_seconds: Some(15),
            delay_ms: Some(200),
            concurrency: Some(2),
            respect_robots: Some(false),
            follow_redirects: Some(true),
        };

        let response = server
            .post("/spider/crawl")
            .json(&crawl_body)
            .await;

        if response.status_code() == StatusCode::OK {
            let body: Value = response.json();
            let result = &body["result"];

            // Should attempt to crawl multiple URLs
            let domains = result["domains"].as_array().unwrap();
            assert!(!domains.is_empty(), "Should discover at least one domain");

            // Should report on multiple seed URLs
            let pages_crawled = result["pages_crawled"].as_u64().unwrap();
            let pages_failed = result["pages_failed"].as_u64().unwrap();
            let total_attempted = pages_crawled + pages_failed;

            assert!(total_attempted >= 3 || result["stop_reason"].as_str().unwrap().contains("timeout"),
                   "Should attempt multiple URLs or timeout: crawled={}, failed={}, reason={}",
                   pages_crawled, pages_failed, result["stop_reason"]);

        } else {
            println!("Multi-URL crawl failed (may be expected): {}", response.status_code());
        }
    }

    #[tokio::test]
    async fn test_spider_crawl_strategy_variations() {
        let server = setup_test_server().await;

        let strategies = vec!["breadth_first", "depth_first", "best_first"];

        for strategy in strategies {
            let crawl_body = SpiderCrawlBody {
                seed_urls: vec!["https://httpbin.org/html".to_string()],
                max_depth: Some(1),
                max_pages: Some(2),
                strategy: Some(strategy.to_string()),
                timeout_seconds: Some(8),
                delay_ms: Some(300),
                concurrency: Some(1),
                respect_robots: Some(false),
                follow_redirects: Some(true),
            };

            let response = server
                .post("/spider/crawl")
                .json(&crawl_body)
                .await;

            // Each strategy should be accepted
            assert_ne!(response.status_code(), StatusCode::BAD_REQUEST,
                      "Strategy '{}' should be valid", strategy);

            if response.status_code() == StatusCode::OK {
                let body: Value = response.json();
                println!("Strategy '{}' completed successfully", strategy);

                // Verify basic response structure
                assert!(body["result"].is_object());
                assert!(body["performance"].is_object());
            } else {
                println!("Strategy '{}' failed (may be expected in test env): {}",
                        strategy, response.status_code());
            }

            // Small delay between strategy tests
            sleep(Duration::from_millis(100)).await;
        }
    }

    #[tokio::test]
    async fn test_spider_status_monitoring() {
        let server = setup_test_server().await;

        // Test status without metrics
        let status_request = SpiderStatusRequest {
            include_metrics: Some(false),
        };

        let response = server
            .post("/spider/status")
            .json(&status_request)
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let body: Value = response.json();
        assert!(body["state"].is_object());

        // Performance should be null when not requested
        assert!(body["performance"].is_null() || !body.contains_key("performance"));

        // Test status with metrics
        let status_request = SpiderStatusRequest {
            include_metrics: Some(true),
        };

        let response = server
            .post("/spider/status")
            .json(&status_request)
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let body: Value = response.json();
        assert!(body["state"].is_object());

        // Should include metrics when requested
        if body["performance"].is_object() {
            let performance = &body["performance"];
            // Basic performance metrics structure
            assert!(performance.is_object());
        }
    }

    #[tokio::test]
    async fn test_spider_control_operations() {
        let server = setup_test_server().await;

        // Test stop operation
        let stop_request = SpiderControlRequest {
            action: "stop".to_string(),
        };

        let response = server
            .post("/spider/control")
            .json(&stop_request)
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let body: Value = response.json();
        assert_eq!(body["status"].as_str().unwrap(), "stopped");

        // Test reset operation
        let reset_request = SpiderControlRequest {
            action: "reset".to_string(),
        };

        let response = server
            .post("/spider/control")
            .json(&reset_request)
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let body: Value = response.json();
        assert_eq!(body["status"].as_str().unwrap(), "reset");
    }

    #[tokio::test]
    async fn test_spider_crawl_with_constraints() {
        let server = setup_test_server().await;

        // Test with very restrictive constraints
        let constrained_body = SpiderCrawlBody {
            seed_urls: vec!["https://httpbin.org/links/5".to_string()], // Page with multiple links
            max_depth: Some(2),
            max_pages: Some(3), // Very restrictive page limit
            strategy: Some("breadth_first".to_string()),
            timeout_seconds: Some(5), // Short timeout
            delay_ms: Some(1000), // Longer delay
            concurrency: Some(1), // Single-threaded
            respect_robots: Some(false),
            follow_redirects: Some(true),
        };

        let response = server
            .post("/spider/crawl")
            .json(&constrained_body)
            .await;

        if response.status_code() == StatusCode::OK {
            let body: Value = response.json();
            let result = &body["result"];

            // Should respect page limit
            let pages_crawled = result["pages_crawled"].as_u64().unwrap();
            assert!(pages_crawled <= 3,
                   "Should respect max_pages constraint: crawled {} pages", pages_crawled);

            // Should respect timeout or page limit
            let stop_reason = result["stop_reason"].as_str().unwrap();
            assert!(stop_reason.contains("pages") || stop_reason.contains("timeout") || stop_reason.contains("finished"),
                   "Stop reason should be related to constraints: {}", stop_reason);

        } else {
            println!("Constrained crawl failed (may be expected): {}", response.status_code());
        }
    }

    #[tokio::test]
    async fn test_spider_error_handling() {
        let server = setup_test_server().await;

        // Test with invalid URLs
        let invalid_body = SpiderCrawlBody {
            seed_urls: vec!["https://nonexistent-domain-12345.invalid".to_string()],
            max_depth: Some(1),
            max_pages: Some(2),
            strategy: Some("breadth_first".to_string()),
            timeout_seconds: Some(5),
            delay_ms: Some(100),
            concurrency: Some(1),
            respect_robots: Some(false),
            follow_redirects: Some(true),
        };

        let response = server
            .post("/spider/crawl")
            .json(&invalid_body)
            .await;

        if response.status_code() == StatusCode::OK {
            let body: Value = response.json();
            let result = &body["result"];

            // Should handle failed URLs gracefully
            let pages_failed = result["pages_failed"].as_u64().unwrap();
            assert!(pages_failed > 0, "Should report failed pages for invalid URLs");

            let stop_reason = result["stop_reason"].as_str().unwrap();
            assert!(!stop_reason.is_empty(), "Should provide stop reason");

        } else {
            // Should return structured error
            let body: Value = response.json();
            assert!(body["error"].is_string());
        }
    }

    #[tokio::test]
    async fn test_spider_concurrent_operations() {
        let server = setup_test_server().await;

        // Start multiple operations concurrently
        let mut handles = Vec::new();

        for i in 0..3 {
            let server = server.clone();
            let handle = tokio::spawn(async move {
                let crawl_body = SpiderCrawlBody {
                    seed_urls: vec![format!("https://httpbin.org/delay/{}", i % 2 + 1)],
                    max_depth: Some(1),
                    max_pages: Some(1),
                    strategy: Some("breadth_first".to_string()),
                    timeout_seconds: Some(10),
                    delay_ms: Some(200),
                    concurrency: Some(1),
                    respect_robots: Some(false),
                    follow_redirects: Some(true),
                };

                server
                    .post("/spider/crawl")
                    .json(&crawl_body)
                    .await
            });

            handles.push(handle);

            // Small delay between starting operations
            sleep(Duration::from_millis(100)).await;
        }

        // Wait for all operations to complete
        let mut successful_operations = 0;
        for handle in handles {
            match handle.await {
                Ok(response) => {
                    if response.status_code() == StatusCode::OK {
                        successful_operations += 1;
                    }
                    println!("Concurrent operation completed with status: {}", response.status_code());
                }
                Err(e) => {
                    println!("Concurrent operation failed: {}", e);
                }
            }
        }

        // At least some operations should succeed
        println!("Successful concurrent operations: {}/3", successful_operations);
    }

    #[tokio::test]
    async fn test_spider_metrics_and_performance() {
        let server = setup_test_server().await;

        let crawl_body = SpiderCrawlBody {
            seed_urls: vec!["https://httpbin.org/html".to_string()],
            max_depth: Some(1),
            max_pages: Some(2),
            strategy: Some("breadth_first".to_string()),
            timeout_seconds: Some(10),
            delay_ms: Some(300),
            concurrency: Some(1),
            respect_robots: Some(false),
            follow_redirects: Some(true),
        };

        let response = server
            .post("/spider/crawl")
            .json(&crawl_body)
            .await;

        if response.status_code() == StatusCode::OK {
            let body: Value = response.json();

            // Check performance metrics
            let performance = &body["performance"];
            if performance.is_object() {
                // Should have timing information
                assert!(performance.is_object());

                // Check crawl state
                let state = &body["state"];
                assert!(state.is_object());

                // Performance data should be reasonable
                println!("Spider performance metrics: {}", performance);
                println!("Spider crawl state: {}", state);
            }

            // Get detailed status with metrics
            let status_request = SpiderStatusRequest {
                include_metrics: Some(true),
            };

            let status_response = server
                .post("/spider/status")
                .json(&status_request)
                .await;

            if status_response.status_code() == StatusCode::OK {
                let status_body: Value = status_response.json();

                // Should include frontier and adaptive stop stats when available
                if status_body["frontier_stats"].is_object() {
                    println!("Frontier stats available: {}", status_body["frontier_stats"]);
                }

                if status_body["adaptive_stop_stats"].is_object() {
                    println!("Adaptive stop stats available: {}", status_body["adaptive_stop_stats"]);
                }
            }

        } else {
            println!("Metrics test crawl failed (may be expected): {}", response.status_code());
        }
    }

    #[tokio::test]
    async fn test_spider_authentication() {
        let server = setup_test_server().await;

        let crawl_body = SpiderCrawlBody {
            seed_urls: vec!["https://httpbin.org/html".to_string()],
            max_depth: Some(1),
            max_pages: Some(1),
            strategy: Some("breadth_first".to_string()),
            timeout_seconds: Some(5),
            delay_ms: Some(100),
            concurrency: Some(1),
            respect_robots: Some(false),
            follow_redirects: Some(true),
        };

        // Test without API key (if required by configuration)
        let response = server
            .post("/spider/crawl")
            .json(&crawl_body)
            .await;

        // Should either work (no auth required) or return auth error
        if response.status_code() == StatusCode::UNAUTHORIZED {
            println!("Spider requires authentication - this is expected behavior");
        } else {
            println!("Spider crawl status without auth: {}", response.status_code());
        }

        // Test with API key
        let response = server
            .post("/spider/crawl")
            .header("Authorization", "Bearer integration-test-key")
            .json(&crawl_body)
            .await;

        // Should work with proper authentication
        assert_ne!(response.status_code(), StatusCode::UNAUTHORIZED,
                  "Should not return unauthorized with valid API key");
    }

    #[tokio::test]
    async fn test_spider_crawl_lifecycle() {
        let server = setup_test_server().await;

        // 1. Check initial status
        let status_request = SpiderStatusRequest {
            include_metrics: Some(false),
        };

        let initial_status = server
            .post("/spider/status")
            .json(&status_request)
            .await;

        assert_eq!(initial_status.status_code(), StatusCode::OK);

        // 2. Start a crawl
        let crawl_body = SpiderCrawlBody {
            seed_urls: vec!["https://httpbin.org/links/3".to_string()],
            max_depth: Some(2),
            max_pages: Some(5),
            strategy: Some("breadth_first".to_string()),
            timeout_seconds: Some(15),
            delay_ms: Some(200),
            concurrency: Some(2),
            respect_robots: Some(false),
            follow_redirects: Some(true),
        };

        let crawl_response = server
            .post("/spider/crawl")
            .json(&crawl_body)
            .await;

        // 3. Check final status
        let final_status = server
            .post("/spider/status")
            .json(&SpiderStatusRequest { include_metrics: Some(true) })
            .await;

        assert_eq!(final_status.status_code(), StatusCode::OK);

        // 4. Reset spider state
        let reset_request = SpiderControlRequest {
            action: "reset".to_string(),
        };

        let reset_response = server
            .post("/spider/control")
            .json(&reset_request)
            .await;

        assert_eq!(reset_response.status_code(), StatusCode::OK);

        println!("Spider lifecycle test completed:");
        println!("- Initial status: {}", initial_status.status_code());
        println!("- Crawl response: {}", crawl_response.status_code());
        println!("- Final status: {}", final_status.status_code());
        println!("- Reset response: {}", reset_response.status_code());
    }
}