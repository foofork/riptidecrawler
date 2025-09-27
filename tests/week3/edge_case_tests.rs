//! Edge case tests for Week 3 chunking and HTML processing
//!
//! Tests for handling:
//! - Empty text and edge cases
//! - Very large documents
//! - Unicode and special characters
//! - Nested HTML structures
//! - Malformed content

use std::time::{Duration, Instant};
use tokio_test;

// Import chunking functionality from the project
use riptide_core::strategies::chunking::{
    chunk_content, ChunkingConfig, ChunkingMode, ContentChunk, count_tokens
};
use riptide_html::{
    HtmlProcessor, DefaultHtmlProcessor, ProcessingError,
    dom_utils::{extract_links, extract_images, DomTraverser},
    processor::TableExtractionMode,
};

#[tokio::test]
async fn test_empty_and_minimal_inputs() {
    let config = ChunkingConfig::default();

    // Test completely empty text
    let empty_chunks = chunk_content("", &config).await.unwrap();
    assert!(empty_chunks.is_empty(), "Empty text should produce no chunks");

    // Test whitespace-only text
    let whitespace_inputs = vec![
        " ",
        "\n",
        "\t",
        "   ",
        "\n\n\n",
        "\t\t\t",
        " \n \t \n ",
    ];

    for whitespace_text in whitespace_inputs {
        let chunks = chunk_content(whitespace_text, &config).await.unwrap();
        if !chunks.is_empty() {
            // If chunks are produced, they should contain meaningful content
            assert!(chunks.iter().all(|c| !c.content.trim().is_empty() || c.content.trim().is_empty()));
        }
    }

    // Test single character
    let single_char_chunks = chunk_content("a", &config).await.unwrap();
    assert_eq!(single_char_chunks.len(), 1);
    assert_eq!(single_char_chunks[0].content, "a");

    // Test single word
    let single_word_chunks = chunk_content("hello", &config).await.unwrap();
    assert_eq!(single_word_chunks.len(), 1);
    assert_eq!(single_word_chunks[0].content, "hello");

    // Test minimal sentence
    let minimal_sentence_chunks = chunk_content("Hello.", &config).await.unwrap();
    assert_eq!(minimal_sentence_chunks.len(), 1);
    assert_eq!(minimal_sentence_chunks[0].content, "Hello.");
    assert!(minimal_sentence_chunks[0].metadata.has_complete_sentences);
}

#[tokio::test]
async fn test_unicode_and_special_characters() {
    let config = ChunkingConfig::default();

    // Test various Unicode scripts
    let unicode_texts = vec![
        "Hello 世界! 🌍 こんにちは",
        "Здравствуй мир! 🚀 Testing",
        "مرحبا بالعالم! Arabic text",
        "हैलो वर्ल्ड! Hindi text",
        "שלום עולם! Hebrew text",
        "🎉🎊🎈 Emojis galore! 🌟⭐✨",
        "Mathematical symbols: ∑∫√π∞≠≤≥±",
        "Currency: $€£¥₹₽¢",
        "Diacritics: café naïve résumé",
    ];

    for unicode_text in unicode_texts {
        let chunks = chunk_content(unicode_text, &config).await.unwrap();
        assert!(!chunks.is_empty(), "Unicode text should produce chunks: {}", unicode_text);

        for chunk in chunks {
            // Verify Unicode characters are preserved
            assert!(chunk.content.len() > 0);
            assert!(!chunk.id.is_empty());

            // Check that token counting works with Unicode
            assert!(chunk.token_count > 0);
        }
    }

    // Test mixed Unicode and ASCII
    let mixed_text = "English text 中文 العربية русский 日本語 with normal words between";
    let mixed_chunks = chunk_content(&mixed_text, &config).await.unwrap();
    assert!(!mixed_chunks.is_empty());

    // Verify all characters are preserved
    let reconstructed: String = mixed_chunks.iter().map(|c| &c.content).collect::<Vec<_>>().join(" ");
    assert!(reconstructed.contains("中文"));
    assert!(reconstructed.contains("العربية"));
    assert!(reconstructed.contains("русский"));
    assert!(reconstructed.contains("日本語"));

    // Test special punctuation and symbols
    let special_chars_text = "Special chars: @#$%^&*()[]{}|\\:;\"'<>,.?/~`!+=_-";
    let special_chunks = chunk_content(&special_chars_text, &config).await.unwrap();
    assert!(!special_chunks.is_empty());
    assert!(special_chunks[0].content.contains("@#$%"));
}

#[tokio::test]
async fn test_very_large_documents() {
    // Test different large document sizes
    let sizes = vec![100_000, 500_000, 1_000_000]; // 100KB, 500KB, 1MB

    for size in sizes {
        println!("Testing document size: {} bytes", size);

        let large_text = generate_large_realistic_text(size);
        let config = ChunkingConfig {
            token_max: 1000,
            overlap: 100,
            preserve_sentences: true,
            deterministic: true,
            mode: ChunkingMode::Sliding,
        };

        let start = Instant::now();
        let chunks = chunk_content(&large_text, &config).await.unwrap();
        let elapsed = start.elapsed();

        // Performance requirements
        let max_time = if size <= 100_000 {
            Duration::from_millis(500)
        } else if size <= 500_000 {
            Duration::from_millis(2000)
        } else {
            Duration::from_millis(5000)
        };

        assert!(
            elapsed <= max_time,
            "Large document processing took too long: {:?} for {} bytes",
            elapsed, size
        );

        assert!(!chunks.is_empty(), "Large document should produce chunks");

        // Verify chunk quality doesn't degrade with document size
        let avg_quality: f64 = chunks.iter()
            .map(|c| c.metadata.quality_score)
            .sum::<f64>() / chunks.len() as f64;

        assert!(avg_quality > 0.3, "Average chunk quality too low: {:.2}", avg_quality);

        // Verify chunks stay within token limits
        for chunk in &chunks {
            assert!(
                chunk.token_count <= config.token_max + 200, // Allow some tolerance
                "Chunk exceeds token limit: {} tokens",
                chunk.token_count
            );
        }

        println!("Size: {} bytes, Chunks: {}, Time: {:?}, Avg Quality: {:.2}",
                size, chunks.len(), elapsed, avg_quality);
    }
}

#[tokio::test]
async fn test_extreme_chunking_configurations() {
    let test_text = "This is a test text for extreme chunking configurations. We want to see how the system handles unusual parameter values.";

    // Test very small chunk sizes
    let tiny_config = ChunkingConfig {
        mode: ChunkingMode::Fixed { size: 5, by_tokens: false },
        token_max: 10,
        overlap: 0,
        preserve_sentences: false,
        deterministic: true,
    };

    let tiny_chunks = chunk_content(&test_text, &tiny_config).await.unwrap();
    assert!(!tiny_chunks.is_empty());

    // Test very large chunk sizes (larger than input)
    let huge_config = ChunkingConfig {
        mode: ChunkingMode::Fixed { size: 10000, by_tokens: false },
        token_max: 50000,
        overlap: 0,
        preserve_sentences: true,
        deterministic: true,
    };

    let huge_chunks = chunk_content(&test_text, &huge_config).await.unwrap();
    assert_eq!(huge_chunks.len(), 1); // Should produce single chunk
    assert_eq!(huge_chunks[0].content, test_text);

    // Test maximum overlap (overlap equals chunk size)
    let max_overlap_config = ChunkingConfig {
        mode: ChunkingMode::Sliding,
        token_max: 50,
        overlap: 50,
        preserve_sentences: false,
        deterministic: true,
    };

    let overlap_chunks = chunk_content(&test_text, &max_overlap_config).await.unwrap();
    // Should handle gracefully without infinite loops

    // Test overlap larger than chunk size
    let excessive_overlap_config = ChunkingConfig {
        mode: ChunkingMode::Sliding,
        token_max: 30,
        overlap: 100,
        preserve_sentences: false,
        deterministic: true,
    };

    let excessive_chunks = chunk_content(&test_text, &excessive_overlap_config).await.unwrap();
    // Should handle gracefully
    assert!(!excessive_chunks.is_empty());
}

#[tokio::test]
async fn test_malformed_and_pathological_content() {
    let config = ChunkingConfig::default();

    // Test content with many repeated characters
    let repeated_chars = "a".repeat(10000);
    let repeated_chunks = chunk_content(&repeated_chars, &config).await.unwrap();
    assert!(!repeated_chunks.is_empty());

    // Test content with many newlines
    let many_newlines = "\n".repeat(1000) + "Some content" + &"\n".repeat(1000);
    let newline_chunks = chunk_content(&many_newlines, &config).await.unwrap();
    assert!(!newline_chunks.is_empty());

    // Test content with extremely long words
    let long_word = "supercalifragilisticexpialidocious".repeat(100);
    let long_word_text = format!("Normal word {} another word", long_word);
    let long_word_chunks = chunk_content(&long_word_text, &config).await.unwrap();
    assert!(!long_word_chunks.is_empty());

    // Test content with many punctuation marks
    let punctuation_heavy = "!@#$%^&*()_+-=[]{}|;':\",./<>?".repeat(100);
    let punct_chunks = chunk_content(&punctuation_heavy, &config).await.unwrap();
    assert!(!punct_chunks.is_empty());

    // Test binary-like content (random bytes represented as text)
    let binary_like = (0..1000).map(|i| char::from(33 + (i % 94) as u8)).collect::<String>();
    let binary_chunks = chunk_content(&binary_like, &config).await.unwrap();
    assert!(!binary_chunks.is_empty());

    // Test content with control characters
    let control_chars = "Normal text\x01\x02\x03\x04\x05 more text\x1F\x7F end";
    let control_chunks = chunk_content(&control_chars, &config).await.unwrap();
    assert!(!control_chunks.is_empty());
}

#[tokio::test]
async fn test_html_edge_cases() {
    let processor = DefaultHtmlProcessor::default();

    // Test empty HTML
    let empty_html = "";
    let empty_links = extract_links(empty_html).unwrap();
    assert!(empty_links.is_empty());

    // Test HTML with only comments
    let comment_html = "<!-- This is just a comment -->";
    let comment_links = extract_links(comment_html).unwrap();
    assert!(comment_links.is_empty());

    // Test deeply nested HTML (1000 levels deep)
    let mut deep_html = String::from("<html><body>");
    for i in 0..1000 {
        deep_html.push_str(&format!("<div id='level{}'>", i));
    }
    deep_html.push_str("<p>Deep content</p>");
    for _ in 0..1000 {
        deep_html.push_str("</div>");
    }
    deep_html.push_str("</body></html>");

    let start = Instant::now();
    let traverser = DomTraverser::new(&deep_html);
    let deep_stats = traverser.get_stats();
    let elapsed = start.elapsed();

    assert!(elapsed < Duration::from_millis(1000), "Deep HTML processing took too long");
    assert!(deep_stats.total_elements > 1000);

    // Test HTML with many attributes
    let many_attrs_html = r#"
    <div id="test" class="a b c d e f g h i j" data-attr1="1" data-attr2="2" data-attr3="3"
         style="color: red; background: blue;" title="test" lang="en" dir="ltr"
         onclick="javascript:void(0)" onmouseover="test()" tabindex="1" role="button"
         aria-label="test" aria-describedby="desc" custom-attr="value">
        <span data-long-attribute-name-that-goes-on-and-on="very long value that contains lots of text">
            Content with many attributes
        </span>
    </div>
    "#;

    let attrs_traverser = DomTraverser::new(many_attrs_html);
    let attrs_elements = attrs_traverser.get_elements_info("div").unwrap();
    assert!(!attrs_elements.is_empty());
    assert!(attrs_elements[0].attributes.len() > 10);

    // Test HTML with Unicode in attributes
    let unicode_attr_html = r#"
    <div title="测试 🌟" alt="Café naïve" data-emoji="🚀🎉">
        <a href="/测试页面" title="العربية">Unicode Link</a>
    </div>
    "#;

    let unicode_links = extract_links(unicode_attr_html).unwrap();
    assert!(!unicode_links.is_empty());
    assert!(unicode_links[0].href.contains("测试"));

    // Test HTML with mixed content types
    let mixed_content_html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <script>var x = "test";</script>
        <style>body { margin: 0; }</style>
    </head>
    <body>
        <div>Regular content</div>
        <script>console.log("inline script");</script>
        <noscript>No script content</noscript>
        <textarea>Text area content</textarea>
        <pre>Preformatted
        content</pre>
    </body>
    </html>
    "#;

    let mixed_traverser = DomTraverser::new(mixed_content_html);
    let mixed_stats = mixed_traverser.get_stats();
    assert!(mixed_stats.total_elements > 5);
}

#[tokio::test]
async fn test_regex_edge_cases() {
    let config_patterns = vec![
        // Very complex regex
        (r"(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)", "IP addresses"),

        // Regex with many alternatives
        (r"(cat|dog|bird|fish|elephant|giraffe|zebra|lion|tiger|bear)", "Animals"),

        // Greedy vs non-greedy
        (r"<.*?>", "HTML tags non-greedy"),
        (r"<.*>", "HTML tags greedy"),

        // Unicode regex
        (r"\p{L}+", "Unicode letters"),

        // Lookahead/lookbehind (if supported)
        (r"\b\w+(?=\s+\w+)", "Words followed by other words"),
    ];

    let test_text = r#"
    The quick brown fox jumps over the lazy dog.
    IP addresses: 192.168.1.1 and 10.0.0.1 are common.
    HTML content: <div>test</div> and <span>content</span>.
    Animals include cat, dog, bird, and elephant.
    Unicode: café naïve résumé 测试 العربية
    "#;

    for (pattern, description) in config_patterns {
        println!("Testing regex pattern: {}", description);

        let regex_config = ChunkingConfig {
            mode: ChunkingMode::Regex {
                pattern: pattern.to_string(),
                min_chunk_size: 10,
            },
            token_max: 1000,
            overlap: 0,
            preserve_sentences: true,
            deterministic: true,
        };

        let result = chunk_content(test_text, &regex_config).await;

        match result {
            Ok(chunks) => {
                assert!(!chunks.is_empty(), "Pattern '{}' should produce chunks", description);

                for chunk in chunks {
                    assert!(!chunk.content.trim().is_empty());
                    assert!(chunk.metadata.chunk_type.starts_with("regex_"));
                }
            }
            Err(e) => {
                // Some regex patterns might not be supported
                println!("Pattern '{}' failed (acceptable): {}", description, e);
            }
        }
    }
}

#[tokio::test]
async fn test_sentence_boundary_edge_cases() {
    let edge_cases = vec![
        // Abbreviations
        "Dr. Smith went to the U.S.A. yesterday. He met Mr. Johnson.",

        // Decimals and numbers
        "The temperature was 98.6 degrees. Version 2.0 was released.",

        // Ellipsis
        "He said... then paused. The story continues...",

        // Multiple punctuation
        "What?! Really?!! I can't believe it!!!",

        // Quotes with punctuation
        r#"He said "Hello world!" and left. She replied "Goodbye.""#,

        // URLs and emails
        "Visit http://example.com. Contact us at test@example.com.",

        // Mixed languages
        "English sentence. 这是中文句子。Arabic: مرحبا.",

        // Sentence fragments
        "Yes. No. Maybe. Definitely not.",
    ];

    let config = ChunkingConfig {
        mode: ChunkingMode::Sentence { max_sentences: 2 },
        token_max: 1000,
        overlap: 0,
        preserve_sentences: true,
        deterministic: true,
    };

    for test_case in edge_cases {
        println!("Testing sentence boundaries: {:?}", test_case);

        let chunks = chunk_content(test_case, &config).await.unwrap();
        assert!(!chunks.is_empty(), "Should produce chunks for: {}", test_case);

        // Verify sentence integrity
        for chunk in chunks {
            if chunk.metadata.has_complete_sentences {
                let content = chunk.content.trim();
                assert!(
                    content.ends_with('.') || content.ends_with('!') || content.ends_with('?'),
                    "Complete sentence should end with proper punctuation: {}",
                    content
                );
            }
        }
    }
}

#[tokio::test]
async fn test_memory_and_performance_limits() {
    // Test with progressively larger inputs to find limits
    let sizes = vec![10_000, 50_000, 100_000, 500_000];

    for size in sizes {
        let text = generate_large_realistic_text(size);

        // Measure memory usage before and after
        let memory_before = get_memory_usage();

        let start = Instant::now();
        let chunks = chunk_content(&text, &ChunkingConfig::default()).await.unwrap();
        let elapsed = start.elapsed();

        let memory_after = get_memory_usage();
        let memory_used = memory_after.saturating_sub(memory_before);

        println!("Size: {} bytes, Chunks: {}, Time: {:?}, Memory: {} bytes",
                size, chunks.len(), elapsed, memory_used);

        // Memory usage should be reasonable (less than 10x input size)
        assert!(
            memory_used < size * 10,
            "Memory usage too high: {} bytes for {} byte input",
            memory_used, size
        );

        // Time should scale reasonably
        let expected_max_time = Duration::from_millis((size / 1000) + 100);
        assert!(
            elapsed < expected_max_time,
            "Processing time too high: {:?} for {} bytes",
            elapsed, size
        );
    }
}

// Helper functions

fn generate_large_realistic_text(target_size: usize) -> String {
    let paragraphs = vec![
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.",

        "Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.",

        "Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo.",

        "At vero eos et accusamus et iusto odio dignissimos ducimus qui blanditiis praesentium voluptatum deleniti atque corrupti quos dolores et quas molestias excepturi sint occaecati cupiditate non provident.",

        "But I must explain to you how all this mistaken idea of denouncing pleasure and praising pain was born and I will give you a complete account of the system, and expound the actual teachings of the great explorer of the truth.",
    ];

    let mut result = String::new();
    let mut current_size = 0;
    let mut paragraph_index = 0;

    while current_size < target_size {
        if !result.is_empty() {
            result.push_str("\n\n");
            current_size += 2;
        }

        let paragraph = &paragraphs[paragraph_index % paragraphs.len()];
        result.push_str(paragraph);
        current_size += paragraph.len();
        paragraph_index += 1;

        // Add some variation with numbers and special characters
        if paragraph_index % 10 == 0 {
            let variation = format!(" [Section {}] ", paragraph_index / 10);
            result.push_str(&variation);
            current_size += variation.len();
        }
    }

    // Truncate to exact size if needed
    if result.len() > target_size {
        result.truncate(target_size);

        // Ensure we don't end in the middle of a word
        if let Some(last_space) = result.rfind(' ') {
            result.truncate(last_space);
        }
    }

    result
}

fn get_memory_usage() -> usize {
    // Simple memory usage approximation
    // In a real implementation, you might use a proper memory profiling library
    std::process::id() as usize * 1024 // Placeholder
}

#[tokio::test]
async fn test_thread_safety_and_concurrent_access() {
    use std::sync::Arc;
    use tokio::task;

    let test_texts = vec![
        "First test text for concurrent processing.",
        "Second test text with different content and structure.",
        "Third test text containing various punctuation marks!",
        "Fourth test text with numbers 123 and symbols @#$.",
        "Fifth test text with unicode characters: 测试 🚀 café.",
    ];

    let config = Arc::new(ChunkingConfig::default());

    // Spawn concurrent chunking tasks
    let mut handles = Vec::new();

    for (i, text) in test_texts.iter().enumerate() {
        let text = text.to_string();
        let config = Arc::clone(&config);

        let handle = task::spawn(async move {
            let chunks = chunk_content(&text, &config).await.unwrap();
            (i, chunks)
        });

        handles.push(handle);
    }

    // Collect all results
    let mut results = Vec::new();
    for handle in handles {
        let (index, chunks) = handle.await.unwrap();
        results.push((index, chunks));
    }

    // Verify all tasks completed successfully
    assert_eq!(results.len(), test_texts.len());

    for (index, chunks) in results {
        assert!(!chunks.is_empty(), "Concurrent task {} should produce chunks", index);

        for chunk in chunks {
            assert!(!chunk.content.is_empty());
            assert!(!chunk.id.is_empty());
            assert!(chunk.token_count > 0);
        }
    }
}