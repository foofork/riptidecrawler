# 🧩 Riptide Current-State Codebase Analysis — Crates-Level Inventory

**Purpose**
This task is to deliver a comprehensive, factual description of the *current Riptide codebase* as it exists today.
We need an authoritative view of the crates, their responsibilities, relationships, and integrations — without suggesting any new architecture or design direction.
This will become the foundation for later consolidation and schema-agnostic work.

---

## 📦 Scope

Focus only on the **Rust workspace under `/crates`** and any directly related internal libraries or dependencies.
Do **not** include SDKs, CLIs, front-end, or deployment configurations unless they directly affect crate relationships.

---

## 🎯 Objectives

Provide a factual report answering:

* What functionality exists today and where it lives.
* How crates depend on each other and on external systems.
* What APIs, traits, and configurations are currently active.
* Where schema- or domain-specific code (e.g., “events”) is embedded.

Avoid speculation — only document observable behavior and code.

---

## 🧾 Deliverable Assets

The deliverable consists of **two assets**:

### **1. Markdown Report**

A single structured Markdown document titled
`riptide_current_state_analysis.md`

It should contain the following sections (each section must be filled in):

#### **1️⃣ Crates Table**
Example:
| Crate          | Path           | Purpose (as implemented)    | Key Exports          | External Dependencies       | Internal Dependencies              |
| -------------- | -------------- | --------------------------- | -------------------- | --------------------------- | ---------------------------------- |
| `riptide-core` | `/crates/core` | Handles main pipeline logic | `Pipeline`, `Runner` | `tokio`, `reqwest`, `redis` | `riptide-fetch`, `riptide-extract` |
| ...            | ...            | ...                         | ...                  | ...                         | ...                                |

#### **2️⃣ Dependency Overview**

* Describe or diagram crate relationships (who depends on whom).
* Identify cross-cutting or circular dependencies if any.
* Note crates that overlap in functionality or naming.

#### **3️⃣ Functional Responsibilities**

For each crate, summarize:

* What it does functionally.
* Where its main logic files live (`src/lib.rs`, `src/main.rs`, etc.).
* Any special behaviors (async tasks, workers, queues, etc.).

#### **4️⃣ Public Interfaces**

* Enumerate all HTTP or RPC routes if present.
* For each: method, path, handler function or module.
* What it accepts and returns (types, structs, or JSON shape).
* Which routes are streaming or long-running.

#### **5️⃣ Configuration & Defaults**

* List all config files, env vars, and feature flags in use.
* Record default values and where they are defined.
* Note how values propagate between crates (directly or via shared config).

#### **6️⃣ External Integrations**

For each integration:

* Name of external system (e.g., Redis, Chrome/CDP, LLM, WASM engine).
* Crate(s) that implement it.
* Purpose and main code files handling the integration.
* How failures or timeouts are handled, if visible.

#### **7️⃣ Data Models & Storage**

* List main structs or data models used for raw data, extracted entities, or persisted state.
* Crate ownership of each type.
* Identify where (if anywhere) schema-specific logic exists.

#### **8️⃣ Observability & Diagnostics**

* What logging, metrics, or tracing functionality exists now.
* Crates responsible for health checks or telemetry.
* Names of key metrics or log event categories.

#### **9️⃣ Concurrency, Scheduling, Background Work**

* Describe current async/task execution models (Tokio, threads, queues).
* Where retry, rate-limit, or backoff behavior is implemented.

#### **🔟 Schema or Domain Coupling**

* Identify crates that contain schema- or domain-specific code (e.g., “events”).
* Specify how strongly coupled this code is to core components.

#### **🔢 General Observations**

* Patterns or inconsistencies in crate structure.
* Duplicate or overlapping code areas.
* Unusual dependencies or naming.

---

### **2. Supplementary Assets**

In addition to the main report:

1. **Dependency Diagram**

   * A simple ASCII diagram, Mermaid graph, or text hierarchy of crate relationships.
   * Example format:

     ```
     core ─┬─ fetch
            ├─ extract
            ├─ normalize
            └─ dedup
     api ──→ core
     ```
2. **Configuration Reference Sheet**

   * A Markdown table of configuration keys, default values, and where they are defined.
   * Example:

     | Key                   | Default                  | Source               | Used By          |
     | --------------------- | ------------------------ | -------------------- | ---------------- |
     | `RIPTIDE_CONCURRENCY` | `8`                      | `.env` / `server.rs` | `core`, `crawl`  |
     | `REDIS_URL`           | `redis://localhost:6379` | `.env`               | `store`, `queue` |

---

## 🧰 Research Expectations

* Include **code references** for each observation (crate path + file + key function or struct name).
* Copy short, relevant code snippets where it clarifies purpose.
* If functionality appears incomplete or disabled, note that explicitly.
* No redesigns, predictions, or idealized descriptions — only *what is currently implemented*.

---

## 🧱 Suggested Workflow

1. **List all crates**

   ```bash
   cargo metadata --no-deps -q | jq -r '.packages[].name'
   ```
2. **Locate route definitions**

   ```bash
   rg -n --glob 'crates/**/src/**/*.rs' '#\\[(get|post|put|delete)]'
   ```
3. **Find major public exports**

   ```bash
   rg -n --glob 'crates/**/src/**/*.rs' 'pub (trait|struct|enum)'
   ```
4. **Identify config/env usage**

   ```bash
   rg -n --glob 'crates/**/src/**/*.rs' 'env|dotenv|Config|PROFILE|timeout|rate|ttl'
   ```

---

## ✅ Completion Criteria

A submission is **complete** when it includes:

* [x] Full crate table (every crate listed).
* [x] Dependency map or diagram.
* [x] Documented responsibilities, interfaces, and data types.
* [x] Configuration reference table.
* [x] Code references for every major claim.
* [x] Clear identification of schema-specific logic.
* [x] Markdown file + any diagrams (Mermaid, PNG, or text) uploaded in `/docs/analysis/`.

---

## 📤 Expected Deliverables Summary

| Deliverable        | Filename                                                            | Description                                                      |
| ------------------ | ------------------------------------------------------------------- | ---------------------------------------------------------------- |
| Main report        | `docs/analysis/riptide_current_state_analysis.md`                   | Full written inventory of crates and their functionality         |
| Dependency diagram | `docs/analysis/riptide_crate_dependencies.mmd` (or `.txt` if ASCII) | Visual or textual crate relationship map                         |
| Config reference   | `docs/analysis/riptide_config_reference.md`                         | Consolidated list of environment keys, defaults, and crate usage |