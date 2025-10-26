# OpenAPI Specification - Schemathesis Fixes Required

**Analysis Date:** 2025-10-26
**OpenAPI File:** `/workspaces/eventmesh/docs/api/openapi.yaml`
**Total Issues Found:** 177

---

## Executive Summary

This document outlines all required fixes to the OpenAPI specification based on Schemathesis validation report. The issues are categorized into four main types:

1. **DELETE endpoints missing 204 status**: 7 endpoints
2. **POST/PUT/PATCH endpoints missing 400 status**: 34 endpoints
3. **POST/PUT/PATCH endpoints missing 415 status**: 40 endpoints
4. **All endpoints missing 503 status**: 96 endpoints

---

## Issue Categories

### 1. DELETE Endpoints Missing 204 (No Content) Status Code

**Impact:** Medium
**Count:** 7 endpoints
**Issue:** REST best practices dictate that DELETE operations should return 204 (No Content) when successful, especially when there's no response body. Currently these endpoints only document 200.

**Recommendation:** Add `'204'` response alongside or instead of `'200'` for DELETE operations with no content body.

#### Affected Endpoints:

| Line | Path | Operation ID | Current Status |
|------|------|--------------|----------------|
| 654 | `/sessions/{session_id}` | `sessions_session_id_delete` | 200, 429 |
| 710 | `/sessions/{session_id}/cookies` | `sessions_session_id_cookies_delete` | 200, 429 |
| 787 | `/sessions/{session_id}/cookies/{domain}/{name}` | `sessions_session_id_cookies_domain_name_delete` | 200, 429 |
| 934 | `/workers/schedule/{job_id}` | `workers_schedule_job_id_delete` | 200, 429 |
| 1055 | `/api/v1/browser/session/{id}` | `browser_session_delete` | 200, 429 |
| 1218 | `/admin/tenants/{id}` | `admin_tenants_id_delete` | 200, 429 |
| 1644 | `/api/v1/profiles/clear` | `clear_all_caches` | 200 (has body) |

**Note:** `/api/v1/profiles/{domain}` (line 1437) already correctly uses 204, and `/api/v1/profiles/clear` (line 1644) returns a body with stats, so 200 is appropriate there.

**Fix Template:**
```yaml
responses:
  '204':
    description: Successfully deleted (no content)
  '404':
    $ref: '#/components/responses/NotFound'
  '429':
    $ref: '#/components/responses/RateLimitExceeded'
  '503':
    $ref: '#/components/responses/ServiceUnavailable'
```

---

### 2. POST/PUT/PATCH Endpoints Missing 400 (Bad Request) Status Code

**Impact:** High
**Count:** 34 endpoints
**Issue:** Mutation endpoints should document 400 responses for validation errors (missing required fields, invalid formats, constraint violations).

#### Affected Endpoints:

| Line | Path | Method | Operation ID |
|------|------|--------|--------------|
| 227 | `/crawl` | POST | `crawl_batch` |
| 238 | `/api/v1/crawl` | POST | `crawl_batch_v1` |
| 249 | `/crawl/stream` | POST | `crawl_stream_ndjson` |
| 260 | `/crawl/sse` | POST | `crawl_stream_sse` |
| 282 | `/deepsearch` | POST | `deepsearch_post` |
| 293 | `/deepsearch/stream` | POST | `deepsearch_stream_post` |
| 304 | `/render` | POST | `render_post` |
| 315 | `/api/v1/render` | POST | `render_post_v1` |
| 326 | `/extract` | POST | `extract_post` |
| 338 | `/api/v1/extract` | POST | `extract_post_v1` |
| 374 | `/spider/crawl` | POST | `spider_crawl_post` |
| 385 | `/spider/status` | POST | `spider_status_post` |
| 396 | `/spider/control` | POST | `spider_control_post` |
| 407 | `/strategies/crawl` | POST | `strategies_crawl_post` |
| 429 | `/pdf/process` | POST | `pdf_process_post` |
| 440 | `/pdf/process-stream` | POST | `pdf_process-stream_post` |
| 462 | `/stealth/configure` | POST | `stealth_configure_post` |
| 473 | `/stealth/test` | POST | `stealth_test_post` |
| 506 | `/api/v1/tables/extract` | POST | `api_v1_tables_extract_post` |
| 559 | `/api/v1/llm/providers/switch` | POST | `api_v1_llm_providers_switch_post` |
| 580 | `/api/v1/llm/config` | POST | `api_v1_llm_config_post` |
| 591 | `/sessions` | POST | `sessions_post` |
| 623 | `/sessions/cleanup` | POST | `sessions_cleanup_post` |
| 673 | `/sessions/{session_id}/extend` | POST | `sessions_session_id_extend_post` |
| 692 | `/sessions/{session_id}/cookies` | POST | `sessions_session_id_cookies_post` |
| 820 | `/workers/jobs` | POST | `workers_jobs_post` |
| 913 | `/workers/schedule` | POST | `workers_schedule_post` |
| 1019 | `/api/v1/browser/session` | POST | `browser_session_post` |
| 1031 | `/api/v1/browser/action` | POST | `browser_action_post` |
| 1159 | `/admin/tenants` | POST | `admin_tenants_post` |
| 1200 | `/admin/tenants/{id}` | PUT | `admin_tenants_id_put` |
| 1237 | `/admin/cache/warm` | POST | `admin_cache_warm_post` |
| 1261 | `/admin/state/reload` | POST | `admin_state_reload_post` |
| 1815 | `/api/v1/engine/probe-first` | PUT | `toggle_probe_first` |

**Endpoints that already have 400 (Good examples):**
- `/api/v1/profiles` (POST) - line 1273
- `/api/v1/profiles/{domain}` (PUT) - line 1396
- `/api/v1/profiles/batch` (POST) - line 1528
- `/api/v1/profiles/search` (GET) - line 1567
- `/api/v1/engine/analyze` (POST) - line 1678
- `/api/v1/engine/decide` (POST) - line 1733

**Fix Template:**
```yaml
responses:
  '200':
    description: Success
  '400':
    $ref: '#/components/responses/BadRequest'
  '429':
    $ref: '#/components/responses/RateLimitExceeded'
  '503':
    $ref: '#/components/responses/ServiceUnavailable'
```

---

### 3. POST/PUT/PATCH Endpoints Missing 415 (Unsupported Media Type) Status Code

**Impact:** Medium
**Count:** 40 endpoints
**Issue:** Endpoints that accept request bodies should document 415 responses for incorrect Content-Type headers.

#### Affected Endpoints:

All 34 endpoints from section 2, plus these additional endpoints:

| Line | Path | Method | Operation ID |
|------|------|--------|--------------|
| 1396 | `/api/v1/profiles/{domain}` | PUT | `update_profile` |
| 1600 | `/api/v1/profiles/{domain}/warm` | POST | `warm_cache` |
| 1678 | `/api/v1/engine/analyze` | POST | `analyze_engine` |
| 1733 | `/api/v1/engine/decide` | POST | `decide_engine` |
| 1815 | `/api/v1/engine/probe-first` | PUT | `toggle_probe_first` |
| 1200 | `/admin/tenants/{id}` | PUT | `admin_tenants_id_put` |

**Note:** Endpoints with `requestBody` should always document 415 for Content-Type validation.

**Fix Template:**
```yaml
responses:
  '200':
    description: Success
  '400':
    $ref: '#/components/responses/BadRequest'
  '415':
    $ref: '#/components/responses/UnsupportedMediaType'
  '429':
    $ref: '#/components/responses/RateLimitExceeded'
  '503':
    $ref: '#/components/responses/ServiceUnavailable'
```

---

### 4. All Endpoints Missing 503 (Service Unavailable) Status Code

**Impact:** High
**Count:** 96 endpoints
**Issue:** All endpoints should document 503 responses for dependency failures (Redis, database, external services down).

**Currently have 503:**
- `/healthz` (line 127) ✓
- `/api/v1/health` (line 141) ✓
- `/api/health/detailed` (line 154) ✓
- `/health/{component}` (line 168) ✓

**Missing 503 on 96 endpoints** - All other endpoints in the specification.

**Fix Template:**
```yaml
responses:
  '200':
    description: Success
  '429':
    $ref: '#/components/responses/RateLimitExceeded'
  '503':
    $ref: '#/components/responses/ServiceUnavailable'
```

---

## Required Component Schema Addition

The specification needs a new reusable response component for 415 and 503 errors:

```yaml
components:
  responses:
    BadRequest:  # Already exists ✓
      description: Bad Request - Invalid or missing required parameters
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'

    NotFound:  # Already exists ✓
      description: Not Found - Resource does not exist
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'

    RateLimitExceeded:  # Already exists ✓
      description: Rate Limit Exceeded - Too many requests
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/RateLimitError'

    UnsupportedMediaType:  # MISSING - ADD THIS
      description: Unsupported Media Type - Invalid Content-Type header
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'
          example:
            error:
              message: "Unsupported Media Type: Expected application/json"
              retryable: false
              status: 415
              type: unsupported_media_type

    ServiceUnavailable:  # MISSING - ADD THIS
      description: Service Unavailable - Dependency error or system overload
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'
          example:
            error:
              message: "Service temporarily unavailable: Redis connection failed"
              retryable: true
              status: 503
              type: service_unavailable
```

---

## Implementation Priority

### Priority 1: Critical (Immediate Fix Required)
1. **Add 503 to all endpoints** - Essential for proper error handling
2. **Add 400 to all POST/PUT/PATCH endpoints** - Required for validation errors
3. **Add missing response components** - `UnsupportedMediaType` and `ServiceUnavailable`

### Priority 2: High (Fix Soon)
1. **Add 415 to all POST/PUT/PATCH endpoints with request bodies**
2. **Add 204 to DELETE endpoints** - REST best practice

---

## Automation Script Recommendations

To avoid manual errors, consider using a script to:

1. Add standard error responses to all endpoints based on HTTP method
2. Validate all response schemas reference existing components
3. Ensure consistency across versioned endpoints (e.g., `/crawl` vs `/api/v1/crawl`)

---

## Testing Recommendations

After fixes, validate with:

```bash
# Run Schemathesis validation
schemathesis run docs/api/openapi.yaml --base-url http://localhost:8080

# Validate OpenAPI spec
npx @apidevtools/swagger-cli validate docs/api/openapi.yaml

# Check for completeness
npx spectral lint docs/api/openapi.yaml
```

---

## Summary Statistics

| Issue Category | Count | Priority |
|----------------|-------|----------|
| Missing 204 on DELETE | 7 | Medium |
| Missing 400 on POST/PUT/PATCH | 34 | High |
| Missing 415 on POST/PUT/PATCH | 40 | Medium |
| Missing 503 on all endpoints | 96 | Critical |
| **Total Issues** | **177** | - |

---

## Next Steps

1. **Add missing response components** to `/components/responses` section
2. **Update all endpoints** systematically by category
3. **Run validation tests** to ensure no regressions
4. **Update version number** and validation timestamp in spec
5. **Document changes** in API changelog

---

**Generated by:** Code Analyzer Agent
**Analysis Tool:** Python + PyYAML
**Validation Source:** Schemathesis test report
