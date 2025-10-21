# RipTide Browser Consolidation Architecture Plan

## Executive Summary

**Problem**: Current riptide-browser is a facade (re-exports only), violating the principle of "actual implementations, not proxies." Browser pool management is duplicated across riptide-engine and riptide-headless with inconsistencies.

**Solution**: Transform riptide-browser into a real implementation crate that consolidates all browser automation logic, then reverse dependencies so other crates depend on it.

**Impact**:
- Eliminates 3 duplicate implementations (~4,000 LOC)
- Single source of truth for browser automation
- Clear dependency hierarchy
- Maintainable, testable codebase

---

## Current Architecture Analysis

### Duplication Map

```
┌─────────────────────────────────────────────────────────────┐
│ CURRENT DUPLICATED STATE (ANTI-PATTERN)                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  riptide-engine/                                            │
│  ├── pool.rs              (1,364 LOC) ← CDP pool added     │
│  ├── cdp_pool.rs          (494 LOC)   ← NEW in P1-B4       │
│  ├── launcher.rs          (varies)                          │
│  └── hybrid_fallback.rs   (331 LOC)   ← Has headless       │
│                                                             │
│  riptide-headless/                                          │
│  ├── pool.rs              (1,326 LOC) ← NO CDP pool        │
│  ├── cdp_pool.rs          (494 LOC)   ← Duplicate copy     │
│  ├── launcher.rs          (varies)                          │
│  └── hybrid_fallback.rs   (varies)    ← Different impl     │
│                                                             │
│  riptide-browser/                                           │
│  └── lib.rs               (81 LOC)    ← FACADE ONLY!!!     │
│      └── Re-exports from engine + headless                 │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Key Differences Discovered

1. **riptide-engine/pool.rs** (1,364 LOC):
   - HAS CDP pool integration (P1-B4)
   - CDP connection multiplexing
   - Reference to `CdpConnectionPool`

2. **riptide-headless/pool.rs** (1,326 LOC):
   - NO CDP pool integration
   - Missing 38 LOC of CDP logic
   - Older implementation

3. **CDP Pool Duplication**:
   - Identical 494 LOC in both crates
   - Should only exist once

4. **Hybrid Fallback**:
   - Different implementations
   - Engine version has `EngineKind` enum
   - Headless version may differ

---

## Target Architecture

### New Directory Structure

```
crates/riptide-browser/
├── Cargo.toml                          # Direct dependencies
├── src/
│   ├── lib.rs                          # Public API exports
│   │
│   ├── pool/
│   │   ├── mod.rs                      # BrowserPool implementation
│   │   ├── config.rs                   # BrowserPoolConfig
│   │   ├── checkout.rs                 # BrowserCheckout
│   │   ├── events.rs                   # PoolEvent
│   │   ├── health.rs                   # Health monitoring
│   │   └── stats.rs                    # PoolStats
│   │
│   ├── cdp/
│   │   ├── mod.rs                      # CdpConnectionPool
│   │   ├── config.rs                   # CdpPoolConfig
│   │   ├── connection.rs               # PooledConnection
│   │   ├── command.rs                  # CdpCommand
│   │   ├── health.rs                   # ConnectionHealth
│   │   └── stats.rs                    # ConnectionStats, CdpPoolStats
│   │
│   ├── launcher/
│   │   ├── mod.rs                      # HeadlessLauncher (unified)
│   │   ├── config.rs                   # LauncherConfig
│   │   ├── session.rs                  # LaunchSession
│   │   └── stats.rs                    # LauncherStats
│   │
│   ├── hybrid/
│   │   ├── mod.rs                      # HybridBrowserFallback
│   │   ├── metrics.rs                  # FallbackMetrics
│   │   └── response.rs                 # BrowserResponse
│   │
│   ├── models/
│   │   └── mod.rs                      # Shared types (from engine)
│   │
│   └── cdp_api/
│       └── mod.rs                      # CDP HTTP API types
│
└── tests/
    ├── pool_tests.rs
    ├── cdp_pool_tests.rs
    └── integration_tests.rs
```

### Dependency Graph (REVERSED)

```
BEFORE (Current - WRONG):
┌────────────────┐
│ riptide-browser│ ← Facade only
└───────┬────────┘
        │ depends on
        ├───────────────────┐
        ▼                   ▼
┌────────────────┐  ┌────────────────┐
│ riptide-engine │  │riptide-headless│ ← Real implementations
└────────────────┘  └────────────────┘
        │                   │
        └───────┬───────────┘
                ▼
        (Duplication)

AFTER (Target - CORRECT):
┌────────────────┐
│ riptide-browser│ ← Real implementation (single source of truth)
└───────┬────────┘
        │ depended on by
        ├───────────────────┐
        ▼                   ▼
┌────────────────┐  ┌────────────────┐
│ riptide-engine │  │riptide-headless│ ← Re-export from browser
└────────────────┘  └────────────────┘
        │                   │
        └───────┬───────────┘
                ▼
        (No duplication)
```

---

## Implementation Phases

### Phase 1: Create Real riptide-browser Implementation

**Goal**: Copy implementations FROM riptide-engine TO riptide-browser

#### Step 1.1: Setup Dependencies

**File**: `/workspaces/eventmesh/crates/riptide-browser/Cargo.toml`

```toml
[package]
name = "riptide-browser"
version = "0.1.0"
edition.workspace = true
license.workspace = true
authors.workspace = true
description = "Unified browser automation for RipTide - browser pool, CDP, launcher"

[dependencies]
# Core RipTide dependencies
riptide-types = { path = "../riptide-types" }
riptide-config = { path = "../riptide-config" }
riptide-stealth = { path = "../riptide-stealth" }
riptide-browser-abstraction = { path = "../riptide-browser-abstraction" }

# Browser automation - DIRECT dependencies (not through other crates)
spider_chromiumoxide_cdp = { workspace = true }
spider_chrome = { workspace = true }  # Exports as chromiumoxide

# Headless hybrid launcher (optional)
riptide-headless-hybrid = { path = "../riptide-headless-hybrid", optional = true }

# Async runtime
tokio = { workspace = true, features = ["full"] }
futures = { workspace = true }
async-trait = { workspace = true }

# Web/HTTP (for CDP API)
axum = { workspace = true }
tower = { workspace = true }
tower-http = { workspace = true }
base64 = "0.22"

# Utilities
tracing = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
dashmap = { workspace = true }
sysinfo = { workspace = true }
psutil = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
tempfile = { workspace = true }
bytes = { workspace = true }

[dev-dependencies]
tokio-test = "0.4"
tempfile = { workspace = true }
serial_test = "3.2.0"

[features]
default = []
headless = ["riptide-headless-hybrid"]
```

#### Step 1.2: Copy Browser Pool Implementation

**Source**: `/workspaces/eventmesh/crates/riptide-engine/src/pool.rs` (1,364 LOC with CDP)

**Actions**:
1. Create module structure:
   ```
   src/pool/
   ├── mod.rs          (core BrowserPool implementation)
   ├── config.rs       (BrowserPoolConfig)
   ├── checkout.rs     (BrowserCheckout, BrowserPoolRef)
   ├── events.rs       (PoolEvent)
   ├── health.rs       (BrowserHealth, health check methods)
   └── stats.rs        (BrowserStats, PoolStats)
   ```

2. Copy from `riptide-engine/src/pool.rs` with modifications:
   - Import `CdpConnectionPool` from `crate::cdp::CdpConnectionPool`
   - Keep CDP pool integration (lines 18, 424, 481-482, 600, 662-663, 1127, 1191, 1244, 1264)
   - Include tiered health checks (P1-B2)
   - Include memory limits (QW-3)

**Key Sections**:
- `BrowserPoolConfig` (lines 23-107) → `config.rs`
- `BrowserHealth` (lines 110-118) → `health.rs`
- `BrowserStats` (lines 123-130) → `stats.rs`
- `PooledBrowser` (lines 133-395) → `mod.rs`
- `PoolEvent` (lines 398-409) → `events.rs`
- `BrowserPool` (lines 412-1106) → `mod.rs`
- `BrowserCheckout` (lines 1186-1301) → `checkout.rs`

#### Step 1.3: Copy CDP Pool Implementation

**Source**: `/workspaces/eventmesh/crates/riptide-engine/src/cdp_pool.rs` (494 LOC)

**Actions**:
1. Create module structure:
   ```
   src/cdp/
   ├── mod.rs          (CdpConnectionPool)
   ├── config.rs       (CdpPoolConfig)
   ├── connection.rs   (PooledConnection)
   ├── command.rs      (CdpCommand)
   ├── health.rs       (ConnectionHealth)
   └── stats.rs        (ConnectionStats, CdpPoolStats)
   ```

2. Copy entire file with split:
   - `CdpPoolConfig` (lines 23-55) → `config.rs`
   - `ConnectionHealth` (lines 58-64) → `health.rs`
   - `ConnectionStats` (lines 67-86) → `stats.rs`
   - `PooledConnection` (lines 89-162) → `connection.rs`
   - `CdpCommand` (lines 174-179) → `command.rs`
   - `CdpConnectionPool` (lines 165-422) → `mod.rs`
   - `CdpPoolStats` (lines 425-431) → `stats.rs`

#### Step 1.4: Copy Launcher Implementation

**Source**: `/workspaces/eventmesh/crates/riptide-engine/src/launcher.rs`

**Actions**:
1. Create module structure:
   ```
   src/launcher/
   ├── mod.rs          (HeadlessLauncher)
   ├── config.rs       (LauncherConfig)
   ├── session.rs      (LaunchSession)
   └── stats.rs        (LauncherStats)
   ```

2. Update imports to use local modules:
   - `use crate::pool::{BrowserPool, BrowserPoolConfig}`
   - `use crate::cdp::CdpConnectionPool`

#### Step 1.5: Copy Hybrid Fallback

**Source**: `/workspaces/eventmesh/crates/riptide-engine/src/hybrid_fallback.rs` (331 LOC)

**Actions**:
1. Create module structure:
   ```
   src/hybrid/
   ├── mod.rs          (HybridBrowserFallback)
   ├── metrics.rs      (FallbackMetrics)
   └── response.rs     (BrowserResponse, BrowserEngine)
   ```

2. Copy with feature gates intact:
   - Keep `#[cfg(feature = "headless")]` guards
   - Update imports to use `riptide_headless_hybrid::HybridHeadlessLauncher`

#### Step 1.6: Copy Models and CDP API

**Sources**:
- `/workspaces/eventmesh/crates/riptide-engine/src/models.rs`
- `/workspaces/eventmesh/crates/riptide-engine/src/cdp.rs`

**Actions**:
1. Create `src/models/mod.rs` - copy as-is
2. Create `src/cdp_api/mod.rs` - copy as-is

#### Step 1.7: Create Public API

**File**: `/workspaces/eventmesh/crates/riptide-browser/src/lib.rs`

```rust
//! RipTide Browser - Unified Browser Automation
//!
//! This crate provides the core browser automation infrastructure for RipTide:
//! - Browser pool lifecycle management with resource tracking
//! - CDP (Chrome DevTools Protocol) connection pooling and batching
//! - High-level launcher API with stealth integration
//! - Hybrid engine fallback (spider-chrome with chromiumoxide fallback)
//!
//! ## Architecture
//!
//! riptide-browser consolidates all browser automation components:
//! - **pool**: Browser pool management with health monitoring
//! - **cdp**: CDP connection pooling and command batching
//! - **launcher**: High-level browser session launcher
//! - **hybrid**: Engine selection and fallback logic
//!
//! ## Usage
//!
//! ```no_run
//! use riptide_browser::launcher::HeadlessLauncher;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create launcher with pooling
//!     let launcher = HeadlessLauncher::new().await?;
//!
//!     // Launch a page with stealth
//!     let session = launcher.launch_page_default("https://example.com").await?;
//!
//!     // Access the page
//!     let page = session.page();
//!     let content = page.content().await?;
//!
//!     // Session automatically returns browser to pool when dropped
//!     Ok(())
//! }
//! ```

// Core browser pool management
pub mod pool;

// CDP connection pooling
pub mod cdp;

// Models and types
pub mod models;

// High-level launcher API
pub mod launcher;

// CDP HTTP API types
pub mod cdp_api;

// Hybrid engine fallback
#[cfg(feature = "headless")]
pub mod hybrid;

// Re-export browser abstraction types
pub use riptide_browser_abstraction::{
    BrowserEngine as AbstractBrowserEngine,
    EngineType,
    NavigateParams,
    PageHandle,
    ChromiumoxideEngine,
    ChromiumoxidePage,
};

// Re-export main public API
pub use cdp::{
    BatchExecutionResult,
    BatchResult,
    CdpCommand,
    CdpConnectionPool,
    CdpPoolConfig,
    ConnectionHealth,
    ConnectionStats,
    PooledConnection,
};

pub use launcher::{
    HeadlessLauncher,
    LaunchSession,
    LauncherConfig,
    LauncherStats,
};

pub use pool::{
    BrowserCheckout,
    BrowserPool,
    BrowserPoolConfig,
    PoolEvent,
    PoolStats,
};

#[cfg(feature = "headless")]
pub use hybrid::{
    BrowserResponse,
    BrowserEngine as EngineKind,
    FallbackMetrics,
    HybridBrowserFallback,
};

// Factory functions for wrapping spider_chrome instances
#[cfg(feature = "headless")]
pub mod factory {
    use chromiumoxide::{Browser, Page};
    use super::{AbstractBrowserEngine, PageHandle, ChromiumoxideEngine, ChromiumoxidePage};

    /// Wrap a chromiumoxide Browser in the BrowserEngine trait
    pub fn wrap_browser(browser: Browser) -> Box<dyn AbstractBrowserEngine> {
        Box::new(ChromiumoxideEngine::new(browser))
    }

    /// Wrap a chromiumoxide Page in the PageHandle trait
    pub fn wrap_page(page: Page) -> Box<dyn PageHandle> {
        Box::new(ChromiumoxidePage::new(page))
    }
}
```

---

### Phase 2: Reverse Dependencies

**Goal**: Make riptide-engine and riptide-headless RE-EXPORT from riptide-browser

#### Step 2.1: Update riptide-engine

**File**: `/workspaces/eventmesh/crates/riptide-engine/Cargo.toml`

```toml
[dependencies]
# Browser automation now comes from riptide-browser
riptide-browser = { path = "../riptide-browser", features = ["headless"] }

# Remove these (now in riptide-browser):
# spider_chromiumoxide_cdp
# spider_chrome
# riptide-headless-hybrid
# ... (keep other dependencies)
```

**File**: `/workspaces/eventmesh/crates/riptide-engine/src/lib.rs`

```rust
//! Browser engine and pool management for RipTide
//!
//! This crate re-exports browser automation from riptide-browser
//! for backward compatibility.

// Re-export everything from riptide-browser
pub use riptide_browser::{
    // Pool
    pool, BrowserCheckout, BrowserPool, BrowserPoolConfig, PoolEvent, PoolStats,

    // CDP
    cdp, BatchExecutionResult, BatchResult, CdpCommand, CdpConnectionPool,
    CdpPoolConfig, ConnectionHealth, ConnectionStats, PooledConnection,

    // Launcher
    launcher, HeadlessLauncher, LaunchSession, LauncherConfig, LauncherStats,

    // Models
    models,

    // CDP API
    cdp_api,

    // Abstraction
    AbstractBrowserEngine, EngineType, NavigateParams, PageHandle,

    // Factory
    factory,
};

// Re-export hybrid fallback (feature-gated)
#[cfg(feature = "headless")]
pub use riptide_browser::hybrid::{
    self as hybrid_fallback,
    BrowserResponse,
    EngineKind,
    FallbackMetrics,
    HybridBrowserFallback,
};
```

#### Step 2.2: Update riptide-headless

**File**: `/workspaces/eventmesh/crates/riptide-headless/Cargo.toml`

```toml
[dependencies]
# Browser automation now comes from riptide-browser
riptide-browser = { path = "../riptide-browser", features = ["headless"] }

# Remove duplicates:
# spider_chromiumoxide_cdp
# spider_chrome
# ... (keep unique dependencies like dynamic rendering)
```

**File**: `/workspaces/eventmesh/crates/riptide-headless/src/lib.rs`

```rust
//! RipTide Headless - Dynamic Rendering and HTTP API
//!
//! This crate provides:
//! - Dynamic rendering with viewport/scroll/wait conditions
//! - HTTP API wrapper for browser automation
//! - Backward compatibility layer

// Re-export browser automation from riptide-browser
pub use riptide_browser::{
    pool, BrowserPool, BrowserPoolConfig,
    cdp, CdpConnectionPool, CdpPoolConfig,
    launcher, HeadlessLauncher,
    hybrid, HybridBrowserFallback,
};

// Unique to riptide-headless
pub mod dynamic;

// Re-export dynamic rendering types
pub use dynamic::{
    DynamicConfig,
    DynamicRenderResult,
    PageAction,
    RenderArtifacts,
    ScrollConfig,
    ViewportConfig,
    WaitCondition,
};
```

#### Step 2.3: Delete Duplicate Files

**Files to DELETE from riptide-engine**:
- `src/pool.rs` (1,364 LOC)
- `src/cdp_pool.rs` (494 LOC)
- `src/launcher.rs` (varies)
- `src/hybrid_fallback.rs` (331 LOC)
- `src/models.rs` (varies)
- `src/cdp.rs` (varies)

**Files to DELETE from riptide-headless**:
- `src/pool.rs` (1,326 LOC)
- `src/cdp_pool.rs` (494 LOC)
- `src/hybrid_fallback.rs` (varies)
- Launcher files (if duplicated)

**Keep in riptide-headless**:
- `src/dynamic.rs` (unique functionality)
- HTTP API wrappers (unique)

---

### Phase 3: Update Consumers

#### Step 3.1: Update Direct Consumers

**Crates to update**:
1. `riptide-cli`
2. `riptide-api`
3. `riptide-workers`
4. Any other crates using browser automation

**Changes**:
```toml
# In Cargo.toml:
[dependencies]
# Option 1: Direct dependency
riptide-browser = { path = "../riptide-browser", features = ["headless"] }

# Option 2: Through engine (backward compatibility)
riptide-engine = { path = "../riptide-engine", features = ["headless"] }

# Option 3: Through headless (for dynamic rendering)
riptide-headless = { path = "../riptide-headless" }
```

```rust
// In source files:
// Option 1: Direct
use riptide_browser::{BrowserPool, HeadlessLauncher};

// Option 2: Through engine (still works)
use riptide_engine::{BrowserPool, HeadlessLauncher};

// Option 3: Through headless (still works)
use riptide_headless::{BrowserPool, HeadlessLauncher, dynamic::DynamicConfig};
```

#### Step 3.2: Update Tests

**Test Updates**:
1. Move tests from `riptide-engine/tests/` to `riptide-browser/tests/`
2. Update test imports:
   ```rust
   use riptide_browser::{BrowserPool, BrowserPoolConfig};
   ```
3. Keep integration tests in original crates for compatibility

---

### Phase 4: Validation and Cleanup

#### Step 4.1: Compilation Verification

```bash
# Build all crates
cargo build --all-features

# Run tests
cargo test --all-features

# Check for unused dependencies
cargo machete

# Verify no duplication
rg "BrowserPool" --files-with-matches | sort
```

#### Step 4.2: Documentation Updates

1. Update README files
2. Update architecture diagrams
3. Update migration guides
4. Add consolidation notes to CHANGELOG

#### Step 4.3: Performance Verification

**Metrics to verify**:
- Browser pool performance (unchanged)
- CDP connection multiplexing (unchanged)
- Memory usage (should decrease with less duplication)
- Build times (should improve with less code)

---

## Migration Checklist

### Pre-Migration
- [ ] Backup current working state
- [ ] Document all consumers of browser automation
- [ ] Identify unique functionality per crate

### Phase 1: Create Real Implementation
- [ ] Update riptide-browser/Cargo.toml with dependencies
- [ ] Create src/pool/ module structure
- [ ] Copy pool implementation from riptide-engine (with CDP)
- [ ] Create src/cdp/ module structure
- [ ] Copy CDP pool implementation
- [ ] Create src/launcher/ module structure
- [ ] Copy launcher implementation
- [ ] Create src/hybrid/ module structure
- [ ] Copy hybrid fallback implementation
- [ ] Copy models and CDP API
- [ ] Create comprehensive lib.rs with exports
- [ ] Build riptide-browser: `cargo build -p riptide-browser --all-features`
- [ ] Run riptide-browser tests: `cargo test -p riptide-browser --all-features`

### Phase 2: Reverse Dependencies
- [ ] Update riptide-engine/Cargo.toml to depend on riptide-browser
- [ ] Replace riptide-engine/src/lib.rs with re-exports
- [ ] Delete duplicate files from riptide-engine
- [ ] Update riptide-headless/Cargo.toml to depend on riptide-browser
- [ ] Replace riptide-headless/src/lib.rs with re-exports + unique
- [ ] Delete duplicate files from riptide-headless
- [ ] Build both: `cargo build -p riptide-engine -p riptide-headless --all-features`
- [ ] Run tests: `cargo test -p riptide-engine -p riptide-headless --all-features`

### Phase 3: Update Consumers
- [ ] Update riptide-cli imports
- [ ] Update riptide-api imports
- [ ] Update riptide-workers imports
- [ ] Build workspace: `cargo build --workspace --all-features`
- [ ] Run workspace tests: `cargo test --workspace --all-features`

### Phase 4: Validation
- [ ] Full workspace compilation check
- [ ] All tests passing
- [ ] No unused dependencies (cargo machete)
- [ ] Documentation updated
- [ ] Performance metrics verified

---

## Risk Assessment and Mitigation

### High Risk: Breaking Changes

**Risk**: Consumers may break if imports change

**Mitigation**:
1. Keep re-exports in riptide-engine and riptide-headless
2. Phase rollout (internal first, then consumers)
3. Comprehensive testing at each phase

### Medium Risk: Feature Inconsistencies

**Risk**: riptide-engine pool has CDP integration, riptide-headless doesn't

**Mitigation**:
1. Use riptide-engine/pool.rs as source of truth (newer, has CDP)
2. Test CDP multiplexing after migration
3. Verify all features work

### Low Risk: Test Coverage

**Risk**: Tests may not cover all edge cases

**Mitigation**:
1. Move all existing tests to riptide-browser
2. Add integration tests for re-exports
3. Run tests at each phase

---

## Success Criteria

### Code Quality
✅ Single source of truth for browser automation
✅ No duplicate implementations
✅ Clear dependency hierarchy
✅ All modules properly organized

### Functionality
✅ All existing features work
✅ CDP connection multiplexing functional
✅ Browser pool with health checks functional
✅ Hybrid fallback operational
✅ All tests passing

### Performance
✅ No performance regression
✅ Memory usage unchanged or improved
✅ Build times unchanged or improved

### Documentation
✅ Architecture clearly documented
✅ Migration guide complete
✅ API documentation updated

---

## Timeline Estimate

**Phase 1**: 4-6 hours (copy implementations, organize modules)
**Phase 2**: 2-3 hours (reverse dependencies, delete duplicates)
**Phase 3**: 2-3 hours (update consumers, fix imports)
**Phase 4**: 2-3 hours (validation, testing, documentation)

**Total**: 10-15 hours for complete consolidation

---

## Post-Migration Benefits

1. **Code Reduction**: ~3,000-4,000 LOC eliminated
2. **Maintainability**: Single place to fix bugs/add features
3. **Clarity**: Clear ownership and dependency flow
4. **Testing**: Easier to test one implementation
5. **Performance**: Less code to compile, potentially faster builds
6. **Extensibility**: Single API surface for future enhancements

---

## Notes and Considerations

### CDP Pool Integration
- riptide-engine/pool.rs has CDP pool (lines 18, 424, 481-482, 600, 662-663, 1127, 1191, 1244, 1264)
- riptide-headless/pool.rs LACKS CDP pool
- **Decision**: Use riptide-engine version as source of truth

### Hybrid Fallback
- Both crates have hybrid_fallback.rs but with differences
- riptide-engine has `EngineKind` enum
- **Decision**: Use riptide-engine version, verify compatibility

### Dynamic Rendering
- Unique to riptide-headless
- **Decision**: Keep in riptide-headless, not part of browser core

### Feature Flags
- Maintain `headless` feature for optional functionality
- Keep feature gates in place during migration

---

## Questions for Review

1. Should we rename `riptide-browser` to `riptide-browser-core` to clarify it's the implementation?
2. Do we want to version this as a breaking change (0.2.0)?
3. Should we create a deprecation period for old import paths?
4. Are there other crates with browser automation we haven't identified?

---

**Document Version**: 1.0
**Author**: System Architect
**Date**: 2025-10-21
**Status**: Ready for Implementation
