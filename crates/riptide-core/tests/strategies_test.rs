//! Comprehensive tests for extraction strategies and chunking modes

use anyhow::Result;
use riptide_core::strategies::chunking::ChunkingMode;
use riptide_core::strategies::*;
use std::collections::HashMap;

#[tokio::test]
async fn test_trek_extraction() -> Result<()> {
    let html = r#"
        <html>
        <head>
            <title>Test Article</title>
            <meta name="description" content="Test description">
            <meta name="author" content="John Doe">
        </head>
        <body>
            <article>
                <h1>Main Title</h1>
                <p>This is the main content of the article.</p>
                <p>It contains multiple paragraphs with meaningful content.</p>
            </article>
        </body>
        </html>
    "#;

    let result = extraction::trek::extract(html, "http://example.com").await?;

    assert!(!result.title.is_empty());
    assert!(!result.content.is_empty());
    assert_eq!(result.strategy_used, "trek_fallback");
    assert!(result.extraction_confidence > 0.0);

    Ok(())
}

#[tokio::test]
async fn test_css_json_extraction() -> Result<()> {
    let html = r#"
        <html>
        <head>
            <title>CSS Test Article</title>
            <meta name="description" content="CSS test description">
        </head>
        <body>
            <article class="content">
                <h1 class="title">CSS Title</h1>
                <div class="author">Jane Smith</div>
                <p>Content paragraph one.</p>
                <p>Content paragraph two.</p>
            </article>
        </body>
        </html>
    "#;

    let selectors = extraction::css_json::default_selectors();
    let result = extraction::css_json::extract(html, "http://example.com", &selectors).await?;

    assert!(result.title.contains("CSS"));
    assert!(!result.content.is_empty());
    assert_eq!(result.strategy_used, "css_json");
    assert!(result.extraction_confidence > 0.0);

    Ok(())
}

#[tokio::test]
async fn test_regex_extraction() -> Result<()> {
    let html = r#"
        <html>
        <body>
            <h1>Regex Test Article</h1>
            <p>Contact us at test@example.com or call (555) 123-4567</p>
            <p>Visit our website at https://example.com</p>
            <p>Published on 2023-12-01</p>
        </body>
        </html>
    "#;

    let patterns = extraction::regex::default_patterns();
    let result = extraction::regex::extract(html, "http://example.com", &patterns).await?;

    assert!(!result.content.is_empty());
    assert_eq!(result.strategy_used, "regex");
    assert!(result.extraction_confidence > 0.0);

    Ok(())
}

#[tokio::test]
async fn test_sliding_chunking() -> Result<()> {
    let content = "This is a test content. ".repeat(100); // Create substantial content
    let config = ChunkingConfig {
        mode: ChunkingMode::Sliding,
        token_max: 50,
        overlap: 10,
        preserve_sentences: true,
        deterministic: true,
    };

    let chunks = chunking::chunk_content(&content, &config).await?;

    assert!(!chunks.is_empty());

    // Test deterministic behavior
    let chunks2 = chunking::chunk_content(&content, &config).await?;
    assert_eq!(chunks.len(), chunks2.len());

    // Test overlap
    if chunks.len() > 1 {
        assert!(chunks[1].start_pos < chunks[0].end_pos);
    }

    // Test chunk metadata
    for chunk in &chunks {
        assert!(!chunk.id.is_empty());
        assert!(chunk.token_count > 0);
        assert_eq!(chunk.metadata.chunk_type, "sliding");
    }

    Ok(())
}

#[tokio::test]
async fn test_sentence_chunking() -> Result<()> {
    let content = "First sentence. Second sentence! Third sentence? Fourth sentence. Fifth sentence. Sixth sentence.";
    let config = ChunkingConfig {
        mode: ChunkingMode::Sentence { max_sentences: 2 },
        token_max: 1000,
        overlap: 0,
        preserve_sentences: true,
        deterministic: true,
    };

    let chunks = chunking::sentence::chunk_by_sentences(content, 2, &config).await?;

    assert!(!chunks.is_empty());

    for chunk in &chunks {
        assert_eq!(chunk.metadata.chunk_type, "sentence");
        assert!(chunk.metadata.has_complete_sentences);
        assert!(chunk.metadata.sentence_count <= 2);
    }

    Ok(())
}

#[tokio::test]
async fn test_fixed_chunking() -> Result<()> {
    let content = "Word ".repeat(200); // Create content with known word count
    let config = ChunkingConfig {
        mode: ChunkingMode::Fixed {
            size: 100,
            by_tokens: false,
        },
        token_max: 1000,
        overlap: 0,
        preserve_sentences: false,
        deterministic: true,
    };

    let chunks = chunking::fixed::chunk_fixed_size(&content, 100, false, &config).await?;

    assert!(!chunks.is_empty());

    for chunk in &chunks {
        assert!(chunk.content.len() <= 100 || chunk.chunk_index == chunks.len() - 1); // Last chunk can be longer
        assert!(chunk.metadata.chunk_type.starts_with("fixed_char"));
    }

    Ok(())
}

#[tokio::test]
async fn test_regex_chunking() -> Result<()> {
    let content = "Section 1\n\nContent for section 1.\n\nSection 2\n\nContent for section 2.\n\nSection 3\n\nContent for section 3.";
    let pattern = r"\n\s*\n";
    let min_chunk_size = 10;
    let config = ChunkingConfig::default();

    let chunks = chunking::regex::chunk_by_regex(content, pattern, min_chunk_size, &config).await?;

    assert!(!chunks.is_empty());

    for chunk in &chunks {
        assert!(chunk.content.len() >= min_chunk_size || chunks.len() == 1);
        assert!(chunk.metadata.chunk_type.starts_with("regex_"));
    }

    Ok(())
}

#[tokio::test]
async fn test_topic_chunking() -> Result<()> {
    let content = "Machine learning is fascinating. Artificial intelligence and neural networks are advancing rapidly. Deep learning models show great promise. Meanwhile, cooking recipes require different skills. Baking bread involves yeast and flour. Culinary arts focus on flavor combinations.";

    let similarity_threshold = 0.3;
    let config = ChunkingConfig::default();

    let chunks = chunking::topic::chunk_by_topics(content, similarity_threshold, &config).await?;

    assert!(!chunks.is_empty());

    for chunk in &chunks {
        assert!(chunk.metadata.chunk_type.starts_with("topic_"));
        assert!(!chunk.metadata.topic_keywords.is_empty());
    }

    Ok(())
}

#[tokio::test]
async fn test_metadata_extraction() -> Result<()> {
    let html = r#"
        <html>
        <head>
            <title>Complete Test Article</title>
            <meta name="description" content="Comprehensive test for metadata extraction">
            <meta name="author" content="Test Author">
            <meta name="keywords" content="test, extraction, metadata">
            <meta property="og:title" content="OG Test Article">
            <meta property="og:description" content="OG test description">
            <meta property="og:image" content="https://example.com/image.jpg">
            <meta property="article:published_time" content="2023-12-01T10:00:00Z">
            <meta property="article:author" content="OG Author">
            <link rel="canonical" href="https://example.com/article">
        </head>
        <body>
            <script type="application/ld+json">
            {
                "@context": "https://schema.org",
                "@type": "Article",
                "headline": "JSON-LD Test Article",
                "description": "JSON-LD test description",
                "author": {
                    "@type": "Person",
                    "name": "JSON-LD Author"
                },
                "datePublished": "2023-12-01T10:00:00Z",
                "keywords": ["json-ld", "test", "article"]
            }
            </script>
            <article>
                <h1>Article Title</h1>
                <p class="byline">By Article Author</p>
                <time datetime="2023-12-01">December 1, 2023</time>
                <p>Article content goes here.</p>
            </article>
        </body>
        </html>
    "#;

    let metadata = metadata::extract_metadata(html, "https://example.com/test").await?;

    // Test that we extracted metadata from multiple sources
    assert!(metadata.title.is_some());
    assert!(metadata.description.is_some());
    assert!(metadata.author.is_some());
    assert!(metadata.published_date.is_some());
    assert!(!metadata.keywords.is_empty());
    assert!(metadata.canonical_url.is_some());

    // Test confidence scores
    assert!(metadata.confidence_scores.overall > 0.5);
    assert!(metadata.confidence_scores.title > 0.0);
    assert!(metadata.confidence_scores.author > 0.0);

    // Test extraction methods used
    assert!(
        metadata.extraction_method.open_graph
            || metadata.extraction_method.json_ld
            || metadata.extraction_method.meta_tags
            || metadata.extraction_method.heuristics
    );

    Ok(())
}

#[tokio::test]
async fn test_byline_extraction_accuracy() -> Result<()> {
    let test_cases = vec![
        (
            r#"<meta property="article:author" content="John Smith">"#,
            "John Smith",
        ),
        (r#"<div class="byline">By Jane Doe</div>"#, "Jane Doe"),
        (r#"<span class="author">Bob Wilson</span>"#, "Bob Wilson"),
        (
            r#"<meta name="author" content="Alice Johnson">"#,
            "Alice Johnson",
        ),
    ];

    let mut correct_extractions = 0;

    for (html_snippet, expected_author) in &test_cases {
        let full_html = format!(
            r#"
            <html>
            <head><title>Test</title></head>
            <body>{}</body>
            </html>
        "#,
            html_snippet
        );

        let metadata = metadata::extract_metadata(&full_html, "http://example.com").await?;

        if let Some(extracted_author) = metadata.author {
            if extracted_author.contains(*expected_author) {
                correct_extractions += 1;
            }
        }
    }

    let accuracy = correct_extractions as f64 / test_cases.len() as f64;
    assert!(
        accuracy >= 0.8,
        "Byline extraction accuracy: {:.2}% (required: 80%)",
        accuracy * 100.0
    );

    Ok(())
}

#[tokio::test]
async fn test_date_extraction_accuracy() -> Result<()> {
    let test_cases = vec![
        (
            r#"<meta property="article:published_time" content="2023-12-01T10:00:00Z">"#,
            "2023-12-01",
        ),
        (
            r#"<time datetime="2023-11-15">November 15, 2023</time>"#,
            "2023-11-15",
        ),
        (r#"<div class="date">2023-10-20</div>"#, "2023-10-20"),
        (
            r#"<span class="published">January 5, 2024</span>"#,
            "2024-01-05",
        ),
    ];

    let mut correct_extractions = 0;

    for (html_snippet, expected_date) in &test_cases {
        let full_html = format!(
            r#"
            <html>
            <head><title>Test</title></head>
            <body>{}</body>
            </html>
        "#,
            html_snippet
        );

        let metadata = metadata::extract_metadata(&full_html, "http://example.com").await?;

        if let Some(extracted_date) = metadata.published_date {
            let date_str = extracted_date.format("%Y-%m-%d").to_string();
            if date_str == *expected_date {
                correct_extractions += 1;
            }
        }
    }

    let accuracy = correct_extractions as f64 / test_cases.len() as f64;
    assert!(
        accuracy >= 0.8,
        "Date extraction accuracy: {:.2}% (required: 80%)",
        accuracy * 100.0
    );

    Ok(())
}

#[tokio::test]
async fn test_strategy_manager() -> Result<()> {
    let config = StrategyConfig::default();
    let mut manager = StrategyManager::new(config);

    let html = r#"
        <html>
        <head><title>Test Article</title></head>
        <body>
            <article>
                <h1>Test Content</h1>
                <p>This is test content for the strategy manager.</p>
            </article>
        </body>
        </html>
    "#;

    let result = manager
        .extract_and_chunk(html, "http://example.com")
        .await?;

    assert!(!result.extracted.title.is_empty());
    assert!(!result.extracted.content.is_empty());
    assert!(!result.chunks.is_empty());
    assert!(result.metadata.title.is_some());
    assert!(result.metrics.is_some());

    // Test metrics
    let metrics = manager.get_metrics();
    assert!(metrics.total_extractions > 0);

    Ok(())
}

#[tokio::test]
async fn test_performance_metrics() -> Result<()> {
    let mut metrics = PerformanceMetrics::new();

    // Record some test extractions
    metrics.record_extraction(
        &ExtractionStrategy::Trek,
        std::time::Duration::from_millis(100),
        1000,
        5,
    );

    metrics.record_extraction(
        &ExtractionStrategy::CssJson {
            selectors: HashMap::new(),
        },
        std::time::Duration::from_millis(150),
        1200,
        6,
    );

    let summary = metrics.get_summary();
    assert_eq!(summary.total_extractions, 2);
    assert!(summary.total_time.as_millis() > 0);
    assert!(!summary.best_performing_strategy.is_empty());

    // Test CSV export
    let csv = metrics.export_csv()?;
    assert!(csv.contains("Strategy,Runs,AvgDuration"));

    Ok(())
}

#[tokio::test]
async fn test_deterministic_chunking() -> Result<()> {
    let content = "Sentence one. Sentence two. Sentence three. Sentence four. Sentence five.";
    let config = ChunkingConfig {
        mode: ChunkingMode::Sliding,
        token_max: 30,
        overlap: 5,
        preserve_sentences: true,
        deterministic: true,
    };

    // Run chunking multiple times
    let chunks1 = chunking::chunk_content(content, &config).await?;
    let chunks2 = chunking::chunk_content(content, &config).await?;
    let chunks3 = chunking::chunk_content(content, &config).await?;

    // Results should be identical
    assert_eq!(chunks1.len(), chunks2.len());
    assert_eq!(chunks2.len(), chunks3.len());

    for i in 0..chunks1.len() {
        assert_eq!(chunks1[i].content, chunks2[i].content);
        assert_eq!(chunks2[i].content, chunks3[i].content);
        assert_eq!(chunks1[i].start_pos, chunks2[i].start_pos);
        assert_eq!(chunks1[i].end_pos, chunks2[i].end_pos);
    }

    Ok(())
}

#[tokio::test]
async fn test_chunk_quality_scoring() -> Result<()> {
    let high_quality_content = "This is a well-formed sentence with good structure. It contains meaningful content and proper punctuation. The text flows naturally and provides valuable information.";

    let low_quality_content = "short text";

    let config = ChunkingConfig::default();

    let high_chunks = chunking::chunk_content(high_quality_content, &config).await?;
    let low_chunks = chunking::chunk_content(low_quality_content, &config).await?;

    // High quality content should have better scores
    let high_score = high_chunks[0].metadata.quality_score;
    let low_score = low_chunks[0].metadata.quality_score;

    assert!(
        high_score > low_score,
        "High quality: {}, Low quality: {}",
        high_score,
        low_score
    );

    Ok(())
}

#[tokio::test]
async fn test_token_counting() -> Result<()> {
    let test_text = "This is a test sentence with exactly ten words here.";
    let token_count = chunking::count_tokens(test_text);

    // Should be roughly 10-13 tokens (words + some overhead)
    assert!(
        token_count >= 8 && token_count <= 15,
        "Token count: {}",
        token_count
    );

    Ok(())
}

#[tokio::test]
async fn test_topic_keyword_extraction() -> Result<()> {
    let content = "Machine learning algorithms utilize artificial intelligence to process data efficiently. Neural networks and deep learning models demonstrate remarkable performance in pattern recognition.";

    let keywords = chunking::extract_topic_keywords(content);

    assert!(!keywords.is_empty());
    assert!(keywords.len() <= 5); // Should limit to top 5

    // Should contain relevant technical terms
    let keywords_str = keywords.join(" ").to_lowercase();
    assert!(
        keywords_str.contains("machine")
            || keywords_str.contains("learning")
            || keywords_str.contains("neural")
    );

    Ok(())
}
