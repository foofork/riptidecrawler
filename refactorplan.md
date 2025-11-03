# A. Ground truths from your analysis (the baseline we honor)

* **Workspace crates (26):**
  `riptide-api, riptide-cli, riptide-types, riptide-config, riptide-events, riptide-monitoring, riptide-fetch, riptide-extraction, riptide-pool, riptide-browser-abstraction, riptide-stealth, riptide-browser, riptide-reliability, riptide-intelligence, riptide-search, riptide-spider, riptide-pdf, riptide-cache, riptide-security, riptide-persistence, riptide-performance, riptide-streaming, riptide-test-utils, riptide-facade, riptide-headless, riptide-workers`.
* **Public routes:** 120+ across crawl, spider/frontier, extract (advanced/LLM/PDF/tables), deepsearch, engine probing, stealth, admin/config, etc.
* **Config:** ~150 env vars, 45+ feature flags; per-crate defaults.
* **Integrations (examples):** Redis, Chrome/CDP, LLM providers, search provider (e.g., Serper), metrics (Prometheus/OpenTelemetry), snapshot storage (e.g., S3/GCS/local), etc.
* **Requirement:** **No feature loss** while consolidating.

---

# B. Canonical API v1 (exact surface; everything else becomes shims or admin)

Mount these under `/api/v1`:

1. **Ops**

   * `GET /healthz`
   * `GET /diagnostics` → structured checks (Redis/headless/LLM/cache/limits)
   * `GET /metrics` → Prometheus text

2. **Schemas**

   * `GET /schemas` → list (e.g., `["events.v1","jobs.v1"]`)
   * `GET /schemas/{schema}` → JSON Schema + keying/dedup + quality metadata

3. **Work endpoints**

   * `POST /extract` → one/few URLs → structured entity (fast path)
   * `POST /crawl` → batch URLs → array
   * `POST /crawl/stream` → same request; **NDJSON** streaming
   * `POST /spider` → create job (seed URLs, depth, limits) → `{job_id}`
   * `GET /jobs/{job_id}` → status/counters/errors/links
   * `GET /jobs/{job_id}/stream` → NDJSON results

4. **Discovery (future-proof)**

   * `POST /discover` → auto find sources for a `schema` + scope (e.g., city)

5. **Read APIs**

   * `GET /entities/{schema}` → query normalized entities (filters, cursor, projection)
   * `GET /format/{schema}.{fmt}` → formatter plugins (e.g., `events.v1.ics`)

### Uniform request **options** block (replaces “tech-specific” endpoints)

```json
{
  "profile": "lite|full",
  "strategies": {
    "allow": ["ics","json","rulepack","wasm","llm","pdf","tables","headless","stealth"],
    "deny": [],
    "llm": { "enabled": true, "max_calls": 50, "max_cost_eur": 2.5 },
    "headless": { "enabled": true, "timeout_ms": 12000 },
    "stealth": { "enabled": true, "fingerprint": "randomized|fixed" }
  },
  "crawl": {
    "concurrency": 8, "rate_limit_per_host": 2, "robots": "respect",
    "max_depth": 2, "max_pages": 200, "cache_ttl_sec": 86400
  },
  "quality": { "min_confidence": 0.75, "require_fields": [] },
  "dedup": { "enabled": true, "window_days": 90 },
  "geo": { "city": "Amsterdam", "timezone": "Europe/Amsterdam" },
  "debug": { "provenance": true, "emit_raw": false }
}
```

### Legacy → v1 **shim map** (no loss; 90-day window)

* `/spider/frontier*` → `/spider` + `/jobs/*` (frontier internalized; expose counters)
* `/crawl-and-extract` → `/crawl`
* `/extract/advanced` → `/extract` with `strategies.allow+=["rulepack","wasm","headless"]`
* `/extract/llm*` → `/extract` with `strategies.llm.enabled=true`
* `/pdf/*` → `/extract` with `strategies.pdf=true` (tables likewise)
* `/stealth/*` → options: `strategies.stealth.enabled=true`
* Engine probing/decide/analyze → internal step; expose via `/diagnostics` or `debug` fields
* Admin/config routes (providers/profiles) → move under `/api/v1/admin/*` (auth-gated)

---

# C. Configuration model (no surprises)

* **Precedence:** **Request > profile > server defaults > schema registry.**
* **`server.yaml`** (checked-in, single source of truth):

  * `defaults` (safe, `lite`): deny `headless/llm/stealth`; conservative crawl
  * `profiles.full`: allow all strategies; higher limits
  * `integrations`: `redis_url`, `headless_pool_url`, LLM keys, search API keys, snapshot store config
* **Env vars:** only to override `server.yaml` (prefix `RIPTIDE_`); reduce 150 → curated set.

---

# D. Schema-agnostic engine (what we add, exactly)

1. **Schema Registry files**

   * `/schemas/events.v1.json`, `/schemas/jobs.v1.json`, `/schemas/registry.json`
   * include `json_schema`, `key_fn`, `quality.required`, `dedup.params`

2. **Adapters** (zero domain logic in core)

   * Trait: `StrategyOut -> Partial<Entity>` using schema metadata
   * Validate final entity against JSON Schema before emit; invalid → quarantine

3. **Dedup**

   * Per-schema key function (e.g., events: `hash(title, start±30m, venue)`); window configurable

---

# E. Per-crate instructions (explicit)

> Owner column = who should take it. Each item ends with an acceptance test.

1. **riptide-api — Router, v1, shims, OpenAPI** *(Backend)*

   * Implement `/api/v1/*` endpoints above.
   * Build shim middleware mapping legacy → v1 + `options`.
   * Emit `Deprecation` header with doc link.
   * **OpenAPI** for v1 only; legacy excluded.
     **Accept:** All legacy smoke tests pass via shims; `openapi.yaml` lints; `/healthz` green.

2. **riptide-api-types — DTOs** *(Backend)*

   * Define request/response structs for v1; versioned.
   * Used at boundary only (no leaking internal types).
     **Accept:** v1 handlers compile against DTOs; downstream SDK/CLI use DTOs.

3. **riptide-cli — Thin client** *(SDK/CLI)*

   * Subcommands: `extract|crawl|crawl --stream|spider|discover|doctor`.
   * `--schema`, `--url`/`RIPTIDE_BASE_URL`, `--profile`, pass-through strategy flags.
   * Stdout data; stderr progress; exit codes 0/1/2/3.
     **Accept:** Snapshot tests for 5 commands; Windows/macOS/Linux CI green.

4. **riptide-types — Core types** *(Platform)*

   * Keep internal models; **do not** expose in API (use DTOs).
   * Add `StrategyOut` & `PartialEntity` skeletons used by adapters.
     **Accept:** No API code imports `riptide-types` models directly.

5. **riptide-config — Unified config** *(Platform)*

   * Load `server.yaml`; implement precedence chain; map env → config.
   * Provide validated `Profiles` and integration blocks.
     **Accept:** `doctor` shows resolved config; invalid keys fail fast with hints.

6. **riptide-events — Event bus** *(Platform)*

   * Keep; add v1 event topics (`job_created`, `entity_emitted`, `budget_exceeded`).
     **Accept:** Events emitted in spider/crawl flows; visible in logs.

7. **riptide-monitoring — Observability** *(SRE)*

   * `/diagnostics` JSON; Prometheus with labels: route, schema, strategy, host.
   * Histograms for latency; counters for errors; gauges for headless/LLM usage.
     **Accept:** Grafana shows headless%/LLM%/errors by route; `/diagnostics` actionable.

8. **riptide-fetch — HTTP & politeness** *(Platform)*

   * Respect robots by default; per-host token buckets; retries w/ backoff+jitter.
   * Timeouts surfaced in `options`.
     **Accept:** Tests prove no more than N concurrent to same host.

9. **riptide-extraction — Core extractors** *(Platform)*

   * Implement strategy runner order (`allow/deny` configured).
   * Produce `StrategyOut` + provenance consistently.
     **Accept:** Fixtures for HTML/JSON/ICS/PDF pass; provenance present.

10. **riptide-pool — Resource pools** *(Platform)*

    * Pooling for headless sessions, LLM clients; enforce limits from options/profile.
      **Accept:** Saturation tests cap concurrency; diagnostics show pool health.

11. **riptide-browser-abstraction / riptide-headless / riptide-browser — Headless** *(Platform)*

    * CDP integration; **heuristic** headless trigger; time-boxed.
      **Accept:** JS-heavy fixtures succeed with headless ON; OFF returns partial/flagged.

12. **riptide-stealth — Anti-detection** *(Platform)*

    * Toggle via options; profiles determine default (off in `lite`).
      **Accept:** When enabled, fingerprint applied; logs show mode.

13. **riptide-reliability — Retries/circuits** *(Platform)*

    * Centralize retry policies; circuit breakers for flaky hosts/providers.
      **Accept:** Chaos tests trip breaker; recover after cooldown.

14. **riptide-intelligence — LLM providers** *(AI)*

    * Schema-constrained JSON; cost/budget caps; prompt packs per schema kit.
      **Accept:** LLM OFF returns rules-only; ON respects max_calls/cost.

15. **riptide-search — Deep search** *(Search)*

    * Provider adapters (e.g., Serper); rate limits; errors → degrade gracefully.
      **Accept:** `/discover` works when key present; clear error otherwise.

16. **riptide-spider — Jobs (frontier internal)** *(Backend)*

    * Move frontier to **internal**; expose jobs API; emit counters.
      **Accept:** Job lifecycle endpoints show progress; stream emits entities.

17. **riptide-pdf / riptide-extraction (tables)** *(Platform)*

    * PDF extraction and table parsing as **strategies** (flagged).
      **Accept:** PDF fixtures pass size/time caps; table outputs present when enabled.

18. **riptide-cache — Caching** *(Platform)*

    * Unified keyspace; TTL policy; per-request `cache_ttl_sec` honored.
      **Accept:** Cache hit/miss metrics visible; TTL respected.

19. **riptide-security — Auth/limits** *(Platform/SRE)*

    * API key support (admin vs data-plane); rate limits per IP/key.
      **Accept:** Unauthorized admin routes blocked; rate-limit tests pass.

20. **riptide-persistence — Stores** *(Platform/Data)*

    * Raw snapshot store (HTML/ICS/PDF digests) + normalized entity store; dedup view.
      **Accept:** Repro is possible via snapshot; dedup window applied.

21. **riptide-performance — Profiling** *(SRE)*

    * pprof hooks; jemalloc opt-in; flamegraphs for hot paths.
      **Accept:** Profiling docs + sample run artifacts.

22. **riptide-streaming — NDJSON transport** *(Backend)*

    * Flush per item; backpressure aware; heartbeat keepalive.
      **Accept:** Large streams don’t stall; client disconnect handled.

23. **riptide-test-utils — Fixtures & snapshots** *(QA)*

    * Test server for fixtures; golden NDJSON snapshots; schema validation harness.
      **Accept:** CI produces snapshot diffs on change.

24. **riptide-facade — Pipeline entrypoint** *(Backend)*

    * Single `run_pipeline(inputs, options)` used by extract/crawl/spider.
      **Accept:** /extract, /crawl, /spider call same facade with different inputs.

25. **riptide-workers — Background** *(Backend)*

    * Job queue/worker runtime; retries with backoff; graceful shutdown.
      **Accept:** Spider jobs survive restarts; idempotent resume.

26. **riptide-events (again)** *(Platform)*

    * Emit audit/provenance signals; optionally webhooks later.
      **Accept:** Event log shows consistent lifecycle.

---

# F. Testing matrix (what proves we kept everything)

* **Legacy parity:** recorded requests for each legacy family → shimmed v1 output **entity-equal** (order/ids ignored).
* **Streaming parity:** `/crawl` vs `/crawl/stream` → same set for 100 URLs.
* **Strategies:** HTML/JSON/ICS/PDF/JS-heavy/stealth/LLM ON/OFF fixtures.
* **Schema validation:** all outputs validate (`events.v1`, `jobs.v1` minimal); invalid → quarantined.
* **Budgets/limits:** default `lite`: 0% LLM, ≤5% headless; `full`: ≤10% LLM, ≤25% headless on the fixture suite.
* **Diagnostics:** missing Redis/headless/keys → actionable `/diagnostics` & `cli doctor` hints.

---

# G. Milestones, owners, acceptance (tight)

**Week 1 — v1 + shims (no features lost)**

* riptide-api, api-types, facade, streaming; CLI aligned; OpenAPI published.
* **Accept:** legacy tests pass through shims; new v1 endpoints green.

**Weeks 2–3 — Schema-agnostic core**

* Schema Registry + adapters; events.v1 complete; jobs.v1 minimal; schema validation on write.
* **Accept:** schema tests pass; dedup working; `/entities/{schema}` live.

**Weeks 3–4 — Ops/budgets/diagnostics**

* `/diagnostics`, metrics, cost/latency caps, rate limits, `doctor`.
* **Accept:** dashboards show headless/LLM usage within caps; alerts configured.

**(Optional) Weeks 4–6 — Discovery/city**

* `/discover`, SourceAudit; city orchestrator; Amsterdam daily run.
* **Accept:** automatic city refresh works; coverage/freshness KPIs recorded.

---

# H. What ships (explicit deliverables)

* **Stable `/api/v1`** + **OpenAPI** (+ **legacy shims** with 90-day deprecation).
* **DTO crate**; **thin CLI** with parity; prebuilt binaries.
* **Schema Registry** (`events.v1`, `jobs.v1`) + adapters + JSON Schema validation.
* **Unified options** replacing tech-specific endpoints (headless/stealth/WASM/LLM/PDF/tables).
* **Diagnostics/metrics** with budgets & alerts; **provenance & snapshots**.
* **Test suites** (legacy parity, streaming parity, strategy toggles, schema validation, golden snapshots).
* **Docs:** mapping table (legacy → v1+options), `server.yaml` reference, profile guide, examples.

Love it—let’s tighten the screws. Based on your analysis (26 crates, 120+ routes, lots of env/flags), here’s what still needs to be **explicitly instructed** so the current Riptide is reworked correctly and nothing slips through the cracks. Think of this as the “balance & hardening” addendum to the plan you just approved.

---

# 1) Architecture hygiene (make the pieces fit cleanly)

**Actions**

* **Single pipeline facade:** expose exactly one async entrypoint in `riptide-facade`
  `run_pipeline(inputs, options) -> Stream<ResultItem>` used by `/extract|/crawl|/spider`.
  *Acceptance:* all three handlers call this function; no alternate code paths.
* **Trait boundaries:** in `riptide-strategies`, standardize:

  ```rust
  trait Strategy { fn name(&self)->&'static str; async fn apply(&self, ctx:&PageCtx, cfg:&StrategyCfg)->StrategyOut; }
  ```

  *Acceptance:* ics/json/rulepack/wasm/llm/pdf/tables/headless/stealth all implement this.
* **Async/runtime alignment:** ensure all crates use **Tokio** (multi-thread), same features.
  *Acceptance:* no `async-std` or mixed executors; one runtime in the API.
* **Error typing:** one `AppError` enum at `riptide-api-types` mapped to HTTP (see §2).
  *Acceptance:* no stringly-typed `500`s.

**Why**: removes drift between crates and guarantees the new v1 endpoints all ride the same rails.

---

# 2) API contract details (no ambiguity left)

**Error model (uniform across endpoints)**

| HTTP | code                   | When                            | Client action             |
| ---- | ---------------------- | ------------------------------- | ------------------------- |
| 400  | `BAD_REQUEST`          | invalid args/schema             | fix request               |
| 401  | `UNAUTHENTICATED`      | missing/invalid API key (admin) | send valid key            |
| 403  | `FORBIDDEN`            | feature forbidden by policy     | request higher tier       |
| 404  | `NOT_FOUND`            | job/entity missing              | verify id                 |
| 409  | `CONFLICT`             | duplicate job/lock              | retry later               |
| 422  | `SCHEMA_INVALID`       | entity fails JSON Schema        | adjust strategies/quality |
| 429  | `RATE_LIMITED`         | per-IP/key rate exceeded        | back off                  |
| 500  | `INTERNAL`             | unhandled error                 | retry w/ jitter           |
| 503  | `UPSTREAM_UNAVAILABLE` | headless/LLM/search down        | failover/backoff          |

*Acceptance:* error JSON format:

```json
{"code":"SCHEMA_INVALID","message":"title missing","hint":"enable rulepack","incident_id":"abc123"}
```

**Versioning**

* Only `/api/v1/*` is supported going forward. Legacy routes get a 90-day shim with a `Deprecation` header and link to a mapping table.
* **Semver** for API responses via Accept header is optional; keep `/api/v1` stable for this phase.

---

# 3) Config & flags consolidation (kill drift)

**Actions**

* Replace scattered envs with **`server.yaml`** + minimal env overrides (`RIPTIDE_*`).
  *Acceptance:* the service boot log prints the resolved config tree (with redacted secrets).
* **Precedence:** request > profile (`lite`/`full`) > server defaults > schema registry.
  *Acceptance:* targeted unit tests flip the same knob via all 4 layers and show the same end result.
* **Flags rationalization:** collapse per-crate feature flags into the **options block** (strategies/crawl/quality/dedup/debug).
  *Acceptance:* no crate reads bespoke flags at request time.

---

# 4) Strategy normalization (keep all features, remove endpoint sprawl)

**Actions**

* Move PDF, tables, stealth, and headless switches behind `options.strategies.*`.
* Implement **ordering & short-circuit** rules:
  `ics/json → rulepack → wasm → llm` (stop on valid entity above confidence threshold).
* Add **budget gates**: LLM `max_calls`, `max_cost_eur`; headless timeout & pool caps (per-request and profile).
* Record **provenance** on each entity: url, strategy path, timings, content hashes.

*Acceptance:* fixtures for HTML/JSON/ICS/PDF/JS-heavy/stealth/LLM all pass with correct toggles, costs under caps.

---

# 5) Headless/CDP subsystem (stability & cost)

**Actions**

* **Heuristic gate**: trigger headless only when static parse yields empty/low-signal + script-heavy DOM.
* **Pool health**: pre-flight CDP, recycle crashed sessions, circuit-break on high error rate.
* **Timeouts**: hard per-page wall clock and per-step caps; screenshot/log on fatal to reproduce.

*Acceptance:* JS-heavy fixture green with headless ON; OFF emits partial + `low_confidence` flag. Pool stats shown in `/diagnostics`.

---

# 6) LLM subsystem (safe & bounded)

**Actions**

* **Schema-constrained JSON**: generate only objects that validate against the target JSON Schema; on fail, retry once with automatic “repair” prompt; else quarantine.
* **Deterministic prompts** in `/schema-kits/<schema>/prompts/…`; include few-shot examples and field-by-field constraints.
* **Cost tracking**: per-request budget + per-profile daily budget; hard-stop when exceeded.

*Acceptance:* With LLM OFF → rule-only outputs; ON → valid JSON, never exceeding `max_cost_eur`, logged in metrics.

---

# 7) PDF & tables (fold-in cleanly)

**Actions**

* Treat PDF parse/table extraction as **strategies** with explicit `enabled` flags and size/page/time caps.
* Normalize table outputs (rows/cols + inferred headers) under a standard field (e.g., `tables`) so adapters can consume them.

*Acceptance:* Large-PDF fixture processed within caps; failure returns `UPSTREAM_UNAVAILABLE` or a structured strategy error, not a 500.

---

# 8) Caching, persistence, and determinism

**Actions**

* **Raw snapshot store**: persist compressed HTML/ICS/PDF keyed by URL+timestamp (for replay).
* **Normalized store**: append-only entities + **dedup view** per schema; store dedup links/provenance.
* **Cache**: explicit TTL; vary by strategy; cache key includes strategy + major config toggles.

*Acceptance:* “Replay” command regenerates entities from snapshot and matches golden JSON (ignoring nondeterministic fields).

---

# 9) Schema Registry + adapters (anchor the agnostic future)

**Actions**

* Ship `events.v1.json` and minimal `jobs.v1.json` with `registry.json` (keying/quality/dedup metadata).
* Implement adapter trait and validate **every** emitted entity against the schema.
* Quarantine invalids with clear reasons; include a minimal UI/CLI to dump quarantine reasons.

*Acceptance:* 100+ fixtures validate; quarantine rate < 5% on the suite.

---

# 10) Observability & diagnostics (see what it’s doing)

**Actions**

* `/diagnostics`: Redis/headless/LLM/search keys, pool sizes, budgets status, cache stats, per-host concurrency.
* Prometheus labels: `route`, `schema`, `strategy`, `host`; histograms on latency; counters for each error code; gauges for **headless%** and **LLM calls/page**.
* **Doctor** (CLI): mirror diagnostics and provide **fix-it** hints.

*Acceptance:* Grafana board shows: success rate, headless %, LLM cost, error spikes by strategy, and top failing hosts.

---

# 11) Security, robots, and compliance

**Actions**

* Default **robots: respect**; allow `ignore` only by explicit option (and log).
* Distinct **admin** routes under `/api/v1/admin/*` with keys/roles; data-plane remains unauthed (or simple key if you prefer).
* Secrets: env or secret manager; **never** serialized in diagnostics/logs (redaction filters).
* Rate limits per-IP/key; request-size caps.

*Acceptance:* Security tests for admin routes (401/403); robots override visible in logs; secrets redacted everywhere.

---

# 12) DevEx, build, and release

**Actions**

* One-liners (Make/Just): `dev-up`, `curl-health`, `crawl-stream <url>`.
* Reproducible builds: lockfiles; `rust-toolchain.toml`; CI matrix (Linux/macOS/Windows).
* Prebuilt **CLI binaries** on release; OpenAPI published from CI.

*Acceptance:* New machine boots dev in <10 min; all CI jobs green; binaries downloadable.

---

# 13) Migration & communication (no surprises)

**Actions**

* Publish **legacy → v1 mapping table** (every route → new path + `options` translation).
* Shims live for **90 days** with `Deprecation` header and link to docs.
* Weekly check: % traffic still on legacy; goal 0% by Week 12.

*Acceptance:* Contract tests verify mapping parity; docs page exists and is referenced in responses.

---

# 14) Final acceptance checklist (print this in the release PR)

* [ ] `/api/v1` live; legacy routes shimmed; deprecation headers present
* [ ] OpenAPI published; CLI aligned; DTO crate used everywhere at boundary
* [ ] Schema Registry with `events.v1` (+ `jobs.v1` minimal); adapters validate outputs
* [ ] All strategies accessible via **options**, including headless/stealth/WASM/LLM/PDF/tables
* [ ] Provenance present on every entity; snapshots stored
* [ ] Diagnostics actionable; dashboards show headless%/LLM%/errors by route
* [ ] Cost/latency budgets enforceable per request & profile
* [ ] Contract tests (legacy parity, streaming parity) pass; schema validation suite green
* [ ] Dev UX: `dev-up`, `doctor`, examples run on a fresh machine
* [ ] Migration docs (mapping table) published; comms plan sent

---

## Short “who does what” (to assign today)

* **API & Shims & DTOs:** `riptide-api` / `riptide-api-types` — *Backend*
* **Pipeline & Strategies:** `riptide-facade` / `riptide-strategies` / `riptide-extraction` — *Platform*
* **Headless & Stealth:** `riptide-headless` / `riptide-browser*` / `riptide-stealth` — *Platform*
* **LLM & Search:** `riptide-intelligence` / `riptide-search` — *AI/Search*
* **Schemas & Adapters:** `riptide-schemas` / `riptide-adapters` — *Data*
* **Jobs/Workers:** `riptide-spider` / `riptide-workers` — *Backend*
* **Stores/Cache:** `riptide-persistence` / `riptide-cache` — *Platform/Data*
* **Obs/Diag/SRE:** `riptide-monitoring` / `riptide-performance` — *SRE*
* **CLI & Docs:** `riptide-cli` / docs site — *SDK/Docs*


Excellent catch — yes, **search is core to Riptide’s autonomy**, and it *must* be formalized in the plan (not treated as an external afterthought).

Let’s correct that omission by fully integrating **search (Serper and others)** into the architecture and API. Below is the **Search System Addendum**, extending the prior blueprint.

---

# 🔍 Search System Addendum — “Discovery & Source Enrollment”

## 1️⃣ Purpose

Before Riptide can crawl or extract, it must **find candidate sources** (sites, feeds, APIs, etc.) for a given schema (events, jobs, products).
This is handled by the **Search and Discovery subsystem**, which automates:

| Stage              | Description                                                                 |
| ------------------ | --------------------------------------------------------------------------- |
| **Discovery**      | Use web search (Serper, Bing, Brave, or local index) to find candidate URLs |
| **Classification** | Identify content type (`listing`, `calendar`, `detail`, `api`, etc.)        |
| **Enrollment**     | Validate, snapshot, and store in `source_registry` with crawl strategy      |
| **Refresh**        | Periodically re-search to add new or updated sources                        |

---

## 2️⃣ Implementation Layers

### A. Crate: `riptide-search`

Handles search provider abstraction and enrichment.

**Responsibilities**

* Plug multiple providers: `serper`, `bing`, `duckduckgo`, `brave`, `localindex`
* Support search intents: `events in <city>`, `jobs in <region>`, etc.
* Rate limit + retry + cache per query
* Return structured results `{title, url, snippet, rank, provider, retrieved_at}`

**Interface**

```rust
pub struct SearchRequest {
    pub query: String,
    pub provider: Provider,
    pub max_results: usize,
    pub schema: Option<String>,
    pub locale: Option<String>
}

pub struct SearchResult {
    pub url: String,
    pub title: String,
    pub snippet: Option<String>,
    pub provider: String,
    pub rank: u32,
    pub timestamp: DateTime<Utc>
}
```

**Acceptance**

* Query “events in Amsterdam” → ≥30 unique candidate URLs
* Supports failover from Serper → Bing → DuckDuckGo
* Cache hit ratio ≥ 60% for repeated queries

---

### B. Crate: `riptide-discovery`

Uses `riptide-search` + heuristics to **classify and enroll** new sources.

**Responsibilities**

* Normalize URLs (resolve redirects, canonicalize)
* Classify by DOM patterns or ML model (listing, event, job, etc.)
* Assign extraction strategy (`rulepack`, `headless`, `api`)
* Store in `source_registry` with confidence score

**Data model**

```rust
struct SourceCandidate {
    url: String,
    schema: String,
    class: String, // listing, calendar, api
    score: f32,
    verified: bool,
    last_checked: DateTime<Utc>
}
```

**Acceptance**

* Given “events in Amsterdam” → 10 verified, crawlable event sources stored
* Classification accuracy ≥ 80% on test set

---

### C. Schema Integration

Each schema (`events`, `jobs`, etc.) defines **discovery templates**:

```json
{
  "schema": "events.v1",
  "queries": [
    "events in {city}",
    "what’s on {city}",
    "concerts in {city}",
    "site:.nl events {city}"
  ],
  "filters": {
    "exclude_domains": ["facebook.com","linkedin.com"]
  }
}
```

Stored in `/schema-kits/<schema>/discover.yaml`.

---

## 3️⃣ API Integration

### `/discover` endpoint (already in OpenAPI)

**Request**

```json
{
  "schema": "events.v1",
  "scope": { "city": "Amsterdam", "country": "NL" },
  "providers": ["serper"],
  "max_sources": 200
}
```

**Response**

```json
{
  "enrolled": [
    {"url":"https://paradiso.nl/events/","score":0.97,"strategy":"rulepack"}
  ],
  "candidates": [
    {"url":"https://iamsterdam.com/en/whats-on","score":0.83}
  ],
  "rejected": [
    {"url":"https://facebook.com/events","reason":"blocked_domain"}
  ]
}
```

**Options**

* `providers`: choose provider(s)
* `audit_only`: run discovery without enrollment
* `confidence_threshold`: float 0–1

**Acceptance**

* Returns ≥ 10 unique crawlable sources for a medium city within 30s.
* All enrolled sources validate (200 OK + extractable HTML).

---

## 4️⃣ Storage

### `source_registry` table or file

| field          | description                    |
| -------------- | ------------------------------ |
| `url`          | canonical URL                  |
| `schema`       | `events.v1`                    |
| `strategy`     | `rulepack`, `headless`, `api`  |
| `confidence`   | float                          |
| `status`       | `active`, `blocked`, `expired` |
| `last_checked` | datetime                       |

---

## 5️⃣ CLI Support

```
riptide discover --schema events --scope "Amsterdam, NL" --max-sources 200
riptide sources list --schema events
riptide sources refresh --schema events
```

**Acceptance**

* CLI output streams enrolled URLs (JSONL)
* `sources list` prints status & scores

---

## 6️⃣ Operational Guardrails

| Control              | Description                                    |
| -------------------- | ---------------------------------------------- |
| **Rate limits**      | per-provider (Serper, Bing)                    |
| **API key rotation** | via `server.yaml` integration block            |
| **Retry policy**     | exponential backoff; circuit breaker on 429    |
| **Cache TTL**        | 24h per `(schema, scope)`                      |
| **Cost tracking**    | store per-query cost if provider paid (Serper) |

---

## 7️⃣ How it ties to the pipeline

`/discover` populates `source_registry` → `spider` reads from it → `crawl` uses the sources.

Flow:

```
Discover (search) → Classify (DOM) → Enroll (registry) → Crawl → Extract → Store
```

This turns Riptide into a **self-growing agent** — not just crawling fixed seeds, but **finding, classifying, and maintaining** its own source set.

---

## 8️⃣ Required Work Items

| Crate                 | Task                                                          |
| --------------------- | ------------------------------------------------------------- |
| `riptide-search`      | unify Serper/Bing/Brave; caching + cost tracking              |
| `riptide-discovery`   | classification + enrollment                                   |
| `riptide-api`         | `/discover` endpoint wiring                                   |
| `riptide-cli`         | `discover` & `sources` subcommands                            |
| `riptide-persistence` | `source_registry` table                                       |
| `riptide-monitoring`  | discovery metrics: candidates, verified, fail rate            |
| `riptide-config`      | `integrations.serper_api_key`, query templates in schema kits |

---

## ✅ Acceptance Summary

* Search layer added (`riptide-search`) with Serper integration and caching
* Discovery layer (`riptide-discovery`) classifies and enrolls sources
* `/discover` API and CLI live and tested
* Registry populated and used by crawler
* Schema kits define discovery templates per domain
* Metrics visible (discovery coverage, candidate success rate, cost per query)

---

Would you like me to now generate the **actual Rust module stubs** (`riptide-search`, `riptide-discovery`, and the `/discover` handler signature**) so your team can drop them in directly?
