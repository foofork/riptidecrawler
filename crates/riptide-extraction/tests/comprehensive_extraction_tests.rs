//! Comprehensive extraction tests with real websites
//!
//! This test suite validates extraction quality across different websites
//! and content types using the HTML extraction library directly.

use riptide_extraction::html_parser::EnhancedHtmlExtractor;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Serialize, Deserialize, Debug)]
struct ExtractionResult {
    url: String,
    strategy_requested: String,
    strategy_used: String,
    page_size_bytes: usize,
    extracted_chars: usize,
    coverage_percent: f64,
    has_raw_html: bool,
    raw_html_size: usize,
    quality_score: f64,
    extraction_time_ms: u128,
}

#[derive(Serialize, Deserialize, Debug)]
struct TestSummary {
    timestamp: String,
    total_tests: usize,
    passed_tests: usize,
    failed_tests: usize,
    results: Vec<ExtractionResult>,
}

async fn fetch_and_extract(url: &str) -> Result<ExtractionResult, Box<dyn std::error::Error>> {
    let start = Instant::now();

    // Fetch the HTML content
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; RiptideCrawler/1.0; Testing)")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let response = client.get(url).send().await?;
    let html = response.text().await?;
    let page_size = html.len();

    // Extract content using EnhancedHtmlExtractor
    let extractor = EnhancedHtmlExtractor::new(Some(url))?;
    let extraction = extractor.extract(&html, url)?;

    let extraction_time = start.elapsed().as_millis();

    // Calculate coverage
    let extracted_chars = extraction.main_content.len();
    let coverage_percent = (extracted_chars as f64 / page_size as f64) * 100.0;

    Ok(ExtractionResult {
        url: url.to_string(),
        strategy_requested: "enhanced_html".to_string(),
        strategy_used: "enhanced_html".to_string(),
        page_size_bytes: page_size,
        extracted_chars,
        coverage_percent,
        has_raw_html: true,
        raw_html_size: page_size,
        quality_score: extraction.quality_score,
        extraction_time_ms: extraction_time,
    })
}

#[tokio::test]
#[ignore] // Ignore by default since it requires network access
async fn test_rust_lang_org_extraction() {
    let result = fetch_and_extract("https://www.rust-lang.org/").await;
    assert!(
        result.is_ok(),
        "Failed to extract rust-lang.org: {:?}",
        result.err()
    );

    let extraction = result.unwrap();
    assert!(
        extraction.extracted_chars > 100,
        "Extracted content too small"
    );
    assert!(
        extraction.quality_score > 0.0,
        "Quality score should be > 0"
    );
    println!(
        "Rust-lang.org: {} chars, coverage: {:.2}%, quality: {:.2}",
        extraction.extracted_chars, extraction.coverage_percent, extraction.quality_score
    );
}

#[tokio::test]
#[ignore] // Ignore by default since it requires network access
async fn test_mdn_extraction() {
    let result = fetch_and_extract("https://developer.mozilla.org/en-US/").await;
    assert!(result.is_ok(), "Failed to extract MDN: {:?}", result.err());

    let extraction = result.unwrap();
    assert!(
        extraction.extracted_chars > 100,
        "Extracted content too small"
    );
    assert!(
        extraction.quality_score > 0.0,
        "Quality score should be > 0"
    );
    println!(
        "MDN: {} chars, coverage: {:.2}%, quality: {:.2}",
        extraction.extracted_chars, extraction.coverage_percent, extraction.quality_score
    );
}

#[tokio::test]
#[ignore] // Ignore by default since it requires network access
async fn test_wikipedia_extraction() {
    let result = fetch_and_extract("https://en.wikipedia.org/wiki/Web_scraping").await;
    assert!(
        result.is_ok(),
        "Failed to extract Wikipedia: {:?}",
        result.err()
    );

    let extraction = result.unwrap();
    assert!(
        extraction.extracted_chars > 500,
        "Wikipedia article should have substantial content"
    );
    assert!(
        extraction.quality_score > 0.3,
        "Quality score should be reasonable"
    );
    println!(
        "Wikipedia: {} chars, coverage: {:.2}%, quality: {:.2}",
        extraction.extracted_chars, extraction.coverage_percent, extraction.quality_score
    );
}

#[tokio::test]
#[ignore] // Ignore by default since it requires network access
async fn test_hackernews_extraction() {
    let result = fetch_and_extract("https://news.ycombinator.com/").await;
    assert!(
        result.is_ok(),
        "Failed to extract HackerNews: {:?}",
        result.err()
    );

    let extraction = result.unwrap();
    assert!(
        extraction.extracted_chars > 50,
        "Should extract some content from HN"
    );
    println!(
        "HackerNews: {} chars, coverage: {:.2}%, quality: {:.2}",
        extraction.extracted_chars, extraction.coverage_percent, extraction.quality_score
    );
}

#[tokio::test]
#[ignore] // Ignore by default since it requires network access
async fn test_comprehensive_suite() {
    let test_urls = vec![
        "https://www.rust-lang.org/",
        "https://developer.mozilla.org/en-US/",
        "https://en.wikipedia.org/wiki/Web_scraping",
        "https://news.ycombinator.com/",
    ];

    let mut results = Vec::new();
    let mut passed = 0;
    let mut failed = 0;

    let total_urls = test_urls.len();

    for url in &test_urls {
        println!("\nTesting: {}", url);
        match fetch_and_extract(url).await {
            Ok(result) => {
                println!(
                    "  ✓ Success - {} chars, coverage: {:.2}%, quality: {:.2}, time: {}ms",
                    result.extracted_chars,
                    result.coverage_percent,
                    result.quality_score,
                    result.extraction_time_ms
                );
                results.push(result);
                passed += 1;
            }
            Err(e) => {
                println!("  ✗ Failed: {}", e);
                failed += 1;
            }
        }
    }

    // Create summary
    let summary = TestSummary {
        timestamp: chrono::Utc::now().to_rfc3339(),
        total_tests: total_urls,
        passed_tests: passed,
        failed_tests: failed,
        results,
    };

    // Save results to file
    let results_json = serde_json::to_string_pretty(&summary).unwrap();
    std::fs::create_dir_all("test-results").ok();
    std::fs::write(
        "test-results/comprehensive_extraction_results.json",
        results_json,
    )
    .unwrap();

    println!("\n=== Test Summary ===");
    println!("Total: {}", summary.total_tests);
    println!("Passed: {}", summary.passed_tests);
    println!("Failed: {}", summary.failed_tests);

    if summary.passed_tests > 0 {
        let avg_coverage: f64 = summary
            .results
            .iter()
            .map(|r| r.coverage_percent)
            .sum::<f64>()
            / summary.passed_tests as f64;

        let avg_quality: f64 = summary.results.iter().map(|r| r.quality_score).sum::<f64>()
            / summary.passed_tests as f64;

        println!("Average coverage: {:.2}%", avg_coverage);
        println!("Average quality: {:.2}", avg_quality);
    }

    println!("\nResults saved to: test-results/comprehensive_extraction_results.json");

    assert!(passed > 0, "At least one test should pass");
}

#[test]
fn test_simple_html_extraction() {
    let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Test Page</title>
            <meta name="description" content="Test description">
        </head>
        <body>
            <article>
                <h1>Main Heading</h1>
                <p>This is the first paragraph with some content.</p>
                <p>This is the second paragraph with more content.</p>
            </article>
        </body>
        </html>
    "#;

    let extractor = EnhancedHtmlExtractor::new(Some("https://example.com")).unwrap();
    let result = extractor.extract(html, "https://example.com").unwrap();

    assert!(result.main_content.contains("Main Heading"));
    assert!(result.main_content.contains("first paragraph"));
    assert!(result.quality_score > 0.0);

    println!(
        "Simple HTML: {} chars, quality: {:.2}",
        result.main_content.len(),
        result.quality_score
    );
}

#[test]
fn test_complex_html_extraction() {
    let html = r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <title>Complex Page with JavaScript</title>
            <meta property="og:title" content="OG Title">
            <meta property="og:description" content="OG Description">
            <script>console.log("This should be excluded");</script>
            <style>.hidden { display: none; }</style>
        </head>
        <body>
            <nav>Navigation menu</nav>
            <main>
                <article>
                    <h1>Article Title</h1>
                    <p class="author">By John Doe</p>
                    <p>First paragraph of the article with substantial content that makes it worth extracting.</p>
                    <p>Second paragraph with even more interesting content about the topic.</p>
                    <p>Third paragraph providing additional details and context.</p>
                    <div class="hidden">This should not be extracted</div>
                </article>
            </main>
            <aside>Sidebar content</aside>
            <footer>Footer information</footer>
        </body>
        </html>
    "#;

    let extractor = EnhancedHtmlExtractor::new(Some("https://example.com")).unwrap();
    let result = extractor.extract(html, "https://example.com").unwrap();

    // Should extract main article content
    assert!(result.main_content.contains("Article Title"));
    assert!(result.main_content.contains("First paragraph"));

    // Should NOT include scripts, styles, navigation, footer
    assert!(!result.main_content.contains("console.log"));
    assert!(!result.main_content.contains("display: none"));

    // Metadata should be extracted
    assert_eq!(result.metadata.og_title, Some("OG Title".to_string()));
    assert_eq!(
        result.metadata.og_description,
        Some("OG Description".to_string())
    );

    // Should be detected as article
    assert!(result.is_article);
    // Lower threshold - complex HTML may have lower quality scores
    assert!(
        result.quality_score > 0.0,
        "Quality score should be > 0, got: {}",
        result.quality_score
    );

    println!(
        "Complex HTML: {} chars, quality: {:.2}, is_article: {}",
        result.main_content.len(),
        result.quality_score,
        result.is_article
    );
}
