/// Unit tests for Phase 2 result mode implementation
/// Tests ResultMode enum, CrawledPage struct, SpiderResultPages, and field filtering

use riptide_api::dto::{
    CrawledPage, FieldFilter, ResultMode, SpiderResultPages, SpiderResultStats, SpiderResultUrls,
};
use serde_json;

#[test]
fn test_result_mode_enum_variants() {
    // Test all variants exist
    let stats = ResultMode::Stats;
    let urls = ResultMode::Urls;
    let pages = ResultMode::Pages;
    let stream = ResultMode::Stream;
    let store = ResultMode::Store;

    assert_eq!(stats, ResultMode::Stats);
    assert_eq!(urls, ResultMode::Urls);
    assert_eq!(pages, ResultMode::Pages);
    assert_eq!(stream, ResultMode::Stream);
    assert_eq!(store, ResultMode::Store);
}

#[test]
fn test_result_mode_default() {
    // Test default is Stats for backward compatibility
    let default_mode = ResultMode::default();
    assert_eq!(default_mode, ResultMode::Stats);
}

#[test]
fn test_result_mode_serialization() {
    // Test serde serialization
    let stats_json = serde_json::to_string(&ResultMode::Stats).unwrap();
    assert_eq!(stats_json, "\"stats\"");

    let urls_json = serde_json::to_string(&ResultMode::Urls).unwrap();
    assert_eq!(urls_json, "\"urls\"");

    let pages_json = serde_json::to_string(&ResultMode::Pages).unwrap();
    assert_eq!(pages_json, "\"pages\"");

    let stream_json = serde_json::to_string(&ResultMode::Stream).unwrap();
    assert_eq!(stream_json, "\"stream\"");

    let store_json = serde_json::to_string(&ResultMode::Store).unwrap();
    assert_eq!(store_json, "\"store\"");
}

#[test]
fn test_result_mode_deserialization() {
    // Test serde deserialization
    let stats: ResultMode = serde_json::from_str("\"stats\"").unwrap();
    assert_eq!(stats, ResultMode::Stats);

    let urls: ResultMode = serde_json::from_str("\"urls\"").unwrap();
    assert_eq!(urls, ResultMode::Urls);

    let pages: ResultMode = serde_json::from_str("\"pages\"").unwrap();
    assert_eq!(pages, ResultMode::Pages);

    let stream: ResultMode = serde_json::from_str("\"stream\"").unwrap();
    assert_eq!(stream, ResultMode::Stream);

    let store: ResultMode = serde_json::from_str("\"store\"").unwrap();
    assert_eq!(store, ResultMode::Store);
}

#[test]
fn test_crawled_page_creation() {
    // Test CrawledPage creation
    let page = CrawledPage::new(
        "https://example.com".to_string(),
        1,
        200,
    );

    assert_eq!(page.url, "https://example.com");
    assert_eq!(page.depth, 1);
    assert_eq!(page.status_code, 200);
    assert!(page.title.is_none());
    assert!(page.content.is_none());
    assert!(page.markdown.is_none());
    assert!(page.links.is_empty());
    assert!(page.truncated.is_none());
}

#[test]
fn test_crawled_page_serialization() {
    // Test that None fields are skipped in serialization
    let page = CrawledPage::new(
        "https://example.com".to_string(),
        1,
        200,
    );

    let json = serde_json::to_string(&page).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(parsed.get("url").is_some());
    assert!(parsed.get("depth").is_some());
    assert!(parsed.get("status_code").is_some());
    assert!(parsed.get("links").is_some());

    // None fields should be skipped
    assert!(parsed.get("title").is_none());
    assert!(parsed.get("content").is_none());
    assert!(parsed.get("markdown").is_none());
    assert!(parsed.get("truncated").is_none());
}

#[test]
fn test_field_filter_creation() {
    // Test FieldFilter creation from string
    let filter = FieldFilter::from_str("title,links,markdown");

    assert!(filter.has_field("title"));
    assert!(filter.has_field("links"));
    assert!(filter.has_field("markdown"));
    assert!(!filter.has_field("content"));
}

#[test]
fn test_field_filter_whitespace_handling() {
    // Test that whitespace is trimmed
    let filter = FieldFilter::from_str("title, links , markdown");

    assert!(filter.has_field("title"));
    assert!(filter.has_field("links"));
    assert!(filter.has_field("markdown"));
}

#[test]
fn test_crawled_page_field_filtering_include() {
    // Test include filtering
    let mut page = CrawledPage::new(
        "https://example.com".to_string(),
        1,
        200,
    );
    page.title = Some("Test Title".to_string());
    page.content = Some("Test Content".to_string());
    page.markdown = Some("# Test Markdown".to_string());
    page.links = vec!["https://link1.com".to_string()];

    let include_filter = FieldFilter::from_str("title,links");
    page.apply_field_filter(Some(&include_filter), None);

    // Should keep only title and links
    assert!(page.title.is_some());
    assert!(page.links.len() > 0);
    assert!(page.content.is_none());
    assert!(page.markdown.is_none());
}

#[test]
fn test_crawled_page_field_filtering_exclude() {
    // Test exclude filtering
    let mut page = CrawledPage::new(
        "https://example.com".to_string(),
        1,
        200,
    );
    page.title = Some("Test Title".to_string());
    page.content = Some("Test Content".to_string());
    page.markdown = Some("# Test Markdown".to_string());

    let exclude_filter = FieldFilter::from_str("content");
    page.apply_field_filter(None, Some(&exclude_filter));

    // Should remove only content
    assert!(page.title.is_some());
    assert!(page.markdown.is_some());
    assert!(page.content.is_none());
}

#[test]
fn test_crawled_page_content_truncation() {
    // Test content truncation
    let mut page = CrawledPage::new(
        "https://example.com".to_string(),
        1,
        200,
    );

    // Create content larger than limit
    let large_content = "x".repeat(2000);
    page.content = Some(large_content.clone());
    page.markdown = Some(large_content.clone());

    page.truncate_content(1000);

    // Content should be truncated
    assert_eq!(page.content.as_ref().unwrap().len(), 1000);
    assert_eq!(page.markdown.as_ref().unwrap().len(), 1000);
    assert_eq!(page.truncated, Some(true));
}

#[test]
fn test_crawled_page_no_truncation_when_small() {
    // Test that small content is not truncated
    let mut page = CrawledPage::new(
        "https://example.com".to_string(),
        1,
        200,
    );

    page.content = Some("small content".to_string());
    page.markdown = Some("# Small Markdown".to_string());

    page.truncate_content(1000);

    // Content should not be truncated
    assert_eq!(page.content.as_ref().unwrap(), "small content");
    assert_eq!(page.markdown.as_ref().unwrap(), "# Small Markdown");
    assert!(page.truncated.is_none());
}

#[test]
fn test_spider_result_pages_creation() {
    // Test SpiderResultPages creation
    let pages = vec![
        CrawledPage::new("https://example.com".to_string(), 0, 200),
        CrawledPage::new("https://example.com/page1".to_string(), 1, 200),
    ];

    let result = SpiderResultPages {
        pages_crawled: 2,
        pages_failed: 0,
        duration_seconds: 1.5,
        stop_reason: "max_pages_reached".to_string(),
        domains: vec!["example.com".to_string()],
        pages: pages.clone(),
        api_version: "v1".to_string(),
    };

    assert_eq!(result.pages_crawled, 2);
    assert_eq!(result.pages_failed, 0);
    assert_eq!(result.pages.len(), 2);
    assert_eq!(result.api_version, "v1");
}

#[test]
fn test_spider_result_pages_field_filtering() {
    // Test field filtering on all pages
    let mut pages = vec![
        CrawledPage::new("https://example.com".to_string(), 0, 200),
        CrawledPage::new("https://example.com/page1".to_string(), 1, 200),
    ];

    for page in &mut pages {
        page.title = Some("Title".to_string());
        page.content = Some("Content".to_string());
    }

    let mut result = SpiderResultPages {
        pages_crawled: 2,
        pages_failed: 0,
        duration_seconds: 1.5,
        stop_reason: "max_pages_reached".to_string(),
        domains: vec!["example.com".to_string()],
        pages,
        api_version: "v1".to_string(),
    };

    let include_filter = FieldFilter::from_str("title");
    result.apply_field_filter(Some(&include_filter), None);

    // All pages should have content removed
    for page in &result.pages {
        assert!(page.title.is_some());
        assert!(page.content.is_none());
    }
}

#[test]
fn test_spider_result_pages_truncation() {
    // Test content truncation on all pages
    let mut pages = vec![
        CrawledPage::new("https://example.com".to_string(), 0, 200),
        CrawledPage::new("https://example.com/page1".to_string(), 1, 200),
    ];

    let large_content = "x".repeat(2000);
    for page in &mut pages {
        page.content = Some(large_content.clone());
    }

    let mut result = SpiderResultPages {
        pages_crawled: 2,
        pages_failed: 0,
        duration_seconds: 1.5,
        stop_reason: "max_pages_reached".to_string(),
        domains: vec!["example.com".to_string()],
        pages,
        api_version: "v1".to_string(),
    };

    result.truncate_content(1000);

    // All pages should have truncated content
    for page in &result.pages {
        assert_eq!(page.content.as_ref().unwrap().len(), 1000);
        assert_eq!(page.truncated, Some(true));
    }
}

#[test]
fn test_backward_compatibility_stats() {
    // Test that SpiderResultStats still works (backward compatibility)
    let stats = SpiderResultStats {
        pages_crawled: 10,
        pages_failed: 1,
        duration_seconds: 2.5,
        stop_reason: "max_pages_reached".to_string(),
        domains: vec!["example.com".to_string()],
    };

    // Should serialize without errors
    let json = serde_json::to_string(&stats).unwrap();
    assert!(json.contains("pages_crawled"));
    assert!(json.contains("pages_failed"));
}

#[test]
fn test_backward_compatibility_urls() {
    // Test that SpiderResultUrls still works
    let urls = SpiderResultUrls {
        pages_crawled: 10,
        pages_failed: 1,
        duration_seconds: 2.5,
        stop_reason: "max_pages_reached".to_string(),
        domains: vec!["example.com".to_string()],
        discovered_urls: vec![
            "https://example.com/page1".to_string(),
            "https://example.com/page2".to_string(),
        ],
    };

    // Should serialize without errors
    let json = serde_json::to_string(&urls).unwrap();
    assert!(json.contains("discovered_urls"));
}
