//! Pipeline module - Type definitions and traits for pipeline orchestration
//!
//! This module provides:
//! - Trait definitions for pipeline executors
//! - Result types for pipeline operations
//! - Configuration types for pipeline behavior
//! - Facade domain types for type-safe operations

pub mod facade_types;
pub mod results;
pub mod traits;

// Re-export traits for convenient access
pub use traits::{CombinedPipelineExecutor, PipelineExecutor, StrategiesPipelineExecutor};

// Re-export result types
pub use results::{
    DualPathResult, EnhancementResult, FastPathResult, GateDecisionStats, PipelineResult,
    PipelineRetryConfig, PipelineStats, StrategiesPipelineResult,
};

// Re-export facade domain types
pub use facade_types::{
    ExtractedData as FacadeExtractedData, FetchData, FieldValue, LocalStorage, PipelineOutput,
    SchemaExtractionResult, ScriptResult, ScriptValue, ScriptValueType, StorageConfirmation,
    TransformedData, ValidationResult,
};
