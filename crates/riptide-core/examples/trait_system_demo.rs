//! Demonstration of the trait-based strategy management system
//!
//! This example shows how to use the new trait-based strategy system
//! that provides better extensibility and composition than the old enum-based approach.
//!
//! Note: This demo focuses on the core extraction functionality in riptide-core.
//! For chunking strategies, see examples in the riptide-html crate.

use std::sync::Arc;
use riptide_core::strategies::{
    // Core traits
    traits::ExtractionStrategy, StrategyRegistry,
    // Implementations
    TrekExtractionStrategy,
    // Enhanced manager
    EnhancedStrategyManager, StrategyManagerConfig,
    // Backward compatibility
    compatibility::{CompatibleStrategyManager, LegacyStrategyConfig},
    // Migration utilities
    compatibility::migrate_extraction_strategy,
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

    // Register core extraction strategies
    registry.register_extraction(Arc::new(TrekExtractionStrategy));

    // List available strategies
    let extraction_strategies = registry.list_extraction_strategies();
    println!("Available extraction strategies:");
    for (name, capabilities) in &extraction_strategies {
        println!("  - {}: {} ({})", name, capabilities.strategy_type,
                 format!("{:?}", capabilities.performance_tier));
    }

    println!("Note: Chunking strategies have been moved to riptide-html crate");

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

    // Create manager with pre-populated registry
    let mut test_registry = StrategyRegistry::new();
    test_registry.register_extraction(Arc::new(TrekExtractionStrategy));

    let manager = EnhancedStrategyManager::with_registry(config, test_registry).await;
    let processed = manager.extract_and_process_with_strategy(test_html, test_url, "trek").await?;

    println!("Strategy used: {}", processed.strategy_used);
    println!("Processing time: {:?}", processed.processing_time);
    println!("Title extracted: {}", processed.extracted.title);
    if let Some(_metrics) = &processed.metrics {
        println!("Metrics available: true");
    }
    println!();

    // 4. Backward Compatibility
    println!("4. Backward Compatibility");
    println!("-------------------------");

    let legacy_config = LegacyStrategyConfig {
        extraction: riptide_core::strategies::ExtractionStrategy::Trek,
        enable_metrics: true,
        validate_schema: true,
    };

    let mut compat_manager = CompatibleStrategyManager::new(legacy_config);
    let legacy_result = compat_manager.extract_content(test_html, test_url).await?;

    println!("Legacy result title: {}", legacy_result.extracted.title);
    println!("Legacy content length: {} chars", legacy_result.extracted.content.len());
    println!();

    // 5. Migration Example
    println!("5. Migration from Enum to Traits");
    println!("--------------------------------");

    let old_strategy = riptide_core::strategies::ExtractionStrategy::Trek;
    let new_strategy = migrate_extraction_strategy(&old_strategy);
    println!("Migrated strategy name: {}", new_strategy.name());
    println!("Strategy capabilities: {:?}", new_strategy.capabilities().strategy_type);
    println!();

    // 6. Custom Strategy Example
    println!("6. Current Strategy Registry Status");
    println!("----------------------------------");

    // Show current state of the registry
    let available_extractions = registry.list_extraction_strategies();
    let available_spider = registry.list_spider_strategies();

    println!("Total extraction strategies: {}", available_extractions.len());
    println!("Total spider strategies: {}", available_spider.len());
    println!("Note: Chunking strategies are now in riptide-html crate");

    println!("\n✅ Trait-based strategy system demonstration complete!");
    println!("The system provides a clean, extensible architecture while maintaining");
    println!("backward compatibility with existing enum-based code.");
    println!("\nFor complete functionality including chunking and specialized extraction,");
    println!("see examples in riptide-html and riptide-intelligence crates.");

    Ok(())
}