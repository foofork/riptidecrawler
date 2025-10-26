# Prioritized Documentation Checklist

## Phase 1: Critical Public API (1-2 hours) 🔴

### lib.rs - Module Declarations (87 items)
**File:** `/workspaces/eventmesh/crates/riptide-api/src/lib.rs`

- [ ] `pub mod config` - Add: "Global API configuration and resource limits"
- [ ] `pub mod errors` - Add: "Error types and result helpers for API operations"
- [ ] `pub mod handlers` - Add: "HTTP request handlers for all API endpoints"
- [ ] `pub mod health` - Add: "Health check and dependency monitoring"
- [ ] `pub mod metrics` - Add: "Prometheus metrics collection and reporting"
- [ ] `pub mod middleware` - Add: "Request/response middleware (auth, rate limiting, etc.)"
- [ ] `pub mod models` - Add: "Request and response data models"
- [ ] `pub mod pipeline` - Add: "Content extraction pipeline orchestration"
- [ ] `pub mod pipeline_enhanced` - Add: "Enhanced pipeline with detailed metrics"
- [ ] `pub mod reliability_integration` - Add: "Circuit breaker and retry logic"
- [ ] `pub mod resource_manager` - Add: "Resource pooling and lifecycle management"
- [ ] `pub mod routes` - Add: "HTTP route definitions"
- [ ] `pub mod rpc_client` - Add: "RPC client for distributed operations"
- [ ] `pub mod sessions` - Add: "Browser session management"
- [ ] `pub mod state` - Add: "Shared application state"
- [ ] `pub mod strategies_pipeline` - Add: "Multi-strategy extraction pipeline"
- [ ] `pub mod streaming` - Add: "Real-time streaming responses (SSE, WebSocket, NDJSON)"
- [ ] `pub mod telemetry_config` - Add: "OpenTelemetry and tracing configuration"
- [ ] `pub mod validation` - Add: "Request validation utilities"

### Public Enums (16 items)

#### errors.rs
- [ ] Line 15: `pub enum ApiError` - ✅ Already documented

#### metrics.rs
- [ ] Line 1363: `pub enum PhaseType` - Add: "Pipeline processing phases for timing metrics"
- [ ] Line 1372: `pub enum ErrorType` - Add: "Error categories for metrics classification"

#### state.rs
- [ ] Line 1152: `pub enum DependencyHealth` - Add: "Health status of external dependencies"

#### sessions/types.rs
- [ ] Line 180: `pub enum BrowserType` - Add: "Supported browser types for sessions"

#### handlers/browser.rs
- [ ] Line 67: `pub enum BrowserAction` - Add: "Browser control actions (navigate, click, screenshot, etc.)"

#### handlers/workers.rs
- [ ] Line 37: `pub enum JobTypeRequest` - Add: "Background job types for async processing"

#### handlers/pdf.rs
- [ ] Line 488: `pub enum PdfProcessingRequest` - Add: "PDF processing operation types"

#### streaming/lifecycle.rs
- [ ] Line 25: `pub enum LifecycleEvent` - Add: "Stream lifecycle events for monitoring"

#### streaming/pipeline.rs
- [ ] Line 496: `pub enum StreamEvent` - Add: "Streaming pipeline events"

#### streaming/response_helpers.rs
- [ ] Line 173: `pub enum StreamingResponseType` - Add: "Streaming response protocol types"

#### streaming/error.rs
- [ ] Line 19: `pub enum StreamingError` - Add: "Errors that can occur during streaming operations"
- [ ] Line 192: `pub enum RecoveryStrategy` - Add: "Strategies for recovering from streaming errors"

#### streaming/mod.rs
- [ ] Line 103: `pub enum StreamingProtocol` - Add: "Supported streaming protocols"
- [ ] Line 175: `pub enum StreamingHealth` - Add: "Health status of streaming subsystem"

#### resource_manager/errors.rs
- [ ] Line 11: `pub enum ResourceManagerError` - Add: "Resource management errors"

### Type Aliases (4 items)
- [ ] errors.rs:346 - `pub type ApiResult<T>` - Add: "Result type using ApiError"
- [ ] streaming/metrics.rs:238 - `pub type SseMetrics` - Add: "Server-Sent Events metrics"
- [ ] streaming/metrics.rs:242 - `pub type WebSocketMetrics` - Add: "WebSocket connection metrics"
- [ ] streaming/error.rs:162 - `pub type StreamingResult<T>` - Add: "Result type for streaming operations"

---

## Phase 2: Core Data Structures (2-3 hours) 🟡

### Pipeline Components

#### strategies_pipeline.rs
- [ ] Line 20: `pub struct StrategiesPipelineResult` - ✅ Already documented
- [ ] Line 56: `pub struct StrategiesPipelineOrchestrator` - ✅ Already documented
- [ ] Line 77: `pub fn with_auto_strategy` - Add example and parameter docs

#### pipeline_enhanced.rs
- [ ] Line 33: `pub struct EnhancedPipelineOrchestrator` - ✅ Already documented
- [ ] Line 515: `pub struct EnhancedPipelineResult` - Add: "Result from enhanced pipeline with detailed metrics"
- [ ] Line 529: `pub struct PhaseTiming` - Add: "Timing information for a single pipeline phase"
- [ ] Line 538: `pub struct EnhancedBatchStats` - Add: "Aggregate statistics for batch operations"

#### pipeline.rs
- [ ] Line 39: `pub struct PipelineResult` - Add: "Standard pipeline execution result"
- [ ] Line 88: `pub struct GateDecisionStats` - Add: "Statistics about gate routing decisions"

#### pipeline_dual.rs
- [ ] Line 29: `pub struct FastPathResult` - Add: "Result from fast-path processing"
- [ ] Line 39: `pub struct EnhancementResult` - Add: "Result from enhancement processing"
- [ ] Line 50: `pub struct DualPathResult` - Add: "Combined result from dual-path pipeline"
- [ ] Line 63: `pub struct DualPathConfig` - Add: "Configuration for dual-path processing"
- [ ] Line 86: `pub struct DualPathOrchestrator` - Add: "Orchestrator for dual-path pipeline"
- [ ] Line 386: `pub struct DualPathStats` - Add: "Performance statistics for dual-path"

### Session Management

#### sessions/types.rs
- [ ] Line 12: `pub struct SessionConfig` - ✅ Already documented
- [ ] Line 48: `pub struct Session` - ✅ Already documented
- [ ] Line 76: `pub struct CookieJar` - ✅ Already documented
- [ ] Line 83: `pub struct Cookie` - ✅ Already documented
- [ ] Line 111: `pub enum SameSite` - ✅ Already documented
- [ ] Line 119: `pub struct SessionMetadata` - ✅ Already documented
- [ ] Line 142: `pub struct SessionBrowserConfig` - ✅ Already documented
- [ ] Line 188: `pub struct Viewport` - ✅ Already documented

### Streaming Components

#### streaming/mod.rs
- [ ] All major structs need documentation review

#### streaming/error.rs
- [ ] Error conversion implementations need docs

### Resource Management

#### resource_manager/*.rs
- [ ] Review all public structs in resource_manager module

---

## Phase 3: Error Helpers & Utility Functions (3-4 hours) 🟢

### Error Construction Helpers (errors.rs)
- [ ] Line 95: `pub fn validation<S>` - ✅ Already documented
- [ ] Line 102: `pub fn invalid_request<S>` - Add: "Create validation error for invalid requests"
- [ ] Line 109: `pub fn invalid_url<S>` - Add: "Create error for malformed URLs"
- [ ] Line 132: `pub fn extraction<S>` - Add: "Create error for extraction failures"
- [ ] Line 139: `pub fn pipeline<S>` - Add: "Create error for pipeline failures"
- [ ] Line 146: `pub fn dependency<S>` - Add: "Create error for dependency failures"
- [ ] Line 177: `pub fn internal<S>` - Add: "Create generic internal server error"
- [ ] Line 184: `pub fn not_found<S>` - Add: "Create 404 not found error"
- [ ] Line 191: `pub fn status_code(&self)` - Add: "Get HTTP status code for this error"
- [ ] Line 212: `pub fn error_type(&self)` - Add: "Get error type string for logging"
- [ ] Line 233: `pub fn is_retryable(&self)` - Add: "Check if this error is retryable"

### Telemetry Functions (telemetry_config.rs)
- [ ] Line 107: `pub fn from_env()` - Add: "Load telemetry config from environment variables"
- [ ] Line 184: `pub fn init_tracing()` - Add: "Initialize OpenTelemetry tracing"
- [ ] Line 322: `pub fn shutdown()` - Add: "Gracefully shutdown telemetry system"
- [ ] Line 361: `pub fn inject_trace_context()` - Add: "Inject trace context into HTTP headers"
- [ ] Line 388: `pub fn parse_trace_id()` - Add: "Parse trace ID from string"
- [ ] Line 406: `pub fn parse_span_id()` - Add: "Parse span ID from string"

### Metrics Recording (metrics.rs) - 228 functions
Focus on most commonly used:
- [ ] Line 863: `pub fn record_http_request()` - Add docs
- [ ] Line 873: `pub fn record_phase_timing()` - Add docs
- [ ] Review top 20 most-called metrics functions

### RPC Client (rpc_client.rs)
- [ ] Line 10: `pub struct RpcClient` - Add: "Client for distributed RPC operations"
- [ ] Line 24: `pub fn new()` - Add: "Create new RPC client"

---

## Phase 4: Handler Functions (2-3 hours) 🔵

### Handler Modules
Review and document public functions in:
- [ ] handlers/extract.rs
- [ ] handlers/browser.rs
- [ ] handlers/health.rs
- [ ] handlers/workers.rs
- [ ] handlers/pdf.rs
- [ ] handlers/search.rs
- [ ] handlers/telemetry.rs

---

## Documentation Templates

### For Enums:
```rust
/// Brief description of enum purpose
///
/// Extended explanation if needed
pub enum MyEnum {
    /// Variant 1 description
    Variant1,
    /// Variant 2 description with details about when used
    Variant2,
}
```

### For Functions:
```rust
/// Brief one-line description
///
/// # Arguments
/// * `param1` - Parameter description
///
/// # Returns
/// Return value description
///
/// # Errors
/// When this function returns an error
///
/// # Example
/// ```no_run
/// let result = my_function(arg)?;
/// ```
pub fn my_function(param1: Type) -> Result<ReturnType> { }
```

### For Structs:
```rust
/// Brief struct description
///
/// Extended description with usage context
pub struct MyStruct {
    /// Field 1 description
    pub field1: Type1,
    /// Field 2 description
    pub field2: Type2,
}
```

---

## Progress Tracking

- **Phase 1:** 0/107 items (0%)
- **Phase 2:** 0/193 items (0%)
- **Phase 3:** 0/228 items (0%)
- **Phase 4:** 0/estimated items (0%)

**Overall:** 0/529+ items documented (0%)

---

## Quick Wins (30 minutes)

Do these first for immediate impact:

1. ✅ Add all module-level docs in lib.rs (19 items)
2. ✅ Document all public enums (16 items)
3. ✅ Document all type aliases (4 items)
4. ✅ Document DependencyHealth enum
5. ✅ Document BrowserType enum

**Impact:** ~40 items = Immediate 7-8% coverage improvement

---

## Validation

After documentation is added:

```bash
# Enable missing docs warning
cargo clippy -- -W missing_docs

# Check documentation builds
cargo doc --no-deps

# Review documentation
cargo doc --open
```
