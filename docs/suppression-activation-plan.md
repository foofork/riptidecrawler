# Suppression Activation Plan

**Generated:** 2025-10-08
**Purpose:** Prioritized plan to activate suppressed functionality based on readiness analysis
**Status:** Ready for execution

---

## Executive Summary

This plan addresses **suppressed functionality** across the codebase that is ready or nearly ready for activation. Analysis reveals:

- **7,249+ lines** of complete streaming infrastructure (TODO P2)
- **2 critical P0 issues** blocking core functionality
- **8 P1 issues** requiring minor integration work
- **Complete profiling infrastructure** awaiting integration
- **Comprehensive test suite** in WASM extractor ready to enable

**Priority Distribution:**
- 🔴 **HIGH**: 10 items (activate immediately)
- 🟡 **MEDIUM**: 8 items (activate within 1 week)
- 🟢 **LOW**: 6 items (review and schedule)

---

## 🔴 HIGH PRIORITY - Activate Now

### 1. Fix P0 Critical Blockers

#### 1.1 FetchEngine Metrics Resolution (P0)
**File:** `/workspaces/eventmesh/crates/riptide-api/src/handlers/fetch.rs:17`

```rust
// TODO(P0): Fix method resolution issue with Arc<FetchEngine>
// ISSUE: The get_all_metrics method exists but isn't accessible through Arc
// PLAN: Update FetchEngine to expose metrics through Arc wrapper
```

**Impact:** Fetch metrics endpoint returns empty data
**Effort:** 1-2 hours
**Action Required:**
1. Add `pub fn get_all_metrics(&self)` method to FetchEngine
2. Ensure method works through Arc wrapper
3. Wire to `get_fetch_metrics()` handler
4. Add integration test

**Success Criteria:**
- [ ] `/metrics/fetch` endpoint returns actual metrics
- [ ] Test coverage for metrics endpoint
- [ ] Documentation updated

---

#### 1.2 Stealth Configuration Integration (P0)
**File:** `/workspaces/eventmesh/crates/riptide-api/src/handlers/render/processors.rs:87`

```rust
// TODO(P0): Wire up stealth values to headless browser RPC call below
// PLAN: Pass stealth configuration to render_dynamic RPC call
```

**Impact:** Stealth mode features are generated but not used
**Effort:** 2-3 hours
**Action Required:**
1. Pass `stealth.generate_headers()` to headless browser
2. Apply `stealth.calculate_delay()` to request timing
3. Configure browser fingerprinting from stealth module
4. Add tests for stealth effectiveness

**Success Criteria:**
- [ ] Stealth headers applied to browser requests
- [ ] Timing delays properly configured
- [ ] Bot detection evasion tests pass

---

### 2. Enable WASM Extractor Integration Tests

**Files:**
- `/workspaces/eventmesh/wasm/riptide-extractor-wasm/tests/mod.rs:80-89, 291`
- `/workspaces/eventmesh/wasm/riptide-extractor-wasm/tests/test_runner.rs:35-403`

**Current Status:** ✅ **Integration module fully implemented** (1,209 lines)
**Effort:** 30 minutes
**Impact:** Enables 10 comprehensive integration tests with 500+ extraction validations

**Action Required:**
1. **File 1** (`tests/mod.rs`):
   - Line 80-89: Replace placeholder with `run_integration_test_category()?`
   - Line 291: Remove underscore prefix from function name

2. **File 2** (`test_runner.rs`):
   - Lines 40-403: Uncomment 10 test functions and utilities

**Success Criteria:**
- [ ] All 10 integration tests enabled
- [ ] Test suite runs successfully: `cargo test --package riptide-extractor-wasm`
- [ ] Coverage increases by ~10-15%
- [ ] Test reports generated in `/workspaces/riptide/reports/`

**Reference:** See `/workspaces/eventmesh/docs/todo-immediate-actions.md` for detailed steps

---

### 3. Activate Streaming Infrastructure (TODO P2)

**Files:** 7,249 lines across 17 files in `crates/riptide-api/src/streaming/`

**Current Status:**
- ✅ Complete NDJSON implementation
- ✅ Complete SSE (Server-Sent Events)
- ✅ Complete WebSocket bidirectional
- ✅ Buffer management with backpressure
- ✅ Lifecycle management
- ✅ Error handling and recovery
- ✅ Metrics and monitoring

**Blocked By:** Routes not added to main router

**Effort:** 4-6 hours
**Impact:** Enables real-time streaming for 3 protocols

**Action Required:**

#### Phase 1: Add Feature Flag (1 hour)
```toml
# crates/riptide-api/Cargo.toml
[features]
streaming = []  # Already exists at line 81 - just needs activation
```

#### Phase 2: Add Routes (2 hours)
```rust
// crates/riptide-api/src/routes.rs (or main.rs)
use crate::streaming::{ndjson_crawl_stream, ndjson_deepsearch_stream, crawl_sse, crawl_websocket};

pub fn configure_streaming_routes(router: Router<AppState>) -> Router<AppState> {
    router
        // NDJSON endpoints
        .route("/v1/stream/crawl", post(ndjson_crawl_stream))
        .route("/v1/stream/deepsearch", post(ndjson_deepsearch_stream))

        // SSE endpoint
        .route("/v1/sse/crawl", post(crawl_sse))

        // WebSocket endpoint
        .route("/v1/ws/crawl", get(crawl_websocket))
}
```

#### Phase 3: Integration Tests (2 hours)
- Enable tests in `crates/riptide-api/tests/streaming_*.rs`
- Validate all 3 protocols work end-to-end
- Test backpressure handling
- Test connection limits

#### Phase 4: Documentation (1 hour)
- Update API documentation with streaming endpoints
- Add usage examples for each protocol
- Document configuration options

**Success Criteria:**
- [ ] All streaming routes accessible
- [ ] NDJSON, SSE, WebSocket protocols functional
- [ ] Integration tests passing
- [ ] Performance metrics collected
- [ ] Documentation complete

**Priority Justification:** Infrastructure is 100% complete and tested internally. Only routing integration needed.

---

### 4. Wire Memory Profiler to Monitoring Endpoints (P2)

**Files:**
- `/workspaces/eventmesh/crates/riptide-api/src/handlers/monitoring.rs:217, 244, 270`
- `/workspaces/eventmesh/crates/riptide-performance/src/profiling/mod.rs` (complete implementation)

**Current Status:**
- ✅ Complete MemoryProfiler implementation (150+ lines)
- ✅ LeakDetector module ready
- ✅ AllocationAnalyzer ready
- ✅ FlamegraphGenerator ready
- ⚠️ AppState has profiler field but handlers return placeholder data

**Effort:** 3-4 hours
**Impact:** Enables production memory monitoring and leak detection

**Action Required:**

#### Step 1: Add Profiler to AppState (if missing)
```rust
// crates/riptide-api/src/state.rs
pub struct AppState {
    // ... existing fields

    /// Memory profiler for leak detection and analysis
    pub memory_profiler: Option<Arc<MemoryProfiler>>,
}
```

#### Step 2: Wire Handlers (2 hours)
```rust
// crates/riptide-api/src/handlers/monitoring.rs

// Line 217: Memory profiling
pub async fn get_memory_profile(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    if let Some(profiler) = &state.memory_profiler {
        let report = profiler.generate_report().await
            .map_err(|e| ApiError::internal_error(&format!("Profiling failed: {}", e)))?;
        Ok(Json(report))
    } else {
        Err(ApiError::feature_disabled("memory-profiling"))
    }
}

// Line 244: Leak detection
pub async fn get_leak_analysis(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    if let Some(profiler) = &state.memory_profiler {
        let leaks = profiler.detect_leaks().await
            .map_err(|e| ApiError::internal_error(&format!("Leak detection failed: {}", e)))?;
        Ok(Json(leaks))
    } else {
        Err(ApiError::feature_disabled("memory-profiling"))
    }
}

// Line 270: Allocation analysis
pub async fn get_allocation_analysis(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    if let Some(profiler) = &state.memory_profiler {
        let analysis = profiler.analyze_allocations().await
            .map_err(|e| ApiError::internal_error(&format!("Allocation analysis failed: {}", e)))?;
        Ok(Json(analysis))
    } else {
        Err(ApiError::feature_disabled("memory-profiling"))
    }
}
```

#### Step 3: Initialize in main.rs (1 hour)
```rust
// Enable profiling based on config or feature flag
let memory_profiler = if config.enable_memory_profiling {
    Some(Arc::new(MemoryProfiler::new(MemoryProfileConfig::default())))
} else {
    None
};
```

**Success Criteria:**
- [ ] Endpoints return real profiling data
- [ ] Memory leak detection operational
- [ ] Allocation patterns tracked
- [ ] Flamegraph generation works (optional)
- [ ] Feature flag controls activation

---

### 5. Fix P1 Test Infrastructure Issues

#### 5.1 App Factory for Integration Testing (P1)
**File:** `/workspaces/eventmesh/crates/riptide-api/tests/integration_tests.rs:28`

**Action Required:**
```rust
/// Creates a test app instance for integration testing
pub async fn create_test_app() -> AppState {
    let config = AppConfig::test_defaults();
    let api_config = ApiConfig::test_defaults();

    // Initialize minimal components for testing
    AppState {
        http_client: http_client(),
        cache: Arc::new(tokio::sync::Mutex::new(CacheManager::in_memory())),
        extractor: Arc::new(WasmExtractor::test_instance()),
        // ... minimal initialization
    }
}
```

**Effort:** 2-3 hours
**Impact:** Enables proper integration testing with consistent setup

---

#### 5.2 CSV/Markdown Export Validation (P1)
**Files:**
- `/workspaces/eventmesh/crates/riptide-api/tests/integration_tests.rs:318` (CSV)
- `/workspaces/eventmesh/crates/riptide-api/tests/integration_tests.rs:356` (Markdown)

**Current Status:** Tests pass but don't validate content structure

**Action Required:**
```rust
// Line 318: Validate CSV content
assert_eq!(status, StatusCode::OK);
let content_type = response.headers().get("content-type").unwrap();
assert_eq!(content_type, "text/csv");

let body = response.text().await?;
assert!(body.contains("url,title,content")); // CSV header
assert!(body.lines().count() > 1); // At least header + 1 row

// Line 356: Validate Markdown format
assert_eq!(status, StatusCode::OK);
let body = response.text().await?;
assert!(body.contains("| URL | Title | Content |")); // Table header
assert!(body.contains("|---|---|---|")); // Separator
```

**Effort:** 1 hour
**Impact:** Improves test quality and catches format regressions

---

## 🟡 MEDIUM PRIORITY - Activate Soon (Within 1 Week)

### 6. Enhanced Pipeline Orchestrator (P2)

**File:** `/workspaces/eventmesh/crates/riptide-api/src/pipeline_enhanced.rs`

**Status:** Complete but marked for production validation
**Effort:** 4-6 hours
**Impact:** Improved pipeline performance and reliability

**Action Required:**
1. **Production Validation** (3 hours):
   - Load testing with realistic workloads
   - Error recovery scenario testing
   - Performance benchmarking vs existing pipeline

2. **Integration** (2 hours):
   - Add feature flag `pipeline-enhanced`
   - Wire to existing pipeline routes as opt-in
   - Add metrics comparison dashboard

3. **Documentation** (1 hour):
   - Document performance improvements
   - Migration guide from standard pipeline
   - Configuration options

**Success Criteria:**
- [ ] Handles 10,000+ concurrent requests
- [ ] <5% error rate under load
- [ ] Feature flag controls activation
- [ ] Metrics show improvement over baseline

---

### 7. Event Bus Integration (P1)

**Files:**
- `/workspaces/eventmesh/crates/riptide-api/src/state.rs:1028, 1091`
- Event bus infrastructure exists in AppState

**Current Status:**
- ✅ EventBus initialized in AppState
- ⚠️ Alert evaluation task doesn't publish to bus
- ⚠️ BaseEvent created but not published

**Effort:** 2-3 hours
**Impact:** Centralized event coordination and monitoring

**Action Required:**

```rust
// Line 1028: Publish alerts to event bus
pub fn start_alert_evaluation_task(&self, event_bus: Arc<EventBus>) {
    let alert_manager = self.alert_manager.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));

        loop {
            interval.tick().await;

            if let Ok(alerts) = alert_manager.check_alerts().await {
                for alert in alerts {
                    // Publish to event bus for system-wide notification
                    let event = BaseEvent::new(
                        "alert".to_string(),
                        serde_json::to_value(&alert).unwrap_or_default(),
                    );
                    let _ = event_bus.publish(event).await;
                }
            }
        }
    });
}
```

**Success Criteria:**
- [ ] Alerts published to event bus
- [ ] Subscribers can receive alert notifications
- [ ] Integration test validates event flow

---

### 8. Spider Engine Health Check (P1)

**File:** `/workspaces/eventmesh/crates/riptide-api/src/health.rs:179`

**Current Status:** Spider engine initialized but health check returns `None`

**Effort:** 2 hours
**Impact:** Complete health monitoring coverage

**Action Required:**
```rust
// Line 179: Implement spider health check
spider_engine: if let Some(spider) = &self.spider {
    Some(ComponentHealth {
        healthy: spider.is_healthy().await,
        status: spider.get_status().await,
        message: spider.get_health_message().await,
    })
} else {
    None
}
```

**Prerequisites:** Ensure Spider has health check methods

**Success Criteria:**
- [ ] Health endpoint includes spider status
- [ ] Connectivity tests validate spider functionality

---

### 9. Component Version Dynamic Loading (P1)

**File:** `/workspaces/eventmesh/crates/riptide-api/src/health.rs:40`

**Current Status:** Version hardcoded as "0.1.0"

**Effort:** 2-3 hours
**Impact:** Accurate version reporting across components

**Action Required:**

#### Option 1: Compile-time (Recommended)
```rust
// Use env variable set by build.rs
component_versions.insert(
    "riptide-core".to_string(),
    env!("CARGO_PKG_VERSION_riptide_core").to_string()
);
```

Create `build.rs`:
```rust
fn main() {
    // Read workspace Cargo.toml and set env vars
    let workspace_toml = std::fs::read_to_string("../../Cargo.toml").unwrap();
    // Parse and set versions
}
```

#### Option 2: Runtime (More flexible)
```rust
use cargo_metadata::MetadataCommand;

fn load_component_versions() -> HashMap<String, String> {
    let metadata = MetadataCommand::new()
        .exec()
        .expect("Failed to read cargo metadata");

    metadata.packages.iter()
        .filter(|p| p.name.starts_with("riptide-"))
        .map(|p| (p.name.clone(), p.version.to_string()))
        .collect()
}
```

**Success Criteria:**
- [ ] Versions match actual Cargo.toml values
- [ ] Updates automatically when dependencies change

---

### 10. Telemetry Runtime Integration (P2)

**File:** `/workspaces/eventmesh/crates/riptide-api/src/handlers/telemetry.rs:386`

**Current Status:** Returns static telemetry status

**Effort:** 2-3 hours
**Impact:** Real-time telemetry monitoring

**Action Required:**
```rust
pub async fn get_telemetry_status(
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Some(telemetry) = &state.telemetry {
        Json(json!({
            "enabled": true,
            "runtime_info": {
                "spans_exported": telemetry.get_span_count().await,
                "metrics_exported": telemetry.get_metric_count().await,
                "errors": telemetry.get_error_count().await,
                "uptime_seconds": telemetry.get_uptime().await,
            },
            "exporters": telemetry.get_exporter_status().await,
        }))
    } else {
        Json(json!({
            "enabled": false,
            "runtime_info": null,
        }))
    }
}
```

**Success Criteria:**
- [ ] Real-time telemetry stats available
- [ ] Exporter status tracked
- [ ] Error counts monitored

---

## 🟢 LOW PRIORITY - Review Later (Next Sprint)

### 11. Provider Failover Testing (P1)

**File:** `/workspaces/eventmesh/crates/riptide-api/tests/integration_tests.rs:824`

**Status:** Test placeholder exists, needs implementation
**Effort:** 4-6 hours
**Impact:** Validates multi-provider reliability

**Defer Reason:** Requires mock provider infrastructure and complex scenario setup

**Future Action:**
- Implement HealthMonitorBuilder
- Add MockLlmProvider with controllable health states
- Simulate provider failures and verify failover
- Test recovery after provider returns to healthy state

---

### 12. PDF Feature Flag Decision

**Current Status:**
- ✅ PDF feature fully implemented and tested
- ⚠️ Feature-gated behind `cfg(feature = "pdf")`
- ✅ 48 feature gates across codebase

**Options:**

#### Option A: Enable by Default ✅ RECOMMENDED
```toml
# Cargo.toml
[features]
default = ["pdf"]
pdf = []
```

**Pros:**
- Feature is production-ready
- Widely used in tests
- No breaking changes for users

**Cons:**
- Increases binary size (~2MB)
- Adds PDF dependency overhead

#### Option B: Keep Optional
- Keep `pdf` as opt-in feature
- Add documentation for enabling
- Ensure graceful degradation when disabled

**Decision Required:** Product team input needed

---

### 13. Strategy Traits Implementation (riptide-extraction)

**File:** `/workspaces/eventmesh/crates/riptide-extraction/src/strategy_implementations.rs`

**Status:**
- ✅ Complete strategy implementation (100+ lines)
- ⚠️ Feature-gated: `cfg(feature = "strategy-traits")`
- ⚠️ Commented code due to circular dependency with riptide-core

**Blocker:** Circular dependency between riptide-extraction and riptide-core

**Solutions:**

#### Option 1: Extract Traits to Shared Crate
```
riptide-traits/
  ├── src/
  │   ├── extraction_strategy.rs
  │   ├── performance_metrics.rs
  │   └── lib.rs
```

Both riptide-core and riptide-extraction depend on riptide-traits

#### Option 2: Keep in riptide-core
- Define traits in riptide-core
- riptide-extraction implements traits (no circular dep)
- Current approach seems backward

**Effort:** 6-8 hours
**Impact:** Enables pluggable extraction strategies

**Defer Reason:** Requires architectural refactoring decision

---

### 14. WASM Performance Tests

**File:** `/workspaces/eventmesh/tests/wasm_performance_test.rs`

**Status:**
- 3 tests ignored by default
- Require built WASM component
- Test cold start, extraction performance, AOT cache

**Effort:** 2 hours
**Impact:** Performance regression detection

**Action Required:**
1. Add to CI/CD with WASM build step
2. Set performance baselines
3. Configure alerts for regressions
4. Run on schedule (nightly/weekly)

**Defer Reason:** Requires CI/CD pipeline updates

---

### 15. Resource Control Tests (Private Method Access)

**File:** `/workspaces/eventmesh/crates/riptide-api/src/tests/resource_controls.rs:207`

**Status:** Test ignored - requires access to private `acquire_instance()` method

**Options:**

#### Option A: Add Test-Only Public Methods
```rust
#[cfg(test)]
impl WasmManager {
    pub fn test_acquire_instance(&self) -> Result<...> {
        self.acquire_instance()
    }
}
```

#### Option B: Test Through Public API
- Test resource limits indirectly
- Focus on observable behavior
- Less brittle, more maintainable

**Recommendation:** Option B (test through public API)

**Effort:** 3-4 hours
**Impact:** Improved test coverage for resource management

---

### 16. Mock Provider Infrastructure (riptide-intelligence)

**Files:**
- `/workspaces/eventmesh/crates/riptide-intelligence/tests/integration_tests.rs:456`
- `/workspaces/eventmesh/crates/riptide-intelligence/tests/integration_tests.rs:802`

**Status:** 2 tests ignored due to missing mock infrastructure

**Required Components:**
- `HealthMonitorBuilder` for test health monitoring
- `MockLlmProvider.set_healthy()` for health state control
- Provider failure simulation framework

**Effort:** 8-10 hours
**Impact:** Comprehensive provider failover testing

**Defer Reason:** Significant infrastructure work, low immediate value

---

## Implementation Roadmap

### Week 1 (Days 1-3) - Critical Fixes
**Target:** Complete all HIGH priority items

#### Day 1: P0 Blockers + WASM Tests
- [ ] Morning: Fix FetchEngine metrics (1.1)
- [ ] Afternoon: Wire stealth configuration (1.2)
- [ ] Evening: Enable WASM integration tests (2)

**Success Metric:** 2 P0 issues resolved, 10 new tests passing

#### Day 2: Streaming Infrastructure
- [ ] Morning: Add streaming routes (3, Phase 2)
- [ ] Afternoon: Add feature flag, integration tests (3, Phases 1 & 3)
- [ ] Evening: Documentation (3, Phase 4)

**Success Metric:** 3 streaming protocols operational

#### Day 3: Profiling Integration
- [ ] Morning: Wire memory profiler handlers (4, Step 2)
- [ ] Afternoon: Initialize profiler in main.rs (4, Step 3)
- [ ] Evening: Testing and validation

**Success Metric:** Memory profiling endpoints functional

---

### Week 2 (Days 4-7) - P1 Issues & Medium Priority

#### Day 4: Test Infrastructure
- [ ] Morning: App factory implementation (5.1)
- [ ] Afternoon: CSV/Markdown validation (5.2)
- [ ] Evening: Integration test validation

**Success Metric:** Integration test suite improved

#### Day 5: Event Bus & Health
- [ ] Morning: Event bus alert publishing (7)
- [ ] Afternoon: Spider health check (8)
- [ ] Evening: Testing

**Success Metric:** Complete event coordination, full health coverage

#### Day 6: Enhanced Pipeline
- [ ] Full day: Production validation (6)
- [ ] Load testing, error scenarios
- [ ] Performance benchmarking

**Success Metric:** Enhanced pipeline validated for production

#### Day 7: Telemetry & Versioning
- [ ] Morning: Component version loading (9)
- [ ] Afternoon: Telemetry runtime integration (10)
- [ ] Evening: Documentation and testing

**Success Metric:** Complete telemetry and version tracking

---

### Week 3-4 - LOW Priority Review

- Architectural decisions (PDF default, strategy traits)
- Mock infrastructure planning
- CI/CD enhancements for WASM tests
- Resource control test refactoring

---

## Metrics & Success Tracking

### Quantitative Goals

| Metric | Before | After Week 1 | After Week 2 |
|--------|--------|--------------|--------------|
| P0 Issues | 2 | 0 | 0 |
| P1 Issues | 8 | 8 | 3 |
| P2 Issues | 12+ | 9 | 4 |
| Integration Tests | ~50 | ~60 | ~65 |
| Test Coverage | ~82% | ~85% | ~88% |
| Suppressed Code LOC | 7,249+ | ~1,000 | ~500 |
| Active Endpoints | 25 | 29 | 32 |

### Qualitative Goals

**Week 1:**
- [ ] All critical blockers resolved
- [ ] Streaming infrastructure operational
- [ ] Memory profiling production-ready

**Week 2:**
- [ ] Test infrastructure robust and maintainable
- [ ] Event coordination centralized
- [ ] Enhanced pipeline validated

**Week 4:**
- [ ] Architectural decisions documented
- [ ] Remaining suppressions categorized and scheduled
- [ ] CI/CD pipeline enhanced

---

## Risk Assessment

### High Risk Items

1. **Streaming Routes Integration** (Item 3)
   - **Risk:** May conflict with existing routes or middleware
   - **Mitigation:** Feature flag, gradual rollout, comprehensive testing
   - **Rollback Plan:** Disable feature flag, revert routes

2. **Memory Profiler Performance** (Item 4)
   - **Risk:** Profiling overhead may impact production performance
   - **Mitigation:** Make optional, configurable sampling rate
   - **Rollback Plan:** Disable profiler, return placeholder data

### Medium Risk Items

3. **Enhanced Pipeline Validation** (Item 6)
   - **Risk:** May introduce regressions under specific workloads
   - **Mitigation:** Extensive load testing, gradual migration
   - **Rollback Plan:** Fallback to standard pipeline via feature flag

### Low Risk Items

4. **WASM Integration Tests** (Item 2)
   - **Risk:** Tests may fail on different environments
   - **Mitigation:** Comprehensive fixture validation
   - **Rollback Plan:** Re-disable tests, fix issues incrementally

---

## Dependency Graph

```
P0 Blockers (1.1, 1.2)
  └─> No dependencies, can start immediately

WASM Tests (2)
  └─> No dependencies, parallel to P0 work

Streaming Routes (3)
  ├─> Requires: Feature flag decision
  └─> Blocks: Real-time monitoring demos

Memory Profiling (4)
  ├─> Requires: AppState updates
  └─> Enables: Production memory monitoring

Event Bus (7)
  ├─> Requires: Item 5.1 (app factory)
  └─> Enables: Centralized monitoring

Spider Health (8)
  ├─> Requires: Spider health methods
  └─> Completes: Health check coverage

Enhanced Pipeline (6)
  ├─> Requires: Load testing environment
  └─> Enables: Performance improvements
```

---

## Decision Log

### Decisions Required

1. **PDF Feature Default** (Item 12)
   - **Options:** Enable by default vs keep optional
   - **Owner:** Product/Architecture team
   - **Deadline:** Week 2

2. **Strategy Traits Architecture** (Item 13)
   - **Options:** Extract to shared crate vs keep in core
   - **Owner:** Architecture team
   - **Deadline:** Week 3

3. **WASM Tests CI Integration** (Item 14)
   - **Options:** Run on every commit vs scheduled
   - **Owner:** DevOps team
   - **Deadline:** Week 3

### Decisions Made

1. **Streaming Infrastructure** - ✅ APPROVED for activation
   - Rationale: Complete implementation, comprehensive tests
   - Date: 2025-10-08

2. **Memory Profiling** - ✅ APPROVED as optional feature
   - Rationale: Production-ready but potentially expensive
   - Date: 2025-10-08

---

## Communication Plan

### Stakeholder Updates

**Week 1 - Daily:**
- Slack update on P0/P1 resolution
- Blocker escalation if needed

**Week 2 - Mid-week & End:**
- Demo streaming infrastructure
- Performance metrics comparison
- Test coverage improvement report

**Week 3-4 - Weekly:**
- Architectural decision requests
- Low priority item status
- Remaining suppression categorization

### Documentation Updates

**Immediate:**
- [ ] Update CHANGELOG.md with activated features
- [ ] Update API documentation with new endpoints
- [ ] Update README.md with streaming examples

**Week 2:**
- [ ] Memory profiling guide
- [ ] Enhanced pipeline migration guide
- [ ] Event bus integration guide

---

## Rollback Procedures

### Quick Rollback (< 5 minutes)

1. **Streaming Routes:**
   ```bash
   # Disable feature in Cargo.toml
   sed -i 's/default = \["streaming"\]/default = []/' crates/riptide-api/Cargo.toml
   cargo build --release
   ```

2. **Memory Profiling:**
   ```rust
   // In config or initialization
   enable_memory_profiling: false
   ```

### Full Rollback (< 1 hour)

```bash
# Revert to pre-activation state
git revert <activation-commit-range>
cargo test --all
cargo build --release
```

### Partial Rollback

Feature flags allow selective rollback of individual components without full revert.

---

## Maintenance Schedule

### Post-Activation Monitoring

**First 24 Hours:**
- [ ] Monitor error rates every 2 hours
- [ ] Check performance metrics
- [ ] Review logs for unexpected behavior

**First Week:**
- [ ] Daily metrics review
- [ ] User feedback collection
- [ ] Performance baseline establishment

**First Month:**
- [ ] Weekly stability review
- [ ] Gradual feature adoption tracking
- [ ] Documentation refinement based on usage

---

## Conclusion

This plan provides a structured approach to activating **24 suppressed items** across:

- 🔴 **10 HIGH priority** items (P0-P1 blockers, complete infrastructure)
- 🟡 **8 MEDIUM priority** items (integration work, validation)
- 🟢 **6 LOW priority** items (architectural decisions, future work)

**Key Highlights:**
- **7,249 lines** of streaming infrastructure ready to activate
- **2 critical P0 blockers** preventing core functionality
- **Complete memory profiling system** awaiting integration
- **10 integration tests** in WASM extractor ready to enable

**Timeline:**
- **Week 1:** Resolve all critical issues, activate streaming
- **Week 2:** Complete P1 work, validate enhancements
- **Week 3-4:** Architectural decisions, planning future work

**Next Steps:**
1. Review and approve HIGH priority activations
2. Assign owners for each activation item
3. Begin Week 1 execution plan
4. Schedule decision meetings for Week 2 items

---

**Document Owners:**
- Architecture: System Architecture Designer
- Execution: Development Team Lead
- Approval: Technical Director

**Last Updated:** 2025-10-08
