Heck yeah—let’s lock a **single, comprehensive, production-grade CLI spec** for RipTide that reflects your ideal product (including **PDF**, **tables**, **WASM**, **headless**, **streaming**, **stealth**, **metrics**, **jobs**, etc.). This is ready to drop into `cli/README.md` and mirrors the architecture you shared.

---

# RipTide CLI — Production-Grade Spec

## Design tenets

* **One binary, many roles:** crawl • render • extract • pdf • table • search • stream • monitor • admin.
* **Deterministic contracts:** stable JSON/NDJSON outputs, explicit exit codes, reproducible artifacts.
* **Progressive enhancement:** **Raw → WASM → Headless** (auto-gate), **Schema → CSS → Regex → LLM** (extraction).
* **Observability-first:** rich traces, timings, per-phase metrics, artifacts for replay.
* **Policy driven:** domain profiles & schema cache; flags > env > profile > config.
* **Offline-friendly:** cache-first modes, replay from saved HTML/DOM/PDF.
* **Safety & stealth:** robots by default, rate limiting, anti-detection, PII redaction.

---

## Global interface

```
riptide
  {crawl, render, extract, pdf, table, search, schema, domain, session, job,
   cache, stream, headless, metrics, bench, test, validate, system,
   plugins, auth, config}
  [global options]
```

### Global options

* `-o, --output {json,ndjson,table,yaml,md}`  (TTY→table, piped→json)
* `-v/--verbose` (repeatable), `--trace`, `--quiet`, `--no-color`
* `--api-url <url>`, `--api-key <key>` (`RIPTIDE_API_URL`, `RIPTIDE_API_KEY`)
* `--profile <name>`, `--config <path>` (YAML/TOML)
* `--timeout <ms>`, `--retries <n>`, `--concurrency <n>`
* `--save-artifacts <dir>` (html, dom.json, pdf, har, screenshot, logs)
* `--pii-scan {on,off}` (default off)
* `--telemetry {on,off}` (default off)

### Exit codes

```
0  OK
2  Partial success
3  Validation/config error
4  Network/DNS/TLS error
5  Extraction failure (raw/wasm/css/regex/llm)
6  Headless renderer failure
7  Robots/rate-limit policy violation
8  Cache/storage error
9  Plugin/WASM module error
10 Internal/unknown
```

### Artifacts & replay

* **Saved**: `*.html`, `*.dom.json`, `*.pdf`, `*.har`, `*.png`, `run.log`, `trace.json`.
* **Replay**: any command that normally fetches can accept `--input-file` or `--stdin` to work offline.

### Config precedence

**flags > env > profile > config file**
Profiles: `~/.config/riptide/profiles/<name>.yml`
Domain profiles: `~/.config/riptide/domains/<host>.yml`

---

## Subcommands

### 1) `extract` — Unified content extraction (raw/WASM/headless)

**Purpose:** Structured content extraction with adaptive engine + multi-strategy extraction.

**Key flags**

```
--url/-u <url> | --input-file <html|md|pdf> | --stdin
--engine {auto,raw,wasm,headless}
--strategy {auto,css,regex,llm,chain:css,regex,parallel:all,fallback:css}
--schema <file.json> | --schema-id <id>
--selector "<css>" | --regex "<re>"
--chunk {none,sentence,topic,sliding:2048/256}
--show-confidence --metadata
--confidence-threshold <0..1>  (default 0.75)
--headless-wait {load,network-idle,selector:#ready}
--headless-timeout <ms> --proxy <url> --stealth {off,low,med,high,auto}
--no-wasm  # force skip WASM path
```

**Output (JSON/NDJSON)**

```json
{
  "url":"https://example.com",
  "engine":"wasm",
  "strategy":"chain:css,regex",
  "timestamp":"2025-10-15T09:12:14Z",
  "content":{"markdown":"...","fields":{"title":"...","author":"...","date":"..." }},
  "confidence":0.92,
  "metadata":{
    "http":{"status":200,"content_type":"text/html","bytes":12456},
    "timings_ms":{"fetch":120,"gate":8,"extract":240,"total":410},
    "gate_decision":"wasm",
    "schema_id":"news/article@v1",
    "pii":{"emails":0,"phones":0}
  },
  "artifacts":{"html":"artifacts/abc.html","dom":"artifacts/abc.dom.json"},
  "errors":[]
}
```

**Behavior**

* **Engine auto:** gate decides **raw → wasm → headless** based on js intensity, dom depth, resource fetches.
* **Strategy auto:** checks domain profile & schema cache → `css→regex→llm` fallback; `parallel:all` picks best by confidence.
* **Confidence**: field coverage + validators (dates/urls/emails) + selector hit rate + denoising score.

**Examples**

```
riptide extract -u https://blog -o json --show-confidence --schema article.schema.json
riptide extract --input-file page.html --strategy chain:css,regex -o md -f out.md
riptide extract -u https://spa.site --engine headless --headless-wait network-idle
```

---

### 2) `render` — Deterministic JS rendering & capture

**Flags**

```
--url <url>
--wait {load,network-idle,selector:#ready}
--screenshot {none,viewport,full}
--pdf --html --dom --har
--cookie-jar <file> --storage-state <file>
--proxy <url> --stealth {off,low,med,high,auto}
```

**Output**

```json
{
  "url":"https://app.site",
  "wait":"network-idle",
  "artifacts":{"html":"...","dom":"...","har":"...","screenshot":"...","pdf":"..."},
  "timings_ms":{"render":1380,"total":1460},
  "errors":[]
}
```

---

### 3) `crawl` — High-throughput spider

**Flags**

```
--url/-u <url> (repeatable) --depth <n> --max-pages <n> --follow-external
--rate <req/s> --delay-ms <ms> --robots {respect,ignore} --user-agent <str>
--allow <glob|re> --deny <glob|re> (repeatable)
--state {save,load} <file> --resume
--with-content  # inlines extract result per page
--stream -o ndjson
```

**Per-page NDJSON**

```json
{"url":"https://example.com/a","status":"visited","depth":1,
 "links_found":23,"extract":{"fields":{"title":"..."},"confidence":0.84},
 "timings_ms":{"fetch":80,"extract":200,"total":320},"policy":{"robots":"respected"}}
```

---

### 4) `pdf` — PDF pipeline

**Subcommands**

```
pdf extract --file <doc.pdf> [--tables] [--images] [--ocr] --out out.json
pdf to-md --file <doc.pdf> --out out.md  # clean markdown
pdf info --file <doc.pdf>                 # metadata, page count, fonts
pdf stream --file <doc.pdf> -o ndjson     # page-by-page streaming
```

**`pdf extract` output**

```json
{
  "file":"doc.pdf",
  "pages":42,
  "text":"... (optional) ...",
  "tables":[
    {"page":3,"cells":[{"r":0,"c":0,"rowspan":1,"colspan":2,"text":"Header"}], "classification":"data"}
  ],
  "images":[{"page":5,"bbox":[...],"format":"png","path":"artifacts/img-5-1.png"}],
  "timings_ms":{"parse":420,"tables":210}
}
```

---

### 5) `table` — HTML/Doc table extraction

**Subcommands**

```
table extract --url <url>|--input-file <html> --merge-cells --header-detect --footer-detect --classify --out tables.json
table to-csv --input tables.json --out out.csv
table to-md  --input tables.json --out out.md
```

**Normalized table model**

```json
{"tables":[
  {"source":"url/page#1","classification":"data",
   "cells":[{"r":0,"c":0,"rowspan":1,"colspan":2,"text":"Header"}],
   "headers":[{"r":0,"c":0,"span":2}], "footers":[]}
]}
```

---

### 6) `search` — Web search + optional deep follow

```
search --query/-q "rust web scraping" --limit 10 --domain github.com --include-content \
       --follow-depth 1 --max-pages 50 --strategy auto -o table
```

NDJSON: each hit with URL, snippet, rank, optional extracted fields if `--include-content`.

---

### 7) `schema` — Per-site extraction intelligence

```
schema learn --url https://site/page --goal article --out schema.json
schema test  --schema schema.json --url https://site/page --report md|json
schema diff  --old old.json --new new.json
schema push  --schema schema.json --schema-id site/article@v1
schema list | schema show <id> | schema rm <id>
```

`schema test` computes selector hit rates, field coverage, **DOM drift score**, suggestions.

---

### 8) `domain` — Domain profiles & drift

```
domain init example.com
domain profile set example.com --stealth auto --wait network-idle \
  --engine auto --strategy auto --rate 1.5 --robots respect \
  --schema-id news/article@v1 --confidence-threshold 0.75
domain drift --url https://example.com --baseline baseline.dom.json --report
```

Profiles auto-apply by host.

---

### 9) `session` — Browser/auth state

```
session new --name acme --cookie-jar jar.json --storage-state state.json
session set-cookie --name acme --cookie 'k=v; Domain=example.com'
session export --name acme --out state.json
session list | session rm <name>
```

---

### 10) `job` — Async queue

```
job submit extract --url ... --strategy auto --stream
job list | job status <id> | job logs <id> --follow | job cancel <id> | job replay <id>
```

---

### 11) `cache` — Inspect, warm, purge

```
cache status | cache stats -o json
cache warm --url-file urls.txt
cache clear [--domain example.com]
cache validate
```

---

### 12) `stream` — Client helpers

```
stream sse --endpoint /crawl --params '...'
stream ws  --endpoint /crawl --subscribe events
```

---

### 13) `headless` — Renderer controls (debug)

```
headless pool status
headless screenshot --url ... --mode full --out shot.png
headless inspect --url ... --console --network --storage
```

---

### 14) `metrics` — Ops monitoring

```
metrics show
metrics tail --interval 2s
metrics export --prom --file metrics.prom
metrics health-score
metrics phases
```

---

### 15) `bench` — Performance benchmarking

```
bench urls --file tests/bench.txt --iterations 5 \
  --targets "static<500,news<1000,complex<3000" --export bench.json
```

---

### 16) `test` — Corpus tests & reports

```
test suite --urls tests/corpus.txt --out report/ --stream
test report --dir report/ --format markdown
test coverage --strategies --urls tests/corpus.txt
```

---

### 17) `validate` & `system`

```
validate --comprehensive          # config, WASM path, Redis, headless connectivity
system check --production         # CPU features, WASM runtime, memory limits
system profile                    # quick perf baseline
```

---

### 18) `plugins` — WASM components

```
plugins list
plugins install ./my_extractor.wasm
plugins remove <id>
plugins verify --id <id>          # signature + WASI caps
plugins bench --id <id> --input sample.html
```

---

### 19) `auth` & `config`

```
auth login --api-key ...
auth whoami
config init | config show | config set key=value | config path
riptide completion {bash|zsh|fish|powershell}
```

---

## Strategy Router v1.0 (contract)

* **Engine (auto):**
  Gate heuristics → `raw` (clean HTML) / `wasm` (unclean/static+scripts) / `headless` (SPA/JS-heavy).
* **Extraction (auto):**
  If domain schema exists → prefer `css`. Else `chain(css→regex→llm)` with confidence gating.
  `parallel:all` runs all, picks highest confidence; includes `alt_candidates` when `--trace`.

**Confidence drivers**

* Field validators (e.g., ISO dates, URL shape, numeric ranges)
* Selector hit rate / regex precision
* Content density & boilerplate ratio
* Table structure sanity (if table mode)
* Optional **PII penalty** (if `--pii-scan on`)

---

## Headless abstraction seam

Create a trait so you can swap Chromiumoxide ↔ Spider-Chrome:

```rust
#[async_trait::async_trait]
pub trait Headless {
    async fn render(&self, url: Url, wait: WaitCond, timeout: Duration, stealth: Stealth) -> Result<Artifacts>;
    async fn screenshot(&self, url: Url, mode: Screenshot) -> Result<PathBuf>;
    async fn har(&self) -> Result<Har>;
}
```

Provide adapters:

* `riptide-headless-chromiumoxide` (default)
* `riptide-headless-spiderchrome` (feature-flagged)

---

## WASM path & initialization (robustness)

* Config keys: `wasm.module_path`, `wasm.preload={on,off}`, `wasm.memory_limit_mb`.
* CLI override: `--wasm-path`.
* `validate` checks path & WASI capabilities; **fail fast** with exit `3`.
* `--init-timeout-ms` for WASM; on timeout:

  * if `--engine auto`: warn + fall back to `raw`; else exit `5`.
* `--no-wasm` for isolation when debugging.

---

## PDF & Table quality (production)

* **PDF:**
  Text + layout blocks; **OCR** fallback (Tesseract/Leptonica) feature-flagged; image extraction to artifacts; clean Markdown via reading order; streaming page events via NDJSON.
* **Tables:**
  `colspan/rowspan`, header/footer detection, **layout vs data** classification, CSV/MD parity, confidence + per-cell provenance (source bbox for PDF; DOM path for HTML).

---

## Stealth & policy

* `stealth {off,low,med,high,auto}` with **progressive escalation** on detection (HTTP 403/signals).
* Domain profile keys: `stealth`, `rate`, `robots`, `user_agent`, `proxy_pool`, `cookie_policy`.
* **Robots respected by default** for `crawl`/`search`.
* **Rate guard:** warn or refuse extremely aggressive settings unless `--force`.

---

## Observability

* `--trace` emits per-phase spans (fetch, gate, wasm, headless, extract), timings, selected strategies, and alt candidates (if `parallel:all`).
* Prometheus metrics: counters, histograms for phase timings, error classes, cache hit rate, headless pool stats.
* `/pipeline/phases` mirrored via `metrics phases`.

---

## CI harness (ready-to-use)

**Comprehensive corpus run**

```bash
# tests/run-corpus.sh
set -euo pipefail
URLS=${1:-tests/corpus.txt}
OUT="artifacts/$(date +%Y%m%d_%H%M%S)"; mkdir -p "$OUT"
while read -r url; do
  riptide extract --url "$url" --engine auto --strategy auto \
    --timeout 15000 --save-artifacts "$OUT" -o ndjson \
    >> "$OUT/results.ndjson" || true
done < "$URLS"
riptide test report --dir "$OUT" --format markdown > "$OUT/report.md"
```

**Perf gates**

* p95: **static < 500ms**, news < 1s, complex < 3s
* memory ceiling: < 100MB/extraction (avg), watchdog 2GB/job

---

## Production checklist

* `validate --comprehensive`
* `system check --production`
* `bench urls --file tests/bench.txt --iterations 5 --export bench.json`
* **Success criteria**

  * Static: >90% success
  * News: >85%
  * E-commerce: >70%
  * SPA (with headless): >80% post-integration
  * Overall errors <1% (non-policy)

---

## Quick recipes

**Crawl docs, stream extracts**

```bash
riptide crawl -u https://docs.example.com --depth 4 --max-pages 1000 \
  --with-content --stream -o ndjson > corpus.ndjson
```

**Learn schema once; auto-heal drift**

```bash
riptide schema learn --url https://news.site/x --goal article --out news.article.v1.json
riptide schema push --schema news.article.v1.json --schema-id news/article@v1
riptide extract -u https://news.site/y --schema-id news/article@v1 --show-confidence \
  --confidence-threshold 0.8 -o json
```

**Render SPA deterministically**

```bash
riptide render -u https://app.site --wait network-idle --dom --har --pdf \
  --screenshot full --save-artifacts ./art
```

**PDF → Markdown + tables**

```bash
riptide pdf to-md --file whitepaper.pdf --out whitepaper.md
riptide pdf extract --file whitepaper.pdf --tables --out tables.json
riptide table to-csv --input tables.json --out tables.csv
```

---

This spec folds in **everything you listed** (WASM, headless, dual-path pipeline, PDF pipeline, tables with merges, streaming protocols, session/stealth, domain profiles, schema intelligence, metrics/bench/tests, caching, plugins). If you want, I can generate:

* a **`clap` struct skeleton** for all subcommands/flags, and
* **JSON Schemas** for `extract`, `render`, `pdf extract`, `table extract` outputs,

so your docs, tests, and clients all lock onto one stable contract.
Excellent — defining **CLI UX** is the next step to making RipTide feel premium and “human-grade.” You already have the power features; what’s missing is **the surface polish**: menus, contextual help, progressive disclosure, onboarding cues, and discoverability.

Here’s a comprehensive UX specification covering **menus, help system, command discoverability, user flows, and interactive features** — tuned for a power-tool CLI like RipTide.

---

# 🧭 RipTide CLI UX Specification

## 1. CLI personality & tone

* **Voice:** professional but approachable (“command-center for intelligent crawling”)
* **Tone:** concise, confident, never noisy
* **Style:** lower-case subcommands; helpful summaries with syntax examples before flag walls
* **Consistency:** every command returns a one-line summary before long help

Example header:

```
RipTide — adaptive web crawling & content extraction CLI
Version 0.1.0 (build a1b2c3d) | Docs: https://riptide.dev/docs/cli
```

---

## 2. Command discovery & menus

### 🧱 Top-level menu (`riptide --help`)

```
RipTide CLI 0.1.0
High-performance crawler & extractor built in Rust + WASM

USAGE:
  riptide <COMMAND> [OPTIONS]

CORE COMMANDS:
  extract      Unified content extraction (Raw/WASM/Headless)
  render       JavaScript rendering & artifact capture
  crawl        Multi-threaded spider with adaptive routing
  pdf          PDF text, table & image extraction
  table        Extract and export HTML or PDF tables
  search       Web search and deep content retrieval
  schema       Manage extraction schemas (learn/test/push)
  domain       Manage domain profiles and drift reports

OPERATIONS & ADMIN:
  job          Background jobs (submit/list/logs)
  cache        Cache management and validation
  session      Manage browser sessions & cookies
  stealth      Stealth mode testing and configuration
  metrics      Monitor system and performance metrics
  bench        Run performance benchmarks
  test         Execute URL test suites
  validate     Preflight checks for configuration & safety
  system       System inspection and diagnostics
  plugins      Manage WASM extraction plugins
  auth         Authentication management
  config       Global configuration management

Use 'riptide help <COMMAND>' or '<COMMAND> --help' for details.
```

---

### 🧭 Secondary help (`riptide help extract`)

```
riptide extract — Adaptive content extraction pipeline

USAGE:
  riptide extract [OPTIONS] --url <URL> | --input-file <FILE>

EXAMPLES:
  riptide extract --url https://example.com --strategy auto
  riptide extract --input-file page.html --output json
  riptide extract --url https://app.site --engine headless --show-confidence

OPTIONS:
  -u, --url <URL>               Source URL to extract
  -i, --input-file <PATH>       Use local HTML/PDF instead of fetching
      --engine <auto|raw|wasm|headless>
      --strategy <auto|css|regex|llm|chain:css,regex|parallel:all|fallback:css>
      --schema <FILE> | --schema-id <ID>
      --chunk <none|sentence|topic|sliding:2048/256>
      --show-confidence         Display confidence score per field
      --metadata                Include HTTP & extraction metadata
      --format <json|ndjson|markdown|csv|md>
  -o, --output <FORMAT>         Output format (json default)
      --save-artifacts <DIR>    Save HTML/DOM/screenshot bundles
      --timeout <MS>            Timeout per extraction
  -v, --verbose                 Show detailed logs
  -q, --quiet                   Suppress non-critical logs

For advanced options:  riptide extract --help --advanced
Docs: https://riptide.dev/docs/cli/extract
```

---

### 🎛  Hierarchical help (progressive disclosure)

Each help screen supports:

* `--examples` → shows real examples grouped by use case
* `--flags` → only prints flag list (machine-parseable)
* `--advanced` → prints expert flags (e.g., `--proxy`, `--stealth`, `--chunk`)
* `--json-schema` → prints output JSON schema for this command

---

## 3. Interactive modes (optional but powerful)

### 🧩 `riptide menu`

Interactive top-level dashboard for exploration:

```
┌───────────────────────────────────────────────────┐
│  RipTide Command Menu                            │
├───────────────────────────────────────────────────┤
│ ▶ Extract content                                 │
│   Render page (JS execution)                      │
│   Crawl multiple pages                            │
│   PDF or Table extraction                         │
│   Search or Deepsearch                            │
│   View metrics / health                           │
│   Configure or validate system                    │
│   Quit                                            │
└───────────────────────────────────────────────────┘
↑↓ to move, Enter to select
```

Built using `inquire` or `dialoguer` crates, toggled via `riptide menu` or `riptide --interactive`.

Each submenu surfaces recommended flags interactively (autocomplete + validation).

---

### 💡 Inline tips

Every long-running command prints contextual hints:

```
[tip] Run with --save-artifacts ./artifacts for debugging.
[tip] Use --engine headless if the page relies on JavaScript.
```

---

### 🧠 Smart suggestions

If a user typo’s a command:

```
$ riptide extracct
Unknown command 'extracct'. Did you mean 'extract'?
```

If an extraction fails due to missing WASM:

```
Error: WASM module not found at /opt/riptide/wasm/...
[hint] Run 'riptide validate --wasm' to verify installation.
```

---

### 🪄 Onboarding walkthrough

First-run prompt:

```
Welcome to RipTide 🦑
We'll set up your environment quickly.
→ Detected Redis running ✔
→ WASM module found ✔
→ Headless renderer available ✔
→ Config profile 'default' created
Try:  riptide extract --url https://example.com
```

Triggered when no config/profile exists or `riptide init`.

---

## 4. Output ergonomics

### Smart defaults

* **TTY:** pretty tables with colorized status & emojis

  * ✅ success, ⚠ warnings, ❌ errors
* **Non-TTY / piping:** machine-readable JSON/NDJSON
* Columns auto-resize; truncated text supports `--full`

Example table output:

```
URL                               ENGINE   STRATEGY        CONF  STATUS
────────────────────────────────  ───────  ──────────────  ────  ───────
https://example.com               wasm     chain:css,regex 0.91  ✅ OK
https://news.site/article/123     headless auto            0.88  ⚠ slow(2.3s)
https://react.dev                 headless auto            0.82  ✅ OK
```

---

## 5. Command grouping & navigation

| Group            | Commands                                                                               | UX Notes                              |
| ---------------- | -------------------------------------------------------------------------------------- | ------------------------------------- |
| **Core Ops**     | `extract`, `render`, `crawl`, `pdf`, `table`                                           | Always top-listed in help             |
| **Intelligence** | `schema`, `domain`, `search`                                                           | Shown under “Intelligence & Profiles” |
| **System**       | `job`, `cache`, `session`, `stealth`, `metrics`, `bench`, `test`, `validate`, `system` | Shown under “Operations & Monitoring” |
| **Admin/Config** | `plugins`, `auth`, `config`                                                            | Grouped last                          |

Navigation in interactive mode follows same order.

---

## 6. Contextual help triggers

| Situation                         | Help cue                                                                     |
| --------------------------------- | ---------------------------------------------------------------------------- |
| Missing `--url` or `--input-file` | “Add --url or --input-file. See examples: `riptide extract --examples`.”     |
| Timeout                           | “Extraction exceeded timeout. Try `--timeout 15000` or `--engine headless`.” |
| Robots blocked                    | “Blocked by robots.txt. Override with `--robots ignore` if policy allows.”   |
| Stealth detection                 | “Target flagged request. Try `--stealth high` or rotate proxies.”            |
| Cache miss                        | “First visit; caching result. Use `riptide cache warm` for bulk prefill.”    |

---

## 7. Developer & CI UX

### Logging

* **Structured JSON logs** for CI (`--log-format json`)
* **Human logs** for dev (color + emoji)
* Verbosity levels: `-v`, `-vv`, `-vvv`, `--trace`

### Progress indicators

* For batch/streaming: dynamic progress bar (`indicatif`)
* Example:

  ```
  [fetch] 42/200 (21%) | avg 0.32s | ok:39 fail:3
  ```

### Artifacts notice

When `--save-artifacts`:

```
Artifacts saved → ./artifacts/run_2025-10-15_09-30
```

---

## 8. Command aliases (shortcuts)

| Full            | Alias | Purpose                 |
| --------------- | ----- | ----------------------- |
| `extract`       | `ex`  | Fast content extraction |
| `render`        | `rd`  | Render page headlessly  |
| `crawl`         | `cr`  | Crawl multiple pages    |
| `pdf extract`   | `pe`  | Extract from PDF        |
| `table extract` | `te`  | Extract tables          |
| `metrics`       | `mt`  | Show metrics            |
| `validate`      | `val` | Quick validation        |
| `system check`  | `sys` | Diagnostics             |

---

## 9. Interactive examples explorer

`riptide examples` or `riptide extract --examples` opens categorized examples:

```
EXAMPLES: Extraction Scenarios
──────────────────────────────
[1] Basic extraction (auto strategy)
[2] Chain strategies with fallback
[3] Headless rendering for SPAs
[4] Table & PDF processing
[5] Real-time streaming pipeline
[6] Benchmark & performance test
Select an example to print its full command.
```

---

## 10. Contextual error recovery

Errors are never dead-ends; every error message includes:

* **classification**
* **remediation**
* **reference doc**

Example:

```
Error[E005]: Extraction engine timeout (WASM)
↳ Context: riptide-extractor-wasm took >5s to initialize
Suggested fix:
  • Run with --init-timeout-ms 8000
  • Or disable WASM: --no-wasm
Docs: https://riptide.dev/docs/troubleshooting#wasm-timeouts
```

---

## 11. Multi-command examples help (`riptide --recipes`)

```
Top Recipes
────────────
• Crawl a domain and save NDJSON → crawl+extract pipeline
• Extract PDF tables to CSV
• Benchmark performance on a URL set
• Run full validation suite before deploy
Run 'riptide --recipes <name>' for details.
```

---

## 12. Help system technical spec

Implementation hints:

* Use [`clap`](https://docs.rs/clap/latest/clap/) with:

  * `Command::before_help` → short summary
  * `Command::after_help` → example section
  * `Command::mut_subcommand` → attach advanced examples
* Optional: generate dynamic help menus with [`clap_complete`](https://docs.rs/clap_complete/) + `dialoguer` for interactive `riptide menu`.

---

## 13. Accessibility & internationalization

* ANSI color detection + `--no-color`
* Emoji toggle `--no-emoji`
* `LANG` env for localized help messages (planned)

---

## 14. Hidden developer modes

* `--debug-schema` → dumps resolved schema (merged from domain + CLI)
* `--dump-trace` → saves JSON trace of strategy decisions
* `--dry-run` → prints planned actions only
* `--replay` → process saved artifact folder instead of live crawl

---

## 15. Visual consistency

* Every help screen:

  * 80-column soft wrap
  * aligned flag/description columns
  * cyan for commands, yellow for flags, magenta for examples
* Title banner (when in TTY):

  ```
  ╭──────────────────────────────────────────────╮
  │  RipTide CLI 0.1.0 — adaptive web extraction │
  ╰──────────────────────────────────────────────╯
  ```

---

## 16. Future UX ideas

* **TUI dashboard** (`riptide tui`) showing metrics, job queue, and worker load.
* **`riptide doctor`** — auto-diagnose common setup issues.
* **`riptide learn`** — AI-guided schema learning wizard (interactive).
* **Contextual completions:** dynamic flag suggestions based on previous input.

---

## ✅ Summary

**RipTide’s CLI UX goals:**

| Category            | Goal                                                             |
| ------------------- | ---------------------------------------------------------------- |
| **Discoverability** | Clear menus, contextual tips, example-first help                 |
| **Usability**       | Smart defaults, readable tables, consistent JSON                 |
| **Guidance**        | Inline hints, onboarding wizard, recovery suggestions            |
| **Power UX**        | Interactive menu, profiles, recipes, schemas, streaming feedback |
| **Reliability**     | Deterministic exit codes, structured logs, replay artifacts      |

---

Would you like me to follow this up by designing **the help-text layout templates and color scheme (TTY vs JSON modes)** — basically, how the CLI should *look and feel* in live terminals (color codes, spacing, indentation, progress bars, etc.)? That would complete the UX definition.
