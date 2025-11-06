# Facade Layer Architecture Violations Analysis

**Date:** 2025-11-06
**Scope:** `crates/riptide-facade/src/`
**Rule:** Facade layer must handle *workflows only* — not transport

---

## Executive Summary

Analysis of the facade layer reveals **multiple architectural violations** where transport-level concerns have leaked into the workflow orchestration layer. While no direct HTTP framework dependencies (axum, actix, tower) were found, there are significant violations involving:

1. **HTTP method definitions** in facade code
2. **Extensive JSON serialization logic** beyond domain types
3. **Transport-level error handling** mixed with domain logic
4. **Request/response data structures** (headers, methods)

### Violation Summary

| Category | Count | Severity | Files Affected |
|----------|-------|----------|----------------|
| HTTP Types | 3 | HIGH | pipeline.rs |
| Serialization Logic | 35+ | MEDIUM | pipeline.rs, browser.rs, extractor.rs |
| Transport Structures | 4 | HIGH | pipeline.rs |
| Config Violations | 2 | MEDIUM | config.rs |

---

## Detailed Violations

### 1. HTTP Method Definitions in Facade (HIGH SEVERITY)

#### Violation: HttpMethod Enum in Pipeline Facade

**File:** `/workspaces/eventmesh/crates/riptide-facade/src/facades/pipeline.rs`
**Lines:** 441-445, 267, 426, 434, 442

```rust
// ❌ WRONG: HTTP method enum in facade layer
#[derive(Debug, Clone)]
pub enum HttpMethod {
    Get,
    Post,
}

// Usage in FetchOptions
pub struct FetchOptions {
    pub method: HttpMethod,      // Line 426
    pub headers: Vec<(String, String)>,  // Also transport concern
    pub timeout: Duration,
}

// Usage in template
options: FetchOptions {
    method: HttpMethod::Get,      // Line 267
    headers: vec![("Accept".to_string(), "application/pdf".to_string())],
    timeout: Duration::from_secs(60),
}
```

**Rule Violated:** Facade defines HTTP-specific types (GET, POST)

**Impact:**
- Couples facade to HTTP protocol details
- Makes it impossible to support other transports without changing facade
- Violates separation of concerns

**Suggested Fix:**
```rust
// ✅ CORRECT: Move to riptide-api or handlers
// In crates/riptide-api/src/types.rs
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

// In facade, use domain operations instead
pub enum FetchOperation {
    Retrieve,    // Maps to GET
    Submit,      // Maps to POST
    Update,      // Maps to PUT
    Remove,      // Maps to DELETE
}
```

---

### 2. HTTP Headers in Facade Structure (HIGH SEVERITY)

#### Violation: Transport Headers in FetchOptions

**File:** `/workspaces/eventmesh/crates/riptide-facade/src/facades/pipeline.rs`
**Lines:** 427

```rust
// ❌ WRONG: HTTP headers as Vec<(String, String)>
pub struct FetchOptions {
    pub method: HttpMethod,
    pub headers: Vec<(String, String)>,  // Raw HTTP headers
    pub timeout: Duration,
}
```

**Rule Violated:** Transport-level headers exposed in facade API

**Suggested Fix:**
```rust
// ✅ CORRECT: Use domain-level metadata
pub struct FetchOptions {
    pub operation: FetchOperation,
    pub metadata: HashMap<String, String>,  // Generic metadata
    pub timeout: Duration,
}

// Let handlers convert metadata to HTTP headers
impl From<FetchOptions> for HttpRequest {
    fn from(opts: FetchOptions) -> Self {
        let mut req = HttpRequest::new(opts.operation.to_http_method());
        for (key, value) in opts.metadata {
            req.insert_header(key, value);
        }
        req
    }
}
```

---

### 3. Excessive JSON Serialization in Facade (MEDIUM-HIGH SEVERITY)

#### Violation: Direct serde_json Usage in Business Logic

**File:** `/workspaces/eventmesh/crates/riptide-facade/src/facades/pipeline.rs`
**Lines:** 186-195, 202-207, 214-216, 223-225, 232-237, 460-462, 470-472, 498, 507, 522-523, 530, 534, 538, 542, 549, 559, 563, 576-578, 597-599

```rust
// ❌ WRONG: Pipeline stages use serde_json::Value everywhere
async fn execute_fetch(
    &self,
    url: &str,
    _options: &FetchOptions,
    _context: &PipelineContext,
) -> RiptideResult<serde_json::Value> {   // Line 186
    Ok(serde_json::json!({                 // Line 188
        "url": url,
        "content": format!("Fetched content from {}", url),
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs(),
    }))
}

// ❌ WRONG: Context stores serde_json::Value
struct PipelineContext {
    data: HashMap<String, serde_json::Value>,      // Line 522
    current_output: serde_json::Value,             // Line 523
}

// ❌ WRONG: Public API returns JSON values
pub struct PipelineResult {
    pub final_output: serde_json::Value,           // Line 498
}

pub struct StageResult {
    pub output: serde_json::Value,                 // Line 507
}
```

**Rule Violated:** Serialization logic belongs in handlers, not facade

**Impact:**
- Facade is doing format conversion instead of business logic
- Tight coupling to JSON format
- Cannot change serialization format without changing facade
- Mixes transport concerns (JSON) with domain workflows

**Suggested Fix:**
```rust
// ✅ CORRECT: Use domain types
pub enum PipelineOutput {
    FetchResult(FetchData),
    ExtractResult(ExtractedData),
    TransformResult(TransformedData),
    ValidateResult(ValidationResult),
    StoreResult(StorageConfirmation),
}

pub struct FetchData {
    pub url: String,
    pub content: Vec<u8>,
    pub timestamp: SystemTime,
    pub metadata: HashMap<String, String>,
}

pub struct PipelineResult {
    pub stages_completed: usize,
    pub total_duration: Duration,
    pub stage_results: Vec<StageResult>,
    pub final_output: PipelineOutput,  // Domain type, not JSON
}

// Let handlers serialize to JSON when needed
impl From<PipelineOutput> for serde_json::Value {
    fn from(output: PipelineOutput) -> Self {
        // Serialization happens in handler layer
    }
}
```

---

### 4. Serialization in Browser Facade (MEDIUM SEVERITY)

**File:** `/workspaces/eventmesh/crates/riptide-facade/src/facades/browser.rs`
**Lines:** 451, 821, 826, 829, 928

```rust
// ❌ WRONG: execute_script returns serde_json::Value
pub async fn execute_script(
    &self,
    session: &BrowserSession<'_>,
    script: &str,
) -> RiptideResult<serde_json::Value> {    // Line 451
    let page = &session.session.page;
    let result = page.evaluate(script).await
        .map_err(|e| RiptideError::Fetch(format!("Script execution failed: {}", e)))?;

    let value = result.into_value()
        .map_err(|e| RiptideError::Fetch(format!("Failed to parse script result: {}", e)))?;

    Ok(value)
}

// ❌ WRONG: get_local_storage returns serde_json::Value
pub async fn get_local_storage(
    &self,
    session: &BrowserSession<'_>,
) -> RiptideResult<serde_json::Value> {     // Line 821
    let script = "JSON.stringify(localStorage)";
    let result = self.execute_script(session, script).await?;

    if let Some(storage_str) = result.as_str() {
        serde_json::from_str(storage_str)       // Line 826
            .map_err(|e| RiptideError::Extraction(format!("Failed to parse storage: {}", e)))
    } else {
        Ok(serde_json::json!({}))               // Line 829
    }
}
```

**Rule Violated:** Browser facade performs JSON parsing/serialization

**Suggested Fix:**
```rust
// ✅ CORRECT: Return domain types
pub struct ScriptResult {
    pub value: String,
    pub value_type: ScriptValueType,
}

pub enum ScriptValueType {
    String(String),
    Number(f64),
    Boolean(bool),
    Object(HashMap<String, String>),
    Array(Vec<String>),
}

pub struct LocalStorage {
    pub entries: HashMap<String, String>,
}

// Facade returns domain types
pub async fn execute_script(
    &self,
    session: &BrowserSession<'_>,
    script: &str,
) -> RiptideResult<ScriptResult> {
    // Returns domain type
}

pub async fn get_local_storage(
    &self,
    session: &BrowserSession<'_>,
) -> RiptideResult<LocalStorage> {
    // Returns domain type
}

// Handler converts to JSON
impl Serialize for ScriptResult {
    // Serialization in handler layer
}
```

---

### 5. Serialization in Extractor Facade (MEDIUM SEVERITY)

**File:** `/workspaces/eventmesh/crates/riptide-facade/src/facades/extractor.rs`
**Lines:** 500-539

```rust
// ❌ WRONG: extract_schema returns serde_json::Value
pub async fn extract_schema(
    &self,
    html: &str,
    _url: &str,
    schema: &Schema,
) -> Result<serde_json::Value> {            // Line 500
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);
    let mut result = serde_json::Map::new();  // Line 504

    for (field_name, field_spec) in &schema.fields {
        let selector = Selector::parse(&field_spec.selector)
            .map_err(|e| RiptideError::extraction(format!("Invalid selector: {}", e)))?;

        let value = if let Some(element) = document.select(&selector).next() {
            let text = element.text().collect::<Vec<_>>().join(" ").trim().to_string();

            match field_spec.field_type {
                FieldType::Text => serde_json::Value::String(text),     // Line 519
                FieldType::Number => text.parse::<f64>()
                    .map(serde_json::Value::from)                       // Line 522
                    .unwrap_or(serde_json::Value::Null),                // Line 523
                FieldType::Url => serde_json::Value::String(text),      // Line 524
                FieldType::Date => serde_json::Value::String(text),     // Line 525
            }
        } else {
            serde_json::Value::Null                                     // Line 533
        };

        result.insert(field_name.clone(), value);
    }

    Ok(serde_json::Value::Object(result))                               // Line 539
}
```

**Rule Violated:** Schema extraction builds JSON in facade layer

**Suggested Fix:**
```rust
// ✅ CORRECT: Return domain type
pub struct SchemaExtractionResult {
    pub fields: HashMap<String, FieldValue>,
    pub missing_required: Vec<String>,
}

pub enum FieldValue {
    Text(String),
    Number(f64),
    Url(String),
    Date(String),
    Missing,
}

pub async fn extract_schema(
    &self,
    html: &str,
    url: &str,
    schema: &Schema,
) -> Result<SchemaExtractionResult> {
    let mut fields = HashMap::new();
    let mut missing_required = Vec::new();

    for (field_name, field_spec) in &schema.fields {
        match self.extract_field(html, field_spec) {
            Some(value) => {
                fields.insert(field_name.clone(), value);
            }
            None if field_spec.required => {
                missing_required.push(field_name.clone());
            }
            None => {
                fields.insert(field_name.clone(), FieldValue::Missing);
            }
        }
    }

    Ok(SchemaExtractionResult { fields, missing_required })
}

// Handler serializes to JSON
impl Serialize for SchemaExtractionResult {
    // In handler layer
}
```

---

### 6. Config Layer Violations (MEDIUM SEVERITY)

**File:** `/workspaces/eventmesh/crates/riptide-facade/src/config.rs`
**Lines:** 12, 22

```rust
// ❌ BORDERLINE: Config mentions "Request" in documentation
/// Request timeout duration                    // Line 12
pub timeout: Duration,

/// Additional headers to include in requests   // Line 22
pub headers: Vec<(String, String)>,
```

**Rule Violated:** Config uses transport-specific terminology

**Suggested Fix:**
```rust
// ✅ CORRECT: Use domain terminology
/// Operation timeout duration
pub timeout: Duration,

/// Additional metadata for operations
pub operation_metadata: HashMap<String, String>,
```

---

## Architecture Principles Violated

### 1. **Separation of Concerns**
- Pipeline facade defines HTTP methods (GET, POST)
- Browser facade performs JSON parsing
- Extractor facade builds JSON objects

### 2. **Layer Boundaries**
```
❌ CURRENT (Violated):
Handler → Facade (with HTTP/JSON) → Domain

✅ CORRECT:
Handler (HTTP/JSON) → Facade (workflows) → Domain
```

### 3. **Serialization Ownership**
- **Current:** Facade owns serialization (serde_json::Value everywhere)
- **Correct:** Handlers own serialization (facade uses domain types)

---

## Impact Assessment

### Testability
- **Issue:** Facade tests must mock JSON serialization
- **Fix:** Facade tests only verify business logic with domain types

### Extensibility
- **Issue:** Cannot add gRPC/GraphQL handlers without changing facade
- **Fix:** Facade agnostic to transport, handlers adapt to any protocol

### Maintainability
- **Issue:** Changes to JSON format require facade changes
- **Fix:** Format changes isolated to handler layer

### Type Safety
- **Issue:** `serde_json::Value` loses type information
- **Fix:** Strong domain types provide compile-time guarantees

---

## Recommended Refactoring Plan

### Phase 1: Extract HTTP Types (HIGH Priority)
1. Move `HttpMethod` to `riptide-api/src/types.rs`
2. Create domain `FetchOperation` enum in facade
3. Add conversion traits in handlers

**Estimate:** 2-4 hours
**Files:** 1 (pipeline.rs)

### Phase 2: Remove Headers from Facade (HIGH Priority)
1. Replace `Vec<(String, String)>` with `HashMap<String, String>`
2. Rename to `metadata` instead of `headers`
3. Update handlers to convert metadata to HTTP headers

**Estimate:** 2-3 hours
**Files:** 2 (pipeline.rs, config.rs)

### Phase 3: Replace JSON with Domain Types (MEDIUM Priority)
1. Create domain types for each stage output
2. Replace `serde_json::Value` in public APIs
3. Add serialization traits in handler layer
4. Update pipeline context to use domain types

**Estimate:** 8-12 hours
**Files:** 3 (pipeline.rs, browser.rs, extractor.rs)

### Phase 4: Clean Up Config (LOW Priority)
1. Update documentation terminology
2. Consider renaming `headers` to `metadata`

**Estimate:** 1 hour
**Files:** 1 (config.rs)

---

## Testing Strategy

### Before Refactoring
```bash
# Establish baseline
cargo test -p riptide-facade
cargo clippy -p riptide-facade -- -D warnings
```

### During Refactoring
```bash
# Test each phase
cargo test -p riptide-facade --no-fail-fast
cargo test -p riptide-api  # Handler tests
```

### After Refactoring
```bash
# Verify no transport coupling
rg "serde_json::Value" crates/riptide-facade/src/facades/
rg "HttpMethod|Response|Request" crates/riptide-facade/src/facades/
rg "StatusCode|Headers" crates/riptide-facade/src/facades/

# Should return zero matches in facade layer
```

---

## Files Affected Summary

| File | Lines | Violations | Priority |
|------|-------|------------|----------|
| `facades/pipeline.rs` | 779 | 15+ | HIGH |
| `facades/browser.rs` | 1298 | 8+ | MEDIUM |
| `facades/extractor.rs` | 782 | 12+ | MEDIUM |
| `config.rs` | 189 | 2 | LOW |
| **Total** | **3048** | **37+** | |

---

## Success Criteria

✅ **Zero HTTP types** in facade public APIs
✅ **Zero serde_json::Value** in facade return types
✅ **Zero transport headers** in facade structures
✅ **All serialization** happens in handler layer
✅ **Domain types** used throughout facade

---

## Conclusion

The facade layer currently violates architectural boundaries by:
1. Defining HTTP protocol types (methods, headers)
2. Performing JSON serialization (37+ instances)
3. Using transport-specific terminology
4. Coupling business logic to transport formats

**Recommended Action:** Execute 4-phase refactoring plan, starting with HIGH priority items (HTTP types and headers removal).

**Estimated Total Effort:** 13-20 hours

**Risk:** LOW - Changes are localized, well-defined, and testable incrementally.
