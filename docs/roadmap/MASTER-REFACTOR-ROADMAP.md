# 🎯 RipTide v1.0 Master Refactoring Roadmap
**Comprehensive, Prioritized, Reality-Based Implementation Plan**

**Generated:** 2025-11-04
**Current Version:** 0.9.0
**Target Version:** 1.0.0
**Architecture:** 26 crates, distributed microservices

---

## 📋 Executive Summary

**Objective:** Consolidate RipTide's 26 existing crates into a stable, schema-agnostic v1.0 platform with:
- ✅ Single canonical API surface (`/api/v1`)
- ✅ Unified configuration model (`server.yaml` + env precedence)
- ✅ Schema registry with validation
- ✅ Zero feature loss
- ✅ Comprehensive observability

**Critical Context:** This is a **REFACTORING**, not a rewrite. Most functionality exists and works. We're organizing, consolidating, and adding a unified API layer.

---

## 🎯 Ground Truth: What Already Exists

### ✅ Fully Functional Components (DO NOT RECREATE)
| Component | Location | Status |
|-----------|----------|--------|
| **HTTP Client** | `riptide-fetch` | ✅ Complete with retry/backoff |
| **Browser Automation** | `riptide-browser`, `riptide-headless` | ✅ Spider Chrome CDP integration |
| **Extraction** | `riptide-extraction` | ✅ HTML/CSS/regex/WASM/PDF/tables |
| **Crawling** | `riptide-spider` | ✅ 4 strategies, robots.txt, sitemaps |
| **Cache** | `riptide-cache` | ✅ Redis + local with TTL |
| **Persistence** | `riptide-persistence` | ✅ Multi-tenancy, state management |
| **Monitoring** | `riptide-monitoring` | ✅ OpenTelemetry, Prometheus |
| **Security** | `riptide-security` | ✅ Auth, rate limiting |
| **Config** | `riptide-config` | ✅ Builder pattern, validation |
| **Types** | `riptide-types` | ✅ `RiptideError`, core traits |
| **Events** | `riptide-events` | ✅ Pub/sub architecture |
| **Reliability** | `riptide-reliability` | ✅ Circuit breakers, retries |
| **LLM** | `riptide-intelligence` | ✅ OpenAI, Anthropic integration |
| **Search** | `riptide-search` | ✅ Search provider abstraction |
| **PDF** | `riptide-pdf` | ✅ pdfium-render integration |
| **Stealth** | `riptide-stealth` | ✅ Fingerprinting avoidance |
| **Workers** | `riptide-workers` | ✅ Background job processing |
| **CLI** | `riptide-cli` | ✅ Command-line interface |
| **API** | `riptide-api` | ✅ 120+ routes, Axum 0.7 |
| **Facade** | `riptide-facade` | ✅ Builder pattern, high-level API |

### 🔧 What Needs Adding
1. **riptide-utils** - Consolidate shared code (NEW crate)
2. **riptide-api-types** - API boundary DTOs (NEW crate)
3. **riptide-schemas** - JSON Schema registry (NEW crate)
4. **riptide-adapters** - Schema adapters (NEW crate)
5. **riptide-validation** - Validation engine (NEW crate)
6. `/api/v1` routes - New routes that CALL existing logic
7. Legacy shims - Backward compatibility layer
8. `server.yaml` - Unified configuration file
9. Enhanced facade - Add `run_pipeline()` orchestration

---

## 📊 Dependency Architecture (Current State)

### Layer 1: Foundation (No upstream dependencies)
```
riptide-types → [all other crates]
riptide-config → [service + orchestration layers]
riptide-utils → [NEW - consolidate shared utilities]
```

### Layer 2: Infrastructure
```
riptide-monitoring, riptide-security, riptide-reliability,
riptide-performance, riptide-events, riptide-cache
```

### Layer 3: Services
```
riptide-extraction, riptide-browser, riptide-headless,
riptide-spider, riptide-fetch, riptide-intelligence,
riptide-search, riptide-persistence, riptide-stealth,
riptide-pdf, riptide-workers
```

### Layer 4: Orchestration
```
riptide-facade → calls Layer 3 services
riptide-workers → background job orchestration
```

### Layer 5: Interface
```
riptide-api → ONLY calls facade
riptide-cli → calls API or facade directly
```

### Support Layer
```
riptide-schemas, riptide-adapters, riptide-validation,
riptide-test-utils
```

---

## 🚀 Implementation Phases (12 Weeks)

### Phase 0: Preparation & Foundation (Weeks 0-1) ⚡ CRITICAL PATH

#### W0.1: Create riptide-utils Crate
**Priority:** P0 (BLOCKS ALL OTHERS)
**Status:** Not Started
**Complexity:** Medium

**Actions:**
```bash
cargo new --lib crates/riptide-utils
```

**Consolidate by MOVING (not rewriting):**
- [ ] Move Redis pool creation from `riptide-persistence/src/redis.rs`
- [ ] Move HTTP client from `riptide-fetch/src/client.rs`
- [ ] Re-export error types from `riptide-types`
- [ ] Extract time utilities from scattered locations
- [ ] Create module structure:
  ```
  riptide-utils/src/
    lib.rs
    redis.rs      # MOVED from riptide-persistence
    http.rs       # MOVED from riptide-fetch
    error.rs      # RE-EXPORT from riptide-types
    time.rs       # EXTRACTED from various crates
  ```

**Acceptance Criteria:**
- [ ] `cargo build -p riptide-utils` succeeds
- [ ] All moved code still passes original tests
- [ ] Importing crates updated to use `riptide-utils`
- [ ] No duplicate implementations

**Files Created:**
- `/crates/riptide-utils/Cargo.toml`
- `/crates/riptide-utils/src/lib.rs`
- `/crates/riptide-utils/src/{redis,http,error,time}.rs`
- `/crates/riptide-utils/README.md`

---

#### W0.2: Create server.yaml Configuration
**Priority:** P0
**Status:** Not Started
**Complexity:** Low

**Actions:**
```bash
touch server.yaml
```

**Consolidate existing environment variables into:**
```yaml
version: "1.0"

server:
  host: "0.0.0.0"
  port: 8080
  timeout_ms: 30000

security:
  data_api_keys: []
  admin_api_keys: []
  rate_limits:
    per_ip_per_minute: 60
    per_key_per_minute: 600
  max_request_body_mb: 50

redis:
  url: "${REDIS_URL}"
  pool_size: 10

headless:
  url: "${HEADLESS_URL:http://localhost:9222}"
  pool_size: 8
  timeout_ms: 30000

integrations:
  llm:
    daily_budget_eur: 10.0
    providers:
      openai: "${OPENAI_API_KEY}"
      anthropic: "${ANTHROPIC_API_KEY}"

  search:
    provider: "serper"
    api_key: "${SERPER_API_KEY}"

profiles:
  lite:
    strategies:
      enable_llm: false
      enable_headless: false
      enable_wasm: true
  full:
    strategies:
      enable_llm: true
      enable_headless: true
      enable_wasm: true

observability:
  otel_endpoint: "${OTEL_ENDPOINT}"
  metrics_enabled: true
  tracing_enabled: true
```

**Acceptance Criteria:**
- [ ] YAML lints without errors
- [ ] All existing env vars mapped
- [ ] Profiles defined (lite, full)
- [ ] Defaults work without env vars

**Files Created:**
- `/server.yaml`
- `/server.staging.yaml`
- `/server.canary.yaml`

---

#### W0.3: Enhance riptide-config for Precedence
**Priority:** P0
**Status:** Not Started
**Complexity:** Medium

**Actions:**
Enhance `/crates/riptide-config/src/lib.rs`:

```rust
// ADD to existing ConfigBuilder
impl ConfigBuilder {
    /// Apply configuration precedence:
    /// Request options > Profile > server.yaml > Schema defaults
    pub fn with_precedence(
        request: Option<RequestConfig>,
        profile: ProfileConfig,
        server_defaults: ServerConfig
    ) -> Self {
        // Implementation that merges configs
    }

    /// Load server.yaml with environment substitution
    pub fn from_server_yaml(path: &str) -> Result<Self> {
        // Parse YAML, substitute ${VAR} with env
    }
}
```

**Acceptance Criteria:**
- [ ] Loads `server.yaml` with env substitution
- [ ] Precedence order enforced
- [ ] Existing tests still pass
- [ ] New tests for precedence logic

**Files Modified:**
- `/crates/riptide-config/src/lib.rs`
- `/crates/riptide-config/src/precedence.rs` (NEW)
- `/crates/riptide-config/tests/precedence.rs` (NEW)

---

#### W0.4: Establish CI Baseline
**Priority:** P0
**Status:** Not Started
**Complexity:** Low

**Actions:**
```bash
# Document current state
cargo build --workspace --locked 2>&1 | tee baseline-build.log
cargo test --workspace 2>&1 | tee baseline-tests.log
cargo clippy --workspace 2>&1 | tee baseline-clippy.log
```

**Acceptance Criteria:**
- [ ] All crates build successfully
- [ ] Document current test pass/fail status
- [ ] Baseline metrics captured
- [ ] No regressions in subsequent phases

**Files Created:**
- `/docs/baseline-build.log`
- `/docs/baseline-tests.log`
- `/docs/baseline-clippy.log`

---

### Phase 1: API v1 Layer (Weeks 1-2) 🔥 HIGH PRIORITY

#### W1.1: Create riptide-api-types Crate
**Priority:** P1
**Status:** Not Started
**Complexity:** Medium

**Actions:**
```bash
cargo new --lib crates/riptide-api-types
```

**Create DTOs for API boundary:**
```
riptide-api-types/src/
  lib.rs
  v1/
    mod.rs
    extract.rs    # ExtractRequestV1, ExtractResponseV1
    crawl.rs      # CrawlRequestV1, CrawlResponseV1
    spider.rs     # SpiderRequestV1, SpiderResponseV1
    common.rs     # Shared types (PageCtx, ResultItem)
    error.rs      # ErrorResponseV1
```

**Pattern: DTOs convert to/from internal types:**
```rust
// Example: Don't duplicate business logic
impl From<ExtractRequestV1> for ExtractionRequest {
    fn from(dto: ExtractRequestV1) -> Self {
        // Conversion only
    }
}

impl From<ExtractionResult> for ExtractResponseV1 {
    fn from(result: ExtractionResult) -> Self {
        // Conversion only
    }
}
```

**Acceptance Criteria:**
- [ ] DTOs defined for all v1 endpoints
- [ ] Conversions to/from internal types
- [ ] Serde serialization tests
- [ ] No business logic in DTOs

**Files Created:**
- `/crates/riptide-api-types/Cargo.toml`
- `/crates/riptide-api-types/src/lib.rs`
- `/crates/riptide-api-types/src/v1/*.rs`

---

#### W1.2: Implement /api/v1 Routes
**Priority:** P1
**Status:** Not Started
**Complexity:** High

**Actions:**
Create `/crates/riptide-api/src/routes/v1/`:
```
v1/
  mod.rs
  extract.rs
  crawl.rs
  spider.rs
  search.rs
  browser.rs
  health.rs
  diagnostics.rs
```

**Pattern: Thin handlers that call existing logic:**
```rust
// Example: /api/v1/extract
async fn extract_v1(
    State(app_state): State<AppState>,
    Json(request): Json<ExtractRequestV1>,
) -> Result<Json<ExtractResponseV1>> {
    // 1. Convert DTO to internal type
    let internal_request = request.into();

    // 2. Call EXISTING extraction logic
    let result = riptide_extraction::extract(internal_request).await?;

    // 3. Convert back to DTO
    Ok(Json(result.into()))
}
```

**Endpoints to implement:**
```
POST /api/v1/extract
POST /api/v1/crawl
POST /api/v1/spider
POST /api/v1/discover
GET  /api/v1/health
GET  /api/v1/diagnostics
GET  /api/v1/metrics
POST /api/v1/browser/session
POST /api/v1/browser/action
```

**Acceptance Criteria:**
- [ ] All v1 routes defined in `/routes/v1/`
- [ ] Routes call existing service layer
- [ ] No duplicate business logic
- [ ] Integration tests pass

**Files Created:**
- `/crates/riptide-api/src/routes/v1/mod.rs`
- `/crates/riptide-api/src/routes/v1/*.rs`

---

#### W1.3: Create Legacy Shim Middleware
**Priority:** P1
**Status:** Not Started
**Complexity:** Medium

**Actions:**
Create `/crates/riptide-api/src/shims.rs`:

```rust
/// Maps legacy routes to v1 endpoints
pub struct LegacyShimMiddleware {
    mappings: HashMap<&'static str, &'static str>,
}

impl LegacyShimMiddleware {
    pub fn new() -> Self {
        Self {
            mappings: hashmap! {
                "/crawl" => "/api/v1/crawl",
                "/extract" => "/api/v1/extract",
                "/spider/crawl" => "/api/v1/spider",
                // ... all legacy routes
            }
        }
    }
}

// Middleware that:
// 1. Detects legacy route
// 2. Proxies to v1 endpoint
// 3. Adds Deprecation header
// 4. Logs usage for tracking
```

**Acceptance Criteria:**
- [ ] All legacy routes mapped
- [ ] Deprecation headers added
- [ ] Metrics track legacy usage
- [ ] Legacy tests pass via shims

**Files Created:**
- `/crates/riptide-api/src/shims.rs`
- `/crates/riptide-api/src/middleware/legacy_shim.rs`

---

#### W1.4: Update CLI to Use v1 API
**Priority:** P1
**Status:** Not Started
**Complexity:** Low

**Actions:**
Update `/crates/riptide-cli/src/commands/*.rs`:

```rust
// Change all API calls to v1 endpoints
// OLD: POST /extract
// NEW: POST /api/v1/extract

// Use riptide-api-types DTOs
use riptide_api_types::v1::*;
```

**Acceptance Criteria:**
- [ ] All CLI commands use v1 endpoints
- [ ] CLI tests pass
- [ ] No legacy endpoint calls
- [ ] Help text updated

**Files Modified:**
- `/crates/riptide-cli/src/commands/*.rs`
- `/crates/riptide-cli/Cargo.toml` (add riptide-api-types dep)

---

### Phase 2: Schema Registry & Validation (Weeks 2-3) 🎯 CORE FEATURE

#### W2.1: Create riptide-schemas Crate
**Priority:** P1
**Status:** Not Started
**Complexity:** Medium

**Actions:**
```bash
cargo new --lib crates/riptide-schemas
```

**Create schema registry:**
```
riptide-schemas/
  schemas/
    registry.json
    events.v1.json
    jobs.v1.json
    options.schema.json
    diagnostics.schema.json
  src/
    lib.rs
    loader.rs
    registry.rs
```

**registry.json:**
```json
{
  "version": "1.0",
  "schemas": [
    {
      "name": "events",
      "version": "v1",
      "file": "events.v1.json",
      "entity_type": "event",
      "dedup_strategy": "content_hash",
      "key_fields": ["url", "title", "start_date"]
    },
    {
      "name": "jobs",
      "version": "v1",
      "file": "jobs.v1.json",
      "entity_type": "job",
      "dedup_strategy": "url_hash",
      "key_fields": ["url", "title"]
    }
  ]
}
```

**Acceptance Criteria:**
- [ ] Registry loads at boot
- [ ] JSON Schemas validate
- [ ] Schema versioning supported
- [ ] Tests for schema loading

**Files Created:**
- `/crates/riptide-schemas/Cargo.toml`
- `/crates/riptide-schemas/schemas/*.json`
- `/crates/riptide-schemas/src/*.rs`

---

#### W2.2: Create riptide-adapters Crate
**Priority:** P1
**Status:** Not Started
**Complexity:** High

**Actions:**
```bash
cargo new --lib crates/riptide-adapters
```

**Create adapters for each schema:**
```
riptide-adapters/src/
  lib.rs
  events_v1.rs    # Maps StrategyOut → Event entity
  jobs_v1.rs      # Maps StrategyOut → Job entity
  common.rs       # Shared adapter utilities
  dedup.rs        # Dedup key computation
```

**Pattern: Field mapping with confidence:**
```rust
pub struct EventsV1Adapter;

impl SchemaAdapter for EventsV1Adapter {
    fn adapt(&self, output: StrategyOut) -> AdapterResult {
        // Extract fields
        let url = output.get_url()?;
        let title = output.get_title()?;
        let confidence = output.confidence;

        // Compute dedup key
        let dedup_key = compute_hash(&[url, title]);

        // Create entity
        let entity = EventEntity {
            url,
            title,
            confidence,
            dedup_key,
            provenance: output.provenance,
        };

        AdapterResult::Valid(entity)
    }
}
```

**Acceptance Criteria:**
- [ ] Adapters for all schemas
- [ ] Dedup key computation
- [ ] Confidence thresholds
- [ ] Quarantine handling

**Files Created:**
- `/crates/riptide-adapters/Cargo.toml`
- `/crates/riptide-adapters/src/*.rs`

---

#### W2.3: Create riptide-validation Crate
**Priority:** P1
**Status:** Not Started
**Complexity:** High

**Actions:**
```bash
cargo new --lib crates/riptide-validation
```

**Create validation engine:**
```rust
pub struct AdapterEngine {
    registry: SchemaRegistry,
    adapters: HashMap<String, Box<dyn SchemaAdapter>>,
}

impl AdapterEngine {
    pub async fn validate_and_adapt(
        &self,
        schema_name: &str,
        output: StrategyOut,
    ) -> ValidationResult {
        // 1. Get adapter for schema
        let adapter = self.adapters.get(schema_name)?;

        // 2. Adapt strategy output to entity
        let result = adapter.adapt(output)?;

        // 3. Validate against JSON Schema
        match result {
            AdapterResult::Valid(entity) => {
                if self.validate_schema(&entity)? {
                    ValidationResult::Valid(entity)
                } else {
                    ValidationResult::Quarantine {
                        reason: "SCHEMA_INVALID",
                        entity,
                    }
                }
            }
            AdapterResult::LowConfidence(entity) => {
                ValidationResult::Quarantine {
                    reason: "LOW_CONFIDENCE",
                    entity,
                }
            }
        }
    }
}
```

**Acceptance Criteria:**
- [ ] Engine validates all schemas
- [ ] Quarantine reasons defined
- [ ] < 5% quarantine rate on fixtures
- [ ] Performance < 10ms per entity

**Files Created:**
- `/crates/riptide-validation/Cargo.toml`
- `/crates/riptide-validation/src/lib.rs`
- `/crates/riptide-validation/src/engine.rs`

---

#### W2.4: Integrate Validation into Persistence
**Priority:** P1
**Status:** Not Started
**Complexity:** Medium

**Actions:**
Enhance `/crates/riptide-persistence/`:

```sql
-- migrations/entities.sql
CREATE TABLE entities (
  id UUID PRIMARY KEY,
  schema_name VARCHAR NOT NULL,
  schema_version VARCHAR NOT NULL,
  entity_type VARCHAR NOT NULL,
  dedup_key VARCHAR NOT NULL,
  data JSONB NOT NULL,
  confidence FLOAT NOT NULL,
  provenance JSONB NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_dedup ON entities(dedup_key, schema_name);
CREATE INDEX idx_schema ON entities(schema_name, schema_version);

-- migrations/quarantine.sql
CREATE TABLE quarantine (
  id UUID PRIMARY KEY,
  schema_name VARCHAR NOT NULL,
  reason VARCHAR NOT NULL,
  data JSONB NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW()
);
```

**Acceptance Criteria:**
- [ ] Tables created
- [ ] Dedup index enforced
- [ ] Quarantine logged
- [ ] Window enforcement (90 days)

**Files Created:**
- `/crates/riptide-persistence/migrations/*.sql`
- `/crates/riptide-persistence/src/entity_store.rs`

---

### Phase 3: Enhanced Facade & Pipeline (Weeks 3-4) ⚙️ ORCHESTRATION

#### W3.1: Enhance riptide-facade with run_pipeline()
**Priority:** P1
**Status:** Not Started
**Complexity:** High

**Actions:**
Add to `/crates/riptide-facade/src/lib.rs`:

```rust
/// Unified pipeline entry point
pub async fn run_pipeline(
    inputs: PipelineInputs,
    options: PipelineOptions,
) -> impl Stream<Item = ResultItem> {
    // Determine execution mode
    match inputs.mode {
        Mode::Extract => {
            // Strategy execution order:
            // 1. ICS + JSON (parallel, fast)
            // 2. Rulepack (if confidence < threshold)
            // 3. WASM (if enabled and confidence < threshold)
            // 4. LLM (if enabled and confidence < threshold)
            run_extraction_strategies(inputs, options).await
        }
        Mode::Crawl => {
            // Use existing spider logic
            riptide_spider::crawl_stream(inputs, options).await
        }
        Mode::Spider => {
            // Full spider with depth/breadth
            riptide_spider::spider_stream(inputs, options).await
        }
    }
}

async fn run_extraction_strategies(
    inputs: PipelineInputs,
    options: PipelineOptions,
) -> impl Stream<Item = ResultItem> {
    let mut results = Vec::new();
    let mut confidence = 0.0;

    // Fast strategies first
    if let Some(ics_result) = riptide_extraction::ics::extract(&inputs).await {
        confidence = ics_result.confidence;
        results.push(ics_result);
    }

    if confidence < options.quality.min_confidence {
        if let Some(json_result) = riptide_extraction::json::extract(&inputs).await {
            confidence = confidence.max(json_result.confidence);
            results.push(json_result);
        }
    }

    // Medium strategies
    if confidence < options.quality.min_confidence {
        if let Some(rulepack_result) = riptide_extraction::rulepack::extract(&inputs).await {
            confidence = confidence.max(rulepack_result.confidence);
            results.push(rulepack_result);
        }
    }

    // Expensive strategies (conditional)
    if confidence < options.quality.min_confidence {
        if options.strategies.enable_llm {
            if let Some(llm_result) = riptide_intelligence::extract(&inputs).await {
                confidence = confidence.max(llm_result.confidence);
                results.push(llm_result);
            }
        }
    }

    // Convert to stream
    stream::iter(results)
}
```

**Acceptance Criteria:**
- [ ] `run_pipeline()` implemented
- [ ] Strategy execution order correct
- [ ] Confidence-based early stopping
- [ ] Provenance chain tracked

**Files Created:**
- `/crates/riptide-facade/src/pipeline.rs`
- `/crates/riptide-facade/src/strategies/mod.rs`

---

### Phase 4: Diagnostics & Observability (Weeks 4-5) 📊 MONITORING

#### W4.1: Implement /api/v1/diagnostics
**Priority:** P2
**Status:** Not Started
**Complexity:** Medium

**Actions:**
Create `/crates/riptide-api/src/routes/v1/diagnostics.rs`:

```rust
#[derive(Serialize, JsonSchema)]
pub struct DiagnosticsResponse {
    version: String,
    profile: String,
    redis: HealthStatus,
    headless: HealthStatus,
    llm: HealthStatus,
    search_providers: Vec<ProviderStatus>,
    pools: PoolStats,
    budgets: BudgetStats,
    cache: CacheStats,
    warnings: Vec<String>,
}

pub async fn get_diagnostics(
    State(app_state): State<AppState>,
) -> Json<DiagnosticsResponse> {
    // Collect from all subsystems
    let diagnostics = DiagnosticsResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        profile: app_state.config.profile.clone(),
        redis: check_redis(&app_state).await,
        headless: check_headless(&app_state).await,
        llm: check_llm(&app_state).await,
        search_providers: check_search(&app_state).await,
        pools: get_pool_stats(&app_state).await,
        budgets: get_budget_stats(&app_state).await,
        cache: get_cache_stats(&app_state).await,
        warnings: collect_warnings(&app_state).await,
    };

    Json(diagnostics)
}
```

**Acceptance Criteria:**
- [ ] Endpoint returns < 500ms
- [ ] All subsystems checked
- [ ] JSON schema validates
- [ ] Warnings actionable

**Files Created:**
- `/crates/riptide-api/src/routes/v1/diagnostics.rs`
- `/schemas/diagnostics.schema.json`

---

#### W4.2: Create Grafana Dashboards
**Priority:** P2
**Status:** Not Started
**Complexity:** Medium

**Actions:**
```bash
mkdir -p dashboards/grafana
```

**Create dashboards:**
1. **Overview.json** - Request rate, latency, errors
2. **Headless_LLM.json** - Budget tracking
3. **Pipeline.json** - Strategy usage, confidence
4. **Cache.json** - Hit rates, memory
5. **Jobs.json** - Worker status, queue depth

**Metrics to expose:**
```rust
// In riptide-monitoring/src/metrics.rs
pub struct Metrics {
    requests_total: Counter,
    request_duration: Histogram,
    headless_active: Gauge,
    llm_cost_eur: Counter,
    entities_emitted: Counter,
    entities_quarantined: Counter,
    cache_hit_ratio: Gauge,
    errors_total: Counter,
}
```

**Acceptance Criteria:**
- [ ] 5 dashboards created
- [ ] All metrics exported
- [ ] Prometheus scrapes successfully
- [ ] Alerts defined

**Files Created:**
- `/dashboards/grafana/*.json`
- `/dashboards/alerts.yml`

---

### Phase 5: Discovery & Search (Weeks 5-6) 🔍 AUTOMATION

#### W5.1: Implement Search Providers
**Priority:** P2
**Status:** Not Started
**Complexity:** Medium

**Actions:**
Enhance `/crates/riptide-search/src/providers/`:

```
providers/
  mod.rs
  serper.rs      # Serper.dev integration
  bing.rs        # Bing Search API
  brave.rs       # Brave Search (optional)
```

**Pattern: Provider trait:**
```rust
#[async_trait]
pub trait SearchProvider {
    async fn search(&self, query: &str, count: usize) -> Result<Vec<SearchResult>>;
}

pub struct SerperProvider {
    api_key: String,
    client: reqwest::Client,
}

impl SearchProvider for SerperProvider {
    async fn search(&self, query: &str, count: usize) -> Result<Vec<SearchResult>> {
        // Call Serper API
    }
}
```

**Acceptance Criteria:**
- [ ] 2+ providers implemented
- [ ] Provider abstraction works
- [ ] Results cached (24h TTL)
- [ ] Rate limiting enforced

**Files Created:**
- `/crates/riptide-search/src/providers/*.rs`

---

#### W5.2: Implement /api/v1/discover
**Priority:** P2
**Status:** Not Started
**Complexity:** High

**Actions:**
Create `/crates/riptide-api/src/routes/v1/discover.rs`:

```rust
pub async fn discover_sources(
    State(app_state): State<AppState>,
    Json(request): Json<DiscoverRequest>,
) -> Json<DiscoverResponse> {
    // 1. Build search query
    let query = build_query(&request.schema, &request.scope);

    // 2. Search for candidate URLs
    let candidates = app_state.search.search(&query, 50).await?;

    // 3. Classify each candidate
    let sources = classify_sources(candidates, &request.schema).await?;

    // 4. Store in source_registry
    for source in &sources {
        app_state.persistence.store_source(source).await?;
    }

    Json(DiscoverResponse { sources })
}
```

**Acceptance Criteria:**
- [ ] Returns ≥10 sources per city
- [ ] Classification accuracy ≥0.8
- [ ] Sources stored in registry
- [ ] CLI command works

**Files Created:**
- `/crates/riptide-api/src/routes/v1/discover.rs`
- `/crates/riptide-persistence/src/source_registry.rs`

---

### Phase 6: Testing & CI (Weeks 6-8) ✅ QUALITY

#### W6.1: Create Test Fixtures
**Priority:** P2
**Status:** Not Started
**Complexity:** Medium

**Actions:**
```bash
mkdir -p crates/riptide-test-utils/tests/fixtures
mkdir -p crates/riptide-test-utils/tests/goldens
```

**Create fixtures for each strategy:**
- HTML pages (event listings, job postings)
- ICS calendar files
- JSON-LD embedded data
- PDFs
- Tables (HTML, CSV)

**Golden responses:**
```
goldens/
  extract/
    events_ics.request.json
    events_ics.response.json
    events_json.request.json
    events_json.response.json
  crawl/
    basic.request.json
    basic.response.json
```

**Acceptance Criteria:**
- [ ] 20+ fixtures covering all strategies
- [ ] Golden tests for each endpoint
- [ ] Normalizers handle non-determinism
- [ ] < 5% quarantine rate

**Files Created:**
- `/crates/riptide-test-utils/tests/fixtures/*.html`
- `/crates/riptide-test-utils/tests/goldens/*.json`

---

#### W6.2: Implement CI Workflows
**Priority:** P2
**Status:** Not Started
**Complexity:** Medium

**Actions:**
Create `.github/workflows/ci.yml`:

```yaml
name: CI

on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo fmt --check
      - run: cargo clippy --all --all-features

  test:
    runs-on: ubuntu-latest
    services:
      redis:
        image: redis:7
        ports:
          - 6379:6379
    steps:
      - uses: actions/checkout@v4
      - run: cargo test --workspace --all-features

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo tarpaulin --out Xml
      - run: |
          # Fail if coverage < 80% per crate

  build:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - run: cargo build --release
```

**Acceptance Criteria:**
- [ ] All jobs green
- [ ] Coverage ≥80% per crate
- [ ] Cross-platform builds work
- [ ] Artifacts uploaded

**Files Created:**
- `.github/workflows/ci.yml`
- `.github/workflows/nightly.yml`

---

### Phase 7: Security & Hardening (Weeks 8-10) 🔒 SECURITY

#### W7.1: Implement Authentication
**Priority:** P2
**Status:** Not Started
**Complexity:** Medium

**Actions:**
Enhance `/crates/riptide-api/src/middleware/auth.rs`:

```rust
pub async fn auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    // Check x-api-key header
    let api_key = req.headers()
        .get("x-api-key")
        .and_then(|h| h.to_str().ok());

    match api_key {
        Some(key) if state.config.security.admin_api_keys.contains(&key.to_string()) => {
            req.extensions_mut().insert(AuthRole::Admin);
            next.run(req).await
        }
        Some(key) if state.config.security.data_api_keys.contains(&key.to_string()) => {
            req.extensions_mut().insert(AuthRole::Data);
            next.run(req).await
        }
        _ => {
            // Optional auth for data plane
            if req.uri().path().starts_with("/api/v1/admin/") {
                StatusCode::UNAUTHORIZED.into_response()
            } else {
                next.run(req).await
            }
        }
    }
}
```

**Acceptance Criteria:**
- [ ] Admin routes require key
- [ ] 401/403 tests pass
- [ ] Secrets redacted in logs
- [ ] Audit events emitted

**Files Modified:**
- `/crates/riptide-api/src/middleware/auth.rs`

---

### Phase 8: Migration & Release (Weeks 10-12) 🚀 DEPLOYMENT

#### W8.1: Canary Deployment
**Priority:** P3
**Status:** Not Started
**Complexity:** High

**Actions:**
1. Deploy to staging
2. Run full test suite
3. Deploy to canary (5% traffic)
4. Monitor for 48h
5. Promote to production

**Rollback plan:**
```bash
# Automated rollback if errors > 2%
./scripts/rollback.sh
```

**Acceptance Criteria:**
- [ ] Canary stable for 48h
- [ ] Error rate < 1%
- [ ] Latency SLO met
- [ ] Rollback tested

**Files Created:**
- `/scripts/deploy-canary.sh`
- `/scripts/rollback.sh`

---

#### W8.2: Legacy Shim Removal
**Priority:** P3
**Status:** Not Started
**Complexity:** Low

**Timeline:**
- Day 0: v1 deployed, shims active
- Day 30: Warning logs
- Day 60: HTTP 410 on legacy routes
- Day 90: Remove shim code

**Acceptance Criteria:**
- [ ] Legacy traffic < 5% for 14 days
- [ ] Migration guide published
- [ ] Shims removed
- [ ] Archive legacy tests

**Files Deleted:**
- `/crates/riptide-api/src/shims.rs`
- `/tests/legacy_*.rs`

---

## 📊 Success Metrics

### Technical Metrics
| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| API Surface | `/api/v1` only | Mixed | 🔴 Not Started |
| Test Coverage | ≥80% | TBD | 🟡 Baseline Needed |
| Schema Validation | ≥95% valid | N/A | 🔴 Not Started |
| Legacy Traffic | 0% | 100% | 🔴 Not Started |
| Error Rate | <1% | TBD | 🟡 Monitor |
| Latency p95 (extract) | <500ms | TBD | 🟡 Baseline Needed |
| Latency p95 (headless) | <3500ms | TBD | 🟡 Baseline Needed |

### Operational Metrics
| Metric | Target | Status |
|--------|--------|--------|
| Availability | ≥99.5% | 🟡 To Measure |
| LLM Budget | ≤10 EUR/day | 🟡 To Track |
| Headless Usage | ≤25% | 🟡 To Track |
| Cache Hit Rate | ≥70% | 🟡 To Measure |

---

## 🎯 Critical Path (MUST NOT BLOCK)

### P0 Blockers (Weeks 0-1)
1. ✅ Create `riptide-utils` - BLOCKS: All subsequent work
2. ✅ Create `server.yaml` - BLOCKS: Config precedence
3. ✅ Enhance `riptide-config` - BLOCKS: API v1
4. ✅ CI baseline - BLOCKS: Validation

### P1 Deliverables (Weeks 1-3)
5. ✅ `riptide-api-types` - BLOCKS: v1 routes
6. ✅ `/api/v1` routes - BLOCKS: CLI, shims
7. ✅ Legacy shims - BLOCKS: Migration
8. ✅ Schema registry - BLOCKS: Validation
9. ✅ Adapters - BLOCKS: Entity storage
10. ✅ Validation engine - BLOCKS: Quality

### P2 Features (Weeks 3-6)
11. ⚙️ Enhanced facade
12. 📊 Diagnostics
13. 🔍 Discovery
14. ✅ Testing

### P3 Polish (Weeks 6-12)
15. 🔒 Security hardening
16. 🚀 Deployment
17. 📚 Documentation

---

## 🚨 Anti-Patterns to Avoid

### ❌ DON'T: Rewrite Existing Code
```rust
// WRONG
pub fn new_extraction_logic() {
    // Brand new implementation
}

// RIGHT
pub use riptide_extraction::extract;
```

### ❌ DON'T: Duplicate Business Logic
```rust
// WRONG
async fn extract_v1(url: String) {
    let html = fetch(url);  // New implementation
    parse(html);            // New implementation
}

// RIGHT
async fn extract_v1(url: String) {
    riptide_extraction::extract(url).await
}
```

### ❌ DON'T: Break Existing Tests
```bash
# WRONG
cargo test  # Failing tests ignored

# RIGHT
cargo test  # All tests pass or failures documented
```

---

## 📁 File Structure (Complete)

```
/workspaces/eventmesh/
├── server.yaml                        # NEW
├── server.staging.yaml                # NEW
├── server.canary.yaml                 # NEW
├── openapi.yaml                       # NEW
├── schemas/                           # NEW
│   ├── registry.json
│   ├── events.v1.json
│   ├── jobs.v1.json
│   ├── options.schema.json
│   └── diagnostics.schema.json
├── dashboards/                        # NEW
│   ├── grafana/
│   │   ├── Overview.json
│   │   ├── Headless_LLM.json
│   │   ├── Pipeline.json
│   │   ├── Cache.json
│   │   └── Jobs.json
│   └── alerts.yml
├── docs/
│   ├── roadmap/
│   │   └── MASTER-REFACTOR-ROADMAP.md
│   ├── baseline-build.log             # NEW
│   ├── baseline-tests.log             # NEW
│   ├── metrics.md                     # NEW
│   └── migrate-v1.md                  # NEW
├── crates/
│   ├── riptide-utils/                 # NEW
│   │   └── src/
│   │       ├── redis.rs
│   │       ├── http.rs
│   │       ├── error.rs
│   │       └── time.rs
│   ├── riptide-api-types/             # NEW
│   │   └── src/v1/
│   │       ├── extract.rs
│   │       ├── crawl.rs
│   │       └── common.rs
│   ├── riptide-schemas/               # NEW
│   ├── riptide-adapters/              # NEW
│   ├── riptide-validation/            # NEW
│   ├── riptide-api/
│   │   └── src/
│   │       ├── routes/v1/             # NEW
│   │       └── shims.rs               # NEW
│   ├── riptide-facade/
│   │   └── src/
│   │       └── pipeline.rs            # ENHANCED
│   ├── [all other existing crates]
└── .github/workflows/
    ├── ci.yml                         # NEW
    └── nightly.yml                    # NEW
```

---

## 🎯 Next Actions (Immediate)

### Week 0 (This Week)
1. **Create `riptide-utils` crate** (W0.1)
2. **Create `server.yaml`** (W0.2)
3. **Enhance `riptide-config`** (W0.3)
4. **Capture CI baseline** (W0.4)

### Week 1
5. **Create `riptide-api-types`** (W1.1)
6. **Implement `/api/v1` routes** (W1.2)
7. **Create legacy shims** (W1.3)

### Week 2
8. **Create schema registry** (W2.1)
9. **Create adapters** (W2.2)
10. **Create validation engine** (W2.3)

---

## 📞 Getting Help

- **Architecture Questions:** Review Doc 2 (Architecture & Crate Layout)
- **Implementation Details:** Review workstream docs (W*.*)
- **Testing Questions:** Review Phase 6 (Testing & CI)
- **Deployment Questions:** Review Phase 8 (Migration & Release)

---

**Document Version:** 1.0
**Last Updated:** 2025-11-04
**Status:** 🔴 Phase 0 Not Started
**Next Review:** Weekly during implementation
