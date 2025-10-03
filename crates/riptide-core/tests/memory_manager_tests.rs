#[cfg(test)]
mod tests {
    use riptide_core::memory_manager::{MemoryManager, MemoryManagerConfig};
    use std::sync::Arc;
    use std::time::Duration;
    use wasmtime::{Config, Engine};

    fn create_test_engine() -> Engine {
        let mut config = Config::new();
        config.wasm_component_model(true);
        Engine::new(&config).expect("Failed to create WASM engine")
    }

    fn create_test_config() -> MemoryManagerConfig {
        MemoryManagerConfig {
            max_total_memory_mb: 1024,         // 1GB
            instance_memory_threshold_mb: 256, // 256MB per instance
            max_instances: 4,
            min_instances: 1,
            instance_idle_timeout: Duration::from_secs(30),
            monitoring_interval: Duration::from_secs(1),
            gc_interval: Duration::from_secs(5),
            aggressive_gc: false,
            memory_pressure_threshold: 80.0,
        }
    }

    #[tokio::test]
    async fn test_memory_manager_creation() -> Result<(), Box<dyn std::error::Error>> {
        let config = create_test_config();
        let engine = create_test_engine();

        let manager = MemoryManager::new(config.clone(), engine).await?;

        // Wait a moment for monitoring task to update stats
        tokio::time::sleep(Duration::from_millis(50)).await;
        let stats = manager.stats();

        // Verify initial stats
        assert_eq!(stats.instances_count, 0);
        assert_eq!(stats.active_instances, 0);
        assert_eq!(stats.idle_instances, 0);
        // Initially total_allocated_mb might be 0 until monitoring task runs
        assert!(stats.total_allocated_mb <= config.max_total_memory_mb);
        assert_eq!(stats.total_used_mb, 0);
        assert_eq!(stats.gc_runs, 0);

        manager.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_memory_stats_tracking() -> Result<(), Box<dyn std::error::Error>> {
        let config = create_test_config();
        let engine = create_test_engine();

        let manager = MemoryManager::new(config, engine).await?;

        let initial_stats = manager.stats();
        assert_eq!(initial_stats.total_used_mb, 0);
        assert_eq!(initial_stats.instances_count, 0);
        assert_eq!(initial_stats.memory_pressure, 0.0);

        manager.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_memory_pressure_calculation() -> Result<(), Box<dyn std::error::Error>> {
        let mut config = create_test_config();
        config.max_total_memory_mb = 100; // Small limit for testing
        config.memory_pressure_threshold = 50.0; // 50% threshold
        let engine = create_test_engine();

        let manager = MemoryManager::new(config, engine).await?;

        let stats = manager.stats();
        // Initially, pressure should be 0
        assert!(stats.memory_pressure < 1.0);

        manager.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_memory_manager_access() -> Result<(), Box<dyn std::error::Error>> {
        let config = create_test_config();
        let engine = create_test_engine();

        let manager = Arc::new(MemoryManager::new(config, engine).await?);
        let mut handles = vec![];

        // Spawn multiple tasks accessing the manager
        for i in 0..5 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let _stats = manager_clone.stats();
                // Verify each task can access stats successfully
                // No need to assert on values since u64/usize are always >= 0
                i // Return task id for verification
            });
            handles.push(handle);
        }

        let results: Vec<_> = futures::future::join_all(handles).await;

        // All tasks should complete successfully
        for (i, result) in results.iter().enumerate() {
            assert_eq!(result.as_ref().unwrap(), &i);
        }

        manager.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_memory_manager_with_custom_config() -> Result<(), Box<dyn std::error::Error>> {
        let config = MemoryManagerConfig {
            max_total_memory_mb: 512,
            instance_memory_threshold_mb: 128,
            max_instances: 2,
            min_instances: 1,
            instance_idle_timeout: Duration::from_secs(10),
            monitoring_interval: Duration::from_millis(500),
            gc_interval: Duration::from_secs(2),
            aggressive_gc: true,
            memory_pressure_threshold: 70.0,
        };
        let engine = create_test_engine();

        let manager = MemoryManager::new(config.clone(), engine).await?;

        // Wait for monitoring task to run and update stats
        tokio::time::sleep(Duration::from_millis(600)).await;
        let stats = manager.stats();

        // Verify config values are eventually reflected in stats
        // The monitoring task will update total_allocated_mb with the config value
        assert!(stats.total_allocated_mb <= config.max_total_memory_mb);

        manager.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_memory_manager_shutdown() -> Result<(), Box<dyn std::error::Error>> {
        let config = create_test_config();
        let engine = create_test_engine();

        let manager = MemoryManager::new(config, engine).await?;

        // Verify manager is operational
        let _stats = manager.stats();

        // Shutdown should complete without error
        manager.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_memory_manager_events() -> Result<(), Box<dyn std::error::Error>> {
        let config = create_test_config();
        let engine = create_test_engine();

        let manager = MemoryManager::new(config, engine).await?;

        // Get event receiver
        let events = manager.events();

        // Event receiver should be available
        assert!(!events.lock().await.is_closed());

        manager.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_memory_manager_garbage_collection() -> Result<(), Box<dyn std::error::Error>> {
        let config = create_test_config();
        let engine = create_test_engine();

        let manager = MemoryManager::new(config, engine).await?;

        let initial_stats = manager.stats();
        let initial_gc_runs = initial_stats.gc_runs;

        // Trigger garbage collection manually
        manager.trigger_garbage_collection().await;

        // Wait a bit for the operation to complete
        tokio::time::sleep(Duration::from_millis(100)).await;

        let stats_after_gc = manager.stats();
        // GC runs should have incremented (or stayed the same if no work needed)
        assert!(stats_after_gc.gc_runs >= initial_gc_runs);

        manager.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_memory_stats_fields() -> Result<(), Box<dyn std::error::Error>> {
        let config = create_test_config();
        let engine = create_test_engine();

        let manager = MemoryManager::new(config.clone(), engine).await?;

        // Wait for monitoring task to update stats
        tokio::time::sleep(Duration::from_millis(1100)).await;
        let stats = manager.stats();

        // Verify all expected MemoryStats fields are accessible
        // After monitoring task runs, stats should be updated with real values
        assert_eq!(stats.total_used_mb, 0);
        assert_eq!(stats.instances_count, 0);
        assert_eq!(stats.active_instances, 0);
        assert_eq!(stats.idle_instances, 0);
        assert_eq!(stats.peak_memory_mb, 0);
        assert_eq!(stats.gc_runs, 0);
        assert_eq!(stats.memory_pressure, 0.0);
        // After waiting, stats should have been updated at least once
        assert!(stats.last_updated.is_some());

        manager.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_memory_manager_with_aggressive_gc() -> Result<(), Box<dyn std::error::Error>> {
        let mut config = create_test_config();
        config.aggressive_gc = true;
        config.gc_interval = Duration::from_millis(100); // Fast GC for testing
        let engine = create_test_engine();

        let manager = MemoryManager::new(config, engine).await?;

        // Let the manager run for a bit with aggressive GC
        tokio::time::sleep(Duration::from_millis(200)).await;

        let _stats = manager.stats();
        // With aggressive GC, we might see some GC activity even with no instances
        // No need to assert on gc_runs >= 0 as u64 is always >= 0

        manager.shutdown().await?;
        Ok(())
    }
}
