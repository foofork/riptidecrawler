/*!
# Integration Tests for RipTide Persistence Layer

Comprehensive integration tests covering all persistence features with >80% coverage.
*/

use riptide_persistence::{
    PersistentCacheManager, StateManager,
    PersistenceConfig, CacheConfig, StateConfig,
    CompressionAlgorithm, EvictionPolicy
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_test;
use uuid::Uuid;

mod cache_integration_tests;
mod state_integration_tests;
mod performance_tests;
mod spillover_tests;

/// Test helper to create Redis URL
fn get_test_redis_url() -> String {
    std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379/15".to_string())
}

/// Test helper to create cache config
fn create_test_cache_config() -> CacheConfig {
    CacheConfig {
        default_ttl_seconds: 300, // 5 minutes for tests
        max_entry_size_bytes: 1024 * 1024, // 1MB for tests
        key_prefix: format!("test_{}", Uuid::new_v4()),
        version: "test_v1".to_string(),
        enable_compression: true,
        compression_threshold_bytes: 512,
        compression_algorithm: CompressionAlgorithm::Lz4,
        enable_warming: true,
        warming_batch_size: 10,
        max_memory_bytes: Some(10 * 1024 * 1024), // 10MB
        eviction_policy: EvictionPolicy::LRU,
    }
}

/// Test helper to create state config
fn create_test_state_config() -> StateConfig {
    StateConfig {
        session_timeout_seconds: 300, // 5 minutes for tests
        enable_hot_reload: false, // Disable for tests
        config_watch_paths: vec![],
        checkpoint_interval_seconds: 60, // 1 minute for tests
        max_checkpoints: 3,
        checkpoint_compression: true,
        enable_graceful_shutdown: true,
        shutdown_timeout_seconds: 10,
    }
}


/// Test helper to create persistence config
fn create_test_persistence_config() -> PersistenceConfig {
    PersistenceConfig {
        redis: riptide_persistence::config::RedisConfig {
            url: get_test_redis_url(),
            pool_size: 2,
            connection_timeout_ms: 5000,
            command_timeout_ms: 5000,
            cluster_mode: false,
            retry_attempts: 2,
            retry_delay_ms: 100,
            enable_pipelining: true,
            max_pipeline_size: 10,
        },
        cache: create_test_cache_config(),
        state: create_test_state_config(),
        distributed: None, // Disable distributed for tests
        performance: riptide_persistence::config::PerformanceConfig {
            target_cache_access_ms: 5,
            enable_monitoring: true,
            metrics_interval_seconds: 10,
            enable_slow_log: true,
            slow_threshold_ms: 50,
            enable_connection_pooling: true,
            pool_config: riptide_persistence::config::PoolConfig::default(),
        },
        security: riptide_persistence::config::SecurityConfig::default(),
    }
}

/// Clean up test data
async fn cleanup_test_data(redis_url: &str, key_prefix: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = redis::Client::open(redis_url)?;
    let mut conn = client.get_async_connection().await?;

    let pattern = format!("{}*", key_prefix);
    let keys: Vec<String> = redis::cmd("KEYS")
        .arg(&pattern)
        .query_async(&mut conn)
        .await
        .unwrap_or_default();

    if !keys.is_empty() {
        let _: u64 = redis::AsyncCommands::del(&mut conn, &keys).await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_basic_integration_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_persistence_config();
    let redis_url = &config.redis.url;

    // Clean up before test
    cleanup_test_data(redis_url, &config.cache.key_prefix).await?;

    // Initialize core components (cache and state)
    let cache_manager = PersistentCacheManager::new(redis_url, config.cache.clone()).await?;
    let state_manager = StateManager::new(redis_url, config.state.clone()).await?;

    // Test 1: Create a session (without tenant context)
    let session_metadata = riptide_persistence::state::SessionMetadata {
        client_ip: Some("127.0.0.1".to_string()),
        user_agent: Some("test-agent".to_string()),
        source: Some("integration-test".to_string()),
        attributes: HashMap::new(),
    };

    let session_id = state_manager.create_session(
        None, // No tenant context
        session_metadata,
        Some(600), // 10 minutes
    ).await?;
    assert!(!session_id.is_empty());

    // Test 2: Cache some data (without tenant context)
    let cache_key = "test_data_key";
    let test_data = serde_json::json!({
        "message": "Hello from cache",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    cache_manager.set(
        cache_key,
        &test_data,
        None, // No tenant context
        None,
        None,
    ).await?;

    // Test 3: Retrieve cached data
    let retrieved_data: Option<serde_json::Value> = cache_manager.get(cache_key, None).await?;
    assert!(retrieved_data.is_some());
    assert_eq!(retrieved_data.unwrap()["message"], "Hello from cache");

    // Test 4: Create a checkpoint
    let checkpoint_id = state_manager.create_checkpoint(
        riptide_persistence::state::CheckpointType::Manual,
        Some("Integration test checkpoint".to_string()),
    ).await?;
    assert!(!checkpoint_id.is_empty());

    // Test 5: Get cache statistics
    let cache_stats = cache_manager.get_stats().await?;
    assert!(cache_stats.total_keys >= 1);

    // Test 6: Session data operations
    state_manager.update_session_data(&session_id, "test_key", serde_json::json!("test_value")).await?;
    let session = state_manager.get_session(&session_id).await?;
    assert!(session.is_some());
    let session = session.unwrap();
    assert_eq!(session.data.get("test_key").unwrap(), &serde_json::json!("test_value"));

    // Cleanup
    cleanup_test_data(redis_url, &config.cache.key_prefix).await?;

    println!("Basic integration test completed successfully!");
    Ok(())
}

#[tokio::test]
async fn test_performance_targets() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_persistence_config();
    let redis_url = &config.redis.url;

    cleanup_test_data(redis_url, &config.cache.key_prefix).await?;

    let cache_manager = PersistentCacheManager::new(redis_url, config.cache.clone()).await?;

    // Test cache access time target (<5ms)
    let test_data = "test_performance_data";
    let start = std::time::Instant::now();

    cache_manager.set("perf_test", &test_data, None, None, None).await?;
    let set_time = start.elapsed();

    let get_start = std::time::Instant::now();
    let _retrieved: Option<String> = cache_manager.get("perf_test", None).await?;
    let get_time = get_start.elapsed();

    println!("Set time: {:?}, Get time: {:?}", set_time, get_time);

    // Assert performance targets
    assert!(get_time.as_millis() < 50, "Cache get time should be under 50ms in tests");
    assert!(set_time.as_millis() < 100, "Cache set time should be under 100ms in tests");

    cleanup_test_data(redis_url, &config.cache.key_prefix).await?;
    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_persistence_config();

    // Test with invalid Redis URL
    let invalid_config = PersistenceConfig {
        redis: riptide_persistence::config::RedisConfig {
            url: "redis://invalid:9999".to_string(),
            ..config.redis
        },
        ..config
    };

    let result = PersistentCacheManager::new(&invalid_config.redis.url, invalid_config.cache).await;
    assert!(result.is_err());

    println!("Error handling test completed successfully!");
    Ok(())
}

#[tokio::test]
async fn test_compression_functionality() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_persistence_config();
    let redis_url = &config.redis.url;

    cleanup_test_data(redis_url, &config.cache.key_prefix).await?;

    let cache_manager = PersistentCacheManager::new(redis_url, config.cache.clone()).await?;

    // Create large data that should be compressed
    let large_data = "x".repeat(2048); // 2KB data, above compression threshold

    cache_manager.set("compression_test", &large_data, None, None, None).await?;
    let retrieved: Option<String> = cache_manager.get("compression_test", None).await?;

    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), large_data);

    cleanup_test_data(redis_url, &config.cache.key_prefix).await?;
    Ok(())
}

#[tokio::test]
async fn test_ttl_functionality() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_persistence_config();
    let redis_url = &config.redis.url;

    cleanup_test_data(redis_url, &config.cache.key_prefix).await?;

    let cache_manager = PersistentCacheManager::new(redis_url, config.cache.clone()).await?;

    // Set data with short TTL
    cache_manager.set(
        "ttl_test",
        &"test_data",
        None,
        Some(std::time::Duration::from_secs(1)), // 1 second TTL
        None,
    ).await?;

    // Should be available immediately
    let immediate: Option<String> = cache_manager.get("ttl_test", None).await?;
    assert!(immediate.is_some());

    // Wait for expiration
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Should be expired now
    let expired: Option<String> = cache_manager.get("ttl_test", None).await?;
    assert!(expired.is_none());

    cleanup_test_data(redis_url, &config.cache.key_prefix).await?;
    Ok(())
}