# RipTide Pool

**Infrastructure Layer - Resource Pooling Adapter**

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

Resource pooling and lifecycle management for the RipTide framework, providing efficient pooling for WASM instances, native extractors, and other expensive-to-create resources.

## Quick Overview

RipTide Pool is the infrastructure adapter that manages expensive resource lifecycles through pooling patterns. It provides both WASM instance pooling and native extractor pooling with health monitoring, circuit breakers, and memory management.

**What it does:**
- Pools WASM component instances for extraction operations
- Pools native CSS and Regex extractors for fast extraction
- Manages resource lifecycle (creation, validation, cleanup)
- Integrates health monitoring and circuit breakers
- Provides event-driven pool coordination

**Port Implementation:**
- Implements `Pool<T>` generic adapter pattern
- Provides `AdvancedInstancePool` for WASM instances
- Provides `NativeExtractorPool` for CSS/Regex extractors
- Integrates with `EventBus` for pub/sub messaging

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                   RipTide Pool Layer                         │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │    WASM     │  │    Native    │  │     Health       │   │
│  │  Instance   │  │   Extractor  │  │   Monitoring     │   │
│  │    Pool     │  │     Pool     │  │  & Metrics       │   │
│  └─────────────┘  └──────────────┘  └──────────────────┘   │
│         │                 │                    │            │
│         └─────────────────┴────────────────────┘            │
│                           │                                 │
│                  ┌────────▼────────┐                        │
│                  │   Circuit       │                        │
│                  │   Breaker &     │                        │
│                  │   Event Bus     │                        │
│                  └─────────────────┘                        │
└──────────────────────────────────────────────────────────────┘
```

## Port Implementation

This adapter implements generic resource pooling patterns as infrastructure adapters.

### `Pool<T>` Generic Pattern

The pool adapter provides a generic pooling pattern for any resource type:

```rust
pub trait Poolable: Send + Sync {
    async fn is_healthy(&self) -> bool;
    async fn reset(&mut self) -> Result<()>;
}
```

### Why This Adapter Exists

The `riptide-pool` adapter exists to:
1. **Optimize performance** - Reuse expensive-to-create resources (WASM instances, extractors)
2. **Manage lifecycle** - Automatic creation, validation, and cleanup
3. **Ensure reliability** - Health checks and circuit breakers prevent using unhealthy resources
4. **Enable observability** - Event emission for pool operations and metrics
5. **Support scalability** - Dynamic pool sizing based on demand

## Configuration

### Environment Variables

```bash
# WASM Pool Configuration
WASM_POOL_SIZE=10
WASM_MAX_INSTANCES=50
WASM_IDLE_TIMEOUT_MS=60000
WASM_HEALTH_CHECK_INTERVAL_MS=30000

# Native Pool Configuration
NATIVE_POOL_SIZE=20
NATIVE_MAX_INSTANCES=100
NATIVE_IDLE_TIMEOUT_MS=30000

# Memory Management
WASM_MEMORY_LIMIT_MB=512
MEMORY_PRESSURE_THRESHOLD=0.8
GC_INTERVAL_MS=60000

# Circuit Breaker
CB_FAILURE_THRESHOLD=5
CB_SUCCESS_THRESHOLD=2
CB_TIMEOUT_MS=30000

# Health Monitoring
HEALTH_CHECK_ENABLED=true
HEALTH_CHECK_INTERVAL_MS=10000
```

### Programmatic Configuration

```rust
use riptide_pool::{NativeExtractorPool, NativePoolConfig};
use std::time::Duration;

let config = NativePoolConfig {
    min_pool_size: 5,
    max_pool_size: 20,
    idle_timeout: Duration::from_secs(60),
    enable_health_checks: true,
    health_check_interval: Duration::from_secs(10),
    enable_metrics: true,
};

let pool = NativeExtractorPool::new(config).await?;
```

## Usage Examples

### Native Extractor Pool (CSS & Regex)

The native extractor pool provides high-performance pooling for CSS and Regex extractors:

```rust
use riptide_pool::{NativeExtractorPool, NativePoolConfig, NativeExtractorType};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create native extractor pool
    let pool = NativeExtractorPool::new(NativePoolConfig::default()).await?;

    // Checkout CSS extractor
    let extractor = pool.checkout(NativeExtractorType::Css).await?;

    // Use extractor
    let html = r#"<html><body><h1>Title</h1><p>Content</p></body></html>"#;
    let result = extractor.extract(html, "h1").await?;
    println!("Extracted: {}", result);

    // Extractor returns to pool when dropped
    drop(extractor);

    // Get pool metrics
    let metrics = pool.metrics().await;
    println!("CSS extractors: {}/{}", metrics.css_in_use, metrics.css_total);
    println!("Regex extractors: {}/{}", metrics.regex_in_use, metrics.regex_total);

    Ok(())
}
```

### WASM Instance Pool

The WASM instance pool manages WebAssembly component instances:

```rust
#[cfg(feature = "wasm-pool")]
use riptide_pool::{AdvancedInstancePool, ExtractorConfig};
#[cfg(feature = "wasm-pool")]
use wasmtime::Engine;

#[cfg(feature = "wasm-pool")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create WASM engine
    let engine = Engine::default();

    // Configure pool
    let config = ExtractorConfig {
        pool_size: 10,
        max_instances: 50,
        memory_limit_mb: 512,
        enable_health_checks: true,
        ..Default::default()
    };

    // Create pool
    let pool = AdvancedInstancePool::new(
        config,
        engine,
        "path/to/extractor.wasm"
    ).await?;

    // Checkout instance
    let instance = pool.checkout().await?;

    // Use instance for extraction
    let result = instance.extract(html, "css-selector").await?;

    // Instance returns to pool when dropped
    drop(instance);

    Ok(())
}
```

### Event-Aware Pool

Integrate pool with event bus for pub/sub messaging:

```rust
#[cfg(feature = "wasm-pool")]
use riptide_pool::{create_event_aware_pool, ExtractorConfig};
#[cfg(feature = "wasm-pool")]
use riptide_events::EventBus;
#[cfg(feature = "wasm-pool")]
use wasmtime::Engine;

#[cfg(feature = "wasm-pool")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let engine = Engine::default();
    let event_bus = EventBus::new();

    // Create event-aware pool
    let pool = create_event_aware_pool(
        ExtractorConfig::default(),
        engine,
        "extractor.wasm",
        event_bus.clone()
    ).await?;

    // Subscribe to pool events
    let mut events = event_bus.subscribe().await;

    tokio::spawn(async move {
        while let Ok(event) = events.recv().await {
            match event {
                PoolEvent::InstanceCreated { id } => {
                    println!("Instance created: {}", id);
                }
                PoolEvent::HealthCheckFailed { id, reason } => {
                    eprintln!("Health check failed: {} - {}", id, reason);
                }
                PoolEvent::MemoryPressure { level } => {
                    warn!("Memory pressure: {:?}", level);
                }
                _ => {}
            }
        }
    });

    Ok(())
}
```

### Health Monitoring

Monitor pool health and performance:

```rust
use riptide_pool::{NativeExtractorPool, PoolHealthStatus};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = NativeExtractorPool::new(Default::default()).await?;

    // Get health status
    let health = pool.health_status().await;

    println!("Overall health: {:?}", health.level);
    println!("Pool utilization: {:.1}%", health.utilization * 100.0);
    println!("Memory usage: {} MB", health.memory_usage_mb);
    println!("Health score: {}/100", health.score);

    // Check specific health aspects
    if health.memory_pressure > 0.8 {
        eprintln!("High memory pressure: {:.1}%", health.memory_pressure * 100.0);
    }

    if health.error_rate > 0.05 {
        eprintln!("High error rate: {:.2}%", health.error_rate * 100.0);
    }

    Ok(())
}
```

### Circuit Breaker Integration

Pool integrates with circuit breaker for fault tolerance:

```rust
use riptide_pool::CircuitBreakerState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = NativeExtractorPool::new(Default::default()).await?;

    // Get circuit breaker state
    let cb_state = pool.circuit_breaker_state().await;

    match cb_state {
        CircuitBreakerState::Closed => {
            println!("Circuit breaker closed - normal operation");
        }
        CircuitBreakerState::Open => {
            eprintln!("Circuit breaker open - pool is failing");
            // Implement fallback strategy
        }
        CircuitBreakerState::HalfOpen => {
            println!("Circuit breaker half-open - testing recovery");
        }
    }

    Ok(())
}
```

### Memory Management

Track and manage pool memory usage:

```rust
#[cfg(feature = "wasm-pool")]
use riptide_pool::MemoryStats;

#[cfg(feature = "wasm-pool")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = /* ... */;

    // Get memory statistics
    let mem_stats = pool.memory_stats().await;

    println!("Total allocated: {} MB", mem_stats.total_allocated_mb);
    println!("Currently in use: {} MB", mem_stats.in_use_mb);
    println!("Peak usage: {} MB", mem_stats.peak_mb);
    println!("GC runs: {}", mem_stats.gc_count);

    // Trigger manual garbage collection if needed
    if mem_stats.in_use_mb > 400 {
        pool.force_gc().await?;
        println!("Garbage collection triggered");
    }

    Ok(())
}
```

## Technical Details

### External Dependencies

- **wasmtime**: WebAssembly runtime (optional, `wasm-pool` feature)
- **scraper**: HTML parsing for native extractors
- **tokio**: Async runtime for pool operations
- **riptide-events**: Event bus integration
- **riptide-extraction**: Extractor implementations (optional)

### Resource Lifecycle

**Native Extractor Lifecycle:**
```
┌─────────────┐
│   Create    │  Extractor instantiated
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Warmup    │  Pre-compile patterns/selectors
└──────┬──────┘
       │
       ▼
┌─────────────┐
│    Pool     │  Available for checkout
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Checkout   │  In use by extraction operation
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Return    │  Back to pool
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Cleanup    │  If idle too long or unhealthy
└─────────────┘
```

**WASM Instance Lifecycle:**
```
┌─────────────┐
│   Compile   │  WASM module compiled
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Instantiate │  Instance created with memory
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Health    │  Periodic health checks
│   Check     │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Checkout   │  Instance assigned to operation
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Cleanup    │  Memory reclaimed, instance dropped
└─────────────┘
```

### Performance Characteristics

| Pool Type | Checkout Latency | Memory per Instance | Max Instances |
|-----------|------------------|---------------------|---------------|
| Native CSS | <1ms | ~1 MB | 100+ |
| Native Regex | <1ms | ~500 KB | 100+ |
| WASM (cold) | 50-100ms | 10-50 MB | 50 |
| WASM (warm) | <5ms | 10-50 MB | 50 |

### Memory Management

**Native Extractors:**
- Minimal memory overhead (~1-2 MB per extractor)
- Compiled regex patterns cached
- Parsed CSS selectors cached

**WASM Instances:**
- Each instance: 10-50 MB (configurable via `memory_limit_mb`)
- Memory tracked per instance
- Automatic GC when memory pressure exceeds threshold
- Memory snapshots for profiling

## Anti-Corruption Layer

The pool adapter works with domain operations without exposing pooling implementation:

```rust
// Domain operation (doesn't know about pooling)
async fn extract_content(html: &str, selector: &str) -> Result<String> {
    // Extraction logic
}

// Pool adapter transparently provides extractor
let pool = NativeExtractorPool::new(config).await?;
let extractor = pool.checkout(NativeExtractorType::Css).await?;

// Use extractor (looks like direct usage to domain layer)
let result = extract_content(html, selector).await?;

// Pool manages lifecycle transparently
```

## Testing

### Unit Tests

```bash
# Run all tests
cargo test -p riptide-pool

# Run native pool tests
cargo test -p riptide-pool native_pool

# Run with WASM features
cargo test -p riptide-pool --features wasm-pool

# Run with output
cargo test -p riptide-pool -- --nocapture
```

### Integration Tests

```rust
// tests/pool_integration_test.rs
use riptide_pool::{NativeExtractorPool, NativePoolConfig, NativeExtractorType};

#[tokio::test]
async fn test_pool_checkout_return() {
    let pool = NativeExtractorPool::new(NativePoolConfig {
        min_pool_size: 2,
        max_pool_size: 5,
        ..Default::default()
    }).await.unwrap();

    // Checkout extractor
    let extractor = pool.checkout(NativeExtractorType::Css).await.unwrap();

    // Verify pool metrics
    let metrics = pool.metrics().await;
    assert_eq!(metrics.css_in_use, 1);

    // Return to pool (via drop)
    drop(extractor);

    // Verify returned
    let metrics = pool.metrics().await;
    assert_eq!(metrics.css_in_use, 0);
}
```

### Benchmarks

```bash
# Run pool performance benchmarks
cargo bench -p riptide-pool

# Benchmark native pool checkout
cargo bench -p riptide-pool --bench native_pool_bench
```

## Error Handling

### Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PoolError {
    #[error("Pool exhausted - no available instances")]
    PoolExhausted,

    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),

    #[error("Memory limit exceeded: {used}MB / {limit}MB")]
    MemoryLimitExceeded { used: usize, limit: usize },

    #[error("Instance creation failed: {0}")]
    CreationFailed(String),

    #[error("Circuit breaker open")]
    CircuitBreakerOpen,
}
```

### Recovery Strategies

When pool encounters errors:
1. **Pool Exhausted**: Wait for instance to return or create new (if under max)
2. **Health Check Failed**: Remove unhealthy instance, create replacement
3. **Memory Exceeded**: Trigger GC, reduce pool size
4. **Circuit Breaker Open**: Stop creating instances, use cached/fallback

## Production Considerations

### Resource Limits

**Native Extractor Pool:**
- Memory: ~1-2 MB per extractor
- Recommended max pool size: 50-100
- Suitable for high-throughput scenarios

**WASM Instance Pool:**
- Memory: 10-50 MB per instance
- Recommended max pool size: 20-50
- Monitor memory usage closely

### Connection Pooling

**Recommended Pool Sizes:**

| Load Level | Native Min/Max | WASM Min/Max |
|-----------|----------------|--------------|
| Low | 5/20 | 2/10 |
| Medium | 10/50 | 5/20 |
| High | 20/100 | 10/50 |

### Monitoring and Metrics

```rust
// Export metrics to Prometheus/monitoring system
let metrics = pool.metrics().await;

println!("Pool metrics:");
println!("  Total instances: {}", metrics.total);
println!("  In use: {}", metrics.in_use);
println!("  Available: {}", metrics.available);
println!("  Utilization: {:.1}%", metrics.utilization * 100.0);
println!("  Checkout wait time: {}ms", metrics.avg_wait_time_ms);
println!("  Instance creation time: {}ms", metrics.avg_creation_time_ms);
```

### Failure Modes

**Pool Exhaustion:**
- All instances in use → Increase `max_pool_size`
- Slow operations → Optimize extraction logic
- Resource leak → Check that instances are returned

**Memory Leak:**
- Memory usage growing → Enable GC, reduce `memory_limit_mb`
- WASM instances not cleaned → Check instance lifecycle
- Native extractors accumulating → Verify idle timeout

**Health Check Failures:**
- Instances unhealthy → Check extractor implementation
- Frequent replacements → Investigate root cause
- Circuit breaker opening → Service degradation detected

## Dependencies

### External Systems Required

None (self-contained, no external services)

### Rust Crate Dependencies

| Dependency | Purpose |
|------------|---------|
| wasmtime | WASM runtime (optional) |
| scraper | HTML parsing for native extractors |
| tokio | Async runtime |
| riptide-events | Event bus integration |
| riptide-extraction | Extractor implementations (optional) |
| uuid | Instance identifiers |
| serde/serde_json | Configuration serialization |

## Feature Flags

```toml
[dependencies]
riptide-pool = { version = "0.9", features = ["wasm-pool", "native-pool"] }
```

**Available Features:**
- `native-pool` (default): Native CSS/Regex extractor pooling
- `wasm-pool`: WASM instance pooling (requires wasmtime)

## Performance Tips

1. **Use native pools** when possible (much faster than WASM)
2. **Set appropriate min/max sizes** based on load patterns
3. **Enable health checks** to remove unhealthy instances
4. **Monitor memory usage** especially for WASM pools
5. **Tune idle timeout** to balance memory vs warmup time
6. **Use event integration** for real-time monitoring

## Related Crates

- **riptide-extraction**: Provides extractors that are pooled
- **riptide-events**: Event bus integration
- **riptide-monitoring**: Metrics collection
- **riptide-cache**: Cache integration for WASM modules
- **riptide-browser**: Browser pooling builds on this

## License

Apache-2.0
