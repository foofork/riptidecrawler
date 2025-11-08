# API Layering Refactoring Roadmap

**Date:** 2025-11-07
**Status:** Ready to Execute
**Objective:** Enforce clean layering principles across API, Facade, and Infrastructure layers

---

## Executive Summary

### The Problem

The EventMesh API violates clean architecture principles:

1. **API Handlers contain business logic** (820+ lines that should be in facades)
2. **Facades contain transport concerns** (HTTP types, JSON serialization)
3. **Infrastructure logic scattered** (retry, circuit breakers not in reliability crate)

### The Goal

```
✅ TARGET ARCHITECTURE:
┌─────────────────────────────────────────────────────────┐
│ API Layer (HTTP/Transport)                              │
│ - 20-30 line handlers                                   │
│ - Request validation (format only)                      │
│ - DTO mapping                                           │
│ - HTTP status codes                                     │
└─────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────┐
│ Facade Layer (Workflow Orchestration)                   │
│ - Business workflows                                    │
│ - Domain types (NO JSON)                                │
│ - Multi-step orchestration                             │
│ - NO HTTP types/headers                                 │
└─────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────┐
│ Domain Layer (Business Logic)                           │
│ - Core business rules                                   │
│ - Domain entities                                       │
│ - Pure functions                                        │
└─────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────┐
│ Infrastructure Layer (Technical Concerns)               │
│ - HTTP clients (riptide-reliability)                    │
│ - Circuit breakers                                      │
│ - Retries, timeouts                                     │
│ - Database, cache, queues                               │
└─────────────────────────────────────────────────────────┘
```

### Success Metrics

| Metric | Current | Target | Improvement |
|--------|---------|--------|-------------|
| Average handler LOC | 145 | 25 | **83% reduction** |
| Business logic in handlers | 820 lines | 0 lines | **100% removal** |
| HTTP types in facades | 3 enums | 0 | **100% removal** |
| JSON serialization in facades | 37+ instances | 0 | **100% removal** |
| Test coverage (facades) | ~60% | 90%+ | **+30%** |

---

## Roadmap Overview

### 3-Phase Approach

**Phase 1: API Layer Refactoring** (2-3 weeks)
- Move business logic from handlers to facades
- Create thin handlers (20-30 lines)
- Fix circular dependencies

**Phase 2: Facade Layer Cleanup** (1-2 weeks)
- Remove transport concerns from facades
- Replace JSON with domain types
- Clean architecture boundaries

**Phase 3: Infrastructure Separation** (1 week)
- Move reliability logic to riptide-reliability
- Consolidate HTTP client code
- Extract circuit breakers and retry logic

**Total Estimated Duration:** 4-6 weeks

---

## Phase 1: API Layer Refactoring (Weeks 1-3)

**Goal:** Move 820 lines of business logic from handlers to facades

### Week 1: Foundation + Simple Handlers

#### Sprint 1.1: Fix Circular Dependency (2 days)

**Problem:** `riptide-api` ↔ `riptide-facade` circular dependency

**Tasks:**
- [ ] Refactor `AppState::new()` to split initialization
- [ ] Create `AppState::new_with_facades()` method
- [ ] Initialize facades AFTER base AppState construction
- [ ] Update `main.rs` to use new initialization pattern

**Files Changed:**
- `crates/riptide-api/src/state.rs`
- `crates/riptide-api/src/main.rs`

**Success Criteria:**
- ✅ `cargo build` passes without circular dependency errors
- ✅ All existing tests pass
- ✅ Facades accessible in handlers via `state.extraction_facade`

**Estimated Effort:** 4-6 hours

---

#### Sprint 1.2: Extract Handler Refactoring (2 days)

**Goal:** Reduce `extract.rs` from 80 lines to 25 lines

**Tasks:**
- [ ] Create `ExtractionFacade::extract_from_url()` method
- [ ] Move HTTP client logic to facade (lines 40-72)
- [ ] Move URL validation to facade
- [ ] Update handler to thin pattern
- [ ] Add facade unit tests
- [ ] Verify integration tests pass

**Before:**
```rust
// extract.rs - 80 lines with HTTP calls, parsing, error handling
pub async fn extract(
    State(state): State<AppState>,
    Json(payload): Json<ExtractRequest>,
) -> impl IntoResponse {
    // URL validation (5 lines)
    // HTTP client setup (10 lines)
    // Fetch HTML (15 lines)
    // Parse response (20 lines)
    // Error handling (15 lines)
    // Strategy selection (15 lines)
}
```

**After:**
```rust
// extract.rs - 25 lines, only HTTP concerns
pub async fn extract(
    State(state): State<AppState>,
    Json(payload): Json<ExtractRequest>,
) -> impl IntoResponse {
    // Validate URL format (HTTP concern)
    if let Err(e) = url::Url::parse(&payload.url) {
        return ApiError::invalid_url(&payload.url, e.to_string()).into_response();
    }

    // Call facade (all business logic)
    let result = state.extraction_facade
        .extract_from_url(&payload.url, payload.options)
        .await;

    // Map to HTTP response
    match result {
        Ok(doc) => Json(ExtractResponse { document: doc }).into_response(),
        Err(e) => ApiError::from(e).into_response(),
    }
}
```

**Files Changed:**
- `crates/riptide-facade/src/facades/extraction.rs` (NEW)
- `crates/riptide-api/src/handlers/extract.rs`

**Success Criteria:**
- ✅ Handler is 25 lines or less
- ✅ All tests pass: `cargo test -p riptide-api --test extract_tests`
- ✅ Metrics/tracing preserved
- ✅ Facade has 90%+ test coverage

**Estimated Effort:** 6-8 hours

---

#### Sprint 1.3: Search Handler Refactoring (1 day)

**Goal:** Reduce `search.rs` from 52 lines to 22 lines

**Tasks:**
- [ ] Create `SearchFacade::search()` method
- [ ] Move query validation to facade
- [ ] Move limit clamping logic to facade (business rule: 1-50)
- [ ] Update handler to thin pattern
- [ ] Add facade unit tests

**Files Changed:**
- `crates/riptide-facade/src/facades/search.rs` (UPDATE)
- `crates/riptide-api/src/handlers/search.rs`

**Success Criteria:**
- ✅ Handler is 22 lines or less
- ✅ All tests pass: `cargo test -p riptide-api --test search_tests`
- ✅ Provider fallback works

**Estimated Effort:** 4-5 hours

---

### Week 2: Medium Complexity Handlers

#### Sprint 2.1: Spider Handlers Refactoring (2 days)

**Goal:** Reduce 3 spider handlers from 87 lines total to 30 lines total

**Tasks:**
- [ ] Create `SpiderFacade::crawl()` method
- [ ] Create `SpiderFacade::get_status()` method
- [ ] Create `SpiderFacade::control()` method
- [ ] Move URL parsing to facade
- [ ] Move spider config building to facade
- [ ] Update all 3 handlers to thin pattern
- [ ] Add comprehensive facade tests

**Handlers:**
1. `spider_crawl` (68-87) → 10 lines
2. `spider_status` (90-98) → 10 lines
3. `spider_control` (101-109) → 10 lines

**Files Changed:**
- `crates/riptide-facade/src/facades/spider.rs` (UPDATE)
- `crates/riptide-api/src/handlers/spider.rs`

**Success Criteria:**
- ✅ Each handler is ~10 lines
- ✅ All tests pass: `cargo test -p riptide-api --test spider_tests`
- ✅ Session management works
- ✅ All 3 operations (crawl/status/control) functional

**Estimated Effort:** 8-10 hours

---

#### Sprint 2.2: Profile & Session Handlers (2 days)

**Goal:** Fix violations in `profiles.rs` and `sessions.rs`

**From `api-handler-violations-analysis.md`:**
- `profiles.rs`: Configuration orchestration, batch operations, cache warming
- `sessions.rs`: Session expiry filtering logic

**Tasks:**
- [ ] Create `ProfileFacade::create_with_config()` method
- [ ] Create `ProfileFacade::batch_create()` method
- [ ] Create `ProfileFacade::warm_cache()` method
- [ ] Move configuration merge logic to facade
- [ ] Update `SessionManager::list_sessions()` to handle filtering
- [ ] Refactor handlers to thin pattern

**Files Changed:**
- `crates/riptide-facade/src/facades/profile.rs` (NEW)
- `crates/riptide-api/src/handlers/profiles.rs`
- `crates/riptide-api/src/handlers/sessions.rs`

**Success Criteria:**
- ✅ No business logic in handlers
- ✅ All tests pass
- ✅ Profile batch operations work
- ✅ Session filtering delegated to manager

**Estimated Effort:** 8-10 hours

---

### Week 3: Most Complex Handlers

#### Sprint 3.1: PDF Handlers Refactoring (3 days)

**Goal:** Reduce 3 PDF handlers from 553 lines total to 85 lines total

**Complexity:** **HIGHEST** - Most business logic to move

**Tasks:**
- [ ] Create `PdfFacade::process_pdf()` method
- [ ] Create `PdfFacade::process_pdf_stream()` method
- [ ] Create `PdfFacade::process_multipart()` method
- [ ] Move base64 decoding to facade
- [ ] Move file validation to facade (size, magic bytes)
- [ ] Move resource acquisition to facade
- [ ] Move multipart parsing state machine to facade
- [ ] Move progress stream creation to facade
- [ ] Extract `create_enhanced_progress_stream` (120 lines) to facade
- [ ] Update all 3 handlers to thin pattern
- [ ] Add comprehensive tests

**Handlers:**
1. `process_pdf` (75-155) → 30 lines
2. `process_pdf_stream` (161-234) → 25 lines
3. `upload_pdf` (386-549) → 30 lines

**Files Changed:**
- `crates/riptide-facade/src/facades/pdf.rs` (NEW)
- `crates/riptide-api/src/handlers/pdf.rs`

**Success Criteria:**
- ✅ `process_pdf`: 30 lines
- ✅ `process_pdf_stream`: 25 lines
- ✅ `upload_pdf`: 30 lines
- ✅ All tests pass: `cargo test -p riptide-api --test pdf_tests`
- ✅ Streaming still works
- ✅ Multipart upload works
- ✅ Resource acquisition works

**Estimated Effort:** 12-16 hours

---

#### Sprint 3.2: Crawl Handler Refactoring (2 days)

**Goal:** Reduce crawl handlers from 395 lines total to 60 lines total

**Tasks:**
- [ ] Create `CrawlFacade::crawl_batch()` method
- [ ] Create `CrawlFacade::crawl_spider_mode()` method
- [ ] Move pipeline selection logic to facade (enhanced vs standard)
- [ ] Move result transformation to facade (40+ lines)
- [ ] Move statistics calculation to facade
- [ ] Move chunking application to facade
- [ ] Consider: Move event emission to facade (or keep in handler?)
- [ ] Update both handlers to thin pattern
- [ ] Add comprehensive tests

**Handlers:**
1. `crawl` (43-286) → 35 lines
2. `handle_spider_crawl` (290-397) → 25 lines

**Files Changed:**
- `crates/riptide-facade/src/facades/crawl.rs` (NEW)
- `crates/riptide-api/src/handlers/crawl.rs`

**Success Criteria:**
- ✅ `crawl`: 35 lines
- ✅ `crawl_spider_mode`: 25 lines
- ✅ All tests pass: `cargo test -p riptide-api --test crawl_tests`
- ✅ Enhanced pipeline works
- ✅ Spider mode works
- ✅ Statistics accurate

**Estimated Effort:** 8-12 hours

---

#### Sprint 3.3: Table & Render Handlers (2 days)

**Goal:** Fix complex violations in `tables.rs` and `render/strategies.rs`

**From `api-handler-violations-analysis.md`:**
- `tables.rs`: Data type detection algorithm, type inference heuristics, storage management
- `render/strategies.rs`: URL content analysis, adaptive configuration builder

**Tasks:**
- [ ] Create `TableAnalyzer` service (type detection, sampling)
- [ ] Create `TableCacheService` (replace global HashMap)
- [ ] Create `TableFacade::store_and_summarize()` method
- [ ] Create `ContentAnalyzer` service (URL analysis, SPA detection)
- [ ] Create `RenderStrategySelector` service (adaptive config)
- [ ] Move all 120+ lines of heuristics to services
- [ ] Update handlers to thin pattern

**Files Changed:**
- `crates/riptide-intelligence/src/table_analyzer.rs` (NEW)
- `crates/riptide-intelligence/src/content_analyzer.rs` (NEW)
- `crates/riptide-facade/src/facades/table.rs` (NEW)
- `crates/riptide-facade/src/facades/render_strategy.rs` (NEW)
- `crates/riptide-api/src/handlers/tables.rs`
- `crates/riptide-api/src/handlers/render/strategies.rs`

**Success Criteria:**
- ✅ No business logic in handlers
- ✅ Type detection in `riptide-intelligence`
- ✅ Storage in dedicated service
- ✅ All tests pass

**Estimated Effort:** 10-14 hours

---

### Phase 1 Deliverables

**At end of Week 3:**

| Handler | Before | After | Reduction |
|---------|--------|-------|-----------|
| extract.rs | 80 | 25 | 69% |
| search.rs | 52 | 22 | 58% |
| spider.rs | 87 | 30 | 66% |
| pdf.rs | 553 | 85 | 85% |
| crawl.rs | 395 | 60 | 85% |
| profiles.rs | ~100 | ~30 | 70% |
| sessions.rs | ~80 | ~25 | 69% |
| tables.rs | ~150 | ~40 | 73% |
| render/strategies.rs | ~120 | ~30 | 75% |
| **TOTAL** | **~1,617** | **~347** | **79%** |

**Validation:**
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo build --release
```

---

## Phase 2: Facade Layer Cleanup (Week 4-5)

**Goal:** Remove transport concerns from facade layer

### Week 4: Remove HTTP Types & Headers

#### Sprint 4.1: Remove HttpMethod Enum (1 day)

**From `facade-layer-violations-analysis.md`:**
- `HttpMethod` enum in `facades/pipeline.rs` (lines 441-445)
- Used in `FetchOptions` struct
- Violates separation of concerns

**Tasks:**
- [ ] Create domain `FetchOperation` enum in facade
  ```rust
  pub enum FetchOperation {
      Retrieve,  // Maps to GET
      Submit,    // Maps to POST
      Update,    // Maps to PUT
      Remove,    // Maps to DELETE
  }
  ```
- [ ] Move `HttpMethod` to `crates/riptide-api/src/types.rs`
- [ ] Add conversion traits in handlers
  ```rust
  impl From<FetchOperation> for HttpMethod {
      fn from(op: FetchOperation) -> Self { ... }
  }
  ```
- [ ] Update all `FetchOptions` usage
- [ ] Update tests

**Files Changed:**
- `crates/riptide-facade/src/facades/pipeline.rs`
- `crates/riptide-api/src/types.rs`
- `crates/riptide-api/src/handlers/*.rs` (conversion logic)

**Success Criteria:**
- ✅ No HTTP types in facade public APIs
- ✅ `rg "HttpMethod" crates/riptide-facade/` returns 0 results
- ✅ All tests pass

**Estimated Effort:** 4-6 hours

---

#### Sprint 4.2: Remove HTTP Headers from Facade (2 days)

**From `facade-layer-violations-analysis.md`:**
- `headers: Vec<(String, String)>` in `FetchOptions` (line 427)
- Transport-level headers in facade API

**Tasks:**
- [ ] Replace `Vec<(String, String)>` with `HashMap<String, String>`
- [ ] Rename field to `metadata` (not "headers")
- [ ] Update `config.rs` documentation
  ```rust
  // Before: "Additional headers to include in requests"
  // After: "Additional metadata for operations"
  ```
- [ ] Update handlers to convert metadata → HTTP headers
- [ ] Update all usages across codebase

**Files Changed:**
- `crates/riptide-facade/src/facades/pipeline.rs`
- `crates/riptide-facade/src/config.rs`
- `crates/riptide-api/src/handlers/*.rs` (conversion logic)

**Success Criteria:**
- ✅ No `Vec<(String, String)>` headers in facade
- ✅ `rg "headers.*Vec" crates/riptide-facade/` returns 0 results
- ✅ All tests pass

**Estimated Effort:** 6-8 hours

---

### Week 5: Replace JSON with Domain Types

#### Sprint 5.1: Pipeline Facade JSON Removal (3 days)

**From `facade-layer-violations-analysis.md`:**
- 37+ instances of `serde_json::Value` in facade layer
- Pipeline stages return JSON instead of domain types
- `PipelineContext` stores `serde_json::Value`

**Tasks:**
- [ ] Create domain types for pipeline outputs
  ```rust
  pub enum PipelineOutput {
      FetchResult(FetchData),
      ExtractResult(ExtractedData),
      TransformResult(TransformedData),
      ValidateResult(ValidationResult),
      StoreResult(StorageConfirmation),
  }
  ```
- [ ] Replace `serde_json::Value` in `PipelineResult`
- [ ] Replace `serde_json::Value` in `StageResult`
- [ ] Replace `serde_json::Value` in `PipelineContext`
- [ ] Update all pipeline stages (fetch, extract, transform, validate, store)
- [ ] Add serialization traits in handler layer
  ```rust
  impl Serialize for PipelineOutput { ... }
  ```
- [ ] Update tests

**Files Changed:**
- `crates/riptide-facade/src/facades/pipeline.rs`
- `crates/riptide-types/src/pipeline.rs` (NEW domain types)
- `crates/riptide-api/src/handlers/*.rs` (serialization)

**Success Criteria:**
- ✅ No `serde_json::Value` in pipeline public APIs
- ✅ Strong typing with compile-time guarantees
- ✅ All tests pass

**Estimated Effort:** 12-16 hours

---

#### Sprint 5.2: Browser Facade JSON Removal (2 days)

**From `facade-layer-violations-analysis.md`:**
- `execute_script()` returns `serde_json::Value` (line 451)
- `get_local_storage()` returns `serde_json::Value` (line 821)
- JSON parsing in facade layer

**Tasks:**
- [ ] Create domain types
  ```rust
  pub struct ScriptResult {
      pub value: String,
      pub value_type: ScriptValueType,
  }

  pub enum ScriptValueType {
      String(String),
      Number(f64),
      Boolean(bool),
      Object(HashMap<String, String>),
      Array(Vec<String>),
  }

  pub struct LocalStorage {
      pub entries: HashMap<String, String>,
  }
  ```
- [ ] Update `execute_script()` to return `ScriptResult`
- [ ] Update `get_local_storage()` to return `LocalStorage`
- [ ] Add serialization traits in handler layer
- [ ] Update tests

**Files Changed:**
- `crates/riptide-facade/src/facades/browser.rs`
- `crates/riptide-types/src/browser.rs` (NEW domain types)
- `crates/riptide-api/src/handlers/*.rs` (serialization)

**Success Criteria:**
- ✅ No `serde_json::Value` in browser facade public APIs
- ✅ All tests pass

**Estimated Effort:** 8-10 hours

---

#### Sprint 5.3: Extractor Facade JSON Removal (1 day)

**From `facade-layer-violations-analysis.md`:**
- `extract_schema()` returns `serde_json::Value` (line 500)
- Builds JSON in facade layer (lines 500-539)

**Tasks:**
- [ ] Create domain types
  ```rust
  pub struct SchemaExtractionResult {
      pub fields: HashMap<String, FieldValue>,
      pub missing_required: Vec<String>,
  }

  pub enum FieldValue {
      Text(String),
      Number(f64),
      Url(String),
      Date(String),
      Missing,
  }
  ```
- [ ] Update `extract_schema()` to return `SchemaExtractionResult`
- [ ] Remove JSON building logic
- [ ] Add serialization traits in handler layer
- [ ] Update tests

**Files Changed:**
- `crates/riptide-facade/src/facades/extractor.rs`
- `crates/riptide-types/src/extractor.rs` (NEW domain types)
- `crates/riptide-api/src/handlers/*.rs` (serialization)

**Success Criteria:**
- ✅ No `serde_json::Value` in extractor facade
- ✅ All tests pass

**Estimated Effort:** 4-6 hours

---

### Phase 2 Deliverables

**At end of Week 5:**

**Validation Commands:**
```bash
# Should return ZERO matches
rg "serde_json::Value" crates/riptide-facade/src/facades/
rg "HttpMethod|Response|Request" crates/riptide-facade/src/facades/
rg "StatusCode|Headers" crates/riptide-facade/src/facades/

# All tests should pass
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

**Success Criteria:**
- ✅ Zero HTTP types in facade public APIs
- ✅ Zero `serde_json::Value` in facade return types
- ✅ Zero transport headers in facade structures
- ✅ All serialization happens in handler layer
- ✅ Domain types used throughout facade

---

## Phase 3: Infrastructure Separation (Week 6)

**Goal:** Consolidate infrastructure concerns in `riptide-reliability` crate

### Sprint 6.1: Move HTTP Client Logic (2 days)

**Current State:**
- HTTP client calls scattered across handlers (now in facades)
- Retry logic in multiple places
- Circuit breaker usage inconsistent

**Tasks:**
- [ ] Audit all HTTP client usage in facades
- [ ] Create `HttpClientService` in `riptide-reliability`
  ```rust
  pub struct HttpClientService {
      client: reqwest::Client,
      circuit_breaker: Arc<CircuitBreaker>,
      retry_policy: RetryPolicy,
  }

  impl HttpClientService {
      pub async fn get(&self, url: &str, options: FetchOptions) -> Result<Response> {
          // With retry + circuit breaker
      }

      pub async fn post(&self, url: &str, body: Vec<u8>, options: FetchOptions) -> Result<Response> {
          // With retry + circuit breaker
      }
  }
  ```
- [ ] Move retry logic from facades to `HttpClientService`
- [ ] Move circuit breaker integration from facades
- [ ] Update facades to use `HttpClientService`
- [ ] Add comprehensive tests

**Files Changed:**
- `crates/riptide-reliability/src/http_client.rs` (NEW)
- `crates/riptide-facade/src/facades/*.rs` (use service)
- `crates/riptide-api/src/state.rs` (initialize service)

**Success Criteria:**
- ✅ Single source of truth for HTTP operations
- ✅ Consistent retry + circuit breaker usage
- ✅ All tests pass

**Estimated Effort:** 8-10 hours

---

### Sprint 6.2: Resource Management Consolidation (1 day)

**Current State:**
- `ResourceManager` in `riptide-api/src/resource.rs`
- Resource acquisition duplicated in handlers

**Tasks:**
- [ ] Move `ResourceManager` to `riptide-reliability`
- [ ] Create `ResourceGuard` RAII type
- [ ] Consolidate all resource acquisition patterns
- [ ] Update facades to use reliability crate
- [ ] Add tests

**Files Changed:**
- `crates/riptide-reliability/src/resource_manager.rs` (MOVE)
- `crates/riptide-facade/src/facades/*.rs` (use service)

**Success Criteria:**
- ✅ Resource management in reliability crate
- ✅ RAII pattern enforced
- ✅ All tests pass

**Estimated Effort:** 4-6 hours

---

### Sprint 6.3: Circuit Breaker Patterns (1 day)

**Tasks:**
- [ ] Audit circuit breaker usage across codebase
- [ ] Ensure all external calls use circuit breakers
- [ ] Create circuit breaker presets
  ```rust
  pub mod presets {
      pub fn http_external() -> CircuitBreakerConfig { ... }
      pub fn database() -> CircuitBreakerConfig { ... }
      pub fn cache() -> CircuitBreakerConfig { ... }
  }
  ```
- [ ] Document usage patterns
- [ ] Add integration tests

**Files Changed:**
- `crates/riptide-reliability/src/circuit.rs`
- Documentation

**Success Criteria:**
- ✅ Circuit breaker presets available
- ✅ Usage documented
- ✅ All external calls protected

**Estimated Effort:** 4-6 hours

---

### Sprint 6.4: Final Architecture Validation (1 day)

**Tasks:**
- [ ] Run architecture lints
  ```bash
  # Custom lint rules
  ./scripts/validate_architecture.sh
  ```
- [ ] Generate dependency graphs
- [ ] Document final architecture
- [ ] Create architecture decision records (ADRs)
- [ ] Update developer documentation

**Deliverables:**
- Architecture validation script
- Dependency graph visualization
- Updated docs in `docs/architecture/`
- ADRs for layering decisions

**Success Criteria:**
- ✅ No circular dependencies
- ✅ Clean layer boundaries
- ✅ Documentation updated

**Estimated Effort:** 4-6 hours

---

## Risk Management

### High-Risk Areas

#### 1. Circular Dependency Fix (Sprint 1.1)
**Risk:** Breaking existing facade initialization
**Mitigation:**
- Incremental changes with tests after each step
- Keep old initialization path temporarily (feature flag)
- Rollback plan documented

#### 2. PDF Handler Refactoring (Sprint 3.1)
**Risk:** Breaking streaming, multipart upload, or resource acquisition
**Mitigation:**
- Comprehensive integration tests
- Manual testing of streaming endpoints
- Gradual migration (feature flag for new implementation)

#### 3. JSON to Domain Type Conversion (Sprints 5.1-5.3)
**Risk:** Breaking existing APIs, serialization issues
**Mitigation:**
- Maintain backward compatibility with serialization traits
- Test API contracts thoroughly
- Consider API versioning if needed

### Medium-Risk Areas

#### 4. Handler LOC Targets
**Risk:** Handlers might not fit 20-30 line target
**Mitigation:**
- Target is guideline, not hard rule
- Focus on "no business logic" over LOC count
- Some handlers may be 35-40 lines (acceptable)

#### 5. Test Coverage
**Risk:** Tests may break during refactoring
**Mitigation:**
- Run tests after each sprint
- Fix tests incrementally
- Maintain >90% coverage on facades

---

## Testing Strategy

### Per-Sprint Testing

After each sprint:
```bash
# Unit tests for affected crates
cargo test -p riptide-api
cargo test -p riptide-facade
cargo test -p riptide-reliability

# Integration tests
cargo test --test '*'

# Clippy (zero warnings)
cargo clippy --workspace -- -D warnings

# Check build
cargo build --release
```

### Phase Validation Testing

After each phase:
```bash
# Full test suite
cargo test --workspace

# Architecture validation
./scripts/validate_architecture.sh

# Performance benchmarks
cargo bench

# E2E tests (if available)
./scripts/e2e_tests.sh
```

### Regression Testing

Critical paths to test after each change:
1. **Extract endpoint**: `POST /extract`
2. **Crawl endpoint**: `POST /crawl`
3. **PDF processing**: `POST /pdf/process`
4. **Search endpoint**: `GET /search`
5. **Spider endpoints**: `POST /spider/crawl`, `GET /spider/status`

---

## Rollback Strategy

### Sprint-Level Rollback

If a sprint fails:
```bash
# Revert specific sprint commits
git log --oneline --grep="Sprint X.Y"
git revert <commit-sha>

# Re-run tests
cargo test --workspace
```

### Phase-Level Rollback

If entire phase needs rollback:
```bash
# Tag before starting phase
git tag phase-N-start

# If rollback needed
git revert phase-N-start..HEAD

# Or hard reset (destructive)
git reset --hard phase-N-start
```

### Feature Flag Fallback

For high-risk changes, use feature flags:
```rust
#[cfg(feature = "new-layering")]
pub async fn extract_new(/* thin handler */) -> impl IntoResponse { ... }

#[cfg(not(feature = "new-layering"))]
pub async fn extract(/* old handler */) -> impl IntoResponse { ... }
```

Enable gradually:
```bash
# Test new implementation
cargo test --features new-layering

# Deploy to staging
cargo build --release --features new-layering

# Production rollout
# Update Cargo.toml: default = ["new-layering"]
```

---

## Success Metrics & KPIs

### Code Quality Metrics

| Metric | Baseline | Target | Measurement |
|--------|----------|--------|-------------|
| **Handler LOC** | 1,617 | 347 | `tokei crates/riptide-api/src/handlers/` |
| **Business logic in handlers** | 820 lines | 0 | Manual audit |
| **HTTP types in facades** | 3 | 0 | `rg "HttpMethod\|StatusCode" crates/riptide-facade/` |
| **JSON in facades** | 37+ | 0 | `rg "serde_json::Value" crates/riptide-facade/` |
| **Cyclomatic complexity (avg)** | ~15 | <8 | `cargo clippy` |
| **Test coverage** | 60% | 90% | `cargo tarpaulin` |

### Architecture Metrics

| Metric | Baseline | Target | Measurement |
|--------|----------|--------|-------------|
| **Circular dependencies** | 1 | 0 | `cargo depgraph` |
| **Layer violations** | 52+ | 0 | `./scripts/validate_architecture.sh` |
| **Facade test coverage** | ~60% | 90%+ | `cargo tarpaulin -p riptide-facade` |

### Performance Metrics

Ensure refactoring doesn't degrade performance:

| Metric | Baseline | Target |
|--------|----------|--------|
| **Extract endpoint latency** | p50: 150ms | ±5% |
| **Crawl endpoint throughput** | 100 req/s | ±5% |
| **PDF processing time** | 2s/doc | ±10% |

Monitor with:
```bash
cargo bench --bench api_benchmarks
```

---

## Dependencies & Prerequisites

### Tools Required

```bash
# Rust toolchain
rustc 1.75+
cargo 1.75+

# Analysis tools
cargo install tokei          # LOC counting
cargo install cargo-tarpaulin # Code coverage
cargo install cargo-depgraph # Dependency graphs
cargo install cargo-modules  # Module structure

# Testing
cargo install cargo-nextest  # Faster test runner
```

### Documentation

Before starting:
- [ ] Read `THIN-HANDLER-DESIGN.md`
- [ ] Read `api-handler-violations-analysis.md`
- [ ] Read `facade-layer-violations-analysis.md`
- [ ] Understand current architecture

---

## Communication & Tracking

### Daily Standups

During active development:
- What sprint am I on?
- What's blocking me?
- What tests are failing?

### Sprint Reviews

After each sprint:
- Demo changes
- Review test results
- Update metrics dashboard
- Document any deviations from plan

### Phase Retrospectives

After each phase:
- What went well?
- What could improve?
- Update estimates for next phase
- Adjust plan if needed

---

## Appendix A: File Organization

### New Files Created

```
crates/riptide-facade/src/facades/
├── extraction.rs           (NEW - Sprint 1.2)
├── pdf.rs                  (NEW - Sprint 3.1)
├── crawl.rs                (NEW - Sprint 3.2)
├── profile.rs              (NEW - Sprint 2.2)
├── table.rs                (NEW - Sprint 3.3)
└── render_strategy.rs      (NEW - Sprint 3.3)

crates/riptide-intelligence/src/
├── table_analyzer.rs       (NEW - Sprint 3.3)
├── content_analyzer.rs     (NEW - Sprint 3.3)
└── render_strategy.rs      (NEW - Sprint 3.3)

crates/riptide-types/src/
├── pipeline.rs             (NEW - Sprint 5.1)
├── browser.rs              (NEW - Sprint 5.2)
└── extractor.rs            (NEW - Sprint 5.3)

crates/riptide-reliability/src/
├── http_client.rs          (NEW - Sprint 6.1)
└── resource_manager.rs     (MOVE - Sprint 6.2)
```

### Modified Files

```
crates/riptide-api/src/
├── main.rs                 (Sprint 1.1)
├── state.rs                (Sprint 1.1)
└── handlers/
    ├── extract.rs          (Sprint 1.2)
    ├── search.rs           (Sprint 1.3)
    ├── spider.rs           (Sprint 2.1)
    ├── profiles.rs         (Sprint 2.2)
    ├── sessions.rs         (Sprint 2.2)
    ├── pdf.rs              (Sprint 3.1)
    ├── crawl.rs            (Sprint 3.2)
    ├── tables.rs           (Sprint 3.3)
    └── render/strategies.rs (Sprint 3.3)

crates/riptide-facade/src/
├── facades/
│   ├── pipeline.rs         (Sprints 4.1, 4.2, 5.1)
│   ├── browser.rs          (Sprint 5.2)
│   ├── extractor.rs        (Sprint 5.3)
│   ├── search.rs           (Sprint 1.3)
│   └── spider.rs           (Sprint 2.1)
└── config.rs               (Sprint 4.2)
```

---

## Appendix B: Quick Reference Commands

### Validation Commands

```bash
# Check handler sizes
tokei crates/riptide-api/src/handlers/ --sort lines

# Check for HTTP types in facades
rg "HttpMethod|StatusCode|HeaderMap" crates/riptide-facade/src/

# Check for JSON in facades
rg "serde_json::Value" crates/riptide-facade/src/

# Check for circular dependencies
cargo depgraph | grep cycle

# Run architecture validation
./scripts/validate_architecture.sh

# Test coverage
cargo tarpaulin --workspace --out Html

# Benchmarks
cargo bench --workspace
```

### Development Commands

```bash
# Build specific phase changes
cargo build -p riptide-api -p riptide-facade

# Test specific crate
cargo nextest run -p riptide-api

# Watch mode for development
cargo watch -x 'test -p riptide-api'

# Clippy with zero warnings
RUSTFLAGS="-D warnings" cargo clippy --workspace

# Format code
cargo fmt --all
```

---

## Appendix C: Architecture Decision Records

### ADR-001: Thin Handler Pattern

**Status:** Accepted
**Date:** 2025-11-07

**Context:**
Handlers contain 820+ lines of business logic, violating separation of concerns.

**Decision:**
Enforce "thin handler" pattern: handlers are 20-30 lines, only HTTP concerns.

**Consequences:**
- **Positive:** Testability, maintainability, clear boundaries
- **Negative:** More files, learning curve for new developers

---

### ADR-002: Domain Types Over JSON

**Status:** Accepted
**Date:** 2025-11-07

**Context:**
Facades use `serde_json::Value` extensively, losing type safety.

**Decision:**
Facades return domain types, handlers serialize to JSON.

**Consequences:**
- **Positive:** Type safety, compile-time checks, transport-agnostic
- **Negative:** More boilerplate serialization code

---

### ADR-003: Infrastructure in riptide-reliability

**Status:** Accepted
**Date:** 2025-11-07

**Context:**
HTTP client, circuit breaker, retry logic scattered across codebase.

**Decision:**
Consolidate all infrastructure concerns in `riptide-reliability` crate.

**Consequences:**
- **Positive:** Single source of truth, reusability, testability
- **Negative:** Initial migration effort

---

## Summary

This roadmap provides a **6-week plan** to enforce clean layering across the EventMesh API:

**Week 1:** Foundation + simple handlers (extract, search)
**Week 2:** Medium complexity (spider, profiles, sessions)
**Week 3:** Complex handlers (PDF, crawl, tables, render)
**Week 4:** Remove HTTP types from facades
**Week 5:** Replace JSON with domain types
**Week 6:** Consolidate infrastructure

**Expected Outcomes:**
- 79% reduction in handler complexity
- 100% removal of transport concerns from facades
- 90%+ test coverage on facades
- Clean, maintainable architecture

**Next Step:** Begin Sprint 1.1 - Fix circular dependency

---

**End of Roadmap**
