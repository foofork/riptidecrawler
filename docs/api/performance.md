# Performance Monitoring Architecture

## Overview

The RipTide EventMesh implements a comprehensive performance monitoring system with Prometheus metrics integration, real-time alerting, and distributed tracing capabilities. This architecture ensures production-grade observability and proactive performance management.

## Monitoring Architecture

### Core Components

#### 1. MetricsCollector
The central metrics collection system with OpenTelemetry integration:

```rust
pub struct MetricsCollector {
    telemetry: Option<Arc<TelemetrySystem>>,
    extraction_times: Arc<Mutex<TimeSeriesBuffer>>,
    request_rates: Arc<Mutex<TimeSeriesBuffer>>,
    memory_usage: Arc<Mutex<TimeSeriesBuffer>>,
    error_rates: Arc<Mutex<TimeSeriesBuffer>>,
    current_metrics: Arc<RwLock<PerformanceMetrics>>,
    health_calculator: HealthCalculator,
}
```

#### 2. Time-Series Data Storage
- **TimeSeriesBuffer**: Configurable retention and max data points
- **Real-time Analysis**: Percentile calculations and trend detection
- **Memory Efficient**: Automatic cleanup of expired data points

#### 3. Alert System
- **Rule-Based Alerting**: Configurable thresholds and conditions
- **Cooldown Periods**: Prevents alert flooding
- **Severity Levels**: Info, Warning, Error, Critical

## Key Performance Indicators (KPIs)

### Production Metrics

| Metric | Target | Good | Warning | Critical |
|--------|--------|------|---------|----------|
| **Extraction Time P95** | <2s | <5s | 5-10s | >10s |
| **Extraction Time P99** | <5s | <10s | 10-15s | >15s |
| **Requests/Second** | >100 | >50 | 20-50 | <20 |
| **Error Rate** | <1% | <5% | 5-10% | >10% |
| **Memory Usage** | <2GB | <4GB | 4-6GB | >6GB |
| **CPU Usage** | <60% | <70% | 70-90% | >90% |
| **Cache Hit Ratio** | >80% | >60% | 30-60% | <30% |
| **Health Score** | >95 | >85 | 50-85 | <50 |

### Core Performance Metrics Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    // Timing metrics
    pub avg_extraction_time_ms: f64,
    pub p95_extraction_time_ms: f64,
    pub p99_extraction_time_ms: f64,

    // Throughput metrics
    pub requests_per_second: f64,
    pub successful_extractions: u64,
    pub failed_extractions: u64,
    pub total_extractions: u64,

    // Resource metrics
    pub memory_usage_bytes: u64,
    pub cpu_usage_percent: f32,
    pub pool_size: usize,
    pub active_instances: usize,
    pub idle_instances: usize,

    // Quality metrics
    pub avg_content_quality_score: f64,
    pub avg_extracted_word_count: f64,
    pub cache_hit_ratio: f64,

    // Error metrics
    pub error_rate: f64,
    pub timeout_rate: f64,
    pub circuit_breaker_trips: u64,

    // System health
    pub health_score: f32,
    pub uptime_seconds: u64,
}
```

## Prometheus Metrics Integration

### Metric Naming Convention
All metrics follow the Prometheus naming conventions with the `riptide_` prefix:

```
# Timing metrics
riptide_extraction_duration_seconds_bucket
riptide_extraction_duration_seconds_sum
riptide_extraction_duration_seconds_count

# Throughput metrics
riptide_requests_total
riptide_requests_per_second

# Resource metrics
riptide_memory_usage_bytes
riptide_cpu_usage_ratio
riptide_pool_instances{state="active|idle"}

# Quality metrics
riptide_content_quality_score_avg
riptide_cache_hit_ratio

# Error metrics
riptide_errors_total{type="timeout|network|parse"}
riptide_circuit_breaker_trips_total

# Health metrics
riptide_health_score_ratio
riptide_uptime_seconds_total
```

### Metrics Collection Configuration

```toml
# monitoring.toml
[monitoring]
collection_interval_secs = 30
retention_period_hours = 24
max_data_points = 10000
alert_cooldown_secs = 300

[monitoring.health_thresholds]
error_rate_warning = 5.0
error_rate_critical = 10.0
cpu_usage_warning = 70.0
cpu_usage_critical = 90.0
memory_usage_warning = 2147483648  # 2GB
memory_usage_critical = 4294967296 # 4GB
extraction_time_warning_ms = 5000.0
extraction_time_critical_ms = 10000.0
```

### Metrics Recording API

```rust
// Recording extraction metrics
metrics_collector.record_extraction(
    duration,
    success,
    Some(quality_score),
    Some(word_count),
    was_cached
).await?;

// Recording error metrics
metrics_collector.record_error(
    "network_timeout",
    true // is_timeout
).await?;

// Recording pool statistics
metrics_collector.update_pool_stats(
    pool_size,
    active_instances,
    idle_instances
).await?;
```

## Real-Time Monitoring Dashboards

### Grafana Dashboard Configuration

#### System Overview Dashboard

```json
{
  "dashboard": {
    "id": null,
    "title": "RipTide EventMesh - System Overview",
    "tags": ["riptide", "performance", "monitoring"],
    "timezone": "browser",
    "panels": [
      {
        "id": 1,
        "title": "Request Rate",
        "type": "stat",
        "targets": [{
          "expr": "rate(riptide_requests_total[5m])",
          "legendFormat": "Requests/sec"
        }],
        "fieldConfig": {
          "defaults": {
            "unit": "reqps",
            "thresholds": {
              "steps": [
                {"color": "red", "value": 0},
                {"color": "yellow", "value": 20},
                {"color": "green", "value": 50}
              ]
            }
          }
        }
      },
      {
        "id": 2,
        "title": "Extraction Latency P95",
        "type": "stat",
        "targets": [{
          "expr": "histogram_quantile(0.95, rate(riptide_extraction_duration_seconds_bucket[5m]))",
          "legendFormat": "P95 Latency"
        }],
        "fieldConfig": {
          "defaults": {
            "unit": "s",
            "thresholds": {
              "steps": [
                {"color": "green", "value": 0},
                {"color": "yellow", "value": 2},
                {"color": "red", "value": 5}
              ]
            }
          }
        }
      }
    ]
  }
}
```

#### Performance Trends Dashboard

```json
{
  "dashboard": {
    "title": "RipTide EventMesh - Performance Trends",
    "panels": [
      {
        "id": 3,
        "title": "Extraction Time Percentiles",
        "type": "timeseries",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, rate(riptide_extraction_duration_seconds_bucket[5m]))",
            "legendFormat": "P50"
          },
          {
            "expr": "histogram_quantile(0.95, rate(riptide_extraction_duration_seconds_bucket[5m]))",
            "legendFormat": "P95"
          },
          {
            "expr": "histogram_quantile(0.99, rate(riptide_extraction_duration_seconds_bucket[5m]))",
            "legendFormat": "P99"
          }
        ]
      },
      {
        "id": 4,
        "title": "Error Rate by Type",
        "type": "timeseries",
        "targets": [{
          "expr": "rate(riptide_errors_total[5m]) by (type)",
          "legendFormat": "{{type}}"
        }]
      },
      {
        "id": 5,
        "title": "Resource Utilization",
        "type": "timeseries",
        "targets": [
          {
            "expr": "riptide_cpu_usage_ratio * 100",
            "legendFormat": "CPU %"
          },
          {
            "expr": "riptide_memory_usage_bytes / 1024 / 1024 / 1024",
            "legendFormat": "Memory GB"
          }
        ]
      }
    ]
  }
}
```

#### Alerting Dashboard

```json
{
  "dashboard": {
    "title": "RipTide EventMesh - Alerts & Health",
    "panels": [
      {
        "id": 6,
        "title": "Health Score",
        "type": "gauge",
        "targets": [{
          "expr": "riptide_health_score_ratio * 100",
          "legendFormat": "Health Score"
        }],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "min": 0,
            "max": 100,
            "thresholds": {
              "steps": [
                {"color": "red", "value": 0},
                {"color": "yellow", "value": 50},
                {"color": "green", "value": 85}
              ]
            }
          }
        }
      },
      {
        "id": 7,
        "title": "Active Alerts",
        "type": "table",
        "targets": [{
          "expr": "ALERTS{job=\"riptide\"}",
          "format": "table"
        }]
      }
    ]
  }
}
```

## Alerting Rules and Thresholds

### Prometheus Alerting Rules

```yaml
# alerts.yml
groups:
- name: riptide.rules
  rules:
  # High Error Rate
  - alert: RipTideHighErrorRate
    expr: (
      rate(riptide_errors_total[5m]) /
      rate(riptide_requests_total[5m])
    ) * 100 > 10
    for: 5m
    labels:
      severity: critical
      service: riptide
    annotations:
      summary: "RipTide error rate is above 10%"
      description: "Error rate is {{ $value | humanizePercentage }} for the last 5 minutes"

  # High Latency
  - alert: RipTideHighLatency
    expr: (
      histogram_quantile(0.95,
        rate(riptide_extraction_duration_seconds_bucket[5m])
      )
    ) > 10
    for: 10m
    labels:
      severity: warning
      service: riptide
    annotations:
      summary: "RipTide P95 latency is above 10 seconds"
      description: "P95 latency is {{ $value }}s for the last 10 minutes"

  # High CPU Usage
  - alert: RipTideHighCPU
    expr: riptide_cpu_usage_ratio > 0.90
    for: 15m
    labels:
      severity: critical
      service: riptide
    annotations:
      summary: "RipTide CPU usage is above 90%"
      description: "CPU usage is {{ $value | humanizePercentage }} for the last 15 minutes"

  # High Memory Usage
  - alert: RipTideHighMemory
    expr: riptide_memory_usage_bytes > 4294967296  # 4GB
    for: 10m
    labels:
      severity: warning
      service: riptide
    annotations:
      summary: "RipTide memory usage is above 4GB"
      description: "Memory usage is {{ $value | humanizeBytes }} for the last 10 minutes"

  # Low Health Score
  - alert: RipTideLowHealthScore
    expr: riptide_health_score_ratio < 0.50
    for: 5m
    labels:
      severity: critical
      service: riptide
    annotations:
      summary: "RipTide health score is below 50%"
      description: "Health score is {{ $value | humanizePercentage }} indicating system degradation"

  # Circuit Breaker Trips
  - alert: RipTideCircuitBreakerTrips
    expr: increase(riptide_circuit_breaker_trips_total[10m]) > 5
    labels:
      severity: warning
      service: riptide
    annotations:
      summary: "RipTide circuit breaker trips detected"
      description: "{{ $value }} circuit breaker trips in the last 10 minutes"

  # Low Cache Hit Ratio
  - alert: RipTideLowCacheHitRatio
    expr: riptide_cache_hit_ratio < 0.30
    for: 15m
    labels:
      severity: warning
      service: riptide
    annotations:
      summary: "RipTide cache hit ratio is below 30%"
      description: "Cache hit ratio is {{ $value | humanizePercentage }} for the last 15 minutes"
```

### Alert Manager Configuration

```yaml
# alertmanager.yml
route:
  group_by: ['alertname', 'service']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 12h
  receiver: 'riptide-alerts'
  routes:
  - match:
      severity: critical
    receiver: 'critical-alerts'
    repeat_interval: 5m

receivers:
- name: 'riptide-alerts'
  slack_configs:
  - api_url: 'YOUR_SLACK_WEBHOOK_URL'
    channel: '#riptide-monitoring'
    title: 'RipTide Alert - {{ .Status }}'
    text: |
      {{ range .Alerts }}
      *Alert:* {{ .Annotations.summary }}
      *Description:* {{ .Annotations.description }}
      *Severity:* {{ .Labels.severity }}
      {{ end }}

- name: 'critical-alerts'
  slack_configs:
  - api_url: 'YOUR_SLACK_WEBHOOK_URL'
    channel: '#riptide-critical'
    title: '🚨 CRITICAL: RipTide Alert'
    text: |
      {{ range .Alerts }}
      *CRITICAL ALERT:* {{ .Annotations.summary }}
      *Description:* {{ .Annotations.description }}
      *Time:* {{ .StartsAt.Format "2006-01-02 15:04:05" }}
      {{ end }}
  pagerduty_configs:
  - routing_key: 'YOUR_PAGERDUTY_INTEGRATION_KEY'
    description: '{{ .GroupLabels.alertname }} - {{ .Annotations.summary }}'
```

## Performance Baselines and Benchmarks

### Baseline Metrics Collection

```bash
#!/bin/bash
# baseline-collection.sh

# Collect baseline metrics for 1 hour
echo "Starting baseline collection..."

# Single URL extraction baseline
echo "=== Single URL Baseline ==="
for i in {1..100}; do
  curl -w "@curl-format.txt" -s -X POST 'http://localhost:8080/crawl' \
    -H 'Content-Type: application/json' \
    -d '{"urls": ["https://httpbin.org/html"]}' >> single_url_baseline.log
  sleep 1
done

# Batch processing baseline
echo "=== Batch Processing Baseline ==="
for i in {1..50}; do
  curl -w "@curl-format.txt" -s -X POST 'http://localhost:8080/crawl' \
    -H 'Content-Type: application/json' \
    -d '{
      "urls": [
        "https://httpbin.org/html",
        "https://httpbin.org/json",
        "https://httpbin.org/xml",
        "https://httpbin.org/headers",
        "https://httpbin.org/user-agent"
      ],
      "options": {"concurrency": 3}
    }' >> batch_baseline.log
  sleep 2
done

# High concurrency baseline
echo "=== High Concurrency Baseline ==="
for i in {1..20}; do
  curl -w "@curl-format.txt" -s -X POST 'http://localhost:8080/crawl' \
    -H 'Content-Type: application/json' \
    -d '{
      "urls": [
        "https://httpbin.org/delay/1",
        "https://httpbin.org/delay/2",
        "https://httpbin.org/delay/3"
      ],
      "options": {"concurrency": 10}
    }' >> high_concurrency_baseline.log
  sleep 5
done

echo "Baseline collection complete"

# curl-format.txt content:
# time_namelookup:%{time_namelookup}\n
# time_connect:%{time_connect}\n
# time_appconnect:%{time_appconnect}\n
# time_pretransfer:%{time_pretransfer}\n
# time_redirect:%{time_redirect}\n
# time_starttransfer:%{time_starttransfer}\n
# time_total:%{time_total}\n
# http_code:%{http_code}\n
# size_download:%{size_download}\n
```

### Performance Benchmark Analysis

```python
# analyze_baselines.py
import json
import statistics
from typing import Dict, List

class BaselineAnalyzer:
    def __init__(self):
        self.metrics = {
            'response_times': [],
            'throughput': [],
            'error_rates': [],
            'resource_usage': []
        }

    def parse_curl_output(self, log_file: str) -> List[Dict]:
        """Parse curl timing output into structured data"""
        results = []
        with open(log_file, 'r') as f:
            for line in f:
                if line.strip():
                    parts = line.strip().split('\n')
                    result = {}
                    for part in parts:
                        if ':' in part:
                            key, value = part.split(':', 1)
                            try:
                                result[key] = float(value)
                            except ValueError:
                                result[key] = value
                    results.append(result)
        return results

    def calculate_percentiles(self, values: List[float]) -> Dict:
        """Calculate performance percentiles"""
        if not values:
            return {}

        sorted_values = sorted(values)
        return {
            'p50': statistics.median(sorted_values),
            'p90': sorted_values[int(0.9 * len(sorted_values))],
            'p95': sorted_values[int(0.95 * len(sorted_values))],
            'p99': sorted_values[int(0.99 * len(sorted_values))],
            'mean': statistics.mean(sorted_values),
            'std': statistics.stdev(sorted_values) if len(sorted_values) > 1 else 0
        }

    def generate_baseline_report(self, baseline_files: List[str]) -> Dict:
        """Generate comprehensive baseline report"""
        report = {
            'baseline_date': '2024-01-01',
            'test_scenarios': {}
        }

        for file_path in baseline_files:
            scenario_name = file_path.replace('_baseline.log', '')
            results = self.parse_curl_output(file_path)

            if results:
                response_times = [r.get('time_total', 0) for r in results]
                success_rate = sum(1 for r in results if r.get('http_code') == 200) / len(results)

                report['test_scenarios'][scenario_name] = {
                    'response_times': self.calculate_percentiles(response_times),
                    'success_rate': success_rate,
                    'total_requests': len(results),
                    'throughput_rps': len(results) / max(response_times) if response_times else 0
                }

        return report

# Usage example
analyzer = BaselineAnalyzer()
baseline_report = analyzer.generate_baseline_report([
    'single_url_baseline.log',
    'batch_baseline.log',
    'high_concurrency_baseline.log'
])

with open('baseline_report.json', 'w') as f:
    json.dump(baseline_report, f, indent=2)
```

## Resource Monitoring

### System Resource Metrics

#### CPU Monitoring

```rust
// CPU usage collection implementation
use sysinfo::{CpuExt, System, SystemExt};

pub struct SystemMonitor {
    system: System,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self { system }
    }

    pub fn get_cpu_usage(&mut self) -> f32 {
        self.system.refresh_cpu();
        self.system.global_cpu_info().cpu_usage()
    }

    pub fn get_memory_usage(&mut self) -> u64 {
        self.system.refresh_memory();
        self.system.used_memory()
    }

    pub fn get_process_cpu_usage(&mut self, pid: u32) -> Option<f32> {
        self.system.refresh_process(sysinfo::Pid::from(pid as usize));
        self.system.process(sysinfo::Pid::from(pid as usize))
            .map(|p| p.cpu_usage())
    }
}
```

#### Memory Monitoring

```rust
// Memory usage tracking
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

struct MemoryTracker {
    allocated: AtomicU64,
    deallocated: AtomicU64,
}

static MEMORY_TRACKER: MemoryTracker = MemoryTracker {
    allocated: AtomicU64::new(0),
    deallocated: AtomicU64::new(0),
};

struct TrackedAllocator;

unsafe impl GlobalAlloc for TrackedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            MEMORY_TRACKER.allocated.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        MEMORY_TRACKER.deallocated.fetch_add(layout.size() as u64, Ordering::Relaxed);
    }
}

impl MemoryTracker {
    pub fn current_allocated(&self) -> u64 {
        self.allocated.load(Ordering::Relaxed) - self.deallocated.load(Ordering::Relaxed)
    }

    pub fn total_allocated(&self) -> u64 {
        self.allocated.load(Ordering::Relaxed)
    }
}
```

#### Network Monitoring

```rust
// Network metrics collection
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct NetworkMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connections_active: u32,
    pub connections_total: u64,
    pub response_times: Vec<Duration>,
    pub error_count: u64,
}

pub struct NetworkMonitor {
    connections: HashMap<SocketAddr, Instant>,
    metrics: NetworkMetrics,
}

impl NetworkMonitor {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            metrics: NetworkMetrics {
                bytes_sent: 0,
                bytes_received: 0,
                connections_active: 0,
                connections_total: 0,
                response_times: Vec::new(),
                error_count: 0,
            },
        }
    }

    pub fn record_connection(&mut self, addr: SocketAddr) {
        self.connections.insert(addr, Instant::now());
        self.metrics.connections_active = self.connections.len() as u32;
        self.metrics.connections_total += 1;
    }

    pub fn record_response(&mut self, addr: SocketAddr, bytes_sent: u64, bytes_received: u64) {
        if let Some(start_time) = self.connections.remove(&addr) {
            let duration = start_time.elapsed();
            self.metrics.response_times.push(duration);
            self.metrics.connections_active = self.connections.len() as u32;
        }

        self.metrics.bytes_sent += bytes_sent;
        self.metrics.bytes_received += bytes_received;
    }

    pub fn record_error(&mut self, addr: SocketAddr) {
        self.connections.remove(&addr);
        self.metrics.error_count += 1;
        self.metrics.connections_active = self.connections.len() as u32;
    }

    pub fn get_metrics(&self) -> &NetworkMetrics {
        &self.metrics
    }
}
```

## Application-Level Metrics

### Custom Business Metrics

```rust
// Business-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessMetrics {
    pub documents_processed: u64,
    pub avg_document_size: f64,
    pub content_types: HashMap<String, u64>,
    pub quality_distribution: HashMap<String, u64>, // excellent, good, fair, poor
    pub extraction_success_by_domain: HashMap<String, f64>,
    pub cache_effectiveness: f64,
    pub user_satisfaction_score: f64,
}

impl BusinessMetrics {
    pub fn record_document_processing(
        &mut self,
        size: u64,
        content_type: String,
        quality: u8,
        domain: String,
        extraction_success: bool,
    ) {
        self.documents_processed += 1;

        // Update average document size
        let total_docs = self.documents_processed as f64;
        let new_size = size as f64;
        self.avg_document_size =
            (self.avg_document_size * (total_docs - 1.0) + new_size) / total_docs;

        // Update content type distribution
        *self.content_types.entry(content_type).or_insert(0) += 1;

        // Update quality distribution
        let quality_label = match quality {
            90..=100 => "excellent",
            70..=89 => "good",
            50..=69 => "fair",
            _ => "poor",
        };
        *self.quality_distribution.entry(quality_label.to_string()).or_insert(0) += 1;

        // Update domain success rate
        let domain_entry = self.extraction_success_by_domain.entry(domain).or_insert(0.0);
        if extraction_success {
            *domain_entry += 1.0;
        }
    }
}
```

## Distributed Tracing Integration

### OpenTelemetry Setup

```rust
// OpenTelemetry tracing setup
use opentelemetry::{
    global, sdk::trace::TracerProvider, trace::TraceError, KeyValue,
};
use opentelemetry_jaeger::JaegerPipeline;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing(service_name: &str) -> Result<(), TraceError> {
    let tracer = JaegerPipeline::builder()
        .with_service_name(service_name)
        .with_endpoint("http://localhost:14268/api/traces")
        .with_tags(vec![
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
            KeyValue::new("service.environment", "production"),
        ])
        .install_batch(opentelemetry::runtime::Tokio)?;

    tracing_subscriber::registry()
        .with(OpenTelemetryLayer::new(tracer))
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

// Usage in extraction pipeline
use tracing::{info, warn, error, instrument, Span};

#[instrument(skip(content), fields(url = %url, content_length = content.len()))]
pub async fn extract_content(url: &str, content: &str) -> Result<ExtractionResult, ExtractionError> {
    let span = Span::current();

    // Record custom attributes
    span.record("extraction.start_time", chrono::Utc::now().timestamp());

    info!("Starting content extraction for {}", url);

    // Perform extraction
    let start_time = Instant::now();
    match perform_extraction(content).await {
        Ok(result) => {
            let duration = start_time.elapsed();
            span.record("extraction.duration_ms", duration.as_millis() as i64);
            span.record("extraction.success", true);
            span.record("extraction.word_count", result.word_count);
            span.record("extraction.quality_score", result.quality_score);

            info!(
                "Extraction completed successfully in {}ms, {} words extracted",
                duration.as_millis(),
                result.word_count
            );

            Ok(result)
        }
        Err(e) => {
            span.record("extraction.success", false);
            span.record("extraction.error", e.to_string());

            error!("Extraction failed for {}: {}", url, e);
            Err(e)
        }
    }
}
```

### Trace Correlation

```rust
// Correlate traces across service boundaries
use opentelemetry::propagation::{Extractor, Injector};
use std::collections::HashMap;

struct HttpHeaderExtractor<'a>(&'a HashMap<String, String>);

impl<'a> Extractor for HttpHeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|v| v.as_str())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

struct HttpHeaderInjector<'a>(&'a mut HashMap<String, String>);

impl<'a> Injector for HttpHeaderInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        self.0.insert(key.to_string(), value);
    }
}

// Extract trace context from incoming requests
pub fn extract_trace_context(headers: &HashMap<String, String>) -> Context {
    let extractor = HttpHeaderExtractor(headers);
    global::get_text_map_propagator(|propagator| {
        propagator.extract(&extractor)
    })
}

// Inject trace context into outgoing requests
pub fn inject_trace_context(context: &Context) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    let injector = HttpHeaderInjector(&mut headers);
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(context, &injector)
    });
    headers
}
```

## Performance Optimization Strategies

### 1. Extraction Pipeline Optimization

#### WASM Component Optimization

```rust
// Optimized WASM extraction with memory management
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn time(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn timeEnd(s: &str);
}

#[wasm_bindgen]
pub struct OptimizedExtractor {
    memory_pool: Vec<Vec<u8>>,
    max_pool_size: usize,
}

#[wasm_bindgen]
impl OptimizedExtractor {
    #[wasm_bindgen(constructor)]
    pub fn new(max_pool_size: usize) -> OptimizedExtractor {
        OptimizedExtractor {
            memory_pool: Vec::new(),
            max_pool_size,
        }
    }

    pub fn extract_optimized(&mut self, html: &str) -> ExtractionResult {
        time("extraction");

        // Reuse memory from pool
        let mut buffer = self.get_or_create_buffer(html.len());

        // Perform extraction with optimized algorithms
        let result = self.fast_extract(html, &mut buffer);

        // Return buffer to pool
        self.return_buffer(buffer);

        timeEnd("extraction");
        result
    }

    fn get_or_create_buffer(&mut self, min_size: usize) -> Vec<u8> {
        for (i, buffer) in self.memory_pool.iter().enumerate() {
            if buffer.capacity() >= min_size {
                return self.memory_pool.swap_remove(i);
            }
        }
        Vec::with_capacity(min_size)
    }

    fn return_buffer(&mut self, mut buffer: Vec<u8>) {
        if self.memory_pool.len() < self.max_pool_size {
            buffer.clear();
            self.memory_pool.push(buffer);
        }
    }
}
```

#### Content Gate Optimization

```rust
// Intelligent content routing for optimal processing
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ContentGate {
    routing_rules: Arc<RwLock<HashMap<String, ProcessingStrategy>>>,
    performance_stats: Arc<RwLock<HashMap<String, PerformanceStats>>>,
}

#[derive(Debug, Clone)]
pub enum ProcessingStrategy {
    FastTrack,    // Simple HTML, skip complex parsing
    Standard,     // Regular processing
    Intensive,    // Complex content, use advanced algorithms
    Cached,       // Serve from cache if available
}

#[derive(Debug, Clone)]
struct PerformanceStats {
    avg_processing_time: f64,
    success_rate: f64,
    cache_hit_rate: f64,
}

impl ContentGate {
    pub async fn route_content(&self, url: &str, content: &str) -> ProcessingStrategy {
        let content_analysis = self.analyze_content(content).await;
        let domain_stats = self.get_domain_stats(url).await;

        match content_analysis {
            ContentComplexity::Simple if domain_stats.cache_hit_rate > 0.8 => {
                ProcessingStrategy::Cached
            }
            ContentComplexity::Simple => ProcessingStrategy::FastTrack,
            ContentComplexity::Medium => ProcessingStrategy::Standard,
            ContentComplexity::Complex => ProcessingStrategy::Intensive,
        }
    }

    async fn analyze_content(&self, content: &str) -> ContentComplexity {
        let size = content.len();
        let has_tables = content.contains("<table");
        let has_scripts = content.contains("<script");
        let has_forms = content.contains("<form");

        match (size, has_tables, has_scripts, has_forms) {
            (s, _, _, _) if s > 100_000 => ContentComplexity::Complex,
            (_, true, true, _) => ContentComplexity::Complex,
            (_, _, true, true) => ContentComplexity::Medium,
            (s, _, _, _) if s < 10_000 => ContentComplexity::Simple,
            _ => ContentComplexity::Medium,
        }
    }
}

enum ContentComplexity {
    Simple,
    Medium,
    Complex,
}
```

### 2. Caching Strategy Optimization

#### Multi-Layer Caching Architecture

```rust
// Advanced caching with multiple tiers
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use lru::LruCache;

pub struct MultiTierCache {
    // L1: In-memory LRU cache (fastest)
    l1_cache: Arc<RwLock<LruCache<String, CachedContent>>>,

    // L2: Redis cache (fast, shared)
    redis_client: Arc<redis::Client>,

    // L3: Database cache (persistent)
    db_pool: Arc<sqlx::PgPool>,

    cache_stats: Arc<RwLock<CacheStats>>,
}

#[derive(Clone)]
struct CachedContent {
    data: String,
    quality_score: u8,
    word_count: u32,
    cached_at: chrono::DateTime<chrono::Utc>,
    access_count: u32,
    ttl: u64,
}

#[derive(Default)]
struct CacheStats {
    l1_hits: u64,
    l2_hits: u64,
    l3_hits: u64,
    misses: u64,
    evictions: u64,
}

impl MultiTierCache {
    pub async fn get(&self, key: &str) -> Option<CachedContent> {
        // Try L1 cache first
        {
            let mut l1 = self.l1_cache.write().await;
            if let Some(content) = l1.get_mut(key) {
                content.access_count += 1;
                self.record_hit(CacheLevel::L1).await;
                return Some(content.clone());
            }
        }

        // Try L2 cache (Redis)
        if let Ok(content) = self.get_from_redis(key).await {
            // Promote to L1
            self.promote_to_l1(key, &content).await;
            self.record_hit(CacheLevel::L2).await;
            return Some(content);
        }

        // Try L3 cache (Database)
        if let Ok(content) = self.get_from_db(key).await {
            // Promote to L2 and L1
            self.promote_to_l2(key, &content).await;
            self.promote_to_l1(key, &content).await;
            self.record_hit(CacheLevel::L3).await;
            return Some(content);
        }

        self.record_miss().await;
        None
    }

    pub async fn put(&self, key: String, content: CachedContent) {
        // Store in all tiers
        self.store_in_l1(&key, &content).await;
        self.store_in_redis(&key, &content).await;
        self.store_in_db(&key, &content).await;
    }

    async fn adaptive_eviction(&self) {
        let stats = self.cache_stats.read().await;
        let total_requests = stats.l1_hits + stats.l2_hits + stats.l3_hits + stats.misses;

        if total_requests > 0 {
            let l1_hit_rate = stats.l1_hits as f64 / total_requests as f64;

            // Adjust L1 cache size based on hit rate
            if l1_hit_rate < 0.2 {
                // Low hit rate, increase L1 cache size
                let mut l1 = self.l1_cache.write().await;
                l1.resize(l1.cap().saturating_mul(2).min(10000));
            } else if l1_hit_rate > 0.8 && l1.cap() > 1000 {
                // High hit rate, can reduce L1 cache size
                l1.resize(l1.cap() / 2);
            }
        }
    }
}

enum CacheLevel {
    L1,
    L2,
    L3,
}
```

#### Intelligent Cache Warming

```rust
// Predictive cache warming based on usage patterns
use std::collections::BTreeMap;
use tokio::time::{interval, Duration};

pub struct CacheWarmer {
    usage_patterns: Arc<RwLock<BTreeMap<String, UsagePattern>>>,
    warming_queue: Arc<RwLock<Vec<WarmingTask>>>,
}

#[derive(Debug, Clone)]
struct UsagePattern {
    url: String,
    access_frequency: f64,
    peak_hours: Vec<u8>,
    last_accessed: chrono::DateTime<chrono::Utc>,
    prediction_confidence: f64,
}

#[derive(Debug)]
struct WarmingTask {
    url: String,
    priority: f32,
    scheduled_time: chrono::DateTime<chrono::Utc>,
}

impl CacheWarmer {
    pub async fn analyze_and_warm(&self) {
        let patterns = self.analyze_usage_patterns().await;
        let warming_tasks = self.generate_warming_tasks(patterns).await;

        for task in warming_tasks {
            if task.priority > 0.7 {
                self.execute_warming_task(task).await;
            }
        }
    }

    async fn predict_next_access(&self, pattern: &UsagePattern) -> Option<chrono::DateTime<chrono::Utc>> {
        let now = chrono::Utc::now();
        let current_hour = now.hour() as u8;

        // Check if current hour is a peak hour
        if pattern.peak_hours.contains(&current_hour) {
            // Predict access within next hour
            Some(now + chrono::Duration::minutes(30))
        } else {
            // Find next peak hour
            let next_peak = pattern.peak_hours.iter()
                .find(|&&hour| hour > current_hour)
                .or_else(|| pattern.peak_hours.first())
                .copied()?;

            let hours_until_peak = if next_peak > current_hour {
                next_peak - current_hour
            } else {
                24 - current_hour + next_peak
            };

            Some(now + chrono::Duration::hours(hours_until_peak as i64))
        }
    }
}
```

### 3. Load Balancing and Auto-Scaling

#### Dynamic Load Balancer

```rust
// Adaptive load balancing with health checks
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct LoadBalancer {
    instances: Arc<RwLock<Vec<ServiceInstance>>>,
    health_checker: Arc<HealthChecker>,
    load_strategy: LoadStrategy,
    metrics: Arc<RwLock<LoadBalancerMetrics>>,
}

#[derive(Debug, Clone)]
struct ServiceInstance {
    id: String,
    endpoint: String,
    current_load: f32,
    health_score: f32,
    last_health_check: Instant,
    response_time_avg: f64,
    active_connections: u32,
    max_connections: u32,
}

#[derive(Debug, Clone)]
enum LoadStrategy {
    RoundRobin,
    LeastConnections,
    WeightedResponseTime,
    HealthAware,
}

impl LoadBalancer {
    pub async fn select_instance(&self) -> Option<ServiceInstance> {
        let instances = self.instances.read().await;
        let healthy_instances: Vec<_> = instances.iter()
            .filter(|i| i.health_score > 0.7 && i.current_load < 0.9)
            .collect();

        if healthy_instances.is_empty() {
            return None;
        }

        match self.load_strategy {
            LoadStrategy::HealthAware => {
                // Select instance with best health-to-load ratio
                healthy_instances.iter()
                    .max_by(|a, b| {
                        let score_a = a.health_score / (a.current_load + 0.1);
                        let score_b = b.health_score / (b.current_load + 0.1);
                        score_a.partial_cmp(&score_b).unwrap()
                    })
                    .cloned()
                    .cloned()
            }
            LoadStrategy::LeastConnections => {
                healthy_instances.iter()
                    .min_by_key(|i| i.active_connections)
                    .cloned()
                    .cloned()
            }
            LoadStrategy::WeightedResponseTime => {
                healthy_instances.iter()
                    .min_by(|a, b| {
                        a.response_time_avg.partial_cmp(&b.response_time_avg).unwrap()
                    })
                    .cloned()
                    .cloned()
            }
            LoadStrategy::RoundRobin => {
                // Implementation for round-robin
                healthy_instances.first().cloned().cloned()
            }
        }
    }

    pub async fn auto_scale(&self) {
        let metrics = self.metrics.read().await;
        let avg_load = metrics.average_load();
        let error_rate = metrics.error_rate();

        if avg_load > 0.8 || error_rate > 0.1 {
            // Scale out
            self.scale_out().await;
        } else if avg_load < 0.3 && error_rate < 0.01 {
            // Scale in
            self.scale_in().await;
        }
    }
}
```

### 4. Circuit Breaker Pattern

```rust
// Advanced circuit breaker with adaptive thresholds
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    config: CircuitConfig,
    metrics: Arc<RwLock<CircuitMetrics>>,
}

#[derive(Debug)]
enum CircuitState {
    Closed,
    Open { opened_at: Instant },
    HalfOpen { attempt_count: u32 },
}

#[derive(Debug, Clone)]
struct CircuitConfig {
    failure_threshold: f32,
    success_threshold: u32,
    timeout: Duration,
    half_open_max_calls: u32,
    window_size: Duration,
}

#[derive(Debug, Default)]
struct CircuitMetrics {
    total_calls: u64,
    failed_calls: u64,
    success_calls: u64,
    last_failure_time: Option<Instant>,
    consecutive_successes: u32,
    window_start: Instant,
}

impl CircuitBreaker {
    pub async fn call<F, R, E>(&self, operation: F) -> Result<R, CircuitBreakerError>
    where
        F: std::future::Future<Output = Result<R, E>>,
        E: std::fmt::Debug,
    {
        // Check if circuit allows call
        if !self.can_execute().await {
            return Err(CircuitBreakerError::CircuitOpen);
        }

        // Execute operation
        let start_time = Instant::now();
        match operation.await {
            Ok(result) => {
                self.record_success(start_time.elapsed()).await;
                Ok(result)
            }
            Err(e) => {
                self.record_failure(start_time.elapsed()).await;
                Err(CircuitBreakerError::OperationFailed(format!("{:?}", e)))
            }
        }
    }

    async fn can_execute(&self) -> bool {
        let mut state = self.state.write().await;
        let now = Instant::now();

        match *state {
            CircuitState::Closed => true,
            CircuitState::Open { opened_at } => {
                if now.duration_since(opened_at) > self.config.timeout {
                    *state = CircuitState::HalfOpen { attempt_count: 0 };
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen { attempt_count } => {
                attempt_count < self.config.half_open_max_calls
            }
        }
    }

    async fn adaptive_threshold_adjustment(&self) {
        let metrics = self.metrics.read().await;
        let current_error_rate = if metrics.total_calls > 0 {
            metrics.failed_calls as f32 / metrics.total_calls as f32
        } else {
            0.0
        };

        // Adjust thresholds based on historical performance
        if current_error_rate < 0.01 {
            // System is very stable, can be more tolerant
            self.config.failure_threshold = (self.config.failure_threshold * 1.1).min(0.5);
        } else if current_error_rate > 0.1 {
            // System is unstable, be more aggressive
            self.config.failure_threshold = (self.config.failure_threshold * 0.9).max(0.05);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError {
    #[error("Circuit breaker is open")]
    CircuitOpen,
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}
```

## Production Deployment Checklist

### Pre-Deployment
- [ ] Performance baselines established
- [ ] Monitoring dashboards configured
- [ ] Alert rules tested and verified
- [ ] Circuit breakers configured
- [ ] Load balancing strategy validated
- [ ] Cache warming procedures tested
- [ ] Distributed tracing enabled
- [ ] Resource limits configured

### Post-Deployment
- [ ] Health checks passing
- [ ] Metrics collection verified
- [ ] Alert notifications working
- [ ] Performance within SLA targets
- [ ] Error rates below thresholds
- [ ] Resource utilization optimal
- [ ] Cache hit ratios acceptable
- [ ] Trace data flowing correctly

### Ongoing Maintenance
- [ ] Weekly performance reviews
- [ ] Monthly baseline updates
- [ ] Quarterly capacity planning
- [ ] Alert rule tuning
- [ ] Dashboard improvements
- [ ] Performance optimization
- [ ] Incident post-mortems
- [ ] Trend analysis reports

---

**Note**: This monitoring architecture provides comprehensive observability for production environments. Regular review and optimization of these systems ensures optimal performance and reliability of the RipTide EventMesh platform.
