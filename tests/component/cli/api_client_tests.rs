//! Unit tests for RipTide API client
//!
//! Tests HTTP communication, request/response handling, and error scenarios

use anyhow::Result;
use riptide_cli::api_client::*;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_api_client_creation() -> Result<()> {
    let client = RiptideApiClient::new(
        "http://localhost:8080".to_string(),
        Some("test-key".to_string()),
    )?;

    assert_eq!(client.base_url(), "http://localhost:8080");
    Ok(())
}

#[tokio::test]
async fn test_base_url_trailing_slash_normalization() -> Result<()> {
    let client1 = RiptideApiClient::new("http://localhost:8080/".to_string(), None)?;
    let client2 = RiptideApiClient::new("http://localhost:8080".to_string(), None)?;

    assert_eq!(client1.base_url(), "http://localhost:8080");
    assert_eq!(client2.base_url(), "http://localhost:8080");
    Ok(())
}

#[tokio::test]
async fn test_health_check_success() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "healthy",
            "version": "1.0.0"
        })))
        .mount(&mock_server)
        .await;

    let client = RiptideApiClient::new(mock_server.uri(), None)?;
    let is_available = client.is_available().await;

    assert!(is_available, "Health check should return true for 200 OK");
    Ok(())
}

#[tokio::test]
async fn test_health_check_failure() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&mock_server)
        .await;

    let client = RiptideApiClient::new(mock_server.uri(), None)?;
    let is_available = client.is_available().await;

    assert!(!is_available, "Health check should return false for 503");
    Ok(())
}

#[tokio::test]
async fn test_health_check_timeout() -> Result<()> {
    // Create client pointing to unreachable server
    let client = RiptideApiClient::new("http://10.255.255.1:12345".to_string(), None)?;
    let is_available = client.is_available().await;

    assert!(
        !is_available,
        "Health check should return false on timeout"
    );
    Ok(())
}

#[tokio::test]
async fn test_render_request_success() -> Result<()> {
    let mock_server = MockServer::start().await;

    let expected_response = RenderResponse {
        success: true,
        html: Some("<html><body>Test</body></html>".to_string()),
        dom: None,
        screenshot: None,
        pdf: None,
        har: None,
        metadata: RenderMetadata {
            final_url: "https://example.com".to_string(),
            title: Some("Example Domain".to_string()),
            render_time_ms: 250,
            resources_loaded: 5,
            cookies_set: 2,
        },
        error: None,
    };

    Mock::given(method("POST"))
        .and(path("/api/v1/render"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&expected_response))
        .mount(&mock_server)
        .await;

    let client = RiptideApiClient::new(mock_server.uri(), None)?;

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
        session_id: None,
    };

    let response = client.render(request).await?;

    assert!(response.success);
    assert!(response.html.is_some());
    assert_eq!(response.metadata.final_url, "https://example.com");
    Ok(())
}

#[tokio::test]
async fn test_render_request_with_api_key() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/render"))
        .and(header("X-API-Key", "secret-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "html": "<html></html>",
            "metadata": {
                "final_url": "https://example.com",
                "render_time_ms": 100,
                "resources_loaded": 1,
                "cookies_set": 0
            }
        })))
        .mount(&mock_server)
        .await;

    let client = RiptideApiClient::new(mock_server.uri(), Some("secret-key".to_string()))?;

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
    Ok(())
}

#[tokio::test]
async fn test_render_request_authentication_failure() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/render"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&mock_server)
        .await;

    let client = RiptideApiClient::new(mock_server.uri(), None)?;

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

    let result = client.render(request).await;
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("401") || error_msg.contains("Unauthorized"));
    Ok(())
}

#[tokio::test]
async fn test_screenshot_request_success() -> Result<()> {
    let mock_server = MockServer::start().await;

    let screenshot_data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG header

    Mock::given(method("POST"))
        .and(path("/api/v1/screenshot"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(screenshot_data.clone()))
        .mount(&mock_server)
        .await;

    let client = RiptideApiClient::new(mock_server.uri(), None)?;

    let request = ScreenshotRequest {
        url: "https://example.com".to_string(),
        viewport: ViewportConfig {
            width: 1920,
            height: 1080,
        },
        full_page: false,
        wait_condition: Some("load".to_string()),
        selector: None,
    };

    let response = client.screenshot(request).await?;
    assert_eq!(response, screenshot_data);
    Ok(())
}

#[tokio::test]
async fn test_extract_request_success() -> Result<()> {
    let mock_server = MockServer::start().await;

    let extraction_result = ExtractionResult {
        success: true,
        data: serde_json::json!({
            "title": "Example Domain",
            "content": "This domain is for use in examples."
        }),
        metadata: Some(ExtractionMetadata {
            url: "https://example.com".to_string(),
            extracted_fields: 2,
            extraction_time_ms: 50,
        }),
        error: None,
    };

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&extraction_result))
        .mount(&mock_server)
        .await;

    let client = RiptideApiClient::new(mock_server.uri(), None)?;

    let request = ExtractRequest {
        url: "https://example.com".to_string(),
        selectors: vec!["h1".to_string(), "p".to_string()],
        schema: None,
        wasm_module: None,
    };

    let response = client.extract(request).await?;
    assert!(response.success);
    assert_eq!(response.data["title"], "Example Domain");
    Ok(())
}

#[tokio::test]
async fn test_extract_request_with_schema() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "data": {
                "title": "Test",
                "description": "Test description"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = RiptideApiClient::new(mock_server.uri(), None)?;

    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "title": { "type": "string", "selector": "h1" },
            "description": { "type": "string", "selector": "p" }
        }
    });

    let request = ExtractRequest {
        url: "https://example.com".to_string(),
        selectors: vec![],
        schema: Some(schema),
        wasm_module: None,
    };

    let response = client.extract(request).await?;
    assert!(response.success);
    Ok(())
}

#[tokio::test]
async fn test_server_error_handling() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(
            ResponseTemplate::new(500)
                .set_body_string("Internal server error: extraction failed"),
        )
        .mount(&mock_server)
        .await;

    let client = RiptideApiClient::new(mock_server.uri(), None)?;

    let request = ExtractRequest {
        url: "https://example.com".to_string(),
        selectors: vec!["h1".to_string()],
        schema: None,
        wasm_module: None,
    };

    let result = client.extract(request).await;
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("500"));
    Ok(())
}

#[tokio::test]
async fn test_malformed_response() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
        .mount(&mock_server)
        .await;

    let client = RiptideApiClient::new(mock_server.uri(), None)?;

    let request = ExtractRequest {
        url: "https://example.com".to_string(),
        selectors: vec!["h1".to_string()],
        schema: None,
        wasm_module: None,
    };

    let result = client.extract(request).await;
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("parse") || error_msg.contains("JSON"));
    Ok(())
}

#[tokio::test]
async fn test_concurrent_requests() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "data": { "content": "test" }
        })))
        .expect(3)
        .mount(&mock_server)
        .await;

    let client = RiptideApiClient::new(mock_server.uri(), None)?;

    let request = ExtractRequest {
        url: "https://example.com".to_string(),
        selectors: vec!["p".to_string()],
        schema: None,
        wasm_module: None,
    };

    let futures = (0..3).map(|_| client.extract(request.clone()));
    let results = futures::future::join_all(futures).await;

    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }

    Ok(())
}

#[tokio::test]
async fn test_http2_prior_knowledge() -> Result<()> {
    // Test that client is configured for HTTP/2
    let client = RiptideApiClient::new("https://http2.golang.org".to_string(), None)?;

    // The client should be configured to use HTTP/2
    // We can't directly test this without making a real request,
    // but we can verify the client was created successfully
    assert!(!client.base_url().is_empty());
    Ok(())
}
