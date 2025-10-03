//! Spider Integration Module for Deep Crawling
//!
//! This module provides sophisticated web crawling capabilities with:
//! - Frontier-based URL queue management
//! - Multiple crawling strategies (BFS, DFS, Best-First)
//! - Adaptive stopping based on content analysis
//! - Budget controls and rate limiting
//! - Session persistence for authenticated crawling

pub mod adaptive_stop;
pub mod budget;
pub mod config;
pub mod core;
pub mod frontier;
pub mod query_aware;
pub mod query_aware_benchmark;
pub mod query_aware_tests;
pub mod session;
pub mod sitemap;
pub mod strategy;
pub mod types;
pub mod url_utils;

// Re-export main types
pub use adaptive_stop::AdaptiveStopEngine;
pub use budget::BudgetManager;
pub use config::SpiderConfig;
pub use core::{CrawlState, PerformanceMetrics, Spider, SpiderResult};
pub use frontier::FrontierManager;
pub use query_aware::{QueryAwareConfig, QueryAwareScorer, QueryAwareStats};
pub use session::SessionManager;
pub use sitemap::SitemapParser;
pub use strategy::{CrawlingStrategy, StrategyEngine};
pub use types::*;

#[cfg(test)]
mod tests;
