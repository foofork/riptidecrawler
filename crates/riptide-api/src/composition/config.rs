//! Configuration module for Dependency Injection
//!
//! This module provides configuration structures for ApplicationContext,
//! supporting both TOML files and environment variable overrides.
//!
//! # Example - From Environment
//!
//! ```rust,ignore
//! use riptide_api::composition::DiConfig;
//!
//! let config = DiConfig::from_env()?;
//! assert!(config.database.max_connections > 0);
//! ```
//!
//! # Example - From TOML
//!
//! ```toml
//! [database]
//! url = "postgresql://localhost:5432/riptide"
//! max_connections = 20
//! timeout_secs = 30
//!
//! [redis]
//! url = "redis://localhost:6379"
//! pool_size = 10
//! default_ttl_secs = 3600
//!
//! [features]
//! enable_browser = true
//! enable_pdf = true
//! enable_search = false
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Main DI configuration
///
/// This configuration is used by ApplicationContext to wire dependencies.
/// It can be loaded from TOML files or environment variables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiConfig {
    /// Database configuration
    pub database: DatabaseConfig,

    /// Redis configuration
    pub redis: RedisConfig,

    /// Feature flags
    pub features: FeatureFlags,

    /// Testing mode flag (internal use)
    #[serde(default)]
    pub is_testing: bool,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// PostgreSQL connection URL
    ///
    /// Format: `postgresql://[user[:password]@][host][:port][/dbname][?param1=value1&...]`
    ///
    /// Example: `postgresql://postgres:password@localhost:5432/riptide`
    pub url: String,

    /// Maximum number of connections in pool
    ///
    /// Default: 20
    /// Range: 1-100
    #[serde(default = "DatabaseConfig::default_max_connections")]
    pub max_connections: u32,

    /// Connection acquisition timeout in seconds
    ///
    /// Default: 30 seconds
    #[serde(default = "DatabaseConfig::default_timeout_secs")]
    pub timeout_secs: u64,
}

impl DatabaseConfig {
    fn default_max_connections() -> u32 {
        20
    }

    fn default_timeout_secs() -> u64 {
        30
    }

    /// Get timeout as Duration
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://localhost:5432/riptide".to_string(),
            max_connections: Self::default_max_connections(),
            timeout_secs: Self::default_timeout_secs(),
        }
    }
}

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis connection URL
    ///
    /// Format: `redis://[user[:password]@][host][:port][/dbnum]`
    ///
    /// Example: `redis://localhost:6379/0`
    pub url: String,

    /// Connection pool size
    ///
    /// Default: 10
    /// Range: 1-50
    #[serde(default = "RedisConfig::default_pool_size")]
    pub pool_size: u32,

    /// Default TTL for cached values in seconds
    ///
    /// Default: 3600 (1 hour)
    #[serde(default = "RedisConfig::default_ttl_secs")]
    pub default_ttl_secs: u64,
}

impl RedisConfig {
    fn default_pool_size() -> u32 {
        10
    }

    fn default_ttl_secs() -> u64 {
        3600
    }

    /// Get default TTL as Duration
    pub fn default_ttl(&self) -> Duration {
        Duration::from_secs(self.default_ttl_secs)
    }
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379/0".to_string(),
            pool_size: Self::default_pool_size(),
            default_ttl_secs: Self::default_ttl_secs(),
        }
    }
}

/// Feature flags for optional functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Enable browser automation features
    #[serde(default)]
    pub enable_browser: bool,

    /// Enable PDF processing features
    #[serde(default)]
    pub enable_pdf: bool,

    /// Enable search provider integration
    #[serde(default)]
    pub enable_search: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            enable_browser: false,
            enable_pdf: false,
            enable_search: false,
        }
    }
}

impl DiConfig {
    /// Create default configuration for testing
    pub fn for_testing() -> Self {
        Self {
            database: DatabaseConfig::default(),
            redis: RedisConfig::default(),
            features: FeatureFlags::default(),
            is_testing: true,
        }
    }

    /// Load configuration from environment variables
    ///
    /// Environment variables:
    /// - `DATABASE_URL` - PostgreSQL connection URL
    /// - `DATABASE_MAX_CONNECTIONS` - Max pool size
    /// - `DATABASE_TIMEOUT_SECS` - Connection timeout
    /// - `REDIS_URL` - Redis connection URL
    /// - `REDIS_POOL_SIZE` - Redis pool size
    /// - `REDIS_DEFAULT_TTL_SECS` - Default TTL
    /// - `FEATURE_BROWSER` - Enable browser (true/false)
    /// - `FEATURE_PDF` - Enable PDF (true/false)
    /// - `FEATURE_SEARCH` - Enable search (true/false)
    ///
    /// # Example
    ///
    /// ```bash
    /// export DATABASE_URL="postgresql://localhost:5432/riptide"
    /// export REDIS_URL="redis://localhost:6379"
    /// export FEATURE_BROWSER="true"
    /// ```
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();

        // Database configuration
        if let Ok(url) = std::env::var("DATABASE_URL") {
            config.database.url = url;
        }
        if let Ok(val) = std::env::var("DATABASE_MAX_CONNECTIONS") {
            config.database.max_connections =
                val.parse().context("Invalid DATABASE_MAX_CONNECTIONS")?;
        }
        if let Ok(val) = std::env::var("DATABASE_TIMEOUT_SECS") {
            config.database.timeout_secs = val.parse().context("Invalid DATABASE_TIMEOUT_SECS")?;
        }

        // Redis configuration
        if let Ok(url) = std::env::var("REDIS_URL") {
            config.redis.url = url;
        }
        if let Ok(val) = std::env::var("REDIS_POOL_SIZE") {
            config.redis.pool_size = val.parse().context("Invalid REDIS_POOL_SIZE")?;
        }
        if let Ok(val) = std::env::var("REDIS_DEFAULT_TTL_SECS") {
            config.redis.default_ttl_secs =
                val.parse().context("Invalid REDIS_DEFAULT_TTL_SECS")?;
        }

        // Feature flags
        if let Ok(val) = std::env::var("FEATURE_BROWSER") {
            config.features.enable_browser = val.to_lowercase() == "true";
        }
        if let Ok(val) = std::env::var("FEATURE_PDF") {
            config.features.enable_pdf = val.to_lowercase() == "true";
        }
        if let Ok(val) = std::env::var("FEATURE_SEARCH") {
            config.features.enable_search = val.to_lowercase() == "true";
        }

        Ok(config)
    }

    /// Load configuration from TOML file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to TOML configuration file
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let config = DiConfig::from_toml_file("config.toml")?;
    /// ```
    pub fn from_toml_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let content =
            std::fs::read_to_string(path.as_ref()).context("Failed to read config file")?;
        Self::from_toml_str(&content)
    }

    /// Load configuration from TOML string
    ///
    /// # Arguments
    ///
    /// * `toml_str` - TOML configuration as string
    pub fn from_toml_str(toml_str: &str) -> Result<Self> {
        let config: DiConfig =
            toml::from_str(toml_str).context("Failed to parse TOML configuration")?;
        Ok(config)
    }

    /// Validate configuration
    ///
    /// Checks:
    /// - Database URL is valid
    /// - Connection pool sizes are reasonable
    /// - Timeout values are positive
    ///
    /// # Errors
    ///
    /// Returns error if validation fails
    pub fn validate(&self) -> Result<()> {
        // Validate database URL
        if self.database.url.is_empty() {
            anyhow::bail!("Database URL cannot be empty");
        }
        if !self.database.url.starts_with("postgresql://")
            && !self.database.url.starts_with("postgres://")
        {
            anyhow::bail!("Database URL must start with postgresql:// or postgres://");
        }

        // Validate pool sizes
        if self.database.max_connections == 0 {
            anyhow::bail!("Database max_connections must be greater than 0");
        }
        if self.database.max_connections > 100 {
            anyhow::bail!("Database max_connections should not exceed 100");
        }

        if self.redis.pool_size == 0 {
            anyhow::bail!("Redis pool_size must be greater than 0");
        }
        if self.redis.pool_size > 50 {
            anyhow::bail!("Redis pool_size should not exceed 50");
        }

        // Validate timeouts
        if self.database.timeout_secs == 0 {
            anyhow::bail!("Database timeout_secs must be greater than 0");
        }

        // Validate Redis URL
        if self.redis.url.is_empty() {
            anyhow::bail!("Redis URL cannot be empty");
        }
        if !self.redis.url.starts_with("redis://") {
            anyhow::bail!("Redis URL must start with redis://");
        }

        Ok(())
    }
}

impl Default for DiConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            redis: RedisConfig::default(),
            features: FeatureFlags::default(),
            is_testing: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DiConfig::default();

        assert_eq!(config.database.max_connections, 20);
        assert_eq!(config.database.timeout_secs, 30);
        assert_eq!(config.redis.pool_size, 10);
        assert_eq!(config.redis.default_ttl_secs, 3600);
        assert!(!config.features.enable_browser);
        assert!(!config.is_testing);
    }

    #[test]
    fn test_for_testing() {
        let config = DiConfig::for_testing();

        assert!(config.is_testing);
        assert!(!config.features.enable_browser);
    }

    #[test]
    fn test_validation_valid_config() {
        let config = DiConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validation_invalid_database_url() {
        let mut config = DiConfig::default();
        config.database.url = String::new();

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_pool_size() {
        let mut config = DiConfig::default();
        config.database.max_connections = 0;

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_pool_size_too_large() {
        let mut config = DiConfig::default();
        config.database.max_connections = 200;

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_from_toml_str() {
        let toml = r#"
            [database]
            url = "postgresql://localhost:5432/test"
            max_connections = 15
            timeout_secs = 20

            [redis]
            url = "redis://localhost:6380"
            pool_size = 5
            default_ttl_secs = 1800

            [features]
            enable_browser = true
            enable_pdf = false
            enable_search = true
        "#;

        let config = DiConfig::from_toml_str(toml).unwrap();

        assert_eq!(config.database.url, "postgresql://localhost:5432/test");
        assert_eq!(config.database.max_connections, 15);
        assert_eq!(config.database.timeout_secs, 20);
        assert_eq!(config.redis.url, "redis://localhost:6380");
        assert_eq!(config.redis.pool_size, 5);
        assert_eq!(config.redis.default_ttl_secs, 1800);
        assert!(config.features.enable_browser);
        assert!(!config.features.enable_pdf);
        assert!(config.features.enable_search);
    }

    #[test]
    fn test_database_timeout_duration() {
        let config = DiConfig::default();
        assert_eq!(config.database.timeout(), Duration::from_secs(30));
    }

    #[test]
    fn test_redis_default_ttl_duration() {
        let config = DiConfig::default();
        assert_eq!(config.redis.default_ttl(), Duration::from_secs(3600));
    }
}
