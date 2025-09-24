Absolutely — here’s a **single, consolidated roadmap** that **replaces** the previous draft **without losing a single item**. I kept every task, metric, flag, risk, and acceptance criterion you listed, and reorganized them by **priority and execution order**. At the end there’s a brief **crosswalk** so you can see where each original section landed.

---

# RipTide Crawler — Consolidated & Prioritized Roadmap (Supersedes Prior Draft)

## 0) Snapshot (Updated: 2025-09-24)

* **✅ Done:** Phase 0 (Foundation), Phase 1 (Core), Phase 2-Lite (Reliability), Phase 3 PR-1 (Headless RPC v2), PR-2 (Stealth)
* **✅ Session Management:** Session manager implementation complete and functional
* **📍 Now:** Critical file corruption fixes + **PR-3 (NDJSON Streaming)**
* **🚨 BLOCKED:** Spider integration (corrupted files with literal \n characters - needs immediate fix)
* **⚡ IN PROGRESS:** Strategies implementation (14 files active, needs integration)
* **⚡ IN PROGRESS:** PDF processing (functional but needs final integration)
* **⚡ IN PROGRESS:** Error handling cleanup (424 unwrap/expect remaining, down from 517)
* **🧭 Guardrails:** Feature flags, Prometheus metrics, strict timeouts/pools
* **📜 Reference:** See `COMPLETED.md` for all shipped work.

---

## 1) Critical Path (do in this order)

### **✅ COMPLETED: File Corruption Fix — RESOLVED**

* **Spider Module:** [FIXED] - All syntax errors resolved, files properly formatted
  * ✅ `/crates/riptide-core/src/spider/spider.rs` - Fully functional with proper formatting
  * ✅ **All Issues Resolved:** Proper newlines, correct syntax, compilation ready
  * ✅ **Status:** 100% fixed and functional
  * **Next:** Ready for integration into main pipeline
* **Status:** No longer blocking - spider module ready for use

### 1.1 Core Wiring (unblocks everything) — **P0 / 2–3 days**

* **WASM Extractor Integration**

  * Wire *actual* component calls in `handlers/render.rs:401`, remove placeholders (`render.rs:404-409`).
  * Integrate Trek-rs extractor into the render pipeline; finalize output mapping to `ExtractedDoc`.
* **Dynamic Rendering Implementation**

  * Replace `render.rs:293-297` placeholders with real headless rendering via RPC v2.
  * Execute actions (waits/scroll/js/type/click), content analysis for adaptive rendering (`render.rs:382`).
* **Acceptance:** 5-URL mixed set returns title/text/links; SPA fixture renders with waits/scroll; logs show `phase=fast|headless`.

### 1.2 Eliminate Panics in Prod Paths — **P0 / 3–4 days**

* Replace **517** remaining `unwrap/expect` (25 already fixed); focus on request/fetch/render/WASM/JSON I/O.
* Introduce `ApiError` via `thiserror`; structured error lines in NDJSON.
* **Acceptance:** `clippy -D warnings` green; chaos cases (bad URL, 404, oversize, render timeout) return **error records**, not panics.

### 1.3 Observability (minimal) — **P0 / 1–2 days**

* `/metrics` (Prometheus) + `/healthz` (status/git\_sha/wit/trek).
* Histograms: request, fetch, wasm, render; counters: gate decisions, phase errors; cache hit/miss.
* (Leave OpenTelemetry tracing disabled for now per `telemetry.rs:146`.)
* **Acceptance:** Grafana shows RPS, error-rate, p95 overall/fetch/wasm/render, cache hit-ratio, headless pool gauge.

### 1.4 Sessions & Cookies — **✅ COMPLETED**

* ✅ Persistent `session_id` → user-data-dir + cookie jar; TTL & cleanup implemented
* ✅ Session storage, middleware, and manager fully functional
* ✅ Session types and configuration properly structured
* **Acceptance:** ✅ Same `session_id` preserves login across two `/render` calls - implementation ready

### 1.5 NDJSON Streaming (PR-3) — **P0 / 2–3 days**

* Endpoints: `/crawl/stream`, `/deepsearch/stream` (ON by default).
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

**Week 1**

1. Core wiring (WASM + /render)
2. Remove panics on hot paths
3. `/metrics` + `/healthz` minimal set

**Week 2**
4\) Sessions (cookies)
5\) PR-3: `/crawl/stream` + `/deepsearch/stream`
6\) Report packs + static serving

**Week 3**
7\) Resource controls (pools/caps)
8\) PDF pipeline (PR-4) + PDF concurrency guard

**Week 4**
9\) Spider (PR-5) behind flag; staging tests

**Week 5**
10\) Strategies & chunking (PR-6); defaults to trek+sliding

**Parallel (Weeks 1–5)**

* Build/CI speedups, test coverage to ≥80%, circuit breaker tuning (low priority).

---

## 15) Crosswalk (nothing lost)

* Your **Phase 0** items → §§1.1–1.5 and §5; 0.3/0.5/0.6/0.7 kept verbatim in §5 & §2/§7/§8.
* Your **Phase 3 PR-1..PR-6** → §4 (unchanged content, clearer sequence).
* **Performance targets / resource limits / cache / gate** → §6.
* **Rollout plan** → §7.
* **CI additions** → §8.
* **Phase 4 / Phase 5** → §§9–10.
* **Success metrics** → §11.
* **Risks & version locks** → §12.
* **Timeline + Next steps** → §§13–14.
