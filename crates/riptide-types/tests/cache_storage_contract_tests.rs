//! Integration tests for cache_storage_contract module
//!
//! These tests validate that the contract test suite itself works correctly
//! by running it against a simple in-memory implementation.

mod contracts;

use contracts::cache_storage_contract;
use riptide_types::error::Result as RiptideResult;
use riptide_types::ports::CacheStorage;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Simple in-memory cache for testing the contract tests themselves
struct MemoryCache {
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MemoryCache {
    fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl CacheStorage for MemoryCache {
    async fn get(&self, key: &str) -> RiptideResult<Option<Vec<u8>>> {
        let data = self.data.read().await;
        Ok(data.get(key).cloned())
    }

    async fn set(&self, key: &str, value: &[u8], _ttl: Option<Duration>) -> RiptideResult<()> {
        let mut data = self.data.write().await;
        data.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    async fn delete(&self, key: &str) -> RiptideResult<()> {
        let mut data = self.data.write().await;
        data.remove(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> RiptideResult<bool> {
        let data = self.data.read().await;
        Ok(data.contains_key(key))
    }
}

#[tokio::test]
async fn test_memory_cache_basic_operations() {
    let cache = MemoryCache::new();
    cache_storage_contract::test_basic_operations(&cache)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_memory_cache_exists() {
    let cache = MemoryCache::new();
    cache_storage_contract::test_exists(&cache).await.unwrap();
}

#[tokio::test]
async fn test_memory_cache_batch_operations() {
    let cache = MemoryCache::new();
    cache_storage_contract::test_batch_operations(&cache)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_memory_cache_delete_many() {
    let cache = MemoryCache::new();
    cache_storage_contract::test_delete_many(&cache)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_memory_cache_health_check() {
    let cache = MemoryCache::new();
    cache_storage_contract::test_health_check(&cache)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_memory_cache_large_values() {
    let cache = MemoryCache::new();
    cache_storage_contract::test_large_values(&cache)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_memory_cache_binary_data() {
    let cache = MemoryCache::new();
    cache_storage_contract::test_binary_data(&cache)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_memory_cache_empty_values() {
    let cache = MemoryCache::new();
    cache_storage_contract::test_empty_values(&cache)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_memory_cache_concurrent_operations() {
    let cache = MemoryCache::new();
    cache_storage_contract::test_concurrent_operations(&cache)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_memory_cache_all_contracts() {
    let cache = MemoryCache::new();
    // Run subset of tests that don't require TTL support
    cache_storage_contract::test_basic_operations(&cache)
        .await
        .unwrap();
    cache_storage_contract::test_exists(&cache).await.unwrap();
    cache_storage_contract::test_batch_operations(&cache)
        .await
        .unwrap();
    cache_storage_contract::test_delete_many(&cache)
        .await
        .unwrap();
    cache_storage_contract::test_health_check(&cache)
        .await
        .unwrap();
    cache_storage_contract::test_large_values(&cache)
        .await
        .unwrap();
    cache_storage_contract::test_binary_data(&cache)
        .await
        .unwrap();
    cache_storage_contract::test_empty_values(&cache)
        .await
        .unwrap();
}
