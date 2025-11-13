//! In-memory idempotency store implementation for testing and development
//!
//! This module provides a thread-safe in-memory idempotency store that implements
//! the `IdempotencyStore` trait. It's ideal for:
//! - Unit testing without Redis
//! - Development environments
//! - Single-instance deployments
//!
//! # Features
//!
//! - Thread-safe with `DashMap` for concurrent access
//! - TTL tracking with automatic expiration
//! - Background cleanup task for expired entries
//! - Result caching with separate TTL
//! - Graceful shutdown
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_types::ports::{IdempotencyStore, InMemoryIdempotencyStore};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let store = InMemoryIdempotencyStore::new();
//!
//!     let ttl = Duration::from_secs(3600);
//!     match store.try_acquire("request-123", ttl).await {
//!         Ok(token) => {
//!             // Process request
//!             store.release(token).await?;
//!         }
//!         Err(_) => {
//!             // Duplicate request
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

use crate::error::{Result as RiptideResult, RiptideError};
use crate::ports::idempotency::{IdempotencyStore, IdempotencyToken};
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;

/// Entry in the idempotency store with metadata
#[derive(Debug, Clone)]
struct IdempotencyEntry {
    /// When this entry expires
    expires_at: Instant,
    /// Cached result (if any)
    result: Option<Vec<u8>>,
    /// When the result expires (if different from key)
    result_expires_at: Option<Instant>,
}

impl IdempotencyEntry {
    /// Create new idempotency entry
    fn new(_key: String, ttl: Duration) -> Self {
        Self {
            expires_at: Instant::now() + ttl,
            result: None,
            result_expires_at: None,
        }
    }

    /// Check if entry has expired
    fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }

    /// Check if result has expired
    fn is_result_expired(&self) -> bool {
        if let Some(result_exp) = self.result_expires_at {
            Instant::now() >= result_exp
        } else {
            false
        }
    }

    /// Get remaining TTL
    fn remaining_ttl(&self) -> Option<Duration> {
        let now = Instant::now();
        if now < self.expires_at {
            Some(self.expires_at - now)
        } else {
            None
        }
    }

    /// Store result with TTL
    fn store_result(&mut self, result: Vec<u8>, ttl: Duration) {
        self.result = Some(result);
        self.result_expires_at = Some(Instant::now() + ttl);
    }

    /// Get result if not expired
    fn get_result(&self) -> Option<Vec<u8>> {
        if self.is_result_expired() {
            None
        } else {
            self.result.clone()
        }
    }
}

/// Thread-safe in-memory idempotency store implementation
///
/// Uses `DashMap` for lock-free concurrent access with background cleanup task
/// for expired entries. Supports result caching with independent TTL.
#[derive(Clone)]
pub struct InMemoryIdempotencyStore {
    /// Thread-safe concurrent map for entries
    entries: Arc<DashMap<String, IdempotencyEntry>>,
    /// Background cleanup task handle
    cleanup_handle: Arc<tokio::sync::Mutex<Option<JoinHandle<()>>>>,
    /// Shutdown signal for cleanup task
    shutdown: Arc<AtomicBool>,
    /// Cleanup interval
    cleanup_interval: Duration,
}

impl InMemoryIdempotencyStore {
    /// Create a new in-memory idempotency store with default cleanup interval (60 seconds)
    pub fn new() -> Self {
        Self::with_cleanup_interval(Duration::from_secs(60))
    }

    /// Create a new in-memory idempotency store with custom cleanup interval
    ///
    /// # Arguments
    ///
    /// * `cleanup_interval` - How often to run the background cleanup task
    pub fn with_cleanup_interval(cleanup_interval: Duration) -> Self {
        let store = Self {
            entries: Arc::new(DashMap::new()),
            cleanup_handle: Arc::new(tokio::sync::Mutex::new(None)),
            shutdown: Arc::new(AtomicBool::new(false)),
            cleanup_interval,
        };

        // Start background cleanup task
        let store_clone = store.clone();
        let handle = tokio::spawn(async move {
            store_clone.background_cleanup().await;
        });

        // Store handle for graceful shutdown
        let handle_mutex = store.cleanup_handle.clone();
        tokio::spawn(async move {
            let mut guard = handle_mutex.lock().await;
            *guard = Some(handle);
        });

        store
    }

    /// Background cleanup task that removes expired entries
    async fn background_cleanup(&self) {
        loop {
            // Check shutdown signal
            if self.shutdown.load(Ordering::Relaxed) {
                break;
            }

            // Sleep for cleanup interval
            tokio::time::sleep(self.cleanup_interval).await;

            // Check shutdown again after sleep
            if self.shutdown.load(Ordering::Relaxed) {
                break;
            }

            // Remove expired entries
            self.entries.retain(|_, entry| !entry.is_expired());
        }
    }

    /// Manually trigger cleanup of expired entries (for testing)
    pub async fn cleanup_expired_sync(&self) -> usize {
        let before = self.entries.len();
        self.entries.retain(|_, entry| !entry.is_expired());
        before - self.entries.len()
    }

    /// Get current number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Gracefully shutdown the background cleanup task
    pub async fn shutdown(&self) {
        // Signal shutdown
        self.shutdown.store(true, Ordering::Relaxed);

        // Wait for cleanup task to finish
        let mut guard = self.cleanup_handle.lock().await;
        if let Some(handle) = guard.take() {
            let _ = handle.await;
        }
    }
}

impl Default for InMemoryIdempotencyStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for InMemoryIdempotencyStore {
    fn drop(&mut self) {
        // Signal shutdown when store is dropped
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

#[async_trait]
impl IdempotencyStore for InMemoryIdempotencyStore {
    async fn try_acquire(&self, key: &str, ttl: Duration) -> RiptideResult<IdempotencyToken> {
        // Try to insert new entry atomically
        let entry = IdempotencyEntry::new(key.to_string(), ttl);

        // Check if entry already exists
        if let Some(existing) = self.entries.get(key) {
            if existing.is_expired() {
                // Expired entry - remove and retry
                drop(existing);
                self.entries.remove(key);

                // Insert new entry
                self.entries.insert(key.to_string(), entry);
                Ok(IdempotencyToken::new(key, ttl))
            } else {
                // Valid lock exists
                Err(RiptideError::AlreadyExists(format!(
                    "Idempotency key '{}' already acquired",
                    key
                )))
            }
        } else {
            // No entry exists, insert new one
            self.entries.insert(key.to_string(), entry);
            Ok(IdempotencyToken::new(key, ttl))
        }
    }

    async fn release(&self, token: IdempotencyToken) -> RiptideResult<()> {
        // Remove entry if it exists
        self.entries.remove(&token.key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> RiptideResult<bool> {
        if let Some(entry) = self.entries.get(key) {
            if entry.is_expired() {
                // Clean up expired entry
                drop(entry);
                self.entries.remove(key);
                Ok(false)
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    async fn ttl(&self, key: &str) -> RiptideResult<Option<Duration>> {
        if let Some(entry) = self.entries.get(key) {
            if entry.is_expired() {
                drop(entry);
                self.entries.remove(key);
                Ok(None)
            } else {
                Ok(entry.remaining_ttl())
            }
        } else {
            Ok(None)
        }
    }

    async fn store_result(&self, key: &str, result: &[u8], ttl: Duration) -> RiptideResult<()> {
        if let Some(mut entry) = self.entries.get_mut(key) {
            if !entry.is_expired() {
                entry.store_result(result.to_vec(), ttl);
                return Ok(());
            }
        }

        Err(RiptideError::NotFound(format!(
            "Idempotency key '{}' not found or expired",
            key
        )))
    }

    async fn get_result(&self, key: &str) -> RiptideResult<Option<Vec<u8>>> {
        if let Some(entry) = self.entries.get(key) {
            if entry.is_expired() {
                drop(entry);
                self.entries.remove(key);
                Ok(None)
            } else {
                Ok(entry.get_result())
            }
        } else {
            Ok(None)
        }
    }

    async fn cleanup_expired(&self) -> RiptideResult<usize> {
        Ok(self.cleanup_expired_sync().await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_try_acquire_success() {
        let store = InMemoryIdempotencyStore::new();
        let ttl = Duration::from_secs(3600);

        let token = store.try_acquire("test-key", ttl).await.unwrap();
        assert_eq!(token.key, "test-key");
        assert!(!token.is_expired());
    }

    #[tokio::test]
    async fn test_try_acquire_duplicate() {
        let store = InMemoryIdempotencyStore::new();
        let ttl = Duration::from_secs(3600);

        // First acquisition succeeds
        store.try_acquire("test-key", ttl).await.unwrap();

        // Second acquisition fails
        let result = store.try_acquire("test-key", ttl).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RiptideError::AlreadyExists(_)
        ));
    }

    #[tokio::test]
    async fn test_release() {
        let store = InMemoryIdempotencyStore::new();
        let ttl = Duration::from_secs(3600);

        let token = store.try_acquire("test-key", ttl).await.unwrap();
        assert!(store.exists("test-key").await.unwrap());

        store.release(token).await.unwrap();
        assert!(!store.exists("test-key").await.unwrap());
    }

    #[tokio::test]
    async fn test_exists() {
        let store = InMemoryIdempotencyStore::new();
        let ttl = Duration::from_secs(3600);

        assert!(!store.exists("test-key").await.unwrap());

        store.try_acquire("test-key", ttl).await.unwrap();
        assert!(store.exists("test-key").await.unwrap());
    }

    #[tokio::test]
    async fn test_ttl() {
        let store = InMemoryIdempotencyStore::new();
        let ttl = Duration::from_secs(3600);

        store.try_acquire("test-key", ttl).await.unwrap();

        let remaining = store.ttl("test-key").await.unwrap();
        assert!(remaining.is_some());
        assert!(remaining.unwrap() <= ttl);
        assert!(remaining.unwrap() > Duration::from_secs(3595));
    }

    #[tokio::test]
    async fn test_ttl_nonexistent() {
        let store = InMemoryIdempotencyStore::new();

        let remaining = store.ttl("nonexistent").await.unwrap();
        assert!(remaining.is_none());
    }

    #[tokio::test]
    async fn test_expiration() {
        let store = InMemoryIdempotencyStore::new();
        let ttl = Duration::from_millis(100);

        store.try_acquire("test-key", ttl).await.unwrap();
        assert!(store.exists("test-key").await.unwrap());

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Entry should be expired and removed on access
        assert!(!store.exists("test-key").await.unwrap());
    }

    #[tokio::test]
    async fn test_acquire_after_expiration() {
        let store = InMemoryIdempotencyStore::new();
        let ttl = Duration::from_millis(100);

        store.try_acquire("test-key", ttl).await.unwrap();

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be able to acquire again after expiration
        let token = store
            .try_acquire("test-key", Duration::from_secs(3600))
            .await
            .unwrap();
        assert_eq!(token.key, "test-key");
    }

    #[tokio::test]
    async fn test_store_and_get_result() {
        let store = InMemoryIdempotencyStore::new();
        let ttl = Duration::from_secs(3600);

        store.try_acquire("test-key", ttl).await.unwrap();

        // Store result
        let result_data = b"test result";
        store
            .store_result("test-key", result_data, Duration::from_secs(300))
            .await
            .unwrap();

        // Retrieve result
        let retrieved = store.get_result("test-key").await.unwrap();
        assert_eq!(retrieved, Some(result_data.to_vec()));
    }

    #[tokio::test]
    async fn test_store_result_nonexistent_key() {
        let store = InMemoryIdempotencyStore::new();

        let result = store
            .store_result("nonexistent", b"data", Duration::from_secs(300))
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RiptideError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_result_expiration() {
        let store = InMemoryIdempotencyStore::new();
        let ttl = Duration::from_secs(3600);

        store.try_acquire("test-key", ttl).await.unwrap();

        // Store result with short TTL
        store
            .store_result("test-key", b"test", Duration::from_millis(100))
            .await
            .unwrap();

        // Result should exist immediately
        assert!(store.get_result("test-key").await.unwrap().is_some());

        // Wait for result expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Result should be expired (key still exists)
        assert!(store.get_result("test-key").await.unwrap().is_none());
        assert!(store.exists("test-key").await.unwrap());
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let store = InMemoryIdempotencyStore::new();
        let ttl = Duration::from_millis(100);

        // Add multiple entries
        store.try_acquire("key1", ttl).await.unwrap();
        store.try_acquire("key2", ttl).await.unwrap();
        store
            .try_acquire("key3", Duration::from_secs(3600))
            .await
            .unwrap();

        assert_eq!(store.len(), 3);

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Cleanup expired entries
        let cleaned = store.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 2);
        assert_eq!(store.len(), 1);
    }

    #[tokio::test]
    async fn test_background_cleanup() {
        // Short cleanup interval for testing
        let store = InMemoryIdempotencyStore::with_cleanup_interval(Duration::from_millis(50));
        let ttl = Duration::from_millis(100);

        // Add entries
        store.try_acquire("key1", ttl).await.unwrap();
        store.try_acquire("key2", ttl).await.unwrap();

        assert_eq!(store.len(), 2);

        // Wait for expiration and cleanup
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Background cleanup should have removed expired entries
        assert_eq!(store.len(), 0);

        // Shutdown cleanup task
        store.shutdown().await;
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let store = Arc::new(InMemoryIdempotencyStore::new());
        let ttl = Duration::from_secs(3600);

        // Spawn multiple tasks trying to acquire the same key
        let mut handles = vec![];
        for i in 0..10 {
            let store_clone = store.clone();
            let handle = tokio::spawn(async move {
                store_clone.try_acquire("concurrent-key", ttl).await.is_ok()
            });
            handles.push((i, handle));
        }

        // Collect results
        let mut success_count = 0;
        for (_, handle) in handles {
            if handle.await.unwrap() {
                success_count += 1;
            }
        }

        // Only one should succeed
        assert_eq!(success_count, 1);
        assert!(store.exists("concurrent-key").await.unwrap());
    }

    #[tokio::test]
    async fn test_graceful_shutdown() {
        let store = InMemoryIdempotencyStore::with_cleanup_interval(Duration::from_millis(10));

        // Add some entries
        store
            .try_acquire("key1", Duration::from_secs(3600))
            .await
            .unwrap();

        // Shutdown should complete without hanging
        store.shutdown().await;

        // Store should still be accessible
        assert!(store.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_release_idempotent() {
        let store = InMemoryIdempotencyStore::new();
        let ttl = Duration::from_secs(3600);

        let token = store.try_acquire("test-key", ttl).await.unwrap();

        // First release
        store.release(token.clone()).await.unwrap();

        // Second release should not error (idempotent)
        store.release(token).await.unwrap();
    }
}
