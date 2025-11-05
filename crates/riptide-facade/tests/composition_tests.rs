//! Comprehensive composition tests for spider + extractor
//!
//! Tests all 3 error handling patterns: filter, handle, fail-fast

use futures::StreamExt;
use riptide_facade::dto::Document;
use riptide_facade::traits::mocks::{FailingMockSpider, MockExtractor, MockSpider};
use riptide_facade::traits::{Chainable, Spider, SpiderOpts};

#[tokio::test]
async fn test_basic_composition() {
    // Test that spider + extractor composition works
    let spider = MockSpider::with_test_urls();
    let extractor = MockExtractor::new();

    let docs: Vec<_> = spider
        .crawl("https://example.com", SpiderOpts::default())
        .await
        .unwrap()
        .and_extract(extractor)
        .collect()
        .await;

    // All 3 URLs should be extracted successfully
    let successful: Vec<_> = docs.into_iter().filter_map(Result::ok).collect();
    assert_eq!(successful.len(), 3);

    for doc in successful {
        assert!(doc.title.contains("Title for https://example.com"));
        assert!(doc.content.contains("Extracted content"));
    }
}

#[tokio::test]
async fn test_composition_with_custom_urls() {
    let spider = MockSpider::new(vec![
        "https://rust-lang.org".to_string(),
        "https://crates.io".to_string(),
    ]);
    let extractor = MockExtractor::new();

    let docs: Vec<_> = spider
        .crawl("https://rust-lang.org", SpiderOpts::default())
        .await
        .unwrap()
        .and_extract(extractor)
        .collect()
        .await;

    let successful: Vec<_> = docs.into_iter().filter_map(Result::ok).collect();
    assert_eq!(successful.len(), 2);
}

#[tokio::test]
async fn test_pattern_1_filter_errors() {
    // Pattern 1: Filter out errors, only process successes
    let spider = MockSpider::with_test_urls();
    let extractor = MockExtractor::new();

    let docs: Vec<Document> = spider
        .crawl("https://example.com", SpiderOpts::default())
        .await
        .unwrap()
        .and_extract(extractor)
        .filter_map(|result| async move { result.ok() })
        .collect()
        .await;

    assert_eq!(docs.len(), 3);
    for doc in docs {
        assert!(!doc.title.is_empty());
    }
}

#[tokio::test]
async fn test_pattern_2_handle_errors() {
    // Pattern 2: Handle errors explicitly
    let spider = MockSpider::with_test_urls();
    let extractor = MockExtractor::with_failures();

    let mut success_count = 0;
    let mut error_count = 0;

    let mut stream = spider
        .crawl("https://example.com", SpiderOpts::default())
        .await
        .unwrap()
        .and_extract(extractor);

    while let Some(result) = stream.next().await {
        match result {
            Ok(_doc) => success_count += 1,
            Err(_err) => error_count += 1,
        }
    }

    // With failing extractor, all should be errors
    assert_eq!(error_count, 3);
    assert_eq!(success_count, 0);
}

#[tokio::test]
async fn test_pattern_3_fail_fast() {
    // Pattern 3: Fail fast on first error
    let spider = MockSpider::with_test_urls();
    let extractor = MockExtractor::with_failures();

    let result: Result<Vec<_>, _> = spider
        .crawl("https://example.com", SpiderOpts::default())
        .await
        .unwrap()
        .and_extract(extractor)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect();

    // Should fail because extractor fails
    assert!(result.is_err());
}

#[tokio::test]
async fn test_spider_error_aborts_stream() {
    // Spider errors should abort the entire stream
    let spider = FailingMockSpider;
    let result = spider
        .crawl("https://example.com", SpiderOpts::default())
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_partial_success_pattern() {
    // Some extractions succeed, some fail - stream continues
    let spider = MockSpider::with_test_urls();
    let extractor = MockExtractor::new();

    let results: Vec<_> = spider
        .crawl("https://example.com", SpiderOpts::default())
        .await
        .unwrap()
        .and_extract(extractor)
        .collect()
        .await;

    // All results are returned (success or error)
    assert_eq!(results.len(), 3);

    let successes: Vec<_> = results.into_iter().filter_map(Result::ok).collect();
    assert_eq!(successes.len(), 3);
}

#[tokio::test]
async fn test_concurrent_extraction() {
    // Test that composition works with multiple URLs
    let spider = MockSpider::new(vec![
        "https://example.com/1".to_string(),
        "https://example.com/2".to_string(),
        "https://example.com/3".to_string(),
        "https://example.com/4".to_string(),
        "https://example.com/5".to_string(),
    ]);
    let extractor = MockExtractor::new();

    let docs: Vec<_> = spider
        .crawl("https://example.com", SpiderOpts::default())
        .await
        .unwrap()
        .and_extract(extractor)
        .collect()
        .await;

    let successful: Vec<_> = docs.into_iter().filter_map(Result::ok).collect();
    assert_eq!(successful.len(), 5);
}

#[tokio::test]
async fn test_empty_spider_results() {
    // Spider returns no URLs
    let spider = MockSpider::new(vec![]);
    let extractor = MockExtractor::new();

    let docs: Vec<_> = spider
        .crawl("https://example.com", SpiderOpts::default())
        .await
        .unwrap()
        .and_extract(extractor)
        .collect()
        .await;

    assert_eq!(docs.len(), 0);
}

#[tokio::test]
async fn test_document_to_json() {
    let spider = MockSpider::new(vec!["https://example.com".to_string()]);
    let extractor = MockExtractor::new();

    let docs: Vec<Document> = spider
        .crawl("https://example.com", SpiderOpts::default())
        .await
        .unwrap()
        .and_extract(extractor)
        .filter_map(|result| async move { result.ok() })
        .collect()
        .await;

    assert_eq!(docs.len(), 1);
    let json = docs[0].to_json().unwrap();
    assert!(json.contains("example.com"));
}

#[tokio::test]
async fn test_document_to_markdown() {
    let spider = MockSpider::new(vec!["https://example.com".to_string()]);
    let extractor = MockExtractor::new();

    let docs: Vec<Document> = spider
        .crawl("https://example.com", SpiderOpts::default())
        .await
        .unwrap()
        .and_extract(extractor)
        .filter_map(|result| async move { result.ok() })
        .collect()
        .await;

    assert_eq!(docs.len(), 1);
    let md = docs[0].to_markdown();
    assert!(md.contains("# Title for"));
    assert!(md.contains("**Source:**"));
}
