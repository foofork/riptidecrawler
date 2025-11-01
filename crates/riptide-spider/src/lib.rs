//! # Riptide Spider
//!
//! Spider/crawler engine for the RipTide web scraping framework.
//! This crate provides sophisticated web crawling capabilities with:
//!
//! - **Frontier-based URL queue management**: Efficient URL prioritization and deduplication
//! - **Multiple crawling strategies**: BFS, DFS, Best-First with pluggable algorithms
//! - **Adaptive stopping**: Content-based crawl termination
//! - **Budget controls**: Time, depth, and page count limits
//! - **Rate limiting**: Respectful crawling with configurable delays
//! - **Session persistence**: Support for authenticated crawling
//! - **Query-aware crawling**: Relevance-based URL prioritization
//!
//! ## Architecture
//!
//! The spider module is extracted from riptide-core to maintain separation of concerns:
//! - **riptide-spider**: Crawling logic (this crate)
//! - **riptide-fetch**: HTTP/network layer
//! - **riptide-extraction**: Content parsing and extraction
//! - **riptide-core**: Orchestration and coordination

pub mod adaptive_stop;
pub mod budget;
pub mod config;
pub mod core;
pub mod frontier;
pub mod memory_manager;
pub mod query_aware;
pub mod query_aware_benchmark;
pub mod query_aware_tests;
pub mod robots;
pub mod session;
pub mod sitemap;
pub mod strategy;
pub mod types;
pub mod url_utils;
pub mod wasm_validation;

// Re-export circuit breaker from riptide-types
pub use riptide_types::reliability::circuit::CircuitBreaker;

// Re-export main types
pub use adaptive_stop::AdaptiveStopEngine;
pub use budget::BudgetManager;
pub use config::SpiderConfig;
pub use core::{CrawlState, PerformanceMetrics, Spider, SpiderResult};
pub use frontier::FrontierManager;
pub use query_aware::{
    BM25Scorer, ContentSimilarityAnalyzer, DomainDiversityAnalyzer, QueryAwareConfig,
    QueryAwareScorer, QueryAwareStats, UrlSignalAnalyzer,
};
pub use session::SessionManager;
pub use sitemap::SitemapParser;
pub use strategy::{CrawlingStrategy, StrategyEngine};
pub use types::*;

#[cfg(test)]
mod tests;
