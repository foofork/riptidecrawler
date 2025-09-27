/*!
# Cache Integration Tests

Comprehensive tests for the persistent cache layer functionality.
*/

use super::*;
use riptide_persistence::{PersistentCacheManager, CacheWarmer, DistributedCache};
use std::collections::HashMap;

#[tokio::test]
async fn test_cache_basic_operations() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_cache_config();
    let redis_url = get_test_redis_url();

    cleanup_test_data(&redis_url, &config.key_prefix).await?;

    let mut cache_manager = PersistentCacheManager::new(&redis_url, config.clone()).await?;

    // Test set and get
    let test_value = serde_json::json!({
        "name": "test",
        "value": 42,
        "array": [1, 2, 3]
    });

    cache_manager.set("test_key", &test_value, None, None, None).await?;
    let retrieved: Option<serde_json::Value> = cache_manager.get("test_key", None).await?;

    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), test_value);

    // Test delete
    let deleted = cache_manager.delete("test_key", None).await?;
    assert!(deleted);

    let after_delete: Option<serde_json::Value> = cache_manager.get("test_key", None).await?;
    assert!(after_delete.is_none());

    cleanup_test_data(&redis_url, &config.key_prefix).await?;
    Ok(())
}

#[tokio::test]
async fn test_cache_batch_operations() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_cache_config();
    let redis_url = get_test_redis_url();

    cleanup_test_data(&redis_url, &config.key_prefix).await?;

    let cache_manager = PersistentCacheManager::new(&redis_url, config.clone()).await?;

    // Test batch set
    let mut batch_data = HashMap::new();
    for i in 0..5 {
        batch_data.insert(
            format!("batch_key_{}", i),
            serde_json::json!({"value": i, "data": format!("test_{}", i)})
        );
    }

    cache_manager.set_batch(batch_data.clone(), None, None).await?;

    // Test batch get
    let keys: Vec<String> = batch_data.keys().cloned().collect();
    let retrieved: HashMap<String, serde_json::Value> = cache_manager.get_batch(&keys, None).await?;

    assert_eq!(retrieved.len(), batch_data.len());
    for (key, value) in &batch_data {
        assert_eq!(retrieved.get(key).unwrap(), value);
    }

    cleanup_test_data(&redis_url, &config.key_prefix).await?;
    Ok(())
}

#[tokio::test]
async fn test_cache_with_namespace() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_cache_config();
    let redis_url = get_test_redis_url();

    cleanup_test_data(&redis_url, &config.key_prefix).await?;

    let cache_manager = PersistentCacheManager::new(&redis_url, config.clone()).await?;

    let test_value = "namespaced_value";

    // Set in namespace
    cache_manager.set("test_key", &test_value, Some("namespace1"), None, None).await?;

    // Get from same namespace
    let retrieved: Option<String> = cache_manager.get("test_key", Some("namespace1")).await?;
    assert_eq!(retrieved.unwrap(), test_value);

    // Try to get from different namespace (should not exist)
    let not_found: Option<String> = cache_manager.get("test_key", Some("namespace2")).await?;
    assert!(not_found.is_none());

    // Try to get without namespace (should not exist)
    let no_namespace: Option<String> = cache_manager.get("test_key", None).await?;
    assert!(no_namespace.is_none());

    cleanup_test_data(&redis_url, &config.key_prefix).await?;
    Ok(())
}

#[tokio::test]
async fn test_cache_compression() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_cache_config();
    let redis_url = get_test_redis_url();

    cleanup_test_data(&redis_url, &config.key_prefix).await?;

    let cache_manager = PersistentCacheManager::new(&redis_url, config.clone()).await?;

    // Create data larger than compression threshold
    let large_data = "x".repeat(config.compression_threshold_bytes + 100);

    cache_manager.set("compression_test", &large_data, None, None, None).await?;
    let retrieved: Option<String> = cache_manager.get("compression_test", None).await?;

    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), large_data);

    cleanup_test_data(&redis_url, &config.key_prefix).await?;
    Ok(())
}

#[tokio::test]
async fn test_cache_warming() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_cache_config();
    let redis_url = get_test_redis_url();

    cleanup_test_data(&redis_url, &config.key_prefix).await?;

    let mut cache_manager = PersistentCacheManager::new(&redis_url, config.clone()).await?;

    // Pre-populate some data
    for i in 0..10 {
        cache_manager.set(
            &format!("warm_key_{}", i),
            &format!("warm_value_{}", i),
            None,
            None,
            None,
        ).await?;
    }

    // Enable cache warming
    let warmer = Arc::new(CacheWarmer::new(5));
    cache_manager.enable_warming(warmer);

    // Warm cache with existing keys
    let warm_keys: Vec<String> = (0..10).map(|i| format!("warm_key_{}", i)).collect();
    let warmed_count = cache_manager.warm_cache(warm_keys).await?;

    assert_eq!(warmed_count, 10);

    cleanup_test_data(&redis_url, &config.key_prefix).await?;
    Ok(())
}

#[tokio::test]
async fn test_cache_statistics() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_cache_config();
    let redis_url = get_test_redis_url();

    cleanup_test_data(&redis_url, &config.key_prefix).await?;

    let cache_manager = PersistentCacheManager::new(&redis_url, config.clone()).await?;

    // Add some data
    for i in 0..5 {
        cache_manager.set(
            &format!("stats_key_{}", i),
            &format!("stats_value_{}", i),
            None,
            None,
            None,
        ).await?;
    }

    // Trigger some hits and misses
    let _hit: Option<String> = cache_manager.get("stats_key_0", None).await?;
    let _miss: Option<String> = cache_manager.get("nonexistent_key", None).await?;

    let stats = cache_manager.get_stats().await?;
    assert!(stats.total_keys >= 5);
    assert!(stats.memory_usage_bytes > 0);

    cleanup_test_data(&redis_url, &config.key_prefix).await?;
    Ok(())
}

#[tokio::test]
async fn test_cache_clear() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_cache_config();
    let redis_url = get_test_redis_url();

    cleanup_test_data(&redis_url, &config.key_prefix).await?;

    let cache_manager = PersistentCacheManager::new(&redis_url, config.clone()).await?;

    // Add some data
    for i in 0..3 {
        cache_manager.set(
            &format!("clear_key_{}", i),
            &format!("clear_value_{}", i),
            None,
            None,
            None,
        ).await?;
    }

    // Verify data exists
    let stats_before = cache_manager.get_stats().await?;
    assert!(stats_before.total_keys >= 3);

    // Clear cache
    let cleared_count = cache_manager.clear().await?;
    assert!(cleared_count >= 3);

    // Verify data is gone
    let stats_after = cache_manager.get_stats().await?;
    assert_eq!(stats_after.total_keys, 0);

    cleanup_test_data(&redis_url, &config.key_prefix).await?;
    Ok(())
}

#[tokio::test]
async fn test_cache_size_limits() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = create_test_cache_config();
    config.max_entry_size_bytes = 1024; // 1KB limit

    let redis_url = get_test_redis_url();
    cleanup_test_data(&redis_url, &config.key_prefix).await?;

    let cache_manager = PersistentCacheManager::new(&redis_url, config.clone()).await?;

    // Try to store data larger than limit
    let large_data = "x".repeat(2048); // 2KB, exceeds limit

    let result = cache_manager.set("large_key", &large_data, None, None, None).await;
    assert!(result.is_err());

    // Store data within limit
    let small_data = "x".repeat(512); // 512 bytes, within limit
    let result = cache_manager.set("small_key", &small_data, None, None, None).await;
    assert!(result.is_ok());

    cleanup_test_data(&redis_url, &config.key_prefix).await?;
    Ok(())
}

#[tokio::test]
async fn test_distributed_cache_sync() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_cache_config();
    let redis_url = get_test_redis_url();

    cleanup_test_data(&redis_url, &config.key_prefix).await?;

    let mut cache_manager = PersistentCacheManager::new(&redis_url, config.clone()).await?;

    // Enable distributed sync
    let node_id = Uuid::new_v4().to_string();
    let sync_manager = Arc::new(DistributedCache::new(node_id));
    cache_manager.enable_sync(sync_manager);

    // Test operations with sync enabled
    cache_manager.set("sync_test", &"sync_value", None, None, None).await?;
    let retrieved: Option<String> = cache_manager.get("sync_test", None).await?;
    assert_eq!(retrieved.unwrap(), "sync_value");

    cleanup_test_data(&redis_url, &config.key_prefix).await?;
    Ok(())
}