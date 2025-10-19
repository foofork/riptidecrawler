# Riptide Facade Architecture Analysis

**Date:** 2025-10-19
**Status:** Architecture Decision Required
**Phase:** P1-C1 Week 2 Day 6-7 Post-Integration Review

## Executive Summary

This document analyzes the optimal facade architecture for `riptide-facade` given the current 27-crate structure. The goal is to provide a simple, cohesive user-facing API that hides internal complexity while avoiding facade explosion.

**Current State:**
- 4 fully implemented facades (~1,400 LOC total)
- 5 stub facades (17 LOC each, unimplemented)
- 18+ supporting crates with rich functionality

**Recommendation:** **6 Core Facades** (delete 3 stubs, keep 2, add 1 new)

---

## Current Facade Inventory

### ✅ Fully Implemented (4 facades, ~1,400 LOC)

| Facade | LOC | Purpose | Integration Status |
|--------|-----|---------|-------------------|
| **BrowserFacade** | ~980 | Browser automation, CDP, stealth, screenshots | ✅ **Excellent** - Integrates `riptide-headless-hybrid`, `riptide-stealth`, `riptide-engine` |
| **ExtractionFacade** | ~716 | HTML/PDF extraction, multiple strategies | ✅ **Good** - Integrates `riptide-extraction`, `riptide-pdf` |
| **PipelineFacade** | ~779 | Multi-stage workflows, caching, retries | ✅ **Good** - Orchestration layer with templates |
| **ScraperFacade** | ~147 | Simple HTTP scraping | ✅ **Good** - Uses `riptide-fetch` |

**Total:** ~2,622 LOC across 4 facades

### 🔶 Stub Facades (5 stubs, 17 LOC each)

| Facade | Current State | Backing Crate | User Value |
|--------|---------------|---------------|------------|
| **SpiderFacade** | Stub (has `crawl()` signature) | `riptide-spider` (540 LOC, frontier-based crawling) | **HIGH** - Distinct from scraping |
| **CacheFacade** | Empty stub | `riptide-cache` (Redis, warming, keys) | **LOW** - Internal concern |
| **SecurityFacade** | Empty stub | `riptide-security` | **MEDIUM** - Could be settings on other facades |
| **IntelligenceFacade** | Empty stub | `riptide-intelligence` (LLM abstraction) | **HIGH** - Distinct AI extraction feature |
| **MonitoringFacade** | Empty stub | `riptide-monitoring` (telemetry, alerts) | **LOW** - Internal/operational concern |

### ❓ Potentially Missing Facades

| Facade | Backing Crate | Analysis |
|--------|---------------|----------|
| **SearchFacade** | `riptide-search` (Google, Bing, DuckDuckGo via Serper) | **NEEDED** - Distinct user-facing feature |
| **PdfFacade** | `riptide-pdf` | **REDUNDANT** - Already integrated into `ExtractionFacade` |
| **StreamingFacade** | `riptide-streaming` (NDJSON, reports) | **MAYBE** - Could be a method on `PipelineFacade` |
| **StealthFacade** | `riptide-stealth` | **REDUNDANT** - Already integrated into `BrowserFacade` |
| **EventsFacade** | `riptide-events` (pub/sub) | **NO** - Internal orchestration, not user-facing |

---

## Decision Framework

### Criteria for Standalone Facade

A crate should have a standalone facade if it meets **3+ of these criteria:**

1. ✅ **User-Facing Feature**: End users directly invoke this functionality
2. ✅ **Distinct Use Case**: Not a sub-feature of another facade
3. ✅ **Complex API**: Requires significant configuration or state management
4. ✅ **Cohesive Boundary**: Clear separation from other facades
5. ❌ **Not Infrastructure**: Not just plumbing/monitoring/caching

### Criteria for Merging/Deleting

A stub facade should be **merged or deleted** if:

1. ❌ **Internal Concern**: Used by framework, not by users
2. ❌ **Cross-Cutting**: Affects all facades (better as shared config/middleware)
3. ❌ **Low-Level Plumbing**: Caching, events, connection pools
4. ❌ **Already Integrated**: Functionality exposed through another facade

---

## Architectural Analysis

### 1. BrowserFacade (✅ Keep as-is)

**Status:** Fully implemented, excellent integration
**Size:** ~980 LOC
**Integration:**
- `riptide-headless-hybrid` - Browser launcher with stealth
- `riptide-stealth` - Anti-detection features
- `riptide-engine` - CDP protocol, browser pooling

**Decision:** ✅ **KEEP** - Core user-facing feature with clear boundaries

**Rationale:**
- Browser automation is a distinct use case (vs simple HTTP fetch)
- Complex API: sessions, actions, cookies, screenshots, JavaScript execution
- Stealth integration is appropriate here (not a separate facade)

---

### 2. ExtractionFacade (✅ Keep as-is)

**Status:** Fully implemented
**Size:** ~716 LOC
**Integration:**
- `riptide-extraction` - Multiple extraction strategies
- `riptide-pdf` - PDF text/image extraction

**Decision:** ✅ **KEEP** - Rich extraction layer with fallback chains

**Rationale:**
- Extraction is a core operation separate from fetching
- Handles HTML, PDF, schema-based extraction
- Fallback strategy chain is complex and user-facing
- PDF integration is appropriate here (no separate `PdfFacade` needed)

---

### 3. PipelineFacade (✅ Keep as-is)

**Status:** Fully implemented
**Size:** ~779 LOC
**Integration:**
- Orchestrates `BrowserFacade`, `ExtractionFacade`, `ScraperFacade`
- Provides retry, caching, parallel execution

**Decision:** ✅ **KEEP** - Essential orchestration layer

**Rationale:**
- Multi-stage workflows are a distinct user need
- Provides pre-built pipeline templates (web scraping, PDF extraction, browser automation)
- Handles cross-cutting concerns: retries, caching, parallel execution
- Clear separation from single-operation facades

**Consideration:** `riptide-streaming` could be integrated here for progress tracking and NDJSON output.

---

### 4. ScraperFacade (✅ Keep as-is)

**Status:** Fully implemented
**Size:** ~147 LOC
**Integration:**
- `riptide-fetch` - HTTP client with timeout

**Decision:** ✅ **KEEP** - Simplest entry point for basic scraping

**Rationale:**
- Provides the simplest API for basic HTTP scraping
- Clear use case: "I just want to fetch HTML from a URL"
- Low complexity, but distinct from browser automation
- Users shouldn't need `BrowserFacade` for simple tasks

---

### 5. SpiderFacade (🔶 Implement, currently stub)

**Status:** Stub with `crawl()` signature
**Backing Crate:** `riptide-spider` (540 LOC)
**Features:**
- Frontier-based URL queue management
- Multiple crawling strategies (BFS, DFS, Best-First)
- Adaptive stopping (content-based termination)
- Budget controls (time, depth, page count)
- Rate limiting, session persistence
- Query-aware crawling (relevance-based prioritization)

**Decision:** ✅ **IMPLEMENT** - Crawling is distinct from single-page scraping

**Rationale:**
- **Distinct Use Case:** Multi-page crawling vs single-page scraping
- **Complex API:** Frontier management, crawl budgets, stopping conditions
- **User-Facing:** Users explicitly want to "crawl a site" (not just scrape one page)
- **Rich Backend:** `riptide-spider` has 540 LOC of sophisticated crawling logic

**Recommended API:**
```rust
pub struct SpiderFacade {
    config: RiptideConfig,
    spider: Spider,
}

impl SpiderFacade {
    pub async fn crawl(&self, start_url: &str, budget: CrawlBudget) -> Result<CrawlResult>;
    pub async fn crawl_with_strategy(&self, start_url: &str, strategy: CrawlingStrategy) -> Result<CrawlResult>;
    pub async fn query_aware_crawl(&self, start_url: &str, query: &str) -> Result<CrawlResult>;
}

pub struct CrawlBudget {
    pub max_pages: Option<usize>,
    pub max_depth: Option<u32>,
    pub timeout_secs: Option<u64>,
}

pub struct CrawlResult {
    pub pages: Vec<ExtractedDoc>,
    pub total_pages: usize,
    pub frontier_stats: FrontierStats,
}
```

---

### 6. SearchFacade (➕ Add new)

**Status:** Missing
**Backing Crate:** `riptide-search` (Google, Bing, DuckDuckGo via Serper API)
**Features:**
- Multiple search backends (Serper, None, SearXNG)
- Circuit breaker for reliability
- Async/await support
- Type-safe results with metadata

**Decision:** ✅ **ADD** - Search is a distinct user-facing feature

**Rationale:**
- **User-Facing Feature:** Users want to "search Google for URLs to scrape"
- **Distinct Use Case:** Search is separate from scraping/crawling
- **Clear API:** Search query → list of URLs
- **Rich Backend:** `riptide-search` has multiple backends and circuit breaker

**Recommended API:**
```rust
pub struct SearchFacade {
    config: RiptideConfig,
    provider: Box<dyn SearchProvider>,
}

impl SearchFacade {
    pub async fn search(&self, query: &str, limit: u32) -> Result<Vec<SearchHit>>;
    pub async fn search_with_locale(&self, query: &str, limit: u32, country: &str, locale: &str) -> Result<Vec<SearchHit>>;
    pub fn backend_type(&self) -> SearchBackend;
}

pub struct SearchHit {
    pub url: String,
    pub rank: u32,
    pub title: Option<String>,
    pub snippet: Option<String>,
    pub metadata: HashMap<String, String>,
}
```

**Typical Workflow:**
```rust
// 1. Search for URLs
let search = Riptide::builder().build_search().await?;
let hits = search.search("rust web scraping", 10).await?;

// 2. Scrape search results
let scraper = Riptide::builder().build_scraper().await?;
for hit in hits {
    let html = scraper.fetch_html(&hit.url).await?;
    // ...
}
```

---

### 7. IntelligenceFacade (🔶 Implement, currently stub)

**Status:** Empty stub
**Backing Crate:** `riptide-intelligence` (LLM abstraction layer)
**Features:**
- Vendor-agnostic LLM API (OpenAI, Anthropic, Azure, Bedrock, Vertex, Ollama)
- Circuit breaker, timeout, fallback chains
- Cost tracking, metrics, tenant isolation
- Background processing, hot-reload

**Decision:** ✅ **IMPLEMENT** - AI extraction is a distinct, high-value feature

**Rationale:**
- **Distinct Use Case:** AI-powered extraction vs CSS selectors
- **User-Facing:** Users want "extract product info using AI"
- **Complex API:** Provider selection, prompt engineering, cost tracking
- **Strategic Value:** AI extraction is a key differentiator

**Recommended API:**
```rust
pub struct IntelligenceFacade {
    config: RiptideConfig,
    client: IntelligenceClient,
}

impl IntelligenceFacade {
    pub async fn extract_with_schema(&self, html: &str, schema: &str) -> Result<serde_json::Value>;
    pub async fn extract_with_prompt(&self, html: &str, prompt: &str) -> Result<String>;
    pub async fn structured_extraction(&self, html: &str, fields: Vec<FieldSpec>) -> Result<HashMap<String, String>>;
    pub fn estimate_cost(&self, text_length: usize) -> Option<Cost>;
}
```

**Integration with ExtractionFacade:**
- `ExtractionFacade` should add an `ExtractionStrategy::Ai` variant
- `IntelligenceFacade` is the low-level API for direct AI extraction
- Pipeline-based extraction uses `ExtractionFacade`, one-off AI extraction uses `IntelligenceFacade`

---

### 8. CacheFacade (❌ Delete stub)

**Status:** Empty stub
**Backing Crate:** `riptide-cache` (Redis, warming, conditional requests)

**Decision:** ❌ **DELETE** - Caching is a cross-cutting concern

**Rationale:**
- **Not User-Facing:** Users don't think "I want to cache something"
- **Cross-Cutting:** Affects all facades (HTTP caching, extraction caching, browser cache)
- **Implementation Detail:** Should be configured via `RiptideConfig`, not a facade
- **Better Approach:** Add caching options to each facade

**Better API:**
```rust
// Instead of CacheFacade, add caching to RiptideConfig
let config = RiptideConfig::default()
    .with_redis_cache("redis://localhost:6379")
    .with_cache_ttl(3600)
    .with_cache_warming(true);

// Caching is transparent in facades
let scraper = Riptide::builder().config(config).build_scraper().await?;
let html = scraper.fetch_html("https://example.com").await?; // Cached automatically
```

---

### 9. SecurityFacade (❌ Delete stub)

**Status:** Empty stub
**Backing Crate:** `riptide-security`

**Decision:** ❌ **DELETE** - Security is a cross-cutting concern

**Rationale:**
- **Not a Distinct Use Case:** Security is not a user operation
- **Cross-Cutting:** Rate limiting, authentication, stealth apply to all facades
- **Better Approach:** Security settings in `RiptideConfig` and per-facade options

**Better API:**
```rust
// Security via config
let config = RiptideConfig::default()
    .with_rate_limit(10) // 10 requests/second
    .with_user_agent("MyBot/1.0")
    .with_stealth_enabled(true)
    .with_stealth_preset("High");

// Per-facade security
let browser = Riptide::builder()
    .config(config)
    .build_browser()
    .await?;
```

**Note:** Stealth features are already integrated into `BrowserFacade` via `riptide-headless-hybrid`.

---

### 10. MonitoringFacade (❌ Delete stub)

**Status:** Empty stub
**Backing Crate:** `riptide-monitoring` (telemetry, alerts, metrics)

**Decision:** ❌ **DELETE** - Monitoring is an operational concern

**Rationale:**
- **Not User-Facing:** Users don't think "I want to monitor scraping"
- **Operational Concern:** Monitoring is for observability, not scraping operations
- **Cross-Cutting:** Metrics collection happens transparently across all facades
- **Better Approach:** Monitoring via `RiptideConfig` and middleware

**Better API:**
```rust
// Monitoring via config
let config = RiptideConfig::default()
    .with_telemetry_enabled(true)
    .with_metrics_endpoint("http://localhost:9090");

// Access metrics programmatically
let scraper = Riptide::builder().config(config).build_scraper().await?;
let metrics = scraper.get_metrics().await?;
```

---

## Recommended Facade Structure

### ✅ Final Architecture: 6 Core Facades

| # | Facade | Status | LOC | Purpose | Decision |
|---|--------|--------|-----|---------|----------|
| 1 | **BrowserFacade** | ✅ Implemented | ~980 | Browser automation, stealth, CDP | **KEEP** |
| 2 | **ExtractionFacade** | ✅ Implemented | ~716 | HTML/PDF extraction, strategies | **KEEP** |
| 3 | **PipelineFacade** | ✅ Implemented | ~779 | Multi-stage workflows | **KEEP** |
| 4 | **ScraperFacade** | ✅ Implemented | ~147 | Simple HTTP scraping | **KEEP** |
| 5 | **SpiderFacade** | 🔶 Stub → Implement | ~300 (est.) | Multi-page crawling | **IMPLEMENT** |
| 6 | **SearchFacade** | ➕ Add new | ~200 (est.) | Search engine integration | **ADD** |
| 7 | **IntelligenceFacade** | 🔶 Stub → Implement | ~250 (est.) | AI-powered extraction | **IMPLEMENT** |

**Deleted Stubs:**
- ❌ **CacheFacade** → Cross-cutting concern, belongs in config
- ❌ **SecurityFacade** → Cross-cutting concern, belongs in config
- ❌ **MonitoringFacade** → Operational concern, not user-facing

**Total Estimated LOC:** ~3,372 lines across 6 facades (+ 750 for new implementations)

---

## Implementation Priorities

### Phase 1: Essential (P1-C2)
1. ✅ **SearchFacade** - High user value, simple implementation (~200 LOC)
   - Integrate `riptide-search` with Serper API
   - Circuit breaker wrapper for reliability
   - Clear API: `search(query) → Vec<SearchHit>`

2. ✅ **SpiderFacade** - High user value, moderate complexity (~300 LOC)
   - Integrate `riptide-spider` frontier manager
   - Expose crawl budgets and strategies
   - Query-aware crawling for relevance

### Phase 2: Advanced (P1-C3+)
3. 🔶 **IntelligenceFacade** - Strategic feature, complex integration (~250 LOC)
   - Integrate `riptide-intelligence` LLM client
   - Provide schema-based extraction
   - Cost estimation and tracking
   - Add `ExtractionStrategy::Ai` to `ExtractionFacade`

### Phase 3: Cross-Cutting Concerns
4. 🔧 **Delete stub facades** - Remove `CacheFacade`, `SecurityFacade`, `MonitoringFacade`
5. 🔧 **Enhance RiptideConfig** - Move caching, security, monitoring to config
6. 🔧 **Streaming Integration** - Add streaming methods to `PipelineFacade`

---

## Usage Examples

### 1. Basic Scraping (ScraperFacade)
```rust
let scraper = Riptide::builder().build_scraper().await?;
let html = scraper.fetch_html("https://example.com").await?;
```

### 2. Browser Automation (BrowserFacade)
```rust
let browser = Riptide::builder().build_browser().await?;
let session = browser.launch().await?;
browser.navigate(&session, "https://example.com").await?;
let screenshot = browser.screenshot(&session, ScreenshotOptions::default()).await?;
browser.close(session).await?;
```

### 3. Content Extraction (ExtractionFacade)
```rust
let extractor = Riptide::builder().build_extractor().await?;
let options = HtmlExtractionOptions { as_markdown: true, extract_links: true, ..Default::default() };
let data = extractor.extract_html(&html, "https://example.com", options).await?;
```

### 4. Multi-Stage Pipeline (PipelineFacade)
```rust
let pipeline = Riptide::builder().build_pipeline().await?;
let workflow = pipeline.web_scraping_pipeline("https://example.com").await?;
let result = pipeline.execute(workflow).await?;
```

### 5. Web Crawling (SpiderFacade - NEW)
```rust
let spider = Riptide::builder().build_spider().await?;
let budget = CrawlBudget { max_pages: Some(100), max_depth: Some(3), timeout_secs: Some(300) };
let result = spider.crawl("https://example.com", budget).await?;
println!("Crawled {} pages", result.total_pages);
```

### 6. Search Integration (SearchFacade - NEW)
```rust
let search = Riptide::builder().build_search().await?;
let hits = search.search("rust web scraping", 10).await?;
for hit in hits {
    println!("{}: {}", hit.rank, hit.url);
}
```

### 7. AI Extraction (IntelligenceFacade - NEW)
```rust
let intel = Riptide::builder().build_intelligence().await?;
let schema = r#"{"product_name": "string", "price": "number", "rating": "number"}"#;
let structured_data = intel.extract_with_schema(&html, schema).await?;
```

---

## Configuration Strategy

### RiptideConfig Enhancements

**Current config** focuses on basic settings (user agent, timeout, proxy).
**Enhanced config** should handle cross-cutting concerns:

```rust
pub struct RiptideConfig {
    // Basic settings (existing)
    pub user_agent: String,
    pub timeout: Duration,
    pub proxy: Option<String>,

    // Stealth settings (existing, used by BrowserFacade)
    pub stealth_enabled: bool,
    pub stealth_preset: String,

    // Caching (new)
    pub cache_enabled: bool,
    pub cache_backend: CacheBackend, // Redis, Memory
    pub cache_redis_url: Option<String>,
    pub cache_ttl: u64,
    pub cache_warming_enabled: bool,

    // Security (new)
    pub rate_limit_per_second: Option<u32>,
    pub max_concurrent_requests: usize,
    pub respect_robots_txt: bool,

    // Monitoring (new)
    pub telemetry_enabled: bool,
    pub metrics_endpoint: Option<String>,
    pub tracing_enabled: bool,

    // Intelligence (new)
    pub llm_provider: Option<String>, // "openai", "anthropic", "ollama"
    pub llm_api_key: Option<String>,
    pub llm_model: Option<String>,

    // Search (new)
    pub search_backend: SearchBackend, // Serper, None, SearXNG
    pub search_api_key: Option<String>,
}
```

---

## Facade vs. Config: Decision Matrix

| Feature | Facade? | Config? | Rationale |
|---------|---------|---------|-----------|
| Browser automation | ✅ Yes | ❌ No | User-facing operation with complex state |
| Extraction strategies | ✅ Yes | ❌ No | User-facing operation with multiple strategies |
| Multi-page crawling | ✅ Yes | ❌ No | User-facing operation with frontier management |
| Search integration | ✅ Yes | ❌ No | User-facing operation with search backends |
| AI extraction | ✅ Yes | ❌ No | User-facing operation with prompt engineering |
| Caching | ❌ No | ✅ Yes | Cross-cutting concern, transparent to users |
| Rate limiting | ❌ No | ✅ Yes | Cross-cutting concern, applies to all facades |
| Stealth mode | ❌ No | ✅ Yes | Cross-cutting concern, integrated into BrowserFacade |
| Monitoring | ❌ No | ✅ Yes | Operational concern, not a user operation |
| Authentication | ❌ No | ✅ Yes | Cross-cutting concern, applies to all facades |

---

## Anti-Patterns to Avoid

### ❌ Facade Explosion
**Problem:** 20+ facades, each wrapping a single crate
**Solution:** Only create facades for distinct user-facing operations

### ❌ Anemic Facades
**Problem:** Facades that just delegate to a single crate without adding value
**Solution:** Facades should provide simplified APIs, integrate multiple crates, or orchestrate complex workflows

### ❌ God Facade
**Problem:** Single facade that does everything
**Solution:** Separate concerns by user intent (scrape, crawl, extract, automate)

### ❌ Implementation Leakage
**Problem:** Exposing internal types (e.g., `chromiumoxide::Page`) in facade APIs
**Solution:** Define facade-specific types (e.g., `BrowserSession`, `ExtractedData`)

### ❌ Inconsistent Patterns
**Problem:** Each facade uses different builder patterns, error types, config styles
**Solution:** Standardize on `RiptideBuilder`, `RiptideError`, `RiptideConfig`

---

## Architectural Principles

### 1. User-Centric Design
- **Facades map to user intentions**, not internal crates
- Ask: "What does the user want to do?" not "What crate do we have?"

### 2. Cohesion Over Completeness
- **6-8 well-designed facades** are better than 20+ thin wrappers
- Each facade should have a clear, distinct purpose

### 3. Hide Complexity
- **Internal details** (CDP protocol, WASM modules, Redis keys) stay hidden
- **Simple cases** should be 1-3 lines of code
- **Complex cases** should be possible without breaking abstractions

### 4. Composability
- **Facades can be composed** (e.g., `PipelineFacade` orchestrates `BrowserFacade` + `ExtractionFacade`)
- **Config is shared** across facades via `RiptideConfig`
- **Errors are consistent** using `RiptideError`

### 5. Progressive Disclosure
- **Simple API** for common cases (e.g., `scraper.fetch_html(url)`)
- **Advanced API** for power users (e.g., `extractor.extract_with_fallback(html, url, strategies)`)
- **Escape hatches** for edge cases (access to underlying crate if needed)

---

## Migration Plan

### Week 1: Cleanup & Foundation
1. ✅ **Delete stub facades** (`CacheFacade`, `SecurityFacade`, `MonitoringFacade`)
2. ✅ **Enhance RiptideConfig** with caching, security, monitoring fields
3. ✅ **Update RiptideBuilder** to support new facades

### Week 2: Search & Spider
4. ✅ **Implement SearchFacade** (~200 LOC)
   - Integrate `riptide-search` with circuit breaker
   - Add `build_search()` to `RiptideBuilder`
   - Write integration tests and documentation

5. ✅ **Implement SpiderFacade** (~300 LOC)
   - Integrate `riptide-spider` frontier manager
   - Add crawl budget controls
   - Write integration tests and documentation

### Week 3: Intelligence
6. 🔶 **Implement IntelligenceFacade** (~250 LOC)
   - Integrate `riptide-intelligence` LLM client
   - Add schema-based extraction methods
   - Add `ExtractionStrategy::Ai` to `ExtractionFacade`
   - Write integration tests and documentation

### Week 4: Polish & Documentation
7. 🔧 **Update main facade module** (`riptide-facade/src/lib.rs`)
   - Re-export new facades
   - Update prelude module
   - Update main documentation

8. 🔧 **Write facade integration guide** (`docs/guide/facades.md`)
   - Usage examples for each facade
   - Decision flowchart: "Which facade should I use?"
   - Migration guide from direct crate usage

9. 🔧 **Update CLI and API** (`riptide-cli`, `riptide-api`)
   - Add search, spider, intelligence commands
   - Update API endpoints for new facades

---

## Success Metrics

### Code Quality
- ✅ **Facade count:** 6-8 core facades (not 20+)
- ✅ **Average facade size:** 200-800 LOC (well-factored, not too small, not too large)
- ✅ **Test coverage:** >80% for all facades
- ✅ **Documentation:** Every facade has usage examples and integration tests

### User Experience
- ✅ **Simple cases:** 1-3 lines of code for common operations
- ✅ **Discovery:** Users can find the right facade quickly
- ✅ **Consistency:** Same patterns (builder, config, errors) across facades
- ✅ **Composability:** Facades work together seamlessly

### Architectural Health
- ✅ **Separation of concerns:** Clear boundaries between facades
- ✅ **No leaky abstractions:** Internal types not exposed in facade APIs
- ✅ **Cross-cutting handled:** Caching, security, monitoring in config (not facades)
- ✅ **Extensibility:** Easy to add new facades without refactoring existing ones

---

## Conclusion

**Recommended Facade Structure:**
1. ✅ **BrowserFacade** - Browser automation (keep as-is)
2. ✅ **ExtractionFacade** - Content extraction (keep as-is)
3. ✅ **PipelineFacade** - Multi-stage workflows (keep as-is)
4. ✅ **ScraperFacade** - Simple HTTP scraping (keep as-is)
5. ➕ **SpiderFacade** - Multi-page crawling (implement stub)
6. ➕ **SearchFacade** - Search engine integration (add new)
7. 🔶 **IntelligenceFacade** - AI extraction (implement stub, future)

**Deleted Stubs:**
- ❌ **CacheFacade** → Move to config
- ❌ **SecurityFacade** → Move to config
- ❌ **MonitoringFacade** → Move to config

**Rationale:**
- **User-centric design:** Facades map to user intentions (scrape, crawl, extract, search, automate)
- **Avoid facade explosion:** 6 core facades cover all use cases without overwhelming users
- **Hide complexity:** Cross-cutting concerns (caching, security) in config, not facades
- **Clear boundaries:** Each facade has distinct purpose and API surface

**Next Steps:**
1. Implement `SearchFacade` (highest priority, simple implementation)
2. Implement `SpiderFacade` (high priority, moderate complexity)
3. Enhance `RiptideConfig` for cross-cutting concerns
4. Delete stub facades and update documentation
5. (Future) Implement `IntelligenceFacade` for AI extraction

---

**Document Version:** 1.0
**Last Updated:** 2025-10-19
**Authors:** System Architecture Designer (AI Agent)
**Status:** Ready for Review
