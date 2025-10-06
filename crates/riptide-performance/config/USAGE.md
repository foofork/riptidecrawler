# Production Configuration Usage Guide

## Quick Start

### Using Production Configuration

```bash
# Build with production features
cargo build --release --features production

# Run with production config
RIPTIDE_CONFIG_PATH=config/production.toml cargo run --release --features production
```

### Using Development Configuration

```bash
# Build with development features
cargo build --features development

# Run with development config
RIPTIDE_CONFIG_PATH=config/development.toml cargo run --features development
```

## Feature Flag Combinations

### Production Feature Set
```toml
[features]
production = [
    "jemalloc",
    "memory-profiling",
    "bottleneck-analysis",
    "cache-optimization",
    "resource-limits"
]
```

**Use when:**
- Deploying to production environments
- Need minimal overhead (1-2% CPU)
- Want conservative profiling with 30s sampling
- Require resource limits and rate control

### Development Feature Set
```toml
[features]
development = [
    "jemalloc",
    "memory-profiling",
    "bottleneck-analysis",
    "cache-optimization"
    # Note: resource-limits disabled for testing
]
```

**Use when:**
- Local development and debugging
- Want detailed profiling with 5s sampling
- Need flamegraph generation
- Don't want rate limits interfering with tests

## Loading Configuration

### Rust Code Example

```rust
use serde::Deserialize;
use std::fs;
use std::env;

#[derive(Debug, Deserialize)]
pub struct PerformanceConfig {
    pub memory_profiling: MemoryProfilingConfig,
    pub thresholds: ThresholdsConfig,
    pub telemetry: TelemetryConfig,
    pub alerts: AlertsConfig,
    pub features: FeaturesConfig,
    pub bottleneck_analysis: BottleneckAnalysisConfig,
    pub cache_optimization: CacheOptimizationConfig,
    pub resource_limits: ResourceLimitsConfig,
}

#[derive(Debug, Deserialize)]
pub struct MemoryProfilingConfig {
    pub enabled: bool,
    pub sampling_interval_secs: u64,
    pub max_samples: usize,
    pub track_allocations: bool,
    pub detect_leaks: bool,
    pub generate_flamegraphs: bool,
}

// ... other config structs ...

impl PerformanceConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // Check environment variable first
        let config_path = env::var("RIPTIDE_CONFIG_PATH")
            .unwrap_or_else(|_| {
                // Auto-detect based on build profile
                if cfg!(debug_assertions) {
                    "config/development.toml".to_string()
                } else {
                    "config/production.toml".to_string()
                }
            });

        let config_str = fs::read_to_string(&config_path)?;
        let config: PerformanceConfig = toml::from_str(&config_str)?;

        Ok(config)
    }

    pub fn with_env_overrides(mut self) -> Self {
        // Override with environment variables if present
        if let Ok(interval) = env::var("RIPTIDE_PERF_SAMPLING_INTERVAL") {
            if let Ok(val) = interval.parse() {
                self.memory_profiling.sampling_interval_secs = val;
            }
        }

        if let Ok(threshold) = env::var("RIPTIDE_PERF_WARNING_THRESHOLD") {
            if let Ok(val) = threshold.parse() {
                self.thresholds.warning_threshold_mb = val;
            }
        }

        if let Ok(endpoint) = env::var("RIPTIDE_PERF_OTLP_ENDPOINT") {
            self.telemetry.otlp_endpoint = endpoint;
        }

        if let Ok(enabled) = env::var("RIPTIDE_PERF_ALERTS_ENABLED") {
            self.alerts.enabled = enabled.parse().unwrap_or(true);
        }

        self
    }
}
```

### Integration Example

```rust
use riptide_performance::config::PerformanceConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration with environment overrides
    let config = PerformanceConfig::load()?.with_env_overrides();

    // Initialize profiler with config
    let profiler = if config.memory_profiling.enabled {
        Some(MemoryProfiler::new(config.memory_profiling)?)
    } else {
        None
    };

    // Initialize telemetry with config
    if config.telemetry.export_enabled {
        init_telemetry(&config.telemetry)?;
    }

    // Run application
    run_app(config, profiler).await?;

    Ok(())
}
```

## Environment Variable Overrides

Override any configuration value at runtime:

```bash
# Override sampling interval
export RIPTIDE_PERF_SAMPLING_INTERVAL=15

# Override warning threshold
export RIPTIDE_PERF_WARNING_THRESHOLD=500

# Override OTLP endpoint
export RIPTIDE_PERF_OTLP_ENDPOINT=http://otel-collector:4317

# Disable alerts
export RIPTIDE_PERF_ALERTS_ENABLED=false

# Run with overrides
cargo run --release --features production
```

## Docker Configuration

### Production Dockerfile

```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .

# Build with production features
RUN cargo build --release --features production

FROM debian:bookworm-slim

# Copy binary and config
COPY --from=builder /app/target/release/riptide /usr/local/bin/
COPY --from=builder /app/crates/riptide-performance/config/production.toml /etc/riptide/config.toml

# Set config path
ENV RIPTIDE_CONFIG_PATH=/etc/riptide/config.toml

# Set production OTLP endpoint
ENV RIPTIDE_PERF_OTLP_ENDPOINT=http://otel-collector:4317

CMD ["riptide"]
```

### Docker Compose

```yaml
version: '3.8'

services:
  riptide:
    build:
      context: .
      args:
        FEATURES: production
    environment:
      RIPTIDE_CONFIG_PATH: /etc/riptide/config.toml
      RIPTIDE_PERF_OTLP_ENDPOINT: http://otel-collector:4317
      RIPTIDE_PERF_WARNING_THRESHOLD: 650
    volumes:
      - ./config/production.toml:/etc/riptide/config.toml:ro
    depends_on:
      - otel-collector

  otel-collector:
    image: otel/opentelemetry-collector:latest
    ports:
      - "4317:4317"
      - "4318:4318"
    volumes:
      - ./otel-config.yaml:/etc/otel/config.yaml:ro
```

## Performance Tuning

### Low Overhead (< 1% CPU)
```toml
[memory_profiling]
sampling_interval_secs = 60
track_allocations = false
generate_flamegraphs = false

[telemetry]
export_interval_secs = 120
```

### Balanced (1-2% CPU)
```toml
[memory_profiling]
sampling_interval_secs = 30
track_allocations = true
generate_flamegraphs = false

[telemetry]
export_interval_secs = 60
```

### Detailed Profiling (5-10% CPU)
```toml
[memory_profiling]
sampling_interval_secs = 5
track_allocations = true
generate_flamegraphs = true

[telemetry]
export_interval_secs = 10
```

## Monitoring Setup

### OpenTelemetry Collector

```yaml
# otel-config.yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 10s
    send_batch_size: 1024

exporters:
  prometheus:
    endpoint: "0.0.0.0:8889"
  logging:
    loglevel: debug

service:
  pipelines:
    metrics:
      receivers: [otlp]
      processors: [batch]
      exporters: [prometheus, logging]
```

### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'riptide-performance'
    static_configs:
      - targets: ['otel-collector:8889']
```

## Troubleshooting

### Issue: High memory usage from profiler

**Solution:**
```toml
[memory_profiling]
max_samples = 1000  # Reduce from 2000
```

### Issue: Missing telemetry data

**Check:**
1. `telemetry.export_enabled = true`
2. OTLP endpoint is accessible
3. Network connectivity to collector

**Debug:**
```bash
# Test OTLP endpoint
curl http://localhost:4317

# Check logs
RUST_LOG=riptide_performance=debug cargo run
```

### Issue: Too many alerts

**Solution:**
```toml
[alerts]
alert_cooldown_secs = 600  # Increase from 300
notification_channels = ["log"]  # Reduce channels
```

## Best Practices

1. **Start conservative** - Use production.toml defaults initially
2. **Monitor overhead** - Track profiler's own resource usage
3. **Tune gradually** - Adjust one parameter at a time
4. **Use env vars** - For environment-specific overrides
5. **Test in staging** - Validate config before production
6. **Document changes** - Track tuning decisions
7. **Set alerts wisely** - Balance noise vs coverage

## See Also

- [Configuration Reference](README.md)
- [Cargo Feature Flags](../Cargo.toml)
- [Performance Profiling Documentation](../../docs/performance-monitoring.md)
