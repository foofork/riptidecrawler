Perfect.
Here is **Doc 1 – Executive Overview**, written explicitly for an **AI developer** with full repository access.
It assumes no unstated knowledge and provides concrete directives and references.

---

# 📘 Doc 1 — Executive Overview

**Riptide v1 Consolidation and Stabilization Roadmap**

---

## 1. Purpose and Scope

**Goal:** Consolidate all existing Riptide capabilities (26 crates, 120+ routes, ~150 environment variables) into a single stable, schema-agnostic, versioned platform with no feature loss and explicit observability.

**Outcome:**
After implementation, every crawl, extract, or discovery task must flow through one canonical API surface (`/api/v1`), one pipeline entrypoint (`riptide-facade::run_pipeline()`), one configuration model (`server.yaml` + env overrides), and one validation schema (registry + JSON Schema).

---

## 2. Immutable Invariants (“Ground Truths”)

These statements are not to be re-interpreted or modified unless this document is superseded:

1. **Public API Root:** `/api/v1` is the only stable surface.

   * Legacy routes remain as temporary shims (90 days max).
   * All shimmed routes emit a `Deprecation` header with mapping to `/api/v1`.

2. **Crate Topology:** Exactly the following crates exist in the workspace (no `riptide-core`):

   * Foundation layer: `riptide-types`, `riptide-config`, `riptide-utils`, `riptide-monitoring`, `riptide-reliability`, `riptide-performance`, `riptide-events`.
   * Service layer: `riptide-extraction`, `riptide-browser*`, `riptide-headless`, `riptide-stealth`, `riptide-intelligence`, `riptide-search`, `riptide-persistence`, `riptide-cache`, `riptide-workers`.
   * Interface layer: `riptide-facade`, `riptide-api`, `riptide-cli`.
   * Support layer: `riptide-schemas`, `riptide-adapters`, `riptide-validation`, `riptide-test-utils`.
   * Utilities: `riptide-utils` provides shared HTTP/Redis/error/time helpers.

3. **Pipeline Entry:** Only `riptide-facade::run_pipeline(inputs, options)` may orchestrate multi-strategy extraction, crawling, or spidering. No alternate code paths.

4. **Configuration Precedence:**

   ```
   Request options > Profile (full|lite) > server.yaml defaults > Schema registry defaults
   ```

   Implementation of precedence must exist in `riptide-config`.

5. **Validation and Schemas:**

   * All entities emitted must validate against the JSON Schemas stored in `/schemas/*.json`.
   * Invalid entities go to quarantine with a reason code defined in `riptide-adapters`.

6. **Deduplication:**

   * Keys computed per schema using registry `key_fn`.
   * Window enforcement and replayability required.

7. **Observability:**

   * `/diagnostics` → machine-readable system health.
   * `/metrics` → Prometheus text.
   * Grafana dashboards show headless%, LLM%, error rates per route.

8. **No Feature Loss:** All historical behaviors accessible via shims must remain reachable through `/api/v1` with equivalent outcomes.

---

## 3. Overall Execution Phases

| Phase                                     | Duration    | Primary Deliverables                                                                                      | Key Acceptance                                                |
| ----------------------------------------- | ----------- | --------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| **Phase 0 – Preparation**                 | Week 0 → 1  | `server.yaml` implemented; `riptide-utils` created; OpenAPI v1 checked in; CI baseline green.             | Service boots with config precedence validated.               |
| **Phase 1 – Canonical API and Shim**      | Week 1 → 2  | `/api/v1` endpoints implemented per `openapi.yaml`; legacy shims map; DTO crate active.                   | Legacy tests pass via shims; OpenAPI lints; `/healthz` green. |
| **Phase 2 – Schema Registry & Adapters**  | Week 2 → 3  | `schemas/registry.json`, `events.v1.json`, `jobs.v1.json`, adapter engine with validation and quarantine. | All fixtures validate; < 5 % quarantine rate.                 |
| **Phase 3 – Diagnostics & Budgets**       | Week 3 → 4  | `/diagnostics`, metrics, cost/latency caps, rate limits, CLI doctor.                                      | Dashboards show metrics within caps; alerts configured.       |
| **Phase 4 – Discovery & City Automation** | Week 4 → 6  | `/discover`, classification, source registry, daily run for Amsterdam.                                    | ≥ 10 verified sources per city; auto-refresh works.           |
| **Phase 5 – Hardening & Migration**       | Week 6 → 12 | Security policies, testing matrix, release plan, shim retirement.                                         | 0 % legacy traffic; production SLO ≥ 99.5 %.                  |

---

## 4. Cross-Cutting Requirements

### 4.1 Reliability

* All HTTP operations use the same `reqwest` client from `riptide-utils::http::client()`.
* Retries, backoff, and circuit-breakers handled in `riptide-reliability`.

### 4.2 Concurrency

* Tokio multi-threaded runtime only.
* Max in-flight requests per host = `options.crawl.rate_limit_per_host`.

### 4.3 Data Integrity

* Every entity carries a `provenance` block (`strategies_chain`, `confidence`, `snapshot_key`).
* Snapshot storage required in `riptide-persistence`.

### 4.4 Budget and Safety

* LLM cost tracked per request and per-profile (daily limit via `server.yaml` → `integrations.llm.daily_budget_eur`).
* Headless usage percent and LLM spend exposed in metrics.

### 4.5 Security

* Admin routes under `/api/v1/admin/*` require API key (`security.admin_api_keys` in config).
* All secrets redacted in logs and diagnostics.

### 4.6 Testing and Reproducibility

* Golden requests and responses exist for each strategy and endpoint.
* Schema validation must run in CI.
* Replay command (`riptide replay`) re-executes from snapshot and matches golden JSON.

---

## 5. Success Definition (End State)

By completion of this roadmap:

* `/api/v1` is the only supported surface; legacy routes removed.
* All tasks (`extract`, `crawl`, `spider`, `discover`) use `riptide-facade::run_pipeline()`.
* Config resolved by `riptide-config` with full precedence.
* All outputs validate against JSON Schema (≤ 5 % quarantine).
* LLM/headless usage and costs within profile budgets.
* `/diagnostics` and `/metrics` return actionable, complete data.
* CI green across Linux/macOS/Windows.
* Shim traffic = 0 %; OpenAPI and CLI published as public contracts.

---

## 6. References to Existing Assets (Repository Paths)

| Asset                        | Path                                                       | Description                                           |
| ---------------------------- | ---------------------------------------------------------- | ----------------------------------------------------- |
| **OpenAPI v1**               | `/openapi.yaml`                                            | Defines canonical API surface.                        |
| **Options Schema**           | `/schemas/options.schema.json`                             | Validation for request options block.                 |
| **Server Config**            | `/server.yaml`                                             | Single source of truth for defaults and integrations. |
| **Schema Registry**          | `/schemas/registry.json`                                   | Lists schemas and metadata.                           |
| **Entity Schemas**           | `/schemas/events.v1.json`, `/schemas/jobs.v1.json`         | JSON Schema validation targets.                       |
| **Adapters and Validation**  | `/crates/riptide-adapters/`, `/crates/riptide-validation/` | Mapping and schema validation.                        |
| **Pipeline Facade**          | `/crates/riptide-facade/src/lib.rs`                        | `run_pipeline()` entrypoint.                          |
| **Config Loader**            | `/crates/riptide-config/src/lib.rs`                        | Precedence implementation.                            |
| **CLI Tool**                 | `/crates/riptide-cli/`                                     | Command-line interface mirror of API.                 |
| **Monitoring & Diagnostics** | `/crates/riptide-monitoring/`                              | `/diagnostics` and metrics export.                    |

---

## 7. Next Document Dependencies

The next deliverable (**Doc 2 – Architecture & Crate Layout**) will expand on:

* The exact layering and allowed dependency graph.
* Required internal module boundaries.
* Explicit import/export contracts between crates.

No other sections may proceed until Doc 2 is accepted.

---

**End of Doc 1 – Executive Overview**
*(Authoritative high-level directive for all subsequent roadmap documents.)*

---

Would you like me to proceed now with **Doc 2 – Architecture & Crate Layout** (the explicit dependency and interface map for every crate)?
Excellent.
Below is **Doc 2 – Architecture & Crate Layout**, written for an AI developer that must act deterministically within the existing codebase.

---

# 🧱 Doc 2 — Architecture & Crate Layout

**Riptide v1 Structural Specification**

---

## 1. Purpose

Define every crate’s role, allowed dependencies, and file-level entrypoints so that build, runtime, and orchestration logic are reproducible without inference.
This document is a contract: each listed crate **must exist** under `/crates/<crate-name>/` with a valid `Cargo.toml` manifest and a `src/lib.rs` entrypoint unless noted otherwise.

---

## 2. Layering Model

### 2.1 Three Runtime Layers (+ Support and Tools)

| Layer                    | Description                                                                                                                                                            | Example Crates                                                                                                                                                   |
| ------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Interface Layer**      | External entrypoints (HTTP API, CLI). Handles request decoding, auth, diagnostics, and invokes the facade.                                                             | `riptide-api`, `riptide-cli`                                                                                                                                     |
| **Orchestration Layer**  | Single execution facade and job coordination. No HTTP or CLI code allowed.                                                                                             | `riptide-facade`, `riptide-workers`                                                                                                                              |
| **Service Layer**        | Implements functional capabilities (crawl, extract, LLM, cache, persistence, etc.). Each service is self-contained and stateless except where persistence is explicit. | `riptide-extraction`, `riptide-browser`, `riptide-intelligence`, `riptide-persistence`, `riptide-cache`, `riptide-search`, `riptide-headless`, `riptide-stealth` |
| **Foundation Layer**     | Shared platform libraries and system integrations: types, config, monitoring, reliability, performance, events, utils.                                                 | `riptide-types`, `riptide-config`, `riptide-utils`, `riptide-monitoring`, `riptide-reliability`, `riptide-performance`, `riptide-events`                         |
| **Support / Data Layer** | Schema definition, adapters, validation, and test fixtures.                                                                                                            | `riptide-schemas`, `riptide-adapters`, `riptide-validation`, `riptide-test-utils`                                                                                |

---

## 3. Dependency Rules (Enforced via CI)

### 3.1 Allowed Imports

| From → To                         | Allowed ?                                                | Notes |
| --------------------------------- | -------------------------------------------------------- | ----- |
| Interface → Orchestration         | ✅ Required. API and CLI call facade functions only.      |       |
| Interface → Service               | 🚫 Forbidden (services must be accessed through facade). |       |
| Orchestration → Service           | ✅ Yes; facade owns task execution graph.                 |       |
| Orchestration → Foundation        | ✅ Yes.                                                   |       |
| Service → Foundation              | ✅ Yes (config, types, utils, monitoring).                |       |
| Foundation → Service or Interface | 🚫 Never. Foundation must not import upwards.            |       |
| Support → Foundation or Service   | ✅ Validation uses types and schemas.                     |       |
| Any → CLI                         | 🚫 CLI is top-level consumer only.                       |       |

### 3.2 CI Policy

Add a `cargo-deny` or equivalent rule:

```
[bans]
allow = [
  "riptide-types -> *",
  "riptide-config -> *",
  "riptide-utils -> *",
  "riptide-facade -> riptide-*"
]
deny = [
  "riptide-api -> riptide-extraction",
  "riptide-api -> riptide-intelligence",
  "riptide-api -> riptide-browser"
]
```

Violation → CI fail with error `LAYERING_VIOLATION`.

---

## 4. Crate Responsibilities and Entrypoints

### 4.1 Interface Layer

| Crate         | Primary Entrypoint                     | Responsibilities                                                                                    | Exports              |
| ------------- | -------------------------------------- | --------------------------------------------------------------------------------------------------- | -------------------- |
| `riptide-api` | `src/main.rs` → Axum/Actix HTTP server | Implements OpenAPI v1 routes; legacy shim middleware; auth; diagnostics; metrics export             | None (public binary) |
| `riptide-cli` | `src/main.rs` → `Cli::run()`           | Implements subcommands (`extract`, `crawl`, `spider`, `discover`, `doctor`) and calls API or facade | None (binary)        |

### 4.2 Orchestration Layer

| Crate             | Entrypoint   | Responsibilities                                                                                                 |
| ----------------- | ------------ | ---------------------------------------------------------------------------------------------------------------- |
| `riptide-facade`  | `src/lib.rs` | Defines `async fn run_pipeline(inputs, options)`; executes strategy graph in order; returns `Stream<ResultItem>` |
| `riptide-workers` | `src/lib.rs` | Implements background job runner; reads queued jobs and calls facade; idempotent resume; graceful shutdown       |

### 4.3 Service Layer

| Crate                  | Responsibility                                                                      |
| ---------------------- | ----------------------------------------------------------------------------------- |
| `riptide-extraction`   | Implements Strategy trait and runner logic for HTML/JSON/ICS/PDF/tables strategies. |
| `riptide-browser`      | Base for Chromium CDP integration; provides DOM snapshot and evaluation utilities.  |
| `riptide-headless`     | Implements headless heuristics, pool management, timeouts, and session reuse.       |
| `riptide-stealth`      | Applies fingerprinting avoidance (UA, canvas, WebGL spoof).                         |
| `riptide-intelligence` | LLM providers; prompt packs; cost tracking.                                         |
| `riptide-search`       | Web search provider abstraction and discovery integration.                          |
| `riptide-persistence`  | Snapshot store and entity store implementations; dedup window enforcement.          |
| `riptide-cache`        | Caching layer (Redis or in-memory); TTL honor.                                      |

### 4.4 Foundation Layer

| Crate                 | Responsibility                                                    |
| --------------------- | ----------------------------------------------------------------- |
| `riptide-types`       | Core structs, error enums, DTOs (shared only within backend).     |
| `riptide-config`      | Loads `server.yaml`; applies precedence; validates env overrides. |
| `riptide-utils`       | HTTP client, Redis pool, time, error helpers.                     |
| `riptide-monitoring`  | Prometheus metrics, diagnostics collection.                       |
| `riptide-reliability` | Retry/circuit-breaker logic for HTTP and provider calls.          |
| `riptide-performance` | pprof hooks and profiling tools.                                  |
| `riptide-events`      | Event bus (`job_created`, `entity_emitted`, etc.).                |

### 4.5 Support / Data Layer

| Crate                | Responsibility                                                              |
| -------------------- | --------------------------------------------------------------------------- |
| `riptide-schemas`    | Holds registry and JSON Schema files.                                       |
| `riptide-adapters`   | Maps StrategyOut → PartialEntity; computes dedup keys; quarantine handling. |
| `riptide-validation` | Schema validation logic and adapter engine entry.                           |
| `riptide-test-utils` | Fixtures, snapshot helpers, CI golden comparisons.                          |

---

## 5. Shared Conventions (all crates)

### 5.1 Error Handling

* Use `thiserror` or `anyhow`.
* Global error type is `riptide_utils::errors::AppError`.
* All top-level handlers convert `AppError` → HTTP Error JSON per OpenAPI schema.

### 5.2 Async Runtime

* `tokio = { features = ["rt-multi-thread","macros","time","signal"] }`
* No `async-std`, no mixed executors.

### 5.3 Logging / Telemetry

* Use `tracing` crate with OpenTelemetry export from `riptide-monitoring`.
* Include fields `route`, `schema`, `strategy`, `host`, `duration_ms`.

### 5.4 Configuration Injection

* Each crate accepts a `Config` struct from `riptide-config::resolve()`.
* No direct env access inside runtime code.

### 5.5 Testing Structure

* Unit tests inside crate’s `tests/` folder.
* Integration and golden tests live in `riptide-test-utils`.
* Each crate must build in isolation (`cargo test -p <crate>`).

---

## 6. Inter-Crate Interface Contracts

### 6.1 Facade ↔ Services

`riptide-facade` imports:

* `riptide-extraction::{Strategy, StrategyOut}`
* `riptide-validation::AdapterEngine`
* `riptide-persistence::{EntityStore, SnapshotStore}`
* `riptide-cache::CacheClient`
* `riptide-monitoring::Metrics`
* `riptide-config::ResolvedConfig`

### 6.2 API ↔ Facade

`riptide-api` calls only:

```rust
use riptide_facade::run_pipeline;
```

and translates HTTP requests → DTOs from `riptide-types`.

### 6.3 CLI ↔ API or Facade

* Default mode: call HTTP API (`RIPTIDE_BASE_URL`).
* Fallback: call `riptide-facade` directly when `--local` flag set.

### 6.4 Workers ↔ Facade

`riptide-workers` uses facade internally; no direct service calls.

---

## 7. Filesystem Expectations (per crate)

Each crate must contain:

```
/crates/<name>/
  Cargo.toml
  src/lib.rs
  README.md
  tests/
```

Specific directories:

* `/crates/riptide-facade/src/strategies/` → strategy runner implementation.
* `/crates/riptide-validation/src/` → adapter engine and schema validation.
* `/crates/riptide-schemas/` → JSON files for registry and entities.
* `/crates/riptide-test-utils/tests/goldens/` → golden fixtures per endpoint.

---

## 8. Build and Runtime Contracts

### 8.1 Build

* Workspace root has `Cargo.toml` with `[workspace] members = ["crates/*"]`.
* `cargo build --all --locked` must succeed on Linux, macOS, Windows.

### 8.2 Runtime

* `riptide-api` binary is the only long-running HTTP process.
* `riptide-workers` binary optional for background jobs.
* Other crates are libraries.

### 8.3 Artifacts

* `target/release/riptide-api`
* `target/release/riptide-cli`
* `target/release/riptide-workers`

---

## 9. Acceptance Criteria for Architecture Stage

1. Each crate exists with matching responsibility.
2. `cargo metadata` shows no cyclic dependencies.
3. `cargo deny` layering rules pass.
4. `cargo test --workspace` green.
5. `riptide-api` binary boots with `server.yaml` and returns `/healthz → 200`.
6. Documentation (`/docs/architecture.md`) matches this spec.

---

## 10. Next Document Dependency

**Doc 3 – Phase Roadmap**
will enumerate every deliverable per phase (Prep → v1 Launch → Core Validation → Discovery → Hardening), with explicit entry and exit criteria mapped to the crates defined here.

---

**End of Doc 2 – Architecture & Crate Layout**

---

Shall I proceed with **Doc 3 – Phase Roadmap** (explicit per-phase deliverables, success gates, and dependencies)?
Excellent.
Here is **Doc 3 – Phase Roadmap**, written in the same AI-developer style: everything explicit, deterministic, and bound to repository paths and crate responsibilities defined in **Doc 2**.

---

# 📅 Doc 3 — Phase Roadmap

**Riptide v1 Execution Plan (Phased Implementation Sequence)**

---

## 1. Purpose

To describe the exact chronological sequence of implementation across all crates, with concrete deliverables, acceptance tests, and file locations.
Each phase ends with measurable success conditions and defines prerequisites for the next.

---

## 2. Phase Overview Table

| Phase                                | Duration    | Primary Goal                                                            | Crates Involved                                                                         | Gate to Next Phase                        |
| :----------------------------------- | :---------- | :---------------------------------------------------------------------- | :-------------------------------------------------------------------------------------- | :---------------------------------------- |
| **0 – Preparation & Baseline**       | Week 0 → 1  | Establish config infrastructure and utility foundation                  | `riptide-config`, `riptide-utils`, `riptide-api`, CI tooling                            | All crates build + boot /healthz 200      |
| **1 – Canonical API & Legacy Shims** | Week 1 → 2  | Expose stable `/api/v1` per OpenAPI; preserve legacy behavior via shims | `riptide-api`, `riptide-api-types`, `riptide-facade`, `riptide-cli`                     | Legacy smoke tests pass through shims     |
| **2 – Schema Registry & Adapters**   | Week 2 → 3  | Introduce schema-agnostic validation and dedup                          | `riptide-schemas`, `riptide-adapters`, `riptide-validation`, `riptide-persistence`      | All fixtures validate; < 5 % quarantine   |
| **3 – Diagnostics & Budgets**        | Week 3 → 4  | Observability and resource control                                      | `riptide-monitoring`, `riptide-reliability`, `riptide-intelligence`, `riptide-headless` | Metrics visible in Grafana; alerts firing |
| **4 – Discovery & City Automation**  | Week 4 → 6  | Add search/discovery automation for schemas                             | `riptide-search`, `riptide-discovery`, `riptide-persistence`, `riptide-cli`             | ≥ 10 verified sources per city            |
| **5 – Hardening & Migration**        | Week 6 → 12 | Security, testing, release, shim removal                                | All crates                                                                              | Legacy traffic 0 %; SLO ≥ 99.5 %          |

---

## 3. Phase 0 — Preparation & Baseline

### Objectives

* Create `riptide-utils` crate with HTTP, Redis, time, and error helpers.
* Implement `riptide-config` loading and precedence model (`server.yaml` + env overrides).
* Verify Tokio multi-thread runtime consistency.
* Establish CI workflows for build/test/lint and cargo-deny layering.

### Deliverables

| Artifact                           | Path                        | Purpose |
| ---------------------------------- | --------------------------- | ------- |
| `/crates/riptide-utils/src/lib.rs` | Reusable utility functions  |         |
| `/server.yaml`                     | Default config and profiles |         |
| `.github/workflows/ci.yml`         | Build/test/lint/deny checks |         |

### Acceptance Tests

* `cargo build --workspace` succeeds on all OS.
* `cargo deny` reports no layering violations.
* `/api/v1/healthz` returns 200 with resolved config profile.

---

## 4. Phase 1 — Canonical API & Legacy Shims

### Objectives

* Implement all `/api/v1` endpoints exactly per `openapi.yaml`.
* Create legacy-to-v1 shim middleware (`riptide-api/src/shims.rs`).
* Publish `riptide-api-types` DTO crate used only at API boundary.
* Update `riptide-cli` commands to use v1 DTOs.

### Deliverables

* `/openapi.yaml` committed and lint-clean.
* `/crates/riptide-api/src/routes/v1/*` handlers implemented.
* `/crates/riptide-cli/src/commands/*.rs` mirroring API calls.
* 90-day Deprecation headers active on legacy routes.

### Acceptance Tests

* Legacy requests → shim → v1 produce entity-equal results.
* `/healthz`, `/metrics`, and `/diagnostics` respond.
* CLI subcommands `extract`, `crawl`, `spider` return exit code 0 on fixtures.

---

## 5. Phase 2 — Schema Registry & Adapters

### Objectives

* Load `schemas/registry.json` at boot.
* Implement `riptide-validation::AdapterEngine` and per-schema adapters.
* Store validated entities and quarantined records in `riptide-persistence`.
* Dedup window enforcement.

### Deliverables

| Artifact                                    | Path                         | Description |
| ------------------------------------------- | ---------------------------- | ----------- |
| `/schemas/registry.json`                    | List schemas and dedup rules |             |
| `/schemas/events.v1.json`                   | Event entity JSON Schema     |             |
| `/schemas/jobs.v1.json`                     | Job entity JSON Schema       |             |
| `/crates/riptide-adapters/src/events_v1.rs` | Field mapper                 |             |
| `/crates/riptide-validation/src/lib.rs`     | Validation engine            |             |

### Acceptance Tests

* All fixtures under `riptide-test-utils/tests/fixtures/` validate.
* `LOW_CONFIDENCE` and `SCHEMA_INVALID` quarantine reasons appear as expected.
* < 5 % quarantine rate on suite.

---

## 6. Phase 3 — Diagnostics & Budgets

### Objectives

* Implement `/api/v1/diagnostics` (JSON health snapshot).
* Expose Prometheus metrics (`/metrics`).
* Track LLM and headless usage vs. profile budgets.
* Add retry/circuit logic in `riptide-reliability`.
* Add CLI `doctor` command to summarize diagnostics.

### Deliverables

| Artifact                                        | Path                                           | Description |
| ----------------------------------------------- | ---------------------------------------------- | ----------- |
| `/crates/riptide-monitoring/src/diagnostics.rs` | Collector for Redis/LLM/headless/cache status  |             |
| `/crates/riptide-intelligence/src/metrics.rs`   | LLM cost reporting                             |             |
| `/crates/riptide-cli/src/commands/doctor.rs`    | CLI doctor tool                                |             |
| `/dashboards/grafana/*.json`                    | Dashboards with headless% / LLM spend / errors |             |

### Acceptance Tests

* `/diagnostics` returns structured JSON with all subsystems.
* Prometheus endpoint scrapes without error.
* Alerts trigger when LLM spend > budget or headless% > 25 %.

---

## 7. Phase 4 — Discovery & City Automation

### Objectives

* Implement `/api/v1/discover` and `riptide-search` provider adapters (Serper, Bing).
* Add `riptide-discovery` crate (classification + enrollment).
* Store sources in `source_registry` table via `riptide-persistence`.
* CLI commands `discover` and `sources list/refresh`.
* Schedule daily run for Amsterdam (city example).

### Deliverables

* `/crates/riptide-search/src/providers/serper.rs` and `bing.rs` implementations.
* `/crates/riptide-discovery/src/lib.rs` classification engine.
* `/crates/riptide-persistence/migrations/source_registry.sql`.
* `/crates/riptide-cli/src/commands/discover.rs`.

### Acceptance Tests

* `POST /api/v1/discover` returns ≥ 10 unique sources for Amsterdam.
* Each enrolled source fetches successfully (`/crawl` with status 200).
* Daily run produces updated registry entries.

---

## 8. Phase 5 — Hardening & Migration

### Objectives

* Apply security policies (API keys, robots respect).
* Finalize test matrix and CI job graph.
* Implement release and rollback playbooks.
* Remove legacy shims when traffic < 5 %.

### Deliverables

| Artifact                     | Path                             | Description |
| ---------------------------- | -------------------------------- | ----------- |
| `/docs/security_policy.md`   | Auth, rate limits, robots policy |             |
| `/docs/testing_matrix.md`    | Test types and owners            |             |
| `/docs/release_checklist.md` | Release procedure                |             |
| `/docs/deprecation_plan.md`  | Shim retirement                  |             |

### Acceptance Tests

* Security tests (401/403) pass.
* All CI jobs green on Linux/macOS/Windows.
* Grafana shows SLO ≥ 99.5 %.
* Legacy route requests = 0 for 14 days → shims removed.

---

## 9. Cross-Phase Governance

1. **No feature regression:** parity tests run on every phase.
2. **Documentation:** each phase closes with updated `/docs/status.md`.
3. **Instrumentation:** metrics added in Phase 3 must remain through Phase 5.
4. **Security:** secrets never logged; checked each phase.

---

## 10. Transition to Doc 4

Doc 4 will expand each phase into **Workstreams & Ownership**:
who implements each crate change, the inputs/outputs, and the explicit acceptance conditions per workstream.

---

**End of Doc 3 — Phase Roadmap**

---

Shall I proceed with **Doc 4 – Workstreams & Ownership** (explicit per-crate tasks, owners, acceptance tests)?
Perfect.
Below is **Doc 4 – Workstreams & Ownership**, written for deterministic consumption by an AI developer with full repository access.
All actions are *imperative*, tied to exact files, crates, and measurable completion tests.

---

# 🧩 Doc 4 — Workstreams & Ownership

**Explicit crate-level work plans, acceptance tests, and dependencies**

---

## 1. Purpose

To decompose every roadmap phase into atomic workstreams with:

* **Target crate(s)**
* **Required inputs** (existing files, configs, schemas)
* **Explicit actions** to modify or create code/assets
* **Expected outputs**
* **Acceptance tests** (which must exist in the repo as CI tests or fixtures)

Each workstream maps to exactly one functional area; no implied inference is allowed.

---

## 2. Overview Table

| Workstream ID | Focus                             | Primary Crates                                                                          | Related Docs                                                        | Target Phase |
| ------------- | --------------------------------- | --------------------------------------------------------------------------------------- | ------------------------------------------------------------------- | ------------ |
| W1            | API v1 implementation & shims     | `riptide-api`, `riptide-api-types`, `riptide-facade`, `riptide-cli`                     | OpenAPI (`/openapi.yaml`), Mapping spec (`legacy-to-v1-mapping.md`) | 1            |
| W2            | Schema registry & adapters        | `riptide-schemas`, `riptide-adapters`, `riptide-validation`, `riptide-persistence`      | `registry.json`, `events.v1.json`, `jobs.v1.json`                   | 2            |
| W3            | Strategy runner & execution graph | `riptide-facade`, `riptide-extraction`                                                  | (Doc 6.7 upcoming)                                                  | 2            |
| W4            | Diagnostics, metrics, and budgets | `riptide-monitoring`, `riptide-reliability`, `riptide-intelligence`, `riptide-headless` | `/diagnostics` schema, `metrics.md`                                 | 3            |
| W5            | Discovery and source registry     | `riptide-search`, `riptide-discovery`, `riptide-persistence`, `riptide-cli`             | Search Addendum                                                     | 4            |
| W6            | Security and governance           | `riptide-api`, `riptide-security`, `riptide-config`                                     | `security_policy.md`                                                | 5            |
| W7            | Testing and CI structure          | `riptide-test-utils`, `.github/workflows`                                               | `testing_matrix.md`                                                 | 5            |
| W8            | Release and migration             | All                                                                                     | `release_checklist.md`, `deprecation_plan.md`                       | 5            |

---

## 3. Workstream Details

### W1 — API v1 Implementation & Shims

**Crates:** `riptide-api`, `riptide-api-types`, `riptide-facade`, `riptide-cli`

#### Actions

1. Implement all `/api/v1` routes exactly as defined in `/openapi.yaml`.
2. Generate and commit type-safe DTO structs in `riptide-api-types/src/`.
3. Create `riptide-api/src/shims.rs` to map all legacy endpoints → `/api/v1`.
4. Attach Deprecation header and `Link` to `/docs/migrate-v1`.
5. Ensure `riptide-cli` commands use v1 endpoints only.

#### Acceptance Tests

* `tests/shim_parity.rs` verifies legacy vs v1 entity equality.
* `/openapi.yaml` lints via `spectral` or `openapi-generator-cli validate`.
* CLI commands return 0 on test suite (`extract`, `crawl`, `spider`).
* 100 % of routes covered by `riptide-api/tests/openapi_coverage.rs`.

---

### W2 — Schema Registry & Adapters

**Crates:** `riptide-schemas`, `riptide-adapters`, `riptide-validation`, `riptide-persistence`

#### Actions

1. Place `registry.json` in `/schemas/`.
2. Implement `events.v1.json`, `jobs.v1.json` schemas.
3. Implement `riptide-adapters/src/events_v1.rs` and `jobs_v1.rs`.
4. Implement `riptide-validation::AdapterEngine`.
5. Extend `riptide-persistence` with tables: `entities`, `quarantine`, and `dedup_index`.
6. Write dedup hash functions (`xxh3_64`) per schema.

#### Acceptance Tests

* `tests/validation_fixtures.rs` runs all fixtures; asserts expected reason codes.
* `dedup_tests.rs` ensures identical fixtures yield same key.
* `quarantine_tests.rs` verifies quarantined payloads logged to persistence.

---

### W3 — Strategy Runner & Execution Graph

**Crates:** `riptide-facade`, `riptide-extraction`

#### Actions

1. Implement `Strategy` trait in `riptide-extraction/src/strategy.rs`:

   ```rust
   #[async_trait]
   pub trait Strategy {
       fn name(&self) -> &'static str;
       async fn apply(&self, ctx: &PageCtx, cfg: &StrategyCfg) -> StrategyOut;
   }
   ```
2. Create per-strategy modules (`ics.rs`, `json.rs`, `rulepack.rs`, `wasm.rs`, `pdf.rs`, `tables.rs`, `llm.rs`, `headless.rs`, `stealth.rs`).
3. In `riptide-facade/src/runner.rs`, implement ordered execution:

   ```
   [ics,json] → rulepack → wasm → llm (conditional)
   ```

   Stop early when confidence ≥ `options.quality.min_confidence`.
4. Record provenance (strategy chain, timings, snapshot_key, confidence).
5. Expose `run_pipeline(inputs, options)` returning `Stream<ResultItem>`.

#### Acceptance Tests

* Fixture per strategy succeeds when enabled, skipped when denied.
* Provenance chain matches expected order in golden responses.
* Confidence thresholds enforced (no LLM invocation if above threshold).

---

### W4 — Diagnostics, Metrics, Budgets

**Crates:** `riptide-monitoring`, `riptide-reliability`, `riptide-intelligence`, `riptide-headless`

#### Actions

1. Implement `/api/v1/diagnostics` in `riptide-api/src/routes/v1/diagnostics.rs`.
2. Create `riptide-monitoring/src/collectors.rs` aggregating subsystem health (Redis, headless, LLM, search, cache).
3. Expose Prometheus metrics via `riptide-monitoring/src/metrics.rs`.
4. Add cost/budget tracking to `riptide-intelligence`.
5. Add headless pool gauges and circuit-breaker stats.
6. Create Grafana dashboards in `/dashboards/grafana/`.

#### Acceptance Tests

* `/diagnostics` JSON validates against `diagnostics.schema.json`.
* Metrics endpoint scrape OK via Prometheus test container.
* Alert rule fires when LLM spend > profile cap.
* Grafana dashboard loads successfully with no broken panels.

---

### W5 — Discovery & Source Registry

**Crates:** `riptide-search`, `riptide-discovery`, `riptide-persistence`, `riptide-cli`

#### Actions

1. Implement provider adapters (`serper.rs`, `bing.rs`, optional `brave.rs`).
2. Create classification and enrollment engine (`riptide-discovery/src/lib.rs`).
3. Define `source_registry` schema in `riptide-persistence/migrations/`.
4. Implement CLI commands:

   ```
   riptide discover --schema events --scope "Amsterdam"
   riptide sources list --schema events
   riptide sources refresh
   ```
5. Cache search results with TTL 24h.

#### Acceptance Tests

* `/discover` returns ≥ 10 unique sources for test city.
* Classification model accuracy ≥ 0.8 on fixtures.
* Registry refresh job updates timestamps.

---

### W6 — Security & Governance

**Crates:** `riptide-api`, `riptide-security`, `riptide-config`

#### Actions

1. Add auth middleware for admin routes using `security.admin_api_keys`.
2. Add rate limit per IP/key (Redis-based token bucket).
3. Implement robots respect default in `riptide-config` + override in options.
4. Redact secrets in logs and diagnostics output.
5. Add audit logging (auth, rate-limit, config errors) to `riptide-events`.

#### Acceptance Tests

* `tests/security_admin.rs`: unauthenticated → 401, wrong key → 403.
* `tests/rate_limit.rs`: exceeding N requests → 429.
* `tests/robots_policy.rs`: “ignore” logged when explicitly set.
* No secrets in captured logs.

---

### W7 — Testing & CI Structure

**Crates:** `riptide-test-utils`, `.github/workflows`

#### Actions

1. Create test tag taxonomy: `unit`, `integration`, `golden`, `perf`, `e2e`.
2. Add `.github/workflows/ci.yml` with job matrix:

   * `test_unit` (fast)
   * `test_integration` (with Redis/headless)
   * `test_golden` (fixtures)
   * `lint` (rustfmt, clippy, deny)
3. Add nightly workflow `nightly.yml` to run perf and discovery jobs.
4. Auto-upload golden diff artifacts on failure.

#### Acceptance Tests

* All workflows green.
* `cargo test --all --features full` passes.
* `nightly.yml` produces and stores results under `/artifacts/`.

---

### W8 — Release & Migration

**Crates:** All

#### Actions

1. Create `/docs/release_checklist.md` with staging, canary, rollback, post-mortem steps.
2. Publish `openapi.yaml` and CLI binaries as release assets.
3. Remove legacy shims when legacy traffic < 5 % for 14 days.
4. Update `/docs/deprecation_plan.md` and notify via event `shim_removed`.

#### Acceptance Tests

* `release_checklist.md` and `rollback_playbook.md` committed.
* Canary deploy runs for 48h without error spike.
* Shim routes removed, confirmed via 404 on old paths.
* `/diagnostics` reports version ≥ 1.0.

---

## 4. Dependencies Between Workstreams

| Depends On | Blocks | Explanation                                                  |
| ---------- | ------ | ------------------------------------------------------------ |
| W1 → W2    | ✅      | API surface needed before schema integration                 |
| W2 → W3    | ✅      | Validation engine must exist before runner outputs entities  |
| W3 → W4    | ✅      | Provenance and metrics depend on runner                      |
| W4 → W5    | ✅      | Discovery writes entities validated by W2 and observed by W4 |
| W5 → W6    | ↔️     | Security gating may be needed for discovery endpoints        |
| W6 → W7    | ✅      | CI tests include security                                    |
| W7 → W8    | ✅      | Must have full CI before release                             |

---

## 5. Ownership Matrix

| Domain                          | Owner Role       | Responsible Crates                                                                 |
| ------------------------------- | ---------------- | ---------------------------------------------------------------------------------- |
| **API & CLI**                   | Backend / SDK    | `riptide-api`, `riptide-api-types`, `riptide-cli`                                  |
| **Pipeline & Strategies**       | Platform         | `riptide-facade`, `riptide-extraction`, `riptide-utils`                            |
| **Schema & Validation**         | Data             | `riptide-schemas`, `riptide-adapters`, `riptide-validation`, `riptide-persistence` |
| **Observability & Reliability** | SRE              | `riptide-monitoring`, `riptide-reliability`, `riptide-performance`                 |
| **Discovery**                   | Search           | `riptide-search`, `riptide-discovery`                                              |
| **AI / LLM**                    | AI Team          | `riptide-intelligence`                                                             |
| **Security & Governance**       | Security / Ops   | `riptide-api`, `riptide-security`, `riptide-config`                                |
| **CI & Tests**                  | QA               | `riptide-test-utils`, workflows                                                    |
| **Release / Docs**              | PM / Release Eng | `/docs/`, OpenAPI, CLI binaries                                                    |

---

## 6. Workstream Exit Checklist

Every workstream must:

1. Update `CHANGELOG.md` in the crate root with semantic version bump.
2. Add `#[cfg(test)]` coverage ≥ 80 %.
3. Emit metrics per major function (latency, errors).
4. Update `/docs/status.md` with progress marker.

CI must block merge if any acceptance test fails or coverage < threshold.

---

## 7. Next Document

**Doc 5 – Testing, CI & Quality Gates**
will define:

* Tag taxonomy (unit/integration/perf/e2e)
* Golden snapshot mechanics
* CI pipeline graph and gating rules
* Quality metrics and flake management

---

**End of Doc 4 – Workstreams & Ownership**

---

Would you like me to proceed with **Doc 5 – Testing, CI & Quality Gates** next?
Great — here is **Doc 5 – Testing, CI & Quality Gates**, written for an AI developer/agent with full repo access. It contains only directives and file paths (no embedded code/assets).

---

# ✅ Doc 5 — Testing, CI & Quality Gates

**Authoritative plan for tests, fixtures, CI workflows, and merge gates**

---

## 1) Purpose

Define the **complete quality system** for Riptide v1:

* Test types, locations, and naming.
* Golden/parity mechanics (including NDJSON normalization rules).
* CI workflow graph, caching, and gating.
* Performance/SLO checks and flake remediation.
* Required artifacts and acceptance signals per job.

This doc references assets defined in Docs 1–4 (e.g., `openapi.yaml`, schemas, options, registry) and the crate layout from Doc 2. Do not invent new paths or behaviors beyond those documents.

---

## 2) Test Taxonomy & Directory Layout

### 2.1 Tags (must be mutually exclusive unless stated)

* `unit` – fast, isolated crate logic (no network, no Redis/headless).
* `integration` – single subsystem with real dependencies (e.g., Redis, CDP, LLM mocked).
* `golden` – deterministic I/O snapshots (HTML/PDF fixtures → normalized JSON).
* `e2e` – full API workflow across multiple crates (HTTP server + facade + persistence).
* `perf` – latency/throughput checks, smoke-level stability (nightly only).

### 2.2 File layout (authoritative)

```
/crates/*/tests/                         # unit + crate-level integration
/crates/riptide-api/tests/               # API unit/integration; OpenAPI coverage
/crates/riptide-facade/tests/            # runner sequencing + provenance
/crates/riptide-validation/tests/        # schema & adapter engine fixtures
/crates/riptide-test-utils/              # helpers + golden harness
  /tests/goldens/                         # request/response pairs
  /fixtures/                              # HTML, JSON, ICS, PDF, tables data
  /normalizers/                           # scrub rules for responses/streams
/tests/e2e/                               # end-to-end API tests
/tests/perf/                              # perf smoke (nightly)
```

---

## 3) Golden & Parity Mechanics

### 3.1 Golden sources (must exist)

* Request/response pairs declared in `options-examples-and-goldens.md`.
* Place request JSON under `/crates/riptide-test-utils/tests/goldens/*.request.json`.
* Place expected response JSON under `/crates/riptide-test-utils/tests/goldens/*.response.json`.

### 3.2 Normalization rules (apply before snapshot compare)

Implement scrubbers in `riptide-test-utils/normalizers/` to:

* Strip or replace:

  * `provenance.snapshot_key`
  * `provenance.timings_ms.*`
  * Any generated IDs and timestamps (`incident_id`, `started_at`, `finished_at`)
* Round floats:

  * `provenance.confidence` → 2 decimal places
* Sort arrays by deterministic keys:

  * For entity lists: sort by `url` then `title` if present
* NDJSON:

  * Normalize to array; drop `progress` events; retain `summary` and `error`
* CSV/ICS:

  * Trim trailing spaces; normalize newline to `\n`

Document these in `/crates/riptide-test-utils/README.md`.

### 3.3 Parity rules

* **Legacy vs v1 responses:** entity set equality ignoring order/transient fields.
* **Array vs Stream parity:** `/crawl` array output must equal final set from `/crawl/stream`.
* **Budget behavior:** when budgets exceeded, expect `Error.code` from OpenAPI (`RATE_LIMITED` or `UPSTREAM_UNAVAILABLE`), not transport failure.

---

## 4) Coverage Targets & Quality Bars

* **Line coverage:** ≥ 80% for all crates (`unit+integration`), reported per-crate.
* **Mutation test (optional):** enable for `riptide-adapters` and `riptide-validation` only; threshold ≥ 60%.
* **OpenAPI coverage:** 100% of documented paths have at least one passing test in `riptide-api/tests/openapi_coverage.rs`.
* **Schema validation:** 100% of emitted entities (in tests) validate or are quarantined with an expected reason code.
* **CI wall-clock:** `ci.yml` runtime ≤ 12 min at p95 on default concurrency matrix.

---

## 5) CI Workflow Graph (authoritative)

Create **two** primary workflows and **one** nightly:

### 5.1 `.github/workflows/ci.yml` (per PR / push)

Jobs (run in this order; later jobs require prior success):

1. `lint`

   * rustfmt, clippy, OpenAPI lint (`openapi.yaml`), YAML lint (`server.yaml`).
2. `deny`

   * cargo-deny (licenses + **layering rules** from Doc 2).
3. `unit`

   * `cargo test` with tag `unit` across workspace.
4. `integration`

   * Spin Redis container; mock CDP; run integration tagged tests.
5. `golden`

   * Run golden tests using fixtures; apply normalizers; compare snapshots.
6. `e2e`

   * Boot `riptide-api` (binding ephemeral port), run full API calls using DTOs.
7. `openapi_coverage`

   * Assert every path/verb in `openapi.yaml` hit at least once.
8. `coverage_report`

   * Combine coverage, fail if < 80% per crate.
9. `build_release`

   * Build `riptide-api`, `riptide-cli`, `riptide-workers` for Linux/macOS/Windows (without publishing).

All jobs must upload failing artifacts to `/artifacts/ci/<job>/<run-id>/`.

### 5.2 `.github/workflows/nightly.yml` (scheduled)

Jobs:

* `perf_smoke` (see §7)
* `discovery_refresh` dry-run (no external posting)
* `flake_scan` (see §8)
* `dashboard_snapshot` (export Grafana JSON panels to `dashboards/snapshots/`)

### 5.3 Caching & matrix

* Use Rust cache keyed on `Cargo.lock` + target triple.
* Use `actions/cache` for OpenAPI toolchain binaries.
* Matrix for OS: `ubuntu-latest`, `macos-latest`, `windows-latest` on `build_release` only.

---

## 6) Gating & Branch Protection

* Require the following jobs to pass before merge:
  `lint`, `deny`, `unit`, `integration`, `golden`, `e2e`, `openapi_coverage`, `coverage_report`.
* Required review from codeowners:

  * Changes under `/openapi.yaml`, `/schemas/**`, `/server.yaml`, `/crates/riptide-facade/**` require **Data** + **Platform** reviewers.
* Block merge if:

  * New public endpoints appear not in `openapi.yaml`.
  * Layering violations detected by `cargo-deny`.
  * Coverage < 80% in any crate.

---

## 7) Performance & SLO Tests (nightly)

Create targeted perf checks under `/tests/perf/`:

* **API latency:**

  * `/api/v1/extract` on HTML fixture: p95 ≤ 500 ms (no headless/LLM).
  * `/api/v1/extract` on JS-heavy with headless: p95 ≤ 3500 ms.
* **Throughput:**

  * `/crawl` 100 URLs batch: completes ≤ 2 min on CI instance.
* **Budget adherence:**

  * Headless usage (on perf suite) ≤ 25%; LLM spend ≤ profile caps (full).

Artifacts stored at `/artifacts/perf/YYYY-MM-DD/` with JSON metrics.

---

## 8) Flake Detection & Remediation

* Introduce a simple flake detector in nightly:

  * Re-run any failed test up to 3 times (backoff).
  * If subsequent passes occur, mark test as **flaky** and open/append to `/docs/flake_log.md` with stacktrace and suspected subsystem.
* Quarantine rules:

  * Tests tagged `@flaky` do not gate merges but must be addressed within 7 days.
  * After fix, remove `@flaky` tag and validate stability (3 consecutive nights).

---

## 9) Secrets & External Dependencies in CI

* No real LLM or search keys in PR CI.
* Mock LLM/search by default; real providers run only in `nightly.yml` **with protected secrets**.
* Headless/Chromium: use Docker image with pinned version; expose CDP on `localhost` ephemeral port.

---

## 10) Required Test Suites (content checklist)

* **API v1 routing**: happy paths + error model coverage for every route.
* **Legacy→v1 parity**: per family (`crawl`, `extract-advanced`, `llm`, `pdf`, `tables`, `deepsearch`).
* **Strategy toggles**: ON/OFF permutations for `ics`, `json`, `rulepack`, `wasm`, `pdf`, `tables`, `headless`, `stealth`, `llm`.
* **Schema validation**: required field missing/type mismatch/format error/low confidence.
* **Dedup**: key collision on duplicate fixtures; window enforcement.
* **Provenance**: chain order + confidence threshold stop condition.
* **Diagnostics**: JSON contract (presence and types of all fields).
* **Security**: admin auth (401/403), rate-limit (429), robots override logging.
* **CLI parity**: `extract|crawl|spider|discover|doctor` exit codes and stdout/stderr discipline.

---

## 11) Metrics & Alerting Assertions (CI Static Checks)

Create `/docs/metrics.md` and assert presence of these metric **names** during test scrape:

* `riptide_requests_total{route=...}`
* `riptide_request_duration_ms_bucket{route=...}`
* `riptide_headless_sessions_active`
* `riptide_llm_cost_eur_total`
* `riptide_entities_emitted_total{schema=...}`
* `riptide_entities_quarantined_total{schema=...,code=...}`
* `riptide_cache_hit_ratio`
* `riptide_errors_total{code=...}`

Nightly alert simulations (no paging) confirm thresholds:

* LLM daily cost > `integrations.llm.daily_budget_eur` → `ALERT: LLM_BUDGET_EXCEEDED`
* Headless percent > 25% in `full` suite → `ALERT: HEADLESS_EXCESS`
* Error rate > 2% over 15 min → `ALERT: ERROR_SPIKE`

---

## 12) Outputs & Artifacts

Every CI run must produce:

* `coverage/summary.json` (per-crate coverage)
* `openapi/coverage.json` (routes hit vs defined)
* `goldens/diffs/*.txt` on mismatch
* `perf/results.json` (nightly)
* `diagnostics/snapshots/*.json` (nightly)
  Artifacts retained 14 days.

---

## 13) Merge Gate Summary (copy into repo `docs/quality_gates.md`)

A PR **shall not** merge unless:

1. All required CI jobs (Section 6) pass.
2. Coverage ≥ 80% in each crate.
3. No layering violations (Doc 2 rules).
4. OpenAPI coverage = 100% of paths.
5. No new warnings in `/diagnostics` snapshot compared to main.
6. If touching `schemas/**` or `openapi.yaml`, at least one **Data** and one **Platform** approver have approved.

---

## 14) Handoff

* Place this document at `/docs/testing_ci_quality.md`.
* Ensure workflow files exist and reference the paths exactly as written here.
* Update `/docs/status.md` with CI job names and current pass rates.

---

**End of Doc 5 – Testing, CI & Quality Gates**

I’m ready to proceed with **Doc 6 – Security, Observability & Governance** (auth, robots, rate limits, diagnostics JSON contract, metrics catalog, dashboards, and incident runbook).
Excellent.
Here is **Doc 6 – Security, Observability & Governance**, written in the same explicit, machine-actionable format.

---

# 🛡️ Doc 6 — Security, Observability & Governance

**Authentication, authorization, rate-limit, diagnostics, metrics, dashboards, and incident policies**

---

## 1 Purpose

Define all runtime controls that keep Riptide safe, observable, and auditable.
The instructions here apply to `riptide-api`, `riptide-config`, `riptide-monitoring`, `riptide-reliability`, `riptide-events`, and supporting crates.
Every field or metric referenced must exist in the repo and follow the contracts from `openapi.yaml`, `server.yaml`, and previous documents.

---

## 2 Security Subsystem

### 2.1 Authentication / Authorization

| Route Family                       | Auth Mode           | Config Source (`server.yaml`) | Enforcement Location                              |
| ---------------------------------- | ------------------- | ----------------------------- | ------------------------------------------------- |
| `/api/v1/**` (data plane)          | Optional key        | `security.data_api_keys`      | `riptide-api/src/middleware/auth.rs`              |
| `/api/v1/admin/**` (control plane) | Required key        | `security.admin_api_keys`     | Same middleware, stricter scope                   |
| `/metrics`                         | No auth (read-only) | —                             | Exposed via Prometheus exporter                   |
| `/diagnostics`                     | Optional key        | same as data plane            | Protected by middleware flag `require_auth=false` |

**Action:**
Implement middleware that:

1. Checks `x-api-key` header or `Authorization: Bearer` token.
2. Validates against the active config lists.
3. Attaches `auth.role` (`admin` / `data`) to request extensions.
4. Logs audit entry (`riptide-events::AuthEvent`).

### 2.2 Rate Limiting

* **Type:** Redis-backed token bucket.
* **Config keys:**

  ```
  security.rate_limits.per_ip_per_minute
  security.rate_limits.per_key_per_minute
  ```
* **Storage:** `riptide-cache` namespace `ratelimit:{ip|key}`
* **Headers returned:** `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`
* **Failure code:** `429 RATE_LIMITED` per OpenAPI.

### 2.3 Request Size / Body Limits

* Controlled by `security.max_request_body_mb`.
* Rejected with `413 Payload Too Large`.
* Implemented in `riptide-api/src/main.rs` via web-framework limit config.

### 2.4 Robots Policy

* Default =`respect`.
* Config hierarchy: request option > server.yaml > registry.
* Violations (log “robots: ignore”) must emit warning event `RobotsOverride`.

### 2.5 Secret Handling

* Secrets (`api_key`, `redis_url`, etc.) must never appear in plain logs or diagnostics.
* Redact value with `"***"` before serialization.
* Verified by test `tests/security_secrets.rs`.

---

## 3 Observability Subsystem

### 3.1 Diagnostics Endpoint (`/api/v1/diagnostics`)

**Source Code:** `riptide-monitoring/src/diagnostics.rs`
**Schema Validation:** `/schemas/diagnostics.schema.json`

**Required fields:**

```json
{
  "version": "1.0",
  "profile": "full|lite",
  "redis": "healthy|degraded|down",
  "headless": "healthy|degraded|down",
  "llm": "enabled|disabled|degraded",
  "search_providers": [{"name":"serper","status":"healthy"}],
  "pools": {"headless_active":0,"headless_capacity":8},
  "budgets": {"llm_eur_today":0,"llm_eur_limit":5,"headless_pct_window":0},
  "cache": {"hit_rate":0.0,"items":0},
  "warnings": []
}
```

**Implementation Rules**

1. Return HTTP 200 on partial degradation; 500 only on self-check failure.
2. Set header `X-Riptide-Config-Profile`.
3. Emit event `DiagnosticsSnapshot` to `riptide-events`.
4. Must serialize within 500 ms.

### 3.2 Metrics Endpoint (`/metrics`)

**Format:** Prometheus text.
**Exporter:** `riptide-monitoring/src/metrics.rs`

**Metric name catalog** (`docs/metrics.md` to exist)

| Name                                 | Type      | Labels              | Description              |
| ------------------------------------ | --------- | ------------------- | ------------------------ |
| `riptide_requests_total`             | counter   | route,schema,status | total requests           |
| `riptide_request_duration_ms_bucket` | histogram | route               | latency distribution     |
| `riptide_headless_sessions_active`   | gauge     | pool                | active headless sessions |
| `riptide_llm_cost_eur_total`         | counter   | profile             | cumulative LLM spend     |
| `riptide_entities_emitted_total`     | counter   | schema              | valid entities emitted   |
| `riptide_entities_quarantined_total` | counter   | schema,code         | invalid entities         |
| `riptide_cache_hit_ratio`            | gauge     | –                   | cache efficiency         |
| `riptide_errors_total`               | counter   | code                | error counts             |
| `riptide_jobs_active`                | gauge     | –                   | active spider jobs       |

---

## 4 Dashboards and Alerts

### 4.1 Grafana Dashboards (`/dashboards/grafana/`)

Required dashboards:

1. **Overview.json** – success rate, latency, error codes.
2. **Headless_LLM.json** – headless% and LLM spend vs budgets.
3. **Discovery.json** – discovered sources per schema, fail rate.
4. **Cache_Persistence.json** – hit/miss ratios, store latency.
5. **Jobs.json** – active jobs, failures, retry counts.

Each panel must have `uid`, `title`, and `targets` mapping to metrics above.

### 4.2 Alerts (`/dashboards/alerts.yml`)

| Alert Name          | Condition                                                                   | Severity | Action                                                   |
| ------------------- | --------------------------------------------------------------------------- | -------- | -------------------------------------------------------- |
| LLM_BUDGET_EXCEEDED | `riptide_llm_cost_eur_total > llm_eur_limit`                                | high     | Throttle LLM providers + notify Slack channel `#ai-cost` |
| HEADLESS_EXCESS     | `riptide_headless_sessions_active / capacity > 0.25`                        | medium   | Alert SRE to increase timeout or pool size               |
| ERROR_SPIKE         | `riptide_errors_total{code!="RATE_LIMITED"} > 2% of riptide_requests_total` | critical | Page on-call                                             |
| CACHE_DROP          | `riptide_cache_hit_ratio < 0.7 for 5 min`                                   | medium   | Flush Redis metrics cache                                |
| DISCOVERY_FAIL_RATE | > 0.2 failures on discoveries                                               | low      | Notify search team                                       |

---

## 5 Governance and Audit

### 5.1 Event Audit Log

* Emit all administrative and security events to `riptide-events`.
* Schema `audit_log.schema.json` must define fields:

```json
{"time":"RFC3339","actor":"ip|api_key","action":"auth_success|auth_fail|config_change|rate_limit","details":{}}
```

* Persistence: `riptide-persistence/audit_log` table with 90-day retention.
* CLI: `riptide audit list --since 7d --action auth_fail`.

### 5.2 Compliance Retention

| Data Type          | Retention         | Deletion Mechanism               |
| ------------------ | ----------------- | -------------------------------- |
| Entity snapshots   | 90 days (default) | `riptide-persistence` TTL task   |
| Audit logs         | 90 days           | cron delete old rows             |
| Quarantine records | 180 days          | manual purge via CLI             |
| Source registry    | indefinite        | refresh overwrites stale entries |

---

## 6 Incident Management

### 6.1 Incident Detection

Triggered by alerts in §4.2 or manual report.
Incident record schema (`incident_runbook.md`):

| Field              | Type                                        | Description |
| ------------------ | ------------------------------------------- | ----------- |
| `incident_id`      | string                                      | UUIDv4      |
| `detected_at`      | datetime UTC                                |             |
| `trigger`          | string                                      | alert name  |
| `severity`         | string (enum: info, medium, high, critical) |             |
| `impact`           | string                                      | summary     |
| `mitigation_steps` | array of strings                            |             |
| `resolved_at`      | datetime                                    |             |
| `follow_up`        | array of strings                            |             |

### 6.2 Roles

| Role              | Responsibility                    |
| ----------------- | --------------------------------- |
| On-call SRE       | responds to alerts within 15 min  |
| Security Engineer | auth/rate-limit incidents         |
| AI Lead           | budget / LLM throttling decisions |
| PM / Comms        | customer updates & post-mortem    |

### 6.3 Timeline Expectations

* MTTA ≤ 15 min, MTTR ≤ 2 h for P1.
* All incidents recorded in `/docs/incidents/YYYY-MM-DD/<id>.md`.

---

## 7 Audit and Compliance Checks (CI)

Add a weekly workflow `.github/workflows/audit.yml` that:

1. Scans diagnostics output for redacted fields.
2. Validates retention jobs (`entities`, `audit_log`, `quarantine`).
3. Runs auth/rate-limit tests with fake keys.
4. Generates `/artifacts/audit/report.json` summarizing status.

Failure on any check → notify `#sre-alerts`.

---

## 8 Acceptance Criteria (SRE / Security)

1. Admin routes require valid key (401/403 tests pass).
2. Rate-limit headers present and accurate within ±5 %.
3. Secrets redacted in all logs and diagnostics.
4. Diagnostics responds in < 500 ms with valid JSON.
5. Prometheus scrape success rate ≥ 99 %.
6. Grafana dashboards load without missing panels.
7. Alerts trigger and auto-resolve in staging simulation.
8. Weekly audit workflow produces no errors.

---

## 9 Deliverables Checklist

| Artifact                                | Path                  | Owner         |
| --------------------------------------- | --------------------- | ------------- |
| `riptide-api/src/middleware/auth.rs`    | Auth middleware       | Security Team |
| `riptide-monitoring/src/diagnostics.rs` | Diagnostics collector | SRE           |
| `riptide-monitoring/src/metrics.rs`     | Metrics exporter      | SRE           |
| `docs/metrics.md`                       | Metric catalog        | SRE           |
| `dashboards/grafana/*.json`             | Dashboards            | SRE           |
| `dashboards/alerts.yml`                 | Alert rules           | SRE           |
| `docs/audit_log.schema.json`            | Audit events          | Security      |
| `docs/incident_runbook.md`              | Incident procedure    | PM/SRE        |
| `.github/workflows/audit.yml`           | Weekly audit job      | Security      |

---

## 10 Next Document

**Doc 7 – Release, Migration & Post-Cutover**
will define:

* Canary + rollback process,
* Shim retirement calendar,
* Release artifacts (OpenAPI, CLI, binaries),
* Post-cutover refactors and long-term governance.

---

**End of Doc 6 – Security, Observability & Governance**

---

Shall I continue with **Doc 7 – Release, Migration & Post-Cutover** (the final roadmap document)?
Perfect — here is the **final roadmap document**, designed for an AI developer or agent with full repository access and deterministic control over release automation.

---

# 🚀 Doc 7 — Release, Migration & Post-Cutover

**Deployment, deprecation, rollback, and long-term governance plan**

---

## 1 Purpose

Define the **end-to-end process** for promoting Riptide v1 from staging to production, migrating all traffic, retiring legacy shims, and maintaining the system thereafter.
Every step here is mandatory and must be codified in scripts, CI workflows, or configuration files — no manual assumptions.

---

## 2 Release Objectives

| Objective                       | Description                                         | Validation                      |
| ------------------------------- | --------------------------------------------------- | ------------------------------- |
| Stable `/api/v1` endpoints only | All traffic through v1; shims disabled              | 0 legacy requests for 14 days   |
| Reliable builds                 | Deterministic, cross-platform reproducible binaries | Same SHA256 checksum on rebuild |
| Rollback-safe deploy            | Revert to previous stable within 10 min             | Canary rollback job verified    |
| Observability continuity        | Metrics/dashboards identical across versions        | Grafana diff = 0 panel errors   |
| Controlled migration            | Progressive traffic shift with audit trail          | Traffic logs archived daily     |

---

## 3 Build & Packaging

### 3.1 Build Targets

* `riptide-api` (HTTP service)
* `riptide-cli` (binary SDK)
* `riptide-workers` (optional background)

All builds produced by CI job `build_release`:

```
target/release/{riptide-api,riptide-cli,riptide-workers}
```

### 3.2 Reproducibility Requirements

* Fixed toolchain version (`rust-toolchain.toml`).
* `Cargo.lock` committed.
* Strip symbols; embed `--version` flag returning Git commit hash.
* Run `sha256sum` on all release binaries; compare with previous run.
* Store in `/artifacts/releases/<version>/checksums.txt`.

### 3.3 Platform Matrix

| Platform           | Artifact                                | Notes               |
| ------------------ | --------------------------------------- | ------------------- |
| Linux (x86_64-gnu) | `riptide-api`                           | primary deploy      |
| macOS (arm64)      | `riptide-cli`                           | developer SDK       |
| Windows (x64)      | `riptide-cli.exe`                       | SDK                 |
| Container          | `ghcr.io/riptide/riptide-api:<version>` | base image for prod |

---

## 4 Staging → Canary → Production Promotion

### 4.1 Environments

| Environment  | Purpose                 | Branch        | Config File           |
| ------------ | ----------------------- | ------------- | --------------------- |
| `staging`    | full system validation  | `develop`     | `server.staging.yaml` |
| `canary`     | limited traffic (≤ 5 %) | `release/x.y` | `server.canary.yaml`  |
| `production` | 100 % traffic           | `main`        | `server.yaml`         |

### 4.2 Promotion Pipeline

1. CI builds versioned artifacts (`v1.x.y-rcN`).
2. Deploy to **staging**; run full e2e and perf suites.
3. Promote to **canary** cluster (single region, 5 % traffic).
4. Monitor:

   * Error rate < 1 %.
   * LLM spend within budget.
   * Headless pool saturation < 80 %.
5. If stable for 48 h, promote to **production**:

   * Deploy identical container digest.
   * Tag release `v1.x.y` in Git.
6. Archive metrics and diagnostics snapshot.

---

## 5 Migration Plan (Legacy → v1)

### 5.1 Timeline

| Day | Action                                                    |
| --- | --------------------------------------------------------- |
| D0  | v1 deployed, shims active (`Deprecation` header emitted). |
| D30 | Warning logs for any legacy hits (count only).            |
| D60 | Legacy routes return HTTP 410 with migration link.        |
| D90 | Remove shim handlers and middleware.                      |
| D91 | Delete legacy mapping tests; archive historical results.  |

### 5.2 Telemetry Tracking

* Metric `riptide_legacy_requests_total{route}` added during D0–D90.
* Removal threshold: < 5 % of total traffic for 14 consecutive days.
* Dashboard: *Legacy Traffic Overview* (`dashboards/grafana/Legacy.json`).

### 5.3 Communication

* Notify internal users via `#riptide-announce`.
* Update `/docs/migrate-v1.md` with old→new route table.
* Emit event `ShimRemoved` when handlers deleted.

---

## 6 Rollback Plan

### 6.1 Trigger Conditions

* Error rate > 2 % sustained 10 min.
* Unrecoverable LLM/search outage.
* Schema validation regression > 5 %.
* CI post-deploy checks fail.

### 6.2 Procedure

1. CI job `rollback` pulls previous image tag `<version-1>`.
2. Deploy to production cluster.
3. Restore previous `server.yaml` (backup kept in `/backups/configs`).
4. Confirm `/healthz` green and metrics stable.
5. Mark rollback in `/docs/incidents/<id>.md`.

### 6.3 Rollback Verification

* Canary rollback simulated weekly (auto job).
* Recovery time measured; must be ≤ 10 min (95 %ile).

---

## 7 Post-Cutover Refactors

### 7.1 Crate Reorganization

* Merge browser stack:

  ```
  riptide-browser-abstraction, riptide-headless → riptide-browser
  ```
* Create `riptide-utils` (already defined) as common dependency.
* Split `riptide-extraction` into:

  * `riptide-extraction-core` (HTML/JSON/Rulepack/WASM)
  * `riptide-extraction-pdf` (PDF/Tables)
* Update `Cargo.toml` workspace members accordingly.

### 7.2 Deprecated Asset Cleanup

Delete:

* `/crates/riptide-api/src/shims.rs`
* `/tests/legacy_*`
* `legacy-to-v1-mapping.md` (archive under `/docs/archive/`)
* Any env vars not prefixed `RIPTIDE_`.

### 7.3 Documentation Refresh

* Update `/docs/architecture.md` to remove legacy references.
* Replace examples in README.md with `/api/v1` calls only.

---

## 8 Long-Term Governance

### 8.1 Versioning Policy

* API version frozen at `/api/v1` for 12 months.
* New schema versions introduced as `events.v2.json` etc., never breaking existing ones.
* Backward compatibility guaranteed at DTO layer.

### 8.2 Maintenance Cadence

| Task                             | Frequency | Owner    | Artifact                         |
| -------------------------------- | --------- | -------- | -------------------------------- |
| Schema registry review           | Quarterly | Data     | `registry.json`                  |
| Budget and limits audit          | Monthly   | SRE/AI   | `server.yaml`                    |
| Dependency audit (`cargo-audit`) | Weekly    | Security | `.github/workflows/audit.yml`    |
| Dashboard review                 | Monthly   | SRE      | `dashboards/grafana/*.json`      |
| Incident simulation              | Quarterly | Ops      | `/docs/incidents/simulations.md` |

### 8.3 Documentation Upkeep

All docs must include version tag header:

```yaml
# doc-version: 1.0
# updated: YYYY-MM-DD
```

CI job `docs_version_check` ensures timestamp freshness (< 90 days).

---

## 9 Success & End-State Validation

**System-level SLO targets after full cutover:**

| Metric            | Target                                         | Measurement                                     |
| ----------------- | ---------------------------------------------- | ----------------------------------------------- |
| Availability      | ≥ 99.5 %                                       | `/healthz` uptime (Prometheus)                  |
| Error Rate        | < 1 %                                          | `riptide_errors_total / riptide_requests_total` |
| Latency           | p95 < 500 ms (`extract`), < 3500 ms (headless) | Perf suite                                      |
| Schema Validation | ≥ 95 % valid, ≤ 5 % quarantine                 | Validation logs                                 |
| Budget Compliance | ≤ LLM 10 EUR/day, ≤ 25 % headless              | `/diagnostics`                                  |
| Coverage          | 100 % OpenAPI, ≥ 80 % code                     | CI report                                       |
| Legacy Traffic    | 0 %                                            | Metrics dashboard                               |

---

## 10 Deliverables Summary

| Category              | Artifact / File                                    | Owner       |
| --------------------- | -------------------------------------------------- | ----------- |
| Build                 | `.github/workflows/ci.yml` (build_release)         | Platform    |
| Container             | `Dockerfile` for riptide-api                       | Platform    |
| Promotion             | `.github/workflows/promote.yml`                    | Release Eng |
| Canary                | `server.canary.yaml`                               | SRE         |
| Rollback              | `.github/workflows/rollback.yml`                   | SRE         |
| Dashboards            | `/dashboards/grafana/*.json`                       | SRE         |
| Migration Docs        | `/docs/migrate-v1.md`, `/docs/deprecation_plan.md` | PM          |
| Post-Cutover Cleanups | `crate_reorg_plan.md`                              | Platform    |
| Governance            | `/docs/incidents/*.md`, `/docs/metrics.md`         | Ops         |

---

## 11 Acceptance Criteria

A release is **approved** when:

1. Canary stable ≥ 48 h, no alerts triggered.
2. All dashboards and metrics unchanged between versions.
3. Rollback simulation passes.
4. Legacy traffic < 5 % for 14 days.
5. OpenAPI & CLI binaries published under release tag.
6. Documentation in `/docs/status.md` marked `complete`.

---

## 12 End-of-Project Declaration

Once acceptance criteria are met:

* Tag repository `v1.0.0`.
* Archive Docs 1–7 under `/docs/roadmap/v1/`.
* Freeze schema versions.
* Transition project from “build” to “operate” state.
* Begin quarterly maintenance under governance cadence (§8.2).

---

**End of Doc 7 – Release, Migration & Post-Cutover**
*(This completes the comprehensive Riptide v1 roadmap package.)*
