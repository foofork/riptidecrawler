Absolutely — here’s a **single, consolidated roadmap** that **replaces** the previous draft **without losing a single item**. I kept every task, metric, flag, risk, and acceptance criterion you listed, and reorganized them by **priority and execution order**. At the end there’s a brief **crosswalk** so you can see where each original section landed.

---

# RipTide Crawler — Consolidated & Prioritized Roadmap (Supersedes Prior Draft)

## 0) Snapshot (Updated: 2025-09-24)

* **✅ Done:** Phase 0 (Foundation), Phase 1 (Core), Phase 2-Lite (Reliability), Phase 3 PR-1 (Headless RPC v2), PR-2 (Stealth)
* **✅ Session Management:** Session manager implementation complete and functional
* **✅ Spider Module:** All compilation errors resolved, ready for integration
* **✅ Strategies Module:** All 14 strategy files implemented and compiling
* **✅ PDF Processing:** Module complete with processor, config, and tests
* **📍 Now:** **Critical Integration Gap - Browser Pool & Streaming Pipeline disconnected**
* **⚡ IN PROGRESS:** Error handling cleanup (424 unwrap/expect remaining, down from 517)
* **⚡ IN PROGRESS:** API streaming endpoints (NDJSON implementation in riptide-api)
* **🔴 CRITICAL:** Browser Pool, Streaming Pipeline, and Session System not wired to handlers
* **🧭 Guardrails:** Feature flags, Prometheus metrics, strict timeouts/pools
* **📜 Reference:** See `COMPLETED.md` for all shipped work.

---

## 1) Critical Path (do in this order)

### **🔴 URGENT: Integration Gaps (Based on Unused Code Analysis)**

**Critical unused components that MUST be wired:**
1. **Browser Pool System** - BrowserPool, LaunchSession, ResourceGuard never constructed
2. **Streaming Pipeline** - StreamProcessor, StreamingModule, StreamingPipeline disconnected
3. **Session System** - SessionSystem, SessionHeaders implemented but not integrated
4. **Performance Monitoring** - PerformanceMonitor, metrics collection unused
5. **Deep Search** - DeepSearchMetadata, DeepSearchResultData structs never used

### 1.0 Browser Pool Integration — **✅ COMPLETED**

* **Issue:** BrowserPool, BrowserPoolRef, LaunchSession never instantiated
* **Impact:** Headless rendering non-functional without browser management
* **Fix Completed:**
  * ✅ Wired BrowserPool from riptide-headless into ResourceManager
  * ✅ Connected BrowserCheckout to render resource acquisition
  * ✅ Implemented proper lifecycle management with automatic checkin
  * ✅ Added pool statistics tracking via get_stats()
* **Files Updated:** `resource_manager.rs`, `config.rs`, `riptide-headless/lib.rs`, `pool.rs`
* **Status:** Fully integrated and compiling without errors

### 1.1 Streaming Pipeline Integration — **✅ COMPLETED**

* **Issue:** StreamProcessor, StreamingModule, StreamingPipeline never constructed
* **Impact:** NDJSON streaming endpoints non-functional
* **Fix Completed:**
  * ✅ StreamingModule initialized in API startup (state.rs:221-233)
  * ✅ StreamProcessor wired to `/crawl/stream` and `/deepsearch/stream` endpoints
  * ✅ Pipeline connected to buffer management and backpressure handling
  * ✅ Stream lifecycle methods implemented with maintenance tasks
  * ✅ Health checks integrated for streaming status monitoring
* **Files Updated:** `state.rs`, `streaming/mod.rs`, `streaming/processor.rs`
* **Status:** Fully integrated and functional with lifecycle management

### 1.2 Session System Wiring — **✅ COMPLETED**

* **Issue:** SessionSystem, SessionManager implemented but not connected to handlers
* **Impact:** No session persistence, cookies lost between requests
* **Fix Completed:**
  * ✅ SessionManager initialized in state.rs (line 217)
  * ✅ SessionLayer middleware imported and available (main.rs:18)
  * ✅ All session endpoints wired (/sessions, /sessions/stats, /sessions/cleanup, etc.)
  * ✅ Session cleanup endpoint implemented at /sessions/cleanup
* **Files Updated:** `state.rs`, `main.rs`, `sessions/manager.rs`, `handlers/sessions.rs`
* **Status:** Fully integrated with all endpoints functional

### 1.3 Core WASM & Rendering — **✅ COMPLETED**

* **WASM Extractor Integration**
  * ✅ WASM extractor fully integrated via `extract_with_wasm_extractor` function
  * ✅ Trek-rs extractor properly wired in render pipeline (render.rs:762)
  * ✅ Full error handling and timing metrics implemented
* **Dynamic Rendering Implementation**
  * ✅ Dynamic rendering via RPC client fully functional (render.rs:508)
  * ✅ Browser Pool integrated via ResourceManager (resource_manager.rs:247)
  * ✅ Browser checkout/checkin lifecycle properly managed
* **Status:** WASM extraction and dynamic rendering fully operational

### 1.4 Eliminate Panics in Prod Paths — **IN PROGRESS**

* **Current Status:** 595 total unwrap/expect calls, but majority in test code (acceptable)
* ✅ `ApiError` with `thiserror` already implemented (errors.rs)
* ✅ Structured error handling in place for all API endpoints
* **Production Code Status:** Most unwrap/expect in spider tests, not critical paths
* **Remaining:** Continue gradual replacement in production paths as encountered
* **Acceptance:** Production code handles errors gracefully without panics

### 1.5 Performance Monitoring Integration — **P0 / 1-2 days**

* **Issue:** PerformanceMonitor, GlobalStreamingMetrics, metrics never used
* **Impact:** No visibility into system performance, can't identify bottlenecks
* **Fix Required:**
  * Initialize PerformanceMonitor in main.rs
  * Wire metrics collection to all handlers
  * Connect to Prometheus exporter
  * Add performance checkpoints throughout pipeline
* **Files:** `monitoring/performance.rs`, `metrics/mod.rs`, `handlers/*.rs`
* **Acceptance:** `/metrics` endpoint shows real metrics, performance tracked

### 1.6 Observability (minimal) — **P0 / 1–2 days**

* `/metrics` (Prometheus) + `/healthz` (status/git\_sha/wit/trek).
* Histograms: request, fetch, wasm, render; counters: gate decisions, phase errors; cache hit/miss.
* Wire existing PerformanceMonitor and GlobalStreamingMetrics
* **Acceptance:** Grafana shows RPS, error-rate, p95 overall/fetch/wasm/render, cache hit-ratio, headless pool gauge.

### 1.7 NDJSON Streaming (PR-3) — **P0 / 2–3 days**

* **Depends on:** Streaming Pipeline Integration (1.1)
* Endpoints: `/crawl/stream`, `/deepsearch/stream` (ON by default).
* Connect to StreamProcessor and StreamingPipeline
* Flush one JSON object **per completed URL** (include `metrics`; emit structured error objects on failures).
* **Acceptance:** 10-URL batch → **TTFB < 500ms** (warm cache); all results arrive as lines; Playground viewer shows live lines/sec.

---

## 2) Reliability, Performance & Build

### 2.1 Resource Controls — **P1 / 2–3 days**

* Headless pool cap **= 3**; render **hard cap 3s**; per-host **1.5 rps** with jitter.
* PDF semaphore **= 2**; reuse a single Wasmtime component instance per worker; memory cleanup on timeouts.
* **Acceptance:** 50-URL batch p95 ≤ 5s; no OOM; stable pool usage.

### 2.2 Build/CI Speed — **P2 / 1 day**

* Cache WASM component artifact; incremental builds; parallel CI; binary size lint.
* **Acceptance:** CI time reduced; artifacts uploaded per PR; **3.9GB** build space reclaimed retained.

---

## 3) Monitoring & Reports You Can Review (no heavy UI)

* **Static Report Packs** (served at `/reports/last-run/...`):

  * **Extraction Golden Report** (actual vs expected JSON/MD + diff)
  * **Dynamic Rendering Report** (actions/console/network, before/after HTML, optional screenshot)
  * **Streaming Viewer** (NDJSON TTFB, lines/sec) — single HTML tool
  * **PDF Report** (text/metadata/images; memory note)
  * **Stealth Check** (webdriver flag, languages, canvas/WebGL hashes)
* **Acceptance:** `just report` produces packs; API serves `/reports/*`.

---

## 4) Phase 3 — Advanced Features (keeps *all* of your items)

### Feature Flags (as you specified)

```yaml
features:
  headless_v2: false      # PR-1: actions/waits/scroll/sessions
  stealth:     false      # PR-2: UA rotation + JS evasion
  streaming:   true       # PR-3: NDJSON endpoints
  pdf:         true       # PR-4: pdfium pipeline
  spider:      false      # PR-5: deep crawling
  strategies:  true       # PR-6: css/xpath/regex/llm + chunking
```

### Performance Guardrails (as you specified)

```yaml
perf:
  headless_pool_size:   3
  headless_hard_cap_ms: 3000
  fetch_connect_ms:     3000
  fetch_total_ms:       20000
  pdf_max_concurrent:   2
  streaming_buf_bytes:  65536
  crawl_queue_max:      1000
  per_host_rps:         1.5
```

### PR-1: Headless RPC v2 — **✅ COMPLETE**

* Integration with API pending flag activation.

### PR-2: Stealth Preset — **✅ COMPLETE (merged, commit 75c67c0)**

* Config:

```yaml
stealth:
  ua_pool_file: "configs/ua_list.txt"
  canvas_noise: true
  webgl_vendor: "Intel Inc."
```

* **Acceptance:** ≥80% success on bot-detection targets.

### PR-3: NDJSON Streaming — **NEXT**

* Code sketch already defined; see §1.5.
* **Acceptance:** see §1.5.

### PR-4: PDF Pipeline (pdfium) — **IN PROGRESS / Week 3**

* ✅ **Module Structure:** Complete PDF module with processor, config, types, and utils
* ✅ **Detection:** PDF detection by content-type, extension, and magic bytes
* ✅ **Processing:** PDF processor with pdfium integration and fallback
* ✅ **Integration:** Pipeline integration and processing result types
* 🔧 **Fine-tuning:** Concurrency controls and memory management
* Detect by content-type or suffix; extract **text**, **author/title/dates**, **images**; concurrency cap **=2**.
* **Status:** 85% complete - implementation done, final optimizations needed
* **Acceptance:** PDFs yield text + metadata; images > 0 for illustrated docs; stable memory.

### PR-5: Spider Integration — **IN PROGRESS / Week 4**

* ✅ **Infrastructure:** Full spider module implemented with all components
* ✅ **Core Engine:** Spider, FrontierManager, StrategyEngine, BudgetManager complete
* ✅ **Components:** Sitemap parser, URL utils, adaptive stopping, session management
* 🔧 **Integration Needed:** Wire spider into main API endpoints and handlers
* **Frontier strategies:** BFS/DFS/Best-First with priority scoring; sitemap parsing from robots; budgets (`max_depth`, `max_pages`, time).
* **Adaptive stop:** sliding window of unique_text_chars or scored chunk gain with `gain_threshold`, `window`, `patience`.
* **Status:** 85% complete - all core modules done, integration pending
* **Acceptance:** domain seed respects budgets; sitemap merged; early stop on low gain; returns ≥N pages with extraction.

### PR-6: Strategies & Chunking — **IN PROGRESS / Week 5**

* ✅ **Module Structure:** Complete strategies module with extraction and chunking
* ✅ **Extraction Strategies:** `trek`, `css_json`, `regex`, `llm` (hook-based) all implemented
* ✅ **Chunking System:** 5 modes - regex, sentence, topic, fixed, sliding (default `token_max=1200`, `overlap=120`)
* ✅ **Manager:** StrategyManager with performance metrics and processing pipeline
* 🔧 **Integration Needed:** Wire strategies into main extraction pipeline
* **Schema validation:** `schemars` before output; byline/date from **OG**/**JSON-LD**.
* **Status:** 80% complete - all modules implemented, integration pending
* **Acceptance:** long articles chunk deterministically; CSS/regex extract expected fields; byline/date ≥80% where present.

---

## 5) Phase 0 — Technical Debt & Integration (all original items retained)

### 0.1 Core Integration Gaps — **CRITICAL / \~1 week**

* WASM extractor wiring (see §1.1), dynamic rendering implementation (see §1.1).

### 0.2 Error Handling Improvements — **IN PROGRESS / 2–3 days**

* ✅ Progress: 93 of 517 unwrap/expect calls already fixed (424 remaining)
* Replace remaining `unwrap/expect` (424 down from 517); recovery paths.
* **Current Status:** 18% complete, good progress being made
* **Impact:** Production stability improving; 18% of panic points addressed.

### 0.3 Monitoring & Observability — **HIGH / 1 week**

* (Minimal Prometheus already in §1.3.)
* Add: system metrics placeholders at `health.rs:358-366` → real CPU/mem/fd/disk; RPS/latency dashboards; perf benchmarking suite.
* **Acceptance:** SLA panels; benchmark scripts checked in.

### 0.4 Session & Worker Management — **HIGH / 3–4 days**

* Sessions & cookies (see §1.4).
* Worker service (`riptide-workers/main.rs:13`): batch queue, job scheduling, retries, pool mgmt.
* **Acceptance:** background job runs batch crawl; retries on transient errors.

### 0.5 Resource Management & Performance — **MED / 3–4 days**

* Browser pooling/memory optimization; cleanup on timeouts; WASM lifecycle monitoring; memory alerts.
* Build pipeline optimization (address WASM 5+ min timeouts; dependency caching; incremental; parallel).
* **Acceptance:** stable memory; improved build times.

### 0.6 Test Coverage & Quality — **MED / 3–5 days**

* Raise to **≥80%** (currently **75%**); cover refactored modules & new features; golden tests for new outputs.

### 0.7 Circuit Breaker Enhancements — **LOW / 2 days**

* Adaptive thresholds; performance-based tuning; self-learning failure patterns.
* **Acceptance:** breaker avoids flapping; steady success rate under partial headless failures.

---

## 6) Performance Targets & Acceptance Criteria (unchanged, all preserved)

* **Fast-path:** p50 ≤ **1.5s**, p95 ≤ **5s** (10-URL mixed).
* **Streaming:** TTFB < **500ms** (warm cache).
* **Headless ratio:** < **15%**.
* **PDF:** ≤ **2** concurrent; no > **200MB** RSS spikes per worker.
* **Cache:** Wasmtime instance reuse; Redis read-through (24h TTL; keys include extractor version + strategy + chunking).
* **Gate:** thresholds hi=**0.55** / lo=**0.35**.

---

## 7) Rollout Plan (unchanged, all preserved)

1. Merge PR-1..3; enable `streaming=true` + `pdf=true`; keep `headless_v2`/`stealth` **OFF**.
2. **Canary**: enable `headless_v2` for 10% a week; monitor errors + `render_ms`.
3. Enable **stealth**; validate lower challenge rate.
4. Merge PR-5 (spider) **OFF** by default; stage on selected domains.
5. Merge PR-6; keep `trek + sliding` defaults; enable advanced strategies per job.

---

## 8) CI Additions (unchanged, all preserved)

* Build WASM component once; cache artifact across jobs.
* Unit + integration + streaming tests; **exclude live-web** in CI.
* Binary size lint; PDF concurrency tests behind feature flags.
* Performance regression benchmarks on merge.

---

## 9) Phase 4 — Enterprise (unchanged, all preserved)

* **Scalability & Distribution:** worker service, horizontal scale, LB/failover, distributed coordination.
* **Multi-tenant:** API keys/quotas; per-tenant config/limits; usage analytics/billing; isolation.
* **Advanced Analytics:** success/failure rates, content scoring, per-domain performance, cost analytics.
* **CLI & Dev Tools:** standalone CLI; config files; progress/resume; CI integration.

---

## 10) Phase 5 — Optimization & Maintenance (unchanged, all preserved)

* **Advanced caching:** content dedupe, cache warming, predictive prefetch, edge caches.
* **Resource optimization:** memory/CPU/network/storage tuning.
* **DX:** docs, API examples, quickstarts, SDKs, contribution guides.
* **Ecosystem:** webhooks, plugin architecture, DB integrations, cloud exports.

---

## 11) Success Metrics (unchanged, all preserved)

### Phase 0 (Remaining)

* Replace 517 `unwrap/expect`.
* Coverage **≥80%**.
* Monitoring/observability deployed.
* Resource mgmt optimized (pooling/memory limits).

### Phase 3

* PR-1 ✅; PR-2 ✅.
* **PR-3:** endpoints work; stream per URL; TTFB < 500ms; error lines present.
* **PR-4:** detect PDFs; extract text/meta/images; concurrency = 2.
* **PR-5:** BFS/DFS/Best-First; sitemap; adaptive stop; budget enforcement.
* **PR-6:** strategies + 5 chunkers; schema validate; OG/JSON-LD byline/date.

### Phase 4

* Multi-tenant shipped; 10+ nodes; enterprise onboarding; **99.99%** SLA-ready.

---

## 12) Risks & Mitigations (unchanged, all preserved)

* **WASM (wasip2):** use `wasmtime::component::bindgen!`, single instance/worker.
* **Scale perf:** gradual load tests; monitor p95/p99.
* **Headless stability:** container restart policies; health checks; breaker to fast path.
* **Memory leaks:** WASM/Chrome lifecycle; semaphores & timeouts.

**External:** Serper.dev limits; infra stability; Redis; site blocks.
**Version locks:** `trek-rs = "=0.2.1"`, `wasm32-wasip2`, `chromiumoxide`, `robotstxt`, `axum-prometheus`.

---

## 13) Timeline (unchanged, all preserved)

| Phase       | Duration | Deliverables                             | Risk         | Status                 |
| ----------- | -------- | ---------------------------------------- | ------------ | ---------------------- |
| **Phase 0** | 4 wks    | Integration, errors, monitoring, quality | **HIGH**     | 70% (Spider fixes+Sessions done) |
| **Phase 3** | 2–3 wks  | Parity features                          | MEDIUM       | PR-1/2 ✅; PR-3 next; Strategies 80% |
| **Phase 4** | 6–8 wks  | Enterprise                               | LOW          | Planned                |
| **Phase 5** | Ongoing  | Optimization                             | LOW          | Planned                |

**Total remaining:** \~3.5–4 months (with core first).

---

## 14) Immediate Next Steps (exact order)

**🔴 Week 0 - CRITICAL INTEGRATION (Must do first)**

1. ✅ **Browser Pool Integration** - Initialize pool, wire to handlers (COMPLETED)
2. **Streaming Pipeline Integration** - Wire StreamProcessor to endpoints (1 day)
3. **Session System Wiring** - Connect SessionManager to middleware (1 day)
4. **Performance Monitoring** - Initialize and wire PerformanceMonitor (1 day)

**Week 1**

5. Core WASM wiring + dynamic rendering via Browser Pool
6. Remove panics on hot paths (424 unwrap/expect)
7. `/metrics` + `/healthz` with real metrics from PerformanceMonitor

**Week 2**
8\) PR-3: `/crawl/stream` + `/deepsearch/stream` (using wired StreamingPipeline)
9\) Deep Search implementation (wire DeepSearchMetadata structures)
10\) Report packs + static serving

**Week 3**
11\) Resource controls (pools/caps) - leverage existing Browser Pool
12\) PDF pipeline (PR-4) + PDF concurrency guard
13\) Wire remaining unused methods (push_request, recovery_strategy, etc.)

**Week 4**
14\) Spider (PR-5) integration - connect to main pipeline
15\) Worker service integration (batch processing, job scheduling)

**Week 5**
16\) Strategies & chunking (PR-6) - wire to extraction pipeline
17\) Complete integration testing of all wired components

**Week 6 - INTEGRATION RECONCILIATION (Critical Step)**

18\) **Cross-Component Wiring Audit**
    * Verify all imports are properly connected between modules
    * Ensure session cookies flow through browser pool → render handlers
    * Connect performance metrics from all components to monitoring
    * Wire error types consistently across boundaries
    * Verify WASM extractor receives data from all sources

19\) **Dependency Graph Validation**
    * Map all inter-component dependencies
    * Identify missing connections (e.g., PDF processor → metrics)
    * Ensure streaming pipeline receives from all producers
    * Verify cleanup handlers triggered from all failure paths

20\) **Integration Touch-ups**
    * Add missing trait implementations (From, Into, Error conversions)
    * Wire up orphaned event handlers and callbacks
    * Connect rate limiter to ALL HTTP operations (not just render)
    * Ensure memory manager tracks ALL allocations
    * Complete bidirectional connections (request → response paths)

21\) **End-to-End Flow Verification**
    * Trace a request through entire pipeline
    * Verify all telemetry/metrics collected
    * Ensure all resources properly released
    * Check all error paths return appropriate responses

**Parallel (Weeks 0–6)**

* Build/CI speedups, test coverage to ≥80%, circuit breaker tuning (low priority).

---

## 15) Integration Reconciliation Phase (New Section)

### What This Phase Addresses:
When we build components in isolation, we often miss:
- **Forgotten Imports**: Components that should use shared types/traits but don't
- **Partial Wirings**: A→B connected but B→C forgotten
- **Orphaned Features**: Implemented but never called from anywhere
- **Inconsistent Patterns**: Different error handling/logging/metrics across modules
- **Missing Backpressure**: Component A floods B without flow control
- **Dangling Resources**: Cleanup in error paths not wired to all callers

### Key Reconciliation Tasks:
1. **Import Audit**: Ensure all shared types (SessionId, RequestId, etc.) used consistently
2. **Metrics Wiring**: Every operation reports to performance monitor
3. **Error Propagation**: All errors properly converted and bubbled up
4. **Resource Lifecycle**: All guards/locks/handles properly released
5. **Event Flow**: All events have publishers AND subscribers
6. **Configuration Propagation**: Settings reach all relevant components
7. **Telemetry Coverage**: No blind spots in observability

### Success Criteria:
- No unused imports remain
- No unreachable code warnings
- All pub methods have at least one caller
- Resource leak detection shows clean
- Integration tests pass with all components active
- Metrics dashboard shows data from ALL components

### Integration Checklist (Run After Each Component):
```
□ Does this component import all relevant shared types?
□ Does it emit metrics to the performance monitor?
□ Are errors properly wrapped with context?
□ Do cleanup paths trigger in ALL failure scenarios?
□ Are resources (browsers, memory, handles) released?
□ Is backpressure/rate limiting applied?
□ Do callbacks/event handlers have subscribers?
□ Is configuration read from the global config?
□ Are there integration tests with real dependencies?
□ Does telemetry show this component's activity?
```

### Common Integration Misses (Watch For These):
- **Browser Pool** → Need to wire checkout/checkin to ALL render paths
- **Session Manager** → Must flow through middleware to handlers to browser
- **Rate Limiter** → Often forgotten on secondary HTTP paths (health, metrics)
- **Memory Manager** → PDF/WASM/Spider allocations often not tracked
- **Performance Monitor** → Checkpoints missing in async continuation points
- **Error Context** → Lost when crossing async boundaries
- **Streaming Pipeline** → Backpressure not propagated to producers
- **Resource Guards** → Drop implementations not triggering cleanup

---

## 16) Crosswalk (nothing lost)

* Your **Phase 0** items → §§1.1–1.5 and §5; 0.3/0.5/0.6/0.7 kept verbatim in §5 & §2/§7/§8.
* Your **Phase 3 PR-1..PR-6** → §4 (unchanged content, clearer sequence).
* **Performance targets / resource limits / cache / gate** → §6.
* **Rollout plan** → §7.
* **CI additions** → §8.
* **Phase 4 / Phase 5** → §§9–10.
* **Success metrics** → §11.
* **Risks & version locks** → §12.
* **Timeline + Next steps** → §§13–14.
