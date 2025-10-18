//! Spider strategy trait implementations
//!
//! This module provides trait implementations for the existing spider/crawler strategies.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

use crate::strategies::traits::{SpiderStrategy, CrawlRequest, CrawlResult, Priority};
use crate::spider::strategy::{CrawlingStrategy, StrategyEngine, ScoringConfig};

/// Breadth-First spider strategy
#[derive(Debug, Clone)]
pub struct BreadthFirstSpiderStrategy {
    engine: StrategyEngine,
}

impl BreadthFirstSpiderStrategy {
    pub fn new() -> Self {
        Self {
            engine: StrategyEngine::new(CrawlingStrategy::BreadthFirst),
        }
    }
}

#[async_trait]
impl SpiderStrategy for BreadthFirstSpiderStrategy {
    async fn process_requests(&self, mut requests: Vec<CrawlRequest>) -> Result<Vec<CrawlRequest>> {
        // Convert to spider types
        let mut spider_requests: Vec<crate::spider::types::CrawlRequest> = requests
            .into_iter()
            .map(|req| {
                let mut spider_req = crate::spider::types::CrawlRequest::new(req.url);
                spider_req.depth = req.depth;
                spider_req.metadata = req.metadata;
                spider_req.created_at = req.created_at;
                spider_req
            })
            .collect();

        // Use the strategy engine
        let mut engine = self.engine.clone();
        let processed = engine.process_requests(spider_requests).await?;

        // Convert back
        let result = processed
            .into_iter()
            .map(|req| CrawlRequest {
                url: req.url,
                depth: req.depth,
                priority: req.score,
                score: req.score,
                metadata: req.metadata,
                created_at: req.created_at,
            })
            .collect();

        Ok(result)
    }

    fn name(&self) -> &str {
        "breadth_first"
    }

    async fn calculate_priority(&self, request: &CrawlRequest) -> Priority {
        match request.depth {
            0..=2 => Priority::High,
            3..=5 => Priority::Medium,
            _ => Priority::Low,
        }
    }

    async fn update_context(&mut self, results: &[CrawlResult]) {
        for result in results {
            self.engine.record_crawl_result(result.success).await;
        }
    }

    async fn should_adapt(&self) -> bool {
        false // BFS doesn't adapt
    }
}

impl Default for BreadthFirstSpiderStrategy {
    fn default() -> Self {
        Self::new()
    }
}

/// Depth-First spider strategy
#[derive(Debug, Clone)]
pub struct DepthFirstSpiderStrategy {
    engine: StrategyEngine,
}

impl DepthFirstSpiderStrategy {
    pub fn new() -> Self {
        Self {
            engine: StrategyEngine::new(CrawlingStrategy::DepthFirst),
        }
    }
}

#[async_trait]
impl SpiderStrategy for DepthFirstSpiderStrategy {
    async fn process_requests(&self, requests: Vec<CrawlRequest>) -> Result<Vec<CrawlRequest>> {
        // Convert to spider types
        let mut spider_requests: Vec<crate::spider::types::CrawlRequest> = requests
            .into_iter()
            .map(|req| {
                let mut spider_req = crate::spider::types::CrawlRequest::new(req.url);
                spider_req.depth = req.depth;
                spider_req.metadata = req.metadata;
                spider_req.created_at = req.created_at;
                spider_req
            })
            .collect();

        // Use the strategy engine
        let mut engine = self.engine.clone();
        let processed = engine.process_requests(spider_requests).await?;

        // Convert back
        let result = processed
            .into_iter()
            .map(|req| CrawlRequest {
                url: req.url,
                depth: req.depth,
                priority: req.score,
                score: req.score,
                metadata: req.metadata,
                created_at: req.created_at,
            })
            .collect();

        Ok(result)
    }

    fn name(&self) -> &str {
        "depth_first"
    }

    async fn calculate_priority(&self, request: &CrawlRequest) -> Priority {
        match request.depth {
            0..=3 => Priority::Low,
            4..=7 => Priority::Medium,
            _ => Priority::High,
        }
    }

    async fn update_context(&mut self, results: &[CrawlResult]) {
        for result in results {
            self.engine.record_crawl_result(result.success).await;
        }
    }

    async fn should_adapt(&self) -> bool {
        false // DFS doesn't adapt
    }
}

impl Default for DepthFirstSpiderStrategy {
    fn default() -> Self {
        Self::new()
    }
}

/// Best-First spider strategy with scoring
#[derive(Debug, Clone)]
pub struct BestFirstSpiderStrategy {
    engine: StrategyEngine,
    scoring_config: ScoringConfig,
}

impl BestFirstSpiderStrategy {
    pub fn new(scoring_config: ScoringConfig) -> Self {
        Self {
            engine: StrategyEngine::new(CrawlingStrategy::BestFirst {
                scoring_config: scoring_config.clone(),
            }),
            scoring_config,
        }
    }

    pub fn with_default_scoring() -> Self {
        Self::new(ScoringConfig::default())
    }
}

#[async_trait]
impl SpiderStrategy for BestFirstSpiderStrategy {
    async fn process_requests(&self, requests: Vec<CrawlRequest>) -> Result<Vec<CrawlRequest>> {
        // Convert to spider types
        let mut spider_requests: Vec<crate::spider::types::CrawlRequest> = requests
            .into_iter()
            .map(|req| {
                let mut spider_req = crate::spider::types::CrawlRequest::new(req.url);
                spider_req.depth = req.depth;
                spider_req.metadata = req.metadata;
                spider_req.created_at = req.created_at;
                spider_req
            })
            .collect();

        // Use the strategy engine
        let mut engine = self.engine.clone();
        let processed = engine.process_requests(spider_requests).await?;

        // Convert back
        let result = processed
            .into_iter()
            .map(|req| CrawlRequest {
                url: req.url,
                depth: req.depth,
                priority: req.score,
                score: req.score,
                metadata: req.metadata,
                created_at: req.created_at,
            })
            .collect();

        Ok(result)
    }

    fn name(&self) -> &str {
        "best_first"
    }

    async fn calculate_priority(&self, request: &CrawlRequest) -> Priority {
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

    async fn update_context(&mut self, results: &[CrawlResult]) {
        for result in results {
            self.engine.record_crawl_result(result.success).await;
        }
    }

    async fn should_adapt(&self) -> bool {
        false // Best-first doesn't adapt by default
    }
}

/// Adaptive spider strategy that switches between strategies
#[derive(Debug)]
pub struct AdaptiveSpiderStrategy {
    engine: StrategyEngine,
    primary_strategy: Box<dyn SpiderStrategy + Send + Sync>,
    fallback_strategy: Box<dyn SpiderStrategy + Send + Sync>,
    switch_threshold: f64,
    current_primary: bool,
}

impl AdaptiveSpiderStrategy {
    pub fn new(
        primary: Box<dyn SpiderStrategy + Send + Sync>,
        fallback: Box<dyn SpiderStrategy + Send + Sync>,
        switch_threshold: f64,
    ) -> Self {
        // Create adaptive crawling strategy for the engine
        let crawling_strategy = CrawlingStrategy::Adaptive {
            primary: Box::new(CrawlingStrategy::BreadthFirst),
            fallback: Box::new(CrawlingStrategy::DepthFirst),
            switch_criteria: crate::spider::strategy::AdaptiveCriteria::default(),
        };

        Self {
            engine: StrategyEngine::new(crawling_strategy),
            primary_strategy: primary,
            fallback_strategy: fallback,
            switch_threshold,
            current_primary: true,
        }
    }
}

#[async_trait]
impl SpiderStrategy for AdaptiveSpiderStrategy {
    async fn process_requests(&self, requests: Vec<CrawlRequest>) -> Result<Vec<CrawlRequest>> {
        let strategy = if self.current_primary {
            &self.primary_strategy
        } else {
            &self.fallback_strategy
        };

        strategy.process_requests(requests).await
    }

    fn name(&self) -> &str {
        "adaptive"
    }

    async fn calculate_priority(&self, request: &CrawlRequest) -> Priority {
        let strategy = if self.current_primary {
            &self.primary_strategy
        } else {
            &self.fallback_strategy
        };

        strategy.calculate_priority(request).await
    }

    async fn update_context(&mut self, results: &[CrawlResult]) {
        // Update both strategies
        self.primary_strategy.update_context(results).await;
        self.fallback_strategy.update_context(results).await;

        // Update engine context
        for result in results {
            self.engine.record_crawl_result(result.success).await;
        }
    }

    async fn should_adapt(&self) -> bool {
        // Check if we should switch strategies based on performance
        let context = self.engine.get_context().await;

        // Switch if success rate drops below threshold
        if context.success_rate() < self.switch_threshold {
            return true;
        }

        // Check if current strategy wants to adapt
        let current_strategy = if self.current_primary {
            &self.primary_strategy
        } else {
            &self.fallback_strategy
        };

        current_strategy.should_adapt().await
    }
}

/// Create spider strategy registry with all implementations
pub fn create_spider_registry() -> crate::strategies::StrategyRegistry {
    let mut registry = crate::strategies::StrategyRegistry::new();

    // Register spider strategies
    registry.register_spider(std::sync::Arc::new(BreadthFirstSpiderStrategy::new()));
    registry.register_spider(std::sync::Arc::new(DepthFirstSpiderStrategy::new()));
    registry.register_spider(std::sync::Arc::new(BestFirstSpiderStrategy::with_default_scoring()));

    // Create adaptive strategy
    let adaptive = AdaptiveSpiderStrategy::new(
        Box::new(BreadthFirstSpiderStrategy::new()),
        Box::new(DepthFirstSpiderStrategy::new()),
        0.7, // 70% success rate threshold
    );
    registry.register_spider(std::sync::Arc::new(adaptive));

    registry
}