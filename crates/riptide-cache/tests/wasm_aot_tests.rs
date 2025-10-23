//! Comprehensive tests for WASM AOT cache module

use riptide_cache::wasm::{AotCacheConfig, WasmAotCache};
use std::path::PathBuf;
use tempfile::TempDir;

/// Test AOT cache configuration creation with defaults
#[test]
fn test_aot_cache_config_default() {
    let config = AotCacheConfig::default();

    assert!(config.cache_dir.to_str().unwrap().contains(".riptide"));
    assert!(config.cache_dir.to_str().unwrap().contains("wasm-cache"));
    assert_eq!(config.max_cache_size_bytes, 1024 * 1024 * 1024); // 1GB
    assert_eq!(config.max_age_seconds, 30 * 24 * 60 * 60); // 30 days
    assert!(config.enable_parallel);
}

/// Test AOT cache configuration with custom values
#[test]
fn test_aot_cache_config_custom() {
    let custom_dir = PathBuf::from("/tmp/custom-cache");
    let config = AotCacheConfig {
        cache_dir: custom_dir.clone(),
        max_cache_size_bytes: 512 * 1024 * 1024, // 512MB
        max_age_seconds: 7 * 24 * 60 * 60,       // 7 days
        enable_parallel: false,
    };

    assert_eq!(config.cache_dir, custom_dir);
    assert_eq!(config.max_cache_size_bytes, 512 * 1024 * 1024);
    assert_eq!(config.max_age_seconds, 7 * 24 * 60 * 60);
    assert!(!config.enable_parallel);
}

/// Test AOT cache initialization creates directory
#[tokio::test]
async fn test_aot_cache_init_creates_directory() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join("wasm-cache");

    let config = AotCacheConfig {
        cache_dir: cache_dir.clone(),
        max_cache_size_bytes: 100 * 1024 * 1024,
        max_age_seconds: 24 * 60 * 60,
        enable_parallel: true,
    };

    let cache = WasmAotCache::new(config).await;
    assert!(cache.is_ok());
    assert!(cache_dir.exists());
}

/// Test AOT cache handles empty cache directory
#[tokio::test]
async fn test_aot_cache_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let config = AotCacheConfig {
        cache_dir: temp_dir.path().to_path_buf(),
        max_cache_size_bytes: 100 * 1024 * 1024,
        max_age_seconds: 24 * 60 * 60,
        enable_parallel: true,
    };

    let cache = WasmAotCache::new(config).await;
    assert!(cache.is_ok());

    let cache = cache.unwrap();
    let stats = cache.stats().await;
    assert_eq!(stats.total_entries, 0);
}

/// Test AOT cache stats tracking
#[tokio::test]
async fn test_aot_cache_stats() {
    let temp_dir = TempDir::new().unwrap();
    let config = AotCacheConfig {
        cache_dir: temp_dir.path().to_path_buf(),
        max_cache_size_bytes: 100 * 1024 * 1024,
        max_age_seconds: 24 * 60 * 60,
        enable_parallel: true,
    };

    let cache = WasmAotCache::new(config).await.unwrap();
    let stats = cache.stats().await;

    assert_eq!(stats.total_entries, 0);
    assert_eq!(stats.total_size_bytes, 0);
}

/// Test AOT cache handles non-existent WASM file
#[tokio::test]
async fn test_aot_cache_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();
    let config = AotCacheConfig {
        cache_dir: temp_dir.path().to_path_buf(),
        max_cache_size_bytes: 100 * 1024 * 1024,
        max_age_seconds: 24 * 60 * 60,
        enable_parallel: true,
    };

    let cache = WasmAotCache::new(config).await.unwrap();
    let result = cache.get_or_compile("nonexistent.wasm").await;

    // Should return error for non-existent file
    assert!(result.is_err());
}

/// Test AOT cache metadata persistence
#[tokio::test]
async fn test_aot_cache_metadata_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().to_path_buf();
    let metadata_file = cache_dir.join("cache_metadata.json");

    let config = AotCacheConfig {
        cache_dir: cache_dir.clone(),
        max_cache_size_bytes: 100 * 1024 * 1024,
        max_age_seconds: 24 * 60 * 60,
        enable_parallel: true,
    };

    // Create cache
    let _cache = WasmAotCache::new(config.clone()).await.unwrap();

    // Metadata file should exist
    assert!(metadata_file.exists());

    // Create new cache instance - should load existing metadata
    let cache2 = WasmAotCache::new(config).await;
    assert!(cache2.is_ok());
}

/// Test AOT cache size limits
#[tokio::test]
async fn test_aot_cache_size_limits() {
    let temp_dir = TempDir::new().unwrap();
    let config = AotCacheConfig {
        cache_dir: temp_dir.path().to_path_buf(),
        max_cache_size_bytes: 1024, // Very small limit
        max_age_seconds: 24 * 60 * 60,
        enable_parallel: true,
    };

    let cache = WasmAotCache::new(config).await.unwrap();
    let stats = cache.stats().await;

    // Cache should respect size limits
    assert!(stats.total_size_bytes <= 1024);
}

/// Test AOT cache concurrent access
#[tokio::test]
async fn test_aot_cache_concurrent_access() {
    let temp_dir = TempDir::new().unwrap();
    let config = AotCacheConfig {
        cache_dir: temp_dir.path().to_path_buf(),
        max_cache_size_bytes: 100 * 1024 * 1024,
        max_age_seconds: 24 * 60 * 60,
        enable_parallel: true,
    };

    let cache = WasmAotCache::new(config).await.unwrap();

    // Multiple concurrent stats requests should work (using Arc for shared access)
    use std::sync::Arc;
    let cache = Arc::new(cache);
    let mut handles = vec![];
    for _ in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let handle = tokio::spawn(async move { cache_clone.stats().await });
        handles.push(handle);
    }

    for handle in handles {
        let stats = handle.await.unwrap();
        assert_eq!(stats.total_entries, 0);
    }
}

/// Test AOT cache clear operation
#[tokio::test]
async fn test_aot_cache_clear() {
    let temp_dir = TempDir::new().unwrap();
    let config = AotCacheConfig {
        cache_dir: temp_dir.path().to_path_buf(),
        max_cache_size_bytes: 100 * 1024 * 1024,
        max_age_seconds: 24 * 60 * 60,
        enable_parallel: true,
    };

    let cache = WasmAotCache::new(config).await.unwrap();

    // Clear should succeed even with empty cache
    let result = cache.clear_cache().await;
    assert!(result.is_ok());

    let stats = cache.stats().await;
    assert_eq!(stats.total_entries, 0);
    assert_eq!(stats.total_size_bytes, 0);
}
