//! Comprehensive tests for enhanced topic chunking with improved algorithms
//!
//! Tests the enhanced TextTiling implementation with:
//! - Enhanced coherence scoring (cosine + Jaccard + TF distribution)
//! - Improved valley detection with hysteresis
//! - Performance optimizations and fallback mechanisms
//! - Deterministic boundary detection

use riptide_html::chunking::topic::TopicChunker;
use riptide_html::chunking::{create_strategy, ChunkingConfig, ChunkingMode, ChunkingStrategy};
use std::time::Instant;

/// Test enhanced coherence scoring produces better boundaries than basic cosine similarity
#[tokio::test]
async fn test_enhanced_coherence_scoring() {
    let config = ChunkingConfig::default();
    let chunker = TopicChunker::new_without_fallback(3, 2, config);

    // Create text with clear topic transitions
    let text = "
        Machine learning algorithms are computational methods that learn patterns from data.
        Neural networks use interconnected nodes to process information like the human brain.
        Deep learning architectures can automatically extract features from raw input data.
        Training algorithms adjust weights and biases to minimize prediction errors.

        Climate change refers to long-term shifts in global weather patterns and temperatures.
        Greenhouse gases trap heat in Earth's atmosphere causing global warming effects.
        Carbon emissions from fossil fuels are primary contributors to climate change.
        Environmental policies aim to reduce emissions and promote sustainable practices.

        Economic growth depends on factors like productivity, investment, and innovation.
        Market forces determine prices through supply and demand interactions.
        Fiscal policy involves government spending and taxation to influence the economy.
        Monetary policy controls money supply and interest rates to manage inflation.
    ";

    let start = Instant::now();
    let chunks = chunker.chunk(text).await.unwrap();
    let duration = start.elapsed();

    println!(
        "Enhanced coherence: {} chunks in {}ms",
        chunks.len(),
        duration.as_millis()
    );

    // Should identify the three main topics (ML, climate, economics)
    assert!(
        chunks.len() >= 2 && chunks.len() <= 4,
        "Should detect 2-4 topic boundaries, found {}",
        chunks.len()
    );

    // Should be fast
    assert!(
        duration.as_millis() < 50,
        "Enhanced scoring should be fast: {}ms",
        duration.as_millis()
    );

    // Verify topic coherence by checking keywords
    let topic_keywords: Vec<Vec<String>> = chunks
        .iter()
        .map(|c| c.metadata.topic_keywords.clone())
        .collect();

    // First chunk should be about machine learning
    let ml_terms = [
        "machine",
        "learning",
        "neural",
        "deep",
        "algorithms",
        "training",
    ];
    assert!(
        topic_keywords[0]
            .iter()
            .any(|k| ml_terms.contains(&k.as_str())),
        "First chunk should contain ML terms: {:?}",
        topic_keywords[0]
    );

    // Later chunks should contain different topic terms
    let all_keywords: std::collections::HashSet<String> = chunks
        .iter()
        .flat_map(|c| c.metadata.topic_keywords.iter().cloned())
        .collect();

    // Should have diverse vocabulary across chunks
    assert!(
        all_keywords.len() > 8,
        "Should extract diverse keywords: {:?}",
        all_keywords
    );
}

/// Test valley detection with hysteresis prevents boundary oscillation
#[tokio::test]
async fn test_hysteresis_valley_detection() {
    let config = ChunkingConfig::default();
    let chunker = TopicChunker::new_without_fallback(2, 1, config);

    // Create text with subtle topic boundaries (tests hysteresis)
    let text = "
        Artificial intelligence systems can process complex data patterns efficiently.
        Machine learning models require training data to learn statistical relationships.
        Deep neural networks use multiple layers for feature extraction and classification.

        Computer vision applications analyze images to detect objects and patterns.
        Image recognition systems use convolutional networks for spatial feature detection.
        Visual processing algorithms can identify faces, objects, and scene elements.

        Natural language processing handles text analysis and understanding tasks.
        Text classification models categorize documents based on content and meaning.
        Language models generate coherent text by learning linguistic patterns.
    ";

    let start = Instant::now();
    let chunks = chunker.chunk(text).await.unwrap();
    let duration = start.elapsed();

    println!(
        "Hysteresis detection: {} chunks in {}ms",
        chunks.len(),
        duration.as_millis()
    );

    // Should detect stable boundaries without oscillation
    assert!(
        chunks.len() >= 2 && chunks.len() <= 4,
        "Hysteresis should produce stable boundaries: {} chunks",
        chunks.len()
    );

    // Verify boundary stability by running multiple times
    for i in 0..5 {
        let test_chunks = chunker.chunk(text).await.unwrap();
        assert_eq!(
            chunks.len(),
            test_chunks.len(),
            "Boundary detection should be deterministic (run {})",
            i
        );

        // Verify same boundaries
        for (j, (orig, test)) in chunks.iter().zip(test_chunks.iter()).enumerate() {
            assert_eq!(
                orig.content, test.content,
                "Chunk {} content should be identical across runs",
                j
            );
        }
    }
}

/// Test performance optimizations meet <200ms requirement
#[tokio::test]
async fn test_performance_optimizations() {
    let config = ChunkingConfig::default();
    let chunker = TopicChunker::new_without_fallback(4, 2, config);

    // Generate performance test document (multiple topics, ~30KB)
    let mut text = String::new();
    let topics = [
        "artificial intelligence machine learning deep neural networks algorithms",
        "climate change environmental sustainability carbon emissions policies",
        "economic growth market forces fiscal monetary policy inflation",
        "healthcare medical research pharmaceutical drug development treatments",
        "technology innovation software development programming languages",
    ];

    for round in 0..50 {
        let topic = &topics[round % topics.len()];
        text.push_str(&format!(
            "\n\nSection {}: Research in {} continues to advance rapidly. \
            Scientists and researchers are making significant breakthroughs in this field. \
            Recent studies have shown promising results and practical applications. \
            Future developments will likely transform how we approach these challenges. \
            Industry leaders are investing heavily in research and development efforts. \
            The implications for society and technology are substantial and far-reaching. ",
            round + 1,
            topic
        ));
    }

    println!("Performance test document: {} characters", text.len());

    // Test multiple runs to ensure consistent performance
    let mut durations = Vec::new();
    for run in 0..5 {
        let start = Instant::now();
        let chunks = chunker.chunk(&text).await.unwrap();
        let duration = start.elapsed();
        durations.push(duration.as_millis());

        println!(
            "Run {}: {} chunks in {}ms",
            run + 1,
            chunks.len(),
            duration.as_millis()
        );

        // Should meet performance requirement
        assert!(
            duration.as_millis() < 200,
            "Performance optimization failed: {}ms > 200ms",
            duration.as_millis()
        );

        // Should produce reasonable chunking
        assert!(chunks.len() >= 3, "Should detect multiple topics");
        assert!(chunks.len() <= 15, "Should not over-segment");

        // All chunks should have quality scores
        for chunk in &chunks {
            assert!(
                chunk.metadata.quality_score > 0.0,
                "Chunks should have quality scores"
            );
            assert!(
                !chunk.metadata.topic_keywords.is_empty(),
                "Chunks should have keywords"
            );
        }
    }

    let avg_duration = durations.iter().sum::<u128>() / durations.len() as u128;
    let max_duration = *durations.iter().max().unwrap();

    println!(
        "Average duration: {}ms, Max duration: {}ms",
        avg_duration, max_duration
    );
    assert!(
        avg_duration < 150,
        "Average performance should be well under limit: {}ms",
        avg_duration
    );
}

/// Test fallback mechanism when performance constraints are exceeded
#[tokio::test]
async fn test_fallback_mechanism() {
    let config = ChunkingConfig::default();
    let chunker = TopicChunker::new(3, 2, config); // With fallback

    // Generate very large document to trigger fallback
    let mut large_text = String::new();
    let base_text = "This is a performance test with substantial content to process. ";
    while large_text.len() < 160_000 {
        // Above 150KB threshold
        large_text.push_str(base_text);
        large_text.push_str("Machine learning and artificial intelligence continue evolving. ");
    }

    println!("Large document size: {} characters", large_text.len());

    let start = Instant::now();
    let chunks = chunker.chunk(&large_text).await.unwrap();
    let duration = start.elapsed();

    println!(
        "Fallback test: {} chunks in {}ms",
        chunks.len(),
        duration.as_millis()
    );

    // Should still meet performance requirement due to fallback
    assert!(
        duration.as_millis() < 200,
        "Fallback should ensure performance: {}ms",
        duration.as_millis()
    );

    // Should produce chunks
    assert!(
        !chunks.is_empty(),
        "Should produce chunks even with fallback"
    );

    // Should indicate fallback was used
    let fallback_used = chunks.iter().any(|c| {
        c.metadata.chunk_type == "sliding-fallback"
            || c.metadata.custom.contains_key("fallback_reason")
    });

    if large_text.len() > 150_000 {
        assert!(
            fallback_used,
            "Should use fallback for very large documents"
        );
    }
}

/// Test boundary prominence calculation prevents weak boundaries
#[tokio::test]
async fn test_prominence_filtering() {
    let config = ChunkingConfig::default();
    let chunker = TopicChunker::new_without_fallback(3, 1, config);

    // Text with one strong boundary and several weak ones
    let text = "
        Deep learning revolutionized computer vision and natural language processing.
        Neural networks with multiple layers can learn hierarchical representations.
        Convolutional networks excel at image recognition and spatial pattern detection.
        Recurrent networks handle sequential data like text and time series effectively.

        Quantum computing represents a fundamentally different computational paradigm.
        Quantum bits can exist in superposition states enabling parallel computation.
        Quantum algorithms like Shor's and Grover's provide exponential speedups.
        Quantum error correction is essential for practical quantum computers.

        Blockchain technology ensures distributed consensus without central authority.
        Cryptographic hashing creates tamper-evident chains of transaction blocks.
    ";

    let chunks = chunker.chunk(text).await.unwrap();

    println!("Prominence filtering: {} chunks", chunks.len());

    // Should detect the two strong boundaries (deep learning -> quantum -> blockchain)
    assert!(
        chunks.len() >= 2 && chunks.len() <= 4,
        "Prominence should filter weak boundaries: {} chunks",
        chunks.len()
    );

    // Verify chunks have distinct topics
    let all_keywords: Vec<String> = chunks
        .iter()
        .flat_map(|c| c.metadata.topic_keywords.iter().cloned())
        .collect();

    let has_deep_learning = all_keywords
        .iter()
        .any(|k| k.contains("deep") || k.contains("neural") || k.contains("learning"));
    let has_quantum = all_keywords
        .iter()
        .any(|k| k.contains("quantum") || k.contains("superposition"));
    let has_blockchain = all_keywords
        .iter()
        .any(|k| k.contains("blockchain") || k.contains("cryptographic"));

    assert!(has_deep_learning, "Should detect deep learning topic");
    assert!(has_quantum, "Should detect quantum computing topic");

    if chunks.len() >= 3 {
        assert!(
            has_blockchain,
            "Should detect blockchain topic in separate chunk"
        );
    }
}

/// Test minimum distance filtering prevents overly close boundaries
#[tokio::test]
async fn test_minimum_distance_filtering() {
    let config = ChunkingConfig::default();
    let chunker = TopicChunker::new_without_fallback(2, 1, config);

    // Text with potential boundaries very close together
    let text = "
        AI systems. ML models. Deep networks. Computer vision tasks.
        Image recognition. Object detection. Facial recognition systems.
        Quantum computers. Quantum bits. Superposition states here.
        Quantum algorithms. Shor's algorithm. Grover's search method.
        Blockchain networks. Cryptocurrency systems. Digital ledgers.
        Smart contracts. Decentralized applications. Consensus mechanisms.
    ";

    let chunks = chunker.chunk(text).await.unwrap();

    println!("Distance filtering: {} chunks", chunks.len());

    // Should not create too many small chunks due to minimum distance filtering
    assert!(
        chunks.len() <= 4,
        "Should limit chunks due to minimum distance: {} chunks",
        chunks.len()
    );

    // Each chunk should have reasonable size
    for (i, chunk) in chunks.iter().enumerate() {
        assert!(
            chunk.content.len() >= 50,
            "Chunk {} too small: {} chars",
            i,
            chunk.content.len()
        );
        assert!(
            chunk.metadata.word_count >= 8,
            "Chunk {} has too few words: {}",
            i,
            chunk.metadata.word_count
        );
    }
}

/// Test deterministic boundary detection across multiple runs
#[tokio::test]
async fn test_deterministic_boundaries() {
    let config = ChunkingConfig::default();
    let chunker = TopicChunker::new_without_fallback(3, 2, config);

    let text = "
        Machine learning algorithms process vast amounts of training data efficiently.
        Neural networks learn complex patterns through iterative weight adjustments.
        Deep learning architectures automatically extract hierarchical features from input.

        Climate science studies long-term atmospheric and environmental changes globally.
        Greenhouse gas concentrations affect global temperature and weather patterns.
        Climate models predict future environmental conditions using historical data.

        Economic theories explain market behavior and financial system dynamics.
        Supply and demand forces determine pricing in competitive markets.
        Monetary policy influences inflation rates and economic growth patterns.
    ";

    // Run chunking multiple times
    let mut all_chunks = Vec::new();
    for run in 0..10 {
        let chunks = chunker.chunk(text).await.unwrap();
        all_chunks.push(chunks);

        if run > 0 {
            // Compare with first run
            assert_eq!(
                all_chunks[0].len(),
                all_chunks[run].len(),
                "Number of chunks should be deterministic across runs"
            );

            for (i, (chunk1, chunk2)) in
                all_chunks[0].iter().zip(all_chunks[run].iter()).enumerate()
            {
                assert_eq!(
                    chunk1.content, chunk2.content,
                    "Chunk {} content should be identical across runs",
                    i
                );
                assert_eq!(
                    chunk1.start_pos, chunk2.start_pos,
                    "Chunk {} positions should be identical",
                    i
                );
                assert_eq!(
                    chunk1.end_pos, chunk2.end_pos,
                    "Chunk {} positions should be identical",
                    i
                );
            }
        }
    }

    println!(
        "Deterministic test: {} chunks consistently across 10 runs",
        all_chunks[0].len()
    );
    assert!(all_chunks[0].len() >= 2, "Should detect multiple topics");
}

/// Test enhanced keyword extraction quality
#[tokio::test]
async fn test_keyword_extraction_quality() {
    let config = ChunkingConfig::default();
    let chunker = TopicChunker::new_without_fallback(3, 1, config);

    let text = "
        Natural language processing enables computers to understand human communication.
        Text analysis algorithms extract meaning from written documents and conversations.
        Sentiment analysis determines emotional tone in social media posts and reviews.
        Named entity recognition identifies people, places, and organizations in text.

        Computer vision systems process visual information from cameras and sensors.
        Image classification algorithms categorize photos into predefined object classes.
        Object detection systems locate and identify multiple items within single images.
        Facial recognition technology identifies individuals based on facial features.
    ";

    let chunks = chunker.chunk(text).await.unwrap();

    for (i, chunk) in chunks.iter().enumerate() {
        println!(
            "Chunk {}: {} keywords: {:?}",
            i,
            chunk.metadata.topic_keywords.len(),
            chunk.metadata.topic_keywords
        );

        // Should extract meaningful keywords
        assert!(
            !chunk.metadata.topic_keywords.is_empty(),
            "Chunk {} should have keywords",
            i
        );
        assert!(
            chunk.metadata.topic_keywords.len() <= 5,
            "Should limit keywords to top 5: {}",
            chunk.metadata.topic_keywords.len()
        );

        // Keywords should be relevant to content
        let content_lower = chunk.content.to_lowercase();
        for keyword in &chunk.metadata.topic_keywords {
            assert!(
                content_lower.contains(keyword),
                "Keyword '{}' should appear in chunk content",
                keyword
            );
        }
    }

    // Different chunks should have different primary keywords
    if chunks.len() >= 2 {
        let keywords1 = &chunks[0].metadata.topic_keywords;
        let keywords2 = &chunks[1].metadata.topic_keywords;

        let overlap = keywords1.iter().filter(|k| keywords2.contains(k)).count();
        let total_unique = keywords1.len() + keywords2.len() - overlap;

        // Should have some diversity in keywords between chunks
        assert!(
            total_unique > overlap,
            "Chunks should have some distinct keywords: {} unique vs {} overlap",
            total_unique,
            overlap
        );
    }
}
