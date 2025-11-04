# Error Handling Architecture Analysis - RipTide

**Generated:** 2025-11-04
**Objective:** Comprehensive analysis of error types, conversion patterns, and HTTP mappings to inform error model unification

---

## Executive Summary

The RipTide codebase contains **11 distinct error types** across **9 crates**, with varying levels of sophistication in error handling. The API layer (`ApiError`) provides comprehensive HTTP status code mapping, while other crates use simpler error models. There's significant opportunity for unification and standardization.

### Key Findings

- ✅ **Strong HTTP mapping** in `riptide-api` with detailed status codes
- ✅ **Consistent use of thiserror** across most error types
- ⚠️ **No centralized error codes** or categorization system
- ⚠️ **Inconsistent error context** preservation patterns
- ⚠️ **Limited error chaining** from domain errors to API errors
- ❌ **No strategy-specific errors** - generic error wrapping via `anyhow`

---

## 1. Complete Error Inventory

### 1.1 API Layer (`riptide-api`)

#### **ApiError** - `/crates/riptide-api/src/errors.rs`

**Purpose:** Comprehensive HTTP API error handling with status code mapping

**Variants (19 total):**
```rust
pub enum ApiError {
    // Client errors (4xx)
    ValidationError { message: String },                    // 400
    InvalidUrl { url: String, message: String },            // 400
    InvalidParameter { parameter: String, message: String }, // 400
    MissingRequiredHeader { header: String },               // 400
    InvalidHeaderValue { header: String, message: String }, // 400
    InvalidContentType { content_type: String, message: String }, // 415
    NotFound { resource: String },                          // 404
    RateLimited { message: String },                        // 429
    AuthenticationError { message: String },                // 401
    PayloadTooLarge { message: String },                    // 413

    // Server errors (5xx)
    FetchError { url: String, message: String },            // 502
    CacheError { message: String },                         // 503
    ExtractionError { message: String },                    // 500
    RoutingError { message: String },                       // 500
    PipelineError { message: String },                      // 500
    ConfigError { message: String },                        // 500
    InternalError { message: String },                      // 500
    DependencyError { service: String, message: String },   // 503
    TimeoutError { operation: String, message: String },    // 408
}
```

**Features:**
- ✅ Detailed HTTP status code mapping via `status_code()` method
- ✅ Error type categorization via `error_type()` method
- ✅ Retryability detection via `is_retryable()` method
- ✅ Automatic JSON response formatting via `IntoResponse`
- ✅ Context-aware logging (error/warn/info based on severity)

**Result Alias:**
```rust
pub type ApiResult<T> = Result<T, ApiError>;
```

**Error Conversions:**
```rust
From<anyhow::Error>      -> ApiError::InternalError
From<reqwest::Error>     -> ApiError::{TimeoutError, FetchError}
From<redis::RedisError>  -> ApiError::CacheError
From<url::ParseError>    -> ApiError::InvalidUrl
From<serde_json::Error>  -> ApiError::ValidationError
```

#### **ResourceManagerError** - `/crates/riptide-api/src/resource_manager/errors.rs`

**Purpose:** Resource management and pool operations

**Variants (9 total):**
```rust
pub enum ResourceManagerError {
    BrowserPool(String),
    RateLimit { retry_after: Duration },
    MemoryPressure,
    Wasm(String),
    Timeout { operation: String, duration: Duration },
    ResourceExhausted { resource_type: String },
    Configuration(String),
    InvalidUrl(String),
    Internal(#[from] anyhow::Error),
}
```

**Features:**
- ✅ Duration-based timeout tracking
- ✅ Structured rate limiting with retry hints
- ⚠️ No HTTP status code mapping
- ⚠️ Limited error context

**Result Alias:**
```rust
pub type Result<T> = std::result::Result<T, ResourceManagerError>;
```

**Error Conversions:**
```rust
From<url::ParseError> -> ResourceManagerError::InvalidUrl
From<anyhow::Error>   -> ResourceManagerError::Internal
```

#### **StreamingError** - `/crates/riptide-api/src/streaming/error.rs`

**Purpose:** Real-time streaming operations with backpressure handling

**Variants (8 total):**
```rust
pub enum StreamingError {
    BufferOverflow { message: String },
    Connection { message: String },
    Serialization { #[from] source: serde_json::Error },
    Channel { message: String },
    BackpressureExceeded { connection_id: String },
    ClientDisconnected { reason: String },
    Pipeline { #[from] source: anyhow::Error },
    InvalidRequest { message: String },
    Timeout { seconds: u64 },
}
```

**Features:**
- ✅ Recovery strategy recommendations via `recovery_strategy()` method
- ✅ Client error detection via `is_client_error()` method
- ✅ Retryability detection via `is_retryable()` method
- ✅ Conversion to `ApiError` for HTTP responses
- ✅ Context tracking via `ConnectionContext` struct

**Recovery Strategies:**
```rust
pub enum RecoveryStrategy {
    Retry { attempts: u32, delay_ms: u64 },
    Drop,        // Drop message and continue
    Disconnect,  // Disconnect client
    Fail,        // Fail entire operation
}
```

**Result Alias:**
```rust
pub type StreamingResult<T> = Result<T, StreamingError>;
```

**Error Conversions:**
```rust
From<serde_json::Error> -> StreamingError::Serialization
From<anyhow::Error>     -> StreamingError::Pipeline
From<StreamingError>    -> ApiError (custom mapping)
```

---

### 1.2 Core Types (`riptide-types`)

#### **RiptideError** - `/crates/riptide-types/src/errors.rs`

**Purpose:** Unified error type for core Riptide operations

**Variants (15 total):**
```rust
pub enum RiptideError {
    // Browser operations
    BrowserInitialization(String),
    BrowserOperation(String),
    Navigation(String),

    // Content operations
    Extraction(String),
    Parse(String),

    // Configuration and network
    Configuration(String),
    Network(String),
    Timeout(u64),

    // Storage operations
    Cache(String),
    Storage(String),

    // Resource management
    NotFound(String),
    AlreadyExists(String),
    PermissionDenied(String),

    // Generic and conversions
    Custom(String),
    InvalidUrl(#[from] url::ParseError),
    Json(#[from] serde_json::Error),
    Io(#[from] std::io::Error),
    Other(#[from] anyhow::Error),
}
```

**Features:**
- ✅ HTTP-style error categorization via `is_client_error()` and `is_server_error()`
- ✅ Retryability detection via `is_retryable()` method
- ✅ Comprehensive auto-conversions from common error types
- ⚠️ No HTTP status code mapping
- ⚠️ No error codes or structured categorization

**Result Alias:**
```rust
pub type Result<T> = std::result::Result<T, RiptideError>;
```

**Error Conversions:**
```rust
From<url::ParseError>    -> RiptideError::InvalidUrl
From<serde_json::Error>  -> RiptideError::Json
From<std::io::Error>     -> RiptideError::Io
From<anyhow::Error>      -> RiptideError::Other
```

---

### 1.3 Persistence Layer (`riptide-persistence`)

#### **PersistenceError** - `/crates/riptide-persistence/src/errors.rs`

**Purpose:** Database, cache, and state management errors

**Variants (18 total):**
```rust
pub enum PersistenceError {
    // Infrastructure
    Redis(#[from] redis::RedisError),
    Serialization(#[from] serde_json::Error),
    Compression(String),
    FileSystem(#[from] std::io::Error),
    Watch(#[from] notify::Error),

    // Domain operations
    Configuration(String),
    Cache(String),
    State(String),
    Tenant(String),
    Sync(String),

    // Quality and security
    Performance(String),
    Security(String),
    DataIntegrity(String),

    // Resource management
    QuotaExceeded { resource: String, limit: u64, current: u64 },
    Timeout { timeout_ms: u64 },
    InvalidTenantAccess { tenant_id: String },

    // Observability
    Metrics(String),
    Generic(#[from] anyhow::Error),
}
```

**Features:**
- ✅ Error categorization via `category()` method (returns string tags)
- ✅ Retryability detection via `is_retryable()` method
- ✅ Structured quota violation with usage tracking
- ✅ Comprehensive builder methods for each variant
- ⚠️ String-based categorization (not enum-based)

**Result Alias:**
```rust
pub type PersistenceResult<T> = Result<T, PersistenceError>;
```

**Error Conversions:**
```rust
From<redis::RedisError>  -> PersistenceError::Redis
From<serde_json::Error>  -> PersistenceError::Serialization
From<std::io::Error>     -> PersistenceError::FileSystem
From<notify::Error>      -> PersistenceError::Watch
From<anyhow::Error>      -> PersistenceError::Generic
```

**Category Tags:**
```
"redis", "serialization", "compression", "configuration",
"cache", "state", "tenant", "sync", "performance",
"security", "quota", "timeout", "tenant_access",
"data_integrity", "filesystem", "watch", "metrics", "generic"
```

---

### 1.4 Domain-Specific Errors

#### **PdfError** - `/crates/riptide-pdf/src/errors.rs`

**Purpose:** PDF processing errors with validation

**Variants (9 total):**
```rust
pub enum PdfError {
    InvalidPdf { message: String },
    EncryptedPdf,
    FileTooLarge { size: u64, max_size: u64 },
    CorruptedPdf { message: String },
    Timeout { timeout_seconds: u64 },
    MemoryLimit { used: u64, limit: u64 },
    UnsupportedVersion { version: String },
    ProcessingError { message: String },
    IoError { message: String },
}
```

**Features:**
- ✅ Structured validation with size/limit tracking
- ✅ Clear security boundaries (encrypted, corrupted)
- ✅ Resource limit tracking (memory, timeout)
- ⚠️ Manual Display implementation (not using thiserror derive)
- ⚠️ No retryability detection

**Result Alias:**
```rust
pub type PdfResult<T> = Result<T, PdfError>;
```

**Error Conversions:**
```rust
From<std::io::Error> -> PdfError::IoError
From<anyhow::Error>  -> PdfError::ProcessingError
```

#### **NativeParserError** - `/crates/riptide-extraction/src/native_parser/error.rs`

**Purpose:** HTML parsing and extraction errors

**Variants (9 total):**
```rust
pub enum NativeParserError {
    ParseError(String),
    OversizedHtml { size: usize, max: usize },
    EncodingError(String),
    Timeout { timeout_ms: u64 },
    InvalidUrl(String),
    InvalidStructure(String),
    NoContentFound,
    LowQuality { score: f32, threshold: f32 },
    Internal(String),
}
```

**Features:**
- ✅ Quality-based validation with threshold tracking
- ✅ Size limit enforcement
- ✅ Encoding validation
- ⚠️ No retryability detection
- ⚠️ Generic `Internal` variant (should use anyhow)

**Result Alias:**
```rust
pub type Result<T> = std::result::Result<T, NativeParserError>;
```

#### **AbstractionError** - `/crates/riptide-browser-abstraction/src/error.rs`

**Purpose:** Browser abstraction layer errors

**Variants (9 total):**
```rust
pub enum AbstractionError {
    PageCreation(String),
    Navigation(String),
    ContentRetrieval(String),
    Evaluation(String),
    Screenshot(String),
    PdfGeneration(String),
    PageClose(String),
    BrowserClose(String),
    Unsupported(String),
    Other(String),
}
```

**Features:**
- ⚠️ Operation-based variants (good granularity)
- ⚠️ All variants are String-based (no structured context)
- ⚠️ No retryability detection
- ⚠️ No error codes or categorization

**Result Alias:**
```rust
pub type AbstractionResult<T> = Result<T, AbstractionError>;
```

#### **RiptideError (facade)** - `/crates/riptide-facade/src/error.rs`

**Purpose:** Facade layer errors for simplified API

**Variants (6 total):**
```rust
pub enum RiptideError {
    Config(String),
    Fetch(String),
    Extraction(String),
    InvalidUrl(#[from] url::ParseError),
    Timeout,
    Other(#[from] anyhow::Error),
}
```

**Features:**
- ✅ Simple, focused error set
- ✅ Builder methods for common errors
- ⚠️ Very generic (delegates to anyhow via `Other`)
- ⚠️ No retryability or categorization

**Result Alias:**
```rust
pub type RiptideResult<T> = Result<T, RiptideError>;
```

**Error Conversions:**
```rust
From<url::ParseError> -> RiptideError::InvalidUrl
From<anyhow::Error>   -> RiptideError::Other
```

#### **IntelligenceError** - `/crates/riptide-intelligence/src/lib.rs`

**Purpose:** LLM provider and circuit breaker errors

**Variants (8 total):**
```rust
pub enum IntelligenceError {
    Provider(String),
    Timeout { timeout_ms: u64 },
    CircuitOpen { reason: String },
    AllProvidersFailed,
    Configuration(String),
    RateLimit { retry_after_ms: u64 },
    InvalidRequest(String),
    Network(String),
}
```

**Features:**
- ✅ Circuit breaker awareness
- ✅ Provider fallback handling
- ✅ Structured timeout and rate limit tracking
- ⚠️ No retryability detection (circuit breaker handling is implicit)
- ⚠️ No conversion to ApiError

**Result Alias:**
```rust
pub type Result<T> = std::result::Result<T, IntelligenceError>;
```

#### **PerformanceError** - `/crates/riptide-performance/src/lib.rs`

**Purpose:** Performance monitoring and profiling errors

**Variants (5 total):**
```rust
pub enum PerformanceError {
    Io(#[from] std::io::Error),
    ProfilingError(String),
    MonitoringError(String),
    ConfigError(String),
    ResourceLimitExceeded(String),
    AnyhowError(#[from] anyhow::Error),
}
```

**Features:**
- ⚠️ Very basic error set
- ⚠️ No structured context
- ⚠️ Generic `AnyhowError` catchall

**Result Alias:**
```rust
pub type Result<T> = std::result::Result<T, PerformanceError>;
```

**Error Conversions:**
```rust
From<std::io::Error>  -> PerformanceError::Io
From<anyhow::Error>   -> PerformanceError::AnyhowError
```

#### **MonitoringError** - `/crates/riptide-monitoring/src/monitoring/error.rs`

**Purpose:** Monitoring system errors

**Variants (4 total):**
```rust
pub enum MonitoringError {
    LockPoisoned(String),
    InvalidMetric(String),
    ConfigError(String),
    IoError(std::io::Error),
}
```

**Features:**
- ✅ Lock poisoning handling with recovery via `LockManager`
- ⚠️ Manual Display implementation (not using thiserror)
- ⚠️ Very limited error set

**Result Alias:**
```rust
pub type Result<T> = std::result::Result<T, MonitoringError>;
```

**Error Conversions:**
```rust
From<std::io::Error> -> MonitoringError::IoError
```

#### **StreamingError (lib)** - `/crates/riptide-streaming/src/lib.rs`

**Purpose:** Top-level streaming errors (different from api streaming error)

**Variants (7 total):**
```rust
pub enum StreamingError {
    StreamNotFound(Uuid),
    StreamCompleted(Uuid),
    BackpressureExceeded,
    ReportGenerationFailed(String),
    ConfigError(String),
    IoError(#[from] std::io::Error),
    SerializationError(#[from] serde_json::Error),
}
```

**Features:**
- ✅ UUID-based stream tracking
- ⚠️ Overlaps with `riptide-api/src/streaming/error.rs` StreamingError
- ⚠️ No recovery strategies (unlike API version)

**Result Alias:**
```rust
pub type StreamingResult<T> = Result<T, StreamingError>;
```

**Error Conversions:**
```rust
From<std::io::Error>                     -> StreamingError::IoError
From<serde_json::Error>                  -> StreamingError::SerializationError
From<tokio_util::codec::LinesCodecError> -> StreamingError::{ConfigError, IoError}
```

---

### 1.5 CLI Layer (`riptide-cli`)

#### **ExitCode** - `/crates/riptide-cli/src/error.rs`

**Purpose:** POSIX-compliant exit codes for CLI

**Variants (2 total):**
```rust
pub enum ExitCode {
    Success = 0,
    UserError = 1,  // 4xx status codes, network issues, config errors
}
```

**Features:**
- ✅ Simple, focused on CLI UX
- ✅ Uses `anyhow::Result` for internal error handling
- ⚠️ All errors collapse to single exit code (no differentiation)

**Note:** CLI delegates complex error handling to API server, uses `anyhow::Result<T>` internally.

---

## 2. Error Conversion Patterns

### 2.1 Conversion Graph

```
┌─────────────────────────────────────────────────────────────────┐
│                        ERROR FLOW DIAGRAM                        │
└─────────────────────────────────────────────────────────────────┘

External Errors (stdlib/deps)
├── url::ParseError
│   ├─> ApiError::InvalidUrl
│   ├─> RiptideError::InvalidUrl
│   ├─> ResourceManagerError::InvalidUrl
│   └─> RiptideError (facade)::InvalidUrl
│
├── std::io::Error
│   ├─> RiptideError::Io
│   ├─> PersistenceError::FileSystem
│   ├─> PdfError::IoError
│   ├─> PerformanceError::Io
│   ├─> MonitoringError::IoError
│   └─> StreamingError::IoError
│
├── serde_json::Error
│   ├─> ApiError::ValidationError
│   ├─> RiptideError::Json
│   ├─> PersistenceError::Serialization
│   └─> StreamingError::SerializationError
│
├── redis::RedisError
│   ├─> ApiError::CacheError
│   └─> PersistenceError::Redis
│
├── reqwest::Error
│   ├─> ApiError::TimeoutError (if timeout)
│   ├─> ApiError::FetchError (if connect)
│   └─> ApiError::FetchError (else)
│
├── notify::Error
│   └─> PersistenceError::Watch
│
└── anyhow::Error
    ├─> ApiError::InternalError
    ├─> RiptideError::Other
    ├─> ResourceManagerError::Internal
    ├─> PdfError::ProcessingError
    ├─> RiptideError (facade)::Other
    ├─> PerformanceError::AnyhowError
    ├─> StreamingError::Pipeline
    └─> PersistenceError::Generic

Domain to API Conversions
├── StreamingError ─> ApiError (custom mapping)
│   ├── InvalidRequest        ─> ValidationError
│   ├── Timeout               ─> TimeoutError
│   ├── BackpressureExceeded  ─> RateLimited
│   ├── ClientDisconnected    ─> InternalError
│   └── *                     ─> InternalError
│
└── [MISSING] Other domain errors don't convert to ApiError
    ├── RiptideError          ─> ❌ No conversion
    ├── PersistenceError      ─> ❌ No conversion
    ├── ResourceManagerError  ─> ❌ No conversion
    ├── IntelligenceError     ─> ❌ No conversion
    └── NativeParserError     ─> ❌ No conversion
```

### 2.2 Conversion Patterns Summary

**Pattern 1: Auto-conversion via `#[from]`**
```rust
// Most common pattern
#[error("IO error: {0}")]
Io(#[from] std::io::Error),
```
Used by: All error types except MonitoringError, PdfError

**Pattern 2: Manual `From` implementation**
```rust
impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ApiError::TimeoutError { /* ... */ }
        } else if err.is_connect() {
            ApiError::FetchError { /* ... */ }
        } else {
            ApiError::FetchError { /* ... */ }
        }
    }
}
```
Used by: ApiError (for reqwest, redis), StreamingError (for ApiError)

**Pattern 3: Context-preserving conversion**
```rust
impl From<StreamingError> for ApiError {
    fn from(err: StreamingError) -> Self {
        match err {
            StreamingError::InvalidRequest { message } =>
                ApiError::validation(message),
            StreamingError::Timeout { seconds } =>
                ApiError::timeout("streaming_operation", format!("...")),
            // ... preserves semantic meaning
        }
    }
}
```
Used by: StreamingError → ApiError (only example of semantic preservation)

**Pattern 4: Generic wrapping via anyhow**
```rust
#[error(transparent)]
Other(#[from] anyhow::Error),
```
Used by: RiptideError, ResourceManagerError, PersistenceError, PdfError, RiptideError (facade), PerformanceError

---

## 3. HTTP Response Mapping

### 3.1 ApiError Status Code Mapping

```rust
pub fn status_code(&self) -> StatusCode {
    match self {
        // 4xx Client Errors
        ApiError::ValidationError { .. }         => StatusCode::BAD_REQUEST,          // 400
        ApiError::InvalidUrl { .. }              => StatusCode::BAD_REQUEST,          // 400
        ApiError::InvalidParameter { .. }        => StatusCode::BAD_REQUEST,          // 400
        ApiError::MissingRequiredHeader { .. }   => StatusCode::BAD_REQUEST,          // 400
        ApiError::InvalidHeaderValue { .. }      => StatusCode::BAD_REQUEST,          // 400
        ApiError::AuthenticationError { .. }     => StatusCode::UNAUTHORIZED,         // 401
        ApiError::NotFound { .. }                => StatusCode::NOT_FOUND,            // 404
        ApiError::TimeoutError { .. }            => StatusCode::REQUEST_TIMEOUT,      // 408
        ApiError::PayloadTooLarge { .. }         => StatusCode::PAYLOAD_TOO_LARGE,    // 413
        ApiError::InvalidContentType { .. }      => StatusCode::UNSUPPORTED_MEDIA_TYPE, // 415
        ApiError::RateLimited { .. }             => StatusCode::TOO_MANY_REQUESTS,    // 429

        // 5xx Server Errors
        ApiError::InternalError { .. }           => StatusCode::INTERNAL_SERVER_ERROR, // 500
        ApiError::ExtractionError { .. }         => StatusCode::INTERNAL_SERVER_ERROR, // 500
        ApiError::RoutingError { .. }            => StatusCode::INTERNAL_SERVER_ERROR, // 500
        ApiError::PipelineError { .. }           => StatusCode::INTERNAL_SERVER_ERROR, // 500
        ApiError::ConfigError { .. }             => StatusCode::INTERNAL_SERVER_ERROR, // 500
        ApiError::FetchError { .. }              => StatusCode::BAD_GATEWAY,          // 502
        ApiError::CacheError { .. }              => StatusCode::SERVICE_UNAVAILABLE,  // 503
        ApiError::DependencyError { .. }         => StatusCode::SERVICE_UNAVAILABLE,  // 503
    }
}
```

### 3.2 JSON Response Format

**Response Structure:**
```json
{
  "error": {
    "type": "extraction_error",
    "message": "Content extraction failed: timeout after 30s",
    "retryable": true,
    "status": 500
  }
}
```

**Error Type Strings:**
```
"validation_error", "invalid_url", "rate_limited", "authentication_error",
"fetch_error", "cache_error", "extraction_error", "routing_error",
"pipeline_error", "config_error", "dependency_error", "internal_error",
"timeout_error", "not_found", "payload_too_large", "invalid_content_type",
"missing_required_header", "invalid_header_value", "invalid_parameter"
```

### 3.3 Logging Strategy

```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();

        match status {
            StatusCode::INTERNAL_SERVER_ERROR
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::BAD_GATEWAY => {
                tracing::error!(
                    error_type = self.error_type(),
                    message = %self.to_string(),
                    "API error occurred"
                );
            }
            StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND => {
                tracing::warn!(
                    error_type = self.error_type(),
                    message = %self.to_string(),
                    "Client error occurred"
                );
            }
            _ => {
                tracing::info!(
                    error_type = self.error_type(),
                    message = %self.to_string(),
                    "API error occurred"
                );
            }
        }

        // Return JSON response...
    }
}
```

**Logging Levels:**
- **ERROR:** 5xx errors (server/infrastructure failures)
- **WARN:** 4xx errors (client mistakes)
- **INFO:** Other errors (rate limits, etc.)

---

## 4. Error Context Preservation

### 4.1 Context Preservation Patterns

**Pattern 1: Structured Context (Best)**
```rust
// StreamingError - Rich context with connection tracking
pub struct ConnectionContext {
    pub session_id: String,
    pub client_type: ClientType,
    pub connected_at: std::time::Instant,
}

#[error("Backpressure threshold exceeded for connection {connection_id}")]
BackpressureExceeded { connection_id: String },
```

**Pattern 2: Nested Fields (Good)**
```rust
// ApiError - URL and message context
#[error("Failed to fetch content from {url}: {message}")]
FetchError { url: String, message: String },

// ResourceManagerError - Operation and duration context
#[error("Operation '{operation}' timed out after {duration:?}")]
Timeout { operation: String, duration: Duration },

// PersistenceError - Resource quota context
#[error("Quota exceeded: {resource} limit {limit} exceeded with usage {current}")]
QuotaExceeded { resource: String, limit: u64, current: u64 },
```

**Pattern 3: Single Message (Weak)**
```rust
// Most errors use this pattern
#[error("Browser pool error: {0}")]
BrowserPool(String),

#[error("Extraction error: {0}")]
Extraction(String),
```
Context is lost when converting from source error to string.

**Pattern 4: Transparent Wrapping (Delegates Context)**
```rust
// Preserves full error chain via anyhow
#[error(transparent)]
Other(#[from] anyhow::Error),
```

### 4.2 Context Loss Analysis

**Where Context is PRESERVED:**
- ✅ StreamingError: Connection ID, client type, timing
- ✅ PersistenceError: Resource limits, tenant ID, quotas
- ✅ ApiError: URL, service name, operation name
- ✅ ResourceManagerError: Operation name, duration
- ✅ PdfError: File sizes, memory limits, version
- ✅ NativeParserError: Quality scores, thresholds

**Where Context is LOST:**
- ❌ All String-based error variants
- ❌ AbstractionError: All variants are string-only
- ❌ Generic anyhow wrapping: Source type info lost
- ❌ Error chaining: Most errors don't preserve source errors

### 4.3 Error Chaining

**Current State:**
- Only **anyhow::Error** variants preserve full error chain
- Most conversions flatten to strings
- No structured error cause tracking

**Example of context loss:**
```rust
// ❌ BAD: Context lost
reqwest::Error → ApiError::FetchError { url, message: err.to_string() }
// Lost: reqwest error kind, status code, is_timeout flag

// ✅ GOOD: Context preserved
reqwest::Error → ApiError via custom From impl
// Preserved: timeout detection, connection failure, URL
```

---

## 5. Error Code Standards

### 5.1 Current State

**Finding:** ❌ **No centralized error code system exists**

**What exists:**
- HTTP status codes in ApiError
- Error type strings in ApiError (`error_type()`)
- Category strings in PersistenceError (`category()`)
- No numeric error codes
- No error code registry
- No error code documentation

### 5.2 Error Type Strings (ApiError)

```rust
pub fn error_type(&self) -> &'static str {
    match self {
        ApiError::ValidationError { .. } => "validation_error",
        ApiError::InvalidUrl { .. } => "invalid_url",
        // ... 19 total
    }
}
```

These are **not unique** across crates - only meaningful within ApiError.

### 5.3 Category Strings (PersistenceError)

```rust
pub fn category(&self) -> &'static str {
    match self {
        PersistenceError::Redis(_) => "redis",
        PersistenceError::Cache(_) => "cache",
        // ... 18 total
    }
}
```

These are for **metrics tagging** only, not for client consumption.

### 5.4 Gap Analysis

**Missing:**
- ❌ Numeric error codes (e.g., E1001, E2003)
- ❌ Error code registry/catalog
- ❌ Cross-crate error code uniqueness
- ❌ Error code documentation
- ❌ Error code versioning strategy
- ❌ Machine-readable error codes in responses

**Impact:**
- Clients must parse error messages
- No programmatic error handling by code
- Difficult to track error patterns across versions
- No error code search/documentation

---

## 6. Strategy Error Bubbling

### 6.1 Current State

**Finding:** ❌ **No strategy-specific error types exist**

All extraction strategies return `anyhow::Result<T>`:

```rust
// From strategy implementations
pub async fn extract(&self, request: ExtractionRequest) -> anyhow::Result<ExtractionResult>
```

### 6.2 Error Flow from Strategies

```
┌─────────────────────────────────────────────────────────────────┐
│                    STRATEGY ERROR FLOW                           │
└─────────────────────────────────────────────────────────────────┘

Strategy Implementation
├── CSS Strategy
│   └─> anyhow::Error (generic)
│       └─> lost: selector info, fallback attempts
│
├── WASM Strategy
│   └─> anyhow::Error (generic)
│       └─> lost: WASM module errors, memory issues
│
├── LLM Strategy
│   └─> anyhow::Error (generic)
│       └─> lost: provider failures, token limits, circuit breaker
│
└── Multi Strategy (composition)
    └─> anyhow::Error (generic)
        └─> lost: which strategies failed, fallback chain

Handler Layer
├── anyhow::Error ─> ApiError::InternalError
│   └─> HTTP 500
│   └─> Generic "Internal server error" message
│
└── [Alternative] anyhow::Error ─> ApiError::ExtractionError
    └─> HTTP 500
    └─> Generic "Content extraction failed" message

Result: ❌ All strategy errors become generic 500s
```

### 6.3 Lost Context Examples

**Example 1: CSS Strategy Failure**
```rust
// What we have now:
ApiError::ExtractionError {
    message: "Content extraction failed: CSS selector '.article' not found"
}

// What we could have:
ApiError::ExtractionError {
    strategy: "css",
    selector: ".article",
    fallback_attempted: true,
    fallback_strategy: "wasm",
    retry_with_suggestion: Some("Try .post-content selector"),
}
```

**Example 2: LLM Strategy Circuit Breaker**
```rust
// What we have now:
ApiError::ExtractionError {
    message: "Content extraction failed: LLM provider unavailable"
}

// What we could have:
ApiError::DependencyError {
    service: "anthropic",
    reason: "circuit_breaker_open",
    retry_after_ms: Some(60000),
    fallback_available: true,
    fallback_strategy: "wasm",
}
```

**Example 3: Multi-Strategy Failure**
```rust
// What we have now:
ApiError::ExtractionError {
    message: "Content extraction failed: all strategies exhausted"
}

// What we could have:
ApiError::ExtractionError {
    strategy: "multi",
    attempts: [
        StrategyAttempt {
            name: "css",
            error: "selector_not_found",
            duration_ms: 45,
        },
        StrategyAttempt {
            name: "wasm",
            error: "quality_too_low",
            quality_score: 0.52,
            threshold: 0.7,
            duration_ms: 120,
        },
        StrategyAttempt {
            name: "llm",
            error: "rate_limited",
            retry_after_ms: 30000,
            duration_ms: 5,
        },
    ],
}
```

### 6.4 Handler Error Conversion Count

**Finding:** 92 error conversions in handler files

```bash
$ grep -rn "\.map_err\|From<.*> for.*Error\|into()" crates/riptide-api/src/handlers/*.rs | wc -l
92
```

**Common patterns:**
```rust
// Pattern 1: Generic internal error (most common)
.map_err(|e| ApiError::internal(format!("Operation failed: {}", e)))?

// Pattern 2: Specific error type
.map_err(|e| ApiError::extraction(format!("Extraction failed: {}", e)))?

// Pattern 3: Context-aware
.map_err(|e| ApiError::fetch(&url, format!("Failed: {}", e)))?
```

**Impact:**
- 92 opportunities for context loss
- Manual error mapping in each handler
- Inconsistent error messages
- No standardized error reporting

---

## 7. Gaps in Current Error Handling

### 7.1 Critical Gaps

#### **Gap 1: No Strategy-Specific Errors**
**Severity:** 🔴 Critical
**Impact:** All extraction failures are generic 500s

**Recommendation:**
```rust
pub enum StrategyError {
    // CSS-specific
    SelectorNotFound { selector: String, alternatives: Vec<String> },
    InvalidSelector { selector: String, reason: String },

    // WASM-specific
    WasmExecutionFailed { module: String, error: String },
    WasmMemoryExhausted { used_mb: u64, limit_mb: u64 },

    // LLM-specific
    ProviderUnavailable { provider: String, retry_after_ms: u64 },
    ProviderRateLimited { provider: String, reset_at: DateTime<Utc> },
    CircuitBreakerOpen { provider: String, failures: u32 },
    TokenLimitExceeded { tokens_used: u64, limit: u64 },

    // Quality-based
    QualityTooLow { strategy: String, score: f64, threshold: f64 },

    // Composition-specific
    AllStrategiesFailed { attempts: Vec<StrategyAttempt> },
    FallbackFailed { primary: String, fallback: String },
}
```

#### **Gap 2: No Error Code System**
**Severity:** 🟡 High
**Impact:** Clients can't programmatically handle errors

**Recommendation:**
```rust
pub struct ErrorCode {
    pub code: u32,           // E.g., 1001
    pub category: &'static str,  // E.g., "EXTRACTION"
    pub name: &'static str,      // E.g., "CSS_SELECTOR_NOT_FOUND"
}

impl ApiError {
    pub fn error_code(&self) -> ErrorCode {
        match self {
            ApiError::ExtractionError { .. } if is_css_error(..) =>
                ErrorCode { code: 2001, category: "EXTRACTION", name: "CSS_SELECTOR_NOT_FOUND" },
            // ...
        }
    }
}
```

#### **Gap 3: Inconsistent Error Context**
**Severity:** 🟡 High
**Impact:** Lost debugging information, poor UX

**Examples:**
- AbstractionError: All string-based, no structured fields
- Most error variants: Single message string
- Generic anyhow wrapping: Full context loss

**Recommendation:** Standardize on structured error fields with dedicated types.

#### **Gap 4: No Domain → API Error Conversions**
**Severity:** 🟡 High
**Impact:** Manual mapping in every handler, inconsistency

**Missing conversions:**
```rust
From<RiptideError>          for ApiError  // ❌
From<PersistenceError>      for ApiError  // ❌
From<ResourceManagerError>  for ApiError  // ❌
From<IntelligenceError>     for ApiError  // ❌
From<NativeParserError>     for ApiError  // ❌
From<PdfError>              for ApiError  // ❌
From<AbstractionError>      for ApiError  // ❌
```

**Only exists:**
```rust
From<StreamingError>        for ApiError  // ✅ (semantic conversion)
```

#### **Gap 5: No Retryability Metadata in HTTP Responses**
**Severity:** 🟠 Medium
**Impact:** Clients don't know when to retry

**Current:**
```json
{
  "error": {
    "type": "timeout_error",
    "message": "Operation timed out",
    "retryable": true,
    "status": 408
  }
}
```

**Missing:**
```json
{
  "error": {
    "type": "timeout_error",
    "message": "Operation timed out",
    "retryable": true,
    "retry_after_ms": 5000,
    "max_retries": 3,
    "backoff_strategy": "exponential",
    "status": 408
  }
}
```

#### **Gap 6: No Error Metrics/Observability**
**Severity:** 🟠 Medium
**Impact:** Hard to track error patterns and trends

**Missing:**
- Error count metrics by type
- Error rate tracking
- Error duration histograms
- Error correlation IDs
- Error sampling for detailed traces

### 7.2 Minor Gaps

- No error internationalization (i18n) support
- No error field validation metadata
- No error recovery suggestions in responses
- No error documentation generation from code
- No error versioning strategy

---

## 8. Recommendations for Unification

### 8.1 Unification Strategy

**Option A: Single Unified Error (Not Recommended)**
```rust
// ❌ Too broad, loses domain semantics
pub enum UnifiedError {
    // 100+ variants...
}
```
**Issues:** Giant enum, unclear ownership, difficult to maintain

**Option B: Layered Error Model (Recommended)**
```rust
// ✅ Clear boundaries, composable
Domain Errors (internal)
├── ExtractionError (strategies, parsers)
├── PersistenceError (storage, cache)
├── IntelligenceError (LLM, circuit breaker)
├── ResourceError (pools, limits)
└── PerformanceError (monitoring, profiling)

API Errors (public)
└── ApiError (HTTP responses)
    ├── From<ExtractionError>
    ├── From<PersistenceError>
    ├── From<IntelligenceError>
    ├── From<ResourceError>
    └── From<PerformanceError>
```

### 8.2 Specific Recommendations

#### **Recommendation 1: Create Strategy Error Type**
**Priority:** 🔴 Critical

```rust
// crates/riptide-extraction/src/strategy_error.rs
pub enum StrategyError {
    // CSS
    CssSelectorNotFound {
        selector: String,
        alternatives: Vec<String>,
        dom_stats: DomStats,
    },

    // WASM
    WasmExecutionFailed {
        module_name: String,
        error_message: String,
        stack_trace: Option<String>,
    },

    // LLM
    LlmProviderFailed {
        provider: String,
        error_kind: LlmErrorKind,
        retry_after: Option<Duration>,
    },

    // Composition
    AllStrategiesFailed {
        attempts: Vec<StrategyAttempt>,
        total_duration: Duration,
    },

    // Quality
    QualityBelowThreshold {
        strategy: String,
        score: f64,
        threshold: f64,
        metrics: QualityMetrics,
    },
}

impl From<StrategyError> for ApiError {
    fn from(err: StrategyError) -> Self {
        match err {
            StrategyError::LlmProviderFailed {
                provider,
                error_kind: LlmErrorKind::RateLimited { retry_after },
                ..
            } => ApiError::RateLimited {
                message: format!("LLM provider {} rate limited", provider),
            },

            StrategyError::AllStrategiesFailed { attempts, .. } => {
                ApiError::ExtractionError {
                    message: format!(
                        "All {} strategies failed: {}",
                        attempts.len(),
                        attempts.iter()
                            .map(|a| format!("{}: {}", a.name, a.error))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                }
            },

            _ => ApiError::ExtractionError {
                message: err.to_string(),
            },
        }
    }
}
```

#### **Recommendation 2: Implement Error Codes**
**Priority:** 🟡 High

```rust
// crates/riptide-types/src/error_codes.rs
pub struct ErrorCode {
    pub code: u32,
    pub category: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub retryable: bool,
    pub documentation_url: Option<&'static str>,
}

pub mod codes {
    use super::ErrorCode;

    // 1xxx: Validation errors
    pub const INVALID_URL: ErrorCode = ErrorCode {
        code: 1001,
        category: "VALIDATION",
        name: "INVALID_URL",
        description: "The provided URL is malformed or invalid",
        retryable: false,
        documentation_url: Some("https://docs.riptide.io/errors/1001"),
    };

    // 2xxx: Extraction errors
    pub const CSS_SELECTOR_NOT_FOUND: ErrorCode = ErrorCode {
        code: 2001,
        category: "EXTRACTION",
        name: "CSS_SELECTOR_NOT_FOUND",
        description: "The CSS selector did not match any elements",
        retryable: true,
        documentation_url: Some("https://docs.riptide.io/errors/2001"),
    };

    // 3xxx: LLM/Intelligence errors
    pub const LLM_RATE_LIMITED: ErrorCode = ErrorCode {
        code: 3001,
        category: "INTELLIGENCE",
        name: "LLM_RATE_LIMITED",
        description: "LLM provider rate limit exceeded",
        retryable: true,
        documentation_url: Some("https://docs.riptide.io/errors/3001"),
    };

    // 4xxx: Resource errors
    pub const BROWSER_POOL_EXHAUSTED: ErrorCode = ErrorCode {
        code: 4001,
        category: "RESOURCE",
        name: "BROWSER_POOL_EXHAUSTED",
        description: "No available browser instances in pool",
        retryable: true,
        documentation_url: Some("https://docs.riptide.io/errors/4001"),
    };

    // 5xxx: Persistence errors
    pub const CACHE_UNAVAILABLE: ErrorCode = ErrorCode {
        code: 5001,
        category: "PERSISTENCE",
        name: "CACHE_UNAVAILABLE",
        description: "Cache service is unavailable",
        retryable: true,
        documentation_url: Some("https://docs.riptide.io/errors/5001"),
    };
}

// Add to ApiError
impl ApiError {
    pub fn error_code(&self) -> ErrorCode {
        match self {
            ApiError::InvalidUrl { .. } => codes::INVALID_URL,
            ApiError::CacheError { .. } => codes::CACHE_UNAVAILABLE,
            // ...
        }
    }
}

// Updated JSON response
{
  "error": {
    "code": 2001,
    "type": "extraction_error",
    "category": "EXTRACTION",
    "name": "CSS_SELECTOR_NOT_FOUND",
    "message": "CSS selector '.article' not found",
    "retryable": true,
    "documentation_url": "https://docs.riptide.io/errors/2001",
    "status": 500
  }
}
```

#### **Recommendation 3: Add Domain to API Conversions**
**Priority:** 🟡 High

```rust
// Implement for all domain errors
impl From<RiptideError> for ApiError {
    fn from(err: RiptideError) -> Self {
        match err {
            RiptideError::BrowserInitialization(msg) =>
                ApiError::DependencyError { service: "browser".into(), message: msg },
            RiptideError::Network(msg) =>
                ApiError::FetchError { url: String::new(), message: msg },
            RiptideError::Timeout(ms) =>
                ApiError::TimeoutError { operation: "browser".into(), message: format!("{}ms", ms) },
            RiptideError::Configuration(msg) =>
                ApiError::ConfigError { message: msg },
            RiptideError::NotFound(resource) =>
                ApiError::NotFound { resource },
            _ => ApiError::InternalError { message: err.to_string() },
        }
    }
}

impl From<PersistenceError> for ApiError {
    fn from(err: PersistenceError) -> Self {
        match err {
            PersistenceError::Redis(_) =>
                ApiError::DependencyError { service: "redis".into(), message: err.to_string() },
            PersistenceError::QuotaExceeded { resource, limit, current } =>
                ApiError::RateLimited {
                    message: format!("{} quota exceeded: {}/{}", resource, current, limit)
                },
            PersistenceError::Timeout { timeout_ms } =>
                ApiError::TimeoutError {
                    operation: "persistence".into(),
                    message: format!("{}ms", timeout_ms)
                },
            _ => ApiError::InternalError { message: err.to_string() },
        }
    }
}

// Similar for IntelligenceError, ResourceManagerError, etc.
```

#### **Recommendation 4: Structured Error Context**
**Priority:** 🟠 Medium

Replace string-based errors with structured types:

```rust
// Before
#[error("Browser pool error: {0}")]
BrowserPool(String),

// After
#[error("Browser pool error: {reason}")]
BrowserPool {
    reason: BrowserPoolErrorKind,
    pool_size: usize,
    active_instances: usize,
    waiting_requests: usize,
},

pub enum BrowserPoolErrorKind {
    PoolExhausted,
    InitializationFailed,
    HealthCheckFailed,
    Timeout,
}
```

#### **Recommendation 5: Error Metrics and Observability**
**Priority:** 🟠 Medium

```rust
// Add to ApiError::into_response
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_code = self.error_code();

        // Record metrics
        metrics::increment_counter!(
            "api_errors_total",
            "error_code" => error_code.code.to_string(),
            "error_category" => error_code.category,
            "http_status" => status.as_u16().to_string(),
        );

        // Generate correlation ID
        let correlation_id = Uuid::new_v4();

        // Log with correlation ID
        tracing::error!(
            correlation_id = %correlation_id,
            error_code = error_code.code,
            error_type = self.error_type(),
            "API error occurred"
        );

        // Include in response
        let body = Json(json!({
            "error": {
                "correlation_id": correlation_id,
                "code": error_code.code,
                "type": self.error_type(),
                "message": self.to_string(),
                "retryable": self.is_retryable(),
                "status": status.as_u16()
            }
        }));

        (status, body).into_response()
    }
}
```

#### **Recommendation 6: Retry Metadata**
**Priority:** 🟠 Medium

```rust
pub trait RetryableError {
    fn retry_after(&self) -> Option<Duration>;
    fn max_retries(&self) -> u32;
    fn backoff_strategy(&self) -> BackoffStrategy;
}

pub enum BackoffStrategy {
    Fixed(Duration),
    Linear { base: Duration, increment: Duration },
    Exponential { base: Duration, multiplier: f64, max: Duration },
}

impl RetryableError for ApiError {
    fn retry_after(&self) -> Option<Duration> {
        match self {
            ApiError::RateLimited { .. } => Some(Duration::from_secs(60)),
            ApiError::DependencyError { .. } => Some(Duration::from_secs(5)),
            ApiError::TimeoutError { .. } => Some(Duration::from_secs(30)),
            _ => None,
        }
    }

    fn max_retries(&self) -> u32 {
        match self {
            ApiError::RateLimited { .. } => 0,  // Don't retry rate limits
            ApiError::TimeoutError { .. } => 3,
            ApiError::DependencyError { .. } => 5,
            _ => 0,
        }
    }

    fn backoff_strategy(&self) -> BackoffStrategy {
        match self {
            ApiError::TimeoutError { .. } =>
                BackoffStrategy::Exponential {
                    base: Duration::from_secs(1),
                    multiplier: 2.0,
                    max: Duration::from_secs(60)
                },
            ApiError::DependencyError { .. } =>
                BackoffStrategy::Linear {
                    base: Duration::from_secs(1),
                    increment: Duration::from_secs(1)
                },
            _ => BackoffStrategy::Fixed(Duration::from_secs(5)),
        }
    }
}
```

### 8.3 Implementation Roadmap

**Phase 1: Critical (Sprint 1-2)**
1. Create `StrategyError` type in riptide-extraction
2. Implement strategy error propagation through handlers
3. Add `From<StrategyError>` for `ApiError`

**Phase 2: High Priority (Sprint 3-4)**
1. Design and implement error code system
2. Add error codes to all existing errors
3. Update API responses to include error codes
4. Create error documentation generator

**Phase 3: Medium Priority (Sprint 5-6)**
1. Implement domain → API error conversions
2. Add structured error context to all error types
3. Implement error metrics and correlation IDs
4. Add retry metadata to retryable errors

**Phase 4: Long-term**
1. Error internationalization (i18n)
2. Error recovery suggestions
3. Error versioning strategy
4. Automated error documentation

---

## 9. Appendix

### 9.1 Error Type Summary Table

| Crate | Error Type | Variants | Result Alias | HTTP Mapping | Retryable | Error Codes |
|-------|-----------|----------|--------------|--------------|-----------|-------------|
| riptide-api | ApiError | 19 | ApiResult | ✅ Yes | ✅ Yes | ❌ No |
| riptide-api | ResourceManagerError | 9 | Result | ❌ No | ❌ No | ❌ No |
| riptide-api | StreamingError | 8 | StreamingResult | ✅ Via ApiError | ✅ Yes | ❌ No |
| riptide-types | RiptideError | 15 | Result | ❌ No | ✅ Yes | ❌ No |
| riptide-persistence | PersistenceError | 18 | PersistenceResult | ❌ No | ✅ Yes | ⚠️ Categories |
| riptide-pdf | PdfError | 9 | PdfResult | ❌ No | ❌ No | ❌ No |
| riptide-extraction | NativeParserError | 9 | Result | ❌ No | ❌ No | ❌ No |
| riptide-browser-abstraction | AbstractionError | 9 | AbstractionResult | ❌ No | ❌ No | ❌ No |
| riptide-facade | RiptideError | 6 | RiptideResult | ❌ No | ❌ No | ❌ No |
| riptide-intelligence | IntelligenceError | 8 | Result | ❌ No | ❌ No | ❌ No |
| riptide-performance | PerformanceError | 5 | Result | ❌ No | ❌ No | ❌ No |
| riptide-monitoring | MonitoringError | 4 | Result | ❌ No | ❌ No | ❌ No |
| riptide-streaming | StreamingError | 7 | StreamingResult | ❌ No | ❌ No | ❌ No |
| riptide-cli | ExitCode | 2 | - | N/A | N/A | ❌ No |

**Total:** 14 error types, 128 total error variants

### 9.2 Conversion Matrix

| From Error | To Error | Method | Context Preserved |
|------------|----------|--------|-------------------|
| url::ParseError | ApiError | From trait | ⚠️ Partial |
| url::ParseError | RiptideError | From trait | ⚠️ Partial |
| url::ParseError | ResourceManagerError | From trait | ⚠️ Partial |
| std::io::Error | RiptideError | From trait | ❌ No |
| std::io::Error | PersistenceError | From trait | ❌ No |
| std::io::Error | PdfError | From trait | ❌ No |
| serde_json::Error | ApiError | From trait | ❌ No |
| serde_json::Error | RiptideError | From trait | ❌ No |
| redis::RedisError | ApiError | From trait | ❌ No |
| redis::RedisError | PersistenceError | From trait | ❌ No |
| reqwest::Error | ApiError | Custom From | ✅ Yes |
| anyhow::Error | ApiError | From trait | ❌ No |
| anyhow::Error | RiptideError | From trait | ❌ No |
| anyhow::Error | Multiple | From trait | ❌ No |
| StreamingError | ApiError | Custom From | ✅ Yes |
| Strategy errors | ApiError | ❌ None | ❌ N/A |
| Domain errors | ApiError | ❌ None | ❌ N/A |

### 9.3 Files Analyzed

```
Core Error Definitions:
/crates/riptide-types/src/errors.rs                        (RiptideError)
/crates/riptide-api/src/errors.rs                          (ApiError)
/crates/riptide-api/src/resource_manager/errors.rs         (ResourceManagerError)
/crates/riptide-api/src/streaming/error.rs                 (StreamingError)
/crates/riptide-persistence/src/errors.rs                  (PersistenceError)
/crates/riptide-pdf/src/errors.rs                          (PdfError)
/crates/riptide-cli/src/error.rs                           (ExitCode)
/crates/riptide-facade/src/error.rs                        (RiptideError facade)
/crates/riptide-browser-abstraction/src/error.rs           (AbstractionError)
/crates/riptide-extraction/src/native_parser/error.rs      (NativeParserError)
/crates/riptide-monitoring/src/monitoring/error.rs         (MonitoringError)
/crates/riptide-intelligence/src/lib.rs                    (IntelligenceError)
/crates/riptide-performance/src/lib.rs                     (PerformanceError)
/crates/riptide-streaming/src/lib.rs                       (StreamingError lib)

Handler Usage (Error Conversion Patterns):
/crates/riptide-api/src/handlers/**/*.rs                   (40 files, 92 conversions)

Total Files Analyzed: 54
```

---

## Conclusion

The RipTide error handling architecture is **functional but fragmented**. The API layer has excellent HTTP mapping, but domain errors are poorly integrated. The **critical gap** is the lack of strategy-specific errors, which causes all extraction failures to become generic 500s.

**Top 3 Priorities for Unification:**
1. 🔴 **Create StrategyError type** - Enable meaningful extraction error reporting
2. 🟡 **Implement error code system** - Enable programmatic error handling
3. 🟡 **Add domain → API conversions** - Reduce manual mapping, improve consistency

Implementing these changes will significantly improve error observability, client UX, and developer experience.
