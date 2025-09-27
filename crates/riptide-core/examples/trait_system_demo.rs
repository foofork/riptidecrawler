//! Demonstration of the trait-based strategy management system
//!
//! This example shows how to use the new trait-based strategy system
//! that provides better extensibility and composition than the old enum-based approach.

use std::sync::Arc;
use riptide_core::strategies::{
    // Core traits
    ExtractionStrategy, ChunkingStrategy, StrategyRegistry,
    // Implementations
    TrekExtractionStrategy, SlidingChunkingStrategy, FixedChunkingStrategy,
    // Enhanced manager
    EnhancedStrategyManager, StrategyManagerConfig,
    // Backward compatibility
    CompatibleStrategyManager, StrategyConfig,
    ExtractionStrategy as LegacyExtractionStrategy,
    // Factory and utilities
    StrategyFactory, MigrationUtils,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Riptide Trait-Based Strategy System Demo");
    println!("=============================================\n");

    // Test HTML content
    let test_html = r#"
    <html>
    <head>
        <title>Test Document</title>
        <meta name="description" content="A test document for strategy demonstration">
    </head>
    <body>
        <main>
            <h1>Welcome to Riptide</h1>
            <p>This is a demonstration of the new trait-based strategy system.</p>
            <p>The system allows for better composition and extensibility of strategies.</p>
            <section>
                <h2>Features</h2>
                <ul>
                    <li>Trait-based architecture</li>
                    <li>Enhanced strategy manager</li>
                    <li>Backward compatibility</li>
                    <li>Automatic strategy selection</li>
                </ul>
            </section>
        </main>
    </body>
    </html>
    "#;
    let test_url = "https://example.com/test";

    // 1. Basic trait usage
    println!("1. Basic Trait Usage");
    println!("-------------------");

    let trek_strategy = TrekExtractionStrategy;
    println!("Strategy: {}", trek_strategy.name());
    println!("Capabilities: {:?}", trek_strategy.capabilities());
    println!("Confidence: {:.2}", trek_strategy.confidence_score(test_html));

    let result = trek_strategy.extract(test_html, test_url).await?;
    println!("Extracted title: {}", result.content.title);
    println!("Content length: {} chars", result.content.content.len());
    println!("Quality score: {:.2}\n", result.quality.overall_score());

    // 2. Strategy Registry
    println!("2. Strategy Registry");
    println!("-------------------");

    let mut registry = StrategyRegistry::new();

    // Register strategies
    registry.register_extraction(Arc::new(TrekExtractionStrategy));
    registry.register_chunking(Arc::new(SlidingChunkingStrategy));
    registry.register_chunking(Arc::new(FixedChunkingStrategy::new(500, false)));

    // List available strategies
    let extraction_strategies = registry.list_extraction_strategies();
    println!("Available extraction strategies:");
    for (name, capabilities) in &extraction_strategies {
        println!("  - {}: {} ({})", name, capabilities.strategy_type,
                 format!("{:?}", capabilities.performance_tier));
    }

    let chunking_strategies = registry.list_chunking_strategies();
    println!("Available chunking strategies:");
    for name in &chunking_strategies {
        println!("  - {}", name);
    }

    // Find best strategy
    if let Some(best_strategy) = registry.find_best_extraction(test_html) {
        println!("Best strategy for content: {}\n", best_strategy.name());
    }

    // 3. Enhanced Strategy Manager
    println!("3. Enhanced Strategy Manager");
    println!("---------------------------");

    let config = StrategyManagerConfig {
        enable_metrics: true,
        auto_strategy_selection: true,
        fallback_enabled: true,
        ..Default::default()
    };

    let manager = EnhancedStrategyManager::new(config).await;
    let processed = manager.extract_and_process(test_html, test_url).await?;

    println!("Strategy used: {}", processed.strategy_used);
    println!("Processing time: {:?}", processed.processing_time);
    println!("Chunks created: {}", processed.chunks.len());
    if let Some(metrics) = &processed.metrics {
        println!("Metrics available: true");
    }
    println!();

    // 4. Backward Compatibility
    println!("4. Backward Compatibility");
    println!("-------------------------");

    let legacy_config = StrategyConfig {
        extraction: LegacyExtractionStrategy::Trek,
        enable_metrics: true,
        ..Default::default()
    };

    let mut compat_manager = CompatibleStrategyManager::new(legacy_config).await;
    let legacy_result = compat_manager.extract_and_chunk(test_html, test_url).await?;

    println!("Legacy result title: {}", legacy_result.extracted.title);
    println!("Legacy chunks: {}", legacy_result.chunks.len());
    println!();

    // 5. Migration Example
    println!("5. Migration from Enum to Traits");
    println!("--------------------------------");

    let old_strategy = LegacyExtractionStrategy::Trek;
    let new_strategy = StrategyFactory::create_extraction(old_strategy);
    println!("Migrated strategy name: {}", new_strategy.name());

    let old_config = StrategyConfig::default();
    let upgraded_manager = MigrationUtils::upgrade_manager(old_config).await?;
    let strategies = upgraded_manager.list_strategies().await;
    println!("Upgraded manager has {} extraction strategies", strategies.extraction.len());
    println!();

    // 6. Custom Strategy Example
    println!("6. Custom Strategy Registration");
    println!("------------------------------");

    // This shows how you could register custom strategies
    let custom_registry = registry;
    let available_extractions = custom_registry.list_extraction_strategies();
    let available_chunking = custom_registry.list_chunking_strategies();

    println!("Total extraction strategies: {}", available_extractions.len());
    println!("Total chunking strategies: {}", available_chunking.len());

    println!("\n✅ Trait-based strategy system demonstration complete!");
    println!("The system provides a clean, extensible architecture while maintaining");
    println!("backward compatibility with existing enum-based code.");

    Ok(())
}