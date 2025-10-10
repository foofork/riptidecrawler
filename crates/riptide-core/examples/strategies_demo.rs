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

async fn demo_core_extraction(_html: &str) -> Result<()> {
    println!("📊 CORE EXTRACTION STRATEGY");
    println!("===================================\n");

    // Trek Strategy has been moved to other crates or is accessible through different APIs
    println!("🔧 Core Extraction:");
    println!("  Note: Direct extraction APIs have been refactored\n");

    println!("ℹ️  Note: CSS JSON and Regex extraction strategies have been moved to the riptide-html crate.\n");
    println!("ℹ️  Note: LLM-based extraction strategies have been moved to the riptide-intelligence crate.\n");
    println!("ℹ️  Note: Content chunking features have been moved to the riptide-html crate.\n");
    println!("ℹ️  Note: Trek extraction is now handled through the WASM extractor component.\n");

    Ok(())
}

async fn demo_metadata_extraction(_html: &str) -> Result<()> {
    println!("🏷️  METADATA EXTRACTION ANALYSIS");
    println!("===============================\n");

    println!("ℹ️  Note: Metadata extraction has been refactored and is now part of the strategy system.\n");

    // Commented out as the API has changed
    // let metadata = metadata::extract_metadata(html, "https://example.com").await?;

    println!("📄 Metadata Extraction:");
    println!("  • Open Graph metadata extraction");
    println!("  • JSON-LD structured data");
    println!("  • HTML meta tags");
    println!("  • Microdata support");
    println!("  • Heuristic-based extraction\n");

    println!("  These features are available through the StrategyManager API\n");

    Ok(())
}

async fn demo_strategy_manager(_html: &str) -> Result<()> {
    println!("⚙️  STRATEGY MANAGER INTEGRATION");
    println!("==============================\n");

    // Create configuration with default settings
    let config = StrategyConfig::default();
    let _manager = StrategyManager::new(config);

    println!("🚀 Strategy Manager Features:");
    println!("  • Multi-strategy extraction support");
    println!("  • Adaptive strategy selection");
    println!("  • Performance monitoring");
    println!("  • Quality-based fallback\n");

    println!("📊 Available Strategies:");
    println!("  • Trek: Fast baseline extraction (core)");
    println!("  • CSS: Selector-based extraction (riptide-html)");
    println!("  • Regex: Pattern matching (riptide-html)");
    println!("  • LLM: AI-powered extraction (riptide-intelligence)\n");

    println!("💡 Usage Recommendations:");
    println!("  • Use StrategyManager for automatic strategy selection");
    println!("  • Configure fallback strategies for reliability");
    println!("  • Monitor performance metrics for optimization");
    println!("  • Leverage chunking for large content (riptide-html)");
    println!();

    // Show config info
    println!("📋 Current Configuration:");
    println!("  StrategyManager initialized successfully");
    println!("  Ready for content extraction");
    println!();

    Ok(())
}
