# Memory Telemetry Integration - Quick Reference

## Module Location
`/workspaces/eventmesh/crates/riptide-performance/src/profiling/telemetry.rs`

## Quick Enable

```rust
use riptide_performance::profiling::{
    MemoryProfiler, MemoryProfileConfig, TelemetryConfig
};
use uuid::Uuid;

// Create config with telemetry
let config = MemoryProfileConfig {
    export_telemetry: true,
    telemetry_config: Some(TelemetryConfig {
        endpoint: "http://localhost:4317".to_string(),
        service_name: "my-service".to_string(),
        service_version: "1.0.0".to_string(),
        export_interval_seconds: 10,
        enabled: true,
    }),
    ..Default::default()
};

// Create and start profiler
let session_id = Uuid::new_v4();
let mut profiler = MemoryProfiler::with_config(session_id, config)?;
profiler.start_profiling().await?;

// ... run workload ...

// Stop and get report (telemetry auto-exported)
let report = profiler.stop_profiling().await?;
```

## Exported Metrics

### Memory Snapshots (Automatic)
| Metric | Type | Description | Labels |
|--------|------|-------------|--------|
| `memory.rss_bytes` | Gauge | Resident Set Size | `component`, `session_id` |
| `memory.heap_bytes` | Gauge | Heap memory usage | `component`, `session_id` |
| `memory.virtual_bytes` | Gauge | Virtual memory | `component`, `session_id` |

### Leak Detection
| Metric | Type | Description | Labels |
|--------|------|-------------|--------|
| `memory.leak.count` | Counter | Potential leaks detected | `status=detected` |
| `memory.leak.growth_rate_mb_h` | Gauge | Growth rate (MB/hour) | `metric=growth_rate` |
| `memory.leak.size_bytes` | Histogram | Leak size distribution | `component`, `severity` |
| `memory.leak.allocation_count` | Counter | Allocations in leak | `component`, `severity` |

### Allocation Statistics
| Metric | Type | Description | Labels |
|--------|------|-------------|--------|
| `memory.allocation.count` | Counter | Total allocations | `component`, `operation` |
| `memory.allocation.total_bytes` | Counter | Total bytes allocated | `component` |
| `memory.allocation.size_bytes` | Histogram | Allocation sizes | `component`, `operation` |
| `memory.allocation.efficiency_score` | Gauge | Efficiency (0.0-1.0) | `component` |
| `memory.allocator.total_bytes` | Gauge | Per-component totals | `component` |
| `memory.allocation.distribution` | Counter | Size category counts | `size_category` |

## Severity Levels

### Leak Severity
- **Critical:** >100MB OR >50MB/hour growth
- **High:** >50MB OR >20MB/hour growth
- **Medium:** >10MB OR >5MB/hour growth
- **Low:** Below all thresholds

### Efficiency Score (0.0-1.0)
- **Excellent (0.9-1.0):** Medium-sized, well-aligned allocations
- **Good (0.7-0.9):** Decent allocation patterns
- **Poor (0.3-0.7):** Tiny allocations or large sizes
- **Bad (<0.3):** Very inefficient allocation patterns

## Prometheus Queries

### Memory Growth
```promql
# Memory growth rate (MB/min)
rate(memory_rss_bytes[5m]) * 60 / 1024 / 1024

# Heap vs RSS ratio
memory_heap_bytes / memory_rss_bytes
```

### Leak Detection
```promql
# Critical leaks
memory_leak_count{severity="critical"}

# Leak growth trending
deriv(memory_leak_growth_rate_mb_h[1h])
```

### Allocation Quality
```promql
# Average efficiency
avg(memory_allocation_efficiency_score)

# Allocation rate
rate(memory_allocation_count[1m])

# Size distribution
sum by (size_category) (memory_allocation_distribution)
```

## Grafana Visualization

### Panel 1: Memory Overview
```promql
memory_rss_bytes
memory_heap_bytes
memory_virtual_bytes
```
**Type:** Time series
**Unit:** bytes (IEC)

### Panel 2: Leak Alerts
```promql
memory_leak_count > 0
memory_leak_growth_rate_mb_h
```
**Type:** Stat + Graph
**Thresholds:** Green (0), Yellow (1), Red (3)

### Panel 3: Allocation Efficiency
```promql
memory_allocation_efficiency_score
histogram_quantile(0.95, rate(memory_allocation_size_bytes_bucket[5m]))
```
**Type:** Gauge + Heatmap

### Panel 4: Top Allocators
```promql
topk(10, memory_allocator_total_bytes)
```
**Type:** Bar gauge
**Sort:** Descending

## Configuration Options

### TelemetryConfig
```rust
pub struct TelemetryConfig {
    pub endpoint: String,               // OTLP endpoint
    pub service_name: String,           // Service identifier
    pub service_version: String,        // Version tag
    pub export_interval_seconds: u64,   // Export frequency
    pub enabled: bool,                  // Master switch
}
```

### Default Values
- **endpoint:** `http://localhost:4317`
- **service_name:** `riptide-performance`
- **service_version:** Crate version
- **export_interval_seconds:** `10`
- **enabled:** `false` (must opt-in)

## OTLP Collector Setup

### Docker Compose
```yaml
services:
  otel-collector:
    image: otel/opentelemetry-collector-contrib:latest
    ports:
      - "4317:4317"  # OTLP gRPC
      - "4318:4318"  # OTLP HTTP
    volumes:
      - ./otel-config.yaml:/etc/otel/config.yaml
    command: --config=/etc/otel/config.yaml

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_AUTH_ANONYMOUS_ENABLED=true
```

### OTLP Config (`otel-config.yaml`)
```yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317

exporters:
  prometheus:
    endpoint: 0.0.0.0:8889
  logging:
    loglevel: debug

service:
  pipelines:
    metrics:
      receivers: [otlp]
      exporters: [prometheus, logging]
```

## API Reference

### MemoryTelemetryExporter

```rust
impl MemoryTelemetryExporter {
    // Create new exporter
    pub fn new(config: TelemetryConfig) -> Result<Self>

    // Export memory snapshot
    pub async fn export_snapshot(
        &self,
        snapshot: &MemorySnapshot,
        component: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<()>

    // Export leak analysis
    pub async fn export_leak_analysis(
        &self,
        analysis: &LeakAnalysis
    ) -> Result<()>

    // Export allocation info
    pub async fn export_allocations(
        &self,
        allocations: &[AllocationInfo],
        component: Option<&str>,
    ) -> Result<()>

    // Export allocation statistics
    pub async fn export_allocation_stats(
        &self,
        top_allocators: &[(String, u64)],
        size_distribution: &HashMap<String, u64>,
    ) -> Result<()>

    // Force flush metrics
    pub async fn flush(&self) -> Result<()>

    // Graceful shutdown
    pub async fn shutdown(self) -> Result<()>
}
```

## Labels Reference

### Common Labels
- `component` - Component/module name
- `session_id` - Profiling session UUID
- `operation` - Allocation operation name
- `severity` - Leak severity (critical, high, medium, low)
- `size_category` - Allocation size category
- `metric` - Metric type identifier
- `status` - Status indicator

### Size Categories
- `tiny (<1KB)` - Small allocations
- `small (1KB-64KB)` - Small-medium allocations
- `medium (64KB-1MB)` - Medium allocations
- `large (1MB-16MB)` - Large allocations
- `huge (>16MB)` - Very large allocations

## Best Practices

1. **Enable in Production:** Use sampling to reduce overhead
2. **Set Service Name:** Use descriptive service identifiers
3. **Monitor Export Interval:** Balance freshness vs overhead
4. **Use Labels Wisely:** Component and session_id for correlation
5. **Set Alert Rules:** Critical leaks and efficiency drops
6. **Dashboard Creation:** Standardize across services
7. **Regular Review:** Check efficiency scores weekly

## Troubleshooting

### No Metrics Appearing
1. Verify `enabled: true` in config
2. Check OTLP endpoint connectivity
3. Review collector logs
4. Confirm export_interval is reasonable

### High Overhead
1. Increase export_interval_seconds
2. Reduce sampling frequency in MemoryProfileConfig
3. Disable detailed allocation tracking
4. Use sampling in production

### Missing Labels
1. Ensure component/session_id passed to export methods
2. Check label cardinality limits
3. Verify Prometheus scrape config

## Code Size
- **Lines:** 644
- **Public API Methods:** 9
- **Metric Types:** 13 (3 gauges, 4 counters, 2 histograms)
- **Tests:** Comprehensive unit test coverage
