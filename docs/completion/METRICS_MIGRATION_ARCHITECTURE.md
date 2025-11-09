# Metrics Migration Architecture - Sprint 4.5

## Executive Summary

Migrate from deprecated `RipTideMetrics` (single monolithic struct) to split metrics architecture:
- **BusinessMetrics** (facade layer) - domain operations
- **TransportMetrics** (API layer) - HTTP/WebSocket protocols
- **CombinedMetrics** (integration) - unified `/metrics` endpoint

**Status:** 257 deprecation warnings across codebase (68% in metrics.rs core)

---

## Current State Analysis

### AppState Structure (`crates/riptide-api/src/state.rs`)

```rust
pub struct AppState {
    // DEPRECATED (line 87-93)
    #[deprecated(since = "4.5.0")]
    pub metrics: Arc<RipTideMetrics>,         // ← OLD: Monolithic

    // NEW SPLIT METRICS (lines 95-105)
    pub business_metrics: Arc<BusinessMetrics>,    // ← Facade layer
    pub transport_metrics: Arc<TransportMetrics>,  // ← API layer
    pub combined_metrics: Arc<CombinedMetrics>,    // ← Unified endpoint
}
```

**Problem:** Most code still uses `state.metrics.*` (deprecated API).

### Deprecated API Usage Patterns

**Pattern 1: HTTP Request Recording** (24 occurrences)
```rust
// OLD (deprecated):
state.metrics.record_http_request("GET", "/api/extract", 200, 0.150);

// NEW (transport metrics):
state.transport_metrics.record_http_request("GET", "/api/extract", 200, 0.150);
// OR via combined:
state.combined_metrics.record_http_request("GET", "/api/extract", 200, 0.150);
```

**Pattern 2: Gate Decision Recording** (15 occurrences in pipeline.rs)
```rust
// OLD (deprecated):
state.metrics.record_gate_decision("raw");

// NEW (business metrics):
state.business_metrics.record_gate_decision("raw");
// OR via combined:
state.combined_metrics.record_gate_decision("raw");
```

**Pattern 3: Phase Timing** (12 occurrences in pipeline.rs/pipeline_enhanced.rs)
```rust
// OLD (deprecated):
state.metrics.record_phase_timing(PhaseType::Fetch, 0.123);

// NEW (business metrics):
state.business_metrics.fetch_phase_duration.observe(0.123);
```

**Pattern 4: PDF Processing** (8 occurrences)
```rust
// OLD (deprecated):
state.metrics.record_pdf_processing_success(duration, pages, memory_mb);

// NEW (business metrics):
state.business_metrics.record_pdf_processing_success(duration, pages, memory_mb);
// OR via combined:
state.combined_metrics.record_pdf_processing_success(duration, pages, memory_mb);
```

**Pattern 5: Spider/Worker Metrics** (6 occurrences in handlers/workers.rs)
```rust
// OLD (deprecated):
state.metrics.update_worker_stats(&stats);

// NEW (business metrics):
state.business_metrics.update_worker_stats(&stats);
```

**Pattern 6: Streaming Metrics** (10 occurrences)
```rust
// OLD (deprecated):
state.metrics.update_streaming_metrics(&streaming_metrics);

// NEW (transport metrics):
state.transport_metrics.update_streaming_metrics(&streaming_metrics);
// OR via combined:
state.combined_metrics.update_streaming_metrics(&streaming_metrics);
```

**Pattern 7: Jemalloc Stats** (2 occurrences in handlers/resources.rs)
```rust
// OLD (deprecated):
state.metrics.update_jemalloc_stats();

// NEW (transport metrics):
state.transport_metrics.update_jemalloc_stats();
// OR via combined:
state.combined_metrics.update_jemalloc_stats();
```

---

## Target Architecture

### Metric Ownership by Layer

| Metric Category | Current Location | New Owner | Rationale |
|----------------|------------------|-----------|-----------|
| **HTTP Requests/Responses** | RipTideMetrics | TransportMetrics | Protocol-level transport |
| **WebSocket/SSE Connections** | RipTideMetrics | TransportMetrics | Protocol-level transport |
| **Streaming Messages** | RipTideMetrics | TransportMetrics | Protocol-level transport |
| **Jemalloc Memory** | RipTideMetrics | TransportMetrics | System resource tracking |
| **Gate Decisions** | RipTideMetrics | BusinessMetrics | Business logic |
| **Extraction Quality** | RipTideMetrics | BusinessMetrics | Business outcome |
| **PDF Processing** | RipTideMetrics | BusinessMetrics | Business operation |
| **Spider Crawling** | RipTideMetrics | BusinessMetrics | Business operation |
| **WASM Execution** | RipTideMetrics | BusinessMetrics | Business operation |
| **Worker Jobs** | RipTideMetrics | BusinessMetrics | Business operation |
| **Cache Hit Rate** | RipTideMetrics | BusinessMetrics | Business efficiency |
| **Phase Timing** | RipTideMetrics | BusinessMetrics | Business pipeline |

### New AppState Metric Access

```rust
impl AppState {
    // RECOMMENDED: Use combined_metrics for most operations
    // It delegates to the appropriate underlying metric system

    // Business operations:
    fn record_gate_decision(&self, decision: &str) {
        self.combined_metrics.record_gate_decision(decision);
    }

    // Transport operations:
    fn record_http_request(&self, method: &str, path: &str, status: u16, duration: f64) {
        self.combined_metrics.record_http_request(method, path, status, duration);
    }

    // Direct access for specialized cases:
    fn get_business_stats(&self) -> BusinessStats {
        // Use business_metrics directly when you need business-specific details
        BusinessStats {
            gate_raw: self.business_metrics.gate_decisions_raw.get(),
            gate_wasm: self.business_metrics.gate_decisions_probes_first.get(),
            // ...
        }
    }
}
```

### CombinedMetrics API

**Role:** Unified facade over BusinessMetrics + TransportMetrics

```rust
impl CombinedMetrics {
    // Delegates to BusinessMetrics:
    pub fn record_gate_decision(&self, decision: &str);
    pub fn record_extraction_result(...);
    pub fn record_pdf_processing_success(...);

    // Delegates to TransportMetrics:
    pub fn record_http_request(method, path, status, duration);
    pub fn update_streaming_metrics(...);
    pub fn update_jemalloc_stats();

    // Unified export:
    pub fn gather_all() -> Vec<MetricFamily>;  // Both registries
    pub fn export_text_format() -> String;      // Prometheus format
}
```

---

## Migration Phases

### Phase A: AppState Helper Methods (NON-BREAKING)
**Goal:** Add migration helper methods without breaking existing code
**Duration:** 1 hour
**Risk:** LOW (additive only)

**Files to Modify:**
1. `crates/riptide-api/src/state.rs`

**Changes:**
```rust
impl AppState {
    // NEW: Transitional helper methods

    /// Record HTTP request (migrated to transport metrics)
    pub fn record_http_request(&self, method: &str, path: &str, status: u16, duration: f64) {
        self.combined_metrics.record_http_request(method, path, status, duration);
    }

    /// Record gate decision (migrated to business metrics)
    pub fn record_gate_decision(&self, decision: &str) {
        self.combined_metrics.record_gate_decision(decision);
    }

    /// Record PDF processing success (migrated to business metrics)
    pub fn record_pdf_processing_success(&self, duration: f64, pages: u32, memory_mb: f64) {
        self.combined_metrics.record_pdf_processing_success(duration, pages, memory_mb);
    }

    /// Update streaming metrics (migrated to transport metrics)
    pub fn update_streaming_metrics(&self, metrics: &crate::streaming::GlobalStreamingMetrics) {
        self.combined_metrics.update_streaming_metrics(metrics);
    }

    /// Update jemalloc stats (migrated to transport metrics)
    #[cfg(feature = "jemalloc")]
    pub fn update_jemalloc_stats(&self) {
        self.combined_metrics.update_jemalloc_stats();
    }

    /// Update worker stats (migrated to business metrics)
    #[cfg(feature = "workers")]
    pub fn update_worker_stats(&self, stats: &riptide_workers::WorkerPoolStats) {
        self.business_metrics.update_worker_stats(stats);
    }

    /// Update worker metrics (migrated to business metrics)
    #[cfg(feature = "workers")]
    pub fn update_worker_metrics(&self, metrics: &riptide_workers::WorkerMetricsSnapshot) {
        self.business_metrics.update_worker_metrics(metrics);
    }

    /// Record spider crawl completion (migrated to business metrics)
    pub fn record_spider_crawl_completion(&self, pages: u64, failed: u64, duration: f64) {
        self.business_metrics.record_spider_crawl_completion(pages, failed, duration);
    }

    /// Update spider frontier size (migrated to business metrics)
    pub fn update_spider_frontier_size(&self, size: usize) {
        self.business_metrics.update_spider_frontier_size(size);
    }
}
```

**Testing:**
```bash
cargo check -p riptide-api
cargo test -p riptide-api --lib state
```

**Commit:** `feat(metrics): add AppState helper methods for metrics migration (Phase A)`

---

### Phase B: Pipeline Migration
**Goal:** Update pipeline.rs and pipeline_enhanced.rs to use new metrics
**Duration:** 2 hours
**Risk:** MEDIUM (core business logic)

**Files to Modify:**
1. `crates/riptide-api/src/pipeline.rs` (15 usages)
2. `crates/riptide-api/src/pipeline_enhanced.rs` (12 usages)

**Change Pattern:**

**Before:**
```rust
// pipeline.rs line ~450
self.state.metrics.record_gate_decision(&gate_decision_str);

// pipeline.rs line ~680
self.state.metrics.record_pdf_processing_success(duration, pages, memory_mb);

// pipeline.rs line ~750
let metrics = self.state.metrics.clone();
metrics.record_phase_timing(PhaseType::Fetch, duration);
```

**After:**
```rust
// Use AppState helper methods
self.state.record_gate_decision(&gate_decision_str);

self.state.record_pdf_processing_success(duration, pages, memory_mb);

// Phase timing: Direct access to business_metrics
self.state.business_metrics.fetch_phase_duration.observe(duration);
```

**Critical Dependencies:**
- Phase A must be complete first (helpers exist)
- Tests must pass: `cargo test -p riptide-api pipeline`

**Testing Strategy:**
```bash
# Unit tests
cargo test -p riptide-api test_pipeline
cargo test -p riptide-api test_enhanced_pipeline

# Integration tests (if available)
cargo test -p riptide-api --test '*' -- pipeline

# Clippy validation
cargo clippy -p riptide-api --tests -- -D warnings
```

**Commit:** `feat(metrics): migrate pipeline.rs to split metrics (Phase B)`

---

### Phase C: Handler Migration
**Goal:** Update all handlers to use new metrics
**Duration:** 3 hours
**Risk:** MEDIUM (many files, high impact)

**Files to Modify (by category):**

**C1: HTTP Handlers** (24 usages)
- `crates/riptide-api/src/handlers/tables.rs`
- `crates/riptide-api/src/handlers/llm.rs` (5 occurrences)
- `crates/riptide-api/src/handlers/utils.rs` (2 occurrences)

**Change:**
```rust
// OLD:
state.metrics.record_http_request("GET", "/api/tables", 200, duration);

// NEW:
state.record_http_request("GET", "/api/tables", 200, duration);
```

**C2: Worker Handlers** (6 usages)
- `crates/riptide-api/src/handlers/workers.rs`

**Change:**
```rust
// OLD:
state.metrics.record_worker_job_submission();
state.metrics.update_worker_stats(&s);
state.metrics.update_worker_metrics(&m);

// NEW:
state.business_metrics.record_worker_job_submission();
state.update_worker_stats(&s);
state.update_worker_metrics(&m);
```

**C3: Resource/Jemalloc Handlers** (2 usages)
- `crates/riptide-api/src/handlers/resources.rs`

**Change:**
```rust
// OLD:
state.metrics.update_jemalloc_stats();

// NEW:
state.update_jemalloc_stats();
```

**C4: Spider Handlers** (3 usages)
- `crates/riptide-api/src/handlers/shared/mod.rs`

**Change:**
```rust
// OLD:
self.state.metrics.record_spider_crawl_completion(pages, failed, duration);
self.state.metrics.spider_active_crawls.dec();
self.state.metrics.update_spider_frontier_size(size);

// NEW:
self.state.record_spider_crawl_completion(pages, failed, duration);
self.state.business_metrics.spider_active_crawls.dec();
self.state.update_spider_frontier_size(size);
```

**Testing Strategy:**
```bash
# Per handler file:
cargo test -p riptide-api handlers::tables
cargo test -p riptide-api handlers::llm
cargo test -p riptide-api handlers::workers
cargo test -p riptide-api handlers::resources
cargo test -p riptide-api handlers::shared

# Full handler suite:
cargo test -p riptide-api --lib handlers
```

**Commit:** `feat(metrics): migrate handlers to split metrics (Phase C.1-4)`

---

### Phase D: Cleanup Deprecated Code
**Goal:** Remove deprecated RipTideMetrics and fix remaining references
**Duration:** 2 hours
**Risk:** HIGH (breaking change)

**D1: Remove from AppState**

**File:** `crates/riptide-api/src/state.rs`

**Before:**
```rust
pub struct AppState {
    #[deprecated(since = "4.5.0")]
    pub metrics: Arc<RipTideMetrics>,  // ← REMOVE THIS

    pub business_metrics: Arc<BusinessMetrics>,
    pub transport_metrics: Arc<TransportMetrics>,
    pub combined_metrics: Arc<CombinedMetrics>,
}
```

**After:**
```rust
pub struct AppState {
    // REMOVED: deprecated metrics field
    // Use business_metrics, transport_metrics, or combined_metrics instead

    pub business_metrics: Arc<BusinessMetrics>,
    pub transport_metrics: Arc<TransportMetrics>,
    pub combined_metrics: Arc<CombinedMetrics>,
}
```

**D2: Update AppState::new_base()**

**File:** `crates/riptide-api/src/state.rs` (lines 624-658)

**Before:**
```rust
pub async fn new_base(
    config: AppConfig,
    api_config: RiptideApiConfig,
    metrics: Arc<RipTideMetrics>,  // ← DEPRECATED PARAM
    health_checker: Arc<HealthChecker>,
    telemetry: Option<Arc<TelemetrySystem>>,
) -> Result<Self> {
    // ... initialization ...
    Ok(Self {
        #[allow(deprecated)]
        metrics,  // ← REMOVE THIS
        business_metrics,
        transport_metrics,
        combined_metrics,
        // ... other fields ...
    })
}
```

**After:**
```rust
pub async fn new_base(
    config: AppConfig,
    api_config: RiptideApiConfig,
    // REMOVED: metrics parameter (now created internally)
    health_checker: Arc<HealthChecker>,
    telemetry: Option<Arc<TelemetrySystem>>,
) -> Result<Self> {
    // Create split metrics internally
    let business_metrics = Arc::new(
        BusinessMetrics::new()
            .context("Failed to initialize business metrics")?
    );

    let transport_metrics = Arc::new(
        TransportMetrics::new()
            .context("Failed to initialize transport metrics")?
    );

    let combined_metrics = Arc::new(
        CombinedMetrics::new(business_metrics.clone(), transport_metrics.clone())
            .context("Failed to create combined metrics collector")?
    );

    // ... rest of initialization ...

    Ok(Self {
        // REMOVED: metrics field
        business_metrics,
        transport_metrics,
        combined_metrics,
        // ... other fields ...
    })
}
```

**D3: Update AppState::new() and new_with_facades()**

**File:** `crates/riptide-api/src/state.rs` (lines 624-657)

**Before:**
```rust
pub async fn new(
    config: AppConfig,
    metrics: Arc<RipTideMetrics>,  // ← REMOVE
    health_checker: Arc<HealthChecker>,
) -> Result<Self> {
    Self::new_with_facades(config, metrics, health_checker, None).await
}

pub async fn new_with_facades(
    config: AppConfig,
    metrics: Arc<RipTideMetrics>,  // ← REMOVE
    health_checker: Arc<HealthChecker>,
    telemetry: Option<Arc<TelemetrySystem>>,
) -> Result<Self> {
    let api_config = RiptideApiConfig::from_env();
    let base_state = Self::new_base(config, api_config, metrics, health_checker, telemetry).await?;
    base_state.with_facades().await
}
```

**After:**
```rust
pub async fn new(
    config: AppConfig,
    health_checker: Arc<HealthChecker>,
) -> Result<Self> {
    Self::new_with_facades(config, health_checker, None).await
}

pub async fn new_with_facades(
    config: AppConfig,
    health_checker: Arc<HealthChecker>,
    telemetry: Option<Arc<TelemetrySystem>>,
) -> Result<Self> {
    let api_config = RiptideApiConfig::from_env();
    let base_state = Self::new_base(config, api_config, health_checker, telemetry).await?;
    base_state.with_facades().await
}
```

**D4: Remove metrics.rs (deprecated module)**

**Action:** Keep the file but mark entire module as deprecated for backward compatibility:

**File:** `crates/riptide-api/src/metrics.rs` (lines 1-44)

Update module-level deprecation:
```rust
#![deprecated(
    since = "4.5.0",
    note = "Use BusinessMetrics + TransportMetrics + CombinedMetrics instead. This module will be removed in v5.0.0"
)]
```

**D5: Update main.rs and integration points**

**File:** `crates/riptide-api/src/main.rs` (likely location)

**Before:**
```rust
let metrics = Arc::new(RipTideMetrics::new()?);
let state = AppState::new(config, metrics, health_checker).await?;
```

**After:**
```rust
// Metrics are now created internally by AppState
let state = AppState::new(config, health_checker).await?;
```

**D6: Fix Test Helpers**

**File:** `crates/riptide-api/src/state.rs` (new_test_minimal)

**Before:**
```rust
pub async fn new_test_minimal() -> Self {
    let metrics = Arc::new(RipTideMetrics::new().expect("Failed to create metrics"));
    // ...
    Self {
        #[allow(deprecated)]
        metrics,
        business_metrics,
        // ...
    }
}
```

**After:**
```rust
pub async fn new_test_minimal() -> Self {
    let business_metrics = Arc::new(
        BusinessMetrics::new().expect("Failed to create business metrics")
    );
    let transport_metrics = Arc::new(
        TransportMetrics::new().expect("Failed to create transport metrics")
    );
    let combined_metrics = Arc::new(
        CombinedMetrics::new(business_metrics.clone(), transport_metrics.clone())
            .expect("Failed to create combined metrics")
    );

    // ...
    Self {
        // REMOVED: metrics field
        business_metrics,
        transport_metrics,
        combined_metrics,
        // ...
    }
}
```

**Testing Strategy:**
```bash
# Full workspace build (clean first for deterministic results)
cargo clean
cargo check --workspace
cargo clippy --workspace -- -D warnings

# All tests
cargo test --workspace

# Verify zero deprecation warnings
cargo build --workspace 2>&1 | grep -i "warning.*deprecat" | wc -l
# Expected output: 0
```

**Breaking Changes:**
- `AppState::new()` signature changed (removed `metrics` parameter)
- `AppState::new_with_facades()` signature changed
- `AppState::new_base()` signature changed
- `AppState::metrics` field removed

**Migration Guide for Downstream Users:**
```rust
// OLD:
let metrics = Arc::new(RipTideMetrics::new()?);
let state = AppState::new(config, metrics, health_checker).await?;
state.metrics.record_http_request(...);

// NEW:
let state = AppState::new(config, health_checker).await?;
state.record_http_request(...);  // or state.combined_metrics.record_http_request(...)
```

**Commit:** `feat(metrics)!: remove deprecated RipTideMetrics (Phase D - BREAKING)`

---

## File-by-File Changes Summary

### state.rs (crates/riptide-api/src/state.rs)

**Lines to Modify:**
- **Line 87-93:** Remove deprecated `metrics: Arc<RipTideMetrics>` field
- **Line 624-658:** Update `new()`, `new_with_facades()`, `new_base()` signatures
- **Line 690-718:** Create business/transport/combined metrics internally
- **Line 1332-1382:** Remove `#[allow(deprecated)] metrics` from struct initialization
- **Line 1654-1912:** Update `new_test_minimal()` to use split metrics
- **Add after line 1470:** New helper methods (Phase A)

**Estimated Changes:** ~150 lines modified, ~50 lines added

### pipeline.rs (crates/riptide-api/src/pipeline.rs)

**Pattern Search & Replace:**
```bash
# Find all usages:
grep -n "self\.state\.metrics\." pipeline.rs

# Expected ~15 occurrences
```

**Replace with:**
- `state.metrics.record_gate_decision(...)` → `state.record_gate_decision(...)`
- `state.metrics.record_pdf_processing_success(...)` → `state.record_pdf_processing_success(...)`
- `state.metrics.record_phase_timing(...)` → `state.business_metrics.<phase>_duration.observe(...)`

**Estimated Changes:** ~15 lines modified

### pipeline_enhanced.rs (crates/riptide-api/src/pipeline_enhanced.rs)

**Similar to pipeline.rs:**
- ~12 occurrences of `state.metrics.*`
- Update to use AppState helpers or direct business_metrics access

**Estimated Changes:** ~12 lines modified

### handlers/*.rs (Multiple files)

**tables.rs:** 1 HTTP request recording → `state.record_http_request(...)`
**llm.rs:** 5 HTTP request recordings → `state.record_http_request(...)`
**utils.rs:** 2 streaming/registry usages → `state.update_streaming_metrics(...)` + `state.combined_metrics.registry`
**workers.rs:** 3 worker metric updates → `state.update_worker_stats(...)` etc.
**resources.rs:** 2 jemalloc updates → `state.update_jemalloc_stats()`
**shared/mod.rs:** 3 spider metrics → `state.record_spider_crawl_completion(...)` etc.

**Estimated Changes:** ~24 lines modified across 6 files

### metrics.rs (DEPRECATED MODULE)

**Action:** Keep file, update deprecation warning
**No code changes required** (already deprecated)

---

## Testing Strategy

### Phase A Testing
```bash
# Smoke test: helpers compile
cargo check -p riptide-api

# Unit tests for AppState
cargo test -p riptide-api --lib state::tests

# Integration tests (if available)
cargo test -p riptide-api --test state_integration
```

### Phase B Testing
```bash
# Pipeline unit tests
cargo test -p riptide-api test_pipeline
cargo test -p riptide-api test_enhanced_pipeline

# Clippy validation
cargo clippy -p riptide-api -- -D warnings
```

### Phase C Testing
```bash
# Per-handler tests
cargo test -p riptide-api handlers::tables
cargo test -p riptide-api handlers::llm
cargo test -p riptide-api handlers::workers
cargo test -p riptide-api handlers::resources
cargo test -p riptide-api handlers::shared

# Full handler suite
cargo test -p riptide-api --lib handlers
```

### Phase D Testing (CRITICAL)
```bash
# Clean build for deterministic results
cargo clean

# Full workspace checks
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace

# Verify zero deprecation warnings
cargo build --workspace 2>&1 | grep -i "warning.*deprecat" | wc -l
# Expected: 0

# Verify metrics endpoint works
cargo run -p riptide-api &
sleep 5
curl http://localhost:8080/metrics | grep "riptide_business"
curl http://localhost:8080/metrics | grep "riptide_transport"
```

### Regression Testing Checklist
- [ ] All Phase A tests pass
- [ ] All Phase B tests pass
- [ ] All Phase C tests pass
- [ ] Zero deprecation warnings after Phase D
- [ ] `/metrics` endpoint returns both business and transport metrics
- [ ] Prometheus scraping works (if configured)
- [ ] Integration tests pass (if available)
- [ ] Performance benchmarks unchanged (if available)

---

## Risk Assessment & Mitigation

### HIGH RISK: Phase D (Breaking Changes)
**Risk:** Removing `AppState::metrics` field breaks downstream code
**Mitigation:**
1. Complete Phases A-C first (all code migrated)
2. Run full test suite before merge
3. Document breaking changes in CHANGELOG.md
4. Provide migration guide in commit message
5. Consider feature flag for gradual rollout

### MEDIUM RISK: Phases B-C (Core Business Logic)
**Risk:** Incorrect metric recording affects observability
**Mitigation:**
1. Verify each metric call still records to same Prometheus metric name
2. Test `/metrics` endpoint output before/after
3. Use git diff to review each change
4. Run Prometheus query validation (if available)

### LOW RISK: Phase A (Additive Only)
**Risk:** Minimal (adding new methods doesn't break existing code)
**Mitigation:**
1. Use clear method names to avoid confusion
2. Document each helper method
3. Run basic smoke tests

---

## Rollout Plan

### Stage 1: Development & Testing (THIS PR)
- [ ] Implement Phase A (helpers)
- [ ] Implement Phase B (pipeline)
- [ ] Implement Phase C (handlers)
- [ ] Implement Phase D (cleanup)
- [ ] All tests pass
- [ ] Zero deprecation warnings
- [ ] PR review

### Stage 2: Staging Deployment
- [ ] Deploy to staging environment
- [ ] Verify `/metrics` endpoint
- [ ] Monitor for errors/anomalies
- [ ] Run load tests (if available)
- [ ] Compare metrics output with production

### Stage 3: Production Rollout
- [ ] Gradual rollout (canary deployment if possible)
- [ ] Monitor Prometheus dashboards
- [ ] Verify alerts still trigger correctly
- [ ] Document any new metric names in runbooks

---

## Success Criteria

✅ **Zero deprecation warnings** (`cargo build --workspace 2>&1 | grep deprecat` returns nothing)
✅ **All tests pass** (`cargo test --workspace`)
✅ **Clippy clean** (`cargo clippy --workspace -- -D warnings`)
✅ **Metrics endpoint works** (`curl /metrics` returns both business and transport metrics)
✅ **No performance regression** (same throughput, latency as before)
✅ **Breaking changes documented** (CHANGELOG.md + migration guide)

---

## Appendix A: Metric Name Mapping

### Business Metrics (riptide-facade)
| Old Name (RipTideMetrics) | New Name (BusinessMetrics) | Category |
|---------------------------|----------------------------|----------|
| `riptide_gate_decisions_raw_total` | `riptide_business_gate_decisions_raw_total` | Gate |
| `riptide_gate_score` | `riptide_business_gate_score` | Gate |
| `riptide_extraction_quality_score` | `riptide_business_extraction_quality_score` | Extraction |
| `riptide_pdf_total_processed` | `riptide_business_pdf_total_processed` | PDF |
| `riptide_spider_crawls_total` | `riptide_business_spider_crawls_total` | Spider |
| `riptide_wasm_memory_pages` | `riptide_business_wasm_memory_pages` | WASM |
| `riptide_worker_pool_size` | `riptide_business_worker_pool_size` | Workers |
| `riptide_cache_hit_rate` | `riptide_business_cache_hit_rate` | Cache |

### Transport Metrics (riptide-api)
| Old Name (RipTideMetrics) | New Name (TransportMetrics) | Category |
|---------------------------|------------------------------|----------|
| `riptide_http_requests_total` | `riptide_transport_http_requests_total` | HTTP |
| `riptide_http_request_duration_seconds` | `riptide_transport_http_request_duration_seconds` | HTTP |
| `riptide_active_connections` | `riptide_transport_active_connections` | Connections |
| `riptide_streaming_active_connections` | `riptide_transport_streaming_active_connections` | Streaming |
| `riptide_streaming_messages_sent_total` | `riptide_transport_streaming_messages_sent_total` | Streaming |
| `riptide_jemalloc_allocated_bytes` | `riptide_transport_jemalloc_allocated_bytes` | Memory |

---

## Appendix B: Code Search Commands

```bash
# Find all deprecated metric usages
grep -r "state\.metrics\." crates/riptide-api/src --include="*.rs"

# Count deprecation warnings
cargo build --workspace 2>&1 | grep -i "warning.*deprecat" | wc -l

# Find RipTideMetrics instantiation
grep -r "RipTideMetrics::new" crates/riptide-api --include="*.rs"

# Find AppState::new calls (need signature update)
grep -r "AppState::new\|AppState::new_with_facades\|AppState::new_base" crates/riptide-api --include="*.rs"

# Verify combined_metrics usage
grep -r "combined_metrics\." crates/riptide-api/src --include="*.rs"
```

---

## References

- **RipTideMetrics (deprecated):** `crates/riptide-api/src/metrics.rs`
- **BusinessMetrics:** `crates/riptide-facade/src/metrics/business.rs`
- **TransportMetrics:** `crates/riptide-api/src/metrics_transport.rs`
- **CombinedMetrics:** `crates/riptide-api/src/metrics_integration.rs`
- **AppState:** `crates/riptide-api/src/state.rs`
- **Sprint 4.5 Status:** `/docs/completion/CURRENT_STATUS_AND_NEXT_STEPS.md`

---

**Document Version:** 1.0
**Created:** 2025-11-09
**Author:** System Architecture Designer
**Status:** ✅ READY FOR IMPLEMENTATION
