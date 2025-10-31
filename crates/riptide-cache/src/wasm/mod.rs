//! WASM Caching Infrastructure
//!
//! This module provides comprehensive WASM module caching capabilities:
//!
//! - **AOT Cache** (`aot`): Persistent disk-based caching of AOT-compiled WASM modules
//! - **Module Cache** (`module`): In-memory caching of loaded WASM extractors with lazy loading
//!
//! ## Usage Example
//!
//! ```no_run
//! use riptide_cache::wasm::{AotCacheConfig, WasmAotCache, WasmModuleCache};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // AOT cache for persistent storage
//!     let aot_cache = WasmAotCache::new(AotCacheConfig::default()).await?;
//!     let compiled = aot_cache.get_or_compile("module.wasm").await?;
//!
//!     // Module cache for runtime instances
//!     let module_cache = WasmModuleCache::new(Duration::from_secs(10));
//!     let extractor = module_cache.get_or_load("module.wasm").await?;
//!
//!     Ok(())
//! }
//! ```

pub mod aot;
pub mod module;

// Re-export key types for convenience
pub use aot::{
    get_global_aot_cache, AotCacheConfig, CacheEntry as AotCacheEntry, CacheStats as AotCacheStats,
    CompiledModule, WasmAotCache,
};
#[cfg(feature = "wasm-extractor")]
pub use module::{
    get_cached_extractor, CacheStats as ModuleCacheStats, CachedWasmModule, WasmCache,
    WasmModuleCache,
};
