//! Redis adapter implementing CacheStorage trait
//!
//! This module provides a Redis backend for the `CacheStorage` trait,
//! enabling high-performance distributed caching with:
//! - Connection pooling with multiplexed connections
//! - Atomic batch operations (MSET, MGET, DEL)
//! - Native TTL support with EXPIRE
//! - Statistics and monitoring
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_cache::RedisStorage;
//! use riptide_types::ports::CacheStorage;
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let storage = RedisStorage::new("redis://localhost:6379").await?;
//!
//!     storage.set("key", b"value", Some(Duration::from_secs(3600))).await?;
//!
//!     if let Some(data) = storage.get("key").await? {
//!         println!("Found: {:?}", data);
//!     }
//!
//!     Ok(())
//! }
//! ```

use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client, RedisError};
use riptide_types::error::{Result as RiptideResult, RiptideError};
use riptide_types::ports::cache::{CacheStats, CacheStorage};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};

/// Redis-backed cache storage
///
/// Uses Redis multiplexed connections for efficient concurrent access.
/// All operations are async and thread-safe.
pub struct RedisStorage {
    conn: MultiplexedConnection,
    hits: Arc<AtomicUsize>,
    misses: Arc<AtomicUsize>,
    client: Client, // Keep client for health checks and stats
}

impl RedisStorage {
    /// Create new Redis storage with default connection
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection URL (e.g., "redis://localhost:6379")
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let storage = RedisStorage::new("redis://localhost:6379").await?;
    /// ```
    pub async fn new(redis_url: &str) -> anyhow::Result<Self> {
        let client = Client::open(redis_url)?;
        let conn = client.get_multiplexed_tokio_connection().await?;

        Ok(Self {
            conn,
            hits: Arc::new(AtomicUsize::new(0)),
            misses: Arc::new(AtomicUsize::new(0)),
            client,
        })
    }

    /// Create from existing Redis client
    pub async fn from_client(client: Client) -> anyhow::Result<Self> {
        let conn = client.get_multiplexed_tokio_connection().await?;

        Ok(Self {
            conn,
            hits: Arc::new(AtomicUsize::new(0)),
            misses: Arc::new(AtomicUsize::new(0)),
            client,
        })
    }

    /// Reset statistics counters
    pub fn reset_stats(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
    }

    /// Convert Redis error to RiptideError
    fn convert_error(err: RedisError) -> RiptideError {
        RiptideError::Cache(format!("Redis error: {}", err))
    }

    /// Get database size (number of keys)
    async fn get_db_size(&self) -> RiptideResult<usize> {
        let mut conn = self.conn.clone();
        let size: usize = redis::cmd("DBSIZE")
            .query_async(&mut conn)
            .await
            .map_err(Self::convert_error)?;
        Ok(size)
    }

    /// Get memory usage info from Redis
    async fn get_memory_info(&self) -> RiptideResult<usize> {
        let mut conn = self.conn.clone();
        // Try to get memory usage from INFO command
        let info: String = redis::cmd("INFO")
            .arg("MEMORY")
            .query_async(&mut conn)
            .await
            .map_err(Self::convert_error)?;

        // Parse used_memory from INFO output
        for line in info.lines() {
            if line.starts_with("used_memory:") {
                if let Some(value) = line.split(':').nth(1) {
                    return value
                        .trim()
                        .parse()
                        .map_err(|_| RiptideError::Cache("Failed to parse memory usage".into()));
                }
            }
        }

        Ok(0) // Default if parsing fails
    }
}

#[async_trait::async_trait]
impl CacheStorage for RedisStorage {
    async fn get(&self, key: &str) -> RiptideResult<Option<Vec<u8>>> {
        let mut conn = self.conn.clone();
        let result: Option<Vec<u8>> = conn.get(key).await.map_err(Self::convert_error)?;

        if result.is_some() {
            self.hits.fetch_add(1, Ordering::Relaxed);
            debug!(key = %key, "Redis cache hit");
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            debug!(key = %key, "Redis cache miss");
        }

        Ok(result)
    }

    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> RiptideResult<()> {
        let mut conn = self.conn.clone();

        if let Some(duration) = ttl {
            let seconds = duration.as_secs();
            let _: () = conn
                .set_ex(key, value, seconds)
                .await
                .map_err(Self::convert_error)?;
            debug!(key = %key, ttl_seconds = seconds, "Set with TTL");
        } else {
            let _: () = conn.set(key, value).await.map_err(Self::convert_error)?;
            debug!(key = %key, "Set without TTL");
        }

        Ok(())
    }

    async fn delete(&self, key: &str) -> RiptideResult<()> {
        let mut conn = self.conn.clone();
        let _: () = conn.del(key).await.map_err(Self::convert_error)?;
        debug!(key = %key, "Deleted key");
        Ok(())
    }

    async fn exists(&self, key: &str) -> RiptideResult<bool> {
        let mut conn = self.conn.clone();
        let exists: bool = conn.exists(key).await.map_err(Self::convert_error)?;
        Ok(exists)
    }

    async fn mset(&self, items: Vec<(&str, &[u8])>, ttl: Option<Duration>) -> RiptideResult<()> {
        if items.is_empty() {
            return Ok(());
        }

        let mut conn = self.conn.clone();

        // Redis MSET doesn't support TTL, so we need a pipeline
        let mut pipe = redis::pipe();
        pipe.atomic();

        for (key, value) in &items {
            if let Some(duration) = ttl {
                let seconds = duration.as_secs();
                pipe.set_ex(*key, *value, seconds);
            } else {
                pipe.set(*key, *value);
            }
        }

        let _: () = pipe
            .query_async(&mut conn)
            .await
            .map_err(Self::convert_error)?;

        debug!(count = items.len(), "Batch set completed");
        Ok(())
    }

    async fn mget(&self, keys: &[&str]) -> RiptideResult<Vec<Option<Vec<u8>>>> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = self.conn.clone();
        let results: Vec<Option<Vec<u8>>> = conn.get(keys).await.map_err(Self::convert_error)?;

        // Update statistics
        for result in &results {
            if result.is_some() {
                self.hits.fetch_add(1, Ordering::Relaxed);
            } else {
                self.misses.fetch_add(1, Ordering::Relaxed);
            }
        }

        debug!(count = keys.len(), "Batch get completed");
        Ok(results)
    }

    async fn expire(&self, key: &str, ttl: Duration) -> RiptideResult<bool> {
        let mut conn = self.conn.clone();
        let seconds = ttl.as_secs() as i64;
        let result: bool = conn
            .expire(key, seconds)
            .await
            .map_err(Self::convert_error)?;
        debug!(key = %key, ttl_seconds = seconds, set = result, "Set expiration");
        Ok(result)
    }

    async fn ttl(&self, key: &str) -> RiptideResult<Option<Duration>> {
        let mut conn = self.conn.clone();
        let seconds: i64 = conn.ttl(key).await.map_err(Self::convert_error)?;

        match seconds {
            -2 => Ok(None), // Key doesn't exist
            -1 => Ok(None), // Key has no expiration
            s if s > 0 => Ok(Some(Duration::from_secs(s as u64))),
            _ => Ok(None),
        }
    }

    async fn incr(&self, key: &str, delta: i64) -> RiptideResult<i64> {
        let mut conn = self.conn.clone();
        let new_value: i64 = conn.incr(key, delta).await.map_err(Self::convert_error)?;
        debug!(key = %key, delta, new_value, "Incremented counter");
        Ok(new_value)
    }

    async fn delete_many(&self, keys: &[&str]) -> RiptideResult<usize> {
        if keys.is_empty() {
            return Ok(0);
        }

        let mut conn = self.conn.clone();
        let count: usize = conn.del(keys).await.map_err(Self::convert_error)?;
        debug!(count, total_keys = keys.len(), "Batch delete completed");
        Ok(count)
    }

    async fn clear_pattern(&self, pattern: &str) -> RiptideResult<usize> {
        let mut conn = self.conn.clone();

        // Use SCAN to iterate through keys matching pattern
        let mut cursor = 0;
        let mut total_deleted = 0;

        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(pattern)
                .arg("COUNT")
                .arg(100) // Scan in batches
                .query_async(&mut conn)
                .await
                .map_err(Self::convert_error)?;

            if !keys.is_empty() {
                let keys_refs: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();
                let deleted = self.delete_many(&keys_refs).await?;
                total_deleted += deleted;
            }

            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }

        warn!(pattern = %pattern, deleted = total_deleted, "Cleared keys matching pattern");
        Ok(total_deleted)
    }

    async fn stats(&self) -> RiptideResult<CacheStats> {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);

        let total_keys = self.get_db_size().await.unwrap_or(0);
        let memory_usage = self.get_memory_info().await.unwrap_or(0);

        let mut stats = CacheStats {
            total_keys,
            memory_usage,
            hit_rate: None,
            hits,
            misses,
            metadata: HashMap::new(),
        };

        stats.calculate_hit_rate();
        stats
            .metadata
            .insert("backend".to_string(), "redis".to_string());

        Ok(stats)
    }

    async fn health_check(&self) -> RiptideResult<bool> {
        let mut conn = self.conn.clone();

        // Try a PING command
        let response: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(Self::convert_error)?;

        Ok(response == "PONG")
    }
}

// Implement Clone for RedisStorage
impl Clone for RedisStorage {
    fn clone(&self) -> Self {
        Self {
            conn: self.conn.clone(),
            hits: self.hits.clone(),
            misses: self.misses.clone(),
            client: self.client.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running Redis instance
    // Run with: docker run -p 6379:6379 redis:alpine

    async fn create_test_storage() -> RedisStorage {
        RedisStorage::new("redis://localhost:6379")
            .await
            .expect("Failed to connect to Redis. Is Redis running?")
    }

    #[tokio::test]
    #[ignore] // Requires Redis instance
    async fn test_redis_basic_operations() {
        let storage = create_test_storage().await;

        // Set and get
        storage.set("test:key1", b"value1", None).await.unwrap();
        let result = storage.get("test:key1").await.unwrap();
        assert_eq!(result, Some(b"value1".to_vec()));

        // Exists
        assert!(storage.exists("test:key1").await.unwrap());
        assert!(!storage.exists("test:nonexistent").await.unwrap());

        // Delete
        storage.delete("test:key1").await.unwrap();
        assert!(!storage.exists("test:key1").await.unwrap());
    }

    #[tokio::test]
    #[ignore] // Requires Redis instance
    async fn test_redis_ttl() {
        let storage = create_test_storage().await;

        // Set with TTL
        storage
            .set("test:ttl", b"value", Some(Duration::from_secs(10)))
            .await
            .unwrap();

        // Check TTL
        let ttl = storage.ttl("test:ttl").await.unwrap();
        assert!(ttl.is_some());
        assert!(ttl.unwrap().as_secs() <= 10);

        // Cleanup
        storage.delete("test:ttl").await.unwrap();
    }

    #[tokio::test]
    #[ignore] // Requires Redis instance
    async fn test_redis_batch_operations() {
        let storage = create_test_storage().await;

        // Multi-set
        let items = vec![("test:batch1", b"val1" as &[u8]), ("test:batch2", b"val2")];
        storage.mset(items, None).await.unwrap();

        // Multi-get
        let results = storage
            .mget(&["test:batch1", "test:batch2", "test:batch3"])
            .await
            .unwrap();
        assert_eq!(results[0], Some(b"val1".to_vec()));
        assert_eq!(results[1], Some(b"val2".to_vec()));
        assert_eq!(results[2], None);

        // Cleanup
        storage
            .delete_many(&["test:batch1", "test:batch2"])
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore] // Requires Redis instance
    async fn test_redis_health_check() {
        let storage = create_test_storage().await;
        assert!(storage.health_check().await.unwrap());
    }
}
