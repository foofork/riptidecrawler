//! Content extraction port for hexagonal architecture
//!
//! Provides backend-agnostic trait for content extraction from HTML,
//! enabling dependency inversion and testability.

use crate::error::Result as RiptideResult;
use async_trait::async_trait;

/// Content extraction result
#[derive(Debug, Clone)]
pub struct ExtractionResult {
    /// Extracted text content
    pub text: String,
    /// Extracted metadata (title, description, etc.)
    pub metadata: std::collections::HashMap<String, String>,
    /// Extraction quality score (0.0 to 1.0)
    pub quality_score: f64,
}

/// Content extractor port trait
///
/// Defines the interface for content extraction implementations.
/// Concrete adapters (e.g., WasmExtractor, NativeExtractor, UnifiedExtractor)
/// implement this trait to provide actual extraction logic.
///
/// # Example
///
/// ```rust,ignore
/// use riptide_types::ports::ContentExtractor;
///
/// async fn extract_content(
///     extractor: &dyn ContentExtractor,
///     html: &str,
/// ) -> Result<String> {
///     let result = extractor.extract(html, "https://example.com").await?;
///     Ok(result.text)
/// }
/// ```
#[async_trait]
pub trait ContentExtractor: Send + Sync {
    /// Extract content from HTML
    ///
    /// # Arguments
    /// * `html` - The HTML content to extract from
    /// * `url` - The URL of the page (for context)
    ///
    /// # Returns
    /// * `Ok(ExtractionResult)` - Extracted content and metadata
    /// * `Err(_)` - Extraction error
    async fn extract(&self, html: &str, url: &str) -> RiptideResult<ExtractionResult>;

    /// Get the extractor type/name
    ///
    /// # Returns
    /// String identifying the extractor implementation (e.g., "wasm", "native", "unified")
    fn extractor_type(&self) -> &str;

    /// Check if extractor is available/healthy
    ///
    /// # Returns
    /// `true` if extractor is ready to process requests
    async fn is_available(&self) -> bool;
}

/// Reliable extractor port trait with retry and circuit breaker logic
///
/// Extends basic content extraction with fault tolerance patterns.
#[async_trait]
pub trait ReliableContentExtractor: Send + Sync {
    /// Extract content with retry logic
    ///
    /// # Arguments
    /// * `html` - The HTML content to extract from
    /// * `url` - The URL of the page
    ///
    /// # Returns
    /// * `Ok(ExtractionResult)` - Extracted content (possibly with degraded quality)
    /// * `Err(_)` - Extraction failed after retries
    async fn extract_with_retry(&self, html: &str, url: &str) -> RiptideResult<ExtractionResult>;

    /// Get retry statistics
    ///
    /// # Returns
    /// Statistics about retry attempts and success rates
    async fn stats(&self) -> ReliabilityStats;
}

/// Reliability statistics for monitoring
#[derive(Debug, Clone)]
pub struct ReliabilityStats {
    /// Total extraction attempts
    pub total_attempts: u64,
    /// Successful extractions
    pub successes: u64,
    /// Failed extractions
    pub failures: u64,
    /// Average retry count
    pub avg_retries: f64,
    /// Circuit breaker trips
    pub circuit_breaker_trips: u64,
}

impl Default for ReliabilityStats {
    fn default() -> Self {
        Self {
            total_attempts: 0,
            successes: 0,
            failures: 0,
            avg_retries: 0.0,
            circuit_breaker_trips: 0,
        }
    }
}
