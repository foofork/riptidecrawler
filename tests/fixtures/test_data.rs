/// Test data for London School TDD validation
///
/// This module provides comprehensive test data sets for different content types
/// and scenarios used in behavior-driven testing.

use crate::fixtures::*;
use std::collections::HashMap;

/// HTML test samples for various content types
pub struct HtmlSamples;

impl HtmlSamples {
    /// Valid article HTML with rich content
    pub fn article_html() -> String {
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Test Article - Tech News</title>
    <meta name="description" content="A comprehensive test article about technology">
    <meta name="author" content="Test Author">
    <meta property="og:title" content="Test Article">
    <meta property="og:description" content="Technology article for testing">
</head>
<body>
    <article>
        <header>
            <h1>Revolutionary AI Technology Breakthrough</h1>
            <p class="byline">By Test Author</p>
            <time datetime="2024-01-15">January 15, 2024</time>
        </header>
        <div class="content">
            <p>This is a test article with meaningful content for extraction testing.</p>
            <p>It contains multiple paragraphs with <a href="https://example.com">external links</a>
               and <img src="https://example.com/image.jpg" alt="Test image"> images.</p>
            <blockquote>
                <p>Important quote for testing content extraction quality.</p>
            </blockquote>
            <ul>
                <li>Feature one of the technology</li>
                <li>Feature two with <strong>emphasis</strong></li>
                <li>Feature three with <em>italics</em></li>
            </ul>
        </div>
    </article>
    <aside>
        <div class="sidebar">Sidebar content that should be filtered</div>
    </aside>
</body>
</html>"#.to_string()
    }

    /// SPA HTML that requires dynamic rendering
    pub fn spa_html() -> String {
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Dashboard - SPA App</title>
    <script>
        // Simulate dynamic content loading
        window.addEventListener('DOMContentLoaded', function() {
            setTimeout(function() {
                const container = document.getElementById('dynamic-content');
                if (container) {
                    container.innerHTML = '<div class="loaded-content">Dynamic content loaded</div>';
                }
            }, 1000);
        });
    </script>
</head>
<body>
    <div id="app">
        <nav class="navbar">Navigation</nav>
        <main>
            <div id="dynamic-content" class="loading">Loading...</div>
            <button id="load-more">Load More</button>
        </main>
    </div>
</body>
</html>"#.to_string()
    }

    /// Malformed HTML for error testing
    pub fn malformed_html() -> String {
        r#"<html>
<head><title>Broken HTML</title>
<body>
<div>Unclosed div
<p>Nested without closing</span>
<img src="invalid" onerror="alert('xss')">
"#.to_string()
    }

    /// Empty/minimal HTML
    pub fn empty_html() -> String {
        r#"<!DOCTYPE html><html><head><title>Empty</title></head><body></body></html>"#.to_string()
    }

    /// PDF-like HTML (should be handled differently)
    pub fn pdf_placeholder_html() -> String {
        r#"<!DOCTYPE html>
<html>
<head><title>PDF Document</title></head>
<body>
    <div class="pdf-viewer">
        <embed src="document.pdf" type="application/pdf" width="100%" height="600px">
    </div>
</body>
</html>"#.to_string()
    }
}

/// Mock responses for different scenarios
pub struct MockResponses;

impl MockResponses {
    /// Successful article response
    pub fn successful_article() -> MockResponse {
        MockResponse::new(200, HtmlSamples::article_html())
            .with_headers(HashMap::from([
                ("content-type".to_string(), "text/html; charset=utf-8".to_string()),
                ("content-length".to_string(), "1024".to_string()),
            ]))
            .with_url("https://example.com/article".to_string())
    }

    /// SPA response requiring dynamic rendering
    pub fn spa_response() -> MockResponse {
        MockResponse::new(200, HtmlSamples::spa_html())
            .with_headers(HashMap::from([
                ("content-type".to_string(), "text/html; charset=utf-8".to_string()),
                ("x-spa-app".to_string(), "true".to_string()),
            ]))
            .with_url("https://spa-app.com/dashboard".to_string())
    }

    /// 404 Not Found response
    pub fn not_found() -> MockResponse {
        MockResponse::new(404, "Not Found".to_string())
            .with_url("https://example.com/missing".to_string())
    }

    /// 500 Server Error response
    pub fn server_error() -> MockResponse {
        MockResponse::new(500, "Internal Server Error".to_string())
            .with_url("https://example.com/error".to_string())
    }

    /// Timeout simulation (for chaos testing)
    pub fn timeout_response() -> MockResponse {
        MockResponse::new(0, "".to_string()) // Special case for timeout
    }

    /// PDF response
    pub fn pdf_response() -> MockResponse {
        MockResponse::new(200, "PDF binary content...".to_string())
            .with_headers(HashMap::from([
                ("content-type".to_string(), "application/pdf".to_string()),
                ("content-length".to_string(), "50000".to_string()),
            ]))
            .with_url("https://docs.example.com/api.pdf".to_string())
    }
}

/// Expected extraction results for validation
pub struct ExpectedResults;

impl ExpectedResults {
    /// Expected result from article extraction
    pub fn article_extraction() -> ExtractedContent {
        ExtractedContent {
            url: "https://example.com/article".to_string(),
            title: Some("Revolutionary AI Technology Breakthrough".to_string()),
            content: "This is a test article with meaningful content for extraction testing.\n\nIt contains multiple paragraphs with external links and images.\n\nImportant quote for testing content extraction quality.\n\n• Feature one of the technology\n• Feature two with emphasis\n• Feature three with italics".to_string(),
            links: vec!["https://example.com".to_string()],
            images: vec!["https://example.com/image.jpg".to_string()],
        }
    }

    /// Expected health status
    pub fn healthy_status() -> HealthStatus {
        HealthStatus {
            status: "healthy".to_string(),
            version: "0.1.0".to_string(),
            memory_usage: 1048576, // 1MB
        }
    }

    /// Expected component info
    pub fn component_info() -> ComponentInfo {
        ComponentInfo {
            name: "riptide-extractor-wasm".to_string(),
            version: "0.1.0".to_string(),
            features: vec![
                "article-extraction".to_string(),
                "full-page-extraction".to_string(),
                "metadata-extraction".to_string(),
                "custom-selectors".to_string(),
            ],
        }
    }
}

/// Performance benchmark data
pub struct BenchmarkData;

impl BenchmarkData {
    /// URLs for performance testing
    pub fn performance_urls() -> Vec<&'static str> {
        vec![
            "https://example.com/fast",
            "https://example.com/medium",
            "https://example.com/slow",
            "https://heavy-content.com/article",
            "https://minimal.com/page",
        ]
    }

    /// Large HTML content for stress testing
    pub fn large_html_content() -> String {
        let base_paragraph = "<p>This is a test paragraph with meaningful content for performance testing. It contains enough text to make the extraction meaningful while testing the limits of the system.</p>\n";
        let repeated_content = base_paragraph.repeat(1000); // ~100KB of content

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Large Article for Performance Testing</title>
</head>
<body>
    <article>
        <h1>Performance Test Article</h1>
        {}
    </article>
</body>
</html>"#,
            repeated_content
        )
    }

    /// Batch processing test URLs (50-URL set)
    pub fn batch_test_urls() -> Vec<String> {
        (1..=50)
            .map(|i| format!("https://example.com/article/{}", i))
            .collect()
    }
}

/// Session test data
pub struct SessionData;

impl SessionData {
    /// Valid session for testing
    pub fn valid_session() -> Session {
        Session {
            id: "test-session-123".to_string(),
            created_at: std::time::SystemTime::now(),
            last_accessed: std::time::SystemTime::now(),
            data: HashMap::from([
                ("user_id".to_string(), "user123".to_string()),
                ("preferences".to_string(), r#"{"theme":"dark"}"#.to_string()),
            ]),
        }
    }

    /// Expired session for cleanup testing
    pub fn expired_session() -> Session {
        Session {
            id: "expired-session-456".to_string(),
            created_at: std::time::SystemTime::now() - std::time::Duration::from_secs(3600),
            last_accessed: std::time::SystemTime::now() - std::time::Duration::from_secs(1800),
            data: HashMap::new(),
        }
    }
}