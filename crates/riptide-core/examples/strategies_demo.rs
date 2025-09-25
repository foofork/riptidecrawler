//! Demonstration of extraction strategies and chunking capabilities

use anyhow::Result;
use riptide_core::strategies::*;
use riptide_core::strategies::chunking::ChunkingMode;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Riptide Core Strategies & Chunking Demo");
    println!("==========================================\n");

    // Sample HTML content for demonstration
    let sample_html = r#"
    <html>
    <head>
        <title>Advanced Web Content Processing with Machine Learning</title>
        <meta name="description" content="Exploring how AI and ML enhance web scraping and content extraction">
        <meta name="author" content="Dr. Sarah Chen">
        <meta name="keywords" content="machine learning, web scraping, AI, content extraction">
        <meta property="og:title" content="ML-Enhanced Web Processing">
        <meta property="og:description" content="Revolutionary approaches to web content analysis">
        <meta property="og:image" content="https://example.com/ml-web.jpg">
        <meta property="article:published_time" content="2024-01-15T10:00:00Z">
        <meta property="article:author" content="Dr. Sarah Chen">
        <link rel="canonical" href="https://example.com/ml-web-processing">
    </head>
    <body>
        <script type="application/ld+json">
        {
            "@context": "https://schema.org",
            "@type": "Article",
            "headline": "Machine Learning Transforms Web Content Processing",
            "description": "How artificial intelligence revolutionizes content extraction and analysis",
            "author": {
                "@type": "Person",
                "name": "Dr. Sarah Chen"
            },
            "datePublished": "2024-01-15T10:00:00Z",
            "keywords": ["machine learning", "web scraping", "AI", "natural language processing"],
            "wordCount": 1250
        }
        </script>

        <article>
            <h1>Machine Learning Transforms Web Content Processing</h1>
            <div class="byline">By Dr. Sarah Chen</div>
            <time datetime="2024-01-15">January 15, 2024</time>

            <section class="introduction">
                <p>The field of web content extraction has undergone a revolutionary transformation with the integration of machine learning technologies. Modern approaches leverage sophisticated algorithms to understand content structure, semantic meaning, and optimal processing strategies.</p>

                <p>Traditional rule-based extraction methods often fail when faced with diverse website structures and dynamic content. Machine learning models, however, can adapt to new patterns and provide more robust extraction capabilities.</p>
            </section>

            <section class="methodology">
                <h2>Adaptive Extraction Strategies</h2>
                <p>Our research demonstrates that combining multiple extraction strategies significantly improves accuracy and reliability. The Trek algorithm provides fast baseline extraction, while CSS selectors offer precision for structured content. Regular expressions handle pattern-based data, and LLM integration enables semantic understanding.</p>

                <p>Performance metrics show that adaptive strategy selection based on content characteristics achieves 84.8% accuracy improvement over single-strategy approaches. Token-aware chunking ensures optimal processing within language model constraints.</p>
            </section>

            <section class="results">
                <h2>Chunking and Performance Optimization</h2>
                <p>Intelligent content chunking plays a crucial role in processing efficiency. Our sliding window approach with semantic overlap preservation maintains context while respecting token limits. Sentence-based chunking ensures grammatical completeness, while topic-based segmentation groups related concepts.</p>

                <p>Benchmark results indicate 2.8-4.4x speed improvements when using deterministic chunking with ML-guided strategy selection. Memory efficiency gains of 32.3% were observed across diverse content types.</p>
            </section>

            <section class="conclusion">
                <h2>Future Directions</h2>
                <p>The integration of neural networks with traditional extraction methods opens new possibilities for automated content understanding. Real-time adaptation based on extraction quality feedback enables continuous improvement of processing accuracy.</p>

                <p>As web content becomes increasingly complex and dynamic, machine learning-driven extraction strategies will become essential for maintaining high-quality data processing pipelines.</p>
            </section>
        </article>
    </body>
    </html>
    "#;

    // Demonstrate different extraction strategies
    demo_extraction_strategies(sample_html).await?;

    // Demonstrate chunking modes
    demo_chunking_strategies().await?;

    // Demonstrate metadata extraction
    demo_metadata_extraction(sample_html).await?;

    // Demonstrate strategy manager
    demo_strategy_manager(sample_html).await?;

    // Demonstrate performance comparison
    demo_performance_comparison().await?;

    println!("\n✅ Demo completed successfully!");
    Ok(())
}

async fn demo_extraction_strategies(html: &str) -> Result<()> {
    println!("📊 EXTRACTION STRATEGIES COMPARISON");
    println!("===================================\n");

    // Trek Strategy (Default WASM-based)
    println!("🔧 Trek Strategy (Default):");
    let trek_result = extraction::trek::extract(html, "https://example.com").await?;
    println!("  Title: {}", trek_result.title);
    println!("  Content Length: {} chars", trek_result.content.len());
    println!("  Confidence: {:.2}%\n", trek_result.extraction_confidence * 100.0);

    // CSS JSON Strategy
    println!("🎯 CSS JSON Strategy:");
    let selectors = extraction::css_json::default_selectors();
    let css_result = extraction::css_json::extract(html, "https://example.com", &selectors).await?;
    println!("  Title: {}", css_result.title);
    println!("  Content Length: {} chars", css_result.content.len());
    println!("  Confidence: {:.2}%", css_result.extraction_confidence * 100.0);
    if let Some(summary) = css_result.summary {
        println!("  Summary: {}", summary);
    }
    println!();

    // Regex Strategy
    println!("🔍 Regex Strategy:");
    let patterns = extraction::regex::default_patterns();
    let regex_result = extraction::regex::extract(html, "https://example.com", &patterns).await?;
    println!("  Title: {}", regex_result.title);
    println!("  Content Length: {} chars", regex_result.content.len());
    println!("  Confidence: {:.2}%\n", regex_result.extraction_confidence * 100.0);

    Ok(())
}

async fn demo_chunking_strategies() -> Result<()> {
    println!("📝 CHUNKING STRATEGIES DEMONSTRATION");
    println!("===================================\n");

    let sample_content = "Artificial intelligence has revolutionized content processing. Machine learning algorithms can now understand context and meaning. Neural networks enable sophisticated pattern recognition. Deep learning models process vast amounts of data efficiently. Natural language processing bridges human and machine understanding. Automated systems continuously improve through feedback. The future of content extraction looks increasingly promising.";

    // Sliding Window Chunking
    println!("🌊 Sliding Window Chunking (Default):");
    let sliding_config = ChunkingConfig {
        mode: ChunkingMode::Sliding,
        token_max: 50,
        overlap: 10,
        preserve_sentences: true,
        deterministic: true,
    };
    let sliding_chunks = chunking::chunk_content(sample_content, &sliding_config).await?;
    for (i, chunk) in sliding_chunks.iter().enumerate() {
        println!("  Chunk {}: {} tokens, Quality: {:.2}",
                i + 1, chunk.token_count, chunk.metadata.quality_score);
    }
    println!("  Total chunks: {}\n", sliding_chunks.len());

    // Sentence-based Chunking
    println!("📖 Sentence-based Chunking:");
    let sentence_config = ChunkingConfig {
        mode: ChunkingMode::Sentence { max_sentences: 2 },
        token_max: 100,
        overlap: 0,
        preserve_sentences: true,
        deterministic: true,
    };
    let sentence_chunks = chunking::chunk_content(sample_content, &sentence_config).await?;
    for (i, chunk) in sentence_chunks.iter().enumerate() {
        println!("  Chunk {}: {} sentences, {} words",
                i + 1, chunk.metadata.sentence_count, chunk.metadata.word_count);
    }
    println!("  Total chunks: {}\n", sentence_chunks.len());

    // Topic-based Chunking
    println!("🏷️  Topic-based Chunking:");
    let topic_config = ChunkingConfig {
        mode: ChunkingMode::Topic { similarity_threshold: 0.3 },
        token_max: 150,
        overlap: 0,
        preserve_sentences: true,
        deterministic: true,
    };
    let topic_chunks = chunking::chunk_content(sample_content, &topic_config).await?;
    for (i, chunk) in topic_chunks.iter().enumerate() {
        println!("  Chunk {}: Keywords: {:?}",
                i + 1, chunk.metadata.topic_keywords);
    }
    println!("  Total chunks: {}\n", topic_chunks.len());

    Ok(())
}

async fn demo_metadata_extraction(html: &str) -> Result<()> {
    println!("🏷️  METADATA EXTRACTION ANALYSIS");
    println!("===============================\n");

    let metadata = metadata::extract_metadata(html, "https://example.com").await?;

    println!("📄 Extracted Metadata:");
    if let Some(title) = &metadata.title {
        println!("  Title: {}", title);
    }
    if let Some(author) = &metadata.author {
        println!("  Author: {}", author);
    }
    if let Some(description) = &metadata.description {
        println!("  Description: {}", description);
    }
    if let Some(date) = &metadata.published_date {
        println!("  Published: {}", date.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    if !metadata.keywords.is_empty() {
        println!("  Keywords: {}", metadata.keywords.join(", "));
    }

    println!("\n📊 Extraction Confidence:");
    println!("  Title: {:.1}%", metadata.confidence_scores.title * 100.0);
    println!("  Author: {:.1}%", metadata.confidence_scores.author * 100.0);
    println!("  Date: {:.1}%", metadata.confidence_scores.date * 100.0);
    println!("  Description: {:.1}%", metadata.confidence_scores.description * 100.0);
    println!("  Overall: {:.1}%", metadata.confidence_scores.overall * 100.0);

    println!("\n🔍 Extraction Methods Used:");
    if metadata.extraction_method.open_graph {
        println!("  ✅ Open Graph");
    }
    if metadata.extraction_method.json_ld {
        println!("  ✅ JSON-LD");
    }
    if metadata.extraction_method.meta_tags {
        println!("  ✅ Meta Tags");
    }
    if metadata.extraction_method.microdata {
        println!("  ✅ Microdata");
    }
    if metadata.extraction_method.heuristics {
        println!("  ✅ Heuristics");
    }
    println!();

    Ok(())
}

async fn demo_strategy_manager(html: &str) -> Result<()> {
    println!("⚙️  STRATEGY MANAGER INTEGRATION");
    println!("==============================\n");

    // Create configuration with Trek + Sliding window (defaults)
    let config = StrategyConfig::default();
    let mut manager = StrategyManager::new(config);

    let start_time = std::time::Instant::now();
    let result = manager.extract_and_chunk(html, "https://example.com").await?;
    let processing_time = start_time.elapsed();

    println!("🚀 Processing Results:");
    println!("  Strategy: {}", result.extracted.strategy_used);
    println!("  Processing Time: {:.2}ms", processing_time.as_millis());
    println!("  Content Length: {} chars", result.extracted.content.len());
    println!("  Chunks Created: {}", result.chunks.len());

    println!("\n📊 Chunk Analysis:");
    let mut total_quality = 0.0;
    for chunk in &result.chunks {
        total_quality += chunk.metadata.quality_score;
    }
    let avg_quality = total_quality / result.chunks.len() as f64;
    println!("  Average Quality Score: {:.2}", avg_quality);

    let avg_chunk_size = result.chunks.iter()
        .map(|c| c.content.len())
        .sum::<usize>() / result.chunks.len();
    println!("  Average Chunk Size: {} chars", avg_chunk_size);

    if let Some(metrics) = &result.metrics {
        let summary = metrics.get_summary();
        println!("\n📈 Performance Metrics:");
        println!("  Total Extractions: {}", summary.total_extractions);
        println!("  Best Strategy: {}", summary.best_performing_strategy);
        println!("  Best Throughput: {:.2} MB/s", summary.best_throughput);
    }
    println!();

    Ok(())
}

async fn demo_performance_comparison() -> Result<()> {
    println!("🏁 PERFORMANCE COMPARISON");
    println!("========================\n");

    let test_content = "Performance test content. ".repeat(100);

    let strategies = vec![
        ("Trek", ExtractionStrategy::Trek),
        ("CSS JSON", ExtractionStrategy::CssJson {
            selectors: extraction::css_json::default_selectors()
        }),
        ("Regex", ExtractionStrategy::Regex {
            patterns: extraction::regex::default_patterns()
        }),
    ];

    println!("🚀 Strategy Performance (100 words):");
    for (name, strategy) in strategies {
        let config = StrategyConfig {
            extraction: strategy,
            chunking: ChunkingConfig::default(),
            enable_metrics: true,
            validate_schema: false, // Skip for performance
        };

        let mut manager = StrategyManager::new(config);
        let start = std::time::Instant::now();

        let _result = manager.extract_and_chunk(&test_content, "https://example.com").await?;
        let duration = start.elapsed();

        println!("  {}: {:.2}ms", name, duration.as_millis());
    }

    println!("\n🧩 Chunking Performance:");
    let chunking_modes = vec![
        ("Sliding", ChunkingMode::Sliding),
        ("Sentence", ChunkingMode::Sentence { max_sentences: 3 }),
        ("Fixed", ChunkingMode::Fixed { size: 200, by_tokens: false }),
        ("Topic", ChunkingMode::Topic { similarity_threshold: 0.3 }),
    ];

    for (name, mode) in chunking_modes {
        let config = ChunkingConfig {
            mode,
            token_max: 100,
            overlap: 10,
            preserve_sentences: true,
            deterministic: true,
        };

        let start = std::time::Instant::now();
        let chunks = chunking::chunk_content(&test_content, &config).await?;
        let duration = start.elapsed();

        println!("  {}: {:.2}ms ({} chunks)", name, duration.as_millis(), chunks.len());
    }

    println!("\n💡 Recommendations:");
    println!("  • Use Trek for fastest extraction");
    println!("  • Use CSS JSON for structured content");
    println!("  • Use Sliding chunking for balanced performance");
    println!("  • Enable deterministic mode for consistency");
    println!("  • Monitor quality scores for optimization");
    println!();

    Ok(())
}