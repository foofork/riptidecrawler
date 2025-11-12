//! Live website crawl integration tests
//!
//! These tests crawl real websites to validate end-to-end functionality.
//! They are marked #[ignore] by default and should be run manually or in
//! scheduled CI/CD runs (not on every commit).
//!
//! Run with: cargo test --test live_crawl_integration -- --ignored --nocapture
//!
//! NOTE: These tests will be fully functional after Phase 2B when CrawlFacade
//! is completely implemented. Currently they serve as documentation for the
//! intended testing strategy.

use riptide_types::config::CrawlOptions;
use std::time::Duration;

// Re-export types needed for test signatures
pub enum CrawlMode {
    Standard,
    Enhanced,
}

pub enum CrawlResult {
    Standard(String),
    Enhanced(String),
}

pub struct CrawlFacade;

/// Test crawling example.com - the simplest possible HTML page
#[tokio::test]
#[ignore] // Requires network and optionally Chrome
async fn test_crawl_example_com() {
    let options = CrawlOptions::default();

    // Note: This test will use HTTP client (not browser) unless Chrome is available
    let result = tokio::time::timeout(
        Duration::from_secs(30),
        crawl_url("https://example.com", options, CrawlMode::Standard),
    )
    .await;

    assert!(result.is_ok(), "Request timed out");
    let crawl_result = result.unwrap();

    assert!(
        crawl_result.is_ok(),
        "Crawl failed: {:?}",
        crawl_result.err()
    );
    let content = crawl_result.unwrap();

    // Verify we got content
    assert!(!content.is_empty(), "Content should not be empty");
    assert!(
        content.to_lowercase().contains("example"),
        "Should contain 'example'"
    );

    println!("✓ Successfully crawled example.com");
    println!("  Content length: {} bytes", content.len());
}

/// Test crawling httpbin.org - HTTP testing service
#[tokio::test]
#[ignore] // Requires network
async fn test_crawl_httpbin() {
    let options = CrawlOptions::default();

    let result = tokio::time::timeout(
        Duration::from_secs(30),
        crawl_url("https://httpbin.org/html", options, CrawlMode::Standard),
    )
    .await;

    assert!(result.is_ok(), "Request timed out");
    let crawl_result = result.unwrap();

    assert!(
        crawl_result.is_ok(),
        "Crawl failed: {:?}",
        crawl_result.err()
    );
    let content = crawl_result.unwrap();

    assert!(!content.is_empty(), "Content should not be empty");
    assert!(content.contains("<h1>"), "Should contain HTML");

    println!("✓ Successfully crawled httpbin.org");
    println!("  Content length: {} bytes", content.len());
}

/// Test crawling quotes.toscrape.com - JavaScript-rendered site
#[tokio::test]
#[ignore] // Requires network and potentially Chrome for JS
async fn test_crawl_quotes_toscrape() {
    let options = CrawlOptions::default();

    let result = tokio::time::timeout(
        Duration::from_secs(30),
        crawl_url("https://quotes.toscrape.com", options, CrawlMode::Standard),
    )
    .await;

    assert!(result.is_ok(), "Request timed out");
    let crawl_result = result.unwrap();

    assert!(
        crawl_result.is_ok(),
        "Crawl failed: {:?}",
        crawl_result.err()
    );
    let content = crawl_result.unwrap();

    assert!(!content.is_empty(), "Content should not be empty");
    assert!(
        content.to_lowercase().contains("quote") || content.to_lowercase().contains("quotes"),
        "Should contain quotes"
    );

    println!("✓ Successfully crawled quotes.toscrape.com");
    println!("  Content length: {} bytes", content.len());
}

/// Test batch crawling multiple sites
#[tokio::test]
#[ignore] // Requires network
async fn test_batch_crawl_multiple_sites() {
    let urls = vec![
        "https://example.com",
        "https://httpbin.org/html",
        "https://quotes.toscrape.com",
    ];

    println!("Testing batch crawl of {} sites...", urls.len());

    for (i, url) in urls.iter().enumerate() {
        println!("  [{}/{}] Crawling {}...", i + 1, urls.len(), url);

        let options = CrawlOptions::default();
        let result = tokio::time::timeout(
            Duration::from_secs(30),
            crawl_url(url, options, CrawlMode::Standard),
        )
        .await;

        assert!(result.is_ok(), "Timeout crawling {}", url);
        let crawl_result = result.unwrap();
        assert!(
            crawl_result.is_ok(),
            "Failed to crawl {}: {:?}",
            url,
            crawl_result.err()
        );

        let content = crawl_result.unwrap();
        assert!(!content.is_empty(), "Empty content from {}", url);

        println!("    ✓ Success ({} bytes)", content.len());
    }

    println!("\n✓ All {} sites crawled successfully", urls.len());
}

/// Test crawling with custom options
#[tokio::test]
#[ignore] // Requires network
async fn test_crawl_with_custom_options() {
    let mut options = CrawlOptions::default();
    options.concurrency = 2;

    let result = tokio::time::timeout(
        Duration::from_secs(30),
        crawl_url("https://example.com", options, CrawlMode::Standard),
    )
    .await;

    assert!(result.is_ok(), "Request timed out");
    assert!(result.unwrap().is_ok(), "Crawl with custom options failed");

    println!("✓ Successfully crawled with custom options");
}

/// Test enhanced mode crawling (requires browser)
#[tokio::test]
#[ignore] // Requires Chrome and network
async fn test_enhanced_crawl_with_javascript() {
    let options = CrawlOptions::default();

    // Enhanced mode uses browser for JavaScript rendering
    let result = tokio::time::timeout(
        Duration::from_secs(45),
        crawl_url("https://quotes.toscrape.com", options, CrawlMode::Enhanced),
    )
    .await;

    assert!(result.is_ok(), "Request timed out");
    let crawl_result = result.unwrap();

    // Enhanced mode might fail if Chrome not available - that's OK
    if crawl_result.is_err() {
        println!("⚠ Enhanced mode failed (Chrome might not be available)");
        println!("  Error: {:?}", crawl_result.err());
        return;
    }

    let content = crawl_result.unwrap();
    assert!(!content.is_empty(), "Content should not be empty");

    println!("✓ Successfully crawled with Enhanced mode (JavaScript rendered)");
    println!("  Content length: {} bytes", content.len());
}

/// Test error handling with invalid URL
#[tokio::test]
#[ignore] // Requires network
async fn test_crawl_invalid_url_handling() {
    let options = CrawlOptions::default();

    let result = tokio::time::timeout(
        Duration::from_secs(15),
        crawl_url(
            "https://this-domain-definitely-does-not-exist-12345.com",
            options,
            CrawlMode::Standard,
        ),
    )
    .await;

    // Should complete (not timeout) but return error
    assert!(result.is_ok(), "Should not timeout");
    assert!(result.unwrap().is_err(), "Should fail for invalid domain");

    println!("✓ Correctly handled invalid URL");
}

/// Test crawling with retry on failure
#[tokio::test]
#[ignore] // Requires network
async fn test_crawl_with_retry() {
    let options = CrawlOptions::default();

    let url = "https://httpbin.org/html";
    let mut attempts = 0;
    let max_attempts = 3;

    loop {
        attempts += 1;
        println!("  Attempt {}/{} for {}...", attempts, max_attempts, url);

        let result = tokio::time::timeout(
            Duration::from_secs(30),
            crawl_url(url, options.clone(), CrawlMode::Standard),
        )
        .await;

        if result.is_ok() && result.unwrap().is_ok() {
            println!("✓ Success on attempt {}", attempts);
            break;
        }

        if attempts >= max_attempts {
            panic!("Failed after {} attempts", max_attempts);
        }

        println!("  Retrying...");
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

// Helper function to crawl a URL
async fn crawl_url(_url: &str, _options: CrawlOptions, _mode: CrawlMode) -> Result<String, String> {
    // Placeholder implementation - will be completed in Phase 2B
    // when CrawlFacade is fully implemented
    Err("CrawlFacade not yet fully implemented - pending Phase 2B completion".to_string())
}

// Helper to create test facade
#[allow(dead_code)]
async fn create_test_facade(_options: CrawlOptions) -> Result<CrawlFacade, String> {
    // For now, return a simple error - this will be implemented when we add
    // the actual CrawlFacade::new() constructor in the next phase
    Err("CrawlFacade not yet fully implemented - pending Phase 2B completion".to_string())
}
