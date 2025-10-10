# Sprint 2A: Performance Profiling Integration - Architecture Design

## Executive Summary

This document outlines the architectural design for integrating the `riptide-performance` crate into the `riptide-api` service, providing comprehensive performance profiling, memory analysis, and bottleneck detection capabilities.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         riptide-api                              │
│                                                                   │
│  ┌────────────────────────────────────────────────────────┐     │
│  │              HTTP Endpoints Layer                        │     │
│  │  /api/profiling/memory                                  │     │
│  │  /api/profiling/cpu                                     │     │
│  │  /api/profiling/bottlenecks                            │     │
│  │  /api/profiling/allocations                            │     │
│  │  /api/profiling/snapshot                               │     │
│  │  /api/profiling/leak-detection                         │     │
│  └────────────────────────────────────────────────────────┘     │
│                          ↓                                        │
│  ┌────────────────────────────────────────────────────────┐     │
│  │         handlers/profiling.rs                          │     │
│  │  - get_memory_profile()                                │     │
│  │  - get_cpu_profile()                                   │     │
│  │  - get_bottleneck_analysis()                           │     │
│  │  - get_allocation_metrics()                            │     │
│  │  - trigger_leak_detection()                            │     │
│  │  - trigger_heap_snapshot()                             │     │
│  └────────────────────────────────────────────────────────┘     │
│                          ↓                                        │
│  ┌────────────────────────────────────────────────────────┐     │
│  │            AppState (with PerformanceManager)          │     │
│  └────────────────────────────────────────────────────────┘     │
└───────────────────────────┬───────────────────────────────────────┘
                            │
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│                    riptide-performance                           │
│                                                                   │
│  ┌────────────────────────────────────────────────────────┐     │
│  │             PerformanceManager                          │     │
│  │  - profiler: MemoryProfiler                            │     │
│  │  - monitor: PerformanceMonitor                         │     │
│  │  - optimizer: CacheOptimizer                           │     │
│  │  - limiter: ResourceLimiter                            │     │
│  └────────────────────────────────────────────────────────┘     │
│                          ↓                                        │
│  ┌────────────────────────────────────────────────────────┐     │
│  │        profiling/ Module (Core Features)                │     │
│  │  - memory.rs: MemoryProfiler                           │     │
│  │  - cpu.rs: CpuProfiler (dev builds only)              │     │
│  │  - bottleneck.rs: BottleneckAnalyzer                   │     │
│  │  - leak_detector.rs: LeakDetector                      │     │
│  │  - allocation_analyzer.rs: AllocationAnalyzer          │     │
│  │  - memory_tracker.rs: MemoryTracker                    │     │
│  │  - flamegraph_generator.rs: FlamegraphGenerator        │     │
│  └────────────────────────────────────────────────────────┘     │
│                          ↓                                        │
│  ┌────────────────────────────────────────────────────────┐     │
│  │           jemalloc Integration Layer                    │     │
│  │  - jemalloc-ctl for memory statistics                  │     │
│  │  - Heap profiling and allocation tracking              │     │
│  │  - Memory arena management                              │     │
│  └────────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────┘
```

## Integration Points

### 1. Dependency Management (Cargo.toml)

**File**: `/workspaces/eventmesh/crates/riptide-api/Cargo.toml`

**Changes**:
```toml
[dependencies]
# Add jemalloc feature flag
riptide-performance = { path = "../riptide-performance", features = ["jemalloc"] }

# Add jemalloc allocator
[target.'cfg(not(target_env = "msvc"))'.dependencies]
tikv-jemallocator = "0.5"
```

**Rationale**:
- Enable jemalloc feature for production memory monitoring
- Use tikv-jemallocator for cross-platform compatibility
- Conditional compilation for non-MSVC targets

### 2. Handler Module (handlers/profiling.rs)

**File**: `/workspaces/eventmesh/crates/riptide-api/src/handlers/profiling.rs`

**Structure**:
```rust
pub async fn get_memory_profile(State) -> Result<Json<MemoryProfileResponse>>
pub async fn get_cpu_profile(State) -> Result<Json<CpuProfileResponse>>
pub async fn get_bottleneck_analysis(State) -> Result<Json<BottleneckResponse>>
pub async fn get_allocation_metrics(State) -> Result<Json<AllocationResponse>>
pub async fn trigger_leak_detection(State) -> Result<Json<LeakDetectionResponse>>
pub async fn trigger_heap_snapshot(State) -> Result<Json<SnapshotResponse>>
```

**Response Models**:
- `MemoryProfileResponse`: RSS, heap, virtual memory metrics
- `CpuProfileResponse`: CPU usage, load averages (dev builds only)
- `BottleneckResponse`: Performance hotspots with impact scores
- `AllocationResponse`: Top allocators and size distribution
- `LeakDetectionResponse`: Potential leaks with severity
- `SnapshotResponse`: Heap snapshot metadata and download URL

### 3. Router Integration (main.rs)

**File**: `/workspaces/eventmesh/crates/riptide-api/src/main.rs`

**Routes** (lines 318-330):
```rust
// Performance profiling endpoints
.route("/api/profiling/memory", get(handlers::profiling::get_memory_profile))
.route("/api/profiling/cpu", get(handlers::profiling::get_cpu_profile))
.route("/api/profiling/bottlenecks", get(handlers::profiling::get_bottleneck_analysis))
.route("/api/profiling/allocations", get(handlers::profiling::get_allocation_metrics))
.route("/api/profiling/leak-detection", post(handlers::profiling::trigger_leak_detection))
.route("/api/profiling/snapshot", post(handlers::profiling::trigger_heap_snapshot))
```

**Rationale**:
- GET for read-only profiling data
- POST for triggered operations (leak detection, snapshots)
- Consistent `/api/profiling/*` path structure

### 4. Application State (state.rs)

**File**: `/workspaces/eventmesh/crates/riptide-api/src/state.rs`

**Changes**:
```rust
pub struct AppState {
    // Existing fields...
    pub performance_manager: Arc<riptide_performance::PerformanceManager>,
}

impl AppState {
    pub async fn new(...) -> Result<Self> {
        // Initialize performance manager with default targets
        let performance_manager = Arc::new(
            riptide_performance::PerformanceManager::new()?
        );

        // Start background profiling
        performance_manager.start_monitoring().await?;

        Ok(Self {
            // ...existing fields
            performance_manager,
        })
    }
}
```

**Rationale**:
- Centralized performance manager in application state
- Automatic startup of background profiling
- Shared access across all handlers via Arc

### 5. jemalloc Configuration (main.rs)

**File**: `/workspaces/eventmesh/crates/riptide-api/src/main.rs`

**Global Allocator** (add at top of file):
```rust
#[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
use tikv_jemallocator::Jemalloc;

#[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
```

**Startup Monitoring** (in main function):
```rust
#[cfg(feature = "jemalloc")]
{
    tracing::info!("jemalloc allocator enabled for memory profiling");

    // Log initial memory stats
    if let Ok(allocated) = tikv_jemalloc_ctl::stats::allocated::read() {
        tracing::info!(
            memory_allocated_bytes = allocated,
            memory_allocated_mb = allocated as f64 / 1024.0 / 1024.0,
            "Initial jemalloc memory allocation"
        );
    }
}
```

## API Endpoints Specification

### GET /api/profiling/memory

**Description**: Get current memory usage metrics

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

### GET /api/profiling/cpu

**Description**: Get CPU usage metrics (dev builds only)

**Response**:
```json
{
  "timestamp": "2025-10-10T18:00:00Z",
  "cpu_usage_percent": 23.5,
  "user_time_percent": 18.2,
  "system_time_percent": 5.3,
  "load_average": {
    "one_min": 0.45,
    "five_min": 0.38,
    "fifteen_min": 0.32
  }
}
```

### GET /api/profiling/bottlenecks

**Description**: Get detected performance bottlenecks

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
  "cpu_bound_percent": 60.0,
  "io_bound_percent": 25.0,
  "memory_bound_percent": 15.0,
  "recommendations": [
    "Critical: Optimize riptide_core::spider::crawl (25.3% CPU time, impact score: 0.85)"
  ]
}
```

### GET /api/profiling/allocations

**Description**: Get allocation pattern analysis

**Response**:
```json
{
  "timestamp": "2025-10-10T18:00:00Z",
  "top_allocators": [
    ["riptide_html::parse_document", 45678912],
    ["tokio::task::spawn", 23456789]
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

### POST /api/profiling/leak-detection

**Description**: Trigger memory leak analysis

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

### POST /api/profiling/snapshot

**Description**: Trigger heap snapshot for deep analysis

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

## Performance Overhead Analysis

### Measurement Methodology

1. **Baseline Measurement**: Run API without profiling
2. **With Profiling**: Run API with profiling enabled
3. **Metrics**: Latency (p50, p95, p99), throughput, memory overhead

### Expected Overhead

| Component | Overhead Target | Actual (Est.) |
|-----------|-----------------|---------------|
| Memory sampling | <0.5% CPU | ~0.3% |
| Allocation tracking | <1% latency | ~0.7% |
| Leak detection | <0.3% memory | ~0.2% |
| jemalloc overhead | <1% total | ~0.5% |
| **Total** | **<2%** | **<1.7%** |

### Optimization Strategies

1. **Sampling-based profiling**: Only sample every N allocations
2. **Async collection**: Background tasks for profiling
3. **Lazy initialization**: Start profiling only when needed
4. **Ring buffers**: Fixed-size allocation tracking
5. **Conditional compilation**: Dev-only features (CPU profiling, flamegraphs)

## Testing Strategy

### 1. Unit Tests

**File**: `/workspaces/eventmesh/crates/riptide-api/tests/profiling_tests.rs`

**Coverage**:
- Handler response formats
- Error handling
- State initialization
- Endpoint routing

### 2. Integration Tests

**File**: `/workspaces/eventmesh/crates/riptide-api/tests/profiling_integration_tests.rs`

**Scenarios**:
- Memory profiling during high load
- Leak detection with simulated leaks
- Bottleneck analysis under stress
- jemalloc allocator validation
- Performance overhead measurement

### 3. Soak Tests

**Duration**: 24 hours

**Monitoring**:
- Memory growth rate
- Leak detection accuracy
- Performance degradation
- System stability

**Success Criteria**:
- Memory growth < 5MB/hour
- No false positive leaks
- <2% performance overhead
- Zero crashes or panics

## Documentation

### 1. API Guide

**File**: `/workspaces/eventmesh/docs/api/profiling.md`

**Content**:
- Endpoint descriptions
- Request/response examples
- Authentication requirements
- Rate limiting
- Best practices

### 2. Profiling Guide

**File**: `/workspaces/eventmesh/docs/guides/profiling.md`

**Content**:
- How to read profiling output
- Interpreting memory snapshots
- Leak detection workflow
- Performance optimization tips
- Troubleshooting common issues

### 3. Operations Manual

**File**: `/workspaces/eventmesh/docs/operations/profiling.md`

**Content**:
- Enabling profiling in production
- Monitoring dashboards
- Alert configuration
- Performance budgets
- Incident response

## Deployment Considerations

### Feature Flags

```toml
[features]
default = ["jemalloc"]
jemalloc = ["riptide-performance/jemalloc"]
profiling-full = ["jemalloc", "riptide-performance/bottleneck-analysis-full"]
```

**Recommendation**: Use `default` for production, `profiling-full` for development

### Environment Variables

```bash
# Enable profiling
ENABLE_PROFILING=true

# Configure sampling intervals
PROFILING_MEMORY_INTERVAL_SECS=5
PROFILING_LEAK_CHECK_INTERVAL_SECS=300

# Set memory thresholds
PROFILING_MEMORY_WARNING_MB=650
PROFILING_MEMORY_ALERT_MB=700

# jemalloc tuning
MALLOC_CONF="background_thread:true,narenas:4,dirty_decay_ms:10000"
```

### Monitoring Integration

- **Prometheus**: Export profiling metrics
- **Grafana**: Dashboards for memory trends
- **Alertmanager**: Alerts for memory leaks
- **Jaeger**: Trace profiling overhead

## Migration Path

### Phase 1: Infrastructure (Week 1)
1. Add dependencies and feature flags
2. Configure jemalloc allocator
3. Initialize PerformanceManager in AppState
4. Basic health checks

### Phase 2: Core Features (Week 2)
1. Implement profiling handlers
2. Wire up API endpoints
3. Integration testing
4. Performance measurement

### Phase 3: Advanced Features (Week 3)
1. Automated leak detection
2. Bottleneck analysis
3. Heap snapshots
4. Flamegraph generation

### Phase 4: Production Hardening (Week 4)
1. 24-hour soak tests
2. Documentation completion
3. Operations runbooks
4. Monitoring dashboards

## Success Criteria

### Functional Requirements
- ✅ All profiling endpoints operational
- ✅ jemalloc integration complete
- ✅ Memory tracking accurate within 5%
- ✅ Leak detection with <10% false positives
- ✅ Bottleneck analysis identifies top 10 hotspots

### Non-Functional Requirements
- ✅ Performance overhead <2%
- ✅ Memory overhead <50MB
- ✅ API response time <100ms
- ✅ Zero crashes under load
- ✅ 24-hour stability test passed

### Documentation Requirements
- ✅ API documentation complete
- ✅ Profiling guide published
- ✅ Operations manual complete
- ✅ Troubleshooting guide available

## Risk Assessment

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Performance overhead exceeds 2% | High | Low | Optimize sampling, use ring buffers |
| jemalloc compatibility issues | Medium | Low | Fallback to system allocator |
| False positive leak detection | Medium | Medium | Tune detection thresholds |
| Memory growth from profiling | High | Low | Implement cleanup routines |
| API endpoint abuse | Low | Medium | Rate limiting, authentication |

## Appendix: Architecture Decision Records

### ADR-001: jemalloc vs System Allocator

**Decision**: Use jemalloc with tikv-jemallocator

**Rationale**:
- Better memory statistics
- Cross-platform compatibility
- Production-tested by TiKV
- Minimal overhead (<1%)

**Alternatives Considered**:
- System allocator: No profiling capabilities
- mimalloc: Less mature Rust bindings
- tcmalloc: Licensing concerns

### ADR-002: Profiling API Location

**Decision**: New `/api/profiling/*` namespace

**Rationale**:
- Clear separation of concerns
- Easy to secure separately
- Consistent with REST conventions
- Future-proof for expansion

**Alternatives Considered**:
- Extend `/monitoring/*`: Too generic
- Use `/debug/*`: Implies dev-only
- Separate port: Added complexity

### ADR-003: Profiling Storage

**Decision**: In-memory with optional persistence

**Rationale**:
- Low latency access
- No disk I/O overhead
- Configurable retention
- On-demand snapshots

**Alternatives Considered**:
- Always persist to disk: Too slow
- Redis: Added dependency
- Database: Overkill for profiling data
