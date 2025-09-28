//! Spider Integration Module for Deep Crawling
//!
//! This module provides sophisticated web crawling capabilities with:
//! - Frontier-based URL queue management
//! - Multiple crawling strategies (BFS, DFS, Best-First)
//! - Adaptive stopping based on content analysis
//! - Budget controls and rate limiting
//! - Session persistence for authenticated crawling

pub mod frontier;
pub mod strategy;
pub mod budget;
pub mod adaptive_stop;
pub mod url_utils;
pub mod session;
pub mod sitemap;
pub mod config;
pub mod core;
pub mod types;
pub mod query_aware;

// Re-export main types
pub use config::SpiderConfig;
pub use core::{Spider, SpiderResult, CrawlState, PerformanceMetrics};
pub use types::*;
pub use frontier::FrontierManager;
pub use strategy::{CrawlingStrategy, StrategyEngine};
pub use budget::BudgetManager;
pub use adaptive_stop::AdaptiveStopEngine;
pub use session::SessionManager;
pub use sitemap::SitemapParser;
pub use query_aware::{QueryAwareConfig, QueryAwareScorer, QueryAwareStats};

#[cfg(test)]
mod tests;
