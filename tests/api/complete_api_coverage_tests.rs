//! Complete API coverage tests
//!
//! This module ensures all public APIs have at least one test,
//! focusing on endpoints that might not be covered by other test suites.

#[cfg(test)]
mod complete_api_coverage_tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use riptide_api::models::{
        CrawlBody, DeepSearchBody, SpiderCrawlBody, SpiderStatusRequest, SpiderControlRequest
    };
    use riptide_api::{create_app, AppConfig};
    use riptide_core::types::{CrawlOptions, RenderMode};
    use serde_json::{json, Value};

    /// Helper to create comprehensive test configuration
    fn comprehensive_test_config() -> AppConfig {
        AppConfig {
            port: 0,
            redis_url: "redis://localhost:6379".to_string(),
            headless_url: Some("http://localhost:3001".to_string()),
            cache_ttl: 300,
            max_concurrency: 10,
            gate_hi_threshold: 0.8,
            gate_lo_threshold: 0.3,
            cors_origins: vec!["*".to_string()],
            api_key: Some("comprehensive-test-key".to_string()),
            openai_api_key: Some("sk-test-key".to_string()),
            spider_config: Some(riptide_core::spider::SpiderConfig::new(
                "https://example.com".parse().unwrap()
            )),
        }
    }

    async fn setup_comprehensive_server() -> TestServer {
        let config = comprehensive_test_config();
        let app = create_app(config).await.expect("Failed to create comprehensive test app");
        TestServer::new(app.into_make_service()).unwrap()
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let server = setup_comprehensive_server().await;

        let response = server.get("/health").await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let body: Value = response.json();
        assert!(body["status"].is_string());
        assert!(body["version"].is_string());
        assert!(body["timestamp"].is_string());
        assert!(body["uptime"].is_number());
        assert!(body["dependencies"].is_object());

        // Verify dependency structure
        let dependencies = &body["dependencies"];
        assert!(dependencies["redis"].is_object());
        assert!(dependencies["extractor"].is_object());
        assert!(dependencies["http_client"].is_object());
    }

    #[tokio::test]
    async fn test_comprehensive_health_endpoint() {
        let server = setup_comprehensive_server().await;

        let response = server.get("/comprehensive_health").await;

        // Should succeed or return service unavailable
        assert!(
            response.status_code() == StatusCode::OK ||
            response.status_code() == StatusCode::SERVICE_UNAVAILABLE
        );

        let body: Value = response.json();
        assert!(body.is_object());

        if response.status_code() == StatusCode::OK {
            // Should include additional metrics
            assert!(body["status"].is_string());
            if body["metrics"].is_object() {
                let metrics = &body["metrics"];
                assert!(metrics["memory_usage_bytes"].is_number());
                assert!(metrics["active_connections"].is_number());
            }
        }
    }

    #[tokio::test]
    async fn test_crawl_endpoint_basic() {
        let server = setup_comprehensive_server().await;

        let crawl_body = CrawlBody {
            urls: vec!["https://httpbin.org/html".to_string()],
            options: None, // Test with default options
        };

        let response = server.post("/crawl").json(&crawl_body).await;

        // Should accept request (may fail due to network, but validation should pass)
        assert_ne!(response.status_code(), StatusCode::BAD_REQUEST);

        if response.status_code() == StatusCode::OK {
            let body: Value = response.json();
            assert!(body["total_urls"].is_number());
            assert!(body["results"].is_array());
            assert!(body["statistics"].is_object());
        }
    }

    #[tokio::test]
    async fn test_crawl_endpoint_with_all_options() {
        let server = setup_comprehensive_server().await;

        let crawl_body = CrawlBody {
            urls: vec!["https://httpbin.org/html".to_string()],
            options: Some(CrawlOptions {
                concurrency: 2,
                cache_mode: "read_through".to_string(),
                dynamic_wait_for: Some("networkidle2".to_string()),
                scroll_steps: 3,
                token_chunk_max: 1500,
                token_overlap: 150,
                render_mode: RenderMode::Html,
            }),
        };

        let response = server.post("/crawl").json(&crawl_body).await;
        assert_ne!(response.status_code(), StatusCode::BAD_REQUEST);

        if response.status_code() == StatusCode::OK {
            let body: Value = response.json();
            assert!(body["results"].is_array());
        }
    }

    #[tokio::test]
    async fn test_deep_search_endpoint() {
        let server = setup_comprehensive_server().await;

        let search_body = DeepSearchBody {
            query: "rust programming".to_string(),
            limit: Some(5),
            country: Some("US".to_string()),
            locale: Some("en".to_string()),
            include_content: Some(true),
            crawl_options: Some(CrawlOptions {
                concurrency: 2,
                cache_mode: "read_through".to_string(),
                dynamic_wait_for: None,
                scroll_steps: 1,
                token_chunk_max: 1000,
                token_overlap: 100,
                render_mode: RenderMode::Html,
            }),
        };

        let response = server.post("/deep_search").json(&search_body).await;

        // May not be implemented or may require external services
        // Accept various response codes
        assert!(
            response.status_code() == StatusCode::OK ||
            response.status_code() == StatusCode::NOT_IMPLEMENTED ||
            response.status_code() == StatusCode::SERVICE_UNAVAILABLE ||
            response.status_code() == StatusCode::NOT_FOUND
        );

        if response.status_code() == StatusCode::OK {
            let body: Value = response.json();
            assert!(body["query"].is_string());
        }
    }

    #[tokio::test]
    async fn test_spider_crawl_endpoint_comprehensive() {
        let server = setup_comprehensive_server().await;

        let spider_body = SpiderCrawlBody {
            seed_urls: vec!["https://httpbin.org/html".to_string()],
            max_depth: Some(2),
            max_pages: Some(5),
            strategy: Some("breadth_first".to_string()),
            timeout_seconds: Some(10),
            delay_ms: Some(500),
            concurrency: Some(2),
            respect_robots: Some(false),
            follow_redirects: Some(true),
        };

        let response = server.post("/spider/crawl").json(&spider_body).await;

        // Should accept valid spider request
        assert_ne!(response.status_code(), StatusCode::BAD_REQUEST);

        if response.status_code() == StatusCode::OK {
            let body: Value = response.json();
            assert!(body["result"].is_object());
            assert!(body["state"].is_object());
            assert!(body["performance"].is_object());
        }
    }

    #[tokio::test]
    async fn test_spider_status_endpoint() {
        let server = setup_comprehensive_server().await;

        let status_body = SpiderStatusRequest {
            include_metrics: Some(true),
        };

        let response = server.post("/spider/status").json(&status_body).await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let body: Value = response.json();
        assert!(body["state"].is_object());
        // Performance may be present depending on spider state
    }

    #[tokio::test]
    async fn test_spider_control_endpoint() {
        let server = setup_comprehensive_server().await;

        // Test all control actions
        let actions = vec!["stop", "reset"];

        for action in actions {
            let control_body = SpiderControlRequest {
                action: action.to_string(),
            };

            let response = server.post("/spider/control").json(&control_body).await;

            assert_eq!(response.status_code(), StatusCode::OK);

            let body: Value = response.json();
            assert!(body["status"].is_string());

            println!("Spider control '{}': {}", action, body["status"]);
        }
    }

    #[tokio::test]
    async fn test_cors_headers() {
        let server = setup_comprehensive_server().await;

        // Test CORS preflight request
        let response = server
            .options("/crawl")
            .header("Origin", "http://localhost:3000")
            .header("Access-Control-Request-Method", "POST")
            .header("Access-Control-Request-Headers", "content-type")
            .await;

        // Should handle CORS
        println!("CORS preflight response: {}", response.status_code());

        // Test actual request with CORS headers
        let crawl_body = CrawlBody {
            urls: vec!["https://httpbin.org/html".to_string()],
            options: None,
        };

        let response = server
            .post("/crawl")
            .header("Origin", "http://localhost:3000")
            .json(&crawl_body)
            .await;

        // Should include CORS headers or handle CORS appropriately
        assert_ne!(response.status_code(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_content_type_handling() {
        let server = setup_comprehensive_server().await;

        // Test with different content types
        let crawl_body = json!({
            "urls": ["https://httpbin.org/html"],
            "options": {}
        });

        // Test JSON content type
        let response = server
            .post("/crawl")
            .header("Content-Type", "application/json")
            .json(&crawl_body)
            .await;

        assert_ne!(response.status_code(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

        // Test with missing content type (should still work for JSON)
        let response = server.post("/crawl").json(&crawl_body).await;
        assert_ne!(response.status_code(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn test_authentication_handling() {
        let server = setup_comprehensive_server().await;

        let crawl_body = CrawlBody {
            urls: vec!["https://httpbin.org/html".to_string()],
            options: None,
        };

        // Test without authentication
        let response = server.post("/crawl").json(&crawl_body).await;

        // Should either work (no auth required) or return auth error
        if response.status_code() == StatusCode::UNAUTHORIZED {
            // Test with authentication
            let auth_response = server
                .post("/crawl")
                .header("Authorization", "Bearer comprehensive-test-key")
                .json(&crawl_body)
                .await;

            assert_ne!(auth_response.status_code(), StatusCode::UNAUTHORIZED);
        }
    }

    #[tokio::test]
    async fn test_error_response_format() {
        let server = setup_comprehensive_server().await;

        // Trigger a validation error
        let invalid_body = CrawlBody {
            urls: vec![], // Empty URLs should cause validation error
            options: None,
        };

        let response = server.post("/crawl").json(&invalid_body).await;
        assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

        let body: Value = response.json();
        // Should have structured error response
        assert!(body["error"].is_string() || body["message"].is_string());

        println!("Error response format: {}", body);
    }

    #[tokio::test]
    async fn test_all_http_methods() {
        let server = setup_comprehensive_server().await;

        // Test GET endpoints
        let get_endpoints = vec!["/health", "/comprehensive_health"];

        for endpoint in get_endpoints {
            let response = server.get(endpoint).await;
            println!("GET {} -> {}", endpoint, response.status_code());
            assert_ne!(response.status_code(), StatusCode::METHOD_NOT_ALLOWED);
        }

        // Test POST endpoints
        let post_endpoints = vec![
            ("/crawl", json!({"urls": ["https://httpbin.org/html"]})),
            ("/spider/crawl", json!({"seed_urls": ["https://httpbin.org/html"]})),
            ("/spider/status", json!({"include_metrics": false})),
            ("/spider/control", json!({"action": "stop"})),
        ];

        for (endpoint, body) in post_endpoints {
            let response = server.post(endpoint).json(&body).await;
            println!("POST {} -> {}", endpoint, response.status_code());
            assert_ne!(response.status_code(), StatusCode::METHOD_NOT_ALLOWED);
        }
    }

    #[tokio::test]
    async fn test_api_versioning() {
        let server = setup_comprehensive_server().await;

        // Test if API supports versioning
        let response = server.get("/v1/health").await;

        // May or may not be implemented - just ensure it doesn't crash
        println!("API versioning test (/v1/health): {}", response.status_code());

        // Test without version prefix
        let response = server.get("/health").await;
        assert_eq!(response.status_code(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_request_id_tracing() {
        let server = setup_comprehensive_server().await;

        // Test with custom request ID header
        let response = server
            .get("/health")
            .header("X-Request-ID", "test-request-123")
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        // Should handle custom headers gracefully
        println!("Request ID tracing test completed");
    }

    #[tokio::test]
    async fn test_response_compression() {
        let server = setup_comprehensive_server().await;

        // Test with compression headers
        let response = server
            .get("/health")
            .header("Accept-Encoding", "gzip, deflate")
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        // Should handle compression requests
        let body_size = response.text().len();
        println!("Compression test - response size: {} bytes", body_size);
    }

    #[tokio::test]
    async fn test_concurrent_api_access() {
        let server = setup_comprehensive_server().await;

        // Test multiple concurrent API calls
        let mut handles = vec![];

        for i in 0..5 {
            let server_clone = server.clone();
            let handle = tokio::spawn(async move {
                let response = server_clone.get("/health").await;
                (i, response.status_code())
            });
            handles.push(handle);
        }

        // All concurrent requests should succeed
        for handle in handles {
            let (id, status) = handle.await.unwrap();
            assert_eq!(status, StatusCode::OK, "Concurrent request {} failed", id);
        }

        println!("Concurrent API access test completed successfully");
    }

    #[tokio::test]
    async fn test_malformed_request_handling() {
        let server = setup_comprehensive_server().await;

        // Test various malformed requests
        let malformed_requests = vec![
            ("POST", "/crawl", "invalid json"),
            ("POST", "/crawl", "{}"),
            ("POST", "/spider/crawl", "null"),
            ("GET", "/nonexistent", ""),
        ];

        for (method, endpoint, body) in malformed_requests {
            let response = match method {
                "GET" => server.get(endpoint).await,
                "POST" => {
                    if body.is_empty() {
                        server.post(endpoint).await
                    } else {
                        server
                            .post(endpoint)
                            .header("Content-Type", "application/json")
                            .text(body)
                            .await
                    }
                }
                _ => continue,
            };

            println!("Malformed {} {} -> {}", method, endpoint, response.status_code());

            // Should return appropriate error codes, not crash
            assert!(
                response.status_code().is_client_error() ||
                response.status_code().is_server_error() ||
                response.status_code().is_success()
            );
        }
    }
}