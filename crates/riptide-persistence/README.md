# RipTide Persistence

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

Advanced persistence layer for RipTide with Redis/DragonflyDB backend, providing high-performance caching, multi-tenancy support, and comprehensive state management capabilities.

## Overview

The `riptide-persistence` crate delivers a production-ready persistence infrastructure designed for high-throughput, multi-tenant applications. Built on Redis/DragonflyDB, it provides sub-5ms cache access times, comprehensive tenant isolation, and sophisticated state management with automatic checkpoint/restore capabilities.

### Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                   RipTide Persistence Layer                  │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │   Cache     │  │    State     │  │  Multi-Tenancy  │   │
│  │  Manager    │  │   Manager    │  │     Manager     │   │
│  └─────────────┘  └──────────────┘  └──────────────────┘   │
│         │                 │                    │            │
│         └─────────────────┴────────────────────┘            │
│                           │                                 │
│                  ┌────────▼────────┐                        │
│                  │  Redis/Dragonfly│                        │
│                  │     Backend     │                        │
│                  └─────────────────┘                        │
└──────────────────────────────────────────────────────────────┘
```

## Key Features

### High-Performance Caching
- **Sub-5ms Access Time**: Optimized cache operations with connection pooling and pipelining
- **Automatic Compression**: LZ4 and Zstd support with intelligent compression thresholds
- **TTL-Based Invalidation**: Automatic cache expiration with configurable policies
- **Batch Operations**: Efficient bulk get/set operations for high-throughput scenarios
- **Cache Warming**: Startup optimization for frequently accessed data
- **Memory Spillover**: Automatic disk spillover when memory thresholds are exceeded

### Multi-Tenancy Support
- **Strong Tenant Isolation**: Complete data separation with namespace-based keys
- **Resource Quotas**: Per-tenant limits on memory, storage, operations, and data transfer
- **Billing Integration**: Usage tracking with operations, data transfer, and compute time
- **Security Boundaries**: Tenant-specific encryption keys and access control policies
- **Rate Limiting**: Per-tenant rate limits to prevent resource abuse
- **Usage Analytics**: Real-time monitoring of tenant resource consumption

### State Management
- **Session Persistence**: Durable session storage with configurable TTL
- **Configuration Hot-Reload**: Watch configuration files and reload without downtime
- **Checkpoint/Restore**: Full system state preservation for disaster recovery
- **Graceful Shutdown**: State preservation during application shutdown
- **Session Spillover**: Automatic LRU-based disk spillover for memory management

### Advanced Caching Strategies

#### Write-Through Caching
```rust
cache_manager.set("key", &data, namespace, ttl, metadata).await?;
```

#### Write-Behind with Batching
```rust
let mut batch = HashMap::new();
batch.insert("key1", data1);
batch.insert("key2", data2);
cache_manager.set_batch(batch, namespace, ttl).await?;
```

#### Cache-Aside Pattern
```rust
if let Some(cached) = cache_manager.get("key", namespace).await? {
    return Ok(cached);
}
let data = fetch_from_source().await?;
cache_manager.set("key", &data, namespace, ttl, None).await?;
```

### Compression Options

The persistence layer supports multiple compression algorithms optimized for different use cases:

| Algorithm | Speed | Ratio | Best For |
|-----------|-------|-------|----------|
| **LZ4** | Very Fast | Good | Real-time applications, frequent access |
| **Zstd** | Fast | Excellent | Archival data, infrequent access |
| **None** | N/A | N/A | Pre-compressed or small data |

Compression is automatically applied when:
- Data exceeds configured threshold (default: 1KB)
- Compression achieves >10% size reduction
- Feature flag `compression` is enabled (default)

## Configuration

### Basic Configuration

```rust
use riptide_persistence::{PersistenceConfig, config::*};

let config = PersistenceConfig {
    redis: RedisConfig {
        url: "redis://localhost:6379".to_string(),
        pool_size: 10,
        connection_timeout_ms: 5000,
        command_timeout_ms: 5000,
        cluster_mode: false,
        retry_attempts: 3,
        retry_delay_ms: 100,
        enable_pipelining: true,
        max_pipeline_size: 100,
    },
    cache: CacheConfig {
        default_ttl_seconds: 24 * 60 * 60,      // 24 hours
        max_entry_size_bytes: 20 * 1024 * 1024, // 20MB
        key_prefix: "riptide".to_string(),
        version: "v1".to_string(),
        enable_compression: true,
        compression_threshold_bytes: 1024,
        compression_algorithm: CompressionAlgorithm::Lz4,
        enable_warming: true,
        warming_batch_size: 100,
        max_memory_bytes: Some(1024 * 1024 * 1024), // 1GB
        eviction_policy: EvictionPolicy::LRU,
    },
    // ... additional configuration
};
```

### Environment Variable Configuration

```bash
# Redis Configuration
export REDIS_URL="redis://localhost:6379"

# Cache Configuration
export CACHE_DEFAULT_TTL_SECONDS=86400
export ENABLE_COMPRESSION=true

# Multi-Tenancy
export ENABLE_MULTI_TENANCY=true
```

### Configuration Hot-Reload

The state manager supports configuration hot-reload, automatically detecting and applying configuration changes:

```rust
let state_manager = StateManager::new(&redis_url, StateConfig {
    enable_hot_reload: true,
    config_watch_paths: vec!["./config".to_string()],
    ..Default::default()
}).await?;
```

When configuration files in watched directories change, they are automatically reloaded without service interruption.

## Usage Examples

### Cache Operations

#### Basic Get/Set

```rust
use riptide_persistence::{PersistentCacheManager, config::CacheConfig};

let cache_manager = PersistentCacheManager::new(
    "redis://localhost:6379",
    CacheConfig::default()
).await?;

// Set with TTL
cache_manager.set(
    "article:123",
    &article_data,
    None,
    Some(std::time::Duration::from_secs(3600)),
    None
).await?;

// Get with type inference
let article: Option<ArticleData> = cache_manager
    .get("article:123", None)
    .await?;
```

#### Tenant-Isolated Caching

```rust
// Set data with tenant namespace
cache_manager.set(
    "extraction_result",
    &data,
    Some(&tenant_id),  // Tenant namespace
    Some(std::time::Duration::from_secs(3600)),
    Some(CacheMetadata {
        version: "1.0.0".to_string(),
        content_type: Some("application/json".to_string()),
        source: Some("riptide_extractor".to_string()),
        tags: vec!["extraction".to_string()],
        attributes: HashMap::new(),
    })
).await?;

// Get with tenant isolation
let result: Option<ExtractionResult> = cache_manager
    .get("extraction_result", Some(&tenant_id))
    .await?;
```

#### Batch Operations

```rust
// Batch set
let mut batch = HashMap::new();
batch.insert("key1".to_string(), data1);
batch.insert("key2".to_string(), data2);
batch.insert("key3".to_string(), data3);

cache_manager.set_batch(
    batch,
    Some(&tenant_id),
    Some(std::time::Duration::from_secs(1800))
).await?;

// Batch get
let keys = vec!["key1".to_string(), "key2".to_string(), "key3".to_string()];
let results: HashMap<String, DataType> = cache_manager
    .get_batch(&keys, Some(&tenant_id))
    .await?;
```

### Multi-Tenancy

#### Creating Tenants

```rust
use riptide_persistence::{TenantManager, TenantOwner, BillingPlan};

let tenant_manager = TenantManager::new(
    &redis_url,
    tenant_config,
    tenant_metrics
).await?;

let owner = TenantOwner {
    id: "owner_123".to_string(),
    name: "Acme Corporation".to_string(),
    email: "admin@acme.com".to_string(),
    organization: Some("Acme Corp".to_string()),
};

let tenant_config = TenantConfig {
    tenant_id: "".to_string(), // Auto-generated
    name: "Acme Tenant".to_string(),
    quotas: {
        let mut quotas = HashMap::new();
        quotas.insert("memory_bytes".to_string(), 100 * 1024 * 1024); // 100MB
        quotas.insert("operations_per_minute".to_string(), 1000);
        quotas.insert("storage_bytes".to_string(), 1024 * 1024 * 1024); // 1GB
        quotas
    },
    isolation_level: TenantIsolationLevel::Strong,
    encryption_enabled: true,
    settings: HashMap::new(),
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
};

let tenant_id = tenant_manager
    .create_tenant(tenant_config, owner, BillingPlan::Professional)
    .await?;
```

#### Access Validation and Quota Management

```rust
// Validate tenant access to resource
let has_access = tenant_manager
    .validate_access(&tenant_id, "cache:read", "read")
    .await?;

if !has_access {
    return Err(Error::Unauthorized);
}

// Check quota before operation
tenant_manager
    .check_quota(&tenant_id, "memory_bytes", requested_bytes)
    .await?;

// Record resource usage for billing
let usage = ResourceUsageRecord {
    operation_count: 1,
    data_bytes: 1024,
    compute_time_ms: 50,
    storage_bytes: 2048,
    timestamp: chrono::Utc::now(),
};

tenant_manager
    .record_usage(&tenant_id, "cache_operation", usage)
    .await?;
```

#### Monitoring Tenant Usage

```rust
// Get current usage statistics
let usage = tenant_manager.get_tenant_usage(&tenant_id).await?;
println!("Memory: {} bytes", usage.memory_bytes);
println!("Operations/min: {}", usage.operations_per_minute);

// Get billing information
let billing = tenant_manager.get_billing_info(&tenant_id).await?;
println!("Plan: {:?}", billing.plan);
println!("Operations: {}", billing.current_usage.operations);
println!("Data transfer: {} bytes", billing.current_usage.data_transfer_bytes);
```

### State Management

#### Session Management

```rust
use riptide_persistence::{StateManager, state::*};

let state_manager = StateManager::new(&redis_url, StateConfig::default()).await?;

// Create session
let metadata = SessionMetadata {
    client_ip: Some("192.168.1.100".to_string()),
    user_agent: Some("RipTide-Client/1.0".to_string()),
    source: Some("web_app".to_string()),
    attributes: HashMap::new(),
};

let session_id = state_manager
    .create_session(
        Some("user_123".to_string()),
        metadata,
        Some(1800) // 30 minutes
    )
    .await?;

// Update session data
state_manager
    .update_session_data(
        &session_id,
        "extraction_state",
        serde_json::json!({
            "url": "https://example.com/article",
            "status": "processing",
            "progress": 0.45
        })
    )
    .await?;

// Retrieve session
if let Some(session) = state_manager.get_session(&session_id).await? {
    println!("Session user: {:?}", session.user_id);
    println!("Session data: {:?}", session.data);
}

// Terminate session
state_manager.terminate_session(&session_id).await?;
```

#### Checkpoint and Restore

```rust
// Create checkpoint
let checkpoint_id = state_manager
    .create_checkpoint(
        CheckpointType::Manual,
        Some("Before major update".to_string())
    )
    .await?;

println!("Checkpoint created: {}", checkpoint_id);

// Restore from checkpoint (disaster recovery)
state_manager
    .restore_from_checkpoint(&checkpoint_id)
    .await?;

println!("System restored from checkpoint");
```

#### Graceful Shutdown

```rust
// Preserve state during shutdown
state_manager.shutdown_gracefully().await?;
```

### Cache Statistics and Monitoring

```rust
// Get comprehensive cache statistics
let stats = cache_manager.get_stats().await?;

println!("Total keys: {}", stats.total_keys);
println!("Memory usage: {} bytes", stats.memory_usage_bytes);
println!("Hit rate: {:.2}%", stats.hit_rate * 100.0);
println!("Miss rate: {:.2}%", stats.miss_rate * 100.0);
println!("Avg access time: {} μs", stats.avg_access_time_us);
println!("Operations/sec: {:.2}", stats.ops_per_second);
println!("Compression ratio: {:.2}", stats.avg_compression_ratio);
```

### Memory Spillover Management

```rust
// Get spillover metrics
let spillover_metrics = state_manager.get_spillover_metrics().await;

println!("Total spilled: {}", spillover_metrics.total_spilled);
println!("Total restored: {}", spillover_metrics.total_restored);
println!("Avg spill time: {:.2}ms", spillover_metrics.avg_spill_time_ms);
println!("Avg restore time: {:.2}ms", spillover_metrics.avg_restore_time_ms);
```

## Feature Flags

### Default Features

```toml
[dependencies]
riptide-persistence = { version = "0.1.0" }
```

Includes:
- `compression` - LZ4 and Zstd compression support
- `metrics` - Prometheus metrics integration

### Compression Only

```toml
[dependencies]
riptide-persistence = { version = "0.1.0", default-features = false, features = ["compression"] }
```

### Minimal Build

```toml
[dependencies]
riptide-persistence = { version = "0.1.0", default-features = false }
```

## Performance Benchmarks

Performance benchmarks validate the sub-5ms target and identify optimization opportunities:

```bash
cargo bench --features compression
```

### Benchmark Results

Typical performance on modern hardware (Redis 7.x, local instance):

| Operation | Size | Latency (avg) | Throughput |
|-----------|------|---------------|------------|
| Cache Set | 1KB | 1.2ms | 833 ops/sec |
| Cache Set | 10KB | 1.8ms | 555 ops/sec |
| Cache Set | 100KB | 3.5ms | 285 ops/sec |
| Cache Get (hit) | 1KB | 0.8ms | 1,250 ops/sec |
| Cache Get (miss) | - | 0.6ms | 1,666 ops/sec |
| Batch Set (10) | 1KB each | 2.5ms | 400 batches/sec |
| Batch Get (10) | 1KB each | 1.9ms | 526 batches/sec |
| Session Create | - | 1.5ms | 666 ops/sec |
| Session Update | - | 1.3ms | 769 ops/sec |
| Tenant Create | - | 2.8ms | 357 ops/sec |
| Quota Check | - | 0.3ms | 3,333 ops/sec |

### Compression Performance

| Algorithm | Data Size | Original | Compressed | Ratio | Time |
|-----------|-----------|----------|------------|-------|------|
| LZ4 | 10KB | 10,240 | 4,123 | 40.3% | 0.2ms |
| LZ4 | 100KB | 102,400 | 38,456 | 37.5% | 1.1ms |
| Zstd | 10KB | 10,240 | 3,567 | 34.8% | 0.4ms |
| Zstd | 100KB | 102,400 | 31,234 | 30.5% | 2.3ms |

### Concurrent Access Performance

| Concurrent Clients | Operations/sec | Avg Latency |
|-------------------|----------------|-------------|
| 1 | 1,250 | 0.8ms |
| 4 | 4,200 | 0.95ms |
| 8 | 7,800 | 1.02ms |
| 16 | 12,400 | 1.29ms |

## Testing

### Unit Tests

```bash
cargo test
```

### Integration Tests

Integration tests require a running Redis instance:

```bash
# Start Redis (Docker)
docker run -d -p 6379:6379 redis:7-alpine

# Run integration tests
cargo test --test '*_integration_tests'
```

### Performance Tests

```bash
cargo test --release --test performance_tests
```

### Coverage

```bash
cargo tarpaulin --out Html --output-dir coverage/
```

## Production Deployment

### Docker Deployment

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --features compression,metrics

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates
COPY --from=builder /app/target/release/your-app /usr/local/bin/
CMD ["your-app"]
```

### Configuration Best Practices

#### Production Cache Configuration

```rust
CacheConfig {
    default_ttl_seconds: 3600,           // 1 hour
    max_entry_size_bytes: 10 * 1024 * 1024, // 10MB
    enable_compression: true,
    compression_threshold_bytes: 1024,
    compression_algorithm: CompressionAlgorithm::Lz4,
    max_memory_bytes: Some(4 * 1024 * 1024 * 1024), // 4GB
    eviction_policy: EvictionPolicy::LRU,
}
```

#### Production State Configuration

```rust
StateConfig {
    session_timeout_seconds: 1800,       // 30 minutes
    enable_hot_reload: true,
    checkpoint_interval_seconds: 300,     // 5 minutes
    max_checkpoints: 10,
    checkpoint_compression: true,
    enable_graceful_shutdown: true,
    shutdown_timeout_seconds: 30,
}
```

#### Production Tenant Configuration

```rust
TenantConfig {
    enabled: true,
    default_quotas: {
        let mut quotas = HashMap::new();
        quotas.insert("memory_bytes".to_string(), 500 * 1024 * 1024); // 500MB
        quotas.insert("operations_per_minute".to_string(), 10000);
        quotas.insert("storage_bytes".to_string(), 10 * 1024 * 1024 * 1024); // 10GB
        quotas
    },
    isolation_level: TenantIsolationLevel::Strong,
    enable_billing: true,
    billing_interval_seconds: 60,
    max_tenants: 10000,
    enable_encryption: true,
}
```

### Monitoring and Observability

#### Prometheus Metrics

When the `metrics` feature is enabled, the persistence layer exposes Prometheus metrics:

```rust
use riptide_persistence::metrics::PersistenceMetrics;

let metrics = PersistenceMetrics::new();
// Metrics are automatically collected and exposed
```

Available metrics:
- `riptide_cache_hits_total` - Total cache hits
- `riptide_cache_misses_total` - Total cache misses
- `riptide_cache_access_duration_seconds` - Cache access latency histogram
- `riptide_tenant_operations_total` - Operations per tenant
- `riptide_tenant_data_transfer_bytes` - Data transfer per tenant
- `riptide_session_active_total` - Active sessions count
- `riptide_spillover_operations_total` - Spillover operations count

#### Logging

The crate uses `tracing` for structured logging:

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::new(
        std::env::var("RUST_LOG")
            .unwrap_or_else(|_| "riptide_persistence=info".into()),
    ))
    .with(tracing_subscriber::fmt::layer())
    .init();
```

## Integration with RipTide Components

### RipTide API Integration

```rust
// Cache extraction results
let extraction_result = extractor.extract(url).await?;
cache_manager.set(
    &cache_key,
    &extraction_result,
    Some(&tenant_id),
    Some(Duration::from_secs(3600)),
    Some(metadata)
).await?;
```

### RipTide Search Integration

```rust
// Cache search results
let search_results = search_engine.search(&query).await?;
cache_manager.set(
    &search_cache_key,
    &search_results,
    Some(&tenant_id),
    Some(Duration::from_secs(300)), // 5 minutes for search
    None
).await?;
```

### RipTide Streaming Integration

```rust
// Persist streaming session state
state_manager.update_session_data(
    &session_id,
    "streaming_state",
    serde_json::json!({
        "stream_id": stream.id,
        "position": stream.position,
        "timestamp": Utc::now()
    })
).await?;
```

## Performance Optimization Tips

1. **Connection Pooling**: Use sufficient pool size for concurrent workloads
2. **Batch Operations**: Prefer batch operations for bulk data
3. **Compression**: Enable compression for data >1KB
4. **TTL Management**: Set appropriate TTLs to reduce memory pressure
5. **Namespace Design**: Use logical namespaces for tenant isolation
6. **Monitoring**: Enable metrics to track performance trends
7. **Memory Spillover**: Configure appropriate thresholds for disk spillover

## Troubleshooting

### High Latency

- Check Redis instance health and network latency
- Increase connection pool size
- Enable pipelining for batch operations
- Review compression settings

### Memory Issues

- Reduce `max_memory_bytes` configuration
- Decrease default TTL values
- Enable more aggressive eviction policies
- Monitor spillover metrics

### Tenant Quota Violations

- Review tenant quotas and adjust as needed
- Monitor usage patterns with billing data
- Implement quota increase workflows
- Set up alerts for approaching limits

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE) for details.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## Support

- **Documentation**: https://docs.riptide.dev
- **Issues**: https://github.com/riptide/eventmesh/issues
- **Discussions**: https://github.com/riptide/eventmesh/discussions
