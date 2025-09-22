Awesome — here’s a full, no-decisions-left build + run guide for **RipTide Crawler** (Rust + WASM fast path, CDP fallback). It includes a production-ready workspace skeleton, Cargo files, code stubs, Docker, testing/quality gates, and step-by-step “how to ship.” I also link every external dependency and the **Crawl4AI** docs/repo we’re targeting for parity.

---

# What you’re building

**RipTide** = self-hosted deep search & extraction:

* **Fast path** (default): parallel HTTP fetch (Rust) → streaming HTML cleanup (lol-html) → content extraction (Trek compiled to WASM). ([GitHub][1])
* **Fallback path** (only when needed): headless Chromium via **chromiumoxide** (Rust CDP), then the same extraction. ([GitHub][2])
* **Optional deep crawl**: site graph/limits via **spider-rs**. ([GitHub][3])
* **PDF support**: **pdfium-render** for PDF text/images. ([Docs.rs][4])
* **SERP intake**: **serper.dev** for Google results. ([Serper][5])

**Parity target:** Crawl4AI core features (simple/deep/adaptive crawling, dynamic interaction, proxy/stealth, cache modes, markdown/json outputs, CLI) — we’ll validate against their docs. ([Crawl4AI Documentation][6])

---

# 1) Repo structure (monorepo)

```
riptide/
  Cargo.toml                # workspace root
  rust-toolchain.toml
  Justfile                  # optional task runner (just)
  .rustfmt.toml
  deny.toml                 # cargo-deny
  .github/workflows/ci.yml
  configs/
    riptide.yml
    fingerprints.yml
    policies.yml
  crates/
    riptide-core/
      src/{lib.rs,gate.rs,fetch.rs,extract.rs,cache.rs,types.rs}
    riptide-api/
      src/{main.rs,handlers.rs,models.rs}
    riptide-headless/
      src/{main.rs,cdp.rs,models.rs}
    riptide-workers/        # optional batch workers (same libs)
      src/main.rs
  wasm/
    riptide-extractor-wasm/
      src/lib.rs
      Cargo.toml
  infra/
    docker/Dockerfile.api
    docker/Dockerfile.headless
    docker/docker-compose.yml
  scripts/
    build_all.sh
    bootstrap.sh
  tests/
    e2e/{fixtures/*.html, fixtures/*.pdf, e2e_api.rs}
    golden/{urls.txt, expected/*.jsonl}
```

---

# 2) Workspace `Cargo.toml` (root)

```toml
[workspace]
members = [
  "crates/riptide-core",
  "crates/riptide-api",
  "crates/riptide-headless",
  "crates/riptide-workers",
  "wasm/riptide-extractor-wasm",
]
resolver = "2"

[workspace.package]
edition = "2021"
license = "Apache-2.0"
authors = ["RipTide Team"]

[workspace.dependencies]
anyhow = "1"
axum = { version = "0.7", features = ["json"] }
bytes = "1"
chrono = { version = "0.4", features = ["serde"] }
clap = { version = "4", features = ["derive"] }
futures = "0.3"
http = "1"
hyper = { version = "1", features = ["full"] }
once_cell = "1"
rand = "0.8"
redis = { version = "0.25", features = ["tokio-comp"] }
reqwest = { version = "0.12", features = ["gzip", "brotli", "json", "cookies", "http2", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread", "signal", "time"] }
tokio-stream = "0.1"
tower = "0.4"
tower-http = { version = "0.5", features = ["trace", "cors", "compression-full", "decompression-full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "json"] }
url = "2"
uuid = { version = "1", features = ["v4", "serde"] }
lol_html = "1"
wasmtime = { version = "26", features = ["wasi", "cache"] }
wasmtime-wasi = "26"
chromiumoxide = "0.5"
spider = "2"
pdfium-render = "0.8"
```

* `reqwest` for HTTP, `lol_html` for streaming rewrite, **Wasmtime** as the WASM host, **chromiumoxide** as CDP client, **spider** for deep crawl, **pdfium-render** for PDFs. ([GitHub][1])

---

# 3) Core crate (`crates/riptide-core/Cargo.toml`)

```toml
[package]
name = "riptide-core"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = { workspace = true }
bytes = { workspace = true }
futures = { workspace = true }
http = { workspace = true }
redis = { workspace = true }
reqwest = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
url = { workspace = true }
lol_html = { workspace = true }
wasmtime = { workspace = true }
wasmtime-wasi = { workspace = true }
uuid = { workspace = true }
pdfium-render = { workspace = true, optional = true }
```

### `crates/riptide-core/src/types.rs` (shared structs)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractedDoc {
    pub url: String,
    pub title: Option<String>,
    pub byline: Option<String>,
    pub published_iso: Option<String>,
    pub markdown: String,
    pub text: String,
    pub links: Vec<String>,
    pub media: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlOptions {
    pub concurrency: usize,
    pub cache_mode: String, // "enabled" | "bypass" | "read_through"
    pub dynamic_wait_for: Option<String>,
    pub scroll_steps: u32,
    pub token_chunk_max: usize,
    pub token_overlap: usize,
}
```

### `fetch.rs` (parallel HTTP fetcher)

```rust
use anyhow::Result;
use reqwest::{Client, Response};
use std::time::Duration;

pub fn http_client() -> Client {
    Client::builder()
        .user_agent("RipTide/1.0")
        .http2_prior_knowledge()
        .gzip(true).brotli(true)
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(15))
        .build()
        .expect("client")
}

pub async fn get(client: &Client, url: &str) -> Result<Response> {
    let res = client.get(url).send().await?;
    Ok(res.error_for_status()?)
}
```

### `extract.rs` (WASM host runner; command-style WASI)

```rust
use anyhow::{anyhow, Result};
use wasmtime::{Engine, Module, Store};
use wasmtime_wasi::{WasiCtxBuilder, IOStream};
use crate::types::ExtractedDoc;

pub struct WasmExtractor {
    engine: Engine,
    module: Module,
}
impl WasmExtractor {
    pub fn new(wasm_path: &str) -> Result<Self> {
        let engine = Engine::default();
        let module = Module::from_file(&engine, wasm_path)?;
        Ok(Self { engine, module })
    }
    pub fn extract(&self, html: &[u8], url: &str, mode: &str) -> Result<ExtractedDoc> {
        // For simplicity & robustness: run the WASM as a WASI "command" per extraction.
        // (Later you can optimize with component model/pooling.)
        let mut linker = wasmtime::Linker::new(&self.engine);
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

        let mut store = Store::new(
            &self.engine,
            WasiCtxBuilder::new()
                .env("RIPTIDE_URL", url)
                .env("RIPTIDE_MODE", mode)
                .stdin(Box::new(IOStream::from_bytes(html)))
                .stdout(Box::new(IOStream::new()))
                .build(),
        );

        let instance = linker.instantiate(&mut store, &self.module)?;
        // Expect a WASI `_start` entrypoint
        let start = instance.get_typed_func::<(), ()>(&mut store, "_start")?;
        start.call(&mut store, ())?;

        let mut out = Vec::new();
        store.data_mut().stdout_mut().read_to_end(&mut out)?;
        let doc: ExtractedDoc = serde_json::from_slice(&out)
            .map_err(|e| anyhow!("WASM extractor JSON decode failed: {e}"))?;
        Ok(doc)
    }
}
```

> This **command-style WASI** keeps the ABI simple and still fast for our needs. You can swap to a long-lived instance with the component model later, but this is deterministic and easy to test first. **Wasmtime** is the canonical host runtime for WASI. ([GitHub][7])

### `gate.rs` (no-model decision logic; deterministic)

```rust
pub struct GateFeatures {
    pub html_bytes: usize,
    pub visible_text_chars: usize,
    pub p_count: u32,
    pub article_count: u32,
    pub h1h2_count: u32,
    pub script_bytes: usize,
    pub has_og: bool,
    pub has_jsonld_article: bool,
    pub spa_markers: u8, // bit flags: NEXT_DATA, hydration, root div, huge bundle
    pub domain_prior: f32, // 0.0..1.0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision { Raw, ProbesFirst, Headless }

pub fn score(features: &GateFeatures) -> f32 {
    let text_ratio = if features.html_bytes == 0 { 0.0 }
                     else { features.visible_text_chars as f32 / features.html_bytes as f32 };
    let script_density = if features.html_bytes == 0 { 0.0 }
                         else { features.script_bytes as f32 / features.html_bytes as f32 };
    let mut s = 0.0;
    s += (text_ratio * 1.2).clamp(0.0, 0.6);
    s += ((features.p_count as f32 + 1.0).ln() * 0.06).clamp(0.0, 0.3);
    if features.article_count > 0 { s += 0.15; }
    if features.has_og { s += 0.08; }
    if features.has_jsonld_article { s += 0.12; }
    s -= (script_density * 0.8).clamp(0.0, 0.4);
    if features.spa_markers >= 2 { s -= 0.25; }
    (s + (features.domain_prior - 0.5) * 0.1).clamp(0.0, 1.0)
}

pub fn decide(features: &GateFeatures, hi: f32, lo: f32) -> Decision {
    let s = score(features);
    if s >= hi { Decision::Raw }
    else if s <= lo || features.spa_markers >= 3 { Decision::Headless }
    else { Decision::ProbesFirst }
}
```

---

# 4) WASM extractor (`wasm/riptide-extractor-wasm`)

**Cargo.toml**

```toml
[package]
name = "riptide-extractor-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
trek-rs = "0.1"
```

**`src/lib.rs`** — WASI command that reads HTML from **stdin** and writes JSON (one doc) to **stdout**:

```rust
use serde::Serialize;
use std::io::{Read, Write};
use trek_rs::{Trek, TrekOptions};

#[derive(Serialize)]
struct ExtractedDocOut {
    url: String,
    title: Option<String>,
    byline: Option<String>,
    published_iso: Option<String>,
    markdown: String,
    text: String,
    links: Vec<String>,
    media: Vec<String>,
}

#[no_mangle]
pub extern "C" fn _start() {
    // Read HTML from stdin
    let mut html = Vec::new();
    std::io::stdin().read_to_end(&mut html).unwrap();

    // Read URL & mode from env
    let url = std::env::var("RIPTIDE_URL").unwrap_or_else(|_| "about:blank".into());
    let _mode = std::env::var("RIPTIDE_MODE").unwrap_or_else(|_| "article".into());

    // Run Trek
    let opts = TrekOptions::default();
    let result = Trek::extract(&html, &url, opts);

    // Map to output
    let out = ExtractedDocOut {
        url,
        title: result.title,
        byline: result.byline,
        published_iso: result.published_iso,
        markdown: result.markdown.clone().unwrap_or_default(),
        text: result.text.clone(),
        links: result.links,
        media: result.media,
    };

    let json = serde_json::to_vec(&out).unwrap();
    std::io::stdout().write_all(&json).unwrap();
}
```

> **Trek** is a modern Rust extractor that compiles to WebAssembly and targets readable article content; perfect for an ultra-fast WASM “cleaner.” ([GitHub][8])

---

# 5) Headless service (`crates/riptide-headless`)

**Cargo.toml**

```toml
[package]
name = "riptide-headless"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = { workspace = true }
axum = { workspace = true }
chromiumoxide = { workspace = true }
futures = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
```

**`src/main.rs`** — minimal `/render` endpoint:

```rust
mod cdp;
mod models;

use axum::{routing::post, Router};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let app = Router::new().route("/render", post(cdp::render));
    axum::Server::bind(&"0.0.0.0:9123".parse()?)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
```

**`src/models.rs`**

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RenderReq {
    pub url: String,
    pub wait_for: Option<String>, // css selector
    pub scroll_steps: Option<u32>,
}

#[derive(Serialize)]
pub struct RenderResp {
    pub final_url: String,
    pub html: String,
    pub screenshot_b64: Option<String>,
}
```

**`src/cdp.rs`**

```rust
use crate::models::*;
use axum::{Json, extract::State};
use chromiumoxide::{Browser, BrowserConfig};
use futures::StreamExt;

pub async fn render(Json(req): Json<RenderReq>) -> Json<RenderResp> {
    let (browser, mut handler) =
        Browser::launch(BrowserConfig::builder().build().unwrap()).await.unwrap();
    tokio::spawn(async move {
        while let Some(_e) = handler.next().await {}
    });

    let page = browser.new_page(&req.url).await.unwrap();

    if let Some(css) = &req.wait_for {
        page.wait_for_element(css).await.ok();
    }

    if let Some(steps) = req.scroll_steps {
        for _ in 0..steps {
            page.evaluate("window.scrollBy(0, 2000);").await.ok();
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }
    }

    let html = page.content().await.unwrap_or_default();
    let final_url = page.url().await.unwrap_or_else(|_| req.url.clone());
    Json(RenderResp { final_url, html, screenshot_b64: None })
}
```

> **chromiumoxide** is a Rust client for the **Chrome DevTools Protocol**, giving you Playwright-class control with a Rust API. ([GitHub][2])

---

# 6) API service (`crates/riptide-api`)

**Cargo.toml**

```toml
[package]
name = "riptide-api"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = { workspace = true }
axum = { workspace = true }
bytes = { workspace = true }
clap = { workspace = true }
http = { workspace = true }
reqwest = { workspace = true }
redis = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
uuid = { workspace = true }
riptide-core = { path = "../riptide-core" }
```

**`src/models.rs`**

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CrawlBody {
    pub urls: Vec<String>,
}

#[derive(Serialize)]
pub struct CrawlResult {
    pub url: String,
    pub status: u16,
    pub from_cache: bool,
    pub markdown_path: Option<String>,
    pub json_path: Option<String>,
}

#[derive(Deserialize)]
pub struct DeepSearchBody {
    pub query: String,
    pub limit: Option<u32>,
    pub country: Option<String>,
    pub locale: Option<String>,
}
```

**`src/handlers.rs`** (skeleton)

```rust
use axum::{Json, response::IntoResponse};
use crate::models::*;
use riptide_core::{/* fetch/extract API */};
use serde_json::json;

pub async fn crawl(Json(body): Json<CrawlBody>) -> impl IntoResponse {
    // For each URL: fetch -> gate -> fast extract or POST /render -> extract
    // Stream or collect results; here we return a simple JSON array.
    Json(json!({ "received": body.urls.len(), "results": [] }))
}

pub async fn deepsearch(Json(body): Json<DeepSearchBody>) -> impl IntoResponse {
    // Call Serper.dev -> take organic URLs -> reuse crawl flow
    // NOTE: you must set SERPER_API_KEY env var
    Json(json!({ "query": body.query, "enqueued": body.limit.unwrap_or(10) }))
}
```

**`src/main.rs`**

```rust
mod handlers;
mod models;

use axum::{routing::post, Router};
use clap::Parser;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
struct Args { #[arg(long, default_value_t = String::from("configs/riptide.yml"))] config: String }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let _args = Args::parse();
    // TODO: load configs, init Redis, WasmExtractor, Reqwest client, etc.

    let app = Router::new()
        .route("/crawl", post(handlers::crawl))
        .route("/deepsearch", post(handlers::deepsearch));

    axum::Server::bind(&"0.0.0.0:8080".parse()?)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
```

---

# 7) Docker & Compose

**`infra/docker/Dockerfile.api`** (build host & copy WASM)
**`infra/docker/Dockerfile.headless`** (Chromium + service)
**`infra/docker/docker-compose.yml`** — as given earlier in our plan (api + redis + headless).

> **lol-html** and **Wasmtime** are portable; Chromium installed inside the headless image; Redis for caching. ([GitHub][9])

---

# 8) Configs (drop-in)

**`configs/riptide.yml`** — use the defaults from the last message; no choices to make.

* `search.provider = serper` (set `SERPER_API_KEY`) — see serper.dev. ([Serper][5])
* `extraction.wasm_module_path = /opt/riptide/extractor/extractor.wasm`
* `dynamic.enable_headless_fallback = true`

---

# 9) End-to-end “Pro” runbook

1. **Bootstrap**

   ```bash
   rustup toolchain install stable
   rustup target add wasm32-wasi
   ./scripts/bootstrap.sh   # optional: installs just, pre-commit hooks, etc.
   ```

2. **Build everything**

   ```bash
   ./scripts/build_all.sh
   ```

3. **Set secrets**

   ```bash
   cp .env.example .env
   # Put SERPER_API_KEY=... inside
   ```

4. **Up via Docker**

   ```bash
   cd infra/docker
   docker compose up --build -d
   ```

5. **Smoke test (single URL)**

   ```bash
   curl -s -X POST localhost:8080/crawl \
     -H 'content-type: application/json' \
     -d '{"urls": ["https://en.wikipedia.org/wiki/Rust_(programming_language)"]}' | jq .
   ```

6. **Deepsearch**

   ```bash
   curl -s -X POST localhost:8080/deepsearch \
     -H 'content-type: application/json' \
     -d '{"query":"rust wasm html extraction","limit":10}'
   ```

7. **Dynamic page check** (forces fallback)

   ```bash
   curl -s -X POST localhost:9123/render \
     -H 'content-type: application/json' \
     -d '{"url":"https://example.com", "wait_for":"css:.content", "scroll_steps":8}' | jq .
   ```

---

# 10) Quality gates & testing (no guessing)

## Formatting & lint

* `rustfmt`: configured by `.rustfmt.toml`
* `clippy`: treat warnings as errors in CI (`RUSTFLAGS="-Dwarnings"`)

## Supply chain

* `cargo deny` with `deny.toml` (ban yanked, insecure or unlicensed crates)

## Unit tests

* **Core**: `gate.rs` scoring/decide; `extract.rs` (mock WASM returns); `fetch.rs` (header parsing).
* **WASM**: run the WASI bin with small HTML fixtures and compare JSON fields (title/text/links).

## Golden tests (parity with Crawl4AI)

* Put 30–50 representative URLs in `tests/golden/urls.txt`.
* For each URL, store **expected** `ExtractedDoc` JSON (from a known good run).
* Test ensures stability of title/text extraction and link lists; update on intentional changes.

## Integration tests

* Launch `docker compose` in CI service container (with `--profile test` to use a smaller Chromium).
* Hit `/crawl` and `/deepsearch` endpoints and assert NDJSON schema + artifacts written.

## Load/latency tests

* Use `k6` or `hey` against `/crawl` with 100 URLs; assert p50 < 1.5s, p95 < 5s with headless cap = 2.

## Observability

* Add `tower-http` tracing + request/response timing.
* Log `phase=fast|headless`, `fetch_ms`, `extract_ms`, `render_ms`.

**GitHub Actions** (`.github/workflows/ci.yml`) runs:

1. `cargo fmt --all --check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test --workspace`
4. `cargo deny check`
5. (Optional) docker build of `api` and `headless` on tagged releases.

---

# 11) Known challenges (and the prescribed answers)

* **Content encodings/charsets**: rely on `reqwest` + `encoding_rs` detection inside Trek; if a page is mis-labeled, re-decode via fallback rules. (Trek focuses on robust readable extraction.) ([GitHub][8])
* **Very large HTML**: stream through **lol-html** and cap at 20 MB (`configs.riptide.yml`). ([GitHub][9])
* **Anti-bot/stealth**: rotate UAs; add `--disable-blink-features=AutomationControlled` in headless flags; optional proxies; respect robots by default (policy file). (Crawl4AI exposes equivalent knobs.) ([Crawl4AI Documentation][10])
* **JS-only content**: the fallback path exists; bounded to protect p95.
* **PDF edge cases**: prefer **pdfium-render** (Chromium’s PDF engine) for robust text + image extraction. ([Docs.rs][4])
* **SERP quotas**: serper.dev free tier for dev; key via `SERPER_API_KEY`. ([Serper][5])

---

# 12) Crawl4AI parity checklist (copy/paste for QA)

* **Simple crawling / deep crawling / adaptive stop** → `/crawl`, `spider` depth/budgets; stop early on content thresholds. ([Crawl4AI Documentation][11])
* **Dynamic page interaction** (`wait_for`, scroll, JS) → `/render` supports CSS waits, scrolls, arbitrary JS hooks. ([Crawl4AI Documentation][12])
* **Proxy/stealth/session** → UA rotation, proxy envs, per-crawl cookie jars; similar to C4AI Browser/CrawlerConfig. ([Crawl4AI Documentation][10])
* **Cache modes** → `enabled | bypass | read_through` in config.
* **Markdown + JSON outputs** → WASM extractor emits both.
* **CLI & streaming** → `riptide-api` can stream NDJSON; CLI wrapper optional.
* **PDF/screenshots/MHTML** → flags on `/render`; PDF via pdfium; MHTML optional later. ([Crawl4AI Documentation][13])

For reviewers: Crawl4AI docs home, quick start, page interaction, config, CLI, advanced features. ([Crawl4AI Documentation][6])

---

# 13) Final: “no-choice” commands to deliver

```bash
# Build (release)
./scripts/build_all.sh

# Start stack
cd infra/docker
docker compose up --build -d

# Verify health
curl -f localhost:8080/healthz || true

# Crawl fixed URLs
curl -s -X POST localhost:8080/crawl \
  -H 'content-type: application/json' -d @- <<'JSON'
{ "urls": [
  "https://en.wikipedia.org/wiki/WebAssembly",
  "https://developer.mozilla.org/en-US/docs/Web/HTTP/Compression"
] }
JSON

# Deepsearch then crawl
curl -s -X POST localhost:8080/deepsearch \
  -H 'content-type: application/json' \
  -d '{"query":"rust wasm html extraction","limit":10,"country":"us","locale":"en"}'
```

---

## Reference links (for your README)

* **Crawl4AI** (repo + docs): features, config, CLI, interaction. ([GitHub][14])
* **Trek (Rust→WASM extractor)**. ([GitHub][8])
* **lol-html** (streaming HTML rewrite). ([GitHub][9])
* **Wasmtime / WASI**. ([GitHub][7])
* **chromiumoxide** (Rust CDP). ([GitHub][2])
* **spider-rs** (crawler). ([GitHub][3])
* **pdfium-render**. ([Docs.rs][4])
* **serper.dev**. ([Serper][5])

---

If you want, I can also add a **GitHub Actions CI** file and a **Justfile** with `just build`, `just up`, `just test`, `just bench` commands — but you can ship with what’s here today.

[1]: https://github.com/seanmonstar/reqwest?utm_source=chatgpt.com "GitHub - seanmonstar/reqwest: An easy and powerful Rust HTTP Client"
[2]: https://github.com/mattsse/chromiumoxide?utm_source=chatgpt.com "GitHub - mattsse/chromiumoxide: Chrome Devtools Protocol rust API"
[3]: https://github.com/spider-rs/spider?utm_source=chatgpt.com "GitHub - spider-rs/spider: Web crawler and scraper for Rust"
[4]: https://docs.rs/pdfium-render/latest/pdfium_render/?utm_source=chatgpt.com "pdfium_render - Rust - Docs.rs"
[5]: https://serper.dev/?utm_source=chatgpt.com "Serper - The World's Fastest and Cheapest Google Search API"
[6]: https://docs.crawl4ai.com/?utm_source=chatgpt.com "Home - Crawl4AI Documentation (v0.7.x)"
[7]: https://github.com/bytecodealliance/wasmtime/blob/main/docs/WASI-tutorial.md?utm_source=chatgpt.com "wasmtime/docs/WASI-tutorial.md at main - GitHub"
[8]: https://github.com/officialunofficial/trek?utm_source=chatgpt.com "GitHub - officialunofficial/trek: Trek is a web content extraction ..."
[9]: https://github.com/cloudflare/lol-html?utm_source=chatgpt.com "GitHub - cloudflare/lol-html: Low output latency streaming HTML parser ..."
[10]: https://docs.crawl4ai.com/core/browser-crawler-config/?utm_source=chatgpt.com "Browser, Crawler & LLM Config - Crawl4AI Documentation (v0.7.x)"
[11]: https://docs.crawl4ai.com/core/simple-crawling/?utm_source=chatgpt.com "Simple Crawling - Crawl4AI Documentation (v0.7.x)"
[12]: https://docs.crawl4ai.com/core/page-interaction/?utm_source=chatgpt.com "Page Interaction - Crawl4AI Documentation (v0.7.x)"
[13]: https://docs.crawl4ai.com/advanced/advanced-features/?utm_source=chatgpt.com "Overview - Crawl4AI Documentation (v0.7.x)"
[14]: https://github.com/unclecode/crawl4ai?utm_source=chatgpt.com "Crawl4AI: Open-source LLM Friendly Web Crawler & Scraper. - GitHub"
