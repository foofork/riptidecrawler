# RipTide Performance

Production-ready memory profiling and performance monitoring system with jemalloc integration for the RipTide web scraping framework.

## Overview

The `riptide-performance` crate provides comprehensive performance monitoring, profiling, and optimization capabilities designed for production environments. It offers real-time memory tracking, leak detection, allocation analysis, and performance bottleneck identification with minimal overhead (< 2%).

### Key Features

- **Real-time Memory Monitoring**: Track memory usage patterns with sub-millisecond overhead
- **Memory Leak Detection**: Identify and analyze potential memory leaks with detailed reports
- **Allocation Pattern Analysis**: Profile allocation hotspots and optimization opportunities
- **jemalloc Integration**: Enhanced memory profiling with jemalloc allocator statistics
- **pprof Profiling**: Generate protobuf-formatted profiling data (flamegraph only in development)
- **Bottleneck Analysis**: Identify performance bottlenecks using criterion benchmarks
- **Cache Optimization**: Multi-layer caching with moka and Redis support
- **Resource Limits**: Rate limiting and circuit breakers with governor
- **HTTP Endpoints**: RESTful API for monitoring and metrics collection
- **Minimal Overhead**: < 2% performance impact in production

## Components

### Memory Tracker

Collects system and process memory statistics using jemalloc and system APIs:

- RSS (Resident Set Size) tracking
- Heap allocation monitoring
- Virtual memory usage
- jemalloc-specific statistics (allocated, active, metadata, resident, mapped, retained)
- Memory snapshots with configurable intervals

### Leak Detector

Identifies potential memory leaks through pattern analysis:

- Tracks allocation lifetimes
- Detects long-lived allocations
- Identifies allocation hotspots
- Generates detailed leak reports with location and size information
- Configurable thresholds for leak detection

### Allocation Analyzer

Analyzes memory allocation patterns:

- Top allocators by count and size
- Allocation rate tracking
- Size distribution analysis (average, median)
- Allocation/deallocation ratio monitoring
- Pattern recognition for optimization opportunities

## Quick Start

### Basic Usage

```rust
use riptide_performance::{PerformanceManager, PerformanceTargets};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create performance manager with default targets
    let manager = PerformanceManager::new()?;

    // Start monitoring
    manager.start_monitoring().await?;

    // Your application code here
    // ...

    // Get current metrics
    let metrics = manager.get_metrics().await?;
    println!("Memory usage: {:.2} MB", metrics.memory_rss_mb);
    println!("Throughput: {:.1} pages/sec", metrics.throughput_pps);

    // Stop monitoring and generate report
    let report = manager.stop_monitoring().await?;
    println!("Performance report: {:#?}", report);

    Ok(())
}
```

### Custom Performance Targets

```rust
use riptide_performance::{PerformanceManager, PerformanceTargets};

let targets = PerformanceTargets {
    p50_latency_ms: 1000,          // 1s
    p95_latency_ms: 3000,          // 3s
    max_memory_mb: 400,            // 400MB
    memory_alert_mb: 450,          // Alert at 450MB
    min_throughput_pps: 100.0,     // 100 pages/sec
    max_ai_overhead_percent: 20.0, // 20% max AI impact
};

let manager = PerformanceManager::with_targets(targets)?;
```

### Checking Performance Targets

```rust
// Check if performance targets are being met
let status = manager.check_targets().await?;

if !status.all_targets_met {
    println!("Performance violations:");
    for violation in status.violations {
        println!("  - {}", violation);
    }
}

if !status.warnings.is_empty() {
    println!("Warnings:");
    for warning in status.warnings {
        println!("  - {}", warning);
    }
}
```

## Memory Profiling API

### Memory Snapshots

```rust
use riptide_performance::profiling::MemoryProfiler;

let profiler = MemoryProfiler::new(session_id)?;
profiler.start_profiling().await?;

// Get current snapshot
let snapshot = profiler.get_current_snapshot().await?;
println!("RSS: {} bytes", snapshot.rss_bytes);
println!("Heap: {} bytes", snapshot.heap_bytes);
```

### Leak Detection

```rust
// Detect memory leaks
let leak_report = profiler.detect_leaks().await?;

println!("Found {} potential leaks", leak_report.leak_count);
for leak in leak_report.leaks {
    println!(
        "Leak at {}: {} bytes, age: {:?}",
        leak.location,
        leak.size,
        leak.age
    );
}
```

### Allocation Analysis

```rust
// Analyze allocation patterns
let stats = profiler.get_allocation_stats().await?;

println!("Top allocators:");
for allocator in stats.top_allocators.iter().take(10) {
    println!(
        "  {}: {} allocations, {} bytes",
        allocator.location,
        allocator.count,
        allocator.total_bytes
    );
}
```

## HTTP Endpoints

The crate provides RESTful HTTP endpoints for real-time monitoring:

### Available Endpoints

#### `GET /metrics/memory/snapshot`
Get current memory snapshot with allocation statistics.

**Response:**
```json
{
  "timestamp": "2025-10-11T10:30:00Z",
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "total_allocated": 104857600,
  "total_deallocated": 52428800,
  "current_usage": 52428800,
  "peak_usage": 67108864,
  "allocation_count": 1024,
  "deallocation_count": 512
}
```

#### `GET /metrics/memory/leaks`
Analyze potential memory leaks.

**Response:**
```json
{
  "timestamp": "2025-10-11T10:30:00Z",
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "leak_count": 3,
  "total_leaked_bytes": 1048576,
  "leaks": [
    {
      "location": "module::function::line_42",
      "size": 524288,
      "age_seconds": 3600,
      "allocation_time": "2025-10-11T09:30:00Z"
    }
  ]
}
```

#### `GET /metrics/memory/allocations`
Get top allocators and allocation statistics.

**Response:**
```json
{
  "timestamp": "2025-10-11T10:30:00Z",
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "top_allocators": [
    {
      "location": "buffer::allocate",
      "allocation_count": 256,
      "total_bytes": 16777216,
      "average_size": 65536,
      "percentage_of_total": 32.5
    }
  ],
  "total_allocations": 1024,
  "average_allocation_size": 51200,
  "median_allocation_size": 32768,
  "allocation_rate_per_second": 42.5
}
```

#### `GET /metrics/memory/trend?duration=1h&interval=1m`
Get memory usage trends over time.

**Query Parameters:**
- `duration`: Time range (e.g., "1h", "30m", "120s") - default: "1h"
- `interval`: Sampling interval (e.g., "1m", "30s") - default: "1m"

**Response:**
```json
{
  "timestamp": "2025-10-11T10:30:00Z",
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "duration": "1h",
  "interval": "1m",
  "data_points": [
    {
      "timestamp": "2025-10-11T09:30:00Z",
      "memory_usage": 52428800,
      "allocation_count": 1024,
      "deallocation_count": 512
    }
  ],
  "trend_analysis": {
    "average_usage": 54525952,
    "peak_usage": 67108864,
    "min_usage": 52428800,
    "growth_rate": 145.6,
    "volatility": 2457600.0
  }
}
```

#### `GET /metrics/memory/health`
Health check with performance thresholds.

**Response:**
```json
{
  "timestamp": "2025-10-11T10:30:00Z",
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "healthy",
  "memory_usage_percent": 48.8,
  "leak_severity": "low",
  "allocation_rate_status": "normal",
  "details": {
    "current_memory": 524288000,
    "memory_limit": 1073741824,
    "leak_count": 2,
    "leaked_bytes": 65536,
    "allocation_rate": 42.5,
    "warnings": [],
    "recommendations": []
  }
}
```

#### `POST /metrics/memory/gc`
Force garbage collection and cleanup.

**Response:**
```json
{
  "timestamp": "2025-10-11T10:30:00Z",
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "triggered": true,
  "memory_before": 524288000,
  "memory_after": 471859200,
  "freed_bytes": 52428800,
  "duration_ms": 125
}
```

### Starting HTTP Server

```rust
use riptide_performance::monitoring::MemoryMetricsRouter;
use std::sync::Arc;
use tokio::sync::RwLock;

let profiler = Arc::new(RwLock::new(MemoryProfiler::new(session_id)?));
let state = MemoryMetricsRouter::new(profiler, session_id.to_string());
let router = MemoryMetricsRouter::routes(state);

// Start server
axum::Server::bind(&"0.0.0.0:3000".parse()?)
    .serve(router.into_make_service())
    .await?;
```

## Feature Flags

### Core Features

- **`memory-profiling`** - Enable memory profiling with jemalloc integration
  - Includes: `tikv-jemalloc-ctl`, `pprof`, `memory-stats`
  - Default: **enabled**

- **`bottleneck-analysis`** - Enable performance bottleneck detection with criterion
  - Includes: `criterion`
  - Excludes: `flamegraph` (CI compliance)
  - Default: **enabled**

- **`bottleneck-analysis-full`** - Full bottleneck analysis with flamegraph support
  - Includes: `bottleneck-analysis`, `flamegraph`
  - **Development only** - DO NOT use in CI/production
  - Default: **disabled**

- **`cache-optimization`** - Enable cache optimization features
  - Includes: `moka`, `redis`
  - Default: **enabled**

- **`resource-limits`** - Enable resource limiting and rate limiting
  - Includes: `governor`
  - Default: **enabled**

### Allocator Features

- **`jemalloc`** - Enable jemalloc control interface
  - Includes: `tikv-jemalloc-ctl`
  - Default: **disabled** (use when jemalloc is the global allocator)

### Environment Profiles

- **`production`** - Production-ready feature set
  - Includes: `jemalloc`, `memory-profiling`, `bottleneck-analysis`, `cache-optimization`, `resource-limits`
  - Excludes: `flamegraph` for license compliance
  - Use in CI/CD and production deployments

- **`development`** - Development feature set with full tooling
  - Includes: `jemalloc`, `memory-profiling`, `bottleneck-analysis-full`, `cache-optimization`
  - Includes: `flamegraph` for local visualization
  - Use only in local development

### Feature Flag Usage

```toml
# Production build (recommended for CI/production)
[dependencies]
riptide-performance = { version = "0.1", features = ["production"] }

# Development build (local only)
[dependencies]
riptide-performance = { version = "0.1", features = ["development"] }

# Custom feature selection
[dependencies]
riptide-performance = {
    version = "0.1",
    features = ["memory-profiling", "cache-optimization"]
}
```

## Configuration

### Default Performance Targets

```rust
PerformanceTargets {
    p50_latency_ms: 1500,          // P50 latency: 1.5s
    p95_latency_ms: 5000,          // P95 latency: 5s
    max_memory_mb: 600,            // Max memory: 600MB
    memory_alert_mb: 650,          // Alert threshold: 650MB
    min_throughput_pps: 70.0,      // Min throughput: 70 pages/sec
    max_ai_overhead_percent: 30.0, // Max AI overhead: 30%
}
```

### Environment Variables

```bash
# Memory limits
RIPTIDE_PERF_MAX_MEMORY_MB=600
RIPTIDE_PERF_MEMORY_ALERT_MB=650

# Performance targets
RIPTIDE_PERF_P50_LATENCY_MS=1500
RIPTIDE_PERF_P95_LATENCY_MS=5000
RIPTIDE_PERF_MIN_THROUGHPUT_PPS=70

# Monitoring configuration
RIPTIDE_PERF_METRICS_PORT=3000
RIPTIDE_PERF_METRICS_INTERVAL_SEC=60
```

### TOML Configuration

Create `performance.toml`:

```toml
[targets]
p50_latency_ms = 1500
p95_latency_ms = 5000
max_memory_mb = 600
memory_alert_mb = 650
min_throughput_pps = 70.0
max_ai_overhead_percent = 30.0

[monitoring]
enabled = true
metrics_port = 3000
sample_interval_sec = 60
alert_on_violations = true

[profiling]
enabled = true
leak_detection_interval_sec = 300
allocation_tracking = true
jemalloc_stats = true

[cache]
enabled = true
max_size_mb = 100
ttl_sec = 3600

[limits]
max_concurrent_requests = 100
rate_limit_per_second = 50
circuit_breaker_threshold = 0.5
```

## Performance Overhead

The `riptide-performance` crate is designed to have minimal impact on application performance:

| Feature | Overhead | Notes |
|---------|----------|-------|
| Memory tracking | < 0.5% | Sampling-based, configurable interval |
| Allocation tracking | < 1% | Uses efficient data structures |
| Leak detection | < 0.3% | Periodic analysis, not per-allocation |
| HTTP endpoints | < 0.1% | On-demand metrics, no continuous overhead |
| Cache optimization | < 0.2% | Amortized cost, improves overall performance |
| **Total (all features)** | **< 2%** | Combined overhead in production |

### Overhead Reduction Tips

1. **Increase sampling intervals** for less frequent profiling
2. **Disable allocation tracking** if not needed
3. **Use production feature set** to exclude development-only tools
4. **Configure leak detection interval** based on application needs
5. **Limit HTTP endpoint usage** to monitoring systems only

## Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test memory_tracker
cargo test leak_detector
cargo test allocation_analyzer

# Run with specific features
cargo test --features memory-profiling
cargo test --features bottleneck-analysis
```

### Integration Tests

```bash
# Run integration tests
cargo test --test '*'

# Run with all features
cargo test --all-features
```

### Benchmarks

```bash
# Run benchmarks (requires criterion feature)
cargo bench --features bottleneck-analysis

# Run specific benchmark
cargo bench memory_benchmark
cargo bench bottleneck_benchmark
cargo bench cache_benchmark
```

## CI/CD Considerations

### Important: Flamegraph in CI

**⚠️ CRITICAL**: The `flamegraph` feature is **excluded from CI builds** due to its CDDL-1.0 licensed dependency (`inferno`).

**Correct CI Configuration:**

```yaml
# GitHub Actions example
- name: Build with production features
  run: cargo build --release --features production

- name: Test with production features
  run: cargo test --features production

# DO NOT use 'development' or 'bottleneck-analysis-full' features in CI
```

**Local Development (with flamegraph):**

```bash
# Local builds can use full development features
cargo build --features development
cargo test --features bottleneck-analysis-full
```

### Docker Production Build

```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .

# Build with production features only
RUN cargo build --release --features production

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/app /usr/local/bin/
CMD ["app"]
```

### CI Test Matrix

```yaml
strategy:
  matrix:
    features:
      - "production"
      - "memory-profiling"
      - "bottleneck-analysis"
      - "cache-optimization"
      # DO NOT include "development" or "bottleneck-analysis-full"
```

## Performance Targets

The crate is designed to meet these performance targets:

- **Latency**: P50 ≤ 1.5s, P95 ≤ 5s
- **Memory**: RSS ≤ 600MB (alert at 650MB)
- **Throughput**: ≥ 70 pages/sec with AI processing
- **AI Impact**: ≤ 30% throughput reduction
- **Monitoring Overhead**: < 2% total

## Examples

### Complete Monitoring Setup

```rust
use riptide_performance::{
    PerformanceManager,
    PerformanceTargets,
    monitoring::MemoryMetricsRouter,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize performance manager
    let manager = Arc::new(PerformanceManager::new()?);

    // Start monitoring
    manager.start_monitoring().await?;

    // Start HTTP metrics server
    let profiler = Arc::new(RwLock::new(
        riptide_performance::profiling::MemoryProfiler::new(uuid::Uuid::new_v4())?
    ));
    let state = MemoryMetricsRouter::new(
        profiler,
        uuid::Uuid::new_v4().to_string()
    );
    let router = MemoryMetricsRouter::routes(state);

    tokio::spawn(async move {
        axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
            .serve(router.into_make_service())
            .await
            .unwrap();
    });

    // Your application code
    loop {
        // Check targets every minute
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

        let status = manager.check_targets().await?;
        if !status.all_targets_met {
            eprintln!("Performance targets not met!");
            for violation in status.violations {
                eprintln!("  - {}", violation);
            }
        }

        // Optimize cache
        let optimizations = manager.optimize_cache().await?;
        if !optimizations.is_empty() {
            println!("Cache optimizations: {:?}", optimizations);
        }
    }
}
```

### Rate Limiting and Circuit Breakers

```rust
// Acquire request permit (rate limiting + concurrency control)
let permit = manager.acquire_request_permit().await?;

// Check client-specific rate limits
manager.check_rate_limits(Some("client-123")).await?;

// Make request with circuit breaker tracking
match make_request().await {
    Ok(response) => {
        manager.record_success("external-api").await?;
        Ok(response)
    }
    Err(err) => {
        manager.record_failure("external-api").await?;
        Err(err)
    }
}

// Permit automatically released when dropped
drop(permit);
```

### Resource Usage Monitoring

```rust
// Get current resource usage
let usage = manager.get_resource_usage().await?;

println!("Concurrent requests: {}/{}",
    usage.active_requests,
    usage.max_concurrent_requests
);

println!("Rate limit: {}/{} req/sec",
    usage.current_rate,
    usage.rate_limit
);

if !usage.violations.is_empty() {
    eprintln!("Resource violations:");
    for violation in usage.violations {
        eprintln!("  - {}", violation);
    }
}
```

## License

This crate is part of the RipTide project and follows the same license.

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass: `cargo test --all-features`
2. Code is formatted: `cargo fmt`
3. No clippy warnings: `cargo clippy -- -D warnings`
4. Performance overhead remains < 2%
5. CI builds use only `production` features

## Support

For issues, questions, or contributions, please visit the main RipTide repository.
