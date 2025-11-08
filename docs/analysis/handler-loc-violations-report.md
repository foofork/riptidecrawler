# Handler LOC Violations Analysis Report

**Date:** 2025-11-08
**Analysis:** All handlers in `crates/riptide-api/src/handlers/`
**Target:** <50 LOC per handler (excluding mod.rs and test modules)

---

## Executive Summary

**Total Handlers Analyzed:** 43 files
**Violations Found:** 27 files (62.8%)
**Compliant Handlers:** 16 files (37.2%)
**Critical Violations (>200 LOC):** 10 files

### Severity Breakdown
- **CRITICAL** (>300 LOC): 6 files - Requires immediate facade extraction
- **HIGH** (100-300 LOC): 10 files - Multi-step extraction required
- **MEDIUM** (51-99 LOC): 11 files - Single-step extraction possible

---

## Critical Violations (>300 LOC)

### 1. trace_backend.rs (945 LOC) ⚠️ CRITICAL
**Current Status:** Massive monolith with backend implementations
**Violations:**
- Lines 47-111: 3 struct definitions (CompleteTrace, TraceSpan, SpanEventData)
- Lines 112-317: InMemoryTraceBackend implementation (205 LOC)
- Lines 318-826: OtlpTraceBackend implementation (508 LOC)
- Lines 827-945: Helper functions and tree building (118 LOC)

**Extraction Plan:**
```
✅ ALREADY EXISTS: crates/riptide-facade/src/facades/trace.rs (30KB)
ACTION: Replace trace_backend.rs with thin handler calling TraceFacade
TARGET: <40 LOC (like trace_backend_refactored.rs @ 74 LOC, needs trim)
EXTRACT TO:
  - DTOs → crates/riptide-types/src/dto/trace.rs
  - Backend traits → crates/riptide-types/src/ports/trace_backend.rs
  - Implementations → crates/riptide-facade/src/backends/trace/
```

---

### 2. llm.rs (863 LOC) ⚠️ CRITICAL
**Current Status:** Complex provider management system
**Violations:**
- Lines 29-230: 10+ DTO structs (ProvidersResponse, ProviderInfo, etc.)
- Lines 243-299: get_current_provider_info() - 56 LOC
- Lines 300-375: list_providers() - 75 LOC
- Lines 376-551: switch_provider() - 175 LOC
- Lines 552-666: update_config() - 114 LOC
- Lines 667-863: get_config() and helpers - 196 LOC

**Extraction Plan:**
```
✅ ALREADY EXISTS: crates/riptide-facade/src/facades/llm.rs (24KB)
✅ GOOD EXAMPLES: llm_refactored.rs (58 LOC), llm_minimal.rs (54 LOC)
ACTION: Replace llm.rs with thin handlers like llm_minimal.rs
TARGET: <45 LOC per endpoint
EXTRACT TO:
  - DTOs → crates/riptide-types/src/dto/llm.rs
  - Business logic → Already in LlmFacade
```

---

### 3. health.rs (421 LOC) ⚠️ CRITICAL
**Violations:**
- Lines 34-420: Massive health() function with inline health checks
- Complex timeout handling and dependency validation
- Inline metrics collection and formatting

**Extraction Plan:**
```
CREATE: crates/riptide-facade/src/facades/health.rs
TARGET: <40 LOC handler
EXTRACT TO:
  - DTOs → crates/riptide-types/src/dto/health.rs (HealthResponse exists)
  - Health checks → HealthFacade::check_health()
  - Metrics → HealthFacade::collect_metrics()
```

---

### 4. telemetry.rs (422 LOC) ⚠️ CRITICAL
**Violations:**
- Lines 24-162: 6 DTO structs (TraceQueryParams, TraceMetadata, etc.)
- Lines 164-205: list_traces() - 41 LOC
- Lines 206-261: get_trace_tree() - 55 LOC
- Lines 262-422: get_telemetry_status() - 160 LOC (massive!)

**Extraction Plan:**
```
✅ TraceFacade already exists
ACTION: Split into 3 thin handlers (<40 LOC each)
EXTRACT TO:
  - DTOs → Share with trace_backend DTOs
  - Business logic → TraceFacade methods
```

---

### 5. render/handlers.rs (362 LOC) ⚠️ CRITICAL
**Special Case:** Must be <40 LOC after Sprint 3.3
**Violations:**
- Lines 17-100: render() function - 83 LOC
- Complex resource management
- Multiple rendering strategies inline

**Extraction Plan:**
```
✅ ALREADY EXISTS: crates/riptide-facade/src/facades/render.rs (17KB)
ACTION: Ultra-thin wrapper around RenderFacade
TARGET: <40 LOC (Sprint 3.3 requirement)
EXTRACT TO:
  - Resource management → RenderFacade
  - Strategy selection → RenderFacade
  - Processors → Already extracted to processors.rs
```

---

### 6. render/processors.rs (334 LOC) ⚠️ CRITICAL
**Violations:**
- Lines 1-100: process_static() implementation
- Lines 101-200: process_dynamic() implementation
- Lines 201-300: process_adaptive() implementation
- Lines 301-334: process_pdf() implementation

**Extraction Plan:**
```
ACTION: Move to facade layer (not handler responsibility)
TARGET: Remove file or reduce to <40 LOC dispatcher
EXTRACT TO:
  - crates/riptide-facade/src/render/processors/
  - Separate files per strategy
```

---

## High Severity Violations (100-300 LOC)

### 7. pipeline_metrics.rs (260 LOC)
**Extract to:** PipelineMetricsFacade
**Target:** <40 LOC

### 8. resources.rs (248 LOC)
**Extract to:** ResourceManagementFacade
**Target:** <40 LOC

### 9. admin.rs (194 LOC)
**Extract to:** AdminFacade
**Target:** <40 LOC

### 10. render/extraction.rs (190 LOC)
**Extract to:** ExtractionFacade (already exists partially)
**Target:** <40 LOC

### 11. spider.rs (160 LOC)
**Extract to:** SpiderFacade (exists)
**Target:** <40 LOC

### 12. crawl.rs (133 LOC)
**Extract to:** CrawlFacade (exists)
**Target:** <40 LOC

### 13. stubs.rs (118 LOC)
**Action:** Review if needed, likely test fixtures
**Target:** Move to tests/ or <40 LOC

### 14. search.rs (116 LOC)
**Extract to:** SearchFacade (exists)
**Target:** <40 LOC

### 15. render/models.rs (114 LOC)
**Action:** Move DTOs to riptide-types
**Target:** <30 LOC or remove

### 16. extract.rs (100 LOC)
**Extract to:** ExtractionFacade
**Target:** <40 LOC

---

## Medium Severity Violations (51-99 LOC)

### 17-27. Medium Priority Files
| File | LOC | Target | Facade |
|------|-----|--------|--------|
| workers.rs | 94 | <45 | WorkersFacade ✅ exists |
| profiles.rs | 88 | <45 | ProfileFacade ✅ exists |
| trace_backend_refactored.rs | 74 | <40 | TraceFacade ✅ exists |
| pdf.rs | 70 | <40 | PdfFacade ✅ exists |
| sessions.rs | 65 | <40 | SessionFacade ✅ exists |
| llm_refactored.rs | 58 | <45 | LlmFacade ✅ exists |
| profiling.rs | 57 | <40 | ProfilingFacade ✅ exists |
| browser.rs | 55 | <40 | BrowserFacade ✅ exists |
| llm_minimal.rs | 54 | <45 | LlmFacade ✅ exists |
| tables.rs | 51 | <40 | TableFacade ✅ exists |
| stealth.rs | 287 | <40 | StealthFacade (create) |

---

## Compliant Handlers (<50 LOC) ✅

**16 files meeting the requirement:**
- engine_selection.rs (50 LOC) - borderline
- pipeline_phases.rs (48 LOC)
- chunking.rs (47 LOC)
- render/strategies.rs (43 LOC)
- shared/spider.rs (40 LOC)
- utils.rs (36 LOC)
- streaming.rs (36 LOC)
- fetch.rs (33 LOC)
- deepsearch.rs (22 LOC)
- strategies.rs (21 LOC)
- monitoring.rs (18 LOC)
- admin_stub.rs (13 LOC)
- memory.rs (12 LOC)
- mod.rs files (exempt)

---

## Refactoring Priority Matrix

### Phase 1 (Sprint 3.1): IMMEDIATE - Core Infrastructure
1. **llm.rs** → Replace with llm_minimal.rs pattern (863 → ~45 LOC)
2. **trace_backend.rs** → Use trace_backend_refactored.rs pattern (945 → ~40 LOC)
3. **health.rs** → Extract HealthFacade (421 → ~40 LOC)
4. **telemetry.rs** → Use TraceFacade (422 → ~40 LOC)

### Phase 2 (Sprint 3.2): HIGH - Render Subsystem
5. **render/handlers.rs** → Ultra-thin wrapper (362 → <40 LOC) ⚠️ Required
6. **render/processors.rs** → Move to facade (334 → <40 LOC dispatcher)
7. **render/extraction.rs** → Use ExtractionFacade (190 → ~40 LOC)
8. **render/models.rs** → Move DTOs to types (114 → <30 LOC)

### Phase 3 (Sprint 3.3): MEDIUM - Business Logic
9. **pipeline_metrics.rs** → PipelineMetricsFacade (260 → ~40 LOC)
10. **resources.rs** → ResourceFacade (248 → ~40 LOC)
11. **admin.rs** → AdminFacade (194 → ~40 LOC)
12. **spider.rs** → SpiderFacade exists (160 → ~40 LOC)
13. **crawl.rs** → CrawlFacade exists (133 → ~40 LOC)
14. **search.rs** → SearchFacade exists (116 → ~40 LOC)
15. **extract.rs** → ExtractionFacade (100 → ~40 LOC)

### Phase 4 (Sprint 3.4): LOW - Final Cleanup
16-27. All medium violations (51-99 LOC) → Trim to <45 LOC

---

## Extraction Patterns (from successful refactors)

### Pattern 1: Ultra-Thin Handler (trace_backend_refactored.rs)
```rust
// 1. HTTP validation only (3-5 LOC)
if req.trace_id.is_empty() { return Err(...) }

// 2. Map DTO → Domain (5-10 LOC)
let domain = TraceData { ... };

// 3. Call facade (1 LOC)
let result = facade.method(domain, &authz_ctx).await?;

// 4. Map Domain → DTO (3-5 LOC)
Ok((StatusCode::OK, Json(response)))

// Total: ~15-25 LOC per endpoint
```

### Pattern 2: Minimal Handler (llm_minimal.rs)
```rust
// Inline DTOs for simple cases (10 LOC)
#[derive(Deserialize)] struct Req { ... }
#[derive(Serialize)] struct Resp { ... }

// Single focused endpoint (15-20 LOC)
pub async fn handler(...) -> Result<impl IntoResponse, ApiError> {
    validate()?;
    let domain = map_to_domain();
    let result = facade.execute(domain).await?;
    Ok(map_to_response(result))
}
// Total: ~30-40 LOC
```

### Anti-Pattern: Monolithic Handler (llm.rs, trace_backend.rs)
```rust
❌ 10+ struct definitions in handler
❌ Complex business logic (loops, conditions)
❌ Multiple backend implementations
❌ Inline helper functions
❌ 100+ LOC functions
```

---

## Recommendations

### Immediate Actions (This Sprint)
1. **Replace llm.rs** with llm_minimal.rs pattern → saves 808 LOC
2. **Replace trace_backend.rs** with trace_backend_refactored.rs → saves 870 LOC
3. **Extract health.rs** to HealthFacade → saves 380 LOC
4. **Extract telemetry.rs** to TraceFacade → saves 380 LOC

**Total LOC Reduction:** ~2,438 LOC (from 4 files)

### DTO Migration Strategy
```bash
# Move all handler DTOs to types
crates/riptide-types/src/dto/
  ├── trace.rs       # From trace_backend.rs, telemetry.rs
  ├── llm.rs         # From llm.rs
  ├── health.rs      # From health.rs
  ├── render.rs      # From render/models.rs
  ├── pipeline.rs    # From pipeline_metrics.rs
  └── admin.rs       # From admin.rs
```

### Facade Coverage Check
```bash
✅ Already exist (use them!):
  - LlmFacade, TraceFacade, RenderFacade
  - BrowserFacade, PdfFacade, ProfileFacade
  - TableFacade, WorkersFacade, SessionFacade
  - ExtractionFacade, SpiderFacade, CrawlFacade

🚧 Need creation:
  - HealthFacade, ResourceFacade, AdminFacade
  - PipelineMetricsFacade, StealthFacade
```

---

## Success Metrics

### Current State
- **Average Handler LOC:** 134 LOC
- **Median Handler LOC:** 94 LOC
- **Compliance Rate:** 37.2%

### Target State (After Refactor)
- **Average Handler LOC:** <40 LOC
- **Median Handler LOC:** <35 LOC
- **Compliance Rate:** 100%
- **LOC Reduction:** ~3,500 LOC moved to facade layer

### Validation Command
```bash
# Should output nothing when complete
find crates/riptide-api/src/handlers -name "*.rs" -not -name "mod.rs" -type f \
  -exec sh -c 'lines=$(wc -l < "$1"); [ $lines -gt 50 ] && echo "FAIL: $1 ($lines LOC)"' _ {} \;
```

---

## Appendix: Full Handler Inventory

| File | LOC | Status | Priority | Facade |
|------|-----|--------|----------|--------|
| trace_backend.rs | 945 | ❌ CRITICAL | P1 | TraceFacade ✅ |
| llm.rs | 863 | ❌ CRITICAL | P1 | LlmFacade ✅ |
| telemetry.rs | 422 | ❌ CRITICAL | P1 | TraceFacade ✅ |
| health.rs | 421 | ❌ CRITICAL | P1 | HealthFacade 🚧 |
| render/handlers.rs | 362 | ❌ CRITICAL | P2 | RenderFacade ✅ |
| render/processors.rs | 334 | ❌ CRITICAL | P2 | RenderFacade ✅ |
| stealth.rs | 287 | ❌ HIGH | P4 | StealthFacade 🚧 |
| pipeline_metrics.rs | 260 | ❌ HIGH | P3 | PipelineMetricsFacade 🚧 |
| resources.rs | 248 | ❌ HIGH | P3 | ResourceFacade 🚧 |
| shared/mod.rs | 201 | ⚠️ EXEMPT | - | Module file |
| admin.rs | 194 | ❌ HIGH | P3 | AdminFacade 🚧 |
| render/extraction.rs | 190 | ❌ HIGH | P2 | ExtractionFacade ✅ |
| spider.rs | 160 | ❌ HIGH | P3 | SpiderFacade ✅ |
| render/mod.rs | 134 | ⚠️ EXEMPT | - | Module file |
| crawl.rs | 133 | ❌ HIGH | P3 | CrawlFacade ✅ |
| stubs.rs | 118 | ❌ HIGH | P4 | Test fixtures? |
| search.rs | 116 | ❌ HIGH | P3 | SearchFacade ✅ |
| render/models.rs | 114 | ❌ HIGH | P2 | Move to types |
| extract.rs | 100 | ❌ HIGH | P3 | ExtractionFacade ✅ |
| workers.rs | 94 | ❌ MEDIUM | P4 | WorkersFacade ✅ |
| profiles.rs | 88 | ❌ MEDIUM | P4 | ProfileFacade ✅ |
| trace_backend_refactored.rs | 74 | ❌ MEDIUM | P1 | Trim to <40 |
| pdf.rs | 70 | ❌ MEDIUM | P4 | PdfFacade ✅ |
| mod.rs | 67 | ⚠️ EXEMPT | - | Module file |
| sessions.rs | 65 | ❌ MEDIUM | P4 | SessionFacade ✅ |
| llm_refactored.rs | 58 | ❌ MEDIUM | P1 | Trim to <45 |
| profiling.rs | 57 | ❌ MEDIUM | P4 | ProfilingFacade ✅ |
| browser.rs | 55 | ❌ MEDIUM | P4 | BrowserFacade ✅ |
| llm_minimal.rs | 54 | ❌ MEDIUM | P1 | Use as template |
| tables.rs | 51 | ❌ MEDIUM | P4 | TableFacade ✅ |
| engine_selection.rs | 50 | ⚠️ BORDERLINE | P4 | EngineFacade ✅ |
| pipeline_phases.rs | 48 | ✅ PASS | - | - |
| chunking.rs | 47 | ✅ PASS | - | - |
| render/strategies.rs | 43 | ✅ PASS | - | - |
| shared/spider.rs | 40 | ✅ PASS | - | - |
| utils.rs | 36 | ✅ PASS | - | - |
| streaming.rs | 36 | ✅ PASS | - | - |
| fetch.rs | 33 | ✅ PASS | - | - |
| deepsearch.rs | 22 | ✅ PASS | - | - |
| strategies.rs | 21 | ✅ PASS | - | - |
| monitoring.rs | 18 | ✅ PASS | - | - |
| admin_stub.rs | 13 | ✅ PASS | - | - |
| memory.rs | 12 | ✅ PASS | - | - |

---

**END OF REPORT**
