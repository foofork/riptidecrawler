//! # RipTide Headless Hybrid
//!
//! A hybrid headless browser launcher that combines spider-chrome for browser automation
//! with EventMesh stealth features for anti-detection.
//!
//! ## Architecture
//!
//! This crate provides:
//! - **HybridHeadlessLauncher**: Main launcher using spider-chrome internally
//! - **LaunchSession**: Compatible with existing EventMesh API
//! - **Stealth Middleware**: Applies EventMesh stealth features to spider-chrome pages
//!
//! ## Usage
//!
//! ```no_run
//! # use riptide_headless_hybrid::{HybridHeadlessLauncher, LauncherConfig};
//! # use riptide_stealth::StealthPreset;
//! # async fn example() -> anyhow::Result<()> {
//! // Create launcher with default config
//! let launcher = HybridHeadlessLauncher::new().await?;
//!
//! // Launch a page with stealth
//! let session = launcher.launch_page("https://example.com", Some(StealthPreset::Medium)).await?;
//!
//! // Access the page
//! let html = session.content().await?;
//!
//! // Session automatically cleaned up when dropped
//! # Ok(())
//! # }
//! ```

pub mod launcher;
pub mod models;
pub mod stealth_middleware;

// Re-export main types
pub use launcher::{HybridHeadlessLauncher, LaunchSession, LauncherConfig, LauncherStats};
pub use stealth_middleware::{apply_stealth, StealthMiddleware};

/// Version of this crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Crate name
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");
