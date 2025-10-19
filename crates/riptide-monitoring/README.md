# RipTide Monitoring

Monitoring, telemetry, and metrics collection for the RipTide web scraping framework.

## Overview

`riptide-monitoring` provides comprehensive observability for RipTide, including Prometheus metrics, health checks, performance analysis, and bottleneck detection for production deployments.

## Features

- **Prometheus Metrics**: Standard and custom metrics with labels
- **Health Checks**: Component-level health monitoring
- **Performance Analysis**: Phase-by-phase timing and bottleneck detection
- **Resource Tracking**: Memory, CPU, and connection pool monitoring
- **Alert Management**: Threshold-based alerting system
- **Metrics Collection**: Automatic metric aggregation and reporting
- **Dashboard Support**: Grafana-compatible metrics export
- **Custom Collectors**: Extensible collector system

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│               RipTide Monitoring                         │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │  Metrics    │  │   Health    │  │Performance  │     │
│  │ Collector   │  │   Checker   │  │  Analyzer   │     │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘     │
│         │                │                 │            │
│         └────────────────┼─────────────────┘            │
│                          ▼                              │
│                  ┌───────────────┐                      │
│                  │  Prometheus   │                      │
│                  │   Registry    │                      │
│                  └───────┬───────┘                      │
└────────────────────────────┼─────────────────────────────┘
                            │
                            ▼
                    ┌──────────────┐
                    │  Prometheus  │
                    │    Server    │
                    └──────────────┘
```

## Usage

### Basic Metrics Collection

```rust
use riptide_monitoring::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize monitoring
    let monitoring = Monitoring::new();

    // Record metrics
    monitoring.record_request_duration(
        "extract",
        Duration::from_millis(250),
    );

    monitoring.increment_counter("requests_total", &[
        ("endpoint", "extract"),
        ("status", "success"),
    ]);

    // Export metrics (Prometheus format)
    let metrics = monitoring.export_metrics().await?;
    println!("{}", metrics);

    Ok(())
}
```

### Health Checks

```rust
use riptide_monitoring::*;

let health = HealthChecker::new();

// Register health checks
health.register("redis", Box::new(|| async {
    redis_client.ping().await.is_ok()
}));

health.register("browser_pool", Box::new(|| async {
    browser_pool.is_healthy().await
}));

// Check overall health
let status = health.check_all().await?;
println!("Healthy: {}", status.is_healthy());

// Check specific component
let redis_healthy = health.check("redis").await?;
```

### Performance Analysis

```rust
use riptide_monitoring::*;

let analyzer = PerformanceAnalyzer::new();

// Record phase timings
analyzer.record_phase("fetch", Duration::from_millis(150));
analyzer.record_phase("gate", Duration::from_millis(10));
analyzer.record_phase("extract", Duration::from_millis(200));

// Get performance report
let report = analyzer.generate_report().await?;
println!("Slowest phase: {}", report.bottleneck);
println!("Total time: {}ms", report.total_ms);

// Get recommendations
for recommendation in report.recommendations {
    println!("💡 {}", recommendation);
}
```

### Custom Metrics

```rust
use riptide_monitoring::*;

// Counter
let counter = monitoring.register_counter(
    "custom_events_total",
    "Total custom events processed",
)?;
counter.inc();
counter.inc_by(5);

// Gauge
let gauge = monitoring.register_gauge(
    "active_connections",
    "Number of active connections",
)?;
gauge.set(42);
gauge.inc();
gauge.dec();

// Histogram
let histogram = monitoring.register_histogram(
    "request_duration_seconds",
    "Request duration in seconds",
)?;
histogram.observe(0.250);
histogram.observe(0.180);

// Summary
let summary = monitoring.register_summary(
    "response_size_bytes",
    "Response size in bytes",
)?;
summary.observe(15240);
```

## Prometheus Metrics

### Available Metrics

**Request Metrics:**
- `http_requests_total` - Total HTTP requests (counter)
- `http_request_duration_seconds` - Request duration (histogram)
- `http_request_size_bytes` - Request size (summary)
- `http_response_size_bytes` - Response size (summary)

**Resource Metrics:**
- `browser_pool_size` - Browser pool size (gauge)
- `browser_pool_idle` - Idle browsers (gauge)
- `memory_allocated_bytes` - Allocated memory (gauge)
- `memory_pressure_ratio` - Memory pressure (gauge)

**Cache Metrics:**
- `cache_hits_total` - Cache hits (counter)
- `cache_misses_total` - Cache misses (counter)
- `cache_size_bytes` - Cache size (gauge)
- `cache_evictions_total` - Cache evictions (counter)

**Worker Metrics:**
- `worker_jobs_pending` - Pending jobs (gauge)
- `worker_jobs_completed_total` - Completed jobs (counter)
- `worker_jobs_failed_total` - Failed jobs (counter)
- `worker_job_duration_seconds` - Job duration (histogram)

**Rate Limit Metrics:**
- `rate_limit_permits_available` - Available permits (gauge)
- `rate_limit_delays_total` - Rate limit delays (counter)
- `rate_limit_wait_duration_seconds` - Wait duration (histogram)

### Metrics Endpoint

```rust
use riptide_monitoring::*;
use axum::{Router, routing::get};

let monitoring = Monitoring::new();

let app = Router::new()
    .route("/metrics", get(move || async move {
        monitoring.export_prometheus()
    }));

// Access metrics at http://localhost:8080/metrics
```

### Grafana Dashboard

Import the provided Grafana dashboard for visualization:

```bash
# Import dashboard
curl -X POST http://grafana:3000/api/dashboards/db \
  -H "Content-Type: application/json" \
  -d @dashboards/riptide-monitoring.json
```

## Health Monitoring

### Health Score Calculation

```rust
use riptide_monitoring::*;

let health = HealthChecker::new();

// Get numeric health score (0-100)
let score = health.health_score().await?;
println!("Health Score: {}/100", score);

// Factors affecting score:
// - Component availability (40%)
// - Error rate (30%)
// - Resource utilization (20%)
// - Response time (10%)
```

### Component Health

```rust
use riptide_monitoring::*;

let health_status = health.check_all().await?;

for component in health_status.components {
    println!(
        "{}: {} ({}ms)",
        component.name,
        if component.healthy { "✓" } else { "✗" },
        component.check_duration_ms
    );
}

// Detailed component status
let redis_details = health.check_detailed("redis").await?;
println!("Redis version: {}", redis_details.metadata["version"]);
println!("Redis memory: {} MB", redis_details.metadata["memory_mb"]);
```

## Performance Monitoring

### Phase Analysis

```rust
use riptide_monitoring::*;

let analyzer = PerformanceAnalyzer::new();

// Record pipeline phases
analyzer.start_phase("fetch");
// ... fetch operation ...
analyzer.end_phase("fetch");

analyzer.start_phase("extract");
// ... extraction operation ...
analyzer.end_phase("extract");

// Get phase breakdown
let phases = analyzer.get_phases().await?;
for phase in phases {
    println!(
        "{}: {}ms ({:.1}%)",
        phase.name,
        phase.avg_duration_ms,
        phase.percentage_of_total
    );
}
```

### Bottleneck Detection

```rust
use riptide_monitoring::*;

let bottlenecks = analyzer.detect_bottlenecks().await?;

for bottleneck in bottlenecks {
    println!("⚠️  Bottleneck: {}", bottleneck.phase);
    println!("   Severity: {:?}", bottleneck.severity);
    println!("   Impact: {}ms", bottleneck.impact_ms);
    println!("   Recommendation: {}", bottleneck.recommendation);
}
```

## Alert Management

### Defining Alerts

```rust
use riptide_monitoring::*;

let alerts = AlertManager::new();

// Define alert rules
alerts.add_rule(AlertRule {
    name: "high_error_rate".to_string(),
    condition: Condition::Threshold {
        metric: "error_rate".to_string(),
        operator: Operator::GreaterThan,
        value: 0.05, // 5%
    },
    severity: Severity::Critical,
    message: "Error rate exceeds 5%".to_string(),
});

alerts.add_rule(AlertRule {
    name: "high_memory".to_string(),
    condition: Condition::Threshold {
        metric: "memory_pressure_ratio".to_string(),
        operator: Operator::GreaterThan,
        value: 0.85, // 85%
    },
    severity: Severity::Warning,
    message: "Memory pressure above 85%".to_string(),
});
```

### Alert Notifications

```rust
use riptide_monitoring::*;

// Check for active alerts
let active_alerts = alerts.check_all().await?;

for alert in active_alerts {
    println!("🚨 {}: {}", alert.severity, alert.message);

    // Send notification
    match alert.severity {
        Severity::Critical => {
            send_pagerduty_alert(&alert).await?;
        }
        Severity::Warning => {
            send_slack_notification(&alert).await?;
        }
        _ => {}
    }
}
```

## Configuration

### Environment Variables

```bash
# Metrics
export METRICS_ENABLED=true
export METRICS_PORT=9090
export METRICS_PATH="/metrics"

# Health checks
export HEALTH_CHECK_INTERVAL=30
export HEALTH_CHECK_TIMEOUT=5

# Performance monitoring
export ENABLE_PERFORMANCE_ANALYSIS=true
export PERFORMANCE_SAMPLE_RATE=0.1

# Alerts
export ALERT_CHECK_INTERVAL=60
export ALERT_NOTIFICATION_URL="https://hooks.slack.com/..."
```

### Programmatic Configuration

```rust
use riptide_monitoring::*;

let config = MonitoringConfig {
    metrics: MetricsConfig {
        enabled: true,
        port: 9090,
        path: "/metrics".to_string(),
    },
    health: HealthConfig {
        check_interval: Duration::from_secs(30),
        timeout: Duration::from_secs(5),
    },
    performance: PerformanceConfig {
        enabled: true,
        sample_rate: 0.1,
    },
    alerts: AlertConfig {
        check_interval: Duration::from_secs(60),
        notification_url: Some("https://hooks.slack.com/...".to_string()),
    },
};

let monitoring = Monitoring::with_config(config)?;
```

## Integration with RipTide

This crate is used by:

- **riptide-api**: API server monitoring
- **riptide-core**: Core metrics collection
- **riptide-workers**: Worker job metrics
- **riptide-performance**: Memory profiling integration

## Prometheus Integration

### Scrape Configuration

Add to `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'riptide'
    scrape_interval: 15s
    static_configs:
      - targets: ['riptide-api:8080']
        labels:
          service: 'riptide-api'
```

### Alerting Rules

Create `alerts.yml`:

```yaml
groups:
  - name: riptide
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"

      - alert: HighMemoryPressure
        expr: memory_pressure_ratio > 0.85
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Memory pressure above 85%"
```

## Testing

```bash
# Run tests
cargo test -p riptide-monitoring

# Run with features
cargo test -p riptide-monitoring --features collector

# Integration tests
cargo test -p riptide-monitoring --test '*'
```

## License

Apache-2.0

## Related Crates

- **riptide-events**: Event emission and tracing
- **riptide-performance**: Memory profiling
- **riptide-api**: API metrics endpoint
