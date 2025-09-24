#[cfg(test)]
mod tests {
    use riptide_core::memory_manager::{MemoryManager, MemoryStats, MemoryLimit};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[test]
    fn test_memory_manager_creation() {
        let manager = MemoryManager::new(1024 * 1024 * 100); // 100MB limit
        assert_eq!(manager.limit(), 1024 * 1024 * 100);
        assert_eq!(manager.used(), 0);
        assert!(manager.available() > 0);
    }

    #[test]
    fn test_memory_allocation() {
        let mut manager = MemoryManager::new(1024 * 1024); // 1MB

        // Allocate 512KB
        let result = manager.allocate(512 * 1024);
        assert!(result.is_ok());
        assert_eq!(manager.used(), 512 * 1024);
        assert_eq!(manager.available(), 512 * 1024);

        // Try to allocate more than available
        let result = manager.allocate(1024 * 1024);
        assert!(result.is_err());
        assert_eq!(manager.used(), 512 * 1024); // Unchanged
    }

    #[test]
    fn test_memory_release() {
        let mut manager = MemoryManager::new(1024 * 1024);

        manager.allocate(512 * 1024).unwrap();
        assert_eq!(manager.used(), 512 * 1024);

        manager.release(256 * 1024);
        assert_eq!(manager.used(), 256 * 1024);
        assert_eq!(manager.available(), 768 * 1024);

        // Release more than used (should set to 0)
        manager.release(1024 * 1024);
        assert_eq!(manager.used(), 0);
        assert_eq!(manager.available(), 1024 * 1024);
    }

    #[test]
    fn test_memory_stats() {
        let mut manager = MemoryManager::new(1024 * 1024 * 10); // 10MB

        manager.allocate(1024 * 1024 * 3).unwrap(); // 3MB

        let stats = manager.stats();
        assert_eq!(stats.total, 1024 * 1024 * 10);
        assert_eq!(stats.used, 1024 * 1024 * 3);
        assert_eq!(stats.available, 1024 * 1024 * 7);
        assert_eq!(stats.usage_percent(), 30.0);
    }

    #[test]
    fn test_memory_threshold_warning() {
        let mut manager = MemoryManager::with_threshold(1024 * 1024, 0.8);

        // Allocate 70% - should be ok
        manager.allocate(716800).unwrap(); // 700KB
        assert!(!manager.is_above_threshold());

        // Allocate more to go above 80%
        manager.allocate(204800).unwrap(); // 200KB more (900KB total = 88%)
        assert!(manager.is_above_threshold());
    }

    #[tokio::test]
    async fn test_concurrent_memory_allocation() {
        let manager = Arc::new(Mutex::new(MemoryManager::new(1024 * 1024 * 10))); // 10MB

        let mut handles = vec![];

        // Spawn 10 tasks each allocating 1MB
        for _ in 0..10 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let mut mgr = manager_clone.lock().await;
                mgr.allocate(1024 * 1024)
            });
            handles.push(handle);
        }

        let results: Vec<_> = futures::future::join_all(handles).await;

        // All allocations should succeed (total 10MB = limit)
        for result in &results {
            assert!(result.as_ref().unwrap().is_ok());
        }

        let final_stats = manager.lock().await.stats();
        assert_eq!(final_stats.used, 1024 * 1024 * 10);
        assert_eq!(final_stats.available, 0);
    }

    #[test]
    fn test_memory_guard() {
        let mut manager = MemoryManager::new(1024 * 1024);

        {
            let _guard = manager.allocate_with_guard(512 * 1024).unwrap();
            assert_eq!(manager.used(), 512 * 1024);
            // Guard will auto-release on drop
        }

        // Memory should be released after guard is dropped
        assert_eq!(manager.used(), 0);
    }

    #[test]
    fn test_memory_limit_enforcement() {
        let manager = MemoryManager::new(1024 * 1024); // 1MB

        // Create a limit enforcer
        let limit = MemoryLimit::new(512 * 1024); // 512KB limit

        assert!(limit.check_allocation(256 * 1024).is_ok()); // 256KB ok
        assert!(limit.check_allocation(768 * 1024).is_err()); // 768KB exceeds
    }

    #[test]
    fn test_memory_fragmentation_tracking() {
        let mut manager = MemoryManager::new(1024 * 1024);

        // Simulate fragmented allocations
        manager.allocate(100 * 1024).unwrap();
        manager.allocate(200 * 1024).unwrap();
        manager.release(100 * 1024);
        manager.allocate(150 * 1024).unwrap();

        let stats = manager.stats();
        assert_eq!(stats.used, 350 * 1024); // 200KB + 150KB
        assert!(stats.fragmentation_ratio() < 0.5); // Some fragmentation expected
    }

    #[test]
    fn test_memory_pressure_callback() {
        let mut manager = MemoryManager::with_threshold(1024 * 1024, 0.9);

        let mut pressure_triggered = false;
        manager.set_pressure_callback(Box::new(|| {
            pressure_triggered = true;
        }));

        // Allocate 95% to trigger pressure
        manager.allocate(972800).unwrap(); // 950KB

        // In real implementation, this would be called by the manager
        if manager.is_above_threshold() {
            manager.trigger_pressure_callback();
        }

        assert!(pressure_triggered);
    }
}