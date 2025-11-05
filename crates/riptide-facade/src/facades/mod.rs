//! Facade implementations for different use cases.
//!
//! This module contains specialized facades that provide simplified
//! interfaces for common web scraping tasks.

pub mod browser;
pub mod crawl_facade;
pub mod extractor;
pub mod intelligence;
pub mod pipeline;
pub mod scraper;
pub mod search;
pub mod spider;

pub use browser::{
    BrowserAction, BrowserFacade, BrowserSession, Cookie, ImageFormat, ScreenshotOptions,
};
pub use crawl_facade::{CrawlFacade, CrawlMode, CrawlResult};
pub use extractor::{
    ExtractedData, ExtractionFacade, ExtractionStrategy, FieldSpec, FieldType,
    HtmlExtractionOptions, PdfExtractionOptions, Schema,
};
pub use pipeline::PipelineFacade;
pub use scraper::ScraperFacade;
pub use search::SearchFacade;
pub use spider::{CrawlSummary, SpiderFacade, SpiderPreset};
