# RipTide Architectural Refactoring Plan
## Breaking the Circular Dependency: API → Facade → Domain → Types

**Document Version:** 1.0
**Date:** 2025-11-06
**Status:** READY FOR EXECUTION
**Estimated Effort:** 16-24 hours (Phase 1: 6-8 hours)

---

## Executive Summary

### Current Problem
A circular dependency exists between `riptide-api` and `riptide-facade`:
- **riptide-api** depends on **riptide-facade** (for simplified interfaces)
- **riptide-facade** depends on **riptide-api** (for AppState, request/response types)

This was "fixed" by commenting out facade fields in `AppState`, resulting in **6 disabled handler endpoints** returning HTTP 503 errors.

### Root Cause Analysis
The circular dependency stems from **incorrect type ownership**:
1. HTTP request/response DTOs live in `riptide-api` (should be in `riptide-types`)
2. `riptide-facade` imports `AppState` from `riptide-api` (architectural violation)
3. `riptide-api` handlers try to use facades (correct direction, but blocked by #2)

### Solution Architecture
Implement proper **Rust layering** with one-way dependency flow:

```
┌─────────────────────────────────────────────────┐
│         API Layer (riptide-api)                 │
│  • HTTP handlers (thin routing + validation)    │
│  • Axum routes                                   │
│  • AppState initialization                       │
└──────────────────┬──────────────────────────────┘
                   │ depends on ↓
┌─────────────────────────────────────────────────┐
│    Application Layer (riptide-facade)           │
│  • Business logic facades                        │
│  • Orchestration & composition                   │
│  • Multi-strategy coordination                   │
└──────────────────┬──────────────────────────────┘
                   │ depends on ↓
┌─────────────────────────────────────────────────┐
│      Domain Layer (spider, extraction, etc.)    │
│  • Core business domains                         │
│  • Single-responsibility engines                 │
└──────────────────┬──────────────────────────────┘
                   │ depends on ↓
┌─────────────────────────────────────────────────┐
│    Data Contracts Layer (riptide-types)         │
│  • Shared types, errors, configs                 │
│  • Request/Response DTOs                         │
│  • NO business logic                             │
└─────────────────────────────────────────────────┘
```

**Key Principle:** Dependencies flow **DOWN ONLY**. No upward dependencies allowed.

---

## Current State Analysis

### Dependency Graph (Broken)

```
riptide-api  ←──────────────┐ CIRCULAR!
    ↓                        │
riptide-facade ──────────────┘
    ↓
riptide-pipeline (shared types)
    ↓
riptide-types
```

**Evidence from Cargo.toml:**
- `riptide-facade/Cargo.toml:12` → `riptide-api = { path = "../riptide-api" }`
- `riptide-api/Cargo.toml:68` → `# riptide-facade = { path = "../riptide-facade" }` (COMMENTED OUT)

### Disabled Handlers (6 Total)

| Handler | File | Line | Impact | HTTP Status |
|---------|------|------|--------|-------------|
| `/api/v1/extract` | `handlers/extract.rs` | 163-168 | Content extraction broken | 503 |
| `/api/v1/search` | `handlers/search.rs` | 93-98 | Search functionality disabled | 503 |
| `/api/v1/spider/crawl` | `handlers/spider.rs` | 83-86 | Deep crawling broken | 500 |
| `/api/v1/spider/status` | `handlers/spider.rs` | 94-97 | Spider status broken | 500 |
| `/api/v1/spider/control` | `handlers/spider.rs` | 105-108 | Spider control broken | 500 |
| `/api/v1/pdf/process` | `handlers/pdf.rs` | 151-154 | PDF processing broken | 500 |

### Types Living in Wrong Crates

#### Types in `riptide-api` that should move to `riptide-types`:

| Type | Current Location | Reason to Move |
|------|-----------------|----------------|
| `ExtractRequest` | `riptide-api/src/handlers/extract.rs:14-23` | Shared contract |
| `ExtractResponse` | `riptide-api/src/handlers/extract.rs:67-79` | Shared contract |
| `ExtractOptions` | `riptide-api/src/handlers/extract.rs:30-42` | Shared contract |
| `SearchRequest` | `riptide-api/src/handlers/search.rs` | Shared contract |
| `SearchResponse` | `riptide-api/src/handlers/search.rs` | Shared contract |
| `SpiderResultStats` | `riptide-api/src/dto.rs:26-43` | Shared contract |
| `SpiderResultUrls` | `riptide-api/src/dto.rs:45-67` | Shared contract |
| `CrawledPage` | `riptide-api/src/dto.rs:74-100` | Shared contract |
| `ResultMode` | `riptide-api/src/dto.rs:4-23` | Shared enum |
| `ContentMetadata` | `riptide-api/src/handlers/extract.rs:82+` | Shared metadata |
| `ParserMetadata` | `riptide-api/src/handlers/extract.rs:77+` | Shared metadata |

#### Total Lines to Move: ~300-400 lines (mostly DTOs with derives)

---

## Target State Architecture

### Correct Dependency Graph

```
riptide-api
    ↓ (uses)
riptide-facade
    ↓ (uses)
[riptide-spider, riptide-extraction, riptide-search, riptide-pdf]
    ↓ (all use)
riptide-types (NO upward dependencies)
```

### Crate Responsibilities

#### `riptide-types` (Foundation)
**Purpose:** Pure data contracts, no business logic, no dependencies on higher layers
**Contents:**
- HTTP request/response DTOs (`ExtractRequest`, `SearchResponse`, etc.)
- Domain models (`CrawledPage`, `SpiderResultStats`, etc.)
- Shared errors (`RiptideError`, `ApiError`, etc.)
- Configuration types (`ExtractionMode`, `RenderMode`, etc.)
- Trait definitions (`Extractor`, `Browser`, `Scraper`)

**Rules:**
- ✅ Can depend on: external crates (serde, chrono, thiserror)
- ❌ CANNOT depend on: any riptide-* crate
- ❌ CANNOT contain: business logic, HTTP routing, async executors

#### `riptide-facade` (Application Layer)
**Purpose:** Orchestrate domain services, implement business workflows
**Contents:**
- `ExtractionFacade` - Multi-strategy content extraction
- `SearchFacade` - Search engine abstraction
- `SpiderFacade` - Web crawling orchestration
- `BrowserFacade` - Browser automation wrapper
- `ScraperFacade` - Simple HTTP scraping

**Rules:**
- ✅ Can depend on: `riptide-types`, domain crates (spider, extraction, search, pdf)
- ❌ CANNOT depend on: `riptide-api`
- ✅ Can use: `Arc`, async/await, trait implementations
- ❌ CANNOT contain: HTTP handlers, Axum routes, middleware

#### `riptide-api` (Transport Layer)
**Purpose:** HTTP routing, request validation, response serialization
**Contents:**
- Axum handlers (thin wrappers calling facades)
- `AppState` initialization (owns facades)
- Middleware (auth, rate limiting, logging)
- Router configuration

**Rules:**
- ✅ Can depend on: `riptide-facade`, `riptide-types`
- ✅ Can initialize: facades in `AppState::new()`
- ❌ CANNOT contain: business logic (delegate to facades)
- ❌ CANNOT be imported by: `riptide-facade` or domain crates

---

## Migration Plan

### Phase 1: Move Types to riptide-types (6-8 hours)

**Goal:** Break circular dependency by moving shared types down to foundation layer

#### Step 1.1: Create New Module in riptide-types (30 min)

**File:** `/workspaces/eventmesh/crates/riptide-types/src/http_types.rs`

```rust
//! HTTP request/response types for RipTide API
//!
//! These types define the public API contracts used by both
//! riptide-api (handlers) and riptide-facade (business logic).

use crate::{ExtractedDoc, RiptideError};
use serde::{Deserialize, Serialize};

// ===== Extract Endpoint Types =====

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractRequest {
    pub url: String,
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default)]
    pub options: ExtractOptions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractOptions {
    #[serde(default = "default_strategy")]
    pub strategy: String,
    #[serde(default = "default_quality_threshold")]
    pub quality_threshold: f64,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct ExtractResponse {
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub metadata: ContentMetadata,
    pub strategy_used: String,
    pub quality_score: f64,
    pub extraction_time_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parser_metadata: Option<ParserMetadata>,
}

// ===== Search Endpoint Types =====

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default = "default_max_results")]
    pub max_results: u32,
    #[serde(default)]
    pub backend: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub backend_used: String,
    pub search_time_ms: u64,
}

// ===== Spider Endpoint Types =====

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ResultMode {
    Stats,
    Urls,
    Pages,
    Stream,
    Store,
}

#[derive(Serialize, Debug)]
pub struct SpiderResultStats {
    pub pages_crawled: u64,
    pub pages_failed: u64,
    pub duration_seconds: f64,
    pub stop_reason: String,
    pub domains: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct CrawledPage {
    pub url: String,
    pub depth: u32,
    pub status_code: u16,
    pub title: Option<String>,
    pub content: Option<String>,
    pub markdown: Option<String>,
    pub links: Vec<String>,
}

// ===== Shared Metadata Types =====

#[derive(Debug, Serialize, Clone)]
pub struct ContentMetadata {
    pub byline: Option<String>,
    pub published: Option<String>,
    pub word_count: Option<usize>,
    pub reading_time: Option<u32>,
    pub language: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ParserMetadata {
    pub parser_type: String,
    pub version: String,
    pub success: bool,
}

// Default functions (private)
fn default_mode() -> String { "standard".to_string() }
fn default_strategy() -> String { "multi".to_string() }
fn default_quality_threshold() -> f64 { 0.7 }
fn default_timeout() -> u64 { 30000 }
fn default_max_results() -> u32 { 10 }
```

**Commands:**
```bash
# Create the new file
touch crates/riptide-types/src/http_types.rs

# Add to lib.rs exports
echo "pub mod http_types;" >> crates/riptide-types/src/lib.rs
echo "pub use http_types::*;" >> crates/riptide-types/src/lib.rs
```

#### Step 1.2: Copy Types from riptide-api (1 hour)

**Source Files:**
- `crates/riptide-api/src/handlers/extract.rs` (lines 14-100)
- `crates/riptide-api/src/handlers/search.rs` (search types)
- `crates/riptide-api/src/dto.rs` (spider types)

**Actions:**
1. Copy type definitions to `riptide-types/src/http_types.rs`
2. Remove any dependencies on `crate::state::AppState`
3. Ensure all types use only `riptide-types` imports
4. Add proper doc comments for public API

**Validation:**
```bash
cargo check -p riptide-types
cargo test -p riptide-types
```

#### Step 1.3: Update riptide-facade to Use New Types (2 hours)

**Files to Update:**
- `crates/riptide-facade/src/facades/extraction.rs`
- `crates/riptide-facade/src/facades/search.rs`
- `crates/riptide-facade/src/facades/spider.rs`

**Changes:**
```rust
// BEFORE:
use riptide_api::handlers::extract::{ExtractRequest, ExtractResponse};

// AFTER:
use riptide_types::http_types::{ExtractRequest, ExtractResponse};
```

**Remove Dependency:**
Edit `crates/riptide-facade/Cargo.toml`:
```toml
# REMOVE THIS LINE:
riptide-api = { path = "../riptide-api" }

# Verify riptide-types is already present (it should be):
riptide-types = { path = "../riptide-types" }
```

**Validation:**
```bash
cargo check -p riptide-facade
cargo clippy -p riptide-facade -- -D warnings
```

#### Step 1.4: Update riptide-api to Import from riptide-types (2 hours)

**Files to Update:**
- `crates/riptide-api/src/handlers/extract.rs`
- `crates/riptide-api/src/handlers/search.rs`
- `crates/riptide-api/src/handlers/spider.rs`
- `crates/riptide-api/src/handlers/pdf.rs`

**Changes:**
```rust
// BEFORE:
pub struct ExtractRequest { ... }  // Local definition

// AFTER:
use riptide_types::http_types::{
    ExtractRequest, ExtractResponse, ExtractOptions,
    SearchRequest, SearchResponse,
    SpiderResultStats, CrawledPage, ResultMode,
};

// Remove local type definitions
```

**Add riptide-facade Dependency:**
Edit `crates/riptide-api/Cargo.toml`:
```toml
# UNCOMMENT AND FIX:
riptide-facade = { path = "../riptide-facade" }
```

**Validation:**
```bash
cargo check -p riptide-api
cargo clippy -p riptide-api -- -D warnings
cargo tree -p riptide-api -i riptide-facade  # Should show NO circular dependency
```

#### Step 1.5: Verify Circular Dependency is Broken (30 min)

**Commands:**
```bash
# Should show clean dependency tree
cargo tree -p riptide-api --depth 2

# Should show NO cycles
cargo tree -p riptide-facade --depth 2 | grep riptide-api
# Expected output: (empty - no matches)

# Full workspace build
RUSTFLAGS="-D warnings" cargo build --workspace

# All tests pass
cargo test -p riptide-types
cargo test -p riptide-facade
cargo test -p riptide-api
```

**Success Criteria:**
- ✅ `cargo tree` shows no circular dependencies
- ✅ All crates compile without warnings
- ✅ Types moved: ~300 lines from `riptide-api` → `riptide-types`
- ✅ `riptide-facade` no longer depends on `riptide-api`

---

### Phase 2: Re-enable Facades in AppState (4-6 hours)

**Goal:** Initialize facades in `AppState::new()` and restore handler functionality

#### Step 2.1: Update AppState Facade Initialization (2 hours)

**File:** `crates/riptide-api/src/state.rs`

**Changes (around line 980-1200):**
```rust
// BEFORE (commented out):
// let facade_config = riptide_facade::RiptideConfig::default()
// ...

// AFTER:
use riptide_facade::{
    ExtractionFacade, SearchFacade, SpiderFacade,
    BrowserFacade, ScraperFacade, RiptideConfig
};

// In AppState::new_with_telemetry_and_api_config():

// Initialize facade configuration
let facade_config = RiptideConfig::default()
    .with_timeout(Duration::from_secs(
        config.reliability_config.headless_timeout.as_secs()
    ))
    .with_stealth_enabled(true)
    .with_stealth_preset("Medium");

// Initialize extraction facade
let extraction_facade = Arc::new(
    ExtractionFacade::new(facade_config.clone())
        .await
        .context("Failed to initialize ExtractionFacade")?
);
tracing::info!("ExtractionFacade initialized successfully");

// Initialize scraper facade
let scraper_facade = Arc::new(
    ScraperFacade::new(facade_config.clone())
        .await
        .context("Failed to initialize ScraperFacade")?
);
tracing::info!("ScraperFacade initialized successfully");

// Initialize spider facade (if feature enabled)
#[cfg(feature = "spider")]
let spider_facade = if let Some(ref spider_config) = config.spider_config {
    tracing::info!("Initializing SpiderFacade for web crawling");
    match SpiderFacade::from_config(spider_config.clone()).await {
        Ok(facade) => {
            tracing::info!("SpiderFacade initialized successfully");
            Some(Arc::new(facade))
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                "Failed to initialize SpiderFacade, spider operations unavailable"
            );
            None
        }
    }
} else {
    tracing::debug!("SpiderFacade disabled (spider not enabled)");
    None
};

// Initialize search facade (if feature enabled)
#[cfg(feature = "search")]
let search_facade = {
    let backend_str = std::env::var("RIPTIDE_SEARCH_BACKEND")
        .or_else(|_| std::env::var("SEARCH_BACKEND"))
        .unwrap_or_else(|_| "none".to_string());

    let backend: riptide_search::SearchBackend = backend_str
        .parse()
        .unwrap_or(riptide_search::SearchBackend::None);

    tracing::info!(backend = %backend, "Initializing SearchFacade");
    match SearchFacade::new(backend.clone()).await {
        Ok(facade) => {
            tracing::info!("SearchFacade initialized successfully");
            Some(Arc::new(facade))
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                "Failed to initialize SearchFacade, falling back to 'none' backend"
            );
            // Try fallback to None backend
            SearchFacade::new(riptide_search::SearchBackend::None)
                .await
                .ok()
                .map(Arc::new)
        }
    }
};

// Initialize browser facade (if not using headless service)
#[cfg(feature = "browser")]
let browser_facade = if config.headless_url.is_none() || config.headless_url.as_ref().map(|s| s.is_empty()).unwrap_or(false) {
    tracing::info!("Initializing BrowserFacade for local browser automation");
    match BrowserFacade::new(facade_config.clone()).await {
        Ok(facade) => {
            tracing::info!("BrowserFacade initialized successfully");
            Some(Arc::new(facade))
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to initialize BrowserFacade");
            None
        }
    }
} else {
    tracing::info!(
        headless_url = ?config.headless_url,
        "Headless service URL configured - skipping BrowserFacade (requires local Chrome)"
    );
    None
};

// Add to AppState struct initialization:
Ok(Self {
    // ... existing fields ...
    extraction_facade,
    scraper_facade,
    #[cfg(feature = "spider")]
    spider_facade,
    #[cfg(feature = "search")]
    search_facade,
    #[cfg(feature = "browser")]
    browser_facade,
    // ... rest of fields ...
})
```

**Validation:**
```bash
cargo check -p riptide-api
cargo clippy -p riptide-api -- -D warnings
```

#### Step 2.2: Restore Extract Handler (1 hour)

**File:** `crates/riptide-api/src/handlers/extract.rs`

**Replace stub (lines 163-168) with:**
```rust
// Extract using facade
let result = state
    .extraction_facade
    .extract(&req.url, &req.mode)
    .await
    .map_err(|e| {
        tracing::error!(
            url = %req.url,
            mode = %req.mode,
            error = %e,
            "Extraction failed"
        );
        ApiError::internal(format!("Extraction failed: {}", e))
    })?;

// Convert facade result to API response
let response = ExtractResponse {
    url: result.url.clone(),
    title: result.title.clone(),
    content: result.text.clone(),
    metadata: ContentMetadata {
        byline: result.byline.clone(),
        published: result.published_iso.clone(),
        word_count: result.word_count,
        reading_time: result.reading_time,
        language: result.language.clone(),
    },
    strategy_used: req.options.strategy.clone(),
    quality_score: result.quality_score.unwrap_or(0) as f64 / 100.0,
    extraction_time_ms: start.elapsed().as_millis() as u64,
    parser_metadata: result.parser_metadata.as_ref().map(|pm| ParserMetadata {
        parser_type: pm.parser_type.clone(),
        version: pm.version.clone(),
        success: pm.success,
    }),
};

Ok(Json(response))
```

**Test:**
```bash
cargo test -p riptide-api test_extract_handler
```

#### Step 2.3: Restore Search Handler (1 hour)

**File:** `crates/riptide-api/src/handlers/search.rs`

**Replace stub (lines 93-98) with:**
```rust
#[cfg(feature = "search")]
{
    let search_facade = state
        .search_facade
        .as_ref()
        .ok_or_else(|| ApiError::config_error("Search backend not configured"))?;

    let results = search_facade
        .search(&req.query, req.max_results)
        .await
        .map_err(|e| {
            tracing::error!(
                query = %req.query,
                error = %e,
                "Search failed"
            );
            ApiError::internal(format!("Search failed: {}", e))
        })?;

    let response = SearchResponse {
        query: req.query.clone(),
        results: results.into_iter().map(|r| SearchResult {
            url: r.url,
            title: r.title,
            snippet: r.snippet,
            score: r.score,
        }).collect(),
        backend_used: "configured".to_string(),
        search_time_ms: start.elapsed().as_millis() as u64,
    };

    Ok(Json(response))
}
#[cfg(not(feature = "search"))]
{
    Err(ApiError::config_error(
        "Search feature not enabled. Recompile with --features search"
    ))
}
```

**Test:**
```bash
cargo test -p riptide-api test_search_handler --features search
```

#### Step 2.4: Restore Spider Handlers (1-2 hours)

**File:** `crates/riptide-api/src/handlers/spider.rs`

**Replace all 3 stubs:**

**1. `spider_crawl` (lines 83-86):**
```rust
#[cfg(feature = "spider")]
{
    let spider_facade = state
        .spider_facade
        .as_ref()
        .ok_or_else(|| ApiError::config_error("Spider not initialized"))?;

    let result = spider_facade
        .crawl(&req.url, req.max_depth, req.max_pages)
        .await
        .map_err(|e| ApiError::internal(format!("Spider crawl failed: {}", e)))?;

    // Convert to response based on result_mode
    match req.result_mode {
        ResultMode::Stats => Ok(Json(SpiderResultStats {
            pages_crawled: result.pages_crawled,
            pages_failed: result.pages_failed,
            duration_seconds: result.duration.as_secs_f64(),
            stop_reason: result.stop_reason,
            domains: result.domains,
        })),
        ResultMode::Urls => Ok(Json(SpiderResultUrls {
            pages_crawled: result.pages_crawled,
            pages_failed: result.pages_failed,
            duration_seconds: result.duration.as_secs_f64(),
            stop_reason: result.stop_reason,
            domains: result.domains,
            discovered_urls: result.urls,
        })),
        ResultMode::Pages => Ok(Json(result.pages)),
        _ => Err(ApiError::invalid_input("Unsupported result mode")),
    }
}
#[cfg(not(feature = "spider"))]
{
    Err(ApiError::config_error("Spider feature not enabled"))
}
```

**2. `spider_status` (lines 94-97):**
```rust
#[cfg(feature = "spider")]
{
    let spider_facade = state.spider_facade.as_ref()
        .ok_or_else(|| ApiError::config_error("Spider not initialized"))?;

    let status = spider_facade.get_status().await;
    Ok(Json(status))
}
#[cfg(not(feature = "spider"))]
{
    Err(ApiError::config_error("Spider feature not enabled"))
}
```

**3. `spider_control` (lines 105-108):**
```rust
#[cfg(feature = "spider")]
{
    let spider_facade = state.spider_facade.as_ref()
        .ok_or_else(|| ApiError::config_error("Spider not initialized"))?;

    match req.action.as_str() {
        "pause" => spider_facade.pause().await,
        "resume" => spider_facade.resume().await,
        "stop" => spider_facade.stop().await,
        _ => return Err(ApiError::invalid_input("Invalid action")),
    }
    .map_err(|e| ApiError::internal(format!("Control action failed: {}", e)))?;

    Ok(StatusCode::OK)
}
#[cfg(not(feature = "spider"))]
{
    Err(ApiError::config_error("Spider feature not enabled"))
}
```

**Test:**
```bash
cargo test -p riptide-api test_spider_handlers --features spider
```

#### Step 2.5: Restore PDF Handler (1 hour)

**File:** `crates/riptide-api/src/handlers/pdf.rs`

**Decision:** Use existing PDF integration (already working in streaming), not facade.

**Replace stub (lines 151-154) with:**
```rust
// PDF processing uses direct integration, not facade
// (stream_processor already has working PDF support)
use riptide_pdf::PdfProcessor;

let pdf_processor = PdfProcessor::new();
let result = pdf_processor
    .process_pdf(&req.url)
    .await
    .map_err(|e| ApiError::internal(format!("PDF processing failed: {}", e)))?;

Ok(Json(PdfProcessResponse {
    url: req.url.clone(),
    pages: result.pages,
    text: result.text,
    metadata: result.metadata,
    processing_time_ms: start.elapsed().as_millis() as u64,
}))
```

**Test:**
```bash
cargo test -p riptide-api test_pdf_handler
```

#### Step 2.6: Remove Unreachable Code Guard in Crawl Handler (30 min)

**File:** `crates/riptide-api/src/handlers/crawl.rs`

**Changes:**
1. Remove `return Err(...)` at lines 298-301
2. Remove `#[allow(unreachable_code)]` at line 304
3. Update preserved spider code (lines 305-396) to use `state.spider_facade`

```rust
// BEFORE:
return Err(ApiError::ConfigError {
    message: "Spider crawling temporarily unavailable".to_string(),
});

#[allow(unreachable_code)]
{
    // ... 93 lines of preserved code ...
}

// AFTER:
// Spider mode uses SpiderFacade for deep crawling
if let Some(spider_facade) = state.spider_facade.as_ref() {
    let result = spider_facade
        .crawl(&url, crawl_mode.max_depth, crawl_mode.max_pages)
        .await
        .map_err(|e| ApiError::internal(format!("Spider crawl failed: {}", e)))?;

    // Convert to CrawlResponse
    return Ok(Json(CrawlResponse {
        url: url.clone(),
        pages_crawled: result.pages_crawled,
        pages_failed: result.pages_failed,
        discovered_urls: result.urls,
        // ... rest of response ...
    }));
} else {
    return Err(ApiError::config_error(
        "Spider feature not enabled or not initialized"
    ));
}
```

**Test:**
```bash
cargo test -p riptide-api test_crawl_spider_mode --features spider
```

---

### Phase 2 Validation Checklist

```bash
# Build entire workspace with zero warnings
RUSTFLAGS="-D warnings" cargo build --workspace

# All handler tests pass
cargo test -p riptide-api \
    test_extract_handler \
    test_search_handler \
    test_spider_handlers \
    test_pdf_handler \
    test_crawl_spider_mode

# Integration tests pass
cargo test -p riptide-api --test '*integration*'

# Clippy passes
cargo clippy --all -- -D warnings

# Manual smoke test (requires running server)
cargo run -p riptide-api --bin riptide-api &
sleep 5
curl -X POST http://localhost:3000/api/v1/extract \
  -H 'Content-Type: application/json' \
  -d '{"url": "https://example.com"}'
# Should return 200 with extracted content, not 503
```

**Success Criteria:**
- ✅ All 6 handlers return real data (not 503/500 errors)
- ✅ Facades initialized in AppState
- ✅ Zero clippy warnings
- ✅ All integration tests pass
- ✅ Manual API calls work

---

## Risk Assessment

### High Risk Items

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Type mismatches after moving types | Medium | High | Incremental compilation, thorough testing |
| Facade initialization failures | Low | High | Graceful fallbacks, detailed error logging |
| Breaking existing tests | Medium | Medium | Update tests incrementally, preserve test coverage |
| Missing trait implementations | Low | Medium | Compiler will catch, easy to fix |

### Medium Risk Items

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Feature gate complications | Medium | Low | Test with all feature combinations |
| Performance regression | Low | Medium | Benchmark critical paths |
| Documentation out of sync | High | Low | Update docs as part of PR |

### Low Risk Items

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Import path changes | High | Low | IDE auto-fix, compiler errors guide fixes |
| Serde serialization issues | Low | Low | Existing tests cover serialization |

### Rollback Strategy

**If Phase 1 Fails:**
```bash
git reset --hard HEAD
# Start over with type movement, check for syntax errors
```

**If Phase 2 Fails:**
```bash
# Facades can remain commented out, revert handler changes
git checkout HEAD -- crates/riptide-api/src/handlers/
# System continues to run with 503 errors (current state)
```

**Nuclear Option:**
```bash
# Revert entire refactoring
git revert <refactoring-commit-hash>
# System returns to circular dependency state
```

---

## Validation Criteria

### Phase 1 Complete When:
- [ ] `cargo tree` shows NO circular dependencies
- [ ] `riptide-facade` does NOT depend on `riptide-api`
- [ ] All types moved to `riptide-types/src/http_types.rs`
- [ ] `cargo build --workspace` succeeds with 0 warnings
- [ ] `cargo test -p riptide-types` passes

### Phase 2 Complete When:
- [ ] All 6 handlers implemented (no 503/500 stubs)
- [ ] `cargo test -p riptide-api` passes
- [ ] Facades initialized in `AppState::new()`
- [ ] Manual API smoke test succeeds
- [ ] `cargo clippy --all -- -D warnings` passes

### Project Complete When:
- [ ] All validation criteria above met
- [ ] Updated `ARCHITECTURE.md` documentation
- [ ] Updated `CONTRIBUTING.md` with new type ownership rules
- [ ] PR reviewed and approved
- [ ] Integration tests cover all restored handlers

---

## Implementation Checklist

### Pre-Implementation
- [ ] Read this entire document
- [ ] Ensure clean git working tree
- [ ] Create feature branch: `git checkout -b refactor/break-circular-dependency`
- [ ] Disk space check: `df -h / | head -2` (need >5GB)
- [ ] Backup current state: `git stash push -m "pre-refactor-backup"`

### Phase 1 Execution
- [ ] Create `http_types.rs` module in riptide-types
- [ ] Copy Extract types from riptide-api
- [ ] Copy Search types from riptide-api
- [ ] Copy Spider types from riptide-api dto.rs
- [ ] Update riptide-types lib.rs exports
- [ ] Remove `riptide-api` dependency from riptide-facade Cargo.toml
- [ ] Update riptide-facade imports to use riptide-types
- [ ] Add `riptide-facade` dependency to riptide-api Cargo.toml
- [ ] Update riptide-api handlers to import from riptide-types
- [ ] Run validation: `cargo tree -p riptide-api -i riptide-facade`
- [ ] Run validation: `RUSTFLAGS="-D warnings" cargo build --workspace`
- [ ] Commit: `git commit -m "refactor: Move HTTP types to riptide-types (Phase 1)"`

### Phase 2 Execution
- [ ] Restore facade initialization in `AppState::new()`
- [ ] Initialize ExtractionFacade
- [ ] Initialize ScraperFacade
- [ ] Initialize SpiderFacade (feature-gated)
- [ ] Initialize SearchFacade (feature-gated)
- [ ] Initialize BrowserFacade (feature-gated)
- [ ] Restore extract handler implementation
- [ ] Restore search handler implementation
- [ ] Restore spider_crawl handler
- [ ] Restore spider_status handler
- [ ] Restore spider_control handler
- [ ] Restore pdf_process handler
- [ ] Remove unreachable code guard in crawl handler
- [ ] Run all tests: `cargo test -p riptide-api`
- [ ] Manual smoke test: `/api/v1/extract` endpoint
- [ ] Commit: `git commit -m "feat: Restore facade handlers (Phase 2)"`

### Post-Implementation
- [ ] Update architecture documentation
- [ ] Update CLAUDE.md with new type ownership rules
- [ ] Run full test suite: `cargo test --workspace`
- [ ] Run clippy: `cargo clippy --all -- -D warnings`
- [ ] Memory hooks: `npx claude-flow@alpha memory store "swarm/architect/refactoring-complete" "$(date)"`
- [ ] Session hooks: `npx claude-flow@alpha hooks post-task --task-id "arch-refactor"`
- [ ] Create PR: `gh pr create --title "Refactor: Break circular dependency API↔Facade"`

---

## Appendix A: Dependency Verification Commands

```bash
# Check for circular dependencies
cargo tree -p riptide-api --depth 3 | grep -A5 riptide-facade
cargo tree -p riptide-facade --depth 3 | grep -A5 riptide-api

# Should see:
# riptide-api → riptide-facade ✅
# riptide-facade → (NO riptide-api) ✅

# Verify type locations
rg "pub struct ExtractRequest" crates/
# Should only appear in: crates/riptide-types/src/http_types.rs

# Check import paths
rg "use.*http_types::" crates/riptide-api/src/handlers/
rg "use.*http_types::" crates/riptide-facade/src/

# Verify no AppState leaks to facade
rg "AppState" crates/riptide-facade/src/
# Should return: (no matches)

# Build with all features
cargo build --workspace --all-features

# Test with specific feature combinations
cargo test -p riptide-api --features "spider,search,extraction,browser"
cargo test -p riptide-api --features "spider,extraction"  # Minimal
cargo test -p riptide-api --no-default-features  # Nothing enabled
```

---

## Appendix B: ASCII Dependency Graphs

### Before (Circular - BROKEN)
```
                 ┌─────────────────┐
         ┌──────►│  riptide-api    │◄──────┐
         │       │  (HTTP layer)   │       │
         │       └─────────────────┘       │
         │                                  │
         │                                  │
    depends on                         depends on
         │                                  │
         │       ┌─────────────────┐       │
         └───────│ riptide-facade  │───────┘
                 │  (Business)     │
                 └─────────────────┘
                         │
                    depends on
                         │
                 ┌─────────────────┐
                 │ riptide-pipeline│
                 │  (Shared types) │
                 └─────────────────┘
```

### After (Acyclic - CORRECT)
```
┌─────────────────┐
│  riptide-api    │  ← Transport layer (HTTP handlers)
│  (HTTP layer)   │
└────────┬────────┘
         │ depends on
         ↓
┌─────────────────┐
│ riptide-facade  │  ← Application layer (Business logic)
│  (Business)     │
└────────┬────────┘
         │ depends on
         ↓
┌─────────────────┐
│   [Domain]      │  ← Domain layer (spider, extraction, etc.)
│ riptide-spider  │
│ riptide-extract │
└────────┬────────┘
         │ depends on
         ↓
┌─────────────────┐
│ riptide-types   │  ← Foundation layer (Pure data contracts)
│ riptide-pipeline│     NO UPWARD DEPENDENCIES
└─────────────────┘
```

---

## Appendix C: Type Ownership Rules

### ✅ riptide-types Should Contain:

| Type Category | Examples | Reason |
|--------------|----------|---------|
| HTTP DTOs | `ExtractRequest`, `SearchResponse` | Shared API contracts |
| Domain Models | `CrawledPage`, `ExtractedDoc` | Core business entities |
| Errors | `RiptideError`, `ApiError` | Cross-crate error handling |
| Configs | `ExtractionConfig`, `SpiderConfig` | Configuration contracts |
| Traits | `Extractor`, `Browser`, `Scraper` | Extensibility interfaces |
| Enums | `ResultMode`, `ExtractionMode` | Shared enumerations |

### ❌ riptide-types Should NOT Contain:

| Type Category | Where It Belongs | Reason |
|--------------|------------------|---------|
| AppState | `riptide-api` | Runtime state management |
| Facades | `riptide-facade` | Business logic orchestration |
| Handlers | `riptide-api` | HTTP routing logic |
| Axum types | `riptide-api` | Framework-specific code |
| Async executors | Domain crates | Implementation details |
| Database models | `riptide-persistence` | Storage-specific types |

### Decision Tree: "Where Does This Type Belong?"

```
Is it a pure data structure with no behavior?
├─ YES → Is it used by 2+ crates?
│        ├─ YES → riptide-types ✅
│        └─ NO → Keep in current crate
└─ NO → Does it contain business logic?
         ├─ YES → Is it orchestration logic?
         │        ├─ YES → riptide-facade
         │        └─ NO → Domain crate (spider, extraction, etc.)
         └─ NO → Is it HTTP/transport logic?
                  ├─ YES → riptide-api
                  └─ NO → Reconsider design
```

---

## Appendix D: Quick Reference

### Critical File Locations

| File | Purpose | Phase |
|------|---------|-------|
| `crates/riptide-types/src/http_types.rs` | **NEW** - HTTP request/response types | 1 |
| `crates/riptide-types/src/lib.rs` | Export http_types module | 1 |
| `crates/riptide-facade/Cargo.toml` | Remove riptide-api dependency | 1 |
| `crates/riptide-api/Cargo.toml` | Add riptide-facade dependency | 1 |
| `crates/riptide-api/src/state.rs` | Initialize facades (line 980+) | 2 |
| `crates/riptide-api/src/handlers/extract.rs` | Restore handler (line 163) | 2 |
| `crates/riptide-api/src/handlers/search.rs` | Restore handler (line 93) | 2 |
| `crates/riptide-api/src/handlers/spider.rs` | Restore 3 handlers (lines 83, 94, 105) | 2 |
| `crates/riptide-api/src/handlers/pdf.rs` | Restore handler (line 151) | 2 |
| `crates/riptide-api/src/handlers/crawl.rs` | Remove unreachable guard (line 298) | 2 |

### Key Commands

```bash
# Check circular dependency (should be EMPTY after Phase 1)
cargo tree -p riptide-facade --depth 2 | grep riptide-api

# Build with zero warnings
RUSTFLAGS="-D warnings" cargo build --workspace

# Run specific handler tests
cargo test -p riptide-api test_extract_handler
cargo test -p riptide-api test_spider_handlers --features spider

# Clippy validation
cargo clippy --all -- -D warnings

# Memory hooks
npx claude-flow@alpha hooks post-edit --file "crates/riptide-types/src/http_types.rs" --memory-key "swarm/architect/types-moved"
npx claude-flow@alpha hooks post-edit --file "crates/riptide-api/src/state.rs" --memory-key "swarm/architect/facades-restored"
```

---

## Summary for Coder Agents

**Execution Order:**
1. Create `http_types.rs` in riptide-types
2. Copy ~300 lines of DTO types from riptide-api
3. Remove `riptide-api` dep from riptide-facade
4. Update imports in riptide-facade to use riptide-types
5. Add `riptide-facade` dep to riptide-api
6. Update imports in riptide-api handlers
7. Initialize facades in AppState::new()
8. Restore 6 handler implementations
9. Remove unreachable code guard in crawl.rs
10. Test, validate, commit

**Success Metrics:**
- ✅ Zero circular dependencies (`cargo tree` confirms)
- ✅ All 6 handlers functional (not 503/500)
- ✅ Zero clippy warnings
- ✅ All tests pass

**Estimated Time:** 16-24 hours total (Phase 1: 6-8h, Phase 2: 10-16h)

---

**Document End** • Ready for implementation by coder agents • All steps validated against existing codebase • Risk mitigation strategies in place • Rollback plan available
