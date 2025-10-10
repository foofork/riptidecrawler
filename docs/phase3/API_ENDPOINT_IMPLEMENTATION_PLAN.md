# API Endpoint Implementation Plan

**Date:** 2025-10-10
**Status:** 📋 Implementation Plan for Missing API Endpoints
**Target:** v1.1 (Q2 2025)

---

## Executive Summary

This document outlines the implementation plan for missing API endpoints identified in the future-features-and-todos analysis. The plan addresses:

1. **Missing Endpoints:** `/api/v1/extract` and `/api/v1/search`
2. **Endpoint Standardization:** Consistent path naming across the API
3. **Test Infrastructure:** Update `create_app()` test helper
4. **Documentation:** Update API docs and OpenAPI spec

**Total Effort:** 24-32 hours (3-4 working days)
**Tests Enabled:** 6 ignored tests

---

## Current State Analysis

### Existing Endpoints (59 total)

#### Core Operations
- `GET /healthz` - Health check
- `GET /api/health/detailed` - Detailed health status
- `GET /metrics` - Prometheus metrics
- `POST /crawl` - Single URL crawl
- `POST /render` - Render with browser
- `POST /deepsearch` - Deep search crawl

#### Streaming
- `POST /crawl/stream` - NDJSON stream
- `POST /crawl/sse` - Server-sent events
- `GET /crawl/ws` - WebSocket stream
- `POST /deepsearch/stream` - NDJSON deepsearch

#### Spider (Deep Crawling)
- `POST /spider/crawl` - Start spider crawl
- `POST /spider/status` - Get spider status
- `POST /spider/control` - Control spider

#### Sessions (20 endpoints)
- `POST /sessions` - Create session
- `GET /sessions` - List sessions
- `DELETE /sessions/:id` - Delete session
- Cookie management endpoints
- Session extension endpoints

#### Workers (13 endpoints)
- `POST /workers/jobs` - Submit job
- `GET /workers/jobs` - List jobs
- Scheduled jobs endpoints
- Queue/worker statistics

#### Monitoring (14 endpoints)
- Health score, performance reports
- Alert management
- Memory profiling
- WASM instance health

#### Resources (6 endpoints)
- Browser pool, rate limiter status
- Memory, performance metrics

#### Nested Routes
- `/pdf/*` - PDF processing (5 endpoints)
- `/stealth/*` - Stealth config (3 endpoints)
- `/api/v1/tables/*` - Table extraction (4 endpoints)
- `/api/v1/llm/*` - LLM providers (5 endpoints)

### Missing Endpoints (P0 - Critical)

#### 1. `/api/v1/extract` - Content Extraction
**Status:** ❌ NOT IMPLEMENTED
**Test:** `crates/riptide-api/tests/api_tests.rs:59`
**Priority:** P0
**Effort:** 8-12 hours

**Purpose:** Extract content from a single URL using various strategies

**Payload:**
```json
{
  "url": "https://example.com",
  "mode": "standard",
  "options": {
    "strategy": "multi",
    "quality_threshold": 0.7,
    "timeout_ms": 30000
  }
}
```

**Response:**
```json
{
  "url": "https://example.com",
  "title": "Page Title",
  "content": "Extracted content...",
  "metadata": {
    "author": "...",
    "publish_date": "...",
    "word_count": 1500
  },
  "strategy_used": "css",
  "quality_score": 0.85,
  "extraction_time_ms": 450
}
```

#### 2. `/api/v1/search` - Search Integration
**Status:** ❌ NOT IMPLEMENTED
**Test:** `crates/riptide-api/tests/api_tests.rs:82`
**Priority:** P0
**Effort:** 8-12 hours

**Purpose:** Search using configured providers and extract results

**Query Parameters:**
- `q` - Search query (required)
- `limit` - Number of results (default: 10, max: 50)
- `country` - Country code (default: "us")
- `language` - Language code (default: "en")
- `provider` - Force specific provider (optional)

**Response:**
```json
{
  "query": "rust web scraping",
  "results": [
    {
      "title": "...",
      "url": "...",
      "snippet": "...",
      "position": 1
    }
  ],
  "total_results": 10,
  "provider_used": "none",
  "search_time_ms": 250
}
```

### Endpoint Path Inconsistencies

#### Issue: Mixed Path Patterns
Current paths use inconsistent prefixes:
- `/healthz` vs `/api/health/detailed`
- `/metrics` (root) vs `/api/v1/metrics` (expected)
- `/crawl` (root) vs `/api/v1/crawl` (expected)

#### Standards Decision
**Recommendation:** Support BOTH patterns with v1 as standard

1. **Root paths** (no `/api` prefix): Backward compatibility
2. **Versioned paths** (`/api/v1/*`): New standard

**Implementation:** Route aliases

---

## Implementation Plan

### Phase 1: Create Extract Endpoint (8-12 hours)

#### Step 1.1: Create Handler (3-4 hours)

**File:** `crates/riptide-api/src/handlers/extract.rs`

```rust
//! Content extraction handler for single-URL extraction
//!
//! Provides a unified endpoint for extracting content from URLs using
//! the multi-strategy extraction pipeline.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::state::AppState;

/// Extract endpoint request payload
#[derive(Debug, Deserialize)]
pub struct ExtractRequest {
    /// URL to extract content from
    pub url: String,
    /// Extraction mode (standard, article, product, etc.)
    #[serde(default = "default_mode")]
    pub mode: String,
    /// Extraction options
    #[serde(default)]
    pub options: ExtractOptions,
}

fn default_mode() -> String {
    "standard".to_string()
}

/// Extraction options
#[derive(Debug, Default, Deserialize)]
pub struct ExtractOptions {
    /// Strategy to use (auto, css, wasm, llm, multi)
    #[serde(default = "default_strategy")]
    pub strategy: String,
    /// Minimum quality threshold (0.0-1.0)
    #[serde(default = "default_quality_threshold")]
    pub quality_threshold: f64,
    /// Timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_strategy() -> String {
    "multi".to_string()
}

fn default_quality_threshold() -> f64 {
    0.7
}

fn default_timeout() -> u64 {
    30000
}

/// Extract response
#[derive(Debug, Serialize)]
pub struct ExtractResponse {
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub metadata: ContentMetadata,
    pub strategy_used: String,
    pub quality_score: f64,
    pub extraction_time_ms: u64,
}

/// Content metadata
#[derive(Debug, Default, Serialize)]
pub struct ContentMetadata {
    pub author: Option<String>,
    pub publish_date: Option<String>,
    pub word_count: usize,
    pub language: Option<String>,
}

/// Extract content from a URL using multi-strategy extraction
#[tracing::instrument(skip(state), fields(url = %payload.url))]
pub async fn extract(
    State(state): State<AppState>,
    Json(payload): Json<ExtractRequest>,
) -> Response {
    let start = Instant::now();

    // Validate URL
    if let Err(e) = url::Url::parse(&payload.url) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid URL",
                "message": e.to_string()
            })),
        )
            .into_response();
    }

    // Use existing crawl handler logic or strategies pipeline
    // For now, return a placeholder response
    // TODO: Integrate with strategies_pipeline or create dedicated extraction logic

    let response = ExtractResponse {
        url: payload.url.clone(),
        title: Some("Extracted Title".to_string()),
        content: "Extracted content...".to_string(),
        metadata: ContentMetadata {
            word_count: 150,
            ..Default::default()
        },
        strategy_used: payload.options.strategy,
        quality_score: 0.85,
        extraction_time_ms: start.elapsed().as_millis() as u64,
    };

    (StatusCode::OK, Json(response)).into_response()
}
```

#### Step 1.2: Register Handler (1 hour)

**File:** `crates/riptide-api/src/handlers/mod.rs`

```rust
// Add to module declarations
pub mod extract;

// Add to re-exports
pub use extract::extract;
```

#### Step 1.3: Add Routes (1 hour)

**File:** `crates/riptide-api/src/main.rs`

```rust
// Add after /crawl route
.route("/api/v1/extract", post(handlers::extract))
.route("/extract", post(handlers::extract)) // Alias for backward compatibility
```

#### Step 1.4: Write Tests (3-4 hours)

**File:** `crates/riptide-api/tests/extract_tests.rs`

```rust
//! Integration tests for extract endpoint

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

mod test_helpers;
use test_helpers::create_test_app;

#[tokio::test]
async fn test_extract_valid_url() {
    let app = create_test_app().await;

    let payload = json!({
        "url": "https://example.com",
        "mode": "standard"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/extract")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_extract_invalid_url() {
    let app = create_test_app().await;

    let payload = json!({
        "url": "not-a-valid-url",
        "mode": "standard"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/extract")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// Additional tests for different strategies, timeout handling, etc.
```

---

### Phase 2: Create Search Endpoint (8-12 hours)

#### Step 2.1: Create Handler (3-4 hours)

**File:** `crates/riptide-api/src/handlers/search.rs`

```rust
//! Search integration handler
//!
//! Provides search functionality using configured search providers.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::state::AppState;

/// Search query parameters
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    /// Search query string
    pub q: String,
    /// Number of results
    #[serde(default = "default_limit")]
    pub limit: u32,
    /// Country code
    #[serde(default = "default_country")]
    pub country: String,
    /// Language code
    #[serde(default = "default_language")]
    pub language: String,
    /// Force specific provider
    pub provider: Option<String>,
}

fn default_limit() -> u32 {
    10
}

fn default_country() -> String {
    "us".to_string()
}

fn default_language() -> String {
    "en".to_string()
}

/// Search result
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub position: u32,
}

/// Search response
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub total_results: usize,
    pub provider_used: String,
    pub search_time_ms: u64,
}

/// Search using configured providers
#[tracing::instrument(skip(state), fields(query = %params.q))]
pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Response {
    let start = Instant::now();

    // Validate limit
    let limit = params.limit.min(50);

    // TODO: Use riptide-search providers
    // For now, return placeholder response

    let response = SearchResponse {
        query: params.q.clone(),
        results: vec![
            SearchResult {
                title: "Example Result".to_string(),
                url: "https://example.com".to_string(),
                snippet: "This is an example search result".to_string(),
                position: 1,
            },
        ],
        total_results: 1,
        provider_used: "none".to_string(),
        search_time_ms: start.elapsed().as_millis() as u64,
    };

    (StatusCode::OK, Json(response)).into_response()
}
```

#### Step 2.2: Register Handler (1 hour)

**File:** `crates/riptide-api/src/handlers/mod.rs`

```rust
// Add to module declarations
pub mod search;

// Add to re-exports
pub use search::search;
```

#### Step 2.3: Add Routes (1 hour)

**File:** `crates/riptide-api/src/main.rs`

```rust
// Add after extract routes
.route("/api/v1/search", get(handlers::search))
.route("/search", get(handlers::search)) // Alias for backward compatibility
```

#### Step 2.4: Write Tests (3-4 hours)

**File:** `crates/riptide-api/tests/search_tests.rs`

```rust
//! Integration tests for search endpoint

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

mod test_helpers;
use test_helpers::create_test_app;

#[tokio::test]
async fn test_search_with_query() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/search?q=rust%20web%20scraping&limit=10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_search_missing_query() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/search")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// Additional tests for limits, providers, etc.
```

---

### Phase 3: Endpoint Standardization (4-6 hours)

#### Step 3.1: Add Route Aliases (2 hours)

**File:** `crates/riptide-api/src/main.rs`

Add aliased routes for consistency:

```rust
// Health endpoints - add v1 alias
.route("/healthz", get(handlers::health))
.route("/api/v1/health", get(handlers::health)) // Alias

// Metrics - add v1 alias
.route("/metrics", get(handlers::metrics))
.route("/api/v1/metrics", get(handlers::metrics)) // Alias

// Crawl - add v1 alias
.route("/crawl", post(handlers::crawl))
.route("/api/v1/crawl", post(handlers::crawl)) // Alias
```

#### Step 3.2: Update Tests (2-3 hours)

Update `crates/riptide-api/tests/api_tests.rs` to test both paths:

```rust
#[tokio::test]
async fn test_health_endpoint_root() {
    let app = create_test_app().await;
    let response = app.oneshot(Request::builder().uri("/healthz")...).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_health_endpoint_v1() {
    let app = create_test_app().await;
    let response = app.oneshot(Request::builder().uri("/api/v1/health")...).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
```

---

### Phase 4: Test Infrastructure (8-12 hours)

#### Step 4.1: Create Test Helpers Module (4-6 hours)

**File:** `crates/riptide-api/tests/test_helpers.rs`

```rust
//! Test helpers for API endpoint tests

use axum::Router;
use riptide_api::{
    config::ApiConfig,
    health::HealthChecker,
    metrics::RipTideMetrics,
    state::AppState,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

/// Create a test application with full dependencies
pub async fn create_test_app() -> Router {
    // Initialize test config
    let config = ApiConfig {
        redis_url: "redis://localhost:6379".to_string(),
        wasm_path: "./test-fixtures/wasm-component.wasm".to_string(),
        max_concurrency: 10,
        cache_ttl: 3600,
        ..Default::default()
    };

    // Initialize test metrics
    let metrics = Arc::new(RipTideMetrics::new().unwrap());

    // Initialize test health checker
    let health_checker = Arc::new(HealthChecker::new());

    // Create test app state
    let app_state = AppState::new(config, metrics, health_checker)
        .await
        .expect("Failed to create test app state");

    // Create router with all routes
    create_test_router(app_state)
}

fn create_test_router(state: AppState) -> Router {
    use axum::routing::{get, post};
    use riptide_api::handlers;

    Router::new()
        .route("/healthz", get(handlers::health))
        .route("/api/v1/health", get(handlers::health))
        .route("/metrics", get(handlers::metrics))
        .route("/api/v1/metrics", get(handlers::metrics))
        .route("/crawl", post(handlers::crawl))
        .route("/api/v1/crawl", post(handlers::crawl))
        .route("/api/v1/extract", post(handlers::extract))
        .route("/extract", post(handlers::extract))
        .route("/api/v1/search", get(handlers::search))
        .route("/search", get(handlers::search))
        .with_state(state)
        .layer(CorsLayer::permissive())
}

/// Create a minimal test app without dependencies
pub fn create_minimal_test_app() -> Router {
    use axum::routing::get;

    Router::new()
        .route("/healthz", get(|| async { "OK" }))
}
```

#### Step 4.2: Update Existing Tests (2-3 hours)

Update `crates/riptide-api/tests/api_tests.rs`:

```rust
mod test_helpers;
use test_helpers::create_test_app;

// Remove old create_app() stub
// Replace all test functions to use create_test_app().await

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_app().await;
    // ... rest of test
}
```

#### Step 4.3: Enable Ignored Tests (2-3 hours)

Remove `#[ignore]` attributes from tests:

```rust
// Before:
#[tokio::test]
#[ignore = "TODO: Requires real API server"]
async fn test_crawl_endpoint() { ... }

// After:
#[tokio::test]
async fn test_crawl_endpoint() {
    let app = create_test_app().await;
    // ... test implementation
}
```

---

## Implementation Checklist

### Phase 1: Extract Endpoint ✅
- [ ] Create `handlers/extract.rs` with handler
- [ ] Add module declaration to `handlers/mod.rs`
- [ ] Add routes to `main.rs` (`/api/v1/extract` and `/extract`)
- [ ] Create `tests/extract_tests.rs` with comprehensive tests
- [ ] Update `api_tests.rs` to enable extract test
- [ ] Verify with `cargo test --test extract_tests`

### Phase 2: Search Endpoint ✅
- [ ] Create `handlers/search.rs` with handler
- [ ] Add module declaration to `handlers/mod.rs`
- [ ] Add routes to `main.rs` (`/api/v1/search` and `/search`)
- [ ] Create `tests/search_tests.rs` with comprehensive tests
- [ ] Update `api_tests.rs` to enable search test
- [ ] Verify with `cargo test --test search_tests`

### Phase 3: Endpoint Standardization ✅
- [ ] Add `/api/v1/health` alias for `/healthz`
- [ ] Add `/api/v1/metrics` alias for `/metrics`
- [ ] Add `/api/v1/crawl` alias for `/crawl`
- [ ] Update tests to verify both paths work
- [ ] Document endpoint aliases in API docs

### Phase 4: Test Infrastructure ✅
- [ ] Create `tests/test_helpers.rs` module
- [ ] Implement `create_test_app()` with full AppState
- [ ] Implement `create_test_router()` with all routes
- [ ] Update all tests in `api_tests.rs` to use helper
- [ ] Remove all `#[ignore]` attributes from enabled tests
- [ ] Verify all tests pass with `cargo test --test api_tests`

### Phase 5: Documentation ✅
- [ ] Update API documentation with new endpoints
- [ ] Update OpenAPI spec with extract/search endpoints
- [ ] Add usage examples to README
- [ ] Update CHANGELOG with new endpoints

---

## Testing Strategy

### Unit Tests
- Handler logic validation
- Request/response parsing
- Error handling

### Integration Tests
- Full request/response cycle
- State management
- Middleware interaction

### Test Coverage Goals
- Extract endpoint: 90%+ coverage
- Search endpoint: 90%+ coverage
- Test helpers: 100% coverage
- Overall API tests: 85%+ pass rate

---

## Success Criteria

### Functional Requirements
✅ `/api/v1/extract` endpoint implemented and tested
✅ `/api/v1/search` endpoint implemented and tested
✅ Endpoint path aliases for backward compatibility
✅ Test infrastructure with `create_test_app()` helper
✅ All ignored tests enabled (6 tests)

### Quality Requirements
✅ Zero test failures after implementation
✅ No regression in existing endpoints
✅ <5% performance impact
✅ Comprehensive error handling

### Documentation Requirements
✅ API documentation updated
✅ OpenAPI spec updated
✅ Usage examples provided
✅ CHANGELOG updated

---

## Timeline & Effort

| Phase | Tasks | Effort | Duration |
|-------|-------|--------|----------|
| Phase 1 | Extract Endpoint | 8-12h | 1-1.5 days |
| Phase 2 | Search Endpoint | 8-12h | 1-1.5 days |
| Phase 3 | Standardization | 4-6h | 0.5-1 day |
| Phase 4 | Test Infrastructure | 8-12h | 1-1.5 days |
| Phase 5 | Documentation | 4-6h | 0.5-1 day |
| **Total** | **All Phases** | **32-48h** | **4-6 days** |

---

## Risk Assessment

### Low Risk ✅
- Extract endpoint implementation (standard pattern)
- Search endpoint implementation (existing providers)
- Route aliases (simple additions)

### Medium Risk ⚠️
- Test infrastructure (`create_test_app()` complexity)
- AppState initialization for tests
- Mock vs real dependencies

### Mitigation Strategies
1. **Incremental testing:** Test each phase before proceeding
2. **Mock dependencies:** Use mocks for Redis, WASM in tests
3. **Backward compatibility:** Keep old routes, add new ones
4. **Code review:** Review each handler before integration

---

## Future Enhancements (v1.2+)

### Extract Endpoint Enhancements
- Support for batch extraction
- Streaming extraction results
- Custom extraction rules
- Content caching

### Search Endpoint Enhancements
- Multiple provider fallback
- Result ranking/filtering
- Search history tracking
- Provider health monitoring

---

## Conclusion

This implementation plan provides a clear, phased approach to adding the missing `/api/v1/extract` and `/api/v1/search` endpoints while improving test infrastructure and endpoint consistency.

**Key Deliverables:**
- 2 new REST endpoints
- 6 ignored tests enabled
- Improved test infrastructure
- Better endpoint consistency

**Total Effort:** 32-48 hours (4-6 working days)
**Tests Enabled:** 6 tests (from 78.1% to 80%+ pass rate)

---

**Report Generated:** 2025-10-10
**Status:** ✅ **READY FOR IMPLEMENTATION**
**Next Steps:** Begin Phase 1 - Extract Endpoint Implementation
