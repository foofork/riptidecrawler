//! WASM Module Caching with Lazy Loading - CLI wrapper
//!
//! This module provides CLI access to the WASM module caching functionality
//! from the riptide-cache library.

// Re-export from library for backwards compatibility
pub use riptide_cache::wasm::{
    get_cached_extractor, CachedWasmModule, ModuleCacheStats as CacheStats, WasmCache,
    WasmModuleCache,
};
