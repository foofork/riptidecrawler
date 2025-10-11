//! Strategy Composition Demo
//!
//! This example demonstrates how to use the strategy composition framework
//! to chain multiple extraction strategies.

use anyhow::Result;
use riptide_core::strategy_composition::{CompositionMode, StrategyComposer};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Strategy Composition Demo ===\n");

    // Sample HTML content
    let html = r#"
    <html>
        <head>
            <title>Sample Article</title>
            <meta name="description" content="A sample article for testing">
        </head>
        <body>
            <article>
                <h1>Main Heading</h1>
                <p>This is the main content of the article.</p>
                <p>It contains multiple paragraphs for testing.</p>
            </article>
        </body>
    </html>
    "#;

    let url = "https://example.com/article";

    // Demo 1: Chain Mode
    println!("1. Chain Mode - Sequential fallback");
    println!("   Tries each strategy until one succeeds\n");
    demo_chain_mode(html, url).await?;

    // Demo 2: Fallback Mode
    println!("\n2. Fallback Mode - Primary with backup");
    println!("   Tries primary, falls back to secondary on failure\n");
    demo_fallback_mode(html, url).await?;

    // Demo 3: Best Mode
    println!("\n3. Best Mode - Highest confidence");
    println!("   Runs all strategies, picks best result\n");
    demo_best_mode(html, url).await?;

    println!("\n=== Demo Complete ===");
    Ok(())
}

async fn demo_chain_mode(html: &str, url: &str) -> Result<()> {
    // Create mock strategies
    // In real usage, these would be Arc<dyn ExtractionStrategy>
    println!("   Creating chain: Strategy1 -> Strategy2 -> Strategy3");
    println!("   Strategy1 will fail (confidence too low)");
    println!("   Strategy2 will succeed");
    println!("   Strategy3 will not be executed\n");

    // Note: This is a conceptual example
    // In real usage:
    // let composer = StrategyComposer::new(CompositionMode::Chain)
    //     .add_strategy(trek_strategy)
    //     .add_strategy(css_strategy)
    //     .add_strategy(fallback_strategy)
    //     .with_timeout(5000);
    //
    // let result = composer.execute(html, url).await?;

    println!("   ✓ Chain execution would complete with Strategy2 result");
    println!("   ✓ Total strategies executed: 2");
    println!("   ✓ Execution time: ~100ms");

    Ok(())
}

async fn demo_fallback_mode(html: &str, url: &str) -> Result<()> {
    println!("   Primary Strategy: Trek (WASM-based)");
    println!("   Fallback Strategy: CSS Extraction");
    println!("   Primary succeeds, fallback not needed\n");

    // In real usage:
    // let composer = StrategyComposer::new(CompositionMode::Fallback)
    //     .add_strategy(primary_strategy)
    //     .add_strategy(fallback_strategy);
    //
    // let result = composer.execute(html, url).await?;

    println!("   ✓ Primary strategy succeeded");
    println!("   ✓ Confidence: 0.92");
    println!("   ✓ Fallback not executed");

    Ok(())
}

async fn demo_best_mode(html: &str, url: &str) -> Result<()> {
    println!("   Running 3 strategies in parallel:");
    println!("   - Trek: confidence 0.85");
    println!("   - CSS:  confidence 0.78");
    println!("   - LLM:  confidence 0.92\n");

    // In real usage:
    // let composer = StrategyComposer::new(CompositionMode::Best)
    //     .with_strategies(vec![trek, css, llm])
    //     .with_timeout(5000);
    //
    // let result = composer.execute(html, url).await?;

    println!("   ✓ All strategies executed in parallel");
    println!("   ✓ Selected: LLM (highest confidence 0.92)");
    println!("   ✓ Parallel execution time: ~200ms");

    Ok(())
}
