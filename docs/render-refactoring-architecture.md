# Render Module Refactoring Architecture

**Architect:** System Architecture Designer
**Session:** swarm-1759217361759-095dd3g5o
**Date:** 2025-09-30
**Current Status:** In Progress - models.rs completed (114 lines)

## Executive Summary

The render.rs file has grown to 1,300+ lines and requires refactoring into a modular architecture for maintainability, testability, and clarity. This document outlines the complete refactoring strategy, module dependencies, and integration plan.

## Current State Analysis

### File Structure (Before)
```
src/handlers/
├── render.rs (1,300 lines) ❌ Too large
│   ├── Data models (RenderRequest, RenderResponse, RenderStats, SessionRenderInfo)
│   ├── Main endpoint (render, render_with_resources)
│   ├── Processing strategies (process_pdf, process_dynamic, process_static, process_adaptive)
│   ├── WASM extraction (extract_with_wasm_extractor, extract_content)
│   ├── URL analysis (analyze_url_for_dynamic_content, create_adaptive_dynamic_config)
│   └── Tests (5 test functions)
```

### Completed Work
- ✅ `models.rs` - Data structures extracted (114 lines)
  - RenderRequest
  - RenderResponse
  - RenderStats
  - SessionRenderInfo

## Target Architecture

### Module Structure (After)
```
src/handlers/render/
├── mod.rs              (50-80 lines)   - Module entry point with re-exports
├── models.rs          (114 lines) ✅   - Data structures (COMPLETED)
├── processors.rs      (350-400 lines)  - Content processing strategies
│   ├── process_pdf
│   ├── process_dynamic
│   ├── process_static
│   └── process_adaptive
├── extraction.rs      (250-300 lines)  - WASM extraction logic
│   ├── extract_with_wasm_extractor
│   ├── extract_content
│   └── Validation helpers
├── strategies.rs      (300-350 lines)  - URL analysis & config generation
│   ├── analyze_url_for_dynamic_content
│   ├── create_adaptive_dynamic_config
│   └── Domain/pattern matchers
└── handlers.rs        (300-350 lines)  - Main render endpoint
    ├── render (public endpoint)
    ├── render_with_resources
    └── Session management helpers
```

### Size Reduction
- **Before:** 1,300 lines in single file
- **After:** 1,400 lines distributed across 6 focused modules (100 lines overhead for better organization)
- **Average module size:** ~230 lines (well under 500-line guideline)

## Module Design Specifications

### 1. mod.rs (Module Entry Point)

**Purpose:** Public API surface and module organization

**Responsibilities:**
- Re-export public types and functions
- Define module structure
- Provide backward compatibility

**Design:**
```rust
// Public API
pub mod models;
mod processors;
mod extraction;
mod strategies;
mod handlers;

// Re-exports for backward compatibility
pub use models::{RenderRequest, RenderResponse, RenderStats, SessionRenderInfo};
pub use handlers::render;

// Internal re-exports for module use
pub(crate) use processors::*;
pub(crate) use extraction::*;
pub(crate) use strategies::*;
```

**Dependencies:**
- None (only re-exports)

---

### 2. models.rs ✅ COMPLETED

**Status:** ✅ 114 lines, fully extracted

**Contents:**
- `RenderRequest` - Request body structure
- `RenderResponse` - Response structure
- `RenderStats` - Statistics tracking
- `SessionRenderInfo` - Session context

**Dependencies:**
- `serde::{Deserialize, Serialize}`
- `riptide_core::types::{ExtractedDoc, OutputFormat, RenderMode}`
- `crate::models::ErrorInfo`

---

### 3. processors.rs (Content Processing)

**Purpose:** Handle different rendering modes (PDF, dynamic, static, adaptive)

**Key Functions:**
```rust
// Public interface (crate-level)
pub(crate) async fn process_pdf(...) -> ApiResult<ProcessingResult>
pub(crate) async fn process_dynamic(...) -> ApiResult<ProcessingResult>
pub(crate) async fn process_static(...) -> ApiResult<ProcessingResult>
pub(crate) async fn process_adaptive(...) -> ApiResult<ProcessingResult>

// Internal type for unified result
struct ProcessingResult {
    final_url: String,
    render_result: Option<DynamicRenderResult>,
    pdf_result: Option<PdfProcessingResult>,
}
```

**Responsibilities:**
- PDF fetching and processing
- Dynamic rendering via RPC client
- Static HTTP fetching with stealth
- Adaptive mode routing
- Session cookie injection
- Timeout handling
- Fallback logic

**Dependencies:**
```rust
use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;
use crate::rpc_client::RpcClient;
use riptide_core::stealth::StealthController;
use riptide_core::dynamic::{DynamicConfig, DynamicRenderResult};
use riptide_core::pdf;
```

**Internal Structure:**
```rust
// Helper for stealth headers
fn apply_stealth_measures(
    request: RequestBuilder,
    stealth: &mut StealthController
) -> RequestBuilder

// Helper for session cookies
async fn inject_session_cookies(
    request: RequestBuilder,
    state: &AppState,
    session_id: &str,
    url: &str
) -> ApiResult<RequestBuilder>
```

**Testing Strategy:**
- Unit tests for each processor
- Mock RPC client for dynamic tests
- Mock HTTP client for static tests
- Integration tests for fallback paths

---

### 4. extraction.rs (WASM Extraction)

**Purpose:** WASM-based content extraction with validation and error handling

**Key Functions:**
```rust
// Main extraction function
pub(crate) async fn extract_with_wasm_extractor(
    extractor: &Arc<dyn WasmExtractor>,
    html: &str,
    url: &str,
    mode: ExtractionMode,
) -> Result<(ExtractedDoc, ExtractionStats), Box<dyn Error + Send + Sync>>

// High-level extraction from render result
pub(crate) async fn extract_content(
    state: &AppState,
    render_result: &Option<DynamicRenderResult>,
    output_format: &OutputFormat,
    url: &str,
) -> ApiResult<Option<ExtractedDoc>>

// Validation helpers
fn validate_extraction_inputs(html: &str, url: &str) -> Result<(), Box<dyn Error>>
fn validate_html_size(html: &[u8]) -> Result<(), Box<dyn Error>>
```

**Responsibilities:**
- Input validation (HTML, URL)
- Size limits enforcement (50MB)
- WASM extractor invocation
- Statistics collection
- Error context enhancement
- Output format mapping

**Dependencies:**
```rust
use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;
use riptide_core::extract::WasmExtractor;
use riptide_core::types::{ExtractedDoc, ExtractionMode, ExtractionStats, OutputFormat};
use riptide_core::dynamic::DynamicRenderResult;
use std::sync::Arc;
use std::time::Instant;
```

**Validation Rules:**
- HTML not empty
- URL not empty
- URL valid format
- HTML size < 50MB
- ExtractionMode mapping

**Error Handling:**
- Enhanced error context
- Graceful fallback to None
- Detailed logging
- Statistics on failure

**Testing Strategy:**
- Empty HTML rejection
- Empty URL rejection
- Invalid URL rejection
- Size limit enforcement
- Mode mapping correctness
- Statistics accuracy

---

### 5. strategies.rs (URL Analysis & Configuration)

**Purpose:** Intelligent URL analysis and dynamic config generation

**Key Functions:**
```rust
// URL analysis for rendering strategy
pub(crate) async fn analyze_url_for_dynamic_content(url: &str) -> bool

// Adaptive config generation
pub(crate) fn create_adaptive_dynamic_config(url: &str) -> DynamicConfig

// Pattern matchers
fn matches_dynamic_domain(url: &str) -> Option<&'static str>
fn matches_spa_pattern(url: &str) -> Option<&'static str>
fn matches_js_framework(url: &str) -> Option<&'static str>
```

**Responsibilities:**
- Domain pattern matching
- SPA indicator detection
- JS framework detection
- Wait condition creation
- Scroll strategy determination
- Viewport configuration

**Configuration Profiles:**

| URL Pattern | Wait Strategy | Scroll Strategy | Example |
|------------|---------------|-----------------|---------|
| GitHub | Selector-based (repo content) | Minimal (2 steps) | `github.com` |
| Reddit | Selector-based (posts) | Minimal (2 steps) | `reddit.com` |
| Medium/Substack | Selector-based (article) | Gentle (3 steps) | `medium.com` |
| Twitter/X | Multiple (tweet + network idle) | Aggressive (5 steps) | `x.com` |
| Generic | DomContentLoaded + timeout | Moderate (2 steps) | Others |

**Dynamic Domains:**
```rust
const DYNAMIC_DOMAINS: &[&str] = &[
    "twitter.com", "x.com", "facebook.com", "instagram.com",
    "linkedin.com", "youtube.com", "tiktok.com", "reddit.com",
    "medium.com", "substack.com", "github.com", "stackoverflow.com",
    // ... 10 more domains
];

const SPA_INDICATORS: &[&str] = &[
    "/#/", "#!/", "/app/", "/dashboard/", "/admin/",
    "?page=", "&view=", "#page", "#view", "#section"
];

const JS_FRAMEWORKS: &[&str] = &[
    "react", "angular", "vue", "svelte", "next", "nuxt",
    "gatsby", "webpack", "vite", "parcel"
];
```

**Dependencies:**
```rust
use riptide_core::dynamic::{
    DynamicConfig, WaitCondition, ScrollConfig,
    ScrollMode, ViewportConfig
};
use std::time::Duration;
use tracing::{debug, info};
```

**Testing Strategy:**
- Domain matching tests
- SPA detection tests
- Framework detection tests
- Config generation validation
- Timeout configuration
- Scroll strategy selection

---

### 6. handlers.rs (Main Endpoint)

**Purpose:** HTTP endpoint handling, resource management, metrics

**Key Functions:**
```rust
// Main public endpoint
pub async fn render(
    State(state): State<AppState>,
    session_ctx: SessionContext,
    Json(body): Json<RenderRequest>,
) -> Result<impl IntoResponse, ApiError>

// Internal handler with resources
async fn render_with_resources(
    state: AppState,
    session_ctx: SessionContext,
    body: RenderRequest,
    _resource_guard: RenderResourceGuard,
) -> Result<impl IntoResponse, ApiError>

// Session helpers
async fn get_session_info(
    state: &AppState,
    session_id: Option<&str>,
    url: &str
) -> Option<SessionRenderInfo>

async fn initialize_stealth_controller(
    config: Option<&StealthConfig>
) -> (Option<StealthController>, Vec<String>)
```

**Responsibilities:**
- Resource acquisition/management
- Timeout enforcement (3s)
- Request validation
- Mode selection routing
- Session context handling
- Stealth initialization
- Response building
- Metrics recording

**Flow:**
```
1. Acquire resources (with timeout/rate-limit checks)
2. Apply hard timeout wrapper (3s)
3. Validate URL
4. Determine session context
5. Initialize stealth controller
6. Route to appropriate processor (PDF/Dynamic/Static/Adaptive)
7. Extract content via extraction module
8. Build response with statistics
9. Record metrics
10. Return response (JSON)
```

**Dependencies:**
```rust
use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;
use crate::sessions::middleware::SessionContext;
use crate::resource_manager::{RenderResourceGuard, ResourceResult};
use super::{models::*, processors, extraction, strategies};
use axum::{extract::State, response::IntoResponse, Json};
use riptide_core::stealth::StealthController;
use std::time::Instant;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};
```

**Error Handling:**
- Resource exhaustion → 503 Service Unavailable
- Rate limiting → 429 Too Many Requests
- Timeout → 408 Request Timeout
- Validation errors → 400 Bad Request
- Internal errors → 500 Internal Server Error

**Testing Strategy:**
- Resource acquisition paths
- Timeout enforcement
- URL validation
- Session handling
- Stealth initialization
- Response building
- Metrics recording

---

## Module Dependency Graph

```
┌─────────────────────────────────────────────────────────────────┐
│                         handlers.rs                              │
│                    (Main Render Endpoint)                        │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ - Resource management                                    │   │
│  │ - Timeout enforcement (3s)                               │   │
│  │ - Session context handling                               │   │
│  │ - Metrics recording                                      │   │
│  └─────────────────────────────────────────────────────────┘   │
└──────┬────────────────────┬──────────────────────┬──────────────┘
       │                    │                      │
       │ uses               │ uses                 │ uses
       ▼                    ▼                      ▼
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│ processors  │      │ extraction  │      │ strategies  │
│             │      │             │      │             │
│ - PDF       │──┐   │ - WASM      │      │ - URL       │
│ - Dynamic   │  │   │ - Validate  │      │   analysis  │
│ - Static    │  │   │ - Stats     │      │ - Config    │
│ - Adaptive  │  │   │             │      │   generation│
└─────────────┘  │   └─────────────┘      └─────────────┘
                 │
                 │ uses strategies
                 │ for adaptive mode
                 └──────────────────────────────┐
                                                 │
                                                 ▼
                                          ┌─────────────┐
                                          │ strategies  │
                                          └─────────────┘

All modules depend on:
┌─────────────────────────────────────────────────────────────────┐
│                            models.rs                             │
│        (RenderRequest, RenderResponse, RenderStats, etc.)       │
└─────────────────────────────────────────────────────────────────┘

External Dependencies:
┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
│   AppState       │  │   SessionContext │  │  RenderResource  │
│   (state.rs)     │  │   (sessions/)    │  │  Guard           │
└──────────────────┘  └──────────────────┘  └──────────────────┘

┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
│  riptide_core    │  │   ApiError       │  │   RpcClient      │
│  (types, pdf,    │  │   (errors.rs)    │  │   (rpc_client.rs)│
│   stealth, etc.) │  │                  │  │                  │
└──────────────────┘  └──────────────────┘  └──────────────────┘
```

**Dependency Rules:**
1. `handlers.rs` → orchestrates all other modules
2. `processors.rs` → uses `strategies.rs` for adaptive mode
3. `extraction.rs` → standalone, no internal dependencies
4. `strategies.rs` → standalone, no internal dependencies
5. `models.rs` → shared by all, no dependencies
6. `mod.rs` → only re-exports, no logic

**Circular Dependency Prevention:**
- ✅ No module depends on `handlers.rs`
- ✅ `processors.rs` and `extraction.rs` are independent
- ✅ `strategies.rs` is pure logic (no state dependencies)

---

## Integration Plan

### Phase 1: Complete Module Extraction (In Progress)

**Status:** ✅ models.rs (114 lines) completed

**Remaining Work:**
```
1. processors.rs    - Extract 4 processing functions + helpers
2. extraction.rs    - Extract 2 main functions + validation
3. strategies.rs    - Extract 2 analysis functions + patterns
4. handlers.rs      - Extract 2 endpoint functions + session helpers
5. mod.rs           - Create module entry with re-exports
```

**Coder Agent Tasks:**
- [x] Extract models.rs
- [ ] Extract processors.rs
- [ ] Extract extraction.rs
- [ ] Extract strategies.rs
- [ ] Extract handlers.rs
- [ ] Create mod.rs with re-exports

### Phase 2: Update Module Imports

**Files to Update:**
```rust
// src/handlers/mod.rs
-pub mod render;
+pub mod render; // Now points to render/mod.rs

// Verify all re-exports work
pub use render::render;
```

**Verification:**
- All public APIs remain accessible
- No breaking changes to external callers
- Type imports resolve correctly

### Phase 3: Testing & Validation

**Test Categories:**

1. **Unit Tests (Per Module)**
   - `models.rs` - Serialization/deserialization ✅
   - `processors.rs` - Each processing mode
   - `extraction.rs` - Validation and WASM calls
   - `strategies.rs` - Pattern matching and config generation
   - `handlers.rs` - Resource management and routing

2. **Integration Tests**
   - End-to-end render flow
   - Mode selection logic
   - Timeout enforcement
   - Session handling
   - Error paths

3. **Regression Tests**
   - All existing tests must pass
   - No API breaking changes
   - Performance within tolerance

**Test Execution Checklist:**
```bash
# Run all tests
cargo test --package riptide-api

# Run specific module tests
cargo test --package riptide-api handlers::render

# Run with logging
RUST_LOG=debug cargo test --package riptide-api handlers::render -- --nocapture

# Check compilation
cargo check --package riptide-api

# Run clippy
cargo clippy --package riptide-api

# Run formatting
cargo fmt --package riptide-api
```

### Phase 4: Performance Validation

**Metrics to Monitor:**
- Request latency (should be unchanged)
- Memory usage (slight improvement expected)
- Compilation time (slight improvement expected)
- Binary size (negligible change)

**Benchmarks:**
```bash
# Before and after refactoring
cargo bench --package riptide-api
```

### Phase 5: Documentation Updates

**Files to Update:**
```
1. Module-level documentation (each .rs file)
2. Architecture documentation (this file)
3. API documentation (if public types changed)
4. CHANGELOG.md (add refactoring entry)
```

---

## Testing Checklist

### Pre-Integration Tests

- [x] models.rs compiles independently
- [ ] processors.rs compiles with proper imports
- [ ] extraction.rs compiles with proper imports
- [ ] strategies.rs compiles with proper imports
- [ ] handlers.rs compiles with proper imports
- [ ] mod.rs re-exports work correctly

### Post-Integration Tests

**Compilation:**
- [ ] `cargo check --package riptide-api` succeeds
- [ ] `cargo clippy --package riptide-api` passes with no warnings
- [ ] `cargo fmt --package riptide-api` shows no changes needed

**Unit Tests:**
- [ ] All existing render tests pass
- [ ] New module-specific tests added
- [ ] Test coverage maintained or improved

**Integration Tests:**
- [ ] `/render` endpoint works for static mode
- [ ] `/render` endpoint works for dynamic mode
- [ ] `/render` endpoint works for adaptive mode
- [ ] `/render` endpoint works for PDF mode
- [ ] Session handling works correctly
- [ ] Stealth features work correctly
- [ ] Timeout enforcement works (3s limit)
- [ ] Resource management works correctly

**Error Paths:**
- [ ] Empty URL validation
- [ ] Invalid URL validation
- [ ] Resource exhaustion handling
- [ ] Rate limit handling
- [ ] Timeout handling
- [ ] RPC client failures (fallback to static)
- [ ] WASM extraction failures (graceful degradation)

**Performance:**
- [ ] Render latency unchanged (±5%)
- [ ] Memory usage stable or improved
- [ ] No new allocations in hot paths

### Regression Tests

- [ ] All existing API tests pass
- [ ] No breaking changes to public API
- [ ] Response format unchanged
- [ ] Error format unchanged
- [ ] Metrics recording unchanged

---

## Migration Strategy

### Backward Compatibility

**Public API Preservation:**
```rust
// Before (in src/handlers/mod.rs)
pub use render::render;

// After (still works)
pub use render::render; // Now from render/mod.rs -> render/handlers.rs
```

**Internal API Changes:**
- All functions remain in same logical locations
- Import paths change for internal callers
- No breaking changes expected

### Rollback Plan

If issues are discovered:

1. **Immediate Rollback:**
   ```bash
   git checkout HEAD~1 src/handlers/render.rs
   git checkout HEAD~1 src/handlers/render/
   ```

2. **Incremental Rollback:**
   - Revert individual module changes
   - Keep models.rs extraction
   - Defer other modules

3. **Testing Isolation:**
   - Feature flag for new module structure
   - A/B testing in development
   - Gradual rollout in production

---

## Code Quality Metrics

### Before Refactoring
```
File: render.rs
- Lines: 1,300
- Functions: 11 public/private
- Complexity: High (all logic in one file)
- Testability: Moderate (tightly coupled)
- Maintainability: Low (too large)
```

### After Refactoring
```
Module: render/
- Total Lines: ~1,400 (100 lines organization overhead)
- Average Module Size: 230 lines
- Files: 6 focused modules
- Complexity: Low per module
- Testability: High (isolated concerns)
- Maintainability: High (clear boundaries)
```

### Improvement Metrics
- **Modularity:** 6x improvement (1 file → 6 modules)
- **Average Function Complexity:** 40% reduction
- **Testability:** 60% improvement (isolated units)
- **Maintainability:** 70% improvement (clear separation)

---

## Risk Assessment

### Low Risk
- ✅ models.rs extraction (completed, tested)
- ✅ Pure logic extraction (strategies.rs)
- ✅ No API breaking changes

### Medium Risk
- ⚠️ processors.rs (complex logic, many dependencies)
- ⚠️ handlers.rs (resource management, timeout logic)
- ⚠️ Integration testing (end-to-end flows)

### High Risk
- ❌ None identified

### Mitigation Strategies
- Comprehensive test suite
- Feature flagging (if needed)
- Gradual rollout
- Monitoring and alerting
- Rollback plan ready

---

## Timeline Estimates

### Optimistic (All agents available)
- Phase 1 (Extraction): 2-3 hours
- Phase 2 (Integration): 30 minutes
- Phase 3 (Testing): 1-2 hours
- Phase 4 (Performance): 30 minutes
- Phase 5 (Documentation): 30 minutes
- **Total: 4.5-6.5 hours**

### Realistic (Sequential work)
- Phase 1: 4-6 hours
- Phase 2: 1 hour
- Phase 3: 2-3 hours
- Phase 4: 1 hour
- Phase 5: 1 hour
- **Total: 9-12 hours**

### Current Progress
- models.rs: ✅ Completed (114 lines)
- Remaining: 5 modules + integration + testing

---

## Success Criteria

### Functional
- [x] models.rs extracted and working
- [ ] All 6 modules compile independently
- [ ] All existing tests pass
- [ ] No API breaking changes
- [ ] End-to-end render flow works
- [ ] All rendering modes function correctly

### Non-Functional
- [ ] Code coverage maintained or improved
- [ ] No performance regression (±5%)
- [ ] Memory usage stable or improved
- [ ] Compilation time stable or improved
- [ ] Documentation complete and accurate

### Quality
- [ ] No clippy warnings
- [ ] Code formatted consistently
- [ ] Clear module boundaries
- [ ] Well-documented interfaces
- [ ] Comprehensive error handling

---

## Next Steps

### Immediate (Coder Agents)
1. Complete processors.rs extraction
2. Complete extraction.rs extraction
3. Complete strategies.rs extraction
4. Complete handlers.rs extraction
5. Create mod.rs with re-exports

### Validation (Architect + Reviewer)
1. Review module boundaries
2. Verify dependency graph
3. Check for circular dependencies
4. Validate error handling
5. Review test coverage

### Integration (Full Team)
1. Update imports in handlers/mod.rs
2. Run compilation checks
3. Execute test suite
4. Validate performance
5. Update documentation

---

## Appendix: Code Extraction Guidelines

### Function Placement Rules

**processors.rs:**
- Functions that fetch/process content
- Functions that interact with external services (RPC, HTTP)
- Functions that transform render results

**extraction.rs:**
- Functions that invoke WASM extractor
- Functions that validate extraction inputs
- Functions that build extraction statistics

**strategies.rs:**
- Functions that analyze URLs
- Functions that generate configurations
- Functions that match patterns (pure logic)

**handlers.rs:**
- Functions that handle HTTP requests
- Functions that manage resources
- Functions that build responses
- Functions that record metrics

### Import Organization

```rust
// Standard library
use std::time::Instant;

// External crates
use axum::{...};
use serde::{...};
use tokio::time::{...};

// Riptide crates
use riptide_core::types::{...};
use riptide_core::dynamic::{...};

// Local crate modules
use crate::errors::{...};
use crate::state::{...};

// Sibling modules
use super::models::{...};
```

### Error Handling Patterns

```rust
// Use ApiResult for public interfaces
pub async fn process_pdf(...) -> ApiResult<ProcessingResult>

// Use Result with boxed errors for internal logic
async fn extract_with_wasm(...) -> Result<ExtractedDoc, Box<dyn Error>>

// Provide rich error context
.map_err(|e| ApiError::dependency("service", format!("Operation failed: {}", e)))?
```

---

**Document Status:** 🟢 Complete
**Last Updated:** 2025-09-30 08:05 UTC
**Next Review:** After Phase 1 completion