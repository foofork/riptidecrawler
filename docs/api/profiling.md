# Performance Profiling API

This document describes the performance profiling endpoints provided by the riptide-api service. These endpoints enable real-time monitoring of memory usage, CPU performance, bottleneck detection, and memory leak analysis.

## Overview

The profiling system is designed with minimal overhead (<2% in production) and provides comprehensive insights into application performance. It leverages jemalloc for accurate memory tracking and includes automated leak detection.

### Key Features

- **Real-time Memory Profiling**: Track RSS, heap, and virtual memory usage
- **CPU Monitoring**: Monitor CPU usage and load averages
- **Bottleneck Detection**: Identify performance hotspots with impact scores
- **Memory Leak Analysis**: Automated detection of suspicious allocation patterns
- **Heap Snapshots**: On-demand memory snapshots for deep analysis
- **Low Overhead**: <2% performance impact with jemalloc

## Authentication

All profiling endpoints require authentication. Include your API key in the request headers:

```
X-API-Key: your-api-key-here
```

## Endpoints

### GET /api/profiling/memory

Get current memory usage metrics including RSS, heap, and virtual memory.

**Response**:
```json
{
  "timestamp": "2025-10-10T18:00:00Z",
  "rss_mb": 245.3,
  "heap_mb": 189.7,
  "virtual_mb": 512.1,
  "resident_mb": 245.3,
  "shared_mb": 12.4,
  "growth_rate_mb_per_sec": 0.15,
  "threshold_status": "normal",
  "warnings": []
}
```

**Threshold Status Values**:
- `normal`: Memory usage within normal bounds (<650MB)
- `warning`: Memory usage approaching limit (650-700MB)
- `critical`: Memory usage exceeds limit (>700MB)

**Example**:
```bash
curl -H "X-API-Key: your-key" http://localhost:8080/api/profiling/memory
```

---

### GET /api/profiling/cpu

Get CPU usage metrics and load averages.

**Note**: Full CPU profiling is only available in dev builds with the `profiling-full` feature.

**Response**:
```json
{
  "timestamp": "2025-10-10T18:00:00Z",
  "cpu_usage_percent": 23.5,
  "user_time_percent": 18.2,
  "system_time_percent": 5.3,
  "idle_time_percent": 76.5,
  "load_average": {
    "one_min": 0.45,
    "five_min": 0.38,
    "fifteen_min": 0.32
  },
  "available": true,
  "note": "CPU profiling is simplified. Enable 'profiling-full' feature for detailed CPU profiling."
}
```

**Example**:
```bash
curl -H "X-API-Key: your-key" http://localhost:8080/api/profiling/cpu
```

---

### GET /api/profiling/bottlenecks

Get detected performance bottlenecks and hotspots.

**Response**:
```json
{
  "timestamp": "2025-10-10T18:00:00Z",
  "analysis_duration_ms": 125,
  "hotspots": [
    {
      "function_name": "riptide_core::spider::crawl",
      "file_location": "crates/riptide-core/src/spider/core.rs",
      "line_number": 45,
      "cpu_time_percent": 25.3,
      "wall_time_percent": 30.1,
      "call_count": 1547,
      "average_duration_us": 850,
      "impact_score": 0.85
    }
  ],
  "total_samples": 1000,
  "cpu_bound_percent": 60.0,
  "io_bound_percent": 25.0,
  "memory_bound_percent": 15.0,
  "recommendations": [
    "Critical: Optimize riptide_core::spider::crawl (25.3% CPU time, impact score: 0.85)",
    "Consider optimizing riptide_html::parse_document (18.7% CPU time)"
  ]
}
```

**Impact Score**: Ranges from 0.0 to 1.0, where 1.0 indicates the most significant performance impact.

**Example**:
```bash
curl -H "X-API-Key: your-key" http://localhost:8080/api/profiling/bottlenecks
```

---

### GET /api/profiling/allocations

Get allocation pattern analysis and memory efficiency metrics.

**Response**:
```json
{
  "timestamp": "2025-10-10T18:00:00Z",
  "top_allocators": [
    ["riptide_html::parse_document", 45678912],
    ["tokio::task::spawn", 23456789],
    ["riptide_core::cache::insert", 12345678]
  ],
  "size_distribution": {
    "small_0_1kb": 4521,
    "medium_1_100kb": 892,
    "large_100kb_1mb": 45,
    "huge_1mb_plus": 12
  },
  "efficiency_score": 0.87,
  "fragmentation_percent": 8.3,
  "recommendations": [
    "Consider implementing memory pooling for frequent small allocations",
    "Large allocations detected in riptide_html::parse_document"
  ]
}
```

**Efficiency Score**: Ranges from 0.0 to 1.0, where 1.0 indicates optimal memory efficiency.

**Example**:
```bash
curl -H "X-API-Key: your-key" http://localhost:8080/api/profiling/allocations
```

---

### POST /api/profiling/leak-detection

Trigger memory leak analysis to detect potential memory leaks.

**Response**:
```json
{
  "timestamp": "2025-10-10T18:00:00Z",
  "analysis_duration_ms": 450,
  "potential_leaks": [
    {
      "component": "riptide_html::cache",
      "allocation_count": 2341,
      "total_size_bytes": 52428800,
      "average_size_bytes": 22400.0,
      "growth_rate_mb_per_hour": 12.5,
      "severity": "high",
      "first_seen": "2025-10-10T17:00:00Z",
      "last_seen": "2025-10-10T18:00:00Z"
    }
  ],
  "growth_rate_mb_per_hour": 12.5,
  "highest_risk_component": "riptide_html::cache",
  "suspicious_patterns": [
    "riptide_html::cache: Exponential allocation growth detected"
  ],
  "recommendations": [
    "Investigate riptide_html::cache for potential memory leak",
    "Implement cache size limits and eviction policies"
  ]
}
```

**Severity Levels**:
- `low`: Minor growth, monitoring recommended
- `medium`: Moderate growth, investigation advised
- `high`: Significant growth, immediate action required
- `critical`: Severe growth, urgent intervention needed

**Example**:
```bash
curl -X POST -H "X-API-Key: your-key" http://localhost:8080/api/profiling/leak-detection
```

---

### POST /api/profiling/snapshot

Trigger a heap snapshot for offline analysis.

**Response**:
```json
{
  "timestamp": "2025-10-10T18:00:00Z",
  "snapshot_id": "snapshot_1728583200",
  "file_path": "/tmp/riptide_heap_snapshot_1728583200.json",
  "size_bytes": 15728640,
  "status": "completed",
  "download_url": "/api/profiling/snapshot/snapshot_1728583200/download"
}
```

**Snapshot Contents**: The snapshot includes current memory state, metrics, and allocation patterns for deep analysis.

**Example**:
```bash
curl -X POST -H "X-API-Key: your-key" http://localhost:8080/api/profiling/snapshot
```

---

## Configuration

### Environment Variables

Configure profiling behavior with these environment variables:

```bash
# Enable profiling (default: true)
ENABLE_PROFILING=true

# Memory sampling interval in seconds (default: 5)
PROFILING_MEMORY_INTERVAL_SECS=5

# Leak check interval in seconds (default: 300)
PROFILING_LEAK_CHECK_INTERVAL_SECS=300

# Memory warning threshold in MB (default: 650)
PROFILING_MEMORY_WARNING_MB=650

# Memory alert threshold in MB (default: 700)
PROFILING_MEMORY_ALERT_MB=700

# jemalloc configuration
MALLOC_CONF="background_thread:true,narenas:4,dirty_decay_ms:10000"
```

### Feature Flags

Enable advanced profiling features at compile time:

```toml
# Production mode (default)
[features]
default = ["jemalloc"]

# Development mode with full profiling
profiling-full = ["jemalloc", "riptide-performance/bottleneck-analysis-full"]
```

**Build Commands**:
```bash
# Production build (standard profiling)
cargo build --release --features jemalloc

# Development build (full profiling with flamegraphs)
cargo build --release --features profiling-full
```

---

## Performance Targets

The profiling system maintains these performance targets:

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| P50 Latency | ≤1.5s | >2.0s |
| P95 Latency | ≤5.0s | >7.0s |
| Memory RSS | ≤600MB | >650MB (warning), >700MB (critical) |
| Throughput | ≥70 PPS | <50 PPS |
| AI Overhead | ≤30% | >40% |
| Profiling Overhead | <2% | >5% |

---

## Rate Limiting

Profiling endpoints are subject to standard rate limiting:
- **Memory/CPU/Allocations**: 100 requests/minute
- **Leak Detection**: 10 requests/minute (more expensive operation)
- **Snapshots**: 5 requests/minute (disk I/O intensive)

Exceeding rate limits returns HTTP 429 with retry-after header.

---

## Best Practices

### 1. Regular Monitoring

Poll memory metrics every 30-60 seconds for continuous monitoring:

```bash
# Monitor memory every 30 seconds
watch -n 30 'curl -s -H "X-API-Key: your-key" http://localhost:8080/api/profiling/memory | jq'
```

### 2. Leak Detection Schedule

Run leak detection hourly or when memory alerts trigger:

```bash
# Hourly leak detection (cron)
0 * * * * curl -X POST -H "X-API-Key: your-key" http://localhost:8080/api/profiling/leak-detection
```

### 3. Snapshot Strategy

Take snapshots when investigating specific issues:

- Before/after significant operations
- When memory alerts trigger
- During load testing
- For performance regression analysis

### 4. Bottleneck Analysis

Review bottlenecks weekly or after deployments:

```bash
# Weekly bottleneck analysis
curl -s -H "X-API-Key: your-key" http://localhost:8080/api/profiling/bottlenecks | \
  jq '.hotspots | sort_by(.impact_score) | reverse | .[0:5]'
```

### 5. Dashboard Integration

Integrate profiling metrics with monitoring dashboards:

- **Prometheus**: Scrape `/metrics` endpoint for profiling metrics
- **Grafana**: Create dashboards for memory trends, leak detection
- **Alertmanager**: Configure alerts based on thresholds

---

## Troubleshooting

### High Memory Usage

If memory usage exceeds thresholds:

1. **Check for leaks**:
   ```bash
   curl -X POST -H "X-API-Key: your-key" http://localhost:8080/api/profiling/leak-detection
   ```

2. **Review allocations**:
   ```bash
   curl -H "X-API-Key: your-key" http://localhost:8080/api/profiling/allocations
   ```

3. **Take snapshot for analysis**:
   ```bash
   curl -X POST -H "X-API-Key: your-key" http://localhost:8080/api/profiling/snapshot
   ```

### False Positive Leaks

If leak detection reports false positives:

1. **Check growth rate**: Ensure growth is sustained over time
2. **Review patterns**: Look for suspicious patterns in the report
3. **Adjust thresholds**: Tune leak detection thresholds if needed

### Performance Degradation

If profiling impacts performance:

1. **Reduce sampling frequency**: Increase `PROFILING_MEMORY_INTERVAL_SECS`
2. **Disable non-essential features**: Remove `profiling-full` feature
3. **Check resource constraints**: Review overall system load

---

## Integration Examples

### Prometheus Metrics

Profiling data is automatically exported to Prometheus:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'riptide-api'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
```

### Grafana Dashboard

Example Grafana queries:

```promql
# Memory usage over time
riptide_memory_rss_bytes / 1024 / 1024

# Memory growth rate
rate(riptide_memory_rss_bytes[5m]) * 3600 / 1024 / 1024

# Cache hit rate
riptide_cache_hit_rate * 100
```

### Alerting Rules

Example Alertmanager rules:

```yaml
groups:
  - name: riptide_profiling
    rules:
      - alert: HighMemoryUsage
        expr: riptide_memory_rss_bytes > 650 * 1024 * 1024
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage detected"

      - alert: MemoryLeak
        expr: rate(riptide_memory_rss_bytes[1h]) > 10 * 1024 * 1024
        for: 30m
        labels:
          severity: critical
        annotations:
          summary: "Potential memory leak detected"
```

---

## API Versioning

Current API version: **v1.1**

Profiling endpoints follow semantic versioning:
- **v1.0**: Initial memory and CPU profiling
- **v1.1**: Added leak detection and snapshots (current)
- **v1.2**: Planned flamegraph generation (upcoming)

---

## Security Considerations

### Authentication Required

All profiling endpoints require valid API key authentication. Ensure keys are:
- Rotated regularly
- Stored securely (environment variables, secrets management)
- Limited to appropriate access levels

### Rate Limiting

Profiling endpoints are rate-limited to prevent abuse:
- Leak detection and snapshots have stricter limits
- Contact support to adjust limits if needed

### Data Privacy

Profiling data may contain sensitive information:
- Function names and file paths
- Allocation patterns
- Memory addresses (in snapshots)

Ensure appropriate access controls and data retention policies.

---

## Support

For issues or questions:
- **GitHub Issues**: [github.com/riptide/issues](https://github.com/riptide/issues)
- **Documentation**: [docs.riptide.io](https://docs.riptide.io)
- **Email**: support@riptide.io

---

## Changelog

### v1.1.0 (2025-10-10)
- Added memory leak detection endpoint
- Added heap snapshot endpoint
- Enhanced allocation analysis
- Improved bottleneck detection accuracy

### v1.0.0 (2025-09-15)
- Initial profiling API release
- Memory and CPU monitoring
- Bottleneck detection
- Allocation tracking
