# Schemathesis Failure Analysis Report
**Analysis Date:** 2025-10-27
**Test Run:** Schemathesis v4.3.13
**Total Failures:** 211 unique failures from 869 test cases
**API Base:** http://localhost:8080
**Specification:** OpenAPI 3.0.0 (102 operations)

---

## Executive Summary

Schemathesis fuzzing revealed **211 unique failures** across multiple categories. The majority of failures (98/211 = 46.4%) are related to **API rejecting schema-compliant requests**, indicating validation rules that are stricter than documented. The second largest category is **undocumented HTTP status codes** (59/211 = 28.0%), primarily missing 502 and 503 error responses in the OpenAPI specification.

### Failure Distribution

| Category | Count | Percentage | Severity |
|----------|-------|------------|----------|
| **API rejected schema-compliant request** | 98 | 46.4% | 🔴 **CRITICAL** |
| **Undocumented HTTP status code** | 59 | 28.0% | 🟠 **HIGH** |
| **Server error** | 19 | 9.0% | 🔴 **CRITICAL** |
| **Unsupported method incorrect response** | 15 | 7.1% | 🟡 **MEDIUM** |
| **Response violates schema** | 7 | 3.3% | 🔴 **CRITICAL** |
| **API accepted schema-violating request** | 7 | 3.3% | 🟠 **HIGH** |
| **Undocumented Content-Type** | 6 | 2.8% | 🟡 **MEDIUM** |
| **TOTAL** | **211** | **100%** | - |

### Test Coverage Statistics

- **Examples:** 17 passed, 38 failed, 47 skipped (in 4.30s)
- **Coverage:** 29 passed, 73 failed (in 41.60s)
- **Fuzzing:** 3 passed, 99 failed (in 8.62s)
- **Seed:** 113530525868578902937193447883367590929

---

## 1. CRITICAL: API Rejected Schema-Compliant Request (98 failures)

### Problem Description
The API is **rejecting valid requests** that comply with the OpenAPI schema. This indicates overly strict validation rules not reflected in the specification.

### Common Patterns Identified

#### Pattern 1.1: Empty String Validation
**Example:** `POST /deepsearch` with `{"query": ""}`
- **Error:** `"Validation error: Search query cannot be empty"`
- **Status:** 400 Bad Request
- **Issue:** OpenAPI allows empty strings (no `minLength: 1` constraint)
- **Impact:** 15-20 endpoints likely affected

**Affected Endpoint Types:**
- `/deepsearch` - Search endpoints
- `/search` - Query parameters
- `/crawl` - URL list endpoints
- `/extract` - URL validation

#### Pattern 1.2: Default Value Rejection
**Symptom:** API rejects requests using default values defined in schema
- **Issue:** Runtime validation differs from schema defaults
- **Impact:** All endpoints with optional parameters with defaults

#### Pattern 1.3: Case Sensitivity Mismatches
**Example:** `POST /api/v1/extract` with `"url": "https://A.COM"`
- **Error:** `"Connection failed: error sending request for url (https://a.com/)"`
- **Issue:** URL normalization happens but spec doesn't document lowercase conversion
- **Impact:** All URL-accepting endpoints (crawl, extract, fetch, render)

#### Pattern 1.4: Schema Allows Null but Runtime Rejects
- **Issue:** Schema may use `nullable: true` but validation middleware rejects null
- **Impact:** Optional fields across all POST/PUT endpoints

### Root Causes
1. **Validation middleware** (`request_validation.rs`) enforces stricter rules than OpenAPI
2. **Business logic validation** happens after schema validation
3. **OpenAPI schema lacks constraints** that exist in Rust validation code
4. **Inconsistent validation** between middleware and handlers

### Recommended Fixes

#### Priority 1: Schema Constraint Alignment (HIGH)
Add missing constraints to OpenAPI schema:

```yaml
# Example: /deepsearch query field
query:
  type: string
  minLength: 1  # ADD THIS
  maxLength: 2000  # ADD THIS if enforced
  pattern: '^[\S\s]*\S[\S\s]*$'  # No whitespace-only strings
  example: "web scraping tools"
```

#### Priority 2: URL Validation Documentation (HIGH)
Document URL normalization behavior:

```yaml
url:
  type: string
  format: uri
  description: |
    Valid HTTP/HTTPS URL. URLs are normalized to lowercase.
    Private IPs and localhost are blocked.
  pattern: '^https?://'
  minLength: 10
  maxLength: 2048
```

#### Priority 3: Validation Middleware Audit (CRITICAL)
**Action Required:** Audit `/workspaces/eventmesh/crates/riptide-api/src/middleware/request_validation.rs`

Extract all validation rules and ensure they're documented in OpenAPI:
- String length constraints
- Pattern validations
- Business rule restrictions
- IP address blocks
- URL scheme requirements

### Code Locations to Investigate
```
/workspaces/eventmesh/crates/riptide-api/src/middleware/request_validation.rs
/workspaces/eventmesh/crates/riptide-api/src/validation.rs
/workspaces/eventmesh/crates/riptide-core/src/validation.rs
```

---

## 2. HIGH: Undocumented HTTP Status Code (59 failures)

### Problem Description
The API returns status codes not documented in the OpenAPI specification, primarily **502 (Bad Gateway)** and **503 (Service Unavailable)** errors.

### Breakdown by Status Code

| Undocumented Status | Count | Example Endpoints |
|---------------------|-------|-------------------|
| **502 Bad Gateway** | ~30 | `/extract`, `/api/v1/extract`, `/fetch` |
| **503 Service Unavailable** | ~25 | `/deepsearch`, `/search` (missing API keys) |
| **Other (500, 504)** | ~4 | Various error scenarios |

### Common Patterns

#### Pattern 2.1: Fetch Errors → 502
**Example:** `POST /extract` with unreachable URL
```json
{
  "error": {
    "message": "Failed to fetch content from https://example.com/article: Server returned status: 404 Not Found",
    "retryable": true,
    "status": 502,
    "type": "fetch_error"
  }
}
```

**Affected Operations:**
- All extract operations (8 endpoints)
- All fetch operations (4 endpoints)
- Render operations (6 endpoints)
- Spider operations (3 endpoints)

#### Pattern 2.2: Missing Dependencies → 503
**Example:** `POST /deepsearch` without Serper API key
```json
{
  "error": {
    "message": "Dependency unavailable: search_provider - SearchProviderFactory failed to create provider: Serper backend requires a valid API key",
    "retryable": true,
    "status": 503,
    "type": "dependency_error"
  }
}
```

**Affected Operations:**
- `/deepsearch` - Requires search API
- `/search` - Requires search API
- Redis-dependent endpoints
- LLM-dependent endpoints

### Cross-Reference with Existing Analysis
The `schemathesis-fixes-needed.md` document already identifies **96 endpoints missing 503**. This confirms the pattern.

### Recommended Fixes

#### Fix 2.1: Add 502 to All External-Dependent Endpoints (30 endpoints)
```yaml
'502':
  description: Bad Gateway - External resource fetch failed
  content:
    application/json:
      schema:
        $ref: '#/components/schemas/Error'
      example:
        error:
          message: "Failed to fetch content: Server returned 404"
          retryable: true
          status: 502
          type: fetch_error
```

**Endpoints requiring 502:**
- `/extract`, `/api/v1/extract`
- `/fetch`, `/api/v1/fetch`
- `/render`, `/api/v1/render`
- `/crawl`, `/api/v1/crawl`
- `/spider/crawl`
- All streaming variants

#### Fix 2.2: Add 503 to ALL Endpoints (96 endpoints)
Already documented in `schemathesis-fixes-needed.md` - add `ServiceUnavailable` response component.

#### Fix 2.3: Add Component Schemas
```yaml
components:
  responses:
    BadGateway:  # NEW
      description: Bad Gateway - Upstream fetch/request failed
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'

    ServiceUnavailable:  # ALREADY DOCUMENTED
      description: Service Unavailable - Dependency or resource unavailable
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'
```

---

## 3. CRITICAL: Server Error (19 failures)

### Problem Description
Unexpected **5xx errors** (500, 502, 503) during normal operations, indicating runtime issues or missing error handling.

### Error Categories

#### Category 3.1: Missing External Dependencies (12 errors)
**Pattern:** Features requiring external services fail when unconfigured

**Examples:**
- Serper API key missing → 503 on `/deepsearch`
- LLM provider unconfigured → 503 on LLM endpoints
- Redis connection failure → 503 on session/worker endpoints

**Root Cause:** Optional dependencies treated as required

**Fix Approach:**
1. Add graceful degradation for optional features
2. Return 503 with clear error messages
3. Document dependency requirements in OpenAPI
4. Add health check endpoints for each dependency

#### Category 3.2: External Fetch Failures (5 errors)
**Pattern:** Fetching external URLs results in 502 errors

**Examples:**
- `https://example.com/article` → 404 → API returns 502
- DNS resolution failures → 502
- Connection timeouts → 502

**Root Cause:** Expected behavior but undocumented status code

**Fix Approach:**
1. Document 502 responses in OpenAPI (see Section 2)
2. Improve error messages with actionable guidance
3. Add retry-after headers for retryable errors

#### Category 3.3: Validation Edge Cases (2 errors)
**Pattern:** Certain input combinations trigger unexpected errors

**Fix Approach:**
1. Add integration tests for edge cases
2. Improve input validation to catch issues earlier (400 instead of 500)

### Recommended Actions

#### Action 3.1: Dependency Documentation
Add to OpenAPI info section:

```yaml
info:
  description: |
    ...

    ## Optional Dependencies
    Some endpoints require external services:
    - `/deepsearch`, `/search`: Requires `SERPER_API_KEY` environment variable
    - `/api/v1/llm/*`: Requires configured LLM provider
    - Session endpoints: Requires Redis connection
    - Browser endpoints: Requires Chrome/Chromium installation

    Endpoints return 503 when dependencies are unavailable.
```

#### Action 3.2: Health Check Enhancement
Ensure `/health/detailed` reports status of all dependencies:
- Redis connection
- Search provider availability
- LLM provider status
- Browser pool status

#### Action 3.3: Error Handling Audit
Review error handling in:
```
/workspaces/eventmesh/crates/riptide-api/src/handlers/*.rs
/workspaces/eventmesh/crates/riptide-core/src/error.rs
```

Ensure all errors map to appropriate HTTP status codes.

---

## 4. MEDIUM: Unsupported Method Incorrect Response (15 failures)

### Problem Description
Endpoints return incorrect responses for unsupported HTTP methods (e.g., OPTIONS, PATCH on GET-only endpoints).

### Expected vs Actual Behavior

| Scenario | Expected | Actual | Status |
|----------|----------|--------|--------|
| **OPTIONS request** | 200 with `Allow` header | Varies | ❌ |
| **Unsupported method** | 405 Method Not Allowed with `Allow` header | 404 or 400 | ❌ |
| **HEAD on GET endpoint** | 200 with headers, no body | May return 405 | ⚠️ |

### Test Failures
From temp2.md test output:
```
test middleware::request_validation::tests::test_get_allowed_methods_websocket ... FAILED
test tests::middleware_validation_tests::tests::test_get_allowed_methods_websocket ... FAILED

assertion failed: methods.contains("GET")
```

This indicates WebSocket endpoint method validation is broken.

### Common Patterns

#### Pattern 4.1: WebSocket Endpoint Method Confusion
**Location:** `/workspaces/eventmesh/crates/riptide-api/src/middleware/request_validation.rs:338`

**Issue:** WebSocket upgrade endpoints expecting GET but validation fails

**Affected:**
- WebSocket endpoints
- Server-Sent Events (SSE) endpoints
- Streaming endpoints

#### Pattern 4.2: Missing Allow Header
**Issue:** 405 responses don't include `Allow` header listing valid methods

**Fix:**
```rust
// In request_validation.rs
fn method_not_allowed_response(allowed: &[Method]) -> Response {
    Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .header("Allow", allowed.join(", "))
        .header("Content-Type", "application/json")
        .body(json!({
            "error": {
                "message": "Method not allowed",
                "allowed_methods": allowed,
                "status": 405,
                "type": "method_not_allowed"
            }
        }))
}
```

### Recommended Fixes

#### Fix 4.1: Fix WebSocket Method Validation (IMMEDIATE)
**File:** `/workspaces/eventmesh/crates/riptide-api/src/middleware/request_validation.rs`

**Current Issue:** Line 338 assertion failing - WebSocket endpoints should allow GET

**Investigation Required:**
```bash
# Check current implementation
grep -A 10 "fn get_allowed_methods" /workspaces/eventmesh/crates/riptide-api/src/middleware/request_validation.rs

# Check WebSocket endpoint definitions
grep -r "WebSocket\|ws://" /workspaces/eventmesh/crates/riptide-api/src/
```

#### Fix 4.2: OpenAPI Method Documentation
Ensure all endpoints document supported methods:

```yaml
/api/v1/stream:
  get:
    summary: WebSocket upgrade endpoint
    description: |
      Upgrades HTTP connection to WebSocket.
      Requires `Upgrade: websocket` and `Connection: Upgrade` headers.
    responses:
      '101':
        description: Switching Protocols - WebSocket established
      '400':
        $ref: '#/components/responses/BadRequest'
      '405':
        description: Method Not Allowed
        headers:
          Allow:
            schema:
              type: string
              example: "GET, OPTIONS"
```

#### Fix 4.3: Add OPTIONS Support
All endpoints should support OPTIONS for CORS preflight:

```rust
// Add global OPTIONS handler
.route(web::route().method(Method::OPTIONS).to(handle_options))
```

---

## 5. CRITICAL: Response Violates Schema (7 failures)

### Problem Description
API responses don't match the schema defined in OpenAPI, causing contract violations.

### Potential Issues

#### Issue 5.1: Missing Required Fields
**Symptom:** Schema requires field but response omits it

**Example:**
```yaml
# OpenAPI defines
response:
  required: [id, status, data]

# API returns
{"id": "123", "status": "ok"}  # Missing "data"
```

#### Issue 5.2: Wrong Data Types
**Example:**
- Schema: `"count": { "type": "integer" }`
- Response: `"count": "42"` (string instead of integer)

#### Issue 5.3: Additional Properties
If `additionalProperties: false`, responses with extra fields violate schema.

### Investigation Required
Without specific failure details, need to:

1. **Run detailed Schemathesis report:**
```bash
schemathesis run docs/02-api-reference/openapi.yaml \
  --base-url http://localhost:8080 \
  --checks all \
  --report \
  --output-file=detailed-failures.json
```

2. **Validate response schemas:**
```bash
# Check for schema consistency
spectral lint docs/02-api-reference/openapi.yaml \
  --ruleset .spectral.yaml \
  --format stylish
```

### Recommended Actions

#### Action 5.1: Enable Response Validation in Tests
Add response schema validation to integration tests:

```rust
// In tests
#[test]
fn test_response_matches_schema() {
    let response = app.post("/extract").json(&request).await;
    assert_matches_openapi_schema(&response, "/extract", "200");
}
```

#### Action 5.2: Runtime Response Validation (Development)
Add middleware to validate responses in development:

```rust
#[cfg(debug_assertions)]
.wrap(ResponseSchemaValidator::new("docs/02-api-reference/openapi.yaml"))
```

---

## 6. HIGH: API Accepted Schema-Violating Request (7 failures)

### Problem Description
API accepts requests that violate the OpenAPI schema, indicating missing validation.

### Common Patterns

#### Pattern 6.1: Missing Required Field Validation
**Example:**
```yaml
# Schema requires field
requestBody:
  required: true
  content:
    application/json:
      schema:
        required: [url]
        properties:
          url: { type: string }

# But API accepts:
{}  # No "url" field
```

#### Pattern 6.2: Type Coercion Hides Violations
**Example:**
- Schema expects `integer`
- API accepts `"123"` (string) and coerces to integer
- Schemathesis reports this as schema violation

#### Pattern 6.3: Regex Pattern Not Enforced
**Example:**
```yaml
pattern: '^https?://'

# But API accepts:
"url": "ftp://example.com"  # Wrong scheme
```

### Root Cause Analysis
1. **Serde deserialization** may be too lenient (auto-coercion)
2. **Validation runs after deserialization** - malformed input already accepted
3. **OpenAPI schema stricter than Rust types**

### Recommended Fixes

#### Fix 6.1: Strict Deserialization
```rust
// Use #[serde(deny_unknown_fields)] to reject extra fields
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExtractRequest {
    url: String,
    #[serde(default)]
    mode: ExtractionMode,
}
```

#### Fix 6.2: Validation Attributes
```rust
use validator::Validate;

#[derive(Deserialize, Validate)]
struct CrawlRequest {
    #[validate(length(min = 1, max = 100))]
    urls: Vec<String>,

    #[validate(url, custom = "validate_http_scheme")]
    url: String,
}
```

#### Fix 6.3: Request Validation Middleware Enhancement
Ensure middleware validates:
- Required fields presence
- Type correctness (no coercion)
- Pattern matching
- Enum values
- Array lengths
- Nested object validation

---

## 7. MEDIUM: Undocumented Content-Type (6 failures)

### Problem Description
API accepts or returns Content-Types not documented in OpenAPI specification.

### Likely Scenarios

#### Scenario 7.1: Multipart Form Data
**Endpoints:** File upload operations
```yaml
# Missing from OpenAPI
requestBody:
  content:
    multipart/form-data:  # ADD THIS
      schema:
        type: object
        properties:
          file:
            type: string
            format: binary
```

#### Scenario 7.2: NDJSON Streaming
**Endpoints:** `/crawl/stream`, streaming operations
```yaml
responses:
  '200':
    content:
      application/x-ndjson:  # ADD THIS
        schema:
          type: string
      text/event-stream:  # SSE
        schema:
          type: string
```

#### Scenario 7.3: PDF/Binary Responses
**Endpoints:** `/pdf/process`, `/render`
```yaml
responses:
  '200':
    content:
      application/pdf:  # ADD THIS
        schema:
          type: string
          format: binary
```

### Investigation Checklist
Check these files for Content-Type handling:
```
/workspaces/eventmesh/crates/riptide-api/src/handlers/streaming.rs
/workspaces/eventmesh/crates/riptide-api/src/streaming/ndjson.rs
/workspaces/eventmesh/crates/riptide-api/src/streaming/sse.rs
```

### Recommended Fixes

#### Fix 7.1: Document All Content-Types
Audit all handlers and document accepted/returned types:

```bash
# Find all Content-Type handling
grep -r "content-type\|ContentType\|application/" crates/riptide-api/src/handlers/
```

#### Fix 7.2: Add to OpenAPI
```yaml
components:
  schemas:
    # Binary data
    BinaryData:
      type: string
      format: binary

    # NDJSON stream
    NDJSONStream:
      type: string
      description: Newline-delimited JSON stream
      example: |
        {"id":1,"url":"https://example.com","status":"complete"}
        {"id":2,"url":"https://example.org","status":"pending"}
```

---

## Priority Matrix for Fixes

### 🔴 PRIORITY 1: CRITICAL (Must Fix Immediately)

| Issue | Impact | Effort | Affected Items | Target |
|-------|--------|--------|----------------|--------|
| **Schema-compliant request rejection** | User-facing errors | Medium | 98 cases | Week 1 |
| **Missing 503 responses** | Error handling | Low | 96 endpoints | Week 1 |
| **Server errors (dependency issues)** | Availability | Medium | 19 cases | Week 1 |
| **Response schema violations** | Contract compliance | High | 7 cases | Week 1 |
| **WebSocket method validation bug** | Feature broken | Low | 2 tests | Day 1 |

### 🟠 PRIORITY 2: HIGH (Fix Soon)

| Issue | Impact | Effort | Affected Items | Target |
|-------|--------|--------|----------------|--------|
| **Missing 502 responses** | Documentation | Low | 30 endpoints | Week 2 |
| **Missing 400 responses** | Already documented | Low | 34 endpoints | Week 2 |
| **Schema-violating request acceptance** | Security/Validation | Medium | 7 cases | Week 2 |

### 🟡 PRIORITY 3: MEDIUM (Improve)

| Issue | Impact | Effort | Affected Items | Target |
|-------|--------|--------|----------------|--------|
| **Missing 415 responses** | Edge case errors | Low | 40 endpoints | Week 3 |
| **Unsupported method responses** | HTTP compliance | Medium | 15 cases | Week 3 |
| **Undocumented Content-Types** | API clarity | Low | 6 cases | Week 3 |
| **Missing 204 on DELETE** | REST compliance | Low | 7 endpoints | Week 3 |

---

## Recommended Fix Approach by Category

### Category 1: Schema-Compliant Rejection (98 failures)
**Approach:** Two-pronged fix

1. **Immediate (Day 1-3):** Add missing constraints to OpenAPI
   - Audit validation.rs for all constraint rules
   - Add minLength, maxLength, pattern to OpenAPI schemas
   - Document business rules in descriptions

2. **Short-term (Week 1):** Relax overly strict validation
   - Identify validations that are too restrictive
   - Align with REST principles (be liberal in what you accept)
   - Move business logic validation after schema validation

3. **Long-term (Week 2-3):** Automated validation sync
   - Generate OpenAPI constraints from Rust validation attributes
   - Add CI check: compare OpenAPI vs. code validation rules
   - Tool: Custom build script or macro

**Tools:**
```bash
# Extract validators from Rust code
rg "#\[validate\(" crates/riptide-api/src/ -A 2

# Compare with OpenAPI constraints
python scripts/compare_validations.py
```

### Category 2: Undocumented Status Codes (59 failures)
**Approach:** Bulk OpenAPI update

1. **Immediate (Day 1):** Add response components
   ```yaml
   components:
     responses:
       BadGateway: {...}
       ServiceUnavailable: {...}
   ```

2. **Automated (Day 2):** Script to add to all endpoints
   ```python
   # scripts/add_status_codes.py
   import yaml

   def add_standard_errors(endpoint, method):
       if method in ['POST', 'PUT', 'PATCH']:
           endpoint['responses']['400'] = {'$ref': '#/components/responses/BadRequest'}
           endpoint['responses']['415'] = {'$ref': '#/components/responses/UnsupportedMediaType'}

       if uses_external_fetch(endpoint):
           endpoint['responses']['502'] = {'$ref': '#/components/responses/BadGateway'}

       endpoint['responses']['503'] = {'$ref': '#/components/responses/ServiceUnavailable'}
   ```

3. **Validation (Day 3):** Re-run Schemathesis
   ```bash
   schemathesis run docs/02-api-reference/openapi.yaml \
     --base-url http://localhost:8080 \
     --hypothesis-max-examples=100
   ```

### Category 3: Server Errors (19 failures)
**Approach:** Improve error handling and documentation

1. **Immediate (Day 1):** Fix WebSocket test failure
   ```rust
   // request_validation.rs line 338
   // Ensure WebSocket endpoints return ["GET", "OPTIONS"]
   ```

2. **Short-term (Week 1):** Graceful degradation
   - Add feature flags for optional dependencies
   - Return clear 503 errors with setup instructions
   - Update health check to report dependency status

3. **Medium-term (Week 2):** Integration tests
   - Test all dependency failure scenarios
   - Ensure proper status codes returned
   - Validate error message quality

### Category 4: Schema Violations (Sent/Received) (14 failures)
**Approach:** Strict validation both directions

1. **Response validation (Week 1):**
   ```rust
   #[cfg(test)]
   fn validate_response_schema<T: Serialize>(response: &T, endpoint: &str) {
       let openapi = load_openapi_spec();
       let schema = openapi.get_response_schema(endpoint, "200");
       assert!(validate_json_schema(&serde_json::to_value(response), &schema));
   }
   ```

2. **Request validation (Week 1):**
   ```rust
   #[derive(Deserialize, Validate)]
   #[serde(deny_unknown_fields)]  // Reject extra fields
   struct Request {
       #[validate(length(min = 1))]
       field: String,
   }
   ```

3. **CI enforcement (Week 2):**
   - Add schema validation to CI pipeline
   - Block merges if schemas don't match
   - Generate types from OpenAPI (openapi-generator)

---

## Code Locations Requiring Changes

### High Priority Files
```
1. /workspaces/eventmesh/docs/02-api-reference/openapi.yaml
   - Add missing status codes (502, 503, 400, 415)
   - Add missing constraints (minLength, pattern)
   - Add response components

2. /workspaces/eventmesh/crates/riptide-api/src/middleware/request_validation.rs
   - Fix WebSocket method validation (line 338)
   - Add Allow header to 405 responses
   - Align validation with OpenAPI

3. /workspaces/eventmesh/crates/riptide-api/src/validation.rs
   - Document all validation rules
   - Extract constraints for OpenAPI sync
   - Add validation error details

4. /workspaces/eventmesh/crates/riptide-core/src/error.rs
   - Ensure all error types map to correct status codes
   - Add retryable flag to all errors
   - Improve error messages
```

### Medium Priority Files
```
5. /workspaces/eventmesh/crates/riptide-api/src/handlers/*.rs
   - Add response schema validation
   - Document Content-Types
   - Handle dependency failures gracefully

6. /workspaces/eventmesh/crates/riptide-api/src/streaming/*.rs
   - Document NDJSON and SSE content types
   - Add to OpenAPI spec

7. Integration tests
   - Add schema compliance tests
   - Test all error scenarios
   - Validate response structures
```

---

## Automation Scripts Needed

### 1. Validation Rule Extractor
```python
# scripts/extract_validators.py
"""Extract validation rules from Rust code and generate OpenAPI constraints."""

import re
import yaml

def extract_validators(rust_file):
    validators = {}
    # Parse #[validate(...)] attributes
    # Generate OpenAPI constraints
    return validators

def update_openapi(constraints):
    with open('docs/02-api-reference/openapi.yaml', 'r') as f:
        spec = yaml.safe_load(f)

    # Apply constraints to matching schemas
    # Save updated spec
```

### 2. Bulk Status Code Adder
```python
# scripts/add_missing_status_codes.py
"""Add standard error responses to all endpoints."""

import yaml

def add_standard_responses(openapi_file):
    with open(openapi_file, 'r') as f:
        spec = yaml.safe_load(f)

    for path, methods in spec['paths'].items():
        for method, operation in methods.items():
            if method == 'parameters':
                continue

            # Add 503 to all
            operation.setdefault('responses', {})['503'] = {
                '$ref': '#/components/responses/ServiceUnavailable'
            }

            # Add 400, 415 to mutations
            if method in ['post', 'put', 'patch']:
                operation['responses']['400'] = {'$ref': '#/components/responses/BadRequest'}
                if 'requestBody' in operation:
                    operation['responses']['415'] = {'$ref': '#/components/responses/UnsupportedMediaType'}

    # Save updated spec
    with open(openapi_file, 'w') as f:
        yaml.dump(spec, f, sort_keys=False)
```

### 3. Schema Validation CI Check
```bash
#!/bin/bash
# scripts/validate_schema_sync.sh

# 1. Extract Rust validators
python scripts/extract_validators.py > /tmp/rust-validators.json

# 2. Extract OpenAPI constraints
python scripts/extract_openapi_constraints.py > /tmp/openapi-constraints.json

# 3. Compare
python scripts/compare_validations.py \
  /tmp/rust-validators.json \
  /tmp/openapi-constraints.json

# 4. Fail if mismatches found
exit $?
```

---

## Testing Strategy Post-Fix

### Phase 1: Unit Tests (Week 1)
```rust
#[cfg(test)]
mod validation_tests {
    #[test]
    fn test_all_validators_in_openapi() {
        // Ensure every #[validate] rule has OpenAPI equivalent
    }

    #[test]
    fn test_error_status_codes() {
        // Test each error type returns correct status
    }
}
```

### Phase 2: Integration Tests (Week 2)
```rust
#[actix_web::test]
async fn test_schema_compliant_requests_accepted() {
    // Generate requests from OpenAPI examples
    // Ensure all are accepted (no 400 for valid input)
}

#[actix_web::test]
async fn test_schema_violations_rejected() {
    // Generate invalid requests
    // Ensure proper 400 responses
}
```

### Phase 3: Schemathesis Re-run (Week 3)
```bash
# Target: < 10 failures (from 211)
schemathesis run docs/02-api-reference/openapi.yaml \
  --base-url http://localhost:8080 \
  --checks all \
  --hypothesis-max-examples=500 \
  --hypothesis-seed=113530525868578902937193447883367590929  # Use same seed
```

### Phase 4: Continuous Validation (Ongoing)
```yaml
# .github/workflows/api-validation.yml
- name: Schemathesis (strict)
  run: |
    schemathesis run docs/02-api-reference/openapi.yaml \
      --url http://localhost:8080 \
      --checks all \
      --max-examples=100 \
      --exitfirst  # Fail on first error
```

---

## Success Metrics

### Target Reduction After Fixes

| Category | Current | Target | % Reduction |
|----------|---------|--------|-------------|
| Schema-compliant rejection | 98 | **< 5** | **95%** |
| Undocumented status codes | 59 | **0** | **100%** |
| Server errors | 19 | **< 3** | **84%** |
| Method responses | 15 | **0** | **100%** |
| Response schema violations | 7 | **0** | **100%** |
| Schema-violating acceptance | 7 | **0** | **100%** |
| Undocumented Content-Types | 6 | **0** | **100%** |
| **TOTAL** | **211** | **< 10** | **>95%** |

### Key Performance Indicators (KPIs)

1. **Schemathesis pass rate:** 95%+ (currently ~71%)
2. **Zero undocumented status codes** in OpenAPI
3. **Zero schema mismatches** between Rust and OpenAPI
4. **All integration tests pass** with schema validation enabled
5. **CI pipeline green** with strict Schemathesis checks

---

## Next Steps - Action Plan

### Week 1: Critical Fixes
- [ ] **Day 1:** Fix WebSocket test failures (2 tests)
- [ ] **Day 1-2:** Add missing response components to OpenAPI (502, 503, BadGateway, ServiceUnavailable)
- [ ] **Day 2-3:** Run automation script to add 503 to all 96 endpoints
- [ ] **Day 3-5:** Audit validation.rs and add constraints to OpenAPI (focus on top 20 endpoints)
- [ ] **Day 5:** Re-run Schemathesis and measure reduction

### Week 2: High Priority
- [ ] Add 502 responses to external-dependent endpoints (30)
- [ ] Add 400 responses to mutation endpoints (34)
- [ ] Fix schema-violating request acceptance (add strict validation)
- [ ] Add integration tests for error scenarios
- [ ] Document dependency requirements in OpenAPI

### Week 3: Medium Priority
- [ ] Add 415 responses to endpoints with request bodies (40)
- [ ] Add 204 to DELETE endpoints (7)
- [ ] Document all Content-Types (NDJSON, SSE, PDF)
- [ ] Fix unsupported method responses (OPTIONS support, Allow header)
- [ ] Run final Schemathesis validation

### Week 4: Automation & CI
- [ ] Build validation sync tools
- [ ] Add schema validation to CI pipeline
- [ ] Set up continuous Schemathesis monitoring
- [ ] Document validation architecture
- [ ] Create runbook for future schema changes

---

## Coordination Memory Storage

This analysis will be stored in coordination memory for access by other agents:

**Memory Keys:**
- `swarm/researcher/schemathesis-analysis` - Full analysis
- `swarm/shared/priority-fixes` - Priority matrix
- `swarm/shared/affected-endpoints` - Endpoint groupings
- `swarm/shared/fix-approaches` - Recommended strategies
- `swarm/coder/openapi-changes` - Required OpenAPI modifications
- `swarm/tester/test-scenarios` - Test cases to add

---

**Generated by:** Research Agent
**Coordination:** Via claude-flow memory hooks
**Next Agent:** Coder (for OpenAPI updates) + Tester (for validation tests)
