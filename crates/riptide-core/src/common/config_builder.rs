//! Common configuration builder patterns and utilities.
//!
//! This module provides reusable configuration building patterns to reduce
//! duplicate builder implementations across the codebase.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

/// Builder error types
#[derive(Error, Debug)]
pub enum BuilderError {
    #[error("Missing required field: {field}")]
    MissingRequired { field: String },

    #[error("Invalid value for {field}: {reason}")]
    InvalidValue { field: String, reason: String },

    #[error("Configuration validation failed: {reason}")]
    ValidationFailed { reason: String },

    #[error("Environment variable error: {var} - {reason}")]
    EnvError { var: String, reason: String },

    #[error("Type conversion error for {field}: {reason}")]
    ConversionError { field: String, reason: String },
}

/// Builder result type
pub type BuilderResult<T> = Result<T, BuilderError>;

/// Trait for configuration builders
pub trait ConfigBuilder<T> {
    /// Build the final configuration
    fn build(self) -> BuilderResult<T>;

    /// Validate the configuration
    fn validate(&self) -> BuilderResult<()>;

    /// Set a field from environment variable
    fn load_from_env_var(&mut self, field: &str, env_var: &str) -> &mut Self;

    /// Set multiple fields from environment
    fn load_from_env(&mut self) -> &mut Self;
}

/// Trait for configuration validation
pub trait ConfigValidator {
    /// Validate the configuration
    fn validate(&self) -> BuilderResult<()>;

    /// Get validation errors as a list
    fn validation_errors(&self) -> Vec<BuilderError> {
        match self.validate() {
            Ok(()) => vec![],
            Err(e) => vec![e],
        }
    }

    /// Check if configuration is valid
    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}

/// Default configuration builder with common patterns
pub struct DefaultConfigBuilder<T> {
    fields: HashMap<String, ConfigValue>,
    required_fields: Vec<String>,
    defaults: HashMap<String, ConfigValue>,
    _phantom: std::marker::PhantomData<T>,
}

/// Configuration value wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    UnsignedInteger(u64),
    Float(f64),
    Boolean(bool),
    Duration(Duration),
    OptionalString(Option<String>),
    OptionalInteger(Option<i64>),
    StringList(Vec<String>),
}

impl ConfigValue {
    /// Convert to string
    pub fn as_string(&self) -> BuilderResult<String> {
        match self {
            ConfigValue::String(s) => Ok(s.clone()),
            ConfigValue::OptionalString(Some(s)) => Ok(s.clone()),
            _ => Err(BuilderError::ConversionError {
                field: "unknown".to_string(),
                reason: format!("Cannot convert {:?} to string", self),
            }),
        }
    }

    /// Convert to integer
    pub fn as_integer(&self) -> BuilderResult<i64> {
        match self {
            ConfigValue::Integer(i) => Ok(*i),
            ConfigValue::OptionalInteger(Some(i)) => Ok(*i),
            ConfigValue::String(s) => s.parse().map_err(|e| BuilderError::ConversionError {
                field: "unknown".to_string(),
                reason: format!("Cannot parse '{}' as integer: {}", s, e),
            }),
            _ => Err(BuilderError::ConversionError {
                field: "unknown".to_string(),
                reason: format!("Cannot convert {:?} to integer", self),
            }),
        }
    }

    /// Convert to unsigned integer
    pub fn as_unsigned_integer(&self) -> BuilderResult<u64> {
        match self {
            ConfigValue::UnsignedInteger(u) => Ok(*u),
            ConfigValue::Integer(i) if *i >= 0 => Ok(*i as u64),
            ConfigValue::String(s) => s.parse().map_err(|e| BuilderError::ConversionError {
                field: "unknown".to_string(),
                reason: format!("Cannot parse '{}' as unsigned integer: {}", s, e),
            }),
            _ => Err(BuilderError::ConversionError {
                field: "unknown".to_string(),
                reason: format!("Cannot convert {:?} to unsigned integer", self),
            }),
        }
    }

    /// Convert to float
    pub fn as_float(&self) -> BuilderResult<f64> {
        match self {
            ConfigValue::Float(f) => Ok(*f),
            ConfigValue::Integer(i) => Ok(*i as f64),
            ConfigValue::UnsignedInteger(u) => Ok(*u as f64),
            ConfigValue::String(s) => s.parse().map_err(|e| BuilderError::ConversionError {
                field: "unknown".to_string(),
                reason: format!("Cannot parse '{}' as float: {}", s, e),
            }),
            _ => Err(BuilderError::ConversionError {
                field: "unknown".to_string(),
                reason: format!("Cannot convert {:?} to float", self),
            }),
        }
    }

    /// Convert to boolean
    pub fn as_boolean(&self) -> BuilderResult<bool> {
        match self {
            ConfigValue::Boolean(b) => Ok(*b),
            ConfigValue::String(s) => match s.to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => Ok(true),
                "false" | "0" | "no" | "off" => Ok(false),
                _ => Err(BuilderError::ConversionError {
                    field: "unknown".to_string(),
                    reason: format!("Cannot parse '{}' as boolean", s),
                }),
            },
            _ => Err(BuilderError::ConversionError {
                field: "unknown".to_string(),
                reason: format!("Cannot convert {:?} to boolean", self),
            }),
        }
    }

    /// Convert to duration
    pub fn as_duration(&self) -> BuilderResult<Duration> {
        match self {
            ConfigValue::Duration(d) => Ok(*d),
            ConfigValue::Integer(i) => Ok(Duration::from_secs(*i as u64)),
            ConfigValue::UnsignedInteger(u) => Ok(Duration::from_secs(*u)),
            ConfigValue::String(s) => {
                // Parse duration strings like "30s", "5m", "1h"
                parse_duration_string(s)
            }
            _ => Err(BuilderError::ConversionError {
                field: "unknown".to_string(),
                reason: format!("Cannot convert {:?} to duration", self),
            }),
        }
    }

    /// Convert to optional string
    pub fn as_optional_string(&self) -> BuilderResult<Option<String>> {
        match self {
            ConfigValue::OptionalString(opt) => Ok(opt.clone()),
            ConfigValue::String(s) if !s.is_empty() => Ok(Some(s.clone())),
            ConfigValue::String(_) => Ok(None),
            _ => Err(BuilderError::ConversionError {
                field: "unknown".to_string(),
                reason: format!("Cannot convert {:?} to optional string", self),
            }),
        }
    }

    /// Convert to string list
    pub fn as_string_list(&self) -> BuilderResult<Vec<String>> {
        match self {
            ConfigValue::StringList(list) => Ok(list.clone()),
            ConfigValue::String(s) => {
                // Split by comma and trim whitespace
                Ok(s.split(',').map(|s| s.trim().to_string()).collect())
            }
            _ => Err(BuilderError::ConversionError {
                field: "unknown".to_string(),
                reason: format!("Cannot convert {:?} to string list", self),
            }),
        }
    }
}

impl<T> DefaultConfigBuilder<T> {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            required_fields: Vec::new(),
            defaults: HashMap::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Add a required field
    pub fn require_field(&mut self, field: &str) -> &mut Self {
        self.required_fields.push(field.to_string());
        self
    }

    /// Set a default value
    pub fn default_value(&mut self, field: &str, value: ConfigValue) -> &mut Self {
        self.defaults.insert(field.to_string(), value);
        self
    }

    /// Set a field value
    pub fn set_field(&mut self, field: &str, value: ConfigValue) -> &mut Self {
        self.fields.insert(field.to_string(), value);
        self
    }

    /// Get field value with fallback to default
    pub fn get_field(&self, field: &str) -> Option<&ConfigValue> {
        self.fields.get(field).or_else(|| self.defaults.get(field))
    }

    /// Get field value or return error
    pub fn get_required_field(&self, field: &str) -> BuilderResult<&ConfigValue> {
        self.get_field(field).ok_or_else(|| BuilderError::MissingRequired {
            field: field.to_string(),
        })
    }

    /// Load from environment variables with prefix
    pub fn from_env_with_prefix(&mut self, prefix: &str) -> &mut Self {
        for (key, value) in std::env::vars() {
            if let Some(field_name) = key.strip_prefix(&format!("{}_", prefix)) {
                let field_name = field_name.to_lowercase();
                self.fields.insert(field_name, ConfigValue::String(value));
            }
        }
        self
    }

    /// Validate required fields are present
    pub fn validate_required_fields(&self) -> BuilderResult<()> {
        for field in &self.required_fields {
            if !self.fields.contains_key(field) && !self.defaults.contains_key(field) {
                return Err(BuilderError::MissingRequired {
                    field: field.clone(),
                });
            }
        }
        Ok(())
    }
}

impl<T> Default for DefaultConfigBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Common configuration validation patterns
pub struct ValidationPatterns;

impl ValidationPatterns {
    /// Validate positive integer
    pub fn validate_positive_integer(value: i64, field: &str) -> BuilderResult<()> {
        if value <= 0 {
            return Err(BuilderError::InvalidValue {
                field: field.to_string(),
                reason: "must be greater than 0".to_string(),
            });
        }
        Ok(())
    }

    /// Validate range
    pub fn validate_range(value: f64, min: f64, max: f64, field: &str) -> BuilderResult<()> {
        if value < min || value > max {
            return Err(BuilderError::InvalidValue {
                field: field.to_string(),
                reason: format!("must be between {} and {}", min, max),
            });
        }
        Ok(())
    }

    /// Validate URL format
    pub fn validate_url(url: &str, field: &str) -> BuilderResult<()> {
        if url::Url::parse(url).is_err() {
            return Err(BuilderError::InvalidValue {
                field: field.to_string(),
                reason: "invalid URL format".to_string(),
            });
        }
        Ok(())
    }

    /// Validate non-empty string
    pub fn validate_non_empty_string(value: &str, field: &str) -> BuilderResult<()> {
        if value.trim().is_empty() {
            return Err(BuilderError::InvalidValue {
                field: field.to_string(),
                reason: "cannot be empty".to_string(),
            });
        }
        Ok(())
    }

    /// Validate duration is positive
    pub fn validate_positive_duration(duration: Duration, field: &str) -> BuilderResult<()> {
        if duration.is_zero() {
            return Err(BuilderError::InvalidValue {
                field: field.to_string(),
                reason: "duration must be greater than zero".to_string(),
            });
        }
        Ok(())
    }
}

/// Parse duration string like "30s", "5m", "1h"
fn parse_duration_string(s: &str) -> BuilderResult<Duration> {
    let s = s.trim().to_lowercase();

    if s.ends_with("ms") {
        let num = s[..s.len() - 2].parse::<u64>().map_err(|e| BuilderError::ConversionError {
            field: "duration".to_string(),
            reason: format!("Invalid milliseconds value: {}", e),
        })?;
        Ok(Duration::from_millis(num))
    } else if s.ends_with('s') {
        let num = s[..s.len() - 1].parse::<u64>().map_err(|e| BuilderError::ConversionError {
            field: "duration".to_string(),
            reason: format!("Invalid seconds value: {}", e),
        })?;
        Ok(Duration::from_secs(num))
    } else if s.ends_with('m') {
        let num = s[..s.len() - 1].parse::<u64>().map_err(|e| BuilderError::ConversionError {
            field: "duration".to_string(),
            reason: format!("Invalid minutes value: {}", e),
        })?;
        Ok(Duration::from_secs(num * 60))
    } else if s.ends_with('h') {
        let num = s[..s.len() - 1].parse::<u64>().map_err(|e| BuilderError::ConversionError {
            field: "duration".to_string(),
            reason: format!("Invalid hours value: {}", e),
        })?;
        Ok(Duration::from_secs(num * 3600))
    } else if let Ok(num) = s.parse::<u64>() {
        // Default to seconds if no unit specified
        Ok(Duration::from_secs(num))
    } else {
        Err(BuilderError::ConversionError {
            field: "duration".to_string(),
            reason: format!("Invalid duration format: {}", s),
        })
    }
}

/// Macro for creating configuration builders
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

            pub fn build(self) -> $crate::common::config_builder::BuilderResult<$name> {
                // Validate required fields and apply defaults
                Ok($name {
                    $(
                        $field: self.$field.or_else(|| {
                            $(Some($default))?
                        }).ok_or_else(|| $crate::common::config_builder::BuilderError::MissingRequired {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_value_conversions() {
        let string_val = ConfigValue::String("test".to_string());
        assert_eq!(string_val.as_string().unwrap(), "test");

        let int_val = ConfigValue::Integer(42);
        assert_eq!(int_val.as_integer().unwrap(), 42);

        let bool_val = ConfigValue::String("true".to_string());
        assert!(bool_val.as_boolean().unwrap());

        let duration_val = ConfigValue::String("30s".to_string());
        assert_eq!(duration_val.as_duration().unwrap(), Duration::from_secs(30));

        let list_val = ConfigValue::String("a,b,c".to_string());
        assert_eq!(list_val.as_string_list().unwrap(), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_duration_parsing() {
        assert_eq!(parse_duration_string("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration_string("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration_string("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration_string("500ms").unwrap(), Duration::from_millis(500));
        assert_eq!(parse_duration_string("60").unwrap(), Duration::from_secs(60));
    }

    #[test]
    fn test_validation_patterns() {
        assert!(ValidationPatterns::validate_positive_integer(5, "count").is_ok());
        assert!(ValidationPatterns::validate_positive_integer(0, "count").is_err());

        assert!(ValidationPatterns::validate_range(0.5, 0.0, 1.0, "ratio").is_ok());
        assert!(ValidationPatterns::validate_range(1.5, 0.0, 1.0, "ratio").is_err());

        assert!(ValidationPatterns::validate_url("https://example.com", "url").is_ok());
        assert!(ValidationPatterns::validate_url("invalid-url", "url").is_err());

        assert!(ValidationPatterns::validate_non_empty_string("test", "name").is_ok());
        assert!(ValidationPatterns::validate_non_empty_string("", "name").is_err());
    }

    #[test]
    fn test_default_config_builder() {
        let mut builder = DefaultConfigBuilder::<String>::new();
        builder
            .require_field("name")
            .default_value("timeout", ConfigValue::Integer(30))
            .set_field("name", ConfigValue::String("test".to_string()));

        assert!(builder.validate_required_fields().is_ok());

        let name = builder.get_required_field("name").unwrap();
        assert_eq!(name.as_string().unwrap(), "test");

        let timeout = builder.get_field("timeout").unwrap();
        assert_eq!(timeout.as_integer().unwrap(), 30);
    }
}