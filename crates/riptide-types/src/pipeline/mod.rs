//! Pipeline module - Type definitions and traits for pipeline orchestration
//!
//! This module provides:
//! - Trait definitions for pipeline executors
//! - Result types for pipeline operations
//! - Configuration types for pipeline behavior

pub mod results;
pub mod traits;

// Re-export traits for convenient access
pub use traits::{CombinedPipelineExecutor, PipelineExecutor, StrategiesPipelineExecutor};

// Re-export result types
pub use results::{
    DualPathResult, EnhancementResult, FastPathResult, GateDecisionStats, PipelineResult,
    PipelineRetryConfig, PipelineStats, StrategiesPipelineResult,
};
