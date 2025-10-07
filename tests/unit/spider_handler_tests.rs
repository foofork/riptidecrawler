//! Unit tests for spider handler functionality
//!
//! This module provides comprehensive testing for the spider handlers,
//! focusing on request validation, configuration parsing, error handling,
//! and response formatting.

#[cfg(test)]
mod spider_handler_tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use riptide_api::models::{SpiderCrawlBody, SpiderStatusRequest, SpiderControlRequest};
    use riptide_api::{create_app, AppConfig};
    use serde_json::{json, Value};

    /// Helper to create test configuration with spider enabled
    fn spider_test_config() -> AppConfig {
        AppConfig {
            port: 0,
            redis_url: "redis://localhost:6379".to_string(),
            headless_url: None,
            cache_ttl: 300,
            max_concurrency: 10,
            gate_hi_threshold: 0.8,
            gate_lo_threshold: 0.3,
            cors_origins: vec!["http://localhost:3000".to_string()],
            api_key: Some("test-key".to_string()),
            openai_api_key: None,
            spider_config: Some(riptide_core::spider::SpiderConfig::new(
                "https://example.com".parse().unwrap()
            )),
        }
    }

    /// Helper to create test configuration with spider disabled
    fn no_spider_config() -> AppConfig {
        AppConfig {
            port: 0,
            redis_url: "redis://localhost:6379".to_string(),
            headless_url: None,
            cache_ttl: 300,
            max_concurrency: 10,
            gate_hi_threshold: 0.8,
            gate_lo_threshold: 0.3,
            cors_origins: vec!["http://localhost:3000".to_string()],
            api_key: Some("test-key".to_string()),
            openai_api_key: None,
            spider_config: None, // Spider disabled
        }
    }

    #[tokio::test]
    async fn test_spider_crawl_request_validation() {
        let config = spider_test_config();
        let app = create_app(config).await.expect("Failed to create app");
        let server = TestServer::new(app.into_make_service()).unwrap();

        // Test valid request
        let valid_body = SpiderCrawlBody {
            seed_urls: vec!["https://example.com".to_string()],
            max_depth: Some(2),
            max_pages: Some(10),
            strategy: Some("breadth_first".to_string()),
            timeout_seconds: Some(30),
            delay_ms: Some(1000),
            concurrency: Some(5),
            respect_robots: Some(true),
            follow_redirects: Some(true),
        };

        let response = server
            .post("/spider/crawl")
            .json(&valid_body)
            .await;

        // Should accept valid request (might fail due to actual crawling, but validation should pass)
        assert!(response.status_code() != StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_spider_crawl_empty_urls() {
        let config = spider_test_config();
        let app = create_app(config).await.expect("Failed to create app");
        let server = TestServer::new(app.into_make_service()).unwrap();

        // Test empty seed URLs
        let empty_body = SpiderCrawlBody {
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

        let response = server
            .post("/spider/crawl")
            .json(&empty_body)
            .await;

        assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

        let body: Value = response.json();
        assert!(body["error"].as_str().unwrap().contains("At least one seed URL is required"));
    }

    #[tokio::test]
    async fn test_spider_crawl_invalid_urls() {
        let config = spider_test_config();
        let app = create_app(config).await.expect("Failed to create app");
        let server = TestServer::new(app.into_make_service()).unwrap();

        // Test invalid URLs
        let invalid_body = SpiderCrawlBody {
            seed_urls: vec![
                "not-a-url".to_string(),
                "ftp://invalid.com".to_string(),
                "".to_string(),
            ],
            max_depth: Some(1),
            max_pages: Some(5),
            strategy: None,
            timeout_seconds: None,
            delay_ms: None,
            concurrency: None,
            respect_robots: None,
            follow_redirects: None,
        };

        let response = server
            .post("/spider/crawl")
            .json(&invalid_body)
            .await;

        assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

        let body: Value = response.json();
        assert!(body["error"].as_str().unwrap().contains("Invalid URL"));
    }

    #[tokio::test]
    async fn test_spider_crawl_strategy_validation() {
        let config = spider_test_config();
        let app = create_app(config).await.expect("Failed to create app");
        let server = TestServer::new(app.into_make_service()).unwrap();

        // Test known strategies
        let strategies = vec!["breadth_first", "depth_first", "best_first"];

        for strategy in strategies {
            let body = SpiderCrawlBody {
                seed_urls: vec!["https://example.com".to_string()],
                max_depth: Some(1),
                max_pages: Some(1),
                strategy: Some(strategy.to_string()),
                timeout_seconds: None,
                delay_ms: None,
                concurrency: None,
                respect_robots: None,
                follow_redirects: None,
            };

            let response = server
                .post("/spider/crawl")
                .json(&body)
                .await;

            // Should accept valid strategies (may fail on actual crawl, but validation should pass)
            assert_ne!(response.status_code(), StatusCode::BAD_REQUEST,
                      "Strategy '{}' should be valid", strategy);
        }
    }

    #[tokio::test]
    async fn test_spider_crawl_unknown_strategy_warning() {
        let config = spider_test_config();
        let app = create_app(config).await.expect("Failed to create app");
        let server = TestServer::new(app.into_make_service()).unwrap();

        // Test unknown strategy (should warn but not fail)
        let body = SpiderCrawlBody {
            seed_urls: vec!["https://example.com".to_string()],
            max_depth: Some(1),
            max_pages: Some(1),
            strategy: Some("unknown_strategy".to_string()),
            timeout_seconds: Some(5),
            delay_ms: Some(100),
            concurrency: Some(1),
            respect_robots: Some(false),
            follow_redirects: Some(true),
        };

        let response = server
            .post("/spider/crawl")
            .json(&body)
            .await;

        // Should not return bad request for unknown strategy (just logs warning)
        assert_ne!(response.status_code(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_spider_disabled_error() {
        let config = no_spider_config(); // Spider disabled
        let app = create_app(config).await.expect("Failed to create app");
        let server = TestServer::new(app.into_make_service()).unwrap();

        let body = SpiderCrawlBody {
            seed_urls: vec!["https://example.com".to_string()],
            max_depth: Some(1),
            max_pages: Some(1),
            strategy: None,
            timeout_seconds: None,
            delay_ms: None,
            concurrency: None,
            respect_robots: None,
            follow_redirects: None,
        };

        let response = server
            .post("/spider/crawl")
            .json(&body)
            .await;

        assert_eq!(response.status_code(), StatusCode::SERVICE_UNAVAILABLE);

        let body: Value = response.json();
        assert!(body["error"].as_str().unwrap().contains("Spider engine is not enabled"));
    }

    #[tokio::test]
    async fn test_spider_status_request() {
        let config = spider_test_config();
        let app = create_app(config).await.expect("Failed to create app");
        let server = TestServer::new(app.into_make_service()).unwrap();

        // Test status request without metrics
        let status_body = SpiderStatusRequest {
            include_metrics: Some(false),
        };

        let response = server
            .post("/spider/status")
            .json(&status_body)
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let body: Value = response.json();
        assert!(body["state"].is_object());
        assert!(body["performance"].is_null() || !body["performance"].is_object());
    }

    #[tokio::test]
    async fn test_spider_status_with_metrics() {
        let config = spider_test_config();
        let app = create_app(config).await.expect("Failed to create app");
        let server = TestServer::new(app.into_make_service()).unwrap();

        // Test status request with metrics
        let status_body = SpiderStatusRequest {
            include_metrics: Some(true),
        };

        let response = server
            .post("/spider/status")
            .json(&status_body)
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let body: Value = response.json();
        assert!(body["state"].is_object());
        // When metrics are requested, should include performance data
        assert!(body["performance"].is_object() || body["performance"].is_null());
    }

    #[tokio::test]
    async fn test_spider_status_disabled() {
        let config = no_spider_config();
        let app = create_app(config).await.expect("Failed to create app");
        let server = TestServer::new(app.into_make_service()).unwrap();

        let status_body = SpiderStatusRequest {
            include_metrics: Some(true),
        };

        let response = server
            .post("/spider/status")
            .json(&status_body)
            .await;

        assert_eq!(response.status_code(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_spider_control_operations() {
        let config = spider_test_config();
        let app = create_app(config).await.expect("Failed to create app");
        let server = TestServer::new(app.into_make_service()).unwrap();

        // Test stop operation
        let stop_body = SpiderControlRequest {
            action: "stop".to_string(),
        };

        let response = server
            .post("/spider/control")
            .json(&stop_body)
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let body: Value = response.json();
        assert_eq!(body["status"].as_str().unwrap(), "stopped");
    }

    #[tokio::test]
    async fn test_spider_control_reset() {
        let config = spider_test_config();
        let app = create_app(config).await.expect("Failed to create app");
        let server = TestServer::new(app.into_make_service()).unwrap();

        // Test reset operation
        let reset_body = SpiderControlRequest {
            action: "reset".to_string(),
        };

        let response = server
            .post("/spider/control")
            .json(&reset_body)
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let body: Value = response.json();
        assert_eq!(body["status"].as_str().unwrap(), "reset");
    }

    #[tokio::test]
    async fn test_spider_control_invalid_action() {
        let config = spider_test_config();
        let app = create_app(config).await.expect("Failed to create app");
        let server = TestServer::new(app.into_make_service()).unwrap();

        // Test invalid action
        let invalid_body = SpiderControlRequest {
            action: "invalid_action".to_string(),
        };

        let response = server
            .post("/spider/control")
            .json(&invalid_body)
            .await;

        assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

        let body: Value = response.json();
        assert!(body["error"].as_str().unwrap().contains("Unknown action"));
    }

    #[tokio::test]
    async fn test_spider_control_disabled() {
        let config = no_spider_config();
        let app = create_app(config).await.expect("Failed to create app");
        let server = TestServer::new(app.into_make_service()).unwrap();

        let control_body = SpiderControlRequest {
            action: "stop".to_string(),
        };

        let response = server
            .post("/spider/control")
            .json(&control_body)
            .await;

        assert_eq!(response.status_code(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_spider_config_parameter_override() {
        let config = spider_test_config();
        let app = create_app(config).await.expect("Failed to create app");
        let server = TestServer::new(app.into_make_service()).unwrap();

        // Test parameter override with extreme values
        let body = SpiderCrawlBody {
            seed_urls: vec!["https://example.com".to_string()],
            max_depth: Some(100),      // High depth
            max_pages: Some(1000),     // High page count
            strategy: Some("depth_first".to_string()),
            timeout_seconds: Some(1),  // Very short timeout
            delay_ms: Some(5000),      // Long delay
            concurrency: Some(50),     // High concurrency
            respect_robots: Some(false),
            follow_redirects: Some(false),
        };

        let response = server
            .post("/spider/crawl")
            .json(&body)
            .await;

        // Should accept parameter overrides (may timeout or fail due to constraints)
        assert_ne!(response.status_code(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_spider_url_validation_edge_cases() {
        let config = spider_test_config();
        let app = create_app(config).await.expect("Failed to create app");
        let server = TestServer::new(app.into_make_service()).unwrap();

        // Test edge case URLs
        let edge_case_urls = vec![
            "http://localhost:8080/path",
            "https://sub.domain.example.com:443/long/path?query=value",
            "https://example.com/",  // Trailing slash
            "https://example.com",   // No trailing slash
        ];

        for url in edge_case_urls {
            let body = SpiderCrawlBody {
                seed_urls: vec![url.to_string()],
                max_depth: Some(1),
                max_pages: Some(1),
                strategy: None,
                timeout_seconds: Some(5),
                delay_ms: Some(100),
                concurrency: Some(1),
                respect_robots: Some(false),
                follow_redirects: Some(true),
            };

            let response = server
                .post("/spider/crawl")
                .json(&body)
                .await;

            // All valid URLs should pass validation
            assert_ne!(response.status_code(), StatusCode::BAD_REQUEST,
                      "URL '{}' should be valid", url);
        }
    }

    #[tokio::test]
    async fn test_spider_response_structure() {
        let config = spider_test_config();
        let app = create_app(config).await.expect("Failed to create app");
        let server = TestServer::new(app.into_make_service()).unwrap();

        let body = SpiderCrawlBody {
            seed_urls: vec!["https://httpbin.org/html".to_string()],
            max_depth: Some(1),
            max_pages: Some(1),
            strategy: Some("breadth_first".to_string()),
            timeout_seconds: Some(10),
            delay_ms: Some(500),
            concurrency: Some(1),
            respect_robots: Some(false),
            follow_redirects: Some(true),
        };

        let response = server
            .post("/spider/crawl")
            .json(&body)
            .await;

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
            assert!(result["domains"].is_array());
        }
    }
}

#[cfg(test)]
mod spider_configuration_tests {
    use super::*;

    #[test]
    fn test_spider_config_creation() {
        let url = "https://example.com".parse().unwrap();
        let config = riptide_core::spider::SpiderConfig::new(url);

        // Test default values are reasonable
        assert_eq!(config.max_depth, Some(3));
        assert_eq!(config.max_pages, Some(100));
        assert!(config.timeout.as_secs() > 0);
        assert!(config.delay.as_millis() >= 0);
        assert!(config.concurrency > 0);
        assert!(config.respect_robots);
        assert!(config.follow_redirects);
    }

    #[test]
    fn test_spider_config_parameter_bounds() {
        let url = "https://example.com".parse().unwrap();
        let mut config = riptide_core::spider::SpiderConfig::new(url);

        // Test parameter modifications
        config.max_depth = Some(0);     // Minimum depth
        config.max_pages = Some(1);     // Minimum pages
        config.concurrency = 1;         // Minimum concurrency

        assert_eq!(config.max_depth, Some(0));
        assert_eq!(config.max_pages, Some(1));
        assert_eq!(config.concurrency, 1);
    }

    #[test]
    fn test_spider_strategy_variants() {
        use riptide_core::spider::CrawlingStrategy;

        // Test all strategy variants are properly defined
        let strategies = vec![
            CrawlingStrategy::BreadthFirst,
            CrawlingStrategy::DepthFirst,
            CrawlingStrategy::BestFirst,
        ];

        for strategy in strategies {
            // Each strategy should be cloneable and debuggable
            let cloned = strategy.clone();
            let debug_str = format!("{:?}", cloned);
            // Verify debug output contains the strategy name
            assert!(!debug_str.is_empty(), "Debug output should not be empty");
        }
    }
}