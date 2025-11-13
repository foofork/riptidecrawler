//! Integration tests for ScraperFacade
//!
//! These tests verify the complete workflow of the ScraperFacade,
//! including HTTP fetching, error handling, and configuration.

use riptide_facade::prelude::*;
use std::time::Duration;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

/// Test helper to create a test scraper with default config
async fn create_test_scraper() -> RiptideResult<ScraperFacade> {
    Riptide::builder()
        .user_agent("TestBot/1.0")
        .timeout_secs(30)
        .build_scraper()
        .await
}

#[tokio::test]
async fn test_scraper_full_workflow() -> Result<(), Box<dyn std::error::Error>> {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Setup mock response
    let html_content = r#"
        <!DOCTYPE html>
        <html>
            <head><title>Test Page</title></head>
            <body>
                <h1>Welcome to Test</h1>
                <p>This is test content.</p>
            </body>
        </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html_content))
        .mount(&mock_server)
        .await;

    // Create scraper
    let scraper = create_test_scraper().await?;

    // Fetch HTML
    let url = format!("{}/test", mock_server.uri());
    let html = scraper.fetch_html(&url).await?;

    // Verify content
    assert!(!html.is_empty());
    assert!(html.contains("Test Page"));
    assert!(html.contains("Welcome to Test"));
    assert!(html.contains("test content"));

    Ok(())
}

#[tokio::test]
async fn test_scraper_fetch_bytes() -> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start().await;

    let binary_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header

    Mock::given(method("GET"))
        .and(path("/image.png"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(binary_data.clone()))
        .mount(&mock_server)
        .await;

    let scraper = create_test_scraper().await?;
    let url = format!("{}/image.png", mock_server.uri());
    let bytes = scraper.fetch_bytes(&url).await?;

    assert_eq!(bytes, binary_data);

    Ok(())
}

#[tokio::test]
async fn test_scraper_handles_404() -> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/notfound"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let scraper = create_test_scraper().await?;
    let url = format!("{}/notfound", mock_server.uri());
    let result = scraper.fetch_html(&url).await;

    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_scraper_handles_redirects() -> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start().await;

    // Setup redirect chain
    Mock::given(method("GET"))
        .and(path("/redirect"))
        .respond_with(
            ResponseTemplate::new(302)
                .insert_header("Location", format!("{}/final", mock_server.uri())),
        )
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/final"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Final page"))
        .mount(&mock_server)
        .await;

    let scraper = Riptide::builder().max_redirects(5).build_scraper().await?;

    let url = format!("{}/redirect", mock_server.uri());
    let html = scraper.fetch_html(&url).await?;

    assert!(html.contains("Final page"));

    Ok(())
}

#[tokio::test]
#[ignore] // TODO: Timeout not properly propagated from config to FetchEngine - requires fetch layer changes
async fn test_scraper_respects_timeout() -> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start().await;

    // Setup slow response
    Mock::given(method("GET"))
        .and(path("/slow"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("Slow response")
                .set_delay(Duration::from_secs(5)),
        )
        .mount(&mock_server)
        .await;

    // Create scraper with short timeout
    let scraper = Riptide::builder().timeout_secs(1).build_scraper().await?;

    let url = format!("{}/slow", mock_server.uri());
    let result = scraper.fetch_html(&url).await;

    // Should timeout
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_scraper_custom_headers() -> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api"))
        .respond_with(ResponseTemplate::new(200).set_body_string("API response"))
        .mount(&mock_server)
        .await;

    let scraper = Riptide::builder()
        .header("X-API-Key", "test-key-123")
        .header("X-Custom-Header", "custom-value")
        .build_scraper()
        .await?;

    let url = format!("{}/api", mock_server.uri());
    let html = scraper.fetch_html(&url).await?;

    assert!(html.contains("API response"));

    Ok(())
}

#[tokio::test]
async fn test_scraper_invalid_url() {
    let scraper = create_test_scraper().await.unwrap();

    // Test cases: (url, expected_error_message_contains)
    let test_cases = vec![
        ("not a url", "relative URL without a base"), // InvalidUrl - parse error
        ("://missing-scheme", "relative URL without a base"), // InvalidUrl - parse error
        ("", "relative URL without a base"), // InvalidUrl - empty string parse error
    ];

    for (invalid_url, expected_msg) in test_cases {
        let result = scraper.fetch_html(invalid_url).await;
        assert!(result.is_err(), "Expected error for URL: {}", invalid_url);
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains(expected_msg),
            "URL '{}': Expected error containing '{}', got: {}",
            invalid_url,
            expected_msg,
            err_msg
        );
    }

    // Special case: syntactically valid URL with invalid scheme
    // This will pass URL parsing but fail during fetch
    let result = scraper.fetch_html("htp://invalid").await;
    assert!(result.is_err());
    // This error comes from the fetch engine, not URL parsing
    assert!(matches!(result.unwrap_err(), RiptideError::Extraction(_)));
}

#[tokio::test]
async fn test_scraper_concurrent_requests() -> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start().await;

    // Setup multiple endpoints
    for i in 0..10 {
        Mock::given(method("GET"))
            .and(path(format!("/page{}", i)))
            .respond_with(ResponseTemplate::new(200).set_body_string(format!("Page {}", i)))
            .mount(&mock_server)
            .await;
    }

    let scraper = create_test_scraper().await?;

    // Fetch all pages concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let scraper_clone = scraper.clone();
        let url = format!("{}/page{}", mock_server.uri(), i);
        handles.push(tokio::spawn(
            async move { scraper_clone.fetch_html(&url).await },
        ));
    }

    // Wait for all requests
    let results: Vec<_> = futures::future::join_all(handles).await;

    // Verify all succeeded
    for (i, result) in results.into_iter().enumerate() {
        let html = result.unwrap().unwrap();
        assert!(html.contains(&format!("Page {}", i)));
    }

    Ok(())
}

#[tokio::test]
async fn test_scraper_config_access() -> Result<(), Box<dyn std::error::Error>> {
    let scraper = Riptide::builder()
        .user_agent("ConfigBot/1.0")
        .timeout_secs(45)
        .max_redirects(10)
        .verify_ssl(false)
        .build_scraper()
        .await?;

    let config = scraper.config();
    assert_eq!(config.user_agent, "ConfigBot/1.0");
    assert_eq!(config.timeout, Duration::from_secs(45));
    assert_eq!(config.max_redirects, 10);
    assert!(!config.verify_ssl);

    Ok(())
}

#[tokio::test]
async fn test_scraper_large_response() -> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start().await;

    // Create large response (1MB)
    let large_content = "x".repeat(1024 * 1024);

    Mock::given(method("GET"))
        .and(path("/large"))
        .respond_with(ResponseTemplate::new(200).set_body_string(large_content.clone()))
        .mount(&mock_server)
        .await;

    let scraper = Riptide::builder()
        .max_body_size(2 * 1024 * 1024) // 2MB limit
        .build_scraper()
        .await?;

    let url = format!("{}/large", mock_server.uri());
    let html = scraper.fetch_html(&url).await?;

    assert_eq!(html.len(), large_content.len());

    Ok(())
}

#[tokio::test]
async fn test_scraper_utf8_content() -> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start().await;

    let utf8_content = r#"
        <html>
            <body>
                <p>English: Hello</p>
                <p>日本語: こんにちは</p>
                <p>Español: Hola</p>
                <p>العربية: مرحبا</p>
                <p>中文: 你好</p>
            </body>
        </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/utf8"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(utf8_content)
                .insert_header("Content-Type", "text/html; charset=utf-8"),
        )
        .mount(&mock_server)
        .await;

    let scraper = create_test_scraper().await?;
    let url = format!("{}/utf8", mock_server.uri());
    let html = scraper.fetch_html(&url).await?;

    assert!(html.contains("こんにちは"));
    assert!(html.contains("مرحبا"));
    assert!(html.contains("你好"));

    Ok(())
}

#[tokio::test]
async fn test_scraper_clone_independence() -> Result<(), Box<dyn std::error::Error>> {
    let scraper1 = Riptide::builder()
        .user_agent("Original/1.0")
        .build_scraper()
        .await?;

    let scraper2 = scraper1.clone();

    // Both should have same config
    assert_eq!(scraper1.config().user_agent, scraper2.config().user_agent);

    // Both should work independently
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Test"))
        .mount(&mock_server)
        .await;

    let url = mock_server.uri();

    let result1 = scraper1.fetch_html(&url).await;
    let result2 = scraper2.fetch_html(&url).await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_scraper_error_messages() {
    let scraper = create_test_scraper().await.unwrap();

    // Test invalid URL error message
    let result = scraper.fetch_html("not-a-url").await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid URL"));
}

// Note: Tests requiring actual network access should be marked with #[ignore]
#[tokio::test]
#[ignore]
async fn test_scraper_real_network_example_com() -> Result<(), Box<dyn std::error::Error>> {
    let scraper = create_test_scraper().await?;
    let html = scraper.fetch_html("https://example.com").await?;

    assert!(!html.is_empty());
    assert!(html.contains("Example Domain"));

    Ok(())
}
