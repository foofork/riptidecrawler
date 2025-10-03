//! Simple chunking test to verify basic functionality

use riptide_html::chunking::{create_strategy, ChunkingConfig, ChunkingMode};

#[tokio::test]
async fn test_basic_chunking_functionality() {
    let config = ChunkingConfig::default();
    let text = "This is a test document. It has multiple sentences. Each sentence contains meaningful content. The document should be chunked appropriately.";

    // Test sliding window chunking
    let strategy = create_strategy(ChunkingMode::default(), config.clone());
    let chunks = strategy.chunk(text).await.unwrap();
    assert!(!chunks.is_empty());
    assert_eq!(chunks[0].chunk_index, 0);
    println!("✓ Sliding window chunking works");

    // Test fixed size chunking
    let strategy = create_strategy(
        ChunkingMode::Fixed {
            size: 50,
            by_tokens: false,
        },
        config.clone(),
    );
    let chunks = strategy.chunk(text).await.unwrap();
    assert!(!chunks.is_empty());
    println!("✓ Fixed size chunking works");

    // Test sentence chunking
    let strategy = create_strategy(ChunkingMode::Sentence { max_sentences: 2 }, config.clone());
    let chunks = strategy.chunk(text).await.unwrap();
    assert!(!chunks.is_empty());
    println!("✓ Sentence chunking works");

    // Test regex chunking
    let strategy = create_strategy(
        ChunkingMode::Regex {
            pattern: r"\.".to_string(),
            min_chunk_size: 10,
        },
        config.clone(),
    );
    let chunks = strategy.chunk(text).await.unwrap();
    assert!(!chunks.is_empty());
    println!("✓ Regex chunking works");

    // Test HTML-aware chunking with plain text (should fall back)
    let strategy = create_strategy(
        ChunkingMode::HtmlAware {
            preserve_blocks: true,
            preserve_structure: false,
        },
        config.clone(),
    );
    let chunks = strategy.chunk(text).await.unwrap();
    assert!(!chunks.is_empty());
    println!("✓ HTML-aware chunking (text fallback) works");

    // Test HTML-aware chunking with actual HTML
    let html = r#"<div><p>First paragraph.</p><p>Second paragraph.</p></div>"#;
    let chunks = strategy.chunk(html).await.unwrap();
    assert!(!chunks.is_empty());
    println!("✓ HTML-aware chunking (HTML parsing) works");

    println!("\n🎉 All chunking strategies working correctly!");
}
