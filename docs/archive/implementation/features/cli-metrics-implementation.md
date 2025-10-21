# CLI Metrics Commands Implementation

## Overview

Enhanced the RipTide CLI metrics commands to provide comprehensive metrics monitoring, display, and export capabilities with integration to the new MetricsManager module.

## Implemented Commands

### 1. `riptide metrics show`

Displays current metrics summary with both CLI and server metrics.

**Usage:**
```bash
riptide metrics show
riptide metrics show --output json
riptide metrics show --output table
```

**Features:**
- Shows CLI metrics (total commands, success rate, average duration, bytes transferred, API calls)
- Fetches and displays server metrics if available (requests, RPS, latency, cache hit rate, worker queue)
- Supports multiple output formats (text, json, table)
- Uses the MetricsManager global instance for CLI metrics

**Example Output (Table):**
```
╔══════════════════════╦══════════════╗
║ Metric               ║ Value        ║
╠══════════════════════╬══════════════╣
║ CLI METRICS          ║              ║
║ Total Commands       ║ 42           ║
║ Success Rate         ║ 95.24%       ║
║ Avg Duration         ║ 234.50ms     ║
║ Total Bytes          ║ 12.34 MB     ║
║ API Calls            ║ 156          ║
║                      ║              ║
║ SERVER METRICS       ║              ║
║ Total Requests       ║ 1,234        ║
║ Requests/Second      ║ 45.67        ║
║ Avg Latency          ║ 123.45ms     ║
║ Cache Hit Rate       ║ 78.90%       ║
║ Worker Queue         ║ 5            ║
╚══════════════════════╩══════════════╝
```

### 2. `riptide metrics tail`

Live metrics monitoring with real-time updates and a beautiful terminal UI.

**Usage:**
```bash
riptide metrics tail                    # Default 2s interval
riptide metrics tail --interval 1s      # 1 second updates
riptide metrics tail --interval 500ms   # 500ms updates
riptide metrics tail --limit 20         # Show 20 recent commands
riptide metrics tail --output json      # JSON output mode
```

**Features:**
- Real-time metrics updates with configurable interval
- Beautiful terminal UI with colors and tables
- Shows summary statistics (commands, success rate, avg duration, bytes transferred)
- Displays recent command history in a table (time, command, duration, status, items)
- Clear screen refresh between updates
- Graceful shutdown with Ctrl+C
- Supports both formatted and JSON output

**Example Output:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  RipTide Metrics Monitor (updating every 2s)  2025-10-16 05:30:45 UTC
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📊 SUMMARY
   Commands: 42  |  Success: 95.2%  |  Avg: 235ms
   Transferred: 12.34 MB  |  API Calls: 156

🕒 RECENT COMMANDS
╔══════════╦══════════╦══════════╦════════╦═══════╗
║ Time     ║ Command  ║ Duration ║ Status ║ Items ║
╠══════════╬══════════╬══════════╬════════╬═══════╣
║ 05:30:42 ║ extract  ║ 234ms    ║ ✓ OK   ║ 5     ║
║ 05:30:40 ║ crawl    ║ 456ms    ║ ✓ OK   ║ 12    ║
║ 05:30:38 ║ search   ║ 123ms    ║ ✓ OK   ║ 8     ║
║ 05:30:35 ║ extract  ║ 289ms    ║ ✗ FAIL ║ 0     ║
╚══════════╩══════════╩══════════╩════════╩═══════╝

Press Ctrl+C to stop monitoring
```

### 3. `riptide metrics export`

Export metrics in various formats (Prometheus, CSV, JSON).

**Usage:**
```bash
riptide metrics export --format prom --output metrics.prom
riptide metrics export --format csv --output metrics.csv
riptide metrics export --format json --output metrics.json
riptide metrics export --format prom                  # Print to stdout
riptide metrics export --format prom --metric cache   # Filter metrics
```

**Features:**
- Exports CLI metrics from MetricsManager
- Includes server metrics if available
- Supports Prometheus, CSV, and JSON formats
- Optional filtering by metric name
- Can output to file or stdout
- Properly formatted output for each format

**Example Prometheus Output:**
```prometheus
# CLI Metrics
# HELP riptide_cli_commands_total Total CLI commands executed
# TYPE riptide_cli_commands_total counter
riptide_cli_commands_total 42

# HELP riptide_cli_success_rate CLI command success rate
# TYPE riptide_cli_success_rate gauge
riptide_cli_success_rate 0.9524

# HELP riptide_cli_avg_duration_ms Average CLI command duration
# TYPE riptide_cli_avg_duration_ms gauge
riptide_cli_avg_duration_ms 234.50

# Server Metrics
# HELP riptide_server_requests_total Total server requests
# TYPE riptide_server_requests_total counter
riptide_server_requests_total 1234

# HELP riptide_server_rps Server requests per second
# TYPE riptide_server_rps gauge
riptide_server_rps 45.67
```

## Implementation Details

### Code Structure

**Modified Files:**
1. `/workspaces/eventmesh/crates/riptide-cli/src/commands/mod.rs`
   - Added `Tail` variant to `MetricsCommands` enum
   - Added `interval` and `limit` parameters

2. `/workspaces/eventmesh/crates/riptide-cli/src/commands/metrics.rs`
   - Rewrote entire file with new implementation
   - Added `execute()` for show command
   - Added `tail()` for live monitoring
   - Enhanced `export()` with MetricsManager integration
   - Added helper functions for formatting and parsing

3. `/workspaces/eventmesh/crates/riptide-cli/src/main.rs`
   - Updated match statement to handle Tail command

4. `/workspaces/eventmesh/crates/riptide-cli/Cargo.toml`
   - Added `ctrlc = "3.4"` dependency for Ctrl+C handling

### Key Functions

#### `execute(client, output_format)`
- Fetches CLI metrics from MetricsManager
- Optionally fetches server metrics via API
- Displays combined metrics in requested format

#### `tail(client, interval_str, limit, output_format)`
- Parses interval string (e.g., "2s", "500ms")
- Sets up Ctrl+C handler for graceful shutdown
- Runs infinite loop with periodic updates
- Clears screen and displays fresh metrics each iteration
- Shows summary stats and recent command history

#### `export(client, format, output_path, metric_filter)`
- Determines export format (Prometheus, CSV, JSON)
- Exports from MetricsManager
- Optionally includes server metrics
- Applies filter if specified
- Writes to file or stdout

#### Helper Functions
- `print_metrics_table()` - Beautiful table output with colors
- `print_metrics_text()` - Plain text output
- `print_tail_display()` - Live monitoring UI
- `format_server_metrics()` - Format server metrics for export
- `filter_metrics()` - Apply metric name filter
- `parse_interval()` - Parse interval strings (1s, 500ms, etc.)
- `format_bytes()` - Human-readable byte formatting

### Integration with MetricsManager

The implementation uses the MetricsManager from `/workspaces/eventmesh/crates/riptide-cli/src/metrics/mod.rs`:

```rust
let metrics_manager = MetricsManager::global();

// Get summary
let summary = metrics_manager.get_summary().await?;

// Get recent commands
let recent_commands = metrics_manager.get_recent_commands(limit).await?;

// Export metrics
let export_data = metrics_manager.export(ExportFormat::Prometheus).await?;
```

## Testing

### Unit Tests

The implementation includes unit tests for critical functions:

```rust
#[test]
fn test_parse_interval() {
    assert_eq!(parse_interval("1s").unwrap(), Duration::from_secs(1));
    assert_eq!(parse_interval("500ms").unwrap(), Duration::from_millis(500));
}

#[test]
fn test_format_bytes() {
    assert_eq!(format_bytes(1024), "1.00 KB");
    assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
}

#[test]
fn test_filter_metrics() {
    let data = "metric1,100\nmetric2,200\nother,300";
    let filtered = filter_metrics(data, "metric");
    assert!(filtered.contains("metric1"));
}
```

### Manual Testing Commands

```bash
# Test show command
riptide metrics show
riptide metrics show --output table
riptide metrics show --output json

# Test tail command
riptide metrics tail
riptide metrics tail --interval 1s
riptide metrics tail --interval 500ms --limit 20

# Test export command
riptide metrics export --format prom --output metrics.prom
riptide metrics export --format csv --output metrics.csv
riptide metrics export --format json
riptide metrics export --format prom --metric cache
```

## Compilation Status

✅ **Code compiles successfully** with only warnings about unused code.

```bash
cargo check -p riptide-cli
# Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.34s
# 23 warnings (unused code, not errors)
```

## Features Summary

✅ **Implemented:**
- `riptide metrics show` - Display current metrics summary
- `riptide metrics tail --interval 2s` - Live metrics monitoring
- `riptide metrics export --prom --file metrics.prom` - Export to Prometheus

✅ **Additional Features:**
- Multiple output formats (text, table, json)
- Color-coded terminal output
- Human-readable byte formatting
- Configurable update intervals (supports s and ms)
- Graceful Ctrl+C handling
- Server + CLI metrics integration
- Metric filtering for exports
- Comprehensive error handling

## Dependencies Added

- `ctrlc = "3.4"` - For handling Ctrl+C gracefully in tail command

## Next Steps

1. **Build and test** the CLI binary
2. **Run integration tests** with actual metrics collection
3. **Document** the commands in user-facing documentation
4. **Add more export formats** if needed (e.g., InfluxDB line protocol)
5. **Optimize performance** for high-frequency tail updates

## Notes

- The implementation properly integrates with the existing MetricsManager
- All commands support the global `--output` flag for consistency
- The tail command uses terminal escape codes for screen clearing
- Export command combines both CLI and server metrics when available
- Error handling provides clear user-friendly messages
