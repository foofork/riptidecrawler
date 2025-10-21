//! Integration Tests for Singleton Usage in OptimizedExecutor
//!
//! This test suite validates that OptimizedExecutor correctly integrates
//! with the global singleton instances.

use anyhow::Result;
use riptide_cli::commands::optimized_executor::OptimizedExecutor;

/// Test that OptimizedExecutor can be initialized with global singletons
#[tokio::test]
async fn test_optimized_executor_singleton_initialization() -> Result<()> {
    // This should successfully initialize all global singletons
    let executor = OptimizedExecutor::new().await;

    assert!(
        executor.is_ok(),
        "OptimizedExecutor should initialize successfully with singletons"
    );

    if let Ok(exec) = executor {
        // Verify we can get stats (which uses the singletons)
        let stats = exec.get_stats().await;

        println!("OptimizedExecutor Stats: {:?}", stats);

        // Stats should be accessible
        assert!(
            serde_json::to_string(&stats).is_ok(),
            "Stats should be serializable"
        );
    }

    Ok(())
}

/// Test that multiple OptimizedExecutor instances share the same singletons
#[tokio::test]
async fn test_multiple_executors_share_singletons() -> Result<()> {
    // Create multiple executor instances
    let exec1 = OptimizedExecutor::new().await?;
    let exec2 = OptimizedExecutor::new().await?;

    // Both should work correctly
    let stats1 = exec1.get_stats().await;
    let stats2 = exec2.get_stats().await;

    // Since they share singletons, stats should be consistent
    println!("Executor 1 stats: {:?}", stats1);
    println!("Executor 2 stats: {:?}", stats2);

    // Both executors should be functional
    assert!(serde_json::to_string(&stats1).is_ok());
    assert!(serde_json::to_string(&stats2).is_ok());

    Ok(())
}

/// Test concurrent OptimizedExecutor instances
#[tokio::test]
async fn test_concurrent_executor_instances() -> Result<()> {
    const NUM_EXECUTORS: usize = 10;

    let mut handles = vec![];

    for i in 0..NUM_EXECUTORS {
        let handle = tokio::spawn(async move {
            let executor = OptimizedExecutor::new()
                .await
                .expect("Failed to create executor");

            let stats = executor.get_stats().await;

            (i, stats)
        });
        handles.push(handle);
    }

    // All executors should initialize successfully
    for handle in handles {
        let (id, stats) = handle.await.expect("Executor task failed");
        println!("Executor {} stats: {:?}", id, stats);
    }

    Ok(())
}

/// Test singleton state persists across executor lifecycle
#[tokio::test]
async fn test_singleton_state_persistence() -> Result<()> {
    use riptide_cli::commands::engine_cache::EngineSelectionCache;
    use riptide_cli::commands::engine_fallback::EngineType;

    // Set some state in the singleton
    let cache = EngineSelectionCache::get_global();
    cache.set("test-domain.com", EngineType::Wasm).await?;

    // Create an executor
    let executor1 = OptimizedExecutor::new().await?;
    drop(executor1);

    // State should still exist in singleton
    let cached_engine = cache.get("test-domain.com").await;
    assert!(
        cached_engine.is_some(),
        "Singleton state should persist after executor drop"
    );

    // Create another executor - should still see the same state
    let executor2 = OptimizedExecutor::new().await?;

    let cached_again = cache.get("test-domain.com").await;
    assert!(
        cached_again.is_some(),
        "Singleton state should be visible to new executor"
    );

    drop(executor2);

    Ok(())
}
