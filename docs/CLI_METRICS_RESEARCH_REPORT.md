# CLI Metrics Collection Research Report

**Date:** 2025-10-16
**Researcher:** Research Agent (Hive Mind)
**Objective:** Analyze existing metrics infrastructure and identify requirements for comprehensive CLI metrics collection

---

## Executive Summary

The EventMesh/RipTide codebase has **extensive server-side metrics infrastructure** using Prometheus, but **minimal CLI-level metrics collection**. The CLI currently delegates to API endpoints for metrics display but does not track local command execution, cache performance, or user interaction patterns.

**Key Finding:** We need to build a **lightweight, local-first CLI metrics system** that:
1. Tracks command execution performance
2. Monitors cache hit/miss rates
3. Records engine selection decisions
4. Measures network latencies
5. Tracks WASM initialization times
6. Provides user-facing insights

---

## 1. Existing Metrics Infrastructure

### 1.1 Server-Side Metrics (Prometheus-based)

**Location:** `/workspaces/eventmesh/crates/riptide-api/src/metrics.rs`

**Comprehensive Coverage (70+ metrics):**

#### HTTP & API Metrics
- `http_requests_total` (Counter)
- `http_request_duration_seconds` (Histogram)
- `active_connections` (Gauge)
- `cache_hit_rate` (Gauge)

#### Pipeline Phase Timing
- `fetch_phase_duration_seconds` (Histogram)
- `gate_phase_duration_seconds` (Histogram)
- `wasm_phase_duration_seconds` (Histogram)
- `render_phase_duration_seconds` (Histogram)

#### Gate Decision Metrics (WEEK 1 PHASE 1B)
- `gate_decision_total` (IntCounterVec) - by decision type
- `gate_score_histogram` (Histogram) - score distribution
- `gate_feature_text_ratio` (Histogram)
- `gate_feature_script_density` (Histogram)
- `gate_feature_spa_markers` (IntCounterVec)
- `gate_decision_duration_ms` (Histogram)

#### Extraction Quality Metrics
- `extraction_quality_score` (HistogramVec) - by mode
- `extraction_quality_success_rate` (GaugeVec)
- `extraction_content_length` (HistogramVec)
- `extraction_links_found` (HistogramVec)
- `extraction_images_found` (HistogramVec)
- `extraction_has_author` (IntCounterVec)
- `extraction_has_date` (IntCounterVec)
- `extraction_duration_by_mode` (HistogramVec)
- `extraction_fallback_triggered` (IntCounterVec)

#### WASM Metrics
- `wasm_memory_pages` (Gauge)
- `wasm_grow_failed_total` (Counter)
- `wasm_peak_memory_pages` (Gauge)
- `wasm_cold_start_time_ms` (Gauge)
- `wasm_aot_cache_hits` (Counter) - reserved
- `wasm_aot_cache_misses` (Counter) - reserved

#### PDF Processing
- `pdf_total_processed` (Counter)
- `pdf_total_failed` (Counter)
- `pdf_memory_limit_failures` (Counter)
- `pdf_processing_time_seconds` (Histogram)
- `pdf_peak_memory_mb` (Gauge)
- `pdf_pages_per_pdf` (Gauge)

#### Worker Metrics
- `worker_pool_size` (Gauge)
- `worker_pool_healthy` (Gauge)
- `worker_jobs_submitted_total` (Counter)
- `worker_jobs_completed_total` (Counter)
- `worker_jobs_failed_total` (Counter)
- `worker_jobs_retried_total` (Counter)
- `worker_processing_time_seconds` (Histogram)
- `worker_queue_depth` (Gauge)

#### Streaming Metrics
- `streaming_active_connections` (Gauge)
- `streaming_total_connections` (Gauge)
- `streaming_messages_sent_total` (Counter)
- `streaming_messages_dropped_total` (Counter)
- `streaming_error_rate` (Gauge)
- `streaming_memory_usage_bytes` (Gauge)
- `streaming_connection_duration_seconds` (Histogram)

#### Spider/Crawler Metrics
- `spider_crawls_total` (Counter)
- `spider_pages_crawled_total` (Counter)
- `spider_pages_failed_total` (Counter)
- `spider_active_crawls` (Gauge)
- `spider_frontier_size` (Gauge)
- `spider_crawl_duration_seconds` (Histogram)
- `spider_pages_per_second` (Gauge)

### 1.2 Core Monitoring Module

**Location:** `/workspaces/eventmesh/crates/riptide-core/src/monitoring/metrics.rs`

**Data Structures:**
```rust
pub struct PerformanceMetrics {
    // Timing
    pub avg_extraction_time_ms: f64,
    pub p95_extraction_time_ms: f64,
    pub p99_extraction_time_ms: f64,

    // Throughput
    pub requests_per_second: f64,
    pub successful_extractions: u64,
    pub failed_extractions: u64,

    // Resources
    pub memory_usage_bytes: u64,
    pub cpu_usage_percent: f32,
    pub pool_size: usize,

    // Quality
    pub avg_content_quality_score: f64,
    pub cache_hit_ratio: f64,

    // Errors
    pub error_rate: f64,
    pub timeout_rate: f64,
    pub circuit_breaker_trips: u64,
}
```

### 1.3 Telemetry System

**Location:** `/workspaces/eventmesh/crates/riptide-core/src/telemetry.rs`

**Features:**
- OpenTelemetry integration (OTLP exporter)
- Distributed tracing support
- Sensitive data sanitization (API keys, emails, IPs, SSNs, credit cards)
- SLA monitoring with configurable thresholds
- Resource usage tracking (CPU, memory, network, disk)

**SLA Thresholds:**
```rust
pub struct SlaThreshold {
    pub max_latency_p95: Duration,        // Default: 2000ms
    pub max_latency_p99: Duration,        // Default: 5000ms
    pub min_availability: f64,            // Default: 99.5%
    pub max_error_rate: f64,              // Default: 1.0%
}
```

### 1.4 Cache Metrics

**Location:** `/workspaces/eventmesh/crates/riptide-persistence/src/metrics.rs`

**Cache-Specific Tracking:**
- Hit/Miss counters
- Access time histograms
- Entry size tracking
- Memory usage gauges
- Compression ratio monitoring
- Connection pool metrics

**Location:** `/workspaces/eventmesh/crates/riptide-core/src/cache.rs`

**Cache Implementation:**
- Redis-backed with TTL support
- Version-aware cache keys
- ETag and Last-Modified support
- Content size validation (20MB max)
- Automatic expiration handling

### 1.5 Worker Metrics

**Location:** `/workspaces/eventmesh/crates/riptide-workers/src/metrics.rs`

**Worker-Specific Tracking:**
```rust
pub struct WorkerMetrics {
    pub jobs_submitted: AtomicU64,
    pub jobs_completed: AtomicU64,
    pub jobs_failed: AtomicU64,
    pub jobs_retried: AtomicU64,
    pub jobs_dead_letter: AtomicU64,
    processing_times: Arc<RwLock<Vec<u64>>>,
    queue_sizes: Arc<RwLock<HashMap<String, u64>>>,
    worker_health: Arc<RwLock<HashMap<String, WorkerHealthStatus>>>,
}
```

---

## 2. CLI Current State

### 2.1 Existing CLI Commands

**Location:** `/workspaces/eventmesh/crates/riptide-cli/src/main.rs`

**Available Commands:**
- `extract` - Content extraction
- `render` - Headless rendering
- `crawl` - Web crawling
- `search` - Search operations
- `cache` - Cache management
- `wasm` - WASM operations
- `stealth` - Stealth mode features
- `domain` - Domain-specific operations
- `health` - System health check
- **`metrics`** - Display server metrics (delegated to API)
- `validate` - Validation operations
- `system-check` - System diagnostics
- `tables` - Table extraction
- `schema` - Schema management
- `pdf` - PDF processing
- `job` - Job management
- `session` - Session management

### 2.2 Current Metrics Command

**Location:** `/workspaces/eventmesh/crates/riptide-cli/src/commands/metrics.rs`

**Current Implementation:**
- **API-dependent**: Fetches metrics from `/monitoring/metrics/current` endpoint
- **Display-only**: No local metrics collection
- **Limited scope**: Only shows server-side metrics

**Response Format:**
```rust
struct MetricsResponse {
    requests_total: Option<u64>,
    requests_per_second: Option<f64>,
    average_latency_ms: Option<f64>,
    cache_hit_rate: Option<f64>,
    worker_queue_size: Option<u64>,
}
```

**Export Formats:**
- Prometheus format (`.prom`)
- CSV format (`.csv`)
- JSON format (`.json`)

### 2.3 CLI Timing Measurements

**Limited Local Tracking:**

**Extract Command** (`extract.rs`):
```rust
struct ExtractResponse {
    content: String,
    confidence: Option<f64>,
    method_used: Option<String>,
    extraction_time_ms: Option<u64>,  // ⚠️ Server-reported only
    metadata: Option<serde_json::Value>,
}
```

**Engine Selection** (`extract.rs`):
- Gate decision logic exists for engine selection (Auto/Raw/WASM/Headless)
- Decisions based on HTML characteristics (React/Vue/Angular detection)
- **No metrics tracking** of decision outcomes or success rates

---

## 3. Gaps Analysis: What's Missing

### 3.1 Command Execution Metrics

**NOT TRACKED:**
- ❌ CLI command start/end times
- ❌ Total command execution duration
- ❌ Time spent waiting for API responses
- ❌ Time spent on local WASM extraction
- ❌ File I/O timing (reading HTML from files)
- ❌ Network request/response latencies
- ❌ Command-level success/failure rates

### 3.2 Cache Performance (CLI-Side)

**NOT TRACKED:**
- ❌ Local cache hit/miss counts per command
- ❌ Cache lookup latency from CLI perspective
- ❌ Cache invalidation events
- ❌ Redis connection timing
- ❌ Cache entry size distribution (client view)

### 3.3 Engine Selection Statistics

**NOT TRACKED:**
- ❌ Engine selection decisions (Auto → Raw/WASM/Headless)
- ❌ Gate decision accuracy/success rates
- ❌ Fallback frequency (when primary engine fails)
- ❌ Engine performance comparison per domain
- ❌ User overrides vs. automatic selection

### 3.4 WASM Performance (CLI-Side)

**NOT TRACKED:**
- ❌ WASM module initialization time (cold start)
- ❌ WASM extraction duration (local execution)
- ❌ WASM memory usage (CLI process)
- ❌ WASM compilation/instantiation time
- ❌ WASM cache effectiveness (module reuse)

### 3.5 Network Latency Tracking

**NOT TRACKED:**
- ❌ DNS resolution time
- ❌ TCP connection establishment time
- ❌ TLS handshake duration
- ❌ Time to first byte (TTFB)
- ❌ Total request roundtrip time
- ❌ API endpoint latencies by operation
- ❌ Retry attempts and backoff timing

### 3.6 User Interaction Patterns

**NOT TRACKED:**
- ❌ Command usage frequency (which commands are most used)
- ❌ Output format preferences (JSON/table/text)
- ❌ Error recovery patterns
- ❌ Feature adoption rates
- ❌ Session duration/command sequences

### 3.7 Resource Consumption (CLI Process)

**NOT TRACKED:**
- ❌ CLI process memory usage
- ❌ CLI process CPU usage
- ❌ File descriptor count
- ❌ Thread count for async operations
- ❌ Disk I/O for local operations

---

## 4. Best Practices for CLI Metrics

### 4.1 Rust CLI Metrics Patterns

**Research Findings:**

1. **Lightweight, Local-First**
   - Use in-memory counters (avoid external dependencies)
   - Persist to local storage only when needed
   - JSON/TOML for storage format

2. **Non-Invasive**
   - Metrics collection should not impact UX
   - Async/background collection preferred
   - Graceful degradation if metrics fail

3. **Privacy-Conscious**
   - No PII collection by default
   - Hash URLs/domains before storage
   - Opt-in telemetry transmission
   - Clear user control via config

4. **Standard Rust Crates**
   - `sysinfo` - System resource monitoring (already in use)
   - `chrono` - Timestamps (already in use)
   - `serde_json` - Metrics serialization (already in use)
   - `tokio::time::Instant` - High-precision timing
   - `prometheus-client` - Lightweight Prometheus metrics (optional)

### 4.2 Storage Location

**Recommended Path Structure:**
```
~/.riptide/
  ├── config.toml          # User configuration
  ├── cache/               # Local cache storage
  └── metrics/
      ├── sessions/        # Per-session metrics
      │   └── 2025-10-16-session-abc123.json
      ├── aggregated.json  # Aggregated historical metrics
      └── .gitignore       # Prevent accidental commits
```

**Privacy Controls:**
```toml
# ~/.riptide/config.toml
[metrics]
enabled = true
collect_network_latency = true
collect_cache_stats = true
collect_engine_decisions = true
anonymize_urls = true           # Hash URLs before storage
max_history_days = 30           # Auto-purge old data
telemetry_opt_in = false        # Remote telemetry disabled by default
```

### 4.3 Metrics Data Structure

**Recommended Schema:**
```rust
// Per-command execution metrics
pub struct CommandMetrics {
    pub command: String,                    // e.g., "extract", "crawl"
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_ms: u64,
    pub success: bool,

    // Network metrics
    pub network: Option<NetworkMetrics>,

    // Engine metrics
    pub engine: Option<EngineMetrics>,

    // Cache metrics
    pub cache: Option<CacheMetrics>,

    // WASM metrics
    pub wasm: Option<WasmMetrics>,

    // Resource metrics
    pub resources: Option<ResourceMetrics>,

    // Error info
    pub error: Option<ErrorMetrics>,
}

pub struct NetworkMetrics {
    pub dns_resolution_ms: u64,
    pub tcp_connect_ms: u64,
    pub tls_handshake_ms: u64,
    pub time_to_first_byte_ms: u64,
    pub total_roundtrip_ms: u64,
    pub retry_count: u32,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

pub struct EngineMetrics {
    pub engine_selected: String,        // "raw", "wasm", "headless"
    pub auto_detection_used: bool,
    pub gate_score: f32,
    pub text_ratio: f32,
    pub script_density: f32,
    pub spa_markers_detected: u8,
    pub fallback_triggered: bool,
    pub fallback_reason: Option<String>,
}

pub struct CacheMetrics {
    pub cache_hit: bool,
    pub cache_lookup_ms: u64,
    pub entry_age_seconds: u64,
    pub entry_size_bytes: usize,
}

pub struct WasmMetrics {
    pub cold_start: bool,
    pub initialization_ms: u64,
    pub extraction_ms: u64,
    pub memory_usage_bytes: usize,
    pub peak_memory_bytes: usize,
}

pub struct ResourceMetrics {
    pub cpu_usage_percent: f32,
    pub memory_usage_bytes: u64,
    pub open_file_descriptors: u32,
}

pub struct ErrorMetrics {
    pub error_type: String,
    pub error_message: String,
    pub retry_attempted: bool,
}
```

---

## 5. Implementation Requirements

### 5.1 Key Performance Indicators (KPIs)

**Priority 1: Command Execution**
- Command duration (p50, p95, p99)
- Success rate by command
- Error rate and types

**Priority 2: Cache Performance**
- Cache hit rate (overall and per-domain)
- Cache lookup latency
- Cache invalidation frequency

**Priority 3: Engine Selection**
- Gate decision accuracy
- Engine success rates
- Fallback frequency and reasons

**Priority 4: Network Performance**
- Average TTFB
- DNS resolution time
- API endpoint latencies

**Priority 5: WASM Performance**
- Cold start frequency
- Initialization time
- Extraction speed vs. API extraction

### 5.2 Dependencies Needed

**Already Available:**
- ✅ `chrono` - Timestamps
- ✅ `serde` / `serde_json` - Serialization
- ✅ `tokio` - Async runtime
- ✅ `sysinfo` - System metrics (via telemetry.rs)
- ✅ `anyhow` - Error handling

**New Dependencies:**
- `dirs` - Already in Cargo.toml (for `~/.riptide` path)
- Optional: `prometheus-client` (lightweight Prometheus metrics)

### 5.3 Module Structure

**Proposed File Layout:**
```
crates/riptide-cli/src/
  ├── metrics/
  │   ├── mod.rs              # Public API
  │   ├── collector.rs        # Metrics collection
  │   ├── storage.rs          # Local persistence
  │   ├── types.rs            # Data structures
  │   ├── network.rs          # Network timing
  │   ├── wasm.rs             # WASM timing
  │   ├── engine.rs           # Engine selection tracking
  │   └── aggregator.rs       # Historical aggregation
  ├── commands/
  │   ├── metrics.rs          # Enhanced metrics command
  │   └── ...
  └── main.rs
```

---

## 6. Integration Points

### 6.1 Where to Instrument

**1. Command Entry/Exit** (`main.rs`)
```rust
#[tokio::main]
async fn main() -> Result<()> {
    let metrics = MetricsCollector::new();
    let start = Instant::now();

    // Execute command
    let result = execute_command(&cli.command).await;

    metrics.record_command(
        command_name,
        start.elapsed(),
        result.is_ok(),
    );

    result
}
```

**2. Extract Command** (`commands/extract.rs`)
- Gate decision recording
- Engine selection tracking
- WASM timing (if local extraction)
- API latency (if remote extraction)

**3. Cache Operations** (wherever cache is accessed)
- Hit/miss tracking
- Lookup latency
- Entry metadata

**4. Network Requests** (`client.rs`)
- DNS/TCP/TLS timing via `reqwest` middleware
- Retry tracking
- Bytes transferred

**5. WASM Initialization** (WASM extractor calls)
- Cold start detection
- Module load timing
- Extraction duration

### 6.2 Non-Invasive Integration

**Wrapper Pattern:**
```rust
pub struct MetricsCollector {
    inner: Arc<RwLock<MetricsStorage>>,
    config: MetricsConfig,
}

impl MetricsCollector {
    // Graceful fallback if metrics fail
    pub fn record(&self, metric: CommandMetrics) {
        if let Err(e) = self.try_record(metric) {
            tracing::debug!("Failed to record metric: {}", e);
            // Don't propagate error to user
        }
    }
}
```

---

## 7. Recommendations

### 7.1 Implementation Phases

**Phase 1: Foundation** (Week 1)
- Create metrics module structure
- Implement basic command timing
- Add local storage (JSON files)
- Wire up command entry/exit points

**Phase 2: Network & Cache** (Week 2)
- Network latency tracking
- Cache hit/miss recording
- DNS/TCP/TLS breakdown

**Phase 3: Engine & WASM** (Week 3)
- Gate decision tracking
- Engine selection metrics
- WASM initialization timing
- WASM extraction performance

**Phase 4: Aggregation & Display** (Week 4)
- Historical aggregation
- Enhanced `metrics` command UI
- Export formats (CSV, JSON, Prometheus)
- Performance insights/recommendations

### 7.2 User Experience

**Default Behavior:**
- Metrics collection enabled by default
- No visible performance impact
- Automatic cleanup (30-day retention)

**User Controls:**
```bash
# View collected metrics
riptide metrics show

# View specific metric categories
riptide metrics show --category=cache
riptide metrics show --category=network
riptide metrics show --category=wasm

# Export metrics
riptide metrics export --format=json --output=metrics.json
riptide metrics export --format=prometheus --output=metrics.prom

# Clear metrics
riptide metrics clear --older-than=7d

# Disable metrics collection
riptide config set metrics.enabled false
```

### 7.3 Privacy Considerations

1. **URL Anonymization:**
   - Hash URLs with SHA-256 before storage
   - Only store domain (not full path) if needed

2. **No PII:**
   - Never log request/response bodies
   - Sanitize error messages (remove API keys, tokens)

3. **Local-First:**
   - All metrics stored locally by default
   - Remote telemetry requires explicit opt-in

4. **User Control:**
   - Easy disable/enable via config
   - Clear data purge mechanism

---

## 8. Technical Considerations

### 8.1 Performance Impact

**Metrics Collection Overhead:**
- In-memory operations: < 1μs per metric
- JSON serialization: < 10ms per command
- File I/O: Async, non-blocking

**Storage Size:**
- Per-command: ~200-500 bytes JSON
- Daily usage (100 commands): ~50KB
- Monthly retention: ~1.5MB

### 8.2 Error Handling

**Graceful Degradation:**
```rust
pub fn record_metric<T>(
    &self,
    operation: impl FnOnce() -> Result<T>,
) -> Result<T> {
    let start = Instant::now();
    let result = operation();

    // Metrics failure doesn't affect command execution
    let _ = self.try_record(start.elapsed(), result.is_ok());

    result
}
```

### 8.3 Thread Safety

**Concurrent Access:**
- Use `Arc<RwLock<MetricsStorage>>` for shared state
- Atomic counters for high-frequency metrics
- Batch writes to reduce lock contention

---

## 9. Open Questions

1. **Should we track per-domain statistics?**
   - Pros: Identify problematic domains, optimize per-site
   - Cons: Privacy concerns, storage overhead

2. **Export to Prometheus vs. local-only?**
   - Prometheus export enables monitoring dashboards
   - Requires opt-in for remote transmission

3. **Metrics retention policy?**
   - Default: 30 days
   - Configurable: 7/30/90 days or unlimited

4. **Should we include system-wide metrics (CPU/memory)?**
   - Useful for resource usage tracking
   - May be redundant with system monitoring tools

---

## 10. Conclusion

**Summary:**
- ✅ Extensive server-side metrics exist (Prometheus-based, 70+ metrics)
- ❌ CLI lacks local metrics collection
- 🎯 Need lightweight, privacy-conscious CLI metrics system
- 📊 Focus on command timing, cache, network, WASM, engine selection

**Next Steps:**
1. Design metrics data structures (`types.rs`)
2. Implement collector module (`collector.rs`)
3. Add local storage (`storage.rs`)
4. Wire up command instrumentation
5. Enhance `metrics` command for display/export

**Success Criteria:**
- < 5ms overhead per command
- Zero UX impact
- Privacy-preserving (hashed URLs, local storage)
- Actionable insights for users
- Integration with existing Prometheus infrastructure (optional)

---

**Research Completed By:** Research Agent
**Report Generated:** 2025-10-16
**Session:** Hive Mind CLI Metrics Investigation
