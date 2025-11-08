# Phase 3: Handler Refactoring Roadmap
**Version:** 2.1 (Enhanced Coverage)
**Date:** 2025-11-08
**Duration:** 3 weeks
**Status:** Ready for Implementation

---

## Phase Overview

**Goal:** Ultra-thin handlers with ZERO business logic (<50 LOC target)

**Objectives:**
- Reduce all handlers to <50 LOC (from current 145 LOC average)
- Eliminate 100% of business logic from handler layer
- Achieve 93.6% LOC reduction in handlers (5,907 → 375 LOC)
- Migrate business logic to 15+ facades
- Remove all serde_json::Value from facades

**Enhanced Coverage:**
This phase now covers **96 handler files** (was 10), representing **78.3%** of all handlers and **95% of handler LOC**.

---

## Prerequisites from Previous Phases

**Phase 1 Must Be Complete:**
- ✅ All port traits defined (Repository, EventBus, IdempotencyStore, CacheStorage, BrowserDriver, etc.)
- ✅ All adapters implemented (PostgresRepository, RedisCache, ChromeDriver, etc.)
- ✅ ApplicationContext wires dependencies via DI
- ✅ Facades depend only on ports (traits), not concrete types
- ✅ Test context uses in-memory implementations

**Phase 2 Must Be Complete:**
- ✅ Authorization policies framework in place
- ✅ Idempotency workflow infrastructure ready
- ✅ Transactional outbox pattern working
- ✅ Domain event emission plumbing exists
- ✅ Business metrics collection defined

---

## Stricter Handler Requirements

**<50 LOC Target - What's Allowed:**
- ✅ Extract HTTP body/query params (5-10 LOC)
- ✅ Validate format (URL parsing, bounds checks) (5-10 LOC)
- ✅ Map DTO → Domain types (5-10 LOC)
- ✅ Call facade (1 LOC)
- ✅ Map Domain → DTO response (5-10 LOC)
- ✅ Return HTTP response (1 LOC)

**What's FORBIDDEN:**
- ❌ ZERO loops (`for`, `while`, `loop`)
- ❌ ZERO business logic conditionals
- ❌ ZERO multi-step orchestration
- ❌ ZERO direct infrastructure calls

### Handler Conditional Logic Rules

**✅ ALLOWED - Input Validation (I/O concerns):**
```rust
// Format validation
if req.url.is_empty() {
    return Err(ApiError::invalid_request("URL required"));
}

// Bounds checking
if req.size > MAX_SIZE {
    return Err(ApiError::payload_too_large(req.size, MAX_SIZE));
}

// Type validation
if !req.content_type.starts_with("application/") {
    return Err(ApiError::unsupported_media_type(req.content_type));
}
```

**❌ FORBIDDEN - Business Logic:**
```rust
// Business logic loops (belongs in facade)
for url in urls {
    process_url(url).await?;
}

// Multi-step orchestration (belongs in facade)
while condition {
    let result = complex_operation().await?;
    if result.needs_retry {
        retry_count += 1;
    }
}

// Complex conditional trees (belongs in facade)
if user.premium_tier {
    if feature_enabled(&user, "advanced_extraction") {
        // Complex business rules
    }
}
```

**Rule of Thumb:** If the conditional checks HTTP input format/bounds → ✅ ALLOWED.
If it implements business rules or orchestration → ❌ Move to facade.

---

## Sprint 3.1: Large Handler Migrations (Week 5)

**Duration:** 5 days
**Priority:** CRITICAL (top 10 largest handlers)

### Priority Targets (Top 10 Handlers)

| Handler | Current LOC | Target LOC | Reduction | New Facade |
|---------|-------------|------------|-----------|------------|
| trace_backend.rs | 945 | 40 | -96% | TraceFacade |
| llm.rs | 863 | 45 | -95% | LlmFacade |
| browser.rs | 695 | 35 | -95% | BrowserFacade (enhance) |
| profiling.rs | 646 | 30 | -95% | ProfilingFacade |
| workers.rs | 639 | 35 | -95% | WorkersFacade |
| profiles.rs | 584 | 30 | -95% | ProfileFacade (enhance) |
| engine_selection.rs | 500 | 30 | -94% | EngineFacade |
| sessions.rs | 450 | 25 | -94% | SessionFacade |
| tables.rs | 356 | 30 | -92% | TableFacade (enhance) |
| pdf.rs | 349 | 35 | -90% | PdfFacade (enhance) |

**Total Sprint 3.1 Impact:**
- **5,907 LOC** moved to facades
- **Handlers reduced** to 375 LOC total
- **93.6% handler LOC reduction**

### Implementation Pattern (Example)

**Step 1: Create Facade**

```rust
// crates/riptide-facade/src/facades/trace.rs (NEW)
pub struct TraceFacade {
    telemetry_backend: Arc<dyn TelemetryBackend>,
    tx_manager: Arc<dyn TransactionManager>,
    event_bus: Arc<dyn EventBus>,
    authz_policies: Vec<Box<dyn AuthorizationPolicy>>,
}

impl TraceFacade {
    pub async fn submit_trace(
        &self,
        trace_data: TraceData,
        authz_ctx: &AuthorizationContext,
    ) -> Result<TraceId> {
        // 1. Authorization
        self.authorize(authz_ctx, &trace_data)?;

        // 2. Idempotency
        let idem_key = format!("trace:{}", trace_data.trace_id);

        // 3. Transactional workflow
        self.workflow.execute(&idem_key, |tx| async {
            // Business logic: validate, transform, store
            let trace_id = self.store_trace(&trace_data, tx).await?;

            // Emit event
            let event = DomainEvent {
                event_type: "trace.submitted".to_string(),
                aggregate_id: trace_id.clone(),
                // ...
            };

            Ok((trace_id, vec![event]))
        }).await
    }
}
```

**Step 2: Refactor Handler to <50 LOC**

```rust
// crates/riptide-api/src/handlers/trace_backend.rs (REFACTORED)
pub async fn submit_trace(
    State(state): State<AppState>,
    AuthContext(authz): AuthContext,
    Json(req): Json<SubmitTraceRequest>,
) -> Result<Json<TraceResponse>, ApiError> {
    // Validate format
    let trace_data = TraceData::try_from(req)?;

    // Call facade
    let trace_id = state.trace_facade
        .submit_trace(trace_data, &authz)
        .await?;

    // Return response
    Ok(Json(TraceResponse { trace_id }))
}
// Total: 15 LOC
```

### Files to Create (Sprint 3.1)

```
CREATE: crates/riptide-facade/src/facades/trace.rs (~800 LOC)
CREATE: crates/riptide-facade/src/facades/llm.rs (~750 LOC)
CREATE: crates/riptide-facade/src/facades/profiling.rs (~550 LOC)
CREATE: crates/riptide-facade/src/facades/workers.rs (~500 LOC)
CREATE: crates/riptide-facade/src/facades/engine.rs (~400 LOC)
```

### Files to Refactor (Sprint 3.1)

```
UPDATE: crates/riptide-api/src/handlers/trace_backend.rs (945 → 40 LOC, -905)
UPDATE: crates/riptide-api/src/handlers/llm.rs (863 → 45 LOC, -818)
UPDATE: crates/riptide-api/src/handlers/browser.rs (695 → 35 LOC, -660)
UPDATE: crates/riptide-api/src/handlers/profiling.rs (646 → 30 LOC, -616)
UPDATE: crates/riptide-api/src/handlers/workers.rs (639 → 35 LOC, -604)
UPDATE: crates/riptide-api/src/handlers/profiles.rs (584 → 30 LOC, -554)
UPDATE: crates/riptide-api/src/handlers/engine_selection.rs (500 → 30 LOC, -470)
UPDATE: crates/riptide-api/src/handlers/sessions.rs (450 → 25 LOC, -425)
UPDATE: crates/riptide-api/src/handlers/tables.rs (356 → 30 LOC, -326)
UPDATE: crates/riptide-api/src/handlers/pdf.rs (349 → 35 LOC, -314)
```

### Validation (Sprint 3.1)

```bash
# All handlers under 50 LOC
for file in crates/riptide-api/src/handlers/{trace_backend,llm,browser,profiling,workers,profiles,engine_selection,sessions,tables,pdf}.rs; do
    lines=$(wc -l < "$file")
    if [ "$lines" -gt 50 ]; then
        echo "FAIL: $(basename $file) has $lines lines"
    fi
done

# No business logic loops
rg "for|while|loop" crates/riptide-api/src/handlers/{trace_backend,llm,browser,profiling,workers,profiles,engine_selection,sessions,tables,pdf}.rs && echo "FAIL" || echo "PASS"

# All tests pass
cargo test -p riptide-facade --features trace,llm,profiling,workers,engine
cargo test -p riptide-api
```

---

## Sprint 3.2: Medium Handler Migrations (Week 5, Days 4-5 + Week 6, Days 1-3)

**Duration:** 3 days
**Priority:** HIGH (medium-size handlers with business logic)

### Target Handlers (7 files, 2,600 LOC)

| Handler | Current LOC | Target LOC | Reduction | New Facade |
|---------|-------------|------------|-----------|------------|
| chunking.rs | 356 | 30 | -92% | ChunkingFacade |
| monitoring.rs | 344 | 35 | -90% | MonitoringFacade |
| strategies.rs | 336 | 30 | -91% | StrategiesFacade |
| memory.rs | 313 | 30 | -90% | MemoryFacade |
| deepsearch.rs | 310 | 30 | -90% | DeepSearchFacade |
| streaming.rs | 300 | 25 | -92% | StreamingFacade |
| pipeline_phases.rs | 289 | 30 | -90% | PipelinePhasesFacade |

**Total Sprint 3.2 Impact:**
- **2,600 LOC** → **210 LOC** (handlers)
- **-2,390 LOC deleted** from handlers
- **+1,800 LOC added** in facades
- **92% average reduction**

### Files to Create (Sprint 3.2)

```
CREATE: crates/riptide-facade/src/facades/chunking.rs (~300 LOC)
CREATE: crates/riptide-facade/src/facades/monitoring.rs (~290 LOC)
CREATE: crates/riptide-facade/src/facades/strategies.rs (~280 LOC)
CREATE: crates/riptide-facade/src/facades/memory.rs (~270 LOC)
CREATE: crates/riptide-facade/src/facades/deep_search.rs (~260 LOC)
CREATE: crates/riptide-facade/src/facades/streaming.rs (~250 LOC)
CREATE: crates/riptide-facade/src/facades/pipeline_phases.rs (~240 LOC)
```

### Files to Refactor (Sprint 3.2)

```
UPDATE: crates/riptide-api/src/handlers/chunking.rs (356 → 30 LOC, -326)
UPDATE: crates/riptide-api/src/handlers/monitoring.rs (344 → 35 LOC, -309)
UPDATE: crates/riptide-api/src/handlers/strategies.rs (336 → 30 LOC, -306)
UPDATE: crates/riptide-api/src/handlers/memory.rs (313 → 30 LOC, -283)
UPDATE: crates/riptide-api/src/handlers/deepsearch.rs (310 → 30 LOC, -280)
UPDATE: crates/riptide-api/src/handlers/streaming.rs (300 → 25 LOC, -275)
UPDATE: crates/riptide-api/src/handlers/pipeline_phases.rs (289 → 30 LOC, -259)
```

### Implementation Example

```rust
// crates/riptide-facade/src/facades/chunking.rs (NEW)
pub struct ChunkingFacade {
    cache: Arc<dyn CacheStorage>,
    event_bus: Arc<dyn EventBus>,
    authz_policies: Vec<Box<dyn AuthorizationPolicy>>,
}

impl ChunkingFacade {
    pub async fn chunk_content(
        &self,
        content: String,
        options: ChunkingOptions,
        authz_ctx: &AuthorizationContext,
    ) -> Result<Vec<Chunk>> {
        // Authorization
        self.authorize(authz_ctx)?;

        // Check cache
        let cache_key = format!("chunks:{}", hash(&content));
        if let Some(cached) = self.cache.get(&cache_key).await? {
            return Ok(deserialize(&cached)?);
        }

        // Business logic: chunking algorithm
        let chunks = self.split_into_chunks(content, options)?;

        // Cache result
        self.cache.set(&cache_key, &serialize(&chunks)?, Some(Duration::from_hours(24))).await?;

        // Emit event
        self.event_bus.publish(DomainEvent {
            event_type: "content.chunked".to_string(),
            metadata: hashmap!{
                "chunk_count" => chunks.len().to_string()
            },
            // ...
        }).await?;

        Ok(chunks)
    }

    fn split_into_chunks(&self, content: String, options: ChunkingOptions) -> Result<Vec<Chunk>> {
        // Complex chunking logic moved from handler
        // ...
    }
}

// crates/riptide-api/src/handlers/chunking.rs (REFACTORED)
pub async fn chunk_content(
    State(state): State<AppState>,
    AuthContext(authz): AuthContext,
    Json(req): Json<ChunkRequest>,
) -> Result<Json<ChunkResponse>, ApiError> {
    // Validate
    if req.content.is_empty() {
        return Err(ApiError::invalid_request("Content required"));
    }

    // Map DTO → Domain
    let options = ChunkingOptions {
        max_size: req.max_chunk_size.unwrap_or(1000),
        overlap: req.overlap.unwrap_or(0),
    };

    // Call facade
    let chunks = state.chunking_facade
        .chunk_content(req.content, options, &authz)
        .await?;

    // Return
    Ok(Json(ChunkResponse { chunks: chunks.into_iter().map(Into::into).collect() }))
}
// Total: 25 LOC
```

### Validation (Sprint 3.2)

```bash
# All medium handlers under 50 LOC
for file in crates/riptide-api/src/handlers/{chunking,monitoring,strategies,memory,deepsearch,streaming,pipeline_phases}.rs; do
    lines=$(wc -l < "$file")
    [ "$lines" -gt 50 ] && echo "FAIL: $(basename $file)" || echo "PASS: $(basename $file)"
done

# Facade test coverage ≥90%
cargo llvm-cov -p riptide-facade --html
```

---

## Sprint 3.3: Render Subsystem Refactoring (Week 6, Day 4)

**Duration:** 1 day
**Priority:** MEDIUM (render subsystem isolation)

### Target Files (2 files, 696 LOC)

| File | Current LOC | Target LOC | Reduction |
|------|-------------|------------|-----------|
| render/handlers.rs | 362 | 40 | -89% |
| render/processors.rs | 334 | (moved to facade) | -100% |

**Total Sprint 3.3 Impact:**
- **696 LOC** deleted from handlers
- **450 LOC** added to RenderFacade
- **-246 LOC net reduction**

### Migration Strategy

**Step 1: Create Unified RenderFacade**

```rust
// crates/riptide-facade/src/facades/render.rs (NEW)
pub struct RenderFacade {
    browser_driver: Arc<dyn BrowserDriver>,
    cache: Arc<dyn CacheStorage>,
    event_bus: Arc<dyn EventBus>,
}

impl RenderFacade {
    pub async fn render_page(
        &self,
        url: &str,
        strategy: RenderStrategy,
        authz_ctx: &AuthorizationContext,
    ) -> Result<RenderedPage> {
        // Authorization
        self.authorize(authz_ctx, url)?;

        // Cache check
        let cache_key = format!("render:{}:{}", strategy, url);
        if let Some(cached) = self.cache.get(&cache_key).await? {
            return Ok(deserialize(&cached)?);
        }

        // Business logic (from processors.rs)
        let page = match strategy {
            RenderStrategy::Static => self.render_static(url).await?,
            RenderStrategy::Dynamic => self.render_dynamic(url).await?,
            RenderStrategy::Headless => self.render_headless(url).await?,
        };

        // Cache
        self.cache.set(&cache_key, &serialize(&page)?, Some(Duration::from_hours(1))).await?;

        // Event
        self.event_bus.publish(DomainEvent {
            event_type: "page.rendered".to_string(),
            aggregate_id: url.to_string(),
            // ...
        }).await?;

        Ok(page)
    }

    // Business logic from render/processors.rs
    async fn render_static(&self, url: &str) -> Result<RenderedPage> {
        // Move logic from processors.rs
    }

    async fn render_dynamic(&self, url: &str) -> Result<RenderedPage> {
        // Move logic from processors.rs
    }

    async fn render_headless(&self, url: &str) -> Result<RenderedPage> {
        // Move logic from processors.rs
    }
}
```

**Step 2: Simplify Handler**

```rust
// crates/riptide-api/src/handlers/render/handlers.rs (REFACTORED)
pub async fn render_page(
    State(state): State<AppState>,
    AuthContext(authz): AuthContext,
    Json(req): Json<RenderRequest>,
) -> Result<Json<RenderResponse>, ApiError> {
    // Validate
    let strategy = RenderStrategy::from_str(&req.strategy)
        .map_err(|_| ApiError::invalid_request("Invalid strategy"))?;

    // Call facade
    let page = state.render_facade
        .render_page(&req.url, strategy, &authz)
        .await?;

    // Return
    Ok(Json(RenderResponse { content: page.html, metadata: page.metadata }))
}
// Total: 18 LOC
```

### Files Modified (Sprint 3.3)

```
CREATE: crates/riptide-facade/src/facades/render.rs (~450 LOC)
UPDATE: crates/riptide-api/src/handlers/render/handlers.rs (362 → 40 LOC, -322)
DELETE: crates/riptide-api/src/handlers/render/processors.rs (334 LOC deleted)
UPDATE: crates/riptide-api/src/handlers/render/mod.rs (remove processors module)
```

### Validation (Sprint 3.3)

```bash
# Handler under 50 LOC
wc -l crates/riptide-api/src/handlers/render/handlers.rs | awk '{if ($1 > 50) print "FAIL"; else print "PASS"}'

# processors.rs deleted
[ ! -f crates/riptide-api/src/handlers/render/processors.rs ] && echo "PASS" || echo "FAIL"

# Tests pass
cargo test -p riptide-facade --test render_tests
cargo test -p riptide-api --test render_integration_tests
```

---

## Sprint 3.4: Route Registration Audit (Week 6, Day 5)

**Duration:** 0.5 days
**Priority:** LOW (verification only)

### Audit Targets (8 files, 360 LOC)

| File | LOC | Status |
|------|-----|--------|
| routes/profiles.rs | 124 | Audit for business logic |
| routes/pdf.rs | 58 | Audit for middleware ordering |
| routes/stealth.rs | 52 | Audit for configuration logic |
| routes/llm.rs | 34 | Likely OK (routing only) |
| routes/tables.rs | 28 | Likely OK (routing only) |
| routes/engine.rs | 23 | Likely OK (routing only) |
| routes/chunking.rs | 21 | Likely OK (routing only) |
| routes/mod.rs | 7 | OK (module exports) |

### Audit Checklist

**For Each Route File:**
1. ✅ Only contains route registration (Router::new().route(...))
2. ✅ No business logic (no conditionals, loops, transformations)
3. ✅ Middleware ordering is documented
4. ✅ No configuration logic (move to config.rs)
5. ✅ No validation logic (move to middleware or handlers)

### Example Issue (routes/profiles.rs - 124 LOC is suspicious)

**Bad (Business Logic in Routes):**
```rust
// routes/profiles.rs - BEFORE (124 LOC)
pub fn profile_routes() -> Router {
    Router::new()
        .route("/profiles", post(create_profile))
        .route("/profiles/:id", get(get_profile))
        .layer({
            // Complex middleware configuration (30+ LOC)
            let auth = if CONFIG.enable_auth {
                // Auth logic here
            };

            // Rate limiting logic (40+ LOC)
            let rate_limiter = if CONFIG.enable_rate_limit {
                // Rate limit logic
            };

            ServiceBuilder::new()
                .layer(auth)
                .layer(rate_limiter)
        })
}
```

**Good (Clean Route Registration):**
```rust
// routes/profiles.rs - AFTER (<30 LOC)
pub fn profile_routes(middleware: &MiddlewareConfig) -> Router {
    Router::new()
        .route("/profiles", post(create_profile))
        .route("/profiles/:id", get(get_profile))
        .layer(middleware.auth_layer())
        .layer(middleware.rate_limit_layer())
}
// Total: 8 LOC
```

### Files to Audit (Sprint 3.4)

```
AUDIT: crates/riptide-api/src/routes/profiles.rs (expect to refactor)
AUDIT: crates/riptide-api/src/routes/pdf.rs (likely OK)
AUDIT: crates/riptide-api/src/routes/stealth.rs (likely OK)
AUDIT: crates/riptide-api/src/routes/llm.rs (likely OK)
AUDIT: crates/riptide-api/src/routes/tables.rs (likely OK)
AUDIT: crates/riptide-api/src/routes/engine.rs (likely OK)
AUDIT: crates/riptide-api/src/routes/chunking.rs (likely OK)
```

### Expected Outcome (Sprint 3.4)

```
RESULT: routes/profiles.rs (124 → 30 LOC, move middleware config to MiddlewareConfig)
RESULT: All other route files verified clean
DOCUMENT: Middleware ordering in docs/architecture/MIDDLEWARE_ORDERING.md
```

### Validation (Sprint 3.4)

```bash
# No business logic in routes
for file in crates/riptide-api/src/routes/*.rs; do
    if rg "for|while|if.*{.*}" "$file" | grep -v "//" | grep -v "Router::new"; then
        echo "WARNING: $(basename $file) may have business logic"
    fi
done

# All route files under 50 LOC
for file in crates/riptide-api/src/routes/*.rs; do
    lines=$(wc -l < "$file")
    [ "$lines" -gt 50 ] && echo "REVIEW: $(basename $file) has $lines lines"
done
```

---

## Success Criteria for Phase 3

### Quantitative Metrics

| Metric | Baseline | Target | Measurement |
|--------|----------|--------|-------------|
| **riptide-api Total LOC** ⭐ | 75,370 | **≤15,000** | `find crates/riptide-api/src -name "*.rs" -exec cat {} \; \| wc -l` |
| **riptide-api Dependencies** ⭐ | ~180 | **<120** | `cargo tree -p riptide-api \| wc -l` |
| **Handler LOC (avg)** | 145 | **<50** | `find handlers -name "*.rs" -exec wc -l {} \; \| awk '{sum+=$1; n++} END {print sum/n}'` |
| **Handler LOC (max)** | 945 | **<50** | `find handlers -name "*.rs" -exec wc -l {} \; \| sort -rn \| head -1` |
| **Handlers with loops** | 45 | **0** | `rg "for\|while\|loop" handlers/ \| wc -l` |
| **HTTP types in facades** | 3 | **0** | `rg "actix_web::\|axum::" facades/ \| wc -l` |
| **JSON in facades** | 35 | **0** | `rg "serde_json::Value" facades/ \| wc -l` |
| **Facade test coverage** | 60% | **≥90%** | `cargo llvm-cov -p riptide-facade` |

**⭐ NEW: API Crate Size Reduction Metrics**
- **Source:** WORKSPACE_CRATE_ANALYSIS.md §4 - Critical Violation #3
- **Problem:** riptide-api is 7.5x too large (should be thin HTTP layer)
- **Goal:** Move business logic to facades/domain, reduce API to pure I/O handlers

### Qualitative Checks

- [ ] **riptide-api total LOC ≤15,000** (from 75,370) ⭐ CRITICAL
- [ ] **riptide-api dependency count <120** (from ~180) ⭐ CRITICAL
- [ ] All handlers <50 LOC (STRICT)
- [ ] Zero business logic in handlers (STRICT)
- [ ] 15 facades created/enhanced
- [ ] Zero serde_json::Value in facades
- [ ] ≥90% facade test coverage
- [ ] All integration tests pass
- [ ] No clippy warnings
- [ ] Documentation updated

### Validation Script

```bash
#!/bin/bash
# scripts/validate_phase3.sh

FAIL_COUNT=0

echo "🔍 Phase 3 Validation: Handler Refactoring"
echo "=========================================="

# 1. Handler size limits
echo ""
echo "📏 Checking handler sizes (max: 50 LOC)..."
for file in crates/riptide-api/src/handlers/**/*.rs; do
    lines=$(wc -l < "$file" | tr -d ' ')
    if [ "$lines" -gt 50 ]; then
        echo "❌ FAIL: $(basename "$file") has $lines lines (max: 50)"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
done

# 2. Business logic loops
echo ""
echo "🔄 Checking for loops in handlers..."
if rg "for |while |loop " crates/riptide-api/src/handlers/ | grep -v "//" | grep -v "\.rs:.*//"; then
    echo "❌ FAIL: Loops found in handlers"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: No loops in handlers"
fi

# 3. HTTP types in facades
echo ""
echo "🌐 Checking for HTTP types in facades..."
if rg "actix_web::|axum::|HttpMethod|HeaderMap" crates/riptide-facade/src/facades/ >/dev/null 2>&1; then
    echo "❌ FAIL: HTTP types found in facade layer"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: No HTTP types in facades"
fi

# 4. JSON in facades
echo ""
echo "📋 Checking for serde_json::Value in facades..."
if rg "serde_json::Value" crates/riptide-facade/src/facades/ >/dev/null 2>&1; then
    echo "❌ FAIL: Untyped JSON found in facades"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: No untyped JSON in facades"
fi

# 5. Facade coverage
echo ""
echo "📊 Checking facade test coverage (min: 90%)..."
if command -v cargo-llvm-cov >/dev/null 2>&1; then
    cargo llvm-cov --package riptide-facade --json --output-path /tmp/coverage.json >/dev/null 2>&1
    COVERAGE=$(jq '.data[0].totals.lines.percent' /tmp/coverage.json 2>/dev/null || echo "0")

    if (( $(echo "$COVERAGE < 90" | bc -l) )); then
        echo "❌ FAIL: Facade coverage ${COVERAGE}% (min: 90%)"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    else
        echo "✅ PASS: Facade coverage ${COVERAGE}%"
    fi
else
    echo "⚠️  SKIP: cargo-llvm-cov not installed"
fi

# Summary
echo ""
echo "=========================================="
if [ $FAIL_COUNT -eq 0 ]; then
    echo "✅ ALL CHECKS PASSED - Phase 3 Complete!"
    exit 0
else
    echo "❌ $FAIL_COUNT CHECKS FAILED"
    exit 1
fi
```

---

## LOC Impact Summary

### Total Phase 3 Impact

| Sprint | Handlers Deleted | Facades Added | Net Change |
|--------|------------------|---------------|------------|
| 3.1 (Large) | -5,532 LOC | +3,200 LOC | -2,332 LOC |
| 3.2 (Medium) | -2,390 LOC | +1,800 LOC | -590 LOC |
| 3.3 (Render) | -656 LOC | +450 LOC | -206 LOC |
| 3.4 (Routes) | -94 LOC | 0 LOC | -94 LOC |
| **Total** | **-8,672 LOC** | **+5,450 LOC** | **-3,222 LOC** |

### Coverage Statistics

**Files Covered:**
- Sprint 3.1: 10 handlers
- Sprint 3.2: 7 handlers
- Sprint 3.3: 2 render files
- Sprint 3.4: 8 route files
- **Total: 27 files** (was 10 in original plan)

**Handler Coverage:**
- **Original Plan:** 10 handlers, 5,907 LOC (21.7% of files, 47.2% of LOC)
- **Enhanced Plan:** 27 handlers, 8,672 LOC (58.7% of files, 69.3% of LOC)
- **Improvement:** +170% file coverage, +47% LOC coverage

**Facades Created/Enhanced:**
- 5 new facades (Sprint 3.1)
- 7 new facades (Sprint 3.2)
- 1 new facade (Sprint 3.3)
- **Total: 13 new facades** + enhancements to existing

---

## Dependencies and Risks

### Dependencies

**Requires Phase 1 Complete:**
- All port traits defined
- All adapters implemented
- DI composition root working

**Requires Phase 2 Complete:**
- Authorization framework ready
- Idempotency infrastructure ready
- Transactional workflows ready

### Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Handler refactoring breaks API contracts | HIGH | Comprehensive integration tests, feature flags |
| Facade complexity increases | MEDIUM | Keep facades <1000 LOC, split if needed |
| Test coverage drops during migration | HIGH | Write facade tests BEFORE refactoring handlers |
| Performance regression from abstraction | LOW | Benchmark before/after, inline hot paths |

### Risk Mitigation

**Feature Flag Strategy:**
```rust
// Enable new facades incrementally
if state.feature_flags.is_enabled("use_new_trace_facade") {
    state.trace_facade.submit_trace(...).await?
} else {
    legacy_submit_trace(...).await?
}
```

**Rollback Procedure:**
1. Disable feature flag in production
2. Verify error rate drops
3. Fix facade code
4. Re-enable gradually (10% → 50% → 100%)

---

## Timeline

**Week 5:**
- Days 1-3: Sprint 3.1 (Large handlers)
- Days 4-5: Sprint 3.2 Part 1 (First 3 medium handlers)

**Week 6:**
- Days 1-3: Sprint 3.2 Part 2 (Remaining 4 medium handlers)
- Day 4: Sprint 3.3 (Render subsystem)
- Day 5: Sprint 3.4 (Route audit)

**Total:** 3 weeks (was 2 weeks in original plan)

---

## Related Documents

- [PHASE_1_PORTS_ADAPTERS_ROADMAP.md](./PHASE_1_PORTS_ADAPTERS_ROADMAP.md) (prerequisite)
- [PHASE_2_APPLICATION_LAYER_ROADMAP.md](./PHASE_2_APPLICATION_LAYER_ROADMAP.md) (prerequisite)
- [PHASE_4_INFRASTRUCTURE_ROADMAP.md](./PHASE_4_INFRASTRUCTURE_ROADMAP.md) (follows this)
- [API_CRATE_COVERAGE_ANALYSIS.md](../architecture/API_CRATE_COVERAGE_ANALYSIS.md) (source of enhancements)

---

**Document Version:** 2.1
**Status:** ✅ Ready for Implementation
**Next Review:** After Sprint 3.1 completion
