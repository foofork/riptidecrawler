//! RipTide Pipeline Types and Orchestration
//!
//! This crate provides the core pipeline orchestration types for RipTide's
//! extraction workflow. It defines the public interfaces used by both the
//! API layer (riptide-api) and the facade layer (riptide-facade), breaking
//! the circular dependency between them.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────┐
//! │ riptide-facade  │  (High-level API)
//! └────────┬────────┘
//!          │ depends on
//!          ↓
//! ┌─────────────────┐
//! │ riptide-pipeline│  (Pipeline types - THIS CRATE)
//! └────────┬────────┘
//!          │ used by
//!          ↓
//! ┌─────────────────┐
//! │  riptide-api    │  (Implementation)
//! └─────────────────┘
//! ```
//!
//! ## Breaking the Circular Dependency
//!
//! Previously: riptide-api ↔ riptide-facade (circular!)
//! Now: riptide-facade → riptide-pipeline ← riptide-api (acyclic!)

use riptide_types::ExtractedDoc;
use serde::{Deserialize, Serialize};

// Re-export types from dependencies
pub use riptide_types::config::CrawlOptions;

/// Pipeline execution result containing the extracted document and metadata.
///
/// This is the primary output type from pipeline orchestration, containing
/// both the extracted content and metadata about the extraction process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    /// The extracted document content
    pub document: ExtractedDoc,

    /// Whether the content was served from cache
    pub from_cache: bool,

    /// The decision made by the gate (Raw, ProbesFirst, Headless, pdf, cached)
    pub gate_decision: String,

    /// Content quality score from the gate analysis (0.0-1.0)
    pub quality_score: f32,

    /// Total processing time in milliseconds
    pub processing_time_ms: u64,

    /// Cache key used for this URL
    pub cache_key: String,

    /// HTTP status code from the original fetch
    pub http_status: u16,
}

/// Pipeline execution statistics for monitoring and optimization.
///
/// Aggregates metrics across multiple URL extractions for batch processing
/// and performance analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStats {
    /// Total URLs processed
    pub total_processed: usize,

    /// Number of cache hits
    pub cache_hits: usize,

    /// Number of successful extractions
    pub successful_extractions: usize,

    /// Number of failed extractions
    pub failed_extractions: usize,

    /// Gate decision breakdown
    pub gate_decisions: GateDecisionStats,

    /// Average processing time in milliseconds
    pub avg_processing_time_ms: f64,

    /// Total processing time in milliseconds
    pub total_processing_time_ms: u64,
}

/// Breakdown of gate decisions made during pipeline execution.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GateDecisionStats {
    /// Number of URLs processed with raw extraction
    pub raw: usize,

    /// Number of URLs that required probes first
    pub probes_first: usize,

    /// Number of URLs that required headless rendering
    pub headless: usize,
}

/// Retry configuration for pipeline operations.
///
/// Controls retry behavior for transient failures during content extraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineRetryConfig {
    /// Maximum retry attempts
    pub max_retries: usize,

    /// Initial delay in milliseconds
    pub initial_delay_ms: u64,

    /// Maximum delay in milliseconds
    pub max_delay_ms: u64,

    /// Override strategy (requires llm feature in implementation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
}

impl Default for PipelineRetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 30_000,
            strategy: None, // Auto-select
        }
    }
}

// ============================================================================
// Strategies Pipeline Types
// ============================================================================

/// Enhanced pipeline result with extraction strategies integration.
///
/// Used by the strategies-based pipeline orchestrator for more advanced
/// content processing with chunking and performance metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategiesPipelineResult {
    /// The processed content (includes extraction and chunking)
    /// This is serialized as a generic JSON value since ProcessedContent
    /// is defined in riptide-extraction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processed_content: Option<serde_json::Value>,

    /// Whether the content was served from cache
    pub from_cache: bool,

    /// The decision made by the gate
    pub gate_decision: String,

    /// Content quality score
    pub quality_score: f32,

    /// Total processing time in milliseconds
    pub processing_time_ms: u64,

    /// Cache key used
    pub cache_key: String,

    /// HTTP status code
    pub http_status: u16,

    /// Strategy configuration used (serialized as JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy_config: Option<serde_json::Value>,

    /// Performance metrics if enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance_metrics: Option<serde_json::Value>,
}

// ============================================================================
// Dual-Path Pipeline Types (LLM Feature)
// ============================================================================

/// Result from the fast path (CSS extraction).
///
/// Represents the immediate extraction result before AI enhancement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastPathResult {
    /// Unique task identifier
    pub task_id: String,

    /// Source URL
    pub url: String,

    /// Extracted document from fast path
    pub document: ExtractedDoc,

    /// Fast path processing time in milliseconds
    pub processing_time_ms: u64,

    /// Initial quality score
    pub quality_score: f32,
}

/// Result from the AI enhancement path.
///
/// Represents the async AI processing result that enhances the fast path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancementResult {
    /// Task identifier matching FastPathResult
    pub task_id: String,

    /// Source URL
    pub url: String,

    /// Enhanced document (if successful)
    pub enhanced_document: Option<ExtractedDoc>,

    /// Enhancement processing time in milliseconds
    pub processing_time_ms: u64,

    /// Whether enhancement succeeded
    pub success: bool,

    /// Error message if enhancement failed
    pub error: Option<String>,
}

/// Merged result combining fast path and AI enhancement.
///
/// The final output of the dual-path pipeline, combining immediate results
/// with optional AI enhancements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualPathResult {
    /// Task identifier
    pub task_id: String,

    /// Source URL
    pub url: String,

    /// Fast path result (always present)
    pub fast_result: FastPathResult,

    /// Enhancement result (optional, may arrive later)
    pub enhancement_result: Option<EnhancementResult>,

    /// Whether this result includes enhancements
    pub enhanced: bool,

    /// Total processing time including both paths
    pub total_processing_time_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_result_serialization() {
        let result = PipelineResult {
            document: ExtractedDoc {
                url: "https://example.com".to_string(),
                title: Some("Test".to_string()),
                text: "content".to_string(),
                quality_score: Some(85),
                links: vec![],
                byline: None,
                published_iso: None,
                markdown: None,
                parser_metadata: None,
                media: vec![],
                language: None,
                reading_time: None,
                word_count: Some(1),
                categories: vec![],
                site_name: None,
                description: None,
                html: None,
            },
            from_cache: false,
            gate_decision: "raw".to_string(),
            quality_score: 0.85,
            processing_time_ms: 150,
            cache_key: "test:key".to_string(),
            http_status: 200,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: PipelineResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.gate_decision, "raw");
    }

    #[test]
    fn test_retry_config_default() {
        let config = PipelineRetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay_ms, 100);
        assert_eq!(config.max_delay_ms, 30_000);
    }
}
