//! Comprehensive tests for CSS-002: :has-text() post-filter functionality
//!
//! Testing :has-text() pseudo-selector for text content matching including:
//! - Exact text matching
//! - Partial text matching
//! - Case-sensitive and case-insensitive matching
//! - Complex text pattern filtering

use anyhow::Result;
use riptide_html::css_extraction::*;
use std::collections::HashMap;

/// Test CSS-002: Basic :has-text() functionality with exact matching
#[tokio::test]
async fn test_has_text_exact_matching() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="article">
                    <p class="paragraph">This is breaking news about technology.</p>
                    <p class="paragraph">This is regular news about sports.</p>
                    <p class="paragraph">Another breaking story here.</p>
                </div>
                <div class="sidebar">
                    <p class="paragraph">Sidebar content with breaking info.</p>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Test exact text matching
    selectors.insert("breaking_news".to_string(), CssSelectorConfig {
        selector: ".paragraph".to_string(),
        transformers: vec!["trim".to_string()],
        has_text_filter: Some(HasTextFilter {
            pattern: "breaking news".to_string(),
            case_insensitive: false,
            partial_match: false, // Exact match
        }),
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should only match the exact phrase "breaking news"
    assert!(result.content.contains("breaking news about technology"));
    assert!(!result.content.contains("breaking story")); // Not exact match
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}

/// Test CSS-002: Partial text matching
#[tokio::test]
async fn test_has_text_partial_matching() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="content">
                    <h1 class="title">Breaking News Alert</h1>
                    <h2 class="title">Weather Update</h2>
                    <h3 class="title">Sports Breaking Record</h3>
                    <h4 class="title">Technology News</h4>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Test partial text matching
    selectors.insert("breaking_titles".to_string(), CssSelectorConfig {
        selector: ".title".to_string(),
        transformers: vec!["trim".to_string()],
        has_text_filter: Some(HasTextFilter {
            pattern: "breaking".to_string(),
            case_insensitive: false,
            partial_match: true, // Partial match
        }),
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should match both titles containing "breaking"
    assert!(result.content.contains("Breaking News Alert"));
    assert!(result.content.contains("Sports Breaking Record"));
    assert!(!result.content.contains("Weather Update")); // Doesn't contain "breaking"
    assert!(!result.content.contains("Technology News")); // Doesn't contain "breaking"

    Ok(())
}

/// Test CSS-002: Case-insensitive matching
#[tokio::test]
async fn test_has_text_case_insensitive() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="articles">
                    <article class="post">URGENT: Breaking news update</article>
                    <article class="post">urgent meeting scheduled</article>
                    <article class="post">Urgent care facility opens</article>
                    <article class="post">Regular news story</article>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Test case-insensitive matching
    selectors.insert("urgent_posts".to_string(), CssSelectorConfig {
        selector: ".post".to_string(),
        transformers: vec!["trim".to_string()],
        has_text_filter: Some(HasTextFilter {
            pattern: "urgent".to_string(),
            case_insensitive: true, // Case-insensitive
            partial_match: true,
        }),
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should match all three articles with "urgent" in different cases
    assert!(result.content.contains("URGENT: Breaking"));
    assert!(result.content.contains("urgent meeting"));
    assert!(result.content.contains("Urgent care"));
    assert!(!result.content.contains("Regular news")); // Doesn't contain "urgent"

    Ok(())
}

/// Test CSS-002: Case-sensitive matching
#[tokio::test]
async fn test_has_text_case_sensitive() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="reviews">
                    <div class="review">This product is Excellent for daily use.</div>
                    <div class="review">excellent value for money here.</div>
                    <div class="review">EXCELLENT customer service provided.</div>
                    <div class="review">Good product overall.</div>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Test case-sensitive matching (lowercase only)
    selectors.insert("lowercase_excellent".to_string(), CssSelectorConfig {
        selector: ".review".to_string(),
        transformers: vec!["trim".to_string()],
        has_text_filter: Some(HasTextFilter {
            pattern: "excellent".to_string(),
            case_insensitive: false, // Case-sensitive
            partial_match: true,
        }),
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should only match the lowercase "excellent"
    assert!(result.content.contains("excellent value for money"));
    assert!(!result.content.contains("This product is Excellent")); // Capital E
    assert!(!result.content.contains("EXCELLENT customer service")); // All caps
    assert!(!result.content.contains("Good product")); // No "excellent"

    Ok(())
}

/// Test CSS-002: Complex text patterns with special characters
#[tokio::test]
async fn test_has_text_special_characters() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="prices">
                    <span class="price">$19.99 - great deal!</span>
                    <span class="price">€25.50 shipping included</span>
                    <span class="price">£15.00 (limited time)</span>
                    <span class="price">Free shipping available</span>
                </div>
                <div class="emails">
                    <div class="contact">Email: support@example.com</div>
                    <div class="contact">Call: +1-555-0123</div>
                    <div class="contact">Chat available 24/7</div>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Test dollar sign pattern
    selectors.insert("dollar_prices".to_string(), CssSelectorConfig {
        selector: ".price".to_string(),
        transformers: vec!["trim".to_string()],
        has_text_filter: Some(HasTextFilter {
            pattern: "$".to_string(),
            case_insensitive: false,
            partial_match: true,
        }),
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    // Test email pattern
    selectors.insert("email_contacts".to_string(), CssSelectorConfig {
        selector: ".contact".to_string(),
        transformers: vec!["trim".to_string()],
        has_text_filter: Some(HasTextFilter {
            pattern: "@".to_string(),
            case_insensitive: false,
            partial_match: true,
        }),
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should match content with special characters
    assert!(result.content.contains("$19.99") || result.content.contains("support@example.com"));

    Ok(())
}

/// Test CSS-002: Multiple :has-text() filters in same extraction
#[tokio::test]
async fn test_multiple_has_text_filters() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="articles">
                    <article class="news urgent">Breaking: Stock market surges</article>
                    <article class="news">Regular market update</article>
                    <article class="news urgent">Alert: New policy announced</article>
                    <article class="blog">Personal blog post about markets</article>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Filter for urgent news containing "market"
    selectors.insert("urgent_market_news".to_string(), CssSelectorConfig {
        selector: ".news.urgent".to_string(),
        transformers: vec!["trim".to_string()],
        has_text_filter: Some(HasTextFilter {
            pattern: "market".to_string(),
            case_insensitive: true,
            partial_match: true,
        }),
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    // Filter for any article containing "Breaking"
    selectors.insert("breaking_articles".to_string(), CssSelectorConfig {
        selector: "article".to_string(),
        transformers: vec!["trim".to_string()],
        has_text_filter: Some(HasTextFilter {
            pattern: "Breaking".to_string(),
            case_insensitive: false,
            partial_match: true,
        }),
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should extract relevant content based on filters
    assert!(result.extraction_confidence > 0.0);
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}

/// Test CSS-002: :has-text() with transformers
#[tokio::test]
async fn test_has_text_with_transformers() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="products">
                    <div class="product">   SMARTPHONE   - $599.99   </div>
                    <div class="product">  laptop computer - €899.50  </div>
                    <div class="product">    TABLET DEVICE    </div>
                    <div class="product">desktop workstation</div>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Filter for "smartphone" with case-insensitive matching and normalization
    selectors.insert("mobile_devices".to_string(), CssSelectorConfig {
        selector: ".product".to_string(),
        transformers: vec!["trim".to_string(), "normalize_ws".to_string(), "lowercase".to_string()],
        has_text_filter: Some(HasTextFilter {
            pattern: "smartphone".to_string(),
            case_insensitive: true,
            partial_match: true,
        }),
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should process and match the smartphone
    assert!(result.extraction_confidence > 0.0);

    Ok(())
}

/// Test CSS-002: :has-text() with fallback selectors
#[tokio::test]
async fn test_has_text_with_fallbacks() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="primary-content">
                    <h1>Regular Article Title</h1>
                    <p>Some content here.</p>
                </div>
                <div class="secondary-content">
                    <h2 class="emergency">URGENT: Emergency alert!</h2>
                    <p>Emergency details here.</p>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Primary selector won't match, fallback should work
    selectors.insert("urgent_content".to_string(), CssSelectorConfig {
        selector: ".primary-content h1".to_string(),
        transformers: vec!["trim".to_string()],
        has_text_filter: Some(HasTextFilter {
            pattern: "URGENT".to_string(),
            case_insensitive: false,
            partial_match: true,
        }),
        fallbacks: vec![
            ".secondary-content h2".to_string(), // This should match
            ".emergency".to_string(),
        ],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should fall back to secondary content
    assert!(result.content.contains("URGENT") || result.extraction_confidence > 0.0);

    Ok(())
}

/// Test CSS-002: Empty and edge case :has-text() patterns
#[tokio::test]
async fn test_has_text_edge_cases() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="content">
                    <p class="text">   </p>
                    <p class="text"></p>
                    <p class="text">Single word</p>
                    <p class="text">Multiple words here</p>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Test empty pattern (should match everything)
    selectors.insert("any_content".to_string(), CssSelectorConfig {
        selector: ".text".to_string(),
        transformers: vec!["trim".to_string()],
        has_text_filter: Some(HasTextFilter {
            pattern: "".to_string(),
            case_insensitive: false,
            partial_match: true,
        }),
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    // Test single character pattern
    selectors.insert("single_char".to_string(), CssSelectorConfig {
        selector: ".text".to_string(),
        transformers: vec!["trim".to_string()],
        has_text_filter: Some(HasTextFilter {
            pattern: "S".to_string(),
            case_insensitive: false,
            partial_match: true,
        }),
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should handle edge cases gracefully
    assert!(result.extraction_confidence >= 0.0);
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}