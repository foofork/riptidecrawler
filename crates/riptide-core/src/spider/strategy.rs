use crate::spider::types::{CrawlRequest, ScoringConfig, Priority};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use url::Url;

/// Crawling strategy types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrawlingStrategy {
    /// Breadth-First Search - explores level by level
    BreadthFirst,
    /// Depth-First Search - follows link chains deeply
    DepthFirst,
    /// Best-First Search - uses scoring to prioritize URLs
    BestFirst {
        scoring_config: ScoringConfig,
    },
    /// Adaptive strategy that switches based on conditions
    Adaptive {
        primary: Box<CrawlingStrategy>,
        fallback: Box<CrawlingStrategy>,
        switch_criteria: AdaptiveCriteria,
    },
}

/// Criteria for adaptive strategy switching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveCriteria {
    /// Switch if frontier size exceeds this threshold
    pub max_frontier_size: usize,
    /// Switch if average depth exceeds this threshold
    pub max_average_depth: f64,
    /// Switch if success rate drops below this threshold
    pub min_success_rate: f64,
    /// Minimum pages before considering switch
    pub min_pages_for_switch: usize,
    /// Cooldown period between switches
    pub switch_cooldown_pages: usize,
}

impl Default for AdaptiveCriteria {
    fn default() -> Self {
        Self {
            max_frontier_size: 10000,
            max_average_depth: 5.0,
            min_success_rate: 0.7,
            min_pages_for_switch: 100,
            switch_cooldown_pages: 50,
        }
    }
}

/// Strategy execution context and metrics
#[derive(Debug, Clone)]
pub struct StrategyContext {
    /// Total pages crawled with current strategy
    pub pages_crawled: u64,
    /// Successful pages
    pub successful_pages: u64,
    /// Current frontier size
    pub frontier_size: usize,
    /// Average depth of requests in frontier
    pub average_depth: f64,
    /// Pages since last strategy switch
    pub pages_since_switch: u64,
    /// Current strategy name
    pub current_strategy: String,
    /// Switch history for analysis
    pub switch_history: Vec<StrategySwitch>,
}

/// Record of a strategy switch
#[derive(Debug, Clone)]
pub struct StrategySwitch {
    pub from_strategy: String,
    pub to_strategy: String,
    pub pages_crawled: u64,
    pub reason: String,
    pub timestamp: std::time::SystemTime,
}

impl Default for StrategyContext {
    fn default() -> Self {
        Self {
            pages_crawled: 0,
            successful_pages: 0,
            frontier_size: 0,
            average_depth: 0.0,
            pages_since_switch: 0,
            current_strategy: "Unknown".to_string(),
            switch_history: Vec::new(),
        }
    }
}

impl StrategyContext {
    /// Calculate current success rate
    pub fn success_rate(&self) -> f64 {
        if self.pages_crawled == 0 {
            1.0
        } else {
            self.successful_pages as f64 / self.pages_crawled as f64
        }
    }

    /// Record a crawl result
    pub fn record_crawl(&mut self, success: bool) {
        self.pages_crawled += 1;
        self.pages_since_switch += 1;
        if success {
            self.successful_pages += 1;
        }
    }

    /// Record a strategy switch
    pub fn record_switch(&mut self, from: &str, to: &str, reason: String) {
        let switch = StrategySwitch {
            from_strategy: from.to_string(),
            to_strategy: to.to_string(),
            pages_crawled: self.pages_crawled,
            reason,
            timestamp: std::time::SystemTime::now(),
        };
        self.switch_history.push(switch);
        self.pages_since_switch = 0;
        self.current_strategy = to.to_string();
    }
}

/// URL scoring function for best-first strategy
pub trait ScoringFunction: Send + Sync {
    fn score_url(&self, url: &Url, depth: u32, metadata: &HashMap<String, String>) -> f64;
}

/// Default URL scoring implementation
#[derive(Debug)]
pub struct DefaultScoring {
    config: ScoringConfig,
}

impl DefaultScoring {
    pub fn new(config: ScoringConfig) -> Self {
        Self { config }
    }
}

impl ScoringFunction for DefaultScoring {
    fn score_url(&self, url: &Url, depth: u32, metadata: &HashMap<String, String>) -> f64 {
        let mut score = 1.0;

        // Depth penalty (prefer shallower pages)
        score += self.config.depth_weight * depth as f64;

        // Path length penalty (prefer shorter paths)
        let path_length = url.path().len() as f64;
        score += self.config.path_length_weight * path_length;

        // Parameter penalty (prefer URLs with fewer parameters)
        let param_count = url.query_pairs().count() as f64;
        score += self.config.parameter_weight * param_count;

        // Domain scoring boost
        if let Some(host) = url.host_str() {
            if let Some(domain_score) = self.config.domain_scores.get(host) {
                score += domain_score;
            }
        }

        // File extension scoring
        if let Some(extension) = url.path().split('.').next_back() {
            if let Some(ext_score) = self.config.extension_scores.get(extension) {
                score += self.config.content_type_weight * ext_score;
            }
        }

        // Content type hints from metadata
        if let Some(content_type) = metadata.get("content_type") {
            if content_type.contains("text/html") {
                score += self.config.content_type_weight;
            } else if content_type.contains("application/pdf") {
                score += self.config.content_type_weight * 0.5;
            }
        }

        // Ensure non-negative score
        score.max(0.0_f64)
    }
}

/// Strategy engine that manages crawling strategy execution
pub struct StrategyEngine {
    current_strategy: CrawlingStrategy,
    context: Arc<RwLock<StrategyContext>>,
    scoring_function: Option<Arc<dyn ScoringFunction>>,
}

impl StrategyEngine {
    pub fn new(strategy: CrawlingStrategy) -> Self {
        let mut context = StrategyContext::default();
        context.current_strategy = strategy_name(&strategy);

        let scoring_function = match &strategy {
            CrawlingStrategy::BestFirst { scoring_config } => {
                Some(Arc::new(DefaultScoring::new(scoring_config.clone())) as Arc<dyn ScoringFunction>)
            }
            _ => None,
        };

        Self {
            current_strategy: strategy,
            context: Arc::new(RwLock::new(context)),
            scoring_function,
        }
    }

    /// Process requests according to the current strategy
    pub async fn process_requests(&mut self, mut requests: Vec<CrawlRequest>) -> Result<Vec<CrawlRequest>> {
        loop {
            match &self.current_strategy {
                CrawlingStrategy::BreadthFirst => {
                    // BFS: sort by depth (shallow first), then by insertion order
                    requests.sort_by(|a, b| {
                        a.depth.cmp(&b.depth)
                            .then_with(|| a.created_at.cmp(&b.created_at))
                    });
                    break;
                }
                CrawlingStrategy::DepthFirst => {
                    // DFS: sort by depth (deep first), prefer recent insertions
                    requests.sort_by(|a, b| {
                        b.depth.cmp(&a.depth)
                            .then_with(|| b.created_at.cmp(&a.created_at))
                    });
                    break;
                }
                CrawlingStrategy::BestFirst { scoring_config: _ } => {
                    // Best-First: calculate scores and sort by score
                    if let Some(scoring_fn) = &self.scoring_function {
                        for request in &mut requests {
                            let score = scoring_fn.score_url(&request.url, request.depth, &request.metadata);
                            request.score = Some(score);
                        }
                    }
                    requests.sort_by(|a, b| {
                        b.score.partial_cmp(&a.score)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    break;
                }
                CrawlingStrategy::Adaptive { primary, fallback, switch_criteria } => {
                    // Check if we should switch strategies
                    let should_switch = self.should_switch_strategy(switch_criteria).await?;

                    if should_switch {
                        let current_name = strategy_name(&self.current_strategy);
                        // For simplicity, switch between primary and fallback
                        let target_strategy = if current_name.contains("primary") {
                            fallback.as_ref()
                        } else {
                            primary.as_ref()
                        };

                        self.switch_to_strategy(target_strategy.clone()).await?;
                    }

                    // Continue with the loop to process with current strategy
                    continue;
                }
            }
        }

        Ok(requests)
    }

    /// Check if strategy should switch based on adaptive criteria
    async fn should_switch_strategy(&self, criteria: &AdaptiveCriteria) -> Result<bool> {
        let context = self.context.read().await;

        // Not enough data to make a decision
        if context.pages_crawled < criteria.min_pages_for_switch as u64 {
            return Ok(false);
        }

        // Cooldown period
        if context.pages_since_switch < criteria.switch_cooldown_pages as u64 {
            return Ok(false);
        }

        // Check switch conditions
        let should_switch =
            context.frontier_size > criteria.max_frontier_size ||
            context.average_depth > criteria.max_average_depth ||
            context.success_rate() < criteria.min_success_rate;

        if should_switch {
            debug!(
                frontier_size = context.frontier_size,
                average_depth = context.average_depth,
                success_rate = context.success_rate(),
                "Strategy switch conditions met"
            );
        }

        Ok(should_switch)
    }

    /// Switch to a new strategy
    async fn switch_to_strategy(&mut self, new_strategy: CrawlingStrategy) -> Result<()> {
        let old_name = strategy_name(&self.current_strategy);
        let new_name = strategy_name(&new_strategy);

        // Update scoring function if needed
        self.scoring_function = match &new_strategy {
            CrawlingStrategy::BestFirst { scoring_config } => {
                Some(Arc::new(DefaultScoring::new(scoring_config.clone())) as Arc<dyn ScoringFunction>)
            }
            _ => None,
        };

        // Update context
        {
            let mut context = self.context.write().await;
            context.record_switch(&old_name, &new_name, "Adaptive criteria met".to_string());
        }

        self.current_strategy = new_strategy;

        info!(
            from = %old_name,
            to = %new_name,
            "Strategy switched"
        );

        Ok(())
    }

    /// Record a crawl result for strategy analysis
    pub async fn record_crawl_result(&self, success: bool) {
        let mut context = self.context.write().await;
        context.record_crawl(success);
    }

    /// Update context with current frontier metrics
    pub async fn update_context(&self, frontier_size: usize, average_depth: f64) {
        let mut context = self.context.write().await;
        context.frontier_size = frontier_size;
        context.average_depth = average_depth;
    }

    /// Get current strategy context
    pub async fn get_context(&self) -> StrategyContext {
        self.context.read().await.clone()
    }

    /// Get current strategy name
    pub fn get_strategy_name(&self) -> String {
        strategy_name(&self.current_strategy)
    }

    /// Calculate priority for a request based on current strategy
    pub async fn calculate_priority(&self, request: &CrawlRequest) -> Priority {
        match &self.current_strategy {
            CrawlingStrategy::BreadthFirst => {
                // BFS: prioritize by inverse depth
                match request.depth {
                    0..=2 => Priority::High,
                    3..=5 => Priority::Medium,
                    _ => Priority::Low,
                }
            }
            CrawlingStrategy::DepthFirst => {
                // DFS: prioritize deeper pages
                match request.depth {
                    0..=3 => Priority::Low,
                    4..=7 => Priority::Medium,
                    _ => Priority::High,
                }
            }
            CrawlingStrategy::BestFirst { .. } => {
                // Best-First: use score to determine priority
                if let Some(score) = request.score {
                    if score > 2.0 {
                        Priority::High
                    } else if score > 1.0 {
                        Priority::Medium
                    } else {
                        Priority::Low
                    }
                } else {
                    Priority::Medium
                }
            }
            CrawlingStrategy::Adaptive { .. } => {
                // Adaptive: delegate to current effective strategy
                Priority::Medium // Simplified for adaptive
            }
        }
    }
}

/// Get a human-readable name for a strategy
fn strategy_name(strategy: &CrawlingStrategy) -> String {
    match strategy {
        CrawlingStrategy::BreadthFirst => "BreadthFirst".to_string(),
        CrawlingStrategy::DepthFirst => "DepthFirst".to_string(),
        CrawlingStrategy::BestFirst { .. } => "BestFirst".to_string(),
        CrawlingStrategy::Adaptive { primary, .. } => {
            format!("Adaptive({})", strategy_name(primary))
        }
    }
}

/// Create a breadth-first strategy
pub fn breadth_first_strategy() -> CrawlingStrategy {
    CrawlingStrategy::BreadthFirst
}

/// Create a depth-first strategy
pub fn depth_first_strategy() -> CrawlingStrategy {
    CrawlingStrategy::DepthFirst
}

/// Create a best-first strategy with default scoring
pub fn best_first_strategy() -> CrawlingStrategy {
    CrawlingStrategy::BestFirst {
        scoring_config: ScoringConfig::default(),
    }
}

/// Create a best-first strategy with custom scoring
pub fn best_first_strategy_with_config(scoring_config: ScoringConfig) -> CrawlingStrategy {
    CrawlingStrategy::BestFirst { scoring_config }
}

/// Create an adaptive strategy
pub fn adaptive_strategy(
    primary: CrawlingStrategy,
    fallback: CrawlingStrategy,
    criteria: Option<AdaptiveCriteria>,
) -> CrawlingStrategy {
    CrawlingStrategy::Adaptive {
        primary: Box::new(primary),
        fallback: Box::new(fallback),
        switch_criteria: criteria.unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_default_scoring() {
        let config = ScoringConfig::default();
        let scorer = DefaultScoring::new(config);

        let url1 = Url::from_str("https://example.com/").expect("Valid URL");
        let url2 = Url::from_str("https://example.com/deep/path/page.html").expect("Valid URL");

        let score1 = scorer.score_url(&url1, 0, &HashMap::new());
        let score2 = scorer.score_url(&url2, 3, &HashMap::new());

        // Shallower URLs should generally score higher
        assert!(score1 > score2);
    }

    #[tokio::test]
    async fn test_breadth_first_processing() {
        let mut strategy_engine = StrategyEngine::new(breadth_first_strategy());

        let requests = vec![
            CrawlRequest::new(Url::from_str("https://example.com/deep").expect("Valid URL")).with_depth(5),
            CrawlRequest::new(Url::from_str("https://example.com/shallow").expect("Valid URL")).with_depth(1),
            CrawlRequest::new(Url::from_str("https://example.com/medium").expect("Valid URL")).with_depth(3),
        ];

        let processed = strategy_engine.process_requests(requests).await.expect("Processing should succeed");

        // Should be ordered by depth (shallow first)
        assert_eq!(processed[0].depth, 1);
        assert_eq!(processed[1].depth, 3);
        assert_eq!(processed[2].depth, 5);
    }

    #[tokio::test]
    async fn test_depth_first_processing() {
        let mut strategy_engine = StrategyEngine::new(depth_first_strategy());

        let requests = vec![
            CrawlRequest::new(Url::from_str("https://example.com/shallow").expect("Valid URL")).with_depth(1),
            CrawlRequest::new(Url::from_str("https://example.com/deep").expect("Valid URL")).with_depth(5),
            CrawlRequest::new(Url::from_str("https://example.com/medium").expect("Valid URL")).with_depth(3),
        ];

        let processed = strategy_engine.process_requests(requests).await.expect("Processing should succeed");

        // Should be ordered by depth (deep first)
        assert_eq!(processed[0].depth, 5);
        assert_eq!(processed[1].depth, 3);
        assert_eq!(processed[2].depth, 1);
    }

    #[tokio::test]
    async fn test_best_first_processing() {
        let mut strategy_engine = StrategyEngine::new(best_first_strategy());

        let requests = vec![
            CrawlRequest::new(Url::from_str("https://example.com/low.pdf").expect("Valid URL")),
            CrawlRequest::new(Url::from_str("https://example.com/high.html").expect("Valid URL")),
            CrawlRequest::new(Url::from_str("https://example.com/medium.php").expect("Valid URL")),
        ];

        let processed = strategy_engine.process_requests(requests).await.expect("Processing should succeed");

        // HTML should score higher than PHP, which should score higher than PDF
        assert!(processed[0].url.path().contains("html"));
    }

    #[test]
    fn test_strategy_context() {
        let mut context = StrategyContext::default();

        assert_eq!(context.success_rate(), 1.0);

        context.record_crawl(true);
        context.record_crawl(false);
        context.record_crawl(true);

        assert_eq!(context.pages_crawled, 3);
        assert_eq!(context.successful_pages, 2);
        assert_eq!(context.success_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_adaptive_criteria() {
        let criteria = AdaptiveCriteria::default();

        assert_eq!(criteria.max_frontier_size, 10000);
        assert_eq!(criteria.min_success_rate, 0.7);
        assert_eq!(criteria.min_pages_for_switch, 100);
    }
}