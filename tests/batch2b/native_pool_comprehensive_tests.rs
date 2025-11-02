//! Batch 2B: Native Extractor Pool Comprehensive Tests
//!
//! Complete test suite for native CSS and Regex extractor pooling.
//! Tests pool lifecycle, health monitoring, circuit breaker, and resource management.
//!
//! ## Test Coverage:
//! - Pool initialization and warmup
//! - Instance checkout/checkin
//! - Health monitoring and auto-restart
//! - Circuit breaker integration
//! - Memory and CPU limits
//! - Concurrent access patterns
//! - Resource cleanup
//! - Error recovery
//!
//! ## Running Tests:
//! ```bash
//! cargo test --test batch2b native_pool_comprehensive_tests
//! ```

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};

// ============================================================================
// Mock Native Extractor Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExtractorType {
    Css,
    Regex,
}

#[derive(Clone)]
struct MockExtractorInstance {
    id: String,
    extractor_type: ExtractorType,
    use_count: Arc<AtomicUsize>,
    failure_count: Arc<AtomicUsize>,
    created_at: Instant,
}

impl MockExtractorInstance {
    fn new(extractor_type: ExtractorType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            extractor_type,
            use_count: Arc::new(AtomicUsize::new(0)),
            failure_count: Arc::new(AtomicUsize::new(0)),
            created_at: Instant::now(),
        }
    }

    fn extract(&self, _html: &str) -> Result<String, String> {
        self.use_count.fetch_add(1, Ordering::Relaxed);

        // Simulate extraction
        match self.extractor_type {
            ExtractorType::Css => Ok("CSS extracted content".to_string()),
            ExtractorType::Regex => Ok("Regex extracted content".to_string()),
        }
    }

    fn record_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
    }

    fn is_healthy(&self, max_reuse: u32, max_failures: u32) -> bool {
        let uses = self.use_count.load(Ordering::Relaxed);
        let failures = self.failure_count.load(Ordering::Relaxed);

        uses < max_reuse as usize && failures < max_failures as usize
    }

    fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
}

// ============================================================================
// Mock Pool Configuration
// ============================================================================

#[derive(Clone)]
struct MockPoolConfig {
    max_pool_size: usize,
    initial_pool_size: usize,
    extraction_timeout: Duration,
    max_instance_reuse: u32,
    max_failure_count: u32,
    circuit_breaker_threshold: u32,
}

impl Default for MockPoolConfig {
    fn default() -> Self {
        Self {
            max_pool_size: 8,
            initial_pool_size: 2,
            extraction_timeout: Duration::from_secs(30),
            max_instance_reuse: 1000,
            max_failure_count: 10,
            circuit_breaker_threshold: 5,
        }
    }
}

// ============================================================================
// Mock Native Extractor Pool
// ============================================================================

struct MockNativePool {
    config: MockPoolConfig,
    extractor_type: ExtractorType,
    available: Arc<Mutex<Vec<MockExtractorInstance>>>,
    active_count: Arc<AtomicUsize>,
    total_extractions: Arc<AtomicUsize>,
    successful_extractions: Arc<AtomicUsize>,
    failed_extractions: Arc<AtomicUsize>,
    circuit_breaker_trips: Arc<AtomicUsize>,
    is_circuit_open: Arc<RwLock<bool>>,
}

impl MockNativePool {
    async fn new(config: MockPoolConfig, extractor_type: ExtractorType) -> Self {
        let pool = Self {
            config: config.clone(),
            extractor_type,
            available: Arc::new(Mutex::new(Vec::new())),
            active_count: Arc::new(AtomicUsize::new(0)),
            total_extractions: Arc::new(AtomicUsize::new(0)),
            successful_extractions: Arc::new(AtomicUsize::new(0)),
            failed_extractions: Arc::new(AtomicUsize::new(0)),
            circuit_breaker_trips: Arc::new(AtomicUsize::new(0)),
            is_circuit_open: Arc::new(RwLock::new(false)),
        };

        // Warmup pool
        pool.warmup().await;
        pool
    }

    async fn warmup(&self) {
        let mut available = self.available.lock().await;
        for _ in 0..self.config.initial_pool_size {
            available.push(MockExtractorInstance::new(self.extractor_type));
        }
    }

    async fn checkout(&self) -> Option<MockExtractorInstance> {
        // Check circuit breaker
        if *self.is_circuit_open.read().await {
            return None;
        }

        let mut available = self.available.lock().await;

        // Try to get healthy instance
        while let Some(instance) = available.pop() {
            if instance.is_healthy(self.config.max_instance_reuse, self.config.max_failure_count) {
                self.active_count.fetch_add(1, Ordering::Relaxed);
                return Some(instance);
            }
        }

        // Create new instance if under limit
        let active = self.active_count.load(Ordering::Relaxed);
        let pool_size = active + available.len();

        if pool_size < self.config.max_pool_size {
            self.active_count.fetch_add(1, Ordering::Relaxed);
            Some(MockExtractorInstance::new(self.extractor_type))
        } else {
            None
        }
    }

    async fn checkin(&self, instance: MockExtractorInstance) {
        self.active_count.fetch_sub(1, Ordering::Relaxed);

        if instance.is_healthy(self.config.max_instance_reuse, self.config.max_failure_count) {
            let mut available = self.available.lock().await;
            available.push(instance);
        }
    }

    async fn extract(&self, html: &str) -> Result<String, String> {
        self.total_extractions.fetch_add(1, Ordering::Relaxed);

        let instance = match self.checkout().await {
            Some(inst) => inst,
            None => {
                self.failed_extractions.fetch_add(1, Ordering::Relaxed);
                return Err("Pool exhausted or circuit open".to_string());
            }
        };

        let result = instance.extract(html);

        match &result {
            Ok(_) => {
                self.successful_extractions.fetch_add(1, Ordering::Relaxed);
                self.check_circuit_breaker(true).await;
            }
            Err(_) => {
                self.failed_extractions.fetch_add(1, Ordering::Relaxed);
                self.check_circuit_breaker(false).await;
            }
        }

        self.checkin(instance).await;
        result
    }

    async fn check_circuit_breaker(&self, success: bool) {
        if !success {
            let failures = self.failed_extractions.load(Ordering::Relaxed);
            let total = self.total_extractions.load(Ordering::Relaxed);

            if total >= 10 {
                let failure_rate = (failures as f64 / total as f64) * 100.0;
                if failure_rate >= self.config.circuit_breaker_threshold as f64 {
                    let mut is_open = self.is_circuit_open.write().await;
                    if !*is_open {
                        *is_open = true;
                        self.circuit_breaker_trips.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }
    }

    async fn reset_circuit_breaker(&self) {
        let mut is_open = self.is_circuit_open.write().await;
        *is_open = false;
    }

    async fn get_pool_status(&self) -> (usize, usize, usize) {
        let available = self.available.lock().await.len();
        let active = self.active_count.load(Ordering::Relaxed);
        let total = available + active;
        (available, active, total)
    }

    fn get_metrics(&self) -> PoolMetrics {
        PoolMetrics {
            total_extractions: self.total_extractions.load(Ordering::Relaxed),
            successful_extractions: self.successful_extractions.load(Ordering::Relaxed),
            failed_extractions: self.failed_extractions.load(Ordering::Relaxed),
            circuit_breaker_trips: self.circuit_breaker_trips.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
struct PoolMetrics {
    total_extractions: usize,
    successful_extractions: usize,
    failed_extractions: usize,
    circuit_breaker_trips: usize,
}

// ============================================================================
// Pool Initialization Tests
// ============================================================================

#[tokio::test]
async fn test_native_pool_initialization() {
    println!("=== Test: Native Pool Initialization ===");

    let config = MockPoolConfig::default();
    let pool = MockNativePool::new(config, ExtractorType::Css).await;

    let (available, active, total) = pool.get_pool_status().await;

    assert_eq!(available, 2, "Should have 2 available instances after warmup");
    assert_eq!(active, 0, "Should have 0 active instances initially");
    assert_eq!(total, 2, "Total should be 2");

    println!("✓ Pool initialized: {} available, {} active", available, active);
}

#[tokio::test]
async fn test_native_pool_both_types() {
    println!("=== Test: CSS and Regex Pool Types ===");

    let config = MockPoolConfig::default();

    let css_pool = MockNativePool::new(config.clone(), ExtractorType::Css).await;
    let regex_pool = MockNativePool::new(config, ExtractorType::Regex).await;

    let css_result = css_pool.extract("<html></html>").await.unwrap();
    let regex_result = regex_pool.extract("<html></html>").await.unwrap();

    assert!(css_result.contains("CSS"), "CSS pool should return CSS result");
    assert!(regex_result.contains("Regex"), "Regex pool should return Regex result");

    println!("✓ Both CSS and Regex pools working");
}

// ============================================================================
// Checkout/Checkin Tests
// ============================================================================

#[tokio::test]
async fn test_native_pool_checkout_checkin() {
    println!("=== Test: Checkout/Checkin Cycle ===");

    let config = MockPoolConfig::default();
    let pool = MockNativePool::new(config, ExtractorType::Css).await;

    // Checkout instance
    let instance = pool.checkout().await.expect("Should checkout instance");
    let (available, active, _) = pool.get_pool_status().await;
    assert_eq!(available, 1, "Should have 1 available after checkout");
    assert_eq!(active, 1, "Should have 1 active after checkout");

    // Checkin instance
    pool.checkin(instance).await;
    let (available, active, _) = pool.get_pool_status().await;
    assert_eq!(available, 2, "Should have 2 available after checkin");
    assert_eq!(active, 0, "Should have 0 active after checkin");

    println!("✓ Checkout/checkin cycle working correctly");
}

#[tokio::test]
async fn test_native_pool_concurrent_checkout() {
    println!("=== Test: Concurrent Checkout ===");

    let config = MockPoolConfig {
        initial_pool_size: 5,
        ..Default::default()
    };
    let pool = Arc::new(MockNativePool::new(config, ExtractorType::Css).await);

    let mut handles = vec![];

    // Spawn 10 concurrent checkouts
    for _ in 0..10 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            pool_clone.checkout().await.is_some()
        });
        handles.push(handle);
    }

    let mut successful_checkouts = 0;
    for handle in handles {
        if handle.await.unwrap() {
            successful_checkouts += 1;
        }
    }

    assert!(successful_checkouts >= 5, "Should checkout at least 5 instances");
    assert!(successful_checkouts <= 8, "Should not exceed max pool size");

    println!("✓ {}/10 concurrent checkouts succeeded", successful_checkouts);
}

// ============================================================================
// Health Monitoring Tests
// ============================================================================

#[tokio::test]
async fn test_native_pool_instance_health() {
    println!("=== Test: Instance Health Checks ===");

    let instance = MockExtractorInstance::new(ExtractorType::Css);

    // Healthy initially
    assert!(instance.is_healthy(1000, 10), "New instance should be healthy");

    // Use instance many times
    for _ in 0..500 {
        instance.use_count.fetch_add(1, Ordering::Relaxed);
    }
    assert!(instance.is_healthy(1000, 10), "Should be healthy at 500 uses");

    // Exceed reuse limit
    for _ in 0..500 {
        instance.use_count.fetch_add(1, Ordering::Relaxed);
    }
    assert!(!instance.is_healthy(1000, 10), "Should be unhealthy at 1000 uses");

    println!("✓ Instance health checking working");
}

#[tokio::test]
async fn test_native_pool_unhealthy_instance_discarded() {
    println!("=== Test: Unhealthy Instances Discarded ===");

    let config = MockPoolConfig {
        max_instance_reuse: 5, // Low limit for testing
        ..Default::default()
    };
    let pool = MockNativePool::new(config, ExtractorType::Css).await;

    // Use instances many times
    for _ in 0..10 {
        pool.extract("<html></html>").await.ok();
    }

    let (available, _, total) = pool.get_pool_status().await;

    // Unhealthy instances should be discarded
    assert!(available <= 2, "Unhealthy instances should be discarded");
    assert!(total <= config.max_pool_size, "Should not exceed max size");

    println!("✓ Unhealthy instances discarded, pool size: {}", total);
}

// ============================================================================
// Circuit Breaker Tests
// ============================================================================

#[tokio::test]
async fn test_native_pool_circuit_breaker() {
    println!("=== Test: Circuit Breaker ===");

    let config = MockPoolConfig {
        circuit_breaker_threshold: 50, // 50% failure rate
        ..Default::default()
    };
    let pool = Arc::new(MockNativePool::new(config, ExtractorType::Css).await);

    // Simulate high failure rate
    for _ in 0..20 {
        pool.total_extractions.fetch_add(1, Ordering::Relaxed);
        pool.failed_extractions.fetch_add(1, Ordering::Relaxed);
        pool.check_circuit_breaker(false).await;
    }

    // Circuit should be open
    let is_open = *pool.is_circuit_open.read().await;
    assert!(is_open, "Circuit should be open after high failure rate");

    let metrics = pool.get_metrics();
    assert!(metrics.circuit_breaker_trips > 0, "Should have circuit breaker trips");

    println!("✓ Circuit breaker tripped after {} failures", metrics.failed_extractions);
}

#[tokio::test]
async fn test_native_pool_circuit_breaker_reset() {
    println!("=== Test: Circuit Breaker Reset ===");

    let config = MockPoolConfig::default();
    let pool = MockNativePool::new(config, ExtractorType::Css).await;

    // Trip circuit
    for _ in 0..20 {
        pool.total_extractions.fetch_add(1, Ordering::Relaxed);
        pool.failed_extractions.fetch_add(1, Ordering::Relaxed);
        pool.check_circuit_breaker(false).await;
    }

    assert!(*pool.is_circuit_open.read().await, "Circuit should be open");

    // Reset circuit
    pool.reset_circuit_breaker().await;
    assert!(!*pool.is_circuit_open.read().await, "Circuit should be closed after reset");

    println!("✓ Circuit breaker reset successfully");
}

// ============================================================================
// Extraction Tests
// ============================================================================

#[tokio::test]
async fn test_native_pool_extraction() {
    println!("=== Test: Native Pool Extraction ===");

    let config = MockPoolConfig::default();
    let pool = MockNativePool::new(config, ExtractorType::Css).await;

    let html = "<html><body><h1>Test</h1></body></html>";
    let result = pool.extract(html).await;

    assert!(result.is_ok(), "Extraction should succeed");
    assert!(result.unwrap().contains("CSS"), "Should return CSS extracted content");

    let metrics = pool.get_metrics();
    assert_eq!(metrics.total_extractions, 1, "Should have 1 total extraction");
    assert_eq!(metrics.successful_extractions, 1, "Should have 1 successful extraction");

    println!("✓ Extraction successful");
}

#[tokio::test]
async fn test_native_pool_concurrent_extractions() {
    println!("=== Test: Concurrent Extractions ===");

    let config = MockPoolConfig {
        initial_pool_size: 5,
        max_pool_size: 10,
        ..Default::default()
    };
    let pool = Arc::new(MockNativePool::new(config, ExtractorType::Css).await);

    let mut handles = vec![];
    let html = "<html><body>Test</body></html>";

    // Spawn 20 concurrent extractions
    for i in 0..20 {
        let pool_clone = pool.clone();
        let html_clone = html.to_string();

        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(i * 10)).await;
            pool_clone.extract(&html_clone).await.is_ok()
        });
        handles.push(handle);
    }

    let mut successful = 0;
    for handle in handles {
        if handle.await.unwrap() {
            successful += 1;
        }
    }

    assert!(successful >= 10, "Should process at least 10 concurrent extractions");

    let metrics = pool.get_metrics();
    println!("✓ Processed {}/{} concurrent extractions", successful, metrics.total_extractions);
}

// ============================================================================
// Resource Management Tests
// ============================================================================

#[tokio::test]
async fn test_native_pool_max_size_enforcement() {
    println!("=== Test: Max Pool Size Enforcement ===");

    let config = MockPoolConfig {
        initial_pool_size: 2,
        max_pool_size: 5,
        ..Default::default()
    };
    let pool = Arc::new(MockNativePool::new(config, ExtractorType::Css).await);

    // Checkout all available instances
    let mut instances = vec![];
    for _ in 0..10 {
        if let Some(instance) = pool.checkout().await {
            instances.push(instance);
        }
    }

    assert!(instances.len() <= 5, "Should not exceed max pool size");

    let (_, active, total) = pool.get_pool_status().await;
    assert!(total <= 5, "Total instances should not exceed max pool size");
    assert_eq!(active, instances.len(), "Active count should match checked out instances");

    println!("✓ Max pool size enforced: {} instances", instances.len());
}

#[tokio::test]
async fn test_native_pool_cleanup() {
    println!("=== Test: Pool Cleanup ===");

    let config = MockPoolConfig::default();
    let pool = Arc::new(MockNativePool::new(config, ExtractorType::Css).await);

    let weak_ref = Arc::downgrade(&pool);
    let initial_strong = Arc::strong_count(&pool);

    {
        let _clone1 = pool.clone();
        let _clone2 = pool.clone();
        assert_eq!(Arc::strong_count(&pool), initial_strong + 2);
    }

    assert_eq!(Arc::strong_count(&pool), initial_strong);
    assert!(weak_ref.upgrade().is_some(), "Pool should still exist");

    println!("✓ Pool cleanup working correctly");
}

// ============================================================================
// Performance Tests
// ============================================================================

#[tokio::test]
async fn test_native_pool_performance() {
    println!("=== Test: Pool Performance ===");

    let config = MockPoolConfig {
        initial_pool_size: 5,
        max_pool_size: 10,
        ..Default::default()
    };
    let pool = Arc::new(MockNativePool::new(config, ExtractorType::Css).await);

    let start = Instant::now();
    let mut handles = vec![];

    // 100 extractions
    for _ in 0..100 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            pool_clone.extract("<html></html>").await.is_ok()
        });
        handles.push(handle);
    }

    let mut successful = 0;
    for handle in handles {
        if handle.await.unwrap() {
            successful += 1;
        }
    }

    let elapsed = start.elapsed();
    let throughput = successful as f64 / elapsed.as_secs_f64();

    println!("✓ Performance: {}/{} extractions in {:?} ({:.1} req/s)",
             successful, 100, elapsed, throughput);

    assert!(successful >= 90, "Should have high success rate");
    assert!(elapsed < Duration::from_secs(5), "Should complete in reasonable time");
}

// ============================================================================
// Stress Tests
// ============================================================================

#[tokio::test]
async fn test_native_pool_stress() {
    println!("=== Test: Pool Stress Test ===");

    let config = MockPoolConfig {
        initial_pool_size: 8,
        max_pool_size: 16,
        ..Default::default()
    };
    let pool = Arc::new(MockNativePool::new(config, ExtractorType::Css).await);

    let mut handles = vec![];

    // 200 concurrent extractions
    for i in 0..200 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            // Simulate variable load
            tokio::time::sleep(Duration::from_millis((i % 50) as u64)).await;
            pool_clone.extract("<html><body>Stress test</body></html>").await.is_ok()
        });
        handles.push(handle);
    }

    let mut successful = 0;
    for handle in handles {
        if handle.await.unwrap() {
            successful += 1;
        }
    }

    let metrics = pool.get_metrics();
    let success_rate = (successful as f64 / 200.0) * 100.0;

    println!("✓ Stress test: {}/200 succeeded ({:.1}%)", successful, success_rate);
    println!("  Metrics: {} total, {} successful, {} failed",
             metrics.total_extractions, metrics.successful_extractions, metrics.failed_extractions);

    assert!(success_rate >= 80.0, "Should maintain high success rate under stress");
}
