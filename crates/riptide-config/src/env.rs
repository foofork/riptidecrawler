//! Environment variable loading and configuration management
//!
//! This module provides utilities for loading configuration from environment variables
//! with type conversion, validation, and error handling.

use crate::builder::{BuilderResult, ConfigValue};
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use thiserror::Error;

/// Environment loading errors
#[derive(Error, Debug)]
pub enum EnvError {
    #[error("Environment variable not found: {var}")]
    NotFound { var: String },

    #[error("Invalid value for {var}: {reason}")]
    InvalidValue { var: String, reason: String },

    #[error("Type conversion error for {var}: {reason}")]
    ConversionError { var: String, reason: String },
}

/// Environment configuration loader
pub struct EnvConfigLoader {
    prefix: Option<String>,
    required: Vec<String>,
    defaults: HashMap<String, String>,
}

impl EnvConfigLoader {
    /// Create a new environment config loader
    pub fn new() -> Self {
        Self {
            prefix: None,
            required: Vec::new(),
            defaults: HashMap::new(),
        }
    }

    /// Set prefix for environment variables (e.g., "RIPTIDE_")
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Mark a variable as required
    pub fn require(mut self, var: impl Into<String>) -> Self {
        self.required.push(var.into());
        self
    }

    /// Set default value for a variable
    pub fn default(mut self, var: impl Into<String>, value: impl Into<String>) -> Self {
        self.defaults.insert(var.into(), value.into());
        self
    }

    /// Get environment variable with prefix
    pub fn get(&self, var: &str) -> Result<String, EnvError> {
        let full_var = self.make_var_name(var);

        env::var(&full_var).or_else(|_| {
            // Check default
            self.defaults
                .get(var)
                .cloned()
                .ok_or_else(|| EnvError::NotFound {
                    var: full_var.clone(),
                })
        })
    }

    /// Get optional environment variable
    pub fn get_optional(&self, var: &str) -> Option<String> {
        let full_var = self.make_var_name(var);
        env::var(&full_var)
            .ok()
            .or_else(|| self.defaults.get(var).cloned())
    }

    /// Get environment variable as integer
    pub fn get_int(&self, var: &str) -> Result<i64, EnvError> {
        let value = self.get(var)?;
        value.parse().map_err(|e| EnvError::ConversionError {
            var: self.make_var_name(var),
            reason: format!("Cannot parse as integer: {}", e),
        })
    }

    /// Get environment variable as unsigned integer
    pub fn get_uint(&self, var: &str) -> Result<u64, EnvError> {
        let value = self.get(var)?;
        value.parse().map_err(|e| EnvError::ConversionError {
            var: self.make_var_name(var),
            reason: format!("Cannot parse as unsigned integer: {}", e),
        })
    }

    /// Get environment variable as float
    pub fn get_float(&self, var: &str) -> Result<f64, EnvError> {
        let value = self.get(var)?;
        value.parse().map_err(|e| EnvError::ConversionError {
            var: self.make_var_name(var),
            reason: format!("Cannot parse as float: {}", e),
        })
    }

    /// Get environment variable as boolean
    pub fn get_bool(&self, var: &str) -> Result<bool, EnvError> {
        let value = self.get(var)?;
        match value.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Ok(true),
            "false" | "0" | "no" | "off" => Ok(false),
            _ => Err(EnvError::InvalidValue {
                var: self.make_var_name(var),
                reason: format!("Invalid boolean value: {}", value),
            }),
        }
    }

    /// Get environment variable as duration (supports "30s", "5m", "1h" format)
    pub fn get_duration(&self, var: &str) -> Result<Duration, EnvError> {
        let value = self.get(var)?;
        parse_duration(&value).map_err(|e| EnvError::ConversionError {
            var: self.make_var_name(var),
            reason: format!("Cannot parse as duration: {}", e),
        })
    }

    /// Get environment variable as list (comma-separated)
    pub fn get_list(&self, var: &str) -> Result<Vec<String>, EnvError> {
        let value = self.get(var)?;
        Ok(value.split(',').map(|s| s.trim().to_string()).collect())
    }

    /// Load all environment variables with prefix into a map
    pub fn load_all(&self) -> HashMap<String, String> {
        let mut result = HashMap::new();

        // Add defaults first
        for (key, value) in &self.defaults {
            result.insert(key.clone(), value.clone());
        }

        // Load from environment
        for (key, value) in env::vars() {
            if let Some(ref prefix) = self.prefix {
                if let Some(stripped) = key.strip_prefix(prefix) {
                    result.insert(stripped.to_lowercase(), value);
                }
            }
        }

        result
    }

    /// Validate required variables are present
    pub fn validate(&self) -> Result<(), EnvError> {
        for var in &self.required {
            self.get(var)?;
        }
        Ok(())
    }

    /// Make full variable name with prefix
    fn make_var_name(&self, var: &str) -> String {
        if let Some(ref prefix) = self.prefix {
            format!("{}{}", prefix, var.to_uppercase())
        } else {
            var.to_uppercase()
        }
    }

    /// Convert to ConfigValue map
    pub fn to_config_values(&self) -> HashMap<String, ConfigValue> {
        let all_vars = self.load_all();
        all_vars
            .into_iter()
            .map(|(k, v)| (k, ConfigValue::String(v)))
            .collect()
    }
}

impl Default for EnvConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse duration string (supports "30s", "5m", "1h", "500ms")
fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim().to_lowercase();

    if s.ends_with("ms") {
        let num = s[..s.len() - 2]
            .parse::<u64>()
            .map_err(|e| format!("Invalid milliseconds: {}", e))?;
        Ok(Duration::from_millis(num))
    } else if s.ends_with('s') {
        let num = s[..s.len() - 1]
            .parse::<u64>()
            .map_err(|e| format!("Invalid seconds: {}", e))?;
        Ok(Duration::from_secs(num))
    } else if s.ends_with('m') {
        let num = s[..s.len() - 1]
            .parse::<u64>()
            .map_err(|e| format!("Invalid minutes: {}", e))?;
        Ok(Duration::from_secs(num * 60))
    } else if s.ends_with('h') {
        let num = s[..s.len() - 1]
            .parse::<u64>()
            .map_err(|e| format!("Invalid hours: {}", e))?;
        Ok(Duration::from_secs(num * 3600))
    } else if let Ok(num) = s.parse::<u64>() {
        // Default to seconds
        Ok(Duration::from_secs(num))
    } else {
        Err(format!("Invalid duration format: {}", s))
    }
}

/// Convenience function to load configuration from environment
pub fn load_from_env<T, F>(prefix: &str, builder_fn: F) -> BuilderResult<T>
where
    F: FnOnce(HashMap<String, ConfigValue>) -> BuilderResult<T>,
{
    let loader = EnvConfigLoader::new().with_prefix(prefix);
    let config_values = loader.to_config_values();
    builder_fn(config_values)
}

/// Load specific environment variables into a builder
pub fn load_vars_into_builder<T>(builder: &mut T, vars: &[(&str, &str)]) -> Result<(), EnvError>
where
    T: crate::builder::ConfigBuilder<T>,
{
    for (field, env_var) in vars {
        builder.load_from_env_var(field, env_var);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_env_loader_basic() {
        env::set_var("TEST_VALUE", "123");
        env::set_var("TEST_BOOL", "true");
        env::set_var("TEST_DURATION", "30s");

        let loader = EnvConfigLoader::new().with_prefix("TEST_");

        assert_eq!(loader.get("VALUE").unwrap(), "123");
        assert_eq!(loader.get_int("VALUE").unwrap(), 123);
        assert!(loader.get_bool("BOOL").unwrap());
        assert_eq!(
            loader.get_duration("DURATION").unwrap(),
            Duration::from_secs(30)
        );

        env::remove_var("TEST_VALUE");
        env::remove_var("TEST_BOOL");
        env::remove_var("TEST_DURATION");
    }

    #[test]
    fn test_env_loader_defaults() {
        let loader = EnvConfigLoader::new()
            .with_prefix("MISSING_")
            .default("timeout", "60")
            .default("enabled", "true");

        assert_eq!(loader.get("timeout").unwrap(), "60");
        assert_eq!(loader.get_int("timeout").unwrap(), 60);
        assert!(loader.get_bool("enabled").unwrap());
    }

    #[test]
    fn test_env_loader_optional() {
        env::set_var("OPT_PRESENT", "value");

        let loader = EnvConfigLoader::new().with_prefix("OPT_");

        assert_eq!(loader.get_optional("PRESENT"), Some("value".to_string()));
        assert_eq!(loader.get_optional("MISSING"), None);

        env::remove_var("OPT_PRESENT");
    }

    #[test]
    fn test_env_loader_list() {
        env::set_var("LIST_ITEMS", "a,b,c,d");

        let loader = EnvConfigLoader::new().with_prefix("LIST_");
        let items = loader.get_list("ITEMS").unwrap();

        assert_eq!(items, vec!["a", "b", "c", "d"]);

        env::remove_var("LIST_ITEMS");
    }

    #[test]
    fn test_duration_parsing() {
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration("500ms").unwrap(), Duration::from_millis(500));
        assert_eq!(parse_duration("60").unwrap(), Duration::from_secs(60));
    }

    #[test]
    fn test_env_loader_validation() {
        let loader = EnvConfigLoader::new()
            .with_prefix("VALID_")
            .require("REQUIRED");

        assert!(loader.validate().is_err());

        env::set_var("VALID_REQUIRED", "present");
        assert!(loader.validate().is_ok());

        env::remove_var("VALID_REQUIRED");
    }
}
