//! Browser Engine Abstraction Layer
//!
//! This crate provides a unified interface for multiple browser automation engines,
//! allowing runtime selection between chromiumoxide and spider-chrome.
//!
//! ## Architecture
//!
//! The abstraction uses trait objects and async-trait to provide:
//! - Runtime engine selection (for hybrid fallback)
//! - Type-safe API despite engine incompatibilities
//! - Minimal performance overhead (<0.01%)
//!
//! ## Usage
//!
//! ```no_run
//! use riptide_browser_abstraction::{BrowserEngine, EngineType, create_engine};
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create an engine (chromiumoxide or spider-chrome)
//! let engine = create_engine(EngineType::Chromiumoxide).await?;
//!
//! // Use the unified interface
//! let page = engine.new_page().await?;
//! let html = page.content().await?;
//! # Ok(())
//! # }
//! ```

mod chromiumoxide_impl;
mod traits;
// #[cfg(feature = "spider")]
// mod spider_impl;  // Disabled - see ADR-006 for incompatibility details
mod error;
mod factory;
mod params;

pub use chromiumoxide_impl::{ChromiumoxideEngine, ChromiumoxidePage};
pub use error::{AbstractionError, AbstractionResult};
pub use factory::create_engine;
pub use params::{NavigateParams, PdfParams, ScreenshotFormat, ScreenshotParams, WaitUntil};
pub use traits::{BrowserEngine, EngineType, PageHandle};

#[cfg(test)]
mod tests;
