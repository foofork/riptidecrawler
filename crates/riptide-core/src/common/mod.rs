//! Common utilities and shared code across RipTide modules.
//!
//! This module provides reusable components to reduce code duplication
//! and maintain consistency across the codebase.
//!
//! # Migration Note
//!
//! Configuration and validation functionality has been migrated to `riptide-config` crate.
//! For backward compatibility, these types are re-exported here.

pub mod error_conversions;

// Re-export from riptide-config for backward compatibility
pub use riptide_config::{
    BuilderError, BuilderResult, CommonValidator, ConfigBuilder, ConfigValidator,
    ContentTypeValidator, DefaultConfigBuilder, ParameterValidator, SizeValidator,
    UrlValidator, ValidationConfig, ValidationResult,
};

// Re-export error conversion utilities
pub use error_conversions::{
    CoreErrorConverter, ErrorConverter, ErrorPatterns, IntoCore, WithErrorContext,
};

// Export API integration features conditionally
#[cfg(feature = "api-integration")]
pub use error_conversions::{ApiErrorConverter, IntoApiString};
