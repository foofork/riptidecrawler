# Memory Profiling Examples

## Overview

This guide provides practical examples for using RipTide's memory profiling system in various scenarios.

## Example 1: Basic Profiling

**Scenario:** Profile a simple web crawling operation.

```rust
use riptide_performance::profiling::{MemoryProfiler, MemoryProfileConfig};
use uuid::Uuid;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create profiler with default configuration
    let session_id = Uuid::new_v4();
    let mut profiler = MemoryProfiler::new(session_id)?;

    // Start profiling
    profiler.start_profiling().await?;
    println!("Profiling started for session: {}", session_id);

    // Run your workload
    for i in 0..100 {
        // Simulate crawling URLs
        let url = format!("https://example.com/page{}", i);
        crawl_url(&url).await?;

        // Check memory every 10 iterations
        if i % 10 == 0 {
            let snapshot = profiler.get_current_snapshot().await?;
            let memory_mb = snapshot.rss_bytes as f64 / 1024.0 / 1024.0;
            println!("Memory at iteration {}: {:.2}MB", i, memory_mb);
        }
    }

    // Stop profiling and get report
    let report = profiler.stop_profiling().await?;

    // Print summary
    println!("\n=== Profiling Results ===");
    println!("Duration: {:?}", report.profiling_duration);
    println!("Peak memory: {:.2}MB", report.peak_memory_mb);
    println!("Average memory: {:.2}MB", report.average_memory_mb);
    println!("Growth rate: {:.4}MB/s", report.memory_growth_rate_mb_s);
    println!("Efficiency score: {:.2}/1.0", report.memory_efficiency_score);

    // Print recommendations
    if !report.recommendations.is_empty() {
        println!("\n=== Recommendations ===");
        for rec in &report.recommendations {
            println!("- {}", rec);
        }
    }

    Ok(())
}

async fn crawl_url(url: &str) -> anyhow::Result<()> {
    // Your crawling logic here
    tokio::time::sleep(Duration::from_millis(100)).await;
    Ok(())
}
```

**Expected Output:**

```
Profiling started for session: 550e8400-e29b-41d4-a716-446655440000
Memory at iteration 0: 42.15MB
Memory at iteration 10: 45.32MB
Memory at iteration 20: 48.67MB
Memory at iteration 30: 51.89MB
...

=== Profiling Results ===
Duration: 10.234s
Peak memory: 52.45MB
Average memory: 46.78MB
Growth rate: 1.0234MB/s
Efficiency score: 0.85/1.0

=== Recommendations ===
- Memory usage is within normal parameters. Continue monitoring.
- Enable memory profiling in production with sampling to reduce overhead.
```

## Example 2: Leak Detection

**Scenario:** Detect and diagnose memory leaks in a long-running service.

```rust
use riptide_performance::profiling::{
    MemoryProfiler, MemoryProfileConfig, AllocationInfo
};
use uuid::Uuid;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure profiler for leak detection
    let config = MemoryProfileConfig {
        sampling_interval: Duration::from_secs(2),
        track_allocations: true,
        detect_leaks: true,
        warning_threshold_mb: 100.0,
        alert_threshold_mb: 150.0,
        ..Default::default()
    };

    let session_id = Uuid::new_v4();
    let mut profiler = MemoryProfiler::with_config(session_id, config)?;

    // Start profiling
    profiler.start_profiling().await?;

    // Simulate a leaky workload
    let mut leaked_data = Vec::new();

    for iteration in 0..60 {
        // Simulate work with intentional "leak"
        let data = vec![0u8; 1024 * 1024]; // 1MB allocation
        leaked_data.push(data); // Not freed!

        tokio::time::sleep(Duration::from_secs(1)).await;

        // Check for alerts every 10 seconds
        if iteration % 10 == 0 {
            let alerts = profiler.check_memory_thresholds().await?;

            if !alerts.is_empty() {
                println!("\n=== ALERTS at iteration {} ===", iteration);
                for alert in alerts {
                    eprintln!("{}", alert);
                }
            }

            // Get current memory trend
            let trend = profiler.get_memory_trend(Duration::from_secs(30)).await?;
            if let Some((_, latest_mb)) = trend.last() {
                println!("Current memory: {:.2}MB", latest_mb);
            }
        }
    }

    // Stop profiling and analyze leaks
    let report = profiler.stop_profiling().await?;

    println!("\n=== Leak Analysis ===");
    println!("Growth rate: {:.2}MB/hour",
             report.leak_analysis.growth_rate_mb_per_hour);

    if !report.leak_analysis.potential_leaks.is_empty() {
        println!("\nPotential leaks detected:");
        for leak in &report.leak_analysis.potential_leaks {
            println!("\nComponent: {}", leak.component);
            println!("  Allocations: {}", leak.allocation_count);
            println!("  Total size: {:.2}MB",
                     leak.total_size_bytes as f64 / 1024.0 / 1024.0);
            println!("  Average size: {:.2}KB",
                     leak.average_size_bytes / 1024.0);
            println!("  Growth rate: {:.2} bytes/s", leak.growth_rate);
            println!("  First seen: {}", leak.first_seen);
            println!("  Last seen: {}", leak.last_seen);
        }
    }

    if !report.leak_analysis.suspicious_patterns.is_empty() {
        println!("\n=== Suspicious Patterns ===");
        for pattern in &report.leak_analysis.suspicious_patterns {
            println!("- {}", pattern);
        }
    }

    Ok(())
}
```

**Expected Output:**

```
=== ALERTS at iteration 10 ===
WARNING: Memory usage 110.5MB exceeds warning threshold 100.0MB
Current memory: 110.50MB

=== ALERTS at iteration 20 ===
CRITICAL: Memory usage 160.3MB exceeds alert threshold 150.0MB
Current memory: 160.30MB

=== Leak Analysis ===
Growth rate: 3600.00MB/hour

Potential leaks detected:

Component: vector_allocations
  Allocations: 60
  Total size: 60.00MB
  Average size: 1024.00KB
  Growth rate: 1048576.00 bytes/s
  First seen: 2025-10-06T08:00:00Z
  Last seen: 2025-10-06T08:01:00Z

=== Suspicious Patterns ===
- vector_allocations: Exponential allocation growth detected
- vector_allocations: Frequent large allocations (60)
```

## Example 3: Allocation Analysis

**Scenario:** Analyze allocation patterns to optimize memory usage.

```rust
use riptide_performance::profiling::{MemoryProfiler, MemoryProfileConfig};
use uuid::Uuid;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = MemoryProfileConfig {
        track_allocations: true,
        sampling_interval: Duration::from_secs(1),
        ..Default::default()
    };

    let session_id = Uuid::new_v4();
    let mut profiler = MemoryProfiler::with_config(session_id, config)?;

    profiler.start_profiling().await?;

    // Simulate different allocation patterns
    simulate_workload().await?;

    let report = profiler.stop_profiling().await?;

    // Analyze top allocators
    println!("=== Top Memory Allocators ===");
    for (i, (component, bytes)) in report.top_allocators.iter().enumerate().take(10) {
        println!("{}. {}: {:.2}MB",
                 i + 1,
                 component,
                 *bytes as f64 / 1024.0 / 1024.0);
    }

    // Generate optimization recommendations
    println!("\n=== Optimization Opportunities ===");
    for rec in &report.recommendations {
        println!("- {}", rec);
    }

    // Export report to JSON for further analysis
    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write("memory-report.json", json)?;
    println!("\nDetailed report saved to: memory-report.json");

    Ok(())
}

async fn simulate_workload() -> anyhow::Result<()> {
    // Component 1: Many small allocations
    for _ in 0..1000 {
        let _small = vec![0u8; 128]; // 128 bytes each
    }

    // Component 2: Few large allocations
    for _ in 0..10 {
        let _large = vec![0u8; 1024 * 1024]; // 1MB each
    }

    // Component 3: Medium allocations with variation
    for i in 0..100 {
        let size = 1024 * (i % 64 + 1); // 1KB to 64KB
        let _medium = vec![0u8; size];
    }

    tokio::time::sleep(Duration::from_secs(2)).await;
    Ok(())
}
```

**Expected Output:**

```
=== Top Memory Allocators ===
1. large_buffer_allocator: 10.00MB
2. medium_buffer_pool: 3.25MB
3. small_object_cache: 0.13MB
4. http_client_buffers: 2.50MB
5. json_parser_buffers: 1.75MB

=== Optimization Opportunities ===
- High percentage of small allocations detected. Consider implementing object pools for frequently allocated small objects.
- Component 'large_buffer_allocator' has high memory allocation (10MB). Consider optimizing memory usage or implementing cleanup routines.
- Operation 'buffer_allocation' called very frequently (1000 times, avg 128bytes). Consider batching or caching.

Detailed report saved to: memory-report.json
```

## Example 4: HTTP Endpoint Usage

**Scenario:** Expose memory profiling via REST API for monitoring dashboards.

```rust
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use riptide_performance::profiling::{MemoryProfiler, MemorySnapshot, MemoryReport};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

type SharedProfiler = Arc<RwLock<Option<MemoryProfiler>>>;

#[tokio::main]
async fn main() {
    let profiler: SharedProfiler = Arc::new(RwLock::new(None));

    let app = Router::new()
        .route("/profiling/start", post(start_profiling))
        .route("/profiling/stop", post(stop_profiling))
        .route("/profiling/snapshot", get(get_snapshot))
        .route("/profiling/alerts", get(get_alerts))
        .route("/profiling/trend", get(get_trend))
        .with_state(profiler);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();

    println!("Memory profiling API listening on http://0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}

async fn start_profiling(
    State(profiler): State<SharedProfiler>,
) -> Json<serde_json::Value> {
    let mut lock = profiler.write().await;

    if lock.is_some() {
        return Json(serde_json::json!({
            "status": "error",
            "message": "Profiling already active"
        }));
    }

    let session_id = Uuid::new_v4();
    let mut new_profiler = MemoryProfiler::new(session_id).unwrap();
    new_profiler.start_profiling().await.unwrap();

    *lock = Some(new_profiler);

    Json(serde_json::json!({
        "status": "success",
        "session_id": session_id,
        "message": "Profiling started"
    }))
}

async fn stop_profiling(
    State(profiler): State<SharedProfiler>,
) -> Json<MemoryReport> {
    let mut lock = profiler.write().await;

    if let Some(mut active_profiler) = lock.take() {
        let report = active_profiler.stop_profiling().await.unwrap();
        Json(report)
    } else {
        panic!("No active profiling session")
    }
}

async fn get_snapshot(
    State(profiler): State<SharedProfiler>,
) -> Json<MemorySnapshot> {
    let lock = profiler.read().await;

    if let Some(active_profiler) = lock.as_ref() {
        let snapshot = active_profiler.get_current_snapshot().await.unwrap();
        Json(snapshot)
    } else {
        panic!("No active profiling session")
    }
}

async fn get_alerts(
    State(profiler): State<SharedProfiler>,
) -> Json<Vec<String>> {
    let lock = profiler.read().await;

    if let Some(active_profiler) = lock.as_ref() {
        let alerts = active_profiler.check_memory_thresholds().await.unwrap();
        Json(alerts)
    } else {
        Json(vec![])
    }
}

async fn get_trend(
    State(profiler): State<SharedProfiler>,
) -> Json<Vec<(chrono::DateTime<chrono::Utc>, f64)>> {
    let lock = profiler.read().await;

    if let Some(active_profiler) = lock.as_ref() {
        let trend = active_profiler
            .get_memory_trend(std::time::Duration::from_secs(300))
            .await
            .unwrap();
        Json(trend)
    } else {
        Json(vec![])
    }
}
```

**Usage Examples:**

```bash
# Start profiling
curl -X POST http://localhost:8080/profiling/start
# Response: {"status":"success","session_id":"550e8400-e29b-41d4-a716-446655440000","message":"Profiling started"}

# Get current memory snapshot
curl http://localhost:8080/profiling/snapshot
# Response: {"timestamp":"2025-10-06T08:00:00Z","rss_bytes":104857600,...}

# Check for alerts
curl http://localhost:8080/profiling/alerts
# Response: ["WARNING: Memory usage 680.5MB exceeds warning threshold 650.0MB"]

# Get memory trend (last 5 minutes)
curl http://localhost:8080/profiling/trend
# Response: [["2025-10-06T07:55:00Z",45.2],["2025-10-06T07:56:00Z",46.8],...]

# Stop profiling and get full report
curl -X POST http://localhost:8080/profiling/stop > report.json
```

## Example 5: Production Monitoring

**Scenario:** Continuous monitoring in production with Prometheus integration.

```rust
use riptide_performance::profiling::{MemoryProfiler, MemoryProfileConfig};
use prometheus::{register_gauge, Gauge, Encoder, TextEncoder};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

lazy_static::lazy_static! {
    static ref MEMORY_RSS_MB: Gauge = register_gauge!(
        "riptide_memory_rss_megabytes",
        "Resident set size in megabytes"
    ).unwrap();

    static ref MEMORY_HEAP_MB: Gauge = register_gauge!(
        "riptide_memory_heap_megabytes",
        "Heap memory in megabytes"
    ).unwrap();

    static ref MEMORY_GROWTH_RATE: Gauge = register_gauge!(
        "riptide_memory_growth_rate_mb_per_sec",
        "Memory growth rate in MB/s"
    ).unwrap();

    static ref MEMORY_EFFICIENCY_SCORE: Gauge = register_gauge!(
        "riptide_memory_efficiency_score",
        "Memory efficiency score (0-1)"
    ).unwrap();

    static ref POTENTIAL_LEAKS: Gauge = register_gauge!(
        "riptide_memory_potential_leaks",
        "Number of potential memory leaks detected"
    ).unwrap();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure for production use
    let config = MemoryProfileConfig {
        sampling_interval: Duration::from_secs(30), // Less frequent in production
        max_samples: 500,                           // Reduced memory footprint
        track_allocations: false,                   // Disable for lower overhead
        detect_leaks: true,                         // Keep leak detection
        generate_flamegraphs: false,                // Disabled in production
        warning_threshold_mb: 650.0,
        alert_threshold_mb: 700.0,
    };

    let session_id = Uuid::new_v4();
    let mut profiler = MemoryProfiler::with_config(session_id, config)?;

    // Start continuous profiling
    profiler.start_profiling().await?;
    println!("Production memory monitoring started");

    let profiler = Arc::new(RwLock::new(profiler));

    // Start metrics update loop
    let metrics_profiler = Arc::clone(&profiler);
    tokio::spawn(async move {
        loop {
            update_metrics(&metrics_profiler).await;
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });

    // Start alert check loop
    let alert_profiler = Arc::clone(&profiler);
    tokio::spawn(async move {
        loop {
            check_and_send_alerts(&alert_profiler).await;
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    });

    // Serve Prometheus metrics
    serve_metrics().await;

    Ok(())
}

async fn update_metrics(profiler: &Arc<RwLock<MemoryProfiler>>) {
    let lock = profiler.read().await;

    if let Ok(snapshot) = lock.get_current_snapshot().await {
        let rss_mb = snapshot.rss_bytes as f64 / 1024.0 / 1024.0;
        let heap_mb = snapshot.heap_bytes as f64 / 1024.0 / 1024.0;

        MEMORY_RSS_MB.set(rss_mb);
        MEMORY_HEAP_MB.set(heap_mb);
    }

    if let Ok(trend) = lock.get_memory_trend(Duration::from_secs(300)).await {
        if trend.len() >= 2 {
            let first = trend[0].1;
            let last = trend[trend.len() - 1].1;
            let growth_rate = (last - first) / 300.0; // MB/s over 5 minutes
            MEMORY_GROWTH_RATE.set(growth_rate);
        }
    }
}

async fn check_and_send_alerts(profiler: &Arc<RwLock<MemoryProfiler>>) {
    let lock = profiler.read().await;

    if let Ok(alerts) = lock.check_memory_thresholds().await {
        for alert in alerts {
            eprintln!("ALERT: {}", alert);
            // Send to alerting system (PagerDuty, Slack, etc.)
            send_alert_notification(&alert).await;
        }
    }
}

async fn send_alert_notification(alert: &str) {
    // Integration with alerting systems
    println!("Sending alert: {}", alert);
    // TODO: Implement actual notification (Slack, PagerDuty, email, etc.)
}

async fn serve_metrics() {
    use axum::{routing::get, Router};

    async fn metrics_handler() -> String {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = vec![];
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }

    let app = Router::new().route("/metrics", get(metrics_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9090")
        .await
        .unwrap();

    println!("Prometheus metrics available at http://0.0.0.0:9090/metrics");
    axum::serve(listener, app).await.unwrap();
}
```

**Prometheus Configuration (prometheus.yml):**

```yaml
scrape_configs:
  - job_name: 'riptide-memory'
    scrape_interval: 30s
    static_configs:
      - targets: ['localhost:9090']
```

**Grafana Dashboard Query Examples:**

```promql
# Memory usage over time
riptide_memory_rss_megabytes

# Memory growth rate
rate(riptide_memory_rss_megabytes[5m])

# Alert on high memory
riptide_memory_rss_megabytes > 650

# Efficiency trend
avg_over_time(riptide_memory_efficiency_score[1h])
```

## Summary

These examples demonstrate:

1. **Basic profiling** for simple use cases
2. **Leak detection** for diagnosing memory issues
3. **Allocation analysis** for optimization
4. **HTTP endpoints** for monitoring integration
5. **Production monitoring** with Prometheus and alerts

For more information, see:
- [Memory Profiling Activation Guide](memory-profiling-activation-guide.md)
- [Performance Monitoring](performance-monitoring.md)
- [API Documentation](api/openapi.yaml)
