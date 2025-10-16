//! Advanced CSS selector tests for enhanced combinators and pseudo-selectors
//! Tests CSS-001 advanced features including attribute operators and complex selectors

use anyhow::Result;
use riptide_extraction::css_extraction::*;
use std::collections::HashMap;

/// Test advanced attribute selectors with operators
#[tokio::test]
async fn test_advanced_attribute_selectors() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="products">
                    <!-- Attribute existence -->
                    <div class="product" data-category="electronics" data-price="999.99">
                        <h3>Laptop Pro</h3>
                    </div>

                    <!-- Attribute prefix matching -->
                    <div class="product" data-category="electronics-mobile" data-price="599.99">
                        <h3>Smartphone X</h3>
                    </div>

                    <!-- Attribute suffix matching -->
                    <div class="product" data-category="home-electronics" data-price="299.99">
                        <h3>Smart TV</h3>
                    </div>

                    <!-- Attribute substring matching -->
                    <div class="product" data-category="electronics-gaming" data-price="449.99">
                        <h3>Gaming Console</h3>
                    </div>

                    <!-- Class list matching -->
                    <div class="product featured premium" data-category="accessories" data-price="199.99">
                        <h3>Wireless Headphones</h3>
                    </div>
                </div>

                <div class="links">
                    <a href="https://example.com" class="external">External Link</a>
                    <a href="/internal" class="internal">Internal Link</a>
                    <a href="mailto:contact@example.com" class="email">Email Link</a>
                    <a href="tel:+1234567890" class="phone">Phone Link</a>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Test attribute existence: [attr]
    selectors.insert(
        "has_category".to_string(),
        CssSelectorConfig {
            selector: "[data-category]".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // Test exact attribute value: [attr="value"]
    selectors.insert(
        "electronics_exact".to_string(),
        CssSelectorConfig {
            selector: "[data-category='electronics']".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // Test attribute prefix: [attr^="value"]
    selectors.insert(
        "electronics_prefix".to_string(),
        CssSelectorConfig {
            selector: "[data-category^='electronics']".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // Test attribute suffix: [attr$="value"]
    selectors.insert(
        "electronics_suffix".to_string(),
        CssSelectorConfig {
            selector: "[data-category$='electronics']".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // Test attribute substring: [attr*="value"]
    selectors.insert(
        "electronics_contains".to_string(),
        CssSelectorConfig {
            selector: "[data-category*='electronics']".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // Test class list matching: [class~="value"]
    selectors.insert(
        "featured_products".to_string(),
        CssSelectorConfig {
            selector: "[class~='featured']".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // Test complex attribute selector with URL patterns
    selectors.insert(
        "external_links".to_string(),
        CssSelectorConfig {
            selector: "a[href^='https://']".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    selectors.insert(
        "email_links".to_string(),
        CssSelectorConfig {
            selector: "a[href^='mailto:']".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should extract elements with advanced attribute matching
    assert!(result.extraction_confidence > 0.0);
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}

/// Test complex combinator selectors
#[tokio::test]
async fn test_complex_combinators() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <article class="blog-post">
                    <header class="post-header">
                        <h1 class="post-title">Blog Post Title</h1>
                        <div class="post-meta">
                            <span class="author">John Doe</span>
                            <time class="date">2023-12-15</time>
                        </div>
                    </header>

                    <div class="post-content">
                        <p class="intro">Introduction paragraph</p>
                        <div class="content-section">
                            <h2 class="section-title">First Section</h2>
                            <p class="content-paragraph">First paragraph in section</p>
                            <div class="subsection">
                                <h3 class="subsection-title">Subsection</h3>
                                <p class="content-paragraph">Paragraph in subsection</p>
                            </div>
                        </div>
                        <div class="content-section">
                            <h2 class="section-title">Second Section</h2>
                            <p class="content-paragraph">Second paragraph in section</p>
                        </div>
                    </div>

                    <footer class="post-footer">
                        <div class="tags">
                            <span class="tag">Technology</span>
                            <span class="tag">Programming</span>
                            <span class="tag">Web</span>
                        </div>
                    </footer>
                </article>

                <aside class="sidebar">
                    <h2 class="sidebar-title">Related Posts</h2>
                    <ul class="related-posts">
                        <li class="related-post">
                            <a href="/post1">Related Post 1</a>
                        </li>
                        <li class="related-post">
                            <a href="/post2">Related Post 2</a>
                        </li>
                    </ul>
                </aside>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // Direct child combinator: >
    selectors.insert(
        "direct_sections".to_string(),
        CssSelectorConfig {
            selector: ".post-content > .content-section".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // Adjacent sibling combinator: +
    selectors.insert(
        "title_after_intro".to_string(),
        CssSelectorConfig {
            selector: ".intro + .content-section .section-title".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // General sibling combinator: ~
    selectors.insert(
        "sections_after_intro".to_string(),
        CssSelectorConfig {
            selector: ".intro ~ .content-section".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // Complex descendant with multiple levels
    selectors.insert(
        "nested_paragraphs".to_string(),
        CssSelectorConfig {
            selector: "article .content-section .subsection .content-paragraph".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // Multiple class selector
    selectors.insert(
        "article_headers".to_string(),
        CssSelectorConfig {
            selector: "article.blog-post header.post-header".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // Pseudo-element simulation (first child)
    selectors.insert(
        "first_section_title".to_string(),
        CssSelectorConfig {
            selector: ".content-section:first-child .section-title".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![".section-title".to_string()],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should extract elements using complex combinators
    assert!(result.extraction_confidence > 0.0);
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}

/// Test advanced pseudo-selectors
#[tokio::test]
async fn test_advanced_pseudo_selectors() -> Result<()> {
    let html = r#"
        <html>
            <body>
                <div class="grid">
                    <div class="item">Item 1</div>
                    <div class="item">Item 2</div>
                    <div class="item">Item 3</div>
                    <div class="item">Item 4</div>
                    <div class="item">Item 5</div>
                    <div class="item">Item 6</div>
                    <div class="item">Item 7</div>
                    <div class="item">Item 8</div>
                </div>

                <ul class="list">
                    <li class="list-item">First item</li>
                    <li class="list-item">Second item</li>
                    <li class="list-item">Third item</li>
                    <li class="list-item">Fourth item</li>
                    <li class="list-item">Fifth item</li>
                </ul>

                <div class="mixed">
                    <h1>Heading 1</h1>
                    <p>Paragraph 1</p>
                    <h2>Heading 2</h2>
                    <p>Paragraph 2</p>
                    <h2>Heading 3</h2>
                    <p>Paragraph 3</p>
                </div>
            </body>
        </html>
    "#;

    let mut selectors = HashMap::new();

    // :nth-child variations
    selectors.insert(
        "odd_items".to_string(),
        CssSelectorConfig {
            selector: ".item:nth-child(odd)".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::Merge),
        },
    );

    selectors.insert(
        "even_items".to_string(),
        CssSelectorConfig {
            selector: ".item:nth-child(even)".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::Merge),
        },
    );

    // Specific nth-child positions
    selectors.insert(
        "third_item".to_string(),
        CssSelectorConfig {
            selector: ".item:nth-child(3)".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // Mathematical nth-child expressions
    selectors.insert(
        "every_third_item".to_string(),
        CssSelectorConfig {
            selector: ".item:nth-child(3n)".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::Merge),
        },
    );

    selectors.insert(
        "every_third_plus_one".to_string(),
        CssSelectorConfig {
            selector: ".item:nth-child(3n+1)".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::Merge),
        },
    );

    // First and last child
    selectors.insert(
        "first_items".to_string(),
        CssSelectorConfig {
            selector: ".item:first-child, .list-item:first-child".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::Merge),
        },
    );

    selectors.insert(
        "last_items".to_string(),
        CssSelectorConfig {
            selector: ".item:last-child, .list-item:last-child".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::Merge),
        },
    );

    // :nth-of-type simulation (should be enhanced to work with scraper)
    selectors.insert(
        "second_paragraph".to_string(),
        CssSelectorConfig {
            selector: "p:nth-of-type(2)".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec!["p:nth-child(2)".to_string()],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Should extract elements using advanced pseudo-selectors
    assert!(result.extraction_confidence > 0.0);
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}

/// Test selector performance and optimization
#[tokio::test]
async fn test_selector_performance() -> Result<()> {
    // Generate large HTML document for performance testing
    let large_html = generate_large_html_document(1000);

    let mut selectors = HashMap::new();

    // Multiple complex selectors
    selectors.insert(
        "titles".to_string(),
        CssSelectorConfig {
            selector: "article.post header.post-header h1.post-title".to_string(),
            transformers: vec!["trim".to_string(), "normalize_ws".to_string()],
            has_text_filter: None,
            fallbacks: vec!["h1".to_string(), ".title".to_string()],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    selectors.insert(
        "content_paragraphs".to_string(),
        CssSelectorConfig {
            selector: "article.post .post-content p:nth-child(odd)".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec!["p".to_string()],
            required: false,
            merge_policy: Some(MergePolicy::Merge),
        },
    );

    selectors.insert(
        "tagged_content".to_string(),
        CssSelectorConfig {
            selector: ".post-content [data-type='important']".to_string(),
            transformers: vec!["trim".to_string(), "html_decode".to_string()],
            has_text_filter: Some(HasTextFilter {
                pattern: r"(?i)\b(important|critical|urgent)\b".to_string(),
                case_insensitive: true,
                partial_match: true,
                regex_mode: true,
                regex: None,
            }),
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    selectors.insert(
        "price_elements".to_string(),
        CssSelectorConfig {
            selector: ".price, [data-price], .cost".to_string(),
            transformers: vec!["currency".to_string()],
            has_text_filter: Some(HasTextFilter {
                pattern: r"\$\d+\.\d{2}".to_string(),
                case_insensitive: false,
                partial_match: true,
                regex_mode: true,
                regex: None,
            }),
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // Time the extraction
    let start = std::time::Instant::now();
    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor
        .extract(&large_html, "https://example.com")
        .await?;
    let duration = start.elapsed();

    println!("Performance test - Processing time: {:?}", duration);
    println!("Content length: {} chars", large_html.len());
    println!("Extracted content length: {} chars", result.content.len());
    println!("Confidence score: {:.3}", result.extraction_confidence);

    // Should meet performance target of <100ms for reasonable document sizes
    // Allow some margin for test environment variability
    assert!(duration.as_millis() < 500);
    assert!(result.extraction_confidence > 0.0);
    assert_eq!(result.strategy_used, "css_json_enhanced");

    Ok(())
}

/// Test comprehensive field extraction coverage
#[tokio::test]
async fn test_field_extraction_coverage() -> Result<()> {
    let html = r#"
        <html>
            <head>
                <title>Comprehensive Test Page</title>
                <meta name="description" content="A comprehensive test page for field extraction coverage">
                <meta name="author" content="Test Author">
                <meta property="og:title" content="OG Title">
                <meta property="og:description" content="OG Description">
                <meta name="keywords" content="test, extraction, coverage">
            </head>
            <body>
                <article class="main-content">
                    <header class="article-header">
                        <h1 class="main-title">Main Article Title</h1>
                        <div class="article-meta">
                            <span class="author">Article Author</span>
                            <time class="publish-date" datetime="2023-12-15">December 15, 2023</time>
                            <div class="category">Technology</div>
                        </div>
                    </header>

                    <div class="article-content">
                        <p class="lead">This is the lead paragraph with important information.</p>
                        <div class="content-section">
                            <h2>Section 1</h2>
                            <p>Content paragraph 1 with substantial text content.</p>
                            <p>Content paragraph 2 with more information.</p>
                        </div>
                        <div class="content-section">
                            <h2>Section 2</h2>
                            <p>Another content paragraph with relevant information.</p>
                            <blockquote class="quote">Important quoted content here.</blockquote>
                        </div>
                    </div>

                    <footer class="article-footer">
                        <div class="tags">
                            <span class="tag">web</span>
                            <span class="tag">technology</span>
                            <span class="tag">testing</span>
                        </div>
                        <div class="social-links">
                            <a href="https://twitter.com/share" class="twitter">Share on Twitter</a>
                            <a href="https://facebook.com/share" class="facebook">Share on Facebook</a>
                        </div>
                    </footer>
                </article>

                <aside class="sidebar">
                    <div class="related">
                        <h3>Related Articles</h3>
                        <ul>
                            <li><a href="/article1">Related Article 1</a></li>
                            <li><a href="/article2">Related Article 2</a></li>
                        </ul>
                    </div>
                </aside>
            </body>
        </html>
    "#;

    // Use comprehensive default selectors
    let selectors = default_selectors();
    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor.extract(html, "https://example.com").await?;

    // Check field extraction coverage
    let field_coverage = calculate_field_coverage(&result);

    println!("Field extraction coverage: {:.1}%", field_coverage * 100.0);
    println!("Title: '{}'", result.title);
    println!("Content length: {} chars", result.content.len());
    println!("Summary: {:?}", result.summary);
    println!("Confidence: {:.3}", result.extraction_confidence);

    // Should achieve ≥80% field coverage on clean pages
    assert!(
        field_coverage >= 0.8,
        "Field coverage should be ≥80%, got {:.1}%",
        field_coverage * 100.0
    );
    assert!(!result.title.is_empty());
    assert!(result.content.len() > 100);
    assert!(result.summary.is_some());
    assert!(result.extraction_confidence > 0.7);

    Ok(())
}

// Helper function to generate large HTML document for performance testing
fn generate_large_html_document(num_articles: usize) -> String {
    let mut html = String::from(
        r#"
        <html>
            <head>
                <title>Large Test Document</title>
                <meta name="description" content="Large document for performance testing">
            </head>
            <body>
                <main class="content">
    "#,
    );

    for i in 0..num_articles {
        html.push_str(&format!(
            r#"
            <article class="post" data-id="{}">
                <header class="post-header">
                    <h1 class="post-title">Article Title {}</h1>
                    <div class="post-meta">
                        <span class="author">Author {}</span>
                        <time class="date">2023-12-{:02}</time>
                    </div>
                </header>
                <div class="post-content">
                    <p>This is paragraph 1 of article {} with some content.</p>
                    <div class="special" data-type="important">Important content here.</div>
                    <p>This is paragraph 2 with more information and details.</p>
                    <div class="price">Price: ${}.99</div>
                </div>
            </article>
        "#,
            i,
            i,
            i % 10,
            (i % 28) + 1,
            i,
            (i * 10) + 99
        ));
    }

    html.push_str(
        r#"
                </main>
            </body>
        </html>
    "#,
    );

    html
}

// Helper function to calculate field extraction coverage
fn calculate_field_coverage(result: &riptide_extraction::ExtractedContent) -> f64 {
    let mut filled_fields = 0.0;
    let total_fields = 3.0; // title, content, summary

    // Check title
    if !result.title.is_empty() && result.title != "Untitled" {
        filled_fields += 1.0;
    }

    // Check content
    if !result.content.is_empty() && result.content.len() > 50 {
        filled_fields += 1.0;
    }

    // Check summary
    if result.summary.is_some() && result.summary.as_ref().unwrap().len() > 10 {
        filled_fields += 1.0;
    }

    filled_fields / total_fields
}
