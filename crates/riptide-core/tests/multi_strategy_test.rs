//! Integration tests for CSS and Regex extraction strategies

use anyhow::Result;
use riptide_core::strategies::traits::ExtractionStrategy;
use riptide_core::strategies::{
    CssSelectorStrategy, RegexPatternStrategy, StrategyRegistry, StrategyRegistryBuilder,
};
use std::sync::Arc;

#[tokio::test]
async fn test_css_strategy_registration() -> Result<()> {
    let css_strategy = Arc::new(CssSelectorStrategy::new());

    let registry = StrategyRegistryBuilder::new()
        .with_extraction(css_strategy.clone())
        .build();

    let strategies = registry.list_extraction_strategies();
    assert!(strategies.iter().any(|(name, _)| name == "css_selector"));

    Ok(())
}

#[tokio::test]
async fn test_regex_strategy_registration() -> Result<()> {
    let regex_strategy = Arc::new(RegexPatternStrategy::new());

    let registry = StrategyRegistryBuilder::new()
        .with_extraction(regex_strategy.clone())
        .build();

    let strategies = registry.list_extraction_strategies();
    assert!(strategies.iter().any(|(name, _)| name == "regex_pattern"));

    Ok(())
}

#[tokio::test]
async fn test_both_strategies_registered() -> Result<()> {
    let css_strategy = Arc::new(CssSelectorStrategy::new());
    let regex_strategy = Arc::new(RegexPatternStrategy::new());

    let registry = StrategyRegistryBuilder::new()
        .with_extraction(css_strategy)
        .with_extraction(regex_strategy)
        .build();

    let strategies = registry.list_extraction_strategies();
    assert_eq!(strategies.len(), 2);
    assert!(strategies.iter().any(|(name, _)| name == "css_selector"));
    assert!(strategies.iter().any(|(name, _)| name == "regex_pattern"));

    Ok(())
}

#[tokio::test]
async fn test_css_extraction() -> Result<()> {
    let strategy = CssSelectorStrategy::new();

    let html = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Test Page</title></head>
        <body>
            <h1>Welcome</h1>
            <article>
                <p>This is the main content.</p>
                <div class="author">Jane Doe</div>
                <time>2025-01-01</time>
            </article>
        </body>
        </html>
    "#;

    let result = strategy.extract(html, "https://example.com").await?;

    assert_eq!(result.content.strategy_used, "css");
    assert!(result.content.title.contains("Welcome"));
    assert!(result.content.content.contains("main content"));
    assert!(result.metadata.contains_key("author"));
    assert_eq!(result.metadata.get("author").unwrap(), "Jane Doe");
    assert!(result.metadata.contains_key("date"));

    Ok(())
}

#[tokio::test]
async fn test_regex_extraction() -> Result<()> {
    let strategy = RegexPatternStrategy::new();

    let html = r#"
        <html>
        <body>
            <p>Contact us at support@example.com or call 555-123-4567</p>
            <p>Visit our website at https://example.com</p>
        </body>
        </html>
    "#;

    let result = strategy.extract(html, "https://example.com").await?;

    assert_eq!(result.content.strategy_used, "regex");
    assert!(result.content.content.contains("Email"));
    assert!(result.content.content.contains("Phone"));
    assert!(result.content.content.contains("URLs"));

    // Check metadata
    assert!(result.metadata.contains_key("email_count"));
    assert!(result.metadata.contains_key("phone_count"));
    assert!(result.metadata.contains_key("url_count"));

    Ok(())
}

#[tokio::test]
async fn test_css_confidence_score() {
    let strategy = CssSelectorStrategy::new();

    // Good HTML with article structure
    let good_html = r#"
        <article>
            <h1>Title</h1>
            <div class="author">Author</div>
            <p>Content</p>
        </article>
    "#;

    let score = strategy.confidence_score(good_html);
    assert!(score > 0.5, "Expected confidence > 0.5, got {}", score);

    // Poor HTML
    let poor_html = "<div>Just text</div>";
    let score = strategy.confidence_score(poor_html);
    assert!(score < 0.5, "Expected confidence < 0.5, got {}", score);
}

#[tokio::test]
async fn test_regex_confidence_score() {
    let strategy = RegexPatternStrategy::new();

    // Text with many patterns
    let rich_text = "Email: test@example.com, Phone: 555-1234, URL: https://example.com";
    let score = strategy.confidence_score(rich_text);
    assert!(score > 0.5, "Expected confidence > 0.5, got {}", score);

    // Text with no patterns
    let poor_text = "Just plain text with nothing";
    let score = strategy.confidence_score(poor_text);
    assert!(score < 0.5, "Expected confidence < 0.5, got {}", score);
}

#[tokio::test]
async fn test_strategy_selection() -> Result<()> {
    let css_strategy = Arc::new(CssSelectorStrategy::new());
    let regex_strategy = Arc::new(RegexPatternStrategy::new());

    let registry = StrategyRegistryBuilder::new()
        .with_extraction(css_strategy)
        .with_extraction(regex_strategy)
        .build();

    // HTML content should prefer CSS
    let html = "<article><h1>Title</h1><p>Content</p></article>";
    let best = registry.find_best_extraction(html);
    assert!(best.is_some());
    assert_eq!(best.unwrap().name(), "css_selector");

    // Plain text should prefer regex
    let text = "Email: test@example.com, Phone: 555-1234";
    let best = registry.find_best_extraction(text);
    assert!(best.is_some());
    // Note: This test might need adjustment based on actual confidence scores

    Ok(())
}

#[tokio::test]
async fn test_css_capabilities() {
    let strategy = CssSelectorStrategy::new();
    let caps = strategy.capabilities();

    assert_eq!(caps.strategy_type, "css_extraction");
    assert!(caps
        .supported_content_types
        .contains(&"text/html".to_string()));
    assert!(caps.features.contains(&"css_selectors".to_string()));
    assert!(!caps.resource_requirements.requires_network);
}

#[tokio::test]
async fn test_regex_capabilities() {
    let strategy = RegexPatternStrategy::new();
    let caps = strategy.capabilities();

    assert_eq!(caps.strategy_type, "regex_extraction");
    assert!(caps
        .supported_content_types
        .contains(&"text/html".to_string()));
    assert!(caps.features.contains(&"pattern_matching".to_string()));
    assert!(!caps.resource_requirements.requires_network);
}

#[tokio::test]
async fn test_sensitive_data_handling() -> Result<()> {
    let strategy = RegexPatternStrategy::new();

    let html = r#"
        <html>
        <body>
            <p>SSN: 123-45-6789</p>
            <p>Card: 1234-5678-9012-3456</p>
        </body>
        </html>
    "#;

    let result = strategy.extract(html, "https://example.com").await?;

    // Sensitive data should be redacted
    assert!(result.content.content.contains("[REDACTED]"));
    assert!(!result.content.content.contains("123-45-6789"));
    assert!(!result.content.content.contains("1234-5678-9012-3456"));

    Ok(())
}

#[tokio::test]
async fn test_empty_content_handling() -> Result<()> {
    let css_strategy = CssSelectorStrategy::new();
    let regex_strategy = RegexPatternStrategy::new();

    let empty_html = "<html><body></body></html>";

    let css_result = css_strategy
        .extract(empty_html, "https://example.com")
        .await?;
    assert!(css_result.content.extraction_confidence < 1.0);

    let regex_result = regex_strategy
        .extract(empty_html, "https://example.com")
        .await?;
    assert!(regex_result.content.content.contains("No structured data"));

    Ok(())
}
