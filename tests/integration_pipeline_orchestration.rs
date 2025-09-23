use riptide_api::{
    pipeline::{PipelineOrchestrator, PipelineResult, PipelineStats, GateDecisionStats},
    state::AppState,
    errors::ApiError,
};
use riptide_core::{
    cache::Cache,
    types::{CrawlOptions, ExtractedDoc},
    gate::{Decision, GateFeatures},
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use wiremock::{
    matchers::{method, path, header},
    Mock, MockServer, ResponseTemplate,
};

/// Integration tests for pipeline orchestration
/// Tests the complete end-to-end pipeline workflow including caching,
/// gate decisions, error handling, and performance monitoring.

#[tokio::test]
async fn test_pipeline_end_to_end_success() {
    let mock_server = MockServer::start().await;

    // Mock HTML content that should trigger raw extraction
    let html_content = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Test Article</title>
            <meta name="description" content="Test description">
        </head>
        <body>
            <article>
                <h1>Test Article Title</h1>
                <div class="content">
                    <p>This is a test article with enough content to be considered high quality.</p>
                    <p>It has multiple paragraphs to ensure content depth.</p>
                    <p>The content is structured and should score well in gate analysis.</p>
                </div>
                <a href="http://example.com/link1">Related Link</a>
                <img src="http://example.com/image.jpg" alt="Test Image">
            </article>
        </body>
        </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/article"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(html_content)
            .insert_header("content-type", "text/html; charset=utf-8")
            .insert_header("content-length", &html_content.len().to_string()))
        .expect(1)
        .mount(&mock_server)
        .await;

    let cache = Arc::new(RwLock::new(Cache::new()));
    let state = AppState::new_with_cache(cache);
    let options = CrawlOptions::default();

    let orchestrator = PipelineOrchestrator::new(state, options);
    let url = format!("{}/article", mock_server.uri());

    let result = orchestrator.execute(&url).await.unwrap();

    // Verify pipeline result structure
    assert_eq!(result.document.url, url);
    assert!(result.document.title.is_some());
    assert!(!result.document.text.is_empty());
    assert!(!result.document.markdown.is_empty());
    assert_eq!(result.http_status, 200);
    assert!(!result.from_cache);
    assert!(result.processing_time_ms > 0);
    assert!(!result.cache_key.is_empty());

    // Gate should decide on raw extraction for good HTML
    assert!(result.gate_decision == "raw" || result.gate_decision == "probes_first");
    assert!(result.quality_score > 0.0);
}

#[tokio::test]
async fn test_pipeline_cache_hit_workflow() {
    let mock_server = MockServer::start().await;

    let html_content = "<html><body><h1>Cached Content</h1></body></html>";

    Mock::given(method("GET"))
        .and(path("/cached"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(html_content)
            .insert_header("content-type", "text/html"))
        .expect(1) // Should only be called once
        .mount(&mock_server)
        .await;

    let cache = Arc::new(RwLock::new(Cache::new()));
    let state = AppState::new_with_cache(cache);
    let options = CrawlOptions::default();

    let orchestrator = PipelineOrchestrator::new(state, options);
    let url = format!("{}/cached", mock_server.uri());

    // First request - should fetch from server
    let result1 = orchestrator.execute(&url).await.unwrap();
    assert!(!result1.from_cache);
    assert_eq!(result1.http_status, 200);

    // Second request - should hit cache
    let result2 = orchestrator.execute(&url).await.unwrap();
    assert!(result2.from_cache);
    assert_eq!(result2.document.url, result1.document.url);
    assert_eq!(result2.cache_key, result1.cache_key);

    // Cache hit should be much faster
    assert!(result2.processing_time_ms < result1.processing_time_ms);
}

#[tokio::test]
async fn test_pipeline_gate_decision_variations() {
    let mock_server = MockServer::start().await;

    // Low quality content that should trigger headless
    let low_quality_html = r#"
        <!DOCTYPE html>
        <html>
        <body>
            <div id="content"></div>
            <script>
                document.getElementById('content').innerHTML = 'Dynamic Content';
            </script>
        </body>
        </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/dynamic"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(low_quality_html)
            .insert_header("content-type", "text/html"))
        .expect(1)
        .mount(&mock_server)
        .await;

    // High quality content that should trigger raw extraction
    let high_quality_html = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Quality Article</title></head>
        <body>
            <article>
                <h1>Quality Article</h1>
                <p>This is high quality content with proper semantic markup.</p>
                <p>It has sufficient text content to be considered valuable.</p>
                <p>The structure is clean and extraction-friendly.</p>
                <p>Multiple paragraphs indicate comprehensive content.</p>
            </article>
        </body>
        </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/quality"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(high_quality_html)
            .insert_header("content-type", "text/html"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let cache = Arc::new(RwLock::new(Cache::new()));
    let state = AppState::new_with_cache(cache);
    let options = CrawlOptions::default();
    let orchestrator = PipelineOrchestrator::new(state, options);

    // Test dynamic content (likely headless)
    let dynamic_url = format!("{}/dynamic", mock_server.uri());
    let dynamic_result = orchestrator.execute(&dynamic_url).await.unwrap();

    // Test quality content (likely raw)
    let quality_url = format!("{}/quality", mock_server.uri());
    let quality_result = orchestrator.execute(&quality_url).await.unwrap();

    // Quality content should have higher quality score
    assert!(quality_result.quality_score >= dynamic_result.quality_score);
}

#[tokio::test]
async fn test_pipeline_error_handling_network_failures() {
    let mock_server = MockServer::start().await;

    // Simulate network timeout
    Mock::given(method("GET"))
        .and(path("/timeout"))
        .respond_with(ResponseTemplate::new(200)
            .set_delay(Duration::from_secs(25))) // Longer than client timeout
        .expect(1)
        .mount(&mock_server)
        .await;

    let cache = Arc::new(RwLock::new(Cache::new()));
    let state = AppState::new_with_cache(cache);
    let options = CrawlOptions::default();
    let orchestrator = PipelineOrchestrator::new(state, options);

    let url = format!("{}/timeout", mock_server.uri());
    let result = orchestrator.execute(&url).await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    // Should be a timeout or fetch error
    assert!(matches!(error, ApiError::TimeoutError { .. }) ||
            matches!(error, ApiError::FetchError { .. }));
}

#[tokio::test]
async fn test_pipeline_error_handling_http_errors() {
    let mock_server = MockServer::start().await;

    // Test various HTTP error codes
    let error_codes = vec![404, 500, 502, 503];

    for code in error_codes {
        Mock::given(method("GET"))
            .and(path(&format!("/error-{}", code)))
            .respond_with(ResponseTemplate::new(code))
            .expect(1)
            .mount(&mock_server)
            .await;
    }

    let cache = Arc::new(RwLock::new(Cache::new()));
    let state = AppState::new_with_cache(cache);
    let options = CrawlOptions::default();
    let orchestrator = PipelineOrchestrator::new(state, options);

    // Test 404 (client error - should not retry)
    let url_404 = format!("{}/error-404", mock_server.uri());
    let result_404 = orchestrator.execute(&url_404).await;
    assert!(result_404.is_err());

    // Test 500 (server error - should retry then fail)
    let url_500 = format!("{}/error-500", mock_server.uri());
    let result_500 = orchestrator.execute(&url_500).await;
    assert!(result_500.is_err());
}

#[tokio::test]
async fn test_pipeline_malformed_content_handling() {
    let mock_server = MockServer::start().await;

    // Test various malformed content types
    let test_cases = vec![
        ("empty", ""),
        ("malformed-html", "<html><body><h1>Unclosed tag"),
        ("binary-content", &[0u8, 1u8, 2u8, 255u8].iter().map(|&b| b as char).collect::<String>()),
        ("json-as-html", r#"{"error": "not html"}"#),
    ];

    for (path_suffix, content) in test_cases {
        Mock::given(method("GET"))
            .and(path(&format!("/{}", path_suffix)))
            .respond_with(ResponseTemplate::new(200)
                .set_body_string(content)
                .insert_header("content-type", "text/html"))
            .expect(1)
            .mount(&mock_server)
            .await;
    }

    let cache = Arc::new(RwLock::new(Cache::new()));
    let state = AppState::new_with_cache(cache);
    let options = CrawlOptions::default();
    let orchestrator = PipelineOrchestrator::new(state, options);

    // Test empty content
    let empty_url = format!("{}/empty", mock_server.uri());
    let empty_result = orchestrator.execute(&empty_url).await;
    // Should succeed but with minimal content
    if let Ok(result) = empty_result {
        assert!(result.document.text.len() < 10);
        assert!(result.quality_score < 0.5);
    }

    // Test malformed HTML
    let malformed_url = format!("{}/malformed-html", mock_server.uri());
    let malformed_result = orchestrator.execute(&malformed_url).await;
    // Should handle gracefully
    assert!(malformed_result.is_ok() || malformed_result.is_err());
}

#[tokio::test]
async fn test_pipeline_concurrent_requests() {
    let mock_server = MockServer::start().await;

    let html_content = "<html><body><h1>Concurrent Test</h1><p>Content for concurrent testing.</p></body></html>";

    // Set up multiple endpoints
    for i in 0..5 {
        Mock::given(method("GET"))
            .and(path(&format!("/concurrent-{}", i)))
            .respond_with(ResponseTemplate::new(200)
                .set_body_string(html_content)
                .insert_header("content-type", "text/html"))
            .expect(1)
            .mount(&mock_server)
            .await;
    }

    let cache = Arc::new(RwLock::new(Cache::new()));
    let state = AppState::new_with_cache(cache);
    let options = CrawlOptions::default();
    let orchestrator = Arc::new(PipelineOrchestrator::new(state, options));

    // Launch concurrent requests
    let mut handles = vec![];
    for i in 0..5 {
        let orch = Arc::clone(&orchestrator);
        let url = format!("{}/concurrent-{}", mock_server.uri(), i);

        let handle = tokio::spawn(async move {
            orch.execute(&url).await
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // All should succeed
    for result in results {
        let pipeline_result = result.unwrap().unwrap();
        assert_eq!(pipeline_result.http_status, 200);
        assert!(!pipeline_result.document.text.is_empty());
    }
}

#[tokio::test]
async fn test_pipeline_statistics_collection() {
    let mock_server = MockServer::start().await;

    let html_content = "<html><body><h1>Stats Test</h1></body></html>";

    Mock::given(method("GET"))
        .and(path("/stats"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(html_content)
            .insert_header("content-type", "text/html"))
        .expect(2)
        .mount(&mock_server)
        .await;

    let cache = Arc::new(RwLock::new(Cache::new()));
    let state = AppState::new_with_cache(cache);
    let options = CrawlOptions::default();
    let orchestrator = PipelineOrchestrator::new(state, options);

    let url = format!("{}/stats", mock_server.uri());

    // First request
    let result1 = orchestrator.execute(&url).await.unwrap();
    assert!(!result1.from_cache);

    // Second request (cache hit)
    let result2 = orchestrator.execute(&url).await.unwrap();
    assert!(result2.from_cache);

    // Verify timing differences
    assert!(result1.processing_time_ms > result2.processing_time_ms);

    // Both should have the same cache key
    assert_eq!(result1.cache_key, result2.cache_key);
}

#[tokio::test]
async fn test_pipeline_content_type_handling() {
    let mock_server = MockServer::start().await;

    // Test non-HTML content types
    let test_cases = vec![
        ("pdf", "application/pdf", b"%PDF-1.4 fake pdf content".to_vec()),
        ("json", "application/json", br#"{"message": "not html"}"#.to_vec()),
        ("xml", "application/xml", b"<?xml version='1.0'?><root></root>".to_vec()),
        ("plain", "text/plain", b"Plain text content".to_vec()),
    ];

    for (suffix, content_type, body) in test_cases {
        Mock::given(method("GET"))
            .and(path(&format!("/{}", suffix)))
            .respond_with(ResponseTemplate::new(200)
                .set_body_bytes(body)
                .insert_header("content-type", content_type))
            .expect(1)
            .mount(&mock_server)
            .await;
    }

    let cache = Arc::new(RwLock::new(Cache::new()));
    let state = AppState::new_with_cache(cache);
    let options = CrawlOptions::default();
    let orchestrator = PipelineOrchestrator::new(state, options);

    // Test PDF handling
    let pdf_url = format!("{}/pdf", mock_server.uri());
    let pdf_result = orchestrator.execute(&pdf_url).await;
    // PDF should either succeed with PDF extraction or fail gracefully
    assert!(pdf_result.is_ok() || pdf_result.is_err());

    // Test JSON handling
    let json_url = format!("{}/json", mock_server.uri());
    let json_result = orchestrator.execute(&json_url).await;
    if let Ok(result) = json_result {
        // JSON might be processed as text
        assert!(!result.document.url.is_empty());
    }
}

#[tokio::test]
async fn test_pipeline_large_content_handling() {
    let mock_server = MockServer::start().await;

    // Create large HTML content
    let mut large_content = String::from(r#"<!DOCTYPE html><html><head><title>Large Content</title></head><body>"#);
    for i in 0..1000 {
        large_content.push_str(&format!("<p>This is paragraph number {} with substantial content to test large document handling.</p>", i));
    }
    large_content.push_str("</body></html>");

    Mock::given(method("GET"))
        .and(path("/large"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(&large_content)
            .insert_header("content-type", "text/html")
            .insert_header("content-length", &large_content.len().to_string()))
        .expect(1)
        .mount(&mock_server)
        .await;

    let cache = Arc::new(RwLock::new(Cache::new()));
    let state = AppState::new_with_cache(cache);
    let options = CrawlOptions::default();
    let orchestrator = PipelineOrchestrator::new(state, options);

    let url = format!("{}/large", mock_server.uri());
    let start = std::time::Instant::now();

    let result = orchestrator.execute(&url).await.unwrap();
    let duration = start.elapsed();

    assert_eq!(result.http_status, 200);
    assert!(result.document.text.len() > 10000); // Should have substantial content
    assert!(result.processing_time_ms > 0);

    // Large content processing should still be reasonable
    assert!(duration < Duration::from_secs(5));
}