//! ⚠️ DEPRECATED COMPATIBILITY WRAPPER ⚠️
//!
//! **This crate is deprecated. Use `riptide-browser` instead.**
//!
//! All browser automation functionality has been migrated to `riptide-browser`.
//! This crate now serves as a thin compatibility layer for backward compatibility.
//!
//! ## Migration Guide
//!
//! Update your `Cargo.toml`:
//! ```toml
//! # Old (deprecated)
//! riptide-engine = { path = "../riptide-engine" }
//!
//! # New (recommended)
//! riptide-browser = { path = "../riptide-browser" }
//! ```
//!
//! Update your imports:
//! ```rust
//! // Old (deprecated)
//! use riptide_engine::launcher::HeadlessLauncher;
//! use riptide_engine::pool::BrowserPool;
//!
//! // New (recommended)
//! use riptide_browser::launcher::HeadlessLauncher;
//! use riptide_browser::pool::BrowserPool;
//! ```
//!
//! ## What Changed?
//!
//! | Module | Old Location | New Location | Lines Migrated |
//! |--------|-------------|--------------|----------------|
//! | Pool Management | `riptide-engine/src/pool.rs` | `riptide-browser/src/pool/mod.rs` | 1,363 |
//! | CDP Pooling | `riptide-engine/src/cdp_pool.rs` | `riptide-browser/src/cdp/mod.rs` | 1,630 |
//! | Launcher | `riptide-engine/src/launcher.rs` | `riptide-browser/src/launcher/mod.rs` | 672 |
//! | Models | `riptide-engine/src/models.rs` | `riptide-browser/src/models/mod.rs` | 132 |
//!
//! **Total: 3,797 lines of implementation moved to riptide-browser**

// ========================================
// Re-exports from riptide-browser
// ========================================

// Core browser pool management
pub use riptide_browser::pool;
pub use riptide_browser::{
    BrowserCheckout, BrowserHealth, BrowserPool, BrowserPoolConfig, BrowserPoolRef, BrowserStats,
    PoolEvent, PoolStats, PooledBrowser,
};

// CDP connection pooling
pub use riptide_browser::cdp;
pub use riptide_browser::{
    BatchExecutionResult, BatchResult, CdpCommand, CdpConnectionPool, CdpPoolConfig,
    ConnectionHealth, ConnectionPriority, ConnectionStats, PooledConnection,
};

// Launcher API
pub use riptide_browser::launcher;
pub use riptide_browser::{HeadlessLauncher, LaunchSession, LauncherConfig, LauncherStats};

// Models and types
pub use riptide_browser::models;
pub use riptide_browser::{Artifacts, ArtifactsOut, PageAction, RenderReq, RenderResp, Timeouts};

// Browser abstraction types
pub use riptide_browser::{
    Browser, BrowserConfig, BrowserEngine, ChromiumoxideEngine, ChromiumoxidePage, Page, SessionId,
};

// ========================================
// Legacy Modules (Kept for Compatibility)
// ========================================

// CDP utilities and types (if still needed separately)
pub mod cdp_utils {
    //! Legacy CDP utilities - prefer using riptide_browser::cdp directly
    pub use riptide_browser::cdp::*;
}

// Hybrid fallback is unique to riptide-engine (not migrated)
#[cfg(feature = "headless")]
pub mod hybrid_fallback;

// Factory functions (kept for backward compatibility)
#[cfg(feature = "headless")]
pub mod factory {
    //! Factory functions for wrapping browser instances
    //!
    //! **Note**: These are deprecated. Use `riptide_browser` abstractions directly.
    use chromiumoxide::{Browser, Page};
    use riptide_browser::{BrowserEngine, ChromiumoxideEngine, ChromiumoxidePage};

    /// Wrap a chromiumoxide Browser in the BrowserEngine trait
    #[deprecated(
        since = "0.2.0",
        note = "Use ChromiumoxideEngine::new() from riptide_browser directly"
    )]
    pub fn wrap_browser(browser: Browser) -> Box<dyn BrowserEngine> {
        Box::new(ChromiumoxideEngine::new(browser))
    }

    /// Wrap a chromiumoxide Page in the PageHandle trait
    #[deprecated(
        since = "0.2.0",
        note = "Use ChromiumoxidePage::new() from riptide_browser directly"
    )]
    pub fn wrap_page(page: Page) -> Box<dyn riptide_browser::BrowserEngine> {
        Box::new(ChromiumoxidePage::new(page)) as Box<dyn riptide_browser::BrowserEngine>
    }
}
