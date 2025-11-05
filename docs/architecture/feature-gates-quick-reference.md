# Feature Gates Quick Reference Guide

**For Implementation Agents**

This is a condensed reference for implementing the feature gates solution. See `feature-gates-solution.md` for full architecture details.

---

## 🎯 Quick Start

### Error Count by Category
- **riptide_headless missing**: 12 errors
- **riptide_intelligence missing**: 6 errors
- **Feature-gated imports**: 2 errors
- **Missing AppState fields**: 3 errors
- **TOTAL**: 23 errors

### Implementation Order
1. errors.rs → Add FeatureDisabled variant
2. state.rs → Add accessor methods
3. handlers/ → Feature-gate modules
4. routes/ → Add stub routes
5. pipeline.rs → Fallback logic
6. rpc_client.rs → Stub types
7. resource_manager/ → Feature-gate pools

---

## 📋 Code Snippets

### 1. Error Variant (errors.rs)

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
    pub fn feature_disabled(feature: impl Into<String>, message: impl Into<String>) -> Self {
        Self::FeatureDisabled {
            feature: feature.into(),
            message: message.into(),
        }
    }
}

// Add to IntoResponse
Self::FeatureDisabled { ref feature, ref message } => (
    StatusCode::NOT_IMPLEMENTED,
    json!({
        "error": "feature_disabled",
        "feature": feature,
        "message": message,
    })
),
```

### 2. AppState Accessors (state.rs)

```rust
impl AppState {
    /// Browser facade accessor
    pub fn browser_facade(&self) -> Result<&BrowserFacade, ApiError> {
        #[cfg(feature = "browser")]
        {
            self.browser_facade.as_ref().ok_or_else(|| {
                ApiError::feature_disabled(
                    "browser",
                    "BrowserFacade not enabled. Rebuild with --features browser"
                )
            })
        }
        #[cfg(not(feature = "browser"))]
        {
            Err(ApiError::feature_disabled(
                "browser",
                "Browser features disabled. Rebuild with --features browser"
            ))
        }
    }

    /// Worker service accessor
    pub fn worker_service(&self) -> Result<&WorkerService, ApiError> {
        #[cfg(feature = "workers")]
        { Ok(&self.worker_service) }
        #[cfg(not(feature = "workers"))]
        {
            Err(ApiError::feature_disabled(
                "workers",
                "Worker features disabled. Rebuild with --features workers"
            ))
        }
    }
}
```

### 3. Feature-Gate Handler Module

```rust
// At TOP of handlers/llm.rs and handlers/profiles.rs
#![cfg(feature = "llm")]

//! Module documentation
//!
//! **Requires feature**: `llm`
```

```rust
// In handlers/mod.rs
#[cfg(feature = "llm")]
pub mod llm;

#[cfg(feature = "llm")]
pub mod profiles;
```

### 4. Stub Routes

```rust
// routes/llm.rs
#[cfg(feature = "llm")]
use crate::handlers::llm;

#[cfg(feature = "llm")]
pub fn llm_routes() -> Router<AppState> {
    Router::new()
        .route("/providers", get(llm::list_providers))
}

#[cfg(not(feature = "llm"))]
pub fn llm_routes() -> Router<AppState> {
    use axum::{routing::any, Router};
    Router::new().route("/*path", any(|| async {
        Err::<(), ApiError>(ApiError::feature_disabled(
            "llm",
            "LLM features disabled. Rebuild with --features llm"
        ))
    }))
}
```

### 5. Pipeline Intelligence Fallback

```rust
// pipeline.rs - top imports
#[cfg(feature = "llm")]
use riptide_intelligence::smart_retry::{RetryConfig, SmartRetry};

// Error conversion
fn convert_error(e: reqwest::Error, url: &str) -> ApiError {
    #[cfg(feature = "llm")]
    {
        ApiError::from(riptide_intelligence::IntelligenceError::Network(e.to_string()))
    }
    #[cfg(not(feature = "llm"))]
    {
        ApiError::fetch(url, e.to_string())
    }
}
```

### 6. RPC Client Stubs

```rust
// rpc_client.rs - top
#[cfg(feature = "browser")]
use riptide_headless::dynamic::{DynamicConfig, DynamicRenderResult};

#[cfg(not(feature = "browser"))]
mod dynamic_stubs {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct DynamicConfig {}

    #[derive(Debug, Clone, Serialize)]
    pub struct DynamicRenderResult {}
}

#[cfg(not(feature = "browser"))]
use dynamic_stubs::*;

// In methods
pub async fn render_dynamic(&self, config: DynamicConfig) -> Result<DynamicRenderResult> {
    #[cfg(feature = "browser")]
    {
        // Real implementation
    }
    #[cfg(not(feature = "browser"))]
    {
        Err(ApiError::feature_disabled("browser", "Dynamic rendering unavailable"))
    }
}
```

### 7. Resource Manager Browser Pool

```rust
// resource_manager/mod.rs
#[cfg(feature = "browser")]
use riptide_headless::pool::{BrowserPool, BrowserPoolConfig};

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
```

### 8. Update Field Access

```rust
// handlers/stealth.rs - Replace all 3 occurrences
// OLD:
let facade = state.browser_facade.as_ref().ok_or_else(|| ...)?;

// NEW:
let facade = state.browser_facade()?;
```

```rust
// handlers/telemetry.rs - Replace 1 occurrence
// OLD:
let worker_health = state.worker_service.health_check().await;

// NEW:
let worker_health = state.worker_service()?.health_check().await;
```

---

## 🔍 Files to Modify

### Priority 1 (Compilation Blockers)
- ✅ `src/errors.rs` - Add FeatureDisabled
- ✅ `src/state.rs` - Add accessors
- ✅ `src/handlers/llm.rs` - Add `#![cfg(feature = "llm")]`
- ✅ `src/handlers/profiles.rs` - Add `#![cfg(feature = "llm")]`
- ✅ `src/handlers/mod.rs` - Gate module exports
- ✅ `src/routes/llm.rs` - Stub routes
- ✅ `src/routes/profiles.rs` - Stub routes
- ✅ `src/pipeline.rs` - Intelligence fallback
- ✅ `src/rpc_client.rs` - Headless stubs
- ✅ `src/resource_manager/mod.rs` - Browser pool gates
- ✅ `src/resource_manager/guards.rs` - Guard stubs
- ✅ `src/handlers/stealth.rs` - Update access (3×)
- ✅ `src/handlers/telemetry.rs` - Update access (1×)

### Priority 2 (Cleanup)
- 🔧 `src/handlers/mod.rs` - Add docs
- 🔧 `src/routes/mod.rs` - Add docs

### Priority 3 (Documentation)
- 📄 `docs/features/browser.md`
- 📄 `docs/features/llm.md`
- 📄 `docs/features/workers.md`
- 📄 `README.md` - Feature matrix

---

## ✅ Verification Commands

```bash
# Test minimal build (no optional features)
cargo build -p riptide-api --no-default-features

# Test each feature individually
cargo build -p riptide-api --no-default-features --features spider
cargo build -p riptide-api --no-default-features --features browser
cargo build -p riptide-api --no-default-features --features llm
cargo build -p riptide-api --no-default-features --features workers

# Test default features
cargo build -p riptide-api

# Test all features
cargo build -p riptide-api --all-features

# Check for warnings
RUSTFLAGS="-D warnings" cargo build -p riptide-api --no-default-features
cargo clippy -p riptide-api --all-features -- -D warnings
```

---

## 🚀 Expected Outcomes

### Before (23 errors)
```
error[E0433]: failed to resolve: use of unresolved module `riptide_headless` (×12)
error[E0433]: failed to resolve: use of unresolved module `riptide_intelligence` (×6)
error[E0432]: unresolved import `crate::handlers::llm` (×2)
error[E0609]: no field `browser_facade` on type `AppState` (×3)
```

### After (0 errors)
```
✅ Compiling riptide-api v0.9.0
✅ Finished dev [unoptimized + debuginfo] target(s)
```

---

## 📊 Feature Matrix

| Feature | Crate | Endpoints | Default |
|---------|-------|-----------|---------|
| `spider` | riptide-spider | /spider/* | ✅ Yes |
| `extraction` | riptide-extraction | /extract/* | ✅ Yes |
| `fetch` | riptide-fetch | /fetch/* | ✅ Yes |
| `browser` | riptide-headless | /stealth/*, /render/dynamic | ❌ No |
| `llm` | riptide-intelligence | /llm/*, /profiles/* | ❌ No |
| `workers` | riptide-workers | background jobs | ❌ No |
| `search` | riptide-search | /search/* | ❌ No |

---

## 🐛 Common Pitfalls

### ❌ Don't do this:
```rust
// Direct field access
state.browser_facade.as_ref()
```

### ✅ Do this instead:
```rust
// Use accessor method
state.browser_facade()?
```

### ❌ Don't do this:
```rust
// Unconditional import
use riptide_headless::pool::BrowserPool;
```

### ✅ Do this instead:
```rust
// Feature-gated import
#[cfg(feature = "browser")]
use riptide_headless::pool::BrowserPool;
```

### ❌ Don't do this:
```rust
// No stub route
#[cfg(feature = "llm")]
pub fn llm_routes() -> Router<AppState> { ... }
// Nothing when feature disabled - route panics!
```

### ✅ Do this instead:
```rust
// Provide stub with helpful error
#[cfg(not(feature = "llm"))]
pub fn llm_routes() -> Router<AppState> {
    Router::new().route("/*path", any(|| async {
        Err::<(), _>(ApiError::feature_disabled("llm", "..."))
    }))
}
```

---

## 🎓 Pattern Summary

**3-Step Pattern**:
1. **Import**: `#[cfg(feature = "X")] use crate::Y;`
2. **Implementation**: `#[cfg(feature = "X")] { real_code }`
3. **Stub**: `#[cfg(not(feature = "X"))] { error_or_stub }`

**Always provide helpful errors**:
- ✅ "Feature X disabled. Rebuild with --features X"
- ❌ "Failed to compile" (unhelpful)

**Test all combinations**:
- No features
- Each feature alone
- All features together
- Default features

---

## 📞 Agent Coordination

### Memory Keys
- Architecture: `swarm/architecture/feature-gates-design`
- Progress: `swarm/implementation/feature-gates-progress`
- Blockers: `swarm/blockers/feature-gates`

### Hooks Commands
```bash
# Before work
npx claude-flow@alpha hooks pre-task --description "Implement FILE_NAME"

# After file edit
npx claude-flow@alpha hooks post-edit --file "FILE_PATH" --memory-key "swarm/impl/FILE_NAME"

# Notify completion
npx claude-flow@alpha hooks notify --message "FILE_NAME implementation complete"
```

---

**Quick Reference Version**: 1.0
**Last Updated**: 2025-11-04
**Use with**: `feature-gates-solution.md` (full architecture)
