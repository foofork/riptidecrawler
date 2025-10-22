//! Integration tests for strategy registration and trait implementations
//!
//! Tests for Week 3 integration features including:
//! - Strategy registration and lookup
//! - Trait implementations and polymorphism
//! - Backward compatibility
//! - End-to-end workflows

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio_test;

// Import core functionality
use riptide_core::strategies::chunking::{
    chunk_content, ChunkingConfig, ChunkingMode, ContentChunk, count_tokens
};
use riptide_html::{
    HtmlProcessor, DefaultHtmlProcessor, ProcessingError, ProcessingResult,
    ExtractedContent, RegexPattern, TableExtractionMode, ChunkingMode as HtmlChunkingMode,
    processor::{TableData, ProcessingStats}
};

/// Test trait for strategy registration
#[async_trait]
pub trait ChunkingStrategy: Send + Sync {
    async fn chunk(&self, content: &str, config: &ChunkingConfig) -> anyhow::Result<Vec<ContentChunk>>;
    fn strategy_name(&self) -> &'static str;
    fn supports_config(&self, config: &ChunkingConfig) -> bool;
}

/// Strategy registry for managing chunking strategies
pub struct StrategyRegistry {
    strategies: HashMap<String, Arc<dyn ChunkingStrategy>>,
}

impl StrategyRegistry {
    pub fn new() -> Self {
        Self {
            strategies: HashMap::new(),
        }
    }

    pub fn register<T: ChunkingStrategy + 'static>(&mut self, strategy: T) {
        let name = strategy.strategy_name().to_string();
        self.strategies.insert(name, Arc::new(strategy));
    }

    pub fn get_strategy(&self, name: &str) -> Option<Arc<dyn ChunkingStrategy>> {
        self.strategies.get(name).cloned()
    }

    pub fn list_strategies(&self) -> Vec<String> {
        self.strategies.keys().cloned().collect()
    }

    pub async fn chunk_with_strategy(
        &self,
        strategy_name: &str,
        content: &str,
        config: &ChunkingConfig,
    ) -> anyhow::Result<Vec<ContentChunk>> {
        if let Some(strategy) = self.get_strategy(strategy_name) {
            strategy.chunk(content, config).await
        } else {
            Err(anyhow::anyhow!("Strategy '{}' not found", strategy_name))
        }
    }
}

/// Built-in sliding window strategy implementation
pub struct SlidingWindowStrategy;

#[async_trait]
impl ChunkingStrategy for SlidingWindowStrategy {
    async fn chunk(&self, content: &str, config: &ChunkingConfig) -> anyhow::Result<Vec<ContentChunk>> {
        let updated_config = ChunkingConfig {
            mode: ChunkingMode::Sliding,
            ..config.clone()
        };
        chunk_content(content, &updated_config).await
    }

    fn strategy_name(&self) -> &'static str {
        "sliding_window"
    }

    fn supports_config(&self, config: &ChunkingConfig) -> bool {
        matches!(config.mode, ChunkingMode::Sliding)
    }
}

/// Built-in fixed size strategy implementation
pub struct FixedSizeStrategy;

#[async_trait]
impl ChunkingStrategy for FixedSizeStrategy {
    async fn chunk(&self, content: &str, config: &ChunkingConfig) -> anyhow::Result<Vec<ContentChunk>> {
        match &config.mode {
            ChunkingMode::Fixed { .. } => chunk_content(content, config).await,
            _ => {
                // Default to fixed size if not specified
                let updated_config = ChunkingConfig {
                    mode: ChunkingMode::Fixed { size: config.token_max, by_tokens: true },
                    ..config.clone()
                };
                chunk_content(content, &updated_config).await
            }
        }
    }

    fn strategy_name(&self) -> &'static str {
        "fixed_size"
    }

    fn supports_config(&self, config: &ChunkingConfig) -> bool {
        matches!(config.mode, ChunkingMode::Fixed { .. })
    }
}

/// Custom strategy for testing extensibility
pub struct CustomTestStrategy {
    pub chunk_size: usize,
    pub overlap_ratio: f64,
}

#[async_trait]
impl ChunkingStrategy for CustomTestStrategy {
    async fn chunk(&self, content: &str, _config: &ChunkingConfig) -> anyhow::Result<Vec<ContentChunk>> {
        let words: Vec<&str> = content.split_whitespace().collect();
        let mut chunks = Vec::new();
        let mut start = 0;
        let mut chunk_index = 0;

        let overlap_size = (self.chunk_size as f64 * self.overlap_ratio) as usize;

        while start < words.len() {
            let end = (start + self.chunk_size).min(words.len());
            let chunk_words = &words[start..end];
            let chunk_content = chunk_words.join(" ");

            let start_pos = if start == 0 { 0 } else {
                content.find(chunk_words[0]).unwrap_or(0)
            };

            chunks.push(ContentChunk {
                id: format!("custom_{}_{}", chunk_index, start_pos),
                content: chunk_content.clone(),
                start_pos,
                end_pos: start_pos + chunk_content.len(),
                token_count: count_tokens(&chunk_content),
                chunk_index,
                total_chunks: 0, // Will be updated later
                metadata: riptide_core::strategies::chunking::ChunkMetadata {
                    quality_score: 0.8,
                    sentence_count: chunk_content.matches(['.', '!', '?']).count(),
                    word_count: chunk_words.len(),
                    has_complete_sentences: chunk_content.trim().ends_with('.'),
                    topic_keywords: vec!["custom".to_string()],
                    chunk_type: "custom_test".to_string(),
                },
            });

            if end >= words.len() {
                break;
            }

            start = if overlap_size > 0 && end > overlap_size {
                end - overlap_size
            } else {
                end
            };
            chunk_index += 1;
        }

        // Update total chunk count
        let total_chunks = chunks.len();
        for chunk in &mut chunks {
            chunk.total_chunks = total_chunks;
        }

        Ok(chunks)
    }

    fn strategy_name(&self) -> &'static str {
        "custom_test"
    }

    fn supports_config(&self, _config: &ChunkingConfig) -> bool {
        true // This strategy is flexible
    }
}

#[tokio::test]
async fn test_strategy_registration_and_lookup() {
    let mut registry = StrategyRegistry::new();

    // Register built-in strategies
    registry.register(SlidingWindowStrategy);
    registry.register(FixedSizeStrategy);

    // Register custom strategy
    registry.register(CustomTestStrategy {
        chunk_size: 50,
        overlap_ratio: 0.2,
    });

    // Test strategy listing
    let strategies = registry.list_strategies();
    assert!(strategies.contains(&"sliding_window".to_string()));
    assert!(strategies.contains(&"fixed_size".to_string()));
    assert!(strategies.contains(&"custom_test".to_string()));
    assert_eq!(strategies.len(), 3);

    // Test strategy lookup
    let sliding_strategy = registry.get_strategy("sliding_window");
    assert!(sliding_strategy.is_some());

    let nonexistent_strategy = registry.get_strategy("nonexistent");
    assert!(nonexistent_strategy.is_none());

    // Test strategy execution through registry
    let test_text = "This is a test text for strategy registration. It should be chunked properly by different strategies.";
    let config = ChunkingConfig::default();

    let sliding_chunks = registry.chunk_with_strategy("sliding_window", &test_text, &config).await.unwrap();
    assert!(!sliding_chunks.is_empty());

    let fixed_chunks = registry.chunk_with_strategy("fixed_size", &test_text, &config).await.unwrap();
    assert!(!fixed_chunks.is_empty());

    let custom_chunks = registry.chunk_with_strategy("custom_test", &test_text, &config).await.unwrap();
    assert!(!custom_chunks.is_empty());

    // Verify custom strategy behavior
    assert!(custom_chunks.iter().all(|c| c.metadata.chunk_type == "custom_test"));
}

#[tokio::test]
async fn test_trait_implementations_polymorphism() {
    let strategies: Vec<Arc<dyn ChunkingStrategy>> = vec![
        Arc::new(SlidingWindowStrategy),
        Arc::new(FixedSizeStrategy),
        Arc::new(CustomTestStrategy {
            chunk_size: 25,
            overlap_ratio: 0.1,
        }),
    ];

    let test_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.";
    let config = ChunkingConfig {
        token_max: 500,
        overlap: 50,
        preserve_sentences: true,
        deterministic: true,
        mode: ChunkingMode::Sliding, // Default mode
    };

    // Test polymorphic behavior
    for strategy in strategies {
        let strategy_name = strategy.strategy_name();
        println!("Testing strategy: {}", strategy_name);

        let chunks = strategy.chunk(&test_text, &config).await.unwrap();
        assert!(!chunks.is_empty(), "Strategy {} produced no chunks", strategy_name);

        // Verify chunk properties
        for chunk in &chunks {
            assert!(!chunk.content.is_empty());
            assert!(!chunk.id.is_empty());
            assert!(chunk.token_count > 0);
            assert!(chunk.start_pos <= chunk.end_pos);
        }

        // Test supports_config method
        let supports = strategy.supports_config(&config);
        println!("Strategy {} supports config: {}", strategy_name, supports);
    }
}

#[tokio::test]
async fn test_html_processor_trait_implementation() {
    let processor = DefaultHtmlProcessor::default();

    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Test Page</title>
        <meta name="description" content="Test description">
    </head>
    <body>
        <h1>Main Title</h1>
        <p>This is a test paragraph with some content.</p>
        <ul>
            <li>First item</li>
            <li>Second item</li>
        </ul>
        <table>
            <tr><th>Name</th><th>Age</th></tr>
            <tr><td>John</td><td>30</td></tr>
            <tr><td>Jane</td><td>25</td></tr>
        </table>
    </body>
    </html>
    "#;

    // Test CSS extraction
    let mut css_selectors = HashMap::new();
    css_selectors.insert("title".to_string(), "title".to_string());
    css_selectors.insert("description".to_string(), "meta[name='description']".to_string());
    css_selectors.insert("content".to_string(), "p, li".to_string());

    let css_result = processor.extract_with_css(html, "https://example.com", &css_selectors).await;
    assert!(css_result.is_ok(), "CSS extraction failed: {:?}", css_result.err());

    // Test regex extraction
    let regex_patterns = vec![
        RegexPattern {
            name: "title".to_string(),
            pattern: r"<title>(.*?)</title>".to_string(),
            field: "title".to_string(),
            required: true,
        },
        RegexPattern {
            name: "paragraphs".to_string(),
            pattern: r"<p>(.*?)</p>".to_string(),
            field: "content".to_string(),
            required: false,
        },
    ];

    let regex_result = processor.extract_with_regex(html, "https://example.com", &regex_patterns).await;
    assert!(regex_result.is_ok(), "Regex extraction failed: {:?}", regex_result.err());

    // Test table extraction
    let tables_result = processor.extract_tables(html, TableExtractionMode::All).await;
    assert!(tables_result.is_ok(), "Table extraction failed: {:?}", tables_result.err());
    let tables = tables_result.unwrap();
    assert_eq!(tables.len(), 1);
    assert_eq!(tables[0].headers, vec!["Name", "Age"]);
    assert_eq!(tables[0].rows.len(), 2);

    // Test content chunking
    let text_content = "This is a long text that needs to be chunked into smaller pieces for processing. Each chunk should maintain reasonable boundaries and preserve meaning.";
    let chunking_result = processor.chunk_content(text_content, HtmlChunkingMode::default()).await;
    assert!(chunking_result.is_ok(), "Content chunking failed: {:?}", chunking_result.err());

    // Test confidence scoring
    let confidence = processor.confidence_score(html);
    assert!(confidence > 0.0 && confidence <= 1.0, "Invalid confidence score: {}", confidence);

    // Test processor name
    assert_eq!(processor.processor_name(), "default_html_processor");
}

#[tokio::test]
async fn test_backward_compatibility() {
    // Test that old chunking configurations still work
    let legacy_config = ChunkingConfig {
        mode: ChunkingMode::Fixed { size: 1000, by_tokens: false },
        token_max: 1200,
        overlap: 100,
        preserve_sentences: false,
        deterministic: false,
    };

    let test_text = "This is a test for backward compatibility. The old configuration should still work with the new system.";
    let chunks = chunk_content(&test_text, &legacy_config).await.unwrap();
    assert!(!chunks.is_empty());

    // Test that default values are still valid
    let default_config = ChunkingConfig::default();
    let default_chunks = chunk_content(&test_text, &default_config).await.unwrap();
    assert!(!default_chunks.is_empty());

    // Test HTML processor backward compatibility
    let processor = DefaultHtmlProcessor::default();
    let simple_html = "<html><body><p>Simple content</p></body></html>";

    let chunking_result = processor.chunk_content("Simple text", HtmlChunkingMode::default()).await;
    assert!(chunking_result.is_ok());

    // Test that confidence scoring works with minimal HTML
    let confidence = processor.confidence_score(simple_html);
    assert!(confidence >= 0.0);
}

#[tokio::test]
async fn test_end_to_end_workflow() {
    // Test complete workflow from HTML input to chunked output
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Research Article</title>
        <meta name="description" content="A comprehensive research article">
    </head>
    <body>
        <article>
            <h1>Introduction to Advanced Topics</h1>
            <p>This article explores several advanced concepts in computer science. The first section covers algorithmic complexity and its implications for software design.</p>

            <h2>Section 1: Algorithmic Complexity</h2>
            <p>Algorithmic complexity is a fundamental concept that every software engineer should understand. It helps us analyze the efficiency of our solutions and make informed decisions about implementation strategies.</p>

            <p>There are several types of complexity analysis including time complexity, space complexity, and communication complexity. Each type provides different insights into algorithm performance.</p>

            <h2>Section 2: Data Structures</h2>
            <p>Choosing the right data structure is crucial for optimal performance. Arrays provide constant-time access but have fixed size. Linked lists offer dynamic sizing but slower access times.</p>

            <p>More advanced structures like hash tables, binary trees, and graphs each have their own trade-offs and use cases. Understanding these trade-offs is essential for system design.</p>

            <h2>Conclusion</h2>
            <p>In conclusion, mastering these fundamental concepts provides a solid foundation for tackling more complex challenges in software development.</p>
        </article>
    </body>
    </html>
    "#;

    // Step 1: Process HTML and extract content
    let processor = DefaultHtmlProcessor::default();

    let mut css_selectors = HashMap::new();
    css_selectors.insert("title".to_string(), "title".to_string());
    css_selectors.insert("content".to_string(), "article p".to_string());
    css_selectors.insert("headings".to_string(), "h1, h2, h3".to_string());

    let extracted = processor.extract_with_css(html, "https://example.com/article", &css_selectors).await.unwrap();
    assert!(!extracted.content.is_empty());
    assert_eq!(extracted.title, "Research Article");

    // Step 2: Chunk the extracted content using multiple strategies
    let mut registry = StrategyRegistry::new();
    registry.register(SlidingWindowStrategy);
    registry.register(FixedSizeStrategy);
    registry.register(CustomTestStrategy {
        chunk_size: 30,
        overlap_ratio: 0.15,
    });

    let config = ChunkingConfig {
        token_max: 200,
        overlap: 30,
        preserve_sentences: true,
        deterministic: true,
        mode: ChunkingMode::Sliding,
    };

    // Test each strategy on the extracted content
    for strategy_name in registry.list_strategies() {
        let chunks = registry.chunk_with_strategy(&strategy_name, &extracted.content, &config).await.unwrap();

        assert!(!chunks.is_empty(), "Strategy {} produced no chunks", strategy_name);

        // Verify chunk quality
        for chunk in &chunks {
            assert!(chunk.token_count <= config.token_max + 50); // Allow some tolerance
            assert!(chunk.metadata.quality_score > 0.0);
            assert!(!chunk.content.trim().is_empty());
        }

        println!("Strategy {}: {} chunks, avg quality: {:.2}",
                strategy_name,
                chunks.len(),
                chunks.iter().map(|c| c.metadata.quality_score).sum::<f64>() / chunks.len() as f64);
    }

    // Step 3: Extract structured data
    let tables = processor.extract_tables(html, TableExtractionMode::All).await.unwrap();
    // No tables in this example, but the extraction should work without error

    // Step 4: Verify the complete workflow maintains content integrity
    let sliding_chunks = registry.chunk_with_strategy("sliding_window", &extracted.content, &config).await.unwrap();

    // Reconstruct content from chunks (without overlap)
    let mut reconstructed = String::new();
    for (i, chunk) in sliding_chunks.iter().enumerate() {
        if i == 0 {
            reconstructed.push_str(&chunk.content);
        } else {
            // For sliding window, we need to handle overlap
            // This is a simplified reconstruction for testing
            let overlap_size = config.overlap;
            let words: Vec<&str> = chunk.content.split_whitespace().collect();
            if words.len() > overlap_size {
                let new_content = words[overlap_size..].join(" ");
                reconstructed.push(' ');
                reconstructed.push_str(&new_content);
            }
        }
    }

    // The reconstructed content should contain key information from the original
    assert!(reconstructed.contains("algorithmic complexity") || reconstructed.contains("Algorithmic complexity"));
    assert!(reconstructed.contains("data structure") || reconstructed.contains("Data structure"));
}

#[tokio::test]
async fn test_error_handling_and_recovery() {
    let mut registry = StrategyRegistry::new();
    registry.register(SlidingWindowStrategy);

    // Test error handling for invalid strategy
    let result = registry.chunk_with_strategy("nonexistent", "test content", &ChunkingConfig::default()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));

    // Test HTML processor error handling
    let processor = DefaultHtmlProcessor::default();

    // Test with malformed HTML
    let malformed_html = "<html><body><p>Unclosed paragraph<div>Nested without closing</body>";
    let css_selectors = HashMap::new();

    // Should handle gracefully and not panic
    let result = processor.extract_with_css(&malformed_html, "https://example.com", &css_selectors).await;
    // The extraction might succeed or fail depending on implementation, but shouldn't panic

    // Test with invalid regex pattern
    let invalid_regex_patterns = vec![
        RegexPattern {
            name: "invalid".to_string(),
            pattern: "[".to_string(), // Invalid regex
            field: "content".to_string(),
            required: true,
        },
    ];

    let regex_result = processor.extract_with_regex("<html></html>", "https://example.com", &invalid_regex_patterns).await;
    // Should return an error for invalid regex
    assert!(regex_result.is_err());

    // Test chunking with extreme configurations
    let extreme_config = ChunkingConfig {
        mode: ChunkingMode::Fixed { size: 0, by_tokens: true }, // Zero size
        token_max: 0,
        overlap: 1000, // Overlap larger than max
        preserve_sentences: true,
        deterministic: true,
    };

    let extreme_result = registry.chunk_with_strategy("sliding_window", "test content", &extreme_config).await;
    // Should handle gracefully, either by producing reasonable chunks or returning an error
    match extreme_result {
        Ok(chunks) => {
            // If it succeeds, chunks should be reasonable
            for chunk in chunks {
                assert!(!chunk.content.is_empty());
            }
        }
        Err(_) => {
            // Error is acceptable for extreme configurations
        }
    }
}

#[tokio::test]
async fn test_concurrent_strategy_execution() {
    let mut registry = StrategyRegistry::new();
    registry.register(SlidingWindowStrategy);
    registry.register(FixedSizeStrategy);
    registry.register(CustomTestStrategy {
        chunk_size: 20,
        overlap_ratio: 0.1,
    });

    let registry = Arc::new(registry);
    let test_text = "This is a concurrent test text that will be processed by multiple strategies simultaneously. Each strategy should produce valid results even when executed concurrently.";
    let config = ChunkingConfig::default();

    // Execute all strategies concurrently
    let mut handles = Vec::new();

    for strategy_name in ["sliding_window", "fixed_size", "custom_test"] {
        let registry_clone = Arc::clone(&registry);
        let text_clone = test_text.to_string();
        let config_clone = config.clone();
        let strategy_name = strategy_name.to_string();

        let handle = tokio::spawn(async move {
            let result = registry_clone.chunk_with_strategy(&strategy_name, &text_clone, &config_clone).await;
            (strategy_name, result)
        });

        handles.push(handle);
    }

    // Collect results
    let mut results = Vec::new();
    for handle in handles {
        let (strategy_name, result) = handle.await.unwrap();
        results.push((strategy_name, result));
    }

    // Verify all strategies completed successfully
    for (strategy_name, result) in results {
        assert!(result.is_ok(), "Strategy {} failed in concurrent execution", strategy_name);
        let chunks = result.unwrap();
        assert!(!chunks.is_empty(), "Strategy {} produced no chunks", strategy_name);
    }
}