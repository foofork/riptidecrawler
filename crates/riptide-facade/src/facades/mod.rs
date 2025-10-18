//! Domain-specific facades for Riptide functionality.

#[cfg(feature = "scraper")]
pub mod scraper;

#[cfg(feature = "spider")]
pub mod spider;

#[cfg(feature = "browser")]
pub mod browser;

#[cfg(feature = "extractor")]
pub mod extractor;

#[cfg(feature = "intelligence")]
pub mod intelligence;

#[cfg(feature = "security")]
pub mod security;

#[cfg(feature = "monitoring")]
pub mod monitoring;

#[cfg(feature = "cache")]
pub mod cache;
