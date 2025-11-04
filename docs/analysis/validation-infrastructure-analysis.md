# Validation Infrastructure Analysis - RipTide Codebase

**Date:** 2025-11-04
**Objective:** Identify existing validation/schema infrastructure to avoid recreating functionality

---

## Executive Summary

The RipTide codebase has **extensive validation infrastructure** already in place across multiple crates. Before implementing new validation for the RipTide Design System, we should **reuse and enhance** existing patterns rather than recreating them.

**Key Findings:**
- ✅ Comprehensive validation framework in `riptide-config/src/validation.rs`
- ✅ Request validation middleware in `riptide-api/src/middleware/request_validation.rs`
- ✅ JSON Schema support via `schemars` crate (3 crates use it)
- ✅ DTO conversion patterns (From/Into traits)
- ✅ Adapter pattern for cross-crate integration
- ✅ Deduplication logic in spider and metadata extraction
- ❌ **NO** centralized JSON Schema validation for DTOs
- ❌ **NO** unified request/response DTO validation layer

---

## 1. Existing Validation Infrastructure

### 1.1 Core Validation Module: `riptide-config/src/validation.rs`

**Status:** ✅ **PRODUCTION-READY** - Comprehensive validation framework

**Features:**
- URL validation with security checks (private IPs, localhost, schemes)
- Content-type validation with allowlist
- HTTP header validation (size, suspicious patterns)
- Content size validation
- Query content validation (SQL injection, XSS detection)
- Configurable validation rules via `ValidationConfig`

**Code Example:**
```rust
pub struct CommonValidator {
    config: ValidationConfig,
}

impl CommonValidator {
    pub fn validate_url(&self, url_str: &str) -> Result<Url>
    pub fn validate_content_type(&self, content_type: &str) -> Result<()>
    pub fn validate_headers(&self, headers: &[(String, String)]) -> Result<()>
    pub fn validate_content_size(&self, size: usize) -> Result<()>
    pub fn validate_query_content(&self, query: &str) -> Result<()>
    pub fn is_private_or_local_address(&self, host: &str) -> bool
}
```

**Validation Rules:**
- **URL Length:** Max 2048 chars
- **Header Size:** Max 8192 bytes
- **Content Size:** Default 20MB
- **Blocked Patterns:** localhost, private IPs (10.0.0.0/8, 192.168.0.0/16, etc.)
- **Allowed Schemes:** http, https only
- **Security Checks:** SQL injection, XSS, path traversal detection

**Usage in Codebase:**
- Used in `riptide-api/src/validation.rs` for API request validation
- Provides reusable validators: `UrlValidator`, `SizeValidator`, `ParameterValidator`, `ContentTypeValidator`

**Recommendation:** ✅ **REUSE THIS** for RipTide Design System URL/content validation

---

### 1.2 API Validation: `riptide-api/src/validation.rs`

**Status:** ✅ **API-SPECIFIC** - Thin wrapper over CommonValidator

**Features:**
- Crawl request validation (URL count limits, URL patterns)
- Deep search request validation (query length, search limits)
- Suspicious pattern detection (file extensions, URL encoding)
- Delegates to `CommonValidator` for core validation

**Constants:**
```rust
const MAX_URLS_PER_REQUEST: usize = 100;
const MAX_QUERY_LENGTH: usize = 500;
const MAX_SEARCH_LIMIT: u32 = 50;
```

**Functions:**
```rust
pub fn validate_crawl_request(body: &CrawlBody) -> ApiResult<()>
pub fn validate_deepsearch_request(body: &DeepSearchBody) -> ApiResult<()>
```

**Recommendation:** ✅ **PATTERN TO FOLLOW** for design system specific validation

---

### 1.3 Request Validation Middleware: `riptide-api/src/middleware/request_validation.rs`

**Status:** ✅ **COMPREHENSIVE** - Production-grade middleware

**Features:**
- HTTP method validation (405 Method Not Allowed)
- Content-Type validation (415 Unsupported Media Type)
- Payload size validation (413 Payload Too Large)
- URL parameter sanitization (SQL injection, XSS, path traversal)
- Required header validation (API keys, Authorization)
- Path-specific allowed methods configuration

**Validation Checks:**
```rust
pub async fn request_validation_middleware(request: Request, next: Next) -> Response {
    // 1. Validate HTTP method (fast path)
    // 2. Validate and sanitize URL parameters
    // 3. Validate required headers
    // 4. Validate Content-Type for requests with bodies
    // 5. Validate payload size early
}
```

**Security Patterns Detected:**
- **SQL Injection:** `union`, `select`, `drop`, `--`, `/*`, etc.
- **XSS:** `<script`, `javascript:`, `onerror=`, `onload=`, `eval(`, etc.
- **Path Traversal:** `../`, `..\`
- **Control Characters:** Null bytes, non-printable chars

**Path-Specific Method Validation:**
```rust
fn get_allowed_methods(path: &str) -> HashSet<&'static str> {
    // Health/metrics: GET, HEAD only
    // Crawl endpoints: POST only
    // Browser/LLM APIs: GET, POST, PUT, PATCH, DELETE
    // WebSocket: GET only
}
```

**Recommendation:** ✅ **REUSE PATTERN** for design system endpoint validation

---

### 1.4 CLI Spec Validation: `cli-spec/src/validation.rs`

**Status:** ✅ **SPEC-SPECIFIC** - CLI specification validator

**Features:**
- Validates CLI spec metadata (version, name, about)
- Validates commands (name, description, examples)
- Validates exit codes (success must be 0)
- Validates error mapping (4xx → user_error, 5xx → server_error)

**Recommendation:** ⚠️ **NOT APPLICABLE** to design system (CLI-specific)

---

## 2. JSON Schema Support

### 2.1 Current Usage of `schemars` Crate

**Crates Using `schemars`:**
1. **`riptide-extraction`** - For extraction strategies and metadata
2. **`riptide-spider`** - For spider configuration and types
3. **Workspace root** - Version 0.8 with chrono features

**Example from `riptide-extraction/src/strategies/metadata.rs`:**
```rust
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub published_date: Option<DateTime<Utc>>,
    pub confidence_scores: MetadataConfidence,
    pub extraction_method: ExtractionMethod,
}
```

**Current Scope:**
- ✅ Schema **generation** (derive macro)
- ❌ **NO** runtime schema validation
- ❌ **NO** request/response DTO validation

**Recommendation:**
- ✅ **ADD** `jsonschema` crate for runtime validation
- ✅ **EXTEND** existing `JsonSchema` derives for DTOs
- ✅ **CREATE** validation middleware using generated schemas

---

## 3. DTO and Conversion Patterns

### 3.1 API Models: `riptide-api/src/models.rs`

**Request/Response DTOs:**
```rust
// Request DTOs
pub struct CrawlBody {
    pub urls: Vec<String>,
    pub options: Option<CrawlOptions>,
}

pub struct DeepSearchBody {
    pub query: String,
    pub limit: Option<u32>,
    pub country: Option<String>,
    pub locale: Option<String>,
    pub include_content: Option<bool>,
    pub crawl_options: Option<CrawlOptions>,
}

// Response DTOs
pub struct CrawlResponse {
    pub total_urls: usize,
    pub successful: usize,
    pub failed: usize,
    pub from_cache: usize,
    pub results: Vec<CrawlResult>,
    pub statistics: CrawlStatistics,
}

pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub dependencies: DependencyStatus,
    pub metrics: Option<SystemMetrics>,
}
```

**Observations:**
- ✅ Consistent DTO naming: `*Body`, `*Response`, `*Request`
- ✅ Optional fields for flexibility
- ✅ Statistics embedded in responses
- ❌ **NO** validation attributes (e.g., `#[validate(length(min = 1))]`)
- ❌ **NO** JSON Schema validation

---

### 3.2 DTO Conversion Pattern: `riptide-api/src/dto.rs`

**Spider Result Conversion:**
```rust
impl From<&riptide_facade::facades::spider::CrawlSummary> for SpiderResultStats {
    fn from(summary: &riptide_facade::facades::spider::CrawlSummary) -> Self {
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

**Patterns:**
- ✅ Uses standard `From`/`Into` traits
- ✅ Converts internal types to API-friendly DTOs
- ✅ Field filtering support (`FieldFilter`)
- ✅ Content truncation for large payloads

**Recommendation:** ✅ **FOLLOW THIS PATTERN** for design system DTOs

---

### 3.3 Service Health Conversion

**`riptide-api/src/models.rs` (line 275):**
```rust
impl From<DependencyHealth> for ServiceHealth {
    fn from(health: DependencyHealth) -> Self {
        match health {
            DependencyHealth::Healthy => ServiceHealth { status: "healthy".to_string(), ... },
            DependencyHealth::Unhealthy(msg) => ServiceHealth { status: "unhealthy".to_string(), message: Some(msg), ... },
            DependencyHealth::Unknown => ServiceHealth { status: "unknown".to_string(), ... },
        }
    }
}
```

**Recommendation:** ✅ **USE SIMILAR PATTERN** for error type conversions

---

## 4. Adapter and Transformation Patterns

### 4.1 Validation Adapter: `riptide-cli/src/validation_adapter.rs`

**Purpose:** Bridge between CLI client and monitoring validation

```rust
#[async_trait::async_trait]
impl HttpClient for RipTideClient {
    async fn get_json(&self, path: &str) -> Result<serde_json::Value> {
        let response = self.get(path).await?;
        Ok(response.json().await?)
    }

    async fn get_health(&self, path: &str) -> Result<()> {
        self.get(path).await?;
        Ok(())
    }
}
```

**Pattern:** Trait implementation adapter

**Recommendation:** ✅ **USE ADAPTER PATTERN** for cross-crate validation

---

### 4.2 Persistence Adapter: `riptide-api/src/persistence_adapter.rs`

**Purpose:** High-level facade for persistence operations

**Features:**
- Unified error handling with `ApiError` conversion
- Tenant-aware operations
- Graceful degradation
- Performance tracking

**Example:**
```rust
pub async fn get_cached<T: DeserializeOwned>(
    &self,
    key: &str,
    tenant_id: Option<&str>,
) -> Result<Option<T>> {
    self.cache_manager
        .get(key, tenant_id)
        .await
        .map_err(|e| anyhow::anyhow!("Cache get failed: {}", e))
}
```

**Pattern:** Facade pattern with error conversion

**Recommendation:** ✅ **USE FACADE PATTERN** for design system services

---

## 5. Deduplication Logic

### 5.1 Spider URL Deduplication: `riptide-spider/src/config.rs`

**Configuration:**
```rust
pub struct UrlProcessingConfig {
    pub enable_deduplication: bool,
    pub bloom_filter_capacity: usize,  // Must be > 0 when enabled
    pub max_exact_urls: usize,          // Must be > 0 when enabled
}
```

**Validation:**
```rust
if self.url_processing.enable_deduplication {
    if self.url_processing.bloom_filter_capacity == 0 {
        return Err("bloom_filter_capacity must be greater than 0");
    }
    if self.url_processing.max_exact_urls == 0 {
        return Err("max_exact_urls must be greater than 0");
    }
}
```

**Algorithm:** Bloom filter + exact tracking for high accuracy

---

### 5.2 Metadata Keyword Deduplication: `riptide-extraction/src/strategies/metadata.rs`

**Code (line 785-787):**
```rust
// Deduplicate keywords
metadata.keywords.sort();
metadata.keywords.dedup();
```

**Pattern:** Sort + dedup for Vec deduplication

**Recommendation:** ✅ **USE STANDARD DEDUP** for simple list deduplication

---

## 6. What Needs to Be Created

### 6.1 Missing Infrastructure

❌ **Centralized DTO Validation Framework**
- No unified request/response validation
- No JSON Schema validation middleware
- No validation error standardization

❌ **Schema-Based Validation**
- `schemars` used only for schema **generation**
- No runtime validation using generated schemas
- No validation error messages from schema violations

❌ **Validation Error DTOs**
- No standardized validation error response
- No field-level error reporting
- No error code standardization

❌ **DTO Validation Attributes**
- No `#[validate(...)]` macro usage
- No declarative validation rules
- No custom validators

---

### 6.2 Recommended New Components

**Component 1: Schema Validation Middleware**
```rust
// Create: crates/riptide-api/src/middleware/schema_validation.rs

use jsonschema::{JSONSchema, ValidationError};

pub async fn schema_validation_middleware<T: JsonSchema>(
    request: Request,
    next: Next,
) -> Response {
    // 1. Generate schema from T::json_schema()
    // 2. Parse request body to JSON Value
    // 3. Validate against schema
    // 4. Return 400 with field errors if invalid
    // 5. Proceed if valid
}
```

**Component 2: Validation Error DTO**
```rust
// Create: crates/riptide-api/src/validation_error.rs

#[derive(Serialize, JsonSchema)]
pub struct ValidationErrorResponse {
    pub error_type: String,
    pub message: String,
    pub field_errors: Vec<FieldError>,
    pub status: u16,
}

#[derive(Serialize, JsonSchema)]
pub struct FieldError {
    pub field: String,
    pub constraint: String,
    pub message: String,
}
```

**Component 3: DTO Validator Trait**
```rust
// Create: crates/riptide-api/src/dto_validator.rs

pub trait DtoValidator: JsonSchema + Sized {
    fn validate(&self) -> Result<(), ValidationErrorResponse>;

    fn validate_with_context(
        &self,
        context: &ValidationContext,
    ) -> Result<(), ValidationErrorResponse>;
}
```

---

## 7. Integration Strategy

### 7.1 Reuse Existing Components

**For URL/Content Validation:**
```rust
use riptide_config::CommonValidator;

// Reuse existing validators
let validator = CommonValidator::new_default();
validator.validate_url(url)?;
validator.validate_content_type(content_type)?;
```

**For Request Sanitization:**
```rust
use riptide_api::middleware::request_validation;

// Reuse existing middleware
app.layer(axum::middleware::from_fn(request_validation_middleware));
```

---

### 7.2 Enhance Existing Components

**Add JSON Schema Validation to Existing DTOs:**
```rust
// Before (current)
#[derive(Serialize, Deserialize)]
pub struct CrawlBody {
    pub urls: Vec<String>,
    pub options: Option<CrawlOptions>,
}

// After (enhanced)
#[derive(Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Crawl Request", description = "Request body for crawling URLs")]
pub struct CrawlBody {
    #[schemars(length(min = 1, max = 100), description = "URLs to crawl (1-100)")]
    pub urls: Vec<String>,

    #[schemars(description = "Optional crawl configuration")]
    pub options: Option<CrawlOptions>,
}
```

---

### 7.3 Create New Validation Layer

**Middleware Stack:**
```rust
// app/src/main.rs

app
    .layer(request_validation_middleware)           // Existing: HTTP-level validation
    .layer(schema_validation_middleware::<T>)       // NEW: Schema validation
    .layer(business_logic_validation_middleware)    // NEW: Business rules
```

---

## 8. Recommendations Summary

### ✅ **REUSE (Don't Recreate)**

1. **`CommonValidator`** - Use for URL, content-type, size validation
2. **Request validation middleware** - Use for HTTP-level validation
3. **DTO conversion patterns** - Follow `From`/`Into` trait pattern
4. **Adapter pattern** - Use for cross-crate integration
5. **Deduplication logic** - Use sort+dedup or bloom filter patterns

---

### 🔧 **ENHANCE (Extend Existing)**

1. **Add `JsonSchema` derives** to all DTOs
2. **Add schema descriptions** using `#[schemars(...)]`
3. **Extend validation errors** with field-level details
4. **Add validation context** for tenant-aware validation

---

### 🆕 **CREATE (New Infrastructure)**

1. **Schema validation middleware** using `jsonschema` crate
2. **Validation error DTOs** with standardized format
3. **DTO validator trait** for declarative validation
4. **Validation utility functions** for common patterns
5. **Validation test utilities** for DTO testing

---

## 9. File Locations Reference

**Validation Code:**
- `/crates/riptide-config/src/validation.rs` - Core validation framework ✅
- `/crates/riptide-api/src/validation.rs` - API-specific validation ✅
- `/crates/riptide-api/src/middleware/request_validation.rs` - HTTP middleware ✅
- `/cli-spec/src/validation.rs` - CLI spec validation ⚠️

**DTOs and Models:**
- `/crates/riptide-api/src/models.rs` - API request/response models ✅
- `/crates/riptide-api/src/dto.rs` - DTO conversion logic ✅
- `/crates/riptide-types/src/types.rs` - Core type definitions ✅

**Adapters:**
- `/crates/riptide-cli/src/validation_adapter.rs` - Validation adapter ✅
- `/crates/riptide-api/src/persistence_adapter.rs` - Persistence facade ✅

**Schema Support:**
- `/crates/riptide-extraction/src/strategies/metadata.rs` - Metadata with JsonSchema ✅
- `/crates/riptide-extraction/src/strategies/mod.rs` - Strategy types with JsonSchema ✅

**Deduplication:**
- `/crates/riptide-spider/src/config.rs` - URL deduplication config ✅
- `/crates/riptide-extraction/src/strategies/metadata.rs` - Keyword dedup ✅

---

## 10. Next Steps

### **Immediate Actions:**

1. ✅ **Audit** this analysis for completeness
2. 🔧 **Add** `jsonschema` crate to `Cargo.toml`
3. 🔧 **Create** validation middleware module
4. 🔧 **Enhance** existing DTOs with `JsonSchema` attributes
5. 🆕 **Implement** validation error response DTOs

### **Short-term Goals:**

1. Create schema validation middleware
2. Add comprehensive DTO validation
3. Standardize error responses
4. Add validation test utilities

### **Long-term Goals:**

1. Centralize all validation logic
2. Create validation documentation
3. Add validation examples
4. Implement automated schema testing

---

## Appendix A: Validation Checklist

**Before Creating New Validation Code:**

- [ ] Check if `CommonValidator` already provides the validation
- [ ] Review existing middleware for similar functionality
- [ ] Check if a DTO conversion pattern exists
- [ ] Look for similar adapter patterns
- [ ] Search for existing deduplication logic
- [ ] Review existing error handling patterns

**When Adding New Validation:**

- [ ] Add `JsonSchema` derive to DTOs
- [ ] Document validation rules in schema
- [ ] Create validation error DTOs
- [ ] Add middleware integration
- [ ] Write validation tests
- [ ] Update API documentation

---

**End of Analysis**
