# Phase 4/5 TODO & Deferred Work - Comprehensive Tracking

**Generated:** 2025-11-09
**Status:** Compilation complete (0 errors), Implementation incomplete
**Source:** Extracted from all Phase 4/5 completion documents

---

## 📋 EXECUTIVE SUMMARY

**Total Deferred Items:** ~150+ tasks across 10 categories
**Critical Path:** Handler stub implementation → Metrics migration → Testing
**Estimated Effort:** 40-60 hours (Phase 6 work)

---

## 🎯 CATEGORY 1: HANDLER STUBS (17 items)

**Priority:** CRITICAL
**Blocking:** Full API functionality
**Effort:** 8-12 hours

### Files with todo!() Stubs

#### `/crates/riptide-api/src/handlers/strategies.rs` (2 stubs)
- [ ] `get_engine_priority()` - Implement engine selection priority logic
- [ ] `update_engine_weights()` - Implement dynamic engine weight adjustment

#### `/crates/riptide-api/src/handlers/sessions.rs` (4 stubs)
- [ ] `create_session()` - Implement browser session creation
- [ ] `get_session_status()` - Implement session status retrieval
- [ ] `close_session()` - Implement graceful session closure
- [ ] `list_active_sessions()` - Implement session inventory

#### `/crates/riptide-api/src/handlers/memory.rs` (1 stub)
- [ ] `get_memory_stats()` - Implement jemalloc memory statistics

#### `/crates/riptide-api/src/handlers/monitoring.rs` (10 stubs)
- [ ] `get_health_score()` - Implement health scoring algorithm
- [ ] `get_system_metrics()` - Implement system metrics collection
- [ ] `get_performance_metrics()` - Implement performance metrics
- [ ] `get_resource_utilization()` - Implement resource utilization tracking
- [ ] `get_error_rates()` - Implement error rate calculation
- [ ] `get_throughput_metrics()` - Implement throughput measurement
- [ ] `get_latency_percentiles()` - Implement latency percentile calculation
- [ ] `get_cache_stats()` - Implement cache statistics
- [ ] `get_queue_depths()` - Implement queue depth monitoring
- [ ] `get_connection_pools()` - Implement connection pool status

**Implementation Pattern:**
```rust
// Current stub:
pub async fn get_health_score() -> Json<serde_json::Value> {
    todo!("Implement get_health_score")
}

// Phase 6 implementation:
pub async fn get_health_score(State(state): State<AppState>) -> Result<Json<HealthScore>, ApiError> {
    let facade = &state.monitoring_facade;
    let score = facade.calculate_health_score().await?;
    Ok(Json(score))
}
```

---

## 🎯 CATEGORY 2: FACADE METHODS (11 items)

**Priority:** HIGH
**Blocking:** Handler implementation
**Effort:** 4-6 hours

### ProfileFacade Missing Methods (4 items)
```rust
// File: /crates/riptide-facade/src/facades/profile.rs
```

- [ ] `create_profile(profile: DomainProfile)` - Create new browser profile
- [ ] `batch_create_profiles(profiles: Vec<DomainProfile>)` - Batch profile creation
- [ ] `get_caching_metrics()` - Profile cache performance metrics
- [ ] `clear_all_caches()` - Cache invalidation across all profiles

### TableFacade Missing Methods (1 item)
```rust
// File: /crates/riptide-facade/src/facades/table.rs
```

- [ ] `get_extraction_stats()` - Table extraction performance statistics

### StreamingFacade Missing Methods (3 items)
```rust
// File: /crates/riptide-facade/src/facades/streaming.rs
```

- [ ] `with_lifecycle_manager(manager: LifecycleManager)` - Attach lifecycle management
- [ ] `get_active_streams()` - List all active streaming connections
- [ ] `close_stream(stream_id: String)` - Gracefully close specific stream

### ProfileManager Missing Methods (3 items)
```rust
// File: /crates/riptide-browser/src/profile_manager.rs
```

- [ ] `search(query: ProfileSearchQuery)` - Search profiles by criteria
- [ ] `list_by_tag(tag: String)` - Filter profiles by tag
- [ ] `get_statistics()` - Profile usage statistics

**Referenced In:**
- `PHASE_5_FINAL_SUMMARY.md:145` - Missing ProfileFacade methods blocking handlers
- `SPRINT_4.5_INTEGRATION_GUIDE.md:53` - Missing BusinessMetrics methods
- `PHASE_5_HANDLER_INTEGRATION_COMPLETE.md:29` - Facade integration incomplete

---

## 🎯 CATEGORY 3: MISSING FIELDS (6 items)

**Priority:** HIGH
**Blocking:** Data serialization
**Effort:** 2-3 hours

### DomainProfile Missing Fields (4 items)
```rust
// File: /crates/riptide-types/src/browser/profile.rs
// Current: Fields accessed via profile.field
// Should be: profile.metadata.field
```

- [ ] `avg_response_time_ms: f64` - Average response time metric
- [ ] `last_accessed: DateTime<Utc>` - Last access timestamp
- [ ] `success_rate: f64` - Success rate percentage
- [ ] `total_requests: u64` - Total request count

### ResourceStatus Missing Fields (2 items)
```rust
// File: /crates/riptide-types/src/resource/status.rs
// Current field names incorrect
```

- [ ] Fix `headless_pool_capacity` → `headless_pool_total`
- [ ] Fix `headless_pool_active` → `headless_pool_in_use`

**Referenced In:**
- `PHASE_5_FINAL_SUMMARY.md:165` - Missing fields breaking serialization
- `NEXT_AGENT_INSTRUCTIONS.md:204` - Field migration instructions

---

## 🎯 CATEGORY 4: METRICS MIGRATION (341 warnings)

**Priority:** MEDIUM
**Blocking:** Clean clippy, production readiness
**Effort:** 6-8 hours

### Deprecation Warnings to Fix

```rust
// Current deprecated pattern:
state.metrics.record_error(ErrorType::Http);

// Phase 6 migration to:
state.business_metrics.record_error(BusinessError::RequestFailed);
state.transport_metrics.record_http_error();
```

### Files with Deprecation Warnings
- [ ] `/crates/riptide-api/src/handlers/pdf.rs` (15 warnings)
- [ ] `/crates/riptide-api/src/handlers/spider.rs` (12 warnings)
- [ ] `/crates/riptide-api/src/handlers/tables.rs` (8 warnings)
- [ ] `/crates/riptide-api/src/handlers/profiles.rs` (10 warnings)
- [ ] `/crates/riptide-api/src/state.rs` (5 warnings)
- [ ] ~291 additional warnings across other files

### Migration Strategy

**Step 1: Add New Metrics Infrastructure**
```rust
// state.rs
pub struct AppState {
    pub business_metrics: Arc<BusinessMetrics>,
    pub transport_metrics: Arc<TransportMetrics>,
    #[deprecated] pub metrics: Arc<OldMetrics>,  // Keep for compatibility
}
```

**Step 2: Update Handlers Incrementally**
```bash
# Pattern replacement:
rg "metrics.record_error" --type rust -l | \
  xargs sed -i 's/metrics.record_error(ErrorType::\([^)]*\))/business_metrics.record_error(BusinessError::\1)/'
```

**Step 3: Remove Deprecated After Full Migration**

**Referenced In:**
- `PHASE_4_5_FINAL_COMPLETION_REPORT.md:289` - 341 deprecation warnings documented
- `SPRINT_4.5_METRICS_SPLIT_SUMMARY.md` - Metrics split completion
- `SPRINT_4.5_INTEGRATION_GUIDE.md:220` - Integration pending

---

## 🎯 CATEGORY 5: STREAMING FACADE INITIALIZATION

**Priority:** MEDIUM
**Blocking:** Streaming feature completeness
**Effort:** 3-4 hours

### AppState Integration Deferred

**Current State:**
```rust
// File: /crates/riptide-api/src/state.rs
pub struct AppState {
    // TODO Phase 4.3: Streaming facade initialization is deferred
    // pub streaming_facade: Arc<StreamingFacade>,
}
```

### Tasks
- [ ] Initialize `StreamingFacade` in `AppState::new()`
- [ ] Wire up SSE transport adapter
- [ ] Wire up WebSocket transport adapter
- [ ] Add lifecycle manager integration
- [ ] Update handler dependency injection

**Implementation:**
```rust
impl AppState {
    pub async fn new(config: Config) -> RiptideResult<Self> {
        // ... existing initialization

        let streaming_facade = Arc::new(StreamingFacade::new(
            sse_transport,
            websocket_transport,
            lifecycle_manager,
        ));

        Ok(Self {
            // ... existing fields
            streaming_facade,
        })
    }
}
```

**Referenced In:**
- `PHASE_5_METRICS_WIRING_COMPLETE.md:69` - Initialization commented out
- `PHASE_5_METRICS_WIRING_COMPLETE.md:126` - Deferred work section

---

## 🎯 CATEGORY 6: RESOURCE MANAGER CLEANUP (4,820 LOC)

**Priority:** LOW
**Blocking:** Code cleanliness
**Effort:** 8-12 hours

### Files to Remove/Refactor (Phase 6)

All located in `/crates/riptide-api/src/resource_manager/`:

- [ ] `mod.rs` (still used by AppState) - Migrate to ResourceFacade
- [ ] `memory_manager.rs` (used in tests) - Create test utilities
- [ ] `wasm_manager.rs` (AppState dependency) - Migrate to ResourceFacade
- [ ] `guards.rs` (still referenced) - Move to facade or remove
- [ ] `metrics.rs` (still referenced) - Migrate to BusinessMetrics
- [ ] `errors.rs` (still referenced) - Consolidate into RiptideError

**Migration Strategy:**
1. Create `ResourceFacade` wrapper for all functionality
2. Update `AppState` to use facade instead of manager
3. Update tests to use facade or mock implementations
4. Remove old resource_manager module
5. Delete 4,820 lines of legacy code

**Referenced In:**
- `PHASE_5_FILE_VERIFICATION.md:233-241` - Files marked for Phase 6 removal
- `PHASE_5_HANDLER_INTEGRATION_COMPLETE.md:369` - Hybrid approach until Phase 6

---

## 🎯 CATEGORY 7: MULTI-TENANCY ENHANCEMENTS

**Priority:** LOW
**Blocking:** None (foundation ready)
**Effort:** 4-6 hours

### Tenant ID Extraction (Temporary Hardcoded)

**Current Implementation:**
```rust
// File: /crates/riptide-api/src/handlers/pdf.rs:167
let tenant_id = "pdf-processing"; // TODO: Extract from request context in Phase 6

// File: /crates/riptide-api/src/handlers/spider.rs:83
let tenant_id = "spider-crawl"; // TODO: Extract from request context in Phase 6
```

### Tasks
- [ ] Implement JWT token parsing for tenant extraction
- [ ] Add tenant context middleware
- [ ] Update all handlers to use extracted tenant ID
- [ ] Add tenant-based rate limiting configuration
- [ ] Add tenant-based resource quotas

**Implementation:**
```rust
// Phase 6 multi-tenancy
pub async fn extract_tenant_id(headers: &HeaderMap) -> Result<String, ApiError> {
    let auth_header = headers
        .get("Authorization")
        .ok_or_else(|| ApiError::Unauthorized("Missing authorization"))?;

    let token = parse_jwt(auth_header)?;
    Ok(token.claims.tenant_id)
}

// Updated handler:
pub async fn process_pdf(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<PdfProcessRequest>,
) -> Result<Json<PdfProcessResponse>, ApiError> {
    let tenant_id = extract_tenant_id(&headers).await?;  // No longer hardcoded
    // ... rest of implementation
}
```

**Referenced In:**
- `PHASE_5_HANDLER_INTEGRATION_COMPLETE.md:25` - Hardcoded tenant IDs
- `PHASE_5_HANDLER_INTEGRATION_COMPLETE.md:304` - Temporary until Phase 6 auth

---

## 🎯 CATEGORY 8: TEST COVERAGE (INCOMPLETE)

**Priority:** HIGH
**Blocking:** Production readiness
**Effort:** 12-16 hours

### Missing Test Categories

#### Integration Tests (Blocked)
- [ ] Browser pool integration tests (requires Chrome/D-Bus)
- [ ] Spider crawl integration tests (requires network)
- [ ] PDF processing integration tests (requires pdfium)
- [ ] Multi-tenant workflow tests

#### Handler Tests (Missing)
- [ ] All 17 stub handlers need unit tests
- [ ] Error path coverage for all handlers
- [ ] Rate limiting integration tests
- [ ] Resource exhaustion scenarios

#### Facade Tests (Partial)
- [x] ProfileFacade: 237/237 tests passing
- [x] ReliabilityFacade: 56/56 tests passing
- [ ] StreamingFacade: Tests not implemented
- [ ] ResourceFacade: Tests not implemented
- [ ] MonitoringFacade: Tests not implemented

#### Current Test Results
```
riptide-facade:     237/237 passed ✅
riptide-reliability: 56/56 passed ✅
riptide-cache:       23/23 passed ✅
riptide-browser:     21/24 passed ⚠️  (3 D-Bus failures)
riptide-api:         0 tests (stubs) ❌
```

**Test Implementation Priority:**
1. **Critical:** Handler unit tests for completed handlers
2. **High:** Facade tests for StreamingFacade and ResourceFacade
3. **Medium:** Integration tests for core workflows
4. **Low:** Edge case and performance tests

**Referenced In:**
- `PHASE_5_INTEGRATION_TESTS_CRITICAL_FINDINGS.md:244` - Test coverage incomplete
- `PHASE_4_5_FINAL_COMPLETION_REPORT.md:277` - Integration tests ready but require implementation

---

## 🎯 CATEGORY 9: PERSISTENCE REFACTORING

**Priority:** LOW (Complete but needs testing)
**Blocking:** None
**Effort:** 2-3 hours

### Completed Work
- [x] Redis dependency unified to 0.27.6
- [x] Connection pool migration (conn → pool) completed
- [x] Async recursion bug fixed

### Remaining Tasks
- [ ] Add integration tests for persistence layer
- [ ] Performance benchmarks for connection pooling
- [ ] Error recovery scenarios testing
- [ ] Connection pool exhaustion handling

**Referenced In:**
- `PHASE_5_PERSISTENCE_MIGRATION.md` - Migration complete
- `PHASE_4_5_FINAL_COMPLETION_REPORT.md:93` - Field migration incomplete (now complete)

---

## 🎯 CATEGORY 10: MISSING TYPE IMPLEMENTATIONS

**Priority:** MEDIUM
**Blocking:** Some handler functionality
**Effort:** 3-4 hours

### CookieJar Methods (2 items)
```rust
// File: /crates/riptide-types/src/http/cookies.rs
impl CookieJar {
    // Missing methods:
    pub fn len(&self) -> usize { todo!() }
    pub fn values(&self) -> impl Iterator<Item = &Cookie> { todo!() }
}
```

### TableSummary Serialization
```rust
// File: /crates/riptide-types/src/extraction/table.rs
#[derive(Serialize, Deserialize)]  // Add Serialize derive
pub struct TableSummary {
    // ... existing fields
}
```

### ResourceResult Pattern Matching
```rust
// Add exhaustive match arms for all ResourceResult variants
match state.resource_facade.acquire_wasm_slot(tenant_id).await {
    Ok(FacadeResult::Success(slot)) => { /* ... */ },
    Ok(FacadeResult::RateLimited { retry_after }) => { /* ... */ },
    Ok(FacadeResult::MemoryPressure) => { /* ... */ },
    Ok(FacadeResult::ResourceExhausted) => { /* ... */ },
    Ok(FacadeResult::Timeout) => { /* ... */ },
    Err(e) => { /* ... */ },
}
```

**Referenced In:**
- `PHASE_5_FINAL_SUMMARY.md:133-136` - Missing fields and variants
- `NEXT_AGENT_INSTRUCTIONS.md:261` - Pattern matching instructions

---

## 📊 PRIORITY MATRIX

### CRITICAL (Must do before production)
1. Handler stub implementation (17 stubs)
2. Test coverage for handlers and facades
3. Missing facade methods (11 methods)

### HIGH (Should do soon)
4. Missing fields (6 fields)
5. Type implementations (CookieJar, TableSummary)
6. Metrics migration (341 warnings)

### MEDIUM (Can defer)
7. Streaming facade initialization
8. Multi-tenancy enhancements
9. Persistence testing

### LOW (Nice to have)
10. Resource manager cleanup (4,820 LOC)

---

## 🗓️ PHASE 6 ROADMAP

### Sprint 6.1: Handler Implementation (Week 1)
- Days 1-2: Monitoring handlers (10 stubs)
- Days 3-4: Session handlers (4 stubs)
- Day 5: Strategy + Memory handlers (3 stubs)

### Sprint 6.2: Facades & Methods (Week 2)
- Days 1-2: ProfileFacade methods (4 methods)
- Day 3: StreamingFacade initialization
- Days 4-5: Missing type implementations

### Sprint 6.3: Testing (Week 3)
- Days 1-2: Handler unit tests
- Days 3-4: Facade integration tests
- Day 5: Integration test cleanup

### Sprint 6.4: Cleanup & Migration (Week 4)
- Days 1-3: Metrics migration (341 warnings)
- Day 4: Multi-tenancy enhancements
- Day 5: Resource manager cleanup

**Total Estimated Timeline:** 4 weeks (20 working days)
**Effort:** 40-60 hours

---

## 📝 TRACKING STATUS

**Document Version:** 1.0
**Last Updated:** 2025-11-09
**Items Tracked:** ~150 tasks
**Completion:** 0% (all deferred to Phase 6)

### Next Actions
1. ✅ Complete Phase 4/5 validation (clippy + tests)
2. ⏳ Generate final completion report
3. ⏳ Commit all Phase 4/5 work
4. ⏳ Begin Phase 6 Sprint 6.1

---

## 🔗 REFERENCE DOCUMENTS

All deferred work extracted from:
- `PHASE_4_5_FINAL_COMPLETION_REPORT.md` - Main completion report
- `PHASE_5_FINAL_SUMMARY.md` - Error catalog
- `PHASE_5_HANDLER_INTEGRATION_COMPLETE.md` - Handler work
- `SPRINT_4.5_INTEGRATION_GUIDE.md` - Missing methods
- `PHASE_5_METRICS_WIRING_COMPLETE.md` - Metrics deferred work
- `PHASE_5_FILE_VERIFICATION.md` - Cleanup tasks
- `NEXT_AGENT_INSTRUCTIONS.md` - Fix patterns

**Total Documentation:** 15+ completion documents, ~280KB

---

**Generated by:** Phase 4/5 Completion Analysis
**Policy:** Zero-tolerance for errors (met ✅), Implementation deferred to Phase 6
