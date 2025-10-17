# OpenAPI Specification Gaps Analysis - RipTide API

**Analysis Date**: 2025-10-17
**OpenAPI Spec Version**: 1.0.0 (Last Validated: 2025-10-10T07:44:00Z)
**Implementation File**: `/workspaces/eventmesh/crates/riptide-api/src/main.rs`

## Executive Summary

The RipTide API has **significant gaps** between the OpenAPI specification and actual implementation:

- **OpenAPI Spec**: 59 documented endpoints (claims 54 operation IDs)
- **Actual Implementation**: 95+ route declarations in main.rs
- **Gap**: **~41 undocumented endpoints** (43% of implementation missing from spec)

### Critical Findings

1. **Major Feature Categories Missing from OpenAPI**:
   - Browser session management (4 endpoints)
   - Resource monitoring (6 endpoints)
   - Fetch engine metrics (1 endpoint)
   - Admin endpoints (13 endpoints - feature-gated)
   - Telemetry/tracing (3 endpoints)
   - Profiling endpoints (6 new + 4 legacy)
   - Health endpoint variants (4 endpoints)

2. **Versioned Aliases Not Documented**: Many endpoints have `/api/v1/*` aliases not in spec

3. **HTTP Method Discrepancies**: Several endpoints have incorrect HTTP methods

---

## Category 1: MISSING ENDPOINTS - Core Features

### 1.1 Health Endpoints (Priority: P0)

**Missing from OpenAPI**:

| Endpoint | Method | Handler | Purpose |
|----------|--------|---------|---------|
| `/api/v1/health` | GET | `handlers::health` | v1 alias for /healthz |
| `/api/health/detailed` | GET | `handlers::health_detailed` | Detailed health status |
| `/health/:component` | GET | `handlers::health::component_health_check` | Component-specific health |
| `/health/metrics` | GET | `handlers::health::health_metrics_check` | Health metrics endpoint |

**Recommendation**: Add all health variants to OpenAPI spec under "Health" tag.

---

### 1.2 Extract Endpoint (Priority: P0)

**NEW v1.1 Feature - Completely Missing**:

| Endpoint | Method | Handler | Purpose |
|----------|--------|---------|---------|
| `/api/v1/extract` | POST | `handlers::extract` | Primary extraction endpoint |
| `/extract` | POST | `handlers::extract` | Root alias for backward compatibility |

**Impact**: This is a **major new feature** in v1.1 that enables direct content extraction without crawling. Critical gap.

**Recommendation**: Add to OpenAPI spec immediately with full request/response schemas.

---

### 1.3 Search Endpoint (Priority: P0)

**NEW v1.1 Feature - Completely Missing**:

| Endpoint | Method | Handler | Purpose |
|----------|--------|---------|---------|
| `/api/v1/search` | GET | `handlers::search` | Web search functionality |
| `/search` | GET | `handlers::search` | Root alias for backward compatibility |

**Impact**: Major new feature for search integration. Should be prominently documented.

**Recommendation**: Add to OpenAPI spec under new "Search" category with query parameters.

---

## Category 2: MISSING ENDPOINTS - Browser Management

### 2.1 Browser Session Management (Priority: P1)

**Completely Missing - 4 Endpoints**:

| Endpoint | Method | Handler | Purpose |
|----------|--------|---------|---------|
| `/api/v1/browser/session` | POST | `handlers::browser::create_browser_session` | Create browser session |
| `/api/v1/browser/action` | POST | `handlers::browser::execute_browser_action` | Execute browser action |
| `/api/v1/browser/pool/status` | GET | `handlers::browser::get_browser_pool_status` | Get pool status |
| `/api/v1/browser/session/:id` | DELETE | `handlers::browser::close_browser_session` | Close session |

**Impact**: Browser pool management is a core feature for headless rendering but completely undocumented.

**Recommendation**: Add new "Browser Management" section to OpenAPI with full schemas for:
- `CreateSessionRequest` (stealth_preset, initial_url, timeout_secs)
- `BrowserAction` enum (Navigate, ExecuteScript, Screenshot, GetContent, etc.)
- `ActionResult` response
- `PoolStatusInfo` response

---

## Category 3: MISSING ENDPOINTS - Resource Monitoring

### 3.1 Resource Monitoring Endpoints (Priority: P1)

**Completely Missing - 6 Endpoints**:

| Endpoint | Method | Handler | Purpose |
|----------|--------|---------|---------|
| `/resources/status` | GET | `handlers::resources::get_resource_status` | Overall resource status |
| `/resources/browser-pool` | GET | `handlers::resources::get_browser_pool_status` | Browser pool resources |
| `/resources/rate-limiter` | GET | `handlers::resources::get_rate_limiter_status` | Rate limiter status |
| `/resources/memory` | GET | `handlers::resources::get_memory_status` | Memory usage stats |
| `/resources/performance` | GET | `handlers::resources::get_performance_status` | Performance metrics |
| `/resources/pdf/semaphore` | GET | `handlers::resources::get_pdf_semaphore_status` | PDF semaphore status |

**Impact**: Critical for production monitoring and debugging. Should be documented.

**Recommendation**: Add new "Resource Monitoring" category to OpenAPI spec.

---

### 3.2 Fetch Engine Metrics (Priority: P1)

**Missing**:

| Endpoint | Method | Handler | Purpose |
|----------|--------|---------|---------|
| `/fetch/metrics` | GET | `handlers::fetch::get_fetch_metrics` | Fetch engine statistics |

**Recommendation**: Add to "Monitoring" category in OpenAPI.

---

## Category 4: MISSING ENDPOINTS - Profiling & Telemetry

### 4.1 Performance Profiling (Priority: P1)

**New riptide-performance integration - 6 Endpoints Missing**:

| Endpoint | Method | Handler | Purpose |
|----------|--------|---------|---------|
| `/api/profiling/memory` | GET | `handlers::profiling::get_memory_profile` | Memory profiling |
| `/api/profiling/cpu` | GET | `handlers::profiling::get_cpu_profile` | CPU profiling |
| `/api/profiling/bottlenecks` | GET | `handlers::profiling::get_bottleneck_analysis` | Bottleneck analysis |
| `/api/profiling/allocations` | GET | `handlers::profiling::get_allocation_metrics` | Allocation metrics |
| `/api/profiling/leak-detection` | POST | `handlers::profiling::trigger_leak_detection` | Trigger leak detection |
| `/api/profiling/snapshot` | POST | `handlers::profiling::trigger_heap_snapshot` | Create heap snapshot |

**Recommendation**: Add new "Profiling" category to OpenAPI spec.

---

### 4.2 Legacy Monitoring Profiling (Priority: P2)

**Deprecated but still in code - 4 Endpoints**:

| Endpoint | Method | Handler | Status |
|----------|--------|---------|--------|
| `/monitoring/profiling/memory` | GET | `handlers::monitoring::get_memory_metrics` | DEPRECATED |
| `/monitoring/profiling/leaks` | GET | `handlers::monitoring::get_leak_analysis` | DEPRECATED |
| `/monitoring/profiling/allocations` | GET | `handlers::monitoring::get_allocation_metrics` | DEPRECATED |
| `/monitoring/wasm-instances` | GET | `handlers::monitoring::get_wasm_health` | Active |

**Recommendation**: Mark as deprecated in OpenAPI or remove from implementation.

---

### 4.3 Telemetry & Tracing (Priority: P1)

**TELEM-005 Implementation - 3 Endpoints Missing**:

| Endpoint | Method | Handler | Purpose |
|----------|--------|---------|---------|
| `/api/telemetry/status` | GET | `handlers::telemetry::get_telemetry_status` | Telemetry system status |
| `/api/telemetry/traces` | GET | `handlers::telemetry::list_traces` | List trace data |
| `/api/telemetry/traces/:trace_id` | GET | `handlers::telemetry::get_trace_tree` | Get specific trace tree |

**Recommendation**: Add new "Telemetry" category to OpenAPI spec.

---

## Category 5: MISSING ENDPOINTS - Admin (Feature-Gated)

### 5.1 Admin Endpoints (Priority: P2)

**Feature-gated under `#[cfg(feature = "persistence")]` - 13 Endpoints**:

#### Tenant Management (7 endpoints):
| Endpoint | Method | Handler |
|----------|--------|---------|
| `/admin/tenants` | POST | `handlers::admin::create_tenant` |
| `/admin/tenants` | GET | `handlers::admin::list_tenants` |
| `/admin/tenants/:id` | GET | `handlers::admin::get_tenant` |
| `/admin/tenants/:id` | PUT | `handlers::admin::update_tenant` |
| `/admin/tenants/:id` | DELETE | `handlers::admin::delete_tenant` |
| `/admin/tenants/:id/usage` | GET | `handlers::admin::get_tenant_usage` |
| `/admin/tenants/:id/billing` | GET | `handlers::admin::get_tenant_billing` |

#### Cache Management (3 endpoints):
| Endpoint | Method | Handler |
|----------|--------|---------|
| `/admin/cache/warm` | POST | `handlers::admin::warm_cache` |
| `/admin/cache/invalidate` | POST | `handlers::admin::invalidate_cache` |
| `/admin/cache/stats` | GET | `handlers::admin::get_cache_stats` |

#### State Management (3 endpoints):
| Endpoint | Method | Handler |
|----------|--------|---------|
| `/admin/state/reload` | POST | `handlers::admin::reload_state` |
| `/admin/state/checkpoint` | POST | `handlers::admin::create_checkpoint` |
| `/admin/state/restore/:id` | POST | `handlers::admin::restore_checkpoint` |

**Recommendation**: Add new "Admin" category to OpenAPI spec with feature flag documentation.

---

## Category 6: VERSIONED ALIASES NOT DOCUMENTED

### 6.1 Missing v1 Aliases (Priority: P1)

The following endpoints have `/api/v1/*` aliases that are **not in the OpenAPI spec**:

| Root Endpoint | v1 Alias | Status |
|---------------|----------|--------|
| `/healthz` | `/api/v1/health` | MISSING |
| `/metrics` | `/api/v1/metrics` | MISSING |
| `/crawl` | `/api/v1/crawl` | MISSING |
| `/render` | `/api/v1/render` | MISSING |

**Recommendation**: Add all v1 aliases to OpenAPI spec or document aliasing strategy.

---

## Category 7: ENDPOINT METHOD MISMATCHES

### 7.1 Workers Endpoints (Priority: P1)

**Issue**: OpenAPI documents only POST for `/workers/jobs`, but implementation has both:

| Endpoint | OpenAPI Method | Actual Methods | Handler |
|----------|---------------|----------------|---------|
| `/workers/jobs` | POST | POST, GET | `submit_job`, `list_jobs` |

**Recommendation**: Update OpenAPI to document GET method for listing jobs.

---

### 7.2 LLM Config Endpoint

**Issue**: OpenAPI documents `/api/v1/llm/config` with separate GET and POST, implementation correct.

**Status**: ✅ Correctly documented

---

## Category 8: ADDITIONAL MISSING ENDPOINTS

### 8.1 Resource Status Endpoint

| Endpoint | Method | Handler | Purpose |
|----------|--------|---------|---------|
| `/api/resources/status` | GET | `handlers::monitoring::get_resource_status` | Resource management status |

**Note**: Different from `/resources/status` - needs clarification.

---

## Category 9: CORRECTLY DOCUMENTED ENDPOINTS

The following endpoint categories are **correctly documented** in OpenAPI:

✅ **Crawling** (5 endpoints):
- `/crawl`, `/crawl/stream`, `/crawl/sse`, `/crawl/ws`, `/render`

✅ **DeepSearch** (2 endpoints):
- `/deepsearch`, `/deepsearch/stream`

✅ **Spider** (3 endpoints):
- `/spider/crawl`, `/spider/status`, `/spider/control`

✅ **Strategies** (2 endpoints):
- `/strategies/crawl`, `/strategies/info`

✅ **PDF** (3 endpoints):
- `/pdf/process`, `/pdf/process-stream`, `/pdf/health`

✅ **Stealth** (4 endpoints):
- `/stealth/configure`, `/stealth/test`, `/stealth/capabilities`, `/stealth/health`

✅ **Tables** (2 endpoints):
- `/api/v1/tables/extract`, `/api/v1/tables/:id/export`

✅ **LLM** (4 endpoints):
- `/api/v1/llm/providers`, `/api/v1/llm/providers/switch`, `/api/v1/llm/config` (GET/POST)

✅ **Sessions** (12 endpoints):
- All session management endpoints correctly documented

✅ **Workers** (9 endpoints):
- Most worker endpoints documented (except GET `/workers/jobs`)

✅ **Monitoring** (6 endpoints):
- `/monitoring/health-score`, `/monitoring/performance-report`, etc.

✅ **Pipeline** (1 endpoint):
- `/pipeline/phases`

✅ **Basic Health** (2 endpoints):
- `/healthz`, `/metrics`

---

## Priority Recommendations

### P0 - Critical (Must Fix Immediately)

1. **Add Extract Endpoints** (`/api/v1/extract`, `/extract`) - Major v1.1 feature
2. **Add Search Endpoints** (`/api/v1/search`, `/search`) - Major v1.1 feature
3. **Add Health Variants** (4 endpoints) - Critical for monitoring
4. **Fix Workers GET Method** - Breaking change if not documented

### P1 - High Priority (Fix in Next Release)

1. **Add Browser Management** (4 endpoints) - Core feature set
2. **Add Resource Monitoring** (6 endpoints) - Production monitoring
3. **Add Telemetry Endpoints** (3 endpoints) - TELEM-005 implementation
4. **Add Profiling Endpoints** (6 endpoints) - Performance monitoring
5. **Add Fetch Metrics** (1 endpoint)
6. **Document v1 Aliases** (4+ endpoints)

### P2 - Medium Priority (Document When Time Permits)

1. **Add Admin Endpoints** (13 endpoints) - Feature-gated but important
2. **Clean Up Legacy Profiling** (4 endpoints) - Mark as deprecated or remove
3. **Clarify Resource Status Endpoints** - Two similar endpoints need clarification

---

## Schema Gaps

The following request/response schemas are **missing** from OpenAPI:

### Missing Request Schemas:
- `ExtractRequest` (for `/api/v1/extract`)
- `SearchQuery` (for `/api/v1/search`)
- `CreateSessionRequest` (for `/api/v1/browser/session`)
- `BrowserAction` enum (for `/api/v1/browser/action`)
- All admin request schemas

### Missing Response Schemas:
- `ExtractResponse`
- `SearchResults`
- `SessionResponse`
- `ActionResult`
- `PoolStatusInfo`
- `ResourceStatus`
- `FetchMetrics`
- `TelemetryStatus`
- `TraceTree`
- `ProfilingData`
- All admin response schemas

---

## Deprecated Endpoints to Remove

The following endpoints are marked as **deprecated** in code comments but still present:

| Endpoint | Replacement | Action |
|----------|-------------|--------|
| `/monitoring/profiling/*` | `/api/profiling/*` | Remove in v2.0 |
| (None others identified) | - | - |

---

## Summary Statistics

| Category | OpenAPI Count | Implementation Count | Gap |
|----------|--------------|---------------------|-----|
| **Health** | 2 | 6 | +4 |
| **Crawling** | 5 | 7 | +2 (v1 aliases) |
| **Search** | 2 | 4 | +2 (new feature) |
| **Browser** | 0 | 4 | +4 |
| **Resources** | 0 | 7 | +7 |
| **Profiling** | 0 | 10 | +10 |
| **Telemetry** | 0 | 3 | +3 |
| **Admin** | 0 | 13 | +13 |
| **Workers** | 9 | 10 | +1 (GET method) |
| **Other** | Various | Various | - |
| **TOTAL** | ~59 | ~95+ | **+41 (43%)** |

---

## Action Items

1. ✅ **Create this gap analysis document** - COMPLETED
2. ⏭️ **Update OpenAPI spec** with all P0 endpoints
3. ⏭️ **Add missing request/response schemas**
4. ⏭️ **Generate new API documentation** from updated spec
5. ⏭️ **Update API client SDKs** to include new endpoints
6. ⏭️ **Add integration tests** for undocumented endpoints
7. ⏭️ **Remove or document deprecated endpoints**
8. ⏭️ **Create versioning strategy** for v1 aliases

---

## Next Steps

1. **Immediate**: Update OpenAPI spec with P0 endpoints (Extract, Search, Health variants)
2. **Short-term**: Add P1 endpoints (Browser, Resources, Telemetry, Profiling)
3. **Medium-term**: Document P2 endpoints (Admin, clean up legacy)
4. **Long-term**: Establish CI/CD check to prevent spec drift

---

**Generated by**: RipTide Hive-Mind Code Analysis
**Analysis Method**: Comparative analysis of `/workspaces/eventmesh/docs/api/openapi.yaml` vs `/workspaces/eventmesh/crates/riptide-api/src/main.rs`
**Confidence Level**: HIGH (based on direct source code inspection)
