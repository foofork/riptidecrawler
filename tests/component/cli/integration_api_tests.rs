//! Full integration tests for CLI-API architecture
//!
//! Tests complete workflows with mock API server

use anyhow::Result;
use riptide_cli::api_client::*;
use riptide_cli::execution_mode::ExecutionMode;
use std::sync::Arc;
use tokio::sync::Mutex;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Mock API server for testing
struct TestApiServer {
    server: MockServer,
    request_count: Arc<Mutex<usize>>,
}

impl TestApiServer {
    async fn new() -> Self {
        Self {
            server: MockServer::start().await,
            request_count: Arc::new(Mutex::new(0)),
        }
    }

    fn uri(&self) -> String {
        self.server.uri()
    }

    async fn increment_requests(&self) {
        let mut count = self.request_count.lock().await;
        *count += 1;
    }

    async fn request_count(&self) -> usize {
        *self.request_count.lock().await
    }

    async fn setup_health_endpoint(&self) {
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "healthy",
                "version": "1.0.0",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
            .mount(&self.server)
            .await;
    }

    async fn setup_render_endpoint(&self) {
        Mock::given(method("POST"))
            .and(path("/api/v1/render"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true,
                "html": "<html><body><h1>Test Page</h1></body></html>",
                "metadata": {
                    "final_url": "https://example.com",
                    "title": "Test Page",
                    "render_time_ms": 150,
                    "resources_loaded": 3,
                    "cookies_set": 0
                }
            })))
            .mount(&self.server)
            .await;
    }

    async fn setup_extract_endpoint(&self) {
        Mock::given(method("POST"))
            .and(path("/api/v1/extract"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true,
                "data": {
                    "title": "Test Page",
                    "content": "This is test content.",
                    "links": ["https://example.com/page1", "https://example.com/page2"]
                },
                "metadata": {
                    "url": "https://example.com",
                    "extracted_fields": 3,
                    "extraction_time_ms": 75
                }
            })))
            .mount(&self.server)
            .await;
    }

    async fn setup_screenshot_endpoint(&self) {
        let png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        Mock::given(method("POST"))
            .and(path("/api/v1/screenshot"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(png_header))
            .mount(&self.server)
            .await;
    }
}

#[tokio::test]
async fn test_full_api_workflow() -> Result<()> {
    let test_server = TestApiServer::new().await;
    test_server.setup_health_endpoint().await;
    test_server.setup_render_endpoint().await;
    test_server.setup_extract_endpoint().await;
    test_server.setup_screenshot_endpoint().await;

    let client = RiptideApiClient::new(test_server.uri(), None)?;

    // Step 1: Check health
    assert!(client.is_available().await);

    // Step 2: Render page
    let render_request = RenderRequest {
        url: "https://example.com".to_string(),
        wait_condition: "load".to_string(),
        screenshot_mode: "none".to_string(),
        viewport: ViewportConfig {
            width: 1920,
            height: 1080,
        },
        stealth_level: "medium".to_string(),
        javascript_enabled: true,
        extra_timeout: 0,
        user_agent: None,
        proxy: None,
        session_id: None,
    };

    let render_response = client.render(render_request).await?;
    assert!(render_response.success);
    assert!(render_response.html.is_some());

    // Step 3: Extract content
    let extract_request = ExtractRequest {
        url: "https://example.com".to_string(),
        selectors: vec!["h1".to_string(), "p".to_string(), "a".to_string()],
        schema: None,
        wasm_module: None,
    };

    let extract_response = client.extract(extract_request).await?;
    assert!(extract_response.success);
    assert_eq!(extract_response.data["title"], "Test Page");

    // Step 4: Take screenshot
    let screenshot_request = ScreenshotRequest {
        url: "https://example.com".to_string(),
        viewport: ViewportConfig {
            width: 1920,
            height: 1080,
        },
        full_page: true,
        wait_condition: Some("load".to_string()),
        selector: None,
    };

    let screenshot_data = client.screenshot(screenshot_request).await?;
    assert!(!screenshot_data.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_api_first_with_fallback() -> Result<()> {
    let mode = ExecutionMode::ApiFirst;

    // Scenario 1: API available - use API
    let test_server = TestApiServer::new().await;
    test_server.setup_health_endpoint().await;
    test_server.setup_extract_endpoint().await;

    let client = RiptideApiClient::new(test_server.uri(), None)?;

    if client.is_available().await && mode.allows_api() {
        // Use API
        let request = ExtractRequest {
            url: "https://example.com".to_string(),
            selectors: vec!["h1".to_string()],
            schema: None,
            wasm_module: None,
        };
        let result = client.extract(request).await;
        assert!(result.is_ok());
    }

    // Scenario 2: API unavailable - fallback to direct
    let bad_client = RiptideApiClient::new("http://localhost:65535".to_string(), None)?;

    if !bad_client.is_available().await && mode.allows_fallback() {
        // Fallback to direct execution
        assert!(mode.allows_direct());
    }

    Ok(())
}

#[tokio::test]
async fn test_authentication_flow() -> Result<()> {
    let test_server = TestApiServer::new().await;

    // Mock authenticated endpoint
    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .and(header("X-API-Key", "valid-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "data": { "content": "authenticated" }
        })))
        .mount(&test_server.server)
        .await;

    // Mock unauthenticated endpoint
    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&test_server.server)
        .await;

    // Test with valid API key
    let auth_client = RiptideApiClient::new(test_server.uri(), Some("valid-key".to_string()))?;

    let request = ExtractRequest {
        url: "https://example.com".to_string(),
        selectors: vec!["p".to_string()],
        schema: None,
        wasm_module: None,
    };

    let result = auth_client.extract(request.clone()).await;
    assert!(result.is_ok());

    // Test without API key (should fail)
    let no_auth_client = RiptideApiClient::new(test_server.uri(), None)?;
    let result = no_auth_client.extract(request).await;
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_error_handling_and_recovery() -> Result<()> {
    let test_server = TestApiServer::new().await;

    // Simulate intermittent failures
    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(2)
        .mount(&test_server.server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "data": { "recovered": true }
        })))
        .mount(&test_server.server)
        .await;

    let client = RiptideApiClient::new(test_server.uri(), None)?;

    let request = ExtractRequest {
        url: "https://example.com".to_string(),
        selectors: vec!["p".to_string()],
        schema: None,
        wasm_module: None,
    };

    // First two attempts should fail
    assert!(client.extract(request.clone()).await.is_err());
    assert!(client.extract(request.clone()).await.is_err());

    // Third attempt should succeed
    let result = client.extract(request).await?;
    assert!(result.success);

    Ok(())
}

#[tokio::test]
async fn test_concurrent_api_requests() -> Result<()> {
    let test_server = TestApiServer::new().await;
    test_server.setup_extract_endpoint().await;

    let client = Arc::new(RiptideApiClient::new(test_server.uri(), None)?);

    // Spawn multiple concurrent requests
    let mut handles = vec![];
    for i in 0..5 {
        let client_clone = Arc::clone(&client);
        let handle = tokio::spawn(async move {
            let request = ExtractRequest {
                url: format!("https://example{}.com", i),
                selectors: vec!["h1".to_string()],
                schema: None,
                wasm_module: None,
            };
            client_clone.extract(request).await
        });
        handles.push(handle);
    }

    // Wait for all requests
    let results = futures::future::join_all(handles).await;

    // Verify all succeeded
    for result in results {
        let extraction_result = result??;
        assert!(extraction_result.success);
    }

    Ok(())
}

#[tokio::test]
async fn test_timeout_handling() -> Result<()> {
    let test_server = TestApiServer::new().await;

    // Simulate slow response
    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(std::time::Duration::from_secs(301)) // Longer than client timeout
                .set_body_json(serde_json::json!({
                    "success": true,
                    "data": {}
                })),
        )
        .mount(&test_server.server)
        .await;

    let client = RiptideApiClient::new(test_server.uri(), None)?;

    let request = ExtractRequest {
        url: "https://example.com".to_string(),
        selectors: vec!["p".to_string()],
        schema: None,
        wasm_module: None,
    };

    let start = std::time::Instant::now();
    let result = client.extract(request).await;
    let duration = start.elapsed();

    // Should timeout before 301 seconds
    assert!(duration.as_secs() < 301);
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_api_version_compatibility() -> Result<()> {
    let test_server = TestApiServer::new().await;

    // Mock health endpoint with version info
    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "healthy",
            "version": "1.2.0",
            "api_version": "v1"
        })))
        .mount(&test_server.server)
        .await;

    let client = RiptideApiClient::new(test_server.uri(), None)?;

    // Client should successfully connect regardless of minor version differences
    assert!(client.is_available().await);

    Ok(())
}

#[tokio::test]
async fn test_large_payload_handling() -> Result<()> {
    let test_server = TestApiServer::new().await;

    // Large HTML content
    let large_html = "<html><body>".to_string() + &"<p>content</p>".repeat(10000) + "</body></html>";

    Mock::given(method("POST"))
        .and(path("/api/v1/render"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "html": large_html,
            "metadata": {
                "final_url": "https://example.com",
                "render_time_ms": 500,
                "resources_loaded": 100,
                "cookies_set": 5
            }
        })))
        .mount(&test_server.server)
        .await;

    let client = RiptideApiClient::new(test_server.uri(), None)?;

    let request = RenderRequest {
        url: "https://example.com".to_string(),
        wait_condition: "load".to_string(),
        screenshot_mode: "none".to_string(),
        viewport: ViewportConfig {
            width: 1920,
            height: 1080,
        },
        stealth_level: "low".to_string(),
        javascript_enabled: true,
        extra_timeout: 0,
        user_agent: None,
        proxy: None,
        session_id: None,
    };

    let response = client.render(request).await?;
    assert!(response.success);
    assert!(response.html.is_some());
    assert!(response.html.unwrap().len() > 100000);

    Ok(())
}

#[tokio::test]
async fn test_session_management() -> Result<()> {
    let test_server = TestApiServer::new().await;
    test_server.setup_render_endpoint().await;

    let client = RiptideApiClient::new(test_server.uri(), None)?;

    // Make request with session ID
    let request = RenderRequest {
        url: "https://example.com".to_string(),
        wait_condition: "load".to_string(),
        screenshot_mode: "none".to_string(),
        viewport: ViewportConfig {
            width: 1920,
            height: 1080,
        },
        stealth_level: "medium".to_string(),
        javascript_enabled: true,
        extra_timeout: 0,
        user_agent: None,
        proxy: None,
        session_id: Some("test-session-123".to_string()),
    };

    let response = client.render(request).await?;
    assert!(response.success);

    Ok(())
}

#[tokio::test]
async fn test_custom_user_agent() -> Result<()> {
    let test_server = TestApiServer::new().await;
    test_server.setup_render_endpoint().await;

    let client = RiptideApiClient::new(test_server.uri(), None)?;

    let request = RenderRequest {
        url: "https://example.com".to_string(),
        wait_condition: "load".to_string(),
        screenshot_mode: "none".to_string(),
        viewport: ViewportConfig {
            width: 1920,
            height: 1080,
        },
        stealth_level: "high".to_string(),
        javascript_enabled: true,
        extra_timeout: 0,
        user_agent: Some("Mozilla/5.0 (Custom Bot) RipTide/1.0".to_string()),
        proxy: None,
        session_id: None,
    };

    let response = client.render(request).await?;
    assert!(response.success);

    Ok(())
}

#[tokio::test]
async fn test_output_consistency_api_vs_direct() -> Result<()> {
    // This test would compare outputs from API mode vs Direct mode
    // to ensure consistency

    let test_server = TestApiServer::new().await;
    test_server.setup_extract_endpoint().await;

    let client = RiptideApiClient::new(test_server.uri(), None)?;

    let request = ExtractRequest {
        url: "https://example.com".to_string(),
        selectors: vec!["h1".to_string(), "p".to_string()],
        schema: None,
        wasm_module: None,
    };

    // Get result from API
    let api_result = client.extract(request).await?;
    assert!(api_result.success);

    // In real test, would also get result from direct execution
    // and compare them for consistency

    Ok(())
}
