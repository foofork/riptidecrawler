//! Spider facade for web crawling operations.
//!
//! The SpiderFacade provides a simplified interface for multi-page web crawling
//! with frontier management, budgets, and various crawling strategies.
//!
//! # Features
//!
//! - **Budget Controls**: Limit crawls by pages, depth, or time
//! - **Crawling Strategies**: BFS, DFS, Best-First algorithms
//! - **Query-Aware Crawling**: Relevance-based URL prioritization
//! - **Frontier Access**: Inspect URL queue statistics
//!
//! # Example
//!
//! ```no_run
//! use riptide_facade::{Riptide, CrawlBudget};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let spider = Riptide::builder().build_spider().await?;
//!
//! // Crawl with budget
//! let budget = CrawlBudget {
//!     max_pages: Some(100),
//!     max_depth: Some(3),
//!     timeout_secs: Some(300),
//! };
//!
//! let result = spider.crawl("https://example.com", budget).await?;
//! println!("Crawled {} pages", result.total_pages);
//! # Ok(())
//! # }
//! ```

use crate::config::RiptideConfig;
use crate::error::{Result, RiptideError};
use crate::runtime::RiptideRuntime;
use riptide_spider::{
    BudgetManager, CrawlingStrategy, FrontierManager, QueryAwareConfig, QueryAwareScorer, Spider,
    SpiderConfig,
};
use riptide_types::ExtractedDoc;
use std::sync::Arc;
use std::time::Duration;

/// Spider facade for multi-page web crawling.
///
/// Provides simplified access to the riptide-spider crate's crawling capabilities
/// with sensible defaults and an ergonomic API.
#[derive(Clone)]
pub struct SpiderFacade {
    config: RiptideConfig,
    runtime: Arc<RiptideRuntime>,
    spider: Arc<Spider>,
    frontier: Arc<FrontierManager>,
}

impl SpiderFacade {
    /// Create a new SpiderFacade instance.
    ///
    /// # Arguments
    ///
    /// * `config` - Riptide configuration
    /// * `runtime` - Shared runtime instance
    pub(crate) async fn new(config: RiptideConfig, runtime: Arc<RiptideRuntime>) -> Result<Self> {
        // Create spider configuration from Riptide config
        let spider_config = SpiderConfig {
            user_agent: config.user_agent.clone(),
            max_concurrent_requests: config.max_concurrent_requests,
            respect_robots_txt: config.respect_robots_txt,
            rate_limit_per_second: config.rate_limit.unwrap_or(10),
            ..Default::default()
        };

        let spider = Arc::new(
            Spider::new(spider_config)
                .map_err(|e| RiptideError::spider(format!("Failed to create spider: {}", e)))?,
        );

        let frontier = Arc::new(FrontierManager::new());

        Ok(Self {
            config,
            runtime,
            spider,
            frontier,
        })
    }

    /// Crawl a website starting from the given URL with budget constraints.
    ///
    /// # Arguments
    ///
    /// * `start_url` - The URL to start crawling from
    /// * `budget` - Budget constraints (max pages, depth, timeout)
    ///
    /// # Returns
    ///
    /// A `CrawlResult` containing crawled pages and statistics.
    ///
    /// # Errors
    ///
    /// - Invalid URL (missing protocol, malformed)
    /// - Network errors during crawling
    /// - Budget exceeded
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use riptide_facade::{Riptide, CrawlBudget};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let spider = Riptide::builder().build_spider().await?;
    /// let budget = CrawlBudget::pages(50);
    /// let result = spider.crawl("https://docs.rs", budget).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn crawl(&self, start_url: &str, budget: CrawlBudget) -> Result<CrawlResult> {
        // Validate URL
        self.validate_url(start_url)?;

        // Create budget manager
        let budget_mgr = self.create_budget_manager(budget);

        // Perform crawl
        let spider_result = self
            .spider
            .crawl(start_url, Some(budget_mgr))
            .await
            .map_err(|e| RiptideError::spider(format!("Crawl failed: {}", e)))?;

        Ok(CrawlResult {
            pages: spider_result.pages,
            total_pages: spider_result.pages.len(),
            urls_visited: spider_result.urls_visited,
            urls_queued: spider_result.urls_queued,
        })
    }

    /// Crawl with a specific crawling strategy.
    ///
    /// # Arguments
    ///
    /// * `start_url` - The URL to start crawling from
    /// * `strategy` - Crawling strategy (BFS, DFS, BestFirst)
    /// * `budget` - Budget constraints
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use riptide_facade::{Riptide, CrawlBudget};
    /// # use riptide_spider::CrawlingStrategy;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let spider = Riptide::builder().build_spider().await?;
    /// let result = spider.crawl_with_strategy(
    ///     "https://example.com",
    ///     CrawlingStrategy::DepthFirst,
    ///     CrawlBudget::depth(5)
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn crawl_with_strategy(
        &self,
        start_url: &str,
        strategy: CrawlingStrategy,
        budget: CrawlBudget,
    ) -> Result<CrawlResult> {
        self.validate_url(start_url)?;

        let budget_mgr = self.create_budget_manager(budget);

        let spider_result = self
            .spider
            .crawl_with_strategy(start_url, strategy, Some(budget_mgr))
            .await
            .map_err(|e| RiptideError::spider(format!("Strategy crawl failed: {}", e)))?;

        Ok(CrawlResult {
            pages: spider_result.pages,
            total_pages: spider_result.pages.len(),
            urls_visited: spider_result.urls_visited,
            urls_queued: spider_result.urls_queued,
        })
    }

    /// Crawl with query-aware URL prioritization.
    ///
    /// URLs are scored based on relevance to the query and prioritized accordingly.
    ///
    /// # Arguments
    ///
    /// * `start_url` - The URL to start crawling from
    /// * `query` - Search query for relevance scoring
    /// * `budget` - Budget constraints
    ///
    /// # Errors
    ///
    /// Returns an error if the query is empty or whitespace-only.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use riptide_facade::{Riptide, CrawlBudget};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let spider = Riptide::builder().build_spider().await?;
    /// let result = spider.query_aware_crawl(
    ///     "https://docs.rs",
    ///     "async programming",
    ///     CrawlBudget::pages(20)
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_aware_crawl(
        &self,
        start_url: &str,
        query: &str,
        budget: CrawlBudget,
    ) -> Result<CrawlResult> {
        self.validate_url(start_url)?;

        // Validate query
        if query.trim().is_empty() {
            return Err(RiptideError::spider(
                "Query cannot be empty for query-aware crawl",
            ));
        }

        let budget_mgr = self.create_budget_manager(budget);

        // Create query-aware scorer
        let qa_config = QueryAwareConfig::default();
        let scorer = QueryAwareScorer::new(query, qa_config);

        let spider_result = self
            .spider
            .crawl_query_aware(start_url, scorer, Some(budget_mgr))
            .await
            .map_err(|e| RiptideError::spider(format!("Query-aware crawl failed: {}", e)))?;

        Ok(CrawlResult {
            pages: spider_result.pages,
            total_pages: spider_result.pages.len(),
            urls_visited: spider_result.urls_visited,
            urls_queued: spider_result.urls_queued,
        })
    }

    /// Get access to the frontier manager for inspecting URL queue.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use riptide_facade::Riptide;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let spider = Riptide::builder().build_spider().await?;
    /// let frontier = spider.frontier();
    /// println!("Queued: {}, Visited: {}",
    ///     frontier.queued_count(),
    ///     frontier.visited_count()
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn frontier(&self) -> FrontierStats {
        FrontierStats {
            frontier: Arc::clone(&self.frontier),
        }
    }

    // Helper methods

    fn validate_url(&self, url: &str) -> Result<()> {
        if url.trim().is_empty() {
            return Err(RiptideError::spider("URL cannot be empty"));
        }

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(RiptideError::spider(format!(
                "Invalid URL '{}': must start with http:// or https://",
                url
            )));
        }

        Ok(())
    }

    fn create_budget_manager(&self, budget: CrawlBudget) -> BudgetManager {
        let mut mgr = BudgetManager::new();

        if let Some(max_pages) = budget.max_pages {
            mgr = mgr.with_max_pages(max_pages);
        }

        if let Some(max_depth) = budget.max_depth {
            mgr = mgr.with_max_depth(max_depth);
        }

        if let Some(timeout_secs) = budget.timeout_secs {
            mgr = mgr.with_timeout(Duration::from_secs(timeout_secs));
        }

        mgr
    }
}

/// Crawl budget for controlling crawl scope.
///
/// All constraints are optional. If not specified, the crawl will continue
/// until no more URLs are available or an error occurs.
#[derive(Debug, Clone, Default)]
pub struct CrawlBudget {
    /// Maximum number of pages to crawl
    pub max_pages: Option<usize>,
    /// Maximum crawl depth from start URL
    pub max_depth: Option<u32>,
    /// Maximum crawl time in seconds
    pub timeout_secs: Option<u64>,
}

impl CrawlBudget {
    /// Create a budget limited by max pages only.
    pub fn pages(max_pages: usize) -> Self {
        Self {
            max_pages: Some(max_pages),
            max_depth: None,
            timeout_secs: None,
        }
    }

    /// Create a budget limited by max depth only.
    pub fn depth(max_depth: u32) -> Self {
        Self {
            max_pages: None,
            max_depth: Some(max_depth),
            timeout_secs: None,
        }
    }

    /// Create a budget limited by timeout only.
    pub fn timeout(timeout_secs: u64) -> Self {
        Self {
            max_pages: None,
            max_depth: None,
            timeout_secs: Some(timeout_secs),
        }
    }
}

/// Result of a crawl operation.
#[derive(Debug, Clone)]
pub struct CrawlResult {
    /// Crawled pages with extracted content
    pub pages: Vec<ExtractedDoc>,
    /// Total number of pages crawled
    pub total_pages: usize,
    /// Number of URLs visited
    pub urls_visited: usize,
    /// Number of URLs queued
    pub urls_queued: usize,
}

/// Statistics for the frontier URL queue.
#[derive(Clone)]
pub struct FrontierStats {
    frontier: Arc<FrontierManager>,
}

impl FrontierStats {
    /// Number of URLs currently queued for crawling.
    pub fn queued_count(&self) -> usize {
        self.frontier.queued_count()
    }

    /// Number of URLs already visited.
    pub fn visited_count(&self) -> usize {
        self.frontier.visited_count()
    }

    /// Total number of URLs discovered.
    pub fn total_discovered(&self) -> usize {
        self.queued_count() + self.visited_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crawl_budget_builders() {
        let budget = CrawlBudget::pages(100);
        assert_eq!(budget.max_pages, Some(100));
        assert_eq!(budget.max_depth, None);

        let budget = CrawlBudget::depth(5);
        assert_eq!(budget.max_depth, Some(5));
        assert_eq!(budget.max_pages, None);

        let budget = CrawlBudget::timeout(300);
        assert_eq!(budget.timeout_secs, Some(300));
        assert_eq!(budget.max_pages, None);
    }

    #[test]
    fn test_url_validation() {
        let config = RiptideConfig::default();
        let runtime = Arc::new(RiptideRuntime::new(config.clone()).unwrap());

        // Create spider with runtime
        let spider_config = SpiderConfig::default();
        let spider = Arc::new(Spider::new(spider_config).unwrap());
        let frontier = Arc::new(FrontierManager::new());

        let facade = SpiderFacade {
            config,
            runtime,
            spider,
            frontier,
        };

        // Valid URLs
        assert!(facade.validate_url("https://example.com").is_ok());
        assert!(facade.validate_url("http://example.com").is_ok());

        // Invalid URLs
        assert!(facade.validate_url("").is_err());
        assert!(facade.validate_url("   ").is_err());
        assert!(facade.validate_url("example.com").is_err());
        assert!(facade.validate_url("ftp://example.com").is_err());
    }

    #[test]
    fn test_empty_query_validation() {
        let config = RiptideConfig::default();
        let runtime = Arc::new(RiptideRuntime::new(config.clone()).unwrap());

        let spider_config = SpiderConfig::default();
        let spider = Arc::new(Spider::new(spider_config).unwrap());
        let frontier = Arc::new(FrontierManager::new());

        let facade = SpiderFacade {
            config,
            runtime,
            spider,
            frontier,
        };

        // Empty query should error in query-aware crawl
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(facade.query_aware_crawl("https://example.com", "", CrawlBudget::pages(10)));

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Query cannot be empty"));
    }
}
