//! Direct Execution Mode Tests
//!
//! Comprehensive tests for direct execution mode including:
//! - Engine selection logic validation
//! - WASM module integration
//! - Browser pool coordination
//! - Error handling and fallback chains

use anyhow::Result;
use std::time::Instant;

#[cfg(test)]
mod direct_execution_tests {
    use super::*;

    /// Test direct execution mode initialization
    #[tokio::test]
    async fn test_direct_mode_initialization() {
        // Test that direct mode can be initialized without API key
        // This validates the core functionality is independent

        // Mock config without API key
        let result = initialize_direct_mode().await;
        assert!(result.is_ok(), "Direct mode should initialize without API key");
    }

    /// Test engine selection based on content type
    #[tokio::test]
    async fn test_engine_selection_content_analysis() {
        let test_cases = vec![
            // (HTML content, expected engine)
            (create_simple_html(), "wasm"),
            (create_react_html(), "headless"),
            (create_spa_html(), "headless"),
            (create_anti_scraping_html(), "stealth"),
        ];

        for (html, expected_engine) in test_cases {
            let selected_engine = analyze_and_select_engine(&html, "https://example.com");
            assert_eq!(
                selected_engine, expected_engine,
                "Wrong engine selected for content type"
            );
        }
    }

    /// Test WASM engine execution path
    #[tokio::test]
    async fn test_wasm_engine_execution() {
        let html = create_simple_html();
        let url = "https://example.com";

        let start = Instant::now();
        let result = execute_with_wasm(&html, url).await;
        let duration = start.elapsed();

        assert!(result.is_ok(), "WASM extraction should succeed");
        assert!(duration.as_millis() < 1000, "WASM extraction should be fast");

        let content = result.unwrap();
        assert!(!content.is_empty(), "Extracted content should not be empty");
        assert!(content.len() > 50, "Content should have reasonable length");
    }

    /// Test headless engine execution path
    #[tokio::test]
    async fn test_headless_engine_execution() {
        let html = create_react_html();
        let url = "https://example.com";

        let start = Instant::now();
        let result = execute_with_headless(&html, url).await;
        let duration = start.elapsed();

        assert!(result.is_ok(), "Headless extraction should succeed");
        assert!(duration.as_millis() < 5000, "Headless extraction should complete within 5s");

        let content = result.unwrap();
        assert!(!content.is_empty(), "Extracted content should not be empty");
    }

    /// Test stealth engine execution path
    #[tokio::test]
    async fn test_stealth_engine_execution() {
        let html = create_anti_scraping_html();
        let url = "https://protected.example.com";

        let start = Instant::now();
        let result = execute_with_stealth(&html, url).await;
        let duration = start.elapsed();

        assert!(result.is_ok(), "Stealth extraction should succeed");
        assert!(duration.as_millis() < 10000, "Stealth extraction should complete within 10s");

        let content = result.unwrap();
        assert!(!content.is_empty(), "Extracted content should not be empty");
    }

    /// Test fallback chain: WASM -> Headless -> Stealth
    #[tokio::test]
    async fn test_engine_fallback_chain() {
        // Simulate WASM failure, should fallback to headless
        let html = create_complex_html();
        let url = "https://example.com";

        let result = execute_with_fallback(&html, url).await;

        assert!(result.is_ok(), "Fallback chain should eventually succeed");

        let (content, engine_used) = result.unwrap();
        assert!(!content.is_empty(), "Content should be extracted");
        assert!(
            ["wasm", "headless", "stealth"].contains(&engine_used.as_str()),
            "Should use one of the available engines"
        );
    }

    /// Test concurrent extractions using different engines
    #[tokio::test]
    async fn test_concurrent_multi_engine_extraction() {
        let tasks = vec![
            tokio::spawn(async { execute_with_wasm(&create_simple_html(), "https://test1.com").await }),
            tokio::spawn(async { execute_with_wasm(&create_simple_html(), "https://test2.com").await }),
            tokio::spawn(async { execute_with_headless(&create_react_html(), "https://test3.com").await }),
            tokio::spawn(async { execute_with_headless(&create_react_html(), "https://test4.com").await }),
        ];

        let results: Vec<Result<String>> = futures::future::join_all(tasks)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        for result in results {
            assert!(result.is_ok(), "All concurrent extractions should succeed");
        }
    }

    /// Test error handling for invalid HTML
    #[tokio::test]
    async fn test_invalid_html_error_handling() {
        let invalid_html = "Not valid HTML at all!";
        let url = "https://example.com";

        let result = execute_with_wasm(invalid_html, url).await;

        // Should either succeed with minimal extraction or fail gracefully
        match result {
            Ok(content) => assert!(!content.is_empty(), "Should extract something"),
            Err(e) => assert!(e.to_string().contains("Invalid") || e.to_string().contains("Parse"),
                            "Error should indicate parsing issue"),
        }
    }

    /// Test memory limits during extraction
    #[tokio::test]
    async fn test_memory_limit_enforcement() {
        // Create very large HTML to test memory limits
        let large_html = create_large_html(10_000); // 10KB of content
        let url = "https://example.com";

        let result = execute_with_wasm(&large_html, url).await;

        // Should succeed or fail with memory error, not crash
        match result {
            Ok(_) => {
                // Success is good
            },
            Err(e) => {
                let err_str = e.to_string();
                assert!(
                    err_str.contains("memory") || err_str.contains("limit") || err_str.contains("resource"),
                    "Should fail with resource error, got: {}", err_str
                );
            }
        }
    }

    /// Test timeout handling
    #[tokio::test]
    async fn test_extraction_timeout() {
        let html = create_slow_rendering_html();
        let url = "https://example.com";

        let start = Instant::now();
        let result = execute_with_timeout(&html, url, std::time::Duration::from_secs(5)).await;
        let duration = start.elapsed();

        assert!(duration.as_secs() <= 6, "Should respect timeout");

        match result {
            Ok(_) => {
                // Completed within timeout
            },
            Err(e) => {
                assert!(e.to_string().contains("timeout"), "Should indicate timeout error");
            }
        }
    }

    /// Test engine selection caching
    #[tokio::test]
    async fn test_engine_selection_caching() {
        let url = "https://example.com";
        let html = create_react_html();

        // First selection
        let engine1 = analyze_and_select_engine(&html, url);

        // Second selection for same domain should be cached
        let start = Instant::now();
        let engine2 = analyze_and_select_engine(&html, url);
        let duration = start.elapsed();

        assert_eq!(engine1, engine2, "Should return same engine for same domain");
        assert!(duration.as_micros() < 100, "Cached lookup should be very fast");
    }

    // Helper functions for test data generation

    fn create_simple_html() -> String {
        r#"<!DOCTYPE html>
<html>
<head><title>Simple Page</title></head>
<body>
    <article>
        <h1>Simple Content</h1>
        <p>This is a simple HTML page with standard content.</p>
    </article>
</body>
</html>"#.to_string()
    }

    fn create_react_html() -> String {
        r#"<!DOCTYPE html>
<html>
<head>
    <title>React App</title>
    <script>window.__NEXT_DATA__={page: "/"}</script>
</head>
<body>
    <div id="root"></div>
    <div id="_reactRoot"></div>
</body>
</html>"#.to_string()
    }

    fn create_spa_html() -> String {
        r#"<!DOCTYPE html>
<html>
<head>
    <title>SPA</title>
    <script>window.__INITIAL_STATE__={}</script>
    <meta name="generator" content="webpack">
</head>
<body>
    <div class="app" data-react-helmet="true"></div>
</body>
</html>"#.to_string()
    }

    fn create_anti_scraping_html() -> String {
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Protected</title>
    <script src="cf-browser-verification.js"></script>
</head>
<body>
    <div id="challenge">Please verify you're human...</div>
    <script>if (typeof grecaptcha !== 'undefined') {}</script>
</body>
</html>"#.to_string()
    }

    fn create_complex_html() -> String {
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Complex Page</title>
    <script>
        window.__webpack_require__ = function() {};
        window.dataLayer = [];
    </script>
</head>
<body>
    <div class="content">
        <article class="main-content">
            <h1>Complex Content</h1>
            <p>Multiple layers of content and scripts.</p>
        </article>
    </div>
</body>
</html>"#.to_string()
    }

    fn create_large_html(size_kb: usize) -> String {
        let mut html = String::from("<!DOCTYPE html><html><body>");
        let content = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(20);

        // Add content until we reach desired size
        while html.len() < size_kb * 1024 {
            html.push_str(&format!("<p>{}</p>", content));
        }

        html.push_str("</body></html>");
        html
    }

    fn create_slow_rendering_html() -> String {
        r#"<!DOCTYPE html>
<html>
<head><title>Slow Page</title></head>
<body>
    <div id="content">Loading...</div>
    <script>
        // Simulate slow rendering
        setTimeout(() => {
            document.getElementById('content').innerHTML = 'Loaded!';
        }, 10000); // 10 second delay
    </script>
</body>
</html>"#.to_string()
    }

    // Mock implementation functions (replace with actual implementations)

    async fn initialize_direct_mode() -> Result<()> {
        // TODO: Replace with actual initialization code
        Ok(())
    }

    fn analyze_and_select_engine(_html: &str, _url: &str) -> String {
        // TODO: Replace with actual engine selection logic
        // For now, return based on simple heuristics
        if _html.contains("__NEXT_DATA__") || _html.contains("_reactRoot") {
            "headless".to_string()
        } else if _html.contains("cf-browser-verification") || _html.contains("grecaptcha") {
            "stealth".to_string()
        } else {
            "wasm".to_string()
        }
    }

    async fn execute_with_wasm(_html: &str, _url: &str) -> Result<String> {
        // TODO: Replace with actual WASM execution
        Ok("Extracted content from WASM engine".to_string())
    }

    async fn execute_with_headless(_html: &str, _url: &str) -> Result<String> {
        // TODO: Replace with actual headless execution
        Ok("Extracted content from headless engine".to_string())
    }

    async fn execute_with_stealth(_html: &str, _url: &str) -> Result<String> {
        // TODO: Replace with actual stealth execution
        Ok("Extracted content from stealth engine".to_string())
    }

    async fn execute_with_fallback(_html: &str, _url: &str) -> Result<(String, String)> {
        // TODO: Replace with actual fallback chain
        Ok(("Extracted content".to_string(), "wasm".to_string()))
    }

    async fn execute_with_timeout(
        _html: &str,
        _url: &str,
        _timeout: std::time::Duration,
    ) -> Result<String> {
        // TODO: Replace with actual timeout handling
        Ok("Extracted content within timeout".to_string())
    }
}
