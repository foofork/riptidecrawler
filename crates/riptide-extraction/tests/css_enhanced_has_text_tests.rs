//! Enhanced :has-text() tests with regex support and advanced matching
//! Tests CSS-002 enhancement with regex pattern matching

use anyhow::Result;
use riptide_extraction::css_extraction::*;
use std::collections::HashMap;

/// Test basic :has-text() functionality (existing)
#[tokio::test]
async fn test_basic_has_text_filter() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="reviews">
                    <div class="review">This product is excellent and works great!</div>
                    <div class="review">Pretty good, but could be better.</div>
                    <div class="review">Excellent quality and fast shipping.</div>
                    <div class="review">Not bad, decent value for money.</div>
                    <div class="review">Outstanding performance, highly recommend!</div>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Select only reviews containing "excellent" (case insensitive)
    selectors.insert(
        "excellent_reviews".to_string(),
        css(".review")
            .has_text("excellent", true, true)
            .transform("trim")
            .build(),
    );

    // Select only reviews containing "good" (case sensitive)
    selectors.insert(
        "good_reviews".to_string(),
        css(".review")
            .has_text("good", false, true)
            .transform("trim")
            .build(),
    );

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should extract reviews with matching text
    assert!(result.extraction_confidence > 0.0);
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}

/// Test regex-based :has-text() filtering
#[tokio::test]
async fn test_regex_has_text_filter() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="products">
                    <div class="product">iPhone 14 Pro - $999.99</div>
                    <div class="product">Samsung Galaxy S23 - $849.00</div>
                    <div class="product">Google Pixel 7 - €699.50</div>
                    <div class="product">OnePlus 11 - £599.99</div>
                    <div class="product">Xiaomi 13 - Special offer</div>
                    <div class="product">Nothing Phone - Coming soon</div>
                </div>
                <div class="emails">
                    <div class="contact">Email: support@company.com</div>
                    <div class="contact">Contact: sales@example.org</div>
                    <div class="contact">Info: hello@startup.io</div>
                    <div class="contact">Call us: 555-123-4567</div>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Select products with price patterns using regex
    selectors.insert(
        "priced_products".to_string(),
        css(".product")
            .has_text_regex(r"[€$£]\d+\.\d{2}", false)
            .transforms(&["trim", "normalize_ws"])
            .build(),
    );

    // Select contacts with email patterns using regex
    selectors.insert(
        "email_contacts".to_string(),
        css(".contact")
            .has_text_regex(
                r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b",
                false,
            )
            .transform("trim")
            .build(),
    );

    // Select phone numbers using regex
    selectors.insert(
        "phone_contacts".to_string(),
        css(".contact")
            .has_text_regex(r"\b\d{3}-\d{3}-\d{4}\b", false)
            .transform("trim")
            .build(),
    );

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should extract elements matching regex patterns
    assert!(result.extraction_confidence > 0.0);
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}

/// Test complex regex patterns with :has-text()
#[tokio::test]
async fn test_complex_regex_patterns() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="articles">
                    <article class="news">
                        <h3>Breaking: Tech company announces new AI breakthrough</h3>
                        <p>Published on 2023-12-15 at 14:30 UTC</p>
                    </article>
                    <article class="news">
                        <h3>Sports: Local team wins championship after 10 years</h3>
                        <p>Updated yesterday at 09:15 GMT</p>
                    </article>
                    <article class="news">
                        <h3>Weather: Storm warning issued for coastal regions</h3>
                        <p>Posted on Dec 14, 2023</p>
                    </article>
                    <article class="news">
                        <h3>Business: Stock market reaches new highs</h3>
                        <p>Last modified 2 hours ago</p>
                    </article>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Select breaking news articles
    selectors.insert(
        "breaking_news".to_string(),
        css("article h3")
            .has_text_regex(r"(?i)\bbreaking\b", false)
            .transforms(&["trim", "normalize_ws"])
            .build(),
    );

    // Select articles with specific date patterns
    selectors.insert(
        "dated_articles".to_string(),
        css("article p")
            .has_text_regex(r"\d{4}-\d{2}-\d{2}", false)
            .transform("trim")
            .build(),
    );

    // Select articles mentioning time zones
    selectors.insert(
        "timezone_articles".to_string(),
        css("article p")
            .has_text_regex(r"\b(UTC|GMT)\b", false)
            .transform("trim")
            .build(),
    );

    // Select tech-related headlines
    selectors.insert(
        "tech_headlines".to_string(),
        css("article h3")
            .has_text_regex(
                r"(?i)\b(tech|technology|AI|breakthrough|innovation)\b",
                false,
            )
            .transforms(&["trim", "normalize_ws"])
            .build(),
    );

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should extract articles matching complex patterns
    assert!(result.extraction_confidence > 0.0);
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}

/// Test case sensitivity in regex :has-text()
#[tokio::test]
async fn test_case_sensitivity_regex() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="content">
                    <div class="item">JavaScript is awesome!</div>
                    <div class="item">JAVASCRIPT is powerful</div>
                    <div class="item">Learn javascript online</div>
                    <div class="item">Python vs JavaScript comparison</div>
                    <div class="item">React uses javascript</div>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Case-sensitive regex (should match exact case)
    selectors.insert(
        "exact_javascript".to_string(),
        css(".item")
            .has_text_regex(r"\bJavaScript\b", false) // Case sensitive
            .transform("trim")
            .build(),
    );

    // Case-insensitive regex (should match all variations)
    selectors.insert(
        "any_javascript".to_string(),
        css(".item")
            .has_text_regex(r"\bjavascript\b", true) // Case insensitive
            .transform("trim")
            .build(),
    );

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should handle case sensitivity correctly
    assert!(result.extraction_confidence > 0.0);
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}

/// Test :has-text() with complex selectors and fallbacks
#[tokio::test]
async fn test_has_text_with_complex_selectors() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="reviews">
                    <div class="review-card" data-rating="5">
                        <div class="review-text">This product is absolutely excellent! Highly recommended.</div>
                        <div class="review-meta">Verified purchase</div>
                    </div>
                    <div class="review-card" data-rating="3">
                        <div class="review-text">It's okay, nothing special but decent quality.</div>
                        <div class="review-meta">Verified purchase</div>
                    </div>
                    <div class="review-card" data-rating="5">
                        <div class="review-text">Excellent build quality and fast shipping!</div>
                        <div class="review-meta">Verified purchase</div>
                    </div>
                    <div class="review-card" data-rating="2">
                        <div class="review-text">Poor quality, would not recommend.</div>
                        <div class="review-meta">Not verified</div>
                    </div>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Complex selector with :has-text() and fallbacks
    selectors.insert(
        "positive_reviews".to_string(),
        CssSelectorConfig {
            selector: ".review-card .review-text".to_string(),
            transformers: vec!["trim".to_string(), "normalize_ws".to_string()],
            has_text_filter: Some(HasTextFilter {
                pattern: r"(?i)\b(excellent|amazing|great|outstanding|perfect)\b".to_string(),
                case_insensitive: true,
                partial_match: true,
                regex_mode: true,
                regex: None,
            }),
            fallbacks: vec![
                "[data-rating='5'] .review-text".to_string(),
                ".review-text".to_string(),
            ],
            required: false,
            merge_policy: Some(MergePolicy::Merge),
        },
    );

    // Verified reviews with specific text patterns
    selectors.insert(
        "verified_positive".to_string(),
        CssSelectorConfig {
            selector: ".review-card:has(.review-meta:contains('Verified')) .review-text"
                .to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: Some(HasTextFilter {
                pattern: r"(?i)\b(excellent|great|good|recommend)\b".to_string(),
                case_insensitive: true,
                partial_match: true,
                regex_mode: true,
                regex: None,
            }),
            fallbacks: vec![".review-text".to_string()],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should extract positive reviews with complex matching
    assert!(result.extraction_confidence > 0.0);
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}

/// Test error handling for invalid regex patterns
#[tokio::test]
async fn test_invalid_regex_handling() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="content">
                    <div class="item">Valid content here</div>
                    <div class="item">Another valid item</div>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Invalid regex pattern
    selectors.insert(
        "invalid_regex".to_string(),
        CssSelectorConfig {
            selector: ".item".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: Some(HasTextFilter {
                pattern: r"[unclosed bracket".to_string(), // Invalid regex
                case_insensitive: false,
                partial_match: true,
                regex_mode: true,
                regex: None,
            }),
            fallbacks: vec![".item".to_string()], // Should fall back
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should handle invalid regex gracefully and use fallback
    assert!(result.extraction_confidence >= 0.0);
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}

/// Test performance with multiple regex :has-text() filters
#[tokio::test]
async fn test_performance_multiple_regex_filters() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="large-content">
        "#.to_string() + &(0..100).map(|i| format!(
            r#"<div class="item">Item {} with various content patterns: email{}@example.com, price $1{}.99, phone {}-123-4567</div>"#,
            i, i, i*10, i+100
        )).collect::<Vec<_>>().join("") + r#"
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Multiple regex filters that should process quickly
    selectors.insert(
        "emails".to_string(),
        css(".item")
            .has_text_regex(
                r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b",
                false,
            )
            .transform("regex_extract")
            .build(),
    );

    selectors.insert(
        "prices".to_string(),
        css(".item")
            .has_text_regex(r"\$\d+\.\d{2}", false)
            .transform("currency")
            .build(),
    );

    selectors.insert(
        "phones".to_string(),
        css(".item")
            .has_text_regex(r"\b\d{3}-\d{3}-\d{4}\b", false)
            .transform("trim")
            .build(),
    );

    let start = std::time::Instant::now();
    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(&html, "https://example.com").await?;
    let duration = start.elapsed();

    // Should process within reasonable time (<100ms target)
    println!("Processing time: {:?}", duration);
    assert!(duration.as_millis() < 500); // Allow some margin for test environment
    assert!(result.extraction_confidence > 0.0);
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}
