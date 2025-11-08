//! Idempotency store port for duplicate request prevention
//!
//! This module provides backend-agnostic idempotency interfaces that enable:
//! - Preventing duplicate API requests
//! - Testing with in-memory idempotency stores
//! - Swapping storage backends (Redis, PostgreSQL, etc.)
//! - Distributed lock semantics with TTL
//!
//! # Design Goals
//!
//! - **Safety**: Prevent duplicate operations in distributed systems
//! - **Testability**: In-memory store for unit tests
//! - **Flexibility**: Support various storage backends
//! - **Performance**: Fast distributed locking with minimal overhead
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_types::ports::IdempotencyStore;
//! use std::time::Duration;
//!
//! async fn example(store: &dyn IdempotencyStore) -> Result<()> {
//!     let key = "request-abc123";
//!     let ttl = Duration::from_secs(3600);
//!
//!     // Try to acquire idempotency lock
//!     match store.try_acquire(key, ttl).await {
//!         Ok(token) => {
//!             // First request - process it
//!             process_request().await?;
//!             store.release(token).await?;
//!         }
//!         Err(_) => {
//!             // Duplicate request - reject or return cached result
//!             return Err(RiptideError::DuplicateRequest);
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

use crate::error::Result as RiptideResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// Idempotency token representing acquired lock
///
/// Tokens are used to release acquired idempotency keys.
/// They should be opaque to callers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyToken {
    /// Idempotency key this token represents
    pub key: String,

    /// Timestamp when lock was acquired
    #[serde(with = "system_time_serialization")]
    pub acquired_at: SystemTime,

    /// Timestamp when lock will expire
    #[serde(with = "system_time_serialization")]
    pub expires_at: SystemTime,
}

impl IdempotencyToken {
    /// Create new idempotency token
    ///
    /// # Arguments
    ///
    /// * `key` - Idempotency key
    /// * `ttl` - Time-to-live for the lock
    pub fn new(key: impl Into<String>, ttl: Duration) -> Self {
        let now = SystemTime::now();
        Self {
            key: key.into(),
            acquired_at: now,
            expires_at: now + ttl,
        }
    }

    /// Check if token has expired
    pub fn is_expired(&self) -> bool {
        SystemTime::now() >= self.expires_at
    }

    /// Get remaining time until expiration
    pub fn remaining_ttl(&self) -> Option<Duration> {
        self.expires_at.duration_since(SystemTime::now()).ok()
    }
}

/// Idempotency store for duplicate request prevention
///
/// Implementations must be thread-safe and support distributed
/// lock semantics. Common backends include Redis and PostgreSQL
/// with advisory locks.
#[async_trait]
pub trait IdempotencyStore: Send + Sync {
    /// Try to acquire idempotency lock
    ///
    /// # Arguments
    ///
    /// * `key` - Idempotency key (e.g., request ID)
    /// * `ttl` - Time-to-live for the lock
    ///
    /// # Returns
    ///
    /// * `Ok(token)` - Lock acquired successfully
    /// * `Err(_)` - Lock already held (duplicate request) or backend error
    ///
    /// # Semantics
    ///
    /// This operation should be atomic (SET NX semantics in Redis).
    /// If the key already exists, acquisition fails immediately.
    async fn try_acquire(&self, key: &str, ttl: Duration) -> RiptideResult<IdempotencyToken>;

    /// Release idempotency lock
    ///
    /// # Arguments
    ///
    /// * `token` - Token from successful acquisition
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Lock released successfully (or already expired)
    /// * `Err(_)` - Backend error
    ///
    /// # Semantics
    ///
    /// Should be idempotent - releasing non-existent lock is not an error.
    /// Implementations should verify token ownership before deletion.
    async fn release(&self, token: IdempotencyToken) -> RiptideResult<()>;

    /// Check if idempotency key exists
    ///
    /// # Arguments
    ///
    /// * `key` - Idempotency key to check
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Key exists and lock is held
    /// * `Ok(false)` - Key doesn't exist or has expired
    /// * `Err(_)` - Backend error
    async fn exists(&self, key: &str) -> RiptideResult<bool>;

    /// Get remaining TTL for idempotency key
    ///
    /// # Arguments
    ///
    /// * `key` - Idempotency key
    ///
    /// # Returns
    ///
    /// * `Ok(Some(duration))` - Key exists with remaining TTL
    /// * `Ok(None)` - Key doesn't exist or has no TTL
    /// * `Err(_)` - Backend error
    async fn ttl(&self, key: &str) -> RiptideResult<Option<Duration>> {
        // Default implementation - backends should override
        if self.exists(key).await? {
            // Cannot determine actual TTL without backend support
            Ok(None)
        } else {
            Ok(None)
        }
    }

    /// Store result associated with idempotency key
    ///
    /// Allows caching operation results for duplicate request handling.
    ///
    /// # Arguments
    ///
    /// * `key` - Idempotency key
    /// * `result` - Serialized result to cache
    /// * `ttl` - Time-to-live for cached result
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Result stored successfully
    /// * `Err(_)` - Backend error
    async fn store_result(&self, key: &str, result: &[u8], ttl: Duration) -> RiptideResult<()> {
        // Default implementation - use key with suffix
        let result_key = format!("{}:result", key);
        // Backends should implement this using their native storage
        let _ = (result_key, result, ttl);
        Ok(())
    }

    /// Retrieve cached result for idempotency key
    ///
    /// # Arguments
    ///
    /// * `key` - Idempotency key
    ///
    /// # Returns
    ///
    /// * `Ok(Some(result))` - Cached result found
    /// * `Ok(None)` - No cached result
    /// * `Err(_)` - Backend error
    async fn get_result(&self, key: &str) -> RiptideResult<Option<Vec<u8>>> {
        // Default implementation
        let _ = key;
        Ok(None)
    }

    /// Cleanup expired idempotency keys
    ///
    /// Optional maintenance operation for backends that don't
    /// support automatic expiration.
    ///
    /// # Returns
    ///
    /// * `Ok(count)` - Number of keys cleaned up
    /// * `Err(_)` - Backend error
    async fn cleanup_expired(&self) -> RiptideResult<usize> {
        // Default implementation - no-op
        Ok(0)
    }
}

// Custom serialization for SystemTime
mod system_time_serialization {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time
            .duration_since(UNIX_EPOCH)
            .map_err(serde::ser::Error::custom)?;
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idempotency_token_creation() {
        let ttl = Duration::from_secs(3600);
        let token = IdempotencyToken::new("test-key", ttl);

        assert_eq!(token.key, "test-key");
        assert!(!token.is_expired());
        assert!(token.remaining_ttl().is_some());
    }

    #[test]
    fn test_idempotency_token_expiration() {
        let ttl = Duration::from_secs(0);
        let token = IdempotencyToken::new("test-key", ttl);

        // Token should be expired immediately
        std::thread::sleep(Duration::from_millis(10));
        assert!(token.is_expired());
        assert!(token.remaining_ttl().is_none());
    }

    #[test]
    fn test_idempotency_token_serialization() {
        let token = IdempotencyToken::new("test-key", Duration::from_secs(3600));

        let json = serde_json::to_string(&token).unwrap();
        let deserialized: IdempotencyToken = serde_json::from_str(&json).unwrap();

        assert_eq!(token.key, deserialized.key);
    }
}
