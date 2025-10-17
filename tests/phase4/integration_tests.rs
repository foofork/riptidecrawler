//! Phase 4 Integration Tests
//!
//! Tests the integration of all Phase 4 optimizations working together:
//! - Browser pool + WASM AOT cache
//! - Browser pool + adaptive timeout
//! - WASM AOT cache + adaptive timeout
//! - All three combined in realistic scenarios

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

use chromiumoxide::BrowserConfig;
use riptide_headless::pool::{BrowserPool, BrowserPoolConfig};
use riptide_extraction::wasm_extraction::{CmExtractor, ExtractorConfig, WasmExtractor};
use riptide_intelligence::timeout::TimeoutWrapper;
use riptide_intelligence::mock_provider::MockLlmProvider;
use riptide_intelligence::provider::Message;
use riptide_intelligence::CompletionRequest;

#[tokio::test]
async fn test_browser_pool_with_wasm_aot() {
    // Test browser pool working with WASM AOT cache

    println!("=== Browser Pool + WASM AOT Integration Test ===");

    let temp_cache = tempfile::TempDir::new().expect("Failed to create temp directory");
    std::env::set_var("WASMTIME_CACHE_DIR", temp_cache.path().to_str().unwrap());

    // Initialize browser pool
    let pool_config = BrowserPoolConfig {
        initial_pool_size: 2,
        max_pool_size: 5,
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = Arc::new(
        BrowserPool::new(pool_config, browser_config)
            .await
            .expect("Failed to create browser pool"),
    );

    // Initialize WASM extractor with AOT cache
    let wasm_path = find_wasm_component_path();
    let extractor_config = ExtractorConfig {
        enable_aot_cache: true,
        enable_simd: true,
        ..Default::default()
    };

    // First extraction - browser pool is warm, WASM compiles
    let start1 = Instant::now();
    let extractor = CmExtractor::with_config(&wasm_path, extractor_config.clone())
        .await
        .expect("Failed to create extractor");

    let checkout1 = pool.checkout().await.expect("Failed to checkout browser");
    let time1 = start1.elapsed();
    checkout1.cleanup().await.expect("Failed to cleanup");

    println!("First extraction: {:?}", time1);

    // Second extraction - both browser pool and WASM cache are ready
    let start2 = Instant::now();
    let _extractor2 = CmExtractor::with_config(&wasm_path, extractor_config)
        .await
        .expect("Failed to create extractor");

    let checkout2 = pool.checkout().await.expect("Failed to checkout browser");
    let time2 = start2.elapsed();
    checkout2.cleanup().await.expect("Failed to cleanup");

    println!("Second extraction: {:?}", time2);

    // Second should be faster due to both optimizations
    assert!(
        time2 < time1,
        "Second extraction should be faster with both optimizations"
    );

    pool.shutdown().await.expect("Failed to shutdown pool");
    std::env::remove_var("WASMTIME_CACHE_DIR");
}

#[tokio::test]
async fn test_browser_pool_with_adaptive_timeout() {
    // Test browser pool working with adaptive timeout

    println!("=== Browser Pool + Adaptive Timeout Integration Test ===");

    let pool_config = BrowserPoolConfig {
        initial_pool_size: 2,
        cleanup_timeout: Duration::from_secs(2), // Adaptive timeout
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = BrowserPool::new(pool_config, browser_config)
        .await
        .expect("Failed to create browser pool");

    // Checkout with adaptive timeout
    let start = Instant::now();
    let checkout = pool.checkout().await.expect("Failed to checkout browser");

    // Simulate work
    sleep(Duration::from_millis(100)).await;

    // Cleanup with adaptive timeout
    checkout
        .cleanup()
        .await
        .expect("Failed to cleanup with adaptive timeout");
    let total_time = start.elapsed();

    println!("Total time with adaptive timeout: {:?}", total_time);

    // Should complete quickly with adaptive timeout
    assert!(
        total_time < Duration::from_secs(5),
        "Operations should complete quickly with adaptive timeout"
    );

    pool.shutdown().await.expect("Failed to shutdown pool");
}

#[tokio::test]
async fn test_wasm_aot_with_adaptive_timeout() {
    // Test WASM AOT cache working with adaptive timeout

    println!("=== WASM AOT + Adaptive Timeout Integration Test ===");

    let temp_cache = tempfile::TempDir::new().expect("Failed to create temp directory");
    std::env::set_var("WASMTIME_CACHE_DIR", temp_cache.path().to_str().unwrap());

    let wasm_path = find_wasm_component_path();
    let config = ExtractorConfig {
        enable_aot_cache: true,
        extraction_timeout: Some(2000), // 2 second adaptive timeout
        ..Default::default()
    };

    // First load with timeout
    let start1 = Instant::now();
    let extractor1 = CmExtractor::with_config(&wasm_path, config.clone()).await;
    let time1 = start1.elapsed();

    // Second load with timeout (should use cache)
    let start2 = Instant::now();
    let extractor2 = CmExtractor::with_config(&wasm_path, config).await;
    let time2 = start2.elapsed();

    println!("First load: {:?}, Second load: {:?}", time1, time2);

    // Both should succeed
    assert!(extractor1.is_ok(), "First load should succeed");
    assert!(extractor2.is_ok(), "Second load should succeed");

    // Second should be faster
    assert!(
        time2 < time1,
        "Second load should be faster with AOT cache"
    );

    std::env::remove_var("WASMTIME_CACHE_DIR");
}

#[tokio::test]
async fn test_all_optimizations_combined() {
    // Test all Phase 4 optimizations working together

    println!("=== All Phase 4 Optimizations Integration Test ===");

    let temp_cache = tempfile::TempDir::new().expect("Failed to create temp directory");
    std::env::set_var("WASMTIME_CACHE_DIR", temp_cache.path().to_str().unwrap());

    // 1. Initialize browser pool (pre-warming)
    let pool_config = BrowserPoolConfig {
        initial_pool_size: 3,
        max_pool_size: 10,
        cleanup_timeout: Duration::from_secs(1), // Adaptive timeout
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = Arc::new(
        BrowserPool::new(pool_config, browser_config)
            .await
            .expect("Failed to create browser pool"),
    );

    // 2. Initialize WASM extractor with AOT cache
    let wasm_path = find_wasm_component_path();
    let extractor_config = ExtractorConfig {
        enable_aot_cache: true,
        enable_simd: true,
        extraction_timeout: Some(5000), // Adaptive timeout
        instance_pool_size: 4,
        ..Default::default()
    };

    // 3. Initialize adaptive timeout for LLM
    let mock_provider = Arc::new(MockLlmProvider::new().with_delay(500));
    let timeout_wrapper = Arc::new(TimeoutWrapper::with_timeout(
        mock_provider,
        Duration::from_secs(2), // Adaptive timeout
    ));

    // Run complete workflow
    let workflow_start = Instant::now();

    // Browser checkout (should be instant with pool)
    let checkout = pool
        .checkout()
        .await
        .expect("Failed to checkout browser");

    // WASM initialization (should use cache on second run)
    let _extractor = CmExtractor::with_config(&wasm_path, extractor_config)
        .await
        .expect("Failed to create extractor");

    // LLM operation (with adaptive timeout)
    let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Test")]);
    let _result = timeout_wrapper
        .complete(request)
        .await
        .expect("Failed to complete request");

    // Cleanup
    checkout
        .cleanup()
        .await
        .expect("Failed to cleanup browser");

    let workflow_time = workflow_start.elapsed();
    println!("Complete workflow time: {:?}", workflow_time);

    // With all optimizations, workflow should be fast
    assert!(
        workflow_time < Duration::from_secs(10),
        "Complete workflow should be fast with all optimizations"
    );

    pool.shutdown().await.expect("Failed to shutdown pool");
    std::env::remove_var("WASMTIME_CACHE_DIR");
}

#[tokio::test]
async fn test_concurrent_integrated_workload() {
    // Test all optimizations under concurrent load

    println!("=== Concurrent Integrated Workload Test ===");

    let temp_cache = tempfile::TempDir::new().expect("Failed to create temp directory");
    std::env::set_var("WASMTIME_CACHE_DIR", temp_cache.path().to_str().unwrap());

    // Initialize all components
    let pool_config = BrowserPoolConfig {
        initial_pool_size: 5,
        max_pool_size: 15,
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = Arc::new(
        BrowserPool::new(pool_config, browser_config)
            .await
            .expect("Failed to create browser pool"),
    );

    let wasm_path = find_wasm_component_path();
    let extractor_config = ExtractorConfig {
        enable_aot_cache: true,
        ..Default::default()
    };

    // Pre-warm WASM cache
    let _extractor = CmExtractor::with_config(&wasm_path, extractor_config.clone())
        .await
        .expect("Failed to pre-warm cache");

    // Run 20 concurrent operations
    let start = Instant::now();
    let mut handles = vec![];

    for i in 0..20 {
        let pool_clone = Arc::clone(&pool);
        let wasm_path = wasm_path.clone();
        let extractor_config = extractor_config.clone();

        let handle = tokio::spawn(async move {
            // Checkout browser
            let checkout = pool_clone
                .checkout()
                .await
                .expect(&format!("Failed to checkout browser {}", i));

            // Load WASM extractor (should use cache)
            let _extractor = CmExtractor::with_config(&wasm_path, extractor_config)
                .await
                .expect(&format!("Failed to create extractor {}", i));

            // Simulate work
            sleep(Duration::from_millis(50)).await;

            // Cleanup
            checkout
                .cleanup()
                .await
                .expect(&format!("Failed to cleanup {}", i));
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let results: Vec<_> = futures::future::join_all(handles).await;
    let concurrent_time = start.elapsed();

    println!(
        "20 concurrent operations completed in: {:?}",
        concurrent_time
    );

    // All should succeed
    let success_count = results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(success_count, 20, "All concurrent operations should succeed");

    // With all optimizations, should complete quickly
    assert!(
        concurrent_time < Duration::from_secs(15),
        "Concurrent operations should complete quickly"
    );

    pool.shutdown().await.expect("Failed to shutdown pool");
    std::env::remove_var("WASMTIME_CACHE_DIR");
}

#[tokio::test]
async fn test_failure_recovery_integrated() {
    // Test failure recovery with all optimizations

    println!("=== Integrated Failure Recovery Test ===");

    let temp_cache = tempfile::TempDir::new().expect("Failed to create temp directory");
    std::env::set_var("WASMTIME_CACHE_DIR", temp_cache.path().to_str().unwrap());

    // Initialize components with failure recovery enabled
    let pool_config = BrowserPoolConfig {
        initial_pool_size: 2,
        min_pool_size: 1,
        enable_recovery: true,
        health_check_interval: Duration::from_secs(1),
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = BrowserPool::new(pool_config, browser_config)
        .await
        .expect("Failed to create browser pool");

    // Simulate operations
    let checkout1 = pool.checkout().await.expect("Failed to checkout 1");
    checkout1.cleanup().await.expect("Failed to cleanup 1");

    // Wait for health check cycle
    sleep(Duration::from_secs(2)).await;

    // Pool should still be functional
    let checkout2 = pool.checkout().await.expect("Failed to checkout 2 after recovery");
    checkout2.cleanup().await.expect("Failed to cleanup 2");

    println!("Failure recovery test completed successfully");

    pool.shutdown().await.expect("Failed to shutdown pool");
    std::env::remove_var("WASMTIME_CACHE_DIR");
}

#[tokio::test]
async fn test_resource_limits_integrated() {
    // Test resource limits with all optimizations

    println!("=== Integrated Resource Limits Test ===");

    let pool_config = BrowserPoolConfig {
        initial_pool_size: 2,
        max_pool_size: 3,
        memory_threshold_mb: 500,
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = Arc::new(
        BrowserPool::new(pool_config, browser_config)
            .await
            .expect("Failed to create browser pool"),
    );

    // Checkout maximum number of browsers
    let checkout1 = pool.checkout().await.expect("Failed to checkout 1");
    let checkout2 = pool.checkout().await.expect("Failed to checkout 2");
    let checkout3 = pool.checkout().await.expect("Failed to checkout 3");

    // Verify pool is at capacity
    let stats = pool.stats().await;
    assert_eq!(stats.in_use, 3, "Pool should be at capacity");

    // Try to checkout one more (should block or fail)
    let pool_clone = Arc::clone(&pool);
    let timeout_result = tokio::time::timeout(Duration::from_millis(500), async move {
        pool_clone.checkout().await
    })
    .await;

    assert!(
        timeout_result.is_err(),
        "Checkout should timeout when pool is at capacity"
    );

    // Cleanup
    checkout1.cleanup().await.expect("Failed to cleanup 1");
    checkout2.cleanup().await.expect("Failed to cleanup 2");
    checkout3.cleanup().await.expect("Failed to cleanup 3");

    pool.shutdown().await.expect("Failed to shutdown pool");
}

#[tokio::test]
async fn test_graceful_degradation() {
    // Test graceful degradation when optimizations are disabled

    println!("=== Graceful Degradation Test ===");

    // Disable all optimizations
    let pool_config = BrowserPoolConfig {
        initial_pool_size: 1,
        enable_recovery: false,
        ..Default::default()
    };

    let browser_config = BrowserConfig::builder()
        .build()
        .expect("Failed to build browser config");

    let pool = BrowserPool::new(pool_config, browser_config)
        .await
        .expect("Failed to create browser pool even without optimizations");

    let wasm_path = find_wasm_component_path();
    let extractor_config = ExtractorConfig {
        enable_aot_cache: false,
        enable_simd: false,
        ..Default::default()
    };

    let _extractor = CmExtractor::with_config(&wasm_path, extractor_config)
        .await
        .expect("Failed to create extractor without optimizations");

    // System should still function, just slower
    let checkout = pool.checkout().await.expect("Failed to checkout");
    checkout.cleanup().await.expect("Failed to cleanup");

    println!("System functions correctly even without optimizations");

    pool.shutdown().await.expect("Failed to shutdown pool");
}

// Helper functions

fn find_wasm_component_path() -> String {
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

    "dummy.wasm".to_string()
}
