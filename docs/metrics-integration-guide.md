# Metrics Integration Guide

## Overview

This guide documents the comprehensive metrics collection system integrated into RipTide CLI commands. The metrics system provides low-overhead (<5ms) tracking of command execution, performance, and resource usage.

## Architecture

```
┌──────────────────────────────────────────────────┐
│              CLI Commands                          │
│  (extract, render, crawl, pdf, etc.)             │
└────────────────┬─────────────────────────────────┘
                 │
                 ▼
┌──────────────────────────────────────────────────┐
│           MetricsManager (Global)                 │
│  - Thread-safe coordination                       │
│  - Automatic command tracking                     │
└────┬────────────┬────────────┬────────────────────┘
     │            │            │
     ▼            ▼            ▼
┌─────────┐ ┌─────────┐ ┌──────────┐
│Collector│ │ Storage │ │Aggregator│
│(Runtime)│ │(Persist)│ │(Analysis)│
└─────────┘ └─────────┘ └──────────┘
     │            │            │
     ▼            ▼            ▼
  Counters    metrics.json  Statistics
  Timers                    Percentiles
  Series                    Trends
```

## Integrated Commands

### 1. Extract Command

**Metrics Tracked:**
- Total execution time
- Engine selection (raw/wasm/headless)
- Strategy used
- Success/failure status
- Response size (bytes)
- API call latency
- Network latency breakdown

**Example Usage:**
```bash
# Run extract command - metrics automatically collected
riptide extract --url https://example.com --engine wasm

# View metrics
riptide metrics show
```

**Collected Metrics:**
```json
{
  "command_name": "extract",
  "duration_ms": 250,
  "success": true,
  "items_processed": 1,
  "bytes_transferred": 5120,
  "api_calls": 1,
  "metadata": {
    "engine": "wasm",
    "strategy": "auto",
    "latency_ms": 120
  }
}
```

### 2. Render Command

**Metrics Tracked:**
- Total execution time
- Wait condition used
- Screenshot mode
- Number of files saved
- Total file size
- Success/failure status

**Example Usage:**
```bash
# Run render command
riptide render --url https://example.com --wait load --screenshot viewport

# Export metrics to Prometheus format
riptide metrics export --format prom --output metrics.prom
```

**Collected Metrics:**
```json
{
  "command_name": "render",
  "duration_ms": 450,
  "success": true,
  "items_processed": 3,
  "bytes_transferred": 15360,
  "metadata": {
    "wait_condition": "load",
    "screenshot_mode": "viewport",
    "files_saved": ["html", "screenshot", "dom"]
  }
}
```

### 3. Crawl Command

**Metrics Tracked:**
- Total execution time
- Number of pages crawled
- Total bytes transferred
- API latency
- Crawl depth
- Success/failure status

**Example Usage:**
```bash
# Run crawl command
riptide crawl --url https://example.com --depth 2 --max-pages 10

# Monitor metrics in real-time
riptide metrics tail --interval 1s
```

**Collected Metrics:**
```json
{
  "command_name": "crawl",
  "duration_ms": 3500,
  "success": true,
  "items_processed": 10,
  "bytes_transferred": 102400,
  "api_calls": 1,
  "metadata": {
    "depth": 2,
    "pages_crawled": 10,
    "api_latency_ms": 2800
  }
}
```

### 4. PDF Commands

**Metrics Tracked:**
- Command type (extract/to_md/info/stream)
- Execution time
- Success/failure status
- Items processed

**Example Usage:**
```bash
# Run PDF extraction
riptide pdf extract --input document.pdf --format json

# View aggregated metrics
riptide metrics show --format table
```

## Metrics Collection API

### Starting Command Tracking

```rust
use riptide_cli::metrics::MetricsManager;

pub async fn execute_command() -> Result<()> {
    // Get global metrics manager
    let metrics_manager = MetricsManager::global();

    // Start tracking
    let tracking_id = metrics_manager.start_command("command_name").await?;

    // Do work...

    // Complete successfully
    metrics_manager.complete_command(&tracking_id).await?;

    Ok(())
}
```

### Recording Progress

```rust
// Record progress during execution
metrics_manager.record_progress(
    &tracking_id,
    items_processed: 10,      // Number of items
    bytes_transferred: 1024,  // Bytes downloaded/uploaded
    cache_hits: 5,           // Cache hits
    api_calls: 2,            // API calls made
).await?;
```

### Recording Custom Metrics

```rust
// Record engine selection
metrics_manager.collector().record_metric(
    "extract.engine.wasm",
    1.0
)?;

// Record latency
metrics_manager.collector().record_metric(
    "api.latency_ms",
    120.5
)?;

// Increment counter
metrics_manager.increment_counter("cache.hits")?;
```

### Handling Failures

```rust
// On failure, record error
match result {
    Ok(_) => {
        metrics_manager.complete_command(&tracking_id).await?;
    }
    Err(e) => {
        metrics_manager.fail_command(&tracking_id, e.to_string()).await?;
    }
}
```

## Metrics Storage

### Configuration

```rust
use riptide_cli::metrics::MetricsStorageConfig;

let config = MetricsStorageConfig {
    max_command_history: 1000,      // Max commands to retain
    retention_days: 30,              // Keep metrics for 30 days
    auto_cleanup: true,              // Auto-cleanup old data
    storage_path: "~/.riptide/metrics.json".to_string(),
    rotation_threshold: 500,         // Rotate after 500 entries
};
```

### Storage Location

Default: `~/.cache/riptide/metrics.json`

**Structure:**
```json
{
  "summary": {
    "total_commands": 150,
    "overall_success_rate": 94.7,
    "total_bytes_transferred": 1048576,
    "avg_command_duration_ms": 285.5
  },
  "commands": [
    {
      "command_name": "extract",
      "started_at": "2025-10-16T10:30:00Z",
      "duration_ms": 250,
      "success": true,
      "items_processed": 1,
      "bytes_transferred": 5120
    }
  ]
}
```

## Export Formats

### JSON Export

```bash
riptide metrics export --format json --output metrics.json
```

**Output:**
```json
{
  "commands": [...],
  "summary": {...},
  "aggregates": {...}
}
```

### CSV Export

```bash
riptide metrics export --format csv --output metrics.csv
```

**Output:**
```csv
timestamp,command,duration_ms,success,items,bytes,api_calls,cache_hits
2025-10-16T10:30:00Z,extract,250,true,1,5120,1,0
2025-10-16T10:31:15Z,render,450,true,3,15360,0,0
```

### Prometheus Export

```bash
riptide metrics export --format prom --output metrics.prom
```

**Output:**
```prometheus
# HELP riptide_cli_commands_total Total CLI commands executed
# TYPE riptide_cli_commands_total counter
riptide_cli_commands_total{command="extract"} 45

# HELP riptide_cli_command_duration_seconds Command execution duration
# TYPE riptide_cli_command_duration_seconds histogram
riptide_cli_command_duration_seconds_sum{command="extract"} 12.5
riptide_cli_command_duration_seconds_count{command="extract"} 45
```

## Performance Monitoring

### Real-Time Monitoring

```bash
# Monitor metrics every 2 seconds
riptide metrics tail --interval 2s --limit 10
```

**Output:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  RipTide Metrics Monitor (updating every 2s)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📊 SUMMARY
   Commands: 45  |  Success: 97.8%  |  Avg: 285ms
   Transferred: 512.5 KB  |  API Calls: 23

🕒 RECENT COMMANDS
┌──────────┬─────────┬──────────┬────────┬───────┐
│   Time   │ Command │ Duration │ Status │ Items │
├──────────┼─────────┼──────────┼────────┼───────┤
│ 10:30:15 │ extract │   250ms  │ ✓ OK   │   1   │
│ 10:30:20 │ render  │   450ms  │ ✓ OK   │   3   │
│ 10:30:45 │ crawl   │  3500ms  │ ✓ OK   │  10   │
└──────────┴─────────┴──────────┴────────┴───────┘
```

### Aggregated Statistics

```bash
riptide metrics show --format table
```

**Output:**
```
┌──────────────────────┬─────────────┐
│       Metric         │    Value    │
├──────────────────────┼─────────────┤
│ Total Commands       │     150     │
│ Success Rate         │   94.70%    │
│ Avg Duration         │  285.50ms   │
│ Total Bytes          │   1.00 MB   │
│ API Calls            │      45     │
└──────────────────────┴─────────────┘
```

## Integration Points

### Command Integration Checklist

When adding metrics to a new command:

1. **Import MetricsManager**
   ```rust
   use crate::metrics::MetricsManager;
   use std::time::Instant;
   ```

2. **Start Tracking**
   ```rust
   let metrics_manager = MetricsManager::global();
   let tracking_id = metrics_manager.start_command("command_name").await?;
   let start_time = Instant::now();
   ```

3. **Record Progress**
   ```rust
   metrics_manager.record_progress(
       &tracking_id,
       items_processed,
       bytes_transferred,
       cache_hits,
       api_calls
   ).await?;
   ```

4. **Record Custom Metrics**
   ```rust
   metrics_manager.collector().record_metric(
       "command.specific.metric",
       value
   )?;
   ```

5. **Complete Tracking**
   ```rust
   match result {
       Ok(_) => {
           metrics_manager.complete_command(&tracking_id).await?;
       }
       Err(e) => {
           metrics_manager.fail_command(&tracking_id, e.to_string()).await?;
       }
   }
   ```

## Best Practices

### 1. Minimal Overhead

- Metrics collection adds <5ms overhead
- Use throttled memory sampling (100ms intervals)
- Batch metric writes to storage
- Async operations don't block command execution

### 2. Error Handling

```rust
// Always handle errors gracefully
if let Err(e) = metrics_manager.start_command("cmd").await {
    tracing::warn!("Failed to start metrics tracking: {}", e);
    // Continue with command execution
}
```

### 3. Metadata Recording

```rust
// Add command-specific metadata
let mut metrics = CommandMetrics::new("extract");
metrics.add_metadata("engine", "wasm");
metrics.add_metadata("strategy", "auto");
```

### 4. Network Latency Breakdown

```rust
// Track different phases
let fetch_start = Instant::now();
let response = client.get(url).send().await?;
let fetch_latency = fetch_start.elapsed();

metrics_manager.collector().record_metric(
    "network.fetch_ms",
    fetch_latency.as_millis() as f64
)?;

let parse_start = Instant::now();
let data = response.json().await?;
let parse_latency = parse_start.elapsed();

metrics_manager.collector().record_metric(
    "network.parse_ms",
    parse_latency.as_millis() as f64
)?;
```

## Troubleshooting

### Metrics Not Appearing

1. **Check storage permissions:**
   ```bash
   ls -la ~/.cache/riptide/
   chmod 755 ~/.cache/riptide/
   ```

2. **Verify metrics file:**
   ```bash
   cat ~/.cache/riptide/metrics.json | jq .
   ```

3. **Enable debug logging:**
   ```bash
   RUST_LOG=debug riptide extract --url https://example.com
   ```

### High Memory Usage

1. **Check retention settings:**
   ```bash
   # Reduce max history
   export RIPTIDE_METRICS_MAX_HISTORY=100
   ```

2. **Enable auto-cleanup:**
   ```rust
   let config = MetricsStorageConfig {
       auto_cleanup: true,
       retention_days: 7,
       ..Default::default()
   };
   ```

### Performance Impact

1. **Disable metrics for critical paths:**
   ```bash
   export RIPTIDE_METRICS_ENABLED=false
   ```

2. **Increase sampling interval:**
   ```rust
   // Only sample memory every 500ms instead of 100ms
   const MEMORY_SAMPLE_INTERVAL_MS: u64 = 500;
   ```

## Future Enhancements

- [ ] OpenTelemetry integration for distributed tracing
- [ ] Grafana dashboard templates
- [ ] Real-time websocket streaming of metrics
- [ ] Machine learning for anomaly detection
- [ ] Predictive performance analysis
- [ ] Cost optimization recommendations

## References

- [Metrics Module Documentation](../crates/riptide-cli/src/metrics/mod.rs)
- [MetricsManager API](../crates/riptide-cli/src/metrics/types.rs)
- [Integration Tests](../crates/riptide-cli/tests/metrics_integration_test.rs)
