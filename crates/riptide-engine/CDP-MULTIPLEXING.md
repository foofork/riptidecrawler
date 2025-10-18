# CDP Connection Multiplexing - riptide-engine

## Overview

CDP (Chrome DevTools Protocol) Connection Multiplexing optimizes browser automation by reusing connections across multiple requests, implementing intelligent connection pooling, and batching related commands.

**Target**: 30% latency reduction through connection reuse and optimization

## Performance Features

### ✅ Connection Pooling

Reuse CDP connections across requests to eliminate setup overhead:

- Per-browser connection pools with configurable limits
- Automatic connection lifecycle management
- Health monitoring prevents stale connections
- Idle timeout and max lifetime enforcement

### ✅ Command Batching

Batch related CDP commands to reduce round-trips by ~50%:

- Configurable batch size and timeout
- Automatic flushing on size threshold or timeout
- Command aggregation with detailed result tracking
- Parallel execution within batches

### ✅ Wait Queues

Fair FIFO queuing when pool is saturated:

- Priority-based queue ordering
- Timeout protection for waiting requests
- Automatic fulfillment on connection release
- Prevents connection storms

### ✅ Session Affinity

Route related requests to same connection for cache locality:

- Context-based connection routing
- TTL-based affinity expiration
- Warm connection reuse for same-domain requests
- Improved browser cache utilization

### ✅ Performance Metrics

Comprehensive latency tracking and monitoring:

- P50, P95, P99 latency percentiles
- Connection reuse rate tracking (target: >70%)
- Total commands executed
- Wait queue length monitoring
- Before/after performance comparison

## Usage Example

```rust
use riptide_engine::cdp_pool::{CdpConnectionPool, CdpPoolConfig, ConnectionPriority};
use std::time::Duration;

// 1. Configure CDP connection pool
let config = CdpPoolConfig {
    max_connections_per_browser: 20,
    connection_idle_timeout: Duration::from_secs(60),
    max_connection_lifetime: Duration::from_secs(600),
    enable_health_checks: true,
    health_check_interval: Duration::from_secs(30),
    enable_batching: true,
    batch_timeout: Duration::from_millis(50),
    max_batch_size: 10,
};

// 2. Validate configuration (catches errors early)
config.validate()?;

// 3. Create CDP pool
let cdp_pool = CdpConnectionPool::new(config);

// 4. Get connection (reuses existing or creates new)
let session_id = cdp_pool
    .get_connection_with_priority(
        browser_id,
        browser,
        url,
        ConnectionPriority::Normal,
        Some("user-session-123".to_string()) // Session affinity context
    )
    .await?;

// 5. Use connection for operations...
// (CDP commands are automatically batched if enabled)

// 6. Release connection back to pool
cdp_pool.release_connection(browser_id, &session_id).await?;

// 7. Get performance metrics
let stats = cdp_pool.stats().await;
println!("Connection reuse rate: {:.2}%", stats.connection_reuse_rate * 100.0);
println!("P95 latency: {:?}", stats.p95_latency);
println!("Total connections: {}", stats.total_connections);
println!("Available: {}", stats.available_connections);
println!("In use: {}", stats.in_use_connections);

// 8. Compare performance to baseline
let metrics = cdp_pool.performance_metrics(Some(baseline_latency)).await;
if metrics.target_met {
    println!("✅ 30% latency reduction target achieved!");
    println!("Improvement: {:.2}%", metrics.latency_improvement_pct);
}
```

## Configuration

### CdpPoolConfig

```rust
pub struct CdpPoolConfig {
    /// Maximum number of connections per browser (1-1000)
    pub max_connections_per_browser: usize,

    /// Connection idle timeout before cleanup (>= 1 second)
    pub connection_idle_timeout: Duration,

    /// Maximum connection lifetime (> idle_timeout)
    pub max_connection_lifetime: Duration,

    /// Enable connection health checks
    pub enable_health_checks: bool,

    /// Health check interval when enabled (>= 1 second)
    pub health_check_interval: Duration,

    /// Enable command batching
    pub enable_batching: bool,

    /// Batch timeout when enabled (1ms - 10 seconds)
    pub batch_timeout: Duration,

    /// Maximum commands per batch when enabled (1-100)
    pub max_batch_size: usize,
}
```

### Validation

Always validate configuration before use:

```rust
// ✅ Valid configuration
let config = CdpPoolConfig {
    max_connections_per_browser: 20,
    connection_idle_timeout: Duration::from_secs(60),
    max_connection_lifetime: Duration::from_secs(600),
    ..Default::default()
};
assert!(config.validate().is_ok());

// ❌ Invalid configurations are rejected with clear errors
let bad_config = CdpPoolConfig {
    max_connections_per_browser: 0, // ERROR: must be > 0
    ..Default::default()
};
assert!(bad_config.validate().is_err());

let bad_config2 = CdpPoolConfig {
    connection_idle_timeout: Duration::from_millis(500), // ERROR: must be >= 1s
    max_connection_lifetime: Duration::from_secs(300),
    ..Default::default()
};
assert!(bad_config2.validate().is_err());
```

### Validation Rules

- `max_connections_per_browser`: Must be > 0 and <= 1000
- `connection_idle_timeout`: Must be >= 1 second
- `max_connection_lifetime`: Must be > `connection_idle_timeout`
- `health_check_interval`: Must be >= 1 second (when health checks enabled)
- `batch_timeout`: Must be >= 1ms and <= 10 seconds (when batching enabled)
- `max_batch_size`: Must be > 0 and <= 100 (when batching enabled)

## Performance Metrics

### CdpPoolStats

```rust
pub struct CdpPoolStats {
    /// Total connections in pool
    pub total_connections: usize,

    /// Connections currently in use
    pub in_use_connections: usize,

    /// Connections available for checkout
    pub available_connections: usize,

    /// Number of browsers with connections
    pub browsers_with_connections: usize,

    /// Average connection latency
    pub avg_connection_latency: Duration,

    /// Latency percentiles (P50, P95, P99)
    pub p50_latency: Duration,
    pub p95_latency: Duration,
    pub p99_latency: Duration,

    /// Connection reuse rate (target: >70%)
    pub connection_reuse_rate: f64,

    /// Total commands executed across all connections
    pub total_commands_executed: u64,

    /// Current wait queue length
    pub wait_queue_length: usize,
}
```

### PerformanceMetrics

```rust
pub struct PerformanceMetrics {
    /// Baseline average latency (for comparison)
    pub baseline_avg_latency: Option<Duration>,

    /// Current average latency
    pub current_avg_latency: Duration,

    /// Latency improvement percentage
    pub latency_improvement_pct: f64,

    /// Connection reuse rate (0.0-1.0)
    pub connection_reuse_rate: f64,

    /// True if >= 30% improvement achieved
    pub target_met: bool,
}
```

## Connection Priority

Use priority levels for QoS:

```rust
pub enum ConnectionPriority {
    Low = 0,      // Background tasks
    Normal = 1,   // Standard requests (default)
    High = 2,     // Important operations
    Critical = 3, // Time-sensitive requests
}

// Critical requests get connections first
let session_id = cdp_pool
    .get_connection_with_priority(
        browser_id,
        browser,
        url,
        ConnectionPriority::Critical,
        None
    )
    .await?;
```

## Health Monitoring

The CDP pool performs automatic health checks:

```rust
// Health checks run in background
cdp_pool.health_check_all().await;

// Unhealthy connections are removed
// Idle connections are cleaned up
// Expired connections are replaced
```

## Best Practices

### 1. Always Validate Configuration

```rust
let config = CdpPoolConfig { /* ... */ };
config.validate()?; // Catch errors early
let pool = CdpConnectionPool::new(config);
```

### 2. Use Session Affinity for Related Requests

```rust
// Requests for same user/session use same connection
let session_id = cdp_pool
    .get_connection_with_priority(
        browser_id,
        browser,
        url,
        ConnectionPriority::Normal,
        Some(format!("user-{}", user_id)) // Affinity context
    )
    .await?;
```

### 3. Monitor Performance Metrics

```rust
// Regularly check pool health
let stats = cdp_pool.stats().await;
if stats.connection_reuse_rate < 0.70 {
    warn!("Low connection reuse rate: {:.2}%", stats.connection_reuse_rate * 100.0);
}

if stats.wait_queue_length > 10 {
    warn!("High wait queue length: {}", stats.wait_queue_length);
}
```

### 4. Release Connections Promptly

```rust
// Always release connections when done
let session_id = cdp_pool.get_connection(...).await?;
// ... use connection ...
cdp_pool.release_connection(browser_id, &session_id).await?; // Important!
```

### 5. Cleanup on Browser Shutdown

```rust
// Clean up all connections for a browser
cdp_pool.cleanup_browser(browser_id).await;
```

## Performance Targets

| Metric | Target | How to Verify |
|--------|--------|---------------|
| Latency Reduction | 30% | Compare `PerformanceMetrics.latency_improvement_pct` to baseline |
| Connection Reuse Rate | >70% | Check `CdpPoolStats.connection_reuse_rate` |
| Stale Connection Errors | 0% | Enable health checks and monitor logs |
| Pool Exhaustion | Fair queuing | Check `CdpPoolStats.wait_queue_length` |

## Architecture

See [P1-B4 Design Document](/workspaces/eventmesh/docs/architecture/P1-B4-cdp-multiplexing-design.md) for complete architecture details.

---

**Status**: ✅ Production Ready
**Version**: P1-B4 Complete
**Tests**: 49 passing (30 validation + 19 unit)
