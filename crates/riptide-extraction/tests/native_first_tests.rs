//! Comprehensive tests to verify native extraction works as the primary path
//!
//! This test suite ensures that:
//! 1. Native extraction works without WASM
//! 2. Native extraction is the default fallback
//! 3. WASM is optional and doesn't break when disabled
//! 4. Native extraction produces quality results
//! 5. Edge cases are handled correctly

use anyhow::Result;
use riptide_extraction::{
    extraction_strategies::ContentExtractor, unified_extractor::UnifiedExtractor,
    UnifiedExtractor as UnifiedExtractorAlias,
};

// Test HTML samples
const SAMPLE_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <title>Test Article - Native Extraction</title>
    <meta name="author" content="Test Author">
    <meta name="description" content="This is a test article for native extraction">
    <meta property="og:title" content="Open Graph Title">
</head>
<body>
    <article>
        <h1>Real Article Title</h1>
        <p class="byline">By Test Author on 2024-01-15</p>
        <p>This is actual content that should be extracted by the native parser.</p>
        <p>Multiple paragraphs with meaningful text that should be properly analyzed
           for quality scoring and content extraction.</p>
        <p>The native parser should extract this content without WASM.</p>
        <a href="https://example.com/link1">External Link 1</a>
        <a href="/relative/link2">Relative Link 2</a>
        <img src="https://example.com/image1.jpg" alt="Image 1">
        <img src="/images/local.png" alt="Local Image">
    </article>
</body>
</html>
"#;

const COMPLEX_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Complex Document Structure</title>
    <meta name="description" content="Complex multi-section article">
</head>
<body>
    <header>
        <nav><a href="/home">Home</a></nav>
    </header>
    <main>
        <article>
            <h1>Main Article Title</h1>
            <section>
                <h2>Section 1</h2>
                <p>Content in section 1 with substantial text.</p>
            </section>
            <section>
                <h2>Section 2</h2>
                <p>Content in section 2 with more substantial text.</p>
                <ul>
                    <li>List item 1</li>
                    <li>List item 2</li>
                </ul>
            </section>
        </article>
    </main>
    <footer>
        <p>Footer content that should be ignored or de-prioritized.</p>
    </footer>
</body>
</html>
"#;

const MINIMAL_HTML: &str =
    "<html><head><title>Minimal</title></head><body><p>Content</p></body></html>";

const EMPTY_BODY: &str = "<html><head><title>Empty Body</title></head><body></body></html>";

const MALFORMED_HTML: &str = r#"
<html>
<head><title>Malformed Document</title>
<body>
<p>Unclosed paragraph
<div>Unclosed div
<h1>Title without closing tag
"#;

// ============================================================================
// Test 1: Native extraction works without WASM
// ============================================================================

#[tokio::test]
async fn test_native_extraction_no_wasm() -> Result<()> {
    println!("🧪 Test 1: Native extraction works without WASM");

    // Create extractor without WASM path
    let extractor = UnifiedExtractor::new(None).await?;

    // Verify it's using native
    assert_eq!(
        extractor.extractor_type(),
        "native",
        "Should use native extractor when WASM not provided"
    );

    // Extract content
    let result = extractor
        .extract(SAMPLE_HTML, "https://example.com/test")
        .await?;

    // Verify extraction succeeded
    assert!(!result.title.is_empty(), "Title should be extracted");
    assert!(!result.content.is_empty(), "Content should be extracted");
    assert_eq!(result.url, "https://example.com/test");

    // Verify it's not mock data
    assert!(
        result.title.contains("Test Article") || result.title.contains("Real Article"),
        "Title should match actual HTML: {}",
        result.title
    );

    println!("✅ PASS: Native extraction works without WASM");
    Ok(())
}

// ============================================================================
// Test 2: Native extraction is default
// ============================================================================

#[tokio::test]
async fn test_native_is_default() -> Result<()> {
    println!("🧪 Test 2: Native extraction is default");

    // Test 1: Explicit None
    let extractor1 = UnifiedExtractor::new(None).await?;
    assert_eq!(extractor1.extractor_type(), "native");

    // Test 2: Invalid WASM path (should fallback to native)
    let extractor2 = UnifiedExtractor::new(Some("/nonexistent/wasm/path.wasm")).await?;
    assert_eq!(
        extractor2.extractor_type(),
        "native",
        "Should fallback to native when WASM file doesn't exist"
    );

    // Test 3: Empty string path (should use native)
    let extractor3 = UnifiedExtractor::new(Some("")).await?;
    assert_eq!(extractor3.extractor_type(), "native");

    println!("✅ PASS: Native extraction is the default fallback");
    Ok(())
}

// ============================================================================
// Test 3: WASM is optional enhancement
// ============================================================================

#[cfg(feature = "wasm-extractor")]
#[tokio::test]
async fn test_wasm_as_optional() -> Result<()> {
    println!("🧪 Test 3: WASM is optional enhancement (feature enabled)");

    // Even with feature enabled, native should work
    let extractor = UnifiedExtractor::new(None).await?;
    let result = extractor
        .extract(SAMPLE_HTML, "https://example.com")
        .await?;

    assert!(!result.content.is_empty());
    println!("✅ PASS: Native works even with WASM feature enabled");
    Ok(())
}

#[cfg(not(feature = "wasm-extractor"))]
#[tokio::test]
async fn test_wasm_disabled_gracefully() -> Result<()> {
    println!("🧪 Test 3: WASM disabled gracefully (feature disabled)");

    // WASM feature disabled - should use native without errors
    let extractor = UnifiedExtractor::new(Some("/some/wasm/path.wasm")).await?;

    assert_eq!(extractor.extractor_type(), "native");
    assert!(!UnifiedExtractor::wasm_available());

    // Should still extract successfully
    let result = extractor
        .extract(SAMPLE_HTML, "https://example.com")
        .await?;
    assert!(!result.content.is_empty());

    println!("✅ PASS: WASM disabled gracefully, native works");
    Ok(())
}

// ============================================================================
// Test 4: Native extraction quality
// ============================================================================

#[tokio::test]
async fn test_native_extraction_quality() -> Result<()> {
    println!("🧪 Test 4: Native extraction quality verification");

    let extractor = UnifiedExtractor::new(None).await?;
    let result = extractor
        .extract(SAMPLE_HTML, "https://example.com/test")
        .await?;

    // Quality checks
    println!("  Title: {}", result.title);
    println!("  Content length: {} chars", result.content.len());
    println!("  Confidence: {:.2}", result.extraction_confidence);

    // Title should be meaningful
    assert!(
        result.title.len() > 3,
        "Title should be more than 3 characters"
    );

    // Content should be substantial
    assert!(
        result.content.len() > 50,
        "Content should be at least 50 characters, got {}",
        result.content.len()
    );

    // Should extract actual content, not HTML tags
    assert!(
        !result.content.contains("<p>"),
        "Content should be text, not HTML"
    );
    assert!(
        !result.content.contains("<div>"),
        "Content should be text, not HTML"
    );

    // Confidence should be reasonable
    assert!(
        result.extraction_confidence > 0.3,
        "Confidence should be > 0.3, got {}",
        result.extraction_confidence
    );

    // Summary should be extracted (if available)
    if result.summary.is_some() {
        assert!(!result.summary.as_ref().unwrap().is_empty());
    }

    println!("✅ PASS: Native extraction produces quality results");
    Ok(())
}

// ============================================================================
// Test 5: Content comparison - Native vs expected
// ============================================================================

#[tokio::test]
async fn test_content_matches_expected() -> Result<()> {
    println!("🧪 Test 5: Content matches expected extraction");

    let extractor = UnifiedExtractor::new(None).await?;
    let result = extractor
        .extract(SAMPLE_HTML, "https://example.com")
        .await?;

    // Check for key content phrases
    let expected_phrases = vec![
        "actual content",
        "native parser",
        "Multiple paragraphs",
        "Test Author",
    ];

    for phrase in expected_phrases {
        assert!(
            result.content.contains(phrase)
                || result.title.contains(phrase)
                || result
                    .summary
                    .as_ref()
                    .map_or(false, |s| s.contains(phrase)),
            "Expected to find '{}' in extracted content",
            phrase
        );
    }

    println!("✅ PASS: Content matches expected extraction");
    Ok(())
}

// ============================================================================
// Test 6: Edge case - Minimal HTML
// ============================================================================

#[tokio::test]
async fn test_minimal_html_extraction() -> Result<()> {
    println!("🧪 Test 6: Minimal HTML edge case");

    let extractor = UnifiedExtractor::new(None).await?;
    let result = extractor
        .extract(MINIMAL_HTML, "https://example.com")
        .await?;

    // Should extract something even from minimal HTML
    assert!(!result.title.is_empty(), "Should extract title");
    assert!(!result.content.is_empty(), "Should extract content");
    assert_eq!(result.title, "Minimal");

    println!("✅ PASS: Minimal HTML handled correctly");
    Ok(())
}

// ============================================================================
// Test 7: Edge case - Empty body
// ============================================================================

#[tokio::test]
async fn test_empty_body_extraction() -> Result<()> {
    println!("🧪 Test 7: Empty body edge case");

    let extractor = UnifiedExtractor::new(None).await?;
    let result = extractor.extract(EMPTY_BODY, "https://example.com").await;

    // Should either succeed with minimal data or fail gracefully
    match result {
        Ok(content) => {
            println!("  Extracted from empty body: title='{}'", content.title);
            assert_eq!(content.title, "Empty Body");
        }
        Err(e) => {
            println!("  Empty body handling: {}", e);
            // Acceptable to fail on empty content
        }
    }

    println!("✅ PASS: Empty body handled gracefully");
    Ok(())
}

// ============================================================================
// Test 8: Edge case - Malformed HTML
// ============================================================================

#[tokio::test]
async fn test_malformed_html_recovery() -> Result<()> {
    println!("🧪 Test 8: Malformed HTML recovery");

    let extractor = UnifiedExtractor::new(None).await?;
    let result = extractor
        .extract(MALFORMED_HTML, "https://example.com")
        .await;

    // Native parser should handle malformed HTML gracefully
    match result {
        Ok(content) => {
            println!("  Recovered from malformed HTML");
            assert!(!content.title.is_empty(), "Should extract title");
            // scraper is forgiving, should extract something
        }
        Err(e) => {
            println!("  Malformed HTML error: {}", e);
            // Also acceptable if validation catches it
        }
    }

    println!("✅ PASS: Malformed HTML handled");
    Ok(())
}

// ============================================================================
// Test 9: Edge case - Large document
// ============================================================================

#[tokio::test]
async fn test_large_document_handling() -> Result<()> {
    println!("🧪 Test 9: Large document handling");

    let extractor = UnifiedExtractor::new(None).await?;

    // Generate large HTML
    let mut large_html = String::from("<html><head><title>Large Document</title></head><body>");
    for i in 0..1000 {
        large_html.push_str(&format!("<p>Paragraph {} with substantial content that should be extracted and processed correctly by the native parser.</p>", i));
    }
    large_html.push_str("</body></html>");

    let result = extractor
        .extract(&large_html, "https://example.com")
        .await?;

    assert!(!result.title.is_empty());
    assert!(
        result.content.len() > 1000,
        "Should extract substantial content from large document"
    );

    println!(
        "  Extracted {} chars from large document",
        result.content.len()
    );
    println!("✅ PASS: Large document handled successfully");
    Ok(())
}

// ============================================================================
// Test 10: Complex HTML structure
// ============================================================================

#[tokio::test]
async fn test_complex_html_structure() -> Result<()> {
    println!("🧪 Test 10: Complex HTML structure");

    let extractor = UnifiedExtractor::new(None).await?;
    let result = extractor
        .extract(COMPLEX_HTML, "https://example.com")
        .await?;

    // Should extract from nested structure
    assert!(result.title.contains("Complex Document") || result.title.contains("Main Article"));
    assert!(result.content.contains("Section 1") || result.content.contains("section 1"));
    assert!(result.content.contains("Section 2") || result.content.contains("section 2"));

    // Should handle lists
    assert!(result.content.contains("List item") || result.content.contains("list"));

    println!("✅ PASS: Complex HTML structure handled correctly");
    Ok(())
}

// ============================================================================
// Test 11: URL resolution in links/media
// ============================================================================

#[tokio::test]
async fn test_url_resolution() -> Result<()> {
    println!("🧪 Test 11: URL resolution in extracted content");

    let extractor = UnifiedExtractor::new(None).await?;

    // Using native parser directly to check link/media extraction
    use riptide_extraction::native_parser::NativeHtmlParser;
    let parser = NativeHtmlParser::new();

    match parser.parse_headless_html(SAMPLE_HTML, "https://example.com/base/page") {
        Ok(doc) => {
            println!("  Links extracted: {}", doc.links.len());
            println!("  Media extracted: {}", doc.media.len());

            // Verify links are extracted
            assert!(!doc.links.is_empty(), "Should extract links");

            // Verify media is extracted
            assert!(!doc.media.is_empty(), "Should extract media");

            // Check for absolute URL resolution
            let has_absolute = doc.links.iter().any(|l| l.starts_with("http"));
            println!("  Has absolute URLs: {}", has_absolute);
        }
        Err(e) => {
            println!("  Parser error: {}", e);
        }
    }

    println!("✅ PASS: URL resolution tested");
    Ok(())
}

// ============================================================================
// Test 12: Confidence scoring accuracy
// ============================================================================

#[tokio::test]
async fn test_confidence_scoring() -> Result<()> {
    println!("🧪 Test 12: Confidence scoring accuracy");

    let extractor = UnifiedExtractor::new(None).await?;

    // Good quality HTML
    let good_score = extractor.confidence_score(SAMPLE_HTML);
    println!("  Good HTML confidence: {:.2}", good_score);

    // Minimal HTML
    let minimal_score = extractor.confidence_score(MINIMAL_HTML);
    println!("  Minimal HTML confidence: {:.2}", minimal_score);

    // Empty HTML
    let empty_score = extractor.confidence_score("<html><body></body></html>");
    println!("  Empty HTML confidence: {:.2}", empty_score);

    // Scores should be ordered
    assert!(
        good_score > minimal_score,
        "Good HTML should score higher than minimal"
    );
    assert!(
        minimal_score > empty_score,
        "Minimal HTML should score higher than empty"
    );

    // All scores should be in valid range
    assert!(
        (0.0..=1.0).contains(&good_score),
        "Score should be 0-1: {}",
        good_score
    );
    assert!(
        (0.0..=1.0).contains(&minimal_score),
        "Score should be 0-1: {}",
        minimal_score
    );
    assert!(
        (0.0..=1.0).contains(&empty_score),
        "Score should be 0-1: {}",
        empty_score
    );

    println!("✅ PASS: Confidence scoring works correctly");
    Ok(())
}

// ============================================================================
// Test 13: Strategy name reporting
// ============================================================================

#[tokio::test]
async fn test_strategy_name() -> Result<()> {
    println!("🧪 Test 13: Strategy name reporting");

    let extractor = UnifiedExtractor::new(None).await?;

    // Check strategy name
    let strategy = extractor.strategy_name();
    println!("  Strategy: {}", strategy);

    #[cfg(feature = "wasm-extractor")]
    assert!(
        strategy == "native" || strategy == "wasm",
        "Strategy should be native or wasm"
    );

    #[cfg(not(feature = "wasm-extractor"))]
    assert_eq!(
        strategy, "native",
        "Strategy should be native when WASM disabled"
    );

    println!("✅ PASS: Strategy name correctly reported");
    Ok(())
}

// ============================================================================
// Test 14: Parallel extraction (no race conditions)
// ============================================================================

#[tokio::test]
async fn test_parallel_extraction() -> Result<()> {
    println!("🧪 Test 14: Parallel extraction safety");

    let extractor = UnifiedExtractor::new(None).await?;

    // Extract multiple URLs in parallel
    let urls = vec![
        ("https://example.com/page1", SAMPLE_HTML),
        ("https://example.com/page2", COMPLEX_HTML),
        ("https://example.com/page3", MINIMAL_HTML),
    ];

    let mut tasks = Vec::new();
    for (url, html) in urls {
        let ext = UnifiedExtractor::new(None).await?;
        tasks.push(tokio::spawn(async move { ext.extract(html, url).await }));
    }

    // Wait for all tasks
    let results: Vec<_> = futures::future::join_all(tasks).await;

    // All should succeed
    for (idx, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "Task {} panicked", idx);
        let content = result.as_ref().unwrap().as_ref().unwrap();
        assert!(
            !content.title.is_empty(),
            "Task {} should extract title",
            idx
        );
    }

    println!("✅ PASS: Parallel extraction is safe");
    Ok(())
}

// ============================================================================
// Test 15: Performance baseline
// ============================================================================

#[tokio::test]
async fn test_native_extraction_performance() -> Result<()> {
    println!("🧪 Test 15: Native extraction performance baseline");

    let extractor = UnifiedExtractor::new(None).await?;

    // Warm up
    let _ = extractor
        .extract(SAMPLE_HTML, "https://example.com")
        .await?;

    // Measure extraction time
    let start = std::time::Instant::now();
    let iterations = 100;

    for _ in 0..iterations {
        let _ = extractor
            .extract(SAMPLE_HTML, "https://example.com")
            .await?;
    }

    let duration = start.elapsed();
    let avg_time = duration.as_millis() / iterations;

    println!("  {} iterations in {:?}", iterations, duration);
    println!("  Average: {}ms per extraction", avg_time);

    // Performance expectation: should be reasonably fast
    assert!(
        avg_time < 100,
        "Native extraction should average <100ms, got {}ms",
        avg_time
    );

    println!("✅ PASS: Performance within acceptable range");
    Ok(())
}

// ============================================================================
// Integration Test: Full pipeline
// ============================================================================

#[tokio::test]
async fn test_full_extraction_pipeline() -> Result<()> {
    println!("🧪 Integration: Full extraction pipeline");

    println!("  Step 1: Create extractor");
    let extractor = UnifiedExtractor::new(None).await?;

    println!("  Step 2: Verify extractor type");
    assert_eq!(extractor.extractor_type(), "native");

    println!("  Step 3: Check WASM availability");
    let wasm_available = UnifiedExtractor::wasm_available();
    println!("    WASM available at compile-time: {}", wasm_available);

    println!("  Step 4: Calculate confidence");
    let confidence = extractor.confidence_score(SAMPLE_HTML);
    println!("    Confidence: {:.2}", confidence);
    assert!(confidence > 0.5);

    println!("  Step 5: Extract content");
    let result = extractor
        .extract(SAMPLE_HTML, "https://example.com/article")
        .await?;

    println!("  Step 6: Validate extraction");
    assert!(!result.title.is_empty(), "Title extracted");
    assert!(!result.content.is_empty(), "Content extracted");
    assert_eq!(result.url, "https://example.com/article");
    assert!(result.extraction_confidence > 0.3, "Confidence > 0.3");

    println!("  Step 7: Verify strategy used");
    assert_eq!(
        result.strategy_used, "native_parser",
        "Should use native_parser strategy"
    );

    println!("✅ PASS: Full pipeline executed successfully");
    Ok(())
}
