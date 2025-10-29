

### Phase 2 (ideal): return full page objects

* New `CrawledPage` + `SpiderResultPages`.
* Allow field filtering to control payload size.

```rust
#[derive(Serialize)]
pub struct CrawledPage {
    pub url: String,
    pub depth: u32,
    pub status_code: u16,
    pub title: Option<String>,
    pub content: Option<String>,      // raw html/text, behind a flag
    pub markdown: Option<String>,     // normalized/extracted, behind a flag
    pub links: Vec<String>,
}

#[derive(Serialize)]
pub struct SpiderResultPages {
    pub pages_crawled: u64,
    pub pages_failed: u64,
    pub duration_seconds: f64,
    pub stop_reason: String,
    pub pages: Vec<CrawledPage>,
}
```

**Field selection** (query params):

* `include=title,links,markdown`
* `exclude=content`
  Server builds only requested fields to keep memory + bandwidth tight.

---

## 3) Add **streaming** and **job storage** options (scales better)

### Streaming (NDJSON or SSE)

* `Accept: application/x-ndjson` or `event-stream`
* Emit one `CrawledPage` per line/event as soon as it’s ready.
* Always end with a final **stats** object so clients know it’s complete.

Example NDJSON:

```
{"type":"page","data":{...}}
{"type":"page","data":{...}}
{"type":"stats","data":{"pages_crawled":123,...}}
```

### Job store (async fetch)

* `POST /spider?result_mode=store` → `{ job_id }`
* `GET /jobs/{job_id}/results?cursor=...&limit=...&include=markdown,title`
* `GET /jobs/{job_id}/stats`
* Optional: `POST /jobs/{job_id}/webhook` to register a callback

This lets big crawls finish server-side, and clients page through results.

---

## 4) Add **extraction-first helpers** to complete the “discover → extract” loop

Two quality-of-life APIs that users expect:

* `POST /extract/batch`
  Body: `{ "urls": ["..."], "format": "markdown" }`
  Returns array/stream of `{url, markdown, metadata}`.

* `POST /spider+extract`
  Body: `{ "seeds": ["..."], "scope": {...}, "include": ["markdown","title"], "result_mode": "store|stream|pages" }`
  Under the hood: crawl → extract markdown for details pages only → deliver combined `CrawledPage` with filled `markdown`.

These can just orchestrate existing internals; no heavy new engine logic needed.

---

## 5) Wire format, limits & safety (so you don’t paint yourself into a corner)

* **Compression**: Always support `gzip` on non-stream responses.
* **Size guardrails**:

  * `max_pages` per request (configurable)
  * `max_content_bytes` per page; truncate + set `truncated: true`.
* **Normalization**:

  * `final_url` (after redirects), `canonical_url` if found
  * `mime`, `charset`, `fetch_time_ms`
* **Robots/TOS flags**: `robots_obeyed: true`, `disallowed: bool` on pages you skip.
* **Error surface**: enrich per-page with `fetch_error` or `parse_error`.
* **Versioning**:

  * Add a top-level `api_version` field in results.
  * Keep `/spider` stable; if you ever need breaking changes, add `/v2/spider`.

---

## 6) Suggested final API surface (concise)

* `POST /spider`
  Params: `result_mode=stats|urls|pages|stream|store`, `include=...`, `max_pages`, `depth`, `scope`
  Returns:

  * `stats|urls|pages` → JSON
  * `stream` → NDJSON/SSE
  * `store` → `{job_id}`

* `GET /jobs/{id}/results` → paginated pages (`cursor`, `limit`, `include`)

* `GET /jobs/{id}/stats` → final/rolling stats

* `POST /extract` → single URL

* `POST /extract/batch` → many URLs

* `POST /spider+extract` → discover + extract in one shot (optional sugar)

---

## 7) Minimal Rust diff you can implement now (Phase 1)

```rust
#[derive(Deserialize)]
pub enum ResultMode { #[serde(rename="stats")] Stats, Urls, Pages, Stream, Store }

#[derive(Serialize)]
pub struct SpiderResultStats { /* existing fields */ }

#[derive(Serialize)]
pub struct SpiderResultUrls {
    pub pages_crawled: u64,
    pub pages_failed: u64,
    pub duration_seconds: f64,
    pub stop_reason: String,
    pub domains: Vec<String>,
    pub discovered_urls: Vec<String>,
}

// in spider handler:
let mode = parse_result_mode(query.result_mode);
let summary = crawl_summary; // existing
match mode {
    ResultMode::Stats => Json(SpiderResultStats::from(&summary)),
    ResultMode::Urls  => Json(SpiderResultUrls {
        pages_crawled: summary.pages_crawled,
        pages_failed: summary.pages_failed,
        duration_seconds: summary.duration_secs,
        stop_reason: summary.stop_reason.clone(),
        domains: summary.domains.clone(),
        discovered_urls: summary.discovered_urls.clone(), // <-- plumb from core
    }),
    _ => unimplemented!(),
}
```

> Implementation tip: accumulate `discovered_urls` in `riptide-spider/src/core.rs` during enqueue/visit. Keep it as a `Vec<String>` capped by `max_pages`.

---

## 8) Client ergonomics (what users will actually write)

**Discover → extract (today)**

```python
res = rt.spider(seeds=["https://example.com"], result_mode="urls", max_pages=500)
for url in res["discovered_urls"]:
    item = rt.extract(url, format="markdown")
    # save item
```

**Large crawl (streaming)**

```python
for line in rt.spider_stream(seeds=[...], result_mode="stream", include="title,links"):
    obj = json.loads(line)
    if obj["type"] == "page":
        handle_page(obj["data"])
```

**Async job (store)**

```python
job = rt.spider(seeds=[...], result_mode="store")
while True:
    pages = rt.results(job["job_id"], cursor=cursor, limit=200, include="markdown,title")
    save(pages)
    if pages["done"]: break
    cursor = pages["next_cursor"]
```

---

### Bottom line

* **Phase 1**: add `discovered_urls` behind `result_mode=urls`. Ships fast, unblocks the “discover → extract” workflow.
* **Phase 2**: add `pages` objects + `stream` + `store` for scale and UX.
* You stay **backward-compatible** while meeting the clear user expectation: *a spider that actually returns what it discovered*.


**Short answer:** put Phase 2 in the **API**, keep only “orchestration sugar” in the facade.

Here’s the split that tends to work best:

# What belongs in the API (core, public, stable)

* **Discovery results themselves**
  `pages: Vec<CrawledPage>` with `url, depth, status_code, title, links, (optional) content/markdown`.
  Rationale: clients expect crawlers to *return what they crawled*; this is table-stakes behavior.
* **Delivery modes**
  `result_mode=stats|urls|pages|stream|store`, plus **NDJSON/SSE streaming** and **paginated fetch** (`/jobs/{id}/results`).
  Rationale: scale, backpressure, and interop. These are transport concerns, not business logic.
* **Field selection & truncation**
  `include=title,links,markdown` and size guards (`max_pages`, `max_content_bytes`, `truncated: true`).
  Rationale: bandwidth/latency control must be uniform for every client.
* **Normalization metadata**
  `final_url`, `canonical_url`, `mime`, `charset`, `fetch_time_ms`, `robots_obeyed`, per-page `fetch_error/parse_error`.
  Rationale: essential debugging/ops surface tied to the crawl itself.

# What belongs in the facade (your tool server / orchestrator)

* **Business workflows**
  “Discover → *select only detail pages* → Extract → Normalize → Save → Dedup.”
* **Entity-aware extraction**
  JSON-LD/CSS/LLM chains, timezone normalization, dedupe keys, quality scoring.
* **Cross-API composition**
  The “`/spider+extract`” convenience endpoint (if you expose it at all) can live in the facade as an orchestrated flow that calls core `/spider` + `/extract/batch` + your DB.
* **User/session logic**
  Per-tenant quotas, auth to your DB, webhooks, notifications, audit trails.

# Why Phase 2 should be in the API

1. **Interoperability**: external clients can consume riptide directly without your facade and still get industry-standard behavior.
2. **Performance & memory**: streaming/pagination needs tight integration with the crawler loop; doing it only in the facade forces buffering/proxying and increases tail latency.
3. **Ecosystem**: libraries/SDKs (Python, JS, Rust) can map `CrawledPage` 1:1; fewer bespoke contracts.
4. **Backward compatibility**: you already have `result_mode=stats`; adding `urls/pages/stream/store` is additive and versionable.

# Practical plan

* **API (Rust)**

  * Add `ResultMode` + `SpiderResultPages`, NDJSON/SSE stream, and `/jobs/{id}/results` pagination.
  * Gate heavy fields (`content`, `markdown`) behind `include=...`; enforce size limits.
* **Facade (Python)**

  * Keep `propose_sources`, `crawl_sources`, `fetch_results`.
  * Optional: `spider+extract` that calls API `/spider?result_mode=store` → filters pages → `/extract/batch` → writes entities.

# Edge cases & guidance

* For very large sites, encourage `result_mode=store` + paginated fetch; only enable `pages` (single response) up to a safe `max_pages`.
* If you ever need to break the page schema, version at the **API path** (`/v2/spider`) rather than only in the facade.

**Bottom line:** implement Phase 2 (pages, streaming, stored results, field selection) **in the API**; keep higher-level “discover+extract+normalize” workflows **in the facade**. This gives users the expected crawler semantics, keeps performance characteristics correct, and lets your facade focus on product logic instead of transport.


Here’s what each checkbox buys you—practically, for users and for you—and how to know it’s “done.”

NDJSON/SSE streaming (result_mode="stream")

What you get

Near-real-time pages as they’re crawled ({"type":"page","data":{...}}), plus a final {"type":"stats",...}.

Low latency UX; no giant response bodies; consumers can start processing immediately.

Developer impact

Implement a streaming writer around your crawl loop.

Heartbeats + graceful end; gzip off (usually) for SSE, okay for NDJSON over HTTP.

Sample line

{"type":"page","data":{"url":"https://ex.com/a","depth":1,"status_code":200,"title":"A","links":["..."]}}


Acceptance criteria

Consumers can parse incrementally; no buffering required.

Backpressure respected; connection survives long runs; final stats always emitted.

Job storage & pagination (result_mode="store")

What you get

Async crawls that scale: POST /spider → {job_id} then GET /jobs/{id}/results?cursor=...&limit=....

Works for very large sites; stable memory footprint.

Developer impact

Persistent store for pages (e.g., sqlite/postgres + blob/table).

Cursor-based pagination; retention & quotas.

Acceptance criteria

Idempotent fetches; stable ordering (e.g., by discover_time then url).

Can retrieve partial fields via include=title,links,markdown.

Jobs have lifecycle: queued|running|done|failed|canceled with accurate stats.

Extraction helpers (POST /extract/batch, POST /spider+extract)

What you get

One-call ergonomics for common flows:

extract/batch: { "urls":[...], "format":"markdown" } → [ {url, markdown, metadata} ] (or stream).

spider+extract: seeds → discovery and detail-page extraction, delivered as pages with markdown populated.

Developer impact

These can live in your facade (or API if you want), orchestrating existing spider + extract.

Optional: only extract on “detail pages” (e.g., heuristic on link density, URL patterns).

Acceptance criteria

Batch honors per-URL errors without failing the whole request.

spider+extract returns only pages meeting extraction criteria (or flags others).

Integration tests (live API)

What you get

Confidence the whole stack works (routing, auth, streaming, storage, pagination).

Prevents regressions in content fields and result shapes.

Developer impact

Spin a test crawler against deterministic fixtures (local HTTP server with canned pages).

Golden tests for schema: assert fields, ordering, pagination, end-of-stream stats.

Acceptance criteria

CI runs tests that: stream small crawl, store big crawl, paginate, batch-extract, and cancel mid-crawl.

All endpoints return correct Content-Type, status codes, and headers.

Performance optimization & benchmarking

What you get

Clear SLOs and levers (concurrency, rate limits, HTML parsing cost).

Proof you’re competitive (or where to improve).

Developer impact

Bench harness: seeds, depth, max_pages; metrics: pages/sec, median/95p fetch+parse, memory, CPU.

Profiling hot spots (DNS, TLS, parsing, DOM→markdown).

Acceptance criteria

Documented baseline vs. target (e.g., 50–150 pages/sec on LAN fixtures).

No unbounded memory with store; consistent tail latency on stream.

Per-domain politeness preserved under load.