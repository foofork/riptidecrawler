//! Unit tests for CrawledPage population logic
//!
//! Tests the data population logic for spider handler response pages

use riptide_api::dto::{CrawledPage, FieldFilter};

#[test]
fn test_crawled_page_creation_with_minimal_data() {
    let page = CrawledPage::new("https://example.com".to_string(), 0, 200);

    assert_eq!(page.url, "https://example.com");
    assert_eq!(page.depth, 0);
    assert_eq!(page.status_code, 200);
    assert!(page.content.is_none());
    assert!(page.markdown.is_none());
    assert!(page.title.is_none());
    assert!(page.links.is_empty());
}

#[test]
fn test_crawled_page_with_metadata() {
    let mut page = CrawledPage::new("https://example.com/page".to_string(), 1, 200);
    page.final_url = Some("https://example.com/page".to_string());
    page.robots_obeyed = Some(true);
    page.fetch_time_ms = Some(150);

    assert_eq!(page.depth, 1);
    assert_eq!(page.final_url.as_deref(), Some("https://example.com/page"));
    assert_eq!(page.robots_obeyed, Some(true));
    assert_eq!(page.fetch_time_ms, Some(150));
}

#[test]
fn test_crawled_page_with_error() {
    let mut page = CrawledPage::new("https://example.com".to_string(), 0, 404);
    page.fetch_error = Some("Page not found".to_string());

    assert_eq!(page.status_code, 404);
    assert!(page.fetch_error.is_some());
    assert_eq!(page.fetch_error.as_deref(), Some("Page not found"));
}

#[test]
fn test_crawled_page_content_not_available_message() {
    let mut page = CrawledPage::new("https://example.com".to_string(), 0, 200);
    page.fetch_error = Some(
        "Page content not available - Spider engine does not persist crawled data. \
         Use 'stats' or 'urls' mode for metadata only.".to_string()
    );

    assert!(page.content.is_none());
    assert!(page.markdown.is_none());
    assert!(page.fetch_error.is_some());

    let error = page.fetch_error.as_ref().unwrap();
    assert!(error.contains("Page content not available"));
    assert!(error.contains("Spider engine does not persist"));
}

#[test]
fn test_crawled_page_include_filter_content() {
    let mut page = CrawledPage::new("https://example.com".to_string(), 0, 200);
    page.title = Some("Test Page".to_string());
    page.content = Some("Page content".to_string());
    page.markdown = Some("# Test Page".to_string());

    let filter = FieldFilter::parse("content");
    page.apply_field_filter(Some(&filter), None);

    // Only content should be included
    assert!(page.title.is_none());
    assert!(page.content.is_some());
    assert!(page.markdown.is_none());
}

#[test]
fn test_crawled_page_exclude_filter() {
    let mut page = CrawledPage::new("https://example.com".to_string(), 0, 200);
    page.title = Some("Test".to_string());
    page.final_url = Some("https://example.com".to_string());
    page.mime = Some("text/html".to_string());

    let filter = FieldFilter::parse("mime");
    page.apply_field_filter(None, Some(&filter));

    // mime should be excluded
    assert!(page.title.is_some());
    assert!(page.final_url.is_some());
    assert!(page.mime.is_none());
}

#[test]
fn test_crawled_page_multiple_field_include() {
    let mut page = CrawledPage::new("https://example.com".to_string(), 0, 200);
    page.title = Some("Title".to_string());
    page.content = Some("Content".to_string());
    page.markdown = Some("Markdown".to_string());
    page.links = vec!["https://example.com/link".to_string()];

    let filter = FieldFilter::parse("title,links");
    page.apply_field_filter(Some(&filter), None);

    assert!(page.title.is_some());
    assert!(!page.links.is_empty());
    assert!(page.content.is_none());
    assert!(page.markdown.is_none());
}

#[test]
fn test_crawled_page_no_truncation_needed() {
    let mut page = CrawledPage::new("https://example.com".to_string(), 0, 200);
    page.content = Some("Short content".to_string());

    page.truncate_content(1000);

    assert_eq!(page.content.as_deref(), Some("Short content"));
    assert!(page.truncated.is_none());
}

#[test]
fn test_crawled_page_truncation_applied() {
    let mut page = CrawledPage::new("https://example.com".to_string(), 0, 200);
    page.content = Some("x".repeat(2000));
    page.markdown = Some("y".repeat(2000));

    page.truncate_content(500);

    assert_eq!(page.content.as_ref().unwrap().len(), 500);
    assert_eq!(page.markdown.as_ref().unwrap().len(), 500);
    assert_eq!(page.truncated, Some(true));
}

#[test]
fn test_crawled_page_depth_tracking() {
    let page_depth_0 = CrawledPage::new("https://example.com".to_string(), 0, 200);
    let page_depth_1 = CrawledPage::new("https://example.com/page1".to_string(), 1, 200);
    let page_depth_2 = CrawledPage::new("https://example.com/page2".to_string(), 2, 200);

    assert_eq!(page_depth_0.depth, 0);
    assert_eq!(page_depth_1.depth, 1);
    assert_eq!(page_depth_2.depth, 2);
}

#[test]
fn test_crawled_page_status_codes() {
    let page_ok = CrawledPage::new("https://example.com".to_string(), 0, 200);
    let page_redirect = CrawledPage::new("https://example.com".to_string(), 0, 301);
    let page_not_found = CrawledPage::new("https://example.com".to_string(), 0, 404);
    let page_error = CrawledPage::new("https://example.com".to_string(), 0, 500);

    assert_eq!(page_ok.status_code, 200);
    assert_eq!(page_redirect.status_code, 301);
    assert_eq!(page_not_found.status_code, 404);
    assert_eq!(page_error.status_code, 500);
}

#[test]
fn test_crawled_page_robots_obedience() {
    let mut page = CrawledPage::new("https://example.com".to_string(), 0, 200);

    // Default is None
    assert!(page.robots_obeyed.is_none());

    // Can be set to true
    page.robots_obeyed = Some(true);
    assert_eq!(page.robots_obeyed, Some(true));

    // Can be set to false (for disallowed pages that were still crawled)
    page.robots_obeyed = Some(false);
    assert_eq!(page.robots_obeyed, Some(false));
}

#[test]
fn test_crawled_page_links_collection() {
    let mut page = CrawledPage::new("https://example.com".to_string(), 0, 200);

    assert!(page.links.is_empty());

    page.links = vec![
        "https://example.com/page1".to_string(),
        "https://example.com/page2".to_string(),
        "https://example.com/page3".to_string(),
    ];

    assert_eq!(page.links.len(), 3);
    assert_eq!(page.links[0], "https://example.com/page1");
}

#[test]
fn test_crawled_page_final_url_after_redirects() {
    let mut page = CrawledPage::new("https://example.com/old".to_string(), 0, 301);
    page.final_url = Some("https://example.com/new".to_string());

    assert_eq!(page.url, "https://example.com/old");
    assert_eq!(page.final_url.as_deref(), Some("https://example.com/new"));
}

#[test]
fn test_crawled_page_mime_type() {
    let mut page = CrawledPage::new("https://example.com/doc.pdf".to_string(), 0, 200);
    page.mime = Some("application/pdf".to_string());

    assert_eq!(page.mime.as_deref(), Some("application/pdf"));
}

#[test]
fn test_crawled_page_parse_error() {
    let mut page = CrawledPage::new("https://example.com".to_string(), 0, 200);
    page.parse_error = Some("Invalid HTML structure".to_string());

    assert!(page.content.is_none());
    assert!(page.parse_error.is_some());
    assert_eq!(page.parse_error.as_deref(), Some("Invalid HTML structure"));
}
