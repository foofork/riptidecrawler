//! Strategy composition framework for chaining multiple extraction strategies
//!
//! This module enables easy composition of multiple extraction strategies with different modes:
//! - **Chain**: Try each strategy in order until one succeeds
//! - **Parallel**: Run all strategies concurrently and merge results
//! - **Fallback**: Try primary, fallback to secondary on failure
//! - **Best**: Run all strategies and pick the one with highest confidence
//!
//! ## Example
//!
//! ```rust,no_run
//! use riptide_extraction::composition::{StrategyComposer, CompositionMode};
//!
//! let composer = StrategyComposer::new(CompositionMode::Chain)
//!     .with_timeout(5000)
//!     .with_strategies(vec![trek_strategy, css_strategy, fallback_strategy]);
//!
//! let result = composer.execute(html, url).await?;
//! ```

use crate::strategies::traits::{ExtractionResult, ExtractionStrategy};
use crate::strategies::PerformanceMetrics;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use riptide_types::ExtractedContent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// Composition mode for combining multiple strategies
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompositionMode {
    /// Try each strategy in order until one succeeds
    Chain,
    /// Run all strategies in parallel and merge results
    Parallel,
    /// Try first strategy, fallback to second on failure
    Fallback,
    /// Run all strategies and pick the one with highest confidence
    Best,
}

/// Result merger trait for combining results from multiple strategies
#[async_trait]
pub trait ResultMerger: Send + Sync {
    /// Merge multiple extraction results into a single result
    async fn merge(&self, results: Vec<ExtractionResult>) -> Result<ExtractionResult>;

    /// Get merger name for identification
    fn name(&self) -> &str;

    /// Get merger configuration
    fn config(&self) -> MergerConfig;
}

/// Configuration for result mergers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergerConfig {
    /// Minimum confidence threshold for including results
    pub min_confidence: f64,
    /// Maximum number of results to merge
    pub max_results: usize,
    /// Weight by confidence scores
    pub weight_by_confidence: bool,
    /// Prefer longer content
    pub prefer_longer_content: bool,
}

impl Default for MergerConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.5,
            max_results: 5,
            weight_by_confidence: true,
            prefer_longer_content: true,
        }
    }
}

/// Union merger - combines all content
pub struct UnionMerger {
    config: MergerConfig,
}

impl UnionMerger {
    pub fn new(config: MergerConfig) -> Self {
        Self { config }
    }
}

impl Default for UnionMerger {
    fn default() -> Self {
        Self::new(MergerConfig::default())
    }
}

#[async_trait]
impl ResultMerger for UnionMerger {
    async fn merge(&self, results: Vec<ExtractionResult>) -> Result<ExtractionResult> {
        if results.is_empty() {
            return Err(anyhow!("No results to merge"));
        }

        // Filter by confidence threshold
        let filtered: Vec<_> = results
            .into_iter()
            .filter(|r| r.content.extraction_confidence >= self.config.min_confidence)
            .collect();

        if filtered.is_empty() {
            return Err(anyhow!("No results meet confidence threshold"));
        }

        // Combine all content
        let mut combined_title = String::new();
        let mut combined_content = String::new();
        let mut combined_summary = String::new();
        let mut total_confidence = 0.0;
        let mut metadata = HashMap::new();

        for (idx, result) in filtered.iter().enumerate() {
            if idx == 0 {
                combined_title = result.content.title.clone();
            }
            combined_content.push_str(&result.content.content);
            combined_content.push('\n');

            if let Some(ref summary) = result.content.summary {
                combined_summary.push_str(summary);
                combined_summary.push(' ');
            }

            total_confidence += result.content.extraction_confidence;

            // Merge metadata
            for (key, value) in &result.metadata {
                metadata.insert(format!("strategy_{}_{}", idx, key), value.clone());
            }
        }

        // Safe conversion: filtered.len() can be large on 64-bit but division will always fit in f64
        #[allow(clippy::cast_precision_loss)]
        let avg_confidence = total_confidence / filtered.len() as f64;

        Ok(ExtractionResult {
            content: ExtractedContent {
                title: combined_title,
                content: combined_content.trim().to_string(),
                summary: if combined_summary.is_empty() {
                    None
                } else {
                    Some(combined_summary.trim().to_string())
                },
                url: filtered
                    .first()
                    .map(|r| r.content.url.clone())
                    .unwrap_or_default(),
                strategy_used: "union_merger".to_string(),
                extraction_confidence: avg_confidence,
            },
            quality: filtered.first().map_or_else(
                || {
                    // Default quality when no results available
                    riptide_types::ExtractionQuality {
                        content_length: 0,
                        title_quality: 50.0,
                        content_quality: 50.0,
                        structure_score: 50.0,
                        metadata_completeness: 50.0,
                    }
                },
                |r| r.quality.clone(),
            ),
            performance: None,
            metadata,
        })
    }

    fn name(&self) -> &str {
        "union"
    }

    fn config(&self) -> MergerConfig {
        self.config.clone()
    }
}

/// Best content merger - picks best fields from different strategies
pub struct BestContentMerger {
    config: MergerConfig,
}

impl BestContentMerger {
    pub fn new(config: MergerConfig) -> Self {
        Self { config }
    }
}

impl Default for BestContentMerger {
    fn default() -> Self {
        Self::new(MergerConfig::default())
    }
}

#[async_trait]
impl ResultMerger for BestContentMerger {
    async fn merge(&self, results: Vec<ExtractionResult>) -> Result<ExtractionResult> {
        if results.is_empty() {
            return Err(anyhow!("No results to merge"));
        }

        // Filter by confidence
        let filtered: Vec<_> = results
            .into_iter()
            .filter(|r| r.content.extraction_confidence >= self.config.min_confidence)
            .collect();

        if filtered.is_empty() {
            return Err(anyhow!("No results meet confidence threshold"));
        }

        // Find best title (longest non-empty)
        let best_title = filtered
            .iter()
            .map(|r| &r.content.title)
            .filter(|t| !t.is_empty())
            .max_by_key(|t| t.len())
            .cloned()
            .unwrap_or_else(|| "Untitled".to_string());

        // Find best content (longest or highest confidence)
        let best_content_result = if self.config.prefer_longer_content {
            filtered
                .iter()
                .max_by_key(|r| r.content.content.len())
                .ok_or_else(|| anyhow!("No valid content found in results"))?
        } else {
            filtered
                .iter()
                .max_by(|a, b| {
                    a.content
                        .extraction_confidence
                        .partial_cmp(&b.content.extraction_confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .ok_or_else(|| anyhow!("No valid content found in results"))?
        };

        // Find best summary
        let best_summary = filtered
            .iter()
            .filter_map(|r| r.content.summary.as_ref())
            .max_by_key(|s| s.len())
            .cloned();

        // Aggregate metadata
        let mut metadata = HashMap::new();
        metadata.insert("merged_from".to_string(), filtered.len().to_string());
        metadata.insert(
            "strategies_used".to_string(),
            filtered
                .iter()
                .map(|r| r.content.strategy_used.clone())
                .collect::<Vec<_>>()
                .join(","),
        );

        Ok(ExtractionResult {
            content: ExtractedContent {
                title: best_title,
                content: best_content_result.content.content.clone(),
                summary: best_summary,
                url: best_content_result.content.url.clone(),
                strategy_used: "best_content_merger".to_string(),
                extraction_confidence: best_content_result.content.extraction_confidence,
            },
            quality: best_content_result.quality.clone(),
            performance: None,
            metadata,
        })
    }

    fn name(&self) -> &str {
        "best_content"
    }

    fn config(&self) -> MergerConfig {
        self.config.clone()
    }
}

/// Strategy composer configuration
#[derive(Debug, Clone)]
pub struct ComposerConfig {
    /// Composition mode
    pub mode: CompositionMode,
    /// Timeout per strategy in milliseconds
    pub timeout_ms: u64,
    /// Global timeout for entire composition in milliseconds
    pub global_timeout_ms: u64,
    /// Minimum confidence threshold to consider a result successful
    pub min_confidence: f64,
    /// Whether to collect performance metrics
    pub collect_metrics: bool,
    /// Maximum number of strategies to execute concurrently
    pub max_concurrent: usize,
}

impl Default for ComposerConfig {
    fn default() -> Self {
        Self {
            mode: CompositionMode::Chain,
            timeout_ms: 5000,
            global_timeout_ms: 15000,
            min_confidence: 0.6,
            collect_metrics: true,
            max_concurrent: 4,
        }
    }
}

/// Composition result with detailed metrics
#[derive(Debug, Clone)]
pub struct CompositionResult {
    /// Final extraction result
    pub result: ExtractionResult,
    /// Composition mode used
    pub mode: CompositionMode,
    /// Number of strategies executed
    pub strategies_executed: usize,
    /// Number of strategies that succeeded
    pub strategies_succeeded: usize,
    /// Total execution time
    pub total_time: Duration,
    /// Individual strategy execution times
    pub strategy_times: HashMap<String, Duration>,
    /// Performance metrics
    pub metrics: Option<PerformanceMetrics>,
}

/// Strategy composer for executing multiple strategies with different modes
pub struct StrategyComposer {
    strategies: Vec<Arc<dyn ExtractionStrategy>>,
    merger: Box<dyn ResultMerger>,
    config: ComposerConfig,
}

impl StrategyComposer {
    /// Create a new strategy composer with default configuration
    pub fn new(mode: CompositionMode) -> Self {
        Self {
            strategies: Vec::new(),
            merger: Box::new(BestContentMerger::default()),
            config: ComposerConfig {
                mode,
                ..Default::default()
            },
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: ComposerConfig) -> Self {
        Self {
            strategies: Vec::new(),
            merger: Box::new(BestContentMerger::default()),
            config,
        }
    }

    /// Add a strategy to the composer
    pub fn add_strategy(mut self, strategy: Arc<dyn ExtractionStrategy>) -> Self {
        self.strategies.push(strategy);
        self
    }

    /// Set strategies all at once
    pub fn with_strategies(mut self, strategies: Vec<Arc<dyn ExtractionStrategy>>) -> Self {
        self.strategies = strategies;
        self
    }

    /// Set the result merger
    pub fn with_merger(mut self, merger: Box<dyn ResultMerger>) -> Self {
        self.merger = merger;
        self
    }

    /// Set timeout per strategy
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.config.timeout_ms = timeout_ms;
        self
    }

    /// Set global timeout
    pub fn with_global_timeout(mut self, timeout_ms: u64) -> Self {
        self.config.global_timeout_ms = timeout_ms;
        self
    }

    /// Set minimum confidence threshold
    pub fn with_min_confidence(mut self, confidence: f64) -> Self {
        self.config.min_confidence = confidence;
        self
    }

    /// Execute composition
    pub async fn execute(&self, html: &str, url: &str) -> Result<CompositionResult> {
        if self.strategies.is_empty() {
            return Err(anyhow!("No strategies configured"));
        }

        let _start = Instant::now();
        let global_timeout_duration = Duration::from_millis(self.config.global_timeout_ms);

        // Wrap the actual execution in a timeout
        let result = timeout(global_timeout_duration, self.execute_with_mode(html, url))
            .await
            .map_err(|_| anyhow!("Global composition timeout exceeded"))??;

        Ok(result)
    }

    /// Execute with the configured mode
    async fn execute_with_mode(&self, html: &str, url: &str) -> Result<CompositionResult> {
        match self.config.mode {
            CompositionMode::Chain => self.execute_chain(html, url).await,
            CompositionMode::Parallel => self.execute_parallel(html, url).await,
            CompositionMode::Fallback => self.execute_fallback(html, url).await,
            CompositionMode::Best => self.execute_best(html, url).await,
        }
    }

    /// Execute chain mode - try each strategy until one succeeds
    async fn execute_chain(&self, html: &str, url: &str) -> Result<CompositionResult> {
        let start = Instant::now();
        let mut strategy_times = HashMap::new();
        let mut last_error = None;

        for (strategies_executed, strategy) in self.strategies.iter().enumerate() {
            let strategies_executed = strategies_executed.saturating_add(1);
            let strategy_start = Instant::now();

            match timeout(
                Duration::from_millis(self.config.timeout_ms),
                strategy.extract(html, url),
            )
            .await
            {
                Ok(Ok(result)) => {
                    let strategy_time = strategy_start.elapsed();
                    strategy_times.insert(strategy.name().to_string(), strategy_time);

                    // Check confidence threshold
                    if result.content.extraction_confidence >= self.config.min_confidence {
                        return Ok(CompositionResult {
                            result,
                            mode: CompositionMode::Chain,
                            strategies_executed,
                            strategies_succeeded: 1,
                            total_time: start.elapsed(),
                            strategy_times,
                            metrics: None,
                        });
                    }
                    last_error = Some(anyhow!(
                        "Confidence {} below threshold",
                        result.content.extraction_confidence
                    ));
                }
                Ok(Err(e)) => {
                    let strategy_time = strategy_start.elapsed();
                    strategy_times.insert(strategy.name().to_string(), strategy_time);
                    last_error = Some(e);
                }
                Err(_) => {
                    let strategy_time = strategy_start.elapsed();
                    strategy_times.insert(strategy.name().to_string(), strategy_time);
                    last_error = Some(anyhow!("Strategy timeout"));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("All strategies failed")))
    }

    /// Execute parallel mode - run all strategies and merge results
    async fn execute_parallel(&self, html: &str, url: &str) -> Result<CompositionResult> {
        let start = Instant::now();
        let mut strategy_times = HashMap::new();
        let mut handles = Vec::new();

        // Launch all strategies in parallel
        for strategy in &self.strategies {
            let strategy = Arc::clone(strategy);
            let html = html.to_string();
            let url = url.to_string();
            let timeout_duration = Duration::from_millis(self.config.timeout_ms);

            let handle = tokio::spawn(async move {
                let strategy_start = Instant::now();
                let result = timeout(timeout_duration, strategy.extract(&html, &url)).await;
                let elapsed = strategy_start.elapsed();
                (strategy.name().to_string(), result, elapsed)
            });

            handles.push(handle);
        }

        // Collect results
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok((name, Ok(Ok(result)), elapsed)) => {
                    strategy_times.insert(name, elapsed);
                    results.push(result);
                }
                Ok((name, Ok(Err(_)), elapsed)) | Ok((name, Err(_), elapsed)) => {
                    strategy_times.insert(name, elapsed);
                }
                Err(_) => {} // Task panicked
            }
        }

        if results.is_empty() {
            return Err(anyhow!("All parallel strategies failed"));
        }

        // Merge results
        let merged_result = self.merger.merge(results.clone()).await?;
        let strategies_succeeded = results.len();

        Ok(CompositionResult {
            result: merged_result,
            mode: CompositionMode::Parallel,
            strategies_executed: self.strategies.len(),
            strategies_succeeded,
            total_time: start.elapsed(),
            strategy_times,
            metrics: None,
        })
    }

    /// Execute fallback mode - try primary, fallback to secondary on failure
    async fn execute_fallback(&self, html: &str, url: &str) -> Result<CompositionResult> {
        if self.strategies.len() < 2 {
            return Err(anyhow!("Fallback mode requires at least 2 strategies"));
        }

        let start = Instant::now();
        let mut strategy_times = HashMap::new();

        // Try primary strategy
        let primary = self
            .strategies
            .first()
            .ok_or_else(|| anyhow!("No primary strategy found"))?;
        let primary_start = Instant::now();

        match timeout(
            Duration::from_millis(self.config.timeout_ms),
            primary.extract(html, url),
        )
        .await
        {
            Ok(Ok(result)) => {
                let primary_time = primary_start.elapsed();
                strategy_times.insert(primary.name().to_string(), primary_time);

                if result.content.extraction_confidence >= self.config.min_confidence {
                    return Ok(CompositionResult {
                        result,
                        mode: CompositionMode::Fallback,
                        strategies_executed: 1,
                        strategies_succeeded: 1,
                        total_time: start.elapsed(),
                        strategy_times,
                        metrics: None,
                    });
                }
            }
            Ok(Err(_)) | Err(_) => {
                strategy_times.insert(primary.name().to_string(), primary_start.elapsed());
            }
        }

        // Fallback to secondary strategy
        let secondary = self
            .strategies
            .get(1)
            .ok_or_else(|| anyhow!("No fallback strategy found"))?;
        let secondary_start = Instant::now();

        let result = timeout(
            Duration::from_millis(self.config.timeout_ms),
            secondary.extract(html, url),
        )
        .await
        .map_err(|_| anyhow!("Fallback strategy timeout"))??;

        strategy_times.insert(secondary.name().to_string(), secondary_start.elapsed());

        Ok(CompositionResult {
            result,
            mode: CompositionMode::Fallback,
            strategies_executed: 2,
            strategies_succeeded: 1,
            total_time: start.elapsed(),
            strategy_times,
            metrics: None,
        })
    }

    /// Execute best mode - run all strategies and pick highest confidence
    async fn execute_best(&self, html: &str, url: &str) -> Result<CompositionResult> {
        let start = Instant::now();
        let mut strategy_times = HashMap::new();
        let mut handles = Vec::new();

        // Launch all strategies
        for strategy in &self.strategies {
            let strategy = Arc::clone(strategy);
            let html = html.to_string();
            let url = url.to_string();
            let timeout_duration = Duration::from_millis(self.config.timeout_ms);

            let handle = tokio::spawn(async move {
                let strategy_start = Instant::now();
                let result = timeout(timeout_duration, strategy.extract(&html, &url)).await;
                let elapsed = strategy_start.elapsed();
                (strategy.name().to_string(), result, elapsed)
            });

            handles.push(handle);
        }

        // Collect successful results
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok((name, Ok(Ok(result)), elapsed)) => {
                    strategy_times.insert(name, elapsed);
                    results.push(result);
                }
                Ok((name, _, elapsed)) => {
                    strategy_times.insert(name, elapsed);
                }
                Err(_) => {}
            }
        }

        if results.is_empty() {
            return Err(anyhow!("All strategies failed"));
        }

        // Pick best result based on confidence
        let best_result = results
            .into_iter()
            .max_by(|a, b| {
                a.content
                    .extraction_confidence
                    .partial_cmp(&b.content.extraction_confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| anyhow!("No valid results to compare"))?;

        let strategies_succeeded = strategy_times.len();

        Ok(CompositionResult {
            result: best_result,
            mode: CompositionMode::Best,
            strategies_executed: self.strategies.len(),
            strategies_succeeded,
            total_time: start.elapsed(),
            strategy_times,
            metrics: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock strategy for testing
    struct MockStrategy {
        name: String,
        should_succeed: bool,
        confidence: f64,
        delay_ms: u64,
    }

    #[async_trait]
    impl ExtractionStrategy for MockStrategy {
        async fn extract(&self, _html: &str, url: &str) -> Result<ExtractionResult> {
            if self.delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
            }

            if !self.should_succeed {
                return Err(anyhow!("Mock strategy failure"));
            }

            Ok(ExtractionResult {
                content: ExtractedContent {
                    title: format!("{} title", self.name),
                    content: format!("{} content", self.name),
                    summary: Some(format!("{} summary", self.name)),
                    url: url.to_string(),
                    strategy_used: self.name.clone(),
                    extraction_confidence: self.confidence,
                },
                quality: crate::strategies::traits::ExtractionQuality {
                    content_length: 100,
                    title_quality: 0.8,
                    content_quality: 0.8,
                    structure_score: 0.8,
                    metadata_completeness: 0.8,
                },
                performance: None,
                metadata: HashMap::new(),
            })
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn capabilities(&self) -> crate::strategies::traits::StrategyCapabilities {
            crate::strategies::traits::StrategyCapabilities {
                strategy_type: "mock".to_string(),
                supported_content_types: vec!["text/html".to_string()],
                performance_tier: crate::strategies::traits::PerformanceTier::Fast,
                resource_requirements: crate::strategies::traits::ResourceRequirements {
                    memory_tier: crate::strategies::traits::ResourceTier::Low,
                    cpu_tier: crate::strategies::traits::ResourceTier::Low,
                    requires_network: false,
                    external_dependencies: vec![],
                },
            }
        }

        fn confidence_score(&self, _html: &str) -> f64 {
            self.confidence
        }
    }

    #[tokio::test]
    async fn test_chain_mode_first_succeeds() {
        let strategy1 = Arc::new(MockStrategy {
            name: "strategy1".to_string(),
            should_succeed: true,
            confidence: 0.9,
            delay_ms: 0,
        });
        let strategy2 = Arc::new(MockStrategy {
            name: "strategy2".to_string(),
            should_succeed: true,
            confidence: 0.8,
            delay_ms: 0,
        });

        let composer = StrategyComposer::new(CompositionMode::Chain)
            .add_strategy(strategy1)
            .add_strategy(strategy2);

        let result = composer
            .execute("<html></html>", "https://example.com")
            .await;
        assert!(result.is_ok());
        let composition_result = result.unwrap();
        assert_eq!(composition_result.strategies_executed, 1);
        assert_eq!(composition_result.result.content.strategy_used, "strategy1");
    }

    #[tokio::test]
    async fn test_fallback_mode() {
        let strategy1 = Arc::new(MockStrategy {
            name: "primary".to_string(),
            should_succeed: false,
            confidence: 0.9,
            delay_ms: 0,
        });
        let strategy2 = Arc::new(MockStrategy {
            name: "fallback".to_string(),
            should_succeed: true,
            confidence: 0.8,
            delay_ms: 0,
        });

        let composer = StrategyComposer::new(CompositionMode::Fallback)
            .add_strategy(strategy1)
            .add_strategy(strategy2);

        let result = composer
            .execute("<html></html>", "https://example.com")
            .await;
        assert!(result.is_ok());
        let composition_result = result.unwrap();
        assert_eq!(composition_result.strategies_executed, 2);
        assert_eq!(composition_result.result.content.strategy_used, "fallback");
    }
}
