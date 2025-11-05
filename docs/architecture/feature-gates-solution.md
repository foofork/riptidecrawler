# Feature Gates Architecture Solution for riptide-api

**Status**: Design Complete - Ready for Implementation
**Date**: 2025-11-04
**Errors Resolved**: 23 compilation errors
**Implementation Pattern**: Option C (Conditional Compilation with Stubs)

---

## Executive Summary

This document provides the complete architectural design for resolving 23 compilation errors in `riptide-api` through a systematic feature gate solution using conditional compilation (`#[cfg(feature = "...")]`), stub implementations, and clean error messages for disabled features.

---

## Error Analysis

### Error Categories (23 Total)

#### Category 1: Missing `riptide_headless` (12 errors)
**Root Cause**: `riptide-headless` crate not available, but code imports and uses it unconditionally.

**Affected Files**:
1. `src/state.rs` (lines 34, 896-897, 919, 933-934, 956, 992, 1005, 1487-1489)
2. `src/resource_manager/mod.rs` (line 87)
3. `src/resource_manager/guards.rs` (line 9)
4. `src/rpc_client.rs` (lines 3, 358, 364, 393)
5. `src/handlers/stealth.rs` (lines 58, 108, 194)

**Solution**: Wrap all `riptide_headless` imports and usages with `#[cfg(feature = "browser")]`.

#### Category 2: Missing `riptide_intelligence` (6 errors)
**Root Cause**: `riptide-intelligence` crate not available, but imported in handlers.

**Affected Files**:
1. `src/handlers/llm.rs` (line 14)
2. `src/handlers/profiles.rs` (line 16)
3. `src/pipeline.rs` (lines 6, 700, 704, 716, 720, 731, 740, 743)
4. `src/routes/llm.rs` (line 3)
5. `src/routes/profiles.rs` (line 8)

**Solution**: Wrap with `#[cfg(feature = "llm")]` and provide feature-disabled stubs.

#### Category 3: Feature-Gated Import Conflicts (2 errors)
**Root Cause**: Routes import handlers that don't exist when features are disabled.

**Affected Files**:
1. `src/routes/llm.rs` - imports `crate::handlers::llm`
2. `src/routes/profiles.rs` - imports `crate::handlers::profiles`

**Solution**: Feature-gate the entire route module or provide stub routes.

#### Category 4: Missing AppState Fields (3 errors)
**Root Cause**: AppState fields accessed unconditionally that are feature-gated.

**Affected Fields**:
- `browser_facade` (3 references in `stealth.rs`)
- `worker_service` (2 references in `telemetry.rs`, `state.rs`)

**Solution**: Conditional field access with runtime error messages.

---

## Architecture Design: Option C Implementation

### Core Principles

1. **Conditional Compilation**: Use `#[cfg(feature = "...")]` for all optional dependencies
2. **Graceful Degradation**: Provide helpful error messages when features are disabled
3. **Type Safety**: Use stub types to maintain API signatures
4. **Zero Runtime Overhead**: Feature-disabled code completely eliminated at compile time
5. **Backward Compatible**: Existing code works with features enabled

### Feature Hierarchy

```
riptide-api features:
├── browser (requires: riptide-headless, riptide-browser)
│   ├── BrowserFacade
│   ├── HeadlessLauncher
│   ├── BrowserPool
│   └── Routes: /stealth/*
├── llm (requires: riptide-intelligence)
│   ├── LlmRegistry
│   ├── ProviderConfig
│   └── Routes: /llm/*
├── workers (requires: riptide-workers)
│   └── WorkerService
└── search (requires: riptide-search)
    └── SearchFacade
```

---

## Implementation Strategy

### Phase 1: Cargo.toml Feature Verification

**Status**: ✅ Already Correct

The Cargo.toml already has correct feature gates:
```toml
[features]
browser = ["dep:riptide-browser", "dep:riptide-headless"]
llm = ["dep:riptide-intelligence"]
workers = ["dep:riptide-workers"]
search = ["dep:riptide-search"]
```

**Action**: No changes needed in Cargo.toml.

---

### Phase 2: AppState Conditional Fields

**File**: `src/state.rs`

**Current State**:
- Fields are already conditionally included with `#[cfg(feature = "...")]`
- Problem: Fields are accessed unconditionally in other modules

**Solution**: Add feature-gated accessor methods

#### Implementation Pattern

```rust
// In AppState implementation
impl AppState {
    /// Get browser facade if browser feature is enabled
    pub fn browser_facade(&self) -> Result<&BrowserFacade, ApiError> {
        #[cfg(feature = "browser")]
        {
            self.browser_facade.as_ref().ok_or_else(|| {
                ApiError::feature_disabled(
                    "browser",
                    "BrowserFacade is not enabled. Enable with feature 'browser' in Cargo.toml"
                )
            })
        }

        #[cfg(not(feature = "browser"))]
        {
            Err(ApiError::feature_disabled(
                "browser",
                "Browser features are disabled. Rebuild with --features browser"
            ))
        }
    }

    /// Get worker service if workers feature is enabled
    pub fn worker_service(&self) -> Result<&WorkerService, ApiError> {
        #[cfg(feature = "workers")]
        {
            Ok(&self.worker_service)
        }

        #[cfg(not(feature = "workers"))]
        {
            Err(ApiError::feature_disabled(
                "workers",
                "Worker features are disabled. Rebuild with --features workers"
            ))
        }
    }
}
```

**Files to Update**:
1. `src/state.rs` - Add accessor methods
2. `src/handlers/stealth.rs` - Use `state.browser_facade()?`
3. `src/handlers/telemetry.rs` - Use `state.worker_service()?`

---

### Phase 3: Handler Module Feature Gates

#### 3.1: LLM Handler (`src/handlers/llm.rs`)

**Strategy**: Feature-gate entire module

```rust
// At top of handlers/llm.rs
#![cfg(feature = "llm")]

//! LLM provider management API handlers
//!
//! **Requires feature**: `llm`
//!
//! This module is only available when compiled with the `llm` feature flag.
```

**Also Update**: `src/handlers/mod.rs`

```rust
// In handlers/mod.rs
#[cfg(feature = "llm")]
pub mod llm;
```

#### 3.2: Profiles Handler (`src/handlers/profiles.rs`)

**Strategy**: Feature-gate entire module (profiles depends on intelligence)

```rust
// At top of handlers/profiles.rs
#![cfg(feature = "llm")]

//! Domain Profile Management API Handlers
//!
//! **Requires feature**: `llm`
```

**Also Update**: `src/handlers/mod.rs`

```rust
#[cfg(feature = "llm")]
pub mod profiles;
```

---

### Phase 4: Route Module Feature Gates

#### 4.1: LLM Routes (`src/routes/llm.rs`)

**Strategy**: Feature-gate module and provide stub when disabled

**Option A - Feature Gate Entire Module**:
```rust
// At top of routes/llm.rs
#![cfg(feature = "llm")]

use crate::handlers::llm;
// ... rest of module
```

**Option B - Provide Stub Routes (Better UX)**:
```rust
// routes/llm.rs
#[cfg(feature = "llm")]
use crate::handlers::llm;

#[cfg(feature = "llm")]
pub fn llm_routes() -> Router<AppState> {
    Router::new()
        .route("/providers", get(llm::list_providers))
        .route("/switch", post(llm::switch_provider))
}

#[cfg(not(feature = "llm"))]
pub fn llm_routes() -> Router<AppState> {
    use axum::{routing::any, Router};
    use crate::errors::ApiError;

    Router::new()
        .route("/*path", any(|| async {
            Err::<(), _>(ApiError::feature_disabled(
                "llm",
                "LLM features are disabled. Rebuild with --features llm to enable /llm/* endpoints"
            ))
        }))
}
```

**Recommendation**: Use Option B for better API discoverability.

#### 4.2: Profiles Routes (`src/routes/profiles.rs`)

Same pattern as LLM routes:

```rust
#[cfg(feature = "llm")]
use crate::handlers::profiles;

#[cfg(feature = "llm")]
pub fn profile_routes() -> Router<AppState> {
    // Real routes
}

#[cfg(not(feature = "llm"))]
pub fn profile_routes() -> Router<AppState> {
    use axum::{routing::any, Router};
    Router::new()
        .route("/*path", any(|| async {
            Err::<(), _>(ApiError::feature_disabled(
                "llm",
                "Profile features are disabled. Rebuild with --features llm"
            ))
        }))
}
```

---

### Phase 5: Pipeline Module Intelligence Dependencies

**File**: `src/pipeline.rs`

**Problem**: Imports `riptide_intelligence::smart_retry` types and uses intelligence errors.

**Strategy**: Feature-gate retry logic and provide fallback

```rust
// Top of pipeline.rs
#[cfg(feature = "llm")]
use riptide_intelligence::smart_retry::{RetryConfig, SmartRetry, SmartRetryStrategy};

// In retry logic section
#[cfg(feature = "llm")]
fn create_smart_retry() -> SmartRetry {
    // Full smart retry implementation
}

#[cfg(not(feature = "llm"))]
fn create_smart_retry() -> SimpleRetry {
    // Basic retry without intelligence
    SimpleRetry::default()
}

// Error conversion
fn convert_fetch_error(e: reqwest::Error) -> ApiError {
    #[cfg(feature = "llm")]
    {
        // Use intelligence error types
        ApiError::from(riptide_intelligence::IntelligenceError::Network(e.to_string()))
    }

    #[cfg(not(feature = "llm"))]
    {
        // Direct conversion
        ApiError::fetch("unknown", e.to_string())
    }
}
```

---

### Phase 6: RPC Client Headless Dependencies

**File**: `src/rpc_client.rs`

**Problem**: Imports and uses `riptide_headless::dynamic` types.

**Strategy**: Feature-gate dynamic rendering

```rust
// Top of rpc_client.rs
#[cfg(feature = "browser")]
use riptide_headless::dynamic::{
    DynamicConfig, DynamicRenderResult, PageAction,
    RenderArtifacts, WaitCondition, PageMetadata
};

// Stub types when browser feature is disabled
#[cfg(not(feature = "browser"))]
mod dynamic_stubs {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct DynamicConfig {
        // Stub fields
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct DynamicRenderResult {
        // Stub fields
    }

    // ... other stub types
}

#[cfg(not(feature = "browser"))]
use dynamic_stubs::*;

// In implementation
pub async fn render_dynamic(&self, config: DynamicConfig) -> Result<DynamicRenderResult> {
    #[cfg(feature = "browser")]
    {
        // Real implementation
    }

    #[cfg(not(feature = "browser"))]
    {
        Err(ApiError::feature_disabled(
            "browser",
            "Dynamic rendering requires browser feature"
        ))
    }
}
```

---

### Phase 7: Resource Manager Browser Dependencies

**Files**:
- `src/resource_manager/mod.rs`
- `src/resource_manager/guards.rs`

**Strategy**: Feature-gate browser pool management

```rust
// resource_manager/mod.rs
#[cfg(feature = "browser")]
use riptide_headless::pool::{BrowserPool, BrowserPoolConfig};

#[cfg(feature = "browser")]
pub struct BrowserPoolManager {
    pool: Arc<BrowserPool>,
}

#[cfg(not(feature = "browser"))]
pub struct BrowserPoolManager;

impl ResourceManager {
    #[cfg(feature = "browser")]
    pub async fn get_browser(&self) -> Result<BrowserCheckout> {
        // Real implementation
    }

    #[cfg(not(feature = "browser"))]
    pub async fn get_browser(&self) -> Result<()> {
        Err(ApiError::feature_disabled("browser", "Browser pool unavailable"))
    }
}
```

```rust
// resource_manager/guards.rs
#[cfg(feature = "browser")]
use riptide_headless::pool::BrowserCheckout;

#[cfg(feature = "browser")]
pub struct BrowserGuard {
    checkout: BrowserCheckout,
}

#[cfg(not(feature = "browser"))]
pub struct BrowserGuard;

impl BrowserGuard {
    #[cfg(not(feature = "browser"))]
    pub fn new() -> Self {
        unreachable!("BrowserGuard cannot be constructed without browser feature")
    }
}
```

---

## Error Message System

### New ApiError Variant

Add to `src/errors.rs`:

```rust
#[derive(Debug, Clone, Serialize, thiserror::Error)]
pub enum ApiError {
    // ... existing variants

    #[error("Feature '{feature}' is disabled: {message}")]
    FeatureDisabled {
        feature: String,
        message: String,
    },
}

impl ApiError {
    /// Create a feature disabled error with helpful message
    pub fn feature_disabled(feature: impl Into<String>, message: impl Into<String>) -> Self {
        Self::FeatureDisabled {
            feature: feature.into(),
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            // ... existing matches

            Self::FeatureDisabled { ref feature, ref message } => (
                StatusCode::NOT_IMPLEMENTED,
                json!({
                    "error": "feature_disabled",
                    "feature": feature,
                    "message": message,
                    "documentation": format!(
                        "https://github.com/your-org/riptide/blob/main/docs/features/{}.md",
                        feature
                    ),
                })
            ),
        };

        (status, Json(error_message)).into_response()
    }
}
```

---

## Feature-Specific Error Messages

### Browser Feature
```json
{
  "error": "feature_disabled",
  "feature": "browser",
  "message": "Browser automation features are disabled. Rebuild with --features browser to enable /stealth/*, /render/dynamic, and headless browser functionality.",
  "documentation": "https://github.com/your-org/riptide/blob/main/docs/features/browser.md"
}
```

### LLM Feature
```json
{
  "error": "feature_disabled",
  "feature": "llm",
  "message": "LLM provider features are disabled. Rebuild with --features llm to enable /llm/*, /profiles/*, and intelligent content analysis.",
  "documentation": "https://github.com/your-org/riptide/blob/main/docs/features/llm.md"
}
```

### Workers Feature
```json
{
  "error": "feature_disabled",
  "feature": "workers",
  "message": "Background worker features are disabled. Rebuild with --features workers to enable job scheduling and async processing.",
  "documentation": "https://github.com/your-org/riptide/blob/main/docs/features/workers.md"
}
```

---

## Implementation Checklist

### File-by-File Tasks

#### High Priority (Compilation Blockers)

1. **src/errors.rs**
   - [ ] Add `FeatureDisabled` variant
   - [ ] Add `feature_disabled()` helper
   - [ ] Add HTTP 501 response for feature-disabled errors

2. **src/state.rs**
   - [ ] Add `browser_facade()` accessor method
   - [ ] Add `worker_service()` accessor method
   - [ ] Update imports with `#[cfg(feature = "browser")]`

3. **src/handlers/llm.rs**
   - [ ] Add `#![cfg(feature = "llm")]` at top
   - [ ] Verify all imports are feature-gated

4. **src/handlers/profiles.rs**
   - [ ] Add `#![cfg(feature = "llm")]` at top
   - [ ] Verify all imports are feature-gated

5. **src/handlers/mod.rs**
   - [ ] Add `#[cfg(feature = "llm")]` before `pub mod llm;`
   - [ ] Add `#[cfg(feature = "llm")]` before `pub mod profiles;`

6. **src/routes/llm.rs**
   - [ ] Add `#[cfg(feature = "llm")]` for imports
   - [ ] Provide stub `llm_routes()` when feature disabled

7. **src/routes/profiles.rs**
   - [ ] Add `#[cfg(feature = "llm")]` for imports
   - [ ] Provide stub `profile_routes()` when feature disabled

8. **src/pipeline.rs**
   - [ ] Add `#[cfg(feature = "llm")]` for intelligence imports
   - [ ] Create fallback retry logic without intelligence
   - [ ] Update error conversion logic

9. **src/rpc_client.rs**
   - [ ] Add `#[cfg(feature = "browser")]` for headless imports
   - [ ] Create stub types for dynamic rendering
   - [ ] Feature-gate render methods

10. **src/resource_manager/mod.rs**
    - [ ] Add `#[cfg(feature = "browser")]` for pool imports
    - [ ] Feature-gate browser pool methods
    - [ ] Add stub implementation

11. **src/resource_manager/guards.rs**
    - [ ] Add `#[cfg(feature = "browser")]` for checkout import
    - [ ] Feature-gate BrowserGuard implementation

12. **src/handlers/stealth.rs**
    - [ ] Replace `state.browser_facade` with `state.browser_facade()?`
    - [ ] Update all 3 occurrences

13. **src/handlers/telemetry.rs**
    - [ ] Replace `state.worker_service` with `state.worker_service()?`

#### Medium Priority (Cleanup)

14. **src/handlers/mod.rs**
    - [ ] Add feature documentation comments
    - [ ] Organize feature-gated modules

15. **src/routes/mod.rs**
    - [ ] Add feature documentation
    - [ ] Document available routes per feature

#### Low Priority (Documentation)

16. **docs/features/browser.md**
    - [ ] Document browser feature requirements
    - [ ] List available endpoints
    - [ ] Show example usage

17. **docs/features/llm.md**
    - [ ] Document LLM feature requirements
    - [ ] List available endpoints
    - [ ] Show provider configuration

18. **docs/features/workers.md**
    - [ ] Document worker feature requirements
    - [ ] Explain job scheduling
    - [ ] Show example usage

19. **README.md**
    - [ ] Update feature flags section
    - [ ] Add feature matrix table
    - [ ] Document build examples

---

## Code Templates for Agents

### Template 1: Feature-Gated Handler Module

```rust
// File: src/handlers/FEATURE_NAME.rs
#![cfg(feature = "FEATURE_FLAG")]

//! FEATURE_NAME Handler Module
//!
//! **Requires feature**: `FEATURE_FLAG`
//!
//! This module provides API endpoints for FEATURE_DESCRIPTION.
//! It is only available when compiled with the `FEATURE_FLAG` feature flag.
//!
//! # Usage
//!
//! Enable in Cargo.toml:
//! ```toml
//! riptide-api = { version = "0.9", features = ["FEATURE_FLAG"] }
//! ```
//!
//! Or build with:
//! ```bash
//! cargo build --features FEATURE_FLAG
//! ```

use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;
use axum::{/* ... */};

// Handler implementations...
```

### Template 2: Feature-Gated Route with Stub

```rust
// File: src/routes/FEATURE_NAME.rs

#[cfg(feature = "FEATURE_FLAG")]
use crate::handlers::FEATURE_NAME;

#[cfg(feature = "FEATURE_FLAG")]
pub fn FEATURE_routes() -> Router<AppState> {
    Router::new()
        .route("/path", get(FEATURE_NAME::handler))
        // ... more routes
}

#[cfg(not(feature = "FEATURE_FLAG"))]
pub fn FEATURE_routes() -> Router<AppState> {
    use axum::{routing::any, Router};

    Router::new()
        .route("/*path", any(|| async {
            Err::<(), ApiError>(ApiError::feature_disabled(
                "FEATURE_FLAG",
                "FEATURE_NAME features are disabled. Rebuild with --features FEATURE_FLAG"
            ))
        }))
}
```

### Template 3: Feature-Gated Struct Field Access

```rust
impl AppState {
    /// Get FIELD_NAME if FEATURE_FLAG feature is enabled
    pub fn FIELD_NAME(&self) -> Result<&TYPE, ApiError> {
        #[cfg(feature = "FEATURE_FLAG")]
        {
            self.FIELD_NAME.as_ref().ok_or_else(|| {
                ApiError::feature_disabled(
                    "FEATURE_FLAG",
                    "FIELD_NAME is not enabled. Enable with feature 'FEATURE_FLAG'"
                )
            })
        }

        #[cfg(not(feature = "FEATURE_FLAG"))]
        {
            Err(ApiError::feature_disabled(
                "FEATURE_FLAG",
                "FIELD_NAME requires FEATURE_FLAG feature. Rebuild with --features FEATURE_FLAG"
            ))
        }
    }
}
```

### Template 4: Conditional Import with Stub

```rust
// Real type when feature enabled
#[cfg(feature = "FEATURE_FLAG")]
use external_crate::{RealType, RealTrait};

// Stub type when feature disabled
#[cfg(not(feature = "FEATURE_FLAG"))]
mod stubs {
    pub struct RealType;
    pub trait RealTrait {}
}

#[cfg(not(feature = "FEATURE_FLAG"))]
use stubs::*;

// Now RealType is available in both cases
```

---

## Testing Strategy

### Compilation Tests

```bash
# Test with no optional features
cargo build -p riptide-api --no-default-features

# Test with each feature individually
cargo build -p riptide-api --features browser
cargo build -p riptide-api --features llm
cargo build -p riptide-api --features workers

# Test with all features
cargo build -p riptide-api --all-features

# Test default features
cargo build -p riptide-api
```

### Runtime Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(not(feature = "browser"))]
    async fn test_browser_feature_disabled() {
        let state = AppState::new_test_minimal().await;
        let result = state.browser_facade();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApiError::FeatureDisabled { feature, .. } if feature == "browser"
        ));
    }

    #[tokio::test]
    #[cfg(feature = "browser")]
    async fn test_browser_feature_enabled() {
        let state = AppState::new_test_minimal().await;
        let result = state.browser_facade();

        // Should work with feature enabled
        assert!(result.is_ok());
    }
}
```

---

## Performance Impact

### Compile-Time
- **Zero runtime overhead**: Feature-disabled code is completely eliminated
- **Faster compilation**: Only builds enabled features
- **Smaller binary**: Unused dependencies not linked

### Runtime
- **No performance impact**: Feature checks happen at compile time
- **No conditional branches**: Code paths don't exist when disabled
- **Optimal for production**: Enable only needed features

---

## Migration Guide

### For Developers

**Before** (unconditional access):
```rust
let facade = state.browser_facade.as_ref().unwrap();
```

**After** (feature-safe access):
```rust
let facade = state.browser_facade()?;
```

### For Users

**Building with specific features**:
```bash
# Minimal build (fastest, smallest)
cargo build -p riptide-api --no-default-features --features spider,fetch

# Production build with browser automation
cargo build -p riptide-api --features browser

# Full-featured development build
cargo build -p riptide-api --all-features
```

---

## Success Criteria

- [ ] All 23 compilation errors resolved
- [ ] Clean compilation with `--no-default-features`
- [ ] Clean compilation with each feature individually
- [ ] Clean compilation with `--all-features`
- [ ] Helpful error messages when features disabled
- [ ] Zero clippy warnings
- [ ] Tests pass for all feature combinations
- [ ] Documentation complete

---

## Next Steps for Implementation Agents

1. **Agent 1: Error System** - Implement `FeatureDisabled` in `errors.rs`
2. **Agent 2: State Accessors** - Add accessor methods to `AppState`
3. **Agent 3: Handler Gates** - Feature-gate `llm.rs` and `profiles.rs`
4. **Agent 4: Route Stubs** - Add stub routes for disabled features
5. **Agent 5: Pipeline Logic** - Add fallback retry without intelligence
6. **Agent 6: RPC Stubs** - Create stubs for dynamic rendering
7. **Agent 7: Resource Manager** - Feature-gate browser pool
8. **Agent 8: Stealth Updates** - Update field access patterns
9. **Agent 9: Testing** - Add feature-gated tests
10. **Agent 10: Documentation** - Create feature docs

Each agent should work independently on their assigned files, using this architecture document as the specification.

---

## Appendix: Feature Dependency Graph

```
riptide-api
├── riptide-types (always required)
├── riptide-cache (always required)
├── riptide-config (always required)
├── riptide-performance (always required)
│
├── [spider] feature
│   └── riptide-spider
│       ├── riptide-fetch
│       └── riptide-extraction
│
├── [browser] feature
│   ├── riptide-browser
│   └── riptide-headless ⚠️ (not available - causes 12 errors)
│
├── [llm] feature
│   └── riptide-intelligence ⚠️ (not available - causes 6 errors)
│
├── [workers] feature
│   └── riptide-workers (causes 3 errors)
│
└── [search] feature
    └── riptide-search
```

**Legend**:
- ✅ Always available
- ⚠️ Currently unavailable - needs feature gates
- 🔧 Optional feature

---

**Document Version**: 1.0
**Last Updated**: 2025-11-04
**Author**: System Architecture Designer
**Review Status**: Ready for Implementation
