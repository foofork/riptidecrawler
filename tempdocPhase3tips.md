Heck yes — let’s land Phase 3 **fast** without blowing up latency or touching risky areas prematurely. Below is a **merge-ready plan** with exact PR order, cut-paste code stubs, config diffs, and perf guardrails. No decisions left for the devs.

---

# 0) Merge strategy (low risk, fast landing)

**Branch:** `feature/phase3`
**PRs (merge in order, each releasable):**

1. **PR-1 Headless RPC v2** — actions/waits/scroll/sessions (flagged OFF by default)
2. **PR-2 Stealth preset** — UA rotation + JS evasion (flagged OFF)
3. **PR-3 NDJSON streaming** — `/crawl/stream` + `/deepsearch/stream`
4. **PR-4 PDF pipeline** — pdfium text+metadata (cap concurrency)
5. **PR-5 Spider integration** — depth/budgets + sitemap + adaptive stop (flagged OFF)
6. **PR-6 Strategies & chunking** — css/xpath/regex + 5 chunkers + schema validate (default TREK)

**Feature flags (config):**

```yaml
features:
  headless_v2: false
  stealth: false
  streaming: true
  pdf: true
  spider: false
  strategies: true
```

Turn flags ON one at a time post-merge.

---

# 1) Perf guardrails (copy/paste defaults)

```yaml
perf:
  headless_pool_size: 3         # hard cap
  headless_hard_cap_ms: 3000    # render budget
  fetch_connect_ms: 3000
  fetch_total_ms: 20000
  pdf_max_concurrent: 2
  streaming_buf_bytes: 65536
  crawl_queue_max: 1000
  per_host_rps: 1.5
```

* **Wasmtime**: instantiate component **once per worker**, reuse the store.
* **Redis**: read-through; TTL 24h; key includes extractor version + strategy + chunking.
* **Headless fallback ratio target** < 15% of pages; gate thresholds hi=0.55 / lo=0.35.

---

# 2) PR-1: Headless RPC v2 (actions/waits/scroll/sessions)

**Files:** `crates/riptide-headless/src/models.rs`, `cdp.rs`

```rust
// models.rs
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RenderRequest {
    pub session_id: Option<String>,
    pub url: String,
    pub actions: Option<Vec<PageAction>>,
    pub timeouts: Option<Timeouts>,
    pub artifacts: Option<Artifacts>,
}

#[derive(Deserialize)]
pub struct Timeouts {
    pub nav_ms: Option<u64>,
    pub idle_after_dcl_ms: Option<u64>,
    pub hard_cap_ms: Option<u64>,
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

#[derive(Deserialize, Serialize, Default)]
pub struct Artifacts { pub screenshot: bool, pub mhtml: bool }

#[derive(Serialize)]
pub struct RenderResponse {
    pub final_url: String,
    pub html: String,
    pub session_id: Option<String>,
    pub artifacts: ArtifactsOut,
}

#[derive(Serialize, Default)]
pub struct ArtifactsOut { pub screenshot_b64: Option<String>, pub mhtml_b64: Option<String> }
```

```rust
// cdp.rs (core skeleton)
async fn exec_actions(page: &Page, actions: &[PageAction]) -> anyhow::Result<()> {
    for a in actions {
        match a {
            PageAction::WaitForCss{css, timeout_ms} => {
                page.wait_for_element_with_timeout(css, timeout_ms.unwrap_or(5000)).await?;
            }
            PageAction::WaitForJs{expr, timeout_ms} => {
                let deadline = tokio::time::Instant::now() + Duration::from_millis(timeout_ms.unwrap_or(5000));
                loop {
                    let ok: bool = page.evaluate(expr).await.unwrap_or(false);
                    if ok { break; }
                    if tokio::time::Instant::now() >= deadline { anyhow::bail!("wait_for_js timeout"); }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
            PageAction::Scroll{steps, step_px, delay_ms} => {
                for _ in 0..*steps {
                    page.evaluate(&format!("window.scrollBy(0,{step_px});")).await.ok();
                    tokio::time::sleep(Duration::from_millis(*delay_ms)).await;
                }
            }
            PageAction::Js{code} => { page.evaluate(code).await.ok(); }
            PageAction::Click{css} => { page.find_element(css).await?.click().await?; }
            PageAction::Type{css, text, delay_ms} => {
                let el = page.find_element(css).await?;
                for ch in text.chars() {
                    el.type_str(&ch.to_string()).await?;
                    tokio::time::sleep(Duration::from_millis(delay_ms.unwrap_or(20))).await;
                }
            }
        }
    }
    Ok(())
}
```

**API passthrough:** `riptide-api` forwards `RenderRequest` when `features.headless_v2=true`.
**Artifacts:** add optional screenshot/MHTML (base64) only when requested.
**Session:** map `session_id -> user-data-dir` (persist cookies).

---

# 3) PR-2: Stealth preset

**Files:** `riptide-headless/src/launcher.rs`, `stealth.js` (injected early)

* Launch flags: `--disable-blink-features=AutomationControlled --no-first-run --mute-audio --disable-dev-shm-usage --headless=new`
* Inject **once per page** before any script:

  * `navigator.webdriver=false`, languages/plugins/platform spoof, tiny canvas/WebGL noise.
* UA rotation: load `configs/ua_list.txt` and pick per **session\_id** (stable hash).

**Toggle:** `features.stealth`.

---

# 4) PR-3: NDJSON streaming

**Files:** `riptide-api/src/streaming.rs`, wire routes

```rust
// streaming.rs
use axum::{response::IntoResponse, extract::State};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use serde_json::to_vec;

pub async fn crawl_stream(State(app): State<AppState>, Json(body): Json<CrawlBody>) -> impl IntoResponse {
    let (tx, rx) = tokio::sync::mpsc::channel::<Vec<u8>>(128);
    tokio::spawn(async move {
        if let Err(e) = orchestrate_stream(app, body, tx).await {
            // optionally emit an error line
        }
    });
    axum::response::Response::builder()
        .header("Content-Type","application/x-ndjson")
        .body(axum::body::Body::from_stream(ReceiverStream::new(rx).map(axum::body::Bytes::from)))
        .unwrap()
}

// inside orchestrate_stream: for each finished URL:
let line = to_vec(&result).unwrap();
let mut buf = line; buf.push(b'\n');
let _ = tx.send(buf).await;
```

**Why NDJSON (not SSE):** lower overhead, trivially consumable, great for batch jobs.

---

# 5) PR-4: PDF pipeline

**Files:** `riptide-core/src/pdf.rs`, `types.rs`, config knob `perf.pdf_max_concurrent`

* Detect PDF via content-type or URL suffix.
* Use `pdfium-render` to pull **text**, **author/title/create/modify dates**, and **images**.
* Semaphore to cap concurrent PDFs: `pdf_max_concurrent`.
* Produce Markdown from text; attach metadata to `ExtractedDoc`.

---

# 6) PR-5: Spider integration (depth/budget/sitemap/adaptive)

**Files:** `riptide-core/src/crawl.rs`, `riptide-api/src/models.rs`

* Frontier modes: BFS / DFS / Best-First (priority on link hints + depth penalty).
* Sitemaps: parse `robots`, merge URLs (respect robots).
* Adaptive stop: sliding window of **unique\_text\_chars** or **scored chunk gain**; stop when below threshold `k` times.
* Enforce budgets: `max_depth`, `max_pages`, `crawl_budget_seconds`.

**Toggle:** `features.spider`.

---

# 7) PR-6: Strategies & chunking (+ schema)

**Files:**

* `riptide-core/src/strategy/{mod.rs, trek.rs, css_json.rs, regex.rs, llm.rs}`
* `riptide-core/src/chunking.rs` (regex/sentence/topic/fixed/sliding)
* `riptide-core/src/schema.rs` (schemars)

**Default:** `trek` only; others behind config.
**Chunking default:** sliding (token\_max=1200, overlap=120).
**Schema:** validate result before writing; if invalid → log+fallback to raw doc output.

---

# 8) Config diffs (drop-in)

```yaml
features:
  headless_v2: false
  stealth: false
  streaming: true
  pdf: true
  spider: false
  strategies: true

dynamic:
  defaults:
    actions: []
    timeouts: { nav_ms: 15000, idle_after_dcl_ms: 1000, hard_cap_ms: 3000 }

stealth:
  ua_pool_file: "configs/ua_list.txt"
  canvas_noise: true
  webgl_vendor: "Intel Inc."

perf:
  headless_pool_size: 3
  headless_hard_cap_ms: 3000
  fetch_connect_ms: 3000
  fetch_total_ms: 20000
  pdf_max_concurrent: 2
  streaming_buf_bytes: 65536
  crawl_queue_max: 1000
  per_host_rps: 1.5

crawler:
  mode: "best-first"
  max_depth: 3
  budget: { pages: 200, seconds: 120 }
  adaptive_stop: { gain_threshold: 600, window: 10, patience: 3 }
  sitemap: { enabled: true }

extraction:
  strategy: "trek"              # trek|css_json|regex|llm
  chunking: { method: "sliding", token_max: 1200, overlap: 120 }
```

---

# 9) CI additions

* Build wasm (component) **once**, cache it as artifact.
* Run unit + integration + streaming tests; keep live-web tests **out** of CI.
* Lint for large binaries; gate **PDF concurrent** tests behind feature flag.

---

# 10) Bench targets (acceptance)

* **Fast-path p50 ≤ 1.5s**, **p95 ≤ 5s** on 10-URL mixed batch (headless pool 3, hard cap 3s).
* **Streaming TTFB** (first NDJSON line) **< 500ms** for warm cache.
* **Headless fallback ratio** < 15%.
* **PDF throughput**: 2 concurrent PDFs, stable memory (no >200MB spikes per worker).
* **Spider**: respects depth/budget; stops early on low gain.

---

# 11) Rollout

1. Merge PR-1..3, enable `streaming=true` + `pdf=true`, keep headless\_v2/stealth OFF.
2. Canary headless\_v2 on a subset (10%) for a week; watch error rate + render\_ms.
3. Enable stealth; validate reduced challenge rate.
4. Merge PR-5 (spider) OFF by default; run staging domain crawls.
5. Merge PR-6; keep defaults (TREK + sliding). Turn on css/regex in configs per job.

---

If you want, I can also drop a **compact action executor test** and a **fake SPA fixture server** snippet so PR-1 lands green with deterministic tests.
