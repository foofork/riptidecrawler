//! Redis Integration Tests with Testcontainers
//!
//! These tests use testcontainers to spin up isolated Redis instances for testing.
//! Run with: `cargo test -p riptide-persistence --test redis_testcontainer_integration`

use anyhow::Result;
use std::collections::HashMap;
use std::time::Duration;
use testcontainers::clients::Cli;
use tokio::time::sleep;

use riptide_persistence::{
    cache::{CacheMetadata, PersistentCacheManager},
    config::{CacheConfig, CompressionAlgorithm, EvictionPolicy},
};

// Import test helpers
mod helpers;
use helpers::RedisTestContainer;

// Helper function to create test config
fn test_cache_config() -> CacheConfig {
    CacheConfig {
        key_prefix: "test".to_string(),
        version: "v1".to_string(),
        default_ttl_seconds: 3600,
        max_entry_size_bytes: 10_000_000,
        enable_compression: false,
        compression_threshold_bytes: 1024,
        compression_algorithm: CompressionAlgorithm::None,
        enable_warming: false,
        warming_batch_size: 100,
        max_memory_bytes: None,
        eviction_policy: EvictionPolicy::LRU,
    }
}

#[tokio::test]
async fn test_redis_connection_with_testcontainer() -> Result<()> {
    let docker = Cli::default();
    let redis_container = RedisTestContainer::new(&docker).await?;
    let config = test_cache_config();

    let cache = PersistentCacheManager::new(redis_container.get_connection_string(), config).await;
    assert!(cache.is_ok(), "Should connect to testcontainer Redis");

    Ok(())
}

#[tokio::test]
async fn test_cache_set_and_get() -> Result<()> {
    let docker = Cli::default();
    let redis_container = RedisTestContainer::new(&docker).await?;
    let config = test_cache_config();
    let cache =
        PersistentCacheManager::new(redis_container.get_connection_string(), config).await?;

    // Set a value
    cache
        .set("test_key", &"test_value", None, None, None)
        .await?;

    // Get the value back
    let value: Option<String> = cache.get("test_key", None).await?;
    assert_eq!(value, Some("test_value".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_cache_delete() -> Result<()> {
    let docker = Cli::default();
    let redis_container = RedisTestContainer::new(&docker).await?;
    let config = test_cache_config();
    let cache =
        PersistentCacheManager::new(redis_container.get_connection_string(), config).await?;

    cache.set("delete_test", &"value", None, None, None).await?;
    cache.delete("delete_test", None).await?;

    let value: Option<String> = cache.get("delete_test", None).await?;
    assert_eq!(value, None);

    Ok(())
}

#[tokio::test]
async fn test_ttl_expiration() -> Result<()> {
    let docker = Cli::default();
    let redis_container = RedisTestContainer::new(&docker).await?;
    let config = test_cache_config();
    let cache =
        PersistentCacheManager::new(redis_container.get_connection_string(), config).await?;

    cache
        .set(
            "ttl_test",
            &"value",
            None,
            Some(Duration::from_secs(1)),
            None,
        )
        .await?;

    let value1: Option<String> = cache.get("ttl_test", None).await?;
    assert!(value1.is_some());

    sleep(Duration::from_secs(2)).await;

    let value2: Option<String> = cache.get("ttl_test", None).await?;
    assert!(value2.is_none());

    Ok(())
}

#[tokio::test]
async fn test_multi_tenant_isolation() -> Result<()> {
    let docker = Cli::default();
    let redis_container = RedisTestContainer::new(&docker).await?;
    let config = test_cache_config();
    let cache =
        PersistentCacheManager::new(redis_container.get_connection_string(), config).await?;

    cache
        .set("shared_key", &"value1", Some("tenant1"), None, None)
        .await?;
    cache
        .set("shared_key", &"value2", Some("tenant2"), None, None)
        .await?;

    let value1: Option<String> = cache.get("shared_key", Some("tenant1")).await?;
    let value2: Option<String> = cache.get("shared_key", Some("tenant2")).await?;

    assert_eq!(value1, Some("value1".to_string()));
    assert_eq!(value2, Some("value2".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_batch_operations() -> Result<()> {
    let docker = Cli::default();
    let redis_container = RedisTestContainer::new(&docker).await?;
    let config = test_cache_config();
    let cache =
        PersistentCacheManager::new(redis_container.get_connection_string(), config).await?;

    let mut entries = HashMap::new();
    entries.insert("batch1".to_string(), "value1");
    entries.insert("batch2".to_string(), "value2");
    entries.insert("batch3".to_string(), "value3");

    cache.set_batch(entries, None, None).await?;

    let value1: Option<String> = cache.get("batch1", None).await?;
    let value2: Option<String> = cache.get("batch2", None).await?;
    let value3: Option<String> = cache.get("batch3", None).await?;

    assert_eq!(value1, Some("value1".to_string()));
    assert_eq!(value2, Some("value2".to_string()));
    assert_eq!(value3, Some("value3".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_batch_get() -> Result<()> {
    let docker = Cli::default();
    let redis_container = RedisTestContainer::new(&docker).await?;
    let config = test_cache_config();
    let cache =
        PersistentCacheManager::new(redis_container.get_connection_string(), config).await?;

    cache.set("batch_get1", &"value1", None, None, None).await?;
    cache.set("batch_get2", &"value2", None, None, None).await?;
    cache.set("batch_get3", &"value3", None, None, None).await?;

    let keys = vec![
        "batch_get1".to_string(),
        "batch_get2".to_string(),
        "batch_get3".to_string(),
    ];

    let results: HashMap<String, String> = cache.get_batch(&keys, None).await?;

    assert_eq!(results.len(), 3);
    assert_eq!(results.get("batch_get1"), Some(&"value1".to_string()));
    assert_eq!(results.get("batch_get2"), Some(&"value2".to_string()));
    assert_eq!(results.get("batch_get3"), Some(&"value3".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_cache_clear() -> Result<()> {
    let docker = Cli::default();
    let redis_container = RedisTestContainer::new(&docker).await?;
    let config = test_cache_config();
    let cache =
        PersistentCacheManager::new(redis_container.get_connection_string(), config).await?;

    cache
        .set("clear_test1", &"value1", None, None, None)
        .await?;
    cache
        .set("clear_test2", &"value2", None, None, None)
        .await?;

    let deleted = cache.clear().await?;
    assert!(deleted >= 2);

    let value1: Option<String> = cache.get("clear_test1", None).await?;
    let value2: Option<String> = cache.get("clear_test2", None).await?;

    assert!(value1.is_none());
    assert!(value2.is_none());

    Ok(())
}

#[tokio::test]
async fn test_large_value_storage() -> Result<()> {
    let docker = Cli::default();
    let redis_container = RedisTestContainer::new(&docker).await?;
    let config = test_cache_config();
    let cache =
        PersistentCacheManager::new(redis_container.get_connection_string(), config).await?;

    let large_value = "x".repeat(10_000);
    cache
        .set("large_value_test", &large_value, None, None, None)
        .await?;

    let retrieved: Option<String> = cache.get("large_value_test", None).await?;
    assert_eq!(retrieved, Some(large_value));

    Ok(())
}

#[tokio::test]
async fn test_metadata_support() -> Result<()> {
    let docker = Cli::default();
    let redis_container = RedisTestContainer::new(&docker).await?;
    let config = test_cache_config();
    let cache =
        PersistentCacheManager::new(redis_container.get_connection_string(), config).await?;

    let mut metadata = CacheMetadata {
        version: "v2".to_string(),
        content_type: Some("application/json".to_string()),
        source: Some("test".to_string()),
        tags: vec!["important".to_string()],
        attributes: HashMap::new(),
    };
    metadata
        .attributes
        .insert("test_key".to_string(), "test_value".to_string());

    cache
        .set(
            "metadata_test",
            &"value_with_metadata",
            None,
            None,
            Some(metadata),
        )
        .await?;

    let value: Option<String> = cache.get("metadata_test", None).await?;
    assert_eq!(value, Some("value_with_metadata".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_concurrent_operations() -> Result<()> {
    let docker = Cli::default();
    let redis_container = RedisTestContainer::new(&docker).await?;
    let config = test_cache_config();
    let cache = std::sync::Arc::new(
        PersistentCacheManager::new(redis_container.get_connection_string(), config).await?,
    );

    let mut handles = vec![];
    for i in 0..10 {
        let cache_clone = cache.clone();
        let handle = tokio::spawn(async move {
            cache_clone
                .set(
                    &format!("concurrent_{}", i),
                    &format!("value_{}", i),
                    None,
                    None,
                    None,
                )
                .await
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await??;
    }

    Ok(())
}

#[tokio::test]
async fn test_performance_rapid_operations() -> Result<()> {
    let docker = Cli::default();
    let redis_container = RedisTestContainer::new(&docker).await?;
    let config = test_cache_config();
    let cache =
        PersistentCacheManager::new(redis_container.get_connection_string(), config).await?;

    let start = std::time::Instant::now();

    for i in 0..100 {
        cache
            .set(
                &format!("perf_{}", i),
                &format!("value_{}", i),
                None,
                None,
                None,
            )
            .await?;
    }

    let duration = start.elapsed();

    // 100 operations should complete in < 2 seconds (generous for CI)
    assert!(duration < Duration::from_secs(2));

    Ok(())
}

#[tokio::test]
async fn test_connection_failure_handling() -> Result<()> {
    let config = test_cache_config();
    let cache = PersistentCacheManager::new("redis://nonexistent:9999", config).await;
    assert!(cache.is_err(), "Should fail with invalid connection");
    Ok(())
}

#[tokio::test]
async fn test_cache_statistics() -> Result<()> {
    let docker = Cli::default();
    let redis_container = RedisTestContainer::new(&docker).await?;
    let config = test_cache_config();
    let cache =
        PersistentCacheManager::new(redis_container.get_connection_string(), config).await?;

    cache.set("stats_test", &"value", None, None, None).await?;

    let stats = cache.get_stats().await?;
    assert!(stats.total_keys >= 1);

    Ok(())
}
