# Riptide Performance Configuration

## Configuration Files

This directory contains environment-specific configuration files for the riptide-performance monitoring system.

### Available Configurations

- **production.toml** - Production-optimized settings with minimal overhead
- **development.toml** - Development settings with verbose debugging and profiling

## Configuration Sections

### Memory Profiling

Controls memory allocation tracking, leak detection, and profiling behavior:

- `enabled` - Enable/disable memory profiling
- `sampling_interval_secs` - How often to sample memory statistics
- `max_samples` - Maximum number of samples to retain
- `track_allocations` - Track individual allocations
- `detect_leaks` - Enable memory leak detection
- `generate_flamegraphs` - Generate flamegraph visualizations

### Thresholds

Memory usage alert thresholds in megabytes:

- `warning_threshold_mb` - Warning level
- `alert_threshold_mb` - Alert level requiring attention
- `critical_threshold_mb` - Critical level requiring immediate action

### Telemetry

OpenTelemetry export configuration:

- `export_enabled` - Enable telemetry export
- `export_interval_secs` - Export frequency
- `otlp_endpoint` - OTLP collector endpoint

### Alerts

Alert notification configuration:

- `enabled` - Enable/disable alerts
- `notification_channels` - Where to send alerts (log, otlp, stdout)
- `alert_cooldown_secs` - Minimum time between duplicate alerts

### Features

Feature flags matching Cargo.toml features:

- `jemalloc` - Use jemalloc allocator
- `memory_profiling` - Enable memory profiling features
- `bottleneck_analysis` - Enable bottleneck detection
- `cache_optimization` - Enable cache optimization
- `resource_limits` - Enable resource limiting

### Bottleneck Analysis

Performance bottleneck detection settings:

- `enabled` - Enable bottleneck analysis
- `analysis_interval_secs` - Analysis frequency
- `track_async_tasks` - Monitor async task performance
- `track_lock_contention` - Monitor lock contention
- `flamegraph_on_demand` - Generate flamegraphs on demand

### Cache Optimization

Cache performance monitoring:

- `enabled` - Enable cache monitoring
- `max_cache_size_mb` - Maximum cache size
- `eviction_policy` - Cache eviction strategy
- `track_hit_rate` - Monitor cache hit rates

### Resource Limits

Resource limiting and rate control:

- `enabled` - Enable resource limits
- `max_concurrent_requests` - Maximum concurrent requests
- `rate_limit_per_second` - Requests per second limit
- `burst_size` - Burst capacity

## Usage

Load configuration in your application:

```rust
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct PerformanceConfig {
    memory_profiling: MemoryProfilingConfig,
    thresholds: ThresholdsConfig,
    telemetry: TelemetryConfig,
    alerts: AlertsConfig,
    features: FeaturesConfig,
}

// Load based on environment
let config_path = if cfg!(debug_assertions) {
    "config/development.toml"
} else {
    "config/production.toml"
};

let config_str = fs::read_to_string(config_path)?;
let config: PerformanceConfig = toml::from_str(&config_str)?;
```

## Environment Variables

Override configuration values using environment variables:

- `RIPTIDE_PERF_SAMPLING_INTERVAL` - Override sampling interval
- `RIPTIDE_PERF_WARNING_THRESHOLD` - Override warning threshold
- `RIPTIDE_PERF_OTLP_ENDPOINT` - Override OTLP endpoint
- `RIPTIDE_PERF_ALERTS_ENABLED` - Enable/disable alerts

## Performance Impact

### Production Configuration

- **CPU Overhead**: ~1-2% with 30s sampling interval
- **Memory Overhead**: ~50-100MB for profiling data
- **I/O Impact**: Minimal with 60s export interval

### Development Configuration

- **CPU Overhead**: ~5-10% with 5s sampling and flamegraphs
- **Memory Overhead**: ~200-300MB with detailed tracking
- **I/O Impact**: Moderate with 10s exports and verbose logging

## Best Practices

1. **Start with production.toml** - Use conservative settings initially
2. **Monitor overhead** - Track the performance impact of profiling itself
3. **Tune sampling intervals** - Balance detail vs overhead
4. **Use flamegraphs sparingly** - Enable on-demand in production
5. **Set appropriate thresholds** - Based on your workload characteristics
6. **Test in staging** - Validate configuration before production deployment

## Troubleshooting

### High CPU Usage

- Increase `sampling_interval_secs`
- Disable `generate_flamegraphs`
- Set `track_allocations = false`

### High Memory Usage

- Reduce `max_samples`
- Decrease `max_cache_size_mb`
- Disable `track_all_allocations` in debug mode

### Missing Metrics

- Check `telemetry.export_enabled = true`
- Verify `otlp_endpoint` is accessible
- Ensure features are enabled in Cargo.toml

## See Also

- [Cargo.toml Feature Flags](/workspaces/eventmesh/crates/riptide-performance/Cargo.toml)
- [Performance Module Documentation](/workspaces/eventmesh/crates/riptide-performance/src/lib.rs)
