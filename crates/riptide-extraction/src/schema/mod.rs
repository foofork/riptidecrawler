//! Schema extraction and learning module
//!
//! This module provides functionality for learning, testing, and managing extraction schemas.
//! Schemas define how to extract structured data from web pages using CSS selectors and validation rules.

pub mod comparator;
pub mod extractor;
pub mod generator;
pub mod registry;
pub mod types;
pub mod validator;

// Re-export main types and interfaces
pub use types::{
    ExtractionSchema, FieldSchema, SchemaAnalysis, SchemaLearnRequest, SchemaLearnResponse,
    SchemaMetadata, SchemaTestRequest, SchemaTestResponse, SelectorRule, TestResult, TestSummary,
    ValidationRules,
};

pub use comparator::SchemaComparator;
pub use extractor::SchemaExtractor;
pub use generator::SchemaGenerator;
pub use registry::SchemaRegistry;
pub use validator::SchemaValidator;
