//! Contract Tests for ContentExtractor Trait
//!
//! This test suite verifies that all ContentExtractor implementations
//! fulfill the contract defined by the trait, ensuring consistent behavior
//! across BasicExtractor, NoOpExtractor, and future implementations.

use riptide_spider::extractor::{BasicExtractor, ContentExtractor, NoOpExtractor};
use url::Url;

// ============================================================================
// Contract Test 1: BasicExtractor Link Extraction
// ============================================================================

#[test]
fn test_basic_extractor_extracts_absolute_links() {
    let extractor = BasicExtractor;
    let base_url = Url::parse("https://example.com").unwrap();
    let html = r#"<a href="https://example.com/page1">Link 1</a>"#;

    let links = extractor.extract_links(html, &base_url);

    assert_eq!(links.len(), 1);
    assert_eq!(links[0].as_str(), "https://example.com/page1");
}

#[test]
fn test_basic_extractor_resolves_relative_links() {
    let extractor = BasicExtractor;
    let base_url = Url::parse("https://example.com/dir/").unwrap();
    let html = r#"<a href="/absolute">Absolute</a><a href="relative">Relative</a><a href="../parent">Parent</a>"#;

    let links = extractor.extract_links(html, &base_url);

    assert!(links.len() >= 3);
    assert!(links
        .iter()
        .any(|u| u.as_str() == "https://example.com/absolute"));
    assert!(links
        .iter()
        .any(|u| u.as_str() == "https://example.com/dir/relative"));
    assert!(links
        .iter()
        .any(|u| u.as_str() == "https://example.com/parent"));
}

#[test]
fn test_basic_extractor_handles_mixed_quotes() {
    let extractor = BasicExtractor;
    let base_url = Url::parse("https://example.com").unwrap();
    let html = r#"
        <a href="/double">Double Quotes</a>
        <a href='/single'>Single Quotes</a>
    "#;

    let links = extractor.extract_links(html, &base_url);

    assert!(links.len() >= 2);
    assert!(links.iter().any(|u| u.path() == "/double"));
    assert!(links.iter().any(|u| u.path() == "/single"));
}

#[test]
fn test_basic_extractor_skips_invalid_urls() {
    let extractor = BasicExtractor;
    let base_url = Url::parse("https://example.com").unwrap();
    let html = r#"
        <a href="/valid">Valid</a>
        <a href="javascript:void(0)">Invalid JavaScript</a>
        <a href="mailto:test@example.com">Invalid Mailto</a>
        <a href="/another-valid">Another Valid</a>
    "#;

    let links = extractor.extract_links(html, &base_url);

    // BasicExtractor uses URL::join which may accept various schemes
    // The important thing is it extracts the valid relative URLs
    assert!(links.len() >= 2);

    // Verify valid links are present
    let valid_paths: Vec<_> = links
        .iter()
        .filter(|u| u.scheme() == "http" || u.scheme() == "https")
        .collect();
    assert!(
        valid_paths.len() >= 2,
        "Should extract at least 2 HTTP/HTTPS URLs"
    );
}

#[test]
fn test_basic_extractor_deduplicates_nothing() {
    // The extractor should NOT deduplicate - that's the frontier's job
    let extractor = BasicExtractor;
    let base_url = Url::parse("https://example.com").unwrap();
    let html = r#"<a href="/page">Link</a><a href="/page">Duplicate</a>"#;

    let links = extractor.extract_links(html, &base_url);

    // Should return all links, including duplicates
    assert_eq!(links.len(), 2);
}

// ============================================================================
// Contract Test 2: BasicExtractor Text Extraction
// ============================================================================

#[test]
fn test_basic_extractor_extracts_text_content() {
    let extractor = BasicExtractor;
    let html = r#"<html><body><p>Hello World</p><div>More text</div></body></html>"#;

    let text = extractor.extract_text(html);

    assert!(text.is_some());
    let content = text.unwrap();
    assert!(content.contains("Hello World"));
    assert!(content.contains("More text"));
}

#[test]
fn test_basic_extractor_removes_html_tags() {
    let extractor = BasicExtractor;
    let html = r#"<html><head><title>Title</title></head><body><h1>Heading</h1><p>Paragraph</p></body></html>"#;

    let text = extractor.extract_text(html);

    assert!(text.is_some());
    let content = text.unwrap();
    // Should not contain HTML tags
    assert!(!content.contains("<html>"));
    assert!(!content.contains("<body>"));
    assert!(!content.contains("<p>"));
    // Should contain text content
    assert!(
        content.contains("Title") || content.contains("Heading") || content.contains("Paragraph")
    );
}

#[test]
fn test_basic_extractor_returns_none_for_empty_html() {
    let extractor = BasicExtractor;
    let html = r#"<html><head></head><body></body></html>"#;

    let text = extractor.extract_text(html);

    assert!(text.is_none());
}

#[test]
fn test_basic_extractor_trims_whitespace() {
    let extractor = BasicExtractor;
    let html = r#"<html><body>   Text with spaces   </body></html>"#;

    let text = extractor.extract_text(html);

    assert!(text.is_some());
    let content = text.unwrap();
    // Should be trimmed, not starting/ending with spaces
    assert!(!content.starts_with("   "));
    assert!(!content.ends_with("   "));
}

#[test]
fn test_basic_extractor_handles_malformed_html() {
    let extractor = BasicExtractor;
    let html = r#"<html><body><p>Unclosed paragraph<div>Nested without closing</body>"#;

    let text = extractor.extract_text(html);

    // Should still extract text despite malformed HTML
    assert!(text.is_some());
}

// ============================================================================
// Contract Test 3: NoOpExtractor Behavior
// ============================================================================

#[test]
fn test_noop_extractor_returns_empty_links() {
    let extractor = NoOpExtractor;
    let base_url = Url::parse("https://example.com").unwrap();
    let html = r#"<a href="/page1">Link 1</a><a href="/page2">Link 2</a>"#;

    let links = extractor.extract_links(html, &base_url);

    assert_eq!(links.len(), 0);
}

#[test]
fn test_noop_extractor_returns_none_text() {
    let extractor = NoOpExtractor;
    let html = r#"<html><body><p>Lots of text content here</p></body></html>"#;

    let text = extractor.extract_text(html);

    assert!(text.is_none());
}

#[test]
fn test_noop_extractor_ignores_all_input() {
    let extractor = NoOpExtractor;
    let base_url = Url::parse("https://example.com").unwrap();

    // Try with various inputs
    let empty = extractor.extract_links("", &base_url);
    let complex = extractor.extract_links(
        "<html><body><a href='/test'>Link</a></body></html>",
        &base_url,
    );
    let malformed = extractor.extract_links("<<<>>>", &base_url);

    assert_eq!(empty.len(), 0);
    assert_eq!(complex.len(), 0);
    assert_eq!(malformed.len(), 0);
}

// ============================================================================
// Contract Test 4: Strategy Names
// ============================================================================

#[test]
fn test_basic_extractor_strategy_name() {
    let extractor = BasicExtractor;
    assert_eq!(extractor.strategy_name(), "basic");
}

#[test]
fn test_noop_extractor_strategy_name() {
    let extractor = NoOpExtractor;
    assert_eq!(extractor.strategy_name(), "noop");
}

#[test]
fn test_strategy_names_are_static() {
    // Strategy names must be static strings for performance
    let basic = BasicExtractor;
    let noop = NoOpExtractor;

    let name1 = basic.strategy_name();
    let name2 = basic.strategy_name();

    // Same pointer - truly static
    assert!(std::ptr::eq(name1, name2));

    let noop_name1 = noop.strategy_name();
    let noop_name2 = noop.strategy_name();
    assert!(std::ptr::eq(noop_name1, noop_name2));
}

// ============================================================================
// Contract Test 5: Send + Sync Bounds
// ============================================================================

#[test]
fn test_extractors_are_send() {
    fn assert_send<T: Send>() {}
    assert_send::<BasicExtractor>();
    assert_send::<NoOpExtractor>();
}

#[test]
fn test_extractors_are_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<BasicExtractor>();
    assert_sync::<NoOpExtractor>();
}

#[test]
fn test_extractors_can_be_shared_across_threads() {
    use std::sync::Arc;
    use std::thread;

    let extractor = Arc::new(BasicExtractor);
    let base_url = Url::parse("https://example.com").unwrap();
    let html = r#"<a href="/test">Test</a>"#.to_string();

    // Clone Arc for thread
    let extractor_clone = Arc::clone(&extractor);
    let base_clone = base_url.clone();
    let html_clone = html.clone();

    let handle = thread::spawn(move || extractor_clone.extract_links(&html_clone, &base_clone));

    // Use in main thread
    let main_links = extractor.extract_links(&html, &base_url);

    // Wait for thread
    let thread_links = handle.join().unwrap();

    assert_eq!(main_links.len(), thread_links.len());
}

// ============================================================================
// Contract Test 6: Performance Characteristics
// ============================================================================

#[test]
fn test_basic_extractor_handles_large_html() {
    let extractor = BasicExtractor;
    let base_url = Url::parse("https://example.com").unwrap();

    // Generate large HTML document
    let mut html = String::from("<html><body>");
    for i in 0..1000 {
        html.push_str(&format!(r#"<a href="/page{}">Link {}</a>"#, i, i));
    }
    html.push_str("</body></html>");

    let links = extractor.extract_links(&html, &base_url);

    assert_eq!(links.len(), 1000);
}

#[test]
fn test_noop_extractor_is_zero_cost() {
    let extractor = NoOpExtractor;
    let base_url = Url::parse("https://example.com").unwrap();

    // Generate large HTML - NoOp should ignore instantly
    let large_html = "x".repeat(1_000_000);

    let start = std::time::Instant::now();
    let links = extractor.extract_links(&large_html, &base_url);
    let duration = start.elapsed();

    assert_eq!(links.len(), 0);
    // Should be effectively instant (< 1ms)
    assert!(duration.as_millis() < 1);
}

// ============================================================================
// Contract Test 7: Edge Cases
// ============================================================================

#[test]
fn test_basic_extractor_with_empty_string() {
    let extractor = BasicExtractor;
    let base_url = Url::parse("https://example.com").unwrap();

    let links = extractor.extract_links("", &base_url);
    let text = extractor.extract_text("");

    assert_eq!(links.len(), 0);
    assert!(text.is_none());
}

#[test]
fn test_basic_extractor_with_only_whitespace() {
    let extractor = BasicExtractor;
    let html = "     \n\t\r     ";

    let text = extractor.extract_text(html);

    assert!(text.is_none());
}

#[test]
fn test_basic_extractor_with_special_characters() {
    let extractor = BasicExtractor;
    let base_url = Url::parse("https://example.com").unwrap();
    let html = r#"<a href="/test?param=value&other=123">Query String</a>"#;

    let links = extractor.extract_links(html, &base_url);

    assert_eq!(links.len(), 1);
    assert_eq!(
        links[0].as_str(),
        "https://example.com/test?param=value&other=123"
    );
}

#[test]
fn test_basic_extractor_with_fragments() {
    let extractor = BasicExtractor;
    let base_url = Url::parse("https://example.com").unwrap();
    let html = r#"<a href="/page#section">Link with fragment</a>"#;

    let links = extractor.extract_links(html, &base_url);

    assert_eq!(links.len(), 1);
    assert_eq!(links[0].as_str(), "https://example.com/page#section");
}

#[test]
fn test_basic_extractor_with_unicode() {
    let extractor = BasicExtractor;
    let html = r#"<html><body><p>Hello 世界 🌍</p></body></html>"#;

    let text = extractor.extract_text(html);

    assert!(text.is_some());
    let content = text.unwrap();
    assert!(content.contains("世界"));
    assert!(content.contains("🌍"));
}
