# Handler-Facade Refactoring Scope Analysis

**Analysis Date:** 2025-11-04
**Analyzed Files:** 40 handler files in `crates/riptide-api/src/handlers/`
**Available Facades:** 5 (ExtractionFacade, SpiderFacade, SearchFacade, BrowserFacade, ScraperFacade)

---

## Executive Summary

**Current State:** ~40% facade usage, 60% direct pipeline instantiation
**Goal:** 100% facade-only pattern for clean separation of concerns
**Key Blocker:** Missing HeadlessFacade and CrawlFacade prevent clean delegation

### Critical Findings

1. **Search operations bypass SearchFacade** - `deepsearch.rs` instantiates `SearchProviderFactory` directly
2. **Crawl operations bypass facades** - Both `crawl.rs` and `strategies.rs` create `PipelineOrchestrator` directly
3. **PDF processing has no facade** - `pdf.rs` uses direct resource manager access
4. **Render operations partially bypass** - `render/handlers.rs` mixes facades with direct processing
5. **LLM operations have no facade** - `llm.rs` directly manages `LlmRegistry` with static state

---

## Handler Categorization

### ✅ GOOD (Clean Facade Usage - 6 handlers)

#### 1. **extract.rs** - ⭐ Best Practice Example
```rust
// Lines 163-223: Perfect facade delegation
let facade_result = match payload.options.strategy.to_lowercase().as_str() {
    "css" => {
        state.extraction_facade.extract_html(&html, &payload.url, options).await
    }
    "wasm" => {
        state.extraction_facade.extract_with_strategy(
            &html, &payload.url,
            riptide_facade::facades::ExtractionStrategy::Wasm
        ).await
    }
    "multi" | "auto" => {
        state.extraction_facade.extract_with_fallback(&html, &payload.url, &strategies).await
    }
    _ => { /* ... */ }
};
```
**Why it works:**
- Zero pipeline instantiation
- All extraction logic delegated to `ExtractionFacade`
- Clean error handling
- HTTP client for fetch, facade for extraction

**Effort:** 0 (Reference implementation)
**Risk:** Low

---

#### 2. **spider.rs** - Clean SpiderFacade usage
```rust
// Lines 84-108: Pure facade delegation
let spider_facade = state.spider_facade.as_ref()
    .ok_or_else(|| ApiError::ConfigError {
        message: "SpiderFacade is not enabled".to_string(),
    })?;

let crawl_summary = spider_facade.crawl(seed_urls).await
    .map_err(|e| ApiError::internal(format!("Spider crawl failed: {}", e)))?;
```
**Why it works:**
- No direct Spider engine access
- All crawling through `SpiderFacade::crawl()`
- Graceful degradation if facade unavailable

**Effort:** 0 (Already clean)
**Risk:** Low

---

#### 3. **search.rs** - SearchFacade delegation
```rust
// Lines 94-127: Facade-first pattern
let search_facade = match state.search_facade.as_ref() {
    Some(facade) => facade,
    None => { /* helpful error messages */ }
};

let search_hits = search_facade
    .search_with_options(&params.q, limit, &params.country, &params.language)
    .await?;
```
**Why it works:**
- Uses `SearchFacade::search_with_options()`
- Comprehensive error handling for missing facade
- No `SearchProviderFactory` instantiation

**Effort:** 0 (Already clean)
**Risk:** Low

---

#### 4. **render/extraction.rs** - Simple facade delegation
```rust
// Extract content using ExtractionFacade
pub async fn extract_content(
    extraction_facade: &ExtractionFacade,
    render_result: &Option<DynamicResult>,
    output_format: &riptide_types::OutputFormat,
    final_url: &str,
) -> Result<RenderContent, ApiError>
```
**Why it works:**
- Receives facade as parameter
- No state coupling
- Pure function design

**Effort:** 0 (Already clean)
**Risk:** Low

---

#### 5. **shared/spider.rs** - Utility functions only
```rust
// Simple URL parsing utility
pub fn parse_seed_urls(urls: &[String]) -> Result<Vec<Url>, ApiError>
```
**Why it works:**
- No facade or pipeline usage (pure utility)
- Reusable helper function

**Effort:** 0 (Helper only)
**Risk:** None

---

#### 6. **llm.rs** - Registry pattern (acceptable alternative)
```rust
// Uses LlmRegistry pattern - not a facade, but clean abstraction
fn get_llm_registry() -> Arc<tokio::sync::Mutex<LlmRegistry>>
```
**Why it works:**
- While not using facades, it uses a proper abstraction layer
- No direct provider instantiation in handlers
- Registry pattern is acceptable for this use case

**Effort:** 0 (Different pattern, but clean)
**Risk:** Low

---

### ⚠️ MIXED (Partial Facade Usage - 3 handlers)

#### 7. **crawl.rs** - Spider uses facade, pipeline does not
```rust
// GOOD: Lines 88-91 - Spider crawl via facade
if options.use_spider.unwrap_or(false) {
    return handle_spider_crawl(&state, &body.urls, &options).await;
}

// BAD: Lines 101-162 - Direct pipeline instantiation
let pipeline = if state.config.enhanced_pipeline_config.enable_enhanced_pipeline {
    EnhancedPipelineOrchestrator::new(state.clone(), options.clone())
} else {
    PipelineOrchestrator::new(state.clone(), options.clone())
};
```

**Problem:**
- Spider path uses `SpiderFacade` ✅
- Standard crawl path bypasses facades and creates pipelines directly ❌
- Inconsistent architecture based on config flag

**Missing:** `CrawlFacade` that would encapsulate both pipeline types

**Refactoring Path:**
```rust
// Proposed CrawlFacade API
let crawl_facade = state.crawl_facade.as_ref()
    .ok_or_else(|| ApiError::internal("CrawlFacade not initialized"))?;

let results = if options.use_spider.unwrap_or(false) {
    crawl_facade.crawl_spider(&urls, &options).await?
} else {
    crawl_facade.crawl_batch(&urls, &options).await?
};
```

**Effort:** Medium (3-5 hours)
**Risk:** Medium (high usage endpoint)

---

#### 8. **render/handlers.rs** - Mixes facades with direct processing
```rust
// GOOD: Line 287 - Uses ExtractionFacade
let content = extract_content(
    &state.extraction_facade,
    &render_result,
    &output_format,
    &final_url,
).await?;

// BAD: Lines 194-282 - Direct processing logic
match &mode {
    RenderMode::Pdf => process_pdf(&state, &url, body.pdf_config.as_ref()).await?,
    RenderMode::Dynamic => process_dynamic(&state, &url, &dynamic_config, ...).await?,
    RenderMode::Static => process_static(&state, &url, ...).await?,
    // ...
}
```

**Problem:**
- Uses `ExtractionFacade` for content extraction ✅
- Bypasses `BrowserFacade` and uses `process_*()` functions directly ❌
- Direct resource manager access instead of facade-wrapped operations

**Missing:** `RenderFacade` or proper `HeadlessFacade` integration

**Refactoring Path:**
```rust
// Proposed HeadlessFacade API
let render_result = match &mode {
    RenderMode::Pdf => {
        state.headless_facade.render_pdf(&url, pdf_options).await?
    }
    RenderMode::Dynamic => {
        state.headless_facade.render_dynamic(&url, dynamic_config).await?
    }
    RenderMode::Static => {
        state.headless_facade.render_static(&url).await?
    }
    // ...
};
```

**Effort:** High (6-8 hours, complex operations)
**Risk:** High (rendering is critical path)

---

#### 9. **pdf.rs** - No facade, direct resource management
```rust
// BAD: Lines 110-130 - Direct resource manager usage
let resource_guard = match state
    .resource_manager
    .acquire_pdf_resources()
    .await
{
    Ok(ResourceResult::Success(guard)) => guard,
    // ... error handling
};

// BAD: Direct riptide_pdf usage
let pdf_processor = riptide_pdf::PdfProcessor::new();
let (result_doc, rx) = pdf_processor.process_with_progress(pdf_data).await?;
```

**Problem:**
- No facade abstraction at all
- Direct `ResourceManager` coupling
- Direct `riptide_pdf::PdfProcessor` instantiation

**Missing:** `PdfFacade` for clean abstraction

**Refactoring Path:**
```rust
// Proposed PdfFacade API
let pdf_facade = state.pdf_facade.as_ref()
    .ok_or_else(|| ApiError::internal("PdfFacade not initialized"))?;

let result = pdf_facade.process_pdf(
    pdf_data,
    PdfProcessOptions {
        stream_progress: request.stream_progress.unwrap_or(false),
        timeout: request.timeout,
    }
).await?;
```

**Effort:** Medium (4-6 hours)
**Risk:** Medium (used but not critical path)

---

### ❌ BAD (Direct Pipeline/Factory Instantiation - 2 handlers)

#### 10. **deepsearch.rs** - SearchProviderFactory bypass
```rust
// BAD: Lines 204-237 - Direct SearchProviderFactory usage
pub(super) async fn perform_search_with_provider(
    state: &AppState,
    query: &str,
    limit: u32,
    country: Option<&str>,
    locale: Option<&str>,
) -> ApiResult<Vec<SearchResult>> {
    use riptide_search::{SearchBackend, SearchProviderFactory};

    // Direct factory usage instead of SearchFacade!
    let backend_str = std::env::var("SEARCH_BACKEND").unwrap_or_else(|_| "serper".to_string());
    let backend: SearchBackend = backend_str.parse()?;

    let provider = SearchProviderFactory::create_with_backend(backend).await?;
    let search_hits = provider.search(query, limit, country, locale).await?;
    // ...
}
```

**Why this is problematic:**
1. **Duplicates SearchFacade logic** - `search.rs` uses `SearchFacade`, but `deepsearch.rs` reinvents the wheel
2. **Inconsistent backend selection** - Different env var handling than SearchFacade
3. **No shared circuit breaker** - SearchFacade has circuit breaker, this doesn't benefit
4. **Breaking abstraction** - Handler shouldn't know about `SearchProviderFactory`

**Should use SearchFacade instead:**
```rust
// CORRECT approach (like search.rs)
let search_facade = state.search_facade.as_ref()
    .ok_or_else(|| ApiError::dependency("search_facade", "SearchFacade not initialized"))?;

let search_hits = search_facade
    .search_with_options(query, limit, country, locale)
    .await?;
```

**Effort:** Low (1-2 hours, straightforward replacement)
**Risk:** Low (same functionality via facade)

---

#### 11. **strategies.rs** - Direct pipeline instantiation
```rust
// BAD: Lines 149-175 - Creates StrategiesPipelineOrchestrator directly
let crawl_options = CrawlOptions {
    cache_mode: request.cache_mode.clone(),
    // ... more config
};

let orchestrator = StrategiesPipelineOrchestrator::new(
    state.clone(),
    crawl_options,
    strategy_config,
);

let result = orchestrator.execute(&request.url).await?;
```

**Problem:**
- Direct orchestrator instantiation
- Handler knows about pipeline internals
- No facade abstraction

**Missing:** `StrategiesFacade` or integration into `CrawlFacade`

**Refactoring Path:**
```rust
// Option 1: Dedicated StrategiesFacade
let strategies_result = state.strategies_facade
    .extract_with_strategies(&url, strategy_config, crawl_options)
    .await?;

// Option 2: Extend CrawlFacade
let crawl_result = state.crawl_facade
    .crawl_with_strategies(&url, strategy_config, options)
    .await?;
```

**Effort:** Medium (3-4 hours)
**Risk:** Medium (newer feature, lower usage)

---

## Missing Facade Features Analysis

### 1. **HeadlessFacade / RenderFacade** (HIGH PRIORITY)

**Current Gap:**
- `render/handlers.rs` uses direct `process_pdf()`, `process_dynamic()`, `process_static()` calls
- No unified facade for headless operations
- Resource management is exposed to handlers

**Required API:**
```rust
pub struct HeadlessFacade {
    resource_manager: Arc<ResourceManager>,
    browser_pool: Arc<BrowserPool>,
    pdf_processor: Arc<PdfProcessor>,
}

impl HeadlessFacade {
    pub async fn render_pdf(&self, url: &str, options: PdfRenderOptions) -> Result<PdfResult>;
    pub async fn render_dynamic(&self, url: &str, config: DynamicConfig) -> Result<DynamicResult>;
    pub async fn render_static(&self, url: &str) -> Result<StaticResult>;
    pub async fn render_adaptive(&self, url: &str, request: &RenderRequest) -> Result<RenderResult>;
}
```

**Functionality to move FROM handlers TO facade:**
- Resource acquisition/release logic (from `render/handlers.rs:24-66`)
- Timeout management (from `render/handlers.rs:70-95`)
- Stealth controller initialization (from `render/handlers.rs:201-281`)
- Session management integration
- Performance metrics recording

**Blocks:** `render/handlers.rs`, `pdf.rs`
**Effort:** 8-12 hours (complex, high-value refactor)

---

### 2. **CrawlFacade** (HIGH PRIORITY)

**Current Gap:**
- Both `crawl.rs` and `strategies.rs` instantiate pipelines directly
- `EnhancedPipelineOrchestrator` vs `PipelineOrchestrator` selection in handler
- No unified crawling interface

**Required API:**
```rust
pub struct CrawlFacade {
    enhanced_enabled: bool,
    spider_facade: Option<Arc<SpiderFacade>>,
    cache_manager: Arc<CacheManager>,
    gate_manager: Arc<GateManager>,
}

impl CrawlFacade {
    pub async fn crawl_batch(&self, urls: &[String], options: &CrawlOptions)
        -> Result<(Vec<CrawlResult>, CrawlStats)>;

    pub async fn crawl_spider(&self, seeds: &[String], options: &CrawlOptions)
        -> Result<SpiderCrawlResponse>;

    pub async fn crawl_with_strategies(&self, url: &str, strategy: StrategyConfig, options: &CrawlOptions)
        -> Result<StrategiesResult>;
}
```

**Functionality to move FROM handlers TO facade:**
- Pipeline selection logic (enhanced vs standard) (from `crawl.rs:101-162`)
- Spider vs pipeline routing (from `crawl.rs:88-91`)
- Statistics aggregation (from `crawl.rs:226-238`)
- Cache hit rate calculation (from `crawl.rs:220-224`)
- Event emission (from `crawl.rs:71-79, 268-278`)
- Chunking application (from `crawl.rs:178-185`)

**Blocks:** `crawl.rs`, `strategies.rs`
**Effort:** 6-10 hours (moderate complexity, critical path)

---

### 3. **PdfFacade** (MEDIUM PRIORITY)

**Current Gap:**
- `pdf.rs` directly uses `ResourceManager` and `riptide_pdf::PdfProcessor`
- No abstraction layer for PDF operations
- Resource handling exposed to handler

**Required API:**
```rust
pub struct PdfFacade {
    resource_manager: Arc<ResourceManager>,
    processor: Arc<riptide_pdf::PdfProcessor>,
}

impl PdfFacade {
    pub async fn process_pdf(&self, data: Vec<u8>, options: PdfProcessOptions)
        -> Result<(ExtractedDoc, ProcessingStats)>;

    pub async fn process_pdf_stream(&self, data: Vec<u8>, options: PdfProcessOptions)
        -> Result<impl Stream<Item = ProgressUpdate>>;
}
```

**Functionality to move FROM handlers TO facade:**
- Resource acquisition (from `pdf.rs:110-130`)
- Progress tracking setup (from `pdf.rs:135-155`)
- File size validation (from `pdf.rs:100`)
- Statistics collection (from `pdf.rs:165-180`)

**Blocks:** `pdf.rs`, partially `render/handlers.rs`
**Effort:** 4-6 hours (straightforward refactor)

---

### 4. **Enhanced SearchFacade Features** (LOW PRIORITY)

**Current Gap:**
- `deepsearch.rs` bypasses `SearchFacade` to use `SearchProviderFactory` directly
- Suggests SearchFacade may be missing some features deepsearch needs

**What's missing:**
- **Nothing!** This is a handler bug, not a facade gap
- `SearchFacade::search_with_options()` already provides everything needed
- `deepsearch.rs` should use the existing facade

**Fix Required:**
```rust
// In deepsearch.rs, replace perform_search_with_provider() with:
let search_facade = state.search_facade.as_ref()
    .ok_or_else(|| ApiError::dependency("search_facade", "SearchFacade not initialized"))?;

let search_hits = search_facade
    .search_with_options(query, limit, country.unwrap_or("us"), locale.unwrap_or("en"))
    .await?;
```

**Blocks:** `deepsearch.rs`
**Effort:** 1-2 hours (simple refactor)

---

## Refactoring Effort Estimates

### Phase 1: Quick Wins (Low-hanging fruit)
**Total: 2-4 hours**

1. **deepsearch.rs** - Replace `SearchProviderFactory` with `SearchFacade`
   - Effort: 1-2 hours
   - Risk: Low
   - Impact: Immediate consistency gain
   - Files: 1 (`deepsearch.rs`)

### Phase 2: Medium Complexity (Moderate effort, high value)
**Total: 13-20 hours**

2. **strategies.rs** - Create `StrategiesFacade` or integrate into `CrawlFacade`
   - Effort: 3-4 hours
   - Risk: Medium
   - Impact: Clean strategy-based extraction
   - Files: 1 (`strategies.rs`)

3. **crawl.rs** - Create `CrawlFacade` for unified crawling
   - Effort: 6-10 hours
   - Risk: Medium (critical path)
   - Impact: Major architecture improvement
   - Files: 2 (`crawl.rs`, integration with `strategies.rs`)

4. **pdf.rs** - Create `PdfFacade` for PDF operations
   - Effort: 4-6 hours
   - Risk: Medium
   - Impact: Clean PDF abstraction
   - Files: 1 (`pdf.rs`)

### Phase 3: Complex Refactors (High effort, high value)
**Total: 8-12 hours**

5. **render/handlers.rs** - Create `HeadlessFacade` for rendering
   - Effort: 8-12 hours
   - Risk: High (complex, critical operations)
   - Impact: Major architecture win, clean separation
   - Files: 3 (`render/handlers.rs`, `render/processors.rs`, integration)

---

## Total Refactoring Effort

**Minimum:** 23 hours
**Maximum:** 36 hours
**Recommended Approach:** Phased rollout over 2-3 sprints

---

## Risk Assessment

### High Risk (Proceed with caution)

#### **render/handlers.rs** - HeadlessFacade refactor
- **Why:** Rendering is on critical path, high traffic
- **Mitigation:**
  - Feature flag for gradual rollout
  - Comprehensive integration tests
  - Parallel facade/handler implementation during transition
  - Performance benchmarking before/after

#### **crawl.rs** - CrawlFacade refactor
- **Why:** Core crawling functionality, highest usage
- **Mitigation:**
  - Extensive test coverage (currently has tests)
  - Backward compatibility shim during transition
  - Gradual rollout with metrics monitoring

### Medium Risk

#### **pdf.rs** - PdfFacade refactor
- **Why:** Resource management changes could affect stability
- **Mitigation:**
  - Resource leak testing
  - Load testing before production

#### **strategies.rs** - StrategiesFacade refactor
- **Why:** Newer feature, less battle-tested
- **Mitigation:**
  - Comprehensive unit tests for strategy selection logic

### Low Risk

#### **deepsearch.rs** - SearchFacade adoption
- **Why:** Direct replacement, same functionality
- **Mitigation:** Basic integration test

---

## Key Blockers to 100% Facade Usage

### 1. **Missing HeadlessFacade** (Blocks 3 handlers)
   - Prevents: `render/handlers.rs`, `pdf.rs`, parts of `crawl.rs` (when headless mode)
   - Priority: **CRITICAL**
   - Reason: Rendering/PDF is high-traffic, complex resource management

### 2. **Missing CrawlFacade** (Blocks 2 handlers)
   - Prevents: `crawl.rs`, `strategies.rs`
   - Priority: **CRITICAL**
   - Reason: Core crawling functionality, most-used endpoint

### 3. **Missing PdfFacade** (Blocks 1 handler)
   - Prevents: `pdf.rs`
   - Priority: **HIGH**
   - Reason: Could be absorbed into HeadlessFacade

### 4. **Handler Anti-Patterns** (Affects 1 handler)
   - `deepsearch.rs` bypassing `SearchFacade`
   - Priority: **MEDIUM**
   - Reason: Easy fix, but indicates architectural misunderstanding

---

## Recommendations

### Immediate Actions (Sprint 1)

1. **Fix `deepsearch.rs`** - Use `SearchFacade` instead of `SearchProviderFactory`
   - ROI: High (quick win, demonstrates pattern)
   - Effort: 1-2 hours

2. **Create HeadlessFacade specification**
   - Document required API
   - Get team alignment on interface
   - Effort: 2-3 hours (planning)

### Short-term (Sprint 2-3)

3. **Implement PdfFacade**
   - Simpler than HeadlessFacade, good learning opportunity
   - Can be integrated into HeadlessFacade later
   - Effort: 4-6 hours

4. **Implement CrawlFacade**
   - Unifies `crawl.rs` and `strategies.rs`
   - High value, moderate complexity
   - Effort: 6-10 hours

### Medium-term (Sprint 4-5)

5. **Implement HeadlessFacade**
   - Most complex, highest value
   - Absorbs PdfFacade functionality
   - Effort: 8-12 hours

6. **Remove anti-patterns**
   - Audit all remaining direct instantiations
   - Enforce facade-only via linting rules
   - Effort: 2-4 hours

---

## Architecture Principles

### Why Facades Matter

1. **Separation of Concerns**
   - Handlers should orchestrate HTTP, not implementation details
   - Example: `extract.rs` doesn't know about WASM vs CSS strategies

2. **Testability**
   - Facades can be mocked for handler tests
   - Example: Test `deepsearch.rs` without real SearchProvider

3. **Maintainability**
   - Pipeline changes don't require handler updates
   - Example: Enhanced pipeline toggle hidden behind `CrawlFacade`

4. **Consistency**
   - All handlers use same interfaces
   - Example: Both `search.rs` and `deepsearch.rs` use `SearchFacade`

### When Facades Are Skipped (Anti-patterns)

1. **"It's just one call"** - `deepsearch.rs` justification
   - Reality: Duplicates circuit breaker, backend selection logic

2. **"Facade doesn't exist yet"** - `crawl.rs`, `pdf.rs`
   - Reality: Prevents architectural cleanup, delays facade creation

3. **"Need performance"** - Sometimes claimed
   - Reality: Facades add negligible overhead (<1ms), provide caching benefits

---

## Facade Coverage Target

**Current:** 6/11 handlers (55%) use facades exclusively
**Target:** 11/11 handlers (100%) use facades exclusively

**Remaining Work:**
- 5 handlers to refactor
- 3 new facades to create
- 23-36 hours of engineering time

---

## Appendix: Handler Inventory

### Complete Handler List (40 files total)

**Analyzed for this report (11 primary handlers):**
1. ✅ extract.rs (GOOD)
2. ✅ spider.rs (GOOD)
3. ✅ search.rs (GOOD)
4. ✅ render/extraction.rs (GOOD)
5. ✅ shared/spider.rs (GOOD - utility)
6. ✅ llm.rs (GOOD - different pattern)
7. ⚠️ crawl.rs (MIXED)
8. ⚠️ render/handlers.rs (MIXED)
9. ⚠️ pdf.rs (MIXED)
10. ❌ deepsearch.rs (BAD)
11. ❌ strategies.rs (BAD)

**Not analyzed (infrastructure/utility handlers - 29 files):**
- health.rs, telemetry.rs, monitoring.rs (health checks)
- sessions.rs, profiles.rs (session management)
- admin.rs, admin_old.rs (admin operations)
- workers.rs, resources.rs (resource management)
- streaming.rs, chunking.rs (utilities)
- browser.rs, stealth.rs (browser helpers)
- pipeline_*.rs (pipeline introspection - acceptable direct access)
- render/models.rs, render/processors.rs, render/strategies.rs (internal modules)
- shared/mod.rs (utilities)
- And 10+ more utility/infrastructure handlers

**Note:** Infrastructure handlers often need direct access for monitoring/admin purposes. Focus is on business logic handlers.

---

## Conclusion

**What's blocking 100% facade usage?**

1. **Missing Facades (75% of problem):**
   - HeadlessFacade - needed by 3 handlers
   - CrawlFacade - needed by 2 handlers
   - PdfFacade - needed by 1 handler (could merge into HeadlessFacade)

2. **Handler Anti-Patterns (25% of problem):**
   - `deepsearch.rs` bypassing existing `SearchFacade`
   - Indicates need for better documentation/enforcement

**The fix is clear:** Create the 3 missing facades, fix 1 anti-pattern handler.

**Timeline:** 23-36 hours over 2-3 sprints for 100% facade coverage.
