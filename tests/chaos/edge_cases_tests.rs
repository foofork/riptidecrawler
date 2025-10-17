//! Edge cases and error condition tests
//!
//! This module tests boundary conditions, error scenarios, and
//! unusual inputs that might cause system failures.

#[cfg(test)]
mod edge_cases_tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use riptide_api::models::{CrawlBody, SpiderCrawlBody, SpiderControlRequest};
    use riptide_api::{create_app, AppConfig};
    use riptide_core::types::{CrawlOptions, RenderMode};
    use serde_json::{json, Value};
    use std::collections::HashMap;

    /// Helper to create test configuration for edge case testing
    fn edge_case_test_config() -> AppConfig {
        AppConfig {
            port: 0,
            redis_url: "redis://localhost:6379".to_string(),
            headless_url: None, // Disabled for edge case testing
            cache_ttl: 1, // Very short TTL to test cache expiration
            max_concurrency: 1, // Low concurrency for controlled testing
            gate_hi_threshold: 0.9, // High threshold
            gate_lo_threshold: 0.1, // Low threshold
            cors_origins: vec![],
            api_key: None, // No auth for edge case testing
            openai_api_key: None,
            spider_config: Some(riptide_core::spider::SpiderConfig::new(
                "https://example.com".parse().unwrap()
            )),
        }
    }

    async fn setup_edge_case_server() -> TestServer {
        let config = edge_case_test_config();
        let app = create_app(config).await.expect("Failed to create edge case test app");
        TestServer::new(app.into_make_service()).unwrap()
    }

    #[tokio::test]
    async fn test_empty_and_null_inputs() {
        let server = setup_edge_case_server().await;

        // Test empty URL list
        let empty_crawl = CrawlBody {
            urls: vec![],
            options: None,
        };

        let response = server.post("/crawl").json(&empty_crawl).await;
        assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

        // Test empty spider URLs
        let empty_spider = SpiderCrawlBody {
            seed_urls: vec![],
            max_depth: None,
            max_pages: None,
            strategy: None,
            timeout_seconds: None,
            delay_ms: None,
            concurrency: None,
            respect_robots: None,
            follow_redirects: None,
        };

        let response = server.post("/spider/crawl").json(&empty_spider).await;
        assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

        // Test null/empty string URLs
        let null_url_crawl = json!({
            "urls": ["", null, "   "],
            "options": {}
        });

        let response = server.post("/crawl").json(&null_url_crawl).await;
        assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_malformed_urls() {
        let server = setup_edge_case_server().await;

        let malformed_urls = vec![
            "not-a-url",
            "http://",
            "https://",
            "ftp://invalid.com", // Unsupported protocol
            "mailto:test@example.com", // Unsupported protocol
            "javascript:alert('xss')", // Dangerous protocol
            "data:text/html,<script>alert('xss')</script>", // Data URL
            "file:///etc/passwd", // Local file access
            "http://[invalid-ipv6", // Malformed IPv6
            "http://256.256.256.256", // Invalid IPv4
            "http://localhost:-1", // Invalid port
            "http://localhost:99999", // Invalid port range
            "http://example..com", // Double dot in domain
            "http://-example.com", // Leading dash in domain
            "http://example-.com", // Trailing dash in domain
            "http://exam ple.com", // Space in domain
            "http://example.com:8080:8080", // Multiple ports
        ];

        for malformed_url in malformed_urls {
            let crawl_body = CrawlBody {
                urls: vec![malformed_url.to_string()],
                options: Some(CrawlOptions::default()),
            };

            let response = server.post("/crawl").json(&crawl_body).await;

            // Should return bad request for malformed URLs
            assert_eq!(response.status_code(), StatusCode::BAD_REQUEST,
                      "URL '{}' should be rejected", malformed_url);

            let body: Value = response.json();
            assert!(body["error"].is_string(),
                   "Should provide error message for malformed URL: {}", malformed_url);
        }
    }

    #[tokio::test]
    async fn test_extreme_parameter_values() {
        let server = setup_edge_case_server().await;

        // Test with extreme parameter values
        let extreme_options = CrawlOptions {
            concurrency: 0, // Zero concurrency
            cache_mode: "invalid_mode".to_string(), // Invalid cache mode
            dynamic_wait_for: Some("invalid_wait".to_string()), // Invalid wait condition
            scroll_steps: u32::MAX, // Maximum scroll steps
            token_chunk_max: 0, // Zero token chunks
            token_overlap: u32::MAX, // Maximum overlap
            render_mode: RenderMode::Html,
        };

        let crawl_body = CrawlBody {
            urls: vec!["https://example.com".to_string()],
            options: Some(extreme_options),
        };

        let response = server.post("/crawl").json(&crawl_body).await;

        // Should handle extreme values gracefully (either succeed or return meaningful error)
        if response.status_code() == StatusCode::BAD_REQUEST {
            let body: Value = response.json();
            assert!(body["error"].is_string());
            println!("Extreme parameters rejected: {}", body["error"]);
        } else if response.status_code() == StatusCode::OK {
            println!("Extreme parameters handled gracefully");
        } else {
            panic!("Unexpected status for extreme parameters: {}", response.status_code());
        }
    }

    #[tokio::test]
    async fn test_very_long_urls() {
        let server = setup_edge_case_server().await;

        // Test extremely long URLs
        let base_url = "https://example.com/";
        let long_path = "a".repeat(2000); // 2KB path
        let very_long_path = "b".repeat(8192); // 8KB path
        let extreme_path = "c".repeat(32768); // 32KB path

        let long_urls = vec![
            format!("{}{}", base_url, long_path),
            format!("{}{}", base_url, very_long_path),
            format!("{}{}", base_url, extreme_path),
        ];

        for (i, long_url) in long_urls.iter().enumerate() {
            let crawl_body = CrawlBody {
                urls: vec![long_url.clone()],
                options: Some(CrawlOptions::default()),
            };

            let response = server.post("/crawl").json(&crawl_body).await;

            println!("Long URL test {} ({} chars): {}",
                    i + 1, long_url.len(), response.status_code());

            // Should either process or reject very long URLs gracefully
            assert!(
                response.status_code() == StatusCode::OK ||
                response.status_code() == StatusCode::BAD_REQUEST ||
                response.status_code() == StatusCode::REQUEST_ENTITY_TOO_LARGE,
                "Long URL should be handled gracefully: {} chars -> {}",
                long_url.len(), response.status_code()
            );
        }
    }

    #[tokio::test]
    async fn test_unicode_and_special_characters() {
        let server = setup_edge_case_server().await;

        let special_urls = vec![
            "https://example.com/café", // Accented characters
            "https://example.com/测试", // Chinese characters
            "https://example.com/тест", // Cyrillic characters
            "https://example.com/🚀", // Emoji
            "https://example.com/path with spaces", // Spaces
            "https://example.com/path%20encoded", // Encoded spaces
            "https://example.com/path?query=value&other=test", // Query parameters
            "https://example.com/path#fragment", // Fragment
            "https://example.com/path?q=hello%20world&lang=en", // Complex query
            "https://sub.domain.example.com:8080/path", // Subdomain and port
        ];

        for special_url in special_urls {
            let crawl_body = CrawlBody {
                urls: vec![special_url.to_string()],
                options: Some(CrawlOptions::default()),
            };

            let response = server.post("/crawl").json(&crawl_body).await;

            println!("Special URL '{}': {}", special_url, response.status_code());

            // Should handle Unicode and special characters (may succeed or fail gracefully)
            assert!(
                response.status_code() == StatusCode::OK ||
                response.status_code() == StatusCode::BAD_REQUEST,
                "Special URL should be handled: {} -> {}",
                special_url, response.status_code()
            );
        }
    }

    #[tokio::test]
    async fn test_concurrent_conflicting_operations() {
        let server = setup_edge_case_server().await;

        // Start multiple conflicting spider operations
        let mut handles = vec![];

        // Start crawl operation
        let server_clone = server.clone();
        let crawl_handle = tokio::spawn(async move {
            let crawl_body = SpiderCrawlBody {
                seed_urls: vec!["https://httpbin.org/delay/2".to_string()],
                max_depth: Some(2),
                max_pages: Some(10),
                strategy: Some("breadth_first".to_string()),
                timeout_seconds: Some(10),
                delay_ms: Some(500),
                concurrency: Some(2),
                respect_robots: Some(false),
                follow_redirects: Some(true),
            };

            server_clone.post("/spider/crawl").json(&crawl_body).await
        });
        handles.push(crawl_handle);

        // Immediately try to stop
        let server_clone = server.clone();
        let stop_handle = tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            let stop_body = SpiderControlRequest {
                action: "stop".to_string(),
            };

            server_clone.post("/spider/control").json(&stop_body).await
        });
        handles.push(stop_handle);

        // Immediately try to reset
        let server_clone = server.clone();
        let reset_handle = tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

            let reset_body = SpiderControlRequest {
                action: "reset".to_string(),
            };

            server_clone.post("/spider/control").json(&reset_body).await
        });
        handles.push(reset_handle);

        // Wait for all operations to complete
        for (i, handle) in handles.into_iter().enumerate() {
            match handle.await {
                Ok(response) => {
                    println!("Concurrent operation {} completed: {}", i, response.status_code());
                }
                Err(e) => {
                    println!("Concurrent operation {} failed: {}", i, e);
                }
            }
        }

        // System should remain stable after conflicting operations
        let status_response = server.get("/healthz").await;
        assert_eq!(status_response.status_code(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_malformed_json_payloads() {
        let server = setup_edge_case_server().await;

        let malformed_payloads = vec![
            r#"{"urls": ["https://example.com"], "options": {invalid json}}"#, // Malformed JSON
            r#"{"urls": ["https://example.com"], "options": {"concurrency": "not_a_number"}}"#, // Wrong type
            r#"{"urls": ["https://example.com"], "options": {"unknown_field": "value"}}"#, // Unknown field
            r#"{"urls": null}"#, // Null URLs
            r#"{"urls": ["https://example.com"], "options": {"render_mode": "invalid_mode"}}"#, // Invalid enum
            r#"{}"#, // Missing required fields
            r#"null"#, // Null payload
            r#""string instead of object""#, // Wrong root type
            r#"123"#, // Number instead of object
        ];

        for (i, payload) in malformed_payloads.iter().enumerate() {
            let response = server
                .post("/crawl")
                .header("Content-Type", "application/json")
                .text(payload)
                .await;

            println!("Malformed payload {}: {} -> {}",
                    i + 1, payload, response.status_code());

            // Should return bad request for malformed JSON
            assert_eq!(response.status_code(), StatusCode::BAD_REQUEST,
                      "Malformed JSON should be rejected: {}", payload);
        }
    }

    #[tokio::test]
    async fn test_memory_intensive_operations() {
        let server = setup_edge_case_server().await;

        // Test with many URLs to potentially stress memory
        let many_urls: Vec<String> = (0..100)
            .map(|i| format!("https://httpbin.org/status/200?id={}", i))
            .collect();

        let memory_test_body = CrawlBody {
            urls: many_urls,
            options: Some(CrawlOptions {
                concurrency: 10, // High concurrency
                cache_mode: "bypass".to_string(), // No cache to stress processing
                dynamic_wait_for: None,
                scroll_steps: 0,
                token_chunk_max: 5000, // Large chunks
                token_overlap: 500, // Large overlap
                render_mode: RenderMode::Html,
            }),
        };

        let response = server
            .post("/crawl")
            .json(&memory_test_body)
            .await;

        // Should handle many URLs without crashing
        println!("Memory test with 100 URLs: {}", response.status_code());

        // System should remain stable
        if response.status_code() == StatusCode::OK {
            let body: Value = response.json();
            println!("Successfully processed {} URLs", body["total_urls"]);
        } else {
            // May fail due to resource constraints, but should fail gracefully
            let body: Value = response.json();
            if body["error"].is_string() {
                println!("Memory test failed gracefully: {}", body["error"]);
            }
        }

        // Health check should still work
        let health_response = server.get("/healthz").await;
        assert_eq!(health_response.status_code(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_boundary_value_analysis() {
        let server = setup_edge_case_server().await;

        // Test boundary values for various parameters
        let boundary_tests = vec![
            // Concurrency boundaries
            (1, "Minimum concurrency"),
            (1000, "Very high concurrency"),

            // Token chunk boundaries
            (1, "Minimum token chunk"),
            (100000, "Maximum token chunk"),

            // Scroll step boundaries
            (0, "No scroll steps"),
            (1000, "Many scroll steps"),
        ];

        for (concurrency, description) in vec![(1, "min"), (1000, "max")] {
            for (token_chunk, chunk_desc) in vec![(1, "min_chunk"), (100000, "max_chunk")] {
                for (scroll_steps, scroll_desc) in vec![(0, "no_scroll"), (1000, "max_scroll")] {
                    let boundary_options = CrawlOptions {
                        concurrency: concurrency as u16,
                        cache_mode: "bypass".to_string(),
                        dynamic_wait_for: None,
                        scroll_steps,
                        token_chunk_max: token_chunk,
                        token_overlap: (token_chunk / 10).max(1),
                        render_mode: RenderMode::Html,
                    };

                    let crawl_body = CrawlBody {
                        urls: vec!["https://httpbin.org/html".to_string()],
                        options: Some(boundary_options),
                    };

                    let response = server.post("/crawl").json(&crawl_body).await;

                    println!("Boundary test {}_{}_{}: {}",
                            description, chunk_desc, scroll_desc, response.status_code());

                    // Should handle boundary values gracefully
                    assert!(
                        response.status_code() == StatusCode::OK ||
                        response.status_code() == StatusCode::BAD_REQUEST,
                        "Boundary values should be handled gracefully"
                    );

                    // Don't overwhelm the server
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    }

    #[tokio::test]
    async fn test_rapid_fire_requests() {
        let server = setup_edge_case_server().await;

        // Send many requests rapidly to test rate limiting and stability
        let mut handles = vec![];

        for i in 0..20 {
            let server_clone = server.clone();
            let handle = tokio::spawn(async move {
                let crawl_body = CrawlBody {
                    urls: vec![format!("https://httpbin.org/status/200?rapid={}", i)],
                    options: Some(CrawlOptions {
                        concurrency: 1,
                        cache_mode: "bypass".to_string(),
                        dynamic_wait_for: None,
                        scroll_steps: 0,
                        token_chunk_max: 500,
                        token_overlap: 50,
                        render_mode: RenderMode::Html,
                    }),
                };

                server_clone.post("/crawl").json(&crawl_body).await
            });

            handles.push(handle);
        }

        // Wait for all rapid requests
        let mut success_count = 0;
        let mut rate_limited_count = 0;
        let mut error_count = 0;

        for handle in handles {
            match handle.await {
                Ok(response) => {
                    match response.status_code() {
                        StatusCode::OK => success_count += 1,
                        StatusCode::TOO_MANY_REQUESTS => rate_limited_count += 1,
                        _ => error_count += 1,
                    }
                }
                Err(_) => error_count += 1,
            }
        }

        println!("Rapid fire results: {} success, {} rate limited, {} errors",
                success_count, rate_limited_count, error_count);

        // At least some requests should complete, system should remain stable
        assert!(success_count > 0 || rate_limited_count > 0,
               "Some requests should complete or be rate limited");

        // System should still be responsive
        let health_response = server.get("/healthz").await;
        assert_eq!(health_response.status_code(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_content_size_limits() {
        let server = setup_edge_case_server().await;

        // Test with URLs that might return very large content
        let large_content_urls = vec![
            "https://httpbin.org/bytes/1024",      // 1KB
            "https://httpbin.org/bytes/10240",     // 10KB
            "https://httpbin.org/bytes/102400",    // 100KB
            "https://httpbin.org/bytes/1048576",   // 1MB (might timeout or fail)
        ];

        for url in large_content_urls {
            let crawl_body = CrawlBody {
                urls: vec![url.to_string()],
                options: Some(CrawlOptions {
                    concurrency: 1,
                    cache_mode: "bypass".to_string(),
                    dynamic_wait_for: None,
                    scroll_steps: 0,
                    token_chunk_max: 2000,
                    token_overlap: 200,
                    render_mode: RenderMode::Html,
                }),
            };

            let response = server.post("/crawl").json(&crawl_body).await;

            println!("Large content test {}: {}", url, response.status_code());

            // Should handle large content gracefully (succeed, timeout, or reject)
            assert!(
                response.status_code() == StatusCode::OK ||
                response.status_code() == StatusCode::REQUEST_TIMEOUT ||
                response.status_code() == StatusCode::PAYLOAD_TOO_LARGE ||
                response.status_code() == StatusCode::INTERNAL_SERVER_ERROR,
                "Large content should be handled gracefully: {} -> {}",
                url, response.status_code()
            );

            // Delay between large content tests
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }
}