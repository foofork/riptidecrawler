//! RipTide Headless Browser HTTP API
//!
//! This library provides HTTP API endpoints for headless browser operations.
//! It wraps the unified `riptide-browser` crate functionality and exposes it via HTTP.
//!
//! ## Architecture
//!
//! **Phase 3 Task 4.4**: Browser code consolidated into `riptide-browser` crate.
//! This crate now focuses exclusively on:
//! - HTTP API endpoints (cdp.rs)
//! - Dynamic content handling (dynamic.rs)
//! - Request/response models (models.rs)
//!
//! All browser engine functionality (pool, launcher, CDP) is now in `riptide-browser`.
//!
//! ## Usage
//!
//! ```no_run
//! # use riptide_headless::{HeadlessLauncher, cdp};
//! # async fn example() -> anyhow::Result<()> {
//! // Create launcher with pooling (from riptide-browser)
//! let launcher = HeadlessLauncher::new().await?;
//!
//! // Launch a page with stealth
//! let session = launcher.launch_page_default("https://example.com").await?;
//!
//! // Access the page
//! let page = session.page();
//! let content = page.content().await?;
//!
//! // Session automatically returns browser to pool when dropped
//! # Ok(())
//! # }
//! ```

// ========================================
// Re-export browser functionality from riptide-browser
// ========================================

// P3-T4.4: Use unified riptide-browser facade
// The circular dependency is resolved by riptide-browser optionally depending on riptide-headless
// only when the 'headless' feature is enabled (for dynamic rendering integration)

// Core browser pool and launcher (from riptide-browser)
pub use riptide_browser::{
    // Models
    models,
    // Browser pool
    BrowserCheckout,
    BrowserPool,
    BrowserPoolConfig,
    // CDP connection pool
    CdpConnectionPool,
    CdpPoolConfig,
    ConnectionHealth,
    ConnectionStats,
    // Launcher
    HeadlessLauncher,
    LaunchSession,
    LauncherConfig,
    LauncherStats,
    PoolEvent,
    PoolStats,
};

// ========================================
// Unique riptide-headless functionality
// ========================================

// HTTP API module (depends on riptide-browser components)
pub mod cdp;

// Dynamic content handling
pub mod dynamic;

// Re-export dynamic types for convenience
pub use dynamic::{DynamicConfig, PageAction, ScrollConfig, ViewportConfig, WaitCondition};

// Backward compatibility: Module re-exports for existing code
pub mod pool {
    //! Browser pool module - RE-EXPORTED from riptide-browser
    //!
    //! Phase 3 Task 4.4: Duplicates removed (-1,325 LOC), now re-exports from riptide-browser
    pub use riptide_browser::{
        BrowserCheckout, BrowserPool, BrowserPoolConfig, PoolEvent, PoolStats,
    };
}

pub mod cdp_pool {
    //! CDP connection pool module - RE-EXPORTED from riptide-browser
    //!
    //! Phase 3 Task 4.4: Duplicates removed (-493 LOC), now re-exports from riptide-browser
    pub use riptide_browser::{
        CdpCommand, CdpConnectionPool, CdpPoolConfig, ConnectionHealth, ConnectionStats,
        PooledConnection,
    };
}

// Launcher module removed - now re-exported at crate root from riptide-browser
