//! Core traits for the Riptide facade
//!
//! This module provides the fundamental traits for composing spider and extractor operations:
//!
//! - [`Spider`] - URL discovery through crawling
//! - [`Extractor`] - Content extraction from web pages
//! - [`Chainable`] - Composition of spider and extractor via `.and_extract()`
//!
//! # Examples
//!
//! ```no_run
//! use riptide_facade::traits::{Spider, Extractor, Chainable, SpiderOpts};
//! use futures::StreamExt;
//!
//! # async fn example(spider: impl Spider, extractor: impl Extractor) -> Result<(), Box<dyn std::error::Error>> {
//! // Discover URLs
//! let urls = spider.crawl("https://example.com", SpiderOpts::default()).await?;
//!
//! // Chain with extraction
//! let docs = urls.and_extract(extractor);
//!
//! // Process documents
//! let docs: Vec<_> = docs.collect().await;
//! # Ok(())
//! # }
//! ```

mod spider;
mod extractor;
mod chainable;

pub mod mocks;

pub use spider::{Spider, SpiderOpts};
pub use extractor::{Extractor, ExtractOpts, Content, ExtractionStrategy};
pub use chainable::{Chainable, ExtractChain};
