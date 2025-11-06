//! Pipeline executor traits - Abstraction layer for orchestrator implementations
//!
//! This module defines trait interfaces that break the circular dependency between
//! riptide-api and riptide-facade by allowing facades to depend on trait abstractions
//! rather than concrete implementations.
//!
//! ## Architecture
//!
//! ```text
//! riptide-facade (depends on traits)
//!         ↓
//! riptide-types::pipeline::traits (trait definitions)
//!         ↑
//! riptide-api::pipeline (trait implementations)
//! ```
//!
//! ## Key Principles
//!
//! 1. **Minimal Interface**: Traits expose only the methods needed by facades
//! 2. **No Implementation**: This module contains ONLY trait definitions, NO business logic
//! 3. **Stable Contracts**: These interfaces should remain stable across refactorings

use crate::error::Result as RiptideResult;
use async_trait::async_trait;

// Import result types from our results module for trait signatures
use super::results::{PipelineResult, PipelineStats, StrategiesPipelineResult};

/// Trait for standard pipeline execution
///
/// Implementations handle the complete crawling pipeline:
/// 1. Cache check
/// 2. Content fetching (with render mode selection)
/// 3. Gate analysis (quality scoring, strategy decision)
/// 4. Content extraction
/// 5. Result caching
///
/// ## Production Implementation
///
/// The main implementation is `PipelineOrchestrator` in `riptide-api::pipeline` (1,071 lines).
#[async_trait]
pub trait PipelineExecutor: Send + Sync {
    /// Execute pipeline for a single URL
    ///
    /// # Arguments
    ///
    /// * `url` - Target URL to crawl and extract
    ///
    /// # Returns
    ///
    /// `PipelineResult` containing extracted content, metadata, and statistics
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - URL is invalid or unreachable
    /// - Fetch operation fails
    /// - Extraction fails
    /// - Timeout is exceeded
    async fn execute_single(&self, url: &str) -> RiptideResult<PipelineResult>;

    /// Execute pipeline for multiple URLs in batch
    ///
    /// Processes URLs with bounded parallelism for efficiency.
    ///
    /// # Arguments
    ///
    /// * `urls` - Slice of URLs to crawl
    ///
    /// # Returns
    ///
    /// Tuple of:
    /// - Vector of optional results (Some = success, None = failure)
    /// - Aggregate statistics for the batch
    async fn execute_batch(&self, urls: &[String]) -> (Vec<Option<PipelineResult>>, PipelineStats);
}

/// Trait for strategies-enhanced pipeline execution
///
/// Extends the standard pipeline with:
/// - Multiple extraction strategies (trek, css_json, regex, llm)
/// - Configurable chunking modes (regex, sentence, topic, fixed, sliding)
/// - Performance tracking and metrics
/// - Strategy selection based on content analysis
///
/// ## Production Implementation
///
/// The main implementation is `StrategiesPipelineOrchestrator` in
/// `riptide-api::strategies_pipeline` (525 lines).
#[async_trait]
pub trait StrategiesPipelineExecutor: Send + Sync {
    /// Execute strategies pipeline for a single URL
    ///
    /// # Arguments
    ///
    /// * `url` - Target URL to crawl and extract
    ///
    /// # Returns
    ///
    /// `StrategiesPipelineResult` containing:
    /// - Processed content with extraction strategies applied
    /// - Chunking results
    /// - Performance metrics
    /// - Strategy configuration used
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - URL is invalid or unreachable
    /// - Fetch operation fails
    /// - All extraction strategies fail
    /// - Timeout is exceeded
    async fn execute_single(&self, url: &str) -> RiptideResult<StrategiesPipelineResult>;
}

/// Combined executor trait for facades that need both capabilities
///
/// This trait is automatically implemented for any type that implements
/// both `PipelineExecutor` and `StrategiesPipelineExecutor`.
pub trait CombinedPipelineExecutor: PipelineExecutor + StrategiesPipelineExecutor {}

// Blanket implementation
impl<T> CombinedPipelineExecutor for T where T: PipelineExecutor + StrategiesPipelineExecutor {}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock implementation for testing
    struct MockPipelineExecutor;

    #[async_trait]
    impl PipelineExecutor for MockPipelineExecutor {
        async fn execute_single(&self, _url: &str) -> RiptideResult<PipelineResult> {
            Ok(PipelineResult {
                url: "https://example.com".to_string(),
                from_cache: false,
                gate_decision: "Raw".to_string(),
                quality_score: 0.9,
                processing_time_ms: 100,
                cache_key: "test-key".to_string(),
                http_status: 200,
                content_type: "text/html".to_string(),
                extracted_doc: None,
            })
        }

        async fn execute_batch(
            &self,
            urls: &[String],
        ) -> (Vec<Option<PipelineResult>>, PipelineStats) {
            let results = urls.iter().map(|_| None).collect();
            let stats = PipelineStats {
                total_urls: urls.len() as u64,
                successful: 0,
                failed: 0,
                cached: 0,
                duration_ms: 0,
                gate_decisions: Default::default(),
            };
            (results, stats)
        }
    }

    #[tokio::test]
    async fn test_trait_object_creation() {
        let executor: Box<dyn PipelineExecutor> = Box::new(MockPipelineExecutor);
        let result = executor.execute_single("https://example.com").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_batch_execution() {
        let executor = MockPipelineExecutor;
        let urls = vec!["https://example.com".to_string()];
        let (results, _stats) = executor.execute_batch(&urls).await;
        assert_eq!(results.len(), 1);
    }
}
