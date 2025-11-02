//! Configuration management for RipTide
//!
//! This crate provides comprehensive configuration loading, validation, and builder patterns
//! for the RipTide web scraping system. It consolidates configuration logic previously
//! scattered across riptide-core and other crates.
//!
//! # Features
//!
//! - **Builder Pattern**: Flexible configuration builders with validation
//! - **Environment Variables**: Automatic loading from environment
//! - **Validation**: Comprehensive security and format validation
//! - **Spider Config**: Specialized spider crawling configurations
//! - **Type Safety**: Strong typing with compile-time guarantees
//!
//! # Example
//!
//! ```rust
//! use riptide_config::{SpiderConfig, SpiderPresets, ValidationConfig, CommonValidator};
//!
//! // Create a configuration using presets
//! let config = SpiderPresets::development();
//!
//! // Or build custom configuration
//! let custom = SpiderConfig::default()
//!     .with_concurrency(8)
//!     .with_timeout(std::time::Duration::from_secs(30));
//! ```

mod api;
mod builder;
mod env;
mod spider;
mod validation;

// Re-export main types
pub use api::{ApiConfig, AuthenticationConfig, CustomRateLimit, RateLimitConfig, RequestConfig};

pub use builder::{
    BuilderError, BuilderResult, ConfigBuilder, ConfigValidator, ConfigValue, DefaultConfigBuilder,
    ValidationPatterns,
};

pub use env::{load_from_env, EnvConfigLoader, EnvError};

pub use spider::{PerformanceConfig, SpiderConfig, SpiderPresets, UrlProcessingConfig};

pub use validation::{
    CommonValidator, ContentTypeValidator, ParameterValidator, SizeValidator, UrlValidator,
    ValidationConfig, ValidationResult, ALLOWED_CONTENT_TYPES, DEFAULT_MAX_CONTENT_SIZE,
    MAX_HEADER_SIZE, MAX_URL_LENGTH,
};

// Re-export config_builder macro
#[macro_export]
macro_rules! config_builder {
    (
        $name:ident {
            $(
                $field:ident: $type:ty $(= $default:expr)?,
            )*
        }
    ) => {
        pub struct $name {
            $(
                $field: Option<$type>,
            )*
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    $(
                        $field: None,
                    )*
                }
            }

            $(
                pub fn $field(mut self, value: $type) -> Self {
                    self.$field = Some(value);
                    self
                }

                paste::paste! {
                    pub fn [<set_ $field>](&mut self, value: $type) -> &mut Self {
                        self.$field = Some(value);
                        self
                    }

                    pub fn [<get_ $field>](&self) -> Option<&$type> {
                        self.$field.as_ref()
                    }
                }
            )*

            pub fn build(self) -> $crate::BuilderResult<$name> {
                // Validate required fields and apply defaults
                Ok($name {
                    $(
                        $field: self.$field.or_else(|| {
                            $(Some($default))?
                        }).ok_or_else(|| $crate::BuilderError::MissingRequired {
                            field: stringify!($field).to_string(),
                        })?,
                    )*
                })
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}
