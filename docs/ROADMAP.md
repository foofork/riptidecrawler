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

## 🌟 PHASE 3: Crawl4AI Parity Features (5 weeks)
*Priority: HIGH - Feature parity with Crawl4AI*

**Goal**: Complete feature parity with Crawl4AI for competitive positioning

### 3.1 Dynamic Content Handling (Week 1)
- [ ] **Enhanced `/render` endpoint**
  - `wait_for` conditions (CSS selectors, custom JS)
  - `scroll` configuration: steps, step_px, delay_ms
  - `actions` support: click, type, evaluate JS
  - Timeout handling and fallback to static

- [ ] **Artifacts & Capture**
  - Optional screenshot capture (PNG/WebP)
  - MHTML capture for complete page preservation
  - Page metadata collection (title, description, OG tags)

**Acceptance**: JS-heavy article (Next/React) → `wait_for` loads content → extracted text present → `scroll` loads lazy content → screenshot artifact saved when enabled

### 3.2 Deep Crawling & Site Discovery (Week 2)
- [ ] **Spider-rs integration**
  - Site-wide crawling with depth limits (max 3-5 levels)
  - Per-host budget controls (time, request limits)
  - Sitemap discovery and parsing (XML sitemaps)
  - Same-host filtering and URL normalization

- [ ] **Adaptive stopping**
  - Content quality scoring to stop early when goal met
  - Duplicate content detection to avoid redundant crawling
  - Link value scoring for priority queuing

**Acceptance**: Domain seed → crawler respects depth/budgets & robots → returns ≥N pages with per-page extraction + outlinks

### 3.3 Stealth & Anti-Detection (Week 3)
- [ ] **User-agent rotation**
  - Configurable UA list with realistic browser signatures
  - `--disable-blink-features=AutomationControlled` for headless
  - Randomized viewport sizes and screen resolutions

- [ ] **Request randomization**
  - Per-host delay jitter (±20% of base delay)
  - Header randomization (Accept-Language, Accept-Encoding)
  - Optional proxy support (HTTP/SOCKS) via environment variables

**Acceptance**: Repeat crawl to sensitive site completes without bot challenge in ≥80% of runs (small sample)

### 3.4 PDF Processing & Content Types (Week 4)
- [ ] **PDF pipeline with pdfium-render**
  - PDF text extraction and metadata parsing
  - Image extraction from PDFs with position data
  - Skip headless rendering for direct PDF URLs
  - Author, creation date, title metadata extraction

- [ ] **Multi-format content handling**
  - Content-type detection and routing
  - Word/PowerPoint basic text extraction (if feasible)
  - Archive file handling (ZIP with HTML/PDF contents)

**Acceptance**: Known PDFs → extract returns text+metadata → image count > 0 for illustrated docs

### 3.5 Output Formats & Streaming (Week 5)
- [ ] **NDJSON streaming**
  - Streaming output for `/crawl` and `/deepsearch`
  - Real-time progress updates during batch processing
  - Configurable batch sizes and flush intervals

- [ ] **Content chunking**
  - Token-aware chunks with configurable size/overlap
  - Preserve paragraph boundaries in chunks
  - Metadata preservation across chunks

- [ ] **Enhanced extraction**
  - Byline/date extraction from Open Graph and JSON-LD
  - Fallback heuristics for author and publication date
  - Mode switch: `article|generic|metadata` extraction
  - Consistent Markdown formatting across runs

**Acceptance**: Top-N URLs stream back record-by-record → chunks produced for long articles → links/media populated → byline/date captured in ≥80% where present

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
- [ ] Dynamic content: wait/scroll/actions + screenshot/MHTML working
- [ ] Deep crawl: seeds → budgets/depth honored, sitemap parsed, robots respected
- [ ] Stealth: UA rotation + stealth flags + proxy support, ≥80% no-challenge rate
- [ ] PDF: text + metadata + images extracted from PDF documents
- [ ] Streaming: NDJSON output, chunking, links/media lists populated
- [ ] Extraction: byline/date from OG/JSON-LD, consistent Markdown output

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
With Phase 1 fully completed, ready to proceed with Crawl4AI parity features:

1. **Dynamic Content Handling** - Enhanced `/render` endpoint with wait_for conditions
2. **Deep Crawling** - Spider-rs integration for site-wide discovery
3. **Stealth Features** - User-agent rotation and anti-detection measures
4. **PDF Processing** - pdfium-render integration for document extraction
5. **Output Formats** - NDJSON streaming and content chunking

### Phase 3 Development (4-6 weeks):
1. Week 1: Dynamic content handling and artifacts
2. Week 2: Deep crawling and site discovery
3. Week 3: Stealth and anti-detection
4. Week 4: PDF processing and multi-format content
5. Week 5: Output formats and streaming

### Month 2:
1. Production deployment and hardening
2. Load testing and optimization
3. Begin advanced feature development
4. Community feedback integration

---

*This roadmap will be updated as development progresses and priorities evolve based on user feedback and technical discoveries.*