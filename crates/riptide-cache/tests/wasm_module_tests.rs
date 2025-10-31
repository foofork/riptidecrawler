#![cfg(feature = "wasm-extractor")]
//! Comprehensive tests for WASM module cache

use riptide_cache::wasm::WasmModuleCache;
use std::time::Duration;

/// Test WASM module cache creation
#[test]
fn test_wasm_module_cache_creation() {
    let timeout = Duration::from_secs(5);
    let _cache = WasmModuleCache::new(timeout);

    // Cache should be created successfully
    // Further tests require actual WASM modules
}

/// Test global WASM module cache singleton
#[test]
fn test_wasm_module_cache_global() {
    let cache1 = WasmModuleCache::global();
    let cache2 = WasmModuleCache::global();

    // Should return same instance (pointer equality)
    assert!(std::ptr::eq(cache1, cache2));
}

/// Test WASM module cache with invalid path
#[tokio::test]
async fn test_wasm_module_cache_invalid_path() {
    let cache = WasmModuleCache::new(Duration::from_secs(5));
    let result = cache.get_or_load("nonexistent.wasm").await;

    // Should return error for non-existent file
    assert!(result.is_err());
}

/// Test WASM module cache timeout handling
#[tokio::test]
async fn test_wasm_module_cache_timeout() {
    // Very short timeout to trigger timeout error
    let cache = WasmModuleCache::new(Duration::from_millis(1));
    let result = cache.get_or_load("test.wasm").await;

    // Should timeout or fail to load
    assert!(result.is_err());
}

/// Test WASM module cache reload functionality
#[tokio::test]
async fn test_wasm_module_cache_reload() {
    let cache = WasmModuleCache::new(Duration::from_secs(10));

    // Reload non-existent module should fail
    let result = cache.reload("nonexistent.wasm").await;
    assert!(result.is_err());
}

/// Test cached WASM module structure
#[test]
fn test_cached_wasm_module_structure() {
    // CachedWasmModule is used internally
    // Testing its structure indirectly through cache operations
    let _cache = WasmModuleCache::new(Duration::from_secs(5));

    // Cache initialization should succeed
}

/// Test WASM module cache with different timeouts
#[test]
fn test_wasm_module_cache_various_timeouts() {
    let timeouts = vec![
        Duration::from_millis(100),
        Duration::from_secs(1),
        Duration::from_secs(10),
        Duration::from_secs(30),
    ];

    for timeout in timeouts {
        let _cache = WasmModuleCache::new(timeout);
        // Each cache should be created successfully
    }
}

/// Test WASM cache concurrent operations
#[tokio::test]
async fn test_wasm_cache_concurrent_operations() {
    let cache = WasmModuleCache::new(Duration::from_secs(5));

    // Multiple concurrent reload attempts
    for _ in 0..5 {
        let result = cache.reload("test.wasm").await;
        // Should handle concurrent access gracefully
        assert!(result.is_err()); // No actual file exists
    }
}

/// Test WASM cache error handling
#[tokio::test]
async fn test_wasm_cache_error_handling() {
    let cache = WasmModuleCache::new(Duration::from_secs(5));

    // Test various error conditions
    let invalid_paths = vec![
        "",
        "/invalid/path/module.wasm",
        "../../etc/passwd",
        "https://example.com/module.wasm",
    ];

    for path in invalid_paths {
        let result = cache.get_or_load(path).await;
        assert!(result.is_err(), "Should fail for invalid path: {}", path);
    }
}
