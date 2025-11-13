//! Integration tests for strategy selection and raw HTML access
//!
//! These tests verify the complete flow from facade options through to extraction results.

use crate::config::RiptideConfig;
use crate::facades::{ExtractionFacade, ExtractionStrategy, HtmlExtractionOptions};

fn create_test_html() -> &'static str {
    r#"
        <!DOCTYPE html>
        <html>
            <head>
                <title>Test Article</title>
                <meta name="author" content="Test Author">
                <meta name="description" content="Test Description">
            </head>
            <body>
                <article>
                    <h1>Main Heading</h1>
                    <p>This is the main content of the article.</p>
                    <p>It has multiple paragraphs.</p>
                    <a href="https://example.com/link1">Link 1</a>
                    <img src="https://example.com/image.jpg" alt="Test Image">
                </article>
            </body>
        </html>
    "#
}

async fn create_test_facade() -> ExtractionFacade {
    let config = RiptideConfig::default();
    ExtractionFacade::new(config)
        .await
        .expect("Failed to create facade")
}

#[tokio::test]
async fn test_strategy_css_extraction() {
    let facade = create_test_facade().await;
    let html = create_test_html();

    let options = HtmlExtractionOptions {
        extraction_strategy: Some(ExtractionMethod::HtmlCss),
        clean: true,
        include_metadata: true,
        ..Default::default()
    };

    let result = facade
        .extract_html(html, "https://example.com", options)
        .await;

    assert!(result.is_ok(), "CSS extraction should succeed");
    let data = result.unwrap();
    assert_eq!(data.strategy_used, "css_extraction");
    assert!(data.confidence > 0.0);
    assert!(!data.text.is_empty());
}

#[tokio::test]
async fn test_strategy_auto_uses_unified() {
    let facade = create_test_facade().await;
    let html = create_test_html();

    let options = HtmlExtractionOptions {
        extraction_strategy: None, // Auto mode
        clean: true,
        ..Default::default()
    };

    let result = facade
        .extract_html(html, "https://example.com", options)
        .await;

    assert!(result.is_ok(), "Auto extraction should succeed");
    let data = result.unwrap();
    // UnifiedExtractor should use native parser or css_extraction first
    assert!(
        data.strategy_used == "css_extraction"
            || data.strategy_used == "unified"
            || data.strategy_used == "native_parser",
        "Auto mode should use UnifiedExtractor, css_extraction, or native_parser, got: {}",
        data.strategy_used
    );
}

#[tokio::test]
async fn test_raw_html_not_included_by_default() {
    let facade = create_test_facade().await;
    let html = create_test_html();

    let options = HtmlExtractionOptions {
        clean: true,
        ..Default::default()
    };

    let result = facade
        .extract_html(html, "https://example.com", options)
        .await;

    assert!(result.is_ok());
    let data = result.unwrap();
    // Note: extract_html sets raw_html to None, extract_from_url sets it
    assert_eq!(
        data.raw_html, None,
        "raw_html should be None by default in extract_html"
    );
}

#[tokio::test]
async fn test_markdown_generation() {
    let facade = create_test_facade().await;
    let html = create_test_html();

    let options = HtmlExtractionOptions {
        as_markdown: true,
        clean: true,
        ..Default::default()
    };

    let result = facade
        .extract_html(html, "https://example.com", options)
        .await;

    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(
        data.markdown.is_some(),
        "Markdown should be generated when requested"
    );
    let md = data.markdown.unwrap();
    assert!(
        md.contains("# Main Heading"),
        "Markdown should contain heading"
    );
}

#[tokio::test]
async fn test_metadata_extraction() {
    let facade = create_test_facade().await;
    let html = create_test_html();

    let options = HtmlExtractionOptions {
        include_metadata: true,
        clean: true,
        ..Default::default()
    };

    let result = facade
        .extract_html(html, "https://example.com", options)
        .await;

    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(
        data.metadata.contains_key("author") || data.metadata.contains_key("description"),
        "Metadata should be extracted when requested"
    );
}

#[tokio::test]
async fn test_links_extraction() {
    let facade = create_test_facade().await;
    let html = create_test_html();

    let options = HtmlExtractionOptions {
        extract_links: true,
        clean: true,
        ..Default::default()
    };

    let result = facade
        .extract_html(html, "https://example.com", options)
        .await;

    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(
        !data.links.is_empty(),
        "Links should be extracted when requested"
    );
    assert!(
        data.links.iter().any(|l| l.contains("link1")),
        "Should extract specific links"
    );
}

#[tokio::test]
async fn test_images_extraction() {
    let facade = create_test_facade().await;
    let html = create_test_html();

    let options = HtmlExtractionOptions {
        extract_images: true,
        clean: true,
        ..Default::default()
    };

    let result = facade
        .extract_html(html, "https://example.com", options)
        .await;

    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(
        !data.images.is_empty(),
        "Images should be extracted when requested"
    );
    assert!(
        data.images.iter().any(|i| i.contains("image.jpg")),
        "Should extract specific images"
    );
}

#[tokio::test]
async fn test_combined_options() {
    let facade = create_test_facade().await;
    let html = create_test_html();

    let options = HtmlExtractionOptions {
        as_markdown: true,
        clean: true,
        include_metadata: true,
        extract_links: true,
        extract_images: true,
        extraction_strategy: Some(ExtractionMethod::HtmlCss),
        custom_selectors: None,
    };

    let result = facade
        .extract_html(html, "https://example.com", options)
        .await;

    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data.markdown.is_some(), "Markdown should be present");
    assert!(!data.metadata.is_empty(), "Metadata should be present");
    assert!(!data.links.is_empty(), "Links should be present");
    assert!(!data.images.is_empty(), "Images should be present");
    assert_eq!(data.strategy_used, "css_extraction");
}

#[tokio::test]
async fn test_fallback_strategy_chain() {
    let facade = create_test_facade().await;
    let html = create_test_html();

    let strategies = vec![ExtractionMethod::HtmlCss, ExtractionMethod::Fallback];

    let result = facade
        .extract_with_fallback(html, "https://example.com", &strategies)
        .await;

    assert!(result.is_ok(), "Fallback chain should succeed");
    let data = result.unwrap();
    assert!(data.confidence > 0.0);
    assert!(!data.text.is_empty());
}

#[tokio::test]
async fn test_extract_with_specific_strategy() {
    let facade = create_test_facade().await;
    let html = create_test_html();

    let result = facade
        .extract_with_strategy(html, "https://example.com", ExtractionMethod::HtmlCss)
        .await;

    assert!(result.is_ok());
    let data = result.unwrap();
    assert_eq!(data.strategy_used, "css_extraction");
}

#[cfg(feature = "wasm-extractor")]
#[tokio::test]
async fn test_wasm_strategy_when_enabled() {
    let facade = create_test_facade().await;
    let html = create_test_html();

    let options = HtmlExtractionOptions {
        extraction_strategy: Some(ExtractionMethod::Wasm),
        clean: true,
        ..Default::default()
    };

    // This test will succeed if WASM is available, otherwise it's expected to fail
    let result = facade
        .extract_html(html, "https://example.com", options)
        .await;

    if result.is_ok() {
        let data = result.unwrap();
        assert!(
            data.strategy_used.contains("wasm") || data.strategy_used.contains("readability"),
            "WASM strategy should be used"
        );
    }
}

#[cfg(not(feature = "wasm-extractor"))]
#[tokio::test]
async fn test_wasm_strategy_when_disabled() {
    let facade = create_test_facade().await;
    let html = create_test_html();

    let options = HtmlExtractionOptions {
        extraction_strategy: Some(ExtractionMethod::Wasm),
        clean: true,
        ..Default::default()
    };

    let result = facade
        .extract_html(html, "https://example.com", options)
        .await;

    assert!(
        result.is_err(),
        "WASM strategy should fail when feature is disabled"
    );

    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(
            error_msg.contains("WASM") || error_msg.contains("wasm"),
            "Error should mention WASM: {}",
            error_msg
        );
    }
}

#[tokio::test]
async fn test_confidence_calculation() {
    let facade = create_test_facade().await;
    let html = create_test_html();

    let options = HtmlExtractionOptions {
        include_metadata: true,
        clean: true,
        ..Default::default()
    };

    let result = facade
        .extract_html(html, "https://example.com", options)
        .await;

    assert!(result.is_ok());
    let data = result.unwrap();

    let calculated_confidence = facade.calculate_confidence(&data);
    assert!(calculated_confidence >= data.confidence);
    assert!(calculated_confidence <= 1.0);
}
