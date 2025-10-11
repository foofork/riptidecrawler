# Strategy Composition Framework

The Strategy Composition framework enables easy chaining and combination of multiple extraction strategies with different execution modes.

## Overview

Strategy composition allows you to:
- **Chain** multiple strategies with sequential fallback
- **Parallelize** strategy execution with result merging
- **Implement fallbacks** with primary/secondary patterns
- **Select best results** based on confidence scores

## Composition Modes

### 1. Chain Mode

Executes strategies sequentially until one succeeds:

```rust
use riptide_core::strategy_composition::{CompositionMode, StrategyComposer};

let composer = StrategyComposer::new(CompositionMode::Chain)
    .add_strategy(trek_strategy)
    .add_strategy(css_strategy)
    .add_strategy(fallback_strategy)
    .with_timeout(5000);

let result = composer.execute(html, url).await?;
```

**Use Cases:**
- Fast primary strategy with reliable fallbacks
- Progressive degradation
- Resource-constrained environments

**Behavior:**
- Stops at first successful strategy
- Respects minimum confidence threshold
- Tracks individual strategy timings

### 2. Parallel Mode

Runs all strategies concurrently and merges results:

```rust
let composer = StrategyComposer::new(CompositionMode::Parallel)
    .with_strategies(vec![trek, css, llm])
    .with_merger(Box::new(BestContentMerger::default()))
    .with_timeout(5000);

let result = composer.execute(html, url).await?;
```

**Use Cases:**
- Maximum accuracy by combining multiple strategies
- A/B testing different extraction methods
- Quality validation through consensus

**Behavior:**
- Executes all strategies simultaneously
- Merges successful results
- Configurable result merging strategy

### 3. Fallback Mode

Primary strategy with backup:

```rust
let composer = StrategyComposer::new(CompositionMode::Fallback)
    .add_strategy(primary_strategy)    // Fast but may fail
    .add_strategy(fallback_strategy);  // Slower but reliable

let result = composer.execute(html, url).await?;
```

**Use Cases:**
- Fast path with reliable backup
- Network-dependent primary with local fallback
- Experimental primary with proven fallback

**Behavior:**
- Only executes fallback if primary fails
- Can trigger fallback based on confidence threshold
- Preserves metadata from both attempts

### 4. Best Mode

Executes all strategies and selects highest confidence result:

```rust
let composer = StrategyComposer::new(CompositionMode::Best)
    .with_strategies(vec![strategy1, strategy2, strategy3])
    .with_min_confidence(0.7);

let result = composer.execute(html, url).await?;
```

**Use Cases:**
- Quality-first extraction
- Multi-model consensus
- Maximizing extraction confidence

**Behavior:**
- Runs all strategies in parallel
- Selects result with highest confidence score
- Includes runner-up metadata for analysis

## Result Mergers

Result mergers combine outputs from multiple strategies in Parallel mode:

### UnionMerger

Combines all content from successful strategies:

```rust
use riptide_core::strategy_composition::{UnionMerger, MergerConfig};

let merger = UnionMerger::new(MergerConfig {
    min_confidence: 0.6,
    max_results: 5,
    weight_by_confidence: true,
    prefer_longer_content: true,
});
```

**Best for:** Comprehensive extraction, maximizing content coverage

### BestContentMerger

Picks best fields from different strategies:

```rust
let merger = BestContentMerger::new(MergerConfig::default());
```

**Best for:** High-quality extraction, selecting optimal components

## Configuration

### Composer Configuration

```rust
use riptide_core::strategy_composition::ComposerConfig;

let config = ComposerConfig {
    mode: CompositionMode::Chain,
    timeout_ms: 5000,           // Per-strategy timeout
    global_timeout_ms: 15000,   // Total composition timeout
    min_confidence: 0.6,        // Minimum acceptable confidence
    collect_metrics: true,      // Performance tracking
    max_concurrent: 4,          // Max parallel strategies
};

let composer = StrategyComposer::with_config(config)
    .with_strategies(strategies);
```

### Timeouts

Composition supports two levels of timeouts:

1. **Per-strategy timeout**: Maximum time for individual strategy
2. **Global timeout**: Maximum time for entire composition

```rust
let composer = StrategyComposer::new(CompositionMode::Chain)
    .with_timeout(5000)           // 5s per strategy
    .with_global_timeout(15000);  // 15s total
```

### Confidence Thresholds

Control when strategies are considered successful:

```rust
let composer = StrategyComposer::new(CompositionMode::Chain)
    .with_min_confidence(0.7);  // Require 70% confidence
```

## Performance Considerations

### Overhead Metrics

Composition adds minimal overhead:
- **Chain mode**: <5% overhead (sequential execution)
- **Parallel mode**: <10% overhead (concurrent coordination)
- **Best mode**: <8% overhead (parallel + comparison)

### Best Practices

1. **Use Chain for speed**: When first strategy usually succeeds
2. **Use Parallel for quality**: When accuracy matters most
3. **Limit concurrent strategies**: Set `max_concurrent` appropriately
4. **Set appropriate timeouts**: Balance speed vs. completeness
5. **Monitor metrics**: Use `collect_metrics` for optimization

## Result Structure

Composition results include comprehensive metadata:

```rust
pub struct CompositionResult {
    /// Final extraction result
    pub result: ExtractionResult,
    /// Composition mode used
    pub mode: CompositionMode,
    /// Number of strategies executed
    pub strategies_executed: usize,
    /// Number of strategies that succeeded
    pub strategies_succeeded: usize,
    /// Total execution time
    pub total_time: Duration,
    /// Individual strategy execution times
    pub strategy_times: HashMap<String, Duration>,
    /// Performance metrics
    pub metrics: Option<PerformanceMetrics>,
}
```

## Integration with Pipeline

Integrate composition with the strategies pipeline:

```rust
use riptide_api::strategies_pipeline::StrategiesPipelineOrchestrator;
use riptide_core::strategy_composition::{CompositionMode, StrategyComposer};

// Create composer with multiple strategies
let composer = StrategyComposer::new(CompositionMode::Chain)
    .with_strategies(vec![trek_strategy, css_strategy]);

// Use in pipeline
let html = fetch_content(url).await?;
let result = composer.execute(&html, url).await?;
```

## Error Handling

Composition provides detailed error context:

```rust
match composer.execute(html, url).await {
    Ok(result) => {
        println!("Success! Used {} strategies", result.strategies_executed);
        println!("Total time: {:?}", result.total_time);
    }
    Err(e) => {
        eprintln!("All strategies failed: {}", e);
        // Error includes details about each failed strategy
    }
}
```

## Testing

The framework includes comprehensive tests:

```bash
# Run all composition tests
cargo test --package riptide-core strategy_composition

# Run specific test suites
cargo test --package riptide-core strategy_composition::tests::test_chain_mode
cargo test --package riptide-core strategy_composition::tests::test_parallel_mode
```

## Examples

See `examples/strategy_composition_demo.rs` for complete examples:

```bash
cargo run --example strategy_composition_demo
```

## API Reference

Full API documentation:

```bash
cargo doc --package riptide-core --open
```

Navigate to `riptide_core::strategy_composition` module.
