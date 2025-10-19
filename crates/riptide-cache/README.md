# RipTide Cache

Cache management system for the RipTide extraction framework, providing high-performance distributed caching with Redis backend.

## Overview

`riptide-cache` implements a comprehensive caching layer for RipTide, supporting distributed cache operations, intelligent TTL management, and cache warming strategies. Built on Redis for reliability and performance.

## Features

- **Distributed Caching**: Redis-backed distributed cache for multi-instance deployments
- **TTL Management**: Configurable time-to-live with automatic expiration
- **Cache Warming**: Pre-populate cache on startup for optimal performance
- **Hit/Miss Tracking**: Built-in metrics for cache effectiveness monitoring
- **Eviction Policies**: LRU, LFU, and custom eviction strategies
- **Namespace Support**: Logical cache separation for different data types
- **Compression**: Automatic compression for large cached values
- **Atomic Operations**: Thread-safe concurrent access patterns

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                RipTide Cache Layer                       │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │   Cache     │  │  Namespace  │  │  Eviction   │     │
│  │  Manager    │  │   Manager   │  │  Strategy   │     │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘     │
│         │                │                 │            │
│         └────────────────┼─────────────────┘            │
│                          ▼                              │
│                  ┌───────────────┐                      │
│                  │  Redis Client │                      │
│                  │  (Connection  │                      │
│                  │     Pool)     │                      │
│                  └───────┬───────┘                      │
└────────────────────────────┼─────────────────────────────┘
                            │
                            ▼
                    ┌──────────────┐
                    │ Redis Server │
                    │   (7.0+)     │
                    └──────────────┘
```

## Usage

### Basic Cache Operations

```rust
use riptide_cache::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize cache
    let cache = CacheManager::new("redis://localhost:6379").await?;

    // Store value with TTL
    cache.set("key", "value", Duration::from_secs(3600)).await?;

    // Retrieve value
    let value: Option<String> = cache.get("key").await?;

    // Check existence
    if cache.exists("key").await? {
        println!("Key exists in cache");
    }

    // Delete value
    cache.del("key").await?;

    Ok(())
}
```

### Advanced Features

```rust
use riptide_cache::*;

// Cache with namespace
let cache = CacheManager::with_namespace("html_content").await?;

// Bulk operations
let keys = vec!["key1", "key2", "key3"];
let values = cache.mget(&keys).await?;

cache.mset(&[
    ("key1", "value1"),
    ("key2", "value2"),
]).await?;

// Cache warming on startup
cache.warm(|manager| async move {
    manager.set("popular_key", "preloaded_value", TTL).await
}).await?;

// Metrics and statistics
let stats = cache.stats().await?;
println!("Hit rate: {:.2}%", stats.hit_rate * 100.0);
println!("Total hits: {}", stats.hits);
println!("Total misses: {}", stats.misses);
```

### TTL and Expiration

```rust
use riptide_cache::*;
use std::time::Duration;

// Set with custom TTL
cache.set_ex("temp_key", "value", Duration::from_secs(60)).await?;

// Update TTL
cache.expire("key", Duration::from_secs(3600)).await?;

// Get remaining TTL
let ttl = cache.ttl("key").await?;
println!("Expires in {} seconds", ttl);

// Persist (remove TTL)
cache.persist("key").await?;
```

### Namespace Management

```rust
use riptide_cache::*;

// Create namespaced cache
let html_cache = CacheManager::with_namespace("html").await?;
let json_cache = CacheManager::with_namespace("json").await?;

// Operations are isolated
html_cache.set("data", "<html>...</html>", TTL).await?;
json_cache.set("data", "{...}", TTL).await?;

// Clear namespace
html_cache.clear_namespace().await?;
```

## Configuration

### Environment Variables

```bash
# Redis connection
REDIS_URL=redis://localhost:6379/0
REDIS_POOL_SIZE=10
REDIS_TIMEOUT_MS=5000

# Cache settings
CACHE_DEFAULT_TTL=3600
CACHE_MAX_SIZE_MB=1024
CACHE_ENABLE_COMPRESSION=true
CACHE_COMPRESSION_THRESHOLD=1024
```

### Programmatic Configuration

```rust
use riptide_cache::*;

let config = CacheConfig {
    redis_url: "redis://localhost:6379".to_string(),
    pool_size: 20,
    timeout: Duration::from_secs(5),
    default_ttl: Duration::from_secs(3600),
    enable_compression: true,
    compression_threshold: 1024,
};

let cache = CacheManager::with_config(config).await?;
```

## Performance Optimization

### Connection Pooling

```rust
// Configure connection pool
let cache = CacheManager::builder()
    .pool_size(20)  // More connections for high concurrency
    .build()
    .await?;
```

### Batch Operations

```rust
// Use batch operations for multiple keys
let results = cache.mget(&["key1", "key2", "key3"]).await?;

// Better than:
// let r1 = cache.get("key1").await?;
// let r2 = cache.get("key2").await?;
// let r3 = cache.get("key3").await?;
```

### Compression

```rust
// Automatic compression for large values
let large_html = "..."; // 10 KB
cache.set("page", large_html, TTL).await?; // Auto-compressed
```

## Monitoring

### Cache Statistics

```rust
let stats = cache.stats().await?;

println!("Cache Statistics:");
println!("  Hit Rate: {:.2}%", stats.hit_rate * 100.0);
println!("  Hits: {}", stats.hits);
println!("  Misses: {}", stats.misses);
println!("  Keys: {}", stats.key_count);
println!("  Memory: {} MB", stats.memory_mb);
```

### Health Checks

```rust
// Check cache health
if cache.is_healthy().await? {
    println!("Cache is operational");
} else {
    println!("Cache connection issues");
}

// Test latency
let latency = cache.ping().await?;
println!("Cache latency: {}ms", latency.as_millis());
```

## Integration with RipTide

This crate is used by:

- **riptide-api**: API response caching
- **riptide-core**: Pipeline result caching
- **riptide-fetch**: HTTP response caching
- **riptide-extraction**: Extracted content caching

## Testing

```bash
# Start Redis for tests
docker run -d --name redis-test -p 6379:6379 redis:7-alpine

# Run tests
cargo test -p riptide-cache

# Run with coverage
cargo tarpaulin -p riptide-cache --out Html
```

## License

Apache-2.0

## Related Crates

- **riptide-core**: Core framework integration
- **riptide-persistence**: Long-term data storage
- **riptide-monitoring**: Cache metrics collection
