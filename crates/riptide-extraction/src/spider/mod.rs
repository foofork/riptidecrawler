//! DOM-specific spider functionality for HTML content processing
//!
//! This module provides HTML-specific crawling capabilities extracted from riptide-core:
//! - Link extraction from HTML content
//! - Form detection and parsing
//! - Meta tag extraction
//! - HTML content analysis for spider optimization

pub mod dom_crawler;
pub mod link_extractor;
pub mod form_parser;
pub mod meta_extractor;
pub mod traits;

#[cfg(test)]
mod tests;

// Re-export main types and traits
pub use traits::{DomSpider, DomCrawlerResult, FormData, MetaData};
pub use dom_crawler::HtmlDomCrawler;
pub use link_extractor::HtmlLinkExtractor;
pub use form_parser::HtmlFormParser;
pub use meta_extractor::HtmlMetaExtractor;