//! WASM AOT cache infrastructure - CLI wrapper
//!
//! This module provides CLI access to the WASM AOT caching functionality
//! from the riptide-cache library.

#![allow(dead_code, unused_imports)]

// Re-export from library for backwards compatibility
pub use riptide_cache::wasm::{
    get_global_aot_cache, AotCacheConfig, AotCacheEntry as CacheEntry, AotCacheStats as CacheStats,
    CompiledModule, WasmAotCache,
};
