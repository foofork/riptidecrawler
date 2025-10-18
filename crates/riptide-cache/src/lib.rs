//! # RipTide Cache
//!
//! Comprehensive caching system for the RipTide extraction framework.
//!
//! ## Features
//!
//! - **Distributed Caching**: Redis-based caching with HTTP semantics
//! - **Cache Key Generation**: Deterministic, collision-resistant key generation
//! - **HTTP Conditional Requests**: ETag and Last-Modified support
//! - **Cache Warming**: Intelligent preloading for performance optimization
//! - **Version-Aware**: Built-in cache invalidation via versioning
//!
//! ## Modules
//!
//! - [`manager`]: Redis-based cache manager with conditional request support
//! - [`key`]: Deterministic cache key generation with SHA256 hashing
//!
//! ## Quick Start
//!
//! ```no_run
//! use riptide_cache::prelude::*;
//! use std::collections::HashMap;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let mut cache = CacheManager::new("redis://localhost:6379").await?;
//!     
//!     // Generate cache key
//!     let key = cache.generate_cache_key(
//!         "https://example.com",
//!         "v1.0.0",
//!         &HashMap::new()
//!     );
//!     
//!     // Store data
//!     cache.set_simple(&key, &"data", 3600).await?;
//!     
//!     // Retrieve data
//!     if let Some(data) = cache.get_simple::<String>(&key).await? {
//!         println!("Cached: {}", data);
//!     }
//!     
//!     Ok(())
//! }
//! ```

pub mod key;
pub mod manager;
pub mod redis;
// pub mod integrated;  // Temporarily disabled: circular dependency with riptide-core
pub mod warming;
pub mod warming_integration;

// Re-exports for convenience
pub use key::{
    generate_fetch_cache_key, generate_strategies_cache_key, generate_wasm_cache_key,
    CacheKeyBuilder, CacheKeyParams,
};
pub use manager::{
    CacheConfig, CacheEntry, CacheManager, CacheMetadata, CacheStats, ConditionalResult,
};
pub use redis::{
    CacheConfig as RedisCacheConfig, CacheEntry as RedisCacheEntry,
    CacheManager as RedisCacheManager, CacheMetadata as RedisCacheMetadata,
    CacheStats as RedisCacheStats, ConditionalResult as RedisConditionalResult,
};
// pub use integrated::{
//     CachedContent, CacheCheckResult, IntegratedCacheConfig, IntegratedCacheManager,
//     IntegratedCacheStats, create_optimized_integrated_cache_manager,
// };
pub use warming::{
    CacheWarmingConfig, CacheWarmingManager, CacheWarmingOperation, CacheWarmingPoolExt,
    CacheWarmingStats, PreFetchPriority, PreFetchResource,
};
pub use warming_integration::{
    CacheWarmingEnabledPool, CacheWarmingHealthMonitor, CacheWarmingHealthStatus,
    CacheWarmingPoolFactory,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::key::{CacheKeyBuilder, CacheKeyParams};
    pub use crate::manager::{
        CacheConfig, CacheEntry, CacheManager, CacheMetadata, CacheStats, ConditionalResult,
    };
    pub use crate::redis::{CacheConfig as RedisCacheConfig, CacheManager as RedisCacheManager};
    // pub use crate::integrated::{IntegratedCacheManager, IntegratedCacheConfig};
    pub use crate::warming::{CacheWarmingConfig, CacheWarmingManager};
}

/// Cache version constant
pub const CACHE_VERSION: &str = "v1";

/// Default TTL in seconds (24 hours)
pub const DEFAULT_TTL: u64 = 24 * 60 * 60;
