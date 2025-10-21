# Crate Architecture Map - Browser Automation Layer

**Date**: 2025-10-21
**Status**: Complete Architecture Analysis

## Executive Summary

This document maps the responsibilities, public APIs, and dependencies of the core browser automation crates in the RipTide project. The architecture has recently undergone a major consolidation where browser engine components were extracted from `riptide-headless` into a dedicated `riptide-engine` crate.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    riptide-engine                           │
│  (Browser Pool + CDP Pool + Launcher + Hybrid Fallback)    │
│                                                             │
│  Re-exported by: riptide-headless (compatibility layer)    │
└────────────┬────────────────────────────────────┬───────────┘
             │                                    │
             │ uses                               │ uses
             ▼                                    ▼
┌────────────────────────────┐    ┌──────────────────────────┐
│ riptide-browser-abstraction│    │   riptide-stealth        │
│   (Unified Browser API)    │    │ (Anti-detection Layer)   │
└────────────────────────────┘    └──────────────────────────┘
             │
             │ wraps
             ▼
┌────────────────────────────┐
│    spider_chrome           │
│  (exports as chromiumoxide)│
│   Chrome DevTools Protocol │
└────────────────────────────┘
```

---

## 1. riptide-engine

**Location**: `/workspaces/eventmesh/crates/riptide-engine/`
**Purpose**: Core browser automation engine with pooling, CDP optimization, and lifecycle management

### Core Responsibilities

1. **Browser Pool Management** (`pool.rs`)
   - Lifecycle management with resource tracking
   - Browser instance creation with unique profile directories
   - Health monitoring (tiered: fast liveness + full diagnostics)
   - Memory limit enforcement (soft/hard limits)
   - Automatic recovery and cleanup

2. **CDP Connection Pooling** (`cdp_pool.rs`)
   - Connection multiplexing (reuse across requests)
   - Command batching (~50% round-trip reduction)
   - Health checking for stale connections
   - Priority-based connection allocation
   - Session affinity for related requests

3. **High-Level Launcher** (`launcher.rs`)
   - Browser session orchestration
   - Stealth integration with presets
   - Page automation (navigation, screenshots, PDF)
   - Statistics tracking and monitoring

4. **Hybrid Engine Fallback** (`hybrid_fallback.rs`)
   - 20% traffic split to spider-chrome
   - Automatic fallback to chromiumoxide on failure
   - Metrics tracking (success rates, fallback rates)
   - Hash-based traffic routing

### Public API

```rust
// Browser Pool
pub struct BrowserPool { ... }
pub struct BrowserCheckout { ... }
pub struct BrowserPoolConfig { ... }
pub struct PoolStats { ... }
pub enum PoolEvent { ... }

// CDP Connection Pool
pub struct CdpConnectionPool { ... }
pub struct PooledConnection { ... }
pub struct CdpPoolConfig { ... }
pub struct ConnectionStats { ... }
pub enum ConnectionHealth { ... }

// Launcher
pub struct HeadlessLauncher { ... }
pub struct LaunchSession<'a> { ... }
pub struct LauncherConfig { ... }
pub struct LauncherStats { ... }

// Hybrid Fallback
pub struct HybridBrowserFallback { ... }
pub struct BrowserResponse { ... }
pub struct FallbackMetrics { ... }
pub enum EngineKind { ... }

// Re-exports from browser-abstraction
pub use riptide_browser_abstraction::{
    BrowserEngine, EngineType, NavigateParams, PageHandle
};
```

### Key Features

- **QW-1**: 4x capacity improvement (max pool size: 20)
- **QW-2**: 5x faster failure detection (tiered health checks)
- **QW-3**: -30% memory footprint (soft/hard memory limits)
- **P1-B4**: 30% latency reduction (CDP connection pooling)
- **Unique Profile Directories**: Each browser instance has isolated profile to prevent SingletonLock conflicts

### Dependencies

```toml
# Internal
riptide-types
riptide-config
riptide-browser-abstraction
riptide-stealth
riptide-headless-hybrid (optional, feature-gated)

# Browser automation
spider_chrome (exports as chromiumoxide)
spider_chromiumoxide_cdp

# Runtime
tokio, futures, async-trait
```

---

## 2. riptide-browser-abstraction

**Location**: `/workspaces/eventmesh/crates/riptide-browser-abstraction/`
**Purpose**: Unified abstraction layer over spider_chrome

### Core Responsibilities

1. **Browser Engine Abstraction** (`traits.rs`)
   - Unified `BrowserEngine` trait
   - Unified `PageHandle` trait for page operations
   - Engine type identification

2. **Chromiumoxide Implementation** (`chromiumoxide_impl.rs`)
   - Wraps spider_chrome's chromiumoxide compatibility layer
   - Provides trait-based access to Browser/Page

3. **Spider-Chrome Implementation** (`spider_impl.rs`)
   - Direct spider_chrome native API usage
   - CDP integration for screenshots/PDF
   - Thread-safe Arc-based page handling

4. **Factory & Params** (`factory.rs`, `params.rs`)
   - Engine creation (placeholder, requires Browser instance)
   - Navigation, screenshot, and PDF parameters

### Public API

```rust
// Core traits
pub trait BrowserEngine: Send + Sync {
    async fn new_page(&self) -> AbstractionResult<Box<dyn PageHandle>>;
    fn engine_type(&self) -> EngineType;
    async fn close(&self) -> AbstractionResult<()>;
    async fn version(&self) -> AbstractionResult<String>;
}

pub trait PageHandle: Send + Sync {
    async fn goto(&self, url: &str, params: NavigateParams) -> AbstractionResult<()>;
    async fn content(&self) -> AbstractionResult<String>;
    async fn url(&self) -> AbstractionResult<String>;
    async fn evaluate(&self, script: &str) -> AbstractionResult<serde_json::Value>;
    async fn screenshot(&self, params: ScreenshotParams) -> AbstractionResult<Vec<u8>>;
    async fn pdf(&self, params: PdfParams) -> AbstractionResult<Vec<u8>>;
    async fn wait_for_navigation(&self, timeout_ms: u64) -> AbstractionResult<()>;
    async fn set_timeout(&self, timeout_ms: u64) -> AbstractionResult<()>;
    async fn close(&self) -> AbstractionResult<()>;
}

// Engine types
pub enum EngineType {
    Chromiumoxide,
    SpiderChrome,
}

// Implementations
pub struct ChromiumoxideEngine { ... }
pub struct ChromiumoxidePage { ... }
pub struct SpiderChromeEngine { ... }
pub struct SpiderChromePage { ... }

// Parameters
pub struct NavigateParams { ... }
pub struct ScreenshotParams { ... }
pub struct PdfParams { ... }
pub enum WaitUntil { Load, DOMContentLoaded, NetworkIdle }
pub enum ScreenshotFormat { Png, Jpeg }
```

### Architecture Notes

- **spider_chrome exports as "chromiumoxide"**: Both implementations use the same underlying library
- **Minimal overhead**: <0.01% performance impact from trait abstraction
- **Arc-based thread safety**: Pages wrapped in Arc for concurrent access
- **Known limitation**: `close()` methods don't work through Arc (auto-cleanup on drop instead)

### Dependencies

```toml
# Core
async-trait, anyhow, thiserror, tokio, tracing
serde, serde_json

# Browser engine
spider_chrome (workspace)
spider_chromiumoxide_cdp (workspace)

# Internal
riptide-types
```

---

## 3. riptide-headless

**Location**: `/workspaces/eventmesh/crates/riptide-headless/`
**Purpose**: Compatibility layer and HTTP API wrapper over riptide-engine

### Core Responsibilities

1. **Re-export Engine Components** (`lib.rs`)
   - Re-exports all types from `riptide-engine` for backward compatibility
   - Maintains legacy module structure (`pool`, `cdp_pool`, `launcher`)

2. **CDP HTTP API** (`cdp.rs`)
   - HTTP endpoints for browser automation
   - Request/response models
   - Integration with engine components

3. **Dynamic Content Handling** (`dynamic.rs`)
   - Dynamic configuration (viewport, scrolling, wait conditions)
   - Page actions (wait, scroll, JavaScript execution)

### Public API

```rust
// Re-exported from riptide-engine
pub use riptide_engine::{
    // Browser pool
    BrowserPool, BrowserPoolConfig, BrowserCheckout,
    PoolEvent, PoolStats,

    // CDP pool
    CdpConnectionPool, CdpPoolConfig,
    ConnectionHealth, ConnectionStats, PooledConnection,

    // Launcher
    HeadlessLauncher, LaunchSession,
    LauncherConfig, LauncherStats,

    // Hybrid fallback
    BrowserResponse, FallbackMetrics, HybridBrowserFallback,

    // Models
    models,
};

// Dynamic content handling
pub use dynamic::{
    DynamicConfig, PageAction, ScrollConfig,
    ViewportConfig, WaitCondition,
};

// Backward compatibility modules
pub mod pool { ... }       // re-exports from riptide-engine
pub mod cdp_pool { ... }   // re-exports from riptide-engine
pub mod launcher { ... }   // re-exports from riptide-engine
```

### Architecture Notes

- **Circular Dependency Fix (P2-F1 Day 3)**: Removed dependency on `riptide-core`
- **Compatibility Layer**: Maintains backward compatibility with existing APIs
- **Future Evolution**: May become pure HTTP API wrapper as engine stabilizes

### Dependencies

```toml
# Internal (primary dependency)
riptide-engine
riptide-stealth

# HTTP/API
axum, tower, tower-http, base64

# Browser
spider_chrome

# Runtime
tokio, futures, serde, serde_json, anyhow
tracing, tracing-subscriber
```

---

## Dependency Graph

```
riptide-headless
    ├── riptide-engine (primary)
    │   ├── riptide-browser-abstraction
    │   │   ├── spider_chrome
    │   │   ├── spider_chromiumoxide_cdp
    │   │   └── riptide-types
    │   ├── riptide-stealth
    │   ├── riptide-types
    │   └── riptide-config
    └── riptide-stealth

riptide-browser-abstraction
    ├── spider_chrome
    ├── spider_chromiumoxide_cdp
    └── riptide-types
```

---

## Key Architectural Patterns

### 1. **Trait-Based Abstraction**

The browser abstraction uses async traits to provide a unified API:

```rust
// Allows swapping engines without changing consumer code
let engine: Box<dyn BrowserEngine> = ...;
let page = engine.new_page().await?;
let html = page.content().await?;
```

### 2. **Pool-Based Resource Management**

Browser instances are pooled for efficiency:

```rust
// Automatic checkin on drop
let checkout = pool.checkout().await?;
let page = checkout.new_page(url).await?;
// checkout automatically returned to pool when dropped
```

### 3. **CDP Connection Multiplexing (P1-B4)**

Reduces latency through connection reuse:

```rust
// Get or reuse connection
let session_id = cdp_pool.get_connection(browser_id, browser, url).await?;
// Release when done
cdp_pool.release_connection(browser_id, &session_id).await?;
```

### 4. **Tiered Health Monitoring (QW-2)**

Fast liveness checks + comprehensive health checks:

```rust
// Fast check every 2s (quick ping)
browser.fast_health_check().await; // 500ms timeout

// Full check every 15s (memory, page count, diagnostics)
browser.full_health_check(soft_limit, hard_limit).await;
```

### 5. **Hybrid Engine with Fallback**

20% traffic to spider-chrome with automatic fallback:

```rust
// Hash-based routing (consistent per URL)
if should_use_spider_chrome(url) {
    match try_spider_chrome(url).await {
        Ok(response) => return Ok(response),
        Err(_) => fallback_to_chromiumoxide(url).await
    }
}
```

---

## Module Organization

### riptide-engine/src/

```
lib.rs              - Public API and re-exports
pool.rs             - BrowserPool implementation
cdp_pool.rs         - CdpConnectionPool (P1-B4)
launcher.rs         - HeadlessLauncher + LaunchSession
hybrid_fallback.rs  - HybridBrowserFallback
models.rs           - Request/response models
cdp.rs              - CDP utilities
```

### riptide-browser-abstraction/src/

```
lib.rs                  - Public exports
traits.rs               - BrowserEngine + PageHandle traits
chromiumoxide_impl.rs   - Chromiumoxide wrapper
spider_impl.rs          - Spider-chrome native wrapper
factory.rs              - Engine creation (placeholder)
params.rs               - Navigation/screenshot/PDF params
error.rs                - Error types
```

### riptide-headless/src/

```
lib.rs      - Re-exports from riptide-engine
cdp.rs      - CDP HTTP API
dynamic.rs  - Dynamic content handling
pool.rs     - Legacy module (re-exports)
cdp_pool.rs - Legacy module (re-exports)
launcher.rs - Legacy module (re-exports)
```

---

## Performance Characteristics

### Browser Pool (QW-1, QW-2, QW-3)

- **Capacity**: 4x improvement (max 20 browsers)
- **Health Checks**: 5x faster failure detection
  - Fast checks: 2s interval, 500ms timeout
  - Full checks: 15s interval with diagnostics
- **Memory**: -30% footprint
  - Soft limit: 400MB (trigger cleanup)
  - Hard limit: 500MB (force eviction)

### CDP Connection Pool (P1-B4)

- **Latency**: 30% reduction target
- **Connection Reuse**: >70% target rate
- **Batching**: ~50% round-trip reduction
- **Max Connections**: 10 per browser
- **Lifetime**: 5 minutes max

### Launcher

- **Page Timeout**: 30s default
- **Stealth Injection**: <100ms overhead
- **Session Tracking**: Real-time statistics

---

## Known Limitations

1. **Arc-based Close Methods**: Cannot explicitly close browsers/pages through Arc due to ownership requirements. Auto-cleanup on drop instead.

2. **Timeout Limitations**: Some chromiumoxide methods don't support custom timeouts through the trait abstraction.

3. **Feature Gating**: `hybrid_fallback` module requires `headless` feature flag.

4. **Profile Directory Management**: Each browser instance requires unique profile directory to prevent Chrome SingletonLock conflicts.

---

## Future Evolution

1. **riptide-headless** → Pure HTTP API wrapper
2. **riptide-engine** → Standalone browser automation library
3. **riptide-browser-abstraction** → Multi-engine support (Playwright, Puppeteer)
4. **Enhanced CDP Pool**: Advanced batching strategies, priority queues

---

## Related Documentation

- `/workspaces/eventmesh/docs/COMPREHENSIVE-ROADMAP.md` - Project roadmap
- `/workspaces/eventmesh/crates/riptide-engine/src/lib.rs` - Engine API documentation
- `/workspaces/eventmesh/crates/riptide-browser-abstraction/src/lib.rs` - Abstraction layer docs

---

**End of Architecture Map**
