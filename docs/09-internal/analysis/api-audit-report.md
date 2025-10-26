# RipTide API Crate Audit - Missing Functionality Report
**Generated**: 2025-10-24
**Scope**: Phase 9, 10, 10.4 features + core API completeness

---

## Executive Summary

**Total Missing Features**: 9 major features
**Incomplete Features**: 3 partially implemented
**Complete Features**: 8 fully implemented

**Priority Breakdown**:
- **P0 (Critical)**: 2 features - Domain profiles, Engine selection APIs
- **P1 (High)**: 2 features - Advanced browser pool, Engine metrics
- **P2 (Medium)**: 2 features - Query-aware crawling, Real-time profiling streams

---

## PHASE 10.4: Domain Warm-Start Caching

**Status**: ❌ **NO API ENDPOINTS**
**Implementation**: `riptide-intelligence/domain_profiling/profiler.rs`

### Missing Endpoints (P0 Priority):

1. **`GET /api/v1/profiles`** - List all domain profiles
2. **`GET /api/v1/profiles/:domain`** - Get specific profile with cache status
3. **`POST /api/v1/profiles`** - Create new domain profile
4. **`PUT /api/v1/profiles/:domain`** - Update profile configuration
5. **`DELETE /api/v1/profiles/:domain`** - Delete profile
6. **`POST /api/v1/profiles/:domain/cache`** - Cache engine preference
   - Body: `{ "engine": "wasm", "confidence": 0.85 }`
7. **`DELETE /api/v1/profiles/:domain/cache`** - Invalidate cache
8. **`GET /api/v1/profiles/:domain/baseline`** - Get site baseline
9. **`PUT /api/v1/profiles/:domain/baseline`** - Update baseline
10. **`POST /api/v1/profiles/import`** - Import profile from JSON
11. **`GET /api/v1/profiles/export/:domain`** - Export profile to JSON

### DomainProfile Features Available:
- ✅ Engine caching with 7-day TTL
- ✅ Confidence scoring (> 70% threshold)
- ✅ Baseline analysis integration
- ✅ Metadata tracking (success_rate, avg_response_time)
- ✅ Pattern matching (subdomain_regex, path_patterns, exclude_patterns)
- ✅ `EngineCacheable` trait implementation
- ✅ `ProfileManager` for CRUD operations

### Implementation Needed:
1. Create `handlers/profiles.rs`
2. Add routes in `routes/mod.rs`
3. Wire `ProfileManager` into `AppState`
4. Add integration tests

### Performance Impact:
- **Cache Hit**: Saves 10-50ms per request (skips content analysis)
- **Cost Savings**: 60-80% reduction on repeat domain visits
- **TTL**: 7-day expiration ensures freshness

---

## PHASE 10: Engine Selection Optimizations

**Status**: ⚠️ **INCOMPLETE** - Logic exists, no API exposure
**Implementation**: `riptide-reliability/src/engine_selection.rs`

### Missing Endpoints (P0 Priority):

1. **`POST /api/v1/engine/analyze`** - Analyze content for engine selection
   - Body: `{ "html": "...", "url": "...", "flags": {...} }`
   - Returns: `ContentAnalysis` with recommendations
2. **`POST /api/v1/engine/decide`** - Decide optimal engine
   - Body: `{ "html": "...", "url": "...", "profile": "example.com" }`
   - Returns: `{ "engine": "wasm", "reason": "..." }`
3. **`GET /api/v1/engine/stats`** - Engine usage statistics
   - Returns: Per-domain engine usage, success rates, costs
4. **`POST /api/v1/engine/probe`** - Test WASM extraction + auto-escalate
   - Body: `{ "url": "...", "quality_threshold": 50 }`
   - Returns: `{ "engine_used": "headless", "escalated": true, "quality": 45 }`

### Available Functions (Not Exposed):
- ✅ `decide_engine(html, url)` - Auto engine selection
- ✅ `decide_engine_with_flags(html, url, flags, profile)` - Phase 10 optimization
- ✅ `analyze_content(html, url)` - Detailed content analysis
- ✅ `should_escalate_to_headless(quality, words)` - Escalation logic
- ✅ `EngineSelectionFlags` - Feature toggles:
  - `probe_first_spa` - Try WASM before headless for SPAs
  - `use_visible_text_density` - Refined content analysis
  - `detect_placeholders` - Skeleton/loading state detection

### Probe-First Escalation (Phase 10):
- **Feature**: `probe_first_spa` flag
- **Behavior**: Returns `Engine::Wasm` for SPA-like pages
- **Caller Responsibility**: Attempt WASM extraction first, escalate if `quality_score < threshold`
- **Cost Savings**: 60-80% on SPAs with server-rendered content
- **Risk**: Minimal - automatic escalation ensures quality

### Implementation Needed:
1. Create `handlers/engine_selection.rs`
2. Add routes in `routes/mod.rs`
3. Optionally add `EngineAnalyzer` to `AppState` for tracking
4. Integration tests for probe-first flow

---

## Browser Pool Management

**Status**: ⚠️ **BASIC ENDPOINTS EXIST**, missing advanced features
**Implementation**: `riptide-browser/src/pool/mod.rs`

### Existing Endpoints ✅:
- `GET /api/v1/browser/pool/status` - Basic pool status
- `POST /api/v1/browser/session` - Create session
- `DELETE /api/v1/browser/session/:id` - Close session
- `POST /api/v1/browser/action` - Execute browser action

### Missing Advanced Endpoints (P1 Priority):

1. **`POST /api/v1/browser/pool/scale`** - Scale pool size
   - Body: `{ "target_size": 10 }`
2. **`POST /api/v1/browser/pool/drain`** - Graceful shutdown
   - Waits for active sessions to complete
3. **`POST /api/v1/browser/pool/health-check`** - Force health check
   - Manually trigger pool health validation
4. **`GET /api/v1/browser/pool/metrics`** - Detailed pool metrics
   - Returns: Connection reuse rate, avg latency, percentiles
5. **`POST /api/v1/browser/pool/warmup`** - Pre-warm connections
   - Body: `{ "count": 5 }`
6. **`GET /api/v1/browser/pool/events`** - Pool event stream (SSE)
   - Real-time pool events (BrowserCreated, BrowserRemoved, etc.)

### Available Features (riptide-browser):
- ✅ `BrowserPool` with connection pooling
- ✅ Health checking (automatic + manual trigger)
- ✅ Memory-based eviction
- ✅ Idle timeout handling
- ✅ Profile cleanup
- ✅ `PoolEvent` stream for monitoring
- ✅ Connection stats (reuse_rate, avg_latency, percentiles)

### Implementation Needed:
1. Extend `handlers/browser.rs` with new endpoints
2. Add SSE endpoint for pool events
3. Wire pool scaling/draining logic
4. Add metrics endpoint using `pool.stats()`

---

## Query-Aware Crawling

**Status**: ❌ **NOT IMPLEMENTED**
**Priority**: P2 (Future Enhancement)

### Missing Endpoints:

1. **`POST /api/v1/crawl/query-aware`** - Crawl with query context
   - Body: `{ "url": "...", "query": "latest news", "schema": {...} }`
2. **`POST /api/v1/crawl/targeted`** - Targeted extraction based on query
3. **`POST /api/v1/extract/llm-guided`** - LLM-guided extraction
   - Uses LLM to guide extraction based on user intent

### Note:
- Current `/crawl` endpoint is generic
- No query-aware logic in `handlers/crawl.rs`
- Intelligence layer exists but not exposed to API

### Implementation Needed:
1. Design query-aware extraction schema
2. Create `handlers/intelligence.rs`
3. Integrate LLM providers (optional)
4. Add targeted extraction logic

---

## Streaming Endpoints

**Status**: ✅ **FULLY IMPLEMENTED**

### Existing Streaming Endpoints:
- `POST /crawl/stream` - NDJSON streaming ✅
- `POST /crawl/sse` - Server-Sent Events ✅
- `GET /crawl/ws` - WebSocket streaming ✅
- `POST /deepsearch/stream` - NDJSON deepsearch ✅

### Available Features:
- ✅ Backpressure handling
- ✅ Progress tracking
- ✅ Report generation (HTML/JSON)
- ✅ Lifecycle management
- ✅ Error handling and recovery

**No action needed** - Streaming is complete.

---

## Performance & Monitoring

**Status**: ⚠️ **BASIC METRICS**, missing real-time profiling
**Priority**: P2

### Existing Endpoints ✅:
- `GET /api/profiling/memory` ✅
- `GET /api/profiling/cpu` ✅
- `GET /api/profiling/bottlenecks` ✅
- `GET /api/profiling/allocations` ✅
- `POST /api/profiling/leak-detection` ✅
- `POST /api/profiling/snapshot` ✅
- `GET /monitoring/health-score` ✅
- `GET /monitoring/performance-report` ✅

### Missing Advanced Features:

1. **Real-time Profiling Streams** (P2)
   - WebSocket stream for live memory/CPU data
2. **Comparative Benchmarks API** (P2)
   - Compare engine performance over time
3. **Phase Timing Breakdown** (P1)
   - Enhanced pipeline visualization (partially exists)
4. **Engine-Specific Metrics** (P1)
   - Track WASM vs Headless performance/cost

### Implementation Needed:
1. Add WebSocket profiling endpoint
2. Create benchmark comparison API
3. Enhance pipeline phase reporting
4. Add per-engine metrics tracking

---

## Handler Analysis

### Existing Handlers (Complete ✅):
- `handlers/crawl.rs` ✅
- `handlers/extract.rs` ✅
- `handlers/search.rs` ✅
- `handlers/browser.rs` ✅
- `handlers/profiling.rs` ✅
- `handlers/monitoring.rs` ✅
- `handlers/resources.rs` ✅
- `handlers/spider.rs` ✅
- `handlers/deepsearch.rs` ✅
- `handlers/workers.rs` ✅
- `handlers/sessions.rs` ✅
- `handlers/fetch.rs` ✅
- `handlers/telemetry.rs` ✅

### Missing Handlers:
1. **`handlers/profiles.rs`** - Domain profile management (P0)
2. **`handlers/engine_selection.rs`** - Engine decision APIs (P0)
3. **`handlers/intelligence.rs`** - Query-aware crawling (P2)

---

## AppState Analysis

### Has All Required Components ✅:
- `browser_launcher: Arc<HeadlessLauncher>` ✅
- `browser_facade: Arc<BrowserFacade>` ✅
- `spider: Option<Arc<Spider>>` ✅
- `streaming: Arc<StreamingModule>` ✅
- `performance_manager: Arc<PerformanceManager>` ✅
- `monitoring_system: Arc<MonitoringSystem>` ✅
- `fetch_engine: Arc<FetchEngine>` ✅
- `extraction_facade: Arc<ExtractionFacade>` ✅
- `scraper_facade: Arc<ScraperFacade>` ✅
- `spider_facade: Option<Arc<SpiderFacade>>` ✅
- `search_facade: Option<Arc<SearchFacade>>` ✅

### Missing Components (Optional):
- `domain_profile_manager: Option<Arc<ProfileManager>>` - For profile API
- `engine_analyzer: Option<Arc<EngineAnalyzer>>` - For usage tracking
- `intelligence_service: Option<Arc<IntelligenceService>>` - For query-aware crawling

---

## Cargo.toml Dependencies

### Has Essential Dependencies ✅:
- `riptide-intelligence` ✅
- `riptide-reliability` ✅
- `riptide-browser` ✅
- `riptide-performance` ✅
- `riptide-monitoring` ✅
- `riptide-facade` ✅
- All other workspace crates ✅

**No missing dependencies identified.**

---

## Testing Gaps

### Missing Test Coverage:

1. **Integration tests** for domain profile endpoints
2. **Engine selection** decision tests with real HTML samples
3. **Browser pool** scaling and draining tests
4. **Probe-first escalation** flow tests (WASM → Headless)
5. **Cache warm-start** performance benchmarks
6. **Engine statistics** tracking tests
7. **Query-aware crawling** integration tests

---

## Priority Recommendations

### P0 (Critical - Ship ASAP):

**1. Domain Profile Management API (Phase 10.4)**
- **Effort**: 1-2 days
- **Files**: `handlers/profiles.rs`, `routes/profiles.rs`
- **Actions**:
  - Create CRUD endpoints for profiles
  - Wire `ProfileManager` to `AppState`
  - Add cache management endpoints
  - Integration tests

**2. Engine Selection API (Phase 10)**
- **Effort**: 1 day
- **Files**: `handlers/engine_selection.rs`
- **Actions**:
  - Expose `decide_engine_with_flags()`
  - Add content analysis endpoint
  - Add probe-first escalation endpoint
  - Document probe-first workflow

### P1 (High - Next Sprint):

**3. Advanced Browser Pool Management**
- **Effort**: 1 day
- **Files**: Extend `handlers/browser.rs`
- **Actions**:
  - Add scale/drain/warmup endpoints
  - Add pool event SSE stream
  - Add detailed metrics endpoint

**4. Engine Statistics & Metrics**
- **Effort**: 1-2 days
- **Files**: New `EngineAnalyzer` service
- **Actions**:
  - Track engine usage per domain
  - Calculate success/failure rates
  - Cost analysis (WASM vs Headless)

### P2 (Medium - Future):

**5. Query-Aware Crawling**
- **Effort**: 3-5 days
- **Actions**:
  - Design schema-aware extraction
  - LLM-guided extraction
  - Targeted content extraction

**6. Real-time Profiling Streams**
- **Effort**: 2 days
- **Actions**:
  - WebSocket profiling data
  - Live bottleneck detection
  - Memory leak alerts

---

## Implementation Roadmap

### Sprint 1 (P0 - 3-4 days):
1. ✅ Day 1-2: Domain Profile API
   - Create `handlers/profiles.rs`
   - Add 11 profile endpoints
   - Wire `ProfileManager` to `AppState`
   - Integration tests
2. ✅ Day 3: Engine Selection API
   - Create `handlers/engine_selection.rs`
   - Add 4 engine endpoints
   - Document probe-first workflow
   - Unit tests

### Sprint 2 (P1 - 2-3 days):
3. ✅ Day 1: Advanced Browser Pool
   - Extend `handlers/browser.rs`
   - Add 6 advanced endpoints
   - SSE event stream
4. ✅ Day 2-3: Engine Metrics
   - Create `EngineAnalyzer` service
   - Track usage statistics
   - Add metrics endpoint

### Sprint 3 (P2 - 5-7 days):
5. Query-aware crawling
6. Real-time profiling streams

---

## Files to Create/Modify

### New Files:
1. `crates/riptide-api/src/handlers/profiles.rs` (P0)
2. `crates/riptide-api/src/handlers/engine_selection.rs` (P0)
3. `crates/riptide-api/src/routes/profiles.rs` (P0)
4. `crates/riptide-api/src/handlers/intelligence.rs` (P2)
5. `crates/riptide-api/src/services/engine_analyzer.rs` (P1)

### Files to Modify:
1. `crates/riptide-api/src/routes/mod.rs` - Add new route modules
2. `crates/riptide-api/src/handlers/mod.rs` - Export new handlers
3. `crates/riptide-api/src/state.rs` - Add optional services to `AppState`
4. `crates/riptide-api/src/handlers/browser.rs` - Extend with advanced endpoints
5. `crates/riptide-api/src/main.rs` - Wire new routes

---

## Estimated Effort

**Total P0 Work**: 3-4 days
**Total P1 Work**: 2-3 days
**Total P2 Work**: 5-7 days

**Grand Total**: 10-14 days for full implementation

---

## Conclusion

The riptide-api crate has **solid foundational features** but is missing critical Phase 10/10.4 optimizations:

1. **Domain warm-start caching** (Phase 10.4) has no API exposure
2. **Engine selection** logic exists but not accessible via API
3. **Browser pool** has basic management, needs advanced features
4. **Streaming** is fully complete ✅
5. **Monitoring** is mostly complete, needs real-time profiling

**Immediate action required**: Implement P0 features (Domain profiles + Engine selection) to unlock Phase 10.4 performance benefits.
