# Facade Layer & Orchestration Pattern Analysis

**Date:** 2025-11-04
**Objective:** Understand current orchestration patterns to plan pipeline enhancement
**Focus:** `/crates/riptide-facade` architecture and integration points

---

## Executive Summary

The Riptide facade layer provides a **high-level, user-friendly API** for web scraping operations with a well-structured builder pattern. However, there is a **significant gap between the facade's simple pipeline and the API's sophisticated `PipelineOrchestrator`**. The facade's `PipelineFacade` is a simplified abstraction that **does not leverage** the complete fetch→gate→extract workflow, provenance tracking, or streaming capabilities available in lower layers.

### Key Findings

| Aspect | Current State | Gap Identified |
|--------|--------------|----------------|
| **Orchestration** | Simple stage-based pipeline | No integration with `PipelineOrchestrator` |
| **Strategy Execution** | Placeholder implementations | No `StrategiesPipelineOrchestrator` support |
| **Provenance Tracking** | ❌ Not implemented | Missing event/metadata tracking |
| **Streaming Support** | ❌ Not implemented | No async stream patterns |
| **Service Integration** | Direct `riptide-fetch` calls | Limited use of `riptide-api` orchestrators |
| **Builder Pattern** | ✅ Well-implemented | Strong foundation for enhancement |

---

## 1. Current Facade Architecture

### 1.1 Module Structure

```
riptide-facade/
├── src/
│   ├── lib.rs              (Entry point - 72 lines)
│   ├── builder.rs          (Builder pattern - 309 lines)
│   ├── config.rs           (Configuration - 189 lines)
│   ├── error.rs            (Error types - 1,406 lines)
│   ├── prelude.rs          (Re-exports - 335 lines)
│   ├── runtime.rs          (Runtime helpers - 1,892 lines)
│   ├── facades/
│   │   ├── mod.rs          (Facade exports - 25 lines)
│   │   ├── pipeline.rs     (Pipeline facade - 779 lines) ⚠️
│   │   ├── scraper.rs      (Basic scraper - 100 lines)
│   │   ├── browser.rs      (Browser automation - 1,109 lines)
│   │   ├── extractor.rs    (Content extraction - 24,725 lines)
│   │   ├── spider.rs       (Crawler - 9,480 lines)
│   │   ├── search.rs       (Search engine - 12,810 lines)
│   │   └── intelligence.rs (AI integration - 772 lines)
│   ├── adapters/           (Empty placeholder)
│   ├── composition/        (Empty placeholder)
│   └── traits/             (Empty placeholder)
```

**Total Code:** ~4,167 lines

### 1.2 Facade Entry Point

```rust
// lib.rs - Main entry
pub struct Riptide;

impl Riptide {
    pub fn builder() -> RiptideBuilder {
        RiptideBuilder::new()
    }
}
```

**Pattern:** Simple static method returning builder instance.

### 1.3 Builder Pattern Analysis

#### RiptideBuilder (Primary)

```rust
pub struct RiptideBuilder {
    config: RiptideConfig,  // Shared configuration
}

impl RiptideBuilder {
    // Configuration methods (fluent API)
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self
    pub fn timeout_secs(mut self, secs: u64) -> Self
    pub fn timeout(mut self, timeout: Duration) -> Self
    pub fn max_redirects(mut self, max_redirects: u32) -> Self
    pub fn verify_ssl(mut self, verify: bool) -> Self
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    pub fn max_body_size(mut self, size: usize) -> Self
    pub fn config(mut self, config: RiptideConfig) -> Self

    // Terminal methods (create facades)
    pub async fn build_scraper(self) -> RiptideResult<ScraperFacade>
    pub async fn build_browser(self) -> RiptideResult<BrowserFacade>
    pub async fn build_extractor(self) -> RiptideResult<ExtractionFacade>
}
```

**Strengths:**
- ✅ Clean fluent interface
- ✅ Type-safe configuration
- ✅ Proper validation before build
- ✅ Async initialization support

**Limitations:**
- ⚠️ No `build_pipeline()` method
- ⚠️ No strategy configuration
- ⚠️ Limited to basic HTTP settings

---

## 2. Pipeline Facade Deep Dive

### 2.1 PipelineFacade Structure

```rust
pub struct PipelineFacade {
    config: Arc<RiptideConfig>,           // Shared config
    cache: Arc<RwLock<PipelineCache>>,    // Simple in-memory cache
}

impl PipelineFacade {
    pub async fn new(config: RiptideConfig) -> RiptideResult<Self>
    pub fn builder(&self) -> PipelineBuilder
    pub async fn execute(&self, pipeline: Pipeline) -> RiptideResult<PipelineResult>

    // Pre-built pipeline templates
    pub async fn web_scraping_pipeline(&self, url: &str) -> RiptideResult<Pipeline>
    pub async fn pdf_extraction_pipeline(&self, url: &str) -> RiptideResult<Pipeline>
    pub async fn browser_automation_pipeline(&self, url: &str, ...) -> RiptideResult<Pipeline>
}
```

### 2.2 Pipeline Stage Types

```rust
pub enum PipelineStage {
    Fetch { url: String, options: FetchOptions },
    Extract { strategy: ExtractionStrategy },
    Transform { transformer: Arc<dyn Transformer> },
    Validate { validator: Arc<dyn Validator> },
    Store { destination: StoreDestination },
}
```

**Stage Execution Flow:**

```
┌─────────────────────────────────────────────────────────────┐
│              Current PipelineFacade Flow                    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. Cache Check (PipelineCache)                            │
│     └─> In-memory HashMap<String, Value>                   │
│                                                              │
│  2. Execute Stages (Sequential or Parallel)                │
│     ├─> Fetch: Placeholder (returns mock JSON)             │
│     ├─> Extract: Placeholder (echoes input)                │
│     ├─> Transform: Custom trait implementation             │
│     ├─> Validate: Custom trait implementation              │
│     └─> Store: Placeholder (echoes input)                  │
│                                                              │
│  3. Retry Logic (execute_stage_with_retry)                 │
│     └─> Exponential backoff (100ms * 2^attempt)            │
│                                                              │
│  4. Cache Store (PipelineCache)                            │
│     └─> Simple set with no TTL                             │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 2.3 Critical Limitation: Placeholder Implementations

```rust
// ALL STAGE EXECUTORS ARE PLACEHOLDERS!

async fn execute_fetch(
    &self,
    url: &str,
    _options: &FetchOptions,
    _context: &PipelineContext,
) -> RiptideResult<serde_json::Value> {
    // ⚠️ RETURNS MOCK DATA, NOT REAL FETCH
    Ok(serde_json::json!({
        "url": url,
        "content": format!("Fetched content from {}", url),
        "timestamp": SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
    }))
}

async fn execute_extract(
    &self,
    strategy: &ExtractionStrategy,
    context: &PipelineContext,
) -> RiptideResult<serde_json::Value> {
    let input = context.get_output();
    // ⚠️ JUST ECHOES INPUT WITH STRATEGY NAME
    Ok(serde_json::json!({
        "strategy": format!("{:?}", strategy),
        "extracted": input,
    }))
}
```

**Problem:** The facade pipeline is a **demonstration/template**, not a production-ready orchestrator.

---

## 3. API Layer Orchestrators (Not Used by Facade)

### 3.1 PipelineOrchestrator (riptide-api/pipeline.rs)

**Sophisticated production orchestrator - 1,072 lines**

```rust
pub struct PipelineOrchestrator {
    state: AppState,
    options: CrawlOptions,
    retry_config: PipelineRetryConfig,
}

impl PipelineOrchestrator {
    // COMPLETE FETCH→GATE→EXTRACT WORKFLOW
    pub async fn execute_single(&self, url: &str) -> ApiResult<PipelineResult>
    pub async fn execute_batch(&self, urls: &[String]) -> (Vec<Option<PipelineResult>>, PipelineStats)

    // REAL IMPLEMENTATIONS
    async fn fetch_content_with_type(&self, url: &str) -> ApiResult<(Response, Vec<u8>, Option<String>)>
    async fn process_pdf_content(&self, pdf_bytes: &[u8], url: &str) -> ApiResult<ExtractedDoc>
    async fn analyze_content(&self, html: &str, url: &str) -> ApiResult<GateFeatures>
    async fn extract_content(&self, html: &str, url: &str, decision: Decision) -> ApiResult<ExtractedDoc>
    async fn check_cache(&self, cache_key: &str) -> ApiResult<Option<ExtractedDoc>>
    async fn store_in_cache(&self, cache_key: &str, document: &ExtractedDoc) -> ApiResult<()>
}
```

**Complete Pipeline Flow:**

```
┌─────────────────────────────────────────────────────────────┐
│          API PipelineOrchestrator (PRODUCTION)              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. Event Emission                                          │
│     └─> pipeline.execution.started                         │
│                                                              │
│  2. Cache Check (Redis)                                     │
│     └─> Deterministic key: riptide:v1:{mode}:{hash}        │
│                                                              │
│  3. Fetch Content (riptide-fetch)                          │
│     ├─> HTTP/HTTPS via reqwest                             │
│     ├─> Smart retry (Adaptive/Exponential/Linear)          │
│     ├─> Content-Type detection                             │
│     └─> Timeout enforcement (15s)                          │
│                                                              │
│  4. PDF Detection & Processing                              │
│     ├─> MIME type + magic bytes check                      │
│     ├─> Resource management (PDF semaphore)                │
│     ├─> Extraction via riptide-pdf                         │
│     └─> Metrics: pages, memory, duration                   │
│                                                              │
│  5. Gate Analysis (riptide-reliability)                     │
│     ├─> Feature extraction (HTML size, scripts, metadata)  │
│     ├─> Quality scoring (0.0-1.0)                          │
│     ├─> Decision: Raw / ProbesFirst / Headless             │
│     └─> Metrics: decision distribution, timings            │
│                                                              │
│  6. Content Extraction (riptide-extraction)                 │
│     ├─> UnifiedExtractor (native/WASM)                     │
│     ├─> ExtractedContent → ExtractedDoc conversion         │
│     ├─> Quality score, metadata, links                     │
│     └─> Event: pipeline.extraction.success                 │
│                                                              │
│  7. Cache Store (Redis)                                     │
│     └─> TTL-based expiration                               │
│                                                              │
│  8. Event Emission                                          │
│     └─> pipeline.execution.completed                       │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

**Features NOT in PipelineFacade:**
- ❌ Event-driven architecture (riptide-events)
- ❌ Gate decision making (riptide-reliability)
- ❌ Smart retry strategies (riptide-intelligence)
- ❌ Resource management (PDF semaphore)
- ❌ Metrics collection (riptide-performance)
- ❌ Redis caching (riptide-cache)
- ❌ Headless browser fallback (riptide-browser)

### 3.2 StrategiesPipelineOrchestrator (riptide-api/strategies_pipeline.rs)

**Advanced orchestrator with strategy patterns - 526 lines**

```rust
pub struct StrategiesPipelineOrchestrator {
    state: AppState,
    options: CrawlOptions,
    strategy_config: StrategyConfig,  // ⭐ Strategy configuration
}

impl StrategiesPipelineOrchestrator {
    pub fn new(state: AppState, options: CrawlOptions, strategy_config: Option<StrategyConfig>) -> Self
    pub fn with_auto_strategy(state: AppState, options: CrawlOptions, url: &str) -> Self

    pub async fn execute_single(&self, url: &str) -> ApiResult<StrategiesPipelineResult>
    async fn process_pdf_pipeline(...) -> ApiResult<StrategiesPipelineResult>

    fn auto_detect_strategy(url: &str, options: &CrawlOptions) -> StrategyConfig
}
```

**Strategy Configuration:**

```rust
pub struct StrategyConfig {
    pub extraction: ExtractionStrategyType,  // Wasm, CssJson, Regex, Llm
    pub enable_metrics: bool,
    pub enable_chunking: bool,
    // Note: Chunking handled by riptide-extraction crate
}

pub enum ExtractionStrategyType {
    Wasm,      // WASM-based extraction (default)
    CssJson,   // CSS selector + JSON path
    Regex,     // Regex pattern matching
    Llm,       // LLM-powered extraction
}
```

**Enhanced Result:**

```rust
pub struct StrategiesPipelineResult {
    pub processed_content: ProcessedContent,  // From StrategyManager
    pub from_cache: bool,
    pub gate_decision: String,
    pub quality_score: f32,
    pub processing_time_ms: u64,
    pub cache_key: String,
    pub http_status: u16,
    pub strategy_config: StrategyConfig,      // ⭐ Strategy used
    pub performance_metrics: Option<PerformanceMetrics>,  // ⭐ Metrics
}
```

**Features NOT in PipelineFacade:**
- ❌ Strategy selection (manual or auto-detect)
- ❌ Multiple extraction strategies
- ❌ Performance metrics collection
- ❌ ProcessedContent output format
- ❌ Chunking configuration

---

## 4. Service Layer Integration Points

### 4.1 Current Facade Dependencies

```toml
# From Cargo.toml
riptide-types         # Shared types (ExtractedDoc, RenderMode, etc.)
riptide-fetch         # HTTP fetching (FetchEngine)
riptide-extraction    # Content extraction (UnifiedExtractor)
riptide-pdf           # PDF processing
riptide-cache         # Caching (not used in PipelineFacade)
riptide-browser       # Headless browsing
riptide-stealth       # Anti-detection
riptide-monitoring    # Metrics (optional)
riptide-spider        # Crawling
riptide-search        # Search engines
```

### 4.2 Service Crates NOT Used by Facade

```
riptide-api              ⚠️ Contains PipelineOrchestrator/StrategiesPipelineOrchestrator
riptide-reliability      ⚠️ Gate decision making, circuit breakers
riptide-intelligence     ⚠️ Smart retry strategies, ML features
riptide-events           ⚠️ Event bus, provenance tracking
riptide-performance      ⚠️ Metrics collection, bottleneck analysis
riptide-streaming        ⚠️ Async streams, backpressure, NDJSON
riptide-persistence      ⚠️ Database integration
riptide-workers          ⚠️ Background job processing
riptide-pool             ⚠️ Resource pooling
```

### 4.3 Integration Gap Analysis

| Service | Facade Usage | API Usage | Gap |
|---------|-------------|-----------|-----|
| **riptide-fetch** | Direct calls via `FetchEngine` | Wrapped in `PipelineOrchestrator` | Facade lacks timeout, retry, error handling |
| **riptide-extraction** | Imported but unused in pipeline | Core of extraction phase | Pipeline uses placeholder instead of real extractor |
| **riptide-cache** | Imported but uses in-memory HashMap | Redis with TTL, key builder | No distributed caching |
| **riptide-reliability** | ❌ Not used | Gate analysis, circuit breakers | No decision-making logic |
| **riptide-events** | ❌ Not used | Complete provenance tracking | No event emission |
| **riptide-streaming** | ❌ Not used | NDJSON, async streams | No streaming support |

---

## 5. Provenance Tracking Analysis

### 5.1 Current State: ❌ NO PROVENANCE TRACKING

**Search Results:**
```bash
$ grep -r "provenance\|Provenance" /workspaces/eventmesh/crates/riptide-facade/src/
No provenance tracking found
```

**Facade has ZERO event emission or metadata tracking.**

### 5.2 API Provenance Implementation

The `PipelineOrchestrator` implements **comprehensive event-driven provenance**:

```rust
// Event emission at key pipeline stages
let mut start_event = BaseEvent::new(
    "pipeline.execution.started",
    "pipeline_orchestrator",
    EventSeverity::Info,
);
start_event.add_metadata("url", url);
start_event.add_metadata("cache_key", &cache_key);
self.state.event_bus.emit(start_event).await?;

// Cache hit event
let mut cache_event = BaseEvent::new(
    "pipeline.cache.hit",
    "pipeline_orchestrator",
    EventSeverity::Info,
);
cache_event.add_metadata("url", url);
cache_event.add_metadata("cache_key", &cache_key);
self.state.event_bus.emit(cache_event).await?;

// PDF processing event
let mut pdf_event = BaseEvent::new(
    "pipeline.pdf.processing",
    "pipeline_orchestrator",
    EventSeverity::Info,
);
pdf_event.add_metadata("url", url);
pdf_event.add_metadata("content_size", &content_bytes.len().to_string());

// Gate decision event
let mut gate_event = BaseEvent::new(
    "pipeline.gate.decision",
    "pipeline_orchestrator",
    EventSeverity::Info,
);
gate_event.add_metadata("decision", &gate_decision_str);
gate_event.add_metadata("quality_score", &quality_score.to_string());

// Extraction success event
let mut event = BaseEvent::new(
    "pipeline.extraction.success",
    "pipeline_orchestrator",
    EventSeverity::Info,
);
event.add_metadata("strategy", strategy);
event.add_metadata("content_length", &extracted_content.content.len().to_string());

// Pipeline completion event
let mut completion_event = BaseEvent::new(
    "pipeline.execution.completed",
    "pipeline_orchestrator",
    EventSeverity::Info,
);
completion_event.add_metadata("processing_time_ms", &processing_time_ms.to_string());
completion_event.add_metadata("http_status", &http_status.to_string());
```

**Event Types:**
- `pipeline.execution.started`
- `pipeline.cache.hit`
- `pipeline.pdf.processing`
- `pipeline.gate.decision`
- `pipeline.extraction.success`
- `pipeline.execution.completed`

**Metadata Tracked:**
- URL, cache key, processing time
- Gate decision, quality score
- HTTP status, content size
- Extraction strategy used
- Content length, timestamps

### 5.3 Provenance Gap

**What Facade Needs:**

1. **Event Bus Integration**
   - Import `riptide-events` crate
   - Create `EventBus` instance in `PipelineFacade`
   - Emit events at stage boundaries

2. **Metadata Collection**
   - Track stage execution times
   - Record strategy decisions
   - Capture error information
   - Store cache hit/miss ratios

3. **Event Schema**
   - Define facade-specific event types
   - Standard metadata fields
   - Severity levels (Info, Warn, Error)

---

## 6. Streaming Pattern Analysis

### 6.1 Current State: ❌ NO STREAMING SUPPORT

**Search Results:**
```bash
$ grep -r "stream\|Stream" /workspaces/eventmesh/crates/riptide-facade/src/
(No async stream patterns found)
```

**Facade is entirely synchronous batch processing.**

### 6.2 Streaming Capabilities Available (riptide-streaming)

```
riptide-streaming/src/
├── lib.rs              (8,633 lines)
├── ndjson.rs           (25,408 lines) - NDJSON streaming
├── progress.rs         (16,177 lines) - Progress tracking
├── backpressure.rs     (21,484 lines) - Flow control
├── config.rs           (21,268 lines) - Stream config
├── reports.rs          (37,406 lines) - Report generation
└── server.rs           (8,411 lines)  - Streaming server
```

**Key Features:**

1. **NDJSON Streaming**
   ```rust
   pub struct NdjsonStreamer {
       config: StreamConfig,
       pipeline: Arc<PipelineOrchestrator>,
   }

   impl NdjsonStreamer {
       pub async fn stream_urls(&self, urls: Vec<String>) -> impl Stream<Item = NdjsonResult>
   }
   ```

2. **Backpressure Management**
   ```rust
   pub struct BackpressureController {
       max_in_flight: usize,
       current_in_flight: AtomicUsize,
       semaphore: Semaphore,
   }
   ```

3. **Progress Tracking**
   ```rust
   pub struct ProgressTracker {
       total: usize,
       completed: AtomicUsize,
       failed: AtomicUsize,
       rate_limiter: RateLimiter,
   }
   ```

### 6.3 Streaming Gap

**What Facade Needs:**

1. **Async Stream Traits**
   ```rust
   use futures::Stream;

   pub trait StreamablePipeline {
       type Item;
       type Error;

       fn execute_stream(&self, urls: Vec<String>)
           -> impl Stream<Item = Result<Self::Item, Self::Error>>;
   }
   ```

2. **Streaming Pipeline Method**
   ```rust
   impl PipelineFacade {
       pub async fn execute_stream(
           &self,
           pipeline: Pipeline,
           inputs: Vec<String>,
       ) -> impl Stream<Item = RiptideResult<PipelineResult>> {
           // Stream results as they complete
       }
   }
   ```

3. **Progress Callbacks**
   ```rust
   pub struct StreamConfig {
       pub on_progress: Box<dyn Fn(usize, usize) + Send + Sync>,
       pub on_error: Box<dyn Fn(&RiptideError) + Send + Sync>,
       pub backpressure_limit: usize,
   }
   ```

---

## 7. Builder Pattern Enhancement Opportunities

### 7.1 Current Builder Capabilities

```rust
RiptideBuilder
    .user_agent("Bot/1.0")
    .timeout_secs(30)
    .max_redirects(5)
    .verify_ssl(true)
    .header("X-Custom", "value")
    .max_body_size(10_000_000)
    .build_scraper()  // ✅
    .build_browser()  // ✅
    .build_extractor() // ✅
```

### 7.2 Missing Builder Methods

```rust
// ❌ NOT AVAILABLE - Needed for pipeline
RiptideBuilder
    .retry_strategy(RetryStrategy::Exponential)
    .cache_ttl(Duration::from_secs(3600))
    .extraction_strategy(ExtractionStrategyType::Wasm)
    .enable_gate_analysis(true)
    .enable_provenance_tracking(true)
    .enable_streaming(true)
    .event_bus(event_bus)
    .metrics_collector(metrics)
    .build_pipeline()  // ❌ DOES NOT EXIST
```

### 7.3 Proposed PipelineBuilder Enhancement

```rust
// New builder for advanced pipelines
pub struct AdvancedPipelineBuilder {
    config: RiptideConfig,
    retry_config: Option<RetryConfig>,
    strategy_config: Option<StrategyConfig>,
    cache_config: Option<CacheConfig>,
    event_bus: Option<Arc<EventBus>>,
    metrics: Option<Arc<MetricsCollector>>,
    streaming_config: Option<StreamConfig>,
}

impl AdvancedPipelineBuilder {
    pub fn with_retry(mut self, config: RetryConfig) -> Self
    pub fn with_strategy(mut self, config: StrategyConfig) -> Self
    pub fn with_cache(mut self, config: CacheConfig) -> Self
    pub fn with_events(mut self, bus: Arc<EventBus>) -> Self
    pub fn with_metrics(mut self, collector: Arc<MetricsCollector>) -> Self
    pub fn with_streaming(mut self, config: StreamConfig) -> Self

    // Terminal methods
    pub async fn build_basic_pipeline(self) -> RiptideResult<PipelineFacade>
    pub async fn build_orchestrated_pipeline(self) -> RiptideResult<OrchestrationFacade>
    pub async fn build_strategies_pipeline(self) -> RiptideResult<StrategiesFacade>
}
```

---

## 8. Architectural Gaps Summary

### 8.1 Critical Gaps (High Priority)

| # | Gap | Impact | Effort |
|---|-----|--------|--------|
| 1 | **No PipelineOrchestrator integration** | Facade cannot execute production-ready pipelines | High |
| 2 | **No provenance tracking** | Zero visibility into pipeline execution | Medium |
| 3 | **No gate analysis** | Cannot make intelligent extraction decisions | Medium |
| 4 | **Placeholder stage implementations** | Pipeline is demo-only, not functional | High |
| 5 | **No streaming support** | Cannot handle large-scale batch processing | Medium |

### 8.2 Enhancement Opportunities (Medium Priority)

| # | Opportunity | Benefit | Effort |
|---|-------------|---------|--------|
| 6 | **StrategiesPipelineOrchestrator wrapper** | Multiple extraction strategies | Medium |
| 7 | **Redis cache integration** | Distributed caching with TTL | Low |
| 8 | **Smart retry integration** | Adaptive error recovery | Low |
| 9 | **Resource management** | PDF/browser resource pooling | Medium |
| 10 | **Metrics collection** | Performance monitoring | Low |

### 8.3 Advanced Features (Lower Priority)

| # | Feature | Value | Effort |
|---|---------|-------|--------|
| 11 | **Event bus abstraction** | Pluggable event backends | Medium |
| 12 | **Custom strategy plugins** | Extensible extraction | High |
| 13 | **Circuit breaker patterns** | Fault tolerance | Medium |
| 14 | **Distributed tracing** | End-to-end observability | High |
| 15 | **GraphQL pipeline API** | Alternative query interface | High |

---

## 9. Service Integration Analysis

### 9.1 Direct Service Calls (Current)

```
Facade Layer
    ↓ (direct call)
riptide-fetch::FetchEngine
    ↓
HTTP Request
```

**Problems:**
- No retry logic
- No timeout enforcement
- No error transformation
- No metrics collection

### 9.2 Orchestrator Integration (Needed)

```
Facade Layer
    ↓ (delegates to)
OrchestrationFacade
    ↓ (uses)
PipelineOrchestrator / StrategiesPipelineOrchestrator
    ↓ (coordinates)
├─> riptide-fetch (with retry)
├─> riptide-reliability (gate analysis)
├─> riptide-extraction (strategies)
├─> riptide-cache (Redis)
├─> riptide-events (provenance)
├─> riptide-performance (metrics)
└─> riptide-browser (headless fallback)
```

**Benefits:**
- ✅ Complete error handling
- ✅ Smart retry strategies
- ✅ Gate-based decision making
- ✅ Provenance tracking
- ✅ Performance metrics
- ✅ Production-ready

---

## 10. Recommendations

### 10.1 Immediate Actions (Week 1)

1. **Create OrchestrationFacade wrapper**
   ```rust
   pub struct OrchestrationFacade {
       orchestrator: Arc<PipelineOrchestrator>,
   }

   impl OrchestrationFacade {
       pub async fn new(config: RiptideConfig, app_state: AppState) -> RiptideResult<Self>
       pub async fn run_pipeline(&self, url: &str) -> RiptideResult<PipelineResult>
       pub async fn run_batch(&self, urls: Vec<String>) -> RiptideResult<BatchResult>
   }
   ```

2. **Add builder method**
   ```rust
   impl RiptideBuilder {
       pub async fn build_orchestrated_pipeline(
           self,
           app_state: AppState,
       ) -> RiptideResult<OrchestrationFacade> {
           OrchestrationFacade::new(self.config, app_state).await
       }
   }
   ```

### 10.2 Short-Term Enhancements (Weeks 2-4)

3. **Integrate StrategiesPipelineOrchestrator**
   ```rust
   pub struct StrategiesFacade {
       orchestrator: Arc<StrategiesPipelineOrchestrator>,
   }

   impl StrategiesFacade {
       pub async fn run_with_strategy(
           &self,
           url: &str,
           strategy: ExtractionStrategyType,
       ) -> RiptideResult<ProcessedContent>
   }
   ```

4. **Add provenance tracking**
   - Import `riptide-events`
   - Emit events at stage boundaries
   - Provide event access via facade methods

5. **Add streaming support**
   - Import `riptide-streaming`
   - Implement `execute_stream()` methods
   - Add progress callbacks

### 10.3 Long-Term Architecture (Months 2-3)

6. **Builder enhancement**
   - Add `AdvancedPipelineBuilder`
   - Strategy configuration
   - Cache configuration
   - Event bus integration

7. **Facade trait abstraction**
   ```rust
   pub trait PipelineExecutor {
       type Result;
       type Error;

       async fn execute_single(&self, url: &str) -> Result<Self::Result, Self::Error>;
       async fn execute_batch(&self, urls: Vec<String>) -> Vec<Result<Self::Result, Self::Error>>;
       async fn execute_stream(&self, urls: Vec<String>) -> impl Stream<Item = Result<Self::Result, Self::Error>>;
   }
   ```

8. **Complete integration**
   - All service crates accessible via facade
   - Unified error handling
   - Comprehensive metrics
   - Full provenance tracking

---

## 11. Implementation Roadmap

### Phase 1: Foundation (Week 1)
- [ ] Create `OrchestrationFacade` wrapper
- [ ] Add `build_orchestrated_pipeline()` to `RiptideBuilder`
- [ ] Implement basic `run_pipeline()` method
- [ ] Add integration tests

### Phase 2: Strategies (Week 2)
- [ ] Create `StrategiesFacade` wrapper
- [ ] Add strategy configuration to builder
- [ ] Implement `run_with_strategy()` method
- [ ] Add strategy selection logic

### Phase 3: Provenance (Week 3)
- [ ] Integrate `riptide-events`
- [ ] Add event emission to facades
- [ ] Implement event retrieval methods
- [ ] Add event filtering/querying

### Phase 4: Streaming (Week 4)
- [ ] Integrate `riptide-streaming`
- [ ] Implement `execute_stream()` methods
- [ ] Add backpressure control
- [ ] Add progress tracking

### Phase 5: Polish (Week 5-6)
- [ ] Complete documentation
- [ ] Add comprehensive examples
- [ ] Performance benchmarks
- [ ] Production testing

---

## 12. Code Examples

### 12.1 Current Usage (Limited)

```rust
// Current facade - simplified but limited
let config = RiptideConfig::default();
let pipeline_facade = PipelineFacade::new(config).await?;

let pipeline = pipeline_facade
    .builder()
    .add_stage(PipelineStage::Fetch { url: "https://example.com".to_string(), options: Default::default() })
    .add_stage(PipelineStage::Extract { strategy: ExtractionStrategy::Html })
    .build()
    .await?;

let result = pipeline_facade.execute(pipeline).await?;
// ⚠️ Result contains mock data, not real extraction
```

### 12.2 Proposed Usage (Enhanced)

```rust
// Proposed facade - production-ready with full features
use riptide_facade::prelude::*;

// Create app state (required for orchestrator)
let app_state = AppState::builder()
    .with_redis("redis://localhost:6379")
    .with_event_bus()
    .with_metrics()
    .build()
    .await?;

// Build orchestrated pipeline
let facade = Riptide::builder()
    .user_agent("Bot/1.0")
    .timeout_secs(30)
    .retry_strategy(RetryStrategy::Adaptive)
    .extraction_strategy(ExtractionStrategyType::Wasm)
    .enable_gate_analysis(true)
    .enable_provenance_tracking(true)
    .build_orchestrated_pipeline(app_state)
    .await?;

// Execute with full orchestration
let result = facade.run_pipeline("https://example.com").await?;

// Access provenance
let events = facade.get_pipeline_events(&result.pipeline_id).await?;
for event in events {
    println!("{}: {}", event.event_type, event.metadata);
}

// Stream multiple URLs
let urls = vec![...];
let mut stream = facade.run_stream(urls).await?;
while let Some(result) = stream.next().await {
    match result {
        Ok(doc) => process_doc(doc),
        Err(e) => log::error!("Extraction failed: {}", e),
    }
}
```

---

## 13. Conclusion

The **Riptide facade layer provides an excellent foundation** with clean builder patterns and user-friendly abstractions. However, there is a **significant gap between the facade's simple pipeline and the production-ready orchestrators** in the API layer.

### Key Findings:

1. **PipelineFacade is a template/demo**, not a production implementation
2. **PipelineOrchestrator exists but is not exposed** via facade
3. **StrategiesPipelineOrchestrator** provides advanced features not accessible
4. **No provenance tracking** in facade (despite API having complete event system)
5. **No streaming support** in facade (despite robust streaming crate)
6. **Service integration is minimal** (direct fetch calls, no orchestration)

### Recommended Approach:

**Create wrapper facades** (`OrchestrationFacade`, `StrategiesFacade`) that **delegate to existing API orchestrators** rather than rebuilding from scratch. This:
- ✅ Leverages proven implementations
- ✅ Minimizes code duplication
- ✅ Provides immediate production readiness
- ✅ Maintains facade simplicity
- ✅ Enables future extensibility

### Next Steps:

1. Review this analysis with the team
2. Prioritize Phase 1 implementation
3. Design `OrchestrationFacade` API
4. Create proof-of-concept wrapper
5. Iterate based on feedback

---

**Analysis Complete:** This document provides a comprehensive foundation for planning the `run_pipeline()` enhancement and broader facade orchestration improvements.
