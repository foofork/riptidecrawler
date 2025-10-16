//! Comprehensive tests for CSS-001: Enhanced CSS selector engine
//!
//! Testing advanced CSS selector features including:
//! - Class, ID, and attribute selectors
//! - Child and descendant combinators
//! - :nth-child pseudo-selectors
//! - Complex selector combinations

use anyhow::Result;
use riptide_extraction::css_extraction::*;
use std::collections::HashMap;

/// Test CSS-001: Basic enhanced selectors (class, id, attribute)
#[tokio::test]
async fn test_enhanced_css_selectors_basic() -> Result<()> {
    let html = r#"
        <html>
            <head><title>Test Page</title></head>
            <body>
                <div id="main-content" class="primary-content" data-type="article">
                    <h1 class="headline">Main Headline</h1>
                    <p class="content-paragraph">First paragraph content.</p>
                    <p class="content-paragraph">Second paragraph content.</p>
                </div>
                <div class="sidebar" data-section="aside">
                    <h2>Sidebar Title</h2>
                    <p>Sidebar content.</p>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Test ID selector
    selectors.insert(
        "main_content".to_string(),
        CssSelectorConfig {
            selector: "#main-content".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: true,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // Test class selector
    selectors.insert(
        "headline".to_string(),
        CssSelectorConfig {
            selector: ".headline".to_string(),
            transformers: vec!["trim".to_string(), "normalize_ws".to_string()],
            has_text_filter: None,
            fallbacks: vec!["h1".to_string()],
            required: true,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // Test attribute selector
    selectors.insert(
        "article_content".to_string(),
        CssSelectorConfig {
            selector: "[data-type='article']".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    assert_eq!(result.title, "Main Headline");
    assert!(result.content.contains("Main Headline"));
    assert!(result.content.contains("First paragraph"));
    assert_eq!(result.strategy_used, "css_json_enhanced");
    assert!(result.extraction_confidence > 0.7);

    Ok(())
}

/// Test CSS-001: Child and descendant combinators
#[tokio::test]
async fn test_css_combinators() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <article>
                    <header>
                        <h1>Article Title</h1>
                        <div class="meta">
                            <span class="author">John Doe</span>
                            <time class="date">2023-12-01</time>
                        </div>
                    </header>
                    <div class="content">
                        <p>Direct child paragraph.</p>
                        <div class="section">
                            <p>Nested paragraph in section.</p>
                        </div>
                    </div>
                </article>
                <aside>
                    <h2>Sidebar</h2>
                    <p>Sidebar paragraph.</p>
                </aside>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Test direct child combinator
    selectors.insert(
        "direct_content".to_string(),
        CssSelectorConfig {
            selector: ".content > p".to_string(), // Direct child only
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // Test descendant combinator
    selectors.insert(
        "all_paragraphs".to_string(),
        CssSelectorConfig {
            selector: "article p".to_string(), // All descendants
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // Test complex combinator
    selectors.insert(
        "meta_author".to_string(),
        CssSelectorConfig {
            selector: "header .meta .author".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should contain article title from h1
    assert!(result.content.contains("Article Title"));
    assert!(result.extraction_confidence > 0.0);

    Ok(())
}

/// Test CSS-001: :nth-child pseudo-selectors
#[tokio::test]
async fn test_nth_child_selectors() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <ul class="items">
                    <li class="item">First item</li>
                    <li class="item">Second item</li>
                    <li class="item">Third item</li>
                    <li class="item">Fourth item</li>
                    <li class="item">Fifth item</li>
                </ul>
                <div class="sections">
                    <section>Section 1</section>
                    <section>Section 2</section>
                    <section>Section 3</section>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Test first child
    selectors.insert(
        "first_item".to_string(),
        CssSelectorConfig {
            selector: ".items li:nth-child(1)".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // Test odd children
    selectors.insert(
        "odd_items".to_string(),
        CssSelectorConfig {
            selector: ".items li:nth-child(odd)".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // Test even children
    selectors.insert(
        "even_sections".to_string(),
        CssSelectorConfig {
            selector: ".sections section:nth-child(even)".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should extract content successfully
    assert!(result.extraction_confidence > 0.0);
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}

/// Test CSS-001: Complex selector combinations
#[tokio::test]
async fn test_complex_css_combinations() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <main class="content" id="main">
                    <article data-type="blog-post" class="primary">
                        <header class="article-header">
                            <h1 id="main-title" class="title primary-title">Complex Blog Post</h1>
                            <div class="meta-info" data-section="metadata">
                                <span class="author" data-field="author">Jane Smith</span>
                                <time class="publish-date" datetime="2023-12-01">December 1, 2023</time>
                            </div>
                        </header>
                        <div class="article-body">
                            <p class="intro-paragraph">Introduction paragraph here.</p>
                            <div class="content-section" data-section="main-content">
                                <h2 class="section-title">First Section</h2>
                                <p class="content-paragraph">Main content paragraph.</p>
                                <blockquote class="quote" data-type="pullquote">
                                    Important quote here.
                                </blockquote>
                            </div>
                        </div>
                    </article>
                </main>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Complex selector: Multiple classes and attributes
    selectors.insert(
        "primary_title".to_string(),
        CssSelectorConfig {
            selector: "article.primary h1.title.primary-title#main-title".to_string(),
            transformers: vec!["trim".to_string(), "normalize_ws".to_string()],
            has_text_filter: None,
            fallbacks: vec!["h1".to_string()],
            required: true,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // Complex selector: Attribute and descendant with class
    selectors.insert(
        "author_info".to_string(),
        CssSelectorConfig {
            selector: "[data-section='metadata'] .author[data-field='author']".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![".author".to_string()],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // Complex selector: Multiple levels with attributes
    selectors.insert("quote_content".to_string(), CssSelectorConfig {
        selector: "article[data-type='blog-post'] .content-section[data-section='main-content'] blockquote[data-type='pullquote']".to_string(),
        transformers: vec!["trim".to_string()],
        has_text_filter: None,
        fallbacks: vec!["blockquote".to_string()],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    assert_eq!(result.title, "Complex Blog Post");
    assert!(result.content.contains("Complex Blog Post"));
    assert!(result.extraction_confidence > 0.8);
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}

/// Test CSS-001: Fallback selector functionality
#[tokio::test]
async fn test_fallback_selectors() -> Result<()> {
    let html = r#"
        <html>
            <head><title>Fallback Test</title></head>
            <body>
                <!-- Primary selector won't match -->
                <div class="main-content">
                    <h2 class="heading">Secondary Heading</h2>
                    <p class="text">Content paragraph.</p>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Primary selector that won't match, with fallbacks
    selectors.insert(
        "title".to_string(),
        CssSelectorConfig {
            selector: "h1.primary-title".to_string(), // This won't match
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![
                "h1".to_string(),         // This won't match either
                "h2.heading".to_string(), // This will match
                ".title".to_string(),     // Backup
            ],
            required: true,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    assert_eq!(result.title, "Secondary Heading");
    assert!(result.extraction_confidence > 0.0);

    Ok(())
}

/// Test CSS-001: Multiple element matching
#[tokio::test]
async fn test_multiple_element_matching() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="tags">
                    <span class="tag">Technology</span>
                    <span class="tag">Web Development</span>
                    <span class="tag">CSS</span>
                    <span class="tag">Testing</span>
                </div>
                <div class="categories">
                    <div class="category">Programming</div>
                    <div class="category">Frontend</div>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Selector that matches multiple elements
    selectors.insert(
        "tags".to_string(),
        CssSelectorConfig {
            selector: ".tag".to_string(),
            transformers: vec!["trim".to_string(), "lowercase".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::Merge),
        },
    );

    selectors.insert(
        "categories".to_string(),
        CssSelectorConfig {
            selector: ".category".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should extract content successfully with multiple tags
    assert!(result.extraction_confidence > 0.0);
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}

/// Test CSS-001: Error handling for invalid selectors
#[tokio::test]
async fn test_invalid_selector_handling() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <h1>Valid Content</h1>
                <p>Some content here.</p>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Invalid CSS selector syntax
    selectors.insert(
        "invalid".to_string(),
        CssSelectorConfig {
            selector: "..invalid..selector".to_string(),
            transformers: vec![],
            has_text_filter: None,
            fallbacks: vec!["h1".to_string()], // Valid fallback
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await;

    // Should still work due to fallback, or handle gracefully
    assert!(result.is_ok());
    let extracted = result.unwrap();
    assert!(!extracted.title.is_empty());

    Ok(())
}
