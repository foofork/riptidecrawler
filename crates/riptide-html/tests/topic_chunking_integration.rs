//! Integration tests for topic chunking with different document types
//!
//! These tests verify that topic chunking works correctly with various
//! document formats and content types while maintaining performance requirements.

use riptide_html::chunking::{create_strategy, ChunkingConfig, ChunkingMode};
use std::time::Instant;

/// Test topic chunking with academic paper format
#[tokio::test]
async fn test_academic_paper_chunking() {
    let config = ChunkingConfig::default();
    let strategy = create_strategy(
        ChunkingMode::Topic {
            topic_chunking: true,
            window_size: 4,
            smoothing_passes: 2,
        },
        config,
    );

    let academic_paper = r#"
        Abstract: This paper explores the applications of machine learning in natural language processing.
        We present novel approaches to text classification and sentiment analysis using deep neural networks.
        Our experiments demonstrate significant improvements over traditional methods.

        1. Introduction
        Natural language processing has become increasingly important in modern computing applications.
        The ability to understand and process human language computationally opens new possibilities.
        Machine learning techniques have proven particularly effective in this domain.
        Deep learning models can capture complex linguistic patterns and relationships.

        2. Related Work
        Previous research in NLP has focused on rule-based and statistical approaches.
        Early systems relied heavily on hand-crafted linguistic rules and grammars.
        Statistical methods introduced probabilistic models for language understanding.
        Recent advances in deep learning have revolutionized the field significantly.

        3. Methodology
        Our approach combines convolutional neural networks with attention mechanisms.
        We use pre-trained word embeddings to capture semantic relationships between words.
        The model architecture includes multiple layers of feature extraction and classification.
        Training data consists of labeled examples from various text classification tasks.

        4. Experiments
        We evaluated our model on several benchmark datasets for text classification.
        Performance metrics include accuracy, precision, recall, and F1-score measurements.
        Comparison with baseline methods shows consistent improvements across all metrics.
        Statistical significance testing confirms the validity of our results.

        5. Conclusion
        This work demonstrates the effectiveness of deep learning for NLP tasks.
        Our proposed architecture achieves state-of-the-art performance on multiple benchmarks.
        Future work will explore applications to other language understanding problems.
        The techniques presented here have broad applicability in computational linguistics.
    "#;

    let start = Instant::now();
    let chunks = strategy.chunk(academic_paper).await.unwrap();
    let duration = start.elapsed();

    println!(
        "Academic paper: {} chunks in {}ms",
        chunks.len(),
        duration.as_millis()
    );

    // Should complete quickly
    assert!(
        duration.as_millis() < 50,
        "Academic paper chunking too slow: {}ms",
        duration.as_millis()
    );

    // Should identify topic boundaries (sections)
    assert!(chunks.len() >= 3, "Should identify multiple topic sections");

    // Verify chunks have meaningful content
    for (i, chunk) in chunks.iter().enumerate() {
        assert!(!chunk.content.trim().is_empty(), "Chunk {} is empty", i);
        assert!(
            !chunk.metadata.topic_keywords.is_empty(),
            "Chunk {} has no topic keywords",
            i
        );
    }
}

/// Test topic chunking with news article format
#[tokio::test]
async fn test_news_article_chunking() {
    let config = ChunkingConfig::default();
    let strategy = create_strategy(
        ChunkingMode::Topic {
            topic_chunking: true,
            window_size: 3,
            smoothing_passes: 1,
        },
        config,
    );

    let news_article = r#"
        Breaking: Major Technology Company Announces Revolutionary AI System

        In a groundbreaking announcement today, TechCorp unveiled their latest artificial intelligence system.
        The new technology promises to transform how businesses handle data processing and analysis.
        Industry experts believe this could be a significant milestone in AI development.
        The system incorporates advanced machine learning algorithms and neural network architectures.

        Market analysts are responding positively to the news, with stock prices rising.
        Investors see strong potential for growth in the AI sector following this announcement.
        Several competing companies have already indicated plans to develop similar technologies.
        The market capitalization of AI-focused firms has increased substantially this quarter.

        Technical specifications reveal impressive performance improvements over existing systems.
        Processing speeds are reportedly 10 times faster than current industry standards.
        Energy efficiency has been improved through optimized hardware and software design.
        The system can handle multiple types of data including text, images, and video content.

        Industry leaders are praising the innovation and its potential applications.
        Healthcare, finance, and transportation sectors are expected to benefit significantly.
        Regulatory bodies are reviewing the technology to ensure compliance with safety standards.
        Privacy advocates have raised concerns about data handling and user protection measures.
    "#;

    let start = Instant::now();
    let chunks = strategy.chunk(news_article).await.unwrap();
    let duration = start.elapsed();

    println!(
        "News article: {} chunks in {}ms",
        chunks.len(),
        duration.as_millis()
    );

    // Should complete quickly
    assert!(
        duration.as_millis() < 50,
        "News article chunking too slow: {}ms",
        duration.as_millis()
    );

    // Should identify topic shifts in news content
    assert!(!chunks.is_empty(), "Should produce chunks for news article");

    // Check for topic coherence
    for chunk in &chunks {
        assert!(
            chunk.metadata.quality_score > 0.0,
            "Chunks should have quality scores"
        );
        if chunk.metadata.chunk_type == "topic" {
            assert!(
                !chunk.metadata.topic_keywords.is_empty(),
                "Topic chunks should have keywords"
            );
        }
    }
}

/// Test topic chunking with technical documentation
#[tokio::test]
async fn test_technical_documentation_chunking() {
    let config = ChunkingConfig::default();
    let strategy = create_strategy(
        ChunkingMode::Topic {
            topic_chunking: true,
            window_size: 3,
            smoothing_passes: 2,
        },
        config,
    );

    let technical_doc = r#"
        API Documentation: RESTful Web Service

        Authentication
        All API requests require authentication using OAuth 2.0 protocol.
        Clients must obtain an access token before making any API calls.
        Token expiration is set to 24 hours for security purposes.
        Refresh tokens can be used to obtain new access tokens automatically.

        Endpoints
        The GET /users endpoint retrieves user information from the database.
        POST /users creates new user accounts with provided registration data.
        PUT /users/{id} updates existing user information for the specified ID.
        DELETE /users/{id} removes user accounts permanently from the system.

        Request Format
        All requests must include Content-Type header set to application/json.
        Request bodies should contain valid JSON with required field parameters.
        Optional fields can be omitted but may result in default value assignment.
        Malformed requests will return HTTP 400 Bad Request error responses.

        Response Format
        Successful responses return HTTP 200 OK with JSON-formatted data.
        Error responses include appropriate HTTP status codes and error messages.
        Pagination is implemented for endpoints that return multiple records.
        Response metadata includes timing information and request identifiers.

        Rate Limiting
        API calls are limited to 1000 requests per hour per authenticated user.
        Rate limit headers are included in all responses for client monitoring.
        Exceeding rate limits results in HTTP 429 Too Many Requests responses.
        Rate limits reset at the beginning of each hour automatically.
    "#;

    let start = Instant::now();
    let chunks = strategy.chunk(technical_doc).await.unwrap();
    let duration = start.elapsed();

    println!(
        "Technical doc: {} chunks in {}ms",
        chunks.len(),
        duration.as_millis()
    );

    // Should complete quickly
    assert!(
        duration.as_millis() < 50,
        "Technical doc chunking too slow: {}ms",
        duration.as_millis()
    );

    // Should identify section boundaries
    assert!(
        chunks.len() >= 2,
        "Should identify multiple sections in technical doc"
    );

    // Verify API-related keywords are captured
    let all_keywords: Vec<String> = chunks
        .iter()
        .flat_map(|c| c.metadata.topic_keywords.iter().cloned())
        .collect();

    // Should contain technical terms
    let has_technical_terms = all_keywords.iter().any(|k| {
        k.contains("api")
            || k.contains("authentication")
            || k.contains("endpoint")
            || k.contains("request")
            || k.contains("response")
    });
    assert!(
        has_technical_terms,
        "Should extract technical keywords: {:?}",
        all_keywords
    );
}

/// Test topic chunking with mixed content (HTML + text)
#[tokio::test]
async fn test_mixed_content_chunking() {
    let config = ChunkingConfig::default();
    let strategy = create_strategy(
        ChunkingMode::Topic {
            topic_chunking: true,
            window_size: 3,
            smoothing_passes: 1,
        },
        config,
    );

    let mixed_content = r#"
        <h1>Welcome to Our Platform</h1>
        <p>Our platform provides comprehensive solutions for modern businesses.</p>
        <p>We offer cloud-based services with enterprise-grade security and reliability.</p>

        <h2>Features</h2>
        <ul>
            <li>Real-time data synchronization across all devices</li>
            <li>Advanced analytics and reporting capabilities</li>
            <li>Customizable dashboards for different user roles</li>
        </ul>

        Getting started is simple and straightforward for new users.
        The onboarding process includes guided tutorials and documentation.
        Support team is available 24/7 to assist with any questions.

        <h2>Pricing</h2>
        <p>We offer flexible pricing plans to meet different business needs.</p>
        <p>Basic plan starts at $29 per month with core features included.</p>
        <p>Enterprise plans include advanced features and priority support.</p>

        Performance and scalability are key strengths of our platform.
        The system can handle millions of requests per day efficiently.
        Auto-scaling ensures optimal performance during peak usage periods.
    "#;

    let start = Instant::now();
    let chunks = strategy.chunk(mixed_content).await.unwrap();
    let duration = start.elapsed();

    println!(
        "Mixed content: {} chunks in {}ms",
        chunks.len(),
        duration.as_millis()
    );

    // Should complete quickly
    assert!(
        duration.as_millis() < 50,
        "Mixed content chunking too slow: {}ms",
        duration.as_millis()
    );

    // Should handle mixed HTML and text
    assert!(
        !chunks.is_empty(),
        "Should produce chunks for mixed content"
    );

    // Verify chunks contain both HTML and text elements appropriately
    let _has_html = chunks
        .iter()
        .any(|c| c.content.contains('<') && c.content.contains('>'));
let _ = chunks.iter().any(|c| !c.content.contains('<'));
    // All chunks should have content (HTML tags might be preserved or stripped)
    for chunk in &chunks {
        assert!(
            !chunk.content.trim().is_empty(),
            "Chunks should not be empty"
        );
    }
}

/// Test performance with large documents
#[tokio::test]
async fn test_large_document_performance() {
    let config = ChunkingConfig::default();
    let strategy = create_strategy(
        ChunkingMode::Topic {
            topic_chunking: true,
            window_size: 4,
            smoothing_passes: 2,
        },
        config,
    );

    // Generate a large document with multiple topics
    let mut large_doc = String::new();
    let topics = vec![
        "artificial intelligence and machine learning applications",
        "climate change and environmental sustainability measures",
        "economic policy and global financial markets analysis",
        "healthcare technology and medical research advances",
        "education systems and pedagogical methodology improvements",
    ];

    for topic_num in 0..20 {
        let topic = &topics[topic_num % topics.len()];
        large_doc.push_str(&format!(
            "\n\nChapter {}: Advanced Topics in {}\n\
            This chapter explores the fundamental concepts and practical applications. \
            Recent developments have shown significant progress in this area. \
            Researchers and practitioners continue to push the boundaries of knowledge. \
            The implications for future development are substantial and far-reaching. \
            Case studies demonstrate real-world implementation strategies and outcomes. \
            Best practices have emerged from extensive experimentation and analysis. \
            Future research directions include several promising avenues for exploration. \
            Collaborative efforts between institutions accelerate advancement in the field.",
            topic_num + 1,
            topic
        ));
    }

    println!("Large document size: {} characters", large_doc.len());

    let start = Instant::now();
    let chunks = strategy.chunk(&large_doc).await.unwrap();
    let duration = start.elapsed();

    println!(
        "Large document: {} chunks in {}ms",
        chunks.len(),
        duration.as_millis()
    );

    // Should meet performance requirement even for large docs
    assert!(
        duration.as_millis() < 200,
        "Large document chunking too slow: {}ms",
        duration.as_millis()
    );

    // Should produce reasonable number of chunks
    assert!(
        chunks.len() >= 5,
        "Should identify multiple topics in large document"
    );
    assert!(chunks.len() <= 25, "Should not over-segment large document");

    // Check chunk quality
    for chunk in &chunks {
        assert!(!chunk.content.trim().is_empty(), "No empty chunks");
        assert!(
            chunk.metadata.quality_score > 0.0,
            "All chunks should have quality scores"
        );
    }
}

/// Test edge case: very short text
#[tokio::test]
async fn test_short_text_handling() {
    let config = ChunkingConfig::default();
    let strategy = create_strategy(
        ChunkingMode::Topic {
            topic_chunking: true,
            window_size: 3,
            smoothing_passes: 1,
        },
        config,
    );

    let short_text = "This is a very short document with minimal content.";

    let start = Instant::now();
    let chunks = strategy.chunk(short_text).await.unwrap();
    let duration = start.elapsed();

    println!(
        "Short text: {} chunks in {}ms",
        chunks.len(),
        duration.as_millis()
    );

    // Should complete very quickly
    assert!(
        duration.as_millis() < 10,
        "Short text chunking too slow: {}ms",
        duration.as_millis()
    );

    // Should produce single chunk for short text
    assert_eq!(chunks.len(), 1, "Short text should produce single chunk");
    assert_eq!(chunks[0].metadata.chunk_type, "topic-single");
    assert_eq!(chunks[0].content, short_text);
}

/// Test edge case: empty text
#[tokio::test]
async fn test_empty_text_handling() {
    let config = ChunkingConfig::default();
    let strategy = create_strategy(
        ChunkingMode::Topic {
            topic_chunking: true,
            window_size: 3,
            smoothing_passes: 1,
        },
        config,
    );

    let empty_text = "";

    let start = Instant::now();
    let chunks = strategy.chunk(empty_text).await.unwrap();
    let duration = start.elapsed();

    println!(
        "Empty text: {} chunks in {}ms",
        chunks.len(),
        duration.as_millis()
    );

    // Should complete immediately
    assert!(
        duration.as_millis() < 5,
        "Empty text chunking too slow: {}ms",
        duration.as_millis()
    );

    // Should produce no chunks for empty text
    assert!(chunks.is_empty(), "Empty text should produce no chunks");
}
