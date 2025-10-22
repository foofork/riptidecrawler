//! Comprehensive tests for Week 3 chunking strategies
//!
//! This module tests all 5 chunking strategies:
//! - Sliding window: 1000 token chunks with 100 overlap
//! - Fixed-size: Various sizes (character and token-based)
//! - Sentence-based: Sentence boundary detection
//! - Regex-based: Custom pattern chunking
//! - HTML-aware: No mid-tag splits

use std::time::Instant;
use std::time::Duration;
use tokio_test;

// Import chunking functionality from the project
use riptide_core::strategies::chunking::{
    chunk_content, ChunkingConfig, ChunkingMode, ContentChunk, count_tokens
};

#[tokio::test]
async fn test_sliding_window_chunking_1000_tokens() {
    let text = generate_large_text(2000); // Generate text with ~2000 tokens
    let config = ChunkingConfig {
        mode: ChunkingMode::Sliding,
        token_max: 1000,
        overlap: 100,
        preserve_sentences: true,
        deterministic: true,
    };

    let chunks = chunk_content(&text, &config).await.unwrap();

    // Verify chunk properties
    assert!(!chunks.is_empty());

    for chunk in &chunks {
        // Each chunk should be close to 1000 tokens (allowing some variation for sentence boundaries)
        assert!(chunk.token_count <= 1200, "Chunk token count {} exceeds maximum", chunk.token_count);
        assert!(chunk.token_count >= 800, "Chunk token count {} too small", chunk.token_count);

        // Verify chunk metadata
        assert!(!chunk.id.is_empty());
        assert_eq!(chunk.metadata.chunk_type, "sliding");
        assert!(chunk.metadata.quality_score > 0.0);
    }

    // Verify overlap exists between consecutive chunks
    if chunks.len() > 1 {
        for i in 0..chunks.len() - 1 {
            let current_end = &chunks[i].content[chunks[i].content.len().saturating_sub(200)..];
            let next_start = &chunks[i + 1].content[..200.min(chunks[i + 1].content.len())];

            // Check for some overlap (approximately 100 tokens worth)
            let words_current: Vec<&str> = current_end.split_whitespace().collect();
            let words_next: Vec<&str> = next_start.split_whitespace().collect();

            let overlap_found = words_current.iter()
                .any(|word| words_next.contains(word));
            assert!(overlap_found, "No overlap found between chunks {} and {}", i, i + 1);
        }
    }
}

#[tokio::test]
async fn test_fixed_size_chunking_various_sizes() {
    let text = generate_large_text(1000);

    // Test character-based chunking
    let char_config = ChunkingConfig {
        mode: ChunkingMode::Fixed { size: 500, by_tokens: false },
        token_max: 1000,
        overlap: 0,
        preserve_sentences: true,
        deterministic: true,
    };

    let char_chunks = chunk_content(&text, &char_config).await.unwrap();

    for chunk in &char_chunks {
        // Character-based chunks should respect character limits
        assert!(chunk.content.len() <= 600, "Character chunk too large: {}", chunk.content.len());
        assert_eq!(chunk.metadata.chunk_type, "fixed_char");
    }

    // Test token-based chunking
    let token_config = ChunkingConfig {
        mode: ChunkingMode::Fixed { size: 200, by_tokens: true },
        token_max: 1000,
        overlap: 0,
        preserve_sentences: true,
        deterministic: true,
    };

    let token_chunks = chunk_content(&text, &token_config).await.unwrap();

    for chunk in &token_chunks {
        // Token-based chunks should respect token limits
        assert!(chunk.token_count <= 250, "Token chunk too large: {}", chunk.token_count);
        assert_eq!(chunk.metadata.chunk_type, "fixed_token");
    }
}

#[tokio::test]
async fn test_sentence_based_chunking() {
    let text = "This is the first sentence. This is the second sentence! Is this the third sentence? \
               This is the fourth sentence. This is the fifth sentence. This is the sixth sentence.";

    let config = ChunkingConfig {
        mode: ChunkingMode::Sentence { max_sentences: 3 },
        token_max: 1000,
        overlap: 0,
        preserve_sentences: true,
        deterministic: true,
    };

    let chunks = chunk_content(&text, &config).await.unwrap();

    // Should have 2 chunks (3 sentences each)
    assert_eq!(chunks.len(), 2);

    for chunk in &chunks {
        // Each chunk should have complete sentences
        assert!(chunk.metadata.has_complete_sentences);
        assert!(chunk.metadata.sentence_count <= 3);
        assert_eq!(chunk.metadata.chunk_type, "sentence");

        // Verify sentences end with proper punctuation
        let content = chunk.content.trim();
        assert!(content.ends_with('.') || content.ends_with('!') || content.ends_with('?'));
    }
}

#[tokio::test]
async fn test_regex_based_chunking() {
    let text = "## Chapter 1\nThis is chapter one content.\n\n## Chapter 2\nThis is chapter two content.\n\n## Chapter 3\nThis is chapter three content.";

    let config = ChunkingConfig {
        mode: ChunkingMode::Regex {
            pattern: r"## Chapter \d+".to_string(),
            min_chunk_size: 10
        },
        token_max: 1000,
        overlap: 0,
        preserve_sentences: true,
        deterministic: true,
    };

    let chunks = chunk_content(&text, &config).await.unwrap();

    // Should create chunks based on chapter boundaries
    assert!(!chunks.is_empty());

    for chunk in &chunks {
        assert!(chunk.metadata.chunk_type.starts_with("regex_"));
        // Each chunk should contain meaningful content
        assert!(chunk.content.len() >= 10);
    }
}

#[tokio::test]
async fn test_html_aware_chunking() {
    let html_text = r#"
    <html>
    <head><title>Test Document</title></head>
    <body>
        <div class="content">
            <p>This is a paragraph with <strong>bold text</strong> and <em>italic text</em>.</p>
            <ul>
                <li>First list item</li>
                <li>Second list item with <a href="https://example.com">a link</a></li>
            </ul>
            <table>
                <tr><td>Cell 1</td><td>Cell 2</td></tr>
                <tr><td>Cell 3</td><td>Cell 4</td></tr>
            </table>
        </div>
    </body>
    </html>
    "#;

    let config = ChunkingConfig {
        mode: ChunkingMode::Fixed { size: 100, by_tokens: false },
        token_max: 1000,
        overlap: 0,
        preserve_sentences: false, // Allow mid-word splits to test HTML handling
        deterministic: true,
    };

    let chunks = chunk_content(html_text, &config).await.unwrap();

    // Verify no chunks split HTML tags
    for chunk in &chunks {
        let content = &chunk.content;

        // Check for orphaned opening tags (< without corresponding >)
        let open_brackets = content.chars().filter(|&c| c == '<').count();
        let close_brackets = content.chars().filter(|&c| c == '>').count();

        // If there are brackets, they should be properly paired within the chunk
        // or the chunk should end/start at tag boundaries
        if content.contains('<') || content.contains('>') {
            assert!(
                !has_orphaned_tags(content),
                "Chunk contains orphaned HTML tags: {}",
                content
            );
        }
    }
}

#[tokio::test]
async fn test_performance_requirements_50kb_text() {
    let text = generate_text_with_size(50_000); // Generate exactly 50KB of text
    let config = ChunkingConfig::default();

    // Test all chunking strategies for performance
    let strategies = vec![
        ChunkingMode::Sliding,
        ChunkingMode::Fixed { size: 1000, by_tokens: false },
        ChunkingMode::Fixed { size: 500, by_tokens: true },
        ChunkingMode::Sentence { max_sentences: 10 },
        ChunkingMode::Regex {
            pattern: r"\n\n".to_string(),
            min_chunk_size: 100
        },
    ];

    for strategy in strategies {
        let config = ChunkingConfig {
            mode: strategy.clone(),
            ..config
        };

        let start = Instant::now();
        let chunks = chunk_content(&text, &config).await.unwrap();
        let elapsed = start.elapsed();

        // Performance requirement: ≤200ms for 50KB text
        assert!(
            elapsed <= Duration::from_millis(200),
            "Chunking strategy {:?} took {:?}, exceeding 200ms requirement",
            strategy, elapsed
        );

        assert!(!chunks.is_empty(), "No chunks produced for strategy {:?}", strategy);

        // Verify total content preservation
        let total_content: String = chunks.iter().map(|c| &c.content).collect::<Vec<_>>().join("");
        let original_words: Vec<&str> = text.split_whitespace().collect();
        let chunked_words: Vec<&str> = total_content.split_whitespace().collect();

        // Allow for some variation due to chunking boundaries
        let word_difference = if original_words.len() > chunked_words.len() {
            original_words.len() - chunked_words.len()
        } else {
            chunked_words.len() - original_words.len()
        };

        assert!(
            word_difference < original_words.len() / 20, // Less than 5% difference
            "Too much content lost/gained during chunking: {} vs {} words",
            original_words.len(),
            chunked_words.len()
        );
    }
}

#[tokio::test]
async fn test_edge_cases() {
    // Test empty text
    let empty_config = ChunkingConfig::default();
    let empty_chunks = chunk_content("", &empty_config).await.unwrap();
    assert!(empty_chunks.is_empty());

    // Test very short text
    let short_chunks = chunk_content("Short", &empty_config).await.unwrap();
    assert_eq!(short_chunks.len(), 1);
    assert_eq!(short_chunks[0].content, "Short");

    // Test text with only whitespace
    let whitespace_chunks = chunk_content("   \n\t  \n  ", &empty_config).await.unwrap();
    assert!(whitespace_chunks.is_empty() || whitespace_chunks[0].content.trim().is_empty());

    // Test text with special characters and Unicode
    let unicode_text = "Hello 世界! This is a test with émojis 🚀 and special chars: @#$%^&*()";
    let unicode_chunks = chunk_content(&unicode_text, &empty_config).await.unwrap();
    assert!(!unicode_chunks.is_empty());
    assert!(unicode_chunks[0].content.contains("世界"));
    assert!(unicode_chunks[0].content.contains("🚀"));

    // Test very large single token
    let large_token = "a".repeat(5000);
    let large_token_chunks = chunk_content(&large_token, &empty_config).await.unwrap();
    assert!(!large_token_chunks.is_empty());
}

#[tokio::test]
async fn test_deterministic_chunking() {
    let text = generate_large_text(500);
    let config = ChunkingConfig {
        deterministic: true,
        ..ChunkingConfig::default()
    };

    // Run chunking multiple times
    let chunks1 = chunk_content(&text, &config).await.unwrap();
    let chunks2 = chunk_content(&text, &config).await.unwrap();
    let chunks3 = chunk_content(&text, &config).await.unwrap();

    // Results should be identical
    assert_eq!(chunks1.len(), chunks2.len());
    assert_eq!(chunks1.len(), chunks3.len());

    for i in 0..chunks1.len() {
        assert_eq!(chunks1[i].content, chunks2[i].content);
        assert_eq!(chunks1[i].content, chunks3[i].content);
        assert_eq!(chunks1[i].id, chunks2[i].id);
        assert_eq!(chunks1[i].id, chunks3[i].id);
        assert_eq!(chunks1[i].start_pos, chunks2[i].start_pos);
        assert_eq!(chunks1[i].start_pos, chunks3[i].start_pos);
    }
}

#[tokio::test]
async fn test_chunk_quality_scoring() {
    let high_quality_text = "This is a well-written paragraph with complete sentences. \
                            It contains proper grammar and punctuation. \
                            The content is meaningful and coherent throughout.";

    let low_quality_text = "short fragment...";

    let config = ChunkingConfig::default();

    let high_quality_chunks = chunk_content(&high_quality_text, &config).await.unwrap();
    let low_quality_chunks = chunk_content(&low_quality_text, &config).await.unwrap();

    assert!(!high_quality_chunks.is_empty());
    assert!(!low_quality_chunks.is_empty());

    // High quality text should have better quality scores
    assert!(
        high_quality_chunks[0].metadata.quality_score > low_quality_chunks[0].metadata.quality_score,
        "High quality chunk score {} should be greater than low quality score {}",
        high_quality_chunks[0].metadata.quality_score,
        low_quality_chunks[0].metadata.quality_score
    );
}

// Helper functions

fn generate_large_text(word_count: usize) -> String {
    let words = vec![
        "lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing", "elit",
        "sed", "do", "eiusmod", "tempor", "incididunt", "ut", "labore", "et", "dolore",
        "magna", "aliqua", "enim", "ad", "minim", "veniam", "quis", "nostrud",
        "exercitation", "ullamco", "laboris", "nisi", "aliquip", "ex", "ea", "commodo",
        "consequat", "duis", "aute", "irure", "in", "reprehenderit", "voluptate",
        "velit", "esse", "cillum", "fugiat", "nulla", "pariatur", "excepteur", "sint",
        "occaecat", "cupidatat", "non", "proident", "sunt", "culpa", "qui", "officia",
        "deserunt", "mollit", "anim", "id", "est", "laborum"
    ];

    let mut text = String::new();
    for i in 0..word_count {
        if i > 0 {
            text.push(' ');
        }
        text.push_str(words[i % words.len()]);

        // Add punctuation periodically for sentence structure
        if (i + 1) % 15 == 0 {
            text.push('.');
        } else if (i + 1) % 30 == 0 {
            text.push_str("!\n\n");
        }
    }

    text
}

fn generate_text_with_size(target_bytes: usize) -> String {
    let base_text = generate_large_text(1000);
    let base_size = base_text.len();

    if base_size >= target_bytes {
        return base_text[..target_bytes].to_string();
    }

    let repetitions = (target_bytes / base_size) + 1;
    let mut result = String::with_capacity(target_bytes + 1000);

    for _ in 0..repetitions {
        result.push_str(&base_text);
        result.push(' ');

        if result.len() >= target_bytes {
            break;
        }
    }

    result.truncate(target_bytes);
    result
}

fn has_orphaned_tags(content: &str) -> bool {
    let mut tag_stack = Vec::new();
    let mut chars = content.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '<' {
            let mut tag = String::new();

            // Read the tag
            while let Some(&next_ch) = chars.peek() {
                chars.next();
                tag.push(next_ch);
                if next_ch == '>' {
                    break;
                }
            }

            if !tag.ends_with('>') {
                return true; // Orphaned opening bracket
            }

            // Parse tag name
            let tag_content = &tag[..tag.len()-1]; // Remove '>'
            if tag_content.starts_with('/') {
                // Closing tag
                let tag_name = tag_content[1..].split_whitespace().next().unwrap_or("");
                if let Some(last_opened) = tag_stack.last() {
                    if last_opened == tag_name {
                        tag_stack.pop();
                    }
                }
            } else if !tag_content.ends_with('/') {
                // Opening tag (not self-closing)
                let tag_name = tag_content.split_whitespace().next().unwrap_or("");
                if !tag_name.is_empty() {
                    tag_stack.push(tag_name.to_string());
                }
            }
        }
    }

    // If stack is not empty, we have unmatched opening tags
    !tag_stack.is_empty()
}

#[cfg(test)]
mod benchmark_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn benchmark_all_strategies() {
        let sizes = vec![1_000, 10_000, 50_000]; // Different text sizes
        let strategies = vec![
            ("sliding", ChunkingMode::Sliding),
            ("fixed_char", ChunkingMode::Fixed { size: 1000, by_tokens: false }),
            ("fixed_token", ChunkingMode::Fixed { size: 500, by_tokens: true }),
            ("sentence", ChunkingMode::Sentence { max_sentences: 10 }),
            ("regex", ChunkingMode::Regex {
                pattern: r"\n\n".to_string(),
                min_chunk_size: 100
            }),
        ];

        println!("Chunking Strategy Performance Benchmark");
        println!("========================================");

        for size in sizes {
            let text = generate_text_with_size(size);
            println!("\nText size: {} bytes", size);
            println!("Strategy\t\tTime (ms)\tChunks\tAvg Chunk Size");
            println!("--------\t\t---------\t------\t--------------");

            for (name, strategy) in &strategies {
                let config = ChunkingConfig {
                    mode: strategy.clone(),
                    ..ChunkingConfig::default()
                };

                let start = Instant::now();
                let chunks = chunk_content(&text, &config).await.unwrap();
                let elapsed = start.elapsed();

                let avg_chunk_size = if !chunks.is_empty() {
                    chunks.iter().map(|c| c.content.len()).sum::<usize>() / chunks.len()
                } else {
                    0
                };

                println!(
                    "{}\t\t{:.2}\t\t{}\t{}",
                    name,
                    elapsed.as_secs_f64() * 1000.0,
                    chunks.len(),
                    avg_chunk_size
                );

                // Ensure we meet performance requirements for 50KB
                if size == 50_000 {
                    assert!(
                        elapsed <= Duration::from_millis(200),
                        "Strategy {} exceeded 200ms requirement with {:.2}ms",
                        name,
                        elapsed.as_secs_f64() * 1000.0
                    );
                }
            }
        }
    }
}