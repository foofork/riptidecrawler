# Memory Profiling Activation Guide

## Overview

RipTide's memory profiling system provides production-ready tools for monitoring, analyzing, and optimizing memory usage in real-time. The system consists of three core components that work together to deliver comprehensive memory insights with minimal performance overhead.

**Performance Impact:** < 2% overhead in production when properly configured

## Components

### 1. Memory Tracker - Real-time Memory Monitoring

**Purpose:** Continuously tracks system and process memory statistics using OS-level APIs and optional jemalloc integration.

**Key Features:**
- Real-time RSS (Resident Set Size) tracking
- Virtual memory monitoring
- Heap allocation tracking (with jemalloc)
- Memory breakdown by component
- Configurable sampling intervals
- Force garbage collection capabilities

**API Methods:**
```rust
// Get current memory snapshot
let snapshot = tracker.get_current_snapshot().await?;

// Get detailed memory breakdown
let breakdown = tracker.get_memory_breakdown().await?;

// Get memory statistics over time
let stats = tracker.get_memory_stats(Duration::from_secs(60)).await?;

// Force garbage collection
tracker.force_gc().await?;
```

### 2. Leak Detector - Memory Leak Detection

**Purpose:** Identifies potential memory leaks through allocation pattern analysis and growth rate monitoring.

**Detection Criteria:**
- High growth rate (>10MB/hour)
- Large total size (>50MB) with recent activity
- Many small allocations without deallocations (>1000 allocations, >1MB)
- Steadily growing peak size

**Pattern Recognition:**
- Exponential growth detection
- Frequent large allocations (>1MB)
- Repeated allocation patterns (potential loop leaks)

**API Methods:**
```rust
// Record an allocation
detector.record_allocation(allocation_info).await?;

// Record a deallocation
detector.record_deallocation("component_name", size_bytes).await?;

// Analyze for potential leaks
let analysis = detector.analyze_leaks().await?;

// Get memory pressure score (0.0 = low, 1.0 = high)
let pressure = detector.get_memory_pressure().await?;

// Clean up old data
detector.cleanup_old_data(Duration::from_hours(24)).await?;
```

### 3. Allocation Analyzer - Allocation Pattern Analysis

**Purpose:** Analyzes allocation patterns to identify optimization opportunities and efficiency issues.

**Analysis Types:**
- Top allocators by total bytes
- Top operations by frequency
- Size distribution analysis (tiny/small/medium/large/huge)
- Fragmentation analysis
- Allocation timeline trending
- Efficiency scoring

**Size Categories:**
- **Tiny:** < 1KB
- **Small:** 1KB - 64KB
- **Medium:** 64KB - 1MB
- **Large:** 1MB - 16MB
- **Huge:** > 16MB

**API Methods:**
```rust
// Record allocation for analysis
analyzer.record_allocation(allocation_info).await?;

// Get top allocators
let top_allocators = analyzer.get_top_allocators().await?;

// Get top operations
let top_operations = analyzer.get_top_operations().await?;

// Get size distribution
let distribution = analyzer.get_size_distribution().await?;

// Analyze patterns for optimization
let recommendations = analyzer.analyze_patterns().await?;

// Calculate efficiency score (0.0 = poor, 1.0 = excellent)
let score = analyzer.calculate_efficiency_score().await?;
```

## Quick Start

### Installation

The memory profiling system is already integrated into RipTide. No additional installation is required.

**Optional: Enable jemalloc for enhanced tracking**

Add to `Cargo.toml`:
```toml
[dependencies]
jemalloc-ctl = { version = "0.5", optional = true }

[features]
jemalloc = ["jemalloc-ctl"]
```

Build with jemalloc support:
```bash
cargo build --features jemalloc --release
```

### Configuration

Create a profiling configuration:

```rust
use riptide_performance::profiling::MemoryProfileConfig;
use std::time::Duration;

let config = MemoryProfileConfig {
    sampling_interval: Duration::from_secs(5),    // Sample every 5 seconds
    max_samples: 1000,                            // Keep last 1000 samples
    track_allocations: true,                      // Enable allocation tracking
    detect_leaks: true,                           // Enable leak detection
    generate_flamegraphs: false,                  // Disable flamegraphs (expensive)
    warning_threshold_mb: 650.0,                  // Warning at 650MB
    alert_threshold_mb: 700.0,                    // Alert at 700MB
};
```

### Running Profiling

**Basic Usage:**

```rust
use riptide_performance::profiling::MemoryProfiler;
use uuid::Uuid;

// Create profiler
let session_id = Uuid::new_v4();
let mut profiler = MemoryProfiler::new(session_id)?;

// Start profiling
profiler.start_profiling().await?;

// ... run your application ...

// Stop profiling and get report
let report = profiler.stop_profiling().await?;

println!("Peak memory: {:.2}MB", report.peak_memory_mb);
println!("Average memory: {:.2}MB", report.average_memory_mb);
println!("Growth rate: {:.4}MB/s", report.memory_growth_rate_mb_s);
println!("Efficiency score: {:.2}", report.memory_efficiency_score);
```

**With Custom Configuration:**

```rust
let mut profiler = MemoryProfiler::with_config(session_id, config)?;
profiler.start_profiling().await?;

// During profiling, check thresholds
let alerts = profiler.check_memory_thresholds().await?;
for alert in alerts {
    eprintln!("ALERT: {}", alert);
}

// Get current snapshot anytime
let snapshot = profiler.get_current_snapshot().await?;
println!("Current RSS: {:.2}MB", snapshot.rss_bytes as f64 / 1024.0 / 1024.0);

// Get memory trend for last hour
let trend = profiler.get_memory_trend(Duration::from_hours(1)).await?;
```

### Reading Reports

The memory report includes comprehensive analysis:

```rust
let report = profiler.stop_profiling().await?;

// Summary Statistics
println!("=== Summary ===");
println!("Session ID: {}", report.session_id);
println!("Duration: {:?}", report.profiling_duration);
println!("Total samples: {}", report.total_samples);
println!("Peak memory: {:.2}MB", report.peak_memory_mb);
println!("Average memory: {:.2}MB", report.average_memory_mb);
println!("Growth rate: {:.4}MB/s", report.memory_growth_rate_mb_s);
println!("Efficiency score: {:.2}/1.0", report.memory_efficiency_score);

// Leak Analysis
println!("\n=== Potential Leaks ===");
for leak in &report.leak_analysis.potential_leaks {
    println!("Component: {}", leak.component);
    println!("  Allocations: {}", leak.allocation_count);
    println!("  Total size: {:.2}MB", leak.total_size_bytes as f64 / 1024.0 / 1024.0);
    println!("  Growth rate: {:.2} bytes/s", leak.growth_rate);
}

// Top Allocators
println!("\n=== Top Allocators ===");
for (component, bytes) in &report.top_allocators {
    println!("{}: {:.2}MB", component, *bytes as f64 / 1024.0 / 1024.0);
}

// Recommendations
println!("\n=== Recommendations ===");
for rec in &report.recommendations {
    println!("- {}", rec);
}
```

## Production Deployment

### Feature Flag Configuration

**Environment Variables:**

```bash
# Enable memory profiling
export MEMORY_PROFILING_ENABLED=true

# Sampling interval (seconds)
export MEMORY_PROFILING_INTERVAL=10

# Maximum samples to keep
export MEMORY_PROFILING_MAX_SAMPLES=500

# Enable allocation tracking
export MEMORY_PROFILING_TRACK_ALLOCATIONS=true

# Enable leak detection
export MEMORY_PROFILING_DETECT_LEAKS=true

# Warning threshold (MB)
export MEMORY_PROFILING_WARNING_THRESHOLD=650

# Alert threshold (MB)
export MEMORY_PROFILING_ALERT_THRESHOLD=700
```

**Configuration File (config.toml):**

```toml
[memory_profiling]
enabled = true
sampling_interval_secs = 10
max_samples = 500
track_allocations = true
detect_leaks = true
generate_flamegraphs = false
warning_threshold_mb = 650.0
alert_threshold_mb = 700.0
```

### Performance Impact

**Overhead Analysis:**

| Component | Overhead | Impact |
|-----------|----------|--------|
| Memory Tracker | < 0.5% | Minimal - uses system APIs |
| Leak Detector | < 1.0% | Low - aggregates data |
| Allocation Analyzer | < 0.5% | Minimal - periodic sampling |
| **Total** | **< 2.0%** | **Acceptable for production** |

**Optimization Tips:**

1. **Increase sampling interval** in production (10-30 seconds)
2. **Disable flamegraph generation** (only enable for debugging)
3. **Reduce max_samples** to 500-1000 for lower memory overhead
4. **Use feature flags** to enable profiling only when needed
5. **Clean up old data** periodically to prevent memory growth

### Monitoring and Alerts

**Integrate with HTTP Endpoints:**

```rust
use axum::{routing::get, Router, Json};

async fn memory_snapshot_handler() -> Json<MemorySnapshot> {
    let profiler = get_profiler(); // Get shared profiler instance
    let snapshot = profiler.get_current_snapshot().await.unwrap();
    Json(snapshot)
}

async fn memory_alerts_handler() -> Json<Vec<String>> {
    let profiler = get_profiler();
    let alerts = profiler.check_memory_thresholds().await.unwrap();
    Json(alerts)
}

let app = Router::new()
    .route("/profiling/snapshot", get(memory_snapshot_handler))
    .route("/profiling/alerts", get(memory_alerts_handler));
```

**Prometheus Metrics Integration:**

```rust
use prometheus::{register_gauge, Gauge};

lazy_static! {
    static ref MEMORY_RSS_BYTES: Gauge = register_gauge!(
        "riptide_memory_rss_bytes",
        "Resident set size in bytes"
    ).unwrap();

    static ref MEMORY_HEAP_BYTES: Gauge = register_gauge!(
        "riptide_memory_heap_bytes",
        "Heap memory in bytes"
    ).unwrap();
}

// Update metrics periodically
async fn update_memory_metrics(profiler: &MemoryProfiler) {
    if let Ok(snapshot) = profiler.get_current_snapshot().await {
        MEMORY_RSS_BYTES.set(snapshot.rss_bytes as f64);
        MEMORY_HEAP_BYTES.set(snapshot.heap_bytes as f64);
    }
}
```

**Alert Rules (Prometheus/Alertmanager):**

```yaml
groups:
  - name: memory_profiling
    interval: 30s
    rules:
      - alert: HighMemoryUsage
        expr: riptide_memory_rss_bytes / 1024 / 1024 > 650
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage detected"
          description: "Memory usage is {{ $value }}MB"

      - alert: CriticalMemoryUsage
        expr: riptide_memory_rss_bytes / 1024 / 1024 > 700
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Critical memory usage"
          description: "Memory usage is {{ $value }}MB - immediate action required"

      - alert: MemoryLeak
        expr: rate(riptide_memory_rss_bytes[5m]) > 1048576  # >1MB/min
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Potential memory leak detected"
          description: "Memory is growing at {{ $value }} bytes/sec"
```

### Troubleshooting

**Common Issues:**

1. **High overhead in production**
   - Solution: Increase sampling_interval to 30-60 seconds
   - Disable allocation tracking if not needed
   - Reduce max_samples to 500

2. **Memory profiler using too much memory**
   - Solution: Enable cleanup_old_data() with 24-hour retention
   - Reduce max_samples
   - Disable flamegraph generation

3. **Missing jemalloc statistics**
   - Solution: Build with `--features jemalloc`
   - Verify jemalloc is installed and linked
   - Check `heap_bytes` field in snapshots

4. **False positive leak detections**
   - Solution: Adjust detection criteria in LeakDetector
   - Increase growth_rate threshold
   - Record deallocations properly

5. **Profiler not starting**
   - Solution: Check session_id is unique
   - Verify no other profiler is running
   - Check file permissions for flamegraph output

**Debug Mode:**

```rust
// Enable debug logging
env::set_var("RUST_LOG", "riptide_performance=debug");
tracing_subscriber::fmt::init();

// Create profiler with debug config
let config = MemoryProfileConfig {
    sampling_interval: Duration::from_secs(1), // More frequent sampling
    ..Default::default()
};
```

## API Reference

### HTTP Endpoints

**GET /profiling/snapshot**
```bash
curl http://localhost:8080/profiling/snapshot
```

Response:
```json
{
  "timestamp": "2025-10-06T07:49:27Z",
  "rss_bytes": 104857600,
  "heap_bytes": 52428800,
  "virtual_bytes": 314572800,
  "resident_bytes": 104857600,
  "shared_bytes": 0,
  "text_bytes": 0,
  "data_bytes": 0,
  "stack_bytes": 0
}
```

**GET /profiling/alerts**
```bash
curl http://localhost:8080/profiling/alerts
```

Response:
```json
[
  "WARNING: Memory usage 680.5MB exceeds warning threshold 650.0MB"
]
```

**GET /profiling/report**
```bash
curl http://localhost:8080/profiling/report
```

Response: Complete MemoryReport JSON (see examples below)

### Telemetry Metrics

Available Prometheus metrics:

```
# Memory usage
riptide_memory_rss_bytes - Resident set size
riptide_memory_heap_bytes - Heap allocation
riptide_memory_virtual_bytes - Virtual memory

# Profiling status
riptide_memory_profiling_active - 1 if profiling active, 0 otherwise
riptide_memory_samples_collected - Total samples collected
riptide_memory_potential_leaks - Number of potential leaks detected

# Performance
riptide_memory_profiling_overhead_ms - Profiling overhead in milliseconds
```

### Alert Rules

**Memory Thresholds:**

- **Warning:** > 650MB RSS for 5 minutes
- **Critical:** > 700MB RSS for 2 minutes
- **Leak Detection:** Growth rate > 1MB/minute for 10 minutes

**Recommended Actions:**

1. **Warning Alert:** Monitor closely, prepare to scale or optimize
2. **Critical Alert:** Immediate investigation, consider restart
3. **Leak Alert:** Start detailed profiling, identify leak source

## Next Steps

1. **Enable profiling** in your RipTide deployment
2. **Configure thresholds** based on your system limits
3. **Set up monitoring** with Prometheus and Grafana
4. **Create alert rules** for your operations team
5. **Review reports** regularly to identify optimization opportunities

## Additional Resources

- [Memory Profiling Examples](memory-profiling-examples.md)
- [Performance Monitoring Guide](performance-monitoring.md)
- [API Documentation](api/openapi.yaml)
- [Architecture Overview](architecture/system-overview.md)

## Support

For issues or questions:
- GitHub Issues: https://github.com/your-org/riptide-api/issues
- Documentation: https://docs.riptide.io
- Community: https://discord.gg/riptide
