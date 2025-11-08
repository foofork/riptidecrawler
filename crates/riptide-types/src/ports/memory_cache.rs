//! In-memory cache implementation for testing and development
//!
//! This module provides a thread-safe in-memory cache that implements
//! the `CacheStorage` trait. It's ideal for:
//! - Unit testing without Redis
//! - Development environments
//! - Embedded scenarios
//!
//! # Features
//!
//! - Thread-safe with `DashMap` for concurrent access
//! - TTL support with automatic expiration checking
//! - Statistics tracking (hits, misses, memory usage)
//! - Zero external dependencies beyond standard library
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_types::ports::{CacheStorage, InMemoryCache};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let cache = InMemoryCache::new();
//!
//!     cache.set("key", b"value", Some(Duration::from_secs(60))).await?;
//!
//!     if let Some(data) = cache.get("key").await? {
//!         println!("Found: {:?}", data);
//!     }
//!
//!     Ok(())
//! }
//! ```

use crate::error::{Result as RiptideResult, RiptideError};
use crate::ports::cache::{CacheStats, CacheStorage};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Entry in the in-memory cache with metadata
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The cached data
    data: Vec<u8>,
    /// When this entry expires (if TTL was set)
    expires_at: Option<Instant>,
    /// When this entry was created (unused but kept for potential future features)
    #[allow(dead_code)]
    created_at: Instant,
}

impl CacheEntry {
    fn new(data: Vec<u8>, ttl: Option<Duration>) -> Self {
        let now = Instant::now();
        Self {
            data,
            expires_at: ttl.map(|d| now + d),
            created_at: now,
        }
    }

    fn is_expired(&self) -> bool {
        self.expires_at
            .map(|exp| Instant::now() >= exp)
            .unwrap_or(false)
    }

    fn remaining_ttl(&self) -> Option<Duration> {
        self.expires_at.and_then(|exp| {
            let now = Instant::now();
            if now < exp {
                Some(exp - now)
            } else {
                None
            }
        })
    }
}

/// Thread-safe in-memory cache implementation
///
/// Uses `RwLock<HashMap>` for thread-safe access with efficient reads.
/// Statistics are tracked using atomic counters.
#[derive(Clone)]
pub struct InMemoryCache {
    store: Arc<RwLock<HashMap<String, CacheEntry>>>,
    hits: Arc<AtomicUsize>,
    misses: Arc<AtomicUsize>,
}

impl InMemoryCache {
    /// Create a new in-memory cache
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            hits: Arc::new(AtomicUsize::new(0)),
            misses: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Create a cache with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::with_capacity(capacity))),
            hits: Arc::new(AtomicUsize::new(0)),
            misses: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Clear all entries (including expired ones)
    pub async fn clear(&self) {
        let mut store = self.store.write().await;
        store.clear();
    }

    /// Remove expired entries
    pub async fn cleanup_expired(&self) -> usize {
        let mut store = self.store.write().await;
        let before = store.len();
        store.retain(|_, entry| !entry.is_expired());
        before - store.len()
    }

    /// Get current cache size in number of entries
    pub async fn len(&self) -> usize {
        let store = self.store.read().await;
        store.len()
    }

    /// Check if cache is empty
    pub async fn is_empty(&self) -> bool {
        let store = self.store.read().await;
        store.is_empty()
    }

    /// Reset statistics counters
    pub fn reset_stats(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
    }

    /// Calculate estimated memory usage in bytes
    async fn calculate_memory_usage(&self) -> usize {
        let store = self.store.read().await;
        store
            .iter()
            .map(|(k, v)| {
                // Key size + value size + overhead for Entry struct
                k.len() + v.data.len() + std::mem::size_of::<CacheEntry>()
            })
            .sum()
    }
}

impl Default for InMemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CacheStorage for InMemoryCache {
    async fn get(&self, key: &str) -> RiptideResult<Option<Vec<u8>>> {
        let store = self.store.read().await;

        if let Some(entry) = store.get(key) {
            if entry.is_expired() {
                // Entry expired, count as miss
                self.misses.fetch_add(1, Ordering::Relaxed);
                drop(store);
                // Clean up expired entry
                let mut write_store = self.store.write().await;
                write_store.remove(key);
                Ok(None)
            } else {
                // Valid entry, count as hit
                self.hits.fetch_add(1, Ordering::Relaxed);
                Ok(Some(entry.data.clone()))
            }
        } else {
            // Not found, count as miss
            self.misses.fetch_add(1, Ordering::Relaxed);
            Ok(None)
        }
    }

    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> RiptideResult<()> {
        let entry = CacheEntry::new(value.to_vec(), ttl);
        let mut store = self.store.write().await;
        store.insert(key.to_string(), entry);
        Ok(())
    }

    async fn delete(&self, key: &str) -> RiptideResult<()> {
        let mut store = self.store.write().await;
        store.remove(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> RiptideResult<bool> {
        let store = self.store.read().await;

        if let Some(entry) = store.get(key) {
            if entry.is_expired() {
                drop(store);
                // Clean up expired entry
                let mut write_store = self.store.write().await;
                write_store.remove(key);
                Ok(false)
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    async fn mset(&self, items: Vec<(&str, &[u8])>, ttl: Option<Duration>) -> RiptideResult<()> {
        let mut store = self.store.write().await;
        for (key, value) in items {
            let entry = CacheEntry::new(value.to_vec(), ttl);
            store.insert(key.to_string(), entry);
        }
        Ok(())
    }

    async fn mget(&self, keys: &[&str]) -> RiptideResult<Vec<Option<Vec<u8>>>> {
        let store = self.store.read().await;
        let mut results = Vec::with_capacity(keys.len());
        let mut expired_keys = Vec::new();

        for key in keys {
            if let Some(entry) = store.get(*key) {
                if entry.is_expired() {
                    expired_keys.push(key.to_string());
                    results.push(None);
                    self.misses.fetch_add(1, Ordering::Relaxed);
                } else {
                    results.push(Some(entry.data.clone()));
                    self.hits.fetch_add(1, Ordering::Relaxed);
                }
            } else {
                results.push(None);
                self.misses.fetch_add(1, Ordering::Relaxed);
            }
        }

        // Clean up expired entries
        if !expired_keys.is_empty() {
            drop(store);
            let mut write_store = self.store.write().await;
            for key in expired_keys {
                write_store.remove(&key);
            }
        }

        Ok(results)
    }

    async fn expire(&self, key: &str, ttl: Duration) -> RiptideResult<bool> {
        let mut store = self.store.write().await;

        if let Some(entry) = store.get_mut(key) {
            if entry.is_expired() {
                store.remove(key);
                Ok(false)
            } else {
                entry.expires_at = Some(Instant::now() + ttl);
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    async fn ttl(&self, key: &str) -> RiptideResult<Option<Duration>> {
        let store = self.store.read().await;

        if let Some(entry) = store.get(key) {
            if entry.is_expired() {
                Ok(None)
            } else {
                Ok(entry.remaining_ttl())
            }
        } else {
            Ok(None)
        }
    }

    async fn incr(&self, key: &str, delta: i64) -> RiptideResult<i64> {
        let mut store = self.store.write().await;

        let current = if let Some(entry) = store.get(key) {
            if entry.is_expired() {
                0
            } else {
                String::from_utf8(entry.data.clone())
                    .map_err(|e| RiptideError::Cache(format!("Invalid UTF-8: {}", e)))?
                    .parse::<i64>()
                    .map_err(|e| RiptideError::Cache(format!("Not a number: {}", e)))?
            }
        } else {
            0
        };

        let new_value = current + delta;
        let entry = CacheEntry::new(new_value.to_string().into_bytes(), None);
        store.insert(key.to_string(), entry);

        Ok(new_value)
    }

    async fn delete_many(&self, keys: &[&str]) -> RiptideResult<usize> {
        let mut store = self.store.write().await;
        let mut count = 0;

        for key in keys {
            if store.remove(*key).is_some() {
                count += 1;
            }
        }

        Ok(count)
    }

    async fn clear_pattern(&self, pattern: &str) -> RiptideResult<usize> {
        // Simple glob pattern matching: "prefix:*" style
        let mut store = self.store.write().await;
        let prefix = pattern.trim_end_matches('*');

        let keys_to_remove: Vec<String> = store
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();

        let count = keys_to_remove.len();
        for key in keys_to_remove {
            store.remove(&key);
        }

        Ok(count)
    }

    async fn stats(&self) -> RiptideResult<CacheStats> {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total_keys = self.len().await;
        let memory_usage = self.calculate_memory_usage().await;

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
            .insert("backend".to_string(), "in-memory".to_string());

        Ok(stats)
    }

    async fn health_check(&self) -> RiptideResult<bool> {
        // In-memory cache is always healthy if we can access it
        const HEALTH_KEY: &str = "__health_check__";
        self.set(HEALTH_KEY, b"ok", Some(Duration::from_secs(1)))
            .await?;
        let exists = self.exists(HEALTH_KEY).await?;
        self.delete(HEALTH_KEY).await?;
        Ok(exists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_operations() {
        let cache = InMemoryCache::new();

        // Set and get
        cache.set("key1", b"value1", None).await.unwrap();
        let result = cache.get("key1").await.unwrap();
        assert_eq!(result, Some(b"value1".to_vec()));

        // Exists
        assert!(cache.exists("key1").await.unwrap());
        assert!(!cache.exists("nonexistent").await.unwrap());

        // Delete
        cache.delete("key1").await.unwrap();
        assert!(!cache.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let cache = InMemoryCache::new();

        // Set with short TTL
        cache
            .set("key1", b"value1", Some(Duration::from_millis(100)))
            .await
            .unwrap();

        // Should exist immediately
        assert!(cache.exists("key1").await.unwrap());

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be expired
        assert!(!cache.exists("key1").await.unwrap());
        assert_eq!(cache.get("key1").await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_increment() {
        let cache = InMemoryCache::new();

        // Increment from zero
        let val = cache.incr("counter", 1).await.unwrap();
        assert_eq!(val, 1);

        // Increment again
        let val = cache.incr("counter", 5).await.unwrap();
        assert_eq!(val, 6);

        // Decrement
        let val = cache.incr("counter", -3).await.unwrap();
        assert_eq!(val, 3);
    }

    #[tokio::test]
    async fn test_batch_operations() {
        let cache = InMemoryCache::new();

        // Multi-set
        let items = vec![("key1", b"val1" as &[u8]), ("key2", b"val2")];
        cache.mset(items, None).await.unwrap();

        // Multi-get
        let results = cache.mget(&["key1", "key2", "key3"]).await.unwrap();
        assert_eq!(results[0], Some(b"val1".to_vec()));
        assert_eq!(results[1], Some(b"val2".to_vec()));
        assert_eq!(results[2], None);
    }

    #[tokio::test]
    async fn test_statistics() {
        let cache = InMemoryCache::new();

        cache.set("key1", b"value1", None).await.unwrap();

        // Generate some hits and misses
        cache.get("key1").await.unwrap(); // hit
        cache.get("key2").await.unwrap(); // miss
        cache.get("key1").await.unwrap(); // hit

        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert!(stats.hit_rate.unwrap() > 0.6);
    }
}
