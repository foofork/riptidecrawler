//! Strategy Selection Integration Tests
//!
//! Tests the strategy selection mechanism across API and facade layers

use riptide_facade::facades::{ExtractionFacade, HtmlExtractionOptions, ExtractionStrategy};
use riptide_config::RiptideConfig;

/// Test that CSS/native strategy is selected correctly
#[tokio::test]
async fn test_strategy_selection_css() {
    let config = RiptideConfig::default();
    let facade = ExtractionFacade::new(config).await.unwrap();

    let html = r#"
        <html>
            <head><title>Test Page</title></head>
            <body>
                <article>
                    <h1>Main Title</h1>
                    <p>This is test content for CSS extraction.</p>
                </article>
            </body>
        </html>
    "#;

    let options = HtmlExtractionOptions {
        clean: true,
        include_metadata: false,
        extraction_strategy: Some(ExtractionStrategy::HtmlCss),
        ..Default::default()
    };

    let result = facade.extract_html(html, "https://example.com", options).await;
    assert!(result.is_ok(), "CSS extraction should succeed");

    let extracted = result.unwrap();
    assert!(!extracted.text.is_empty(), "Extracted text should not be empty");
    assert!(extracted.strategy_used.contains("css") || extracted.strategy_used.contains("native"),
            "Strategy used should be CSS or native, got: {}", extracted.strategy_used);
}

/// Test that auto mode (UnifiedExtractor) works when no strategy specified
#[tokio::test]
async fn test_strategy_selection_auto_mode() {
    let config = RiptideConfig::default();
    let facade = ExtractionFacade::new(config).await.unwrap();

    let html = r#"
        <html>
            <head><title>Auto Mode Test</title></head>
            <body>
                <article>
                    <h1>Testing Auto Selection</h1>
                    <p>This should use UnifiedExtractor by default.</p>
                </article>
            </body>
        </html>
    "#;

    let options = HtmlExtractionOptions {
        clean: true,
        include_metadata: false,
        extraction_strategy: None, // Auto mode
        ..Default::default()
    };

    let result = facade.extract_html(html, "https://example.com", options).await;
    assert!(result.is_ok(), "Auto mode extraction should succeed");

    let extracted = result.unwrap();
    assert!(!extracted.text.is_empty(), "Extracted text should not be empty");
    assert!(!extracted.strategy_used.is_empty(), "Strategy should be reported");
}

/// Test backward compatibility with existing code
#[tokio::test]
async fn test_backward_compatibility_default_options() {
    let config = RiptideConfig::default();
    let facade = ExtractionFacade::new(config).await.unwrap();

    let html = r#"
        <html>
            <body>
                <article>
                    <h1>Backward Compatibility Test</h1>
                    <p>Testing with default options.</p>
                </article>
            </body>
        </html>
    "#;

    // Old code pattern - no extraction_strategy specified
    let options = HtmlExtractionOptions::default();

    let result = facade.extract_html(html, "https://example.com", options).await;
    assert!(result.is_ok(), "Default options should work");

    let extracted = result.unwrap();
    assert!(!extracted.text.is_empty());
}

/// Test that WASM strategy fails gracefully when feature not enabled
#[tokio::test]
async fn test_wasm_strategy_without_feature() {
    let config = RiptideConfig::default();
    let facade = ExtractionFacade::new(config).await.unwrap();

    let html = r#"
        <html>
            <body>
                <article>
                    <h1>WASM Test</h1>
                    <p>Testing WASM strategy.</p>
                </article>
            </body>
        </html>
    "#;

    let options = HtmlExtractionOptions {
        extraction_strategy: Some(ExtractionStrategy::Wasm),
        ..Default::default()
    };

    let result = facade.extract_html(html, "https://example.com", options).await;

    #[cfg(not(feature = "wasm-extractor"))]
    {
        assert!(result.is_err(), "WASM should fail without feature enabled");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("WASM") || err.to_string().contains("not available"),
                "Error should mention WASM unavailability");
    }

    #[cfg(feature = "wasm-extractor")]
    {
        // With feature enabled, it should either succeed or fail for other reasons
        // (like missing WASM file), but not with "not available" message
        if let Err(e) = result {
            assert!(!e.to_string().contains("not available"),
                    "Should not say 'not available' when feature is enabled");
        }
    }
}

/// Test fallback strategy
#[tokio::test]
async fn test_fallback_strategy() {
    let config = RiptideConfig::default();
    let facade = ExtractionFacade::new(config).await.unwrap();

    let html = r#"
        <html>
            <body>
                <article>
                    <h1>Fallback Test</h1>
                    <p>Testing fallback strategy.</p>
                </article>
            </body>
        </html>
    "#;

    let options = HtmlExtractionOptions {
        extraction_strategy: Some(ExtractionStrategy::Fallback),
        ..Default::default()
    };

    let result = facade.extract_html(html, "https://example.com", options).await;
    assert!(result.is_ok(), "Fallback strategy should succeed");

    let extracted = result.unwrap();
    assert!(!extracted.text.is_empty());
}

/// Test raw HTML inclusion with strategy selection
#[tokio::test]
async fn test_raw_html_with_strategy() {
    let config = RiptideConfig::default();
    let facade = ExtractionFacade::new(config).await.unwrap();

    let html = r#"<html><body><h1>Test</h1></body></html>"#;

    let options = HtmlExtractionOptions {
        extraction_strategy: Some(ExtractionStrategy::HtmlCss),
        ..Default::default()
    };

    // extract_html itself doesn't store raw HTML, but extract_from_url does
    let result = facade.extract_html(html, "https://example.com", options).await;
    assert!(result.is_ok());
}
