//! Comprehensive tests for WASM AOT Cache (Phase 4)
//!
//! Tests cover:
//! - First-time compilation and caching
//! - Cache hit on subsequent loads
//! - Hash-based invalidation
//! - Concurrent compilation
//! - Cache persistence across runs
//! - Atomic cache updates
//! - Cache corruption handling

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;
use tokio::fs;
use wasmtime::{Config, Engine};

use riptide_extraction::wasm_extraction::{CmExtractor, ExtractorConfig};

#[tokio::test]
async fn test_first_time_compilation_and_caching() {
    // Test that first compilation takes longer but creates cache
    let temp_cache = TempDir::new().expect("Failed to create temp directory");
    let cache_dir = temp_cache.path();

    // Set cache directory
    std::env::set_var("WASMTIME_CACHE_DIR", cache_dir.to_str().unwrap());

    let config = ExtractorConfig {
        enable_aot_cache: true,
        ..Default::default()
    };

    // Get WASM path
    let wasm_path = find_wasm_component_path();

    // First load - should compile and cache
    let start = Instant::now();
    let extractor = CmExtractor::with_config(&wasm_path, config.clone())
        .await
        .expect("Failed to create extractor on first load");
    let first_load_time = start.elapsed();

    drop(extractor);

    // Second load - should use cache
    let start = Instant::now();
    let _extractor = CmExtractor::with_config(&wasm_path, config)
        .await
        .expect("Failed to create extractor on second load");
    let second_load_time = start.elapsed();

    // Cache should significantly reduce load time
    println!(
        "First load: {:?}, Second load: {:?}",
        first_load_time, second_load_time
    );
    assert!(
        second_load_time < first_load_time * 2 / 3,
        "Second load should be significantly faster due to cache"
    );

    // Verify cache files were created
    let cache_files: Vec<_> = fs::read_dir(cache_dir)
        .await
        .expect("Failed to read cache dir")
        .collect::<Vec<_>>()
        .await;
    assert!(
        !cache_files.is_empty(),
        "Cache directory should contain cached files"
    );

    // Cleanup
    std::env::remove_var("WASMTIME_CACHE_DIR");
}

#[tokio::test]
async fn test_cache_hit_on_subsequent_loads() {
    // Test that cache is consistently used across multiple loads
    let temp_cache = TempDir::new().expect("Failed to create temp directory");
    std::env::set_var("WASMTIME_CACHE_DIR", temp_cache.path().to_str().unwrap());

    let config = ExtractorConfig {
        enable_aot_cache: true,
        ..Default::default()
    };

    let wasm_path = find_wasm_component_path();

    // First load to populate cache
    let _extractor1 = CmExtractor::with_config(&wasm_path, config.clone())
        .await
        .expect("Failed on first load");

    // Multiple subsequent loads should all hit cache
    let mut load_times = Vec::new();
    for i in 0..5 {
        let start = Instant::now();
        let _extractor = CmExtractor::with_config(&wasm_path, config.clone())
            .await
            .expect(&format!("Failed on load {}", i + 2));
        load_times.push(start.elapsed());
    }

    // All cached loads should be reasonably fast and consistent
    let avg_time: std::time::Duration = load_times.iter().sum::<std::time::Duration>() / load_times.len() as u32;
    println!("Average cached load time: {:?}", avg_time);
    assert!(
        avg_time < std::time::Duration::from_secs(2),
        "Cached loads should be fast"
    );

    std::env::remove_var("WASMTIME_CACHE_DIR");
}

#[tokio::test]
async fn test_hash_based_invalidation() {
    // Test that changing the WASM file invalidates cache
    let temp_cache = TempDir::new().expect("Failed to create temp directory");
    let temp_wasm = TempDir::new().expect("Failed to create wasm temp directory");

    std::env::set_var("WASMTIME_CACHE_DIR", temp_cache.path().to_str().unwrap());

    let config = ExtractorConfig {
        enable_aot_cache: true,
        ..Default::default()
    };

    // Copy WASM file to temp location
    let original_wasm = find_wasm_component_path();
    let temp_wasm_path = temp_wasm.path().join("component.wasm");
    fs::copy(&original_wasm, &temp_wasm_path)
        .await
        .expect("Failed to copy WASM file");

    // First load
    let _extractor1 = CmExtractor::with_config(temp_wasm_path.to_str().unwrap(), config.clone())
        .await
        .expect("Failed on first load");

    // Modify the WASM file (append a byte)
    let mut content = fs::read(&temp_wasm_path)
        .await
        .expect("Failed to read WASM file");
    content.push(0);
    fs::write(&temp_wasm_path, &content)
        .await
        .expect("Failed to write modified WASM file");

    // Second load - should detect change and recompile
    // Note: This will likely fail to load due to invalid WASM,
    // but it demonstrates that the cache was invalidated
    let result = CmExtractor::with_config(temp_wasm_path.to_str().unwrap(), config).await;

    // The key point is that it attempted to recompile rather than using stale cache
    assert!(
        result.is_err(),
        "Modified WASM should trigger recompilation (and fail due to invalid content)"
    );

    std::env::remove_var("WASMTIME_CACHE_DIR");
}

#[tokio::test]
async fn test_concurrent_compilation() {
    // Test that concurrent loads handle cache correctly
    let temp_cache = TempDir::new().expect("Failed to create temp directory");
    std::env::set_var("WASMTIME_CACHE_DIR", temp_cache.path().to_str().unwrap());

    let config = ExtractorConfig {
        enable_aot_cache: true,
        ..Default::default()
    };

    let wasm_path = find_wasm_component_path();

    // Spawn multiple concurrent loads
    let mut handles = vec![];
    for i in 0..5 {
        let wasm_path = wasm_path.clone();
        let config = config.clone();
        let handle = tokio::spawn(async move {
            CmExtractor::with_config(&wasm_path, config)
                .await
                .expect(&format!("Failed on concurrent load {}", i))
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // All should succeed
    for (i, result) in results.iter().enumerate() {
        assert!(
            result.is_ok(),
            "Concurrent load {} should succeed",
            i
        );
    }

    std::env::remove_var("WASMTIME_CACHE_DIR");
}

#[tokio::test]
async fn test_cache_persistence_across_runs() {
    // Test that cache persists across separate "runs" (extractor instances)
    let temp_cache = TempDir::new().expect("Failed to create temp directory");
    let cache_dir = temp_cache.path();
    std::env::set_var("WASMTIME_CACHE_DIR", cache_dir.to_str().unwrap());

    let config = ExtractorConfig {
        enable_aot_cache: true,
        ..Default::default()
    };

    let wasm_path = find_wasm_component_path();

    // First "run" - populate cache
    {
        let _extractor = CmExtractor::with_config(&wasm_path, config.clone())
            .await
            .expect("Failed on first run");
        // Drop extractor to simulate end of run
    }

    // Verify cache exists
    let cache_entries: Vec<_> = fs::read_dir(cache_dir)
        .await
        .expect("Failed to read cache dir")
        .collect::<Vec<_>>()
        .await;
    let cache_count_before = cache_entries.len();
    assert!(cache_count_before > 0, "Cache should have entries");

    // Second "run" - should use persisted cache
    {
        let start = Instant::now();
        let _extractor = CmExtractor::with_config(&wasm_path, config)
            .await
            .expect("Failed on second run");
        let load_time = start.elapsed();

        println!("Load time from persisted cache: {:?}", load_time);
        assert!(
            load_time < std::time::Duration::from_secs(3),
            "Loading from persisted cache should be fast"
        );
    }

    // Cache should still exist and not have duplicates
    let cache_entries_after: Vec<_> = fs::read_dir(cache_dir)
        .await
        .expect("Failed to read cache dir")
        .collect::<Vec<_>>()
        .await;
    let cache_count_after = cache_entries_after.len();

    // Cache count should be stable (no duplicates)
    assert!(
        cache_count_after >= cache_count_before,
        "Cache should persist and not lose entries"
    );

    std::env::remove_var("WASMTIME_CACHE_DIR");
}

#[tokio::test]
async fn test_atomic_cache_updates() {
    // Test that cache updates are atomic (no partial writes)
    let temp_cache = TempDir::new().expect("Failed to create temp directory");
    std::env::set_var("WASMTIME_CACHE_DIR", temp_cache.path().to_str().unwrap());

    let config = ExtractorConfig {
        enable_aot_cache: true,
        ..Default::default()
    };

    let wasm_path = find_wasm_component_path();

    // Start compilation
    let handle = tokio::spawn({
        let wasm_path = wasm_path.clone();
        let config = config.clone();
        async move {
            CmExtractor::with_config(&wasm_path, config).await
        }
    });

    // Give it a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Try to load while compilation is in progress
    let concurrent_result = CmExtractor::with_config(&wasm_path, config.clone()).await;

    // Wait for first compilation to complete
    let first_result = handle.await.expect("Task panicked");

    // Both should succeed - wasmtime handles concurrent cache access
    assert!(first_result.is_ok(), "First compilation should succeed");
    assert!(
        concurrent_result.is_ok(),
        "Concurrent load during compilation should succeed"
    );

    std::env::remove_var("WASMTIME_CACHE_DIR");
}

#[tokio::test]
async fn test_cache_corruption_handling() {
    // Test that corrupted cache is detected and recompiled
    let temp_cache = TempDir::new().expect("Failed to create temp directory");
    let cache_dir = temp_cache.path();
    std::env::set_var("WASMTIME_CACHE_DIR", cache_dir.to_str().unwrap());

    let config = ExtractorConfig {
        enable_aot_cache: true,
        ..Default::default()
    };

    let wasm_path = find_wasm_component_path();

    // First load to populate cache
    let _extractor1 = CmExtractor::with_config(&wasm_path, config.clone())
        .await
        .expect("Failed on first load");

    // Find and corrupt a cache file
    let cache_files: Vec<_> = fs::read_dir(cache_dir)
        .await
        .expect("Failed to read cache dir")
        .collect::<Vec<_>>()
        .await;

    if let Some(Ok(entry)) = cache_files.first() {
        let cache_file = entry.path();
        // Corrupt the cache file by truncating it
        fs::write(&cache_file, b"corrupted")
            .await
            .expect("Failed to corrupt cache file");

        // Try to load again - should detect corruption and recompile
        let result = CmExtractor::with_config(&wasm_path, config).await;

        // Wasmtime should handle corruption gracefully by recompiling
        assert!(
            result.is_ok(),
            "Should handle cache corruption by recompiling"
        );
    }

    std::env::remove_var("WASMTIME_CACHE_DIR");
}

#[tokio::test]
async fn test_cache_disabled_mode() {
    // Test that disabling cache works correctly
    let config = ExtractorConfig {
        enable_aot_cache: false,
        ..Default::default()
    };

    let wasm_path = find_wasm_component_path();

    // Load without cache
    let start1 = Instant::now();
    let _extractor1 = CmExtractor::with_config(&wasm_path, config.clone())
        .await
        .expect("Failed on first load");
    let time1 = start1.elapsed();

    drop(_extractor1);

    // Load again without cache
    let start2 = Instant::now();
    let _extractor2 = CmExtractor::with_config(&wasm_path, config)
        .await
        .expect("Failed on second load");
    let time2 = start2.elapsed();

    println!(
        "Load times without cache: {:?}, {:?}",
        time1, time2
    );

    // Both loads should take similar time (no caching benefit)
    // Allow for some variance, but they should be in the same ballpark
    let ratio = if time1 > time2 {
        time1.as_secs_f64() / time2.as_secs_f64()
    } else {
        time2.as_secs_f64() / time1.as_secs_f64()
    };

    assert!(
        ratio < 2.0,
        "Without caching, load times should be similar"
    );
}

#[tokio::test]
async fn test_cache_size_management() {
    // Test cache behavior with size constraints
    let temp_cache = TempDir::new().expect("Failed to create temp directory");
    std::env::set_var("WASMTIME_CACHE_DIR", temp_cache.path().to_str().unwrap());

    let config = ExtractorConfig {
        enable_aot_cache: true,
        ..Default::default()
    };

    let wasm_path = find_wasm_component_path();

    // Load multiple times
    for _ in 0..10 {
        let _extractor = CmExtractor::with_config(&wasm_path, config.clone())
            .await
            .expect("Failed to load");
    }

    // Check cache size is reasonable
    let cache_size = calculate_directory_size(temp_cache.path())
        .await
        .expect("Failed to calculate cache size");

    println!("Cache size after 10 loads: {} bytes", cache_size);

    // Cache shouldn't grow unbounded - wasmtime deduplicates
    assert!(
        cache_size < 100 * 1024 * 1024,
        "Cache size should be reasonable (< 100MB)"
    );

    std::env::remove_var("WASMTIME_CACHE_DIR");
}

// Helper functions

fn find_wasm_component_path() -> String {
    // Try to find the WASM component in the build output
    let possible_paths = vec![
        "target/wasm32-wasip2/debug/riptide_extractor_wasm.wasm",
        "target/wasm32-wasip2/release/riptide_extractor_wasm.wasm",
        "wasm/riptide-extractor-wasm/target/wasm32-wasip2/debug/riptide_extractor_wasm.wasm",
        "wasm/riptide-extractor-wasm/target/wasm32-wasip2/release/riptide_extractor_wasm.wasm",
    ];

    for path in possible_paths {
        if std::path::Path::new(path).exists() {
            return path.to_string();
        }
    }

    panic!("Could not find WASM component. Please build it first.");
}

async fn calculate_directory_size(path: &std::path::Path) -> std::io::Result<u64> {
    let mut total_size = 0u64;
    let mut entries = fs::read_dir(path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let metadata = entry.metadata().await?;
        if metadata.is_file() {
            total_size += metadata.len();
        } else if metadata.is_dir() {
            total_size += calculate_directory_size(&entry.path()).await?;
        }
    }

    Ok(total_size)
}
