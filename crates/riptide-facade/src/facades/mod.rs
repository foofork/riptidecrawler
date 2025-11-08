//! Facade implementations for different use cases.
//!
//! This module contains specialized facades that provide simplified
//! interfaces for common web scraping tasks.

pub mod browser;
pub mod browser_metrics;
pub mod crawl_facade;
pub mod extraction;
pub mod extraction_authz;
pub mod extraction_metrics;
pub mod extractor;
pub mod intelligence;
pub mod pdf;
pub mod pipeline;
pub mod pipeline_metrics;
#[cfg(feature = "llm")]
pub mod profile;
pub mod render_strategy;
pub mod scraper;
pub mod search;
pub mod session;
pub mod session_metrics;
pub mod spider;
pub mod table;

// Re-export metrics for facade use
pub use crate::metrics::BusinessMetrics;
pub use browser_metrics::MetricsBrowserFacade;
pub use extraction_metrics::{ExtractionMetricsExt, MetricsExtractionFacade};
pub use pipeline_metrics::MetricsPipelineFacade;
pub use session_metrics::MetricsSessionFacade;

pub use browser::{
    BrowserAction, BrowserFacade, BrowserSession, Cookie, ImageFormat, ScreenshotOptions,
};
pub use crawl_facade::{CrawlFacade, CrawlMode, CrawlResult};
pub use extraction::{ExtractedDoc, UrlExtractionFacade, UrlExtractionOptions};
pub use extraction_authz::AuthorizedExtractionFacade;
pub use extractor::{
    ExtractedData, ExtractionFacade, ExtractionStrategy, FieldSpec, FieldType,
    HtmlExtractionOptions, PdfExtractionOptions, Schema,
};
pub use pdf::{
    EnhancedProgressUpdate, PdfFacade, PdfInput, PdfMetadata, PdfProcessOptions, PdfProcessResult,
    ProcessingStats,
};
pub use pipeline::PipelineFacade;
#[cfg(feature = "llm")]
pub use profile::{
    BatchCreateResult, BatchFailure, ProfileConfigRequest, ProfileFacade, ProfileMetadataRequest,
};
pub use render_strategy::RenderStrategyFacade;
pub use scraper::ScraperFacade;
pub use search::SearchFacade;
pub use session::{SessionConfig, SessionEvent, SessionFacade};
pub use spider::{CrawlSummary, SpiderFacade, SpiderPreset};
pub use table::{
    TableCacheService, TableExtractionOptions as FacadeTableExtractionOptions, TableFacade,
    TableMetadata as FacadeTableMetadata, TableSummary as FacadeTableSummary,
};
