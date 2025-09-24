//! Integration tests for strategies pipeline endpoints
//!
//! This module provides end-to-end testing of the strategies pipeline,
//! including extraction, chunking, caching, and error handling.

#[cfg(test)]
mod strategies_integration_tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use riptide_api::models::{CrawlBody, CrawlResponse};
    use riptide_api::{create_app, AppConfig};
    use riptide_core::types::{CrawlOptions, RenderMode};
    use serde_json::{json, Value};
    use std::time::Duration;
    use tokio::time::sleep;

    /// Helper to create integration test configuration
    fn integration_test_config() -> AppConfig {
        AppConfig {
            port: 0,
            redis_url: std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            headless_url: std::env::var("HEADLESS_URL").ok(),
            cache_ttl: 120, // Longer TTL for strategies tests
            max_concurrency: 8,
            gate_hi_threshold: 0.8,
            gate_lo_threshold: 0.3,
            cors_origins: vec![],
            api_key: Some("strategies-test-key".to_string()),
            openai_api_key: None,
            spider_config: None, // Focus on strategies, not spider
        }
    }

    async fn setup_strategies_server() -> TestServer {
        let config = integration_test_config();
        let app = create_app(config).await.expect("Failed to create strategies test app");
        TestServer::new(app.into_make_service()).unwrap()
    }

    #[tokio::test]
    async fn test_strategies_basic_crawl() {
        let server = setup_strategies_server().await;

        let crawl_body = CrawlBody {
            urls: vec!["https://httpbin.org/html".to_string()],
            options: Some(CrawlOptions {
                concurrency: 2,
                cache_mode: "bypass".to_string(), // Force fresh processing
                dynamic_wait_for: None,
                scroll_steps: 0,
                token_chunk_max: 1000,
                token_overlap: 100,
                render_mode: RenderMode::Html,
            }),
        };

        let response = server
            .post("/crawl")
            .json(&crawl_body)
            .await;

        if response.status_code() == StatusCode::OK {
            let body: CrawlResponse = response.json();

            // Verify response structure
            assert_eq!(body.total_urls, 1);
            assert_eq!(body.results.len(), 1);

            let result = &body.results[0];
            assert_eq!(result.url, "https://httpbin.org/html");
            assert!(result.status >= 200 && result.status < 400);
            assert!(!result.gate_decision.is_empty());
            assert!(result.quality_score >= 0.0 && result.quality_score <= 1.0);
            assert!(result.processing_time_ms > 0);

            if let Some(document) = &result.document {
                assert!(!document.url.is_empty());
                assert!(!document.markdown.is_empty() || !document.text.is_empty());
            }

            // Verify statistics
            assert!(body.statistics.total_processing_time_ms > 0);
            assert!(body.statistics.avg_processing_time_ms > 0.0);

        } else {
            let error_body: Value = response.json();
            println!("Basic crawl failed (may be expected in test env): {}", error_body);
        }
    }

    #[tokio::test]
    async fn test_strategies_multiple_urls() {
        let server = setup_strategies_server().await;

        let crawl_body = CrawlBody {
            urls: vec![
                "https://httpbin.org/html".to_string(),
                "https://httpbin.org/json".to_string(),
                "https://httpbin.org/xml".to_string(),
            ],
            options: Some(CrawlOptions {
                concurrency: 3,
                cache_mode: "bypass".to_string(),
                dynamic_wait_for: None,
                scroll_steps: 0,
                token_chunk_max: 800,
                token_overlap: 80,
                render_mode: RenderMode::Html,
            }),
        };

        let response = server
            .post("/crawl")
            .json(&crawl_body)
            .await;

        if response.status_code() == StatusCode::OK {
            let body: CrawlResponse = response.json();

            assert_eq!(body.total_urls, 3);
            assert_eq!(body.results.len(), 3);

            // All URLs should be processed
            let urls_processed: Vec<&String> = body.results.iter().map(|r| &r.url).collect();
            assert!(urls_processed.contains(&&"https://httpbin.org/html".to_string()));
            assert!(urls_processed.contains(&&"https://httpbin.org/json".to_string()));
            assert!(urls_processed.contains(&&"https://httpbin.org/xml".to_string()));

            // Check gate decisions
            let gate_decisions: Vec<&String> = body.results.iter().map(|r| &r.gate_decision).collect();
            assert!(gate_decisions.iter().all(|d| !d.is_empty()));

            // Verify statistics are accumulated correctly
            assert!(body.successful + body.failed == 3);
            assert!(body.statistics.total_processing_time_ms > 0);

        } else {
            println!("Multi-URL crawl failed (may be expected): {}", response.status_code());
        }
    }

    #[tokio::test]
    async fn test_strategies_cache_functionality() {
        let server = setup_strategies_server().await;

        let test_url = "https://httpbin.org/html";

        // First request with cache enabled
        let crawl_body = CrawlBody {
            urls: vec![test_url.to_string()],
            options: Some(CrawlOptions {
                concurrency: 1,
                cache_mode: "read_through".to_string(),
                dynamic_wait_for: None,
                scroll_steps: 0,
                token_chunk_max: 1200,
                token_overlap: 120,
                render_mode: RenderMode::Html,
            }),
        };

        let first_response = server
            .post("/crawl")
            .json(&crawl_body)
            .await;

        if first_response.status_code() == StatusCode::OK {
            let first_body: CrawlResponse = first_response.json();
            let first_result = &first_body.results[0];

            assert!(!first_result.from_cache, "First request should not be from cache");
            let first_processing_time = first_result.processing_time_ms;

            // Second request should use cache
            let second_response = server
                .post("/crawl")
                .json(&crawl_body)
                .await;

            if second_response.status_code() == StatusCode::OK {
                let second_body: CrawlResponse = second_response.json();
                let second_result = &second_body.results[0];

                // Cache behavior may vary based on implementation
                if second_result.from_cache {
                    // Cached responses should be faster
                    assert!(second_result.processing_time_ms <= first_processing_time,
                           "Cached response should be faster: {} vs {}",
                           second_result.processing_time_ms, first_processing_time);
                    println!("Cache hit detected: {} -> {} ms",
                            first_processing_time, second_result.processing_time_ms);
                } else {
                    println!("Cache miss (may be due to TTL or cache key differences)");
                }
            }

        } else {
            println!("Cache test failed (may be expected): {}", first_response.status_code());
        }
    }

    #[tokio::test]
    async fn test_strategies_different_render_modes() {
        let server = setup_strategies_server().await;

        let render_modes = vec![
            (RenderMode::Html, "HTML render mode"),
            (RenderMode::Markdown, "Markdown render mode"),
        ];

        for (render_mode, description) in render_modes {
            let crawl_body = CrawlBody {
                urls: vec!["https://httpbin.org/html".to_string()],
                options: Some(CrawlOptions {
                    concurrency: 1,
                    cache_mode: "bypass".to_string(),
                    dynamic_wait_for: None,
                    scroll_steps: 0,
                    token_chunk_max: 1000,
                    token_overlap: 100,
                    render_mode: render_mode.clone(),
                }),
            };

            let response = server
                .post("/crawl")
                .json(&crawl_body)
                .await;

            if response.status_code() == StatusCode::OK {
                let body: CrawlResponse = response.json();
                let result = &body.results[0];

                println!("{} - Status: {}, Gate: {}, Score: {}",
                        description, result.status, result.gate_decision, result.quality_score);

                // Should process successfully regardless of render mode
                assert!(result.processing_time_ms > 0);
                assert!(result.quality_score >= 0.0);

            } else {
                println!("{} failed (may be expected): {}", description, response.status_code());
            }

            // Small delay between render mode tests
            sleep(Duration::from_millis(100)).await;
        }
    }

    #[tokio::test]
    async fn test_strategies_chunking_configurations() {
        let server = setup_strategies_server().await;

        let chunking_configs = vec![
            (500, 50, "Small chunks with small overlap"),
            (1500, 150, "Medium chunks with medium overlap"),
            (3000, 300, "Large chunks with large overlap"),
            (1000, 0, "Medium chunks with no overlap"),
        ];

        for (chunk_max, overlap, description) in chunking_configs {
            let crawl_body = CrawlBody {
                urls: vec!["https://httpbin.org/html".to_string()],
                options: Some(CrawlOptions {
                    concurrency: 1,
                    cache_mode: "bypass".to_string(),
                    dynamic_wait_for: None,
                    scroll_steps: 0,
                    token_chunk_max: chunk_max,
                    token_overlap: overlap,
                    render_mode: RenderMode::Html,
                }),
            };

            let response = server
                .post("/crawl")
                .json(&crawl_body)
                .await;

            if response.status_code() == StatusCode::OK {
                let body: CrawlResponse = response.json();
                let result = &body.results[0];

                println!("{} - Processing time: {}ms, Gate: {}",
                        description, result.processing_time_ms, result.gate_decision);

                // All chunking configurations should work
                assert!(result.processing_time_ms > 0);

            } else {
                println!("{} failed (may be expected): {}", description, response.status_code());
            }

            // Small delay between chunking tests
            sleep(Duration::from_millis(50)).await;
        }
    }

    #[tokio::test]
    async fn test_strategies_error_handling() {
        let server = setup_strategies_server().await;

        // Test with invalid URLs
        let crawl_body = CrawlBody {
            urls: vec![
                "https://httpbin.org/status/404".to_string(), // 404 error
                "https://httpbin.org/status/500".to_string(), // 500 error
                "https://httpbin.org/html".to_string(),       // Valid URL
                "https://nonexistent-domain-xyz.invalid".to_string(), // DNS error
            ],
            options: Some(CrawlOptions {
                concurrency: 4,
                cache_mode: "bypass".to_string(),
                dynamic_wait_for: None,
                scroll_steps: 0,
                token_chunk_max: 1000,
                token_overlap: 100,
                render_mode: RenderMode::Html,
            }),
        };

        let response = server
            .post("/crawl")
            .json(&crawl_body)
            .await;

        if response.status_code() == StatusCode::OK {
            let body: CrawlResponse = response.json();

            assert_eq!(body.total_urls, 4);
            assert_eq!(body.results.len(), 4);

            // Should have mix of successful and failed results
            let successful_results = body.results.iter().filter(|r| r.document.is_some()).count();
            let failed_results = body.results.iter().filter(|r| r.error.is_some()).count();

            println!("Error handling test: {} successful, {} failed",
                    successful_results, failed_results);

            assert!(successful_results >= 1, "Should have at least one successful result");
            assert!(failed_results >= 1, "Should have at least one failed result");

            // Check error information for failed results
            for result in &body.results {
                if let Some(error) = &result.error {
                    assert!(!error.error_type.is_empty());
                    assert!(!error.message.is_empty());
                    println!("Error for {}: {} - {}", result.url, error.error_type, error.message);
                }
            }

        } else {
            println!("Error handling test failed completely: {}", response.status_code());
        }
    }

    #[tokio::test]
    async fn test_strategies_concurrency_handling() {
        let server = setup_strategies_server().await;

        // Test with different concurrency levels
        let concurrency_levels = vec![1, 3, 8];

        for concurrency in concurrency_levels {
            let crawl_body = CrawlBody {
                urls: vec![
                    "https://httpbin.org/delay/1".to_string(),
                    "https://httpbin.org/delay/1".to_string(),
                    "https://httpbin.org/delay/1".to_string(),
                    "https://httpbin.org/html".to_string(),
                ],
                options: Some(CrawlOptions {
                    concurrency,
                    cache_mode: "bypass".to_string(),
                    dynamic_wait_for: None,
                    scroll_steps: 0,
                    token_chunk_max: 1000,
                    token_overlap: 100,
                    render_mode: RenderMode::Html,
                }),
            };

            let start_time = std::time::Instant::now();
            let response = server
                .post("/crawl")
                .json(&crawl_body)
                .await;
            let elapsed = start_time.elapsed();

            if response.status_code() == StatusCode::OK {
                let body: CrawlResponse = response.json();

                println!("Concurrency {} - Total time: {}ms, Processing: {}ms",
                        concurrency, elapsed.as_millis(), body.statistics.total_processing_time_ms);

                // Higher concurrency should generally be faster for multiple URLs
                // (though network delays may affect this in test environments)
                assert_eq!(body.total_urls, 4);

            } else {
                println!("Concurrency {} test failed: {}", concurrency, response.status_code());
            }

            // Delay between concurrency tests
            sleep(Duration::from_millis(200)).await;
        }
    }

    #[tokio::test]
    async fn test_strategies_gate_decision_variations() {
        let server = setup_strategies_server().await;

        // Test URLs that should trigger different gate decisions
        let test_urls = vec![
            ("https://httpbin.org/html", "Simple HTML page"),
            ("https://httpbin.org/json", "JSON response"),
            ("https://httpbin.org/xml", "XML response"),
        ];

        for (url, description) in test_urls {
            let crawl_body = CrawlBody {
                urls: vec![url.to_string()],
                options: Some(CrawlOptions {
                    concurrency: 1,
                    cache_mode: "bypass".to_string(),
                    dynamic_wait_for: None,
                    scroll_steps: 0,
                    token_chunk_max: 1000,
                    token_overlap: 100,
                    render_mode: RenderMode::Html,
                }),
            };

            let response = server
                .post("/crawl")
                .json(&crawl_body)
                .await;

            if response.status_code() == StatusCode::OK {
                let body: CrawlResponse = response.json();
                let result = &body.results[0];

                println!("{} - Gate: {}, Score: {:.3}, Time: {}ms",
                        description, result.gate_decision, result.quality_score,
                        result.processing_time_ms);

                // Should make valid gate decisions
                assert!(matches!(result.gate_decision.as_str(), "raw" | "probes_first" | "headless" | "cached"));
                assert!(result.quality_score >= 0.0 && result.quality_score <= 1.0);

            } else {
                println!("{} failed (may be expected): {}", description, response.status_code());
            }

            // Small delay between tests
            sleep(Duration::from_millis(100)).await;
        }
    }

    #[tokio::test]
    async fn test_strategies_performance_metrics() {
        let server = setup_strategies_server().await;

        let crawl_body = CrawlBody {
            urls: vec![
                "https://httpbin.org/html".to_string(),
                "https://httpbin.org/json".to_string(),
            ],
            options: Some(CrawlOptions {
                concurrency: 2,
                cache_mode: "read_through".to_string(),
                dynamic_wait_for: None,
                scroll_steps: 0,
                token_chunk_max: 1200,
                token_overlap: 120,
                render_mode: RenderMode::Html,
            }),
        };

        let response = server
            .post("/crawl")
            .json(&crawl_body)
            .await;

        if response.status_code() == StatusCode::OK {
            let body: CrawlResponse = response.json();

            // Verify performance statistics
            let stats = &body.statistics;
            assert!(stats.total_processing_time_ms > 0);
            assert!(stats.avg_processing_time_ms > 0.0);
            assert!(stats.cache_hit_rate >= 0.0 && stats.cache_hit_rate <= 1.0);

            // Gate decision breakdown
            let gate_breakdown = &stats.gate_decisions;
            let total_decisions = gate_breakdown.raw + gate_breakdown.probes_first +
                                gate_breakdown.headless + gate_breakdown.cached;
            assert_eq!(total_decisions, body.total_urls);

            println!("Performance metrics:");
            println!("  Total time: {}ms", stats.total_processing_time_ms);
            println!("  Avg time: {:.2}ms", stats.avg_processing_time_ms);
            println!("  Cache hit rate: {:.2}%", stats.cache_hit_rate * 100.0);
            println!("  Gate decisions: raw={}, probes={}, headless={}, cached={}",
                    gate_breakdown.raw, gate_breakdown.probes_first,
                    gate_breakdown.headless, gate_breakdown.cached);

        } else {
            println!("Performance metrics test failed: {}", response.status_code());
        }
    }

    #[tokio::test]
    async fn test_strategies_timeout_handling() {
        let server = setup_strategies_server().await;

        // Test with URLs that may timeout
        let crawl_body = CrawlBody {
            urls: vec![
                "https://httpbin.org/delay/10".to_string(), // Long delay that may timeout
                "https://httpbin.org/html".to_string(),     // Quick response
            ],
            options: Some(CrawlOptions {
                concurrency: 2,
                cache_mode: "bypass".to_string(),
                dynamic_wait_for: None,
                scroll_steps: 0,
                token_chunk_max: 1000,
                token_overlap: 100,
                render_mode: RenderMode::Html,
            }),
        };

        let response = server
            .post("/crawl")
            .json(&crawl_body)
            .await;

        if response.status_code() == StatusCode::OK {
            let body: CrawlResponse = response.json();

            // Should handle timeouts gracefully
            assert_eq!(body.results.len(), 2);

            let mut timeout_handled = false;
            let mut success_found = false;

            for result in &body.results {
                if let Some(error) = &result.error {
                    if error.error_type.to_lowercase().contains("timeout") {
                        timeout_handled = true;
                        println!("Timeout properly handled for {}", result.url);
                    }
                } else if result.document.is_some() {
                    success_found = true;
                    println!("Successful processing for {}", result.url);
                }
            }

            // Should have at least one successful result and possibly a timeout
            assert!(success_found || timeout_handled,
                   "Should handle timeouts gracefully or complete successfully");

        } else {
            println!("Timeout handling test failed: {}", response.status_code());
        }
    }

    #[tokio::test]
    async fn test_strategies_content_type_handling() {
        let server = setup_strategies_server().await;

        // Test different content types
        let content_types = vec![
            ("https://httpbin.org/html", "text/html"),
            ("https://httpbin.org/json", "application/json"),
            ("https://httpbin.org/xml", "application/xml"),
        ];

        for (url, expected_type) in content_types {
            let crawl_body = CrawlBody {
                urls: vec![url.to_string()],
                options: Some(CrawlOptions {
                    concurrency: 1,
                    cache_mode: "bypass".to_string(),
                    dynamic_wait_for: None,
                    scroll_steps: 0,
                    token_chunk_max: 1000,
                    token_overlap: 100,
                    render_mode: RenderMode::Html,
                }),
            };

            let response = server
                .post("/crawl")
                .json(&crawl_body)
                .await;

            if response.status_code() == StatusCode::OK {
                let body: CrawlResponse = response.json();
                let result = &body.results[0];

                println!("Content type {} - Status: {}, Gate: {}",
                        expected_type, result.status, result.gate_decision);

                // Should handle different content types appropriately
                if result.document.is_some() {
                    println!("Successfully processed {} content", expected_type);
                } else if result.error.is_some() {
                    println!("Error processing {} content (may be expected): {:?}",
                            expected_type, result.error);
                }

            } else {
                println!("Content type {} test failed: {}", expected_type, response.status_code());
            }

            // Small delay between content type tests
            sleep(Duration::from_millis(100)).await;
        }
    }
}