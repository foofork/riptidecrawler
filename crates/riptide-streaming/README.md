# RipTide Streaming

Real-time streaming capabilities for RipTide with multiple protocol support and comprehensive report generation.

[![Crates.io](https://img.shields.io/crates/v/riptide-streaming.svg)](https://crates.io/crates/riptide-streaming)
[![Documentation](https://docs.rs/riptide-streaming/badge.svg)](https://docs.rs/riptide-streaming)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

## Overview

`riptide-streaming` provides real-time streaming and reporting capabilities for the RipTide extraction framework. It enables efficient delivery of extraction results through multiple streaming protocols, with built-in backpressure handling, progress tracking, and rich HTML report generation.

### Key Features

- **Multiple Streaming Protocols**
  - NDJSON (Newline-Delimited JSON) for efficient line-by-line streaming
  - Server-Sent Events (SSE) for browser-compatible real-time updates
  - WebSocket for bidirectional communication (planned)

- **Robust Streaming Architecture**
  - Adaptive backpressure control to prevent memory exhaustion
  - Progress tracking with rate estimation and ETA
  - Stream lifecycle management (start, pause, resume, complete)
  - Heartbeat mechanism for connection keep-alive
  - Configurable buffering and flow control

- **Report Generation**
  - HTML reports with interactive charts and visualizations
  - Multiple output formats (HTML, JSON, CSV, PDF)
  - Customizable themes (Light, Dark, Corporate, Modern)
  - Automatic chart generation using Plotters
  - Template-based rendering with Handlebars

- **CLI Tool**
  - Command-line interface for streaming operations
  - Configuration management
  - Server management utilities
  - Report generation from CLI

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
riptide-streaming = "0.1.0"
```

### Feature Flags

```toml
[features]
default = ["reports", "cli"]
reports = []  # Enable HTML report generation
cli = []      # Enable CLI tool support
```

## Supported Protocols

### 1. NDJSON (Newline-Delimited JSON)

NDJSON is the primary streaming format, providing efficient line-by-line JSON streaming with minimal overhead.

**Characteristics:**
- One JSON object per line
- Streamable and parseable line-by-line
- No array delimiters needed
- Ideal for log processing and data pipelines

**Stream Events:**
```json
{"type":"Event","data":{"stream_id":"...","event_type":"Started","timestamp":"..."}}
{"type":"Metadata","data":{"stream_id":"...","content_type":"application/x-ndjson"}}
{"type":"Result","data":{"id":"1","url":"https://example.com","content":"..."}}
{"type":"Progress","data":{"processed":10,"total":100,"rate_per_second":5.2}}
{"type":"Heartbeat","data":{"timestamp":"...","uptime_seconds":30}}
{"type":"Event","data":{"event_type":"Completed","data":{"total_items":100}}}
```

### 2. Server-Sent Events (SSE)

SSE provides unidirectional real-time updates from server to browser clients.

**Characteristics:**
- Native browser support via `EventSource` API
- Automatic reconnection handling
- Event-based protocol
- HTTP-based (firewall-friendly)

**SSE Format:**
```
event: start
data: {"stream_id":"...","extraction_id":"..."}

event: result
data: {"id":"1","url":"https://example.com","title":"..."}

event: progress
data: {"processed":50,"total":100,"rate_per_second":10.5}

event: complete
data: {"total_items":100,"duration_seconds":10}
```

### 3. WebSocket (Planned)

Bidirectional communication for interactive streaming scenarios.

**Planned Features:**
- Real-time control messages
- Stream pause/resume commands
- Dynamic configuration updates
- Bi-directional progress synchronization

## Streaming Architecture

### Core Components

```rust
use riptide_streaming::{
    StreamingCoordinator,
    NdjsonStreamBuilder,
    BackpressureController,
    ProgressTracker,
    ReportGenerator,
};
```

#### StreamingCoordinator

Central coordinator for managing multiple concurrent streams:

```rust
use riptide_streaming::StreamingCoordinator;

let mut coordinator = StreamingCoordinator::new();

// Start a new stream
let stream_id = coordinator
    .start_stream("extraction-123".to_string())
    .await?;

// Update progress
coordinator
    .update_progress(stream_id, 50, Some(100))
    .await?;

// Complete stream
coordinator.complete_stream(stream_id).await?;
```

#### Backpressure Controller

Adaptive backpressure management to prevent resource exhaustion:

```rust
use riptide_streaming::backpressure::{
    BackpressureController,
    BackpressureConfig,
};
use std::time::Duration;

let config = BackpressureConfig {
    max_in_flight: 1000,
    max_memory_bytes: 100 * 1024 * 1024, // 100 MB
    max_total_items: 10000,
    activation_threshold: 0.8,
    recovery_threshold: 0.6,
    check_interval: Duration::from_millis(500),
    adaptive: true,
};

let controller = BackpressureController::new(config);

// Register stream
controller.register_stream(stream_id).await?;

// Acquire resources
let permit = controller.acquire(stream_id, 1024).await?;
// Process item...
drop(permit); // Automatically releases resources

// Get metrics
let metrics = controller.get_metrics().await;
println!("Status: {:?}", metrics.status);
println!("Memory usage: {} bytes", metrics.total_memory_usage);
```

#### Progress Tracker

Track extraction progress with rate estimation:

```rust
use riptide_streaming::ProgressTracker;

let mut tracker = ProgressTracker::new();

// Start tracking
tracker.start_tracking(stream_id).await?;

// Update progress
tracker.update_progress(stream_id, 50, Some(100)).await?;

// Get progress
if let Some(progress) = tracker.get_progress(&stream_id).await {
    println!("Progress: {}/{}", progress.processed, progress.total.unwrap_or(0));
    println!("Rate: {:.2} items/sec", progress.rate_per_second);
}

// Complete tracking
tracker.complete_tracking(stream_id).await?;
```

## Usage Examples

### Basic NDJSON Streaming

```rust
use riptide_streaming::{
    NdjsonStreamBuilder,
    ExtractionResult,
};
use tokio_stream::{StreamExt, iter};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stream_id = Uuid::new_v4();
    let extraction_id = "extraction-123".to_string();

    // Create a stream of extraction results
    let results = vec![
        Ok(ExtractionResult { /* ... */ }),
        Ok(ExtractionResult { /* ... */ }),
    ];
    let results_stream = iter(results);

    // Build NDJSON stream
    let ndjson_stream = NdjsonStreamBuilder::new()
        .buffer_size(1000)
        .include_progress(true)
        .include_heartbeat(true)
        .heartbeat_interval(std::time::Duration::from_secs(30))
        .include_metadata(true)
        .build(stream_id, extraction_id, results_stream);

    // Consume stream
    tokio::pin!(ndjson_stream);
    while let Some(item) = ndjson_stream.next().await {
        match item {
            Ok(ndjson_item) => {
                println!("{}", ndjson_item);
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}
```

### Custom Stream Configuration

```rust
use riptide_streaming::ndjson::{NdjsonConfig, NdjsonStream};
use std::time::Duration;

let config = NdjsonConfig {
    buffer_size: 500,
    include_progress: true,
    include_heartbeat: true,
    heartbeat_interval: Duration::from_secs(15),
    include_metadata: true,
    max_line_length: 2 * 1024 * 1024, // 2MB per line
    compression_enabled: false,
    pretty_print: false,
};

let stream = NdjsonStream::from_stream(
    stream_id,
    extraction_id,
    results_stream,
    config,
);
```

### Stream with Backpressure

```rust
use riptide_streaming::{
    NdjsonStreamBuilder,
    BackpressureController,
    BackpressureConfig,
};

// Setup backpressure
let bp_config = BackpressureConfig::default();
let controller = BackpressureController::new(bp_config);
controller.register_stream(stream_id).await?;

// Process with backpressure control
for result in extraction_results {
    // Acquire resources
    let permit = controller
        .acquire(stream_id, estimate_memory_size(&result))
        .await?;

    // Send result to stream
    send_to_stream(result).await?;

    // Permit automatically released on drop
    drop(permit);
}
```

### Converting Stream to Bytes

```rust
use riptide_streaming::ndjson::utils::into_bytes_stream;

let bytes_stream = into_bytes_stream(ndjson_stream, config);

tokio::pin!(bytes_stream);
while let Some(bytes_result) = bytes_stream.next().await {
    let bytes = bytes_result?;
    // Write bytes to HTTP response, file, etc.
    writer.write_all(&bytes).await?;
}
```

## Report Generation

### HTML Reports with Charts

```rust
use riptide_streaming::reports::{
    ReportGenerator,
    ReportConfig,
    ReportFormat,
    ReportTheme,
};

// Configure report
let config = ReportConfig {
    title: "Extraction Report".to_string(),
    include_charts: true,
    include_raw_data: false,
    include_metadata: true,
    chart_width: 800,
    chart_height: 400,
    theme: ReportTheme::Modern,
};

// Generate report
let generator = ReportGenerator::with_config(config);
let html_bytes = generator
    .generate_report("extraction-123", ReportFormat::Html)
    .await?;

// Save to file
std::fs::write("report.html", html_bytes)?;
```

### Report Formats

```rust
// HTML with charts and visualizations
let html = generator.generate_report(id, ReportFormat::Html).await?;

// JSON for API responses
let json = generator.generate_report(id, ReportFormat::Json).await?;

// CSV for data analysis
let csv = generator.generate_report(id, ReportFormat::Csv).await?;

// PDF for distribution (requires additional dependencies)
let pdf = generator.generate_report(id, ReportFormat::Pdf).await?;
```

### Report Themes

Available themes:
- `ReportTheme::Light` - Clean light theme
- `ReportTheme::Dark` - Dark mode for reduced eye strain
- `ReportTheme::Corporate` - Professional corporate styling
- `ReportTheme::Modern` - Contemporary design with gradients

### Custom Report Data

```rust
use riptide_streaming::reports::{
    ReportData,
    DomainStats,
    TimelineEntry,
    WordFrequency,
};
use std::collections::HashMap;

let report_data = ReportData {
    extraction_id: "extraction-123".to_string(),
    title: "Custom Report".to_string(),
    total_items: 150,
    total_words: 45000,
    total_links: 1200,
    total_images: 300,
    start_time: chrono::Utc::now(),
    end_time: chrono::Utc::now(),
    duration_seconds: 120,
    average_extraction_time_ms: 800,
    domain_stats: HashMap::new(),
    timeline: Vec::new(),
    word_frequencies: Vec::new(),
};

// Generate from custom data
let generator = ReportGenerator::new();
let html = generator.generate_from_data(&report_data, ReportFormat::Html)?;
```

## CLI Tool Usage

The `riptide-streaming` crate includes a comprehensive CLI tool.

### Installation

```bash
cargo install riptide-streaming --features cli
```

### Extract Content

```bash
# Extract from URLs
riptide-cli extract https://example.com https://example.org

# Stream results in real-time
riptide-cli extract --stream https://example.com

# Save to file
riptide-cli extract -o results.ndjson https://example.com

# Control concurrency
riptide-cli extract --concurrency 10 --timeout 30 url1 url2 url3
```

### Stream Monitoring

```bash
# Monitor active stream
riptide-cli stream <extraction-id>

# Follow mode (keep streaming)
riptide-cli stream --follow <extraction-id>

# Different formats
riptide-cli stream --format ndjson <extraction-id>
riptide-cli stream --format json <extraction-id>
riptide-cli stream --format raw <extraction-id>
```

### Generate Reports

```bash
# Generate HTML report
riptide-cli report <extraction-id> --format html -o report.html

# Include charts and visualizations
riptide-cli report <extraction-id> --charts --format html

# Generate CSV for analysis
riptide-cli report <extraction-id> --format csv -o data.csv

# Include raw data
riptide-cli report <extraction-id> --raw-data --format json
```

### Configuration Management

```bash
# Show current configuration
riptide-cli config show

# Validate configuration
riptide-cli config validate

# Reset to defaults
riptide-cli config reset

# Set configuration value
riptide-cli config set key value

# Get configuration value
riptide-cli config get key
```

### Server Management

```bash
# Start streaming server
riptide-cli server start --host 0.0.0.0 --port 8080

# Check server status
riptide-cli server status

# View server logs
riptide-cli server logs --follow
```

### Tool Management

```bash
# List available tools
riptide-cli tools list

# Show tool information
riptide-cli tools info <tool-id>

# Export as Postman collection
riptide-cli tools export -o postman_collection.json
```

## Configuration

### StreamingCoordinator Configuration

```rust
use riptide_streaming::config::StreamingConfig;

let config = StreamingConfig {
    max_concurrent_streams: 100,
    default_buffer_size: 1000,
    enable_compression: false,
    heartbeat_interval_secs: 30,
};
```

### Backpressure Configuration

```rust
use riptide_streaming::config::BackpressureConfig;
use std::time::Duration;

let config = BackpressureConfig {
    max_in_flight: 1000,              // Max items in flight per stream
    max_memory_bytes: 100_000_000,    // 100 MB memory limit
    max_total_items: 10000,           // Max total items across all streams
    activation_threshold: 0.8,        // Activate backpressure at 80%
    recovery_threshold: 0.6,          // Recover at 60%
    check_interval: Duration::from_millis(500),
    adaptive: true,                   // Enable adaptive throttling
};
```

### Report Configuration

```rust
use riptide_streaming::reports::{ReportConfig, ReportTheme};

let config = ReportConfig {
    title: "Extraction Report".to_string(),
    include_charts: true,
    include_raw_data: false,
    include_metadata: true,
    chart_width: 800,
    chart_height: 400,
    theme: ReportTheme::Modern,
};
```

### Configuration File

Create a `riptide-streaming.toml` configuration file:

```toml
[streaming]
max_concurrent_streams = 100
default_buffer_size = 1000
enable_compression = false
heartbeat_interval_secs = 30

[backpressure]
max_in_flight = 1000
max_memory_bytes = 104857600  # 100 MB
max_total_items = 10000
activation_threshold = 0.8
recovery_threshold = 0.6
check_interval_ms = 500
adaptive = true

[reports]
title = "RipTide Extraction Report"
include_charts = true
include_raw_data = false
include_metadata = true
chart_width = 800
chart_height = 400
theme = "modern"
```

Load configuration:

```rust
use riptide_streaming::config::ConfigManager;

let config_manager = ConfigManager::load_from_file("riptide-streaming.toml")?;
let config = config_manager.config();
```

## Performance Considerations

### Memory Management

1. **Buffer Sizing**: Adjust buffer size based on your needs
   - Larger buffers: Better throughput, higher memory usage
   - Smaller buffers: Lower memory usage, more frequent I/O

2. **Backpressure Limits**: Set appropriate limits
   ```rust
   let config = BackpressureConfig {
       max_memory_bytes: calculate_safe_limit(),
       max_in_flight: compute_optimal_concurrency(),
       ..Default::default()
   };
   ```

3. **Stream Cleanup**: Ensure streams are properly closed
   ```rust
   coordinator.complete_stream(stream_id).await?;
   controller.unregister_stream(stream_id).await;
   ```

### Throughput Optimization

1. **Disable Progress Updates**: For maximum throughput
   ```rust
   let config = NdjsonConfig {
       include_progress: false,
       include_heartbeat: false,
       ..Default::default()
   };
   ```

2. **Compression**: Enable for network-constrained scenarios
   ```rust
   let config = NdjsonConfig {
       compression_enabled: true,
       ..Default::default()
   };
   ```

3. **Batch Processing**: Process items in batches when possible
   ```rust
   for batch in extraction_results.chunks(100) {
       process_batch(batch).await?;
   }
   ```

### Monitoring

Monitor stream health with metrics:

```rust
let metrics = controller.get_metrics().await;

println!("Active streams: {}", metrics.total_streams);
println!("Items in flight: {}", metrics.total_in_flight);
println!("Memory usage: {} bytes", metrics.total_memory_usage);
println!("Status: {:?}", metrics.status);
println!("Rejection rate: {:.2}%", metrics.rejection_rate * 100.0);
```

## Testing

### Unit Tests

```bash
cargo test
```

### Integration Tests

```bash
cargo test --test riptide_streaming_integration_tests
```

### Minimal Compile Test

```bash
cargo test --test minimal_compile_test
```

### With All Features

```bash
cargo test --all-features
```

### Example Test

```rust
#[tokio::test]
async fn test_streaming_coordinator() {
    let mut coordinator = StreamingCoordinator::new();

    let stream_id = coordinator
        .start_stream("test-extraction".to_string())
        .await
        .unwrap();

    coordinator
        .update_progress(stream_id, 50, Some(100))
        .await
        .unwrap();

    let stream_info = coordinator.get_stream(&stream_id).unwrap();
    assert_eq!(stream_info.processed_items, 50);

    coordinator.complete_stream(stream_id).await.unwrap();
}
```

## API Reference

### Core Types

- `StreamingCoordinator` - Central coordinator for stream management
- `ExtractionResult` - Result from an extraction operation
- `StreamInfo` - Information about an active stream
- `StreamStatus` - Status enumeration (Active, Paused, Completed, Failed)
- `ProgressUpdate` - Progress update with rate and ETA

### NDJSON Types

- `NdjsonStream` - NDJSON stream producer
- `NdjsonStreamBuilder` - Builder for creating NDJSON streams
- `NdjsonCodec` - Encoder/decoder for NDJSON items
- `NdjsonItem` - Stream item (Result, Progress, Event, Metadata, Heartbeat)
- `NdjsonConfig` - Configuration for NDJSON streaming

### Backpressure Types

- `BackpressureController` - Resource management controller
- `BackpressureConfig` - Backpressure configuration
- `BackpressurePermit` - Resource acquisition permit
- `BackpressureMetrics` - Current backpressure metrics
- `BackpressureStatus` - Status (Normal, Warning, Critical, Throttled)

### Report Types

- `ReportGenerator` - Report generation engine
- `ReportConfig` - Report configuration
- `ReportFormat` - Output format (Html, Json, Csv, Pdf)
- `ReportTheme` - Visual theme (Light, Dark, Corporate, Modern)
- `ReportData` - Report data structure
- `DomainStats` - Statistics per domain
- `TimelineEntry` - Timeline event
- `WordFrequency` - Word frequency data

## Error Handling

The crate uses `StreamingError` for all streaming-related errors:

```rust
use riptide_streaming::StreamingError;

match operation().await {
    Ok(result) => println!("Success: {:?}", result),
    Err(StreamingError::StreamNotFound(id)) => {
        eprintln!("Stream not found: {}", id);
    }
    Err(StreamingError::BackpressureExceeded) => {
        eprintln!("Backpressure limit exceeded, retry later");
    }
    Err(StreamingError::ReportGenerationFailed(msg)) => {
        eprintln!("Report generation failed: {}", msg);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Dependencies

Key dependencies:
- `tokio` - Async runtime
- `axum` - HTTP server framework
- `serde` - Serialization
- `futures` - Stream utilities
- `handlebars` - Template engine
- `plotters` - Chart generation
- `clap` - CLI argument parsing
- `indicatif` - Progress bars

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE) for details.

## Related Crates

- `riptide-core` - Core extraction functionality
- `riptide-api` - HTTP API server
- `riptide-extraction` - HTML parsing utilities

## Examples

See the `examples/` directory for more usage examples:
- `basic_streaming.rs` - Basic NDJSON streaming
- `backpressure_demo.rs` - Backpressure control demonstration
- `report_generation.rs` - Report generation examples
- `cli_usage.sh` - CLI usage examples

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history and changes.

## Support

- GitHub Issues: [Report bugs or request features](https://github.com/yourusername/riptide/issues)
- Documentation: [Full API documentation](https://docs.rs/riptide-streaming)
- Examples: See the `examples/` directory

---

**Note**: This crate is part of the RipTide project, an event-driven data extraction framework.
