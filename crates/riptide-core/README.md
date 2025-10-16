# riptide-core

Core infrastructure for the RipTide framework providing essential components for pipeline orchestration, resource management, and system reliability.

## Overview

`riptide-core` serves as the foundational layer of the RipTide framework, implementing critical infrastructure patterns for building robust, scalable data processing pipelines. It provides the core abstractions, reliability patterns, and resource management primitives that other RipTide crates build upon.

## Features

- **Pipeline Orchestration**: Component-based processing pipeline with flexible stage composition
- **Multi-Level Caching**: Configurable cache infrastructure with warming strategies and TTL management
- **Circuit Breakers**: Fault tolerance patterns for resilient service communication
- **Instance Pooling**: Efficient resource pooling with lifecycle management and health checks
- **Memory Management**: Advanced allocation tracking and cleanup strategies
- **Event Bus**: High-performance pub/sub messaging system for component communication
- **Telemetry**: Comprehensive metrics collection and monitoring infrastructure
- **Security**: Authentication, authorization, rate limiting, and input safety validation
- **Provider Traits**: Clean abstractions for external service integration
- **Spider System**: Deep web crawling with intelligent frontier management
- **Reliability**: Multi-strategy extraction with automatic fallback mechanisms

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      RipTide Core                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐   │
│  │  Pipeline   │  │   Resource   │  │   Reliability   │   │
│  │Orchestration│  │  Management  │  │    Patterns     │   │
│  └─────────────┘  └──────────────┘  └─────────────────┘   │
│        │                  │                    │            │
│        ├──────────────────┼────────────────────┤            │
│        ▼                  ▼                    ▼            │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐   │
│  │   Event     │  │    Cache     │  │    Circuit      │   │
│  │    Bus      │  │Infrastructure│  │   Breakers      │   │
│  └─────────────┘  └──────────────┘  └─────────────────┘   │
│        │                  │                    │            │
│        ├──────────────────┼────────────────────┤            │
│        ▼                  ▼                    ▼            │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐   │
│  │ Telemetry & │  │   Security   │  │    Provider     │   │
│  │  Monitoring │  │   & Safety   │  │     Traits      │   │
│  └─────────────┘  └──────────────┘  └─────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
┌──────────────┐   ┌──────────────┐   ┌──────────────┐
│ riptide-extraction │   │ riptide-     │   │ riptide-     │
│              │   │ intelligence │   │   search     │
└──────────────┘   └──────────────┘   └──────────────┘
```

## Module Overview

### Pipeline (`pipeline.rs`)
Component-based processing pipeline for orchestrating multi-stage data transformations. Supports dynamic stage composition, error handling, and progress tracking.

**Key Components:**
- `Pipeline`: Main orchestration engine
- `Stage`: Individual processing steps
- `Context`: Shared state and configuration

### Cache Infrastructure (`cache/`)
Multi-level caching system with intelligent warming strategies and configurable eviction policies.

**Features:**
- TTL-based expiration
- LRU/LFU eviction policies
- Cache warming on startup
- Hit/miss ratio tracking
- Distributed cache support

### Circuit Breakers (`circuit_breaker.rs`)
Implementation of the circuit breaker pattern for fault-tolerant service communication.

**States:**
- **Closed**: Normal operation, requests pass through
- **Open**: Failure threshold exceeded, requests fail fast
- **Half-Open**: Testing if service recovered

### Instance Pooling (`pool.rs`)
Efficient resource pooling with lifecycle management for expensive-to-create instances.

**Features:**
- Configurable min/max pool sizes
- Health checks and validation
- Automatic cleanup of stale instances
- Connection recycling
- Pool statistics

### Memory Management (`memory/`)
Advanced memory allocation tracking and cleanup strategies for long-running processes.

**Capabilities:**
- Allocation tracking
- Memory leak detection
- Automatic cleanup scheduling
- Memory pressure monitoring
- GC integration

### Event Bus (`event_bus.rs`)
High-performance publish/subscribe messaging system for decoupled component communication.

**Features:**
- Topic-based routing
- Async/sync handlers
- Message filtering
- Backpressure handling
- Event replay capability

### Telemetry (`telemetry/`)
Comprehensive metrics collection and monitoring infrastructure with multiple backend support.

**Metrics:**
- Counters, gauges, histograms
- Request latency tracking
- Error rate monitoring
- Resource utilization
- Custom business metrics

### Security (`security/`)
Security primitives for authentication, authorization, and input validation.

**Components:**
- Authentication providers
- Rate limiting
- Input sanitization
- Token management
- Safety validation

### Provider Traits (`providers/`)
Clean abstractions for integrating external services and data sources.

**Traits:**
- `Provider`: Base provider interface
- `SearchProvider`: Search service abstraction
- `LLMProvider`: Language model integration
- `StorageProvider`: Storage backend abstraction

### Spider (`spider/`)
Deep web crawling system with intelligent frontier management and URL filtering.

**Features:**
- Depth-first/breadth-first crawling
- Frontier queue management
- URL deduplication
- Domain filtering
- Robots.txt compliance

### Reliability (`reliability/`)
Multi-strategy extraction with automatic fallback mechanisms for robust data extraction.

**Strategies:**
- Primary extraction attempt
- Fallback to alternative methods
- Retry with exponential backoff
- Error recovery
- Success rate tracking

## Usage Examples

### Basic Pipeline Setup

```rust
use riptide_core::pipeline::{Pipeline, Stage, Context};

// Create a processing pipeline
let mut pipeline = Pipeline::new("data-processing");

// Add processing stages
pipeline.add_stage(Stage::new("fetch", |ctx| {
    // Fetch data
    Ok(ctx)
}));

pipeline.add_stage(Stage::new("transform", |ctx| {
    // Transform data
    Ok(ctx)
}));

pipeline.add_stage(Stage::new("store", |ctx| {
    // Store results
    Ok(ctx)
}));

// Execute pipeline
let context = Context::new();
let result = pipeline.execute(context).await?;
```

### Circuit Breaker Usage

```rust
use riptide_core::circuit_breaker::{CircuitBreaker, Config};

// Configure circuit breaker
let config = Config {
    failure_threshold: 5,
    timeout: Duration::from_secs(60),
    half_open_timeout: Duration::from_secs(30),
};

let breaker = CircuitBreaker::new("external-api", config);

// Execute with protection
let result = breaker.execute(|| async {
    external_api_call().await
}).await?;
```

### Cache Integration

```rust
use riptide_core::cache::{Cache, CacheConfig};

// Initialize cache
let config = CacheConfig {
    max_entries: 10_000,
    ttl: Duration::from_secs(3600),
    eviction_policy: EvictionPolicy::LRU,
};

let cache = Cache::new(config);

// Use cache
if let Some(value) = cache.get("key").await {
    // Cache hit
    return Ok(value);
}

// Cache miss - fetch and store
let value = fetch_from_source().await?;
cache.set("key", value.clone()).await;
```

### Instance Pool

```rust
use riptide_core::pool::{Pool, PoolConfig};

// Create connection pool
let config = PoolConfig {
    min_size: 5,
    max_size: 20,
    timeout: Duration::from_secs(30),
};

let pool = Pool::new(config, || {
    // Factory function
    create_connection()
});

// Get instance from pool
let conn = pool.get().await?;
// Use connection
conn.execute_query().await?;
// Automatically returned to pool on drop
```

### Event Bus

```rust
use riptide_core::event_bus::{EventBus, Event};

// Create event bus
let bus = EventBus::new();

// Subscribe to events
bus.subscribe("data.processed", |event| {
    println!("Received event: {:?}", event);
});

// Publish events
bus.publish(Event::new("data.processed", payload)).await;
```

### Spider Crawling

```rust
use riptide_core::spider::{Spider, SpiderConfig};

// Configure spider
let config = SpiderConfig {
    max_depth: 3,
    max_pages: 100,
    respect_robots_txt: true,
    concurrent_requests: 10,
};

let spider = Spider::new(config);

// Start crawling
spider.crawl("https://example.com").await?;

// Process discovered URLs
for url in spider.frontier().await {
    println!("Found: {}", url);
}
```

### Reliability with Fallbacks

```rust
use riptide_core::reliability::{ReliableExtractor, Strategy};

// Create extractor with fallback strategies
let extractor = ReliableExtractor::new(vec![
    Strategy::Primary(primary_extraction),
    Strategy::Fallback(css_extraction),
    Strategy::Fallback(regex_extraction),
]);

// Extract with automatic fallback
let result = extractor.extract(content).await?;
```

## Feature Flags

The crate supports several feature flags for conditional compilation:

- **`pdf`**: Enable PDF processing capabilities (moved to `riptide-pdf`)
- **`stealth`**: Enable basic stealth features (moved to `riptide-stealth`)
- **`full-stealth`**: Enable advanced stealth capabilities (moved to `riptide-stealth`)
- **`benchmarks`**: Include benchmark utilities and test harnesses
- **`api-integration`**: Enable external API integration helpers

```toml
[dependencies]
riptide-core = { version = "0.1", features = ["benchmarks", "api-integration"] }
```

## Integration with Other Crates

### riptide-extraction
Provides HTML parsing and CSS/Regex extraction capabilities that build on core pipeline infrastructure.

```rust
use riptide_core::pipeline::Pipeline;
use riptide_html::extractors::CssExtractor;

let mut pipeline = Pipeline::new("html-processing");
pipeline.add_stage(CssExtractor::new("article.content"));
```

### riptide-intelligence
Integrates LLM capabilities using core provider traits and reliability patterns.

```rust
use riptide_core::providers::LLMProvider;
use riptide_intelligence::OpenAIProvider;

let provider: Box<dyn LLMProvider> = Box::new(OpenAIProvider::new(api_key));
```

### riptide-search
Implements search provider abstractions defined in core.

```rust
use riptide_core::providers::SearchProvider;
use riptide_search::BraveSearch;

let search: Box<dyn SearchProvider> = Box::new(BraveSearch::new(api_key));
```

### riptide-pdf
PDF processing that leverages core caching and reliability mechanisms.

```rust
use riptide_core::cache::Cache;
use riptide_pdf::PdfExtractor;

let extractor = PdfExtractor::new(cache);
```

### riptide-stealth
Stealth capabilities that use core circuit breakers and telemetry.

```rust
use riptide_core::circuit_breaker::CircuitBreaker;
use riptide_stealth::StealthClient;

let client = StealthClient::with_breaker(breaker);
```

## Testing

Run the test suite:

```bash
# Run all tests
cargo test -p riptide-core

# Run with all features
cargo test -p riptide-core --all-features

# Run specific test module
cargo test -p riptide-core --test cache_tests

# Run benchmarks (requires nightly)
cargo +nightly bench -p riptide-core --features benchmarks
```

### Integration Tests

Integration tests are located in `tests/` and cover:
- End-to-end pipeline execution
- Cache coherency across operations
- Circuit breaker state transitions
- Pool resource management
- Event bus message delivery
- Spider crawling behavior

## Performance Considerations

### Pipeline Optimization
- Use async stages for I/O-bound operations
- Minimize context cloning between stages
- Leverage parallel stage execution where possible
- Profile with `--features benchmarks` flag

### Cache Tuning
- Set appropriate TTL based on data freshness requirements
- Choose eviction policy matching access patterns (LRU vs LFU)
- Monitor hit rates and adjust max_entries accordingly
- Use cache warming for predictable startup performance

### Circuit Breaker Configuration
- Set failure_threshold based on acceptable error rates
- Adjust timeout to match downstream service SLAs
- Use shorter half_open_timeout for faster recovery detection
- Monitor state transitions for capacity planning

### Memory Management
- Enable allocation tracking in development environments
- Schedule cleanup during low-traffic periods
- Monitor memory pressure metrics
- Use pooling for frequently allocated resources

### Event Bus Scaling
- Use topic filtering to reduce unnecessary processing
- Implement backpressure handling for slow consumers
- Consider message batching for high-throughput scenarios
- Profile handler execution times

## Architecture Guidelines

### Separation of Concerns
- **Core**: Infrastructure primitives (this crate)
- **HTML**: CSS/Regex extraction (`riptide-extraction`)
- **Intelligence**: LLM functionality (`riptide-intelligence`)
- **Search**: Search providers (`riptide-search`)
- **PDF**: PDF processing (`riptide-pdf`)
- **Stealth**: Anti-detection (`riptide-stealth`)

### Extension Points
Extend core functionality by:
1. Implementing provider traits
2. Creating custom pipeline stages
3. Adding telemetry collectors
4. Implementing cache backends
5. Creating custom circuit breaker policies

## Contributing

When contributing to `riptide-core`:

1. Keep the core focused on infrastructure primitives
2. Domain-specific functionality belongs in specialized crates
3. Maintain backward compatibility for public APIs
4. Add comprehensive tests for new features
5. Update documentation and examples
6. Profile performance-critical paths
7. Follow the project's SPARC methodology

## License

See the root `LICENSE` file for license information.

## Related Crates

- [`riptide-extraction`](../riptide-extraction): HTML parsing and extraction
- [`riptide-intelligence`](../riptide-intelligence): LLM integration
- [`riptide-search`](../riptide-search): Search provider implementations
- [`riptide-pdf`](../riptide-pdf): PDF processing
- [`riptide-stealth`](../riptide-stealth): Anti-detection capabilities
- [`riptide-api`](../riptide-api): REST API layer
- [`riptide-streaming`](../riptide-streaming): Real-time data processing
