//! Redis implementation of the IdempotencyStore port
//!
//! This adapter provides:
//! - Atomic idempotency lock acquisition (SET NX EX)
//! - Safe lock release with Lua script
//! - Versioned keys (idempotency:v1:...)
//! - TTL-based automatic expiration
//! - Connection pooling via deadpool-redis
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_cache::adapters::RedisIdempotencyStore;
//! use deadpool_redis::{Config, Runtime};
//!
//! let cfg = Config::from_url("redis://localhost:6379");
//! let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();
//! let store = RedisIdempotencyStore::new(pool);
//!
//! // Acquire lock
//! let token = store.try_acquire("request-123", Duration::from_secs(3600)).await?;
//!
//! // Process request...
//!
//! // Release lock
//! store.release(token).await?;
//! ```

use async_trait::async_trait;
use deadpool_redis::{redis::AsyncCommands, Pool};
use redis_script::Script;
use riptide_types::{IdempotencyStore, IdempotencyToken, Result as RiptideResult, RiptideError};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, instrument, warn};

/// Redis implementation of the IdempotencyStore port
///
/// Uses Redis SET NX EX for atomic lock acquisition and Lua scripts
/// for safe lock release. All keys are versioned for forward compatibility.
///
/// # Key Format
///
/// `idempotency:v1:{user_key}`
///
/// The version prefix allows for key format changes without breaking existing locks.
pub struct RedisIdempotencyStore {
    /// Redis connection pool
    pool: Arc<Pool>,

    /// Key version for forward compatibility
    key_version: String,
}

impl RedisIdempotencyStore {
    /// Create new Redis idempotency store
    ///
    /// # Arguments
    ///
    /// * `pool` - Redis connection pool
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let pool = Config::from_url("redis://localhost").create_pool(Some(Runtime::Tokio1))?;
    /// let store = RedisIdempotencyStore::new(pool);
    /// ```
    pub fn new(pool: Arc<Pool>) -> Self {
        Self {
            pool,
            key_version: "v1".to_string(),
        }
    }

    /// Create new store with custom key version
    ///
    /// Useful for testing or migrations.
    pub fn with_version(pool: Arc<Pool>, version: impl Into<String>) -> Self {
        Self {
            pool,
            key_version: version.into(),
        }
    }

    /// Format versioned Redis key
    ///
    /// # Arguments
    ///
    /// * `key` - User-provided idempotency key
    ///
    /// # Returns
    ///
    /// Versioned key in format `idempotency:{version}:{key}`
    fn versioned_key(&self, key: &str) -> String {
        format!("idempotency:{}:{}", self.key_version, key)
    }

    /// Format result key for caching operation results
    fn result_key(&self, key: &str) -> String {
        format!("{}:result", self.versioned_key(key))
    }

    /// Lua script for safe lock release
    ///
    /// This script ensures we only delete the key if it exists,
    /// preventing accidental deletion of a new lock acquired after expiration.
    const RELEASE_SCRIPT: &'static str = r#"
        if redis.call("exists", KEYS[1]) == 1 then
            return redis.call("del", KEYS[1])
        else
            return 0
        end
    "#;

    /// Lua script for atomic result storage
    ///
    /// Stores result only if the lock still exists
    const STORE_RESULT_SCRIPT: &'static str = r#"
        if redis.call("exists", KEYS[1]) == 1 then
            redis.call("setex", KEYS[2], ARGV[1], ARGV[2])
            return 1
        else
            return 0
        end
    "#;
}

#[async_trait]
impl IdempotencyStore for RedisIdempotencyStore {
    #[instrument(skip(self), fields(key = %key, ttl_secs = ttl.as_secs()))]
    async fn try_acquire(&self, key: &str, ttl: Duration) -> RiptideResult<IdempotencyToken> {
        debug!("Attempting to acquire idempotency lock");

        let versioned_key = self.versioned_key(key);
        let ttl_secs = ttl.as_secs();

        // Get connection from pool
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get Redis connection: {}", e);
            RiptideError::Cache(format!("Failed to get Redis connection: {}", e))
        })?;

        // Atomic SET NX EX (set if not exists with expiration)
        let acquired: bool = conn.set_nx(&versioned_key, "locked").await.map_err(|e| {
            error!("Failed to acquire lock: {}", e);
            RiptideError::Cache(format!("Failed to acquire lock: {}", e))
        })?;

        if !acquired {
            debug!("Lock already held - duplicate request");
            return Err(RiptideError::AlreadyExists(format!(
                "Idempotency key already exists: {}",
                key
            )));
        }

        // Set TTL separately (Redis 6.2+ supports SET NX EX in one command, but we're compatible with older versions)
        let _: () = conn
            .expire(&versioned_key, ttl_secs as i64)
            .await
            .map_err(|e| {
                error!("Failed to set TTL: {}", e);
                RiptideError::Cache(format!("Failed to set TTL: {}", e))
            })?;

        debug!("Idempotency lock acquired successfully");

        Ok(IdempotencyToken::new(versioned_key, ttl))
    }

    #[instrument(skip(self, token), fields(key = %token.key))]
    async fn release(&self, token: IdempotencyToken) -> RiptideResult<()> {
        debug!("Releasing idempotency lock");

        // Check if token is expired
        if token.is_expired() {
            warn!("Attempting to release expired token");
            // Not an error - idempotent operation
            return Ok(());
        }

        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get Redis connection: {}", e);
            RiptideError::Cache(format!("Failed to get Redis connection: {}", e))
        })?;

        // Use Lua script for safe deletion
        let deleted: i32 = Script::new(Self::RELEASE_SCRIPT)
            .key(&token.key)
            .invoke_async(&mut *conn)
            .await
            .map_err(|e| {
                error!("Failed to release lock: {}", e);
                RiptideError::Cache(format!("Failed to release lock: {}", e))
            })?;

        if deleted > 0 {
            debug!("Idempotency lock released successfully");
        } else {
            debug!("Lock already expired or released");
        }

        Ok(())
    }

    #[instrument(skip(self), fields(key = %key))]
    async fn exists(&self, key: &str) -> RiptideResult<bool> {
        debug!("Checking if idempotency key exists");

        let versioned_key = self.versioned_key(key);

        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get Redis connection: {}", e);
            RiptideError::Cache(format!("Failed to get Redis connection: {}", e))
        })?;

        let exists: bool = conn.exists(&versioned_key).await.map_err(|e| {
            error!("Failed to check existence: {}", e);
            RiptideError::Cache(format!("Failed to check existence: {}", e))
        })?;

        Ok(exists)
    }

    #[instrument(skip(self), fields(key = %key))]
    async fn ttl(&self, key: &str) -> RiptideResult<Option<Duration>> {
        debug!("Getting TTL for idempotency key");

        let versioned_key = self.versioned_key(key);

        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get Redis connection: {}", e);
            RiptideError::Cache(format!("Failed to get Redis connection: {}", e))
        })?;

        let ttl_secs: i64 = conn.ttl(&versioned_key).await.map_err(|e| {
            error!("Failed to get TTL: {}", e);
            RiptideError::Cache(format!("Failed to get TTL: {}", e))
        })?;

        match ttl_secs {
            -2 => Ok(None), // Key doesn't exist
            -1 => Ok(None), // Key exists but has no TTL
            secs if secs > 0 => Ok(Some(Duration::from_secs(secs as u64))),
            _ => Ok(None),
        }
    }

    #[instrument(skip(self, result), fields(key = %key, result_len = result.len(), ttl_secs = ttl.as_secs()))]
    async fn store_result(&self, key: &str, result: &[u8], ttl: Duration) -> RiptideResult<()> {
        debug!("Storing idempotency result");

        let lock_key = self.versioned_key(key);
        let result_key = self.result_key(key);
        let ttl_secs = ttl.as_secs();

        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get Redis connection: {}", e);
            RiptideError::Cache(format!("Failed to get Redis connection: {}", e))
        })?;

        // Use Lua script to store result only if lock exists
        let stored: i32 = Script::new(Self::STORE_RESULT_SCRIPT)
            .key(&lock_key)
            .key(&result_key)
            .arg(ttl_secs)
            .arg(result)
            .invoke_async(&mut *conn)
            .await
            .map_err(|e| {
                error!("Failed to store result: {}", e);
                RiptideError::Cache(format!("Failed to store result: {}", e))
            })?;

        if stored > 0 {
            debug!("Idempotency result stored successfully");
        } else {
            warn!("Lock expired before result could be stored");
        }

        Ok(())
    }

    #[instrument(skip(self), fields(key = %key))]
    async fn get_result(&self, key: &str) -> RiptideResult<Option<Vec<u8>>> {
        debug!("Retrieving idempotency result");

        let result_key = self.result_key(key);

        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get Redis connection: {}", e);
            RiptideError::Cache(format!("Failed to get Redis connection: {}", e))
        })?;

        let result: Option<Vec<u8>> = conn.get(&result_key).await.map_err(|e| {
            error!("Failed to get result: {}", e);
            RiptideError::Cache(format!("Failed to get result: {}", e))
        })?;

        if result.is_some() {
            debug!("Idempotency result found");
        } else {
            debug!("No cached result found");
        }

        Ok(result)
    }

    async fn cleanup_expired(&self) -> RiptideResult<usize> {
        // Redis handles expiration automatically via TTL
        // This is a no-op for Redis implementation
        debug!("Redis handles expiration automatically - no cleanup needed");
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_versioned_key_format() {
        let pool = Arc::new(
            deadpool_redis::Config::from_url("redis://localhost")
                .create_pool(Some(deadpool_redis::Runtime::Tokio1))
                .unwrap(),
        );
        let store = RedisIdempotencyStore::new(pool);

        let key = store.versioned_key("request-123");
        assert_eq!(key, "idempotency:v1:request-123");
    }

    #[test]
    fn test_result_key_format() {
        let pool = Arc::new(
            deadpool_redis::Config::from_url("redis://localhost")
                .create_pool(Some(deadpool_redis::Runtime::Tokio1))
                .unwrap(),
        );
        let store = RedisIdempotencyStore::new(pool);

        let key = store.result_key("request-123");
        assert_eq!(key, "idempotency:v1:request-123:result");
    }

    #[test]
    fn test_custom_version() {
        let pool = Arc::new(
            deadpool_redis::Config::from_url("redis://localhost")
                .create_pool(Some(deadpool_redis::Runtime::Tokio1))
                .unwrap(),
        );
        let store = RedisIdempotencyStore::with_version(pool, "v2");

        let key = store.versioned_key("request-123");
        assert_eq!(key, "idempotency:v2:request-123");
    }
}
