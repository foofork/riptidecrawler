# RipTide CLI Metrics Architecture

## Overview

The RipTide CLI metrics module provides lightweight, thread-safe metrics collection with < 5ms overhead per command. It tracks command execution, performance, and resource usage with automatic persistence and rotation.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     RipTide CLI Metrics                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐    │
│  │   Collector  │───▶│   Storage    │───▶│  Aggregator  │    │
│  └──────┬───────┘    └──────┬───────┘    └──────┬───────┘    │
│         │                   │                   │              │
│         ├───────────────────┼───────────────────┤              │
│         │                   │                   │              │
│         ▼                   ▼                   ▼              │
│  ┌──────────────────────────────────────────────────────┐     │
│  │              MetricsManager (Global)                 │     │
│  └──────────────────────────────────────────────────────┘     │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
   [Counters]           [metrics.json]        [Statistics]
   [Timers]             [Archives]            [Percentiles]
   [Series]
```

## Module Structure

### 1. `metrics/types.rs` - Type Definitions

**Purpose**: Core data structures for metrics collection

**Key Types**:
- `CommandMetrics`: Tracks individual command execution
  - Command name, duration, success/failure
  - Items processed, bytes transferred
  - Cache hits, API calls
  - Peak memory usage
  - Error details and metadata

- `CommandAggregates`: Statistical summaries per command
  - Total/successful/failed executions
  - Average, P50, P95, P99 durations
  - Cache hit rate
  - Error distribution

- `CliMetricsSummary`: Overall CLI statistics
  - Total commands, success rate
  - Per-command aggregates
  - Total bytes/API calls
  - Session information

- `Counter`: Simple incrementing metric
- `Timer`: Duration measurement with percentiles
- `MetricPoint`: Time-series data point

**Performance**: Zero allocation for metric updates, simple struct copies

### 2. `metrics/collector.rs` - Metrics Collection

**Purpose**: Thread-safe, low-latency metrics collection

**Key Features**:
- **< 5ms overhead**: Optimized lock acquisition and minimal allocations
- **Thread-safe**: RwLock-based concurrent access
- **Memory efficient**: Automatic cleanup of old data points
- **Progress tracking**: Record intermediate progress during execution

**API**:
```rust
let collector = MetricsCollector::new();

// Start tracking
let tracking_id = collector.start_command("extract")?;

// Record progress
collector.record_progress(&tracking_id, 10, 1024, 5, 2)?;

// Complete
let metrics = collector.complete_command(&tracking_id)?;
```

**Implementation Details**:
- Uses `Arc<RwLock<>>` for thread-safe access
- Maintains HashMap of active command trackers
- Throttled memory sampling (100ms intervals)
- Automatic cleanup of time-series data (max 1000 points)

### 3. `metrics/storage.rs` - Persistent Storage

**Purpose**: Durable metrics storage with automatic rotation

**Key Features**:
- **JSON-based storage**: Human-readable format
- **Automatic rotation**: Configurable thresholds
- **Retention policies**: Age-based and size-based cleanup
- **Atomic writes**: Temporary file + rename for safety
- **Archiving**: Old data archived before rotation

**Configuration**:
```rust
MetricsStorageConfig {
    max_command_history: 1000,     // Max entries to keep
    retention_days: 30,            // Max age
    auto_cleanup: true,            // Enable rotation
    storage_path: "~/.riptide/metrics.json",
    rotation_threshold: 500,       // Trigger cleanup
}
```

**Storage Format**:
```json
{
  "command_history": [...],
  "summary": {
    "total_commands": 1234,
    "overall_success_rate": 98.5,
    "command_stats": {...}
  }
}
```

**Export Formats**:
- JSON: Full detailed export
- CSV: Tabular format for analysis
- Prometheus: Metrics format for scraping

### 4. `metrics/aggregator.rs` - Statistical Analysis

**Purpose**: Compute statistics and detect patterns

**Key Features**:
- **Percentile calculation**: P50, P95, P99
- **Moving averages**: Time-window based
- **Anomaly detection**: Z-score method
- **Rate of change**: Trend analysis
- **Time bucketing**: Group by hour/day/week

**API**:
```rust
let aggregator = MetricsAggregator::new();

// Aggregate by command
let aggregates = aggregator.aggregate_by_command(&metrics);

// Calculate percentiles
let (p50, p95, p99) = aggregator.calculate_metric_percentiles(&points);

// Detect anomalies
let anomalies = aggregator.detect_anomalies(&points, 2.0);
```

**Algorithms**:
- Percentiles: Linear interpolation on sorted values
- Moving average: Sliding window
- Anomaly detection: Statistical z-score (threshold configurable)
- Rate of change: Delta over time

### 5. `metrics/mod.rs` - Public API

**Purpose**: Unified interface and global manager

**Key Features**:
- **Global singleton**: `MetricsManager::global()`
- **Async API**: All operations are async-safe
- **Macro support**: `track_command!` for automatic tracking
- **Telemetry integration**: OpenTelemetry export

**Usage Example**:
```rust
use riptide_cli::metrics::MetricsManager;

async fn execute_extract() -> Result<()> {
    let metrics = MetricsManager::global();
    let tracking_id = metrics.start_command("extract").await?;

    // Do work...
    metrics.record_progress(&tracking_id, 10, 1024, 5, 2).await?;

    metrics.complete_command(&tracking_id).await?;
    Ok(())
}
```

**Macro Usage**:
```rust
track_command!("crawl", {
    // Your command logic
    let results = crawl_website().await?;
    Ok(results)
})
```

## Integration Points

### 1. Command Integration

Each CLI command should integrate metrics tracking:

```rust
pub async fn execute(client: RipTideClient, args: Args, output: &str) -> Result<()> {
    let metrics = MetricsManager::global();
    let tracking_id = metrics.start_command("extract").await?;

    match perform_extraction(client, args).await {
        Ok(result) => {
            metrics.record_progress(
                &tracking_id,
                result.items_count,
                result.bytes_transferred,
                result.cache_hits,
                result.api_calls,
            ).await?;

            metrics.complete_command(&tracking_id).await?;
            output_result(result, output)
        }
        Err(e) => {
            metrics.fail_command(&tracking_id, e.to_string()).await?;
            Err(e)
        }
    }
}
```

### 2. Telemetry Integration

Metrics automatically integrate with riptide-core telemetry:

```rust
use riptide_cli::metrics::telemetry_integration;

// Export to OpenTelemetry
let otel_attrs = telemetry_integration::to_otel_attributes(&metrics);

// Record via tracing
tracing::info!(
    command = %metrics.command_name,
    duration_ms = %metrics.duration_ms,
    "Command completed"
);
```

### 3. API Server Integration

CLI metrics can be queried via the existing `/monitoring/metrics/current` endpoint by extending the response:

```rust
#[derive(Serialize)]
struct ExtendedMetricsResponse {
    // Existing API metrics
    requests_total: u64,
    requests_per_second: f64,

    // CLI metrics
    cli_commands_total: u64,
    cli_success_rate: f64,
    cli_command_stats: HashMap<String, CommandAggregates>,
}
```

## Performance Characteristics

### Collection Overhead
- **Start command**: < 1ms (HashMap insert)
- **Record progress**: < 0.5ms (RwLock write + memory check)
- **Complete command**: < 3ms (finalize + persist)
- **Total per command**: < 5ms

### Memory Usage
- **Active commands**: ~500 bytes per command
- **History**: ~1KB per command metric
- **Max memory**: ~1MB for 1000 commands

### Storage
- **Write frequency**: After each command completion
- **File size**: ~100KB for 100 commands (JSON)
- **Rotation**: Automatic at 500 commands
- **Archive size**: ~1MB per 1000 archived commands

## Configuration

### Environment Variables
```bash
# Storage location
export RIPTIDE_METRICS_PATH="~/.riptide/metrics.json"

# Retention (days)
export RIPTIDE_METRICS_RETENTION=30

# Max history entries
export RIPTIDE_METRICS_MAX_HISTORY=1000

# Disable metrics collection
export RIPTIDE_METRICS_DISABLED=false
```

### Configuration File
```yaml
# ~/.riptide/config.yaml
metrics:
  enabled: true
  storage_path: ~/.riptide/metrics.json
  retention_days: 30
  max_command_history: 1000
  rotation_threshold: 500
  auto_cleanup: true
```

## CLI Commands

### View Metrics
```bash
# Show current metrics summary
riptide metrics show

# Show specific command stats
riptide metrics show --command extract

# Show recent history
riptide metrics history --limit 20
```

### Export Metrics
```bash
# Export as JSON
riptide metrics export --format json --output metrics.json

# Export as CSV for analysis
riptide metrics export --format csv --output metrics.csv

# Export Prometheus format
riptide metrics export --format prometheus
```

### Manage Storage
```bash
# Clear all metrics
riptide metrics clear

# Archive current metrics
riptide metrics archive

# Show storage info
riptide metrics storage-info
```

## Testing

### Unit Tests
```bash
cd crates/riptide-cli
cargo test --lib metrics
```

### Integration Tests
```bash
cargo test --test metrics_integration
```

### Performance Tests
```bash
cargo bench --bench metrics_perf
```

## Future Enhancements

1. **Real-time Dashboard**: Web UI for live metrics visualization
2. **Alerting**: Threshold-based alerts for errors/performance
3. **Distributed Tracing**: Full OpenTelemetry integration
4. **ML-based Anomaly Detection**: Smarter pattern recognition
5. **Multi-tenant Metrics**: Per-user/project isolation
6. **Metric Streaming**: Real-time export to external systems

## References

- [OpenTelemetry Specification](https://opentelemetry.io/docs/specs/otel/)
- [Prometheus Metrics](https://prometheus.io/docs/concepts/metric_types/)
- [RipTide Core Telemetry](../crates/riptide-core/src/telemetry.rs)
- [Metrics Best Practices](https://sre.google/sre-book/monitoring-distributed-systems/)
