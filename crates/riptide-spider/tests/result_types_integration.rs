//! Integration Tests for Result Types
//!
//! Tests the complete workflow of RawCrawlResult → EnrichedCrawlResult conversion
//! and the enrich() function with different ContentExtractor implementations.

use http::{HeaderMap, StatusCode};
use riptide_spider::extractor::{BasicExtractor, NoOpExtractor};
use riptide_spider::results::{enrich, RawCrawlResult};
use url::Url;

// ============================================================================
// Helper Functions
// ============================================================================

fn create_raw_result(url: &str, html: &str, status: StatusCode) -> RawCrawlResult {
    RawCrawlResult {
        url: Url::parse(url).unwrap(),
        html: html.to_string(),
        status,
        headers: HeaderMap::new(),
    }
}

fn create_raw_result_with_headers(
    url: &str,
    html: &str,
    status: StatusCode,
    content_type: &str,
) -> RawCrawlResult {
    let mut headers = HeaderMap::new();
    headers.insert("content-type", content_type.parse().unwrap());

    RawCrawlResult {
        url: Url::parse(url).unwrap(),
        html: html.to_string(),
        status,
        headers,
    }
}

// ============================================================================
// Integration Test 1: RawCrawlResult → EnrichedCrawlResult Conversion
// ============================================================================

#[test]
fn test_raw_to_enriched_conversion_preserves_url() {
    let raw = create_raw_result(
        "https://example.com/test-page",
        "<html><body>Test</body></html>",
        StatusCode::OK,
    );

    let enriched = enrich(raw, &BasicExtractor);

    assert_eq!(enriched.raw.url.as_str(), "https://example.com/test-page");
}

#[test]
fn test_raw_to_enriched_conversion_preserves_html() {
    let html_content =
        r#"<html><head><title>Test</title></head><body><p>Content</p></body></html>"#;
    let raw = create_raw_result("https://example.com", html_content, StatusCode::OK);

    let enriched = enrich(raw, &BasicExtractor);

    assert_eq!(enriched.raw.html, html_content);
}

#[test]
fn test_raw_to_enriched_conversion_preserves_status() {
    let statuses = vec![
        StatusCode::OK,
        StatusCode::CREATED,
        StatusCode::NOT_FOUND,
        StatusCode::INTERNAL_SERVER_ERROR,
    ];

    for status in statuses {
        let raw = create_raw_result("https://example.com", "<html></html>", status);
        let enriched = enrich(raw, &BasicExtractor);

        assert_eq!(enriched.raw.status, status);
    }
}

#[test]
fn test_raw_to_enriched_conversion_preserves_headers() {
    let raw = create_raw_result_with_headers(
        "https://example.com",
        "<html></html>",
        StatusCode::OK,
        "text/html; charset=utf-8",
    );

    let enriched = enrich(raw, &BasicExtractor);

    assert!(enriched.raw.headers.contains_key("content-type"));
}

// ============================================================================
// Integration Test 2: enrich() Function with Different Extractors
// ============================================================================

#[test]
fn test_enrich_with_basic_extractor_extracts_links() {
    let html = r#"
        <html>
            <body>
                <a href="/page1">Link 1</a>
                <a href="/page2">Link 2</a>
                <a href="https://external.com/page">External</a>
            </body>
        </html>
    "#;

    let raw = create_raw_result("https://example.com", html, StatusCode::OK);
    let enriched = enrich(raw, &BasicExtractor);

    assert_eq!(enriched.extracted_urls.len(), 3);
    assert!(enriched
        .extracted_urls
        .iter()
        .any(|u| u.as_str() == "https://example.com/page1"));
    assert!(enriched
        .extracted_urls
        .iter()
        .any(|u| u.as_str() == "https://example.com/page2"));
    assert!(enriched
        .extracted_urls
        .iter()
        .any(|u| u.as_str() == "https://external.com/page"));
}

#[test]
fn test_enrich_with_basic_extractor_extracts_text() {
    let html = r#"
        <html>
            <head><title>Page Title</title></head>
            <body>
                <h1>Main Heading</h1>
                <p>Paragraph content with important information.</p>
                <div>More text in a div</div>
            </body>
        </html>
    "#;

    let raw = create_raw_result("https://example.com", html, StatusCode::OK);
    let enriched = enrich(raw, &BasicExtractor);

    assert!(enriched.text_content.is_some());
    let text = enriched.text_content.unwrap();

    // Should contain text from various elements
    assert!(text.contains("Main Heading") || text.contains("Paragraph content"));
}

#[test]
fn test_enrich_with_noop_extractor_returns_empty() {
    let html = r#"
        <html>
            <body>
                <a href="/page1">Link 1</a>
                <a href="/page2">Link 2</a>
                <p>Text content</p>
            </body>
        </html>
    "#;

    let raw = create_raw_result("https://example.com", html, StatusCode::OK);
    let enriched = enrich(raw, &NoOpExtractor);

    // NoOp extractor should return empty results
    assert_eq!(enriched.extracted_urls.len(), 0);
    assert!(enriched.text_content.is_none());
}

#[test]
fn test_enrich_with_different_extractors_on_same_raw() {
    let html = r#"<html><body><a href="/page">Link</a><p>Text</p></body></html>"#;
    let raw = create_raw_result("https://example.com", html, StatusCode::OK);

    // Enrich with BasicExtractor
    let enriched_basic = enrich(raw.clone(), &BasicExtractor);
    assert!(!enriched_basic.extracted_urls.is_empty());
    assert!(enriched_basic.text_content.is_some());

    // Enrich with NoOpExtractor (same raw data)
    let enriched_noop = enrich(raw, &NoOpExtractor);
    assert_eq!(enriched_noop.extracted_urls.len(), 0);
    assert!(enriched_noop.text_content.is_none());
}

// ============================================================================
// Integration Test 3: extracted_urls Population
// ============================================================================

#[test]
fn test_extracted_urls_are_absolute() {
    let html = r#"
        <a href="/relative">Relative</a>
        <a href="https://absolute.com/page">Absolute</a>
        <a href="../parent">Parent</a>
    "#;

    let raw = create_raw_result("https://example.com/dir/", html, StatusCode::OK);
    let enriched = enrich(raw, &BasicExtractor);

    // All extracted URLs should be absolute
    for url in &enriched.extracted_urls {
        assert!(url.scheme() == "http" || url.scheme() == "https");
        assert!(url.has_host());
    }
}

#[test]
fn test_extracted_urls_resolved_against_base() {
    let html = r#"<a href="page1">Relative to current dir</a>"#;

    let raw = create_raw_result("https://example.com/dir/index.html", html, StatusCode::OK);
    let enriched = enrich(raw, &BasicExtractor);

    assert_eq!(enriched.extracted_urls.len(), 1);
    assert_eq!(
        enriched.extracted_urls[0].as_str(),
        "https://example.com/dir/page1"
    );
}

#[test]
fn test_extracted_urls_from_complex_page() {
    let html = r#"
        <html>
            <head>
                <link href="/stylesheet.css" />
            </head>
            <body>
                <nav>
                    <a href="/">Home</a>
                    <a href="/about">About</a>
                    <a href="/contact">Contact</a>
                </nav>
                <article>
                    <a href="/article/1">Article 1</a>
                    <a href="/article/2">Article 2</a>
                </article>
                <footer>
                    <a href="https://twitter.com/example">Twitter</a>
                    <a href="https://github.com/example">GitHub</a>
                </footer>
            </body>
        </html>
    "#;

    let raw = create_raw_result("https://example.com", html, StatusCode::OK);
    let enriched = enrich(raw, &BasicExtractor);

    // Should extract multiple links
    assert!(enriched.extracted_urls.len() >= 5);

    // Check for specific internal links
    assert!(enriched.extracted_urls.iter().any(|u| u.path() == "/about"));
    assert!(enriched
        .extracted_urls
        .iter()
        .any(|u| u.path() == "/contact"));

    // Check for external links
    assert!(enriched
        .extracted_urls
        .iter()
        .any(|u| u.host_str() == Some("twitter.com")));
}

#[test]
fn test_extracted_urls_empty_when_no_links() {
    let html = r#"
        <html>
            <body>
                <p>Just text, no links here.</p>
                <div>More text without links.</div>
            </body>
        </html>
    "#;

    let raw = create_raw_result("https://example.com", html, StatusCode::OK);
    let enriched = enrich(raw, &BasicExtractor);

    assert_eq!(enriched.extracted_urls.len(), 0);
}

// ============================================================================
// Integration Test 4: Text Content Extraction
// ============================================================================

#[test]
fn test_text_content_from_various_elements() {
    let html = r#"
        <html>
            <head>
                <title>Page Title</title>
                <script>console.log("should not appear");</script>
            </head>
            <body>
                <h1>Main Heading</h1>
                <p>First paragraph</p>
                <p>Second paragraph</p>
                <div>Div content</div>
                <span>Span content</span>
            </body>
        </html>
    "#;

    let raw = create_raw_result("https://example.com", html, StatusCode::OK);
    let enriched = enrich(raw, &BasicExtractor);

    assert!(enriched.text_content.is_some());
}

#[test]
fn test_text_content_none_for_empty_page() {
    let empty_html_variants = vec![
        "",
        "<html></html>",
        "<html><head></head><body></body></html>",
        "     \n\t     ",
    ];

    for html in empty_html_variants {
        let raw = create_raw_result("https://example.com", html, StatusCode::OK);
        let enriched = enrich(raw, &BasicExtractor);

        assert!(
            enriched.text_content.is_none(),
            "Empty HTML should result in None text_content: {:?}",
            html
        );
    }
}

#[test]
fn test_text_content_handles_nested_elements() {
    let html = r#"
        <div>
            <div>
                <div>
                    <p>Deeply nested <strong>text</strong> content</p>
                </div>
            </div>
        </div>
    "#;

    let raw = create_raw_result("https://example.com", html, StatusCode::OK);
    let enriched = enrich(raw, &BasicExtractor);

    assert!(enriched.text_content.is_some());
    let text = enriched.text_content.unwrap();
    assert!(text.contains("nested") && text.contains("text") && text.contains("content"));
}

// ============================================================================
// Integration Test 5: End-to-End Workflow
// ============================================================================

#[test]
fn test_complete_crawl_workflow_simulation() {
    // Simulate: HTTP fetch → Raw result → Enrichment
    let fetched_html = r#"
        <html>
            <head><title>Example Page</title></head>
            <body>
                <h1>Welcome to Example.com</h1>
                <p>This is a test page for the spider.</p>
                <nav>
                    <a href="/page1">Page 1</a>
                    <a href="/page2">Page 2</a>
                    <a href="/page3">Page 3</a>
                </nav>
            </body>
        </html>
    "#;

    // Step 1: Create RawCrawlResult (as spider would after HTTP fetch)
    let raw = create_raw_result("https://example.com", fetched_html, StatusCode::OK);

    // Verify raw result
    assert_eq!(raw.status, StatusCode::OK);
    assert!(raw.html.contains("Welcome to Example.com"));

    // Step 2: Enrich with BasicExtractor
    let enriched = enrich(raw, &BasicExtractor);

    // Verify enrichment
    assert!(enriched.extracted_urls.len() >= 3);
    assert!(enriched.text_content.is_some());

    // Step 3: Verify URLs ready for frontier
    for url in &enriched.extracted_urls {
        assert_eq!(url.host_str(), Some("example.com"));
        assert!(url.path().starts_with("/page"));
    }

    // Step 4: Verify text ready for analysis
    let text = enriched.text_content.unwrap();
    assert!(text.contains("Welcome") || text.contains("test page"));
}

#[test]
fn test_spider_only_mode_workflow() {
    // Simulate spider-only mode (URL discovery without extraction)
    let html = r#"
        <html>
            <body>
                <a href="/page1">Link 1</a>
                <a href="/page2">Link 2</a>
            </body>
        </html>
    "#;

    let raw = create_raw_result("https://example.com", html, StatusCode::OK);

    // Use NoOpExtractor for spider-only mode
    let enriched = enrich(raw, &NoOpExtractor);

    // In spider-only mode: no content extraction
    assert_eq!(enriched.extracted_urls.len(), 0);
    assert!(enriched.text_content.is_none());

    // But raw data is still available for other purposes
    assert!(enriched.raw.html.contains("Link 1"));
}

#[test]
fn test_error_page_handling() {
    // Test handling of error status codes
    let error_html = r#"<html><body><h1>404 Not Found</h1></body></html>"#;

    let raw = create_raw_result(
        "https://example.com/missing",
        error_html,
        StatusCode::NOT_FOUND,
    );

    let enriched = enrich(raw, &BasicExtractor);

    // Should still enrich even on error status
    assert_eq!(enriched.raw.status, StatusCode::NOT_FOUND);
    assert!(enriched.text_content.is_some()); // Can still extract error page text
}

// ============================================================================
// Integration Test 6: Performance and Memory
// ============================================================================

#[test]
fn test_enrich_handles_large_html_efficiently() {
    // Generate large HTML document
    let mut html = String::from("<html><body>");
    for i in 0..1000 {
        html.push_str(&format!(
            r#"<div><p>Paragraph {}</p><a href="/page{}">Link {}</a></div>"#,
            i, i, i
        ));
    }
    html.push_str("</body></html>");

    let raw = create_raw_result("https://example.com", &html, StatusCode::OK);

    let start = std::time::Instant::now();
    let enriched = enrich(raw, &BasicExtractor);
    let duration = start.elapsed();

    // Should complete in reasonable time (< 100ms for 1000 links)
    assert!(duration.as_millis() < 100);

    // Should extract all links
    assert_eq!(enriched.extracted_urls.len(), 1000);
}

#[test]
fn test_enrich_clones_raw_result() {
    let raw = create_raw_result(
        "https://example.com",
        "<html><body>Test</body></html>",
        StatusCode::OK,
    );

    // Raw result is moved into enrich
    let enriched = enrich(raw, &BasicExtractor);

    // Can still access raw data through enriched result
    assert_eq!(enriched.raw.url.as_str(), "https://example.com/");
}

// ============================================================================
// Integration Test 7: Real-World Scenarios
// ============================================================================

#[test]
fn test_blog_post_extraction() {
    let blog_html = r#"
        <html>
            <head><title>My Blog Post</title></head>
            <body>
                <article>
                    <h1>Understanding Web Crawlers</h1>
                    <p>Web crawlers are automated programs...</p>
                    <p>They work by following links...</p>
                    <a href="/related-post-1">Related Post 1</a>
                    <a href="/related-post-2">Related Post 2</a>
                </article>
                <aside>
                    <a href="/about">About</a>
                    <a href="/contact">Contact</a>
                </aside>
            </body>
        </html>
    "#;

    let raw = create_raw_result(
        "https://blog.example.com/post/123",
        blog_html,
        StatusCode::OK,
    );
    let enriched = enrich(raw, &BasicExtractor);

    // Should extract internal links
    assert!(enriched.extracted_urls.len() >= 4);

    // Should have text content
    assert!(enriched.text_content.is_some());
}

#[test]
fn test_navigation_heavy_page() {
    let nav_html = r#"
        <html>
            <body>
                <header>
                    <nav>
                        <a href="/">Home</a>
                        <a href="/products">Products</a>
                        <a href="/services">Services</a>
                        <a href="/about">About</a>
                        <a href="/contact">Contact</a>
                    </nav>
                </header>
                <main>
                    <p>Welcome to our site</p>
                </main>
                <footer>
                    <a href="/privacy">Privacy</a>
                    <a href="/terms">Terms</a>
                </footer>
            </body>
        </html>
    "#;

    let raw = create_raw_result("https://example.com", nav_html, StatusCode::OK);
    let enriched = enrich(raw, &BasicExtractor);

    // Should extract all navigation links
    assert!(enriched.extracted_urls.len() >= 7);

    // All should be internal links
    for url in &enriched.extracted_urls {
        assert_eq!(url.host_str(), Some("example.com"));
    }
}
