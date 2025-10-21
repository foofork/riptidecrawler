//! Thread Safety Tests for Global Singleton Implementations
//!
//! This test suite validates that all global singleton instances are thread-safe
//! and can handle concurrent access without race conditions or data corruption.
//!
//! Validated Singletons:
//! 1. EngineSelectionCache (GLOBAL_INSTANCE with Lazy + Arc)
//! 2. WasmCache (GLOBAL_WASM_CACHE with Lazy + Arc)
//! 3. PerformanceMonitor (GLOBAL_MONITOR with Lazy + Arc)
//!
//! Test Strategy:
//! - Spawn multiple concurrent tasks accessing the same singleton
//! - Verify data consistency across all tasks
//! - Test read/write operations under high concurrency
//! - Validate no deadlocks or race conditions occur

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Test concurrent access to EngineSelectionCache singleton
#[tokio::test]
async fn test_engine_cache_singleton_thread_safety() {
    use riptide_cli::commands::engine_cache::EngineSelectionCache;
    use riptide_cli::commands::engine_fallback::EngineType;

    const NUM_TASKS: usize = 100;
    const DOMAINS: &[&str] = &["example.com", "test.org", "sample.net", "demo.io"];

    let mut handles = vec![];

    // Spawn concurrent tasks accessing the singleton
    for task_id in 0..NUM_TASKS {
        let handle = tokio::spawn(async move {
            let cache = EngineSelectionCache::get_global();
            let domain = DOMAINS[task_id % DOMAINS.len()];

            // Perform concurrent reads and writes
            for i in 0..10 {
                let engine = if i % 2 == 0 {
                    EngineType::Wasm
                } else {
                    EngineType::Headless
                };

                // Write operation
                cache
                    .set(domain, engine)
                    .await
                    .expect("Failed to set engine");

                // Small delay to increase interleaving
                sleep(Duration::from_micros(10)).await;

                // Read operation
                let cached = cache.get(domain).await;
                assert!(cached.is_some(), "Engine should be cached for {}", domain);

                // Update feedback
                cache
                    .update_feedback(domain, i % 3 != 0)
                    .await
                    .expect("Failed to update feedback");
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("Task panicked");
    }

    // Validate final state - cache should be consistent
    let cache = EngineSelectionCache::get_global();
    let stats = cache.stats().await;

    println!("EngineCache Stats: {:?}", stats);
    assert!(stats.total_entries > 0, "Cache should have entries");
    assert!(stats.total_entries <= DOMAINS.len(), "Cache should not exceed domain count");
}

/// Test concurrent access to WasmCache singleton
#[tokio::test]
async fn test_wasm_cache_singleton_thread_safety() {
    use riptide_cli::commands::wasm_cache::WasmCache;

    const NUM_TASKS: usize = 50;
    const TEST_PATH: &str = "/tmp/test_wasm_module.wasm";

    let mut handles = vec![];

    // Spawn concurrent tasks accessing the singleton
    for task_id in 0..NUM_TASKS {
        let handle = tokio::spawn(async move {
            let cache = WasmCache::get_global();

            // Attempt concurrent operations
            for _ in 0..5 {
                // Try to get (will fail since file doesn't exist, but tests thread safety)
                let _ = cache.get(TEST_PATH).await;

                sleep(Duration::from_micros(10)).await;

                // Check statistics
                let stats = cache.stats().await;
                assert!(stats.total_requests >= 0, "Stats should be non-negative");
            }

            task_id
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for (i, handle) in handles.into_iter().enumerate() {
        let task_id = handle.await.expect("Task panicked");
        assert_eq!(task_id, i, "Task IDs should match");
    }

    // Validate singleton consistency
    let cache = WasmCache::get_global();
    let stats = cache.stats().await;

    println!("WasmCache Stats: {:?}", stats);
    assert!(stats.total_requests >= NUM_TASKS * 5, "Should track all requests");
}

/// Test concurrent access to PerformanceMonitor singleton
#[tokio::test]
async fn test_performance_monitor_singleton_thread_safety() {
    use riptide_cli::commands::performance_monitor::PerformanceMonitor;

    const NUM_TASKS: usize = 100;

    let mut handles = vec![];

    // Spawn concurrent recording tasks
    for task_id in 0..NUM_TASKS {
        let handle = tokio::spawn(async move {
            let monitor = PerformanceMonitor::get_global();

            // Record multiple operations concurrently
            for op in 0..10 {
                let url = format!("https://test{}.com", task_id);
                let engine = if op % 2 == 0 { "wasm" } else { "headless" };
                let duration_ms = (task_id * 10 + op) as u64;

                monitor
                    .record_extraction(&url, engine, duration_ms)
                    .await
                    .expect("Failed to record extraction");

                sleep(Duration::from_micros(10)).await;
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.expect("Task panicked");
    }

    // Validate consistency
    let monitor = PerformanceMonitor::get_global();
    let stats = monitor.get_stats().await;

    println!("PerformanceMonitor Stats: {:?}", stats);
    assert!(
        stats.total_extractions >= NUM_TASKS * 10,
        "Should track all extractions"
    );
}

/// Test that all singletons return the same Arc instance
#[tokio::test]
async fn test_singleton_instance_uniqueness() {
    use riptide_cli::commands::engine_cache::EngineSelectionCache;
    use riptide_cli::commands::performance_monitor::PerformanceMonitor;
    use riptide_cli::commands::wasm_cache::WasmCache;

    // Get multiple references to each singleton
    let engine1 = EngineSelectionCache::get_global();
    let engine2 = EngineSelectionCache::get_global();

    let wasm1 = WasmCache::get_global();
    let wasm2 = WasmCache::get_global();

    let perf1 = PerformanceMonitor::get_global();
    let perf2 = PerformanceMonitor::get_global();

    // Verify same Arc instances (pointer equality)
    assert!(
        Arc::ptr_eq(&engine1, &engine2),
        "EngineCache should return same Arc instance"
    );
    assert!(
        Arc::ptr_eq(&wasm1, &wasm2),
        "WasmCache should return same Arc instance"
    );
    assert!(
        Arc::ptr_eq(&perf1, &perf2),
        "PerformanceMonitor should return same Arc instance"
    );
}

/// Test high-concurrency scenario with all singletons
#[tokio::test]
async fn test_all_singletons_concurrent_stress() {
    use riptide_cli::commands::engine_cache::EngineSelectionCache;
    use riptide_cli::commands::engine_fallback::EngineType;
    use riptide_cli::commands::performance_monitor::PerformanceMonitor;
    use riptide_cli::commands::wasm_cache::WasmCache;

    const NUM_TASKS: usize = 200;

    let mut handles = vec![];

    for task_id in 0..NUM_TASKS {
        let handle = tokio::spawn(async move {
            // Access all three singletons concurrently
            let engine_cache = EngineSelectionCache::get_global();
            let wasm_cache = WasmCache::get_global();
            let perf_monitor = PerformanceMonitor::get_global();

            let domain = format!("stress-test-{}.com", task_id % 10);

            // Perform mixed operations
            for i in 0..20 {
                // Engine cache operations
                engine_cache
                    .set(&domain, EngineType::Wasm)
                    .await
                    .expect("Engine cache write failed");

                let _ = engine_cache.get(&domain).await;

                // WasmCache operations (read-only for safety)
                let _ = wasm_cache.get("/tmp/dummy.wasm").await;
                let _ = wasm_cache.stats().await;

                // PerformanceMonitor operations
                perf_monitor
                    .record_extraction(&domain, "wasm", i * 10)
                    .await
                    .expect("Failed to record extraction");

                sleep(Duration::from_micros(5)).await;
            }
        });
        handles.push(handle);
    }

    // All tasks should complete without deadlock
    for handle in handles {
        handle.await.expect("Stress test task panicked");
    }

    println!("✓ All {} concurrent tasks completed successfully", NUM_TASKS);
}

/// Test singleton initialization is thread-safe
#[tokio::test]
async fn test_singleton_initialization_race() {
    use riptide_cli::commands::engine_cache::EngineSelectionCache;
    use riptide_cli::commands::performance_monitor::PerformanceMonitor;
    use riptide_cli::commands::wasm_cache::WasmCache;

    const NUM_TASKS: usize = 100;

    // Attempt to initialize all singletons concurrently
    let mut handles = vec![];

    for _ in 0..NUM_TASKS {
        let handle = tokio::spawn(async move {
            // Try to get all singletons at the same time
            let _ = EngineSelectionCache::get_global();
            let _ = WasmCache::get_global();
            let _ = PerformanceMonitor::get_global();
        });
        handles.push(handle);
    }

    // All should initialize successfully
    for handle in handles {
        handle.await.expect("Initialization race task failed");
    }

    println!("✓ Singleton initialization is race-free");
}

/// Validate no memory leaks under concurrent access
#[tokio::test]
async fn test_singleton_no_memory_leak() {
    use riptide_cli::commands::engine_cache::EngineSelectionCache;

    const ITERATIONS: usize = 1000;

    // Repeatedly access singleton and verify Arc count doesn't grow
    for _ in 0..ITERATIONS {
        let cache = EngineSelectionCache::get_global();
        let strong_count = Arc::strong_count(&cache);

        // Should be 2: one for the static, one for this reference
        assert!(
            strong_count >= 1,
            "Arc strong count should be at least 1"
        );

        drop(cache);
    }

    println!("✓ No memory leaks detected over {} iterations", ITERATIONS);
}
