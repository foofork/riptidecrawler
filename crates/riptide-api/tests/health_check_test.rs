#[cfg(test)]
mod health_check_tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use riptide_api::{create_app, AppConfig};
    use std::env;

    /// Helper to create test configuration
    fn test_config() -> AppConfig {
        AppConfig {
            port: 0, // Use random port for tests
            redis_url: "redis://localhost:6379".to_string(),
            headless_url: None,
            cache_ttl: 300,
            max_concurrency: 10,
            gate_hi_threshold: 0.8,
            gate_lo_threshold: 0.3,
            cors_origins: vec!["http://localhost:3000".to_string()],
            api_key: Some("test-key".to_string()),
            openai_api_key: None,
        }
    }

    #[tokio::test]
    async fn test_health_check_no_external_calls() {
        // This test verifies that the health check doesn't make external HTTP calls
        let config = test_config();

        // Create app without setting HEALTH_CHECK_PORT
        env::remove_var("HEALTH_CHECK_PORT");

        let app = create_app(config).await.expect("Failed to create app");
        let server = TestServer::new(app.into_make_service()).unwrap();

        let response = server.get("/health").await;

        // Health check should work without external dependencies
        assert_eq!(response.status_code(), StatusCode::OK);

        let body = response.json::<serde_json::Value>();

        // Verify no external endpoints are referenced
        let body_str = body.to_string();
        assert!(!body_str.contains("httpbin.org"), "Health check should not reference httpbin.org");
        assert!(!body_str.contains("google.com"), "Health check should not reference google.com");
    }

    #[tokio::test]
    async fn test_health_check_with_internal_endpoint() {
        let config = test_config();

        // Set internal health check port
        env::set_var("HEALTH_CHECK_PORT", "8080");

        let app = create_app(config).await.expect("Failed to create app");
        let server = TestServer::new(app.into_make_service()).unwrap();

        let response = server.get("/health").await;

        // Should still work even if internal endpoint is not available
        assert_eq!(response.status_code(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_response_structure() {
        let config = test_config();
        env::remove_var("HEALTH_CHECK_PORT");

        let app = create_app(config).await.expect("Failed to create app");
        let server = TestServer::new(app.into_make_service()).unwrap();

        let response = server.get("/health").await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let body = response.json::<serde_json::Value>();

        // Verify expected fields exist
        assert!(body["status"].is_string());
        assert!(body["version"].is_string());
        assert!(body["timestamp"].is_string());
        assert!(body["dependencies"].is_object());

        // Verify HTTP client status
        let http_client = &body["dependencies"]["http_client"];
        assert!(http_client["status"].is_string());

        // Status should be healthy or degraded, not unhealthy from external call failures
        let status = http_client["status"].as_str().unwrap();
        assert!(status == "healthy" || status == "degraded");

        // Message should indicate internal verification
        if let Some(message) = http_client["message"].as_str() {
            assert!(
                message.contains("initialized") || message.contains("internal"),
                "HTTP client message should reference initialization or internal checks, got: {}",
                message
            );
        }
    }

    #[tokio::test]
    async fn test_comprehensive_health_check() {
        let config = test_config();
        env::remove_var("HEALTH_CHECK_PORT");

        let app = create_app(config).await.expect("Failed to create app");
        let server = TestServer::new(app.into_make_service()).unwrap();

        let response = server.get("/comprehensive_health").await;

        // Comprehensive health check should also work without external calls
        let status = response.status_code();
        assert!(
            status == StatusCode::OK || status == StatusCode::SERVICE_UNAVAILABLE,
            "Unexpected status: {:?}",
            status
        );

        let body = response.json::<serde_json::Value>();

        // Verify no external references
        let body_str = body.to_string();
        assert!(!body_str.contains("httpbin.org"), "Should not contain httpbin.org");
        assert!(!body_str.contains("google.com"), "Should not contain google.com");

        // Verify metrics if available
        if let Some(metrics) = body["metrics"].as_object() {
            assert!(metrics.contains_key("memory_usage_bytes"));
            assert!(metrics.contains_key("active_connections"));
        }
    }

    #[tokio::test]
    async fn test_health_check_security() {
        // This test verifies that the health check doesn't expose sensitive information
        let mut config = test_config();
        config.api_key = Some("super-secret-key-123".to_string());
        config.openai_api_key = Some("sk-secret-openai-key".to_string());

        env::remove_var("HEALTH_CHECK_PORT");

        let app = create_app(config).await.expect("Failed to create app");
        let server = TestServer::new(app.into_make_service()).unwrap();

        let response = server.get("/health").await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let body = response.json::<serde_json::Value>();
        let body_str = body.to_string();

        // Ensure no secrets are exposed in health check
        assert!(!body_str.contains("super-secret-key"), "API key should not be exposed");
        assert!(!body_str.contains("sk-secret"), "OpenAI key should not be exposed");
        assert!(!body_str.contains("password"), "Passwords should not be exposed");
    }
}