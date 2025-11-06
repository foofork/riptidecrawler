// ============================================================================
// TRAIT EXTRACTION CODE - Phase 2C.2
// ============================================================================
//
// This file contains the EXACT Rust code for trait extraction.
// Ready for copy-paste implementation.
//
// File locations:
// - Trait definitions: crates/riptide-types/src/pipeline/traits.rs
// - Implementations:   crates/riptide-api/src/pipeline.rs
//                      crates/riptide-api/src/strategies_pipeline.rs
// - Usage:             crates/riptide-facade/src/facades/crawl_facade.rs

// ============================================================================
// PART 1: Trait Definitions (crates/riptide-types/src/pipeline/traits.rs)
// ============================================================================

use async_trait::async_trait;
use crate::error::{RiptideError, RiptideResult};

// Re-export pipeline types
use crate::pipeline::{PipelineResult, PipelineStats, StrategiesPipelineResult};

/// Core pipeline execution trait for single URL processing.
///
/// This trait defines the minimal interface for any pipeline orchestrator
/// that processes URLs into extracted documents.
///
/// # Design Rationale
///
/// - **Minimal Surface Area:** Only the method actually used by facades
/// - **Async Support:** Uses async-trait for async methods
/// - **Error Handling:** Uses riptide-types error types (no riptide-api deps)
/// - **Send + Sync:** Required for multi-threaded async contexts
///
/// # Example
///
/// ```no_run
/// use riptide_types::pipeline::traits::PipelineExecutor;
///
/// async fn process_url(executor: &dyn PipelineExecutor, url: &str) {
///     let result = executor.execute_single(url).await?;
///     println!("Extracted: {}", result.document.title.unwrap_or_default());
/// }
/// ```
#[async_trait]
pub trait PipelineExecutor: Send + Sync {
    /// Execute pipeline for a single URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to process
    ///
    /// # Returns
    ///
    /// `PipelineResult` containing extracted document and metadata
    ///
    /// # Errors
    ///
    /// Returns error for:
    /// - Invalid URLs
    /// - Network failures
    /// - Extraction failures
    /// - Cache errors
    /// - Timeouts
    async fn execute_single(&self, url: &str) -> RiptideResult<PipelineResult>;
}

/// Batch pipeline execution trait for processing multiple URLs.
///
/// Extends the core executor with batch processing capabilities,
/// allowing efficient concurrent processing of multiple URLs.
///
/// # Example
///
/// ```no_run
/// use riptide_types::pipeline::traits::BatchPipelineExecutor;
///
/// async fn process_batch(executor: &dyn BatchPipelineExecutor) {
///     let urls = vec![
///         "https://example.com".to_string(),
///         "https://test.com".to_string(),
///     ];
///     let (results, stats) = executor.execute_batch(&urls).await;
///     println!("Processed {} URLs", stats.total_processed);
/// }
/// ```
#[async_trait]
pub trait BatchPipelineExecutor: PipelineExecutor {
    /// Execute pipeline for multiple URLs concurrently.
    ///
    /// # Arguments
    ///
    /// * `urls` - List of URLs to process
    ///
    /// # Returns
    ///
    /// Tuple of (results, statistics):
    /// - Results: Vec where Some(result) = success, None = failure
    /// - Statistics: Aggregate metrics across all URLs
    ///
    /// # Performance
    ///
    /// Uses semaphore-based concurrency control to prevent overwhelming
    /// target servers while maximizing throughput.
    async fn execute_batch(
        &self,
        urls: &[String],
    ) -> (Vec<Option<PipelineResult>>, PipelineStats);
}

/// Enhanced pipeline execution with extraction strategies.
///
/// This trait extends the core executor to support advanced extraction
/// strategies with chunking and performance metrics.
///
/// # Features
///
/// - Multiple extraction strategies (trek, css_json, regex, llm)
/// - Configurable chunking modes (regex, sentence, topic, fixed, sliding)
/// - Performance tracking and metrics
/// - Strategy selection based on content analysis
///
/// # Example
///
/// ```no_run
/// use riptide_types::pipeline::traits::StrategiesPipelineExecutor;
///
/// async fn process_with_strategies(executor: &dyn StrategiesPipelineExecutor) {
///     let result = executor.execute_single("https://example.com").await?;
///     if let Some(content) = result.processed_content {
///         println!("Chunks: {}", content.chunks.len());
///     }
/// }
/// ```
#[async_trait]
pub trait StrategiesPipelineExecutor: Send + Sync {
    /// Execute strategies pipeline for a single URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to process
    ///
    /// # Returns
    ///
    /// `StrategiesPipelineResult` containing:
    /// - Processed content with chunking
    /// - Extraction metadata
    /// - Strategy configuration used
    /// - Performance metrics (if enabled)
    ///
    /// # Errors
    ///
    /// Returns error for:
    /// - Network failures
    /// - Strategy processing failures
    /// - Content analysis failures
    async fn execute_single(&self, url: &str) -> RiptideResult<StrategiesPipelineResult>;
}

// ============================================================================
// PART 2: Trait Implementations (crates/riptide-api/src/pipeline.rs)
// ============================================================================

// Add these implementations to the existing PipelineOrchestrator

use async_trait::async_trait;
use riptide_types::pipeline::traits::{PipelineExecutor, BatchPipelineExecutor};
use riptide_types::error::{RiptideError, RiptideResult};
use riptide_pipeline::{PipelineResult, PipelineStats};

#[async_trait]
impl PipelineExecutor for PipelineOrchestrator {
    async fn execute_single(&self, url: &str) -> RiptideResult<PipelineResult> {
        // Delegate to existing method (1,071 lines preserved!)
        self.execute_single(url)
            .await
            .map_err(|e| RiptideError::Other(Box::new(e)))
    }
}

#[async_trait]
impl BatchPipelineExecutor for PipelineOrchestrator {
    async fn execute_batch(
        &self,
        urls: &[String],
    ) -> (Vec<Option<PipelineResult>>, PipelineStats) {
        // Delegate to existing method (batch logic preserved!)
        self.execute_batch(urls).await
    }
}

// ============================================================================
// PART 3: Trait Implementations (crates/riptide-api/src/strategies_pipeline.rs)
// ============================================================================

// Add this implementation to the existing StrategiesPipelineOrchestrator

use async_trait::async_trait;
use riptide_types::pipeline::traits::StrategiesPipelineExecutor;
use riptide_types::error::{RiptideError, RiptideResult};
use riptide_pipeline::StrategiesPipelineResult;

#[async_trait]
impl StrategiesPipelineExecutor for StrategiesPipelineOrchestrator {
    async fn execute_single(&self, url: &str) -> RiptideResult<StrategiesPipelineResult> {
        // Delegate to existing method (525 lines preserved!)
        let result = self.execute_single(url)
            .await
            .map_err(|e| RiptideError::Other(Box::new(e)))?;

        // Convert from riptide_api::StrategiesPipelineResult to riptide_pipeline::StrategiesPipelineResult
        // This uses the From implementation already defined in riptide-api
        Ok(result.into())
    }
}

// ============================================================================
// PART 4: CrawlFacade Updates (crates/riptide-facade/src/facades/crawl_facade.rs)
// ============================================================================

// Replace concrete types with trait objects

use riptide_types::pipeline::traits::{
    PipelineExecutor, BatchPipelineExecutor, StrategiesPipelineExecutor
};
use std::sync::Arc;

/// CrawlFacade - Thin wrapper for production pipeline orchestrators
///
/// **UPDATED: Now uses trait objects instead of concrete types**
pub struct CrawlFacade {
    /// Reference to standard pipeline executor (trait object)
    pipeline_orchestrator: Arc<dyn BatchPipelineExecutor>,

    /// Reference to strategies pipeline executor (trait object)
    strategies_orchestrator: Arc<dyn StrategiesPipelineExecutor>,
}

impl CrawlFacade {
    /// Create a new CrawlFacade with generic executors
    ///
    /// This allows any implementation of the traits to be used,
    /// breaking the dependency on concrete riptide-api types.
    ///
    /// # Arguments
    ///
    /// * `pipeline` - Any type implementing BatchPipelineExecutor
    /// * `strategies` - Any type implementing StrategiesPipelineExecutor
    ///
    /// # Example
    ///
    /// ```no_run
    /// use riptide_api::pipeline::PipelineOrchestrator;
    /// use riptide_api::strategies_pipeline::StrategiesPipelineOrchestrator;
    /// use riptide_facade::facades::CrawlFacade;
    ///
    /// let state = AppState::new().await?;
    /// let options = CrawlOptions::default();
    ///
    /// let pipeline = PipelineOrchestrator::new(state.clone(), options.clone());
    /// let strategies = StrategiesPipelineOrchestrator::new(state, options, None);
    ///
    /// let facade = CrawlFacade::new(pipeline, strategies);
    /// ```
    pub fn new<P, S>(pipeline: P, strategies: S) -> Self
    where
        P: BatchPipelineExecutor + 'static,
        S: StrategiesPipelineExecutor + 'static,
    {
        Self {
            pipeline_orchestrator: Arc::new(pipeline),
            strategies_orchestrator: Arc::new(strategies),
        }
    }

    /// Crawl a single URL using the specified mode
    ///
    /// **UNCHANGED: Method signature and logic remain the same**
    pub async fn crawl_single(
        &self,
        url: &str,
        _options: CrawlOptions,
        mode: CrawlMode,
    ) -> RiptideResult<CrawlResult> {
        match mode {
            CrawlMode::Standard => {
                // Delegate to trait method (same as before)
                let result = self
                    .pipeline_orchestrator
                    .execute_single(url)
                    .await?;
                Ok(CrawlResult::Standard(result))
            }
            CrawlMode::Enhanced => {
                // Delegate to trait method (same as before)
                let result = self
                    .strategies_orchestrator
                    .execute_single(url)
                    .await?;
                Ok(CrawlResult::Enhanced(result))
            }
        }
    }

    /// Batch crawl multiple URLs using standard mode
    ///
    /// **UNCHANGED: Method signature and logic remain the same**
    pub async fn crawl_batch(
        &self,
        urls: &[String],
    ) -> (Vec<Option<PipelineResult>>, PipelineStats) {
        // Delegate to trait method (same as before)
        self.pipeline_orchestrator.execute_batch(urls).await
    }
}

// ============================================================================
// PART 5: Cargo.toml Updates
// ============================================================================

// File: crates/riptide-types/Cargo.toml
// Add async-trait dependency:
//
// [dependencies]
// async-trait = "0.1"
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"
// thiserror = "1.0"

// File: crates/riptide-facade/Cargo.toml
// Remove riptide-api dependency:
//
// [dependencies]
// # REMOVED: riptide-api = { path = "../riptide-api" }  <-- Circular!
//
// # KEPT: Types-only dependencies (no circular!)
// riptide-types = { path = "../riptide-types" }
// riptide-pipeline = { path = "../riptide-pipeline" }
// riptide-extraction = { path = "../riptide-extraction" }
// async-trait = "0.1"
// tokio = { version = "1.0", features = ["full"] }

// ============================================================================
// PART 6: Module Exports
// ============================================================================

// File: crates/riptide-types/src/pipeline/mod.rs
// Add trait exports:
//
// pub mod traits;
//
// // Re-export traits at module level
// pub use traits::{
//     PipelineExecutor,
//     BatchPipelineExecutor,
//     StrategiesPipelineExecutor,
// };

// File: crates/riptide-types/src/lib.rs
// Ensure pipeline module is exported:
//
// pub mod pipeline;

// ============================================================================
// VERIFICATION COMMANDS
// ============================================================================

// 1. Check for circular dependencies:
// cargo tree --no-dedupe | grep -E "(riptide-facade|riptide-api)"

// 2. Build with traits:
// cargo build --workspace

// 3. Run tests:
// cargo test --workspace

// 4. Verify trait implementations:
// cargo clippy --workspace -- -D warnings

// ============================================================================
// SUCCESS CRITERIA
// ============================================================================

// ✅ Circular dependency removed
// ✅ All tests pass
// ✅ No production code refactored
// ✅ Trait objects work correctly
// ✅ cargo tree shows acyclic dependencies
