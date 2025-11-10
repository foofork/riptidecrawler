# Roadmap Addendum: Deferred Work

**Purpose**: Document work identified during analysis but NOT included in 16-week refactoring roadmap
**Status**: DEFERRED - To be addressed after Sprint 16 completion

---

## Overview

The initial comprehensive analysis (Week 0 research) identified multiple improvement opportunities across browser pooling, extraction capabilities, and streaming support. However, the 16-week roadmap intentionally focuses ONLY on:

- Hexagonal architecture compliance (24% → 95%)
- AppState god object removal
- Port-adapter pattern implementation
- Testing and production readiness

**This addendum catalogs the work that was analyzed but intentionally deferred** to keep the roadmap focused and achievable.

---

## Scope Boundaries

### IN SCOPE (16-week roadmap)
✅ Hexagonal architecture refactoring
✅ ApplicationContext creation
✅ AppState removal
✅ Port-adapter implementation
✅ Facade migration to ports
✅ Testing to 90%+ coverage
✅ Production deployment

### OUT OF SCOPE (deferred to post-Sprint 16)
❌ Browser pooling enhancements
❌ Multi-browser support (Firefox, WebKit)
❌ Extraction capability improvements
❌ Streaming enhancements
❌ RiptideRuntime integration
❌ Feature additions (new functionality)
❌ Performance optimizations (beyond refactoring needs)

---

## Category 1: Browser Pooling & Multi-Browser Support

### Current State (from initial analysis)

**Findings**:
- ✅ **Chromium support**: Production-ready, 21/24 tests passing
- ✅ **Multi-Chromium pooling**: Works well (health monitoring, resource limits)
- ✅ **CDP connection pooling**: 70%+ connection reuse achieved
- ❌ **Firefox support**: Missing entirely
- ❌ **WebKit support**: Missing entirely
- ❌ **3 failing tests**: CDP batch execution issues

**Analysis Documents**:
- `docs/architecture/browser-pooling-analysis.md`
- `docs/research/extraction-capabilities-analysis.md`

### Deferred Work Items

#### 1.1: Multi-Browser Support (Firefox)
**Effort**: 3-4 weeks
**Priority**: P1 (high value for cross-browser testing)

**Tasks**:
- [ ] Create `FirefoxBrowserAdapter` implementing `BrowserDriver` port
- [ ] Firefox CDP protocol differences (devtools protocol variations)
- [ ] Firefox binary detection and version management
- [ ] Firefox-specific browser pool configuration
- [ ] Firefox headless mode support
- [ ] Integration tests for Firefox adapter

**Dependencies**:
- ✅ BrowserDriver port exists (created in Sprint 2)
- Requires: Firefox binary, geckodriver

**ROI**: Enables cross-browser testing, broader market coverage

#### 1.2: Multi-Browser Support (WebKit)
**Effort**: 3-4 weeks
**Priority**: P2 (lower priority than Firefox)

**Tasks**:
- [ ] Create `WebKitBrowserAdapter` implementing `BrowserDriver` port
- [ ] WebKit/Safari technology preview integration
- [ ] Playwright WebKit backend (if using Playwright)
- [ ] macOS-specific challenges (Safari requires macOS)
- [ ] Integration tests for WebKit adapter

**Dependencies**:
- ✅ BrowserDriver port exists
- Requires: macOS environment or WebKit GTK on Linux

**ROI**: Safari testing coverage, iOS simulation

#### 1.3: Fix CDP Batch Execution Tests
**Effort**: 1-2 days
**Priority**: P0 (failing tests)

**Failing Tests** (from `crates/riptide-browser/`):
1. `test_batch_execute_with_commands`
2. `test_batch_config_disabled`
3. `test_browser_checkout_checkin`

**Root Cause Analysis Needed**:
- [ ] Investigate timing issues in batch execution
- [ ] Check browser lifecycle management
- [ ] Verify CDP command queuing
- [ ] Test under different load conditions

**Why Deferred**: Not blocking refactoring, can be fixed post-Sprint 16

#### 1.4: Browser Pool Enhancements
**Effort**: 2 weeks
**Priority**: P2 (nice-to-have)

**Enhancements Identified**:
- [ ] Dynamic pool sizing based on load
- [ ] Browser version pinning (prevent auto-updates mid-crawl)
- [ ] Pool warmup on startup (pre-launch browsers)
- [ ] Browser recycling after N requests (prevent memory leaks)
- [ ] Pool metrics dashboard (checkout/checkin rates, wait times)
- [ ] Connection pool optimization (reduce 70% → 85%+ reuse)

**Why Deferred**: Current pooling works, optimizations can wait

---

## Category 2: Extraction Capabilities

### Current State (from initial analysis)

**Findings**:
- ✅ **Screenshots**: Working (browser-based)
- ✅ **PDFs**: Working (browser-based rendering)
- ✅ **HTML extraction**: Working
- ⚠️ **Markdown extraction**: Scattered across multiple modules (needs consolidation)
- ❌ **Image download**: Missing (identified in gap analysis)
- ⚠️ **JSON markdown format**: Not standardized

**Analysis Documents**:
- `docs/research/extraction-capabilities-analysis.md`
- Gap analysis identified 14 capability matrices

### Deferred Work Items

#### 2.1: Consolidate Markdown Extraction
**Effort**: 1-2 weeks
**Priority**: P1 (high value, scattered code is technical debt)

**Current State**:
- Markdown extraction code exists in 4+ locations:
  - `crates/riptide-facade/src/extractor.rs`
  - `crates/riptide-browser/src/extraction/`
  - `crates/riptide-api/src/handlers/extract.rs`
  - Possibly more (needs audit)

**Tasks**:
- [ ] Audit all markdown extraction code locations
- [ ] Create unified `MarkdownExtractor` port trait
- [ ] Implement single canonical adapter (html2text or pulldown-cmark)
- [ ] Migrate all facades to use unified extractor
- [ ] Standardize markdown output format
- [ ] Tests for all markdown extraction scenarios

**Why Deferred**: Not blocking refactoring, but should be done soon after

#### 2.2: Image Download Support
**Effort**: 1 week
**Priority**: P2 (nice-to-have)

**Missing Functionality**:
- Download images referenced in HTML (`<img src="...">`)
- Store images locally or in blob storage
- Return image URLs in extraction results
- Handle image formats (PNG, JPG, WebP, SVG)
- Lazy loading support (images loaded via JS)

**Tasks**:
- [ ] Create `ImageDownloader` port trait
- [ ] Implement adapter (using `reqwest` or browser CDP)
- [ ] Wire into `ExtractorFacade`
- [ ] Add image metadata (dimensions, format, size)
- [ ] Tests for image download scenarios

**Why Deferred**: Not core functionality, can add later

#### 2.3: JSON Markdown Format Standardization
**Effort**: 3-5 days
**Priority**: P1 (was mentioned as "key format" in initial request)

**Current State**:
- JSON output exists but not standardized
- Markdown format varies by extractor

**Tasks**:
- [ ] Define canonical JSON markdown schema
- [ ] Document schema (OpenAPI/JSON Schema)
- [ ] Update all extractors to output standard format
- [ ] Version the schema (v1, v2, etc.)
- [ ] Add schema validation

**Example Target Schema**:
```json
{
  "version": "1.0",
  "url": "https://example.com",
  "extracted_at": "2025-11-10T12:00:00Z",
  "content": {
    "markdown": "# Title\n\nParagraph...",
    "html": "<h1>Title</h1><p>Paragraph...</p>",
    "text": "Title\nParagraph...",
    "metadata": {
      "title": "Page Title",
      "description": "Meta description",
      "author": "Author Name"
    }
  },
  "media": {
    "images": [
      {"url": "https://example.com/image.jpg", "alt": "Alt text"}
    ],
    "videos": [],
    "pdfs": []
  }
}
```

**Why Deferred**: Format standardization important but not blocking refactoring

#### 2.4: Non-Browser Extraction Modes
**Effort**: 2-3 weeks
**Priority**: P1 (user explicitly asked about "with and without full browser")

**Gap Identified**:
- Current extraction heavily browser-dependent
- Need lightweight extraction for simple HTML (no JS rendering)

**Tasks**:
- [ ] Create `LightweightExtractor` adapter (reqwest + html scraping)
- [ ] Create `HeavyweightExtractor` adapter (full browser CDP)
- [ ] Auto-select extractor based on URL complexity
- [ ] Configuration to force lightweight/heavyweight mode
- [ ] Performance comparison (lightweight 10x faster for static pages)

**Use Cases**:
- **Lightweight**: Static HTML pages, blogs, documentation
- **Heavyweight**: SPAs, dynamic content, JS-rendered pages

**Why Deferred**: Browser extraction works, optimization can wait

---

## Category 3: Streaming Support

### Current State (from initial analysis)

**Findings**:
- ✅ **StreamingFacade exists**: 1464 LOC in `crates/riptide-facade/src/streaming.rs`
- ✅ **Basic streaming**: BoxStream for crawl results
- ⚠️ **Limited use cases**: Mostly internal, not exposed to API
- ❌ **Missing**: Event-driven streaming patterns
- ❌ **Missing**: WebSocket streaming API

**Analysis Documents**:
- Initial analysis identified streaming as "minimal implementation"

### Deferred Work Items

#### 3.1: Event-Driven Streaming Patterns
**Effort**: 2-3 weeks
**Priority**: P2 (nice-to-have)

**Current Gap**:
- Streaming exists but not event-driven
- No pub/sub pattern for crawl events

**Tasks**:
- [ ] Implement event bus for crawl lifecycle events
  - `CrawlStarted`
  - `PageDiscovered`
  - `PageCrawled`
  - `ExtractionComplete`
  - `CrawlFinished`
- [ ] Wire event bus into `StreamingFacade`
- [ ] Create event subscribers (logging, metrics, notifications)
- [ ] Add event filtering (subscribe to specific event types)
- [ ] Tests for event propagation

**Why Deferred**: Current streaming functional, events are enhancement

#### 3.2: WebSocket Streaming API
**Effort**: 1-2 weeks
**Priority**: P2 (nice-to-have)

**Missing Functionality**:
- Real-time crawl progress via WebSocket
- Live updates as pages are crawled
- Frontend integration for live dashboards

**Tasks**:
- [ ] Add `axum::extract::ws::WebSocket` support
- [ ] Create `/api/v1/stream/crawl` WebSocket endpoint
- [ ] Stream events as JSON over WebSocket
- [ ] Handle client disconnections gracefully
- [ ] Tests for WebSocket streaming

**Why Deferred**: Not in current requirements, can add later

#### 3.3: Streaming Response Pagination
**Effort**: 3-5 days
**Priority**: P2 (nice-to-have)

**Enhancement**:
- Stream large result sets in chunks
- Server-sent events (SSE) for HTTP/2
- Backpressure handling

**Why Deferred**: Current pagination works, streaming is optimization

---

## Category 4: State System Unification

### Current State (from initial analysis)

**Critical Finding**:
- ⚠️ **THREE competing state systems exist**:
  1. `AppState` (2213 LOC, 40+ fields) - being removed in Sprint 10
  2. `ApplicationContext` (created in Sprint 1) - port-based DI
  3. `RiptideRuntime` (mentioned in analysis, not in roadmap)

**QUESTION**: How does `RiptideRuntime` relate to the refactoring?

### Deferred Work Items

#### 4.1: RiptideRuntime Analysis
**Effort**: 1 week (investigation)
**Priority**: P0 (CRITICAL - needs investigation)

**Questions to Answer**:
- [ ] Where is `RiptideRuntime` defined?
- [ ] What is its relationship to `AppState`?
- [ ] What is its relationship to `ApplicationContext`?
- [ ] Is it being used in production?
- [ ] Should it be removed, merged, or kept separate?

**Action Required**:
```bash
# Find RiptideRuntime
grep -r "RiptideRuntime" crates/ --include="*.rs"

# Analyze usage
cargo tree -p riptide-api --depth 3 | grep -i runtime

# Review architecture
# Determine: Is RiptideRuntime part of the runtime infrastructure?
#            Or is it another state container?
```

**Why Deferred**: Not mentioned in roadmap, needs investigation post-Sprint 16

#### 4.2: State System Unification Plan
**Effort**: TBD (depends on 4.1 investigation)
**Priority**: P0 (if RiptideRuntime conflicts with ApplicationContext)

**Potential Scenarios**:

**Scenario A**: RiptideRuntime is infrastructure runtime (no conflict)
- Keep both: ApplicationContext for DI, RiptideRuntime for runtime
- Document separation of concerns

**Scenario B**: RiptideRuntime is another state container (CONFLICT)
- Merge with ApplicationContext
- Remove duplicate functionality
- Migrate usages

**Scenario C**: RiptideRuntime is deprecated
- Remove entirely
- Migrate to ApplicationContext

**Why Deferred**: Need investigation (4.1) first before planning

---

## Category 5: Performance Optimizations

### Current State

**Identified but Not Prioritized**:
- Composition benchmarks exist (`crates/riptide-facade/benches/composition_benchmarks.rs`)
- Baseline performance captured in Week 0
- No degradation expected from refactoring (zero-cost abstractions)

### Deferred Work Items

#### 5.1: Facade Performance Optimization
**Effort**: 1-2 weeks
**Priority**: P2 (optimization, not blocker)

**Potential Optimizations**:
- [ ] Profile facade method calls (flamegraphs)
- [ ] Reduce Arc cloning (use references where possible)
- [ ] Optimize hot paths (crawl, extract)
- [ ] Batch database queries (N+1 query prevention)
- [ ] Cache frequently accessed data
- [ ] Connection pool sizing tuning

**Baseline** (from Week 0):
- Baseline stream (1000 items): [TBD]ms
- BoxStream (1000 items): [TBD]ms
- Composed stream (1000 items): [TBD]ms

**Target**: <10% overhead vs baseline

**Why Deferred**: Refactoring shouldn't degrade performance; optimize after if needed

#### 5.2: Database Query Optimization
**Effort**: 1 week
**Priority**: P2 (optimization)

**Tasks**:
- [ ] Run EXPLAIN ANALYZE on all queries
- [ ] Add missing indexes
- [ ] Optimize slow queries (>100ms)
- [ ] Consider query result caching
- [ ] Connection pool tuning

**Why Deferred**: Not blocking refactoring

---

## Category 6: Feature Additions (New Functionality)

### Deferred Features

#### 6.1: Advanced Spider Features
**Effort**: 2-4 weeks
**Priority**: P2 (nice-to-have)

**Features Identified**:
- [ ] Sitemap.xml parsing and crawling
- [ ] Robots.txt respect and configuration
- [ ] Link depth limiting (crawl N levels deep)
- [ ] Domain whitelisting/blacklisting
- [ ] Crawl politeness (delays between requests)
- [ ] User-agent rotation
- [ ] Proxy support
- [ ] Distributed crawling (multiple workers)

**Why Deferred**: Not in current requirements

#### 6.2: Extraction Format Support
**Effort**: 1-2 weeks per format
**Priority**: P2 (nice-to-have)

**Additional Formats**:
- [ ] DOCX extraction
- [ ] XLSX extraction
- [ ] PPTX extraction
- [ ] EPUB extraction
- [ ] Video metadata extraction
- [ ] Audio transcription

**Why Deferred**: PDF, HTML, markdown cover most use cases

---

## Post-Sprint 16 Roadmap Recommendations

### Phase 5: Browser & Extraction Enhancements (Weeks 17-20)

**Duration**: 4 weeks
**Goal**: Complete browser pooling and extraction capabilities

**Sprint 17**: Multi-browser support (Firefox)
- Create FirefoxBrowserAdapter
- Integration tests
- Fix 3 failing CDP batch tests

**Sprint 18**: Extraction consolidation
- Consolidate markdown extraction
- Standardize JSON markdown format
- Image download support

**Sprint 19**: Non-browser extraction modes
- Lightweight extractor (reqwest-based)
- Auto-selection logic
- Performance benchmarks

**Sprint 20**: Streaming enhancements
- Event-driven patterns
- WebSocket API
- Pagination

**Deliverables**:
- Multi-browser support (Firefox + WebKit)
- Unified extraction pipeline
- JSON markdown v1.0 schema
- Real-time streaming API

### Phase 6: State System Unification (Week 21)

**Duration**: 1 week (investigation + planning)
**Goal**: Understand RiptideRuntime and create unification plan

**Tasks**:
- Investigate RiptideRuntime usage
- Determine relationship to ApplicationContext
- Create unification plan (if needed)
- Document state system architecture

**Deliverables**:
- RiptideRuntime analysis report
- State unification plan (if needed)
- Updated architecture diagrams

### Phase 7: Performance & Production Hardening (Weeks 22-24)

**Duration**: 3 weeks
**Goal**: Optimize performance and add enterprise features

**Sprint 22**: Performance optimization
- Facade profiling
- Database query optimization
- Cache tuning
- Benchmark validation

**Sprint 23**: Advanced spider features
- Sitemap.xml support
- Robots.txt respect
- Distributed crawling

**Sprint 24**: Enterprise features
- Additional extraction formats
- Advanced configuration
- Multi-tenant isolation

**Deliverables**:
- 10x performance improvement (lightweight extraction)
- Advanced crawling capabilities
- Enterprise-ready feature set

---

## Effort Summary

### Total Deferred Work: 22-30 weeks

| Category | Effort | Priority | Dependencies |
|----------|--------|----------|--------------|
| Browser (Multi-browser) | 6-8 weeks | P1 | Sprint 16 complete |
| Extraction (Consolidation) | 3-5 weeks | P1 | Sprint 16 complete |
| Streaming (Enhancements) | 3-5 weeks | P2 | Sprint 16 complete |
| State Unification | 1-2 weeks | P0 (investigation) | Sprint 16 complete |
| Performance | 2-3 weeks | P2 | Sprint 16 complete |
| Features (Advanced) | 4-6 weeks | P2 | All above complete |
| **TOTAL** | **22-30 weeks** | | |

### Recommended Sequencing

**Immediately After Sprint 16** (P0):
1. RiptideRuntime investigation (1 week) - CRITICAL
2. Fix 3 failing CDP tests (2 days) - Quick win

**Phase 5** (P1 - High Value):
3. Multi-browser support (4 weeks)
4. Extraction consolidation (3 weeks)

**Phase 6** (P2 - Nice-to-Have):
5. Streaming enhancements (3 weeks)
6. Performance optimization (2 weeks)

**Phase 7** (P2 - Future):
7. Advanced features (4-6 weeks)

---

## Dependencies on 16-Week Roadmap

### What Must Complete First

**Before ANY deferred work can start**:
- ✅ Sprint 16: Production deployment successful
- ✅ All quality gates passed
- ✅ Hexagonal compliance 95%+
- ✅ Zero infrastructure violations
- ✅ Feature flags removed (production stable)

**Why**: Deferred work adds NEW functionality. Must have stable foundation first.

### Ports Created During Refactoring (Usable for Deferred Work)

**Available Ports** (Sprint 1-6):
- ✅ `BrowserDriver` - Used for multi-browser support
- ✅ `HttpClient` - Used for lightweight extraction
- ✅ `SearchEngine` - Created in Sprint 4 (already done!)
- ✅ `PdfProcessor` - Created in Sprint 4 (already done!)
- ✅ `IdempotencyStore` - Can be used for crawl deduplication
- ✅ `CircuitBreaker` - Can be used for resilience
- ✅ `RateLimiter` - Can be used for crawl politeness

**Benefit**: Deferred work can immediately use clean port abstractions!

---

## Risk Assessment

### Risks of Deferring This Work

**Low Risk**:
- Multi-browser support (Firefox, WebKit) - Chromium works fine
- Streaming enhancements - Current streaming adequate
- Performance optimizations - No degradation from refactoring
- Advanced features - Nice-to-have, not required

**Medium Risk**:
- Extraction consolidation - Scattered code is technical debt
- JSON markdown standardization - Format inconsistency
- 3 failing tests - Small test failures

**High Risk**:
- **RiptideRuntime investigation** - Unknown relationship to ApplicationContext
- State system confusion - Three systems is confusing

### Mitigation

**For High Risk Items**:
- Investigate RiptideRuntime immediately after Sprint 16
- Document state system architecture clearly
- Create ADR for state system decisions

**For Medium Risk Items**:
- Add to Phase 5 roadmap (weeks 17-20)
- Don't defer indefinitely

**For Low Risk Items**:
- Nice-to-have, defer to Phase 7 or beyond

---

## Questions for Stakeholders

### Strategic Questions

1. **Multi-browser support**: Is Firefox/WebKit support required for your use cases?
   - If YES: Prioritize in Phase 5
   - If NO: Defer to Phase 7 or later

2. **Extraction formats**: Which formats are most important?
   - Markdown, HTML, PDF (done)
   - Images (deferred)
   - DOCX, XLSX (deferred)
   - Video/audio (deferred)

3. **Streaming**: Is real-time streaming required?
   - WebSocket API (deferred)
   - Event-driven patterns (deferred)
   - Current batch processing sufficient?

4. **Performance**: What are performance requirements?
   - Current: [baseline]
   - Target: [?]
   - If performance critical, prioritize Phase 7

### Technical Questions

5. **RiptideRuntime**: What is its purpose and relationship to ApplicationContext?
   - INVESTIGATION REQUIRED (post-Sprint 16)

6. **JSON markdown format**: Is there a required schema?
   - If YES: Provide spec, implement in Phase 5
   - If NO: Can standardize in Phase 5

---

## Conclusion

**Summary**:
- **22-30 weeks of deferred work** identified
- **NOT included in 16-week roadmap** (intentionally focused)
- **Should be addressed** in Phases 5-7 (post-Sprint 16)
- **RiptideRuntime investigation** is P0 (critical unknown)

**Recommendation**:
1. **Complete Sprint 16 first** - Get to production with hexagonal architecture
2. **Investigate RiptideRuntime** - Resolve state system confusion (Week 17)
3. **Execute Phase 5** - Browser + extraction enhancements (Weeks 17-20)
4. **Execute Phase 6** - State unification if needed (Week 21)
5. **Execute Phase 7** - Performance + features (Weeks 22-24)

**Total Timeline** (including deferred work):
- **16 weeks**: Hexagonal refactoring (current roadmap)
- **22-30 weeks**: Deferred enhancements (this addendum)
- **38-46 weeks total**: Complete browser enhancement + refactoring

**Next Steps**:
1. Review this addendum with stakeholders
2. Prioritize deferred work (P0, P1, P2)
3. Create Phase 5-7 roadmaps after Sprint 16
4. Investigate RiptideRuntime immediately after production deployment

---

**Version**: 1.0
**Last Updated**: 2025-11-10
**Status**: Deferred work catalog - To be addressed post-Sprint 16
**Related**: ROADMAP-OVERVIEW.md, ROADMAP-PHASE-4.md
