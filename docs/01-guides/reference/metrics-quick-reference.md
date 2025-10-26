# Metrics Integration Quick Reference

## 🚀 Quick Start

```rust
use crate::metrics::MetricsManager;
use std::time::Instant;

pub async fn my_command() -> Result<()> {
    // 1. Start tracking
    let metrics = MetricsManager::global();
    let id = metrics.start_command("my_command").await?;

    // 2. Do work
    let result = do_work().await;

    // 3. Complete
    match result {
        Ok(_) => metrics.complete_command(&id).await?,
        Err(e) => metrics.fail_command(&id, e.to_string()).await?
    }

    Ok(())
}
```

## 📊 Common Patterns

### Record Progress
```rust
metrics.record_progress(
    &id,
    items: 10,      // Items processed
    bytes: 1024,    // Bytes transferred
    cache: 5,       // Cache hits
    api: 2          // API calls
).await?;
```

### Custom Metrics
```rust
// Counter
metrics.collector().record_metric("my.metric", 1.0)?;

// Latency
let start = Instant::now();
do_network_call().await?;
metrics.collector().record_metric(
    "network.latency_ms",
    start.elapsed().as_millis() as f64
)?;
```

### Metadata
```rust
// Add custom metadata
let mut cmd = CommandMetrics::new("my_cmd");
cmd.add_metadata("engine", "wasm");
cmd.add_metadata("strategy", "auto");
```

## 🎯 Per-Command Metrics

### Extract
```rust
// Engine selection
metrics.collector().record_metric("extract.engine.wasm", 1.0)?;

// API latency
let api_start = Instant::now();
let response = client.post("/api/v1/extract", &req).await?;
metrics.collector().record_metric(
    "extract.api.latency_ms",
    api_start.elapsed().as_millis() as f64
)?;

// Response size
metrics.record_progress(&id, 1, response.len() as u64, 0, 1).await?;
```

### Render
```rust
// Wait condition
metrics.collector().record_metric("render.wait.load", 1.0)?;

// Files saved
let bytes = files.iter().map(|f| f.size).sum::<u64>();
metrics.record_progress(&id, files.len() as u64, bytes, 0, 0).await?;
```

### Crawl
```rust
// Pages crawled
metrics.collector().record_metric("crawl.pages", 10.0)?;

// API latency + bytes
let bytes: u64 = pages.iter().map(|p| p.size).sum();
metrics.record_progress(&id, pages.len() as u64, bytes, 0, 1).await?;
```

## 🔧 CLI Commands

```bash
# View metrics
riptide metrics show

# Real-time monitoring
riptide metrics tail --interval 2s

# Export JSON
riptide metrics export --format json -o metrics.json

# Export Prometheus
riptide metrics export --format prom -o metrics.prom

# Export CSV
riptide metrics export --format csv -o metrics.csv
```

## 📈 Metrics Collected

| Metric | Type | Description |
|--------|------|-------------|
| `command_name` | string | Command name (extract, render, etc.) |
| `duration_ms` | u64 | Total execution time |
| `success` | bool | Success/failure status |
| `items_processed` | u64 | Number of items processed |
| `bytes_transferred` | u64 | Total bytes downloaded/uploaded |
| `cache_hits` | u64 | Number of cache hits |
| `api_calls` | u64 | Number of API calls made |
| `peak_memory_bytes` | u64 | Peak memory usage |
| `error` | string? | Error message if failed |

## ⚡ Performance

- **Overhead:** <5ms per command
- **Memory sampling:** Every 100ms (throttled)
- **Storage:** Auto-rotation after 500 entries
- **Thread-safe:** Lock-free counters where possible

## ❌ Error Handling

```rust
// Always handle gracefully
if let Err(e) = metrics.start_command("cmd").await {
    tracing::warn!("Metrics failed: {}", e);
    // Continue with command execution
}
```

## 🧪 Testing

```rust
#[tokio::test]
async fn test_metrics() {
    let m = MetricsManager::new().unwrap();
    let id = m.start_command("test").await.unwrap();
    m.record_progress(&id, 10, 1024, 0, 1).await.unwrap();
    m.complete_command(&id).await.unwrap();

    let summary = m.get_summary().await.unwrap();
    assert_eq!(summary.total_commands, 1);
}
```

## 📚 Full Documentation

See `/workspaces/eventmesh/docs/metrics-integration-guide.md`
