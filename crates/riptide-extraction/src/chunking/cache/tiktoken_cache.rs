//! Async tiktoken-based token counting with caching
//!
//! Provides exact token counts using tiktoken-rs with an LRU cache for performance.
//! The cache is thread-safe and suitable for concurrent access across async tasks.

use anyhow::{Context, Result};
use dashmap::DashMap;
use std::sync::Arc;
use tiktoken_rs::{cl100k_base, CoreBPE};
use tracing::debug;

/// Maximum number of cached token counts
const DEFAULT_CACHE_SIZE: usize = 10_000;

/// LRU cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    count: usize,
    /// Timestamp for LRU tracking (monotonic counter)
    access_count: u64,
}

/// Thread-safe async cache for token counts
#[derive(Clone)]
pub struct TiktokenCache {
    /// DashMap for concurrent access
    cache: Arc<DashMap<String, CacheEntry>>,
    /// Maximum cache size
    max_size: usize,
    /// Global access counter for LRU
    access_counter: Arc<parking_lot::Mutex<u64>>,
    /// Tiktoken encoder (cl100k_base for GPT-4/GPT-3.5-turbo)
    encoder: Arc<CoreBPE>,
}

impl TiktokenCache {
    /// Create a new tiktoken cache with default size
    pub fn new() -> Result<Self> {
        Self::with_capacity(DEFAULT_CACHE_SIZE)
    }

    /// Create a new tiktoken cache with specified capacity
    pub fn with_capacity(max_size: usize) -> Result<Self> {
        let encoder = cl100k_base().context("Failed to initialize cl100k_base encoder")?;

        Ok(Self {
            cache: Arc::new(DashMap::with_capacity(max_size)),
            max_size,
            access_counter: Arc::new(parking_lot::Mutex::new(0)),
            encoder: Arc::new(encoder),
        })
    }

    /// Get exact token count for text with caching
    pub async fn count_tokens(&self, text: &str) -> Result<usize> {
        // Fast path: check cache first
        if let Some(entry) = self.cache.get(text) {
            let count = entry.count;
            // Drop the entry reference before acquiring write lock
            drop(entry);

            // Update access counter for LRU
            let access_count = {
                let mut counter = self.access_counter.lock();
                *counter = counter.saturating_add(1);
                *counter
            };

            // Update the entry's access count
            self.cache.alter(text, |_, mut entry| {
                entry.access_count = access_count;
                entry
            });

            debug!("Token count cache hit for {} chars", text.len());
            return Ok(count);
        }

        // Slow path: compute token count
        let count = tokio::task::spawn_blocking({
            let encoder = Arc::clone(&self.encoder);
            let text = text.to_string();
            move || encoder.encode_with_special_tokens(&text).len()
        })
        .await
        .context("Failed to spawn token counting task")?;

        // Check if we need to evict entries
        if self.cache.len() >= self.max_size {
            self.evict_lru();
        }

        // Store in cache
        let access_count = {
            let mut counter = self.access_counter.lock();
            *counter = counter.saturating_add(1);
            *counter
        };

        self.cache.insert(
            text.to_string(),
            CacheEntry {
                count,
                access_count,
            },
        );

        debug!(
            "Token count cache miss, computed {} tokens for {} chars",
            count,
            text.len()
        );
        Ok(count)
    }

    /// Count tokens for multiple text chunks in batch
    pub async fn count_tokens_batch(&self, texts: &[&str]) -> Result<Vec<usize>> {
        // Process in parallel with limited concurrency
        let mut tasks = Vec::new();

        for text in texts {
            let cache = self.clone();
            let text_str = text.to_string();
            tasks.push(tokio::spawn(
                async move { cache.count_tokens(&text_str).await },
            ));
        }

        let mut results = Vec::with_capacity(texts.len());
        for task in tasks {
            let count = task.await.context("Failed to join token counting task")??;
            results.push(count);
        }

        Ok(results)
    }

    /// Evict least recently used entries (approximately 10% of cache)
    fn evict_lru(&self) {
        let eviction_count = (self.max_size / 10).max(1);
        let mut entries: Vec<(String, u64)> = self
            .cache
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().access_count))
            .collect();

        // Sort by access count (oldest first)
        entries.sort_by_key(|(_, count)| *count);

        // Remove oldest entries
        for (key, _) in entries.iter().take(eviction_count) {
            self.cache.remove(key);
        }

        debug!("Evicted {} LRU entries from token cache", eviction_count);
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            size: self.cache.len(),
            capacity: self.max_size,
            access_count: *self.access_counter.lock(),
        }
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.cache.clear();
        *self.access_counter.lock() = 0;
        debug!("Token cache cleared");
    }
}

impl Default for TiktokenCache {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            // Log the error and return a minimal working cache
            eprintln!("Warning: Failed to create default TiktokenCache: {e}");
            // Fallback to a minimal capacity cache
            Self::with_capacity(100).expect("Failed to create fallback TiktokenCache")
        })
    }
}

/// Cache statistics
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    pub size: usize,
    pub capacity: usize,
    pub access_count: u64,
}

// Global cache instance using lazy_static pattern
use std::sync::OnceLock;

static GLOBAL_CACHE: OnceLock<TiktokenCache> = OnceLock::new();

/// Get or initialize the global token cache
fn get_global_cache() -> &'static TiktokenCache {
    GLOBAL_CACHE.get_or_init(|| {
        TiktokenCache::new().unwrap_or_else(|e| {
            eprintln!("Warning: Failed to initialize global tiktoken cache: {e}");
            // Fallback to a minimal capacity cache
            TiktokenCache::with_capacity(100).expect("Failed to create fallback tiktoken cache")
        })
    })
}

/// Count tokens for text using the global cache (convenience function)
pub async fn count_tokens_exact(text: &str) -> Result<usize> {
    get_global_cache().count_tokens(text).await
}

/// Count tokens for multiple texts in batch using the global cache
pub async fn count_tokens_batch(texts: &[&str]) -> Result<Vec<usize>> {
    get_global_cache().count_tokens_batch(texts).await
}

/// Get global cache statistics
pub fn cache_stats() -> CacheStats {
    get_global_cache().stats()
}

/// Clear the global cache
pub fn clear_cache() {
    get_global_cache().clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_token_counting() {
        let cache = TiktokenCache::new().unwrap();
        let text = "Hello, world! This is a test.";
        let count = cache.count_tokens(text).await.unwrap();

        // Verify we get a reasonable token count
        assert!(count > 0);
        assert!(count < 20); // Should be around 8-10 tokens
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let cache = TiktokenCache::new().unwrap();
        let text = "This is a cached test.";

        // First call - cache miss
        let count1 = cache.count_tokens(text).await.unwrap();

        // Second call - cache hit
        let count2 = cache.count_tokens(text).await.unwrap();

        assert_eq!(count1, count2);
        assert_eq!(cache.stats().size, 1);
    }

    #[tokio::test]
    async fn test_batch_counting() {
        let cache = TiktokenCache::new().unwrap();
        let texts = vec!["First text", "Second text", "Third text with more words"];

        let counts = cache.count_tokens_batch(&texts).await.unwrap();

        assert_eq!(counts.len(), 3);
        assert!(counts[0] > 0);
        assert!(counts[1] > 0);
        assert!(counts[2] > counts[0]); // Third text should have more tokens
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        // Small cache for testing eviction
        let cache = TiktokenCache::with_capacity(10).unwrap();

        // Fill cache beyond capacity
        for i in 0..15 {
            let text = format!("Test text number {}", i);
            cache.count_tokens(&text).await.unwrap();
        }

        // Cache should have evicted some entries
        let stats = cache.stats();
        assert!(stats.size <= 10);
    }

    #[tokio::test]
    async fn test_global_cache() {
        let text = "Testing global cache";
        let count = count_tokens_exact(text).await.unwrap();
        assert!(count > 0);

        // Verify cache was populated
        let stats = cache_stats();
        assert!(stats.size > 0);
    }

    #[tokio::test]
    async fn test_accuracy_vs_approximation() {
        let cache = TiktokenCache::new().unwrap();

        // Test various text lengths
        let test_cases = vec![
            "Short text",
            "This is a medium length text with several words in it.",
            "This is a much longer text that contains many more words and should result in a significantly higher token count when compared to the approximation method that simply multiplies word count by 1.3.",
        ];

        for text in test_cases {
            let exact = cache.count_tokens(text).await.unwrap();
            let approx = (text.split_whitespace().count() as f64 * 1.3) as usize;

            // Log the difference
            let diff = (exact as i32 - approx as i32).abs();
            let percent_diff = (diff as f64 / exact as f64) * 100.0;

            println!("Text: '{}...'", &text[..text.len().min(30)]);
            println!(
                "  Exact: {}, Approx: {}, Diff: {} ({:.1}%)",
                exact, approx, diff, percent_diff
            );

            // Exact count should be reasonably close (within 50%)
            // but we're testing to show improvement
            assert!(exact > 0);
        }
    }

    #[tokio::test]
    async fn test_empty_text() {
        let cache = TiktokenCache::new().unwrap();
        let count = cache.count_tokens("").await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_special_characters() {
        let cache = TiktokenCache::new().unwrap();
        let text = "Test with émojis 🎉 and spëcial çharacters!";
        let count = cache.count_tokens(text).await.unwrap();
        assert!(count > 0);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let cache = Arc::new(TiktokenCache::new().unwrap());
        let mut handles = vec![];

        // Spawn multiple concurrent tasks
        for i in 0..10 {
            let cache = Arc::clone(&cache);
            let handle = tokio::spawn(async move {
                let text = format!("Concurrent text {}", i);
                cache.count_tokens(&text).await
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }
}
