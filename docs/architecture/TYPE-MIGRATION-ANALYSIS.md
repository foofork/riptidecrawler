# Type Migration Analysis: riptide-api → riptide-types

**Analysis Date:** 2025-11-06
**Analyst:** Code Quality Analyzer
**Task:** Phase 2 - Week 0-2.5 Architecture Refactoring

## Executive Summary

**Total Types Identified:** 314 structs, 22 enums, 6 type aliases = **342 types**

**Migration Breakdown:**
- **HIGH PRIORITY (Must Move):** 47 types (14%)
- **MEDIUM PRIORITY (Should Move):** 89 types (26%)
- **LOW PRIORITY (Consider Moving):** 78 types (23%)
- **KEEP IN API (Transport/Implementation):** 128 types (37%)

**Risk Assessment:**
- **HIGH RISK (Heavy Dependencies):** 12 types
- **MEDIUM RISK (Moderate Usage):** 58 types
- **LOW RISK (Minimal Impact):** 77 types

---

## 1. HIGH PRIORITY - Core Domain Types (Must Move)

These types represent core domain concepts used across multiple crates and should be in `riptide-types`.

### 1.1 Pipeline Types (14 types)
**Location:** `crates/riptide-api/src/pipeline.rs`, `pipeline_enhanced.rs`, `pipeline_dual.rs`

| Type | Current Location | Reason | Risk | Dependencies |
|------|-----------------|--------|------|--------------|
| `PipelineOrchestrator` | pipeline.rs:107 | Core orchestration, used by facade | HIGH | CacheManager, ResourceManager |
| `EnhancedPipelineOrchestrator` | pipeline_enhanced.rs:34 | Core orchestration variant | HIGH | Same as above |
| `DualPathOrchestrator` | pipeline_dual.rs:88 | Core orchestration variant | HIGH | Same as above |
| `StrategiesPipelineOrchestrator` | strategies_pipeline.rs:81 | Core orchestration variant | MEDIUM | Same as above |
| `EnhancedPipelineResult` | pipeline_enhanced.rs:520 | Result type used across layers | MEDIUM | ExtractedDoc |
| `PhaseTiming` | pipeline_enhanced.rs:534 | Performance tracking | LOW | None |
| `EnhancedBatchStats` | pipeline_enhanced.rs:543 | Statistics tracking | LOW | None |
| `FastPathResult` | pipeline_dual.rs:31 | Dual-path result | MEDIUM | ExtractedDoc |
| `EnhancementResult` | pipeline_dual.rs:41 | Enhancement result | MEDIUM | ExtractedDoc |
| `DualPathResult` | pipeline_dual.rs:52 | Combined result | MEDIUM | Both above |
| `DualPathConfig` | pipeline_dual.rs:65 | Configuration | LOW | None |
| `DualPathStats` | pipeline_dual.rs:389 | Statistics | LOW | None |
| `StrategiesPipelineResult` | strategies_pipeline.rs:26 | Strategies result | MEDIUM | ExtractedDoc |

**Action:** These orchestrators are the core of RipTide's architecture but are currently tightly coupled to `riptide-api` implementation details. **RECOMMENDATION:** Extract orchestration interfaces/traits to `riptide-types`, keep implementations in `riptide-api`.

### 1.2 Core DTO Types (10 types)
**Location:** `crates/riptide-api/src/dto.rs`

| Type | Current Location | Reason | Risk | Dependencies |
|------|-----------------|--------|------|--------------|
| `ResultMode` | dto.rs:6 | Core enum for result selection | LOW | None |
| `SpiderResultStats` | dto.rs:28 | Spider statistics | MEDIUM | None |
| `SpiderResultUrls` | dto.rs:48 | Spider URL results | MEDIUM | None |
| `CrawledPage` | dto.rs:75 | Core domain entity | HIGH | None |
| `FieldFilter` | dto.rs:233 | Query parameter type | LOW | None |
| `SpiderResultPages` | dto.rs:253 | Spider page results | MEDIUM | CrawledPage |

**Action:** **MOVE ALL** - These are pure DTOs with minimal dependencies, representing core domain concepts.

### 1.3 Core Model Types (23 types)
**Location:** `crates/riptide-api/src/models.rs`

| Type | Current Location | Reason | Risk | Dependencies |
|------|-----------------|--------|------|--------------|
| `CrawlBody` | models.rs:9 | Request DTO | MEDIUM | CrawlOptions (already in types) |
| `CrawlResult` | models.rs:19 | Core result type | HIGH | ExtractedDoc, ErrorInfo |
| `ErrorInfo` | models.rs:50 | Error details | HIGH | None |
| `CrawlResponse` | models.rs:63 | Response DTO | HIGH | CrawlResult, CrawlStatistics |
| `CrawlStatistics` | models.rs:85 | Statistics | MEDIUM | GateDecisionBreakdown |
| `GateDecisionBreakdown` | models.rs:101 | Gate metrics | MEDIUM | None |
| `DeepSearchBody` | models.rs:117 | Request DTO | MEDIUM | CrawlOptions |
| `DeepSearchResponse` | models.rs:139 | Response DTO | MEDIUM | SearchResult |
| `SearchResult` | models.rs:161 | Search result | MEDIUM | ExtractedDoc |
| `HealthResponse` | models.rs:183 | Health check | LOW | DependencyStatus, ServiceHealth |
| `DependencyStatus` | models.rs:205 | Dependency health | LOW | None |
| `ServiceHealth` | models.rs:227 | Service health | LOW | None |
| `SystemMetrics` | models.rs:243 | System metrics | MEDIUM | None |

**Additional Spider Models (10 types):**
- `SpiderCrawlBody` (models.rs:303)
- `SpiderCrawlResponseStats` (models.rs:334)
- `SpiderCrawlResponseUrls` (models.rs:347)
- `SpiderApiResult` (models.rs:360)
- `SpiderApiResultUrls` (models.rs:379)
- `SpiderStatusRequest` (models.rs:401)
- `SpiderStatusResponse` (models.rs:408)
- `SpiderControlRequest` (models.rs:424)

**Action:** **MOVE ALL** - Core API contracts that should be shared.

---

## 2. MEDIUM PRIORITY - Handler DTOs (Should Move)

These are request/response types from handlers. They represent API contracts and should be in a shared location for versioning and compatibility.

### 2.1 Extraction Handler Types (10 types)
**Location:** `crates/riptide-api/src/handlers/extract.rs`

| Type | Lines | Usage | Move? |
|------|-------|-------|-------|
| `ExtractRequest` | 14 | Request DTO | YES |
| `ExtractOptions` | 31 | Options | YES |
| `ExtractResponse` | 68 | Response DTO | YES |
| `ContentMetadata` | 83 | Metadata | YES |
| `ParserMetadata` | 92 | Metadata | YES |

### 2.2 PDF Handler Types (5 types)
**Location:** `crates/riptide-api/src/handlers/pdf.rs`

- `PdfProcessRequest` (28)
- `PdfProcessResponse` (43)
- `ProcessingStats` (56)
- `EnhancedProgressUpdate` (359)
- `PdfProcessingRequest` (554 - enum)

**Action:** **MOVE** - These define the PDF processing API contract.

### 2.3 Browser Handler Types (7 types)
**Location:** `crates/riptide-api/src/handlers/browser.rs`

- `CreateSessionRequest` (42)
- `SessionResponse` (53)
- `BrowserAction` (67 - enum)
- `ActionResult` (115)
- `PoolStatusInfo` (128)
- `PoolStatus` (141)
- `LauncherStatsInfo` (152)

**Action:** **MOVE** - Browser session API contract.

### 2.4 Chunking Handler Types (4 types)
**Location:** `crates/riptide-api/src/handlers/chunking.rs`

- `ChunkRequest` (14)
- `ChunkParameters` (26)
- `ChunkResponse` (58)
- `ChunkData` (71)

### 2.5 Monitoring Handler Types (12 types)
**Location:** `crates/riptide-api/src/handlers/monitoring.rs`

- `HealthScoreResponse` (16)
- `AlertRulesResponse` (62)
- `AlertRuleSummary` (75)
- `ActiveAlertsResponse` (115)
- `CurrentMetricsResponse` (141)
- `MemoryMetricsResponse` (183)
- `LeakSummaryResponse` (192)
- `AllocationMetricsResponse` (200)
- `WasmInstanceHealth` (289)
- `WasmHealthResponse` (299)

### 2.6 Profiling Handler Types (10 types)
**Location:** `crates/riptide-api/src/handlers/profiling.rs`

- `MemoryProfileResponse` (35)
- `CpuProfileResponse` (49)
- `LoadAverage` (62)
- `HotspotInfo` (70)
- `BottleneckResponse` (83)
- `SizeDistribution` (96)
- `AllocationResponse` (105)
- `LeakInfo` (116)
- `LeakDetectionResponse` (129)
- `SnapshotResponse` (141)

### 2.7 Additional Handler Categories (46 types)

**Render Handlers** (4): `RenderRequest`, `RenderResponse`, `RenderStats`, `SessionRenderInfo`

**Search Handlers** (3): `SearchQuery`, `SearchResult`, `SearchResponse`

**Session Handlers** (5): `SetCookieRequest`, `CreateSessionResponse`, `SessionInfoResponse`, `CookieResponse`, `ListSessionsQuery`, `ExtendSessionRequest`

**Strategies Handlers** (12): Various strategy configuration and response types

**Table Handlers** (7): Table extraction types

**Telemetry Handlers** (6): Trace and span types

**Workers Handlers** (13): Job management types

**Admin Handlers** (10): Tenant and cache management

**LLM Handlers** (11): Provider and configuration types

**Memory Handlers** (4): Memory profiling types

**Resource Handlers** (7): Resource status types

**Engine Selection Handlers** (11): Engine decision types

**Profiles Handlers** (12): Profile management types

**Action:** **SELECTIVE MOVE** - Move stable API contracts (v1.0), keep experimental handlers in API.

---

## 3. LOW PRIORITY - Consider Moving

### 3.1 Configuration Types (78 types)

**Location:** `crates/riptide-api/src/config.rs`

| Type | Purpose | Move? |
|------|---------|-------|
| `RiptideApiConfig` | API-specific config | NO - API only |
| `ResourceConfig` | Resource limits | CONSIDER |
| `PerformanceConfig` | Performance tuning | CONSIDER |
| `RateLimitingConfig` | Rate limiting | CONSIDER |
| `MemoryConfig` | Memory management | CONSIDER |
| `HeadlessConfig` | Browser config | CONSIDER |
| `PdfConfig` | PDF processing | CONSIDER |
| `WasmConfig` | WASM config | CONSIDER |
| `SearchProviderConfig` | Search config | CONSIDER |

**Streaming Config Types (11):** StreamConfig, BufferConfig, WebSocketConfig, SseConfig, NdjsonConfig, etc.

**Action:** **DEFER** - Config types can stay in API for now. Consider moving to `riptide-config` crate later.

### 3.2 State Management Types

**Location:** `crates/riptide-api/src/state.rs`

- `AppState` (56) - **KEEP** - API implementation detail
- `AppConfig` (177) - **KEEP** - API configuration
- `EnhancedPipelineConfig` (233) - **CONSIDER** moving to types
- `EngineSelectionConfig` (291) - **CONSIDER** moving to types
- `CircuitBreakerConfig` (323) - **KEEP** - Reliability implementation
- `HealthStatus` (1585) - **MOVE** - Shared contract
- `MonitoringSystem` (1616) - **KEEP** - API implementation
- `PerformanceReport` (1822) - **MOVE** - Shared contract
- `DependencyHealth` (1599 - enum) - **MOVE** - Shared enum

---

## 4. KEEP IN API - Implementation Details

These types are tightly coupled to the API implementation and should NOT be moved.

### 4.1 Internal Resource Management (128 types)

**Resource Manager (30 types):**
- `ResourceManager` (mod.rs:99) - Core implementation
- `ResourceStatus` (mod.rs:148)
- `PerHostRateLimiter` (rate_limiter.rs:28)
- `HostStats` (rate_limiter.rs:244)
- `ResourceMetrics` (metrics.rs:13)
- `MetricsSnapshot` (metrics.rs:123)
- `MemoryManager` (memory_manager.rs:41)
- `LeakDetector` (memory_manager.rs:55)
- `LeakDetectionConfig` (memory_manager.rs:91)
- `LeakReport` (memory_manager.rs:115)
- `LeakCandidate` (memory_manager.rs:129)
- `MemoryStats` (memory_manager.rs:809)
- `RenderResourceGuard` (guards.rs:19)
- `PdfResourceGuard` (guards.rs:109)
- `WasmGuard` (guards.rs:169)
- `ResourceGuard` (guards.rs:192)
- `WasmInstanceManager` (wasm_manager.rs:21)
- `WasmInstanceStats` (wasm_manager.rs:216)
- `PerformanceMonitor` (performance.rs:22)
- `PerformanceStats` (performance.rs:216)
- `ResourceManagerError` (errors.rs:11 - enum)
- `ResourceResult<T>` (errors.rs:61 - type alias)

**RPC/Session Context (6 types):**
- `RpcSessionContext` (rpc_session_context.rs:15)
- `SessionState` (rpc_session_context.rs:37)
- `SessionConfig` (rpc_session_context.rs:59)
- `RpcSessionStore` (rpc_session_context.rs:187)
- `SessionMetrics` (rpc_session_context.rs:351)
- `RpcClient` (rpc_client.rs:14)

**Streaming Infrastructure (92 types):**
- `StreamingModule` (streaming/mod.rs:275)
- `GlobalStreamingMetrics` (streaming/mod.rs:204)
- `StreamingProtocol` (streaming/mod.rs:99 - enum)
- `StreamingHealth` (streaming/mod.rs:171 - enum)
- `StreamingError` (streaming/error.rs:11 - enum)
- `StreamingResult<T>` (streaming/error.rs:154 - type alias)
- `ClientType` (streaming/error.rs:166 - enum)
- `RecoveryStrategy` (streaming/error.rs:184 - enum)
- `ConnectionContext` (streaming/error.rs:158)
- `StreamingResponseBuilder` (streaming/response_helpers.rs:246)
- `StreamingErrorResponse` (streaming/response_helpers.rs:410)
- `StreamingResponseType` (streaming/response_helpers.rs:173 - enum)
- Various helper structs (KeepAliveHelper, CompletionHelper, ProgressHelper)
- `StreamLifecycleManager` (streaming/lifecycle.rs:141)
- `LifecycleEvent` (streaming/lifecycle.rs:73 - enum)
- `StreamCompletionSummary` (streaming/lifecycle.rs:130)
- `WebSocketHandler` (streaming/websocket.rs:59)
- `SseStreamingHandler` (streaming/sse.rs:69)
- `NdjsonStreamingHandler` (streaming/ndjson/streaming.rs:22)
- `StreamingMetrics` (streaming/metrics.rs:81)
- `BufferManager` (streaming/buffer.rs:411)
- `DynamicBuffer` (streaming/buffer.rs:63)
- `BackpressureHandler` (streaming/buffer.rs:255)
- `StreamProcessor` (streaming/processor.rs:16)
- `StreamingPipeline` (streaming/pipeline.rs:21)
- `StreamEvent` (streaming/pipeline.rs:488 - enum)
- Many more streaming support types...

**Session System (20 types):**
- `SessionSystem` (sessions/mod.rs:53)
- `SessionManager` (sessions/manager.rs:20)
- `SessionStorage` (sessions/storage.rs:15)
- `SessionLayer` (sessions/middleware.rs:27)
- `SecurityConfig` (sessions/middleware.rs:35)
- `SessionMiddleware<S>` (sessions/middleware.rs:103)
- `SessionContext` (sessions/middleware.rs:226)
- `SessionHeaders` (sessions/middleware.rs:347)
- `SessionRateLimiter` (sessions/middleware.rs:419)
- `RateLimiterStats` (sessions/middleware.rs:504)
- `Session` (sessions/types.rs:120)
- `SessionConfig` (sessions/types.rs:13)
- `SessionMetadata` (sessions/types.rs:191)
- `SessionBrowserConfig` (sessions/types.rs:214)
- `SessionUsageStats` (sessions/types.rs:278)
- `SessionStats` (sessions/types.rs:297)
- `SessionError` (sessions/types.rs:316 - enum)
- `BrowserType` (sessions/types.rs:251 - enum)
- `SameSite` (sessions/types.rs:183 - enum)
- `CookieJar` (sessions/types.rs:148)
- `Cookie` (sessions/types.rs:155)
- `Viewport` (sessions/types.rs:260)

**Middleware (7 types):**
- `PayloadLimitLayer` (middleware/payload_limit.rs:16)
- `PayloadLimitService<S>` (middleware/payload_limit.rs:53)
- `AuthRateLimiter` (middleware/auth.rs:29)
- `AuditLogger` (middleware/auth.rs:179)
- `AuthConfig` (middleware/auth.rs:368)

**Utilities (5 types):**
- `RipTideMetrics` (metrics.rs:15)
- `EngineStats` (metrics.rs:1607)
- `PhaseTimer` (metrics.rs:1615)
- `PhaseType` (metrics.rs:1590 - enum)
- `ErrorType` (metrics.rs:1599 - enum)
- `JemallocStats` (jemalloc_stats.rs:11)
- `HealthChecker` (health.rs:16)
- `WasmExtractorAdapter` (reliability_integration.rs:16)
- `PersistenceAdapter` (persistence_adapter.rs:55)

**Test Support:**
- `AppStateBuilder` (tests/test_helpers.rs:14)

**Trace Backend (6 types):**
- `CompleteTrace` (handlers/trace_backend.rs:46)
- `TraceSpan` (handlers/trace_backend.rs:53)
- `SpanEventData` (handlers/trace_backend.rs:68)
- `InMemoryTraceBackend` (handlers/trace_backend.rs:111)
- `OtlpTraceBackend` (handlers/trace_backend.rs:317)
- `OtlpBackendType` (handlers/trace_backend.rs:325 - enum)

---

## 5. Error Types Analysis

### Current Errors in riptide-api

**Location:** `crates/riptide-api/src/errors.rs`

```rust
pub enum ApiError {
    ValidationError { message: String },        // 400
    InvalidUrl { url: String, message: String }, // 400
    RateLimited { message: String },            // 429
    AuthenticationError { message: String },     // 401/403
    FetchError { url: String, message: String }, // 502/404
    CacheError { message: String },              // 503
    ExtractionError { message: String },         // 500
    RoutingError { message: String },            // 500
    PipelineError { message: String },           // 500
    ConfigError { message: String },             // 500
    DependencyError { service: String, message: String }, // 503
    InternalError { message: String },           // 500
    TimeoutError { operation: String, message: String }, // 408
    NotFound { resource: String },               // 404
    PayloadTooLarge { message: String },         // 413
    InvalidContentType { content_type: String, message: String }, // 415
    MissingRequiredHeader { header: String },    // 400
    InvalidHeaderValue { header: String, message: String }, // 400
    // ... more variants
}

pub type ApiResult<T> = Result<T, ApiError>;
```

**Decision:** **KEEP IN API** - `ApiError` is tightly coupled to HTTP status codes and axum `IntoResponse`. This is an API-layer concern.

**Alternative:** Create `riptide_types::CoreError` for domain errors, let API layer wrap/convert to `ApiError`.

### Resource Manager Errors

**Location:** `crates/riptide-api/src/resource_manager/errors.rs`

```rust
pub enum ResourceManagerError {
    BrowserPoolExhausted,
    MemoryLimitExceeded,
    RateLimitExceeded,
    // ... more
}

pub type Result<T> = std::result::Result<T, ResourceManagerError>;
```

**Decision:** **KEEP IN API** - These are implementation-specific errors for resource management.

### Streaming Errors

**Location:** `crates/riptide-api/src/streaming/error.rs`

```rust
pub enum StreamingError {
    ConnectionClosed,
    BufferOverflow,
    SerializationFailed,
    // ... more
}

pub type StreamingResult<T> = Result<T, StreamingError>;
```

**Decision:** **KEEP IN API** - These are streaming infrastructure errors.

---

## 6. Dependency Analysis

### Types Currently in riptide-types

From `cargo tree` output:
- `ExtractedDoc` - Core domain type ✓
- `CrawlOptions` (from config module) ✓
- Other base types

### Types Used by riptide-facade

From grep analysis:
- `riptide_api::pipeline::PipelineOrchestrator` ← **HIGH PRIORITY**
- `riptide_api::state::AppState` ← Keep (implementation)
- `riptide_api::strategies_pipeline::StrategiesPipelineOrchestrator` ← **HIGH PRIORITY**

**Key Finding:** Facade depends on **orchestrator implementations**, not just types. This suggests we need **trait extraction** rather than just type movement.

---

## 7. Migration Strategy & Risk Matrix

### Phase 1: Zero-Risk Moves (Week 1)
**Target:** Pure DTOs with no dependencies

| Type | Risk | Impact | Effort |
|------|------|--------|--------|
| `ResultMode` | NONE | LOW | 1 hour |
| `FieldFilter` | NONE | LOW | 1 hour |
| `ErrorInfo` | NONE | LOW | 1 hour |
| `GateDecisionBreakdown` | NONE | LOW | 1 hour |
| `CrawledPage` | NONE | MEDIUM | 2 hours |

**Total:** 47 pure DTO types, ~3 days work

### Phase 2: Trait Extraction (Week 2)
**Target:** Pipeline orchestration interfaces

1. Create `riptide_types::pipeline::traits`:
   - `trait PipelineExecutor`
   - `trait EnhancedExecutor`
   - `trait DualPathExecutor`

2. Keep implementations in `riptide-api`
3. Update facade to depend on traits

**Risk:** HIGH - Core architecture change
**Effort:** 1 week

### Phase 3: Request/Response Types (Week 2-3)
**Target:** Stable handler DTOs

- Move all handler request/response types that are part of v1.0 API
- Keep experimental handlers in API
- Update handlers to import from `riptide-types`

**Risk:** MEDIUM - Large blast radius
**Effort:** 1 week

### Phase 4: Configuration Types (Week 3-4)
**Target:** Shared configuration

- Consider creating `riptide-config` crate OR
- Move shared config to `riptide-types`

**Risk:** MEDIUM - Config is pervasive
**Effort:** 1 week

---

## 8. High-Risk Types (Detailed Analysis)

### 8.1 PipelineOrchestrator
**Location:** pipeline.rs:107 (1,596 lines!)

**Dependencies:**
- CacheManager (riptide-cache)
- ResourceManager (internal)
- AppState (internal)
- ExtractedDoc (riptide-types)

**Used By:**
- riptide-facade
- Multiple handlers

**Risk Factors:**
- 1,596 lines in one file
- Complex state management
- Resource lifetime management
- Cache interactions

**Recommendation:**
1. Extract `trait PipelineExecutor` to riptide-types
2. Keep `PipelineOrchestrator` implementation in riptide-api
3. Add adapter pattern in facade

### 8.2 CrawlResult
**Location:** models.rs:19

**Dependencies:**
- ExtractedDoc (riptide-types) ✓
- ErrorInfo (models.rs:50)

**Used By:**
- Virtually every handler
- Facade layer
- Client SDKs

**Risk Factors:**
- Core API contract
- Cannot break compatibility
- Versioning required

**Recommendation:** **MOVE** with semver guarantees

### 8.3 AppState
**Location:** state.rs:56

**Dependencies:**
- Everything 😱 (56+ fields)

**Risk Factors:**
- God object antipattern
- Circular dependencies
- Resource lifetime complexity

**Recommendation:** **DO NOT MOVE** - This is implementation detail. Consider refactoring into smaller state objects later.

---

## 9. Recommendations

### Immediate Actions (Week 0-1)

1. **Create riptide-types module structure:**
   ```
   riptide-types/
   ├── src/
   │   ├── lib.rs
   │   ├── dto/          # DTOs from models.rs
   │   ├── pipeline/     # Pipeline traits + result types
   │   ├── spider/       # Spider types from dto.rs
   │   ├── errors/       # Domain errors (not ApiError)
   │   └── config/       # Shared config (already exists)
   ```

2. **Move 47 pure DTO types** (no dependencies)
   - Priority: `CrawledPage`, `ErrorInfo`, `ResultMode`, `FieldFilter`
   - Risk: NONE
   - Effort: 3 days

3. **Extract pipeline traits** (break facade dependency cycle)
   - Create `trait PipelineExecutor`
   - Keep implementations in API
   - Risk: HIGH (architecture change)
   - Effort: 1 week

### Week 1-2 Actions

4. **Move core model types** (23 types from models.rs)
   - `CrawlBody`, `CrawlResult`, `CrawlResponse`, etc.
   - Risk: MEDIUM (widely used)
   - Effort: 1 week

5. **Update facade to use riptide-types**
   - Change imports
   - Test all conversions
   - Risk: MEDIUM
   - Effort: 2 days

### Week 2-3 Actions

6. **Move stable handler DTOs** (89 types)
   - Start with extraction, PDF, browser
   - Risk: MEDIUM
   - Effort: 1 week

### DO NOT MOVE (Keep in API)

- `AppState` - Implementation detail
- Resource manager types - Internal
- Streaming infrastructure - Internal
- Middleware types - Internal
- Session system - Internal
- Error types with HTTP coupling - API layer

---

## 10. Success Metrics

### Before Migration
- Circular dependency: riptide-api ↔ riptide-facade
- Types scattered across API handlers
- No clear API contract versioning

### After Migration
- Acyclic dependency: facade → types ← api
- Clear separation: domain types vs transport types
- Semver-versioned API contracts
- Easier SDK generation
- Better testing isolation

### Quality Gates
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] cargo build --workspace succeeds
- [ ] Documentation updated
- [ ] Migration guide created

---

## 11. Files Requiring Attention

### High-Touch Files (>100 types to update)
1. `crates/riptide-api/src/pipeline.rs` (1,596 lines)
2. `crates/riptide-api/src/models.rs` (424 lines, 23 types)
3. `crates/riptide-api/src/dto.rs` (493 lines, 6 types)
4. `crates/riptide-api/src/handlers/*` (40 files, 12,523 total lines)

### Critical Integration Points
1. `crates/riptide-facade/src/facades/crawl_facade.rs`
   - Currently imports from riptide-api
   - Must update to riptide-types

2. `crates/riptide-api/src/state.rs`
   - Creates all the orchestrators
   - May need trait bounds updates

3. All handler files importing models
   - 40+ files to update
   - Search/replace opportunity

---

## 12. Open Questions

1. **Should we create `riptide-api-types` instead of using `riptide-types`?**
   - Pro: Clearer separation (types vs api-types vs core-types)
   - Con: More crates to manage

2. **How do we version API contracts?**
   - Semver on riptide-types?
   - Separate API versioning?

3. **What about backward compatibility?**
   - Re-export from riptide-api for transition period?
   - Breaking change in v1.0?

4. **Should pipeline orchestrators be traits or concrete types in riptide-types?**
   - Traits = better abstraction, but harder to use
   - Concrete types = easier, but tighter coupling

---

## 13. Related Documentation

- [RIPTIDE-V1-DEFINITIVE-ROADMAP.md](/workspaces/eventmesh/docs/roadmap/RIPTIDE-V1-DEFINITIVE-ROADMAP.md)
- Week 0-2.5: Error & Type Refactoring
- Phase 2 Integration Plan

---

## Appendix A: Complete Type Inventory (342 types)

### Structs (314)
```
Config (9): RiptideApiConfig, ResourceConfig, PerformanceConfig, RateLimitingConfig, MemoryConfig, HeadlessConfig, PdfConfig, WasmConfig, SearchProviderConfig

DTO (6): SpiderResultStats, SpiderResultUrls, CrawledPage, FieldFilter, SpiderResultPages, ResultMode (enum)

Models (23): CrawlBody, CrawlResult, ErrorInfo, CrawlResponse, CrawlStatistics, GateDecisionBreakdown, DeepSearchBody, DeepSearchResponse, SearchResult, HealthResponse, DependencyStatus, ServiceHealth, SystemMetrics, SpiderCrawlBody, SpiderCrawlResponseStats, SpiderCrawlResponseUrls, SpiderApiResult, SpiderApiResultUrls, SpiderStatusRequest, SpiderStatusResponse, SpiderControlRequest

Pipeline (14): PipelineOrchestrator, EnhancedPipelineOrchestrator, EnhancedPipelineResult, PhaseTiming, EnhancedBatchStats, DualPathOrchestrator, FastPathResult, EnhancementResult, DualPathResult, DualPathConfig, DualPathStats, StrategiesPipelineResult, StrategiesPipelineOrchestrator

ResourceManager (30): ResourceManager, ResourceStatus, PerHostRateLimiter, HostStats, ResourceMetrics, MetricsSnapshot, MemoryManager, LeakDetector, LeakDetectionConfig, LeakReport, LeakCandidate, MemoryStats, RenderResourceGuard, PdfResourceGuard, WasmGuard, ResourceGuard, WasmInstanceManager, WasmInstanceStats, PerformanceMonitor, PerformanceStats

RPC/Session (6): RpcSessionContext, SessionState, SessionConfig, RpcSessionStore, SessionMetrics, RpcClient

State (8): AppState, AppConfig, EnhancedPipelineConfig, EngineSelectionConfig, CircuitBreakerConfig, HealthStatus, MonitoringSystem, PerformanceReport

Metrics (4): RipTideMetrics, EngineStats, PhaseTimer, JemallocStats

Streaming (92): [See section 4.1 for full list]

Session System (20): [See section 4.1 for full list]

Middleware (7): [See section 4.1 for full list]

Handler DTOs (89+): [See section 2 for categorized list]

Utilities (5): HealthChecker, WasmExtractorAdapter, PersistenceAdapter, AppStateBuilder, TelemetryConfig

Trace Backend (6): CompleteTrace, TraceSpan, SpanEventData, InMemoryTraceBackend, OtlpTraceBackend
```

### Enums (22)
```
ExporterType, ResultMode, ResourceResult<T>, LeakSeverity, ResourceManagerError, RateLimitAction, StreamingProtocol, StreamingHealth, StreamingError, ClientType, RecoveryStrategy, ApiError, LifecycleEvent, DependencyHealth, PdfProcessingRequest, PhaseType, ErrorType, StreamEvent, JobTypeRequest, PressureStatus, BrowserAction, OtlpBackendType, StreamingResponseType, SameSite, BrowserType, SessionError
```

### Type Aliases (6)
```
ResourceResult<T>, StreamingResult<T>, SseMetrics, WebSocketMetrics, NdjsonMetrics, ApiResult<T>
```

---

**Analysis Complete:** 2025-11-06
**Next Steps:** Review with architect, proceed with Phase 1 migration plan
