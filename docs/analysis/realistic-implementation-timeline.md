# 📅 RipTide v1.0 Realistic Implementation Timeline
**16-Week Production-Ready Delivery Plan**

**Created:** 2025-11-04
**Author:** Strategic Planning Agent
**Status:** 🎯 Ready for Execution
**Methodology:** TDD London School, Consolidation over Creation

---

## 🎯 Executive Summary

### The Reality Check

After analyzing the UX vision, technical roadmap, and ground truth findings, here's what we discovered:

**Good News:**
- ✅ 1,598 lines of production-ready orchestration code exists (hidden in `riptide-api`)
- ✅ 461 test files with 85% London School TDD readiness
- ✅ Comprehensive validation & error infrastructure already built
- ✅ 40% of API handlers already use facade pattern correctly

**The Gap:**
- 🔴 UX wants `crawl4ai`-level simplicity: `result = riptide.extract(url)`
- 🟡 Current facade has placeholder implementations (returns mock data)
- 🟡 ~2,580 lines of duplicated utilities (Redis, HTTP, retry)
- 🟡 60% of handlers bypass facade, instantiate orchestrators directly
- 🟡 No error codes, strategy-specific errors, or proper error context

### The Strategy: Bridge the Gap Pragmatically

**What we WON'T do:**
- ❌ Build new `run_pipeline()` from scratch (wrap existing `PipelineOrchestrator`)
- ❌ Create new validation framework (enhance existing middleware)
- ❌ Recreate utilities (consolidate ~2,580 lines into `riptide-utils`)

**What we WILL do:**
- ✅ Expose existing production orchestrators via simple facade API
- ✅ Create dead-simple extraction API: `client.extract(url)`
- ✅ Consolidate duplicated code before writing new code
- ✅ Refactor 60% of handlers to use facade pattern
- ✅ Add progressive enhancement (schema, streaming, pipelines)

### Timeline Breakdown

| Phase | Duration | Focus | Deliverable |
|-------|----------|-------|-------------|
| **Phase 0: Foundation** | Weeks 0-2 | Critical blockers, consolidation | Utils crate, error types, config |
| **Phase 1: Modularity** | Weeks 2-7 | Facade wrappers, handler refactoring | Unified architecture |
| **Phase 2: UX Layer** | Weeks 7-11 | Simple API, progressive enhancement | Developer-friendly API |
| **Phase 3: Advanced** | Weeks 11-14 | Streaming, pipelines, schemas | Power user features |
| **Phase 4: Polish** | Weeks 14-16 | Testing, docs, deployment | v1.0 release |

### Success Criteria for v1.0

**Must-Have (Production Blocker):**
- [x] Dead-simple API: `client.extract(url)` works ✨
- [x] Schema-driven extraction: `client.extract(url, schema="events")`
- [x] 80%+ test coverage per crate
- [x] All P0/P1 clippy warnings resolved
- [x] No code duplication (consolidated utilities)
- [x] 100% facade usage (no handler bypasses)
- [x] Error codes & structured errors

**Nice-to-Have (v1.1 Deferrable):**
- [ ] Full pipeline API: `client.pipeline(search="...", schema="...")`
- [ ] Streaming support: `for result in client.stream(...)`
- [ ] Auto schema detection (>80% accuracy)
- [ ] Multi-tenancy infrastructure
- [ ] GraphQL API layer

---

## 📊 Gap Analysis: UX Vision vs. Current Reality

### UX Vision (from riptide-v1-ux-design.md)

**Level 1: Dead Simple (80% of users)**
```python
# As simple as crawl4ai
result = client.extract("https://example.com")
print(result.content)
```

**Level 2: Schema-Aware (15% of users)**
```python
events = client.extract(
    "https://eventsite.com",
    schema="events",
    output_format="icalendar"
)
```

**Level 3: Automated Pipeline (5% of users)**
```python
pipeline = client.pipeline(
    search="tech events Amsterdam",
    schema="events",
    output_format="google_calendar"
)
```

### Current Reality (from ground truth findings)

**Facade Layer:**
```rust
// Current PipelineFacade - PLACEHOLDER ONLY
async fn execute_fetch(...) -> RiptideResult<serde_json::Value> {
    // ⚠️ RETURNS MOCK DATA!
    Ok(serde_json::json!({
        "url": url,
        "content": format!("Fetched content from {}", url),
    }))
}
```

**Production Code (Not Exposed):**
```rust
// Hidden in riptide-api/src/pipeline.rs (1,072 lines)
pub struct PipelineOrchestrator {
    // ✅ Complete fetch→gate→extract workflow
    // ✅ Event-driven provenance tracking
    // ✅ Redis caching, retry logic, PDF handling
    // ❌ NOT ACCESSIBLE VIA FACADE
}
```

**Handler Confusion (60% bypass facade):**
```rust
// ❌ WRONG: Direct orchestrator instantiation
let pipeline = PipelineOrchestrator::new(state.clone());
let results = pipeline.execute(url).await?;

// ✅ CORRECT: Facade pattern (only 40% do this)
let result = state.extraction_facade.extract(url).await?;
```

### The Bridge: What We Need to Build

**Week 2-3: Create Facade Wrappers**
```rust
// NEW: Expose orchestrator via simple facade
pub struct SimpleFacade {
    orchestrator: Arc<PipelineOrchestrator>,
}

impl SimpleFacade {
    pub async fn extract(&self, url: &str) -> RiptideResult<ExtractedDoc> {
        // Delegate to existing production code
        self.orchestrator.execute_single(url).await
            .map(|result| result.document)
            .map_err(Into::into)
    }
}
```

**Week 7-9: Add Schema Support**
```rust
// Progressive enhancement
impl SimpleFacade {
    pub async fn extract_with_schema(
        &self,
        url: &str,
        schema: Schema
    ) -> RiptideResult<StructuredData> {
        // Use strategies orchestrator for schema-driven extraction
        self.strategies_orchestrator
            .execute_with_schema(url, schema)
            .await
    }
}
```

**Week 11-13: Add Pipeline (Optional for v1.0)**
```rust
// Advanced feature (may defer to v1.1)
impl SimpleFacade {
    pub async fn pipeline(
        &self,
        search: &str,
        schema: Schema
    ) -> RiptideResult<impl Stream<Item = StructuredData>> {
        // Search → Discover → Extract pipeline
    }
}
```

---

## 📅 Phase-by-Phase Timeline

## Phase 0: Critical Foundation (Weeks 0-2) 🔥 BLOCKER

**Goal:** Eliminate technical debt blockers before building new features

### Week 0: Utility Consolidation

**Priority:** P0 - BLOCKS ALL OTHER WORK

#### W0.1: Create `riptide-utils` Crate (5 days)

**Rationale:** ~2,580 lines of duplicated utilities cause:
- No connection pooling (3 different Redis connection implementations)
- Inconsistent error handling across crates
- 40+ different retry logic implementations
- 8+ identical HTTP client setups in tests

**TDD Approach:**
```rust
// 1. Write tests FIRST (RED)
#[tokio::test]
async fn test_redis_pool_reuses_connections() {
    let pool = RedisPool::new("redis://localhost").await.unwrap();
    let conn1 = pool.get().await.unwrap();
    let conn2 = pool.get().await.unwrap();
    // Both should be from same pool
    assert!(Arc::ptr_eq(&conn1.inner(), &conn2.inner()));
}

// 2. Implement (GREEN)
pub struct RedisPool {
    manager: Arc<ConnectionManager>,
}

// 3. Refactor (REFACTOR)
```

**Deliverables:**
- `/crates/riptide-utils/Cargo.toml`
- `/crates/riptide-utils/src/lib.rs`
- `/crates/riptide-utils/src/{redis,http,retry,error,time}.rs`
- `/crates/riptide-utils/tests/*.rs`

**Migration:**
| Utility | Files Affected | Lines Removed | Lines Added |
|---------|----------------|---------------|-------------|
| Redis pooling | 3 crates | -200 | +50 |
| HTTP clients | 8+ test files | -320 | +40 |
| Retry logic | 40+ implementations | -1,600 | +100 |
| Time utilities | 50+ files | -400 | +60 |
| **TOTAL** | **~100 files** | **-2,520** | **+250** |

**Acceptance Criteria:**
- [ ] All 3 Redis-using crates migrated to `RedisPool`
- [ ] All test files use `riptide_utils::http::create_default_client()`
- [ ] Single canonical retry implementation replaces 40+ duplicates
- [ ] All existing tests pass (461 baseline + new utils tests)
- [ ] No duplicate code remains

**Risks & Mitigation:**
- **Risk:** Breaking changes across multiple crates
- **Mitigation:** Move code, don't rewrite; test each crate individually

---

#### W0.2: Create `StrategyError` Type (3 days)

**Rationale:** All extraction failures currently become generic 500s with no context:
```rust
// ❌ CURRENT: All become "Internal server error"
ApiError::ExtractionError { message: "CSS selector failed" }

// ✅ DESIRED: Structured error with recovery hints
StrategyError::CssSelectorFailed {
    selector: ".article",
    alternatives: vec![".post", ".content"],
    url: "https://example.com",
}
```

**Impact:** 92 manual error conversions in handlers eliminated.

**TDD Approach:**
```rust
// Write test FIRST (RED)
#[test]
fn test_css_error_converts_to_api_error_422() {
    let error = StrategyError::CssSelectorFailed {
        selector: "div.event".to_string(),
        reason: "Element not found".to_string(),
        url: "https://example.com".to_string(),
    };

    let api_error: ApiError = error.into();
    assert_eq!(api_error.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(api_error.to_string().contains("div.event"));
}
```

**Deliverables:**
- `/crates/riptide-types/src/strategy_error.rs` (15+ variants)
- `/crates/riptide-types/tests/strategy_error_tests.rs`
- Updated all extraction crates to return `StrategyError`

**Variants:**
```rust
pub enum StrategyError {
    // CSS-specific
    CssSelectorFailed { selector: String, reason: String, url: String },

    // LLM-specific
    LlmCircuitBreakerOpen { provider: String, retry_after: Duration, failures: u32 },
    LlmRateLimited { provider: String, reset_at: DateTime<Utc> },

    // WASM-specific
    WasmCompilationFailed { module: String, error: String },
    WasmMemoryExhausted { used_mb: u64, limit_mb: u64 },

    // Quality-based
    LowConfidence { strategy: String, confidence: f32, threshold: f32 },

    // ... 10+ more variants
}
```

**Acceptance Criteria:**
- [ ] 15+ specific variants covering all extraction strategies
- [ ] Auto-conversion to `ApiError` with correct HTTP status codes
- [ ] All 92 manual `map_err()` calls in handlers removed
- [ ] Tests for all error conversions

---

#### W0.3: Fix Dual `ApiConfig` Naming (2 days)

**Problem:** Two different `ApiConfig` types cause import conflicts:
- `riptide-config::ApiConfig` (auth, rate limiting)
- `riptide-api::config::ApiConfig` (resources, performance)

**Solution:** Rename `riptide-api`'s config to `ResourceConfig`

**Migration:**
```rust
// OLD (remove):
use riptide_api::config::ApiConfig;
let config = ApiConfig::from_env()?;

// NEW:
use riptide_api::config::ResourceConfig;
let config = ResourceConfig::from_env()?;
```

**Deliverables:**
- Updated `/crates/riptide-api/src/config.rs`
- Updated ~15 files in `riptide-api` crate
- Updated ~5 test files

**Acceptance Criteria:**
- [ ] No naming conflicts between config types
- [ ] All imports updated via compiler-guided refactoring
- [ ] All tests pass

---

### Week 1: Configuration & Testing Baseline

#### W1.1: Add `server.yaml` Support (3 days)

**Rationale:** 69+ environment variables are hard to manage. Add file-based config with env var overrides.

**Precedence:** `server.yaml` defaults + env var overrides + validation

**Implementation:**
```rust
// crates/riptide-config/src/file_loader.rs
pub struct FileConfigLoader;

impl FileConfigLoader {
    pub fn load_yaml<T: DeserializeOwned>(path: &str) -> Result<T> {
        let contents = fs::read_to_string(path)?;
        let substituted = substitute_env_vars(&contents)?; // ${VAR:default}
        serde_yaml::from_str(&substituted).map_err(Into::into)
    }
}

// crates/riptide-config/src/precedence.rs
pub struct ConfigResolver;

impl ConfigResolver {
    pub fn load_with_precedence(file_path: &str) -> Result<ResolvedConfig> {
        let file_config = FileConfigLoader::load_yaml(file_path)?;
        let env_overrides = EnvConfigLoader::new().load_partial()?;

        let mut config = file_config;
        config.merge(env_overrides); // Env wins
        config.validate()?;

        Ok(ResolvedConfig(config))
    }
}
```

**TDD Approach:**
```rust
#[test]
fn test_env_overrides_file_config() {
    // GIVEN: server.yaml has port: 8080
    let yaml = r#"server: { port: 8080 }"#;
    fs::write("test.yaml", yaml).unwrap();

    // AND: ENV sets PORT=9000
    std::env::set_var("PORT", "9000");

    // WHEN: Load config
    let config = ConfigResolver::load_with_precedence("test.yaml").unwrap();

    // THEN: ENV wins
    assert_eq!(config.server.port, 9000);
}
```

**Deliverables:**
- `/server.yaml` (with all 69 current env vars as defaults)
- `/crates/riptide-config/src/file_loader.rs`
- `/crates/riptide-config/src/precedence.rs`
- Tests for precedence logic

**Acceptance Criteria:**
- [ ] YAML loading with `${VAR:default}` substitution works
- [ ] Env vars override file values
- [ ] All existing env vars still work (backward compatible)
- [ ] Validation runs on final merged config

---

#### W1.2: Capture CI Baseline & Create TDD Guide (2 days)

**Baseline Metrics:**
```bash
cargo build --workspace --locked 2>&1 | tee docs/baseline-build.log
cargo test --workspace 2>&1 | tee docs/baseline-tests.log
cargo clippy --workspace 2>&1 | tee docs/baseline-clippy.log
./scripts/benchmark-baseline.sh > docs/perf-baseline.json
```

**TDD London School Guide:**
```markdown
# docs/testing/TDD-LONDON-GUIDE.md

## 1. Write the Test First (RED)

Focus on **behavior verification** not state inspection.

```rust
#[tokio::test]
async fn should_extract_events_using_css_strategy() {
    // GIVEN: Mock dependencies (London School!)
    let mut mock_fetcher = MockFetcher::new();
    mock_fetcher.expect_fetch()
        .with(eq("https://example.com"))
        .times(1)
        .returning(|_| Ok(HTML_EVENT_FIXTURE.to_string()));

    // WHEN: Exercise facade
    let facade = ExtractionFacade::new(mock_fetcher);
    let result = facade.extract(...).await;

    // THEN: Verify behavior
    assert!(result.is_ok());
    // Mock expectations verified automatically on drop
}
```

## 2. Implement Minimum Code (GREEN)
## 3. Refactor (REFACTOR)
```

**Deliverables:**
- Baseline logs (build, test, clippy, perf)
- `/docs/testing/TDD-LONDON-GUIDE.md`
- Example contract tests

**Acceptance Criteria:**
- [ ] All 461 existing tests pass
- [ ] Baseline metrics documented
- [ ] TDD guide with examples published

---

### Phase 0 Success Criteria

**Definition of Done:**
- [x] `riptide-utils` created, ~2,580 lines consolidated
- [x] `StrategyError` eliminates 92 manual conversions
- [x] Dual `ApiConfig` resolved → `ResourceConfig`
- [x] `server.yaml` support with env var precedence
- [x] TDD London School guide published
- [x] CI baseline documented
- [x] All 461 existing tests pass

---

## Phase 1: Facade Layer Unification (Weeks 2-7) 🏗️ CORE

**Goal:** Expose production orchestrators via simple facade API, refactor all handlers to use facade pattern

### Week 2-3: Wrap Existing Orchestrators (DON'T BUILD FROM SCRATCH!)

#### W2.1: Create `OrchestrationFacade` Wrapper (7 days)

**Critical Insight:** Don't build `run_pipeline()` from scratch - **wrap existing production code!**

**What Exists (DO NOT RECREATE):**
1. **`PipelineOrchestrator`** - 1,072 lines of production code
   - Complete fetch→gate→extract workflow
   - Event-driven provenance tracking
   - Redis caching with TTL
   - PDF resource management
   - Metrics collection

2. **`StrategiesPipelineOrchestrator`** - 526 lines
   - WASM, CSS, Regex, LLM strategies
   - Auto-detection based on content
   - Performance metrics

**Action: Create Thin Wrapper**
```rust
// crates/riptide-facade/src/facades/orchestration.rs (NEW)
use riptide_api::pipeline::PipelineOrchestrator;
use riptide_api::strategies_pipeline::StrategiesPipelineOrchestrator;

pub struct OrchestrationFacade {
    pipeline_orchestrator: Arc<PipelineOrchestrator>,
    strategies_orchestrator: Arc<StrategiesPipelineOrchestrator>,
}

impl OrchestrationFacade {
    pub async fn run_pipeline(
        &self,
        inputs: PipelineInputs,
        options: PipelineOptions,
    ) -> RiptideResult<impl Stream<Item = ResultItem>> {
        match inputs.mode {
            PipelineMode::Standard => {
                // Delegate to existing orchestrator
                self.pipeline_orchestrator.execute(inputs, options).await
            }
            PipelineMode::Advanced => {
                // Delegate to strategies orchestrator
                self.strategies_orchestrator.execute(inputs, options).await
            }
        }
    }
}
```

**TDD Approach (London School):**
```rust
// Write test FIRST (RED)
#[tokio::test]
async fn test_facade_delegates_to_pipeline_orchestrator() {
    // GIVEN: Mock orchestrator
    let mut mock_pipeline = MockPipelineOrchestrator::new();
    mock_pipeline.expect_execute()
        .times(1)
        .returning(|_, _| Ok(stream::iter(vec![ResultItem::success()])));

    // WHEN: Call facade
    let facade = OrchestrationFacade::new(Arc::new(mock_pipeline), ...);
    let result = facade.run_pipeline(...).await;

    // THEN: Verify delegation (mock expectations auto-verified)
    assert!(result.is_ok());
}
```

**Impact:**
- ✅ Save 4-6 weeks of development time
- ✅ Leverage 1,598 lines of battle-tested code
- ✅ Immediate production readiness
- ✅ Maintain existing provenance & metrics

**Deliverables:**
- `/crates/riptide-facade/src/facades/orchestration.rs`
- `/crates/riptide-facade/tests/orchestration_facade_tests.rs`

**Acceptance Criteria:**
- [ ] Facade wraps both orchestrators (no duplication)
- [ ] `run_pipeline()` delegates to existing implementations
- [ ] Streaming support works
- [ ] Mock tests verify delegation (not reimplementation)
- [ ] Provenance tracking preserved
- [ ] All existing orchestrator tests still pass

---

### Week 3-4: Create 5 Missing Facades

**Pattern:** Wrap existing services, don't rebuild

#### W3.1: `PdfFacade` (2 days)
```rust
pub struct PdfFacade {
    processor: Arc<PdfProcessor>, // Existing riptide-pdf
}

impl PdfFacade {
    pub async fn extract_text(&self, pdf_data: &[u8]) -> RiptideResult<Vec<String>> {
        self.processor.extract_text(pdf_data).await.map_err(Into::into)
    }
}
```

#### W3.2: `LlmFacade` (2 days)
```rust
pub struct LlmFacade {
    intelligence: Arc<IntelligenceEngine>, // Existing riptide-intelligence
}

impl LlmFacade {
    pub async fn extract_with_llm(&self, content: &str) -> RiptideResult<ExtractedData> {
        self.intelligence.extract(content).await.map_err(Into::into)
    }
}
```

#### W3.3: `StealthFacade` (1 day)
```rust
pub struct StealthFacade {
    stealth_engine: Arc<StealthEngine>, // Existing riptide-stealth
}
```

#### W3.4: `ProfileFacade` (1 day)
```rust
pub struct ProfileFacade {
    profile_manager: Arc<ProfileManager>,
}
```

#### W3.5: `StreamingFacade` (2 days)
```rust
pub struct StreamingFacade {
    ndjson_streamer: Arc<NdjsonStreamer>, // Existing riptide-streaming
}

impl StreamingFacade {
    pub async fn stream_results(
        &self,
        urls: Vec<String>
    ) -> impl Stream<Item = RiptideResult<ExtractedDoc>> {
        self.ndjson_streamer.stream_urls(urls).await
    }
}
```

**Deliverables (per facade):**
- `/crates/riptide-facade/src/facades/{pdf,llm,stealth,profile,streaming}.rs`
- Tests for each facade

**Acceptance Criteria (per facade):**
- [ ] Wraps existing service (no business logic duplication)
- [ ] Clean error conversion (service errors → RiptideError)
- [ ] Mock tests verify delegation
- [ ] Builder pattern for configuration

---

### Week 5-7: Refactor 54 Handlers to Use Facades

**Problem:** 60% of handlers bypass facade and instantiate orchestrators directly.

**Affected Handlers:**
- Crawl handlers (8 endpoints) → use `OrchestrationFacade`
- DeepSearch handlers (4 endpoints) → use `OrchestrationFacade`
- Streaming handlers (6 endpoints) → use `StreamingFacade`
- PDF handlers → use `PdfFacade`
- LLM handlers → use `LlmFacade`
- Stealth handlers → use `StealthFacade`

**Refactoring Pattern:**
```rust
// BEFORE (handlers/crawl.rs)
pub async fn crawl(
    State(state): State<AppState>,
    Json(req): Json<CrawlRequest>,
) -> Result<Json<CrawlResponse>> {
    // ❌ Direct orchestrator instantiation
    let pipeline = PipelineOrchestrator::new(state.clone());
    let results = pipeline.run_enhanced().await?;
    Ok(Json(results.into()))
}

// AFTER (handlers/crawl.rs)
pub async fn crawl(
    State(state): State<AppState>,
    Json(req): Json<CrawlRequest>,
) -> Result<Json<CrawlResponse>> {
    // ✅ Use facade
    let result = state.orchestration_facade
        .run_pipeline(req.into(), req.options.into())
        .await?;
    Ok(Json(result.into()))
}
```

**TDD Approach:**
```rust
// Update handler test (RED - modify existing test)
#[tokio::test]
async fn test_crawl_handler_uses_facade() {
    // GIVEN: Mock facade (NOT orchestrator)
    let mut mock_facade = MockOrchestrationFacade::new();
    mock_facade.expect_run_pipeline()
        .times(1)
        .returning(|_, _| Ok(stream::iter(vec![ResultItem::success()])));

    let app_state = AppState {
        orchestration_facade: Arc::new(mock_facade),
        ...
    };

    // WHEN: Call handler
    let result = crawl(State(app_state), Json(req)).await;

    // THEN: Verify
    assert!(result.is_ok());
}
```

**Migration Order (by risk):**

| Week | Risk Level | Handlers | Count |
|------|-----------|----------|-------|
| Week 5 | Low | PDF, Profile handlers | 6 |
| Week 6 | Medium | Crawl, Streaming handlers | 20 |
| Week 7 | High | DeepSearch, LLM handlers | 28 |

**Acceptance Criteria:**
- [ ] All 54 handlers refactored to use facades
- [ ] No direct orchestrator instantiation in handlers
- [ ] ~1,200 lines of duplicated code removed
- [ ] All handler tests updated to mock facades
- [ ] Consistent error handling via facade layer
- [ ] Golden tests pass (regression prevention)

---

### Phase 1 Success Criteria

**Definition of Done:**
- [x] `OrchestrationFacade` wraps existing orchestrators (not recreated)
- [x] 5 missing facades created (PDF, LLM, Stealth, Profile, Streaming)
- [x] 54 handlers refactored to use facades
- [x] ~1,200 lines of handler duplication removed
- [x] 100% facade usage across all handlers
- [x] All tests pass (London School mocks)

**Code Metrics:**
| Category | Lines Removed | Lines Added | Net Change |
|----------|---------------|-------------|------------|
| Orchestrator wrapper | 0 | +300 | +300 |
| 5 new facades | 0 | +600 | +600 |
| Handler refactoring | -1,200 | +300 | -900 |
| **TOTAL** | **-1,200** | **+1,200** | **0** |

**Impact:** Unified architecture, no net code increase, 100% facade usage

---

## Phase 2: UX Layer - Simple Extraction API (Weeks 7-11) ✨ USER-FACING

**Goal:** Create the dead-simple API that developers love

### Week 7-8: Level 1 - Dead Simple Extraction

**UX Vision:**
```python
# As simple as crawl4ai
from riptide import RipTide

client = RipTide()
result = client.extract("https://example.com")
print(result.content)
```

**Implementation:**
```rust
// crates/riptide-facade/src/simple.rs (NEW)
pub struct RipTide {
    orchestration: Arc<OrchestrationFacade>,
    config: RiptideConfig,
}

impl RipTide {
    pub fn new() -> RiptideResult<Self> {
        let config = RiptideConfig::from_env()?;
        let orchestration = OrchestrationFacade::new(config.clone()).await?;
        Ok(Self { orchestration: Arc::new(orchestration), config })
    }

    pub async fn extract(&self, url: &str) -> RiptideResult<ExtractedDoc> {
        // Delegate to orchestration facade
        self.orchestration
            .run_pipeline(
                PipelineInputs::from_url(url),
                PipelineOptions::default()
            )
            .await
            .map(|stream| stream.next().await.unwrap()) // Take first result
            .map(|result| result.document)
    }
}
```

**Builder Pattern:**
```rust
pub struct RipTideBuilder {
    config: RiptideConfig,
}

impl RipTideBuilder {
    pub fn new() -> Self {
        Self { config: RiptideConfig::default() }
    }

    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.config.user_agent = ua.into();
        self
    }

    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.config.timeout = Duration::from_secs(secs);
        self
    }

    pub async fn build(self) -> RiptideResult<RipTide> {
        RipTide::from_config(self.config).await
    }
}

// Entry point
impl RipTide {
    pub fn builder() -> RipTideBuilder {
        RipTideBuilder::new()
    }
}
```

**Python SDK (via PyO3):**
```python
# crates/riptide-py/src/lib.rs
use pyo3::prelude::*;

#[pyclass]
struct RipTide {
    inner: Arc<riptide_facade::RipTide>,
}

#[pymethods]
impl RipTide {
    #[new]
    fn new() -> PyResult<Self> {
        let rt = tokio::runtime::Runtime::new()?;
        let inner = rt.block_on(riptide_facade::RipTide::new())?;
        Ok(Self { inner: Arc::new(inner) })
    }

    fn extract(&self, url: &str) -> PyResult<PyObject> {
        // Async runtime handling
        let rt = tokio::runtime::Runtime::new()?;
        let result = rt.block_on(self.inner.extract(url))?;

        // Convert to Python dict
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            dict.set_item("content", result.content)?;
            dict.set_item("title", result.title)?;
            Ok(dict.into())
        })
    }
}
```

**TDD Approach:**
```rust
#[tokio::test]
async fn test_simple_extract_returns_content() {
    // GIVEN: Mock orchestration facade
    let mut mock_facade = MockOrchestrationFacade::new();
    mock_facade.expect_run_pipeline()
        .times(1)
        .returning(|_, _| Ok(stream::iter(vec![
            ResultItem {
                document: ExtractedDoc {
                    content: "Hello world".to_string(),
                    ...
                }
            }
        ])));

    // WHEN: Call simple API
    let client = RipTide::with_facade(Arc::new(mock_facade));
    let result = client.extract("https://example.com").await;

    // THEN: Content extracted
    assert_eq!(result.unwrap().content, "Hello world");
}
```

**Deliverables:**
- `/crates/riptide-facade/src/simple.rs` (simple API)
- `/crates/riptide-py/` (Python SDK via PyO3)
- `/crates/riptide-facade/examples/simple_extract.rs`
- Documentation: "Getting Started in 5 Minutes"

**Acceptance Criteria:**
- [ ] Single-line extraction works: `client.extract(url)`
- [ ] Python SDK installable via `pip install riptide`
- [ ] Builder pattern for configuration
- [ ] Response < 500ms p95 (simple extraction)
- [ ] Examples demonstrate crawl4ai-level simplicity

---

### Week 9-10: Level 2 - Schema-Aware Extraction

**UX Vision:**
```python
# Extract with specific schema
events = client.extract(
    "https://eventsite.com",
    schema="events",
    output_format="icalendar"
)

# Or extract jobs
jobs = client.extract(
    "https://careers.example.com",
    schema="jobs",
    filters={"location": "remote"}
)
```

**Implementation:**
```rust
// Add to RipTide facade
impl RipTide {
    pub async fn extract_with_schema(
        &self,
        url: &str,
        schema: impl Into<Schema>,
        options: ExtractionOptions,
    ) -> RiptideResult<StructuredData> {
        let schema = schema.into();

        // Use strategies orchestrator for schema-driven extraction
        self.strategies_orchestration
            .execute_with_schema(url, schema, options)
            .await
            .map(|result| result.structured_data)
    }
}

// Schema registry
pub enum Schema {
    Events,
    Jobs,
    Products,
    Articles,
    Custom(CustomSchema),
}

impl From<&str> for Schema {
    fn from(s: &str) -> Self {
        match s {
            "events" => Schema::Events,
            "jobs" => Schema::Jobs,
            "products" => Schema::Products,
            "articles" => Schema::Articles,
            _ => Schema::Custom(CustomSchema::from_str(s).unwrap()),
        }
    }
}
```

**Schema Definitions:**
```rust
// crates/riptide-types/src/schemas/events.rs
pub struct EventsSchema {
    pub version: String, // "v1"
}

impl EventsSchema {
    pub fn fields(&self) -> Vec<SchemaField> {
        vec![
            SchemaField::required("title", FieldType::String),
            SchemaField::required("date", FieldType::DateTime),
            SchemaField::optional("location", FieldType::String),
            SchemaField::optional("url", FieldType::Url),
        ]
    }

    pub fn output_formats(&self) -> Vec<OutputFormat> {
        vec![
            OutputFormat::ICalendar,
            OutputFormat::Json,
            OutputFormat::Csv,
            OutputFormat::GoogleCalendar,
        ]
    }
}
```

**Output Format Converters:**
```rust
// crates/riptide-types/src/output/icalendar.rs
impl StructuredData {
    pub fn to_icalendar(&self) -> RiptideResult<String> {
        match &self.schema {
            Schema::Events => {
                let mut calendar = ICalendar::new();
                for event in &self.records {
                    calendar.add_event(Event {
                        summary: event["title"].as_str()?,
                        dtstart: event["date"].as_datetime()?,
                        location: event.get("location").map(|v| v.as_str()).transpose()?,
                        url: event.get("url").map(|v| v.as_str()).transpose()?,
                    });
                }
                Ok(calendar.to_string())
            }
            _ => Err(RiptideError::unsupported_format("icalendar", &self.schema)),
        }
    }
}
```

**TDD Approach:**
```rust
#[tokio::test]
async fn test_extract_events_to_icalendar() {
    // GIVEN: Mock strategies orchestrator
    let mut mock_strategies = MockStrategiesOrchestrator::new();
    mock_strategies.expect_execute_with_schema()
        .with(eq("https://eventsite.com"), eq(Schema::Events), always())
        .times(1)
        .returning(|_, _, _| Ok(StructuredData {
            schema: Schema::Events,
            records: vec![
                json!({
                    "title": "Tech Conference",
                    "date": "2025-12-01T10:00:00Z",
                    "location": "Amsterdam",
                })
            ],
        }));

    // WHEN: Extract with schema
    let client = RipTide::with_strategies(Arc::new(mock_strategies));
    let result = client.extract_with_schema(
        "https://eventsite.com",
        "events",
        ExtractionOptions { output_format: Some("icalendar".into()), ..Default::default() }
    ).await;

    // THEN: iCalendar format
    let icalendar = result.unwrap().to_icalendar().unwrap();
    assert!(icalendar.contains("VEVENT"));
    assert!(icalendar.contains("SUMMARY:Tech Conference"));
}
```

**Deliverables:**
- Schema definitions for events, jobs, products, articles
- Output format converters (iCalendar, CSV, JSON)
- Schema auto-detection (optional)
- Examples for each schema type

**Acceptance Criteria:**
- [ ] 4+ built-in schemas (events, jobs, products, articles)
- [ ] Custom schema support via JSON definition
- [ ] Output format conversion (iCalendar, CSV, JSON, Google Calendar)
- [ ] Schema validation on extraction results
- [ ] Response < 1500ms p95 (schema extraction)

---

### Week 11: Error Codes & Enhanced Error Responses

**Goal:** Add error codes and structured error responses

**Implementation:**
```rust
// crates/riptide-types/src/error_codes.rs (NEW)
pub struct ErrorCode {
    pub code: u32,
    pub category: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub documentation_url: &'static str,
}

pub mod codes {
    // 1xxx: Validation
    pub const INVALID_URL: ErrorCode = ErrorCode {
        code: 1001,
        category: "VALIDATION",
        name: "INVALID_URL",
        description: "The provided URL is malformed",
        documentation_url: "https://docs.riptide.io/errors/1001",
    };

    // 2xxx: Extraction
    pub const CSS_SELECTOR_NOT_FOUND: ErrorCode = ErrorCode {
        code: 2001,
        category: "EXTRACTION",
        name: "CSS_SELECTOR_NOT_FOUND",
        description: "CSS selector did not match any elements",
        documentation_url: "https://docs.riptide.io/errors/2001",
    };

    // ... 50+ error codes
}

impl ApiError {
    pub fn error_code(&self) -> ErrorCode {
        match self {
            ApiError::InvalidUrl { .. } => codes::INVALID_URL,
            // ... map all variants to codes
        }
    }
}
```

**Enhanced JSON Response:**
```json
{
  "error": {
    "code": 2001,
    "type": "extraction_error",
    "category": "EXTRACTION",
    "name": "CSS_SELECTOR_NOT_FOUND",
    "message": "CSS selector '.article' not found on page",
    "retryable": true,
    "retry_after_ms": 5000,
    "documentation_url": "https://docs.riptide.io/errors/2001",
    "context": {
      "selector": ".article",
      "url": "https://example.com",
      "alternatives": [".post", ".content"]
    },
    "status": 422
  }
}
```

**Deliverables:**
- `/crates/riptide-types/src/error_codes.rs` (50+ codes)
- Updated `ApiError` with error codes
- Error documentation generator
- Updated all error responses

**Acceptance Criteria:**
- [ ] 50+ error codes defined and documented
- [ ] All API responses include error codes
- [ ] Error docs auto-generated from code
- [ ] Structured context in error responses

---

### Phase 2 Success Criteria

**Definition of Done:**
- [x] Dead-simple API works: `client.extract(url)`
- [x] Schema-driven extraction: `client.extract(url, schema="events")`
- [x] Python SDK available via pip
- [x] 4+ built-in schemas (events, jobs, products, articles)
- [x] Output format conversion (iCalendar, CSV, JSON)
- [x] Error codes system (50+ codes)
- [x] Performance: simple extraction <500ms p95, schema <1500ms p95
- [x] Examples & documentation for all features

---

## Phase 3: Advanced Features (Weeks 11-14) 🚀 POWER USERS

**Goal:** Add streaming, pipelines, and advanced extraction features

### Week 11-12: Streaming Support

**UX Vision:**
```python
# Stream results as they come
urls = ["https://site1.com", "https://site2.com", ...]
for result in client.stream(urls, schema="products"):
    print(f"Found: {result.data['name']}")

# Or async iterator
async for result in client.stream_async(urls):
    await process(result)
```

**Implementation:**
```rust
// Add to RipTide facade
impl RipTide {
    pub async fn stream(
        &self,
        urls: Vec<String>,
        options: StreamOptions,
    ) -> impl Stream<Item = RiptideResult<ExtractedDoc>> {
        self.streaming_facade
            .stream_results(urls, options)
            .await
    }
}

// Streaming facade (already created in Phase 1)
impl StreamingFacade {
    pub async fn stream_results(
        &self,
        urls: Vec<String>,
        options: StreamOptions,
    ) -> impl Stream<Item = RiptideResult<ExtractedDoc>> {
        // Delegate to existing NdjsonStreamer
        self.ndjson_streamer
            .stream_urls(urls)
            .await
            .map(|result| result.map(|item| item.document))
    }
}
```

**Backpressure Control:**
```rust
pub struct StreamOptions {
    pub max_in_flight: usize,  // Concurrent requests
    pub buffer_size: usize,     // Result buffer
    pub on_progress: Option<Box<dyn Fn(usize, usize) + Send + Sync>>,
    pub on_error: Option<Box<dyn Fn(&RiptideError) + Send + Sync>>,
}
```

**Deliverables:**
- Streaming API in facade
- Backpressure control
- Progress callbacks
- Python async iterator support

**Acceptance Criteria:**
- [ ] Stream 100+ URLs efficiently
- [ ] Backpressure prevents memory exhaustion
- [ ] Progress tracking works
- [ ] Error handling per URL (don't fail entire stream)

---

### Week 13-14: Pipeline API (OPTIONAL - May Defer to v1.1)

**UX Vision:**
```python
# Full automated pipeline: Search → Discover → Extract
pipeline = client.pipeline(
    search="tech events Amsterdam December",
    schema="events",
    output_format="icalendar"
)

# Stream results
for event in pipeline.stream():
    print(f"Found: {event.data['title']}")
```

**Decision Point:** This may be deferred to v1.1 if time is tight. Core extraction API is higher priority.

**If Implemented:**
```rust
impl RipTide {
    pub async fn pipeline(
        &self,
        search: &str,
        schema: Schema,
        options: PipelineOptions,
    ) -> RiptideResult<PipelineExecutor> {
        // 1. Search for URLs
        let urls = self.search_facade.search(search).await?;

        // 2. Discover additional URLs
        let discovered = self.spider_facade.discover(urls).await?;

        // 3. Stream extraction
        let stream = self.streaming_facade
            .stream_with_schema(discovered, schema, options)
            .await?;

        Ok(PipelineExecutor { stream })
    }
}
```

**Deliverables (if time permits):**
- Pipeline orchestration
- Search integration
- Discovery/crawling integration
- Deduplication logic

**Acceptance Criteria:**
- [ ] Search → Extract pipeline works end-to-end
- [ ] Deduplication prevents duplicate extractions
- [ ] Streaming results as they're discovered
- [ ] Pipeline completion < 60s for 10 sources

---

### Phase 3 Success Criteria

**Definition of Done:**
- [x] Streaming API works for 100+ URLs
- [x] Backpressure control prevents memory issues
- [x] Progress tracking and error callbacks
- [ ] Pipeline API (if time permits, otherwise defer to v1.1)

---

## Phase 4: Polish & Release (Weeks 14-16) 📚 FINALIZE

**Goal:** Production-ready v1.0 release with comprehensive documentation and testing

### Week 14: Testing & Quality

#### Golden Tests (Regression Prevention)

**Create 20+ golden tests:**
```rust
// tests/golden/extraction_golden_tests.rs
use riptide_test_utils::golden::{GoldenTest, GoldenAssertion};

#[tokio::test]
async fn test_event_extraction_ics_golden() {
    GoldenTest::new("event_extraction_ics")
        .with_fixture("fixtures/events.html")
        .with_golden("goldens/events.json")
        .normalize(|result| {
            // Remove non-deterministic fields
            result.remove_field("timestamp");
            result.remove_field("snapshot_key");
            result
        })
        .assert_matches()
        .await;
}
```

**Coverage:**
- Extract (ICS, JSON, CSS, LLM, WASM) - 10 tests
- Schema extraction (events, jobs, products, articles) - 8 tests
- Streaming (NDJSON, progress) - 2 tests

**Deliverables:**
- 20+ golden tests
- Fixtures committed (HTML, PDF, ICS)
- Golden responses normalized
- CI runs golden tests on every PR

**Acceptance Criteria:**
- [ ] 20+ golden tests created
- [ ] All golden tests pass
- [ ] Coverage ≥ 80% per crate
- [ ] No P0/P1 clippy warnings

---

### Week 15: Documentation & Examples

**Documentation Requirements:**

1. **Getting Started (5 minutes)**
```markdown
# Quick Start

## Installation
```bash
pip install riptide
```

## First Extraction
```python
from riptide import RipTide

client = RipTide()
result = client.extract("https://example.com")
print(result.content)
```

That's it! 🎉
```

2. **API Reference**
- Complete method documentation
- Parameter descriptions
- Return value schemas
- Error codes reference

3. **Examples Gallery**
- Extract events → Google Calendar
- Job board → Email alerts
- Product prices → Monitoring
- News articles → RSS feed

4. **Migration Guides**
- From crawl4ai
- From firecrawl
- From BeautifulSoup

**Deliverables:**
- `/docs/GETTING-STARTED.md`
- `/docs/API-REFERENCE.md`
- `/docs/EXAMPLES.md`
- `/docs/MIGRATION.md`
- `/docs/ERROR-CODES.md`

**Acceptance Criteria:**
- [ ] Getting started guide < 5 minutes
- [ ] 10+ example scripts
- [ ] API reference complete
- [ ] Migration guides for 2+ tools

---

### Week 16: Deployment & Release

**Release Checklist:**

1. **Version Tagging**
```bash
git tag v1.0.0
git push origin v1.0.0
```

2. **Build Artifacts**
- Linux x86_64 binary
- macOS ARM64 binary
- Windows x86_64 binary
- Python wheel (PyPI)

3. **Testing**
- [ ] All 461+ baseline tests pass
- [ ] All 20+ golden tests pass
- [ ] Coverage ≥ 80% per crate
- [ ] Performance benchmarks meet SLOs

4. **Documentation**
- [ ] All docs reviewed and updated
- [ ] Examples tested on fresh install
- [ ] Migration guides validated

5. **Deployment**
- [ ] Deploy to staging
- [ ] Monitor for 48h
- [ ] Deploy to production (gradual rollout)
- [ ] Update public website

**Performance SLOs (v1.0):**
| Operation | p50 | p95 | p99 |
|-----------|-----|-----|-----|
| Simple extraction | 200ms | 500ms | 1000ms |
| Schema extraction | 500ms | 1500ms | 3000ms |
| LLM extraction | 1500ms | 3500ms | 5000ms |
| Streaming (10 URLs) | 2000ms | 5000ms | 8000ms |

**Acceptance Criteria:**
- [ ] All tests pass (unit, integration, golden, e2e)
- [ ] Coverage ≥ 80% per crate
- [ ] Performance SLOs met
- [ ] No P0/P1 bugs
- [ ] Documentation complete
- [ ] v1.0.0 tagged and released

---

### Phase 4 Success Criteria

**Definition of Done:**
- [x] 20+ golden tests pass
- [x] Coverage ≥ 80% per crate
- [x] All documentation complete
- [x] Examples tested on fresh install
- [x] Performance SLOs met
- [x] v1.0.0 deployed to production

---

## 🎯 What's In vs. Out for v1.0

### ✅ v1.0 SCOPE (MUST SHIP)

**Core Extraction API:**
- [x] Dead-simple extraction: `client.extract(url)`
- [x] Schema-driven extraction: `client.extract(url, schema="events")`
- [x] Python SDK via PyO3
- [x] Builder pattern for configuration
- [x] Error codes & structured errors

**Schemas:**
- [x] Events schema with iCalendar output
- [x] Jobs schema
- [x] Products schema
- [x] Articles schema
- [x] Custom schema support

**Infrastructure:**
- [x] Consolidated utilities (no duplication)
- [x] Unified facade architecture (100% usage)
- [x] TDD London School testing
- [x] 80%+ coverage per crate
- [x] CI/CD pipeline

### 🟡 v1.0 STRETCH GOALS (SHIP IF TIME PERMITS)

**Streaming:**
- [ ] Stream API for 100+ URLs
- [ ] Backpressure control
- [ ] Progress tracking

**Pipeline:**
- [ ] Search → Extract pipeline
- [ ] Auto-discovery crawling
- [ ] Deduplication

### ❌ v1.1 DEFERRED (EXPLICITLY OUT OF SCOPE)

**Why Defer:**
- Multi-tenancy infrastructure exists but not exposed (needs stabilization)
- Advanced features can wait until v1.0 adoption proves demand
- Focus v1.0 on core UX excellence

**Deferred Features:**
- [ ] Multi-tenancy (tenant isolation, quotas)
- [ ] Auto schema detection (>80% accuracy)
- [ ] GraphQL API layer
- [ ] WebSocket streaming
- [ ] Browser extension integration
- [ ] Self-healing selectors
- [ ] Visual programming interface

---

## 📊 Success Metrics

### Technical Metrics

| Metric | Baseline | v1.0 Target | Status |
|--------|----------|-------------|--------|
| Test Coverage | TBD | ≥80% per crate | 🔴 |
| Code Duplication | ~2,580 lines | 0 lines | 🔴 |
| Facade Usage | 40% handlers | 100% handlers | 🔴 |
| Error Handling | 92 manual conversions | 0 manual | 🔴 |
| Test Count | 461 | 500+ | 🟡 |
| Performance (simple) | TBD | p95 < 500ms | 🟡 |
| Performance (schema) | TBD | p95 < 1500ms | 🟡 |

### Developer Experience Metrics

**Pre-v1.0 (Current):**
```python
# ❌ Complex, confusing
from riptide_api import PipelineOrchestrator, AppState
state = AppState.from_env()
orchestrator = PipelineOrchestrator(state, options)
result = orchestrator.execute_single(url).await
```

**Post-v1.0 (Target):**
```python
# ✅ Simple, intuitive
from riptide import RipTide
client = RipTide()
result = client.extract(url)
```

**Adoption Metrics (First 30 Days):**
- [ ] 100+ developers try RipTide
- [ ] 1000+ API calls daily
- [ ] 5+ production integrations
- [ ] 4.5+ stars on GitHub
- [ ] < 5 min average time to first extraction

---

## 🚨 Risk Management

### High-Risk Items

**1. Refactoring 54 Handlers (Weeks 5-7)**
- **Risk:** Breaking existing functionality
- **Mitigation:**
  - TDD London School (write tests first)
  - Golden tests (regression detection)
  - Gradual migration (6 → 20 → 28 handlers)
  - Each handler tested independently

**2. Utility Consolidation (Week 0)**
- **Risk:** Breaking changes across multiple crates
- **Mitigation:**
  - Move code, don't rewrite
  - Comprehensive tests before migration
  - Migrate one crate at a time
  - Rollback plan (git revert)

**3. PyO3 Python SDK (Week 7-8)**
- **Risk:** Async runtime complexity
- **Mitigation:**
  - Use proven PyO3 patterns
  - Start with sync API, add async later
  - Comprehensive error handling
  - Clear documentation

### Medium-Risk Items

**4. Schema Auto-Detection (If Attempted)**
- **Risk:** Accuracy below 80%
- **Mitigation:**
  - Defer to v1.1 if time tight
  - Start with manual schema selection
  - Build detection incrementally

**5. Streaming Backpressure (Week 11-12)**
- **Risk:** Memory exhaustion on large streams
- **Mitigation:**
  - Use existing `riptide-streaming` crate
  - Comprehensive load testing
  - Configurable buffer sizes

---

## 📅 Weekly Checkpoint Format

**Every Monday:**
1. **Last Week:**
   - What shipped?
   - What blocked?
   - Metrics update

2. **This Week:**
   - Sprint goals
   - Risk assessment
   - Resource needs

3. **Blockers:**
   - Technical blockers
   - Resource blockers
   - Decision needed

**Example Checkpoint (Week 2):**
```markdown
## Week 2 Checkpoint - 2025-11-11

### Last Week (Week 1):
✅ Shipped:
- riptide-utils crate created (~2,580 lines consolidated)
- StrategyError type added (15 variants)
- server.yaml support implemented

❌ Blocked:
- TDD guide delayed (waiting on final examples)

📊 Metrics:
- Code reduction: -2,520 lines
- Tests passing: 461 baseline + 25 new utils tests

### This Week (Week 2):
🎯 Goals:
- Create OrchestrationFacade wrapper
- Start PdfFacade implementation
- Complete TDD guide

⚠️ Risks:
- Orchestrator wrapping may reveal integration issues
- Need AppState refactoring decision

### Blockers:
None currently

### Decisions Needed:
- Confirm AppState structure for facade integration
```

---

## 🎯 Critical Path Analysis

```
Week 0-1: Foundation (P0 BLOCKERS)
├── riptide-utils ────────────────────┐
├── StrategyError ────────────────────┤
├── ApiConfig rename ─────────────────┤
└── server.yaml ──────────────────────┤
                                      ↓
Week 2-4: Facade Wrappers ────────────┤
├── OrchestrationFacade ──────────────┤
├── 5 missing facades ────────────────┤
└── Builder enhancements ─────────────┤
                                      ↓
Week 5-7: Handler Refactoring ────────┤
├── Low risk (6 handlers) ────────────┤
├── Medium risk (20 handlers) ────────┤
└── High risk (28 handlers) ──────────┤
                                      ↓
Week 7-10: Simple API ─────────────────┤
├── Dead-simple extract() ────────────┤
├── Python SDK ───────────────────────┤
├── Schema support ───────────────────┤
└── Error codes ──────────────────────┤
                                      ↓
Week 11-14: Advanced (OPTIONAL) ───────┤
├── Streaming ────────────────────────┤
└── Pipeline (may defer) ─────────────┤
                                      ↓
Week 14-16: Polish ────────────────────┘
├── Golden tests
├── Documentation
└── Deployment
```

**Critical Path Items:**
- Week 0-1: Foundation (blocks everything)
- Week 2-4: Facade wrappers (blocks handler refactoring)
- Week 5-7: Handler refactoring (blocks API stability)
- Week 7-10: Simple API (blocks v1.0 launch)

**Parallel Work:**
- Weeks 11-14: Streaming can happen while docs are written
- Weeks 14-16: Testing and docs can overlap

---

## 📝 Communication Plan

### Stakeholder Updates

**Weekly (Fridays):**
- Progress report to leadership
- Metrics dashboard update
- Risk/blocker escalation

**Bi-weekly (Mondays):**
- Team sync
- Sprint planning
- Technical decisions

**Ad-hoc:**
- Critical blockers
- Scope change requests
- Architecture decisions

### Documentation Cadence

**Real-time:**
- Code comments
- API documentation (via rustdoc)
- Test documentation

**Weekly:**
- Changelog updates
- Migration guide updates
- Example additions

**Release:**
- Complete API reference
- Getting started guide
- Migration guides
- Release notes

---

## 🎉 Launch Criteria

**v1.0 is ready to launch when:**

**Technical:**
- [x] All P0/P1 tests pass (461 baseline + 100+ new)
- [x] Coverage ≥ 80% per crate
- [x] No P0/P1 clippy warnings
- [x] Performance SLOs met
- [x] Golden tests prevent regressions

**Functional:**
- [x] Dead-simple API works: `client.extract(url)`
- [x] Schema extraction works for 4+ schemas
- [x] Python SDK installable via pip
- [x] Error codes complete (50+ codes)
- [x] Builder pattern supports all configuration

**Documentation:**
- [x] Getting started < 5 minutes
- [x] 10+ working examples
- [x] Complete API reference
- [x] Migration guides (2+ tools)
- [x] Error code documentation

**Deployment:**
- [x] Staging deployment successful
- [x] 48h monitoring shows stability
- [x] Rollback plan tested
- [x] Public website updated

**Adoption:**
- [x] Internal dogfooding successful
- [x] Beta testers approve
- [x] Performance benchmarks public
- [x] Changelog published

---

## 🔄 Continuous Improvement (Post v1.0)

### v1.1 Roadmap (4-6 weeks post v1.0)

**Informed by v1.0 Adoption:**
1. Analyze usage patterns
2. Collect user feedback
3. Identify pain points

**Likely Features:**
- [ ] Auto schema detection (if demanded)
- [ ] Full pipeline API (if streaming popular)
- [ ] Multi-tenancy (if enterprise interest)
- [ ] Advanced streaming (if high-volume use cases)

### v2.0 Vision (6+ months)

**AI-Native Features:**
- Natural language pipelines
- Visual programming interface
- Self-healing selectors
- Automatic optimization

**Ecosystem:**
- Community schemas marketplace
- Custom extractor plugins
- Third-party integrations (Zapier, n8n)

---

## 📖 Appendix

### A. Files to Create/Modify

**New Files (Phase 0):**
- `/crates/riptide-utils/Cargo.toml`
- `/crates/riptide-utils/src/{lib,redis,http,retry,error,time}.rs`
- `/crates/riptide-types/src/strategy_error.rs`
- `/server.yaml`
- `/crates/riptide-config/src/{file_loader,precedence}.rs`
- `/docs/testing/TDD-LONDON-GUIDE.md`

**New Files (Phase 1):**
- `/crates/riptide-facade/src/facades/orchestration.rs`
- `/crates/riptide-facade/src/facades/{pdf,llm,stealth,profile,streaming}.rs`
- `/crates/riptide-facade/tests/*_facade_tests.rs`

**New Files (Phase 2):**
- `/crates/riptide-facade/src/simple.rs`
- `/crates/riptide-py/` (entire Python SDK)
- `/crates/riptide-types/src/schemas/{events,jobs,products,articles}.rs`
- `/crates/riptide-types/src/error_codes.rs`

**Modified Files (Throughout):**
- ~100 files for utility consolidation
- 54 handler files for facade refactoring
- ~20 test files for TDD compliance

### B. External Dependencies

**Existing (No Changes):**
- `reqwest` - HTTP client
- `redis` - Redis client
- `serde` / `serde_json` - Serialization
- `tokio` - Async runtime
- `axum` - Web framework
- `mockall` - Test mocking
- `wiremock` - HTTP mocking

**New Dependencies:**
- `PyO3` - Python bindings (Phase 2)
- `icalendar` - iCalendar output (Phase 2)
- None for Phase 0-1 (use existing)

### C. Team Roles & Responsibilities

**Lead Engineer:**
- Technical decisions
- Architecture review
- Code review
- Blocker resolution

**Backend Engineers (2-3):**
- Rust implementation
- Facade wrappers
- Handler refactoring
- Testing

**SDK Engineer:**
- Python SDK via PyO3
- Language bindings
- SDK documentation

**QA Engineer:**
- Golden tests
- Integration tests
- Performance testing
- Regression prevention

**Technical Writer:**
- API documentation
- Getting started guide
- Examples gallery
- Migration guides

---

## 🎯 Conclusion

This 16-week timeline is **realistic and achievable** because:

1. **We're not rebuilding - we're consolidating:** 1,598 lines of production code already exist
2. **We have excellent test infrastructure:** 461 tests with 85% TDD readiness
3. **We're fixing debt first:** ~2,580 lines of duplication eliminated before building
4. **We're wrapping, not recreating:** Facades delegate to existing orchestrators
5. **We're deferring complexity:** Pipeline and multi-tenancy can wait for v1.1

**The result:** A dead-simple API that developers love, built on battle-tested infrastructure, delivered in 16 weeks.

```python
# The v1.0 promise
from riptide import RipTide

client = RipTide()
result = client.extract("https://example.com")
print(result.content)  # It just works ✨
```

**Status:** 🎯 Ready for Execution
**Next Step:** Team review and sprint kickoff (Week 0)

---

_"Make it work, make it right, make it fast - in that order."_ - Kent Beck

**Let's ship v1.0! 🚀**
