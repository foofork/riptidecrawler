# RipTide Crawler Development Roadmap - Remaining Work

## Current Status

**✅ Completed Phases**: Phase 0 (Foundation), Phase 1 (Core), Phase 2 Lite (Reliability), Phase 3 PR-1 & PR-2
**📍 Current Focus**: Phase 0 Technical Debt & Core Integration (60% Complete)
**🎯 Next Major Milestone**: Complete Phase 3 PR-3 (NDJSON Streaming)

**See [COMPLETED.md](./COMPLETED.md) for detailed list of all completed work.**

---

## 🔴 PHASE 0: Critical Technical Debt Resolution & Core Integration - 60% COMPLETE
*Priority: CRITICAL - Remaining Production Blockers*

**Completed Items**: See [COMPLETED.md](./COMPLETED.md) for full list of resolved technical debt including:
- ✅ Security vulnerabilities patched
- ✅ Mega-file refactoring complete (pdf.rs, stealth.rs, streaming.rs)
- ✅ Performance optimizations applied
- ✅ Dependency conflicts resolved
- ✅ Build optimization (3.9GB recovered)
- ✅ Legacy API removal complete

### 0.1 Error Handling Improvements - HIGH PRIORITY
**Status**: 🚧 IN PROGRESS
**Effort**: 3-4 days
**Scope**: 542 unwrap/expect calls identified (analysis shows higher count than original 305)

- [ ] **Replace remaining unwrap/expect calls**
  - Progress: 25 critical ones fixed, ~517 remaining
  - Focus on production-critical paths first
  - Implement proper Result/Option patterns
  - Add comprehensive error recovery

### 0.2 Test Coverage Gap - MEDIUM PRIORITY
**Status**: 🟡 PENDING
**Effort**: 3-5 days

- [ ] **Achieve 80%+ test coverage** (currently 75%)
  - Critical paths have 40% coverage gaps
  - Add integration tests for refactored modules
  - Implement golden tests for new features

### 0.3 Monitoring & Observability - HIGH PRIORITY
**Status**: 🟡 PENDING
**Effort**: 1 week

- [ ] **Production monitoring infrastructure**
  - Add comprehensive tracing with OpenTelemetry (currently disabled in telemetry.rs:146)
  - Implement performance benchmarking suite
  - Create SLA monitoring dashboards
  - Add detailed performance metrics
  - Implement actual system metrics collection (placeholders in health.rs:358-366)
  - Track active connections from middleware
  - Calculate requests per second from metrics
  - Implement average response time tracking
  - Add disk usage tracking (telemetry.rs:525)
  - Implement file descriptor tracking (telemetry.rs:528)
  - Complete CPU and memory usage collection (monitoring/collector.rs:292-298)

### 0.4 Resource Management & Performance - MEDIUM PRIORITY
**Status**: 🟡 PENDING
**Effort**: 3-4 days

- [ ] **Browser pooling and memory optimization**
  - Implement connection pooling for headless Chrome
  - Add resource cleanup on timeout
  - Monitor WASM component lifecycle
  - Implement memory usage alerts
  - Memory allocation improvements for better performance

- [ ] **Build Pipeline Optimization**
  - Address WASM compilation timeouts (5+ min build times)
  - Implement dependency caching for CI/CD
  - Enable incremental compilation
  - Set up parallel builds
  - Implement component pooling for WASM modules to reduce overhead

- [ ] **Circuit Breaker Enhancements**
  - Consider adaptive thresholds for dynamic adjustment
  - Implement performance-based threshold tuning
  - Add self-learning capabilities for failure patterns

---


### 0.5 Core Integration Gaps - HIGH PRIORITY
**Status**: 🟡 PENDING
**Effort**: 1 week

- [ ] **WASM Extractor Integration**
  - Wire actual WASM extractor in handlers/render.rs:401 (currently placeholder)
  - Remove placeholder content generation (render.rs:404-409)
  - Integrate trek-rs extractor with render pipeline
  - Complete extraction output format handling

- [ ] **Dynamic Rendering Implementation**
  - Implement actual headless browser rendering (render.rs:293-297 placeholder)
  - Wire headless service to render handlers
  - Complete browser action execution (render.rs:290)
  - Implement content analysis for adaptive rendering (render.rs:382)

- [ ] **Session & Cookie Management**
  - Implement session persistence across requests
  - Add cookie jar management for authentication
  - Support session-based crawling
  - Add session cleanup and TTL management

- [ ] **Worker Service Implementation**
  - Complete background worker service (riptide-workers/main.rs:13)
  - Implement batch processing queue
  - Add job scheduling and retry logic
  - Create worker pool management

---

## 🌟 PHASE 3: Crawl4AI Parity Features (4-6 weeks)
*Priority: HIGH - Feature parity with Crawl4AI*

**Goal**: Complete feature parity with Crawl4AI for competitive positioning using merge-ready PR strategy

### 📋 Implementation Strategy

**Branch**: `feature/phase3`
**Merge Strategy**: Low risk, fast landing with 6 sequential PRs (each releasable)

**Feature Flags Configuration**:
```yaml
features:
  headless_v2: false      # PR-1: Actions/waits/scroll/sessions
  stealth: false          # PR-2: UA rotation + JS evasion
  streaming: true         # PR-3: NDJSON streaming endpoints
  pdf: true              # PR-4: PDF pipeline with pdfium
  spider: false          # PR-5: Spider integration with budgets
  strategies: true       # PR-6: CSS/XPath/Regex + chunking
```

**Performance Guardrails**:
```yaml
perf:
  headless_pool_size: 3         # Hard cap on concurrent headless instances
  headless_hard_cap_ms: 3000    # Maximum render time budget
  fetch_connect_ms: 3000        # Connection timeout
  fetch_total_ms: 20000         # Total fetch timeout
  pdf_max_concurrent: 2         # Concurrent PDF processing limit
  streaming_buf_bytes: 65536    # NDJSON streaming buffer size
  crawl_queue_max: 1000         # Maximum crawl queue size
  per_host_rps: 1.5            # Rate limiting per host
```

### 🚀 PR-1: Headless RPC v2 - Dynamic Content ✅ COMPLETE
**Note**: PR-1 has been completed. Integration with API passthrough pending feature flag activation.

### 🚀 PR-2: Stealth Preset - Anti-Detection
**Status**: ✅ COMPLETE (Merged in commit 75c67c0)
**Branch**: `feature/phase3-pr2-stealth` (merged)
**Feature Flag**: `stealth: false` (default OFF, ready for activation)
```javascript
// stealth.js - injected before any page scripts
navigator.webdriver = false;
Object.defineProperty(navigator, 'languages', {
  get: () => ['en-US', 'en']
});
// Canvas/WebGL fingerprint noise
// Platform/plugin spoofing
```

**Configuration**:
```yaml
stealth:
  ua_pool_file: "configs/ua_list.txt"
  canvas_noise: true
  webgl_vendor: "Intel Inc."
```

**Acceptance**: Repeat crawl to bot-detection site → ≥80% success rate without challenges

### 🚀 PR-3: NDJSON Streaming - Real-time Output (Week 2)
**Branch**: `feature/phase3-pr3-streaming`
**Feature Flag**: `streaming: true` (default ON)

**Files**: `riptide-api/src/streaming.rs`, route handlers

- [ ] **Streaming endpoints**: `/crawl/stream`, `/deepsearch/stream`
```rust
pub async fn crawl_stream(State(app): State<AppState>, Json(body): Json<CrawlBody>) -> impl IntoResponse {
    let (tx, rx) = tokio::sync::mpsc::channel::<Vec<u8>>(128);
    tokio::spawn(async move {
        if let Err(e) = orchestrate_stream(app, body, tx).await {
            // Emit error line in NDJSON format
        }
    });
    axum::response::Response::builder()
        .header("Content-Type","application/x-ndjson")
        .body(axum::body::Body::from_stream(ReceiverStream::new(rx).map(axum::body::Bytes::from)))
        .unwrap()
}
```

- [ ] **NDJSON format**: One JSON object per line for each completed URL
- [ ] **Real-time progress**: Stream results as they complete, not batched
- [ ] **Error handling**: Include failed URLs with error details in stream

**Acceptance**: 10-URL batch → TTFB < 500ms → results stream as completed → all results captured

### 🚀 PR-4: PDF Pipeline - Document Processing (Week 3)
**Branch**: `feature/phase3-pr4-pdf`
**Feature Flag**: `pdf: true` (default ON)

**Files**: `riptide-core/src/pdf.rs`, `types.rs`

- [ ] **PDF detection**: Content-type or URL suffix → skip headless rendering
- [ ] **pdfium-render integration**: Extract text, metadata, images
```rust
pub async fn process_pdf(pdf_bytes: &[u8]) -> anyhow::Result<ExtractedDoc> {
    let document = PdfDocument::from_bytes(pdf_bytes)?;
    let text = extract_text_from_pages(&document)?;
    let metadata = extract_metadata(&document)?; // Author, title, creation date
    let images = extract_images(&document)?;

    Ok(ExtractedDoc {
        content: text,
        metadata: Some(metadata),
        images: Some(images),
        ..Default::default()
    })
}
```

- [ ] **Concurrency control**: Semaphore with `pdf_max_concurrent: 2`
- [ ] **Metadata extraction**: Author, title, creation/modification dates
- [ ] **Image extraction**: Count and position data for illustrations

**Acceptance**: PDF URLs → text extracted → metadata populated → image count > 0 for illustrated docs

### 🚀 PR-5: Spider Integration - Deep Crawling (Week 4)
**Branch**: `feature/phase3-pr5-spider`
**Feature Flag**: `spider: false` (default OFF)

**Files**: `riptide-core/src/crawl.rs`, `riptide-api/src/models.rs`

- [ ] **Frontier strategies**: BFS, DFS, Best-First (priority on link hints + depth penalty)
- [ ] **Sitemap parsing**: Discover from robots.txt, merge URLs while respecting robots
- [ ] **Adaptive stopping**: Sliding window of unique_text_chars or scored chunk gain
```rust
pub struct CrawlConfig {
    pub mode: FrontierMode,        // "best-first" | "bfs" | "dfs"
    pub max_depth: u32,            // Hard limit: 3-5 levels
    pub budget: CrawlBudget,       // Time and page limits
    pub adaptive_stop: AdaptiveConfig,
    pub sitemap: SitemapConfig,
}

pub struct AdaptiveConfig {
    pub gain_threshold: u32,       // Min chars/chunk gain to continue
    pub window: u32,               // Sliding window size
    pub patience: u32,             // Stop after N consecutive low-gain pages
}
```

**Configuration**:
```yaml
crawler:
  mode: "best-first"
  max_depth: 3
  budget: { pages: 200, seconds: 120 }
  adaptive_stop: { gain_threshold: 600, window: 10, patience: 3 }
  sitemap: { enabled: true }
```

**Acceptance**: Domain seed → respects depth/budgets → sitemap parsed → returns ≥N pages with extraction

### 🚀 PR-6: Strategies & Chunking - Enhanced Extraction (Week 5)
**Branch**: `feature/phase3-pr6-strategies`
**Feature Flag**: `strategies: true` (default ON)

**Files**: `riptide-core/src/strategy/`, `riptide-core/src/chunking.rs`, `riptide-core/src/schema.rs`

- [ ] **Extraction strategies**:
  - `trek`: Default Trek-rs WASM extraction
  - `css_json`: CSS selector-based extraction with JSON schema
  - `regex`: Pattern-based extraction for structured content
  - `llm`: LLM-powered extraction for complex layouts

- [ ] **Content chunking** (5 methods):
```rust
pub enum ChunkingMethod {
    Regex { pattern: String, max_chunks: u32 },
    Sentence { max_sentences: u32, overlap: u32 },
    Topic { similarity_threshold: f32 },
    Fixed { chunk_size: u32, overlap: u32 },
    Sliding { token_max: u32, overlap: u32 },  // Default
}
```

- [ ] **Schema validation**: Use `schemars` to validate output before returning
- [ ] **Default configuration**: TREK + sliding chunks (token_max=1200, overlap=120)

**Configuration**:
```yaml
extraction:
  strategy: "trek"              # trek|css_json|regex|llm
  chunking: { method: "sliding", token_max: 1200, overlap: 120 }
```

**Acceptance**: Long articles → chunked appropriately → links/media lists populated → byline/date extracted ≥80%

### 📊 Performance Targets & Acceptance Criteria

**Fast-path Performance**:
- **p50 ≤ 1.5s**, **p95 ≤ 5s** on 10-URL mixed batch
- **Streaming TTFB < 500ms** for warm cache
- **Headless fallback ratio < 15%** of total pages

**Resource Limits**:
- **PDF throughput**: 2 concurrent PDFs, no >200MB memory spikes per worker
- **Headless pool**: Hard cap of 3 instances, 3s render budget
- **Spider crawling**: Respects depth/budget, stops early on low content gain

**Cache Performance**:
- **Wasmtime**: Instantiate component once per worker, reuse store
- **Redis**: Read-through cache, 24h TTL, keys include extractor version + strategy + chunking
- **Threshold gates**: Headless fallback thresholds hi=0.55 / lo=0.35

### 🚀 Rollout Plan

1. **Initial Merge**: PR-1..3 merged, enable `streaming=true` + `pdf=true`, keep `headless_v2`/`stealth` OFF
2. **Canary Testing**: Enable `headless_v2` on 10% traffic for 1 week, monitor error rates + render_ms
3. **Stealth Activation**: Enable `stealth` flag, validate reduced bot challenge rate
4. **Spider Integration**: Merge PR-5 with `spider=false`, test on staging domains
5. **Full Activation**: Merge PR-6, keep TREK+sliding defaults, enable advanced strategies per job

### 🧪 CI Additions

- **WASM component**: Build once, cache as artifact across test jobs
- **Test scope**: Unit + integration + streaming tests, exclude live-web tests from CI
- **Resource gates**: Lint for large binaries, feature-flag PDF concurrent tests
- **Performance regression**: Automated benchmarks on merge to detect slowdowns

---

## 🏢 PHASE 4: Enterprise Features (6-8 weeks)
*Priority: LOW - Nice-to-have for enterprise*

### 4.1 Scalability & Distribution
- [ ] **Worker service implementation**
  - Background job processing with queues
  - Horizontal scaling across multiple machines
  - Load balancing and failover
  - Distributed crawl coordination

- [ ] **Multi-tenant architecture**
  - API key management and quotas
  - Per-tenant configuration and rate limits
  - Usage analytics and billing integration
  - Tenant isolation and data separation

### 4.2 Advanced Analytics
- [ ] **Crawl analytics and insights**
  - Success/failure rate tracking
  - Content quality scoring
  - Performance analytics per domain
  - Cost analysis and optimization recommendations

### 4.3 CLI and Developer Tools
- [ ] **Command-line interface**
  - Standalone CLI tool for batch operations
  - Configuration file support
  - Progress reporting and resume capability
  - Integration with CI/CD pipelines

---

## 🔧 PHASE 5: Optimization & Maintenance (Ongoing)
*Priority: CONTINUOUS - Long-term sustainability*

### 5.1 Performance Optimization
- [ ] **Advanced caching strategies**
  - Content-based deduplication
  - Intelligent cache warming
  - Predictive pre-fetching
  - Edge caching integration

- [ ] **Resource optimization**
  - Memory usage optimization
  - CPU profiling and optimization
  - Network bandwidth optimization
  - Storage efficiency improvements

### 5.2 Developer Experience
- [ ] **Enhanced documentation**
  - Interactive API documentation with examples
  - Video tutorials and quickstart guides
  - SDK development for popular languages
  - Community contribution guidelines

### 5.3 Ecosystem Integration
- [ ] **Third-party integrations**
  - Webhook support for real-time notifications
  - Plugin architecture for custom extractors
  - Integration with popular databases
  - Export to cloud storage (S3, GCS, Azure)

---

## 🎯 Success Metrics

### Phase 0 (Technical Debt) Remaining Success Criteria:
- [ ] Error handling completed - Replace remaining 517 unwrap/expect calls
- [ ] Test coverage ≥80% (currently at 75%)
- [ ] Monitoring & observability infrastructure deployed
- [ ] Resource management optimized (browser pooling, memory limits)

### Phase 3 (Crawl4AI Parity) Success Criteria:
**PR-1 (Headless RPC v2)**: ✅ COMPLETE - Awaiting feature flag activation
**PR-2 (Stealth Preset)**: ✅ COMPLETE - Merged, awaiting feature flag activation

**PR-3 (NDJSON Streaming)**:
- [ ] `/crawl/stream` and `/deepsearch/stream` endpoints
- [ ] Real-time NDJSON output (one object per line per URL)
- [ ] TTFB < 500ms for first result, results stream as completed
- [ ] Proper error handling in stream format

**PR-4 (PDF Pipeline)**:
- [ ] PDF detection via content-type/URL suffix
- [ ] pdfium-render integration for text + metadata + images
- [ ] Concurrency control with semaphore (max 2 concurrent)
- [ ] Author, title, creation date extraction

**PR-5 (Spider Integration)**:
- [ ] Frontier strategies: BFS, DFS, Best-First with priority scoring
- [ ] Sitemap discovery and parsing from robots.txt
- [ ] Adaptive stopping with gain threshold and sliding window
- [ ] Budget enforcement: max_depth, max_pages, time limits

**PR-6 (Strategies & Chunking)**:
- [ ] Multiple extraction strategies: trek, css_json, regex, llm
- [ ] 5 chunking methods with sliding window default (1200 tokens, 120 overlap)
- [ ] Schema validation with schemars before output
- [ ] Byline/date extraction from Open Graph and JSON-LD

**Performance Targets**:
- [ ] Fast-path p50 ≤ 1.5s, p95 ≤ 5s on 10-URL mixed batch
- [ ] Streaming TTFB < 500ms for warm cache
- [ ] Headless fallback ratio < 15% of total pages
- [ ] PDF: 2 concurrent max, no >200MB memory spikes per worker


### Phase 4 (Enterprise) Success Criteria:
- [ ] Multi-tenant architecture deployed
- [ ] Horizontal scaling to 10+ nodes
- [ ] Enterprise customer onboarding
- [ ] 99.99% uptime SLA capability

---

## 🚨 Critical Dependencies & Risks

### Technical Risks:
1. **WASM Component Model**: New technology stack with wasip2 target
2. **Performance at Scale**: Unknown bottlenecks in concurrent processing
3. **Headless Browser Stability**: Chrome crashes under heavy load
4. **Memory Leaks**: WASM component lifecycle management

### Mitigation Strategies:
- **WASM Component**: Use proven wasmtime::component::bindgen! API, load once per worker
- **Performance**: Implement gradual load testing and optimization
- **Browser Stability**: Add container restart policies and health checks
- **Memory**: Implement proper component instance pooling and cleanup

### External Dependencies:
- **Serper.dev API**: Rate limits and costs
- **Docker/Kubernetes**: Infrastructure stability
- **Redis**: Data persistence and clustering
- **Domain availability**: Sites blocking our crawlers

### Dependency Version Locks:
- **trek-rs**: `=0.2.1` (confirmed available on crates.io)
- **WASM Target**: `wasm32-wasip2` (Tier-2 rustup support)
- **CDP Client**: `chromiumoxide` (current and maintained)
- **Robots Parser**: `robotstxt` (Google's parser port)
- **Metrics**: `axum-prometheus` (Axum middleware integration)

---

## 📅 Timeline Summary

| Phase | Duration | Key Deliverables | Risk Level | Status |
|-------|----------|------------------|------------|---------|
| **Phase 0** | 2-3 weeks | Critical tech debt, monitoring, integration gaps | CRITICAL | ⚠️ 60% COMPLETE |
| **Phase 3** | 3-4 weeks | Advanced features and parity | MEDIUM | ✅ PR-1 & PR-2 COMPLETE, PR-3 NEXT |
| **Phase 4** | 6-8 weeks | Enterprise capabilities | LOW | PLANNED |
| **Phase 5** | Ongoing | Continuous optimization | LOW | PLANNED |

**Total Estimated Timeline**: 3-3.5 months for remaining work (includes newly identified integration gaps)

---

## 🤝 Next Steps

### Immediate Actions (Phase 0 Extended):
**Priority**: Complete remaining technical debt and integration gaps

1. **Error Handling** (3-4 days): Replace remaining 517 unwrap/expect calls focusing on critical paths
2. **Test Coverage** (3-5 days): Achieve 80% coverage target (currently 75%)
3. **Monitoring & Observability** (1 week): Implement OpenTelemetry tracing, system metrics, and health monitoring
4. **Resource Management** (3-4 days): Browser pooling, WASM build optimization, and memory management
5. **Core Integration** (1 week): WASM extractor wiring, dynamic rendering, session management, worker service

### Following Actions (Phase 3 Continuation):
After Phase 0 completion, resume Crawl4AI parity using the 6-PR strategy:

**Branch Setup**: Create `feature/phase3` branch for coordinated development

**PR Sequence (merge in order)**:
1. **PR-1**: Headless RPC v2 (actions/waits/scroll/sessions) - Feature flag OFF
2. **PR-2**: Stealth preset (UA rotation + JS evasion) - Feature flag OFF
3. **PR-3**: NDJSON streaming (/crawl/stream + /deepsearch/stream) - Feature flag ON
4. **PR-4**: PDF pipeline (pdfium text+metadata) - Feature flag ON, cap concurrency
5. **PR-5**: Spider integration (depth/budgets + sitemap) - Feature flag OFF
6. **PR-6**: Strategies & chunking (css/xpath/regex + 5 chunkers) - Feature flag ON, default TREK

**Rollout Strategy**:
1. Merge PR-1..3, enable streaming + PDF, keep headless_v2/stealth OFF
2. Canary headless_v2 on 10% traffic for 1 week, monitor metrics
3. Enable stealth, validate reduced challenge rate
4. Merge PR-5 (spider) OFF by default, test on staging
5. Merge PR-6, keep TREK+sliding defaults, enable advanced strategies per job

### Phase 3 Development Timeline (4-6 weeks):
1. **Week 1**: PR-1 (Headless RPC v2) + PR-2 (Stealth) development
2. **Week 2**: PR-3 (NDJSON Streaming) development and testing
3. **Week 3**: PR-4 (PDF Pipeline) with pdfium-render integration
4. **Week 4**: PR-5 (Spider Integration) with adaptive stopping
5. **Week 5**: PR-6 (Strategies & Chunking) with schema validation
6. **Week 6**: Integration testing, performance validation, rollout

### Month 2:
1. Production deployment and hardening
2. Load testing and optimization
3. Begin advanced feature development
4. Community feedback integration

---

*This roadmap will be updated as development progresses and priorities evolve based on user feedback and technical discoveries.*