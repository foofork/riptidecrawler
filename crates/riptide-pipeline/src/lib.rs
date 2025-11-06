//! RipTide Pipeline Types and Orchestration
//!
//! **IMPORTANT: As of Phase 2C.2, pipeline result types have been moved to `riptide-types`.**
//!
//! This crate now re-exports types from `riptide-types::pipeline` for backward compatibility.
//! All new code should import from `riptide-types::pipeline` directly.
//!
//! # Architecture (Updated)
//!
//! ```text
//! ┌─────────────────┐
//! │ riptide-facade  │  (High-level API)
//! └────────┬────────┘
//!          │ depends on
//!          ↓
//! ┌─────────────────┐
//! │ riptide-types   │  (Foundation - pipeline types + traits)
//! └────────┬────────┘
//!          │ used by both
//!    ┌─────┴─────┐
//!    ↓           ↓
//! ┌─────────────────┐   ┌─────────────────┐
//! │ riptide-pipeline│   │  riptide-api    │
//! │ (re-exports)    │   │  (implements)   │
//! └─────────────────┘   └─────────────────┘
//! ```
//!
//! ## Breaking the Circular Dependency
//!
//! Previously: riptide-api ↔ riptide-facade (circular!)
//! Phase 2C.1: riptide-facade → riptide-pipeline ← riptide-api
//! Phase 2C.2: riptide-facade → riptide-types ← riptide-api (traits extracted!)

// Re-export ALL pipeline types from riptide-types (moved in Phase 2C.2)
pub use riptide_types::pipeline::{
    DualPathResult, EnhancementResult, FastPathResult, GateDecisionStats, PipelineResult,
    PipelineRetryConfig, PipelineStats, StrategiesPipelineResult,
};

// Re-export types from dependencies
pub use riptide_types::config::CrawlOptions;

#[cfg(test)]
mod tests {
    use super::*;
    use riptide_types::ExtractedDoc;

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
