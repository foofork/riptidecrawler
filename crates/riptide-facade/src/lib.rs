//! # Riptide Facade - Application Layer (Use-Cases)
//!
//! This crate contains **application use-cases** that orchestrate domain logic via ports.
//! It represents the Application Layer in our hexagonal architecture.
//!
//! ## Architectural Rules
//!
//! **FORBIDDEN in this crate:**
//! - ❌ NO HTTP types (actix_web, hyper, axum, etc.)
//! - ❌ NO database types (sqlx, postgres, etc.)
//! - ❌ NO serialization formats (serde_json::Value - use typed DTOs)
//! - ❌ NO SDK/client types (redis, reqwest, etc.)
//! - ❌ NO infrastructure implementations
//!
//! ## What Lives Here
//!
//! **ALLOWED in this crate:**
//! - ✅ Use-case orchestration (workflows, transactions)
//! - ✅ Cross-cutting concerns (retry coordination, timeout management)
//! - ✅ Authorization policies (tenant scoping, RBAC)
//! - ✅ Idempotency management
//! - ✅ Domain event emission
//! - ✅ Transactional outbox writes
//! - ✅ Backpressure and cancellation token management
//! - ✅ Business metrics collection
//!
//! ## Dependencies
//!
//! This crate ONLY depends on:
//! - `riptide-types` (for domain types and port traits)
//! - Common utilities: `riptide-config`, `riptide-events`, `riptide-monitoring`
//! - NO infrastructure crates (riptide-reliability, riptide-cache, riptide-browser, etc.)
//!
//! ## Layer Boundary
//!
//! ```text
//! API Layer (riptide-api)
//!       ↓ calls
//! APPLICATION LAYER (riptide-facade) ← YOU ARE HERE
//!       ↓ uses ports (traits)
//! Domain Layer (riptide-types)
//!       ↑ implemented by
//! Infrastructure Layer (riptide-reliability, riptide-cache, etc.)
//! ```
//!
//! Infrastructure implementations are injected via dependency injection at the
//! composition root (`ApplicationContext` in riptide-api).
//!
//! ## Example Use-Case
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
//!
//! ## Port-Based Design
//!
//! Facades receive dependencies as port traits (not concrete types):
//!
//! ```rust,ignore
//! use riptide_types::ports::{Repository, EventBus, IdempotencyStore};
//!
//! pub struct ExtractionFacade {
//!     browser: Arc<dyn BrowserDriver>,
//!     cache: Arc<dyn CacheStorage>,
//!     events: Arc<dyn EventBus>,
//!     idempotency: Arc<dyn IdempotencyStore>,
//! }
//!
//! impl ExtractionFacade {
//!     pub async fn extract(&self, url: &str) -> Result<ExtractedData> {
//!         // 1. Check idempotency
//!         let token = self.idempotency.try_acquire(url, ttl).await?;
//!
//!         // 2. Check cache
//!         if let Some(cached) = self.cache.get(url).await? {
//!             return Ok(deserialize(cached));
//!         }
//!
//!         // 3. Execute extraction
//!         let session = self.browser.navigate(url).await?;
//!         let data = extract_data(&session).await?;
//!
//!         // 4. Emit domain event
//!         self.events.publish(ExtractionCompleted::new(url)).await?;
//!
//!         // 5. Cache result
//!         self.cache.set(url, serialize(&data), ttl).await?;
//!
//!         // 6. Release idempotency lock
//!         self.idempotency.release(token).await?;
//!
//!         Ok(data)
//!     }
//! }
//! ```
//!
//! ## Testing
//!
//! Use in-memory port implementations for fast, deterministic tests:
//!
//! ```rust,ignore
//! use riptide_types::ports::{InMemoryCache, FakeClock, DeterministicEntropy};
//!
//! #[tokio::test]
//! async fn test_extraction_with_cache() {
//!     let cache = Arc::new(InMemoryCache::new());
//!     let clock = Arc::new(FakeClock::at_epoch());
//!
//!     let facade = ExtractionFacade {
//!         cache,
//!         clock,
//!         // ... other test doubles
//!     };
//!
//!     // Deterministic, fast test with no real infrastructure
//!     let result = facade.extract("https://example.com").await?;
//!     assert_eq!(result.title, "Expected Title");
//! }
//! ```

pub mod authorization;
pub mod builder;
pub mod config;
pub mod dto;
pub mod error;
pub mod facades;
pub mod metrics;
pub mod prelude;
pub mod traits;
pub mod workflows;

#[cfg(test)]
mod tests;

// Re-export important domain types for API layer
pub use facades::pipeline::FetchOperation;

// Re-export core types
pub use builder::RiptideBuilder;
pub use config::RiptideConfig;
pub use dto::{Document, Event, Product, StructuredData, ToDto};
pub use error::{RiptideError, RiptideResult};
pub use facades::{
    BrowserAction, BrowserFacade, BrowserSession, Cookie, CrawlFacade, CrawlMode, CrawlResult,
    CrawlSummary, ImageFormat, PipelineFacade, ScraperFacade, ScreenshotOptions, SpiderFacade,
    SpiderPreset,
};
pub use traits::{
    Chainable, Content, ExtractChain, ExtractOpts, ExtractionStrategy, Extractor, Spider,
    SpiderOpts,
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
