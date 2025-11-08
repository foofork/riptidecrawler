//! Backend-agnostic cache storage interface
//!
//! This trait enables dependency inversion for caching, allowing:
//! - Testing with in-memory implementations
//! - Swapping Redis for other backends
//! - Reducing direct Redis dependencies across the codebase
//!
//! # Design Goals
//!
//! - **Dependency Scoping**: Reduce Redis dependencies from 6 to 2 crates
//! - **Testability**: Enable easy mocking and in-memory testing
//! - **Flexibility**: Support multiple backend implementations
//! - **Performance**: Async-first design with batch operations
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_types::ports::CacheStorage;
//! use std::time::Duration;
//!
//! async fn example(cache: &dyn CacheStorage) -> anyhow::Result<()> {
//!     // Set value with TTL
//!     cache.set("key", b"value", Some(Duration::from_secs(3600))).await?;
//!
//!     // Get value
//!     if let Some(data) = cache.get("key").await? {
//!         println!("Cached data: {:?}", data);
//!     }
//!
//!     // Check existence
//!     if cache.exists("key").await? {
//!         println!("Key exists");
//!     }
//!
//!     Ok(())
//! }
//! ```

use crate::error::Result as RiptideResult;
use async_trait::async_trait;
use std::time::Duration;

/// Backend-agnostic cache storage interface
///
/// Implementations must be thread-safe (`Send + Sync`) and support
/// asynchronous operations. All byte slices should be treated as
/// opaque binary data.
#[async_trait]
pub trait CacheStorage: Send + Sync {
    /// Retrieve a value by key
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key to lookup
    ///
    /// # Returns
    ///
    /// * `Ok(Some(data))` - Value found
    /// * `Ok(None)` - Key not found or expired
    /// * `Err(_)` - Storage backend error
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let value = cache.get("user:123").await?;
    /// if let Some(data) = value {
    ///     let user: User = serde_json::from_slice(&data)?;
    /// }
    /// ```
    async fn get(&self, key: &str) -> RiptideResult<Option<Vec<u8>>>;

    /// Store a value with optional TTL
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    /// * `value` - Binary data to store
    /// * `ttl` - Optional time-to-live duration
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Value stored successfully
    /// * `Err(_)` - Storage backend error
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let data = serde_json::to_vec(&user)?;
    /// cache.set("user:123", &data, Some(Duration::from_secs(3600))).await?;
    /// ```
    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> RiptideResult<()>;

    /// Delete a key from cache
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Key deleted (or didn't exist)
    /// * `Err(_)` - Storage backend error
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// cache.delete("user:123").await?;
    /// ```
    async fn delete(&self, key: &str) -> RiptideResult<()>;

    /// Check if a key exists in cache
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key to check
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Key exists and is not expired
    /// * `Ok(false)` - Key doesn't exist or has expired
    /// * `Err(_)` - Storage backend error
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if cache.exists("user:123").await? {
    ///     println!("User is cached");
    /// }
    /// ```
    async fn exists(&self, key: &str) -> RiptideResult<bool>;

    /// Store multiple key-value pairs atomically
    ///
    /// This operation should be atomic where supported by the backend.
    /// The default implementation falls back to sequential `set` calls.
    ///
    /// # Arguments
    ///
    /// * `items` - Vector of (key, value) tuples
    /// * `ttl` - Optional TTL applied to all keys
    ///
    /// # Returns
    ///
    /// * `Ok(())` - All values stored successfully
    /// * `Err(_)` - Storage backend error (partial writes may occur)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let items = vec![
    ///     ("user:1", b"data1" as &[u8]),
    ///     ("user:2", b"data2"),
    /// ];
    /// cache.mset(items, Some(Duration::from_secs(3600))).await?;
    /// ```
    async fn mset(&self, items: Vec<(&str, &[u8])>, ttl: Option<Duration>) -> RiptideResult<()> {
        // Default implementation: sequential sets
        // Backends can override for atomic batch operations
        for (key, value) in items {
            self.set(key, value, ttl).await?;
        }
        Ok(())
    }

    /// Retrieve multiple values by keys
    ///
    /// This operation may be optimized by backends to batch network calls.
    /// The default implementation makes sequential `get` calls.
    ///
    /// # Arguments
    ///
    /// * `keys` - Slice of keys to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(values)` - Vector of optional values in same order as keys
    /// * `Err(_)` - Storage backend error
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let keys = vec!["user:1", "user:2", "user:3"];
    /// let values = cache.mget(&keys).await?;
    /// for (key, value) in keys.iter().zip(values.iter()) {
    ///     match value {
    ///         Some(data) => println!("{}: found", key),
    ///         None => println!("{}: not found", key),
    ///     }
    /// }
    /// ```
    async fn mget(&self, keys: &[&str]) -> RiptideResult<Vec<Option<Vec<u8>>>> {
        // Default implementation: sequential gets
        // Backends can override for pipelined batch operations
        let mut results = Vec::with_capacity(keys.len());
        for key in keys {
            results.push(self.get(key).await?);
        }
        Ok(results)
    }

    /// Set expiration time on existing key
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    /// * `ttl` - Time-to-live duration
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - TTL set successfully
    /// * `Ok(false)` - Key doesn't exist
    /// * `Err(_)` - Storage backend error
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// cache.expire("user:123", Duration::from_secs(600)).await?;
    /// ```
    async fn expire(&self, key: &str, ttl: Duration) -> RiptideResult<bool> {
        // Default implementation: get + set
        // Backends can override with native EXPIRE commands
        if let Some(value) = self.get(key).await? {
            self.set(key, &value, Some(ttl)).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get remaining time-to-live for a key
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    ///
    /// # Returns
    ///
    /// * `Ok(Some(duration))` - Key exists with TTL
    /// * `Ok(None)` - Key doesn't exist or has no TTL
    /// * `Err(_)` - Storage backend error
    async fn ttl(&self, key: &str) -> RiptideResult<Option<Duration>> {
        // Default implementation: check existence
        // Backends with TTL support should override this
        if self.exists(key).await? {
            // Cannot determine actual TTL without backend support
            Ok(None)
        } else {
            Ok(None)
        }
    }

    /// Increment numeric value atomically
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    /// * `delta` - Amount to increment (can be negative)
    ///
    /// # Returns
    ///
    /// * `Ok(new_value)` - Value after increment
    /// * `Err(_)` - Storage backend error or non-numeric value
    async fn incr(&self, key: &str, delta: i64) -> RiptideResult<i64> {
        // Default implementation for backends without native increment
        let current = if let Some(data) = self.get(key).await? {
            String::from_utf8(data)
                .map_err(|e| crate::error::RiptideError::Cache(format!("Invalid UTF-8: {}", e)))?
                .parse::<i64>()
                .map_err(|e| crate::error::RiptideError::Cache(format!("Not a number: {}", e)))?
        } else {
            0
        };

        let new_value = current + delta;
        self.set(key, new_value.to_string().as_bytes(), None)
            .await?;
        Ok(new_value)
    }

    /// Delete multiple keys
    ///
    /// # Arguments
    ///
    /// * `keys` - Slice of keys to delete
    ///
    /// # Returns
    ///
    /// * `Ok(count)` - Number of keys deleted
    /// * `Err(_)` - Storage backend error
    async fn delete_many(&self, keys: &[&str]) -> RiptideResult<usize> {
        // Default implementation: sequential deletes
        // Backends can override for batch deletion
        let mut count = 0;
        for key in keys {
            self.delete(key).await?;
            count += 1;
        }
        Ok(count)
    }

    /// Clear all keys matching a pattern
    ///
    /// **Warning**: This can be expensive on large datasets.
    /// Use with caution in production.
    ///
    /// # Arguments
    ///
    /// * `pattern` - Key pattern (e.g., "user:*")
    ///
    /// # Returns
    ///
    /// * `Ok(count)` - Number of keys deleted
    /// * `Err(_)` - Storage backend error or unsupported operation
    async fn clear_pattern(&self, _pattern: &str) -> RiptideResult<usize> {
        // Default implementation: not supported
        Err(crate::error::RiptideError::Cache(
            "Pattern clearing not supported by this backend".to_string(),
        ))
    }

    /// Get cache statistics
    ///
    /// # Returns
    ///
    /// * `Ok(stats)` - Cache statistics if available
    /// * `Err(_)` - Storage backend error
    async fn stats(&self) -> RiptideResult<CacheStats> {
        // Default implementation: minimal stats
        Ok(CacheStats::default())
    }

    /// Health check for cache backend
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Cache backend is healthy
    /// * `Ok(false)` - Cache backend is unhealthy
    /// * `Err(_)` - Unable to determine health
    async fn health_check(&self) -> RiptideResult<bool> {
        // Default implementation: try a simple operation
        const HEALTH_KEY: &str = "__health_check__";
        self.set(HEALTH_KEY, b"ok", Some(Duration::from_secs(1)))
            .await?;
        self.delete(HEALTH_KEY).await?;
        Ok(true)
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total number of keys
    pub total_keys: usize,
    /// Total memory usage in bytes
    pub memory_usage: usize,
    /// Hit rate (0.0 - 1.0)
    pub hit_rate: Option<f64>,
    /// Number of hits
    pub hits: usize,
    /// Number of misses
    pub misses: usize,
    /// Backend-specific metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl CacheStats {
    /// Calculate hit rate if hits/misses are available
    pub fn calculate_hit_rate(&mut self) {
        let total = self.hits + self.misses;
        if total > 0 {
            self.hit_rate = Some(self.hits as f64 / total as f64);
        }
    }
}
