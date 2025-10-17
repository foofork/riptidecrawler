//! Unified strategy traits for extraction, chunking, and spider operations
//!
//! This module provides trait-based strategy management to replace the existing enum-based
//! approach, enabling better extensibility and composition of strategies.

use std::collections::HashMap;
use std::sync::Arc;

// Re-export traits and types from riptide-types
pub use riptide_types::{
    CrawlRequest, CrawlResult, ExtractionResult, ExtractionStrategy, PerformanceTier, Priority,
    ResourceRequirements, ResourceTier, SpiderStrategy, StrategyCapabilities,
};

// All trait definitions and supporting types are now in riptide-types
// This module serves as a re-export point for backward compatibility

// Re-export ExtractionQuality for local use
pub use riptide_types::ExtractionQuality;

/// Strategy registry for managing all strategy implementations
pub struct StrategyRegistry {
    extraction_strategies: HashMap<String, Arc<dyn ExtractionStrategy>>,
    spider_strategies: HashMap<String, Arc<dyn SpiderStrategy>>,
}

impl StrategyRegistry {
    /// Create a new strategy registry
    pub fn new() -> Self {
        Self {
            extraction_strategies: HashMap::new(),
            spider_strategies: HashMap::new(),
        }
    }

    /// Register an extraction strategy
    pub fn register_extraction(&mut self, strategy: Arc<dyn ExtractionStrategy>) {
        let name = strategy.name().to_string();
        self.extraction_strategies.insert(name, strategy);
    }

    /// Register a spider strategy
    pub fn register_spider(&mut self, strategy: Arc<dyn SpiderStrategy>) {
        let name = strategy.name().to_string();
        self.spider_strategies.insert(name, strategy);
    }

    /// Get extraction strategy by name
    pub fn get_extraction(&self, name: &str) -> Option<&Arc<dyn ExtractionStrategy>> {
        self.extraction_strategies.get(name)
    }

    /// Get spider strategy by name
    pub fn get_spider(&self, name: &str) -> Option<&Arc<dyn SpiderStrategy>> {
        self.spider_strategies.get(name)
    }

    /// List all available extraction strategies
    pub fn list_extraction_strategies(&self) -> Vec<(String, StrategyCapabilities)> {
        self.extraction_strategies
            .values()
            .map(|s| (s.name().to_string(), s.capabilities()))
            .collect()
    }

    /// List all available chunking strategies (moved to riptide-extraction)
    pub fn list_chunking_strategies(&self) -> Vec<String> {
        vec![] // Empty - chunking moved to riptide-extraction crate
    }

    /// List all available spider strategies
    pub fn list_spider_strategies(&self) -> Vec<String> {
        self.spider_strategies
            .keys()
            .map(|s| s.to_string())
            .collect()
    }

    /// Find best extraction strategy for given content
    pub fn find_best_extraction(&self, html: &str) -> Option<&Arc<dyn ExtractionStrategy>> {
        self.extraction_strategies
            .values()
            .filter(|s| s.is_available())
            .max_by(|a, b| {
                a.confidence_score(html)
                    .partial_cmp(&b.confidence_score(html))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating configured strategy registry
pub struct StrategyRegistryBuilder {
    registry: StrategyRegistry,
}

impl StrategyRegistryBuilder {
    pub fn new() -> Self {
        Self {
            registry: StrategyRegistry::new(),
        }
    }

    pub fn with_extraction(mut self, strategy: Arc<dyn ExtractionStrategy>) -> Self {
        self.registry.register_extraction(strategy);
        self
    }

    pub fn with_spider(mut self, strategy: Arc<dyn SpiderStrategy>) -> Self {
        self.registry.register_spider(strategy);
        self
    }

    pub fn build(self) -> StrategyRegistry {
        self.registry
    }
}

impl Default for StrategyRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
