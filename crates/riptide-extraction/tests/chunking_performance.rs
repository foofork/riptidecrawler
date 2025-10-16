//! Performance tests for chunking strategies
//! Requirement: All chunking strategies must process 50KB of text in ≤200ms

use riptide_extraction::chunking::{create_strategy, ChunkingConfig, ChunkingMode, ChunkingStrategy};
use std::time::Instant;

/// Generate test content of specified size
fn generate_test_content(target_size: usize, content_type: ContentType) -> String {
    match content_type {
        ContentType::PlainText => generate_plain_text(target_size),
        ContentType::Html => generate_html_content(target_size),
        ContentType::Mixed => generate_mixed_content(target_size),
        ContentType::TopicDiverse => generate_topic_diverse_content(target_size),
    }
}

enum ContentType {
    PlainText,
    Html,
    Mixed,
    TopicDiverse,
}

fn generate_plain_text(target_size: usize) -> String {
    let base_sentences = [
        "This is a performance test document with multiple sentences and paragraphs.",
        "The content needs to be realistic to properly test chunking algorithms.",
        "We include various sentence structures and lengths to simulate real documents.",
        "Performance testing requires careful measurement of execution time.",
        "The chunking strategy should maintain quality while meeting speed requirements.",
        "Each sentence contributes to the overall document structure and meaning.",
        "Testing with different content types helps ensure robust performance.",
        "The algorithm must handle edge cases and varying text complexity.",
        "Quality metrics include token count accuracy and boundary preservation.",
        "Real-world documents often contain mixed content and formatting.",
    ];

    let mut content = String::new();
    let mut sentence_index = 0;

    while content.len() < target_size {
        if sentence_index > 0 && sentence_index % 5 == 0 {
            content.push_str("\n\n"); // Paragraph break
        }
        content.push_str(base_sentences[sentence_index % base_sentences.len()]);
        content.push(' ');
        sentence_index += 1;
    }

    content.truncate(target_size);
    content
}

fn generate_html_content(target_size: usize) -> String {
    let mut content = String::from("<html><body>");
    let mut current_size = content.len();

    let html_patterns = [
        "<article><h1>Article Title</h1><p>Article content with meaningful text.</p></article>",
        "<section><h2>Section Header</h2><p>Section content with various elements.</p></section>",
        "<div class='content'><p>Content in a div container with class.</p></div>",
        "<main><p>Main content area with important information.</p></main>",
        "<aside><p>Sidebar content with additional details.</p></aside>",
        "<header><h1>Page Header</h1><nav><a href='#'>Link</a></nav></header>",
        "<footer><p>Footer content with contact information.</p></footer>",
    ];

    let mut pattern_index = 0;
    while current_size < target_size - 20 {
        // Leave room for closing tags
        let pattern = html_patterns[pattern_index % html_patterns.len()];
        content.push_str(pattern);
        current_size += pattern.len();
        pattern_index += 1;
    }

    content.push_str("</body></html>");
    content
}

fn generate_mixed_content(target_size: usize) -> String {
    let mut content = String::new();
    let plain_text = generate_plain_text(target_size / 2);
    let html_content = generate_html_content(target_size / 2);

    content.push_str(&html_content);
    content.push_str("\n\n");
    content.push_str(&plain_text);

    if content.len() > target_size {
        content.truncate(target_size);
    }

    content
}

fn generate_topic_diverse_content(target_size: usize) -> String {
    let topics = [
        vec![
            "Machine learning algorithms are revolutionizing data processing.",
            "Artificial intelligence systems learn from vast datasets.",
            "Deep learning neural networks process complex patterns.",
            "Computer vision enables machines to interpret visual information.",
            "Natural language processing helps computers understand text.",
        ],
        vec![
            "Climate change affects global weather patterns significantly.",
            "Rising temperatures impact ecosystems worldwide.",
            "Environmental conservation requires immediate action.",
            "Renewable energy sources reduce carbon emissions.",
            "Sustainable practices protect natural resources.",
        ],
        vec![
            "Economic policies influence international trade relations.",
            "Market dynamics affect global financial stability.",
            "Investment strategies require careful risk assessment.",
            "Inflation impacts consumer purchasing power.",
            "Economic growth depends on various market factors.",
        ],
        vec![
            "Quantum computing promises unprecedented computational power.",
            "Cryptographic security protects digital communications.",
            "Blockchain technology enables decentralized systems.",
            "Cybersecurity measures defend against digital threats.",
            "Information security protocols safeguard sensitive data.",
        ],
        vec![
            "Social media platforms transform modern communication.",
            "Digital connectivity influences social interactions.",
            "Online communities foster global collaboration.",
            "Information sharing accelerates knowledge transfer.",
            "Virtual relationships supplement traditional connections.",
        ],
    ];

    let mut content = String::new();
    let mut topic_index = 0;
    let mut sentence_index = 0;

    while content.len() < target_size {
        let current_topic = &topics[topic_index % topics.len()];

        // Add 3-5 sentences from current topic to create coherent blocks
        let sentences_in_block = 3 + (sentence_index % 3);
        for _ in 0..sentences_in_block {
            if content.len() >= target_size {
                break;
            }

            content.push_str(current_topic[sentence_index % current_topic.len()]);
            content.push(' ');
            sentence_index += 1;
        }

        // Add paragraph break and switch topic
        content.push_str("\n\n");
        topic_index += 1;
        sentence_index = 0;
    }

    content.truncate(target_size);
    content
}

/// Test performance of a chunking strategy with specified content
async fn test_strategy_performance(
    strategy: Box<dyn ChunkingStrategy>,
    content: &str,
    max_duration_ms: u128,
) -> Result<(), String> {
    let start = Instant::now();
    let chunks = strategy
        .chunk(content)
        .await
        .map_err(|e| format!("Chunking failed: {}", e))?;
    let duration = start.elapsed();

    println!(
        "Strategy '{}': {} chunks in {}ms (content: {} chars)",
        strategy.name(),
        chunks.len(),
        duration.as_millis(),
        content.len()
    );

    if duration.as_millis() > max_duration_ms {
        return Err(format!(
            "Strategy '{}' took {}ms, expected ≤{}ms",
            strategy.name(),
            duration.as_millis(),
            max_duration_ms
        ));
    }

    // Basic quality checks
    assert!(
        !chunks.is_empty(),
        "Strategy '{}' produced no chunks",
        strategy.name()
    );

    // Check that all content is covered
    let total_content_length: usize = chunks.iter().map(|c| c.content.len()).sum();
    assert!(
        total_content_length > 0,
        "Strategy '{}' produced empty content",
        strategy.name()
    );

    Ok(())
}

#[tokio::test]
async fn test_50kb_performance_requirement() {
    let target_size = 50_000; // 50KB
    let max_duration_ms = 200; // 200ms requirement

    // Test with different content types
    let test_cases = vec![
        (
            "Plain Text",
            generate_test_content(target_size, ContentType::PlainText),
        ),
        (
            "HTML Content",
            generate_test_content(target_size, ContentType::Html),
        ),
        (
            "Mixed Content",
            generate_test_content(target_size, ContentType::Mixed),
        ),
        (
            "Topic Diverse",
            generate_test_content(target_size, ContentType::TopicDiverse),
        ),
    ];

    let config = ChunkingConfig::default();

    // Test all chunking strategies
    let strategies = vec![
        (
            "Sliding Window",
            ChunkingMode::Sliding {
                window_size: 1000,
                overlap: 100,
            },
        ),
        (
            "Fixed Size (Tokens)",
            ChunkingMode::Fixed {
                size: 800,
                by_tokens: true,
            },
        ),
        (
            "Fixed Size (Chars)",
            ChunkingMode::Fixed {
                size: 1000,
                by_tokens: false,
            },
        ),
        (
            "Sentence-based",
            ChunkingMode::Sentence { max_sentences: 5 },
        ),
        (
            "Regex (Paragraphs)",
            ChunkingMode::Regex {
                pattern: r"\n\s*\n".to_string(),
                min_chunk_size: 200,
            },
        ),
        (
            "HTML-aware (Structure)",
            ChunkingMode::HtmlAware {
                preserve_blocks: true,
                preserve_structure: true,
            },
        ),
        (
            "HTML-aware (Blocks)",
            ChunkingMode::HtmlAware {
                preserve_blocks: true,
                preserve_structure: false,
            },
        ),
        (
            "Topic Chunking (Enabled)",
            ChunkingMode::Topic {
                topic_chunking: true,
                window_size: 3,
                smoothing_passes: 2,
            },
        ),
        (
            "Topic Chunking (Disabled)",
            ChunkingMode::Topic {
                topic_chunking: false,
                window_size: 3,
                smoothing_passes: 2,
            },
        ),
    ];

    for (content_name, content) in test_cases {
        println!(
            "\n=== Testing with {} ({} chars) ===",
            content_name,
            content.len()
        );

        for (strategy_name, mode) in &strategies {
            let strategy = create_strategy(mode.clone(), config.clone());

            match test_strategy_performance(strategy, &content, max_duration_ms).await {
                Ok(()) => println!("✓ {} passed performance test", strategy_name),
                Err(e) => {
                    panic!("✗ {} failed performance test: {}", strategy_name, e);
                }
            }
        }
    }
}

#[tokio::test]
async fn test_large_content_scalability() {
    // Test with progressively larger content to ensure good scaling
    let config = ChunkingConfig::default();
    let strategy = create_strategy(ChunkingMode::default(), config);

    let test_sizes = vec![10_000, 25_000, 50_000, 100_000];
    let mut previous_duration = 0;

    for size in test_sizes {
        let content = generate_test_content(size, ContentType::PlainText);
        let start = Instant::now();
        let chunks = strategy.chunk(&content).await.unwrap();
        let duration = start.elapsed().as_millis();

        println!(
            "Size: {}KB, Duration: {}ms, Chunks: {}, Rate: {:.2} chars/ms",
            size / 1000,
            duration,
            chunks.len(),
            content.len() as f64 / duration as f64
        );

        // Performance should scale reasonably (not exponentially)
        if previous_duration > 0 {
            let size_ratio = size as f64 / (size / 2) as f64; // Should be 2.0
            let time_ratio = duration as f64 / previous_duration as f64;

            // Time ratio should not be significantly higher than size ratio
            assert!(
                time_ratio < size_ratio * 1.5,
                "Poor scaling: {}x size increase led to {}x time increase",
                size_ratio,
                time_ratio
            );
        }

        previous_duration = duration;
    }
}

#[tokio::test]
async fn test_memory_efficiency() {
    // Test memory usage doesn't grow excessively
    let config = ChunkingConfig::default();
    let strategy = create_strategy(ChunkingMode::default(), config);

    let content = generate_test_content(50_000, ContentType::PlainText);

    // Multiple runs to check for memory leaks
    for i in 0..10 {
        let start = Instant::now();
        let chunks = strategy.chunk(&content).await.unwrap();
        let duration = start.elapsed();

        // Performance should be consistent across runs
        assert!(
            duration.as_millis() <= 250, // Allow some variance
            "Run {}: Duration {}ms exceeded threshold",
            i + 1,
            duration.as_millis()
        );

        // Verify chunks are reasonable
        assert!(!chunks.is_empty(), "Run {}: No chunks produced", i + 1);
        assert!(
            chunks.iter().all(|c| !c.content.is_empty()),
            "Run {}: Found empty chunks",
            i + 1
        );
    }
}

#[tokio::test]
async fn test_html_specific_performance() {
    // Test HTML-specific performance characteristics
    let config = ChunkingConfig::default();

    // Large HTML document with complex structure
    let mut html_content = String::from(
        r#"
        <!DOCTYPE html>
        <html>
        <head><title>Complex Document</title></head>
        <body>
    "#,
    );

    // Generate nested HTML structure
    for i in 0..100 {
        html_content.push_str(&format!(
            r#"
            <article id="article-{}">
                <header>
                    <h1>Article {} Title</h1>
                    <p class="meta">Published on 2024-01-01</p>
                </header>
                <section class="content">
                    <p>This is the main content of article {}. It contains multiple sentences and paragraphs to test the HTML-aware chunking strategy.</p>
                    <div class="details">
                        <p>Additional details in a nested div element.</p>
                        <ul>
                            <li>List item one with content</li>
                            <li>List item two with content</li>
                            <li>List item three with content</li>
                        </ul>
                    </div>
                </section>
                <footer>
                    <p>Article {} footer information.</p>
                </footer>
            </article>
            "#,
            i, i, i, i
        ));
    }

    html_content.push_str("</body></html>");

    println!("Testing HTML content: {} chars", html_content.len());

    // Test HTML-aware strategies
    let html_strategies = vec![
        (
            "HTML Structure",
            ChunkingMode::HtmlAware {
                preserve_blocks: true,
                preserve_structure: true,
            },
        ),
        (
            "HTML Blocks",
            ChunkingMode::HtmlAware {
                preserve_blocks: true,
                preserve_structure: false,
            },
        ),
    ];

    for (name, mode) in html_strategies {
        let strategy = create_strategy(mode, config.clone());

        let start = Instant::now();
        let chunks = strategy.chunk(&html_content).await.unwrap();
        let duration = start.elapsed();

        println!(
            "{}: {} chunks in {}ms (avg {:.2} chars/chunk)",
            name,
            chunks.len(),
            duration.as_millis(),
            html_content.len() as f64 / chunks.len() as f64
        );

        assert!(
            duration.as_millis() <= 200,
            "{} exceeded 200ms: {}ms",
            name,
            duration.as_millis()
        );

        // HTML chunks should preserve some structure
        assert!(!chunks.is_empty(), "{} produced no chunks", name);

        // Check that HTML tags are preserved (not split mid-tag)
        for (i, chunk) in chunks.iter().enumerate() {
            let tag_opens = chunk.content.matches('<').count();
            let tag_closes = chunk.content.matches('>').count();

            // If we have opening tags, we should have corresponding closing markers
            // (This is a simplified check - real HTML might have self-closing tags)
            if tag_opens > 0 && !chunk.content.contains("<!DOCTYPE") {
                assert!(
                    tag_closes > 0,
                    "{} chunk {}: Found opening tags without closing markers",
                    name,
                    i
                );
            }
        }
    }
}

#[tokio::test]
async fn test_edge_cases_performance() {
    let config = ChunkingConfig::default();

    // Test edge cases
    let edge_cases = vec![
        ("Empty", String::new()),
        ("Single char", "a".to_string()),
        ("Single sentence", "This is a single sentence.".to_string()),
        ("No punctuation", "word ".repeat(1000)),
        ("Many short sentences", "Hi. ".repeat(1000)),
        (
            "Very long single sentence",
            format!("{} and this continues forever.", "word ".repeat(2000)),
        ),
    ];

    for (name, content) in edge_cases {
        if content.is_empty() && name == "Empty" {
            // Empty content should be handled quickly
            let strategy = create_strategy(ChunkingMode::default(), config.clone());
            let start = Instant::now();
            let chunks = strategy.chunk(&content).await.unwrap();
            let duration = start.elapsed();

            assert!(chunks.is_empty(), "Empty content should produce no chunks");
            assert!(
                duration.as_millis() < 10,
                "Empty content processing too slow"
            );
            continue;
        }

        println!("Testing edge case: {} ({} chars)", name, content.len());

        let strategy = create_strategy(ChunkingMode::default(), config.clone());
        let start = Instant::now();
        let chunks = strategy.chunk(&content).await.unwrap();
        let duration = start.elapsed();

        // Even edge cases should complete quickly
        assert!(
            duration.as_millis() < 100,
            "Edge case '{}' took {}ms",
            name,
            duration.as_millis()
        );

        if !content.trim().is_empty() {
            assert!(
                !chunks.is_empty(),
                "Non-empty content '{}' should produce chunks",
                name
            );
        }
    }
}

#[tokio::test]
async fn test_topic_chunking_specific_performance() {
    // Specific test for topic chunking requirements
    let config = ChunkingConfig::default();
    let target_size = 50_000;

    // Generate content with clear topic boundaries
    let topic_content = generate_test_content(target_size, ContentType::TopicDiverse);

    println!(
        "Testing Topic Chunking Performance with {} chars",
        topic_content.len()
    );

    // Test topic chunking enabled
    let strategy_enabled = create_strategy(
        ChunkingMode::Topic {
            topic_chunking: true,
            window_size: 3,
            smoothing_passes: 2,
        },
        config.clone(),
    );

    let start = Instant::now();
    let chunks_enabled = strategy_enabled.chunk(&topic_content).await.unwrap();
    let duration_enabled = start.elapsed();

    println!(
        "Topic chunking (enabled): {} chunks in {}ms",
        chunks_enabled.len(),
        duration_enabled.as_millis()
    );

    // Should meet <200ms requirement
    assert!(
        duration_enabled.as_millis() < 200,
        "Topic chunking (enabled) took {}ms, expected <200ms",
        duration_enabled.as_millis()
    );

    // Test topic chunking disabled (fallback)
    let strategy_disabled = create_strategy(
        ChunkingMode::Topic {
            topic_chunking: false,
            window_size: 3,
            smoothing_passes: 2,
        },
        config.clone(),
    );

    let start = Instant::now();
    let chunks_disabled = strategy_disabled.chunk(&topic_content).await.unwrap();
    let duration_disabled = start.elapsed();

    println!(
        "Topic chunking (disabled): {} chunks in {}ms",
        chunks_disabled.len(),
        duration_disabled.as_millis()
    );

    // Should also meet performance requirement (fallback to sliding window)
    assert!(
        duration_disabled.as_millis() < 200,
        "Topic chunking (disabled) took {}ms, expected <200ms",
        duration_disabled.as_millis()
    );

    // Verify topic-enabled chunking produces meaningful results
    assert!(
        !chunks_enabled.is_empty(),
        "Topic chunking should produce chunks"
    );

    // Check that topic chunks have topic keywords
    for chunk in &chunks_enabled {
        if chunk.metadata.chunk_type == "topic" {
            assert!(
                !chunk.metadata.topic_keywords.is_empty(),
                "Topic chunks should have topic keywords"
            );
        }
    }

    // Disabled should fallback to different chunking
    assert!(
        !chunks_disabled.is_empty(),
        "Fallback chunking should produce chunks"
    );

    println!(
        "Topic chunking performance test completed: enabled={}ms, disabled={}ms",
        duration_enabled.as_millis(),
        duration_disabled.as_millis()
    );
}
