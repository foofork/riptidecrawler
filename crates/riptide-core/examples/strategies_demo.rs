//! Demonstration of extraction strategies and chunking capabilities
//! Note: Many features have been moved to separate crates (riptide-html, riptide-intelligence)

use anyhow::Result;
use riptide_core::strategies::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Riptide Core Strategies Demo");
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

    // Demonstrate core extraction
    demo_core_extraction(sample_html).await?;

    // Demonstrate metadata extraction
    demo_metadata_extraction(sample_html).await?;

    // Demonstrate strategy manager
    demo_strategy_manager(sample_html).await?;

    println!("\n✅ Demo completed successfully!");
    Ok(())
}

async fn demo_core_extraction(html: &str) -> Result<()> {
    println!("📊 CORE EXTRACTION STRATEGY");
    println!("===================================\n");

    // Trek Strategy (Default WASM-based)
    println!("🔧 Trek Strategy (Default Core Implementation):");
    let trek_result = extraction::trek::extract(html, "https://example.com").await?;
    println!("  Title: {}", trek_result.title);
    println!("  Content Length: {} chars", trek_result.content.len());
    println!(
        "  Confidence: {:.2}%\n",
        trek_result.extraction_confidence * 100.0
    );

    println!("ℹ️  Note: CSS JSON and Regex extraction strategies have been moved to the riptide-html crate.\n");
    println!("ℹ️  Note: LLM-based extraction strategies have been moved to the riptide-intelligence crate.\n");
    println!("ℹ️  Note: Content chunking features have been moved to the riptide-html crate.\n");

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
    println!(
        "  Author: {:.1}%",
        metadata.confidence_scores.author * 100.0
    );
    println!("  Date: {:.1}%", metadata.confidence_scores.date * 100.0);
    println!(
        "  Description: {:.1}%",
        metadata.confidence_scores.description * 100.0
    );
    println!(
        "  Overall: {:.1}%",
        metadata.confidence_scores.overall * 100.0
    );

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

    // Create configuration with Trek (core default)
    let config = StrategyConfig::default();
    let mut manager = StrategyManager::new(config);

    let start_time = std::time::Instant::now();
    let result = manager.extract_content(html, "https://example.com").await?;
    let processing_time = start_time.elapsed();

    println!("🚀 Processing Results:");
    println!("  Strategy: Trek (Core)");
    println!("  Processing Time: {:.2}ms", processing_time.as_millis());
    println!("  Content Length: {} chars", result.extracted.content.len());

    if let Some(title) = &result.metadata.title {
        println!("  Title: {}", title);
    }
    if let Some(author) = &result.metadata.author {
        println!("  Author: {}", author);
    }

    println!("\n💡 Recommendations:");
    println!("  • Use riptide-html for CSS/Regex extraction");
    println!("  • Use riptide-intelligence for LLM-based extraction");
    println!("  • Use riptide-html for content chunking");
    println!("  • Trek provides fast baseline extraction in core");
    println!();

    Ok(())
}
