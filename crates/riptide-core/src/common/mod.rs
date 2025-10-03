//! Common utilities and shared code across RipTide modules.
//!
//! This module provides reusable components to reduce code duplication
//! and maintain consistency across the codebase.

pub mod config_builder;
pub mod error_conversions;
pub mod validation;

// Re-export commonly used items
pub use error_conversions::{
    CoreErrorConverter, ErrorConverter, ErrorPatterns, IntoCore, WithErrorContext,
};
pub use validation::{
    CommonValidator, ContentTypeValidator, ParameterValidator, SizeValidator, UrlValidator,
    ValidationConfig, ValidationResult,
};

// Export API integration features conditionally
pub use config_builder::{
    BuilderError, BuilderResult, ConfigBuilder, ConfigValidator, DefaultConfigBuilder,
};
#[cfg(feature = "api-integration")]
pub use error_conversions::{ApiErrorConverter, IntoApiString};
