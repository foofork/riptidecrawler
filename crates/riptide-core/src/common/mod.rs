//! Common utilities and shared code across RipTide modules.
//!
//! This module provides reusable components to reduce code duplication
//! and maintain consistency across the codebase.

pub mod validation;
pub mod error_conversions;
pub mod config_builder;

// Re-export commonly used items
pub use validation::{
    CommonValidator, ValidationConfig, ValidationResult, ContentTypeValidator,
    UrlValidator, SizeValidator, ParameterValidator,
};
pub use error_conversions::{
    ErrorConverter, CoreErrorConverter, WithErrorContext,
    IntoCore, ErrorPatterns,
};

// Export API integration features conditionally
#[cfg(feature = "api-integration")]
pub use error_conversions::{ApiErrorConverter, IntoApiString};
pub use config_builder::{
    ConfigBuilder, ConfigValidator, DefaultConfigBuilder,
    BuilderError, BuilderResult,
};