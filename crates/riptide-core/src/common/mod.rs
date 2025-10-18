//! Common utilities and shared code across RipTide modules.
//!
//! This module provides reusable components to reduce code duplication
//! and maintain consistency across the codebase.
//!
//! # Migration Note
//!
//! Configuration and validation functionality has been fully migrated to `riptide-config` crate.
//! All config builder types are now re-exported from riptide-config only.
//! The duplicate config_builder.rs has been removed.

pub mod error_conversions;

// Re-export from riptide-config for backward compatibility
pub use riptide_config::{
    BuilderError, BuilderResult, CommonValidator, ConfigBuilder, ConfigValidator,
    ContentTypeValidator, DefaultConfigBuilder, ParameterValidator, SizeValidator, UrlValidator,
    ValidationConfig, ValidationResult,
};

// Re-export error conversion utilities
pub use error_conversions::{
    CoreErrorConverter, ErrorConverter, ErrorPatterns, IntoCore, WithErrorContext,
};

// Export API integration features conditionally
#[cfg(feature = "api-integration")]
pub use error_conversions::{ApiErrorConverter, IntoApiString};
