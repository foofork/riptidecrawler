//! Cache factory for backend selection and instantiation
//!
//! This module provides a factory for creating cache storage instances
//! based on configuration, with support for:
//! - Backend selection (Redis or In-Memory)
//! - Automatic fallback on connection failures
//! - Connection pooling and error handling
//!
//! # Design Goals
//!
//! - **Flexibility**: Easy switching between cache backends
//! - **Resilience**: Graceful fallback when Redis is unavailable
//! - **Type Safety**: Returns trait objects for polymorphic usage
//!
//! # Example
//!
//! ```rust,no_run
//! use riptide_cache::factory::CacheFactory;
//! use riptide_cache::storage_config::StorageConfig;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Production: Try Redis, fallback to memory
//!     let config = StorageConfig::redis_with_fallback("redis://localhost:6379");
//!     let cache = CacheFactory::create_with_fallback(&config).await;
//!
//!     // Use cache through trait interface
//!     cache.set("key", b"value", None).await?;
//!
//!     Ok(())
//! }
//! ```

use crate::redis_storage::RedisStorage;
use crate::storage_config::{CacheBackend, StorageConfig};
use anyhow::{Context, Result};
use riptide_types::ports::cache::CacheStorage;
use riptide_types::ports::memory_cache::InMemoryCache;
use std::sync::Arc;
use tracing::{error, info, warn};

/// Cache factory for creating storage backends
pub struct CacheFactory;

impl CacheFactory {
    /// Create cache storage from configuration
    ///
    /// This method creates the appropriate cache backend based on the
    /// configuration. It will fail if Redis is configured but unavailable.
    ///
    /// # Arguments
    ///
    /// * `config` - Cache configuration
    ///
    /// # Returns
    ///
    /// * `Ok(cache)` - Successfully created cache
    /// * `Err(e)` - Configuration error or connection failure
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use riptide_cache::factory::CacheFactory;
    /// use riptide_cache::storage_config::StorageConfig;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let config = StorageConfig::redis("redis://localhost:6379");
    ///     let cache = CacheFactory::create(&config).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn create(config: &StorageConfig) -> Result<Arc<dyn CacheStorage>> {
        // Validate configuration first
        config
            .validate()
            .map_err(|e| anyhow::anyhow!("Invalid cache configuration: {}", e))?;

        match config.backend {
            CacheBackend::Memory => {
                info!("Creating in-memory cache backend");
                let cache = InMemoryCache::new();
                Ok(Arc::new(cache) as Arc<dyn CacheStorage>)
            }
            CacheBackend::Redis => {
                let url = config
                    .redis_url
                    .as_ref()
                    .context("Redis URL required when backend = 'redis'")?;

                info!(
                    url = %url,
                    max_connections = config.max_connections,
                    "Creating Redis cache backend"
                );

                // Attempt to connect to Redis with timeout
                let result =
                    tokio::time::timeout(config.connection_timeout(), RedisStorage::new(url)).await;

                match result {
                    Ok(Ok(storage)) => {
                        info!("Redis cache backend created successfully");
                        Ok(Arc::new(storage) as Arc<dyn CacheStorage>)
                    }
                    Ok(Err(e)) => {
                        error!(error = %e, "Failed to create Redis cache backend");
                        Err(e.context("Failed to connect to Redis"))
                    }
                    Err(_) => {
                        error!(
                            timeout_secs = config.connection_timeout_secs,
                            "Redis connection timed out"
                        );
                        Err(anyhow::anyhow!(
                            "Redis connection timed out after {} seconds",
                            config.connection_timeout_secs
                        ))
                    }
                }
            }
        }
    }

    /// Create cache storage with automatic fallback
    ///
    /// This method attempts to create the configured cache backend.
    /// If Redis connection fails and fallback is enabled, it automatically
    /// falls back to an in-memory cache with a warning.
    ///
    /// # Arguments
    ///
    /// * `config` - Cache configuration
    ///
    /// # Returns
    ///
    /// Always returns a valid cache instance (never fails)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use riptide_cache::factory::CacheFactory;
    /// use riptide_cache::storage_config::StorageConfig;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     // This will never fail - falls back to memory if Redis unavailable
    ///     let config = StorageConfig::redis_with_fallback("redis://localhost:6379");
    ///     let cache = CacheFactory::create_with_fallback(&config).await;
    ///     Ok(())
    /// }
    /// ```
    pub async fn create_with_fallback(config: &StorageConfig) -> Arc<dyn CacheStorage> {
        // Try to create the requested backend
        match Self::create(config).await {
            Ok(cache) => cache,
            Err(e) => {
                if config.enable_fallback && config.backend == CacheBackend::Redis {
                    warn!(
                        error = %e,
                        "Redis cache unavailable, falling back to in-memory cache"
                    );
                    info!("Creating fallback in-memory cache backend");
                    Arc::new(InMemoryCache::new()) as Arc<dyn CacheStorage>
                } else {
                    // If fallback not enabled or backend is already memory, panic
                    error!(error = %e, "Failed to create cache and fallback is disabled");
                    panic!("Failed to create cache backend: {}", e);
                }
            }
        }
    }

    /// Create in-memory cache (convenience method)
    ///
    /// # Example
    ///
    /// ```rust
    /// use riptide_cache::factory::CacheFactory;
    ///
    /// let cache = CacheFactory::memory();
    /// ```
    pub fn memory() -> Arc<dyn CacheStorage> {
        info!("Creating in-memory cache backend");
        Arc::new(InMemoryCache::new())
    }

    /// Attempt to create Redis cache, return None if unavailable
    ///
    /// This is useful for optional Redis caching where you want to
    /// handle the absence of Redis differently.
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection URL
    /// * `timeout` - Connection timeout
    ///
    /// # Returns
    ///
    /// * `Some(cache)` - Redis cache created successfully
    /// * `None` - Redis unavailable or connection failed
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use riptide_cache::factory::CacheFactory;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let timeout = Duration::from_secs(5);
    ///     match CacheFactory::try_redis("redis://localhost:6379", timeout).await {
    ///         Some(cache) => println!("Using Redis cache"),
    ///         None => println!("Redis unavailable, proceeding without cache"),
    ///     }
    /// }
    /// ```
    pub async fn try_redis(
        redis_url: &str,
        timeout: std::time::Duration,
    ) -> Option<Arc<dyn CacheStorage>> {
        info!(url = %redis_url, "Attempting to create Redis cache");

        let result = tokio::time::timeout(timeout, RedisStorage::new(redis_url)).await;

        match result {
            Ok(Ok(storage)) => {
                info!("Redis cache created successfully");
                Some(Arc::new(storage) as Arc<dyn CacheStorage>)
            }
            Ok(Err(e)) => {
                warn!(error = %e, "Failed to create Redis cache");
                None
            }
            Err(_) => {
                warn!(
                    timeout_secs = timeout.as_secs(),
                    "Redis connection timed out"
                );
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_create_memory_backend() {
        let config = StorageConfig::memory();
        let cache = CacheFactory::create(&config).await.unwrap();

        // Verify it works
        cache.set("test", b"value", None).await.unwrap();
        let result = cache.get("test").await.unwrap();
        assert_eq!(result, Some(b"value".to_vec()));
    }

    #[tokio::test]
    async fn test_create_memory_convenience() {
        let cache = CacheFactory::memory();

        // Verify it works
        cache.set("test", b"value", None).await.unwrap();
        let result = cache.get("test").await.unwrap();
        assert_eq!(result, Some(b"value".to_vec()));
    }

    #[tokio::test]
    async fn test_create_redis_missing_url() {
        let config = StorageConfig {
            backend: CacheBackend::Redis,
            redis_url: None,
            ..Default::default()
        };

        let result = CacheFactory::create(&config).await;
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("Redis URL"));
    }

    #[tokio::test]
    async fn test_create_redis_invalid_url() {
        let config = StorageConfig::redis("invalid://url");

        let result = CacheFactory::create(&config).await;
        // Should fail validation or connection
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore] // Requires running Redis instance
    async fn test_create_redis_success() {
        let config = StorageConfig::redis("redis://localhost:6379");

        let cache = CacheFactory::create(&config).await.unwrap();

        // Verify it works
        cache.set("test:key", b"value", None).await.unwrap();
        let result = cache.get("test:key").await.unwrap();
        assert_eq!(result, Some(b"value".to_vec()));

        // Cleanup
        cache.delete("test:key").await.unwrap();
    }

    #[tokio::test]
    async fn test_create_with_fallback_memory() {
        let config = StorageConfig::memory();
        let cache = CacheFactory::create_with_fallback(&config).await;

        // Should work
        cache.set("test", b"value", None).await.unwrap();
        let result = cache.get("test").await.unwrap();
        assert_eq!(result, Some(b"value".to_vec()));
    }

    #[tokio::test]
    async fn test_create_with_fallback_redis_unavailable() {
        // Point to non-existent Redis instance
        let config = StorageConfig::redis_with_fallback("redis://localhost:9999")
            .with_connection_timeout_secs(1); // 1 second timeout

        // Should fallback to memory without panic
        let cache = CacheFactory::create_with_fallback(&config).await;

        // Should work with in-memory backend
        cache.set("test", b"value", None).await.unwrap();
        let result = cache.get("test").await.unwrap();
        assert_eq!(result, Some(b"value".to_vec()));
    }

    #[tokio::test]
    #[ignore] // Requires running Redis instance
    async fn test_create_with_fallback_redis_success() {
        let config = StorageConfig::redis_with_fallback("redis://localhost:6379");

        let cache = CacheFactory::create_with_fallback(&config).await;

        // Verify it works with Redis
        cache.set("test:fallback", b"value", None).await.unwrap();
        let result = cache.get("test:fallback").await.unwrap();
        assert_eq!(result, Some(b"value".to_vec()));

        // Cleanup
        cache.delete("test:fallback").await.unwrap();
    }

    #[tokio::test]
    async fn test_try_redis_unavailable() {
        let result =
            CacheFactory::try_redis("redis://localhost:9999", Duration::from_millis(100)).await;

        assert!(result.is_none());
    }

    #[tokio::test]
    #[ignore] // Requires running Redis instance
    async fn test_try_redis_success() {
        let cache = CacheFactory::try_redis("redis://localhost:6379", Duration::from_secs(5)).await;

        assert!(cache.is_some());

        if let Some(cache) = cache {
            cache.set("test:try", b"value", None).await.unwrap();
            let result = cache.get("test:try").await.unwrap();
            assert_eq!(result, Some(b"value".to_vec()));
            cache.delete("test:try").await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_validate_config_before_create() {
        let config = StorageConfig {
            max_connections: 0, // Invalid
            ..Default::default()
        };

        let result = CacheFactory::create(&config).await;
        assert!(result.is_err());
        assert!(result
            .err()
            .unwrap()
            .to_string()
            .contains("Invalid cache configuration"));
    }
}
