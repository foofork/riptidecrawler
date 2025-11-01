#![allow(dead_code)]

use anyhow::Result;
use once_cell::sync::Lazy;
/// Engine Selection Cache for Domain-Based Optimization
///
/// This module provides intelligent caching of engine selection decisions
/// based on domain patterns to avoid repeated analysis.
///
/// **Note**: This is infrastructure code designed for Phase 5+ API integration.
/// Some methods and structs are intentionally unused in current implementation.
use riptide_reliability::engine_selection::Engine;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Global singleton instance of the engine selection cache
/// Infrastructure: Used by test suite and future API endpoints
#[allow(dead_code)]
static GLOBAL_INSTANCE: Lazy<Arc<EngineSelectionCache>> =
    Lazy::new(|| Arc::new(EngineSelectionCache::default()));

/// Cache entry for engine selection
/// Infrastructure: Internal implementation detail
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub engine: Engine,
    pub timestamp: Instant,
    pub hit_count: u64,
    pub success_rate: f64,
}

/// Engine selection cache with TTL and domain-based heuristics
/// Infrastructure: Core caching implementation for future API
#[allow(dead_code)]
pub struct EngineSelectionCache {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    ttl: Duration,
    max_entries: usize,
}

impl EngineSelectionCache {
    /// Get the global singleton instance of the engine selection cache
    pub fn get_global() -> Arc<Self> {
        Arc::clone(&GLOBAL_INSTANCE)
    }

    /// Create a new engine selection cache
    pub fn new(ttl: Duration, max_entries: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
            max_entries,
        }
    }

    /// Get cached engine decision for a domain
    pub async fn get(&self, domain: &str) -> Option<Engine> {
        let cache = self.cache.read().await;

        if let Some(entry) = cache.get(domain) {
            // Check if entry is still valid
            if entry.timestamp.elapsed() < self.ttl {
                return Some(entry.engine);
            }
        }

        None
    }

    /// Store engine decision for a domain
    pub async fn set(&self, domain: &str, engine: Engine) -> Result<()> {
        let mut cache = self.cache.write().await;

        // Evict old entries if cache is full
        if cache.len() >= self.max_entries {
            self.evict_oldest(&mut cache);
        }

        let entry = CacheEntry {
            engine,
            timestamp: Instant::now(),
            hit_count: 1,
            success_rate: 1.0,
        };

        cache.insert(domain.to_string(), entry);
        Ok(())
    }

    /// Store engine decision with confidence score
    pub async fn store(&self, domain: &str, engine: Engine, confidence: f64) -> Result<()> {
        let mut cache = self.cache.write().await;

        // Evict old entries if cache is full
        if cache.len() >= self.max_entries {
            self.evict_oldest(&mut cache);
        }

        let entry = CacheEntry {
            engine,
            timestamp: Instant::now(),
            hit_count: 1,
            success_rate: confidence,
        };

        cache.insert(domain.to_string(), entry);
        Ok(())
    }

    /// Update cache entry with success/failure feedback
    pub async fn update_feedback(&self, domain: &str, success: bool) -> Result<()> {
        let mut cache = self.cache.write().await;

        if let Some(entry) = cache.get_mut(domain) {
            entry.hit_count += 1;
            let current_success = entry.success_rate * (entry.hit_count - 1) as f64;
            let new_success = if success { 1.0 } else { 0.0 };
            entry.success_rate = (current_success + new_success) / entry.hit_count as f64;
        }

        Ok(())
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;

        let mut total_hits = 0u64;
        let mut avg_success_rate = 0.0;

        for entry in cache.values() {
            total_hits += entry.hit_count;
            avg_success_rate += entry.success_rate;
        }

        if !cache.is_empty() {
            avg_success_rate /= cache.len() as f64;
        }

        CacheStats {
            entries: cache.len(),
            total_hits,
            avg_success_rate,
            max_capacity: self.max_entries,
        }
    }

    /// Clear expired entries
    pub async fn cleanup_expired(&self) {
        let mut cache = self.cache.write().await;
        cache.retain(|_, entry| entry.timestamp.elapsed() < self.ttl);
    }

    /// Extract domain from URL
    pub fn extract_domain(url: &str) -> String {
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                return host.to_string();
            }
        }
        url.to_string()
    }

    /// Evict oldest entries when cache is full
    fn evict_oldest(&self, cache: &mut HashMap<String, CacheEntry>) {
        if cache.is_empty() {
            return;
        }

        // Find and remove oldest entry
        if let Some(oldest_key) = cache
            .iter()
            .min_by_key(|(_, entry)| entry.timestamp)
            .map(|(k, _)| k.clone())
        {
            cache.remove(&oldest_key);
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub total_hits: u64,
    pub avg_success_rate: f64,
    pub max_capacity: usize,
}

impl Default for EngineSelectionCache {
    fn default() -> Self {
        Self::new(Duration::from_secs(3600), 1000) // 1 hour TTL, 1000 entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let cache = EngineSelectionCache::new(Duration::from_secs(60), 10);

        // Test set and get
        cache.set("example.com", Engine::Wasm).await.unwrap();
        assert_eq!(cache.get("example.com").await, Some(Engine::Wasm));

        // Test non-existent domain
        assert_eq!(cache.get("nonexistent.com").await, None);
    }

    #[tokio::test]
    async fn test_cache_eviction() {
        let cache = EngineSelectionCache::new(Duration::from_secs(60), 2);

        cache.set("domain1.com", Engine::Wasm).await.unwrap();
        cache.set("domain2.com", Engine::Headless).await.unwrap();
        cache.set("domain3.com", Engine::Raw).await.unwrap();

        let stats = cache.stats().await;
        assert_eq!(stats.entries, 2); // Should evict oldest
    }

    #[tokio::test]
    async fn test_domain_extraction() {
        assert_eq!(
            EngineSelectionCache::extract_domain("https://example.com/path"),
            "example.com"
        );
        assert_eq!(
            EngineSelectionCache::extract_domain("http://subdomain.example.com"),
            "subdomain.example.com"
        );
    }
}
