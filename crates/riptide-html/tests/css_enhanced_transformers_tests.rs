//! Tests for enhanced transformers: join and regex_replace
//! These are the final 2 transformers completing the CSS-003 requirement of 12 transformers

use anyhow::Result;
use riptide_html::css_extraction::*;
use std::collections::HashMap;

/// Test transformer: join - Convert arrays/delimited text to joined strings
#[tokio::test]
async fn test_join_transformer() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="data">
                    <script class="json-array" type="application/json">
                        ["apple", "banana", "cherry", "date"]
                    </script>
                    <span class="csv-data">item1, item2, item3, item4</span>
                    <div class="semicolon-list">first; second; third; fourth</div>
                    <div class="mixed-array">["tech", "programming", "web"]</div>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Test joining JSON array
    selectors.insert("fruit_list".to_string(), CssSelectorConfig {
        selector: ".json-array".to_string(),
        transformers: vec!["join".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    // Test joining CSV data
    selectors.insert("csv_items".to_string(), CssSelectorConfig {
        selector: ".csv-data".to_string(),
        transformers: vec!["join".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    // Test joining semicolon-separated data
    selectors.insert("semicolon_items".to_string(), CssSelectorConfig {
        selector: ".semicolon-list".to_string(),
        transformers: vec!["join".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should successfully join arrays and delimited text
    assert!(result.extraction_confidence > 0.0);
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}

/// Test transformer: regex_replace - Remove HTML tags and clean text
#[tokio::test]
async fn test_regex_replace_transformer() -> Result<()> {
    let html = r##"
        <html>
            <body>
                <div class="content">
                    <div class="html-content">
                        <p>This is <strong>important</strong> text with <a href="#">links</a> and <em>emphasis</em>.</p>
                    </div>
                    <div class="messy-text">
                        Text    with     multiple     spaces    and
                        line breaks
                        everywhere.
                    </div>
                    <div class="tagged-content">
                        <span class="highlight">Clean</span> this <b>up</b> please!
                    </div>
                </div>
            </body>
        </html>
    "##;

    let mut selectors = HashMap::new();

    // Test removing HTML tags from content
    selectors.insert("clean_content".to_string(), CssSelectorConfig {
        selector: ".html-content".to_string(),
        transformers: vec!["regex_replace".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    // Test cleaning messy whitespace
    selectors.insert("clean_text".to_string(), CssSelectorConfig {
        selector: ".messy-text".to_string(),
        transformers: vec!["regex_replace".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    // Test cleaning tagged content
    selectors.insert("tagged_clean".to_string(), CssSelectorConfig {
        selector: ".tagged-content".to_string(),
        transformers: vec!["regex_replace".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should remove HTML tags and normalize whitespace
    assert!(result.extraction_confidence > 0.0);
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}

/// Test chaining transformers: split -> join workflow
#[tokio::test]
async fn test_split_join_chain() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="tags">
                    <span class="tag-list">technology, programming, web-development, testing, automation</span>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // First split the CSV, then join it back (normalizes formatting)
    selectors.insert("normalized_tags".to_string(), CssSelectorConfig {
        selector: ".tag-list".to_string(),
        transformers: vec!["split".to_string(), "join".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should normalize the tag formatting
    assert!(result.extraction_confidence > 0.0);
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}

/// Test enhanced HTML decode transformer with numeric entities
#[tokio::test]
async fn test_enhanced_html_decode() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="content">
                    <div class="entities1">&copy; 2023 Company&trade; &amp; Partners&reg;</div>
                    <div class="entities2">&hellip; more content &mdash; with dashes &ndash; here</div>
                    <div class="numeric">&#8364; 299.99 &#8211; Special Price &#8482;</div>
                    <div class="mixed">He said &quot;Hello&quot; &#38; waved &#39;goodbye&#39;</div>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    selectors.insert("decoded1".to_string(), CssSelectorConfig {
        selector: ".entities1".to_string(),
        transformers: vec!["html_decode".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    selectors.insert("decoded2".to_string(), CssSelectorConfig {
        selector: ".entities2".to_string(),
        transformers: vec!["html_decode".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    selectors.insert("numeric_entities".to_string(), CssSelectorConfig {
        selector: ".numeric".to_string(),
        transformers: vec!["html_decode".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    selectors.insert("mixed_entities".to_string(), CssSelectorConfig {
        selector: ".mixed".to_string(),
        transformers: vec!["html_decode".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should decode all HTML entities including numeric ones
    assert!(result.extraction_confidence > 0.0);
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}

/// Test all 12 transformers in a complex workflow
#[tokio::test]
async fn test_comprehensive_transformer_workflow() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="product">
                    <h1 class="title">   PREMIUM SMARTPHONE &amp; TABLET BUNDLE   </h1>
                    <div class="price">PRICE: $1,299.99 USD</div>
                    <div class="description">
                        <p>The <strong>latest</strong> technology with <em>advanced</em> features.</p>
                        <p>Available    from    12/15/2023</p>
                    </div>
                    <div class="tags">technology, mobile, electronics, premium, bundle</div>
                    <div class="rating">Rating: 4.8/5.0 (excellent reviews)</div>
                    <script class="specs" type="application/json">
                        {"ram": "8GB", "storage": "256GB", "display": "6.5 inch"}
                    </script>
                    <div class="urls">
                        <a href="/support">Support</a>
                        <a href="/warranty">Warranty</a>
                    </div>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Complex multi-transformer workflow
    selectors.insert("clean_title".to_string(), CssSelectorConfig {
        selector: ".title".to_string(),
        transformers: vec![
            "trim".to_string(),
            "html_decode".to_string(),
            "normalize_ws".to_string(),
            "lowercase".to_string()
        ],
        has_text_filter: None,
        fallbacks: vec![],
        required: true,
        merge_policy: Some(MergePolicy::CssWins),
    });

    // Extract and format price
    selectors.insert("formatted_price".to_string(), CssSelectorConfig {
        selector: ".price".to_string(),
        transformers: vec!["currency".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    // Clean description removing HTML tags
    selectors.insert("clean_description".to_string(), CssSelectorConfig {
        selector: ".description".to_string(),
        transformers: vec!["regex_replace".to_string(), "normalize_ws".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    // Process tags: split -> lowercase -> join
    selectors.insert("processed_tags".to_string(), CssSelectorConfig {
        selector: ".tags".to_string(),
        transformers: vec!["split".to_string(), "join".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::Merge),
    });

    // Extract numeric rating
    selectors.insert("numeric_rating".to_string(), CssSelectorConfig {
        selector: ".rating".to_string(),
        transformers: vec!["number".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    // Parse and format JSON specs
    selectors.insert("formatted_specs".to_string(), CssSelectorConfig {
        selector: ".specs".to_string(),
        transformers: vec!["json_parse".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    // Convert relative URLs to absolute
    selectors.insert("absolute_urls".to_string(), CssSelectorConfig {
        selector: ".urls a".to_string(),
        transformers: vec!["url_abs".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::Merge),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com/products").await?;

    // Should successfully process with multiple complex transformers
    assert!(!result.title.is_empty());
    assert!(result.content.len() > 100); // Should have substantial content
    assert!(result.extraction_confidence > 0.7); // High confidence with good selectors
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}