//! Example demonstrating topic-based chunking using the TextTiling algorithm
//!
//! This example shows how to use topic chunking for intelligent document segmentation
//! based on topic boundaries rather than fixed sizes.

use riptide_html::chunking::{create_strategy, ChunkingConfig, ChunkingMode};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Topic Chunking Example");
    println!("======================\n");

    // Sample document with clear topic shifts
    let document = r#"
        Introduction to Machine Learning

        Machine learning is a subset of artificial intelligence that focuses on algorithms.
        These algorithms can learn from and make predictions or decisions based on data.
        The field has grown tremendously in recent years due to increased computational power.
        Applications range from recommendation systems to autonomous vehicles.

        Deep Learning Fundamentals

        Deep learning uses neural networks with multiple layers to model complex patterns.
        Convolutional neural networks excel at image recognition and computer vision tasks.
        Recurrent neural networks are particularly effective for sequential data processing.
        Transformer architectures have revolutionized natural language processing applications.

        Climate Change and Environmental Impact

        Global warming continues to be one of the most pressing issues of our time.
        Rising sea levels threaten coastal communities around the world.
        Extreme weather events are becoming more frequent and severe.
        Renewable energy sources offer hope for reducing carbon emissions significantly.

        Economic Policy and Market Analysis

        Interest rates play a crucial role in economic stability and growth patterns.
        Inflation affects consumer purchasing power and business investment decisions.
        International trade agreements influence global supply chain operations.
        Monetary policy decisions impact currency values and market volatility.

        Healthcare Technology Advances

        Telemedicine has expanded access to healthcare services in remote areas.
        Medical imaging technologies enable earlier detection of diseases and conditions.
        Electronic health records improve care coordination between healthcare providers.
        Artificial intelligence assists in diagnosis and treatment recommendation systems.
    "#;

    // Configuration for topic chunking
    let config = ChunkingConfig {
        max_tokens: 1500,
        overlap_tokens: 0, // No overlap for topic chunks
        preserve_sentences: true,
        preserve_html_tags: false,
        min_chunk_size: 200,
        max_chunk_size: 8000,
    };

    println!("Document length: {} characters\n", document.len());

    // Example 1: Topic chunking enabled
    println!("1. Topic Chunking (Enabled)");
    println!("---------------------------");

    let topic_strategy = create_strategy(
        ChunkingMode::Topic {
            topic_chunking: true,
            window_size: 3,      // Analyze 3 sentences at a time
            smoothing_passes: 2, // Apply 2 smoothing passes
        },
        config.clone(),
    );

    let start = Instant::now();
    let topic_chunks = topic_strategy.chunk(document).await?;
    let topic_duration = start.elapsed();

    println!("Processing time: {}ms", topic_duration.as_millis());
    println!("Number of chunks: {}", topic_chunks.len());
    println!();

    for (i, chunk) in topic_chunks.iter().enumerate() {
        println!(
            "Topic Chunk {}: ({} chars, {} tokens)",
            i + 1,
            chunk.content.len(),
            chunk.token_count
        );
        println!("Quality Score: {:.3}", chunk.metadata.quality_score);
        println!("Topic Keywords: {:?}", chunk.metadata.topic_keywords);
        println!(
            "Content Preview: {}",
            chunk
                .content
                .lines()
                .take(2)
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .chars()
                .take(100)
                .collect::<String>()
                + "..."
        );
        println!();
    }

    // Example 2: Topic chunking disabled (fallback)
    println!("\n2. Topic Chunking (Disabled - Fallback)");
    println!("---------------------------------------");

    let fallback_strategy = create_strategy(
        ChunkingMode::Topic {
            topic_chunking: false, // Disabled - falls back to sliding window
            window_size: 3,
            smoothing_passes: 2,
        },
        config.clone(),
    );

    let start = Instant::now();
    let fallback_chunks = fallback_strategy.chunk(document).await?;
    let fallback_duration = start.elapsed();

    println!("Processing time: {}ms", fallback_duration.as_millis());
    println!("Number of chunks: {}", fallback_chunks.len());
    println!();

    for (i, chunk) in fallback_chunks.iter().enumerate() {
        println!(
            "Fallback Chunk {}: ({} chars, {} tokens)",
            i + 1,
            chunk.content.len(),
            chunk.token_count
        );
        println!(
            "Content Preview: {}",
            chunk.content.chars().take(80).collect::<String>() + "..."
        );
        println!();
    }

    // Example 3: Comparison with other strategies
    println!("\n3. Comparison with Other Strategies");
    println!("----------------------------------");

    let strategies = vec![
        (
            "Sliding Window",
            ChunkingMode::Sliding {
                window_size: 1000,
                overlap: 100,
            },
        ),
        (
            "Fixed Size",
            ChunkingMode::Fixed {
                size: 1000,
                by_tokens: false,
            },
        ),
        (
            "Sentence-based",
            ChunkingMode::Sentence { max_sentences: 5 },
        ),
    ];

    for (name, mode) in strategies {
        let strategy = create_strategy(mode, config.clone());
        let start = Instant::now();
        let chunks = strategy.chunk(document).await?;
        let duration = start.elapsed();

        println!(
            "{}: {} chunks in {}ms",
            name,
            chunks.len(),
            duration.as_millis()
        );
    }

    // Example 4: Performance with larger document
    println!("\n4. Performance Test with Larger Document");
    println!("---------------------------------------");

    // Generate larger document
    let mut large_doc = String::new();
    for i in 0..10 {
        large_doc.push_str(&format!("\n\nSection {}: ", i + 1));
        large_doc.push_str(document);
    }

    println!("Large document length: {} characters", large_doc.len());

    let start = Instant::now();
    let large_chunks = topic_strategy.chunk(&large_doc).await?;
    let large_duration = start.elapsed();

    println!(
        "Topic chunking: {} chunks in {}ms",
        large_chunks.len(),
        large_duration.as_millis()
    );
    println!(
        "Performance: {:.1} chars/ms",
        large_doc.len() as f64 / large_duration.as_millis() as f64
    );

    // Verify performance requirement
    if large_duration.as_millis() < 200 {
        println!("✓ Meets <200ms performance requirement");
    } else {
        println!("⚠ Exceeds 200ms performance requirement");
    }

    // Example 5: Topic quality analysis
    println!("\n5. Topic Quality Analysis");
    println!("------------------------");

    for (i, chunk) in topic_chunks.iter().enumerate() {
        let metadata = &chunk.metadata;
        println!(
            "Chunk {}: Quality={:.3}, Words={}, Sentences={}, Complete={}",
            i + 1,
            metadata.quality_score,
            metadata.word_count,
            metadata.sentence_count,
            metadata.has_complete_sentences
        );
    }

    println!("\nTopic chunking example completed successfully!");
    Ok(())
}
