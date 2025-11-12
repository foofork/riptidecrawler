//! Spider Engine Port - Trait abstraction for web crawling engines
//!
//! This port follows the Hexagonal Architecture pattern, providing a
//! backend-agnostic interface for web crawling operations. It enables
//! dependency inversion and facilitates testing with mock implementations.
//!
//! # Architecture
//!
//! ```text
//! Domain Layer (riptide-types)
//!     ↓ defines SpiderEngine trait
//! Infrastructure Layer (riptide-spider)
//!     ↓ implements Spider adapter
//! Composition Root (riptide-api)
//!     ↓ wires Arc<dyn SpiderEngine>
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_types::ports::SpiderEngine;
//!
//! async fn crawl_site(spider: &dyn SpiderEngine, seeds: Vec<Url>) -> Result<()> {
//!     let results = spider.crawl(seeds).await?;
//!     let state = spider.get_crawl_state().await;
//!     spider.stop().await?;
//!     Ok(())
//! }
//! ```

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;
use url::Url;

/// SpiderEngine trait for web crawling operations
///
/// This trait abstracts the spider/crawler engine, allowing different
/// implementations (depth-first, breadth-first, focused crawling, etc.)
/// while maintaining a consistent interface for the application layer.
///
/// # Design Principles
///
/// - **Async-first**: All operations are async for non-blocking I/O
/// - **Type-safe**: Uses strongly-typed parameters and results
/// - **Observable**: Provides crawl state and metrics
/// - **Controllable**: Supports pause/resume/stop operations
#[async_trait]
pub trait SpiderEngine: Send + Sync {
    /// Start crawling from seed URLs
    ///
    /// # Arguments
    ///
    /// * `seeds` - Initial URLs to start crawling from
    ///
    /// # Returns
    ///
    /// Crawl results including pages crawled, duration, and performance metrics
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let seeds = vec![Url::parse("https://example.com")?];
    /// let results = spider.crawl(seeds).await?;
    /// println!("Crawled {} pages", results.pages_crawled);
    /// ```
    async fn crawl(&self, seeds: Vec<Url>) -> Result<CrawlResults>;

    /// Get current crawl state (active, URLs queued, etc.)
    ///
    /// This is useful for monitoring and displaying progress during a crawl.
    ///
    /// # Returns
    ///
    /// Current state snapshot including activity status, queue sizes, and domains
    async fn get_crawl_state(&self) -> CrawlState;

    /// Stop crawl and cleanup
    ///
    /// Gracefully terminates the crawl, ensuring all resources are released.
    /// This is safe to call multiple times.
    async fn stop(&self) -> Result<()>;
}

/// Crawl state information
///
/// Represents a snapshot of the crawler's current state, useful for
/// monitoring, progress tracking, and determining when to stop.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrawlState {
    /// Whether crawling is currently active
    pub active: bool,

    /// Total pages crawled so far
    pub pages_crawled: u64,

    /// Total pages that failed during crawl
    pub pages_failed: u64,

    /// Number of URLs currently in the frontier queue
    pub frontier_size: usize,

    /// Set of domains being actively crawled
    pub active_domains: HashSet<String>,
}

/// Crawl results
///
/// Summary of a completed (or stopped) crawl operation, including
/// performance metrics and discovered content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlResults {
    /// Total number of pages successfully crawled
    pub pages_crawled: u64,

    /// Total number of pages that failed
    pub pages_failed: u64,

    /// Duration of the crawl operation
    pub duration: Duration,

    /// Reason the crawl stopped (budget exhausted, adaptive stop, manual, etc.)
    pub stop_reason: String,

    /// Performance metrics for this crawl
    pub performance: PerformanceMetrics,

    /// List of domains that were crawled
    pub domains: Vec<String>,

    /// URLs discovered during the crawl (may be capped based on config)
    pub discovered_urls: Vec<String>,
}

/// Performance metrics for crawl operations
///
/// Tracks performance characteristics of the crawl, useful for
/// optimization and capacity planning.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Throughput: pages crawled per second
    pub pages_per_second: f64,

    /// Average response time per request
    #[serde(with = "duration_serde")]
    pub avg_response_time: Duration,

    /// Memory usage in bytes
    pub memory_usage: usize,

    /// Error rate as a percentage (0.0 to 1.0)
    pub error_rate: f64,
}

/// Serde module for Duration serialization
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs_f64().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = f64::deserialize(deserializer)?;
        Ok(Duration::from_secs_f64(secs))
    }
}
