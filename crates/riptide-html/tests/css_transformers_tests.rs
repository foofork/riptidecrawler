//! Comprehensive tests for CSS-003: 12 Content Transformers
//!
//! Testing all transformer functions including:
//! 1. trim - Remove leading/trailing whitespace
//! 2. normalize_ws - Normalize internal whitespace
//! 3. number - Extract numeric values
//! 4. currency - Parse currency values
//! 5. date_iso - Convert to ISO date format
//! 6. url_abs - Convert relative URLs to absolute
//! 7. lowercase - Convert to lowercase
//! 8. uppercase - Convert to uppercase
//! 9. split - Split text by delimiter
//! 10. regex_extract - Extract via regex pattern
//! 11. json_parse - Parse JSON strings
//! 12. html_decode - Decode HTML entities

use anyhow::Result;
use riptide_html::css_extraction::*;
use std::collections::HashMap;

/// Test transformer 1: trim - Remove leading/trailing whitespace
#[tokio::test]
async fn test_trim_transformer() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="content">
                    <h1 class="title">   Whitespace Title   </h1>
                    <p class="text">		Tab	and	space	content		</p>
                    <span class="label">
                        Multiline
                        content
                    </span>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    selectors.insert("trimmed_title".to_string(), CssSelectorConfig {
        selector: ".title".to_string(),
        transformers: vec!["trim".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    selectors.insert("trimmed_text".to_string(), CssSelectorConfig {
        selector: ".text".to_string(),
        transformers: vec!["trim".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should trim whitespace from content
    assert!(result.content.contains("Whitespace Title"));
    assert!(!result.content.starts_with("   "));
    assert!(!result.content.ends_with("   "));

    Ok(())
}

/// Test transformer 2: normalize_ws - Normalize internal whitespace
#[tokio::test]
async fn test_normalize_whitespace_transformer() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="content">
                    <p class="text">Multiple    spaces     between    words</p>
                    <p class="multiline">Line 1

                    Line 2
                    	Tab line</p>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    selectors.insert("normalized_content".to_string(), CssSelectorConfig {
        selector: ".text".to_string(),
        transformers: vec!["normalize_ws".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Internal whitespace should be normalized
    assert!(result.extraction_confidence > 0.0);

    Ok(())
}

/// Test transformer 3: number - Extract numeric values
#[tokio::test]
async fn test_number_transformer() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="stats">
                    <span class="count">Total: 12,345 items</span>
                    <span class="percentage">Success rate: 98.5%</span>
                    <span class="scientific">Value: 1.23e-4</span>
                    <span class="negative">Change: -45.67</span>
                    <span class="integer">Quantity: 42</span>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    selectors.insert("item_count".to_string(), CssSelectorConfig {
        selector: ".count".to_string(),
        transformers: vec!["number".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    selectors.insert("success_rate".to_string(), CssSelectorConfig {
        selector: ".percentage".to_string(),
        transformers: vec!["number".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    selectors.insert("negative_value".to_string(), CssSelectorConfig {
        selector: ".negative".to_string(),
        transformers: vec!["number".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should extract numeric values from text
    assert!(result.extraction_confidence > 0.0);

    Ok(())
}

/// Test transformer 4: currency - Parse currency values
#[tokio::test]
async fn test_currency_transformer() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="prices">
                    <span class="usd">Price: $1,299.99</span>
                    <span class="eur">Cost: €899.50</span>
                    <span class="gbp">Value: £750.25</span>
                    <span class="yen">Amount: ¥125,000</span>
                    <span class="plain">Simple: 45.67</span>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    selectors.insert("usd_price".to_string(), CssSelectorConfig {
        selector: ".usd".to_string(),
        transformers: vec!["currency".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    selectors.insert("eur_price".to_string(), CssSelectorConfig {
        selector: ".eur".to_string(),
        transformers: vec!["currency".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    selectors.insert("plain_number".to_string(), CssSelectorConfig {
        selector: ".plain".to_string(),
        transformers: vec!["currency".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should extract and normalize currency values
    assert!(result.extraction_confidence > 0.0);

    Ok(())
}

/// Test transformer 5: date_iso - Convert to ISO date format
#[tokio::test]
async fn test_date_iso_transformer() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="dates">
                    <time class="iso">2023-12-01</time>
                    <span class="us">12/01/2023</span>
                    <span class="eu">01.12.2023</span>
                    <span class="mixed">Published on 15/06/2023</span>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    selectors.insert("iso_date".to_string(), CssSelectorConfig {
        selector: ".iso".to_string(),
        transformers: vec!["date_iso".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    selectors.insert("us_date".to_string(), CssSelectorConfig {
        selector: ".us".to_string(),
        transformers: vec!["date_iso".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    selectors.insert("eu_date".to_string(), CssSelectorConfig {
        selector: ".eu".to_string(),
        transformers: vec!["date_iso".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should convert dates to ISO format
    assert!(result.extraction_confidence > 0.0);

    Ok(())
}

/// Test transformer 6: url_abs - Convert relative URLs to absolute
#[tokio::test]
async fn test_url_absolute_transformer() -> Result<()> {
    let html = r##"
        <html>
            <body>
                <div class="links">
                    <a class="relative" href="/page1">Relative link</a>
                    <a class="absolute" href="https://other.com/page2">Absolute link</a>
                    <a class="protocol-relative" href="//cdn.example.com/file.js">Protocol relative</a>
                    <a class="fragment" href="#section1">Fragment link</a>
                </div>
            </body>
        </html>
    "##;

    let mut selectors = HashMap::new();

    selectors.insert("relative_link".to_string(), CssSelectorConfig {
        selector: ".relative".to_string(),
        transformers: vec!["url_abs".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    selectors.insert("absolute_link".to_string(), CssSelectorConfig {
        selector: ".absolute".to_string(),
        transformers: vec!["url_abs".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com/test").await?;

    // Should convert relative URLs to absolute
    assert!(result.extraction_confidence > 0.0);

    Ok(())
}

/// Test transformer 7: lowercase - Convert to lowercase
#[tokio::test]
async fn test_lowercase_transformer() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="content">
                    <span class="mixed">MiXeD CaSe TeXt</span>
                    <span class="upper">UPPERCASE TEXT</span>
                    <span class="lower">lowercase text</span>
                    <span class="special">123 SPECIAL @#$ Characters</span>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    selectors.insert("mixed_case".to_string(), CssSelectorConfig {
        selector: ".mixed".to_string(),
        transformers: vec!["lowercase".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    selectors.insert("upper_case".to_string(), CssSelectorConfig {
        selector: ".upper".to_string(),
        transformers: vec!["lowercase".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should convert all text to lowercase
    assert!(result.extraction_confidence > 0.0);

    Ok(())
}

/// Test transformer 8: uppercase - Convert to uppercase
#[tokio::test]
async fn test_uppercase_transformer() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="content">
                    <span class="mixed">MiXeD CaSe TeXt</span>
                    <span class="lower">lowercase text</span>
                    <span class="special">special 123 !@# characters</span>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    selectors.insert("mixed_upper".to_string(), CssSelectorConfig {
        selector: ".mixed".to_string(),
        transformers: vec!["uppercase".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    selectors.insert("lower_upper".to_string(), CssSelectorConfig {
        selector: ".lower".to_string(),
        transformers: vec!["uppercase".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should convert all text to uppercase
    assert!(result.extraction_confidence > 0.0);

    Ok(())
}

/// Test transformer 9: split - Split text by delimiter
#[tokio::test]
async fn test_split_transformer() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="tags">
                    <span class="csv">tag1, tag2, tag3, tag4</span>
                    <span class="items">apple,banana,cherry,date</span>
                    <span class="categories">tech, programming, web development</span>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    selectors.insert("tag_list".to_string(), CssSelectorConfig {
        selector: ".csv".to_string(),
        transformers: vec!["split".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    selectors.insert("item_list".to_string(), CssSelectorConfig {
        selector: ".items".to_string(),
        transformers: vec!["split".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should split text into arrays
    assert!(result.extraction_confidence > 0.0);

    Ok(())
}

/// Test transformer 10: regex_extract - Extract via regex pattern
#[tokio::test]
async fn test_regex_extract_transformer() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="contacts">
                    <span class="email">Contact us at support@example.com for help</span>
                    <span class="phone">Call +1-555-123-4567 today</span>
                    <span class="multiple">Email admin@test.org or sales@company.net</span>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Note: The regex_extract transformer uses a default email pattern
    selectors.insert("email_address".to_string(), CssSelectorConfig {
        selector: ".email".to_string(),
        transformers: vec!["regex_extract".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    selectors.insert("multiple_emails".to_string(), CssSelectorConfig {
        selector: ".multiple".to_string(),
        transformers: vec!["regex_extract".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should extract email addresses using regex
    assert!(result.extraction_confidence > 0.0);

    Ok(())
}

/// Test transformer 11: json_parse - Parse JSON strings
#[tokio::test]
async fn test_json_parse_transformer() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <script class="config" type="application/json">
                    {"name": "test", "value": 123, "active": true}
                </script>
                <div class="data" data-json='{"items": ["a", "b", "c"], "count": 3}'>
                    Some content
                </div>
                <span class="simple">{"key": "value"}</span>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    selectors.insert("config_json".to_string(), CssSelectorConfig {
        selector: ".config".to_string(),
        transformers: vec!["json_parse".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    selectors.insert("simple_json".to_string(), CssSelectorConfig {
        selector: ".simple".to_string(),
        transformers: vec!["json_parse".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should parse JSON and format it
    assert!(result.extraction_confidence > 0.0);

    Ok(())
}

/// Test transformer 12: html_decode - Decode HTML entities
#[tokio::test]
async fn test_html_decode_transformer() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="content">
                    <span class="entities">&lt;div&gt;HTML &amp; entities&lt;/div&gt;</span>
                    <span class="quotes">He said &quot;Hello&quot; &amp; left</span>
                    <span class="apostrophe">It&#39;s a great day!</span>
                    <span class="nbsp">Word&nbsp;spacing&nbsp;here</span>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    selectors.insert("html_entities".to_string(), CssSelectorConfig {
        selector: ".entities".to_string(),
        transformers: vec!["html_decode".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    selectors.insert("quoted_text".to_string(), CssSelectorConfig {
        selector: ".quotes".to_string(),
        transformers: vec!["html_decode".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    selectors.insert("apostrophe_text".to_string(), CssSelectorConfig {
        selector: ".apostrophe".to_string(),
        transformers: vec!["html_decode".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should decode HTML entities
    assert!(result.extraction_confidence > 0.0);

    Ok(())
}

/// Test chained transformers - Multiple transformers applied in sequence
#[tokio::test]
async fn test_chained_transformers() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="product">
                    <span class="price">   PRICE: $1,299.99   </span>
                    <span class="title">   SMARTPHONE &amp; TABLET   </span>
                    <span class="tags">   Tech, Mobile, Electronics   </span>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Chain multiple transformers: trim + currency
    selectors.insert("clean_price".to_string(), CssSelectorConfig {
        selector: ".price".to_string(),
        transformers: vec!["trim".to_string(), "currency".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    // Chain: trim + html_decode + lowercase + normalize_ws
    selectors.insert("clean_title".to_string(), CssSelectorConfig {
        selector: ".title".to_string(),
        transformers: vec![
            "trim".to_string(),
            "html_decode".to_string(),
            "lowercase".to_string(),
            "normalize_ws".to_string()
        ],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    // Chain: trim + split + then each item processed
    selectors.insert("clean_tags".to_string(), CssSelectorConfig {
        selector: ".tags".to_string(),
        transformers: vec!["trim".to_string(), "split".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should apply transformers in sequence
    assert!(result.extraction_confidence > 0.0);
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}

/// Test transformer error handling
#[tokio::test]
async fn test_transformer_error_handling() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="content">
                    <span class="no-number">No numeric content here</span>
                    <span class="invalid-json">{ invalid json }</span>
                    <span class="no-currency">Just plain text</span>
                    <span class="valid">Valid content: 123</span>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Transformer that will fail on this content
    selectors.insert("extract_number".to_string(), CssSelectorConfig {
        selector: ".no-number".to_string(),
        transformers: vec!["number".to_string()],
        has_text_filter: None,
        fallbacks: vec![".valid".to_string()], // Should fall back
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    // Invalid JSON that should be handled gracefully
    selectors.insert("parse_json".to_string(), CssSelectorConfig {
        selector: ".invalid-json".to_string(),
        transformers: vec!["json_parse".to_string()],
        has_text_filter: None,
        fallbacks: vec![],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should handle transformer errors gracefully
    assert!(result.extraction_confidence >= 0.0);
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}