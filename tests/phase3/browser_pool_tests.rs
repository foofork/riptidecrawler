//! Browser Pool Management Tests
//!
//! Comprehensive tests for browser pool functionality including:
//! - Pool initialization and sizing
//! - Browser checkout/checkin
//! - Health checks and recovery
//! - Concurrent access handling
//! - Resource cleanup

use std::time::Duration;

#[cfg(test)]
mod browser_pool_tests {
    use super::*;

    /// Test basic pool initialization
    #[tokio::test]
    async fn test_pool_initialization() {
        let config = create_test_pool_config(3, 10);
        let pool = BrowserPool::new(config).await;

        assert!(pool.is_ok(), "Pool initialization should succeed");

        if let Ok(pool) = pool {
            let stats = pool.stats().await;
            assert_eq!(stats.available, 3, "Should initialize with 3 browsers");
            assert_eq!(stats.in_use, 0, "No browsers should be in use");
            assert_eq!(stats.total_capacity, 10, "Total capacity should be 10");
        }
    }

    /// Test browser checkout
    #[tokio::test]
    async fn test_browser_checkout() {
        let config = create_test_pool_config(2, 5);
        let pool = BrowserPool::new(config).await.unwrap();

        let checkout = pool.checkout().await;
        assert!(checkout.is_ok(), "Checkout should succeed");

        let stats = pool.stats().await;
        assert_eq!(stats.available, 1, "One browser should remain available");
        assert_eq!(stats.in_use, 1, "One browser should be in use");
    }

    /// Test browser checkin
    #[tokio::test]
    async fn test_browser_checkin() {
        let config = create_test_pool_config(2, 5);
        let pool = BrowserPool::new(config).await.unwrap();

        let checkout = pool.checkout().await.unwrap();
        let browser_id = checkout.id();

        // Check in the browser
        checkout.checkin().await.unwrap();

        // Give time for async checkin to complete
        tokio::time::sleep(Duration::from_millis(100)).await;

        let stats = pool.stats().await;
        assert_eq!(stats.available, 2, "Browser should be returned to pool");
        assert_eq!(stats.in_use, 0, "No browsers should be in use");
    }

    /// Test concurrent checkouts
    #[tokio::test]
    async fn test_concurrent_checkouts() {
        let config = create_test_pool_config(5, 10);
        let pool = std::sync::Arc::new(BrowserPool::new(config).await.unwrap());

        let tasks: Vec<_> = (0..5)
            .map(|i| {
                let pool_clone = std::sync::Arc::clone(&pool);
                tokio::spawn(async move {
                    let checkout = pool_clone.checkout().await;
                    assert!(
                        checkout.is_ok(),
                        "Concurrent checkout {} should succeed",
                        i
                    );
                    checkout.unwrap()
                })
            })
            .collect();

        let checkouts: Vec<_> = futures::future::join_all(tasks)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        let stats = pool.stats().await;
        assert_eq!(stats.in_use, 5, "All browsers should be checked out");
        assert_eq!(stats.available, 0, "No browsers should be available");

        // Clean up
        for checkout in checkouts {
            let _ = checkout.checkin().await;
        }
    }

    /// Test pool expansion when all browsers are in use
    #[tokio::test]
    async fn test_pool_expansion() {
        let config = create_test_pool_config(2, 5);
        let pool = BrowserPool::new(config).await.unwrap();

        // Checkout all initial browsers
        let _checkout1 = pool.checkout().await.unwrap();
        let _checkout2 = pool.checkout().await.unwrap();

        // This should trigger pool expansion
        let checkout3 = pool.checkout().await;
        assert!(checkout3.is_ok(), "Should create new browser on demand");

        let stats = pool.stats().await;
        assert_eq!(stats.in_use, 3, "Three browsers should be in use");
    }

    /// Test maximum pool size enforcement
    #[tokio::test]
    async fn test_max_pool_size() {
        let config = create_test_pool_config(1, 2);
        let pool = BrowserPool::new(config).await.unwrap();

        let _checkout1 = pool.checkout().await.unwrap();
        let _checkout2 = pool.checkout().await.unwrap();

        // This should block or timeout as max size is reached
        let checkout3_result = tokio::time::timeout(
            Duration::from_secs(1),
            pool.checkout()
        ).await;

        assert!(
            checkout3_result.is_err(),
            "Should timeout when pool is at max capacity"
        );
    }

    /// Test browser health checks
    #[tokio::test]
    async fn test_browser_health_checks() {
        let config = create_test_pool_config(3, 5);
        let pool = BrowserPool::new(config).await.unwrap();

        // Perform health check
        let health_result = pool.health_check().await;
        assert!(health_result.is_ok(), "Health check should succeed");

        let health_status = health_result.unwrap();
        assert_eq!(
            health_status.healthy_count, 3,
            "All browsers should be healthy"
        );
        assert_eq!(health_status.unhealthy_count, 0, "No unhealthy browsers");
    }

    /// Test unhealthy browser removal
    #[tokio::test]
    async fn test_unhealthy_browser_removal() {
        let config = create_test_pool_config(3, 5);
        let pool = BrowserPool::new(config).await.unwrap();

        // Simulate browser becoming unhealthy
        pool.mark_browser_unhealthy("browser_1").await;

        // Health check should remove unhealthy browser
        let _ = pool.health_check().await;

        let stats = pool.stats().await;
        assert!(
            stats.available < 3,
            "Unhealthy browser should be removed"
        );
    }

    /// Test idle timeout cleanup
    #[tokio::test]
    async fn test_idle_timeout_cleanup() {
        let mut config = create_test_pool_config(2, 5);
        config.idle_timeout = Duration::from_millis(500);

        let pool = BrowserPool::new(config).await.unwrap();

        // Wait for idle timeout
        tokio::time::sleep(Duration::from_millis(600)).await;

        // Trigger cleanup
        pool.cleanup_idle_browsers().await;

        let stats = pool.stats().await;
        // Should maintain minimum pool size
        assert!(
            stats.available >= 1,
            "Should maintain minimum pool size"
        );
    }

    /// Test browser lifetime limits
    #[tokio::test]
    async fn test_browser_lifetime_limits() {
        let mut config = create_test_pool_config(2, 5);
        config.max_lifetime = Duration::from_millis(500);

        let pool = BrowserPool::new(config).await.unwrap();

        // Wait for browsers to expire
        tokio::time::sleep(Duration::from_millis(600)).await;

        // Trigger cleanup
        pool.cleanup_expired_browsers().await;

        let stats = pool.stats().await;
        // Expired browsers should be replaced
        assert_eq!(
            stats.available, 2,
            "Should replace expired browsers"
        );
    }

    /// Test graceful pool shutdown
    #[tokio::test]
    async fn test_graceful_shutdown() {
        let config = create_test_pool_config(3, 5);
        let pool = BrowserPool::new(config).await.unwrap();

        let _checkout = pool.checkout().await.unwrap();

        // Shutdown pool
        let shutdown_result = pool.shutdown().await;
        assert!(shutdown_result.is_ok(), "Shutdown should succeed");

        // All browsers should be cleaned up
        let stats = pool.stats().await;
        assert_eq!(stats.available, 0, "All browsers should be cleaned up");
        assert_eq!(stats.in_use, 0, "No browsers should remain in use");
    }

    /// Test pool statistics accuracy
    #[tokio::test]
    async fn test_pool_statistics() {
        let config = create_test_pool_config(3, 10);
        let pool = BrowserPool::new(config).await.unwrap();

        let stats = pool.stats().await;
        assert_eq!(stats.total_capacity, 10);
        assert_eq!(stats.utilization, 0.0, "Initial utilization should be 0");

        let _checkout1 = pool.checkout().await.unwrap();
        let _checkout2 = pool.checkout().await.unwrap();

        let stats = pool.stats().await;
        assert_eq!(stats.in_use, 2);
        assert_eq!(stats.utilization, 20.0, "Utilization should be 20%");
    }

    /// Test pool events monitoring
    #[tokio::test]
    async fn test_pool_events() {
        let config = create_test_pool_config(2, 5);
        let pool = BrowserPool::new(config).await.unwrap();

        let mut events = pool.subscribe_events();

        // Perform operations
        let checkout = pool.checkout().await.unwrap();
        let _ = checkout.checkin().await;

        // Check events
        let event1 = events.recv().await;
        assert!(event1.is_some(), "Should receive checkout event");

        let event2 = events.recv().await;
        assert!(event2.is_some(), "Should receive checkin event");
    }

    /// Test memory usage tracking
    #[tokio::test]
    async fn test_memory_usage_tracking() {
        let config = create_test_pool_config(3, 5);
        let pool = BrowserPool::new(config).await.unwrap();

        let memory_stats = pool.get_memory_stats().await;
        assert!(
            memory_stats.total_mb > 0,
            "Should track memory usage"
        );
        assert_eq!(
            memory_stats.browser_count, 3,
            "Should track browser count"
        );
    }

    /// Test pool recovery after crashes
    #[tokio::test]
    async fn test_crash_recovery() {
        let mut config = create_test_pool_config(2, 5);
        config.enable_recovery = true;

        let pool = BrowserPool::new(config).await.unwrap();

        // Simulate browser crash
        pool.simulate_browser_crash("browser_1").await;

        // Recovery should create replacement
        tokio::time::sleep(Duration::from_millis(100)).await;

        let stats = pool.stats().await;
        assert_eq!(
            stats.available, 2,
            "Should recover crashed browser"
        );
    }

    /// Test concurrent stress test
    #[tokio::test]
    async fn test_concurrent_stress() {
        let config = create_test_pool_config(5, 10);
        let pool = std::sync::Arc::new(BrowserPool::new(config).await.unwrap());

        let tasks: Vec<_> = (0..50)
            .map(|i| {
                let pool_clone = std::sync::Arc::clone(&pool);
                tokio::spawn(async move {
                    for _ in 0..5 {
                        let checkout = pool_clone.checkout().await;
                        if let Ok(checkout) = checkout {
                            // Simulate work
                            tokio::time::sleep(Duration::from_millis(10)).await;
                            let _ = checkout.checkin().await;
                        }
                    }
                })
            })
            .collect();

        // Wait for all tasks
        for task in tasks {
            task.await.expect("Task should complete");
        }

        let stats = pool.stats().await;
        assert_eq!(stats.in_use, 0, "All browsers should be returned");
    }

    /// Test pool configuration validation
    #[test]
    fn test_pool_config_validation() {
        // Invalid: min > max
        let config = BrowserPoolConfig {
            min_pool_size: 10,
            max_pool_size: 5,
            initial_pool_size: 3,
            idle_timeout: Duration::from_secs(30),
            max_lifetime: Duration::from_secs(300),
            enable_recovery: true,
        };

        let validation = config.validate();
        assert!(validation.is_err(), "Should reject invalid config");
    }

    // Helper functions and types

    fn create_test_pool_config(initial_size: usize, max_size: usize) -> BrowserPoolConfig {
        BrowserPoolConfig {
            min_pool_size: 1,
            max_pool_size: max_size,
            initial_pool_size: initial_size,
            idle_timeout: Duration::from_secs(60),
            max_lifetime: Duration::from_secs(300),
            enable_recovery: true,
        }
    }

    // Mock BrowserPool implementation
    struct BrowserPool {
        config: BrowserPoolConfig,
        browsers: std::sync::Arc<tokio::sync::Mutex<Vec<Browser>>>,
        in_use: std::sync::Arc<tokio::sync::Mutex<Vec<Browser>>>,
    }

    struct BrowserPoolConfig {
        min_pool_size: usize,
        max_pool_size: usize,
        initial_pool_size: usize,
        idle_timeout: Duration,
        max_lifetime: Duration,
        enable_recovery: bool,
    }

    impl BrowserPoolConfig {
        fn validate(&self) -> Result<(), String> {
            if self.min_pool_size > self.max_pool_size {
                return Err("min_pool_size cannot exceed max_pool_size".to_string());
            }
            Ok(())
        }
    }

    struct Browser {
        id: String,
        created_at: std::time::Instant,
        last_used: std::time::Instant,
        healthy: bool,
    }

    struct BrowserCheckout {
        id: String,
        pool: std::sync::Arc<BrowserPool>,
    }

    #[derive(Debug)]
    struct PoolStats {
        available: usize,
        in_use: usize,
        total_capacity: usize,
        utilization: f64,
    }

    #[derive(Debug)]
    struct HealthStatus {
        healthy_count: usize,
        unhealthy_count: usize,
    }

    #[derive(Debug)]
    struct MemoryStats {
        total_mb: usize,
        browser_count: usize,
    }

    impl BrowserPool {
        async fn new(config: BrowserPoolConfig) -> Result<Self, String> {
            config.validate()?;

            let browsers: Vec<Browser> = (0..config.initial_pool_size)
                .map(|i| Browser {
                    id: format!("browser_{}", i),
                    created_at: std::time::Instant::now(),
                    last_used: std::time::Instant::now(),
                    healthy: true,
                })
                .collect();

            Ok(Self {
                config,
                browsers: std::sync::Arc::new(tokio::sync::Mutex::new(browsers)),
                in_use: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
            })
        }

        async fn checkout(&self) -> Result<BrowserCheckout, String> {
            let mut browsers = self.browsers.lock().await;
            let mut in_use = self.in_use.lock().await;

            if browsers.is_empty() {
                if in_use.len() >= self.config.max_pool_size {
                    return Err("Pool at maximum capacity".to_string());
                }

                // Create new browser
                let browser = Browser {
                    id: format!("browser_{}", in_use.len()),
                    created_at: std::time::Instant::now(),
                    last_used: std::time::Instant::now(),
                    healthy: true,
                };
                let id = browser.id.clone();
                in_use.push(browser);

                return Ok(BrowserCheckout {
                    id,
                    pool: std::sync::Arc::new(Self {
                        config: self.config.clone(),
                        browsers: self.browsers.clone(),
                        in_use: self.in_use.clone(),
                    }),
                });
            }

            let browser = browsers.pop().unwrap();
            let id = browser.id.clone();
            in_use.push(browser);

            Ok(BrowserCheckout {
                id,
                pool: std::sync::Arc::new(Self {
                    config: self.config.clone(),
                    browsers: self.browsers.clone(),
                    in_use: self.in_use.clone(),
                }),
            })
        }

        async fn stats(&self) -> PoolStats {
            let browsers = self.browsers.lock().await;
            let in_use = self.in_use.lock().await;

            PoolStats {
                available: browsers.len(),
                in_use: in_use.len(),
                total_capacity: self.config.max_pool_size,
                utilization: (in_use.len() as f64 / self.config.max_pool_size as f64) * 100.0,
            }
        }

        async fn health_check(&self) -> Result<HealthStatus, String> {
            let browsers = self.browsers.lock().await;
            let healthy = browsers.iter().filter(|b| b.healthy).count();
            Ok(HealthStatus {
                healthy_count: healthy,
                unhealthy_count: browsers.len() - healthy,
            })
        }

        async fn mark_browser_unhealthy(&self, _id: &str) {
            // Mock implementation
        }

        async fn cleanup_idle_browsers(&self) {
            // Mock implementation
        }

        async fn cleanup_expired_browsers(&self) {
            // Mock implementation
        }

        async fn shutdown(&self) -> Result<(), String> {
            let mut browsers = self.browsers.lock().await;
            let mut in_use = self.in_use.lock().await;
            browsers.clear();
            in_use.clear();
            Ok(())
        }

        fn subscribe_events(&self) -> EventReceiver {
            EventReceiver::new()
        }

        async fn get_memory_stats(&self) -> MemoryStats {
            let browsers = self.browsers.lock().await;
            let in_use = self.in_use.lock().await;
            MemoryStats {
                total_mb: (browsers.len() + in_use.len()) * 100, // 100MB per browser estimate
                browser_count: browsers.len() + in_use.len(),
            }
        }

        async fn simulate_browser_crash(&self, _id: &str) {
            // Mock implementation
        }
    }

    impl BrowserCheckout {
        fn id(&self) -> String {
            self.id.clone()
        }

        async fn checkin(self) -> Result<(), String> {
            let mut browsers = self.pool.browsers.lock().await;
            let mut in_use = self.pool.in_use.lock().await;

            if let Some(pos) = in_use.iter().position(|b| b.id == self.id) {
                let mut browser = in_use.remove(pos);
                browser.last_used = std::time::Instant::now();
                browsers.push(browser);
            }

            Ok(())
        }
    }

    struct EventReceiver {
        events: tokio::sync::mpsc::UnboundedReceiver<PoolEvent>,
    }

    impl EventReceiver {
        fn new() -> Self {
            let (_tx, rx) = tokio::sync::mpsc::unbounded_channel();
            Self { events: rx }
        }

        async fn recv(&mut self) -> Option<PoolEvent> {
            self.events.recv().await
        }
    }

    #[derive(Debug)]
    enum PoolEvent {
        BrowserCheckedOut(String),
        BrowserCheckedIn(String),
    }

    impl Clone for BrowserPoolConfig {
        fn clone(&self) -> Self {
            Self {
                min_pool_size: self.min_pool_size,
                max_pool_size: self.max_pool_size,
                initial_pool_size: self.initial_pool_size,
                idle_timeout: self.idle_timeout,
                max_lifetime: self.max_lifetime,
                enable_recovery: self.enable_recovery,
            }
        }
    }
}
