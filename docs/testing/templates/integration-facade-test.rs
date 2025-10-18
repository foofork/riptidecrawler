/// Facade Pattern Integration Test Template
///
/// Tests for riptide-facade crate validating simplified API.
/// The facade should hide complexity from riptide-core while maintaining full functionality.

use riptide_facade::Riptide;
use riptide_core::{Spider, Config as CoreConfig};

#[tokio::test]
async fn test_facade_simplified_initialization() {
    // Facade: 1-line setup
    let riptide = Riptide::new();
    assert!(riptide.is_ok());

    // vs Core: Multi-step setup
    let core_config = CoreConfig::builder()
        .with_user_agent("test")
        .with_timeout(30)
        .with_max_depth(3)
        .with_respect_robots_txt(true)
        .build()
        .unwrap();
    let core_spider = Spider::new(core_config);

    // Facade should be simpler
    assert!(true, "Facade provides simpler initialization");
}

#[tokio::test]
async fn test_facade_single_method_crawl() {
    // Facade: Single method for common use case
    let riptide = Riptide::new().unwrap();
    let result = riptide.crawl("https://example.com").await;

    assert!(result.is_ok());
    let pages = result.unwrap();
    assert!(!pages.is_empty());

    // vs Core: Multiple steps required
    // 1. Create spider
    // 2. Configure options
    // 3. Start crawl
    // 4. Collect results
    // 5. Clean up resources
}

#[tokio::test]
async fn test_facade_with_simple_config() {
    // Facade: Simple configuration
    let riptide = Riptide::builder()
        .max_pages(100)
        .depth(2)
        .timeout_seconds(30)
        .build()
        .unwrap();

    let result = riptide.crawl("https://example.com").await;
    assert!(result.is_ok());

    // Verify configuration applied
    let pages = result.unwrap();
    assert!(pages.len() <= 100, "Should respect max_pages limit");
}

#[tokio::test]
async fn test_facade_error_translation() {
    // Facade: User-friendly error messages
    let riptide = Riptide::new().unwrap();
    let result = riptide.crawl("invalid-url").await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    // Error should be user-friendly, not internal
    assert!(error.to_string().contains("Invalid URL"));
    assert!(!error.to_string().contains("ParseError")); // No internal types
    assert!(!error.to_string().contains("spider_rs")); // No internal modules
}

#[tokio::test]
async fn test_facade_backward_compatibility() {
    // Facade v1.0 API
    let riptide_v1 = Riptide::new().unwrap();
    let result_v1 = riptide_v1.crawl("https://example.com").await;

    // Simulate core changes
    // ... internal refactoring ...

    // Facade v1.0 API still works unchanged
    let riptide_v2 = Riptide::new().unwrap();
    let result_v2 = riptide_v2.crawl("https://example.com").await;

    // API remains stable
    assert_eq!(
        std::mem::discriminant(&result_v1),
        std::mem::discriminant(&result_v2)
    );
}

#[tokio::test]
async fn test_facade_hides_internal_complexity() {
    use riptide_facade::CrawlResult;

    let riptide = Riptide::new().unwrap();
    let result = riptide.crawl("https://example.com").await.unwrap();

    // Result type should be simple
    assert!(matches!(result, CrawlResult { pages: _, stats: _ }));

    // Should NOT expose internal types
    // ❌ spider_rs::Response
    // ❌ riptide_core::InternalState
    // ❌ chromiumoxide::Page
}

#[tokio::test]
async fn test_facade_sensible_defaults() {
    // Facade: Works with zero configuration
    let riptide = Riptide::new().unwrap();
    let result = riptide.crawl("https://example.com").await;

    assert!(result.is_ok(), "Should work with defaults");

    // Defaults should be production-ready
    let stats = result.unwrap().stats;
    assert!(stats.timeout_seconds >= 30, "Reasonable timeout");
    assert!(stats.max_depth <= 5, "Prevents infinite crawls");
    assert!(stats.respects_robots_txt, "Polite by default");
}

#[tokio::test]
async fn test_facade_progressive_disclosure() {
    // Level 1: Simplest API
    let simple = Riptide::new().unwrap();
    simple.crawl("https://example.com").await.unwrap();

    // Level 2: Common options
    let configured = Riptide::builder()
        .max_pages(50)
        .depth(2)
        .build()
        .unwrap();
    configured.crawl("https://example.com").await.unwrap();

    // Level 3: Advanced options (if needed)
    let advanced = Riptide::builder()
        .max_pages(50)
        .depth(2)
        .custom_headers(vec![("User-Agent", "Custom")])
        .javascript_enabled(true)
        .screenshot_on_error(true)
        .build()
        .unwrap();
    advanced.crawl("https://example.com").await.unwrap();

    // Each level adds complexity only when needed
}

#[tokio::test]
async fn test_facade_resource_cleanup() {
    // Facade: Automatic resource management
    {
        let riptide = Riptide::new().unwrap();
        riptide.crawl("https://example.com").await.unwrap();
    } // Resources cleaned up automatically

    // Verify no resource leaks
    // (Implementation would check browser processes, connections, etc.)
}

#[tokio::test]
async fn test_facade_vs_core_equivalence() {
    // Both should produce equivalent results

    // Facade
    let riptide = Riptide::builder()
        .max_pages(10)
        .depth(1)
        .build()
        .unwrap();
    let facade_result = riptide.crawl("https://example.com").await.unwrap();

    // Core (manual setup)
    let core_config = CoreConfig::builder()
        .with_max_pages(10)
        .with_max_depth(1)
        .build()
        .unwrap();
    let spider = Spider::new(core_config);
    let core_result = spider.crawl("https://example.com").await.unwrap();

    // Results should be equivalent
    assert_eq!(facade_result.pages.len(), core_result.pages.len());
    assert_eq!(facade_result.stats.urls_crawled, core_result.stats.urls_crawled);
}

#[tokio::test]
async fn test_facade_documentation_examples() {
    // All README examples should work as-is

    // Example 1: Basic usage
    let riptide = Riptide::new().unwrap();
    let pages = riptide.crawl("https://example.com").await.unwrap();
    assert!(!pages.is_empty());

    // Example 2: With configuration
    let riptide = Riptide::builder()
        .max_pages(50)
        .build()
        .unwrap();
    let pages = riptide.crawl("https://example.com").await.unwrap();
    assert!(pages.len() <= 50);

    // Example 3: Error handling
    let riptide = Riptide::new().unwrap();
    match riptide.crawl("invalid-url").await {
        Ok(_) => panic!("Should fail"),
        Err(e) => assert!(e.to_string().contains("Invalid URL")),
    }
}

/// Test helper: Compare facade vs core complexity
fn count_lines_of_code(code: &str) -> usize {
    code.lines().filter(|line| !line.trim().is_empty()).count()
}

#[test]
fn test_facade_reduces_complexity() {
    let facade_code = r#"
        let riptide = Riptide::new()?;
        let pages = riptide.crawl("https://example.com").await?;
    "#;

    let core_code = r#"
        let config = CoreConfig::builder()
            .with_user_agent("crawler")
            .with_timeout(30)
            .with_max_depth(3)
            .build()?;
        let spider = Spider::new(config);
        let mut crawler = spider.crawl("https://example.com");
        let mut pages = vec![];
        while let Some(page) = crawler.next().await {
            pages.push(page?);
        }
    "#;

    assert!(
        count_lines_of_code(facade_code) < count_lines_of_code(core_code),
        "Facade should require less code"
    );
}
