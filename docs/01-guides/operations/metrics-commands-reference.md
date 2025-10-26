# RipTide CLI Metrics Commands - Quick Reference

## Command Overview

| Command | Description | Key Options |
|---------|-------------|-------------|
| `riptide metrics show` | Display current metrics summary | `--output <format>` |
| `riptide metrics tail` | Live metrics monitoring | `--interval <time>`, `--limit <n>` |
| `riptide metrics export` | Export metrics to file | `--format <type>`, `--output <file>`, `--metric <filter>` |

## Examples

### Show Metrics

```bash
# Default text output
riptide metrics show

# Table format
riptide metrics show --output table

# JSON format
riptide metrics show --output json
```

### Live Monitoring

```bash
# Default (2 second updates)
riptide metrics tail

# Fast updates (1 second)
riptide metrics tail --interval 1s

# Very fast updates (500ms)
riptide metrics tail --interval 500ms

# Show more history
riptide metrics tail --limit 20

# JSON streaming output
riptide metrics tail --output json
```

### Export Metrics

```bash
# Prometheus format to file
riptide metrics export --format prom --output metrics.prom

# CSV format to file
riptide metrics export --format csv --output metrics.csv

# JSON format to stdout
riptide metrics export --format json

# Filter specific metrics
riptide metrics export --format prom --metric cache

# Filter and output to file
riptide metrics export --format prom --metric latency --output latency.prom
```

## Output Formats

### Text Format (Default)
```
✓ CLI Metrics Summary

Total Commands: 42
Success Rate: 95.24%
Average Duration: 234.50ms
Total Bytes Transferred: 12.34 MB
API Calls: 156
```

### Table Format
```
╔══════════════════════╦══════════════╗
║ Metric               ║ Value        ║
╠══════════════════════╬══════════════╣
║ CLI METRICS          ║              ║
║ Total Commands       ║ 42           ║
║ Success Rate         ║ 95.24%       ║
...
```

### JSON Format
```json
{
  "cli": {
    "total_commands": 42,
    "overall_success_rate": 95.24,
    "avg_command_duration_ms": 234.50,
    "total_bytes_transferred": 12939264,
    "total_api_calls": 156
  },
  "server": {
    "requests_total": 1234,
    "requests_per_second": 45.67,
    "average_latency_ms": 123.45
  }
}
```

### Prometheus Format
```prometheus
# HELP riptide_cli_commands_total Total CLI commands executed
# TYPE riptide_cli_commands_total counter
riptide_cli_commands_total 42

# HELP riptide_cli_success_rate CLI command success rate
# TYPE riptide_cli_success_rate gauge
riptide_cli_success_rate 0.9524
```

### CSV Format
```csv
timestamp,command,duration_ms,success,items_processed,bytes_transferred
2025-10-16T05:30:42Z,extract,234,true,5,1024
2025-10-16T05:30:40Z,crawl,456,true,12,4096
```

## Interval Formats

The `--interval` flag supports multiple time formats:

| Format | Example | Description |
|--------|---------|-------------|
| Seconds | `1s`, `2s`, `10s` | Update every N seconds |
| Milliseconds | `500ms`, `1000ms`, `100ms` | Update every N milliseconds |
| Number only | `1`, `2`, `5` | Treated as seconds |

## Integration with Monitoring Systems

### Prometheus

Export metrics and configure Prometheus to scrape them:

```bash
# Export metrics
riptide metrics export --format prom --output /var/lib/prometheus/riptide_metrics.prom

# Add to prometheus.yml
scrape_configs:
  - job_name: 'riptide-cli'
    file_sd_configs:
      - files:
        - /var/lib/prometheus/riptide_metrics.prom
```

### Grafana

1. Export metrics to Prometheus format
2. Import into Prometheus
3. Create Grafana dashboard using riptide_* metrics

### CSV Analysis

Export to CSV for analysis in Excel, Python, or R:

```bash
riptide metrics export --format csv --output metrics.csv

# Analyze in Python
import pandas as pd
df = pd.read_csv('metrics.csv')
print(df.describe())
```

## Metrics Collected

### CLI Metrics
- `total_commands` - Total number of commands executed
- `overall_success_rate` - Percentage of successful commands
- `avg_command_duration_ms` - Average execution time in milliseconds
- `total_bytes_transferred` - Total bytes downloaded/uploaded
- `total_api_calls` - Total API calls made

### Server Metrics (when available)
- `requests_total` - Total server requests
- `requests_per_second` - Current RPS
- `average_latency_ms` - Average request latency
- `cache_hit_rate` - Cache hit rate percentage
- `worker_queue_size` - Current worker queue size

### Per-Command Metrics
- `command_name` - Name of the command
- `started_at` - Execution start time
- `duration_ms` - Execution duration
- `success` - Success/failure status
- `error` - Error message (if failed)
- `items_processed` - Number of items processed
- `bytes_transferred` - Bytes transferred
- `cache_hits` - Number of cache hits
- `api_calls` - Number of API calls

## Tips and Best Practices

### Performance Monitoring

```bash
# Monitor during heavy workload
riptide metrics tail --interval 500ms &

# Run your workload
for i in {1..100}; do
  riptide extract --url https://example.com
done

# Stop monitoring
fg  # Ctrl+C
```

### Debugging

```bash
# Export detailed metrics after a failed command
riptide metrics export --format json --output debug.json

# Filter specific metrics
riptide metrics export --format json --metric extract
```

### Continuous Monitoring

```bash
# Run in background with output redirect
riptide metrics tail --output json > metrics.log 2>&1 &

# View logs
tail -f metrics.log | jq .
```

### Metric Filtering

```bash
# Export only cache-related metrics
riptide metrics export --format prom --metric cache

# Export only latency metrics
riptide metrics export --format prom --metric latency
```

## Error Handling

All commands provide clear error messages:

```bash
# Invalid interval format
$ riptide metrics tail --interval abc
Error: Invalid interval format

# Unsupported export format
$ riptide metrics export --format xml
Error: Unsupported export format: xml. Use prom, csv, or json

# File write error
$ riptide metrics export --format prom --output /invalid/path.prom
Error: Failed to create output file: /invalid/path.prom
```

## Environment Variables

Metrics commands respect these environment variables:

- `RIPTIDE_API_URL` - API server URL (default: http://localhost:8080)
- `RIPTIDE_API_KEY` - API authentication key
- `RUST_LOG` - Logging level

## Related Commands

- `riptide health` - Check system health
- `riptide system-check` - Comprehensive system check
- `riptide cache stats` - Cache statistics

## Command Architecture

```
User Command
    ↓
CLI Parser (clap)
    ↓
Command Handler (commands/metrics.rs)
    ↓
MetricsManager (metrics/mod.rs)
    ↓
├─ MetricsCollector (in-memory tracking)
├─ MetricsStorage (persistent storage)
└─ MetricsAggregator (statistics calculation)
```

## Files Modified

- `crates/riptide-cli/src/commands/mod.rs` - Added Tail command
- `crates/riptide-cli/src/commands/metrics.rs` - Complete rewrite
- `crates/riptide-cli/src/main.rs` - Added tail handling
- `crates/riptide-cli/Cargo.toml` - Added ctrlc dependency
