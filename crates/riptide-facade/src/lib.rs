//! # Riptide Facade
//!
//! High-level facade API for the Riptide web scraping framework.
//! Provides a simplified, user-friendly interface for common scraping tasks.
//!
//! ## Features
//!
//! - **Builder Pattern**: Fluent API for configuring scrapers
//! - **Type Safety**: Strong typing with compile-time guarantees
//! - **Extensible**: Easy to add new facade types
//! - **Error Handling**: Comprehensive error types
//!
//! ## Example
//!
//! ```no_run
//! use riptide_facade::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), RiptideError> {
//!     let scraper = Riptide::builder()
//!         .user_agent("MyBot/1.0")
//!         .timeout_secs(30)
//!         .build_scraper()
//!         .await?;
//!
//!     let html = scraper.fetch_html("https://example.com").await?;
//!     println!("Fetched {} bytes", html.len());
//!
//!     Ok(())
//! }
//! ```

pub mod builder;
pub mod config;
pub mod error;
pub mod facades;
pub mod prelude;

// Re-export core types
pub use builder::RiptideBuilder;
pub use config::RiptideConfig;
pub use error::{RiptideError, RiptideResult};
pub use facades::{
    BrowserAction, BrowserFacade, BrowserSession, Cookie, ImageFormat, PipelineFacade,
    ScraperFacade, ScreenshotOptions,
};

/// Main entry point for the Riptide facade API.
pub struct Riptide;

impl Riptide {
    /// Create a new builder for configuring Riptide.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use riptide_facade::Riptide;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let scraper = Riptide::builder()
    ///     .user_agent("MyBot/1.0")
    ///     .build_scraper()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder() -> RiptideBuilder {
        RiptideBuilder::new()
    }
}
