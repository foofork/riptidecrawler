//! Integration tests for global singleton implementations
//!
//! This test suite verifies thread safety, singleton identity, and proper
//! integration of all three global singletons used in the OptimizedExecutor.

use riptide_cli::commands::{
    engine_cache::EngineSelectionCache,
    performance_monitor::PerformanceMonitor,
    wasm_cache::WasmCache,
};
use std::sync::Arc;

/// Test 1: Verify singleton identity - same instance is always returned
#[test]
fn test_singleton_identity() {
    // EngineSelectionCache
    let engine_cache1 = EngineSelectionCache::get_global();
    let engine_cache2 = EngineSelectionCache::get_global();
    assert!(
        Arc::ptr_eq(&engine_cache1, &engine_cache2),
        "EngineSelectionCache should return the same Arc instance"
    );

    // WasmCache
    let wasm_cache1 = WasmCache::get_global();
    let wasm_cache2 = WasmCache::get_global();
    assert!(
        Arc::ptr_eq(&wasm_cache1, &wasm_cache2),
        "WasmCache should return the same Arc instance"
    );

    // PerformanceMonitor
    let perf_monitor1 = PerformanceMonitor::get_global();
    let perf_monitor2 = PerformanceMonitor::get_global();
    assert!(
        Arc::ptr_eq(&perf_monitor1, &perf_monitor2),
        "PerformanceMonitor should return the same Arc instance"
    );
}

/// Test 2: Concurrent singleton access from multiple threads
#[tokio::test]
async fn test_concurrent_singleton_access() {
    let mut handles = vec![];

    // Spawn 10 concurrent tasks accessing each singleton
    for i in 0..10 {
        let handle = tokio::spawn(async move {
            let engine_cache = EngineSelectionCache::get_global();
            let wasm_cache = WasmCache::get_global();
            let perf_monitor = PerformanceMonitor::get_global();

            // Verify all are valid Arc instances with positive reference counts
            assert!(
                Arc::strong_count(&engine_cache) > 0,
                "Task {}: EngineSelectionCache should have positive ref count",
                i
            );
            assert!(
                Arc::strong_count(&wasm_cache) > 0,
                "Task {}: WasmCache should have positive ref count",
                i
            );
            assert!(
                Arc::strong_count(&perf_monitor) > 0,
                "Task {}: PerformanceMonitor should have positive ref count",
                i
            );

            // Return the instances to verify they're the same across tasks
            (engine_cache, wasm_cache, perf_monitor)
        });
        handles.push(handle);
    }

    // Wait for all tasks and collect results
    let mut results = vec![];
    for handle in handles {
        let result = handle.await.expect("Task should not panic");
        results.push(result);
    }

    // Verify all tasks got the same singleton instances
    let (first_engine, first_wasm, first_perf) = &results[0];
    for (i, (engine, wasm, perf)) in results.iter().enumerate().skip(1) {
        assert!(
            Arc::ptr_eq(engine, first_engine),
            "Task {} got different EngineSelectionCache instance",
            i
        );
        assert!(
            Arc::ptr_eq(wasm, first_wasm),
            "Task {} got different WasmCache instance",
            i
        );
        assert!(
            Arc::ptr_eq(perf, first_perf),
            "Task {} got different PerformanceMonitor instance",
            i
        );
    }
}

/// Test 3: Singleton state is shared across threads
#[tokio::test]
async fn test_shared_state_across_threads() {
    use riptide_reliability::engine_selection::Engine;

    // Thread 1: Write to EngineSelectionCache
    let handle1 = tokio::spawn(async {
        let cache = EngineSelectionCache::get_global();
        cache
            .set("thread-test.com", Engine::Wasm)
            .await
            .expect("Should store cache entry");
    });

    // Wait for write
    handle1.await.expect("Task should complete");

    // Thread 2: Read from EngineSelectionCache
    let handle2 = tokio::spawn(async {
        let cache = EngineSelectionCache::get_global();
        let result = cache.get("thread-test.com").await;
        assert_eq!(
            result,
            Some(Engine::Wasm),
            "Should read value written by another thread"
        );
    });

    handle2.await.expect("Task should complete");
}

/// Test 4: OptimizedExecutor initialization uses singletons correctly
#[tokio::test]
async fn test_optimized_executor_initialization() {
    use riptide_cli::commands::optimized_executor::OptimizedExecutor;

    let result = OptimizedExecutor::new().await;
    assert!(
        result.is_ok(),
        "OptimizedExecutor should initialize with singletons: {:?}",
        result.err()
    );

    let executor = result.unwrap();

    // Verify executor can be used (basic smoke test)
    let stats = executor.get_stats().await;
    assert!(
        stats.browser_pool.is_object(),
        "Should get browser pool stats"
    );
}

/// Test 5: Singleton operations are thread-safe under concurrent load
#[tokio::test]
async fn test_thread_safe_concurrent_operations() {
    use riptide_reliability::engine_selection::Engine;

    let mut handles = vec![];

    // 20 concurrent tasks performing mixed operations
    for i in 0..20 {
        let handle = tokio::spawn(async move {
            let engine_cache = EngineSelectionCache::get_global();
            let wasm_cache = WasmCache::get_global();
            let perf_monitor = PerformanceMonitor::get_global();

            // Interleaved reads and writes
            let domain = format!("concurrent-test-{}.com", i);

            // Write to engine cache
            engine_cache
                .set(&domain, Engine::Headless)
                .await
                .expect("Should store");

            // Read from engine cache
            let result = engine_cache.get(&domain).await;
            assert_eq!(result, Some(Engine::Headless));

            // Update feedback
            engine_cache
                .update_feedback(&domain, true)
                .await
                .expect("Should update");

            // Get cache stats
            let stats = engine_cache.stats().await;
            assert!(stats.entries > 0, "Cache should have entries");

            // WasmCache operations
            let wasm_stats = wasm_cache.stats().await;
            assert!(wasm_stats.is_object(), "Should get wasm stats");

            // PerformanceMonitor operations
            let perf_stats = perf_monitor.get_stats().await;
            assert_eq!(perf_stats.total_operations, 0); // No ops recorded yet in this test
        });
        handles.push(handle);
    }

    // Wait for all concurrent operations
    for handle in handles {
        handle.await.expect("Concurrent task should succeed");
    }
}

/// Test 6: Singleton reference counting behavior
#[test]
fn test_singleton_reference_counting() {
    // Get initial references
    let cache1 = EngineSelectionCache::get_global();
    let initial_count = Arc::strong_count(&cache1);

    // Get more references in same thread
    let cache2 = EngineSelectionCache::get_global();
    let cache3 = EngineSelectionCache::get_global();

    // Strong count should increase
    assert!(
        Arc::strong_count(&cache1) > initial_count,
        "Reference count should increase with more Arc clones"
    );

    // Verify all point to same data
    assert!(Arc::ptr_eq(&cache1, &cache2));
    assert!(Arc::ptr_eq(&cache2, &cache3));

    // Drop references
    drop(cache2);
    drop(cache3);

    // Count should decrease back
    assert_eq!(
        Arc::strong_count(&cache1),
        initial_count,
        "Reference count should decrease after dropping"
    );
}

/// Test 7: All three singletons can be used together safely
#[tokio::test]
async fn test_combined_singleton_usage() {
    use riptide_reliability::engine_selection::Engine;

    let engine_cache = EngineSelectionCache::get_global();
    let wasm_cache = WasmCache::get_global();
    let perf_monitor = PerformanceMonitor::get_global();

    // Use all three together
    engine_cache
        .set("combined-test.com", Engine::Wasm)
        .await
        .expect("Should set");

    let wasm_stats = wasm_cache.stats().await;
    assert!(wasm_stats.is_object());

    let perf_stats = perf_monitor.get_stats().await;
    assert_eq!(perf_stats.total_operations, 0);

    // Verify engine cache worked
    let cached = engine_cache.get("combined-test.com").await;
    assert_eq!(cached, Some(Engine::Wasm));
}

/// Test 8: Singleton cleanup and TTL behavior
#[tokio::test]
async fn test_singleton_ttl_cleanup() {
    use riptide_reliability::engine_selection::Engine;
    use std::time::Duration;

    let cache = EngineSelectionCache::get_global();

    // Set an entry
    cache
        .set("ttl-test.com", Engine::Raw)
        .await
        .expect("Should set");

    // Verify it exists
    let result = cache.get("ttl-test.com").await;
    assert_eq!(result, Some(Engine::Raw));

    // Trigger cleanup (won't expire yet - default TTL is 1 hour)
    cache.cleanup_expired().await;

    // Should still exist
    let result = cache.get("ttl-test.com").await;
    assert_eq!(result, Some(Engine::Raw));

    // Verify stats reflect the entry
    let stats = cache.stats().await;
    assert_eq!(stats.entries, 1);
}

/// Test 9: Singleton initialization is lazy and safe
#[test]
fn test_lazy_initialization() {
    // First access initializes the singleton
    let cache1 = EngineSelectionCache::get_global();
    let initial_count = Arc::strong_count(&cache1);

    // Subsequent accesses reuse the same instance
    let cache2 = EngineSelectionCache::get_global();
    assert!(Arc::ptr_eq(&cache1, &cache2));

    // Same for other singletons
    let wasm1 = WasmCache::get_global();
    let wasm2 = WasmCache::get_global();
    assert!(Arc::ptr_eq(&wasm1, &wasm2));

    let perf1 = PerformanceMonitor::get_global();
    let perf2 = PerformanceMonitor::get_global();
    assert!(Arc::ptr_eq(&perf1, &perf2));
}

/// Test 10: Cross-singleton coordination
#[tokio::test]
async fn test_cross_singleton_coordination() {
    use riptide_reliability::engine_selection::Engine;

    // Simulate a typical workflow using all singletons
    let engine_cache = EngineSelectionCache::get_global();
    let wasm_cache = WasmCache::get_global();
    let perf_monitor = PerformanceMonitor::get_global();

    // Step 1: Check engine cache
    let domain = "coordination-test.com";
    let cached_engine = engine_cache.get(domain).await;

    // Step 2: If not cached, make a decision
    if cached_engine.is_none() {
        engine_cache
            .set(domain, Engine::Wasm)
            .await
            .expect("Should cache decision");
    }

    // Step 3: Use WASM cache
    let wasm_stats = wasm_cache.stats().await;
    assert!(wasm_stats.is_object());

    // Step 4: Record performance
    let perf_stats = perf_monitor.get_stats().await;
    assert_eq!(perf_stats.failed_operations, 0);

    // Verify engine cache now has entry
    let result = engine_cache.get(domain).await;
    assert!(result.is_some());
}
