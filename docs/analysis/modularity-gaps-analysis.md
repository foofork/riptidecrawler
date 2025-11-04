# RipTide Modularity Gaps Analysis

**Date:** 2025-01-04
**Scope:** riptide-spider, riptide-extraction, riptide-search component coupling analysis
**Goal:** Identify barriers preventing independent, modular usage of components

---

## Executive Summary

The RipTide codebase exhibits **tight coupling** between spider and extraction components that prevents modular, independent usage. While the architecture claims separation of concerns, the implementation reveals:

1. **Spider assumes extraction will happen** - embeds basic extraction logic directly
2. **Extraction module contains spider-specific code** - has a disabled `spider/` submodule creating circular dependencies
3. **Shared state and assumptions** - both components make implicit assumptions about data flow
4. **Facade layer only provides high-level coordination** - doesn't enable truly independent usage

**Current State:** ❌ Components cannot be used independently
**Target State:** ✅ Each component usable standalone with clean composition interfaces

---

## 1. Current Coupling Patterns

### 1.1 Spider Assumes Extraction (Hard Coupling)

**File:** `/workspaces/eventmesh/crates/riptide-spider/src/core.rs`

#### Link Extraction Embedded in Spider Core
```rust
// Lines 3-17, 608-618
/// Basic link extraction using regex (simplified for core)
fn extract_links_basic(content: &str, base_url: &Url) -> Result<Vec<Url>> {
    let link_regex = regex::Regex::new(r#"href\s*=\s*["']([^"']+)["']"#)?;
    // ... regex-based link extraction
}

/// Extract URLs from content using riptide-extraction DOM parser
async fn extract_urls(&self, content: &str, base_url: &Url) -> Result<Vec<Url>> {
    // Use riptide-extraction for proper DOM-based link extraction
    // Basic link extraction using regex (simplified for core)
    // Full DOM-based extraction is available in riptide-extraction crate
    let links = extract_links_basic(content, base_url)?;
    // ...
}
```

**Problem:**
- Spider has **built-in extraction logic** (regex-based link parsing)
- Comments reference riptide-extraction but don't actually use it
- Cannot swap extraction strategies without modifying spider
- Violates separation of concerns

#### Text Content Extraction Embedded
```rust
// Lines 620-647
/// Extract text content from HTML using riptide-extraction DOM parser
async fn extract_text_content(&self, content: &str) -> Option<String> {
    // Use riptide-extraction for proper DOM-based text extraction
    // Simplified for core - full DOM extraction in riptide-extraction
    self.simple_text_extraction(content)
}

/// Simple text extraction fallback method
fn simple_text_extraction(&self, content: &str) -> Option<String> {
    let mut text = String::new();
    let mut in_tag = false;

    for char in content.chars() {
        match char {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag && !c.is_control() => text.push(c),
            _ => {}
        }
    }
    // ...
}
```

**Problem:**
- Spider implements **its own HTML text extraction**
- Duplicates functionality that should be in extraction crate
- No plugin interface to inject extraction strategies

#### Direct Processing in Crawl Loop
```rust
// Lines 443-560 (process_request method)
async fn process_request(&self, request: CrawlRequest) -> Result<CrawlResult> {
    // ... fetch logic ...

    let (success, content_size, error) = match fetch_result {
        Ok((content, size)) => {
            // COUPLING: Spider directly extracts URLs and text
            let extracted_urls = self.extract_urls(&content, &request.url).await?;
            let text_content = self.extract_text_content(&content).await;

            // ... build result with extracted data
        }
        // ...
    };
}
```

**Problem:**
- Spider's main processing loop **directly calls extraction methods**
- No abstraction layer for pluggable extractors
- Cannot use spider for just crawling without extraction

### 1.2 Extraction Contains Spider Code (Circular Dependency)

**File:** `/workspaces/eventmesh/crates/riptide-extraction/src/lib.rs`

```rust
// Lines 58-59
// pub mod spider;  // Temporarily disabled due to compilation errors
pub mod chunking;
```

**File:** `/workspaces/eventmesh/crates/riptide-extraction/src/spider/mod.rs`

```rust
// Lines 1-23
//! DOM-specific spider functionality for HTML content processing
//!
//! This module provides HTML-specific crawling capabilities extracted from riptide-core:
//! - Link extraction from HTML content
//! - Form detection and parsing
//! - Meta tag extraction
//! - HTML content analysis for spider optimization

pub mod dom_crawler;
pub mod link_extractor;
pub mod form_parser;
pub mod meta_extractor;
pub mod traits;

// Re-export main types and traits
pub use traits::{DomSpider, DomCrawlerResult, FormData, MetaData};
pub use dom_crawler::HtmlDomCrawler;
pub use link_extractor::HtmlLinkExtractor;
pub use form_parser::HtmlFormParser;
pub use meta_extractor::HtmlMetaExtractor;
```

**Problem:**
- Extraction crate has **spider-specific submodule** (`spider/`)
- Module is **currently disabled** to avoid circular dependencies
- Indicates architectural confusion about responsibility boundaries
- Cannot use extraction without thinking about spider concerns

### 1.3 Shared Types Create Coupling

**File:** `/workspaces/eventmesh/crates/riptide-spider/src/types.rs`

```rust
// Lines 99-193
#[derive(Debug, Clone)]
pub struct CrawlResult {
    pub request: CrawlRequest,
    pub success: bool,
    pub status_code: Option<u16>,
    pub content_type: Option<String>,
    pub content_size: usize,
    pub text_content: Option<String>,        // ← Extracted content
    pub extracted_urls: Vec<Url>,            // ← Extracted URLs
    pub processing_time: Duration,
    pub error: Option<String>,
    pub metadata: HashMap<String, String>,
}
```

**Problem:**
- `CrawlResult` type **embeds extraction results** (`text_content`, `extracted_urls`)
- Spider types assume extraction has already happened
- Cannot represent "crawled but not extracted" state cleanly
- Tight coupling between crawl and extract phases

### 1.4 Facade Provides Coordination, Not Independence

**File:** `/workspaces/eventmesh/crates/riptide-facade/src/facades/spider.rs`

```rust
// Lines 29-84
pub struct SpiderFacade {
    spider: Arc<Mutex<Spider>>,
}

impl SpiderFacade {
    pub async fn from_preset(preset: SpiderPreset, base_url: Url) -> Result<Self> {
        let mut config = match preset { /* ... */ };
        config.base_url = base_url;
        let spider = Spider::new(config).await?;

        Ok(Self {
            spider: Arc::new(Mutex::new(spider)),
        })
    }

    pub async fn crawl(&self, seeds: Vec<Url>) -> Result<CrawlSummary> {
        let spider = self.spider.lock().await;
        let result = spider.crawl(seeds).await?;
        Ok(CrawlSummary::from(result))
    }
}
```

**File:** `/workspaces/eventmesh/crates/riptide-facade/src/facades/extractor.rs`

```rust
// Lines 170-189
pub struct ExtractionFacade {
    config: RiptideConfig,
    extractors: Arc<RwLock<ExtractionRegistry>>,
    pdf_processor: AnyPdfProcessor,
}

impl ExtractionFacade {
    pub async fn extract_html(
        &self,
        html: &str,
        url: &str,
        options: HtmlExtractionOptions,
    ) -> Result<ExtractedData> {
        // ... extraction logic
    }
}
```

**Problem:**
- Facades provide **high-level orchestration** only
- SpiderFacade wraps Spider but doesn't decouple extraction
- ExtractionFacade has no awareness of spider concerns
- No composition interface - can't easily chain spider → extraction

**File:** `/workspaces/eventmesh/crates/riptide-facade/src/facades/pipeline.rs`

```rust
// Lines 144-159
match stage {
    PipelineStage::Fetch { url, options } => {
        self.execute_fetch(url, options, context).await?
    }
    PipelineStage::Extract { strategy } => self.execute_extract(strategy, context).await?,
    // ...
}
```

**Problem:**
- Pipeline provides orchestration but **stages are still coupled**
- Extract stage assumes Fetch stage output format
- No clear contract between stages

---

## 2. Barriers to Independent Usage

### 2.1 Can You Use Spider Without Extraction?

**Answer: ❌ NO - Spider always performs extraction**

#### What Would Break?

1. **CrawlResult always includes extracted data**
   ```rust
   // File: riptide-spider/src/types.rs:99-121
   pub struct CrawlResult {
       pub text_content: Option<String>,    // Always populated
       pub extracted_urls: Vec<Url>,        // Always populated
       // ...
   }
   ```
   - No way to get "crawl-only" results
   - Cannot disable extraction step

2. **process_request always extracts**
   ```rust
   // File: riptide-spider/src/core.rs:525-531
   let extracted_urls = self.extract_urls(&content, &request.url).await?;
   let text_content = self.extract_text_content(&content).await;

   result.extracted_urls = extracted_urls;
   result.text_content = text_content;
   ```
   - Hardcoded extraction calls in crawl loop
   - No configuration to skip extraction

3. **Adaptive stopping depends on text content**
   ```rust
   // File: riptide-spider/src/types.rs:154-165
   pub fn unique_text_chars(&self) -> usize {
       self.text_content
           .as_ref()
           .map(|text| {
               let mut chars: Vec<char> = text.chars().collect();
               chars.sort_unstable();
               chars.dedup();
               chars.len()
           })
           .unwrap_or(0)
   }
   ```
   - Spider's adaptive stopping **requires** text extraction
   - Cannot work with raw HTML or skip extraction

#### What's Missing?

```rust
// DESIRED: Spider should allow extraction-free crawling
pub struct SpiderConfig {
    // Should have:
    pub extractor: Option<Box<dyn ContentExtractor>>,  // ← Missing!
    pub extract_on_crawl: bool,                        // ← Missing!
    // ...
}

// DESIRED: CrawlResult should separate crawl from extraction
pub struct CrawlResult {
    pub request: CrawlRequest,
    pub raw_content: Option<String>,       // Raw HTML
    pub extraction: Option<ExtractionData>, // Separate extraction results
    // ...
}
```

### 2.2 Can You Use Extraction Without Spider?

**Answer: ⚠️ PARTIALLY - Extraction works standalone but has spider-related code**

#### What Works?

```rust
// File: riptide-facade/src/facades/extractor.rs:192-278
let result = facade.extract_html(html, url, options).await?;
```
- ExtractionFacade can extract from pre-fetched HTML ✅
- Multiple extraction strategies available ✅
- Schema-based extraction works ✅

#### What's Problematic?

1. **Disabled spider module creates confusion**
   ```rust
   // File: riptide-extraction/src/lib.rs:58
   // pub mod spider;  // Temporarily disabled due to compilation errors
   ```
   - Extraction crate **wants to include spider logic**
   - Architectural boundary unclear

2. **Link extraction duplicated across both crates**
   ```rust
   // Spider has: extract_links_basic() in core.rs:3-17
   // Extraction has: enhanced_link_extraction.rs, spider/link_extractor.rs
   ```
   - Functionality scattered
   - No single source of truth

3. **Extraction strategies assume web context**
   ```rust
   // File: riptide-extraction/src/extraction_strategies.rs
   pub fn extract_links_basic(content: &str, base_url: &Url) -> Result<Vec<Url>>
   ```
   - Extraction functions take `base_url` parameter
   - Assumes web crawling context

### 2.3 Can You Compose Them Flexibly?

**Answer: ❌ NO - No composition interface exists**

#### Missing Composition Patterns

1. **No extractor plugin interface**
   ```rust
   // DESIRED but MISSING:
   pub trait ContentExtractor {
       async fn extract_links(&self, html: &str, base_url: &Url) -> Result<Vec<Url>>;
       async fn extract_text(&self, html: &str) -> Result<String>;
       async fn extract_metadata(&self, html: &str) -> Result<Metadata>;
   }

   impl Spider {
       pub fn with_extractor(mut self, extractor: Box<dyn ContentExtractor>) -> Self {
           self.extractor = Some(extractor);
           self
       }
   }
   ```

2. **No pipeline composition**
   ```rust
   // DESIRED but MISSING:
   let results = spider
       .crawl(seeds)
       .pipe(|crawl_result| extractor.extract(crawl_result))
       .pipe(|extracted| transformer.transform(extracted))
       .collect()
       .await?;
   ```

3. **No lazy evaluation**
   - Spider always extracts immediately
   - Cannot collect URLs then extract later in batch

---

## 3. Shared State Creating Coupling

### 3.1 Implicit Assumptions About Data Flow

**Spider assumes:**
1. HTML content will be available in memory
2. Extraction happens synchronously during crawl
3. Extracted URLs feed back into frontier immediately
4. Text content is needed for adaptive stopping

**Extraction assumes:**
1. Content comes from HTTP responses
2. Base URL is available for link resolution
3. HTML is well-formed enough for DOM parsing

### 3.2 No Clear Boundaries

```
Current (Coupled):
┌─────────────────────────────────────┐
│         Spider                      │
│  ┌──────────────────────────────┐  │
│  │  fetch()                     │  │
│  │    ↓                         │  │
│  │  extract_links_basic()  ←────┼──┼── Embedded extraction!
│  │    ↓                         │  │
│  │  extract_text()         ←────┼──┼── More embedded extraction!
│  │    ↓                         │  │
│  │  frontier.add(urls)          │  │
│  └──────────────────────────────┘  │
└─────────────────────────────────────┘

Desired (Decoupled):
┌──────────────┐      ┌──────────────────┐
│    Spider    │      │   Extraction     │
│              │      │                  │
│  fetch()     │──────┤  extract()       │
│  store_raw() │      │    - links       │
│              │      │    - text        │
│              │      │    - metadata    │
└──────────────┘      └──────────────────┘
       │                      │
       └──────────────────────┘
              Composed via interface
```

### 3.3 Type System Enforces Coupling

```rust
// CrawlResult MUST have extraction data
pub struct CrawlResult {
    pub extracted_urls: Vec<Url>,  // Required, not Optional
    pub text_content: Option<String>,
    // ...
}

// CrawlRequest includes extraction hints
pub struct CrawlRequest {
    pub metadata: HashMap<String, String>,  // Used for extraction config
    // ...
}
```

---

## 4. Code Examples of Tight Coupling

### Example 1: Spider's process_request Method

**File:** `riptide-spider/src/core.rs:443-560`

```rust
async fn process_request(&self, request: CrawlRequest) -> Result<CrawlResult> {
    // 1. Fetch (good - independent)
    let fetch_result = /* ... fetch logic ... */;

    // 2. Extract (BAD - embedded coupling)
    let (success, content_size, error) = match fetch_result {
        Ok((content, size)) => {
            // COUPLING: Spider directly performs extraction
            let extracted_urls = self.extract_urls(&content, &request.url).await?;
            let text_content = self.extract_text_content(&content).await;

            let mut result = CrawlResult::success(request.clone());
            result.content_size = size;
            result.text_content = text_content;        // ← Extraction result
            result.extracted_urls = extracted_urls;    // ← Extraction result
            result.processing_time = start_time.elapsed();

            return Ok(result);
        }
        Err(e) => (false, 0, Some(e.to_string()))
    };

    // ...
}
```

**Problem:** Cannot crawl without extracting. No separation of concerns.

### Example 2: Duplicate Link Extraction

**Spider version:** `riptide-spider/src/core.rs:3-17`
```rust
fn extract_links_basic(content: &str, base_url: &Url) -> Result<Vec<Url>> {
    let link_regex = regex::Regex::new(r#"href\s*=\s*["']([^"']+)["']"#)?;
    let mut links = Vec::new();

    for cap in link_regex.captures_iter(content) {
        if let Some(link_str) = cap.get(1) {
            if let Ok(url) = base_url.join(link_str.as_str()) {
                links.push(url);
            }
        }
    }

    Ok(links)
}
```

**Extraction version:** `riptide-extraction/src/extraction_strategies.rs`
```rust
pub fn extract_links_basic(content: &str, base_url: &Url) -> Result<Vec<Url>> {
    // Similar implementation
}
```

**Problem:** Same functionality in two places. No code reuse.

### Example 3: Facade Pipeline Still Coupled

**File:** `riptide-facade/src/facades/pipeline.rs:144-159`

```rust
let output = match stage {
    PipelineStage::Fetch { url, options } => {
        self.execute_fetch(url, options, context).await?
    }
    PipelineStage::Extract { strategy } => {
        // COUPLING: Extract stage assumes Fetch output format
        self.execute_extract(strategy, context).await?
    }
    // ...
};
```

**Problem:** Stages make assumptions about previous stage output. Not truly composable.

---

## 5. Recommended Decoupling Strategies

### 5.1 Strategy 1: Extract-as-Plugin Pattern

**Effort:** 🟡 Medium (2-3 days)

**Approach:**
1. Define `ContentExtractor` trait in `riptide-types`
2. Move all extraction logic from spider to extraction crate
3. Spider accepts optional `Arc<dyn ContentExtractor>`
4. If no extractor provided, spider returns raw HTML

**Implementation:**

```rust
// In riptide-types/src/extractors.rs
pub trait ContentExtractor: Send + Sync {
    async fn extract_links(&self, html: &str, base_url: &Url) -> Result<Vec<Url>>;
    async fn extract_text(&self, html: &str) -> Result<String>;
    async fn extract_metadata(&self, html: &str) -> Result<Metadata>;
}

// In riptide-spider/src/config.rs
pub struct SpiderConfig {
    pub extractor: Option<Arc<dyn ContentExtractor>>,
    pub extract_on_crawl: bool,  // If false, skip extraction
    // ... existing fields
}

// In riptide-spider/src/core.rs
async fn process_request(&self, request: CrawlRequest) -> Result<CrawlResult> {
    let (content, size) = self.fetch(&request).await?;

    let extraction = if self.config.extract_on_crawl {
        if let Some(extractor) = &self.extractor {
            Some(extractor.extract(&content, &request.url).await?)
        } else {
            None
        }
    } else {
        None
    };

    Ok(CrawlResult {
        raw_content: Some(content),
        extraction,
        // ...
    })
}
```

**Benefits:**
- Spider can be used without extraction ✅
- Extraction strategies pluggable ✅
- No code duplication ✅

**Drawbacks:**
- Requires trait object overhead
- Breaking change to existing API

### 5.2 Strategy 2: Separate Raw Crawl from Enrichment

**Effort:** 🟢 Low (1-2 days)

**Approach:**
1. Spider returns `RawCrawlResult` with just HTML and metadata
2. Separate `enrich()` function applies extraction
3. Pipeline can compose: `crawl → enrich → transform → store`

**Implementation:**

```rust
// In riptide-spider/src/types.rs
pub struct RawCrawlResult {
    pub url: Url,
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub content: String,
    pub content_type: String,
    pub timestamp: SystemTime,
}

pub struct EnrichedCrawlResult {
    pub raw: RawCrawlResult,
    pub extracted_links: Vec<Url>,
    pub text_content: String,
    pub metadata: Metadata,
}

// In riptide-extraction/src/enrichment.rs
pub async fn enrich(
    raw: RawCrawlResult,
    strategy: ExtractionStrategy
) -> Result<EnrichedCrawlResult> {
    // Apply extraction to raw result
}

// Usage:
let raw_results = spider.crawl(seeds).await?;
let enriched: Vec<_> = raw_results
    .into_iter()
    .map(|raw| enrich(raw, ExtractionStrategy::Html))
    .collect();
```

**Benefits:**
- Clear separation: crawl vs. extract ✅
- Can use spider without extraction ✅
- Easy to test each phase independently ✅

**Drawbacks:**
- Two-phase processing might be less efficient
- Need to store raw HTML if extracting later

### 5.3 Strategy 3: Streaming Pipeline with Lazy Extraction

**Effort:** 🔴 High (5-7 days)

**Approach:**
1. Spider emits stream of raw crawl events
2. Extraction is a stream transformer
3. Compose via async stream combinators
4. Extraction happens only when consumed

**Implementation:**

```rust
use futures::Stream;

// In riptide-spider/src/core.rs
impl Spider {
    pub fn crawl_stream(
        &self,
        seeds: Vec<Url>
    ) -> impl Stream<Item = Result<RawCrawlResult>> {
        // Emit crawl results as stream
    }
}

// In riptide-extraction/src/stream.rs
pub fn extract_stream(
    strategy: ExtractionStrategy
) -> impl FnMut(RawCrawlResult) -> Result<EnrichedCrawlResult> {
    move |raw| {
        // Transform raw to enriched
    }
}

// Usage:
use futures::StreamExt;

let results = spider
    .crawl_stream(seeds)
    .map(extract_stream(ExtractionStrategy::Html))
    .map(|enriched| transform(enriched))
    .collect::<Vec<_>>()
    .await;
```

**Benefits:**
- Maximum flexibility ✅
- Memory efficient (streaming) ✅
- Lazy extraction (only when needed) ✅
- Truly composable pipeline ✅

**Drawbacks:**
- Requires significant refactoring
- More complex API
- Harder to debug

### 5.4 Strategy 4: Event-Driven Architecture

**Effort:** 🔴 High (7-10 days)

**Approach:**
1. Spider emits events: `UrlDiscovered`, `PageFetched`, `CrawlComplete`
2. Extraction subscribes to `PageFetched` events
3. Decoupled via event bus
4. Each component can run independently

**Implementation:**

```rust
// In riptide-events (new crate?)
pub enum CrawlEvent {
    UrlDiscovered { url: Url, depth: u32 },
    PageFetched { url: Url, content: String, metadata: Metadata },
    CrawlComplete { stats: CrawlStats },
}

pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: CrawlEvent) -> Result<()>;
}

// In riptide-spider/src/core.rs
impl Spider {
    pub fn with_handler(mut self, handler: Arc<dyn EventHandler>) -> Self {
        self.handlers.push(handler);
        self
    }

    async fn emit(&self, event: CrawlEvent) {
        for handler in &self.handlers {
            let _ = handler.handle(event.clone()).await;
        }
    }
}

// In riptide-extraction/src/handlers.rs
pub struct ExtractionHandler {
    strategy: ExtractionStrategy,
}

impl EventHandler for ExtractionHandler {
    async fn handle(&self, event: CrawlEvent) -> Result<()> {
        match event {
            CrawlEvent::PageFetched { content, .. } => {
                let extracted = self.strategy.extract(&content).await?;
                // Process extracted data
            }
            _ => {}
        }
        Ok(())
    }
}

// Usage:
let spider = Spider::new(config)
    .with_handler(Arc::new(ExtractionHandler::new(strategy)))
    .with_handler(Arc::new(StorageHandler::new(db)));

spider.crawl(seeds).await?;
```

**Benefits:**
- Complete decoupling ✅
- Easy to add new processors ✅
- Spider and extraction fully independent ✅
- Can disable extraction by not registering handler ✅

**Drawbacks:**
- Most complex implementation
- Event overhead
- Harder to reason about dataflow

---

## 6. Effort Estimates

| Strategy | Complexity | Lines Changed | Breaking Change | Effort | Risk |
|----------|-----------|---------------|-----------------|--------|------|
| Extract-as-Plugin | Medium | ~500 | Yes | 2-3 days | Medium |
| Separate Raw/Enriched | Low | ~300 | Partial | 1-2 days | Low |
| Streaming Pipeline | High | ~800 | Yes | 5-7 days | High |
| Event-Driven | Very High | ~1200 | Yes | 7-10 days | High |

**Recommendation:** Start with **Strategy 2 (Separate Raw/Enriched)** for quick wins, then migrate to **Strategy 1 (Extract-as-Plugin)** for long-term flexibility.

---

## 7. Migration Path

### Phase 1: Immediate (1-2 days)
1. Create `RawCrawlResult` type in `riptide-spider/src/types.rs`
2. Add `enrich()` function in `riptide-extraction/src/enrichment.rs`
3. Update facades to support both raw and enriched workflows
4. **No breaking changes** - add new APIs alongside existing

### Phase 2: Refactor (2-3 days)
1. Define `ContentExtractor` trait in `riptide-types`
2. Implement trait for existing extraction strategies
3. Update Spider to accept optional extractor
4. Remove embedded extraction from spider core
5. **Breaking change** - deprecate old APIs, provide migration guide

### Phase 3: Optimize (3-5 days)
1. Add streaming support to spider
2. Implement lazy extraction pipeline
3. Add event handlers for advanced use cases
4. **Non-breaking** - new advanced APIs

---

## 8. Testing Strategy

### Unit Tests
```rust
#[test]
fn test_spider_without_extraction() {
    let spider = Spider::new(config_without_extractor).await?;
    let results = spider.crawl(seeds).await?;

    // Verify: no extraction performed
    for result in results {
        assert!(result.extracted_urls.is_empty());
        assert!(result.text_content.is_none());
        assert!(result.raw_content.is_some());
    }
}

#[test]
fn test_extraction_without_spider() {
    let html = "<html><body><a href='/page'>Link</a></body></html>";
    let extractor = HtmlExtractor::new();
    let result = extractor.extract(html, base_url).await?;

    assert_eq!(result.links.len(), 1);
    assert!(!result.text.is_empty());
}
```

### Integration Tests
```rust
#[test]
async fn test_pipeline_composition() {
    let raw = spider.crawl_raw(seeds).await?;
    let extracted = extract_batch(&raw, strategy).await?;
    let transformed = transform_batch(&extracted).await?;

    assert_eq!(raw.len(), extracted.len());
}
```

---

## 9. Breaking Changes Impact

### APIs that will change:
1. `Spider::crawl()` - may return different result type
2. `CrawlResult` - structure will change
3. `ExtractionFacade::extract_html()` - may accept different input

### Mitigation:
1. Keep old APIs as `_legacy` variants during migration
2. Add `#[deprecated]` warnings with migration examples
3. Provide compatibility shims for 1-2 versions
4. Comprehensive migration guide with before/after code

---

## 10. Success Criteria

### Must Have (Phase 1-2):
- ✅ Spider can crawl without extraction
- ✅ Extraction can process pre-fetched HTML
- ✅ No duplicate link extraction code
- ✅ Clear separation in type system

### Nice to Have (Phase 3):
- ✅ Streaming pipeline support
- ✅ Event-driven composition
- ✅ Plugin architecture for extractors

### Verification:
```rust
// Should compile and run:
let spider_only = Spider::new(config).crawl(seeds).await?;
let extract_only = Extractor::new().extract(html, url).await?;
let composed = spider.crawl(seeds)
    .then(|raw| extractor.extract(raw))
    .collect()
    .await?;
```

---

## Conclusion

The RipTide codebase has **significant coupling** between spider and extraction that prevents truly modular usage. The root causes are:

1. **Embedded extraction logic in spider core** (lines 3-17, 608-647 in core.rs)
2. **Shared types that assume extraction** (CrawlResult in types.rs)
3. **Disabled spider module in extraction crate** (circular dependency attempt)
4. **No plugin/composition interfaces** (missing trait abstractions)

**Recommended immediate action:**
Implement **Strategy 2 (Separate Raw/Enriched)** as a non-breaking addition, then migrate to **Strategy 1 (Extract-as-Plugin)** for long-term modularity.

This will enable:
- Independent spider usage (crawl-only workflows)
- Independent extraction usage (process pre-fetched content)
- Flexible composition (pipeline, streaming, event-driven)
- Better testability and maintainability
