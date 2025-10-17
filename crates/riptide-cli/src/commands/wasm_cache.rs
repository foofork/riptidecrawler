/// WASM Module Caching with Lazy Loading
///
/// This module provides efficient WASM module caching to avoid repeated
/// loading and initialization overhead.
use anyhow::{anyhow, Result};
use once_cell::sync::OnceCell;
use riptide_extraction::wasm_extraction::WasmExtractor;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Global WASM module cache
static WASM_CACHE: OnceCell<WasmModuleCache> = OnceCell::new();

/// Cached WASM extractor with metadata
#[derive(Clone)]
pub struct CachedWasmModule {
    pub extractor: Arc<WasmExtractor>,
    pub loaded_at: Instant,
    pub path: String,
    pub use_count: Arc<RwLock<u64>>,
}

/// WASM module cache manager
pub struct WasmModuleCache {
    module: Arc<RwLock<Option<CachedWasmModule>>>,
    init_timeout: Duration,
}

impl WasmModuleCache {
    /// Create a new WASM module cache
    pub fn new(init_timeout: Duration) -> Self {
        Self {
            module: Arc::new(RwLock::new(None)),
            init_timeout,
        }
    }

    /// Get or initialize the global cache instance
    pub fn global() -> &'static WasmModuleCache {
        WASM_CACHE.get_or_init(|| WasmModuleCache::new(Duration::from_secs(10)))
    }

    /// Get cached WASM module or load it
    pub async fn get_or_load(&self, wasm_path: &str) -> Result<Arc<WasmExtractor>> {
        // Check if we have a cached module
        {
            let cache = self.module.read().await;
            if let Some(cached) = cache.as_ref() {
                if cached.path == wasm_path {
                    // Update use count
                    let mut count = cached.use_count.write().await;
                    *count += 1;

                    return Ok(Arc::clone(&cached.extractor));
                }
            }
        }

        // Need to load new module
        self.load_module(wasm_path).await
    }

    /// Force reload WASM module
    pub async fn reload(&self, wasm_path: &str) -> Result<Arc<WasmExtractor>> {
        self.load_module(wasm_path).await
    }

    /// Load WASM module with timeout
    async fn load_module(&self, wasm_path: &str) -> Result<Arc<WasmExtractor>> {
        let start = Instant::now();

        // Load with timeout
        let extractor = tokio::time::timeout(self.init_timeout, WasmExtractor::new(wasm_path))
            .await
            .map_err(|_| {
                anyhow!(
                    "WASM module initialization timed out after {:?}",
                    self.init_timeout
                )
            })?
            .map_err(|e| anyhow!("Failed to initialize WASM module: {}", e))?;

        let load_time = start.elapsed();
        tracing::info!(
            path = wasm_path,
            load_time_ms = load_time.as_millis(),
            "WASM module loaded and cached"
        );

        let cached = CachedWasmModule {
            extractor: Arc::new(extractor),
            loaded_at: Instant::now(),
            path: wasm_path.to_string(),
            use_count: Arc::new(RwLock::new(1)),
        };

        let result = Arc::clone(&cached.extractor);

        // Update cache
        let mut cache = self.module.write().await;
        *cache = Some(cached);

        Ok(result)
    }

    /// Get cache statistics
    pub async fn stats(&self) -> Option<CacheStats> {
        let cache = self.module.read().await;

        if let Some(cached) = cache.as_ref() {
            let use_count = *cached.use_count.read().await;
            let age = cached.loaded_at.elapsed();

            Some(CacheStats {
                path: cached.path.clone(),
                loaded_at: cached.loaded_at,
                age_seconds: age.as_secs(),
                use_count,
                hit_rate: if use_count > 1 {
                    (use_count - 1) as f64 / use_count as f64
                } else {
                    0.0
                },
            })
        } else {
            None
        }
    }

    /// Clear the cache
    pub async fn clear(&self) {
        let mut cache = self.module.write().await;
        *cache = None;
        tracing::info!("WASM module cache cleared");
    }

    /// Check if cache is populated
    pub async fn is_cached(&self) -> bool {
        let cache = self.module.read().await;
        cache.is_some()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub path: String,
    pub loaded_at: Instant,
    pub age_seconds: u64,
    pub use_count: u64,
    pub hit_rate: f64,
}

/// Helper function to get WASM extractor with caching
pub async fn get_cached_extractor(
    wasm_path: &str,
    init_timeout_ms: u64,
) -> Result<Arc<WasmExtractor>> {
    let cache = WasmModuleCache::global();

    // Temporarily update timeout if different
    if init_timeout_ms != 10000 {
        // For non-standard timeouts, bypass cache and load directly
        let extractor = tokio::time::timeout(
            Duration::from_millis(init_timeout_ms),
            WasmExtractor::new(wasm_path),
        )
        .await
        .map_err(|_| anyhow!("WASM initialization timed out"))?
        .map_err(|e| anyhow!("WASM initialization failed: {}", e))?;

        return Ok(Arc::new(extractor));
    }

    cache.get_or_load(wasm_path).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_initialization() {
        let cache = WasmModuleCache::new(Duration::from_secs(5));
        assert!(!cache.is_cached().await);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = WasmModuleCache::new(Duration::from_secs(5));
        assert!(cache.stats().await.is_none());
    }

    #[test]
    fn test_global_cache() {
        let cache1 = WasmModuleCache::global();
        let cache2 = WasmModuleCache::global();

        // Should be the same instance
        assert!(std::ptr::eq(cache1, cache2));
    }
}
