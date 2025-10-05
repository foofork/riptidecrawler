# RipTide Integration Roadmap - Active Implementation Plan

*Last Updated: 2025-10-04T22:30:00Z • Status: Focus on Remaining Infrastructure Activation*
*Progress Analysis: 61/256 tasks complete (23.8%) • Next: Phase 4A Quick Wins (2 days)*

## 🎯 CURRENT STATUS

**Completed Work**: 61 tasks from Phase 1-3 (see `docs/completed.md`)
- **Phase 1-3**: Event System, Circuit Breaker, Reliability, Monitoring, Enhanced Pipeline - **100% COMPLETE** ✅ (Completed: 2025-10-03)

**Remaining Work**: Infrastructure activation (170+ items identified from dead code analysis)
- **🔥 HIGH PRIORITY**: Dead code items (118 warnings) - **Code already exists, just needs activation**
- **Foundation Tasks**: FetchEngine (6 tasks), Cache Warming (6 tasks) - **Needs implementation**

**Key Insight**: Most roadmap items are **activation not creation** - infrastructure is built and tested, just suppressed with `#[allow(dead_code)]` from commits f7dd96a, d381456, 534ff5d

**Compilation Status**: ✅ `cargo check` passes with 0 errors

---

## ⚠️ PHASE 3 (PARTIAL) - Foundation Tasks Remaining

### 1. FetchEngine Integration ⚠️ **FOUNDATION ONLY** (75% Remaining)
**Location**: `riptide-core/src/fetch.rs`
**Impact**: Advanced HTTP client capabilities
**Status**: Foundation complete, implementation documented
**Priority**: MEDIUM

**Tasks**:
- [x] **FETCH-001**: Add `fetch_engine: Arc<FetchEngine>` to AppState
- [ ] **FETCH-002**: Configure per-host circuit breakers (documented in guide)
- [ ] **FETCH-003**: Replace raw http_client() calls in pipeline (documented)
- [ ] **FETCH-004**: Implement retry policies (documented)
- [ ] **FETCH-005**: Add request/response logging (documented)
- [ ] **FETCH-006**: Implement per-host rate limiting (documented)
- [ ] **FETCH-007**: Create GET /fetch/metrics endpoint (documented)
- [x] **FETCH-008**: Add FetchConfig to AppConfig

**Documentation**: Complete implementation guide at `docs/FETCH_ENGINE_CACHE_WARMING_IMPLEMENTATION.md`

**Expected Impact**:
- Per-host circuit breakers
- Automatic retry on network errors
- Better rate limiting
- Consistent HTTP handling

**Effort**: ~1 day

---

### 2. Cache Warming Integration ⚠️ **FOUNDATION ONLY** (75% Remaining)
**Location**: `riptide-core/src/cache_warming.rs`
**Impact**: Reduce cold-start latency
**Status**: Foundation complete, implementation documented
**Priority**: MEDIUM

**Tasks**:
- [x] **WARM-001**: Add CacheWarmer to AppState (Optional<Arc>)
- [ ] **WARM-002**: Implement popularity-based warming algorithm (documented)
- [ ] **WARM-003**: Add time-based warming scheduler (documented)
- [ ] **WARM-004**: Implement adaptive warming based on metrics (documented)
- [ ] **WARM-005**: Create GET /cache/warming/status endpoint (documented)
- [ ] **WARM-006**: Create POST /cache/warm trigger endpoint (documented)
- [ ] **WARM-007**: Add warming metrics collection integration (documented)
- [x] **WARM-008**: Add CacheWarmingConfig to AppConfig

**Documentation**: Complete implementation guide at `docs/FETCH_ENGINE_CACHE_WARMING_IMPLEMENTATION.md`

**Expected Impact**:
- Higher cache hit rates
- Lower cold-start latency
- Better performance optimization
- Proactive cache population

**Effort**: ~1 day

---

## 🚧 PHASE 4: PLANNED INFRASTRUCTURE ACTIVATION

### 3. Streaming Infrastructure ⚠️ **PREPARED - NOT ACTIVATED** (0% Active)
**Location**: `riptide-api/src/streaming/`
**Impact**: Real-time streaming capabilities for web content extraction
**Status**: Complete infrastructure (64 items), awaiting route activation
**Total Items**: 64 warnings suppressed (f7dd96a)
**Priority**: 🔥 **HIGH** - Code exists, just needs activation (remove `#[allow(dead_code)]`)

**Root Cause**: Complete streaming system (NDJSON, SSE, WebSocket) prepared but endpoints not activated

#### Streaming Core (streaming/mod.rs, processor.rs, pipeline.rs)
- [ ] **STREAM-001**: Activate StreamingPipeline orchestrator
- [ ] **STREAM-002**: Enable StreamProcessor for stream handling
- [ ] **STREAM-003**: Integrate StreamingModule into main.rs routes
- [ ] **STREAM-004**: Add streaming protocol selection (NDJSON/SSE/WebSocket)
- [ ] **STREAM-005**: Test stream processing logic (13 items in processor.rs)
- [ ] **STREAM-006**: Validate pipeline orchestration (10 items in pipeline.rs)

#### Streaming Protocols
- [ ] **STREAM-007**: Activate NDJSON streaming endpoint (streaming/ndjson/streaming.rs)
- [ ] **STREAM-008**: Activate SSE (Server-Sent Events) endpoint (streaming/sse.rs, 2 items)
- [ ] **STREAM-009**: Activate WebSocket streaming endpoint (streaming/websocket.rs, 3 items)
- [ ] **STREAM-010**: Implement keep-alive messages (KeepAliveHelper, 4 methods)
- [ ] **STREAM-011**: Add streaming error handling (StreamingErrorResponse, 4 methods)

#### Streaming Infrastructure
- [ ] **STREAM-012**: Enable BufferManager for stream buffering (streaming/buffer.rs, 3 items)
- [ ] **STREAM-013**: Activate StreamLifecycleManager (streaming/lifecycle.rs, 5 items)
- [ ] **STREAM-014**: Configure StreamingConfig (streaming/config.rs, 1 item)
- [ ] **STREAM-015**: Wire StreamingError types (streaming/error.rs, 6 items)

**Expected Impact**:
- Real-time web content streaming
- Support for 3 streaming protocols (NDJSON, SSE, WebSocket)
- Efficient large document handling
- Progressive result delivery

**Effort**: 2-3 days

---

### 4. Session Management System ⚠️ **PREPARED - NOT ACTIVATED** (0% Active)
**Location**: `riptide-api/src/sessions/`
**Impact**: Browser state persistence and session continuity
**Status**: Complete session system (19 items), awaiting integration
**Total Items**: 19 warnings suppressed (f7dd96a)
**Priority**: 🔥 **HIGH** - Code exists, just needs activation (remove `#[allow(dead_code)]`)

**Root Cause**: Full session system with browser state persistence ready but not integrated

#### Session Core (sessions/mod.rs, manager.rs)
- [ ] **SESSION-001**: Activate SessionSystem orchestrator (2 items)
- [ ] **SESSION-002**: Enable SessionManager lifecycle management (6 items)
- [ ] **SESSION-003**: Implement get_or_create_session API
- [ ] **SESSION-004**: Add session persistence to storage backend
- [ ] **SESSION-005**: Configure session timeout and cleanup

#### Session Middleware (sessions/middleware.rs, types.rs)
- [ ] **SESSION-006**: Integrate SessionMiddleware into Axum (4 items)
- [ ] **SESSION-007**: Add session context to request handlers
- [ ] **SESSION-008**: Implement cookie management (7 items in types.rs)
- [ ] **SESSION-009**: Configure secure session cookies (HttpOnly, Secure, SameSite)
- [ ] **SESSION-010**: Add session validation and expiry checks

**Expected Impact**:
- Stateful extraction workflows
- Browser state persistence across requests
- User session tracking
- Cookie-based authentication support

**Effort**: 1-2 days

---

### 5. Advanced Extraction Strategies ⚠️ **PARTIALLY ACTIVATED** (40% Active)
**Location**: `riptide-api/src/handlers/strategies.rs`, `tables.rs`
**Impact**: Enhanced extraction capabilities with multiple strategies
**Status**: Basic strategy support active, advanced features pending
**Total Items**: 11 warnings suppressed (534ff5d)
**Priority**: 🟡 **MEDIUM** - Partial activation (CSS/Regex HIGH, LLM LOW - defer)

**Root Cause**: CSS_JSON, REGEX, LLM strategies and advanced table features prepared but not implemented

#### Strategy Enhancements (strategies.rs)
- [ ] **STRAT-007**: Implement CSS_JSON strategy with custom selectors
  - Field: `css_selectors: Option<Vec<String>>` (line 46)
  - Struct: `RegexPatternRequest` (4 fields, line 79)
- [ ] **STRAT-008**: Implement REGEX extraction strategy
  - Fields: `regex_patterns: Option<Vec<RegexPatternRequest>>` (line 50)
  - Pattern matching with named capture groups
- [ ] **STRAT-009**: Implement LLM extraction strategy
  - Field: `llm_config: Option<LlmConfigRequest>` (line 54)
  - Struct: `LlmConfigRequest` (3 fields, line 89)
  - AI-powered content extraction
- [ ] **STRAT-010**: Add metrics collection toggle (enable_metrics, line 32)
- [ ] **STRAT-011**: Add schema validation toggle (validate_schema, line 37)
- [ ] **STRAT-012**: Implement cache mode selection (cache_mode, line 42)

#### Table Extraction Enhancements (tables.rs)
- [ ] **TABLE-001**: Implement header inclusion toggle (include_headers, line 38)
- [ ] **TABLE-002**: Implement data type detection (detect_data_types, line 45)
- [ ] **TABLE-003**: Add table structure validation
- [ ] **TABLE-004**: Support complex table layouts (merged cells, headers)

**Expected Impact**:
- 200% increase in extraction strategy options (CSS, Regex, LLM)
- Intelligent data type detection for tables
- Schema validation for structured data
- Flexible extraction configuration

**Effort**: 1 day

---

### 6. Advanced Metrics & Observability ⚠️ **PARTIALLY ACTIVATED** (45% Active)
**Location**: `riptide-api/src/metrics.rs`
**Impact**: Comprehensive production metrics and monitoring
**Status**: Core metrics active, advanced metrics prepared
**Total Items**: 31 warnings suppressed (f7dd96a)
**Priority**: 🔥 **HIGH** - Code exists, just needs activation (remove `#[allow(dead_code)]`)

**Root Cause**: Comprehensive Prometheus metrics prepared but not all integrated

#### Phase Timing Metrics
- [ ] **METRIC-001**: Activate phase timing histograms (4 fields)
  - `fetch_duration`, `parse_duration`, `extract_duration`, `total_duration`
- [ ] **METRIC-002**: Implement PhaseTimer struct for timing tracking
- [ ] **METRIC-003**: Add PhaseType enum for phase categorization

#### Error Metrics
- [ ] **METRIC-004**: Enable error counters by type (3 fields)
  - Network errors, parsing errors, extraction errors
- [ ] **METRIC-005**: Add error classification and tracking
- [ ] **METRIC-006**: Implement error rate alerting

#### Streaming Metrics (3 fields + 3 methods)
- [ ] **METRIC-007**: Track streaming connection metrics
- [ ] **METRIC-008**: Monitor buffer utilization
- [ ] **METRIC-009**: Record streaming throughput
- [ ] **METRIC-010**: Implement streaming error tracking

#### PDF Metrics (9 fields + 5 methods)
- [ ] **METRIC-011**: Track PDF processing duration
- [ ] **METRIC-012**: Monitor PDF conversion errors
- [ ] **METRIC-013**: Record PDF size and page count
- [ ] **METRIC-014**: Add PDF quality metrics
- [ ] **METRIC-015**: Implement PDF cache hit rate

#### WASM Metrics (6 fields + 4 methods)
- [ ] **METRIC-016**: Track WASM execution time
- [ ] **METRIC-017**: Monitor WASM memory usage
- [ ] **METRIC-018**: Record WASM initialization overhead
- [ ] **METRIC-019**: Add WASM error tracking

**Expected Impact**:
- Complete observability of all pipeline phases
- Detailed error classification and tracking
- Performance bottleneck identification
- Production-grade monitoring

**Effort**: 1 day

---

### 7. Advanced Health Checks ⚠️ **PARTIALLY ACTIVATED** (30% Active)
**Location**: `riptide-api/src/health.rs`
**Impact**: Comprehensive system health monitoring
**Status**: Basic health endpoint active, advanced diagnostics prepared
**Total Items**: 14 warnings suppressed (f7dd96a)
**Priority**: 🔥 **HIGH** - Code exists, just needs activation (remove `#[allow(dead_code)]`)

**Root Cause**: Advanced health checks prepared but `/health` endpoint uses basic version

#### Health Checker Enhancements (14 items total)
- [ ] **HEALTH-001**: Activate git_sha build information (field)
- [ ] **HEALTH-002**: Add build_timestamp tracking (field)
- [ ] **HEALTH-003**: Implement component_versions tracking (field)
- [ ] **HEALTH-004**: Enable dependency checking methods (8 methods)
  - Database connectivity checks
  - External service health probes
  - Resource availability checks
  - Cache health validation
- [ ] **HEALTH-005**: Activate ComprehensiveSystemMetrics (struct)
- [ ] **HEALTH-006**: Add system metrics helpers (3 functions)
  - CPU usage monitoring
  - Memory pressure detection
  - Disk space warnings

**Expected Impact**:
- Comprehensive dependency health checks
- Build information tracking
- System resource monitoring
- Proactive failure detection

**Effort**: 4 hours

---

### 8. Production Resource Management ⚠️ **PARTIALLY ACTIVATED** (40% Active)
**Location**: `riptide-api/src/resource_manager.rs`
**Impact**: Advanced resource controls for production scaling
**Status**: Basic resource tracking active, advanced controls prepared
**Total Items**: 10 warnings suppressed (f7dd96a)
**Priority**: 🔥 **HIGH** - Code exists, just needs activation (remove `#[allow(dead_code)]`)

**Root Cause**: Production resource controls prepared for scale

#### Resource Metrics & Guards (10 items total)
- [ ] **RESOURCE-001**: Activate ResourceMetrics fields (3 fields)
  - `headless_pool_size`, `headless_active`, `pdf_active`
- [ ] **RESOURCE-002**: Implement ResourceGuard for automatic cleanup
- [ ] **RESOURCE-003**: Enable PdfResourceGuard for PDF processing
- [ ] **RESOURCE-004**: Activate acquire_pdf_resources method
- [ ] **RESOURCE-005**: Implement ResourceStatus tracking (7 fields)
  - Pool status, active connections, queue depth
- [ ] **RESOURCE-006**: Add ResourceResult::Error variant handling
- [ ] **RESOURCE-007**: Configure resource limits and thresholds
- [ ] **RESOURCE-008**: Implement resource pressure backoff

**Expected Impact**:
- Automatic resource cleanup (RAII pattern)
- Pool management for headless browsers
- PDF processing resource limits
- Production-grade resource controls

**Effort**: 4 hours

---

### 9. Application State & Configuration ⚠️ **PARTIALLY CONFIGURED** (60% Active)
**Location**: `riptide-api/src/state.rs`
**Impact**: Complete application configuration
**Status**: Core state active, advanced configuration fields prepared
**Total Items**: 8 warnings suppressed (f7dd96a)
**Priority**: HIGH

**Root Cause**: State fields exist for features not yet activated

#### AppState Fields (6 fields)
- [ ] **STATE-001**: Activate health_checker field integration
- [ ] **STATE-002**: Enable telemetry field usage
- [ ] **STATE-003**: Integrate pdf_metrics collection
- [ ] **STATE-004**: Activate performance_metrics tracking
- [ ] **STATE-005**: Wire up monitoring_system
- [ ] **STATE-006**: Enable circuit_breaker_state

#### Configuration Structs
- [ ] **STATE-007**: Implement MonitoringConfig full configuration
- [ ] **STATE-008**: Activate EnhancedPipelineConfig (7 fields)
  - Phase timing configuration
  - Debugging toggles
  - Performance tuning parameters
- [ ] **STATE-009**: Enable CircuitBreakerConfig integration
- [ ] **STATE-010**: Add new_with_api_config constructor

**Expected Impact**:
- Complete feature configuration
- All prepared infrastructure activated
- Unified configuration management

**Effort**: 4 hours

---

### 10. Telemetry Advanced Features ⚠️ **INFRASTRUCTURE READY** (0% Active)
**Location**: `riptide-api/src/handlers/telemetry.rs`, `telemetry_config.rs`
**Impact**: Advanced trace visualization and telemetry management
**Status**: Complete telemetry infrastructure (12 items), awaiting activation
**Total Items**: 12 warnings suppressed (f7dd96a)
**Priority**: 🔥 **HIGH** - Code exists, just needs activation (remove `#[allow(dead_code)]`)

**Root Cause**: Advanced telemetry visualization and config not exposed in routes

#### Telemetry Handlers (9 items)
- [ ] **TELEM-008**: Expose TraceQueryParams endpoint
- [ ] **TELEM-009**: Activate TraceMetadata visualization
- [ ] **TELEM-010**: Enable SpanNode tree visualization
- [ ] **TELEM-011**: Implement list_traces endpoint
- [ ] **TELEM-012**: Activate get_trace_tree visualization
- [ ] **TELEM-013**: Enable get_telemetry_status dashboard

#### Telemetry Configuration (3 items)
- [ ] **TELEM-014**: Activate init_tracing advanced initialization
- [ ] **TELEM-015**: Implement shutdown graceful cleanup
- [ ] **TELEM-016**: Add parse_trace_id helper
- [ ] **TELEM-017**: Add parse_span_id helper

**Expected Impact**:
- Advanced trace visualization UI
- Telemetry management endpoints
- Graceful telemetry shutdown
- Production trace debugging

**Effort**: 4 hours

---

### 11. Worker Management ⚠️ **ENDPOINT PREPARED** (0% Active)
**Location**: `riptide-api/src/handlers/workers.rs`
**Impact**: Worker job listing and management
**Status**: Query struct prepared (1 item), endpoint not implemented
**Total Items**: 1 warning suppressed (f7dd96a)
**Priority**: 🔥 **HIGH** - Code exists, just needs activation (remove `#[allow(dead_code)]`)

**Root Cause**: Job listing query params defined but endpoint not implemented

#### Worker Job Listing
- [ ] **WORKER-001**: Implement job listing endpoint using JobListQuery
- [ ] **WORKER-002**: Add job filtering by status
- [ ] **WORKER-003**: Add job pagination support
- [ ] **WORKER-004**: Implement job search by ID/type
- [ ] **WORKER-005**: Add job metrics and statistics

**Expected Impact**:
- Worker job visibility
- Job queue management
- Worker performance monitoring

**Effort**: 2 hours

---

## 📊 ROADMAP SUMMARY METRICS

### Implementation Status (Updated 2025-10-04T22:30:00Z - Post Dead Code Analysis)
- **✅ Completed (Phase 1-3)**: 61 tasks - **100% COMPLETE** (Completed: 2025-10-03) (see `docs/completed.md`)
- **⚠️ Foundation Tasks**: 2 features, 12 tasks - **25% COMPLETE** (3/12 tasks done)
- **⚠️ Planned Infrastructure (Phase 4)**: 9 feature groups, 140 sub-tasks - **0% ACTIVE**
- **Total Remaining Work**: 195 tasks
- **Overall Completion**: 61/256 tasks = **23.8%**

### Progress Summary
| Phase | Tasks | Complete | % | Status | Timeline |
|-------|-------|----------|---|--------|----------|
| Phase 1-3 | 61 | 61 | 100% | ✅ COMPLETE | Completed 2025-10-03 |
| Phase 3 (Partial) | 12 | 3 | 25% | 🟡 FOUNDATION | 2 days remaining |
| Phase 4A (Quick Wins) | 63 | 0 | 0% | ⚠️ READY | 2 days |
| Phase 4B (Infrastructure) | 77 | 0 | 0% | ⚠️ PREPARED | 3 days |
| Phase 5 (Implementation) | 43 | 0 | 0% | 🔨 NEEDS CODE | 3 days |
| Phase 6 (Deferred) | 3 | 0 | 0% | ⏸️ FUTURE | 1-2 weeks |
| **TOTAL** | **259** | **64** | **24.7%** | **IN PROGRESS** | **8-11 days** |

### Prepared Infrastructure Breakdown (Prioritized by Dead Code Analysis)

**🔥 HIGH PRIORITY - Code Exists (Just Activate)**
| Feature | Items | Status | Effort | Notes |
|---------|-------|--------|--------|-------|
| Streaming Infrastructure | 64 items | 0% Active | 2-3 days | **Remove `#[allow(dead_code)]` + add routes** |
| Session Management | 19 items | 0% Active | 1-2 days | **Remove `#[allow(dead_code)]` + integrate middleware** |
| Advanced Metrics | 31 items | 45% Active | 1 day | **Remove `#[allow(dead_code)]` + wire integration points** |
| Advanced Health Checks | 14 items | 30% Active | 4 hours | **Remove `#[allow(dead_code)]` + expose endpoints** |
| Resource Management | 10 items | 40% Active | 4 hours | **Remove `#[allow(dead_code)]` + activate guards** |
| Telemetry Features | 12 items | 0% Active | 4 hours | **Remove `#[allow(dead_code)]` + expose routes** |
| Worker Management | 1 item | 0% Active | 2 hours | **Remove `#[allow(dead_code)]` + implement endpoint** |
| Application State | 8 items | 60% Active | 4 hours | **Remove `#[allow(dead_code)]` + activate fields** |
| **SUBTOTAL (Dead Code)** | **159 items** | **~25% Active** | **6-8 days** | **All code exists** ✅ |

**🟡 MEDIUM PRIORITY - Needs Implementation**
| Feature | Items | Status | Effort | Notes |
|---------|-------|--------|--------|-------|
| FetchEngine | 8 tasks | 25% Complete | 1 day | **Implement missing methods** |
| Cache Warming | 8 tasks | 25% Complete | 1 day | **Implement algorithms** |
| Advanced Strategies (CSS/Regex) | 8 items | 50% Active | 4 hours | **Implement CSS_JSON + REGEX** |
| **SUBTOTAL (Implementation)** | **24 items** | **~33% Active** | **2-3 days** | **Needs coding** 🔨 |

**🔴 LOW PRIORITY - Defer**
| Feature | Items | Status | Effort | Notes |
|---------|-------|--------|--------|-------|
| LLM Strategy | 3 items | 0% Active | 1-2 weeks | **Defer to Phase 7** ⏸️ |

**📊 TOTAL REMAINING** | **186 items** | **~25% Active** | **8-11 days** |

---

## 🎯 REVISED EXECUTION PLAN (Prioritizing Dead Code Activation)

### 🔥 Phase 4A: Quick Wins - Activate Existing Code (2 days) ⏳ **NEXT UP**
**Priority**: Remove `#[allow(dead_code)]` and activate prepared infrastructure
**Status**: 0/63 items complete (0%)
**Start Date**: TBD
**Target Completion**: +2 days from start

#### Tasks Breakdown
1. **Application State & Configuration** (4 hours) - Enable all prepared config fields
   - [ ] STATE-001 to STATE-008 (8 items)
   - **Status**: 0% complete

2. **Advanced Metrics** (1 day) - Activate phase timing, error tracking, PDF/WASM metrics
   - [ ] METRIC-001 to METRIC-019 (31 items)
   - **Status**: 0% complete

3. **Advanced Health Checks** (4 hours) - Expose comprehensive health endpoints
   - [ ] HEALTH-001 to HEALTH-006 (14 items)
   - **Status**: 0% complete

4. **Resource Management** (4 hours) - Activate resource guards and pool management
   - [ ] RESOURCE-001 to RESOURCE-008 (10 items)
   - **Status**: 0% complete

**Deliverable**: 63 items activated (Application State + Metrics + Health + Resources)
**Effort**: 2 days (16 hours)
**Risk**: LOW - Code already exists and compiles
**Dependencies**: None (can start immediately)
**Validation**: Zero dead_code warnings, all endpoints operational

---

### 🔥 Phase 4B: Core Infrastructure Activation (3 days) 📋 **QUEUED**
**Priority**: Activate major infrastructure systems
**Status**: 0/77 items complete (0%)
**Dependencies**: Phase 4A completion (Application State + Metrics)
**Target Completion**: +3 days from Phase 4A completion

#### Tasks Breakdown
1. **Worker Management** (2 hours) - Activate job listing endpoint
   - [ ] WORKER-001 to WORKER-005 (5 items)
   - **Status**: 0% complete
   - **Blocker**: None

2. **Telemetry Features** (4 hours) - Expose trace visualization endpoints
   - [ ] TELEM-008 to TELEM-017 (12 items)
   - **Status**: 0% complete
   - **Blocker**: Requires STATE-002 (telemetry field)

3. **Streaming Infrastructure** (2-3 days) - Activate NDJSON/SSE/WebSocket routes
   - [ ] STREAM-001 to STREAM-015 (60 items)
   - **Status**: 0% complete
   - **Blocker**: Requires METRIC-007 to METRIC-010 (streaming metrics)

**Deliverable**: 77 items activated (Workers + Telemetry + Streaming)
**Effort**: 3 days (24 hours)
**Risk**: MEDIUM - Streaming needs route integration
**Critical Path**: Application State → Metrics → Streaming

---

### 🟡 Phase 5: Implementation Work (3 days) 📋 **QUEUED**
**Priority**: Complete partially implemented features
**Status**: 0/43 items complete (0%)
**Dependencies**: Phase 4A and 4B completion
**Target Completion**: +3 days from Phase 4B completion

#### Tasks Breakdown
1. **FetchEngine** (1 day) - Implement per-host circuit breakers, retry policies, rate limiting
   - [ ] FETCH-002 to FETCH-007 (6 items)
   - **Status**: 2/8 complete (25%)
   - **Blocker**: None (can run in parallel)

2. **Cache Warming** (1 day) - Implement warming algorithms and scheduler
   - [ ] WARM-002 to WARM-007 (6 items)
   - **Status**: 2/8 complete (25%)
   - **Blocker**: Requires FETCH-006 (FetchEngine rate limiting)

3. **Advanced Strategies (CSS/Regex)** (4 hours) - Implement CSS_JSON and REGEX strategies
   - [ ] STRAT-007, STRAT-008, STRAT-010 to STRAT-012 (5 items)
   - [ ] TABLE-001 to TABLE-004 (4 items)
   - **Status**: 0/9 complete (0%)
   - **Blocker**: Requires METRIC-010 (metrics toggle)

4. **Session Management** (1 day) - Wire session middleware (code exists, needs integration)
   - [ ] SESSION-001 to SESSION-010 (10 items)
   - **Status**: 0/10 complete (0%)
   - **Blocker**: None (can run in parallel)

**Deliverable**: 43 items completed (FetchEngine + Cache + Strategies + Sessions)
**Effort**: 3 days (24 hours)
**Risk**: MEDIUM - Requires new implementation
**Parallel Opportunities**: FetchEngine + Session Management can run concurrently

---

### 🔴 Phase 6: Deferred (Future Sprint) ⏸️ **ON HOLD**
**Priority**: High-complexity items for later
**Status**: 0/3 items complete (0%)
**Dependencies**: Phase 5 completion + Architecture review
**Target Start**: After Phase 5 + 1 sprint buffer

#### Deferred Items
1. **LLM Strategy** (1-2 weeks) - AI-powered extraction (STRAT-009)
   - [ ] Implement LLM extraction strategy
   - [ ] Add llm_config field integration
   - [ ] Configure LlmConfigRequest struct
   - **Reason for Deferral**: High complexity, requires external API integration, prompt engineering
   - **Dependencies**: External LLM API contract, prompt templates, evaluation framework

**Deliverable**: 3 items deferred to future sprint
**Effort**: 1-2 weeks (40-80 hours)
**Risk**: HIGH - Requires external API integration, prompt engineering, performance tuning
**Recommendation**: Conduct feasibility study and architecture review before implementation

---

## 📊 Summary & Timeline

### Current Progress (2025-10-04T22:30:00Z)
- **Total Tasks**: 259 tasks
- **Completed**: 64 tasks (24.7%)
  - Phase 1-3: 61 tasks (100%) ✅ Completed 2025-10-03
  - Phase 3 Partial: 3 tasks (25%)
- **Remaining**: 195 tasks (75.3%)
  - Phase 4A: 63 items (0%)
  - Phase 4B: 77 items (0%)
  - Phase 5: 43 items (0%)
  - Phase 6: 3 items (deferred)

### Execution Timeline
**Total Timeline**: **8-11 days** (excluding deferred items)

| Phase | Duration | Items | Status | Dependencies |
|-------|----------|-------|--------|--------------|
| Phase 4A (Quick Wins) | 2 days | 63 | ⏳ NEXT | None |
| Phase 4B (Infrastructure) | 3 days | 77 | 📋 QUEUED | Phase 4A |
| Phase 5 (Implementation) | 3 days | 43 | 📋 QUEUED | Phase 4A, 4B |
| Phase 6 (Deferred) | 1-2 weeks | 3 | ⏸️ FUTURE | Architecture review |

**Total Activated (Planned)**: **183 of 186 items** (98.4%)
**Deferred**: **3 items** (LLM strategy - Phase 6)

### Critical Path
```
Start → Phase 4A (2d) → Phase 4B (3d) → Phase 5 (3d) → Complete
         ↓                ↓                ↓
      State Fields    Streaming      FetchEngine
      Metrics         Telemetry      Cache Warming
      Health          Workers        Sessions
      Resources                      Strategies
```

### Parallel Execution Opportunities
- **Day 1-2**: Application State + Advanced Metrics (concurrent)
- **Day 3**: Health Checks + Resource Management (concurrent)
- **Day 4**: Worker Management + Telemetry (concurrent)
- **Day 5-7**: Streaming Infrastructure (sequential - route integration required)
- **Day 8-10**: FetchEngine + Session Management (concurrent)

---

## 📝 Items Intentionally Excluded

The following items from the original integration gaps analysis are **NOT included** in this roadmap:

### 1. Integrated Cache (IntegratedCacheManager)
- **Status**: ❌ Excluded
- **Reason**: Current `CacheManager` is working fine. `IntegratedCacheManager` adds unnecessary complexity.
- **Priority**: LOW - Skip

### 2. Spider Query-Aware Components
- **Status**: ❌ Excluded
- **Reason**: Specialized for deep crawling, not applicable to single-URL extraction
- **Priority**: LOW - Not applicable to main pipeline

### 3. Chunking Endpoint (`/chunk`)
- **Status**: ❌ Excluded
- **Reason**: Internal usage in deepsearch is sufficient
- **Priority**: LOW - Internal usage sufficient

---

## 📚 Documentation References

- **Completed Tasks**: `docs/completed.md` (Phases 1-3, 61 tasks)
- **Dead Code Analysis**: `docs/dead-code-categorization-analysis.md` (f7dd96a)
- **Implementation Guides**: `docs/FETCH_ENGINE_CACHE_WARMING_IMPLEMENTATION.md`
- **Commit Details**:
  - Comprehensive dead code elimination (f7dd96a) - 118 warnings resolved
  - Streaming helpers (d381456) - 8 items documented
  - Strategies & tables (534ff5d) - 11 items documented

---

*This roadmap focuses on remaining infrastructure activation identified through dead code analysis. All completed work has been archived to `docs/completed.md`.*

**Source**: Based on comprehensive integration gaps analysis (archived at `docs/archive/INTEGRATION_GAPS_ANALYSIS.md`) and dead code analysis from commits f7dd96a, d381456, 534ff5d
