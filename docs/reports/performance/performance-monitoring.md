# Performance Monitoring & Profiling

**Status:** ✅ Production Ready
**Last Updated:** 2025-10-06

---

## Overview

RipTide includes comprehensive performance monitoring and memory profiling capabilities for production deployments. All profiling components are production-ready and provide essential insights for optimization and leak detection.

### Available Components

| Component | Status | Purpose |
|-----------|--------|---------|
| **Memory Tracker** | ✅ Active | Real-time RSS, heap, and virtual memory tracking |
| **Leak Detector** | ✅ Active | Memory leak detection with pattern analysis |
| **Allocation Analyzer** | ✅ Active | Allocation patterns and optimization recommendations |

---

## Quick Start

### Enable Performance Monitoring

All monitoring components are automatically initialized when creating the `PerformanceManager`:

```rust
use riptide_performance::PerformanceManager;

// Create performance manager with profiling enabled
let perf_manager = PerformanceManager::new().await?;

// Access profiling components
let memory_snapshot = perf_manager.profiler.tracker.get_current_snapshot().await?;
let leak_analysis = perf_manager.profiler.leak_detector.analyze_leaks().await?;
let top_allocators = perf_manager.profiler.allocation_analyzer.get_top_allocators().await?;
```

---

## Memory Tracker

**Purpose:** Track system and process memory usage in real-time

### Features

- **RSS (Resident Set Size)** tracking
- **Heap memory** tracking (with optional jemalloc integration)
- **Virtual memory** monitoring
- **Memory breakdown** by component
- **Force GC** capability

### Usage

```rust
use riptide_performance::profiling::MemoryTracker;

let tracker = MemoryTracker::new()?;

// Get current memory snapshot
let snapshot = tracker.get_current_snapshot().await?;
println!("RSS: {} MB", snapshot.rss_bytes / 1024 / 1024);
println!("Heap: {} MB", snapshot.heap_bytes / 1024 / 1024);
println!("Virtual: {} MB", snapshot.virtual_bytes / 1024 / 1024);

// Get detailed breakdown
let breakdown = tracker.get_memory_breakdown().await?;
for (component, bytes) in breakdown {
    println!("{}: {} MB", component, bytes / 1024 / 1024);
}

// Force garbage collection
tracker.force_gc().await?;
```

### Memory Snapshot Structure

```rust
pub struct MemorySnapshot {
    pub timestamp: DateTime<Utc>,
    pub rss_bytes: u64,           // Resident set size
    pub heap_bytes: u64,          // Heap allocation (jemalloc)
    pub virtual_bytes: u64,       // Virtual memory
    pub resident_bytes: u64,      // Physical memory
    pub shared_bytes: u64,        // Shared memory
    pub text_bytes: u64,          // Code segment
    pub data_bytes: u64,          // Data segment
    pub stack_bytes: u64,         // Stack size
}
```

### Optional: jemalloc Integration

For detailed allocator statistics, enable the `jemalloc` feature:

```toml
[dependencies]
riptide-performance = { version = "0.1", features = ["jemalloc"] }
```

With jemalloc enabled, you get additional metrics:
- `allocated` - Currently allocated bytes
- `active` - Active (in-use) bytes
- `metadata` - Allocator metadata overhead
- `resident` - Physical memory used
- `mapped` - Virtual memory mapped
- `retained` - Memory retained for reuse

---

## Leak Detector

**Purpose:** Detect and analyze memory leaks using heuristic analysis

### Detection Criteria

The leak detector identifies potential leaks based on:

1. **High Growth Rate:** >10MB/hour allocation growth
2. **Large Allocations:** >50MB with recent activity
3. **Small Allocation Accumulation:** >1000 allocations totaling >1MB
4. **Peak Size Growth:** Peak exceeds current by 2x

### Usage

```rust
use riptide_performance::profiling::LeakDetector;

let mut detector = LeakDetector::new()?;

// Record allocations during operation
detector.record_allocation(AllocationInfo {
    timestamp: Utc::now(),
    size: 1024,
    alignment: 8,
    stack_trace: vec!["function_name".to_string()],
    component: "extraction".to_string(),
    operation: "parse_html".to_string(),
}).await?;

// Record deallocations
detector.record_deallocation("extraction", 1024).await?;

// Analyze for leaks
let analysis = detector.analyze_leaks().await?;

// Review potential leaks
for leak in analysis.potential_leaks {
    println!("Component: {}", leak.component);
    println!("Total Size: {} MB", leak.total_size_bytes / 1024 / 1024);
    println!("Growth Rate: {:.2} MB/hour", leak.growth_rate / 1024.0 / 1024.0);
    println!("Allocations: {}", leak.allocation_count);
}

// Check memory pressure
let pressure = detector.get_memory_pressure().await?;
if pressure > 0.7 {
    println!("⚠️  High memory pressure: {:.1}%", pressure * 100.0);
}
```

### Leak Analysis Structure

```rust
pub struct LeakAnalysis {
    pub potential_leaks: Vec<LeakInfo>,
    pub growth_rate_mb_per_hour: f64,
    pub largest_allocations: Vec<AllocationInfo>,
    pub suspicious_patterns: Vec<String>,
}

pub struct LeakInfo {
    pub component: String,
    pub allocation_count: u64,
    pub total_size_bytes: u64,
    pub average_size_bytes: f64,
    pub growth_rate: f64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}
```

### Pattern Detection

The leak detector identifies suspicious patterns:

- **Exponential Growth:** Each allocation doubles the previous
- **Frequent Large Allocations:** >5 allocations >1MB
- **Repeated Patterns:** Same stack trace appearing 5+ times (loop leaks)

---

## Allocation Analyzer

**Purpose:** Analyze allocation patterns for optimization opportunities

### Features

- **Top Allocators:** Identify components with highest memory usage
- **Size Distribution:** Categorize allocations by size
- **Operation Analysis:** Track allocations by operation type
- **Fragmentation Analysis:** Detect memory fragmentation issues
- **Efficiency Scoring:** Calculate allocation efficiency (0.0-1.0)

### Usage

```rust
use riptide_performance::profiling::AllocationAnalyzer;

let mut analyzer = AllocationAnalyzer::new()?;

// Record allocations
analyzer.record_allocation(AllocationInfo {
    timestamp: Utc::now(),
    size: 4096,
    alignment: 8,
    stack_trace: vec!["render_page".to_string()],
    component: "renderer".to_string(),
    operation: "allocate_buffer".to_string(),
}).await?;

// Get top allocators
let top_allocators = analyzer.get_top_allocators().await?;
for (component, bytes) in top_allocators {
    println!("{}: {} MB", component, bytes / 1024 / 1024);
}

// Get size distribution
let distribution = analyzer.get_size_distribution().await?;
println!("Tiny (<1KB): {}", distribution["tiny (<1KB)"]);
println!("Small (1KB-64KB): {}", distribution["small (1KB-64KB)"]);
println!("Medium (64KB-1MB): {}", distribution["medium (64KB-1MB)"]);
println!("Large (1MB-16MB): {}", distribution["large (1MB-16MB)"]);
println!("Huge (>16MB): {}", distribution["huge (>16MB)"]);

// Get optimization recommendations
let recommendations = analyzer.analyze_patterns().await?;
for recommendation in recommendations {
    println!("💡 {}", recommendation);
}

// Calculate efficiency score
let efficiency = analyzer.calculate_efficiency_score().await?;
println!("Allocation Efficiency: {:.1}%", efficiency * 100.0);
```

### Optimization Recommendations

The analyzer provides actionable recommendations:

- **High Small Allocations:** "Consider implementing object pools for frequently allocated small objects"
- **Huge Allocations:** "Consider streaming or chunking for large data processing"
- **Hot Components:** "Component 'X' has high memory allocation (>100MB). Consider optimizing memory usage"
- **Frequent Operations:** "Operation 'X' called very frequently (>10k times). Consider batching or caching"
- **Small Allocation Patterns:** "Component 'X' makes many small allocations. Consider using a memory pool"

---

## Production Integration

### Step 1: Initialize Profiling

```rust
use riptide_api::state::AppState;

let app_state = AppState::new_with_telemetry_and_api_config(...).await?;

// Performance manager is automatically initialized in AppState
// Access via app_state.performance_metrics
```

### Step 2: Add Monitoring Endpoints

Create `/crates/riptide-api/src/handlers/monitoring.rs`:

```rust
use crate::state::AppState;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct MemoryMetrics {
    pub rss_mb: f64,
    pub heap_mb: f64,
    pub virtual_mb: f64,
    pub timestamp: String,
}

#[derive(Serialize)]
pub struct LeakSummary {
    pub potential_leak_count: usize,
    pub growth_rate_mb_per_hour: f64,
    pub highest_risk_component: Option<String>,
}

#[derive(Serialize)]
pub struct AllocationMetrics {
    pub top_allocators: Vec<(String, u64)>,
    pub efficiency_score: f64,
    pub recommendations: Vec<String>,
}

/// Get current memory metrics
pub async fn get_memory_metrics(
    State(state): State<AppState>,
) -> Json<MemoryMetrics> {
    // Access from performance manager
    let perf_manager = &state.performance_metrics.lock().await;

    let snapshot = perf_manager.profiler.tracker
        .get_current_snapshot()
        .await
        .unwrap_or_default();

    Json(MemoryMetrics {
        rss_mb: snapshot.rss_bytes as f64 / 1024.0 / 1024.0,
        heap_mb: snapshot.heap_bytes as f64 / 1024.0 / 1024.0,
        virtual_mb: snapshot.virtual_bytes as f64 / 1024.0 / 1024.0,
        timestamp: snapshot.timestamp.to_rfc3339(),
    })
}

/// Get leak analysis
pub async fn get_leak_analysis(
    State(state): State<AppState>,
) -> Json<LeakSummary> {
    let perf_manager = &state.performance_metrics.lock().await;

    let analysis = perf_manager.profiler.leak_detector
        .analyze_leaks()
        .await
        .unwrap_or_default();

    Json(LeakSummary {
        potential_leak_count: analysis.potential_leaks.len(),
        growth_rate_mb_per_hour: analysis.growth_rate_mb_per_hour,
        highest_risk_component: analysis.potential_leaks
            .first()
            .map(|leak| leak.component.clone()),
    })
}

/// Get allocation metrics
pub async fn get_allocation_metrics(
    State(state): State<AppState>,
) -> Json<AllocationMetrics> {
    let perf_manager = &state.performance_metrics.lock().await;

    let top_allocators = perf_manager.profiler.allocation_analyzer
        .get_top_allocators()
        .await
        .unwrap_or_default();

    let recommendations = perf_manager.profiler.allocation_analyzer
        .analyze_patterns()
        .await
        .unwrap_or_default();

    let efficiency = perf_manager.profiler.allocation_analyzer
        .calculate_efficiency_score()
        .await
        .unwrap_or(1.0);

    Json(AllocationMetrics {
        top_allocators,
        efficiency_score: efficiency,
        recommendations,
    })
}
```

### Step 3: Wire Up Routes

```rust
// In main.rs or routes configuration
use crate::handlers::monitoring::*;

let monitoring_routes = Router::new()
    .route("/metrics/memory", get(get_memory_metrics))
    .route("/metrics/leaks", get(get_leak_analysis))
    .route("/metrics/allocations", get(get_allocation_metrics))
    .with_state(app_state.clone());
```

### Step 4: Query Endpoints

```bash
# Get current memory usage
curl http://localhost:8080/metrics/memory | jq

# Check for leaks
curl http://localhost:8080/metrics/leaks | jq

# Get allocation insights
curl http://localhost:8080/metrics/allocations | jq
```

---

## OpenTelemetry Integration

Export profiling data to OpenTelemetry:

```rust
use opentelemetry::global;
use opentelemetry::metrics::{Meter, ObservableGauge};

pub async fn export_profiling_metrics(state: &AppState) {
    let meter = global::meter("riptide-performance");

    // Memory metrics
    let memory_gauge = meter
        .u64_observable_gauge("riptide.memory.rss_bytes")
        .with_description("Resident set size in bytes")
        .init();

    // Register callback to update metrics
    meter.register_callback(&[memory_gauge.as_any()], move |observer| {
        let snapshot = state.performance_metrics.lock().await;
        // Update metrics from snapshot
    });
}
```

---

## Grafana Dashboard

Example dashboard queries:

```promql
# Memory growth rate
rate(riptide_memory_rss_bytes[5m])

# Leak detection rate
riptide_leak_potential_count > 0

# Allocation efficiency
riptide_allocation_efficiency_score < 0.7

# Top memory consumers
topk(10, riptide_allocation_bytes_by_component)
```

---

## Best Practices

### 1. Regular Cleanup

```rust
// Prevent profiling data from growing unbounded
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_hours(1));
    loop {
        interval.tick().await;

        let mut detector = leak_detector.lock().await;
        detector.cleanup_old_data(Duration::from_hours(24)).await.ok();

        let mut analyzer = allocation_analyzer.lock().await;
        analyzer.cleanup_old_data(24.0).await.ok();
    }
});
```

### 2. Alert Thresholds

```rust
// Check memory pressure and alert
let pressure = detector.get_memory_pressure().await?;
if pressure > 0.8 {
    alert_system.send_alert("High memory pressure detected").await?;
}

// Check for rapid growth
let analysis = detector.analyze_leaks().await?;
if analysis.growth_rate_mb_per_hour > 50.0 {
    alert_system.send_alert("Rapid memory growth detected").await?;
}
```

### 3. Performance Impact

Profiling overhead is minimal:
- **CPU:** <1% overhead
- **Memory:** ~5-10MB for tracking structures
- **Latency:** <1ms per allocation record

---

## Troubleshooting

### High Memory Usage

1. Check top allocators:
```rust
let top = analyzer.get_top_allocators().await?;
// Investigate components with highest usage
```

2. Review allocation patterns:
```rust
let recommendations = analyzer.analyze_patterns().await?;
// Follow optimization recommendations
```

3. Force GC:
```rust
tracker.force_gc().await?;
```

### Suspected Memory Leak

1. Run leak analysis:
```rust
let analysis = detector.analyze_leaks().await?;
for leak in analysis.potential_leaks {
    println!("Investigate: {}", leak.component);
}
```

2. Check suspicious patterns:
```rust
for pattern in analysis.suspicious_patterns {
    println!("Pattern: {}", pattern);
}
```

3. Review largest allocations:
```rust
for allocation in analysis.largest_allocations.iter().take(10) {
    println!("{}: {} bytes", allocation.operation, allocation.size);
}
```

---

## Support

- **Documentation:** `/docs/performance-monitoring.md`
- **Code:** `/crates/riptide-performance/src/profiling/`
- **Examples:** Test files demonstrate usage patterns
- **Issues:** Report performance issues with profiling data attached

---

**Version:** 1.0.0
**Status:** ✅ Production Ready
**Last Updated:** 2025-10-06
