# RipTide Crawler Development Roadmap

## Current Status Assessment

Based on the original RipTide specifications and current implementation state:

**✅ COMPLETED (Phase 0 - 100%)**:
- Complete project structure with workspace configuration (5 crates: riptide-core, riptide-api, riptide-headless, riptide-workers, riptide-extractor-wasm)
- Core crate foundations with complete types, fetch, extract, gate, and cache modules
- Headless service with full Chrome DevTools Protocol integration using chromiumoxide
- Docker infrastructure with production-ready Dockerfile.api, Dockerfile.headless, and docker-compose.yml
- CI/CD pipeline with GitHub Actions (fmt, clippy, tests, cargo-deny, docker builds)
- Complete documentation framework with architecture, API, and deployment guides
- WASM extractor implementation with Trek-rs integration and WASI command interface
- API handlers for /crawl, /deepsearch endpoints with models and business logic
- Configuration system with riptide.yml, policies.yml, fingerprints.yml
- Testing infrastructure: unit tests, golden tests, integration tests, and quality gates
- Redis caching integration with read-through cache patterns
- Full build system with scripts/build_all.sh and Justfile task runner

**🚀 READY FOR PHASE 1 (Next Steps)**:
- WASM Component Model migration (from WASI command to wasip2)
- Production optimization and monitoring
- Advanced features and Crawl4AI parity

---

## 🎯 PHASE 1: Core Foundation (2-3 weeks)
*Priority: CRITICAL - Required for MVP*

### 1.1 WASM Integration - CRITICAL BLOCKER
**Issues**: Trek-rs dependency version mismatch, placeholder WASM implementation

- [x] **Pin trek-rs dependency and target** ✅ COMPLETED
  - ✅ Pin to `trek-rs = "=0.2.1"` (confirmed available on crates.io)
  - ✅ Pin to `wasm32-wasip2` Component Model (Tier-2 support via rustup)
  - ✅ Remove any `wasm32-wasip1` references

- [x] **Implement Component Model (no WASI I/O)** ✅ COMPLETED
  - ✅ Add `wit/extractor.wit` with typed `extract(html, url, mode) -> ExtractedContent` function
  - ✅ Implement guest with `wit-bindgen` (no `_start` entrypoint)
  - ✅ Host: use `wasmtime::component::bindgen!` and call typed `extract()` function
  - ✅ Remove stdin/stdout piping and WASI command interface

- [x] **WASM-Core integration** ✅ COMPLETED
  - ✅ Replace extract.rs WASI command code with Component Model calls
  - ✅ Handle WASM errors and fallbacks gracefully with structured error types
  - ✅ Enhanced instance management with resource cleanup and performance monitoring

### 1.2 API Implementation - HIGH PRIORITY
**Issues**: Missing handler implementations, no health endpoints

- [x] **Complete API handlers** ✅ COMPLETED
  - ✅ Implement `/healthz` endpoint with comprehensive dependency checks
  - ✅ Complete `/crawl` batch processing logic with concurrent execution
  - ✅ Implement `/deepsearch` with Serper.dev integration
  - ✅ Add proper error handling and HTTP status codes (400, 401, 404, 408, 429, 500, 502, 503)

- [x] **Core business logic** ✅ COMPLETED
  - ✅ Implement gate.rs decision algorithm with content quality scoring
  - ✅ Connect fetch → gate → extract → render pipeline orchestration
  - ✅ Add caching layer with Redis integration and cache-first strategy
  - ✅ Wire all components together with state management and validation

### 1.3 Essential Testing - HIGH PRIORITY
**Issues**: No working test suite, validation failures

- [x] **Golden test suite** ✅ COMPLETED
  - ✅ Use offline fixtures in CI to avoid flaky tests
  - ✅ Create expected output JSONs for multiple content types (articles, SPAs, news)
  - ✅ Implement golden test runner for regression detection in `/tests/golden/`
  - ✅ Include property-based testing for edge case coverage

- [x] **Integration tests** ✅ COMPLETED
  - ✅ Create working e2e test suite in `tests/integration/`
  - ✅ Test full pipeline with offline fixtures and mocked services
  - ✅ Add comprehensive error scenario testing
  - ✅ Performance benchmarks and stress testing with 200+ concurrent requests

---

## 🚀 PHASE 2 LITE: Minimal Reliability Foundation (2-4 days) ✅ COMPLETED
*Priority: HIGH - Essential scaffolding before Crawl4AI parity work*

**Goal**: Minimal production-readiness to enable safe Phase-3 development

### 2.1 Metrics & Health (1 day) ✅ COMPLETED
- [x] **Prometheus `/metrics` endpoint** ✅ COMPLETED
  - ✅ Added `axum-prometheus` middleware to API + headless
  - ✅ Request counters, duration histograms (p50/p95)
  - ✅ Export bucket config for performance monitoring

- [x] **Enhanced `/healthz` endpoint** ✅ COMPLETED
  - ✅ Report `git_sha`, `wit=riptide:extractor@version`, `trek=version`
  - ✅ Component status checks (Redis, WASM, headless)
  - ✅ Build metadata and dependency versions

- [x] **Phase timing logs** ✅ COMPLETED
  - ✅ Log phase timings: `fetch_ms`, `gate_ms`, `wasm_ms`, `render_ms`
  - ✅ Structured logging for performance analysis

### 2.2 Timeouts & Circuit Breakers (1 day) ✅ COMPLETED
- [x] **Fetch reliability** ✅ COMPLETED
  - ✅ Connect timeout: 3s, total timeout: 15-20s
  - ✅ 1 retry for idempotent requests only
  - ✅ Proper error propagation and fallback

- [x] **Headless resilience** ✅ COMPLETED
  - ✅ `DOMContentLoaded + 1s idle`, hard cap 3s
  - ✅ Circuit breaker on consecutive headless failures
  - ✅ Graceful degradation to fast path when headless fails

### 2.3 Robots & Throttling (1 day) ✅ COMPLETED
- [x] **Robots.txt compliance** ✅ COMPLETED
  - ✅ Parse `robots.txt` using `robotstxt` crate
  - ✅ Toggle to bypass in development mode
  - ✅ Per-host robots.txt cache with TTL

- [x] **Per-host throttling** ✅ COMPLETED
  - ✅ Token bucket per host (1-2 RPS default)
  - ✅ Configurable delay/throttling based on robot rules
  - ✅ Jitter (±20%) to avoid request pattern detection

### 2.4 Cache & Input Hardening (1 day) ✅ COMPLETED
- [x] **Redis read-through + TTL** ✅ COMPLETED
  - ✅ Cache key includes extractor version/options
  - ✅ TTL 24h default, configurable per content type
  - ✅ Respect `ETag`/`Last-Modified` with conditional GET

- [x] **Input validation** ✅ COMPLETED
  - ✅ URL validation and content-type allowlist
  - ✅ Max bytes limit (20MB default)
  - ✅ CORS and header size limits
  - ✅ XSS/injection protection

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

### 🚀 PR-1: Headless RPC v2 - Dynamic Content (Week 1)
**Branch**: `feature/phase3-pr1-headless-v2`
**Feature Flag**: `headless_v2: false` (default OFF)

**Files**: `crates/riptide-headless/src/models.rs`, `cdp.rs`

- [ ] **Enhanced RenderRequest model**
```rust
#[derive(Deserialize)]
pub struct RenderRequest {
    pub session_id: Option<String>,    // Persistent browser sessions
    pub url: String,
    pub actions: Option<Vec<PageAction>>,  // Interactive page actions
    pub timeouts: Option<Timeouts>,    // Configurable timing
    pub artifacts: Option<Artifacts>,  // Screenshot/MHTML capture
}

#[derive(Deserialize)]
#[serde(tag="type", rename_all="snake_case")]
pub enum PageAction {
    WaitForCss { css: String, timeout_ms: Option<u64> },
    WaitForJs { expr: String, timeout_ms: Option<u64> },
    Scroll { steps: u32, step_px: u32, delay_ms: u64 },
    Js { code: String },
    Click { css: String },
    Type { css: String, text: String, delay_ms: Option<u64> },
}
```

- [ ] **Action executor implementation**
```rust
async fn exec_actions(page: &Page, actions: &[PageAction]) -> anyhow::Result<()> {
    for action in actions {
        match action {
            PageAction::WaitForCss{css, timeout_ms} => {
                page.wait_for_element_with_timeout(css, timeout_ms.unwrap_or(5000)).await?;
            }
            PageAction::Scroll{steps, step_px, delay_ms} => {
                for _ in 0..*steps {
                    page.evaluate(&format!("window.scrollBy(0,{step_px});")).await.ok();
                    tokio::time::sleep(Duration::from_millis(*delay_ms)).await;
                }
            }
            // ... other action implementations
        }
    }
    Ok(())
}
```

- [ ] **Session management**: Map `session_id -> user-data-dir` for cookie persistence
- [ ] **Artifacts capture**: Optional screenshot/MHTML (base64 encoded)
- [ ] **API passthrough**: Forward `RenderRequest` when `features.headless_v2=true`

**Acceptance**: JS-heavy article (Next/React) → `wait_for` loads content → `scroll` loads lazy content → screenshot artifact captured

### 🚀 PR-2: Stealth Preset - Anti-Detection (Week 2)
**Branch**: `feature/phase3-pr2-stealth`
**Feature Flag**: `stealth: false` (default OFF)

**Files**: `riptide-headless/src/launcher.rs`, `stealth.js`

- [ ] **Launch flags**: `--disable-blink-features=AutomationControlled --no-first-run --mute-audio --headless=new`
- [ ] **UA rotation**: Load `configs/ua_list.txt`, pick per `session_id` (stable hash)
- [ ] **JS injection** (early page lifecycle):
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

### Phase 1 (MVP) Success Criteria: ✅ COMPLETED
- [x] All critical APIs functional (`/crawl`, `/deepsearch`, `/healthz`) ✅ COMPLETED
- [x] WASM extraction working with trek-rs ✅ COMPLETED - Component Model migration with performance optimization
- [x] Golden tests passing (comprehensive test suite with fixtures) ✅ COMPLETED
- [x] Technical debt resolution (compilation errors, performance, monitoring) ✅ COMPLETED
- [x] Docker deployment working end-to-end ✅ COMPLETED - Simple setup validated with docker-compose.yml
- [x] Basic load testing (100 concurrent requests, <2s p95) ✅ COMPLETED - Comprehensive script with 100 concurrent support

### Phase 2 Lite Success Criteria: ✅ COMPLETED
- [x] `/healthz` + `/metrics` endpoints reporting build info and performance ✅ COMPLETED
  - ✅ Prometheus metrics with axum-prometheus middleware
  - ✅ Enhanced health checks with component status (Redis, WASM, headless)
  - ✅ Git SHA, WIT version, and Trek version reporting
- [x] Timeouts, retries, and circuit breaker preventing cascading failures ✅ COMPLETED
  - ✅ Fetch timeouts (3s connect, 15-20s total) with retry logic
  - ✅ Headless circuit breaker with graceful degradation
- [x] Robots.txt compliance and per-host throttling active ✅ COMPLETED
  - ✅ Robotstxt crate integration with per-host caching
  - ✅ Token bucket throttling with jitter (±20%)
- [x] Redis read-through cache with TTL and conditional GET support ✅ COMPLETED
  - ✅ Cache keys include extractor version and options
  - ✅ 24h TTL with configurable per content type
  - ✅ ETag/Last-Modified conditional GET implementation
- [x] Input validation and security hardening in place ✅ COMPLETED
  - ✅ URL validation with content-type allowlisting
  - ✅ 20MB max bytes limit and CORS protection
  - ✅ XSS/injection protection implemented

### Phase 3 (Crawl4AI Parity) Success Criteria:
**PR-1 (Headless RPC v2)**:
- [ ] Enhanced RenderRequest with actions, timeouts, artifacts
- [ ] Page actions: wait_for_css, wait_for_js, scroll, click, type
- [ ] Session management with persistent browser instances
- [ ] Screenshot/MHTML artifact capture (base64 encoded)

**PR-2 (Stealth Preset)**:
- [ ] Launch flags: `--disable-blink-features=AutomationControlled`
- [ ] UA rotation from configurable list with stable session hashing
- [ ] JS injection for navigator.webdriver spoofing and fingerprint noise
- [ ] ≥80% success rate on bot-detection sites

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

### Phase 2 (Production) Success Criteria:
- [ ] 99.9% uptime over 30 days
- [ ] <500ms p95 response time for simple crawls
- [ ] Handles 1000+ requests/minute sustained load
- [ ] Comprehensive monitoring and alerting
- [ ] Security audit passed

### Phase 3 (Advanced) Success Criteria:
- [ ] Crawl4AI feature parity (90% compatibility)
- [ ] PDF processing capability
- [ ] Stealth mode effectiveness (>80% success rate)
- [ ] Advanced content extraction accuracy (>90%)

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
| **Phase 1** | 2-3 weeks | Working MVP with basic crawling | HIGH | ✅ COMPLETED |
| **Phase 2 Lite** | 2-4 days | Minimal reliability foundation | MEDIUM | ✅ COMPLETED |
| **Phase 3** | 4-6 weeks | Advanced features and parity | MEDIUM | 🚀 READY TO START |
| **Phase 4** | 6-8 weeks | Enterprise capabilities | LOW | PLANNED |
| **Phase 5** | Ongoing | Continuous optimization | LOW | PLANNED |

**Total Estimated Timeline**: 4-6 months for full implementation

---

## 🤝 Next Steps

### Immediate Actions (Next Steps - Phase 3):
With Phase 1 & 2 fully completed, ready to proceed with Crawl4AI parity using the 6-PR strategy:

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