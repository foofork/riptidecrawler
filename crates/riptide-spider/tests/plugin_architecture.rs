//! Spider Plugin Architecture Tests
//!
//! Tests the plugin architecture that allows swapping ContentExtractor implementations:
//! 1. Spider-only mode (no extractor)
//! 2. Spider with BasicExtractor
//! 3. Spider with NoOpExtractor
//! 4. Swapping extractors dynamically

use http::{HeaderMap, StatusCode};
use riptide_spider::extractor::{BasicExtractor, ContentExtractor, NoOpExtractor};
use riptide_spider::results::{enrich, RawCrawlResult};
use url::Url;

// ============================================================================
// Helper Functions
// ============================================================================

fn create_test_raw_result(url: &str, html: &str) -> RawCrawlResult {
    RawCrawlResult {
        url: Url::parse(url).unwrap(),
        html: html.to_string(),
        status: StatusCode::OK,
        headers: HeaderMap::new(),
    }
}

// Custom extractor for testing plugin architecture
struct CustomTestExtractor {
    prefix: String,
}

impl ContentExtractor for CustomTestExtractor {
    fn extract_links(&self, _html: &str, base_url: &Url) -> Vec<Url> {
        // Custom behavior: always return a fixed test URL
        vec![base_url.join("/custom-link").unwrap()]
    }

    fn extract_text(&self, html: &str) -> Option<String> {
        // Custom behavior: prefix all text
        if html.trim().is_empty() {
            None
        } else {
            Some(format!("{}: {}", self.prefix, html))
        }
    }

    fn strategy_name(&self) -> &'static str {
        "custom-test"
    }
}

// ============================================================================
// Test 1: Spider-Only Mode (No Content Extraction)
// ============================================================================

#[test]
fn test_spider_only_mode_with_noop_extractor() {
    let html = r#"
        <html>
            <body>
                <a href="/page1">Link 1</a>
                <a href="/page2">Link 2</a>
                <p>Text content that should be ignored</p>
            </body>
        </html>
    "#;

    let raw = create_test_raw_result("https://example.com", html);

    // Use NoOpExtractor for spider-only mode
    let enriched = enrich(raw, &NoOpExtractor);

    // Should not extract any content
    assert_eq!(enriched.extracted_urls.len(), 0);
    assert!(enriched.text_content.is_none());

    // Strategy name should be correct
    let noop = NoOpExtractor;
    assert_eq!(noop.strategy_name(), "noop");
}

#[test]
fn test_spider_only_mode_preserves_raw_data() {
    let html = "<html><body><a href='/test'>Link</a></body></html>";
    let raw = create_test_raw_result("https://example.com", html);

    let enriched = enrich(raw, &NoOpExtractor);

    // Raw data should still be accessible
    assert_eq!(enriched.raw.url.as_str(), "https://example.com/");
    assert_eq!(enriched.raw.html, html);
    assert_eq!(enriched.raw.status, StatusCode::OK);
}

// ============================================================================
// Test 2: Spider with BasicExtractor
// ============================================================================

#[test]
fn test_spider_with_basic_extractor_extracts_links() {
    let html = r#"
        <html>
            <body>
                <a href="/page1">Page 1</a>
                <a href="/page2">Page 2</a>
                <a href="https://external.com">External</a>
            </body>
        </html>
    "#;

    let raw = create_test_raw_result("https://example.com", html);

    // Use BasicExtractor
    let enriched = enrich(raw, &BasicExtractor);

    // Should extract all links
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
        .any(|u| u.as_str() == "https://external.com/"));
}

#[test]
fn test_spider_with_basic_extractor_extracts_text() {
    let html = r#"
        <html>
            <body>
                <h1>Main Title</h1>
                <p>Paragraph 1</p>
                <p>Paragraph 2</p>
            </body>
        </html>
    "#;

    let raw = create_test_raw_result("https://example.com", html);

    // Use BasicExtractor
    let enriched = enrich(raw, &BasicExtractor);

    // Should extract text content
    assert!(enriched.text_content.is_some());
    let text = enriched.text_content.unwrap();
    assert!(text.contains("Main Title") || text.contains("Paragraph"));
}

#[test]
fn test_basic_extractor_strategy_name() {
    let basic = BasicExtractor;
    assert_eq!(basic.strategy_name(), "basic");
}

// ============================================================================
// Test 3: Swapping Extractors
// ============================================================================

#[test]
fn test_swap_extractors_on_same_content() {
    let html = r#"
        <html>
            <body>
                <a href="/page">Link</a>
                <p>Text content</p>
            </body>
        </html>
    "#;

    let raw = create_test_raw_result("https://example.com", html);

    // First extraction with BasicExtractor
    let enriched_basic = enrich(raw.clone(), &BasicExtractor);
    assert!(!enriched_basic.extracted_urls.is_empty());
    assert!(enriched_basic.text_content.is_some());

    // Second extraction with NoOpExtractor (same raw data)
    let enriched_noop = enrich(raw, &NoOpExtractor);
    assert_eq!(enriched_noop.extracted_urls.len(), 0);
    assert!(enriched_noop.text_content.is_none());

    // Both have the same raw data
    assert_eq!(enriched_basic.raw.html, enriched_noop.raw.html);
}

#[test]
fn test_swap_to_custom_extractor() {
    let html = "<html><body>Test content</body></html>";
    let raw = create_test_raw_result("https://example.com", html);

    let custom_extractor = CustomTestExtractor {
        prefix: "CUSTOM".to_string(),
    };

    let enriched = enrich(raw, &custom_extractor);

    // Custom extractor should return its custom link
    assert_eq!(enriched.extracted_urls.len(), 1);
    assert_eq!(
        enriched.extracted_urls[0].as_str(),
        "https://example.com/custom-link"
    );

    // Custom extractor should prefix the text
    assert!(enriched.text_content.is_some());
    let text = enriched.text_content.unwrap();
    assert!(text.starts_with("CUSTOM:"));
}

// ============================================================================
// Test 4: Plugin Interface Compliance
// ============================================================================

#[test]
fn test_all_extractors_implement_trait() {
    // Verify that all extractors implement ContentExtractor
    fn accepts_extractor<T: ContentExtractor>(_: &T) {}

    let basic = BasicExtractor;
    let noop = NoOpExtractor;
    let custom = CustomTestExtractor {
        prefix: "TEST".to_string(),
    };

    accepts_extractor(&basic);
    accepts_extractor(&noop);
    accepts_extractor(&custom);
}

#[test]
fn test_extractors_are_send_and_sync() {
    // Verify trait bounds for concurrent usage
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<BasicExtractor>();
    assert_send_sync::<NoOpExtractor>();
    // CustomTestExtractor needs String which is Send + Sync
    assert_send_sync::<CustomTestExtractor>();
}

#[test]
fn test_trait_object_usage() {
    // Test that extractors can be used as trait objects (dyn ContentExtractor)
    let html = "<html><body><a href='/test'>Link</a></body></html>";
    let raw = create_test_raw_result("https://example.com", html);

    let extractors: Vec<Box<dyn ContentExtractor>> = vec![
        Box::new(BasicExtractor),
        Box::new(NoOpExtractor),
        Box::new(CustomTestExtractor {
            prefix: "BOXED".to_string(),
        }),
    ];

    for extractor in extractors {
        let enriched = enrich(raw.clone(), extractor.as_ref());
        // Each extractor produces different results
        let _ = enriched.extracted_urls;
    }
}

// ============================================================================
// Test 5: Extractor Behavior Consistency
// ============================================================================

#[test]
fn test_basic_extractor_consistent_results() {
    let html = "<html><body><a href='/page'>Link</a></body></html>";
    let raw = create_test_raw_result("https://example.com", html);

    let extractor = BasicExtractor;

    // Call multiple times
    let result1 = enrich(raw.clone(), &extractor);
    let result2 = enrich(raw.clone(), &extractor);
    let result3 = enrich(raw, &extractor);

    // Results should be consistent (same URLs)
    assert_eq!(result1.extracted_urls.len(), result2.extracted_urls.len());
    assert_eq!(result2.extracted_urls.len(), result3.extracted_urls.len());

    for i in 0..result1.extracted_urls.len() {
        assert_eq!(result1.extracted_urls[i], result2.extracted_urls[i]);
        assert_eq!(result2.extracted_urls[i], result3.extracted_urls[i]);
    }
}

#[test]
fn test_noop_extractor_always_returns_empty() {
    let test_cases = vec![
        "",
        "<html></html>",
        "<a href='/link'>Link</a>",
        "<html><body><p>Text</p></body></html>",
        "Random non-HTML content",
    ];

    let extractor = NoOpExtractor;

    for html in test_cases {
        let raw = create_test_raw_result("https://example.com", html);
        let enriched = enrich(raw, &extractor);

        assert_eq!(enriched.extracted_urls.len(), 0);
        assert!(enriched.text_content.is_none());
    }
}

// ============================================================================
// Test 6: Performance with Different Extractors
// ============================================================================

#[test]
fn test_noop_extractor_faster_than_basic() {
    let mut html = String::from("<html><body>");
    for i in 0..1000 {
        html.push_str(&format!(r#"<a href="/page{}">Link</a>"#, i));
        html.push_str(&format!("<p>Paragraph {}</p>", i));
    }
    html.push_str("</body></html>");

    let raw = create_test_raw_result("https://example.com", &html);

    // Time NoOpExtractor
    let start_noop = std::time::Instant::now();
    let _noop_result = enrich(raw.clone(), &NoOpExtractor);
    let noop_duration = start_noop.elapsed();

    // Time BasicExtractor
    let start_basic = std::time::Instant::now();
    let _basic_result = enrich(raw, &BasicExtractor);
    let basic_duration = start_basic.elapsed();

    // NoOp should be significantly faster (nearly instant)
    assert!(
        noop_duration < basic_duration,
        "NoOp ({:?}) should be faster than Basic ({:?})",
        noop_duration,
        basic_duration
    );

    // NoOp should be < 1ms
    assert!(noop_duration.as_millis() < 1);
}

// ============================================================================
// Test 7: Real-World Plugin Scenarios
// ============================================================================

#[test]
fn test_domain_specific_extractor_pattern() {
    // Simulate a domain-specific extractor (e.g., for e-commerce sites)
    struct ProductExtractor;

    impl ContentExtractor for ProductExtractor {
        fn extract_links(&self, html: &str, base_url: &Url) -> Vec<Url> {
            // Only extract product links
            let mut links = Vec::new();
            if html.contains("product") {
                links.push(base_url.join("/product/123").unwrap());
            }
            links
        }

        fn extract_text(&self, html: &str) -> Option<String> {
            // Extract product descriptions
            if html.contains("description") {
                Some("Product description found".to_string())
            } else {
                None
            }
        }

        fn strategy_name(&self) -> &'static str {
            "product"
        }
    }

    let html = r#"<html><body><div class="product">Product XYZ</div><p class="description">Great product!</p></body></html>"#;
    let raw = create_test_raw_result("https://shop.example.com", html);

    let extractor = ProductExtractor;
    let enriched = enrich(raw, &extractor);

    assert_eq!(extractor.strategy_name(), "product");
    assert_eq!(enriched.extracted_urls.len(), 1);
    assert!(enriched.text_content.is_some());
}

#[test]
fn test_extraction_strategy_selection() {
    // Test selecting the right extractor based on content type
    let html_content = "<html><body><a href='/page'>Link</a></body></html>";
    let raw = create_test_raw_result("https://example.com", html_content);

    // For general HTML, use BasicExtractor
    let enriched_basic = enrich(raw.clone(), &BasicExtractor);
    assert!(!enriched_basic.extracted_urls.is_empty());

    // For URL discovery only, use NoOpExtractor
    let enriched_noop = enrich(raw, &NoOpExtractor);
    assert_eq!(enriched_noop.extracted_urls.len(), 0);
}

// ============================================================================
// Test 8: Error Handling and Edge Cases
// ============================================================================

#[test]
fn test_extractors_handle_malformed_html() {
    let malformed_html = r#"<html><body><a href="/good"><a href="bad url"><p>Unclosed"#;
    let raw = create_test_raw_result("https://example.com", malformed_html);

    let basic_result = enrich(raw.clone(), &BasicExtractor);
    let noop_result = enrich(raw, &NoOpExtractor);

    // BasicExtractor should handle gracefully (may extract some valid URLs)
    let _ = basic_result.extracted_urls.len();

    // NoOpExtractor should still return empty
    assert_eq!(noop_result.extracted_urls.len(), 0);
}

#[test]
fn test_extractors_handle_empty_content() {
    let empty_cases = vec!["", "   ", "\n\t\r"];

    for empty_html in empty_cases {
        let raw = create_test_raw_result("https://example.com", empty_html);

        let basic_result = enrich(raw.clone(), &BasicExtractor);
        let noop_result = enrich(raw, &NoOpExtractor);

        assert_eq!(basic_result.extracted_urls.len(), 0);
        assert!(basic_result.text_content.is_none());

        assert_eq!(noop_result.extracted_urls.len(), 0);
        assert!(noop_result.text_content.is_none());
    }
}

#[test]
fn test_custom_extractor_with_state() {
    // Test that custom extractors can maintain state
    struct StatefulExtractor {
        call_count: std::sync::Arc<std::sync::Mutex<usize>>,
    }

    impl ContentExtractor for StatefulExtractor {
        fn extract_links(&self, _html: &str, _base_url: &Url) -> Vec<Url> {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;
            Vec::new()
        }

        fn extract_text(&self, _html: &str) -> Option<String> {
            None
        }

        fn strategy_name(&self) -> &'static str {
            "stateful"
        }
    }

    let call_count = std::sync::Arc::new(std::sync::Mutex::new(0));
    let extractor = StatefulExtractor {
        call_count: call_count.clone(),
    };

    let raw = create_test_raw_result("https://example.com", "<html></html>");

    // Call multiple times
    let _ = enrich(raw.clone(), &extractor);
    let _ = enrich(raw.clone(), &extractor);
    let _ = enrich(raw, &extractor);

    // Verify state was maintained
    assert_eq!(*call_count.lock().unwrap(), 3);
}
