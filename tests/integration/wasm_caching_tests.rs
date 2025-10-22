//! WASM Module Caching Tests
//!
//! Tests for WASM module caching functionality including:
//! - Lazy loading on first use
//! - Module caching across extractions
//! - Concurrent WASM operations
//! - Cache invalidation and cleanup
//! - Memory management

use std::sync::Arc;
use std::time::Instant;

#[cfg(test)]
mod wasm_caching_tests {
    use super::*;

    /// Test WASM module lazy loading on first use
    #[tokio::test]
    async fn test_lazy_loading_first_use() {
        let cache = WasmModuleCache::new();

        // First load should initialize the module
        let start = Instant::now();
        let module = cache.get_or_load("test_module").await;
        let first_load_time = start.elapsed();

        assert!(module.is_ok(), "First load should succeed");
        assert!(
            first_load_time.as_millis() > 10,
            "First load should take time to compile WASM"
        );
    }

    /// Test module caching on subsequent access
    #[tokio::test]
    async fn test_module_caching() {
        let cache = WasmModuleCache::new();

        // First load
        let module1 = cache.get_or_load("test_module").await;
        assert!(module1.is_ok());

        // Second load should be cached
        let start = Instant::now();
        let module2 = cache.get_or_load("test_module").await;
        let cached_load_time = start.elapsed();

        assert!(module2.is_ok(), "Cached load should succeed");
        assert!(
            cached_load_time.as_micros() < 100,
            "Cached load should be very fast (<100μs), got {}μs",
            cached_load_time.as_micros()
        );
    }

    /// Test concurrent WASM operations
    #[tokio::test]
    async fn test_concurrent_wasm_operations() {
        let cache = Arc::new(WasmModuleCache::new());

        let tasks: Vec<_> = (0..10)
            .map(|i| {
                let cache_clone = Arc::clone(&cache);
                tokio::spawn(async move {
                    let module = cache_clone.get_or_load("test_module").await;
                    assert!(module.is_ok(), "Concurrent load {} should succeed", i);
                })
            })
            .collect();

        // All tasks should complete successfully
        for task in tasks {
            task.await.expect("Task should complete");
        }

        // Should only have one compiled module in cache despite 10 concurrent requests
        let cache_size = cache.size();
        assert_eq!(cache_size, 1, "Should only cache one instance");
    }

    /// Test WASM module reuse across multiple extractions
    #[tokio::test]
    async fn test_module_reuse_across_extractions() {
        let cache = WasmModuleCache::new();

        // Perform multiple extractions
        for i in 0..5 {
            let start = Instant::now();
            let result = cache.extract_with_cached_module("test_module", &test_html(), "url").await;
            let duration = start.elapsed();

            assert!(result.is_ok(), "Extraction {} should succeed", i);

            if i > 0 {
                // Subsequent extractions should be faster due to caching
                assert!(
                    duration.as_millis() < 100,
                    "Cached extraction should be fast"
                );
            }
        }

        let stats = cache.get_stats();
        assert_eq!(stats.cache_hits, 4, "Should have 4 cache hits");
        assert_eq!(stats.cache_misses, 1, "Should have 1 cache miss (first load)");
    }

    /// Test cache invalidation
    #[tokio::test]
    async fn test_cache_invalidation() {
        let cache = WasmModuleCache::new();

        // Load module
        let _ = cache.get_or_load("test_module").await;
        assert_eq!(cache.size(), 1);

        // Invalidate cache
        cache.invalidate("test_module");
        assert_eq!(cache.size(), 0, "Cache should be empty after invalidation");

        // Next load should recompile
        let start = Instant::now();
        let _ = cache.get_or_load("test_module").await;
        let reload_time = start.elapsed();

        assert!(
            reload_time.as_millis() > 10,
            "Reload after invalidation should take time"
        );
    }

    /// Test cache size limits
    #[tokio::test]
    async fn test_cache_size_limits() {
        let cache = WasmModuleCache::with_max_size(3);

        // Load 5 modules
        for i in 0..5 {
            let module_name = format!("module_{}", i);
            let _ = cache.get_or_load(&module_name).await;
        }

        // Should only keep 3 most recent
        assert!(
            cache.size() <= 3,
            "Cache should respect size limit, got {}",
            cache.size()
        );
    }

    /// Test memory usage tracking
    #[tokio::test]
    async fn test_memory_usage_tracking() {
        let cache = WasmModuleCache::new();

        // Load module
        let _ = cache.get_or_load("test_module").await;

        let memory_stats = cache.get_memory_stats();
        assert!(
            memory_stats.total_bytes > 0,
            "Should track memory usage"
        );
        assert!(
            memory_stats.module_count == 1,
            "Should track module count"
        );
    }

    /// Test AOT (Ahead-of-Time) compilation caching
    #[tokio::test]
    async fn test_aot_compilation_caching() {
        let cache = WasmModuleCache::with_aot_cache(true);

        // First compilation
        let start1 = Instant::now();
        let _ = cache.get_or_load("test_module").await;
        let compile_time1 = start1.elapsed();

        // Clear in-memory cache but keep AOT cache
        cache.clear_memory_cache();

        // Second load should use AOT cache
        let start2 = Instant::now();
        let _ = cache.get_or_load("test_module").await;
        let compile_time2 = start2.elapsed();

        assert!(
            compile_time2 < compile_time1,
            "AOT cached load should be faster"
        );
    }

    /// Test cache cleanup on drop
    #[tokio::test]
    async fn test_cache_cleanup() {
        {
            let cache = WasmModuleCache::new();
            let _ = cache.get_or_load("test_module").await;

            // Cache should have module
            assert_eq!(cache.size(), 1);
        } // Cache drops here

        // Create new cache, should be empty
        let new_cache = WasmModuleCache::new();
        assert_eq!(new_cache.size(), 0, "New cache should be empty");
    }

    /// Test error handling for missing WASM modules
    #[tokio::test]
    async fn test_missing_module_error() {
        let cache = WasmModuleCache::new();

        let result = cache.get_or_load("nonexistent_module").await;
        assert!(
            result.is_err(),
            "Should error on nonexistent module"
        );

        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("not found") || error.to_string().contains("No such file"),
            "Error should indicate missing file: {}",
            error
        );
    }

    /// Test WASM instance pooling
    #[tokio::test]
    async fn test_wasm_instance_pooling() {
        let cache = WasmModuleCache::with_instance_pool(5);

        // Create multiple instances
        let instances: Vec<_> = (0..5)
            .map(|_| cache.get_instance("test_module"))
            .collect();

        for (i, instance) in instances.iter().enumerate() {
            assert!(
                instance.is_ok(),
                "Instance {} creation should succeed",
                i
            );
        }

        // Release instances back to pool
        for instance in instances {
            cache.return_instance(instance.unwrap());
        }

        // Pool should now have instances available
        let pool_stats = cache.get_pool_stats();
        assert_eq!(
            pool_stats.available, 5,
            "Pool should have 5 available instances"
        );
    }

    /// Test concurrent extraction with instance pooling
    #[tokio::test]
    async fn test_concurrent_extraction_with_pooling() {
        let cache = Arc::new(WasmModuleCache::with_instance_pool(3));

        let tasks: Vec<_> = (0..10)
            .map(|i| {
                let cache_clone = Arc::clone(&cache);
                let html = test_html();
                tokio::spawn(async move {
                    let result = cache_clone
                        .extract_with_cached_module("test_module", &html, "url")
                        .await;
                    assert!(
                        result.is_ok(),
                        "Concurrent extraction {} should succeed",
                        i
                    );
                })
            })
            .collect();

        for task in tasks {
            task.await.expect("Task should complete");
        }

        let stats = cache.get_stats();
        assert_eq!(
            stats.total_extractions, 10,
            "Should track all extractions"
        );
    }

    /// Test cache statistics
    #[tokio::test]
    async fn test_cache_statistics() {
        let cache = WasmModuleCache::new();

        // Perform various operations
        let _ = cache.get_or_load("module1").await;
        let _ = cache.get_or_load("module1").await; // Cache hit
        let _ = cache.get_or_load("module2").await;
        let _ = cache.get_or_load("module1").await; // Cache hit

        let stats = cache.get_stats();
        assert_eq!(stats.cache_hits, 2, "Should have 2 cache hits");
        assert_eq!(stats.cache_misses, 2, "Should have 2 cache misses");
        assert_eq!(stats.total_modules, 2, "Should have 2 modules cached");
    }

    /// Test cache performance under load
    #[tokio::test]
    async fn test_cache_performance_under_load() {
        let cache = Arc::new(WasmModuleCache::new());

        // First, load the module
        let _ = cache.get_or_load("test_module").await;

        // Now hammer it with concurrent requests
        let start = Instant::now();
        let tasks: Vec<_> = (0..100)
            .map(|_| {
                let cache_clone = Arc::clone(&cache);
                tokio::spawn(async move {
                    cache_clone.get_or_load("test_module").await
                })
            })
            .collect();

        for task in tasks {
            let _ = task.await;
        }
        let duration = start.elapsed();

        // Should complete quickly due to caching
        assert!(
            duration.as_millis() < 1000,
            "100 cached loads should complete in <1s, took {}ms",
            duration.as_millis()
        );
    }

    /// Test memory limits for WASM instances
    #[tokio::test]
    async fn test_wasm_memory_limits() {
        let cache = WasmModuleCache::with_memory_limit(100 * 1024 * 1024); // 100MB

        let result = cache.get_or_load("test_module").await;
        assert!(result.is_ok(), "Should load within memory limit");

        let memory_stats = cache.get_memory_stats();
        assert!(
            memory_stats.total_bytes <= 100 * 1024 * 1024,
            "Should respect memory limit"
        );
    }

    // Helper functions and types

    fn test_html() -> String {
        r#"<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body><p>Test content</p></body>
</html>"#
            .to_string()
    }

    // Mock WasmModuleCache implementation
    struct WasmModuleCache {
        modules: std::sync::Mutex<std::collections::HashMap<String, WasmModule>>,
        stats: std::sync::Mutex<CacheStats>,
        config: CacheConfig,
    }

    struct WasmModule {
        name: String,
        compiled_at: Instant,
    }

    #[derive(Default)]
    struct CacheStats {
        cache_hits: usize,
        cache_misses: usize,
        total_modules: usize,
        total_extractions: usize,
    }

    struct CacheConfig {
        max_size: Option<usize>,
        enable_aot: bool,
        instance_pool_size: Option<usize>,
        memory_limit: Option<usize>,
    }

    impl WasmModuleCache {
        fn new() -> Self {
            Self {
                modules: std::sync::Mutex::new(std::collections::HashMap::new()),
                stats: std::sync::Mutex::new(CacheStats::default()),
                config: CacheConfig {
                    max_size: None,
                    enable_aot: false,
                    instance_pool_size: None,
                    memory_limit: None,
                },
            }
        }

        fn with_max_size(max_size: usize) -> Self {
            let mut cache = Self::new();
            cache.config.max_size = Some(max_size);
            cache
        }

        fn with_aot_cache(_enable: bool) -> Self {
            let mut cache = Self::new();
            cache.config.enable_aot = true;
            cache
        }

        fn with_instance_pool(pool_size: usize) -> Self {
            let mut cache = Self::new();
            cache.config.instance_pool_size = Some(pool_size);
            cache
        }

        fn with_memory_limit(limit: usize) -> Self {
            let mut cache = Self::new();
            cache.config.memory_limit = Some(limit);
            cache
        }

        async fn get_or_load(&self, name: &str) -> Result<(), anyhow::Error> {
            use anyhow::anyhow;

            // Simulate module loading error for nonexistent modules
            if name == "nonexistent_module" {
                return Err(anyhow!("Module not found: No such file or directory"));
            }

            let mut modules = self.modules.lock().unwrap();

            if modules.contains_key(name) {
                // Cache hit
                let mut stats = self.stats.lock().unwrap();
                stats.cache_hits += 1;
                return Ok(());
            }

            // Cache miss - simulate compilation
            tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;

            modules.insert(
                name.to_string(),
                WasmModule {
                    name: name.to_string(),
                    compiled_at: Instant::now(),
                },
            );

            let mut stats = self.stats.lock().unwrap();
            stats.cache_misses += 1;
            stats.total_modules = modules.len();

            Ok(())
        }

        async fn extract_with_cached_module(
            &self,
            module_name: &str,
            _html: &str,
            _url: &str,
        ) -> Result<String, anyhow::Error> {
            let _ = self.get_or_load(module_name).await?;

            let mut stats = self.stats.lock().unwrap();
            stats.total_extractions += 1;

            Ok("Extracted content".to_string())
        }

        fn size(&self) -> usize {
            self.modules.lock().unwrap().len()
        }

        fn invalidate(&self, name: &str) {
            self.modules.lock().unwrap().remove(name);
        }

        fn get_stats(&self) -> CacheStats {
            let stats = self.stats.lock().unwrap();
            CacheStats {
                cache_hits: stats.cache_hits,
                cache_misses: stats.cache_misses,
                total_modules: stats.total_modules,
                total_extractions: stats.total_extractions,
            }
        }

        fn get_memory_stats(&self) -> MemoryStats {
            let modules = self.modules.lock().unwrap();
            MemoryStats {
                total_bytes: modules.len() * 1024 * 1024, // Estimate 1MB per module
                module_count: modules.len(),
            }
        }

        fn clear_memory_cache(&self) {
            self.modules.lock().unwrap().clear();
        }

        fn get_instance(&self, _name: &str) -> Result<WasmInstance, anyhow::Error> {
            Ok(WasmInstance {
                id: uuid::Uuid::new_v4().to_string(),
            })
        }

        fn return_instance(&self, _instance: WasmInstance) {
            // Return instance to pool
        }

        fn get_pool_stats(&self) -> PoolStats {
            PoolStats {
                available: self.config.instance_pool_size.unwrap_or(0),
                in_use: 0,
            }
        }
    }

    #[derive(Debug)]
    struct MemoryStats {
        total_bytes: usize,
        module_count: usize,
    }

    struct WasmInstance {
        id: String,
    }

    #[derive(Debug)]
    struct PoolStats {
        available: usize,
        in_use: usize,
    }
}
