//! Cache configuration and backend selection
//!
//! This module provides configuration for the cache layer, enabling:
//! - Backend selection (Redis or In-Memory)
//! - Connection settings and performance tuning
//! - TTL and capacity management
//!
//! # Design Goals
//!
//! - **Flexibility**: Easy switching between cache backends
//! - **Safety**: Type-safe configuration with validation
//! - **Testability**: Support both production (Redis) and test (in-memory) scenarios
//!
//! # Example
//!
//! ```rust
//! use riptide_cache::storage_config::{StorageConfig, CacheBackend};
//! use std::time::Duration;
//!
//! // Production configuration with Redis
//! let redis_config = StorageConfig {
//!     backend: CacheBackend::Redis,
//!     redis_url: Some("redis://localhost:6379".to_string()),
//!     default_ttl: Duration::from_secs(3600),
//!     max_connections: 10,
//! };
//!
//! // Development/test configuration with in-memory cache
//! let memory_config = StorageConfig {
//!     backend: CacheBackend::Memory,
//!     redis_url: None,
//!     default_ttl: Duration::from_secs(300),
//!     max_connections: 1,
//! };
//! ```

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Cache backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CacheBackend {
    /// In-memory cache for development and testing
    #[default]
    Memory,
    /// Redis-based distributed cache for production
    Redis,
}

impl std::fmt::Display for CacheBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Memory => write!(f, "memory"),
            Self::Redis => write!(f, "redis"),
        }
    }
}

/// Cache configuration
///
/// Provides type-safe configuration for cache backend selection
/// and connection settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Backend type (memory or redis)
    #[serde(default)]
    pub backend: CacheBackend,

    /// Redis connection URL (required when backend = redis)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_url: Option<String>,

    /// Default TTL for cache entries (in seconds)
    #[serde(default = "default_ttl_secs", skip_serializing_if = "is_default_ttl")]
    pub default_ttl_secs: u64,

    /// Maximum number of Redis connections in pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Enable automatic fallback to in-memory cache if Redis fails
    #[serde(default = "default_enable_fallback")]
    pub enable_fallback: bool,

    /// Connection timeout for Redis (in seconds)
    #[serde(
        default = "default_connection_timeout_secs",
        skip_serializing_if = "is_default_connection_timeout"
    )]
    pub connection_timeout_secs: u64,
}

// Default value functions for serde
fn default_ttl_secs() -> u64 {
    3600 // 1 hour
}

fn is_default_ttl(ttl: &u64) -> bool {
    *ttl == default_ttl_secs()
}

fn default_max_connections() -> u32 {
    10
}

fn default_enable_fallback() -> bool {
    false
}

fn default_connection_timeout_secs() -> u64 {
    5
}

fn is_default_connection_timeout(timeout: &u64) -> bool {
    *timeout == default_connection_timeout_secs()
}

impl Default for StorageConfig {
    /// Create default configuration with in-memory backend
    ///
    /// Suitable for development and testing without Redis dependency.
    fn default() -> Self {
        Self {
            backend: CacheBackend::Memory,
            redis_url: None,
            default_ttl_secs: default_ttl_secs(),
            max_connections: default_max_connections(),
            enable_fallback: default_enable_fallback(),
            connection_timeout_secs: default_connection_timeout_secs(),
        }
    }
}

impl StorageConfig {
    /// Create a new Redis-backed cache configuration
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection URL (e.g., "redis://localhost:6379")
    ///
    /// # Example
    ///
    /// ```rust
    /// use riptide_cache::storage_config::StorageConfig;
    ///
    /// let config = StorageConfig::redis("redis://localhost:6379");
    /// ```
    pub fn redis(redis_url: impl Into<String>) -> Self {
        Self {
            backend: CacheBackend::Redis,
            redis_url: Some(redis_url.into()),
            ..Default::default()
        }
    }

    /// Create a new Redis-backed cache configuration with automatic fallback
    ///
    /// If Redis connection fails, automatically falls back to in-memory cache.
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection URL
    ///
    /// # Example
    ///
    /// ```rust
    /// use riptide_cache::storage_config::StorageConfig;
    ///
    /// let config = StorageConfig::redis_with_fallback("redis://localhost:6379");
    /// ```
    pub fn redis_with_fallback(redis_url: impl Into<String>) -> Self {
        Self {
            backend: CacheBackend::Redis,
            redis_url: Some(redis_url.into()),
            enable_fallback: true,
            ..Default::default()
        }
    }

    /// Create a new in-memory cache configuration
    ///
    /// # Example
    ///
    /// ```rust
    /// use riptide_cache::storage_config::StorageConfig;
    ///
    /// let config = StorageConfig::memory();
    /// ```
    pub fn memory() -> Self {
        Self::default()
    }

    /// Validate configuration
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Configuration is valid
    /// * `Err(msg)` - Configuration error message
    ///
    /// # Example
    ///
    /// ```rust
    /// use riptide_cache::storage_config::{StorageConfig, CacheBackend};
    ///
    /// let mut config = StorageConfig::default();
    /// config.backend = CacheBackend::Redis;
    /// // Missing redis_url
    /// assert!(config.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<(), String> {
        match self.backend {
            CacheBackend::Redis => {
                if self.redis_url.is_none() {
                    return Err("Redis URL is required when backend is set to 'redis'".to_string());
                }
                if let Some(url) = &self.redis_url {
                    if !url.starts_with("redis://") && !url.starts_with("rediss://") {
                        return Err(format!(
                            "Invalid Redis URL format: {}. Must start with redis:// or rediss://",
                            url
                        ));
                    }
                }
            }
            CacheBackend::Memory => {
                // No specific validation needed for in-memory backend
            }
        }

        if self.max_connections == 0 {
            return Err("max_connections must be greater than 0".to_string());
        }

        if self.default_ttl_secs == 0 {
            return Err("default_ttl must be greater than 0".to_string());
        }

        Ok(())
    }

    /// Set TTL for cache entries
    ///
    /// # Example
    ///
    /// ```rust
    /// use riptide_cache::storage_config::StorageConfig;
    ///
    /// let config = StorageConfig::memory()
    ///     .with_ttl_secs(300);
    /// ```
    pub fn with_ttl_secs(mut self, ttl_secs: u64) -> Self {
        self.default_ttl_secs = ttl_secs;
        self
    }

    /// Get default TTL as Duration
    pub fn default_ttl(&self) -> Duration {
        Duration::from_secs(self.default_ttl_secs)
    }

    /// Set maximum connection pool size
    ///
    /// # Example
    ///
    /// ```rust
    /// use riptide_cache::storage_config::StorageConfig;
    ///
    /// let config = StorageConfig::redis("redis://localhost:6379")
    ///     .with_max_connections(20);
    /// ```
    pub fn with_max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// Set connection timeout
    ///
    /// # Example
    ///
    /// ```rust
    /// use riptide_cache::storage_config::StorageConfig;
    ///
    /// let config = StorageConfig::redis("redis://localhost:6379")
    ///     .with_connection_timeout_secs(10);
    /// ```
    pub fn with_connection_timeout_secs(mut self, timeout_secs: u64) -> Self {
        self.connection_timeout_secs = timeout_secs;
        self
    }

    /// Get connection timeout as Duration
    pub fn connection_timeout(&self) -> Duration {
        Duration::from_secs(self.connection_timeout_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = StorageConfig::default();
        assert_eq!(config.backend, CacheBackend::Memory);
        assert!(config.redis_url.is_none());
        assert_eq!(config.default_ttl_secs, 3600);
        assert_eq!(config.max_connections, 10);
        assert!(!config.enable_fallback);
    }

    #[test]
    fn test_redis_config() {
        let config = StorageConfig::redis("redis://localhost:6379");
        assert_eq!(config.backend, CacheBackend::Redis);
        assert_eq!(config.redis_url, Some("redis://localhost:6379".to_string()));
        assert!(!config.enable_fallback);
    }

    #[test]
    fn test_redis_with_fallback() {
        let config = StorageConfig::redis_with_fallback("redis://localhost:6379");
        assert_eq!(config.backend, CacheBackend::Redis);
        assert!(config.enable_fallback);
    }

    #[test]
    fn test_memory_config() {
        let config = StorageConfig::memory();
        assert_eq!(config.backend, CacheBackend::Memory);
        assert!(config.redis_url.is_none());
    }

    #[test]
    fn test_validate_redis_missing_url() {
        let config = StorageConfig {
            backend: CacheBackend::Redis,
            redis_url: None,
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Redis URL is required"));
    }

    #[test]
    fn test_validate_invalid_redis_url() {
        let config = StorageConfig {
            backend: CacheBackend::Redis,
            redis_url: Some("http://localhost:6379".to_string()),
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid Redis URL format"));
    }

    #[test]
    fn test_validate_valid_redis() {
        let config = StorageConfig::redis("redis://localhost:6379");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_valid_rediss() {
        let config = StorageConfig::redis("rediss://secure.redis.example.com:6380");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_memory() {
        let config = StorageConfig::memory();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_zero_max_connections() {
        let config = StorageConfig {
            max_connections: 0,
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("max_connections"));
    }

    #[test]
    fn test_validate_zero_ttl() {
        let config = StorageConfig {
            default_ttl_secs: 0,
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("default_ttl"));
    }

    #[test]
    fn test_builder_methods() {
        let config = StorageConfig::redis("redis://localhost:6379")
            .with_ttl_secs(600)
            .with_max_connections(20)
            .with_connection_timeout_secs(10);

        assert_eq!(config.default_ttl_secs, 600);
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.connection_timeout_secs, 10);
    }

    #[test]
    fn test_backend_display() {
        assert_eq!(CacheBackend::Memory.to_string(), "memory");
        assert_eq!(CacheBackend::Redis.to_string(), "redis");
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = StorageConfig::redis("redis://localhost:6379").with_ttl_secs(300);

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: StorageConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.backend, deserialized.backend);
        assert_eq!(config.redis_url, deserialized.redis_url);
        assert_eq!(config.default_ttl_secs, deserialized.default_ttl_secs);
    }
}
