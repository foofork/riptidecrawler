//! Performance tests and merge policy enhancements for CSS-004
//! Tests comprehensive conflict resolution and performance optimization

use anyhow::Result;
use riptide_extraction::css_extraction::*;
use std::collections::HashMap;

/// Test merge policy performance with large number of conflicts
#[tokio::test]
async fn test_merge_policy_performance() -> Result<()> {
    // Create two large result sets that will have conflicts
    let css_results = generate_large_result_set("css", 1000);
    let other_results = generate_large_result_set("other", 1000);

    let selectors = create_test_selectors();
    let extractor = CssJsonExtractor::new(selectors).with_merge_policy(MergePolicy::CssWins);

    let start = std::time::Instant::now();
    let (merged, conflicts) = extractor.merge_with_other(&css_results, &other_results);
    let duration = start.elapsed();

    println!("Merge policy performance:");
    println!("  - Processing time: {:?}", duration);
    println!("  - CSS results: {} fields", css_results.len());
    println!("  - Other results: {} fields", other_results.len());
    println!("  - Merged results: {} fields", merged.len());
    println!("  - Conflicts detected: {}", conflicts.len());

    // Should process merge efficiently
    assert!(duration.as_millis() < 50, "Merge should complete in <50ms");
    assert!(!merged.is_empty());
    assert!(!conflicts.is_empty()); // Should detect conflicts

    Ok(())
}

/// Test detailed conflict audit trail
#[tokio::test]
async fn test_detailed_conflict_audit() -> Result<()> {
    let css_results = create_conflicting_css_results();
    let other_results = create_conflicting_other_results();

    let mut selectors = HashMap::new();

    // Field with CssWins policy
    selectors.insert(
        "title".to_string(),
        CssSelectorConfig {
            selector: "h1".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: true,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // Field with Merge policy
    selectors.insert(
        "tags".to_string(),
        CssSelectorConfig {
            selector: ".tag".to_string(),
            transformers: vec!["lowercase".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::Merge),
        },
    );

    // Field with FirstValid policy
    selectors.insert(
        "author".to_string(),
        CssSelectorConfig {
            selector: ".author".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::FirstValid),
        },
    );

    // Field with OtherWins policy
    selectors.insert(
        "date".to_string(),
        CssSelectorConfig {
            selector: "time".to_string(),
            transformers: vec!["date_iso".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::OtherWins),
        },
    );

    let extractor = CssJsonExtractor::new(selectors);
    let (merged, conflicts) = extractor.merge_with_other(&css_results, &other_results);

    // Verify conflict audit details
    println!("Detailed conflict audit:");
    for conflict in &conflicts {
        println!("  Field: {}", conflict.field);
        println!("    CSS value: {:?}", conflict.css_value);
        println!("    Other value: {:?}", conflict.other_value);
        println!("    Resolution: {}", conflict.resolution);
        println!("    Policy: {:?}", conflict.policy_used);
        println!();
    }

    // Should have detailed conflict information
    assert!(!conflicts.is_empty(), "Should detect conflicts");

    for conflict in &conflicts {
        assert!(!conflict.field.is_empty());
        assert!(!conflict.resolution.is_empty());
        // At least one value should be present for a conflict
        assert!(conflict.css_value.is_some() || conflict.other_value.is_some());
    }

    // Verify merge results match expected policies
    assert!(merged.contains_key("title")); // Should be present
    assert!(merged.contains_key("tags")); // Should be merged
    assert!(merged.contains_key("author")); // Should use first valid
    assert!(merged.contains_key("date")); // Should use other

    Ok(())
}

/// Test merge policy with real-world content extraction scenario
#[tokio::test]
async fn test_realistic_merge_scenario() -> Result<()> {
    let html = r#"
        <html>
            <head>
                <title>Original Title</title>
                <meta name="description" content="Meta description">
                <meta name="author" content="Meta Author">
                <meta property="og:title" content="OG Title">
            </head>
            <body>
                <article>
                    <h1>Article Headline</h1>
                    <div class="author">Article Author</div>
                    <time datetime="2023-12-15">Dec 15, 2023</time>
                    <div class="content">
                        <p>Article content goes here with substantial text.</p>
                        <p>More content to extract and analyze.</p>
                    </div>
                    <div class="tags">
                        <span class="tag">Technology</span>
                        <span class="tag">Web</span>
                        <span class="tag">Testing</span>
                    </div>
                </article>
            </body>
        </html>
    "#;

    // Extract using CSS selectors
    let css_selectors = create_comprehensive_selectors();
    let css_extractor = CssJsonExtractor::new(css_selectors);
    let css_result = css_extractor.extract(html, "https://example.com").await?;

    // Simulate other extraction method results (e.g., from regex or other strategy)
    let mut other_results = HashMap::new();
    other_results.insert(
        "title".to_string(),
        vec!["Regex Extracted Title".to_string()],
    );
    other_results.insert("author".to_string(), vec!["Regex Author".to_string()]);
    other_results.insert(
        "content".to_string(),
        vec!["Regex content extraction".to_string()],
    );
    other_results.insert("summary".to_string(), vec!["Generated summary".to_string()]);
    other_results.insert(
        "tags".to_string(),
        vec!["regex".to_string(), "extraction".to_string()],
    );

    // Create CSS results from extraction
    let mut css_results = HashMap::new();
    css_results.insert("title".to_string(), vec![css_result.title.clone()]);
    css_results.insert("content".to_string(), vec![css_result.content.clone()]);
    if let Some(summary) = css_result.summary {
        css_results.insert("summary".to_string(), vec![summary]);
    }

    // Merge with CSS-wins policy
    let css_wins_extractor = CssJsonExtractor::new(create_comprehensive_selectors())
        .with_merge_policy(MergePolicy::CssWins);

    let (merged, conflicts) = css_wins_extractor.merge_with_other(&css_results, &other_results);

    // Analyze merge results
    println!("Realistic merge scenario results:");
    println!(
        "  CSS-extracted title: '{}'",
        css_results.get("title").unwrap().first().unwrap()
    );
    println!(
        "  Other-extracted title: '{}'",
        other_results.get("title").unwrap().first().unwrap()
    );
    println!(
        "  Merged title: '{}'",
        merged.get("title").unwrap().first().unwrap()
    );
    println!("  Conflicts: {}", conflicts.len());

    // With CSS-wins policy, CSS results should take precedence
    assert_eq!(
        merged.get("title").unwrap().first().unwrap(),
        css_results.get("title").unwrap().first().unwrap()
    );

    // Should have comprehensive conflict audit
    assert!(!conflicts.is_empty());

    for conflict in conflicts {
        match conflict.policy_used {
            MergePolicy::CssWins => {
                assert_eq!(conflict.resolution, "CSS wins");
            }
            MergePolicy::Merge => {
                assert_eq!(conflict.resolution, "Merged both");
            }
            MergePolicy::FirstValid => {
                assert!(conflict.resolution.contains("First valid"));
            }
            MergePolicy::OtherWins => {
                assert_eq!(conflict.resolution, "Other wins");
            }
        }
    }

    Ok(())
}

/// Test field coverage calculation accuracy
#[tokio::test]
async fn test_field_coverage_calculation() -> Result<()> {
    // Test different coverage scenarios
    let test_cases = vec![
        // High coverage scenario
        (
            create_high_coverage_html(),
            "High coverage test",
            0.8, // Expected minimum coverage
        ),
        // Medium coverage scenario
        (
            create_medium_coverage_html(),
            "Medium coverage test",
            0.6, // Expected minimum coverage
        ),
        // Low coverage scenario
        (
            create_low_coverage_html(),
            "Low coverage test",
            0.3, // Expected minimum coverage
        ),
    ];

    for (html, description, expected_min_coverage) in test_cases {
        let selectors = default_selectors();
        let extractor = CssJsonExtractor::new(selectors);
        let result = extractor.extract(&html, "https://example.com").await?;

        // Calculate field coverage
        let coverage = calculate_field_coverage(&result);

        println!("{}: Coverage = {:.1}%", description, coverage * 100.0);
        println!(
            "  Title: '{}' (length: {})",
            result.title,
            result.title.len()
        );
        println!("  Content length: {}", result.content.len());
        println!("  Summary: {:?}", result.summary);
        println!("  Confidence: {:.3}", result.extraction_confidence);
        println!();

        assert!(
            coverage >= expected_min_coverage,
            "{} coverage {:.1}% should be >= {:.1}%",
            description,
            coverage * 100.0,
            expected_min_coverage * 100.0
        );
    }

    Ok(())
}

/// Test comprehensive performance with all features enabled
#[tokio::test]
async fn test_comprehensive_performance() -> Result<()> {
    let large_html = create_complex_performance_html();

    // Create comprehensive selector configuration using all features
    let mut selectors = HashMap::new();

    // Complex selector with multiple transformers
    selectors.insert(
        "processed_title".to_string(),
        CssSelectorConfig {
            selector: "article header h1.main-title".to_string(),
            transformers: vec![
                "trim".to_string(),
                "html_decode".to_string(),
                "normalize_ws".to_string(),
                "lowercase".to_string(),
            ],
            has_text_filter: Some(HasTextFilter {
                pattern: r"(?i)\b(article|post|news)\b".to_string(),
                case_insensitive: true,
                partial_match: true,
                regex_mode: true,
                regex: None,
            }),
            fallbacks: vec!["h1".to_string(), ".title".to_string()],
            required: true,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    // Complex content extraction with regex filtering
    selectors.insert(
        "important_content".to_string(),
        CssSelectorConfig {
            selector: ".content p, .article-body p".to_string(),
            transformers: vec!["regex_replace".to_string(), "trim".to_string()],
            has_text_filter: Some(HasTextFilter {
                pattern: r"(?i)\b(important|critical|key|main)\b".to_string(),
                case_insensitive: true,
                partial_match: true,
                regex_mode: true,
                regex: None,
            }),
            fallbacks: vec!["p".to_string()],
            required: false,
            merge_policy: Some(MergePolicy::Merge),
        },
    );

    // Price extraction with currency transformation
    selectors.insert(
        "prices".to_string(),
        CssSelectorConfig {
            selector: "[data-price], .price, .cost".to_string(),
            transformers: vec!["currency".to_string()],
            has_text_filter: Some(HasTextFilter {
                pattern: r"[$€£¥]\d+".to_string(),
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

    // Complex tag processing
    selectors.insert(
        "processed_tags".to_string(),
        CssSelectorConfig {
            selector: ".tag, .category, .keyword".to_string(),
            transformers: vec![
                "trim".to_string(),
                "lowercase".to_string(),
                "split".to_string(),
                "join".to_string(),
            ],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::Merge),
        },
    );

    let start = std::time::Instant::now();
    let extractor = CssJsonExtractor::new(selectors);
    let result = extractor
        .extract(&large_html, "https://example.com")
        .await?;
    let duration = start.elapsed();

    println!("Comprehensive performance test:");
    println!("  - HTML size: {} chars", large_html.len());
    println!("  - Processing time: {:?}", duration);
    println!("  - Extracted content: {} chars", result.content.len());
    println!("  - Confidence score: {:.3}", result.extraction_confidence);
    println!("  - Title: '{}'", result.title);

    // Should meet performance targets
    assert!(
        duration.as_millis() < 200,
        "Should process in <200ms, took {:?}",
        duration
    );
    assert!(!result.title.is_empty());
    assert!(result.content.len() > 100);
    assert!(result.extraction_confidence > 0.5);

    Ok(())
}

// Helper functions for test data generation

fn generate_large_result_set(prefix: &str, size: usize) -> HashMap<String, Vec<String>> {
    let mut results = HashMap::new();

    for i in 0..size {
        let field_name = format!("field_{}", i);
        let values = vec![
            format!("{}_value_{}_1", prefix, i),
            format!("{}_value_{}_2", prefix, i),
        ];
        results.insert(field_name, values);
    }

    results
}

fn create_test_selectors() -> HashMap<String, CssSelectorConfig> {
    let mut selectors = HashMap::new();

    selectors.insert(
        "test_field".to_string(),
        CssSelectorConfig {
            selector: ".test".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    selectors
}

fn create_conflicting_css_results() -> HashMap<String, Vec<String>> {
    let mut results = HashMap::new();
    results.insert("title".to_string(), vec!["CSS Title".to_string()]);
    results.insert("author".to_string(), vec!["CSS Author".to_string()]);
    results.insert(
        "tags".to_string(),
        vec!["css".to_string(), "web".to_string()],
    );
    results.insert("date".to_string(), vec!["2023-12-15".to_string()]);
    results
}

fn create_conflicting_other_results() -> HashMap<String, Vec<String>> {
    let mut results = HashMap::new();
    results.insert("title".to_string(), vec!["Other Title".to_string()]);
    results.insert("author".to_string(), vec!["Other Author".to_string()]);
    results.insert(
        "tags".to_string(),
        vec!["regex".to_string(), "extraction".to_string()],
    );
    results.insert("date".to_string(), vec!["2023-12-16".to_string()]);
    results
}

fn create_comprehensive_selectors() -> HashMap<String, CssSelectorConfig> {
    let mut selectors = HashMap::new();

    selectors.insert(
        "title".to_string(),
        CssSelectorConfig {
            selector: "h1, title".to_string(),
            transformers: vec!["trim".to_string(), "normalize_ws".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: true,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    selectors.insert(
        "content".to_string(),
        CssSelectorConfig {
            selector: "article, .content".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec!["p".to_string()],
            required: true,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    selectors.insert(
        "tags".to_string(),
        CssSelectorConfig {
            selector: ".tag".to_string(),
            transformers: vec!["lowercase".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::Merge),
        },
    );

    selectors
}

fn create_high_coverage_html() -> String {
    r#"
        <html>
            <head>
                <title>Complete Article Title</title>
                <meta name="description" content="Comprehensive article description with details">
                <meta name="author" content="Article Author">
            </head>
            <body>
                <article>
                    <h1>Main Article Headline</h1>
                    <div class="author">Author Name</div>
                    <time>2023-12-15</time>
                    <div class="content">
                        <p>Substantial article content with multiple paragraphs of meaningful information.</p>
                        <p>Additional content that provides value and context to readers.</p>
                        <p>More detailed information covering the topic thoroughly.</p>
                    </div>
                    <div class="tags">
                        <span class="tag">Technology</span>
                        <span class="tag">Web</span>
                        <span class="tag">Development</span>
                    </div>
                </article>
            </body>
        </html>
    "#.to_string()
}

fn create_medium_coverage_html() -> String {
    r#"
        <html>
            <head>
                <title>Partial Article</title>
            </head>
            <body>
                <h1>Article Title</h1>
                <div class="content">
                    <p>Some content here but not comprehensive.</p>
                </div>
            </body>
        </html>
    "#
    .to_string()
}

fn create_low_coverage_html() -> String {
    r#"
        <html>
            <body>
                <div>
                    <span>Minimal content</span>
                </div>
            </body>
        </html>
    "#
    .to_string()
}

fn create_complex_performance_html() -> String {
    let mut html = String::from(
        r#"
        <html>
            <head>
                <title>Performance Test Document</title>
                <meta name="description" content="Large document for performance testing">
            </head>
            <body>
                <main class="content">
    "#,
    );

    // Generate complex nested structure
    for i in 0..200 {
        html.push_str(&format!(r#"
            <article class="post" data-id="{}">
                <header class="post-header">
                    <h1 class="main-title">Important Article Title {}</h1>
                    <div class="post-meta">
                        <span class="author">Author {}</span>
                        <time class="date">2023-12-{:02}</time>
                    </div>
                </header>
                <div class="article-body">
                    <p class="content">This is critical content for article {} with important information.</p>
                    <div class="special" data-price="${}.99">Price information here.</div>
                    <p>Additional key details that are main points of the article.</p>
                    <div class="tags">
                        <span class="tag">tech, web, important</span>
                        <span class="category">Technology</span>
                    </div>
                </div>
            </article>
        "#, i, i, i % 10, (i % 28) + 1, i, (i * 10) + 199));
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

fn calculate_field_coverage(result: &riptide_extraction::ExtractedContent) -> f64 {
    let mut filled_fields = 0.0;
    let total_fields = 3.0; // title, content, summary

    // Check title (non-empty and not default)
    if !result.title.is_empty() && result.title != "Untitled" && result.title.len() > 5 {
        filled_fields += 1.0;
    }

    // Check content (substantial content)
    if !result.content.is_empty() && result.content.len() > 100 {
        filled_fields += 1.0;
    }

    // Check summary (meaningful summary)
    if let Some(summary) = &result.summary {
        if !summary.is_empty() && summary.len() > 20 {
            filled_fields += 1.0;
        }
    }

    filled_fields / total_fields
}
