# RipTide Crates Inventory - Comprehensive Analysis

**Generated:** 2025-11-03
**Total Crates:** 26
**Workspace Version:** 0.9.0
**Workspace Path:** /workspaces/eventmesh/crates

## Executive Summary

The RipTide workspace consists of 26 specialized crates organized in a clean layered architecture:
- **Foundation Layer:** Core types and traits (1 crate)
- **Infrastructure Layer:** Config, events, monitoring (3 crates)
- **Network Layer:** HTTP client and fetching (1 crate)
- **Extraction Layer:** HTML processing and pooling (2 crates)
- **Browser Layer:** CDP automation (3 crates)
- **Intelligence Layer:** LLM, reliability, search (3 crates)
- **Crawler Layer:** Spider/crawler engine (1 crate)
- **Processing Layer:** PDF, cache, security, persistence, performance (5 crates)
- **Service Layer:** Workers, headless, streaming (3 crates)
- **Facade Layer:** Simplified API (1 crate)
- **Application Layer:** API server and CLI (2 crates)
- **Testing Layer:** Test utilities (1 crate)

## Complete Crates Inventory Table

| Crate | Path | Purpose | Key Exports | External Dependencies | Internal Dependencies |
|-------|------|---------|-------------|----------------------|----------------------|
| **riptide-types** | `/crates/riptide-types` | Core shared types and traits - foundational data structures | `RiptideError`, `Result`, `Browser`/`Extractor`/`Scraper` traits, `ExtractionRequest`, `CircuitBreaker`, `HtmlParser` trait, `ConditionalRequest/Response` | serde, tokio, async-trait, url, chrono, uuid, sha2 | *(none - foundation)* |
| **riptide-config** | `/crates/riptide-config` | Configuration management and validation framework | `ApiConfig`, `SpiderConfig`, `ConfigBuilder` trait, `ValidationConfig`, `EnvConfigLoader`, `load_from_env()` | serde, regex, url, once_cell | riptide-types |
| **riptide-events** | `/crates/riptide-events` | Event bus and event-driven architecture | `EventBus`, Event types, `EventEmitter`, `EventSubscriber` | tokio, futures, opentelemetry | riptide-types, riptide-monitoring |
| **riptide-monitoring** | `/crates/riptide-monitoring` | Monitoring, telemetry, metrics, health checks | `TelemetrySystem`, `MetricsCollector`, `AlertManager`, `HealthChecker`, Prometheus integration | opentelemetry, sysinfo, psutil, hdrhistogram, prometheus (opt) | riptide-types |
| **riptide-fetch** | `/crates/riptide-fetch` | HTTP/network layer with circuit breakers, rate limiting | `FetchEngine`, `ReliableHttpClient`, `RateLimiter`, `TelemetrySystem`, `RobotsManager`, `http_client()` | reqwest, http, hyper, opentelemetry, dashmap | riptide-types, riptide-config |
| **riptide-extraction** | `/crates/riptide-extraction` | HTML processing: CSS, regex, DOM, chunking, tables | `CssExtractor`, `RegexExtractor`, `DomProcessor`, `TextChunker`, `TableExtractor`, `NativeHtmlParser`, WASM extractor (opt) | scraper, lol_html, regex, tiktoken-rs, dashmap, wasmtime (opt) | riptide-types |
| **riptide-pool** | `/crates/riptide-pool` | Resource pooling for extractors (native + WASM) | `ExtractorPool`, `PoolManager`, native/WASM pooling | tokio, uuid, scraper, wasmtime (opt) | riptide-types, riptide-events, riptide-extraction (opt) |
| **riptide-browser-abstraction** | `/crates/riptide-browser-abstraction` | Browser abstraction layer and CDP traits | `BrowserTrait`, `PageTrait`, CDP protocol abstractions | spider_chrome, spider_chromiumoxide_cdp, async-trait | riptide-types |
| **riptide-stealth** | `/crates/riptide-stealth` | Anti-detection for browser automation | `StealthConfig`, `FingerprintRandomizer`, evasion techniques | serde, rand, dashmap | *(none)* |
| **riptide-browser** | `/crates/riptide-browser` | Unified browser automation: pools, CDP, launcher | `BrowserPool`, `BrowserLauncher`, CDP connection pooling | spider_chrome, spider_chromiumoxide_cdp, tokio, uuid | riptide-browser-abstraction, riptide-stealth |
| **riptide-reliability** | `/crates/riptide-reliability` | Reliability patterns: circuit breakers, retry, fault tolerance | `CircuitBreaker` (native), `RetryPolicy`, fault tolerance | tokio, reqwest, uuid, chrono | riptide-types, riptide-fetch, riptide-events (opt), riptide-monitoring (opt) |
| **riptide-intelligence** | `/crates/riptide-intelligence` | LLM abstraction (OpenAI, Anthropic, Groq) | `LlmProvider` trait, provider implementations, `ProviderPool`, token counting, safety | reqwest, dashmap, tiktoken-rs, notify | riptide-reliability, riptide-types, riptide-events |
| **riptide-search** | `/crates/riptide-search` | Search provider abstraction (Google, Bing, etc.) | `SearchProvider` trait, provider implementations, `SearchResult` | reqwest, regex, url, async-trait | *(none)* |
| **riptide-spider** | `/crates/riptide-spider` | Web crawler engine with strategies and budgets | `Spider`, `SpiderConfig`, `CrawlingStrategy` (4 types), `FrontierManager`, `BudgetManager`, `SessionManager`, `QueryAwareScorer` | reqwest, robotstxt, regex, dashmap, wasmtime, xml, sysinfo | riptide-types, riptide-config, riptide-fetch |
| **riptide-pdf** | `/crates/riptide-pdf` | PDF processing and text extraction | `PdfProcessor`, `PdfExtractor`, `PdfMetadata`, text extraction | pdfium-render (opt), lopdf (opt), bytes, chrono | riptide-types |
| **riptide-cache** | `/crates/riptide-cache` | Cache management with Redis and local cache | `CacheManager`, `RedisCache`, `LocalCache`, `CacheWarmer`, conditional GET | redis, sha2, chrono, dashmap, url, dirs | riptide-types, riptide-pool, riptide-events, riptide-extraction |
| **riptide-security** | `/crates/riptide-security` | Security middleware: API keys, PII, audit, rate limiting | API key management, PII scrubbing, audit logging, budget enforcement | sha2, base64, governor, rand, uuid | riptide-types |
| **riptide-persistence** | `/crates/riptide-persistence` | Persistence with Redis/DragonflyDB, multi-tenancy | `PersistenceManager`, `MultiTenantCache`, `StateStore`, compression (LZ4/Zstd) | redis, blake3, lz4_flex (opt), zstd (opt), prometheus (opt) | *(none)* |
| **riptide-performance** | `/crates/riptide-performance` | Performance profiling, memory analysis, benchmarking | `MemoryProfiler`, `BottleneckAnalyzer`, benchmarks, cache optimizer | tokio-metrics, sysinfo, jemalloc (opt), pprof (opt), criterion (opt) | *(none)* |
| **riptide-workers** | `/crates/riptide-workers` | Background workers and job processing | `WorkerManager`, `JobQueue`, `TaskScheduler`, cron scheduling | tokio, redis, cron, dashmap, num_cpus | riptide-types, riptide-reliability, riptide-extraction, riptide-cache, riptide-pdf |
| **riptide-headless** | `/crates/riptide-headless` | Headless browser API endpoints | Headless browser HTTP API, screenshot/PDF generation, session management | axum, spider_chrome, base64, tower | riptide-browser, riptide-stealth |
| **riptide-streaming** | `/crates/riptide-streaming` | Streaming infrastructure (SSE, WebSocket, NDJSON) | `StreamingHandler`, SSE/WebSocket/NDJSON streams, `ReportGenerator` | axum, tokio-stream, async-stream, handlebars, plotters, utoipa | riptide-extraction |
| **riptide-facade** | `/crates/riptide-facade` | Simplified facade API for common workflows | `SimpleScraper`, `ScraperBuilder`, unified high-level interface | tokio, scraper, url, spider_chromiumoxide_cdp | 10 riptide crates (unified API) |
| **riptide-api** | `/crates/riptide-api` | Main HTTP API server (REST + streaming + WebSocket) | API server binary, REST handlers, streaming endpoints, session mgmt, OpenAPI | axum, tower, redis, tokio-stream, prometheus, utoipa | **ALL riptide crates** (orchestrator) |
| **riptide-cli** | `/crates/riptide-cli` | Thin CLI client (delegates to API server) | 7 commands: scrape, batch, stream, search, extract, cache, health | clap, reqwest, colored, indicatif, comfy-table, futures-util | *(none - thin client)* |
| **riptide-test-utils** | `/crates/riptide-test-utils` | Testing utilities and mock HTTP servers | `MockHttpServer`, test fixtures, test utilities | tokio, tempfile, axum (opt), tower (opt) | *(none)* |

## Dependency Analysis

### Foundation Layer (0 internal deps)
- **riptide-types**: The foundation - all other crates depend on it directly or indirectly

### Infrastructure Layer
- **riptide-config** → riptide-types
- **riptide-events** → riptide-types, riptide-monitoring
- **riptide-monitoring** → riptide-types

### Network & Communication
- **riptide-fetch** → riptide-types, riptide-config

### Extraction & Processing
- **riptide-extraction** → riptide-types
- **riptide-pool** → riptide-types, riptide-events, riptide-extraction

### Browser Automation
- **riptide-browser-abstraction** → riptide-types
- **riptide-stealth** → *(no internal deps)*
- **riptide-browser** → riptide-browser-abstraction, riptide-stealth

### Intelligence & Reliability
- **riptide-reliability** → riptide-types, riptide-fetch, riptide-events, riptide-monitoring, riptide-pool
- **riptide-intelligence** → riptide-reliability, riptide-types, riptide-events
- **riptide-search** → *(no internal deps)*

### Crawler
- **riptide-spider** → riptide-types, riptide-config, riptide-fetch

### Specialized Processing
- **riptide-pdf** → riptide-types
- **riptide-cache** → riptide-types, riptide-pool, riptide-events, riptide-extraction
- **riptide-security** → riptide-types
- **riptide-persistence** → *(no internal deps)*
- **riptide-performance** → *(no internal deps)*

### Service Layer
- **riptide-workers** → riptide-types, riptide-reliability, riptide-extraction, riptide-cache, riptide-pdf
- **riptide-headless** → riptide-browser, riptide-stealth
- **riptide-streaming** → riptide-extraction

### Application Layer
- **riptide-facade** → 10 riptide crates
- **riptide-api** → ALL riptide crates (orchestrator)
- **riptide-cli** → *(no internal deps - thin client)*

## Architecture Patterns

### Circular Dependency Resolution
The codebase successfully avoids circular dependencies using **trait abstraction**:
- **Problem:** `riptide-reliability` needed HTML parsing from `riptide-extraction`, but `riptide-extraction` depends on `riptide-reliability`
- **Solution:** `HtmlParser` trait defined in `riptide-types` (foundation), implemented by `NativeHtmlParser` in `riptide-extraction`
- **Result:** Dependency injection pattern enables clean layering

### WASM Strategy
Optional WASM features throughout the codebase:
- **Default:** Native Rust implementations (fast, no WASM runtime overhead)
- **Optional:** WASM-based extraction via `wasm-extractor` feature flag
- **Crates affected:** `riptide-extraction`, `riptide-pool`, `riptide-cache`, `riptide-api`

### Thin CLI Pattern
- **riptide-cli** has ZERO internal dependencies
- All business logic resides in **riptide-api** server
- CLI is a thin HTTP client with 7 commands
- Benefits: Smaller binary, easier updates, centralized logic

### Browser Consolidation
- All browser automation uses `spider_chrome` consistently
- No mixed browser engine dependencies
- `spider_chromiumoxide_cdp` for CDP protocol types
- Unified through `riptide-browser` abstraction

### Feature-Gated Heavy Dependencies
Optional features for heavy dependencies:
- **jemalloc:** Memory profiling (riptide-performance, riptide-api)
- **pprof:** CPU profiling (riptide-performance)
- **flamegraph:** Visualization (dev-only, not in CI - license compliance)
- **prometheus:** Metrics export (riptide-monitoring, riptide-persistence)

## Key Metrics

- **Total Crates:** 26
- **Foundation Crates:** 1 (riptide-types)
- **Orchestrator Crates:** 1 (riptide-api - depends on all)
- **Standalone Crates:** 4 (riptide-stealth, riptide-search, riptide-persistence, riptide-performance)
- **Average Internal Deps:** 2.3 per crate
- **Max Internal Deps:** 26 (riptide-api - orchestrator)
- **Min Internal Deps:** 0 (4 crates)

## Common External Dependencies

Most frequently used external crates across workspace:
1. **tokio** - 24 crates (async runtime)
2. **serde** - 24 crates (serialization)
3. **anyhow** - 18 crates (error handling)
4. **thiserror** - 16 crates (error types)
5. **async-trait** - 14 crates (async traits)
6. **tracing** - 14 crates (logging)
7. **chrono** - 12 crates (time handling)
8. **uuid** - 12 crates (identifiers)
9. **reqwest** - 11 crates (HTTP client)
10. **dashmap** - 10 crates (concurrent maps)

## Next Steps for Analysis

1. **Dependency Graph Visualization:** Create visual diagram of crate relationships
2. **API Surface Analysis:** Document public API of each crate
3. **Feature Flag Matrix:** Map all feature flags and their interactions
4. **Performance Characteristics:** Document performance profiles of each crate
5. **Migration Paths:** Identify upgrade paths and deprecation strategies

---

**Raw Data:** See `/workspaces/eventmesh/docs/analysis/crates_inventory_raw.json` for complete structured data.
