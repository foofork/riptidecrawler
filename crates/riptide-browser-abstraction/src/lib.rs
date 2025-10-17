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

// Modules
mod traits;
mod error;
mod params;

// Conditional compilation to avoid chromiumoxide name collision
// spider_chrome exports its library as "chromiumoxide", which conflicts with standard chromiumoxide
// Solution: Only compile one implementation at a time
#[cfg(not(feature = "spider"))]
mod chromiumoxide_impl;
#[cfg(not(feature = "spider"))]
mod factory;

#[cfg(feature = "spider")]
mod spider_impl;

// Public exports
pub use error::{AbstractionError, AbstractionResult};
pub use params::{NavigateParams, PdfParams, ScreenshotFormat, ScreenshotParams, WaitUntil};
pub use traits::{BrowserEngine, EngineType, PageHandle};

// Engine-specific exports
#[cfg(not(feature = "spider"))]
pub use chromiumoxide_impl::{ChromiumoxideEngine, ChromiumoxidePage};
#[cfg(not(feature = "spider"))]
pub use factory::create_engine;

#[cfg(feature = "spider")]
pub use spider_impl::{SpiderChromeEngine, SpiderChromePage};

#[cfg(test)]
mod tests;
