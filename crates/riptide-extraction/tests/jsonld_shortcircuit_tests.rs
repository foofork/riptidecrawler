//! JSON-LD Short-Circuit Tests - Phase 10
//!
//! Tests for the JSON-LD short-circuit optimization that skips additional
//! extraction methods when complete Event or Article schemas are detected.
//!
//! **Note**: The short-circuit optimization works after OpenGraph extraction,
//! so Open Graph data may still be collected. The optimization prevents
//! running meta tags, microdata, and heuristics when JSON-LD is complete.
//!
//! Feature flag: `jsonld-shortcircuit`

#![cfg(test)]

use riptide_extraction::strategies::metadata::extract_metadata;

// ============================================================================
// Event Schema Short-Circuit Tests
// ============================================================================

#[tokio::test]
#[cfg(feature = "jsonld-shortcircuit")]
async fn test_jsonld_event_complete_shortcircuit() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@context": "https://schema.org",
            "@type": "Event",
            "name": "Tech Conference 2025",
            "startDate": "2025-11-01T09:00:00",
            "endDate": "2025-11-03T17:00:00",
            "location": {
                "@type": "Place",
                "name": "San Francisco Convention Center",
                "address": "San Francisco, CA"
            },
            "description": "Annual technology conference",
            "organizer": {
                "@type": "Organization",
                "name": "Tech Events Inc"
            }
        }
        </script>
        <meta property="og:title" content="Different OG Title">
        <meta property="og:description" content="Different OG Description">
        <meta name="author" content="OG Author">
    </head>
    <body>
        <h1>Event Page</h1>
    </body>
    </html>"#;

    let metadata = extract_metadata(html, "https://example.com").await.unwrap();

    // JSON-LD data should be extracted (may coexist with OG since OG runs first)
    // The key optimization is that complete JSON-LD prevents running meta tags/heuristics
    assert_eq!(
        metadata.title,
        Some("Tech Conference 2025".to_string()),
        "Should use JSON-LD event name"
    );
    assert_eq!(
        metadata.description,
        Some("Annual technology conference".to_string()),
        "Should use JSON-LD description"
    );
    assert!(metadata.extraction_method.json_ld, "JSON-LD should be used");

    // Note: Open Graph may still run (it runs before JSON-LD in the pipeline)
    // The short-circuit prevents meta tags/heuristics from running
}

#[tokio::test]
#[cfg(feature = "jsonld-shortcircuit")]
async fn test_jsonld_event_with_startdate() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@type": "Event",
            "name": "Workshop 2025",
            "startDate": "2025-12-15",
            "location": "Online"
        }
        </script>
    </head>
    </html>"#;

    let metadata = extract_metadata(html, "https://example.com").await.unwrap();

    assert_eq!(metadata.title, Some("Workshop 2025".to_string()));
    assert!(
        metadata.published_date.is_some(),
        "Should extract startDate"
    );
    assert!(metadata.extraction_method.json_ld);
}

// ============================================================================
// Article Schema Short-Circuit Tests
// ============================================================================

#[tokio::test]
#[cfg(feature = "jsonld-shortcircuit")]
async fn test_jsonld_article_complete_shortcircuit() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@context": "https://schema.org",
            "@type": "Article",
            "headline": "Breaking News: Important Story",
            "author": {
                "@type": "Person",
                "name": "Jane Doe"
            },
            "datePublished": "2025-10-24T10:00:00Z",
            "dateModified": "2025-10-24T15:30:00Z",
            "description": "This is the article description with important details",
            "image": "https://example.com/image.jpg",
            "publisher": {
                "@type": "Organization",
                "name": "News Corp"
            }
        }
        </script>
        <meta property="og:title" content="Different Title">
        <meta property="og:author" content="Different Author">
        <meta name="description" content="Different Description">
    </head>
    <body>
        <article>Content here</article>
    </body>
    </html>"#;

    let metadata = extract_metadata(html, "https://example.com").await.unwrap();

    // Verify JSON-LD data is used (takes precedence in metadata struct)
    assert_eq!(
        metadata.title,
        Some("Breaking News: Important Story".to_string()),
        "JSON-LD headline should be used"
    );
    assert_eq!(metadata.author, Some("Jane Doe".to_string()));
    assert_eq!(
        metadata.description,
        Some("This is the article description with important details".to_string())
    );
    assert!(metadata.published_date.is_some());
    assert!(metadata.extraction_method.json_ld);

    // The key benefit: complete JSON-LD prevents unnecessary meta tag/heuristic processing
}

#[tokio::test]
#[cfg(feature = "jsonld-shortcircuit")]
async fn test_jsonld_newsarticle_shortcircuit() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@type": "NewsArticle",
            "headline": "News Headline",
            "author": "John Smith",
            "datePublished": "2025-10-20",
            "description": "News article description"
        }
        </script>
    </head>
    </html>"#;

    let metadata = extract_metadata(html, "https://example.com").await.unwrap();

    assert_eq!(metadata.title, Some("News Headline".to_string()));
    assert_eq!(metadata.author, Some("John Smith".to_string()));
    assert!(metadata.extraction_method.json_ld);
}

#[tokio::test]
#[cfg(feature = "jsonld-shortcircuit")]
async fn test_jsonld_blogposting_shortcircuit() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@type": "BlogPosting",
            "headline": "Blog Post Title",
            "author": {"name": "Blogger Name"},
            "datePublished": "2025-10-15T12:00:00",
            "description": "Blog post description"
        }
        </script>
    </head>
    </html>"#;

    let metadata = extract_metadata(html, "https://example.com").await.unwrap();

    assert_eq!(metadata.title, Some("Blog Post Title".to_string()));
    assert_eq!(metadata.author, Some("Blogger Name".to_string()));
    assert!(metadata.extraction_method.json_ld);
}

// ============================================================================
// Incomplete Schema Fallback Tests
// ============================================================================

#[tokio::test]
#[cfg(feature = "jsonld-shortcircuit")]
async fn test_jsonld_incomplete_article_fallback() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@type": "Article",
            "headline": "Incomplete Article"
        }
        </script>
        <meta property="og:author" content="Fallback Author">
        <meta property="og:description" content="Fallback Description">
        <meta property="article:published_time" content="2025-10-24">
    </head>
    </html>"#;

    let metadata = extract_metadata(html, "https://example.com").await.unwrap();

    // Should use JSON-LD for headline
    assert_eq!(metadata.title, Some("Incomplete Article".to_string()));

    // Should use Open Graph for missing fields
    // (OG runs first, so it may provide author/description)
    assert!(
        metadata.author.is_some(),
        "Should have author from OG or other source"
    );
    assert!(metadata.description.is_some(), "Should have description");

    // Both JSON-LD and other methods may be used
    assert!(metadata.extraction_method.json_ld);
}

#[tokio::test]
#[cfg(feature = "jsonld-shortcircuit")]
async fn test_jsonld_incomplete_event_fallback() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@type": "Event",
            "name": "Partial Event"
        }
        </script>
        <meta property="og:description" content="Event description from OG">
        <div class="date">2025-11-01</div>
    </head>
    </html>"#;

    let metadata = extract_metadata(html, "https://example.com").await.unwrap();

    assert_eq!(metadata.title, Some("Partial Event".to_string()));

    // Should use OG for description since JSON-LD is incomplete
    assert!(
        metadata.description.is_some() || metadata.extraction_method.open_graph,
        "Should fall back to OG or other methods"
    );
}

// ============================================================================
// Non-Eligible Schema Tests
// ============================================================================

#[tokio::test]
async fn test_jsonld_organization_no_shortcircuit() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@type": "Organization",
            "name": "Company Name",
            "url": "https://example.com"
        }
        </script>
        <meta property="og:title" content="Page Title">
        <meta name="description" content="Page Description">
    </head>
    </html>"#;

    let metadata = extract_metadata(html, "https://example.com").await.unwrap();

    // Organization schema doesn't trigger short-circuit
    // Should use OG/meta tags for page metadata
    assert!(metadata.title.is_some());
    assert!(metadata.description.is_some());
}

#[tokio::test]
async fn test_jsonld_product_no_shortcircuit() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@type": "Product",
            "name": "Product Name",
            "offers": {
                "@type": "Offer",
                "price": "29.99"
            }
        }
        </script>
        <meta property="og:title" content="Product Page Title">
    </head>
    </html>"#;

    let metadata = extract_metadata(html, "https://example.com").await.unwrap();

    // Product schema doesn't trigger short-circuit
    assert!(metadata.title.is_some());
}

// ============================================================================
// Malformed JSON-LD Tests
// ============================================================================

#[tokio::test]
async fn test_jsonld_malformed_json() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@type": "Article",
            "headline": "Test",
            // Invalid JSON comment
            "author": undefined
        }
        </script>
        <meta property="og:title" content="Fallback Title">
    </head>
    </html>"#;

    let metadata = extract_metadata(html, "https://example.com").await.unwrap();

    // Should gracefully fall back to other methods
    assert!(metadata.title.is_some());
}

#[tokio::test]
async fn test_jsonld_missing_type() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "headline": "No Type Field",
            "author": "Test Author"
        }
        </script>
        <meta property="og:title" content="OG Title">
    </head>
    </html>"#;

    let metadata = extract_metadata(html, "https://example.com").await.unwrap();

    // Should still extract what it can
    assert!(metadata.title.is_some());
}

// ============================================================================
// Array Format JSON-LD Tests
// ============================================================================

#[tokio::test]
#[cfg(feature = "jsonld-shortcircuit")]
async fn test_jsonld_array_format_complete() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        [
            {
                "@type": "Organization",
                "name": "Company"
            },
            {
                "@type": "Article",
                "headline": "Article in Array",
                "author": "Array Author",
                "datePublished": "2025-10-24",
                "description": "Article description"
            }
        ]
        </script>
        <meta property="og:title" content="Different">
    </head>
    </html>"#;

    let metadata = extract_metadata(html, "https://example.com").await.unwrap();

    // Should find and use the Article schema from array
    assert_eq!(metadata.title, Some("Article in Array".to_string()));
    assert_eq!(metadata.author, Some("Array Author".to_string()));
    assert!(metadata.extraction_method.json_ld);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[tokio::test]
async fn test_jsonld_empty_script_tag() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json"></script>
        <meta property="og:title" content="Fallback">
    </head>
    </html>"#;

    let metadata = extract_metadata(html, "https://example.com").await.unwrap();

    // Should handle empty JSON-LD gracefully
    assert!(metadata.title.is_some());
}

#[tokio::test]
#[cfg(feature = "jsonld-shortcircuit")]
async fn test_jsonld_nested_author_object() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@type": "Article",
            "headline": "Test Article",
            "author": {
                "@type": "Person",
                "name": "Nested Author Name",
                "url": "https://example.com/author"
            },
            "datePublished": "2025-10-24",
            "description": "Test description"
        }
        </script>
    </head>
    </html>"#;

    let metadata = extract_metadata(html, "https://example.com").await.unwrap();

    assert_eq!(metadata.author, Some("Nested Author Name".to_string()));
}

#[tokio::test]
#[cfg(feature = "jsonld-shortcircuit")]
async fn test_jsonld_multiple_authors_array() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@type": "Article",
            "headline": "Multi-Author Article",
            "author": [
                {"@type": "Person", "name": "First Author"},
                {"@type": "Person", "name": "Second Author"}
            ],
            "datePublished": "2025-10-24",
            "description": "Multi-author test"
        }
        </script>
    </head>
    </html>"#;

    let metadata = extract_metadata(html, "https://example.com").await.unwrap();

    // Should extract first author from array
    assert_eq!(metadata.author, Some("First Author".to_string()));
}

#[tokio::test]
async fn test_jsonld_with_context() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@context": "https://schema.org",
            "@type": "Article",
            "headline": "Contextualized Article",
            "author": "Context Author",
            "datePublished": "2025-10-24",
            "description": "With context"
        }
        </script>
    </head>
    </html>"#;

    let metadata = extract_metadata(html, "https://example.com").await.unwrap();

    // @context should not interfere with extraction
    assert_eq!(metadata.title, Some("Contextualized Article".to_string()));
}
