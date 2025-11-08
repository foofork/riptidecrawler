# Phase 3 Sprints 3.2-3.4: Detailed Execution Plan

**Date:** 2025-11-08
**Status:** 📋 **PLANNING COMPLETE**
**Target Completion:** Sprint 3.2 (3 days) | Sprint 3.3 (2 days) | Sprint 3.4 (2 days)

---

## Executive Summary

This document outlines the detailed execution plan for Sprints 3.2, 3.3, and 3.4, covering the remaining handler refactoring work for Phase 3. The plan focuses on medium-sized handlers (2,600 LOC), the render subsystem (696 LOC), and route auditing.

### Total Scope
- **Sprint 3.2:** 7 medium handlers → 7 facades (2,600 LOC)
- **Sprint 3.3:** 2 render handlers → 1 unified RenderFacade (696 LOC)
- **Sprint 3.4:** 8 route files audit + refactoring (as needed)
- **Total LOC to migrate:** 3,296 LOC

---

## Sprint 3.2: Medium Handler Migrations (7 Handlers, 2,600 LOC)

### Overview

Sprint 3.2 targets **7 medium-sized handlers** totaling **2,600 LOC**. These handlers contain moderate complexity business logic that needs extraction to dedicated facades.

### Prioritization Strategy

Handlers are prioritized by **LOC (largest first)** to maximize impact:

| Priority | Handler | LOC | Target Facade | Complexity | Estimated Facade Size |
|----------|---------|-----|---------------|------------|----------------------|
| 1 | chunking.rs | 356 | ChunkingFacade | Medium | 450 LOC |
| 2 | monitoring.rs | 344 | MonitoringFacade | High | 600 LOC |
| 3 | strategies.rs | 336 | StrategiesFacade | High | 550 LOC |
| 4 | memory.rs | 313 | MemoryFacade | Medium | 400 LOC |
| 5 | deepsearch.rs | 310 | DeepSearchFacade | High | 500 LOC |
| 6 | streaming.rs | 300 | StreamingFacade | High | 550 LOC |
| 7 | pipeline_phases.rs | 289 | PipelinePhasesFacade | Low | 350 LOC |

**Total Facade LOC:** ~3,400 LOC
**Handler Reduction:** 2,600 → <350 LOC (target: 7 handlers × 50 LOC = 350 LOC)
**Net LOC Change:** +800 LOC (adding business logic layer)

---

## Sprint 3.2 Detailed Breakdown

### 1. ChunkingFacade (Priority 1)

**Handler:** `crates/riptide-api/src/handlers/chunking.rs` (356 LOC)
**Target Facade:** `crates/riptide-facade/src/facades/chunking.rs` (450 LOC estimated)

#### Current Handler Analysis
- **Endpoint:** `POST /api/v1/chunk`
- **Business Logic:**
  - Content chunking with multiple strategies (topic, sliding, fixed, sentence, html-aware)
  - Parameter validation (chunk_size, overlap_size, min_chunk_size)
  - Strategy configuration mapping
  - HTML content type detection
  - Performance timing and metrics
  - Error handling for invalid modes

#### Facade Methods Required
```rust
pub struct ChunkingFacade {
    // Dependencies via ports
}

impl ChunkingFacade {
    /// Chunk content using specified strategy
    pub async fn chunk_content(
        &self,
        content: String,
        mode: ChunkingMode,
        config: ChunkingConfig,
    ) -> RiptideResult<ChunkedContentResult>;

    /// Validate chunking parameters
    pub fn validate_chunking_config(&self, config: &ChunkingConfig) -> RiptideResult<()>;

    /// Get supported chunking modes
    pub fn list_supported_modes(&self) -> Vec<ChunkingModeInfo>;

    /// Estimate chunk count for given content
    pub fn estimate_chunks(&self, content: &str, config: &ChunkingConfig) -> usize;
}
```

#### Dependencies on Existing Facades
- **None** - Self-contained facade
- Uses `riptide_extraction::chunking` directly

#### Test Coverage Requirements
- ✅ Test all 5 chunking modes (topic, sliding, fixed, sentence, html-aware)
- ✅ Parameter validation (invalid chunk sizes, negative overlaps)
- ✅ Edge cases (empty content, content < min_chunk_size)
- ✅ HTML content detection
- ✅ Strategy configuration mapping
- **Target:** 15+ unit tests

#### Complexity Assessment
- **Complexity:** Medium
- **Risk:** Low - well-defined chunking strategies
- **Estimated Time:** 4 hours (facade + tests + handler refactoring)

---

### 2. MonitoringFacade (Priority 2)

**Handler:** `crates/riptide-api/src/handlers/monitoring.rs` (344 LOC)
**Target Facade:** `crates/riptide-facade/src/facades/monitoring.rs` (600 LOC estimated)

#### Current Handler Analysis
- **Endpoints:**
  - `GET /monitoring/health-score`
  - `GET /monitoring/performance-report`
  - `GET /monitoring/alert-rules`
  - `GET /monitoring/active-alerts`
  - `GET /monitoring/current-metrics`
  - `GET /monitoring/resource-status`
  - `GET /monitoring/memory-metrics`
  - `GET /monitoring/leak-analysis`
  - `GET /monitoring/allocation-metrics`
  - `GET /monitoring/wasm-health`

- **Business Logic:**
  - Health score calculation (0-100)
  - Performance report generation
  - Alert rule management
  - Active alert filtering
  - Real-time metrics collection
  - Resource status tracking
  - Memory profiling integration
  - Leak detection analysis
  - WASM health monitoring

#### Facade Methods Required
```rust
pub struct MonitoringFacade {
    monitoring_system: Arc<dyn MonitoringSystemPort>,
    metrics_collector: Arc<dyn MetricsCollectorPort>,
}

impl MonitoringFacade {
    /// Calculate current health score (0-100)
    pub async fn calculate_health_score(&self) -> RiptideResult<f32>;

    /// Generate comprehensive performance report
    pub async fn generate_performance_report(&self) -> RiptideResult<PerformanceReport>;

    /// Get all alert rules
    pub async fn get_alert_rules(&self) -> RiptideResult<Vec<AlertRule>>;

    /// Get currently active alerts
    pub async fn get_active_alerts(&self) -> RiptideResult<Vec<ActiveAlert>>;

    /// Get current system metrics snapshot
    pub async fn get_current_metrics(&self) -> RiptideResult<MetricsSnapshot>;

    /// Get resource status (CPU, memory, disk)
    pub async fn get_resource_status(&self) -> RiptideResult<ResourceStatus>;

    /// Get memory profiling metrics
    pub async fn get_memory_metrics(&self) -> RiptideResult<MemoryMetrics>;

    /// Analyze memory leaks
    pub async fn analyze_memory_leaks(&self) -> RiptideResult<LeakAnalysis>;

    /// Get allocation metrics
    pub async fn get_allocation_metrics(&self) -> RiptideResult<AllocationMetrics>;

    /// Get WASM health status
    pub async fn get_wasm_health(&self) -> RiptideResult<WasmHealthStatus>;
}
```

#### Dependencies on Existing Facades
- **MonitoringSystem** (via port) - from AppState
- **MetricsCollector** (via port) - from AppState
- **ProfilingFacade** - for memory metrics integration (Sprint 3.1)

#### Test Coverage Requirements
- ✅ Health score calculation with various states
- ✅ Performance report generation
- ✅ Alert rule retrieval and filtering
- ✅ Active alert filtering by severity
- ✅ Metrics snapshot consistency
- ✅ Resource status boundary conditions (high CPU, low memory)
- ✅ Memory leak detection scenarios
- ✅ WASM health monitoring
- **Target:** 20+ unit tests

#### Complexity Assessment
- **Complexity:** High - multiple subsystems integration
- **Risk:** Medium - depends on monitoring_system implementation
- **Estimated Time:** 6 hours (facade + tests + handler refactoring)

---

### 3. StrategiesFacade (Priority 3)

**Handler:** `crates/riptide-api/src/handlers/strategies.rs` (336 LOC)
**Target Facade:** `crates/riptide-facade/src/facades/strategies.rs` (550 LOC estimated)

#### Current Handler Analysis
- **Endpoints:**
  - `POST /strategies/crawl` - Strategy-based crawling with extraction
  - Query parameters for extraction strategy override

- **Business Logic:**
  - Strategy selection (CSS_JSON, REGEX, LLM - future features)
  - Pipeline orchestration with strategies
  - Cache mode configuration
  - Schema validation (future)
  - Performance metrics collection
  - Custom CSS selectors (future)
  - Custom regex patterns (future)
  - LLM configuration (future)
  - Result aggregation

#### Facade Methods Required
```rust
pub struct StrategiesFacade {
    pipeline_orchestrator: Arc<StrategiesPipelineOrchestrator>,
    strategy_config: Arc<dyn StrategyConfigPort>,
}

impl StrategiesFacade {
    /// Execute strategy-based crawl
    pub async fn execute_strategy_crawl(
        &self,
        url: String,
        strategy: ExtractionStrategyType,
        config: StrategyCrawlConfig,
    ) -> RiptideResult<StrategyResult>;

    /// List available extraction strategies
    pub fn list_strategies(&self) -> Vec<StrategyInfo>;

    /// Validate strategy configuration
    pub fn validate_strategy_config(
        &self,
        strategy: &ExtractionStrategyType,
        config: &StrategyConfig,
    ) -> RiptideResult<()>;

    /// Configure CSS selectors for CSS_JSON strategy
    pub async fn configure_css_strategy(
        &self,
        selectors: HashMap<String, String>,
    ) -> RiptideResult<()>;

    /// Configure regex patterns for REGEX strategy
    pub async fn configure_regex_strategy(
        &self,
        patterns: Vec<RegexPattern>,
    ) -> RiptideResult<()>;

    /// Configure LLM strategy
    pub async fn configure_llm_strategy(
        &self,
        llm_config: LlmStrategyConfig,
    ) -> RiptideResult<()>;
}
```

#### Dependencies on Existing Facades
- **ScraperFacade** - for URL crawling (Phase 2)
- **CacheFacade** - for cache mode handling (Phase 2)

#### Test Coverage Requirements
- ✅ Strategy-based crawl with all strategies
- ✅ Strategy validation (valid/invalid configs)
- ✅ CSS selector configuration
- ✅ Regex pattern configuration
- ✅ LLM configuration
- ✅ Cache mode integration
- ✅ Performance metrics collection
- ✅ Error handling for invalid URLs
- **Target:** 18+ unit tests

#### Complexity Assessment
- **Complexity:** High - multiple strategy types and future expansion
- **Risk:** Medium - orchestrator dependencies
- **Estimated Time:** 6 hours (facade + tests + handler refactoring)

---

### 4. MemoryFacade (Priority 4)

**Handler:** `crates/riptide-api/src/handlers/memory.rs` (313 LOC)
**Target Facade:** `crates/riptide-facade/src/facades/memory.rs` (400 LOC estimated)

#### Current Handler Analysis
- **Endpoint:** `GET /api/v1/memory/profile`
- **Business Logic:**
  - Memory profiling data collection
  - Component-wise breakdown (extraction, api, cache, other)
  - Peak usage tracking
  - Current usage calculation
  - Memory pressure detection
  - jemalloc statistics (if enabled)
  - Fragmentation ratio calculation
  - ISO 8601 timestamp formatting

#### Facade Methods Required
```rust
pub struct MemoryFacade {
    profiler: Arc<dyn MemoryProfilerPort>,
}

impl MemoryFacade {
    /// Get current memory profile
    pub async fn get_memory_profile(&self) -> RiptideResult<MemoryProfile>;

    /// Get memory breakdown by component
    pub async fn get_component_breakdown(&self) -> RiptideResult<HashMap<String, f64>>;

    /// Get peak memory usage
    pub fn get_peak_usage(&self) -> f64;

    /// Detect memory pressure level
    pub fn detect_memory_pressure(&self) -> MemoryPressureLevel;

    /// Get jemalloc statistics (if available)
    pub fn get_jemalloc_stats(&self) -> Option<JemallocStats>;

    /// Calculate fragmentation ratio
    pub fn calculate_fragmentation(&self) -> f64;
}
```

#### Dependencies on Existing Facades
- **ProfilingFacade** (Sprint 3.1) - for memory metrics integration

#### Test Coverage Requirements
- ✅ Memory profile retrieval
- ✅ Component breakdown calculation
- ✅ Peak usage tracking
- ✅ Pressure detection (normal, moderate, high)
- ✅ jemalloc stats (with and without feature flag)
- ✅ Fragmentation calculation
- **Target:** 12+ unit tests

#### Complexity Assessment
- **Complexity:** Medium - profiling integration
- **Risk:** Low - well-defined profiling interface
- **Estimated Time:** 5 hours (facade + tests + handler refactoring)

---

### 5. DeepSearchFacade (Priority 5)

**Handler:** `crates/riptide-api/src/handlers/deepsearch.rs` (310 LOC)
**Target Facade:** `crates/riptide-facade/src/facades/deepsearch.rs` (500 LOC estimated)

#### Current Handler Analysis
- **Endpoint:** `POST /deepsearch`
- **Business Logic:**
  - Search query validation
  - Web search using configured SearchProvider (Serper, etc.)
  - URL extraction from search results
  - Pipeline orchestration for discovered URLs
  - Combined result aggregation (search + content)
  - Telemetry integration (trace context)
  - Event emission (deepsearch.started, deepsearch.completed)
  - Error handling for search failures

#### Facade Methods Required
```rust
pub struct DeepSearchFacade {
    search_provider: Arc<dyn SearchProviderPort>,
    pipeline_orchestrator: Arc<dyn PipelineOrchestratorPort>,
    event_bus: Arc<dyn EventBusPort>,
}

impl DeepSearchFacade {
    /// Execute deep search with content extraction
    pub async fn execute_deep_search(
        &self,
        query: String,
        limit: usize,
        include_content: bool,
        ctx: AuthorizationContext,
    ) -> RiptideResult<DeepSearchResult>;

    /// Validate search query
    pub fn validate_query(&self, query: &str) -> RiptideResult<()>;

    /// Search web using configured provider
    async fn search_web(&self, query: &str, limit: usize) -> RiptideResult<Vec<SearchResult>>;

    /// Extract URLs from search results
    fn extract_urls(&self, results: &[SearchResult]) -> Vec<String>;

    /// Crawl discovered URLs
    async fn crawl_urls(&self, urls: Vec<String>) -> RiptideResult<Vec<CrawlResult>>;

    /// Combine search and crawl results
    fn combine_results(
        &self,
        search_results: Vec<SearchResult>,
        crawl_results: Vec<CrawlResult>,
    ) -> DeepSearchResult;
}
```

#### Dependencies on Existing Facades
- **ScraperFacade** (Phase 2) - for URL crawling
- **AuthorizationContext** - for tenant scoping

#### Test Coverage Requirements
- ✅ Deep search with valid query
- ✅ Query validation (empty, too long, invalid characters)
- ✅ Web search with multiple providers
- ✅ URL extraction from results
- ✅ Pipeline orchestration for URLs
- ✅ Result aggregation
- ✅ Event emission
- ✅ Error handling for search provider failures
- **Target:** 18+ unit tests

#### Complexity Assessment
- **Complexity:** High - multiple subsystems integration
- **Risk:** Medium - depends on search provider availability
- **Estimated Time:** 6 hours (facade + tests + handler refactoring)

---

### 6. StreamingFacade (Priority 6)

**Handler:** `crates/riptide-api/src/handlers/streaming.rs` (300 LOC)
**Target Facade:** `crates/riptide-facade/src/facades/streaming.rs` (550 LOC estimated)

#### Current Handler Analysis
- **Endpoints:**
  - `POST /v1/crawl/stream` - NDJSON crawl streaming
  - `POST /v1/deepsearch/stream` - NDJSON deep search streaming

- **Business Logic:**
  - Real-time NDJSON streaming
  - Backpressure handling
  - Progress updates
  - Request ID generation (UUID)
  - Timing metrics
  - Error handling with stream recovery
  - Content-Type: application/x-ndjson

#### Facade Methods Required
```rust
pub struct StreamingFacade {
    ndjson_handler: Arc<NdjsonStreamingHandler>,
    pipeline: Arc<dyn PipelineOrchestratorPort>,
}

impl StreamingFacade {
    /// Stream crawl results in NDJSON format
    pub async fn stream_crawl(
        &self,
        urls: Vec<String>,
        options: CrawlOptions,
    ) -> RiptideResult<impl Stream<Item = Result<Bytes, Error>>>;

    /// Stream deep search results in NDJSON format
    pub async fn stream_deep_search(
        &self,
        query: String,
        limit: usize,
    ) -> RiptideResult<impl Stream<Item = Result<Bytes, Error>>>;

    /// Create NDJSON line from result
    fn create_ndjson_line(&self, result: impl Serialize) -> Result<Bytes, Error>;

    /// Handle backpressure
    async fn apply_backpressure(&self, queue_size: usize);

    /// Generate progress update
    fn create_progress_update(&self, completed: usize, total: usize) -> ProgressUpdate;
}
```

#### Dependencies on Existing Facades
- **ScraperFacade** (Phase 2) - for URL crawling
- **DeepSearchFacade** (Sprint 3.2, Priority 5) - for deep search

#### Test Coverage Requirements
- ✅ NDJSON crawl streaming
- ✅ NDJSON deep search streaming
- ✅ Backpressure handling
- ✅ Progress updates at intervals
- ✅ Error recovery in stream
- ✅ Request ID generation
- ✅ Content-Type validation
- **Target:** 15+ unit tests

#### Complexity Assessment
- **Complexity:** High - streaming and backpressure management
- **Risk:** Medium - depends on streaming infrastructure
- **Estimated Time:** 6 hours (facade + tests + handler refactoring)

---

### 7. PipelinePhasesFacade (Priority 7)

**Handler:** `crates/riptide-api/src/handlers/pipeline_phases.rs` (289 LOC)
**Target Facade:** `crates/riptide-facade/src/facades/pipeline_phases.rs` (350 LOC estimated)

#### Current Handler Analysis
- **Endpoint:** `GET /pipeline/phases` (implied)
- **Business Logic:**
  - Pipeline phase breakdown analysis
  - Overall metrics calculation (total requests, avg time, percentiles)
  - Individual phase metrics (avg duration, percentage of total, count)
  - Bottleneck detection with impact scores
  - Success rate calculation per phase
  - P50, P95, P99 latency calculation

#### Facade Methods Required
```rust
pub struct PipelinePhasesFacade {
    metrics_collector: Arc<dyn MetricsCollectorPort>,
}

impl PipelinePhasesFacade {
    /// Get pipeline phase breakdown
    pub async fn get_phase_breakdown(&self) -> RiptideResult<PipelinePhaseBreakdown>;

    /// Calculate overall metrics
    pub fn calculate_overall_metrics(&self, requests: &[Request]) -> OverallMetrics;

    /// Get individual phase metrics
    pub fn get_phase_metrics(&self, phase: &str) -> RiptideResult<PhaseMetrics>;

    /// Detect bottlenecks
    pub fn detect_bottlenecks(&self, phases: &[PhaseMetrics]) -> Vec<BottleneckInfo>;

    /// Calculate success rates
    pub fn calculate_success_rates(&self, phases: &[PhaseMetrics]) -> SuccessRates;

    /// Calculate percentiles (P50, P95, P99)
    pub fn calculate_percentiles(&self, latencies: &[f64]) -> Percentiles;
}
```

#### Dependencies on Existing Facades
- **MonitoringFacade** (Sprint 3.2, Priority 2) - for metrics integration

#### Test Coverage Requirements
- ✅ Phase breakdown calculation
- ✅ Overall metrics with various request counts
- ✅ Individual phase metrics
- ✅ Bottleneck detection (single and multiple bottlenecks)
- ✅ Success rate calculation
- ✅ Percentile calculation (edge cases: 0 requests, 1 request, many requests)
- **Target:** 14+ unit tests

#### Complexity Assessment
- **Complexity:** Low - metrics aggregation
- **Risk:** Low - well-defined metrics calculations
- **Estimated Time:** 4 hours (facade + tests + handler refactoring)

---

## Sprint 3.2 Batch Execution Strategy

### Multi-Agent Swarm Approach (4 Concurrent Agents)

Sprint 3.2 will use **4 concurrent agents** to maximize development velocity:

| Agent | Responsibility | Handlers/Facades | Total LOC | Complexity | Est. Time |
|-------|---------------|------------------|-----------|------------|-----------|
| **Agent #1** | Chunking + Memory | ChunkingFacade (450) + MemoryFacade (400) | 850 LOC | Medium | 9 hours |
| **Agent #2** | Monitoring + Pipeline | MonitoringFacade (600) + PipelinePhasesFacade (350) | 950 LOC | Medium-High | 10 hours |
| **Agent #3** | Strategies + DeepSearch | StrategiesFacade (550) + DeepSearchFacade (500) | 1,050 LOC | High | 12 hours |
| **Agent #4** | Streaming | StreamingFacade (550) | 550 LOC | High | 6 hours |

**Total Parallel Execution Time:** ~12 hours (longest agent)
**Sequential Execution Time:** ~43 hours
**Speedup:** 3.6x faster

### Agent Instructions Template

Each agent will receive:
1. **Facade specification** (methods, dependencies, ports)
2. **Handler analysis** (current business logic)
3. **Test requirements** (coverage targets)
4. **Integration points** (existing facades)
5. **Quality gates** (clippy, tests pass, handler <50 LOC)

---

## Sprint 3.3: Render Subsystem (2 Handlers, 696 LOC)

### Overview

Sprint 3.3 consolidates the **render subsystem** into a unified **RenderFacade**. This involves merging business logic from two handlers:

1. `render/handlers.rs` (362 LOC) - Core rendering logic
2. `render/processors.rs` (334 LOC) - Content processing strategies

### Target: RenderFacade (900 LOC estimated)

**Location:** `crates/riptide-facade/src/facades/render.rs`

#### Current Render Subsystem Analysis

**Endpoints:**
- `POST /render` - Main render endpoint with dynamic content handling

**Business Logic:**
- Resource management and acquisition
- Timeout controls
- Session context handling
- Rendering mode selection (PDF, dynamic, static, adaptive)
- Content extraction
- Stealth controller integration
- Performance metrics collection

**Processing Strategies:**
- `process_pdf()` - PDF content processing
- `process_dynamic()` - Dynamic content rendering
- `process_static()` - Static content extraction
- `process_adaptive()` - Adaptive strategy selection

#### RenderFacade Methods Required

```rust
pub struct RenderFacade {
    resource_manager: Arc<dyn ResourceManagerPort>,
    scraper_facade: Arc<ScraperFacade>,
    pdf_processor: Arc<dyn PdfProcessorPort>,
    stealth_controller: Arc<StealthController>,
}

impl RenderFacade {
    /// Render URL with specified mode
    pub async fn render(
        &self,
        url: String,
        mode: RenderMode,
        config: RenderConfig,
        ctx: SessionContext,
    ) -> RiptideResult<RenderResult>;

    /// Process PDF content
    async fn process_pdf(
        &self,
        url: &str,
        config: Option<PdfConfig>,
    ) -> RiptideResult<PdfRenderResult>;

    /// Process dynamic content
    async fn process_dynamic(
        &self,
        url: &str,
        config: DynamicConfig,
    ) -> RiptideResult<DynamicRenderResult>;

    /// Process static content
    async fn process_static(
        &self,
        url: &str,
    ) -> RiptideResult<StaticRenderResult>;

    /// Select adaptive strategy
    async fn process_adaptive(
        &self,
        url: &str,
        request: &RenderRequest,
    ) -> RiptideResult<AdaptiveRenderResult>;

    /// Extract content from rendered page
    async fn extract_content(
        &self,
        html: &str,
    ) -> RiptideResult<ExtractedContent>;

    /// Acquire render resources
    async fn acquire_resources(
        &self,
        url: &str,
    ) -> RiptideResult<ResourceGuard>;
}
```

#### Dependencies on Existing Facades
- **ScraperFacade** (Phase 2) - for URL fetching
- **PdfFacade** (Sprint 3.1) - for PDF processing
- **SessionContext** - from middleware

#### Test Coverage Requirements
- ✅ Render with all modes (PDF, dynamic, static, adaptive)
- ✅ Resource acquisition success/failure
- ✅ Timeout handling
- ✅ Session context integration
- ✅ PDF processing
- ✅ Dynamic content rendering
- ✅ Static content extraction
- ✅ Adaptive strategy selection
- ✅ Content extraction
- ✅ Stealth integration
- **Target:** 20+ unit tests

#### Complexity Assessment
- **Complexity:** High - multiple rendering strategies and resource management
- **Risk:** Medium - depends on resource_manager and stealth_controller
- **Estimated Time:** 8 hours (facade + tests + handler refactoring)

### Sprint 3.3 Execution Strategy

**Single Agent Approach:**
- Agent #5 will handle RenderFacade creation, testing, and handler refactoring
- Sequential execution due to tight coupling between handlers and processors

---

## Sprint 3.4: Route Audit (8 Route Files)

### Overview

Sprint 3.4 audits **8 route files** for business logic violations. Route files should ONLY contain:
- Route registration (Router setup)
- Path definitions
- HTTP method mappings
- NO business logic

### Route Files to Audit

| Route File | LOC | Business Logic Risk | Action Required |
|------------|-----|---------------------|-----------------|
| routes/profiles.rs | 124 | ⚠️ High | Refactor if logic found |
| routes/pdf.rs | 58 | ⚠️ Medium | Review and refactor |
| routes/stealth.rs | 52 | ⚠️ Medium | Review and refactor |
| routes/llm.rs | 34 | ✅ Low | Review only |
| routes/tables.rs | 28 | ✅ Low | Review only |
| routes/engine.rs | 23 | ✅ Low | Review only |
| routes/chunking.rs | 21 | ✅ Low | Review only |
| routes/mod.rs | 7 | ✅ Low | Review only |

**Total Route LOC:** 347 LOC

### Audit Checklist

For each route file, verify:
1. ✅ No business logic (calculations, validations, transformations)
2. ✅ No direct database/cache access
3. ✅ Only handler function calls
4. ✅ Clean Router::new() setup
5. ✅ Proper HTTP method mappings (get, post, put, delete)

### Expected Findings

**routes/profiles.rs (124 LOC)** - Likely contains:
- Profile validation logic
- DTO transformations
- Error handling beyond simple delegation

**Action:** Extract to ProfileFacade (already exists from Sprint 3.1) or create helper utilities

### Sprint 3.4 Execution Strategy

**Single Agent Approach:**
- Agent #6 will audit all 8 route files
- Generate refactoring recommendations
- Apply minimal fixes to high-risk files (profiles.rs, pdf.rs, stealth.rs)

**Estimated Time:** 6 hours (audit + refactoring)

---

## Overall Timeline (Sprints 3.2-3.4)

### Sprint 3.2 (Days 1-3)
- **Day 1:** Agents #1-4 start parallel facade development
- **Day 2:** Agents complete facades and tests
- **Day 3:** Handler refactoring, integration testing, quality gates

### Sprint 3.3 (Days 4-5)
- **Day 4:** Agent #5 creates RenderFacade
- **Day 5:** Handler refactoring, testing, integration

### Sprint 3.4 (Days 6-7)
- **Day 6:** Agent #6 audits all route files
- **Day 7:** Refactoring and final quality gates

**Total Duration:** 7 days
**With parallel execution:** Effective 3-4 days

---

## Quality Gates (All Sprints)

### Compilation
```bash
RUSTFLAGS="-D warnings" cargo build --workspace
```

### Clippy
```bash
cargo clippy --all -- -D warnings
```

### Tests
```bash
cargo test -p riptide-facade
cargo test -p riptide-api
```

### Handler LOC Verification
```bash
for file in crates/riptide-api/src/handlers/*.rs; do
    wc -l "$file"
done | sort -rn | head -20
```

### Success Criteria
- ✅ All facades compile without errors
- ✅ Zero clippy warnings
- ✅ All tests pass (100% success rate)
- ✅ All handlers <50 LOC (target: 100% compliance)
- ✅ No business logic in route files

---

## Risk Mitigation

### Risk #1: Dependency Conflicts
**Mitigation:** Agents coordinate via memory using claude-flow hooks
**Fallback:** Sequential execution if conflicts arise

### Risk #2: Missing Ports/Interfaces
**Mitigation:** Identify required ports during facade design phase
**Fallback:** Create temporary mock ports, implement in Phase 4

### Risk #3: Test Failures
**Mitigation:** TDD approach - write tests before implementation
**Fallback:** Dedicated testing agent for troubleshooting

### Risk #4: Handler LOC Target Miss
**Mitigation:** Extract DTO converters and helper utilities
**Fallback:** Accept 50-70 LOC if business logic is 100% in facades

---

## Memory Coordination Strategy

All agents will use **claude-flow hooks** for coordination:

### Pre-Task Hook
```bash
npx claude-flow@alpha hooks pre-task --description "[agent-task]"
```

### Post-Edit Hook
```bash
npx claude-flow@alpha hooks post-edit --file "[file]" --memory-key "swarm/agent-[n]/[facade]"
```

### Session Restore
```bash
npx claude-flow@alpha hooks session-restore --session-id "sprint-3.2-3.4"
```

### Post-Task Hook
```bash
npx claude-flow@alpha hooks post-task --task-id "[agent-task]"
```

---

## Success Metrics

### Quantitative Targets

| Metric | Baseline | Target | Sprint 3.2 | Sprint 3.3 | Sprint 3.4 |
|--------|----------|--------|------------|------------|------------|
| **Handler LOC** | 2,896 | <600 | -2,250 | -646 | Variable |
| **Facades Created** | 8 | 15 | +7 | +1 | 0 |
| **Unit Tests** | 70 | 160+ | +60 | +20 | +10 |
| **Avg Handler LOC** | 207 | <50 | ~50 | ~40 | ~40 |
| **Route Files Clean** | Unknown | 100% | N/A | N/A | 100% |

### Qualitative Targets
- ✅ All facades follow hexagonal architecture
- ✅ All facades have port-based dependencies
- ✅ Zero HTTP types in facades
- ✅ Comprehensive test coverage (>90% per facade)
- ✅ Clean route files (no business logic)

---

## Completion Checklist

### Sprint 3.2
- [ ] ChunkingFacade created and tested (15+ tests)
- [ ] MonitoringFacade created and tested (20+ tests)
- [ ] StrategiesFacade created and tested (18+ tests)
- [ ] MemoryFacade created and tested (12+ tests)
- [ ] DeepSearchFacade created and tested (18+ tests)
- [ ] StreamingFacade created and tested (15+ tests)
- [ ] PipelinePhasesFacade created and tested (14+ tests)
- [ ] All 7 handlers refactored to <50 LOC
- [ ] Quality gates pass (compilation, clippy, tests)

### Sprint 3.3
- [ ] RenderFacade created and tested (20+ tests)
- [ ] render/handlers.rs refactored to <50 LOC
- [ ] render/processors.rs logic migrated to RenderFacade
- [ ] Quality gates pass

### Sprint 3.4
- [ ] All 8 route files audited
- [ ] Business logic violations documented
- [ ] routes/profiles.rs refactored (if needed)
- [ ] routes/pdf.rs refactored (if needed)
- [ ] routes/stealth.rs refactored (if needed)
- [ ] Quality gates pass

---

## Next Steps After Completion

1. **Phase 3 Completion Report**
   - Document total LOC migration
   - Validate handler refactoring success
   - Review hexagonal architecture compliance

2. **Phase 4 Preparation**
   - Identify missing ports/interfaces
   - Plan infrastructure adapters
   - Prepare domain layer enhancements

3. **Technical Debt Review**
   - Assess remaining handlers (if any)
   - Identify refactoring opportunities
   - Plan optimization sprints

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
