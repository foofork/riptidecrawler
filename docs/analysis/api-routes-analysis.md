# Riptide API Routes Analysis - Complete Surface Mapping for v1

**Generated:** 2025-11-04
**Purpose:** Map existing API surface to identify refactorable vs recreate candidates for v1 implementation

---

## Executive Summary

The Riptide API currently exposes **90+ endpoints** across 15 route categories with a **mixed architecture** combining:
- **Facade pattern** for simplified service access (extraction, spider, browser, search, scraper)
- **Direct pipeline orchestrators** for crawl operations (standard + enhanced)
- **Middleware stack** with 6 layers (auth, rate limiting, validation, timeout, CORS, compression)
- **Session management** with persistent browser contexts
- **Streaming support** via NDJSON for real-time progress

### Key Architectural Patterns

**Service Layer Calling Patterns:**
1. **Facade-First** (Recommended for v1): `handlers → facades → services`
   - Extract, Spider, Search, Browser, Scraper handlers use this pattern
   - Clean abstraction, simplified error handling, reusable across APIs

2. **Direct Pipeline** (Legacy, needs refactoring):
   - Crawl handlers directly instantiate `PipelineOrchestrator` or `EnhancedPipelineOrchestrator`
   - Tight coupling, harder to test, duplication between standard/enhanced

3. **Hybrid** (Needs consolidation):
   - DeepSearch uses both search facade AND pipeline orchestrator
   - Profiles use direct cache/config access
   - Resource handlers directly access state managers

---

## Complete Route Inventory

### 1. Health & Metrics Routes

| Method | Path | Handler | Facade/Service Used | Reusable? |
|--------|------|---------|---------------------|-----------|
| GET | `/healthz` | `handlers::health` | Direct state health check | ✅ Yes (standardize) |
| GET | `/api/health/detailed` | `handlers::health_detailed` | Health checker, all services | ✅ Yes (enhance) |
| GET | `/health/:component` | `health::component_health_check` | Component-specific health | ✅ Yes |
| GET | `/health/metrics` | `health::health_metrics_check` | Metrics collector | ✅ Yes |
| GET | `/metrics` | `handlers::metrics` | Prometheus exporter | ✅ Yes |
| GET | `/api/v1/metrics` | `handlers::metrics` | (v1 alias) | ✅ Yes |

**DTOs:**
- `HealthResponse` - standardized health format
- `DependencyStatus` - per-service health
- `ServiceHealth` - individual service status
- `SystemMetrics` - performance counters

**Calling Pattern:** Direct AppState methods
```rust
state.health_checker.check_all_dependencies().await
state.metrics.export_prometheus()
```

---

### 2. Core Crawl Routes

| Method | Path | Handler | Facade/Service Used | Reusable? |
|--------|------|---------|---------------------|-----------|
| POST | `/crawl` | `handlers::crawl` | PipelineOrchestrator (direct) | ⚠️ Refactor to facade |
| POST | `/api/v1/crawl` | `handlers::crawl` | (v1 alias) | ⚠️ Refactor to facade |
| POST | `/crawl/stream` | `handlers::crawl_stream` | PipelineOrchestrator + streaming | ⚠️ Refactor |
| POST | `/api/v1/crawl/stream` | `handlers::crawl_stream` | (v1 alias) | ⚠️ Refactor |

**DTOs:**
- `CrawlBody` - batch URL input
- `CrawlResponse` - results with statistics
- `CrawlResult` - single URL result
- `CrawlStatistics` - aggregated stats
- `GateDecisionBreakdown` - engine routing stats

**Calling Pattern:** Direct instantiation (needs refactoring)
```rust
// Current (tightly coupled):
let pipeline = PipelineOrchestrator::new(state.clone(), options);
let results = pipeline.execute_batch(&urls).await;

// Recommended for v1:
let results = state.extraction_facade.batch_crawl(&urls, options).await;
```

**Issues:**
- Dual pipeline paths (standard vs enhanced) - conditional logic in handler
- No unified facade for crawl operations
- Streaming handled separately with code duplication

---

### 3. Extraction Routes (v1.1 Feature)

| Method | Path | Handler | Facade/Service Used | Reusable? |
|--------|------|---------|---------------------|-----------|
| POST | `/api/v1/extract` | `handlers::extract` | `ExtractionFacade` | ✅ YES (gold standard) |
| POST | `/extract` | `handlers::extract` | (backward compat alias) | ✅ YES |

**DTOs:**
- `ExtractRequest` - single URL + mode/options
- `ExtractResponse` - extracted content + metadata
- `ContentMetadata` - author, publish date, word count
- `ParserMetadata` - observability data (parser used, confidence, fallbacks)

**Calling Pattern:** Facade-first (BEST PRACTICE)
```rust
// Clean facade API:
state.extraction_facade.extract_with_strategy(&html, &url, strategy).await
state.extraction_facade.extract_with_fallback(&html, &url, strategies).await
```

**Why This Works:**
- Single responsibility: handler validates, facade orchestrates
- Strategy pattern: css, wasm, multi, auto
- Fallback chains for reliability
- Comprehensive metadata for debugging

---

### 4. Search Routes (v1.1 Feature)

| Method | Path | Handler | Facade/Service Used | Reusable? |
|--------|------|---------|---------------------|-----------|
| GET | `/api/v1/search` | `handlers::search` | `SearchFacade` | ✅ YES |
| GET | `/search` | `handlers::search` | (backward compat) | ✅ YES |
| POST | `/deepsearch` | `handlers::deepsearch` | SearchFacade + PipelineOrchestrator | ⚠️ Hybrid |
| POST | `/deepsearch/stream` | `handlers::deepsearch_stream` | SearchFacade + streaming | ⚠️ Hybrid |
| POST | `/api/v1/deepsearch/stream` | `handlers::deepsearch_stream` | (v1 alias) | ⚠️ Hybrid |

**DTOs:**
- `DeepSearchBody` - query + crawl options
- `DeepSearchResponse` - search + crawled content
- `SearchResult` - individual result with extracted doc

**Calling Pattern:** Facade + pipeline (needs consolidation)
```rust
// Search step (good):
let search_results = state.search_facade.search(&query).await?;

// Crawl step (should use facade):
let pipeline = PipelineOrchestrator::new(state.clone(), options);
let crawled = pipeline.execute_batch(&urls).await;
```

**Recommendation for v1:**
- Add `search_and_crawl()` method to ExtractionFacade
- Eliminate direct pipeline instantiation
- Unify streaming implementation

---

### 5. Spider Routes (Deep Crawling)

| Method | Path | Handler | Facade/Service Used | Reusable? |
|--------|------|---------|---------------------|-----------|
| POST | `/spider/crawl` | `handlers::spider::spider_crawl` | `SpiderFacade` | ✅ YES |
| POST | `/spider/status` | `handlers::spider::spider_status` | `SpiderFacade` | ✅ YES |
| POST | `/spider/control` | `handlers::spider::spider_control` | `SpiderFacade` | ✅ YES |

**DTOs:**
- `SpiderCrawlBody` - seed URLs + depth/page limits
- `SpiderCrawlResponseStats` - statistics mode
- `SpiderCrawlResponseUrls` - with URL list
- `SpiderResultPages` - full page objects (Phase 2)
- `ResultMode` enum - stats/urls/pages/stream/store
- `CrawledPage` - comprehensive page data
- `FieldFilter` - selective field inclusion/exclusion

**Calling Pattern:** Facade-first (BEST PRACTICE)
```rust
// Clean spider facade API:
let summary = state.spider_facade.crawl(seed_urls).await?;
let state = state.spider_facade.get_state().await?;
let metrics = state.spider_facade.get_metrics().await?;
```

**Advanced Features:**
- Query-based result modes (stats, urls, pages)
- Field filtering (include/exclude parameters)
- Content truncation for memory management
- Adaptive stopping based on content analysis

---

### 6. PDF Processing Routes

| Method | Path | Handler | Facade/Service Used | Reusable? |
|--------|------|---------|---------------------|-----------|
| POST | `/pdf/process` | `pdf::process_pdf` | Direct PDF integration | ⚠️ Needs facade |
| POST | `/pdf/upload` | `pdf::upload_pdf` | Direct PDF integration | ⚠️ Needs facade |
| POST | `/pdf/process-stream` | `pdf::process_pdf_stream` | PDF + streaming | ⚠️ Needs facade |
| GET | `/pdf/healthz` | `pdf_health_check` | Direct capabilities check | ✅ Yes |

**Calling Pattern:** Direct service (should use facade)
```rust
// Current:
let integration = create_pdf_integration_for_pipeline();
let result = integration.process_pdf_bytes(&bytes).await;

// Recommended:
// Add PDFFacade to riptide-facade
let result = state.pdf_facade.process(&bytes, options).await;
```

---

### 7. Stealth & Browser Routes

| Method | Path | Handler | Facade/Service Used | Reusable? |
|--------|------|---------|---------------------|-----------|
| POST | `/stealth/configure` | `stealth::configure_stealth` | Direct StealthController | ⚠️ Needs facade |
| POST | `/stealth/test` | `stealth::test_stealth` | Direct StealthController | ⚠️ Needs facade |
| GET | `/stealth/capabilities` | `stealth::get_stealth_capabilities` | Direct capabilities | ✅ Yes |
| GET | `/stealth/healthz` | `stealth_health_check` | Direct health | ✅ Yes |
| POST | `/api/v1/browser/session` | `browser::create_browser_session` | `BrowserFacade` | ✅ YES |
| POST | `/api/v1/browser/action` | `browser::execute_browser_action` | `BrowserFacade` | ✅ YES |
| GET | `/api/v1/browser/pool/status` | `browser::get_browser_pool_status` | `BrowserFacade` | ✅ YES |
| DELETE | `/api/v1/browser/session/:id` | `browser::close_browser_session` | `BrowserFacade` | ✅ YES |

**Browser Facade Pattern (BEST PRACTICE):**
```rust
state.browser_facade.create_session(options).await
state.browser_facade.execute_action(session_id, action).await
state.browser_facade.get_pool_status().await
```

---

### 8. Session Management Routes

| Method | Path | Handler | Facade/Service Used | Reusable? |
|--------|------|---------|---------------------|-----------|
| POST | `/sessions` | `sessions::create_session` | `SessionManager` | ✅ Yes |
| GET | `/sessions` | `sessions::list_sessions` | `SessionManager` | ✅ Yes |
| GET | `/sessions/stats` | `sessions::get_session_stats` | `SessionManager` | ✅ Yes |
| POST | `/sessions/cleanup` | `sessions::cleanup_expired_sessions` | `SessionManager` | ✅ Yes |
| GET | `/sessions/:session_id` | `sessions::get_session_info` | `SessionManager` | ✅ Yes |
| DELETE | `/sessions/:session_id` | `sessions::delete_session` | `SessionManager` | ✅ Yes |
| POST | `/sessions/:session_id/extend` | `sessions::extend_session` | `SessionManager` | ✅ Yes |
| POST | `/sessions/:session_id/cookies` | `sessions::set_cookie` | `SessionManager` | ✅ Yes |
| DELETE | `/sessions/:session_id/cookies` | `sessions::clear_cookies` | `SessionManager` | ✅ Yes |
| GET | `/sessions/:session_id/cookies/:domain` | `sessions::get_cookies_for_domain` | `SessionManager` | ✅ Yes |
| GET | `/sessions/:session_id/cookies/:domain/:name` | `sessions::get_cookie` | `SessionManager` | ✅ Yes |
| DELETE | `/sessions/:session_id/cookies/:domain/:name` | `sessions::delete_cookie` | `SessionManager` | ✅ Yes |

**Session Implementation:**
- Persistent browser contexts with Redis backing
- Cookie management per domain
- TTL-based expiration with cleanup task
- Session extension for long-running operations
- Used via `SessionLayer` middleware for `/render` endpoint

---

### 9. Table Extraction Routes

| Method | Path | Handler | Facade/Service Used | Reusable? |
|--------|------|---------|---------------------|-----------|
| POST | `/api/v1/tables/extract` | `tables::extract_tables` | Direct table extractor | ⚠️ Needs facade |
| GET | `/api/v1/tables/:id/export` | `tables::export_table` | Direct table storage | ⚠️ Needs facade |

---

### 10. LLM Provider Management Routes

| Method | Path | Handler | Facade/Service Used | Reusable? |
|--------|------|---------|---------------------|-----------|
| GET | `/api/v1/llm/providers` | `llm::list_providers` | Direct LLM config | ⚠️ Needs facade |
| GET | `/api/v1/llm/providers/current` | `llm::get_current_provider_info` | Direct LLM config | ⚠️ Needs facade |
| POST | `/api/v1/llm/providers/switch` | `llm::switch_provider` | Direct LLM config | ⚠️ Needs facade |
| GET | `/api/v1/llm/config` | `llm::get_config` | Direct LLM config | ⚠️ Needs facade |
| POST | `/api/v1/llm/config` | `llm::update_config` | Direct LLM config | ⚠️ Needs facade |

---

### 11. Content Chunking Routes

| Method | Path | Handler | Facade/Service Used | Reusable? |
|--------|------|---------|---------------------|-----------|
| POST | `/api/v1/content/chunk` | `chunking::chunk_content` | Direct chunking service | ⚠️ Needs facade |

---

### 12. Engine Selection Routes (Phase 10)

| Method | Path | Handler | Facade/Service Used | Reusable? |
|--------|------|---------|---------------------|-----------|
| POST | `/engine/analyze` | `engine_selection::analyze_engine` | Direct engine analyzer | ⚠️ Needs facade |
| POST | `/engine/decide` | `engine_selection::decide_engine` | Direct engine decider | ⚠️ Needs facade |
| GET | `/engine/stats` | `engine_selection::get_engine_stats` | Direct stats collector | ✅ Yes |
| PUT | `/engine/probe-first` | `engine_selection::toggle_probe_first` | Direct config update | ✅ Yes |

---

### 13. Domain Profile Routes (Phase 10.4 Warm-Start Caching)

| Method | Path | Handler | Facade/Service Used | Reusable? |
|--------|------|---------|---------------------|-----------|
| POST | `/api/v1/profiles` | `profiles::create_profile` | Direct profile manager | ⚠️ Needs facade |
| GET | `/api/v1/profiles` | `profiles::list_profiles` | Direct profile manager | ⚠️ Needs facade |
| GET | `/api/v1/profiles/:domain` | `profiles::get_profile` | Direct profile manager | ⚠️ Needs facade |
| PUT | `/api/v1/profiles/:domain` | `profiles::update_profile` | Direct profile manager | ⚠️ Needs facade |
| DELETE | `/api/v1/profiles/:domain` | `profiles::delete_profile` | Direct profile manager | ⚠️ Needs facade |
| GET | `/api/v1/profiles/:domain/stats` | `profiles::get_profile_stats` | Direct stats collector | ✅ Yes |
| GET | `/api/v1/profiles/metrics` | `profiles::get_caching_metrics` | Direct metrics | ✅ Yes |
| POST | `/api/v1/profiles/batch` | `profiles::batch_create_profiles` | Direct batch processor | ⚠️ Needs facade |
| GET | `/api/v1/profiles/search` | `profiles::search_profiles` | Direct search | ⚠️ Needs facade |
| POST | `/api/v1/profiles/:domain/warm` | `profiles::warm_cache` | Direct cache warmer | ⚠️ Needs facade |
| DELETE | `/api/v1/profiles/clear` | `profiles::clear_all_caches` | Direct cache manager | ⚠️ Needs facade |

---

### 14. Worker Management Routes

| Method | Path | Handler | Facade/Service Used | Reusable? |
|--------|------|---------|---------------------|-----------|
| POST | `/workers/jobs` | `workers::submit_job` | `WorkerService` | ✅ Yes |
| GET | `/workers/jobs` | `workers::list_jobs` | `WorkerService` | ✅ Yes |
| GET | `/workers/jobs/:job_id` | `workers::get_job_status` | `WorkerService` | ✅ Yes |
| GET | `/workers/jobs/:job_id/result` | `workers::get_job_result` | `WorkerService` | ✅ Yes |
| GET | `/workers/stats/queue` | `workers::get_queue_stats` | `WorkerService` | ✅ Yes |
| GET | `/workers/stats/workers` | `workers::get_worker_stats` | `WorkerService` | ✅ Yes |
| GET | `/workers/metrics` | `workers::get_worker_metrics` | `WorkerService` | ✅ Yes |
| POST | `/workers/schedule` | `workers::create_scheduled_job` | `WorkerService` | ✅ Yes |
| GET | `/workers/schedule` | `workers::list_scheduled_jobs` | `WorkerService` | ✅ Yes |
| DELETE | `/workers/schedule/:job_id` | `workers::delete_scheduled_job` | `WorkerService` | ✅ Yes |

---

### 15. Resource Monitoring Routes

| Method | Path | Handler | Facade/Service Used | Reusable? |
|--------|------|---------|---------------------|-----------|
| GET | `/resources/status` | `resources::get_resource_status` | `ResourceManager` | ✅ Yes |
| GET | `/resources/browser-pool` | `resources::get_browser_pool_status` | `ResourceManager` | ✅ Yes |
| GET | `/resources/rate-limiter` | `resources::get_rate_limiter_status` | `ResourceManager` | ✅ Yes |
| GET | `/resources/memory` | `resources::get_memory_status` | `ResourceManager` | ✅ Yes |
| GET | `/resources/performance` | `resources::get_performance_status` | `PerformanceManager` | ✅ Yes |
| GET | `/resources/pdf/semaphore` | `resources::get_pdf_semaphore_status` | `ResourceManager` | ✅ Yes |
| GET | `/api/v1/memory/profile` | `memory::memory_profile_handler` | `PerformanceManager` | ✅ Yes |
| GET | `/api/v1/memory/leaks` | `resources::get_memory_leaks` | `PerformanceManager` | ✅ Yes |
| GET | `/fetch/metrics` | `fetch::get_fetch_metrics` | `FetchEngine` | ✅ Yes |

---

### 16. Monitoring & Performance Routes

| Method | Path | Handler | Facade/Service Used | Reusable? |
|--------|------|---------|---------------------|-----------|
| GET | `/monitoring/health-score` | `monitoring::get_health_score` | `MonitoringSystem` | ✅ Yes |
| GET | `/monitoring/performance-report` | `monitoring::get_performance_report` | `MonitoringSystem` | ✅ Yes |
| GET | `/monitoring/metrics/current` | `monitoring::get_current_metrics` | `MetricsCollector` | ✅ Yes |
| GET | `/monitoring/alerts/rules` | `monitoring::get_alert_rules` | `AlertManager` | ✅ Yes |
| GET | `/monitoring/alerts/active` | `monitoring::get_active_alerts` | `AlertManager` | ✅ Yes |
| GET | `/api/profiling/memory` | `profiling::get_memory_profile` | `PerformanceManager` | ✅ Yes |
| GET | `/api/profiling/cpu` | `profiling::get_cpu_profile` | `PerformanceManager` | ✅ Yes |
| GET | `/api/profiling/bottlenecks` | `profiling::get_bottleneck_analysis` | `PerformanceManager` | ✅ Yes |
| GET | `/api/profiling/allocations` | `profiling::get_allocation_metrics` | `PerformanceManager` | ✅ Yes |
| POST | `/api/profiling/leak-detection` | `profiling::trigger_leak_detection` | `PerformanceManager` | ✅ Yes |
| POST | `/api/profiling/snapshot` | `profiling::trigger_heap_snapshot` | `PerformanceManager` | ✅ Yes |
| GET | `/monitoring/profiling/memory` | `monitoring::get_memory_metrics` | (deprecated) | ⚠️ Remove |
| GET | `/monitoring/profiling/leaks` | `monitoring::get_leak_analysis` | (deprecated) | ⚠️ Remove |
| GET | `/monitoring/profiling/allocations` | `monitoring::get_allocation_metrics` | (deprecated) | ⚠️ Remove |
| GET | `/monitoring/wasm-instances` | `monitoring::get_wasm_health` | `MonitoringSystem` | ✅ Yes |
| GET | `/api/resources/status` | `monitoring::get_resource_status` | `ResourceManager` | ✅ Yes |
| GET | `/pipeline/phases` | `handlers::get_pipeline_phases` | Direct pipeline stats | ✅ Yes |

---

### 17. Telemetry & Tracing Routes (TELEM-005)

| Method | Path | Handler | Facade/Service Used | Reusable? |
|--------|------|---------|---------------------|-----------|
| GET | `/api/telemetry/status` | `telemetry::get_telemetry_status` | `TelemetrySystem` | ✅ Yes |
| GET | `/api/telemetry/traces` | `telemetry::list_traces` | `TraceBackend` | ✅ Yes |
| GET | `/api/telemetry/traces/:trace_id` | `telemetry::get_trace_tree` | `TraceBackend` | ✅ Yes |

---

### 18. Admin Routes (Feature-Gated: `persistence`)

| Method | Path | Handler | Facade/Service Used | Reusable? |
|--------|------|---------|---------------------|-----------|
| POST | `/admin/tenants` | `admin::create_tenant` | `PersistenceAdapter` | ⚠️ Future |
| GET | `/admin/tenants` | `admin::list_tenants` | `PersistenceAdapter` | ⚠️ Future |
| GET | `/admin/tenants/:id` | `admin::get_tenant` | `PersistenceAdapter` | ⚠️ Future |
| PUT | `/admin/tenants/:id` | `admin::update_tenant` | `PersistenceAdapter` | ⚠️ Future |
| DELETE | `/admin/tenants/:id` | `admin::delete_tenant` | `PersistenceAdapter` | ⚠️ Future |
| GET | `/admin/tenants/:id/usage` | `admin::get_tenant_usage` | `PersistenceAdapter` | ⚠️ Future |
| GET | `/admin/tenants/:id/billing` | `admin::get_tenant_billing` | `PersistenceAdapter` | ⚠️ Future |
| POST | `/admin/cache/warm` | `admin::warm_cache` | `CacheManager` | ⚠️ Future |
| POST | `/admin/cache/invalidate` | `admin::invalidate_cache` | `CacheManager` | ⚠️ Future |
| GET | `/admin/cache/stats` | `admin::get_cache_stats` | `CacheManager` | ⚠️ Future |
| POST | `/admin/state/reload` | `admin::reload_state` | State management | ⚠️ Future |
| POST | `/admin/state/checkpoint` | `admin::create_checkpoint` | State management | ⚠️ Future |
| POST | `/admin/state/restore/:id` | `admin::restore_checkpoint` | State management | ⚠️ Future |

---

### 19. Strategy Routes (Advanced Extraction)

| Method | Path | Handler | Facade/Service Used | Reusable? |
|--------|------|---------|---------------------|-----------|
| POST | `/strategies/crawl` | `strategies::strategies_crawl` | Direct strategy pipeline | ⚠️ Needs facade |
| GET | `/strategies/info` | `strategies::get_strategies_info` | Direct strategy metadata | ✅ Yes |

---

### 20. Render Route (Session-Aware)

| Method | Path | Handler | Facade/Service Used | Reusable? |
|--------|------|---------|---------------------|-----------|
| POST | `/render` | `handlers::render` | `BrowserFacade` + SessionLayer | ✅ Yes |
| POST | `/api/v1/render` | `handlers::render` | (v1 alias) | ✅ Yes |

**Special Implementation:**
- Uses `SessionLayer` middleware for persistent browser context
- Separate router merge to apply middleware before state
- Supports authenticated/stateful rendering

---

## Middleware Stack Analysis

**Order of Execution (outermost → innermost):**

1. **CompressionLayer** - Response compression (gzip/brotli)
2. **CorsLayer** - Permissive CORS for development
3. **TimeoutLayer** - 30s global timeout
4. **TraceLayer** - HTTP request tracing
5. **Prometheus metrics layer** - Request/response tracking
6. **PayloadLimitLayer** - 50MB max payload (PDF/HTML support)
7. **rate_limit_middleware** - Per-host rate limiting + concurrency control
8. **auth_middleware** - API key validation (optional, env-driven)
9. **request_validation_middleware** - Reject malformed payloads (400/405)
10. **SessionLayer** - (Only on `/render` routes) Session context injection

**Middleware Configuration:**
```rust
// From state::ApiConfig
pub struct ApiConfig {
    pub rate_limiting: RateLimitConfig,
    pub headless: HeadlessConfig,
    pub pdf: PdfConfig,
}

pub struct RateLimitConfig {
    pub requests_per_second_per_host: u32,  // Default: 10
    pub max_concurrent_per_host: usize,     // Default: 5
}
```

**Auth Configuration:**
```rust
// From middleware::AuthConfig
impl AuthConfig {
    pub fn requires_auth(&self) -> bool {
        // Based on RIPTIDE_API_KEYS env var
        !self.api_keys.is_empty()
    }
}
```

---

## DTO Definitions & Conversion Logic

### Core DTOs (`/crates/riptide-api/src/dto.rs`)

**Spider DTOs:**
```rust
pub enum ResultMode {
    Stats,   // Statistics only (default)
    Urls,    // Statistics + discovered URLs
    Pages,   // Full page objects with content
    Stream,  // NDJSON streaming (not yet implemented)
    Store,   // Async retrieval (not yet implemented)
}

pub struct SpiderResultStats {
    pub pages_crawled: u64,
    pub pages_failed: u64,
    pub duration_seconds: f64,
    pub stop_reason: String,
    pub domains: Vec<String>,
}

pub struct SpiderResultUrls {
    // Same as Stats +
    pub discovered_urls: Vec<String>,
}

pub struct CrawledPage {
    pub url: String,
    pub depth: u32,
    pub status_code: u16,
    pub title: Option<String>,
    pub content: Option<String>,
    pub markdown: Option<String>,
    pub links: Vec<String>,
    pub truncated: Option<bool>,
    // Metadata fields...
}

pub struct FieldFilter {
    fields: Vec<String>,
}
impl FieldFilter {
    pub fn parse(s: &str) -> Self // Comma-separated
    pub fn has_field(&self, field: &str) -> bool
}
```

**Conversion:**
```rust
impl From<&riptide_facade::facades::spider::CrawlSummary> for SpiderResultStats {
    fn from(summary: &CrawlSummary) -> Self {
        Self {
            pages_crawled: summary.pages_crawled,
            pages_failed: summary.pages_failed,
            duration_seconds: summary.duration_secs,
            stop_reason: summary.stop_reason.clone(),
            domains: summary.domains.clone(),
        }
    }
}
```

### Models DTOs (`/crates/riptide-api/src/models.rs`)

**Crawl DTOs:**
```rust
pub struct CrawlBody {
    pub urls: Vec<String>,
    pub options: Option<CrawlOptions>,
}

pub struct CrawlResult {
    pub url: String,
    pub status: u16,
    pub from_cache: bool,
    pub gate_decision: String,
    pub quality_score: f32,
    pub processing_time_ms: u64,
    pub document: Option<ExtractedDoc>,
    pub error: Option<ErrorInfo>,
    pub cache_key: String,
}

pub struct CrawlResponse {
    pub total_urls: usize,
    pub successful: usize,
    pub failed: usize,
    pub from_cache: usize,
    pub results: Vec<CrawlResult>,
    pub statistics: CrawlStatistics,
}

pub struct GateDecisionBreakdown {
    pub raw: usize,
    pub probes_first: usize,
    pub headless: usize,
    pub cached: usize,
}
```

**Health DTOs:**
```rust
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: String,
    pub uptime: u64,
    pub dependencies: DependencyStatus,
    pub metrics: Option<SystemMetrics>,
}

pub struct DependencyStatus {
    pub redis: ServiceHealth,
    pub extractor: ServiceHealth,
    pub http_client: ServiceHealth,
    pub headless_service: Option<ServiceHealth>,
    pub spider_engine: Option<ServiceHealth>,
    pub worker_service: Option<ServiceHealth>,
}

pub struct SystemMetrics {
    pub memory_usage_bytes: u64,
    pub active_connections: u32,
    pub total_requests: u64,
    pub requests_per_second: f64,
    pub avg_response_time_ms: f64,
    pub cpu_usage_percent: Option<f32>,
    // More metrics...
}
```

**DeepSearch DTOs:**
```rust
pub struct DeepSearchBody {
    pub query: String,
    pub limit: Option<u32>,
    pub country: Option<String>,
    pub locale: Option<String>,
    pub include_content: Option<bool>,
    pub crawl_options: Option<CrawlOptions>,
}

pub struct DeepSearchResponse {
    pub query: String,
    pub urls_found: usize,
    pub urls_crawled: usize,
    pub results: Vec<SearchResult>,
    pub status: String,
    pub processing_time_ms: u64,
}

pub struct SearchResult {
    pub url: String,
    pub rank: u32,
    pub search_title: Option<String>,
    pub search_snippet: Option<String>,
    pub content: Option<ExtractedDoc>,
    pub crawl_result: Option<CrawlResult>,
}
```

**Shared Types:**
- `riptide_types::ExtractedDoc` - Core document type (from riptide-types)
- `riptide_types::config::CrawlOptions` - Re-exported from types crate

---

## Handler Calling Patterns

### Pattern 1: Facade-First (RECOMMENDED)

**Example: Extract Handler**
```rust
pub async fn extract(
    State(state): State<AppState>,
    Json(payload): Json<ExtractRequest>,
) -> impl IntoResponse {
    // 1. Validate input
    let _url = url::Url::parse(&payload.url)?;

    // 2. Fetch HTML (could be extracted to facade)
    let html = state.http_client.get(&payload.url).send().await?.text().await?;

    // 3. Use facade for extraction
    let result = state.extraction_facade
        .extract_with_fallback(&html, &payload.url, strategies)
        .await?;

    // 4. Convert to API response
    Ok(Json(ExtractResponse::from(result)))
}
```

**Advantages:**
- ✅ Clean separation of concerns
- ✅ Testable (mock facade)
- ✅ Reusable across different APIs
- ✅ Error handling in one place
- ✅ Metrics/telemetry in facade

### Pattern 2: Direct Pipeline (LEGACY, NEEDS REFACTORING)

**Example: Crawl Handler**
```rust
pub async fn crawl(
    State(state): State<AppState>,
    Json(body): Json<CrawlBody>,
) -> Result<Json<CrawlResponse>, ApiError> {
    // ❌ Direct instantiation - tight coupling
    let pipeline = if state.config.enhanced_pipeline_config.enable_enhanced_pipeline {
        EnhancedPipelineOrchestrator::new(state.clone(), options)
    } else {
        PipelineOrchestrator::new(state.clone(), options)
    };

    // ❌ Handler knows about pipeline internals
    let (results, stats) = pipeline.execute_batch(&body.urls).await;

    // ❌ Manual conversion
    let crawl_results = convert_pipeline_to_api(results);

    Ok(Json(CrawlResponse { results, stats }))
}
```

**Problems:**
- ❌ Tight coupling to pipeline implementation
- ❌ Duplication (standard vs enhanced)
- ❌ Harder to test
- ❌ No abstraction for future changes

**Recommended Refactoring:**
```rust
// Add to riptide-facade
impl ExtractionFacade {
    pub async fn batch_crawl(
        &self,
        urls: &[String],
        options: CrawlOptions,
    ) -> Result<(Vec<CrawlResult>, CrawlStats)> {
        // Encapsulate pipeline selection and execution
        if self.config.use_enhanced_pipeline {
            self.enhanced_batch_crawl(urls, options).await
        } else {
            self.standard_batch_crawl(urls, options).await
        }
    }
}

// Handler becomes simple:
pub async fn crawl(
    State(state): State<AppState>,
    Json(body): Json<CrawlBody>,
) -> Result<Json<CrawlResponse>, ApiError> {
    let (results, stats) = state.extraction_facade
        .batch_crawl(&body.urls, body.options.unwrap_or_default())
        .await?;

    Ok(Json(CrawlResponse::from_results(results, stats)))
}
```

### Pattern 3: Service Manager (GOOD)

**Example: Session Handler**
```rust
pub async fn create_session(
    State(state): State<AppState>,
    Json(request): Json<CreateSessionRequest>,
) -> Result<Json<SessionInfo>, ApiError> {
    // ✅ Clean service API
    let session_id = state.session_manager
        .create_session(request.ttl_seconds)
        .await?;

    Ok(Json(SessionInfo { session_id }))
}
```

**Advantages:**
- ✅ Service owns lifecycle
- ✅ Clear API surface
- ✅ Reusable

---

## Streaming Implementation Patterns

### NDJSON Streaming (handlers/streaming.rs)

**Current Implementation:**
```rust
pub async fn crawl_stream(
    State(state): State<AppState>,
    Json(body): Json<CrawlBody>,
) -> impl IntoResponse {
    let (tx, rx) = mpsc::channel(100);

    // Spawn background task
    tokio::spawn(async move {
        for url in body.urls {
            // Process URL
            let result = process_url(&url).await;

            // Stream as NDJSON
            let json = serde_json::to_string(&result).unwrap();
            tx.send(Ok(format!("{}\n", json))).await.ok();
        }
    });

    // Return SSE stream
    Sse::new(ReceiverStream::new(rx))
}
```

**Issues:**
- ⚠️ Code duplication with non-streaming handlers
- ⚠️ No backpressure handling
- ⚠️ Error handling inconsistent

**Recommended for v1:**
- Create `StreamingFacade` or add streaming methods to existing facades
- Unified error handling
- Proper backpressure with `StreamingModule`

---

## Session Management Implementation

**Architecture:**
- `SessionManager` - Core session lifecycle management
- `SessionStore` (Redis) - Persistent storage
- `SessionLayer` (middleware) - Request injection
- Background cleanup task - TTL-based expiration

**Key Components:**

```rust
pub struct SessionManager {
    store: Arc<SessionStore>,
    config: SessionConfig,
    cleanup_handle: Option<tokio::task::JoinHandle<()>>,
}

impl SessionManager {
    pub async fn create_session(&self, ttl_seconds: Option<u64>) -> Result<String>
    pub async fn get_session(&self, session_id: &str) -> Result<Option<SessionData>>
    pub async fn delete_session(&self, session_id: &str) -> Result<()>
    pub async fn extend_session(&self, session_id: &str, ttl_seconds: u64) -> Result<()>
    pub async fn set_cookie(&self, session_id: &str, cookie: Cookie) -> Result<()>
    pub async fn get_cookies(&self, session_id: &str, domain: &str) -> Result<Vec<Cookie>>

    fn start_cleanup_task(&self) {
        // Background task for expired session cleanup
    }
}
```

**SessionLayer Middleware:**
```rust
// Applied only to /render routes
let session_routes = Router::new()
    .route("/render", post(handlers::render))
    .layer(SessionLayer::new(session_manager.clone()))
    .with_state(app_state.clone());
```

**Session Data Structure:**
```rust
pub struct SessionData {
    pub session_id: String,
    pub cookies: HashMap<String, Vec<Cookie>>,
    pub created_at: SystemTime,
    pub last_accessed: SystemTime,
    pub metadata: HashMap<String, String>,
}
```

---

## Reusable Patterns for v1

### ✅ **KEEP AS-IS (High Quality)**

1. **Facade Pattern Implementation**
   - ExtractionFacade, SpiderFacade, SearchFacade, BrowserFacade, ScraperFacade
   - Clean abstraction, error handling, metrics
   - **Action:** Extend to cover all operations

2. **Session Management**
   - Redis-backed persistent sessions
   - Cookie management per domain
   - Background cleanup task
   - **Action:** Keep implementation, possibly enhance API

3. **Health Check System**
   - Comprehensive dependency checking
   - Component-level health endpoints
   - Detailed metrics
   - **Action:** Keep, add to v1 spec

4. **Middleware Stack**
   - Auth, rate limiting, validation, timeout, compression
   - Well-ordered, composable
   - **Action:** Keep order, review configs

5. **Worker Service Integration**
   - Background job processing
   - Queue management
   - Scheduler support
   - **Action:** Keep, document in v1

6. **Monitoring & Telemetry**
   - Performance metrics
   - Alert system
   - Distributed tracing
   - **Action:** Keep, enhance dashboards

### ⚠️ **REFACTOR (Medium Priority)**

1. **Crawl Handlers → Use Facade**
   - Current: Direct `PipelineOrchestrator` instantiation
   - Target: `ExtractionFacade::batch_crawl()`
   - **Action:** Add batch_crawl to facade, update handlers

2. **DeepSearch → Unified Facade Method**
   - Current: SearchFacade + direct PipelineOrchestrator
   - Target: `SearchFacade::search_and_crawl()`
   - **Action:** Add method to SearchFacade

3. **Streaming → Dedicated Facade**
   - Current: Duplicated logic in streaming handlers
   - Target: `StreamingFacade` or facade methods
   - **Action:** Create StreamingFacade with proper backpressure

4. **PDF Processing → Add Facade**
   - Current: Direct `riptide_pdf` integration calls
   - Target: `PDFFacade` in riptide-facade
   - **Action:** Create PDFFacade

5. **Stealth Configuration → Add Facade**
   - Current: Direct `StealthController` access
   - Target: `StealthFacade` or integrate into BrowserFacade
   - **Action:** Create StealthFacade

6. **Table Extraction → Add Facade**
   - Current: Direct table extractor calls
   - Target: `TableFacade` or method on ExtractionFacade
   - **Action:** Add table extraction to ExtractionFacade

7. **LLM Provider Management → Add Facade**
   - Current: Direct LLM config access
   - Target: `LLMFacade`
   - **Action:** Create LLMFacade for provider management

8. **Content Chunking → Facade Method**
   - Current: Direct chunking service
   - Target: Method on ExtractionFacade
   - **Action:** Add `chunk_content()` to ExtractionFacade

9. **Engine Selection → Add Facade**
   - Current: Direct engine analyzer/decider
   - Target: `EngineFacade` or integrate into ExtractionFacade
   - **Action:** Consolidate into ExtractionFacade

10. **Domain Profiles → Add Facade**
    - Current: Direct profile manager access
    - Target: `ProfileFacade`
    - **Action:** Create ProfileFacade for cache warming

11. **Strategy Routes → Deprecated?**
    - Current: Direct strategy pipeline
    - Target: Covered by ExtractionFacade modes
    - **Action:** Evaluate if needed, possibly deprecate

### ❌ **DEPRECATE/REMOVE**

1. **Duplicate Profiling Endpoints**
   - `/monitoring/profiling/*` routes (deprecated)
   - **Action:** Remove in v1, use `/api/profiling/*` only

2. **Dual Pipeline Paths**
   - PipelineOrchestrator vs EnhancedPipelineOrchestrator
   - **Action:** Consolidate behind facade, make transparent

3. **Admin Routes (if not needed)**
   - Feature-gated admin endpoints for multi-tenancy
   - **Action:** Defer to v2 if not needed for v1

---

## Breaking Changes Required for v1

### API Changes

1. **Standardize Path Prefixes**
   - **Current:** Mixed `/`, `/api/v1/`, no consistent versioning
   - **v1:** All endpoints under `/api/v1/`
   - **Migration:** Keep legacy paths as aliases with deprecation warnings

2. **Consolidate Crawl Endpoints**
   - **Current:** `/crawl`, `/crawl/stream`, `/strategies/crawl`
   - **v1:** Single `/api/v1/crawl` with `stream=true` query param
   - **Migration:** Redirect old endpoints

3. **Unified Result Modes**
   - **Current:** Spider has result_mode query param, crawl doesn't
   - **v1:** All crawl endpoints support `result_mode` query param
   - **Migration:** Add to crawl endpoints

4. **Consistent Metadata**
   - **Current:** Different metadata structures per endpoint
   - **v1:** Standardized `ResponseMetadata` on all responses
   - **Migration:** Add metadata wrapper

### Service Changes

1. **All Operations Through Facades**
   - **Current:** Mixed facade + direct service access
   - **v1:** All handlers use facades exclusively
   - **Migration:** Create missing facades, update handlers

2. **Streaming via Facade Methods**
   - **Current:** Separate streaming handlers
   - **v1:** `stream: bool` parameter on facade methods
   - **Migration:** Add streaming support to facades

3. **Remove Pipeline Direct Access**
   - **Current:** Handlers instantiate PipelineOrchestrator
   - **v1:** ExtractionFacade handles pipeline selection
   - **Migration:** Add batch_crawl to facade

### Configuration Changes

1. **Environment Variable Standardization**
   - **Current:** Mixed prefixes (RIPTIDE_, no prefix, etc.)
   - **v1:** All `RIPTIDE_*` prefixed
   - **Migration:** Add env var aliases

2. **Feature Flags**
   - **Current:** Compile-time feature gates
   - **v1:** Runtime feature toggles via config
   - **Migration:** Add feature flag system

---

## Recommended v1 Implementation Roadmap

### Phase 1: Facade Completion (Priority: High)
- [ ] Create `PDFFacade` for PDF processing
- [ ] Create `StealthFacade` for stealth configuration
- [ ] Create `ProfileFacade` for domain profile management
- [ ] Create `LLMFacade` for provider management
- [ ] Add `batch_crawl()` method to `ExtractionFacade`
- [ ] Add `search_and_crawl()` method to `SearchFacade`
- [ ] Add `chunk_content()` method to `ExtractionFacade`
- [ ] Add streaming support to all facades (`.stream()` method)

### Phase 2: Handler Refactoring (Priority: High)
- [ ] Update `crawl` handler to use `ExtractionFacade::batch_crawl()`
- [ ] Update `deepsearch` handler to use `SearchFacade::search_and_crawl()`
- [ ] Update streaming handlers to use facade `.stream()` methods
- [ ] Update PDF handlers to use `PDFFacade`
- [ ] Update stealth handlers to use `StealthFacade`
- [ ] Update profile handlers to use `ProfileFacade`
- [ ] Update LLM handlers to use `LLMFacade`
- [ ] Remove direct `PipelineOrchestrator` instantiation

### Phase 3: API Standardization (Priority: Medium)
- [ ] Add `/api/v1` prefix to all endpoints
- [ ] Implement legacy path aliases with deprecation warnings
- [ ] Add `result_mode` query param to all crawl endpoints
- [ ] Standardize response metadata across all endpoints
- [ ] Add `stream` query param to all batch operations
- [ ] Implement consistent error response format

### Phase 4: Deprecation & Cleanup (Priority: Low)
- [ ] Remove duplicate profiling endpoints (`/monitoring/profiling/*`)
- [ ] Deprecate `/strategies` routes (covered by extraction modes)
- [ ] Remove EnhancedPipelineOrchestrator direct access
- [ ] Consolidate pipeline implementations behind facade
- [ ] Clean up unused admin routes (if not needed)

### Phase 5: Documentation & Testing (Priority: High)
- [ ] OpenAPI/Swagger spec for all v1 endpoints
- [ ] Migration guide from legacy to v1
- [ ] Integration tests for all facades
- [ ] Load tests for streaming endpoints
- [ ] Backward compatibility tests

---

## Key Metrics & Statistics

**Total Endpoints:** 90+
**Route Categories:** 15
**Facades in Use:** 5 (Extraction, Spider, Search, Browser, Scraper)
**Missing Facades:** 5 (PDF, Stealth, Profile, LLM, Streaming)
**Handlers Using Facades:** ~40%
**Handlers Needing Refactoring:** ~60%
**Middleware Layers:** 9 (+ SessionLayer for /render)
**DTOs Defined:** 30+
**Reusable Patterns:** 6
**Breaking Changes:** 8 major categories

---

## Conclusion

The Riptide API has a **solid foundation** with:
- ✅ Excellent facade pattern for 5 major services
- ✅ Comprehensive session management
- ✅ Robust monitoring and telemetry
- ✅ Well-designed middleware stack

**Main areas for v1 improvement:**
- 🔧 Complete facade coverage (add 5 missing facades)
- 🔧 Eliminate direct pipeline instantiation in handlers
- 🔧 Standardize API paths and response formats
- 🔧 Consolidate streaming implementation
- 🔧 Unify error handling and metadata

**Estimated v1 Implementation Effort:**
- **Facade Creation:** 2-3 weeks
- **Handler Refactoring:** 2-3 weeks
- **API Standardization:** 1-2 weeks
- **Testing & Documentation:** 2 weeks
- **Total:** 7-10 weeks

**Risk Assessment:**
- Low risk for facade pattern (proven approach)
- Medium risk for breaking changes (requires migration plan)
- Low risk for backward compatibility (can maintain legacy aliases)

This analysis provides a complete roadmap for v1 implementation with clear priorities and actionable tasks.
