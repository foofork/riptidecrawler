//! Integration tests for cache management
//!
//! These tests verify the cache system functionality including:
//! - Cache entry management
//! - LRU eviction
//! - Domain-based operations
//! - Statistics tracking
//! - Persistence

use riptide_cli::cache::{Cache, CacheConfig, CacheEntry, WarmOptions};
use tempfile::TempDir;

#[tokio::test]
async fn test_cache_basic_operations() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = CacheConfig::default();
    config.cache_dir = temp_dir.path().to_string_lossy().to_string();
    config.persistent = false;

    let cache = Cache::with_config(config).await.unwrap();

    // Test insert and get
    let entry = CacheEntry::new(
        "https://example.com/page".to_string(),
        "<html>test content</html>".to_string(),
        "text/html".to_string(),
    );

    cache.insert(entry).await.unwrap();

    let retrieved = cache.get("https://example.com/page").await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().content, "<html>test content</html>");

    // Test cache miss
    let missing = cache.get("https://nonexistent.com").await.unwrap();
    assert!(missing.is_none());

    // Check statistics
    let stats = cache.get_stats().await.unwrap();
    assert_eq!(stats.total_entries, 1);
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
}

#[tokio::test]
async fn test_cache_domain_operations() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = CacheConfig::default();
    config.cache_dir = temp_dir.path().to_string_lossy().to_string();
    config.persistent = false;

    let cache = Cache::with_config(config).await.unwrap();

    // Insert entries from different domains
    for i in 1..=3 {
        let entry = CacheEntry::new(
            format!("https://example.com/page{}", i),
            format!("content{}", i),
            "text/html".to_string(),
        );
        cache.insert(entry).await.unwrap();
    }

    for i in 1..=2 {
        let entry = CacheEntry::new(
            format!("https://other.com/page{}", i),
            format!("content{}", i),
            "text/html".to_string(),
        );
        cache.insert(entry).await.unwrap();
    }

    // Verify total entries
    let stats = cache.get_stats().await.unwrap();
    assert_eq!(stats.total_entries, 5);

    // List URLs by domain
    let example_urls = cache.list_domain_urls("example.com").await.unwrap();
    assert_eq!(example_urls.len(), 3);

    let other_urls = cache.list_domain_urls("other.com").await.unwrap();
    assert_eq!(other_urls.len(), 2);

    // Clear one domain
    let cleared = cache.clear_domain("example.com").await.unwrap();
    assert_eq!(cleared, 3);

    // Verify remaining entries
    let stats = cache.get_stats().await.unwrap();
    assert_eq!(stats.total_entries, 2);

    let example_urls = cache.list_domain_urls("example.com").await.unwrap();
    assert_eq!(example_urls.len(), 0);
}

#[tokio::test]
async fn test_cache_lru_eviction() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = CacheConfig::default();
    config.cache_dir = temp_dir.path().to_string_lossy().to_string();
    config.persistent = false;
    config.max_entries = 3;

    let cache = Cache::with_config(config).await.unwrap();

    // Insert 3 entries (max capacity)
    for i in 1..=3 {
        let entry = CacheEntry::new(
            format!("https://example.com/{}", i),
            format!("content{}", i),
            "text/html".to_string(),
        );
        cache.insert(entry).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // Access entry 1 to make it recently used
    cache.get("https://example.com/1").await.unwrap();

    // Insert a 4th entry, should evict entry 2 (oldest unused)
    let entry4 = CacheEntry::new(
        "https://example.com/4".to_string(),
        "content4".to_string(),
        "text/html".to_string(),
    );
    cache.insert(entry4).await.unwrap();

    // Entry 1 should still be present (recently accessed)
    let result1 = cache.get("https://example.com/1").await.unwrap();
    assert!(result1.is_some());

    // Entry 2 should be evicted
    let result2 = cache.get("https://example.com/2").await.unwrap();
    assert!(result2.is_none());

    // Entry 4 should be present
    let result4 = cache.get("https://example.com/4").await.unwrap();
    assert!(result4.is_some());
}

#[tokio::test]
async fn test_cache_statistics() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = CacheConfig::default();
    config.cache_dir = temp_dir.path().to_string_lossy().to_string();
    config.persistent = false;

    let cache = Cache::with_config(config).await.unwrap();

    // Insert entries
    for i in 1..=5 {
        let entry = CacheEntry::new(
            format!("https://example.com/{}", i),
            format!("content with data {}", i),
            "text/html".to_string(),
        );
        cache.insert(entry).await.unwrap();
    }

    // Create some hits and misses
    cache.get("https://example.com/1").await.unwrap(); // hit
    cache.get("https://example.com/2").await.unwrap(); // hit
    cache.get("https://nonexistent.com").await.unwrap(); // miss

    let stats = cache.get_stats().await.unwrap();

    assert_eq!(stats.total_entries, 5);
    assert_eq!(stats.hits, 2);
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.insertions, 5);
    assert!(stats.hit_rate() > 0.0);
    assert!(stats.total_size_bytes > 0);

    // Check domain statistics
    assert!(stats.entries_by_domain.contains_key("example.com"));
    assert_eq!(stats.entries_by_domain["example.com"], 5);
}

#[tokio::test]
async fn test_cache_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().to_string_lossy().to_string();

    // Create cache and insert entries
    {
        let mut config = CacheConfig::default();
        config.cache_dir = cache_dir.clone();
        config.persistent = true;

        let cache = Cache::with_config(config).await.unwrap();

        for i in 1..=3 {
            let entry = CacheEntry::new(
                format!("https://example.com/{}", i),
                format!("content{}", i),
                "text/html".to_string(),
            );
            cache.insert(entry).await.unwrap();
        }
    }

    // Create new cache instance and verify entries are loaded
    {
        let mut config = CacheConfig::default();
        config.cache_dir = cache_dir;
        config.persistent = true;

        let cache = Cache::with_config(config).await.unwrap();

        let stats = cache.get_stats().await.unwrap();
        assert_eq!(stats.total_entries, 3);

        let result = cache.get("https://example.com/1").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().content, "content1");
    }
}

#[tokio::test]
async fn test_cache_clear_operations() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = CacheConfig::default();
    config.cache_dir = temp_dir.path().to_string_lossy().to_string();
    config.persistent = false;

    let cache = Cache::with_config(config).await.unwrap();

    // Insert entries
    for i in 1..=5 {
        let entry = CacheEntry::new(
            format!("https://example.com/{}", i),
            format!("content{}", i),
            "text/html".to_string(),
        );
        cache.insert(entry).await.unwrap();
    }

    let stats = cache.get_stats().await.unwrap();
    assert_eq!(stats.total_entries, 5);

    // Clear all
    cache.clear().await.unwrap();

    let stats = cache.get_stats().await.unwrap();
    assert_eq!(stats.total_entries, 0);
}

#[tokio::test]
async fn test_cache_list_urls() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = CacheConfig::default();
    config.cache_dir = temp_dir.path().to_string_lossy().to_string();
    config.persistent = false;

    let cache = Cache::with_config(config).await.unwrap();

    let test_urls = vec![
        "https://example.com/page1",
        "https://example.com/page2",
        "https://other.com/page1",
    ];

    for url in &test_urls {
        let entry = CacheEntry::new(
            url.to_string(),
            "content".to_string(),
            "text/html".to_string(),
        );
        cache.insert(entry).await.unwrap();
    }

    let all_urls = cache.list_urls().await.unwrap();
    assert_eq!(all_urls.len(), 3);

    for url in test_urls {
        assert!(all_urls.contains(&url.to_string()));
    }
}
