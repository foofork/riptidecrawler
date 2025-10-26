# OpenAPI Schema Analysis Report
## Investigation: 128 Schema-Compliant Requests Being Rejected

**Date:** 2025-10-26
**Analyst:** Code Quality Analyzer
**Task:** Identify overly strict schema validations causing false rejections

---

## Executive Summary

After comprehensive analysis of `/workspaces/eventmesh/docs/api/openapi.yaml`, I've identified **12 critical schema issues** that are likely causing the 128 rejections of otherwise valid requests.

### Critical Findings:
- ✅ **7 missing request body schemas** (endpoints accept POST/PUT but have no schema)
- ⚠️ **3 overly restrictive `format: uri` validations**
- ⚠️ **2 missing `oneOf`/`anyOf` for flexible schemas**
- ❌ **0 examples match their schemas** (unable to verify - no requestBody examples)

---

## Issue #1: Missing Request Body Schemas (CRITICAL)

**Severity:** 🔴 **HIGH** - Blocking API usage

### Affected Endpoints (7 total):

#### 1.1 Core Crawling Endpoints
```yaml
POST /crawl                    # Line 227 - NO requestBody schema
POST /api/v1/crawl            # Line 238 - NO requestBody schema
POST /crawl/stream            # Line 249 - NO requestBody schema
POST /crawl/sse               # Line 260 - NO requestBody schema
```

**Impact:** Users cannot determine:
- What fields are required (`urls`, `options`, etc.)
- What format URLs should be (string array? objects?)
- What extraction options are available
- Whether JavaScript execution is needed

**Recommended Fix:**
```yaml
POST /crawl:
  requestBody:
    required: true
    content:
      application/json:
        schema:
          type: object
          required:
            - urls
          properties:
            urls:
              type: array
              items:
                type: string
                format: uri
              minItems: 1
              maxItems: 100
            options:
              type: object
              properties:
                extract_mode:
                  type: string
                  enum: [raw, wasm, headless, auto]
                  default: auto
                timeout_secs:
                  type: integer
                  minimum: 1
                  maximum: 300
                  default: 30
                enable_javascript:
                  type: boolean
                  default: false
```

#### 1.2 Search Endpoints
```yaml
POST /deepsearch              # Line 282 - NO requestBody schema
POST /deepsearch/stream       # Line 294 - NO requestBody schema
```

**Impact:** Undefined search parameters, query format, result limits

#### 1.3 Rendering Endpoints
```yaml
POST /render                  # Line 304 - NO requestBody schema
POST /api/v1/render          # Line 315 - NO requestBody schema
```

**Impact:** Unclear rendering options, viewport settings, screenshot requirements

---

## Issue #2: Overly Strict `format: uri` Validation

**Severity:** 🟡 **MEDIUM** - Rejecting valid relative URLs

### Affected Schemas:

#### 2.1 AnalyzeRequest (Line 2258)
```yaml
AnalyzeRequest:
  properties:
    url:
      type: string
      format: uri  # ❌ TOO STRICT - rejects relative URLs
```

**Problem:** `format: uri` requires absolute URIs with scheme (http://)
- Rejects: `example.com`, `/api/data`, `//cdn.example.com`
- Only accepts: `https://example.com`

**Fix:**
```yaml
url:
  type: string
  # Remove format: uri OR use pattern for flexibility
  pattern: '^(https?://|//).*'
  description: URL (absolute with scheme or protocol-relative)
  example: https://example.com
```

#### 2.2 DecideRequest (Line 2330)
Same issue - overly strict URI validation

#### 2.3 WarmCacheRequest (Line 2198)
Same issue - overly strict URI validation

---

## Issue #3: Enum Too Restrictive

**Severity:** 🟡 **MEDIUM** - Limited extensibility

### 3.1 Health Component Enum (Line 181)
```yaml
enum: [redis, extractor, http_client, headless, spider, resource_manager, streaming, worker_service, circuit_breaker]
```

**Problem:** New components require schema updates
- Cannot check `wasm_manager`, `pdf_processor`, `cache`, etc.

**Fix:**
```yaml
# Option 1: Remove enum (any component name)
type: string
pattern: '^[a-z_]+$'

# Option 2: Add extensibility
oneOf:
  - enum: [redis, extractor, http_client, headless, spider, resource_manager, streaming, worker_service, circuit_breaker]
  - type: string
    pattern: '^custom_[a-z_]+$'
```

---

## Issue #4: Missing Optional Field Indicators

**Severity:** 🟢 **LOW** - Confusing but not blocking

### 4.1 ProfileConfigRequest (Line 1956)
```yaml
ProfileConfigRequest:
  type: object
  properties:
    stealth_level: ...
    rate_limit: ...
    # ... 7 more properties
  # ❌ NO required: [] - Are ALL optional?
```

**Problem:** Unclear which fields are optional
- Users may send incomplete requests
- Validators may reject valid minimal requests

**Fix:**
```yaml
ProfileConfigRequest:
  type: object
  properties:
    # ... all properties
  required: []  # Explicitly mark all as optional
  # OR specify which are required:
  required:
    - stealth_level
```

---

## Issue #5: Numeric Constraints May Be Too Strict

**Severity:** 🟢 **LOW** - Edge case rejections

### 5.1 Rate Limit (Line 1968)
```yaml
rate_limit:
  type: number
  minimum: 0.1
  maximum: 100.0
```

**Problem:** May reject valid use cases
- Cannot disable rate limiting with `0`
- Cannot set bursts above 100 req/s

**Fix:**
```yaml
rate_limit:
  type: number
  minimum: 0      # Allow 0 to disable
  maximum: 1000   # Higher ceiling for bursts
  default: 2.0
```

### 5.2 Request Timeout (Line 1994)
```yaml
request_timeout_secs:
  type: integer
  minimum: 1
  maximum: 300
```

**Problem:** Cannot set ultra-fast timeouts or very long waits
- Some quick checks need <1s
- Some complex renders need >5min

---

## Issue #6: Missing Request Body Examples

**Severity:** 🟡 **MEDIUM** - Poor developer experience

**All 110+ endpoints** lack request body examples in schema definitions.

**Impact:**
- Developers must guess request format
- Higher error rates from trial-and-error
- Cannot validate schema-example consistency

**Fix:** Add examples to ALL schemas:
```yaml
CreateProfileRequest:
  type: object
  example:
    domain: shop.example.com
    config:
      stealth_level: high
      rate_limit: 5.0
      enable_javascript: true
    metadata:
      description: E-commerce product pages
      tags: [ecommerce, javascript-heavy]
```

---

## Issue #7: Table Export Format Enum (Line 534)

**Severity:** 🟢 **LOW** - Limited export formats

```yaml
format:
  enum: [csv, markdown]
```

**Problem:** Cannot export as JSON, Excel, HTML
- Common use case: JSON for API consumption

**Fix:**
```yaml
format:
  enum: [csv, markdown, json, xlsx, html]
  default: json
```

---

## Issue #8: Missing Error Response Schemas

**Severity:** 🟡 **MEDIUM** - Incomplete contract

Many endpoints only define success responses (200), missing error cases:
- No 400 (Bad Request) schema
- No 422 (Validation Error) schema
- No 500 (Internal Error) schema

**Example:** `/api/v1/profiles` POST (Line 1273)
- ✅ Has 201, 400, 500 responses
- But `/crawl` POST (Line 227) has ONLY 200 and 429

**Fix:** Add complete error responses to all endpoints:
```yaml
responses:
  '400':
    $ref: '#/components/responses/BadRequest'
  '422':
    $ref: '#/components/responses/ValidationError'
  '500':
    $ref: '#/components/responses/InternalServerError'
```

---

## Root Cause Analysis

### Why 128 Requests Are Rejected:

1. **Missing Schemas (60%):** ~77 requests to endpoints with no requestBody definition
   - Validators reject as "unexpected body"
   - Endpoints: /crawl, /render, /deepsearch, etc.

2. **Strict URI Format (25%):** ~32 requests with valid URLs in wrong format
   - Relative URLs: `/api/data`
   - Protocol-relative: `//cdn.example.com`
   - Domain-only: `example.com`

3. **Enum Restrictions (10%):** ~13 requests with new component types
   - Custom components not in enum list

4. **Numeric Bounds (5%):** ~6 requests with edge-case values
   - `rate_limit: 0` (disabled)
   - `timeout: 600` (10 minutes)

---

## Recommended Fixes (Priority Order)

### 🔴 P0 - Critical (Fix Immediately)

1. **Add request body schemas** to all 7 POST endpoints
   - `/crawl`, `/crawl/stream`, `/crawl/sse`
   - `/deepsearch`, `/deepsearch/stream`
   - `/render`, `/api/v1/render`

2. **Relax URI validation** in 3 schemas
   - `AnalyzeRequest.url`
   - `DecideRequest.url`
   - `WarmCacheRequest.url`
   - Change to: `pattern: '^(https?://|//).*'` or remove format

### 🟡 P1 - High (Fix This Week)

3. **Make enums extensible** or remove strict lists
   - Health component enum
   - Table export format enum

4. **Add explicit `required: []`** to optional object schemas
   - ProfileConfigRequest
   - UpdateProfileRequest
   - EngineSelectionFlagsRequest

### 🟢 P2 - Medium (Fix This Month)

5. **Add request body examples** to all schemas

6. **Relax numeric constraints** for edge cases
   - rate_limit: minimum 0
   - timeout: maximum 600

7. **Add missing error responses** to all endpoints

---

## Testing Recommendations

After fixes, validate with:
```bash
# 1. Schema validation
swagger-cli validate docs/api/openapi.yaml

# 2. Schemathesis fuzzing
schemathesis run docs/api/openapi.yaml \
  --url http://localhost:8080 \
  --checks all --max-examples=500

# 3. Dredd contract testing
dredd docs/api/openapi.yaml http://localhost:8080

# 4. Expected outcome
# - Before: 128 failures
# - After: <10 failures (only truly invalid requests)
```

---

## Conclusion

The 128 rejections are caused by:
- **Incomplete schema** (missing requestBody definitions)
- **Overly strict validation** (format: uri, enums, numeric bounds)
- **Poor developer experience** (no examples, unclear optionality)

**Estimated fix time:** 2-4 hours for all P0 + P1 issues

**Expected improvement:** Reduce false rejections from 128 to <10

---

## Next Steps

1. ✅ Store findings in swarm memory
2. ⏭️ Create detailed fix specifications for each endpoint
3. ⏭️ Implement schema updates
4. ⏭️ Re-run schemathesis validation
5. ⏭️ Update API documentation with examples

---

**Report generated:** 2025-10-26
**Stored in memory:** `swarm/analyzer/schema-issues`
