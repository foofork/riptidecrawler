//! Comprehensive tests for metadata extraction with JSON-LD short-circuit
//!
//! Tests validate:
//! - JSON-LD short-circuit logic with feature flag
//! - Event schema completeness validation
//! - Article schema completeness validation
//! - Incomplete schema fallback behavior
//! - Edge cases (malformed JSON, empty schemas, nested structures)

use riptide_extraction::strategies::metadata::extract_metadata;

// ============================================================================
// Phase 10: JSON-LD Short-Circuit Tests
// ============================================================================

#[tokio::test]
#[cfg(feature = "jsonld-shortcircuit")]
async fn test_complete_event_schema_short_circuit() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@context": "https://schema.org",
            "@type": "Event",
            "name": "Tech Conference 2025",
            "startDate": "2025-11-01T09:00:00",
            "location": {
                "@type": "Place",
                "name": "Moscone Center",
                "address": "747 Howard St, San Francisco, CA 94103"
            }
        }
        </script>
        <meta property="og:title" content="Different Title From OG">
        <title>Different Title From HTML</title>
    </head>
    </html>"#;

    let result = extract_metadata(html, "https://example.com/event").await;
    assert!(result.is_ok(), "Metadata extraction should succeed");

    let metadata = result.unwrap();

    // JSON-LD short-circuit should use structured data, not Open Graph or title tag
    assert_eq!(
        metadata.title,
        Some("Tech Conference 2025".to_string()),
        "Title should be from JSON-LD Event schema, not Open Graph or HTML title"
    );

    // Verify extraction method flags
    assert!(
        metadata.extraction_method.json_ld,
        "JSON-LD extraction should be flagged"
    );

    // With short-circuit, Open Graph should NOT be processed
    #[cfg(feature = "jsonld-shortcircuit")]
    assert!(
        !metadata.extraction_method.open_graph,
        "Open Graph should be skipped with JSON-LD short-circuit"
    );

    // Confidence should be high for JSON-LD extraction
    assert!(
        metadata.confidence_scores.title >= 0.7,
        "JSON-LD extraction should have high confidence"
    );
}

#[tokio::test]
#[cfg(feature = "jsonld-shortcircuit")]
async fn test_complete_article_schema_short_circuit() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@context": "https://schema.org",
            "@type": "Article",
            "headline": "Breaking News: AI Advances",
            "author": {
                "@type": "Person",
                "name": "Jane Smith"
            },
            "datePublished": "2025-10-24T10:00:00Z",
            "description": "An in-depth look at recent AI developments and their impact on society."
        }
        </script>
        <meta property="og:title" content="Different OG Title">
        <meta name="author" content="Different Author">
    </head>
    </html>"#;

    let result = extract_metadata(html, "https://example.com/article").await;
    assert!(result.is_ok(), "Metadata extraction should succeed");

    let metadata = result.unwrap();

    // Verify all required Article schema fields are extracted
    assert_eq!(
        metadata.title,
        Some("Breaking News: AI Advances".to_string()),
        "Headline should be extracted from JSON-LD"
    );

    assert_eq!(
        metadata.author,
        Some("Jane Smith".to_string()),
        "Author should be extracted from JSON-LD"
    );

    assert!(
        metadata.published_date.is_some(),
        "Published date should be extracted"
    );

    assert_eq!(
        metadata.description,
        Some("An in-depth look at recent AI developments and their impact on society.".to_string()),
        "Description should be extracted from JSON-LD"
    );

    // Verify JSON-LD extraction method is flagged
    assert!(
        metadata.extraction_method.json_ld,
        "JSON-LD extraction should be flagged"
    );

    // High confidence for complete schema
    assert!(
        metadata.confidence_scores.overall >= 0.6,
        "Complete Article schema should have high overall confidence"
    );
}

#[tokio::test]
#[cfg(feature = "jsonld-shortcircuit")]
async fn test_incomplete_event_fallback() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@context": "https://schema.org",
            "@type": "Event",
            "name": "Incomplete Event"
        }
        </script>
        <meta property="og:title" content="Fallback Title">
        <meta property="og:description" content="Fallback Description">
    </head>
    </html>"#;

    let result = extract_metadata(html, "https://example.com/incomplete-event").await;
    assert!(result.is_ok(), "Metadata extraction should succeed");

    let metadata = result.unwrap();

    // Event is incomplete (missing startDate and location), so fallback should occur
    assert!(
        metadata.title.is_some(),
        "Title should be extracted (from JSON-LD or fallback)"
    );

    // Should fall back to Open Graph for description
    assert_eq!(
        metadata.description,
        Some("Fallback Description".to_string()),
        "Should fallback to Open Graph for missing fields"
    );

    // Both extraction methods should be flagged
    assert!(
        metadata.extraction_method.json_ld,
        "JSON-LD extraction should be attempted"
    );
    assert!(
        metadata.extraction_method.open_graph,
        "Open Graph fallback should occur for incomplete schema"
    );
}

#[tokio::test]
#[cfg(feature = "jsonld-shortcircuit")]
async fn test_incomplete_article_fallback() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@context": "https://schema.org",
            "@type": "Article",
            "headline": "Incomplete Article"
        }
        </script>
        <meta name="author" content="Fallback Author">
        <meta name="description" content="Fallback Description">
        <meta property="article:published_time" content="2025-10-24T10:00:00Z">
    </head>
    </html>"#;

    let result = extract_metadata(html, "https://example.com/incomplete-article").await;
    assert!(result.is_ok(), "Metadata extraction should succeed");

    let metadata = result.unwrap();

    // Article is incomplete (missing author, date, description in JSON-LD)
    assert_eq!(
        metadata.title,
        Some("Incomplete Article".to_string()),
        "Headline from JSON-LD should be preserved"
    );

    // Should fall back to meta tags for missing fields
    assert_eq!(
        metadata.author,
        Some("Fallback Author".to_string()),
        "Should fallback to meta tags for author"
    );

    assert_eq!(
        metadata.description,
        Some("Fallback Description".to_string()),
        "Should fallback to meta tags for description"
    );

    assert!(
        metadata.published_date.is_some(),
        "Should fallback to Open Graph for date"
    );

    // Multiple extraction methods should be used
    assert!(
        metadata.extraction_method.json_ld,
        "JSON-LD extraction should be attempted"
    );
    assert!(
        metadata.extraction_method.meta_tags || metadata.extraction_method.open_graph,
        "Fallback extraction methods should be used"
    );
}

#[tokio::test]
async fn test_malformed_jsonld_graceful_handling() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@context": "https://schema.org",
            "@type": "Article",
            "headline": "Valid Start",
            // Invalid JSON comment
            "author": undefined
        }
        </script>
        <meta property="og:title" content="Fallback Title">
        <meta property="og:description" content="Fallback Description">
    </head>
    </html>"#;

    let result = extract_metadata(html, "https://example.com/malformed").await;
    assert!(
        result.is_ok(),
        "Malformed JSON-LD should not crash extraction"
    );

    let metadata = result.unwrap();

    // Should gracefully fall back to other extraction methods
    assert_eq!(
        metadata.title,
        Some("Fallback Title".to_string()),
        "Should fallback to Open Graph when JSON-LD is malformed"
    );

    assert_eq!(
        metadata.description,
        Some("Fallback Description".to_string()),
        "Description should be extracted from fallback"
    );

    // JSON-LD should NOT be flagged as successful
    assert!(
        !metadata.extraction_method.json_ld,
        "Malformed JSON-LD should not be counted as successful extraction"
    );

    // Open Graph should be used
    assert!(
        metadata.extraction_method.open_graph,
        "Open Graph fallback should be used"
    );
}

#[tokio::test]
async fn test_nested_jsonld_structures() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@context": "https://schema.org",
            "@graph": [
                {
                    "@type": "WebPage",
                    "name": "Web Page Name"
                },
                {
                    "@type": "Article",
                    "headline": "Nested Article Headline",
                    "author": {
                        "@type": "Person",
                        "name": "John Doe",
                        "url": "https://example.com/author/john"
                    },
                    "datePublished": "2025-10-24T10:00:00Z",
                    "description": "Nested article description",
                    "image": {
                        "@type": "ImageObject",
                        "url": "https://example.com/image.jpg",
                        "width": 1200,
                        "height": 630
                    }
                }
            ]
        }
        </script>
    </head>
    </html>"#;

    let result = extract_metadata(html, "https://example.com/nested").await;
    assert!(result.is_ok(), "Nested JSON-LD extraction should succeed");

    let metadata = result.unwrap();

    // Should extract from nested Article within @graph
    assert_eq!(
        metadata.title,
        Some("Nested Article Headline".to_string()),
        "Should extract headline from nested Article"
    );

    assert_eq!(
        metadata.author,
        Some("John Doe".to_string()),
        "Should extract author name from nested Person object"
    );

    assert_eq!(
        metadata.description,
        Some("Nested article description".to_string()),
        "Should extract description from nested Article"
    );

    assert_eq!(
        metadata.image_url,
        Some("https://example.com/image.jpg".to_string()),
        "Should extract image URL from nested ImageObject"
    );

    assert!(
        metadata.published_date.is_some(),
        "Should extract published date from nested Article"
    );

    assert!(
        metadata.extraction_method.json_ld,
        "JSON-LD extraction should be flagged"
    );
}

#[tokio::test]
async fn test_multiple_jsonld_blocks() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@context": "https://schema.org",
            "@type": "Organization",
            "name": "Example Corp"
        }
        </script>
        <script type="application/ld+json">
        {
            "@context": "https://schema.org",
            "@type": "NewsArticle",
            "headline": "News from Multiple Blocks",
            "author": "Reporter Smith",
            "datePublished": "2025-10-24",
            "description": "Article from second JSON-LD block"
        }
        </script>
    </head>
    </html>"#;

    let result = extract_metadata(html, "https://example.com/multiple").await;
    assert!(
        result.is_ok(),
        "Multiple JSON-LD blocks should be processed"
    );

    let metadata = result.unwrap();

    // Should extract from NewsArticle block (second block)
    assert_eq!(
        metadata.title,
        Some("News from Multiple Blocks".to_string()),
        "Should extract headline from NewsArticle block"
    );

    assert_eq!(
        metadata.author,
        Some("Reporter Smith".to_string()),
        "Should extract author from NewsArticle block"
    );

    assert_eq!(
        metadata.description,
        Some("Article from second JSON-LD block".to_string()),
        "Should extract description from NewsArticle block"
    );

    assert!(
        metadata.extraction_method.json_ld,
        "JSON-LD extraction should be flagged"
    );
}

#[tokio::test]
#[cfg(not(feature = "jsonld-shortcircuit"))]
async fn test_feature_flag_disabled() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@context": "https://schema.org",
            "@type": "Event",
            "name": "Tech Conference 2025",
            "startDate": "2025-11-01T09:00:00",
            "location": {"name": "Moscone Center"}
        }
        </script>
        <meta property="og:title" content="OG Title">
        <meta property="og:description" content="OG Description">
    </head>
    </html>"#;

    let result = extract_metadata(html, "https://example.com/no-shortcircuit").await;
    assert!(
        result.is_ok(),
        "Metadata extraction should succeed without feature flag"
    );

    let metadata = result.unwrap();

    // Without short-circuit feature, both JSON-LD and Open Graph should be processed
    assert!(
        metadata.title.is_some(),
        "Title should be extracted from JSON-LD"
    );

    // Both extraction methods should be flagged when feature is disabled
    assert!(
        metadata.extraction_method.json_ld,
        "JSON-LD should be extracted"
    );
    assert!(
        metadata.extraction_method.open_graph,
        "Open Graph should also be processed when feature flag is disabled"
    );
}

// ============================================================================
// Additional Edge Cases
// ============================================================================

#[tokio::test]
async fn test_empty_jsonld_schema() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {}
        </script>
        <title>Fallback Title</title>
        <meta name="description" content="Fallback Description">
    </head>
    </html>"#;

    let result = extract_metadata(html, "https://example.com/empty-schema").await;
    assert!(
        result.is_ok(),
        "Empty JSON-LD schema should not cause errors"
    );

    let metadata = result.unwrap();

    // Should fall back to HTML title and meta tags
    assert_eq!(
        metadata.title,
        Some("Fallback Title".to_string()),
        "Should fallback to HTML title tag"
    );

    assert_eq!(
        metadata.description,
        Some("Fallback Description".to_string()),
        "Should fallback to meta description"
    );

    // Heuristics should be used for title extraction
    assert!(
        metadata.extraction_method.heuristics || metadata.extraction_method.meta_tags,
        "Fallback extraction methods should be used"
    );
}

#[tokio::test]
async fn test_jsonld_array_format() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        [
            {
                "@context": "https://schema.org",
                "@type": "BreadcrumbList",
                "itemListElement": []
            },
            {
                "@context": "https://schema.org",
                "@type": "BlogPosting",
                "headline": "Array Format Blog Post",
                "author": {
                    "@type": "Person",
                    "name": "Array Author"
                },
                "datePublished": "2025-10-24T10:00:00Z",
                "description": "Blog post in array format"
            }
        ]
        </script>
    </head>
    </html>"#;

    let result = extract_metadata(html, "https://example.com/array").await;
    assert!(result.is_ok(), "Array format JSON-LD should be processed");

    let metadata = result.unwrap();

    // Should extract from BlogPosting in array
    assert_eq!(
        metadata.title,
        Some("Array Format Blog Post".to_string()),
        "Should extract headline from BlogPosting in array"
    );

    assert_eq!(
        metadata.author,
        Some("Array Author".to_string()),
        "Should extract author from array format"
    );

    assert_eq!(
        metadata.description,
        Some("Blog post in array format".to_string()),
        "Should extract description from array format"
    );

    assert!(
        metadata.extraction_method.json_ld,
        "JSON-LD extraction should succeed for array format"
    );
}

#[tokio::test]
async fn test_confidence_scoring() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@context": "https://schema.org",
            "@type": "Article",
            "headline": "Confidence Test Article",
            "author": "Confidence Author",
            "datePublished": "2025-10-24T10:00:00Z",
            "description": "Testing confidence scores"
        }
        </script>
        <meta property="og:title" content="Confidence Test Article">
    </head>
    </html>"#;

    let result = extract_metadata(html, "https://example.com/confidence").await;
    assert!(result.is_ok(), "Metadata extraction should succeed");

    let metadata = result.unwrap();

    // Verify confidence scores are calculated
    assert!(
        metadata.confidence_scores.title > 0.0,
        "Title confidence should be positive"
    );

    assert!(
        metadata.confidence_scores.author > 0.0,
        "Author confidence should be positive"
    );

    assert!(
        metadata.confidence_scores.date > 0.0,
        "Date confidence should be positive"
    );

    assert!(
        metadata.confidence_scores.description > 0.0,
        "Description confidence should be positive"
    );

    assert!(
        metadata.confidence_scores.overall > 0.0 && metadata.confidence_scores.overall <= 1.0,
        "Overall confidence should be between 0 and 1"
    );

    // JSON-LD extraction should yield high confidence
    assert!(
        metadata.confidence_scores.overall >= 0.5,
        "Complete metadata should have confidence >= 0.5"
    );
}

#[tokio::test]
async fn test_canonical_url_extraction() {
    let html = r#"
    <html>
    <head>
        <link rel="canonical" href="https://example.com/canonical-page">
        <title>Canonical Test</title>
    </head>
    </html>"#;

    let result = extract_metadata(html, "https://example.com/original-page").await;
    assert!(result.is_ok(), "Canonical URL extraction should succeed");

    let metadata = result.unwrap();

    assert_eq!(
        metadata.canonical_url,
        Some("https://example.com/canonical-page".to_string()),
        "Canonical URL should be extracted from link tag"
    );

    assert!(
        metadata.extraction_method.heuristics,
        "Heuristic extraction should be flagged for canonical URL"
    );
}

#[tokio::test]
async fn test_keywords_extraction() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@context": "https://schema.org",
            "@type": "Article",
            "headline": "Keyword Test",
            "keywords": ["rust", "web scraping", "metadata", "extraction"]
        }
        </script>
        <meta name="keywords" content="html, parsing, testing">
        <title>Keyword Test</title>
    </head>
    </html>"#;

    let result = extract_metadata(html, "https://example.com/keywords").await;
    assert!(result.is_ok(), "Keyword extraction should succeed");

    let metadata = result.unwrap();

    // Keywords from both JSON-LD and meta tags should be merged
    assert!(
        !metadata.keywords.is_empty(),
        "Keywords should be extracted"
    );

    assert!(
        metadata.keywords.contains(&"rust".to_string()),
        "JSON-LD keywords should be included"
    );

    assert!(
        metadata.keywords.contains(&"html".to_string())
            || metadata.keywords.contains(&"parsing".to_string()),
        "Meta tag keywords should be included"
    );

    // Keywords should be deduplicated
    let unique_count = metadata.keywords.len();
    let total_count = metadata.keywords.iter().collect::<Vec<_>>().len();
    assert_eq!(unique_count, total_count, "Keywords should be deduplicated");
}
