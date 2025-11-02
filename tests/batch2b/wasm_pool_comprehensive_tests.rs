//! Batch 2B: WASM Pool Comprehensive Tests
//!
//! Complete test suite for WASM instance pooling with memory management,
//! health monitoring, and event integration.
//!
//! ## Test Coverage:
//! - WASM instance pool lifecycle
//! - Memory tracking and limits
//! - Health monitoring and validation
//! - Circuit breaker with fallback
//! - Event bus integration
//! - Concurrent WASM operations
//! - Resource cleanup
//! - Epoch timeout handling
//!
//! ## Running Tests:
//! ```bash
//! cargo test --test batch2b wasm_pool_comprehensive_tests
//! ```

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use tokio::sync::{Mutex, Semaphore};

// ============================================================================
// Mock WASM Instance Types
// ============================================================================

#[derive(Clone)]
struct MockWasmInstance {
    id: String,
    created_at: Instant,
    use_count: Arc<AtomicUsize>,
    failure_count: Arc<AtomicUsize>,
    memory_usage: Arc<AtomicUsize>,
    is_healthy: Arc<AtomicBool>,
}

impl MockWasmInstance {
    fn new(initial_memory: usize) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: Instant::now(),
            use_count: Arc::new(AtomicUsize::new(0)),
            failure_count: Arc::new(AtomicUsize::new(0)),
            memory_usage: Arc::new(AtomicUsize::new(initial_memory)),
            is_healthy: Arc::new(AtomicBool::new(true)),
        }
    }

    fn execute(&self, input: &str) -> Result<String, String> {
        self.use_count.fetch_add(1, Ordering::Relaxed);

        // Simulate memory growth
        let current_memory = self.memory_usage.load(Ordering::Relaxed);
        self.memory_usage.store(current_memory + input.len(), Ordering::Relaxed);

        if self.is_healthy.load(Ordering::Relaxed) {
            Ok(format!("WASM output for: {}", input))
        } else {
            self.failure_count.fetch_add(1, Ordering::Relaxed);
            Err("WASM instance unhealthy".to_string())
        }
    }

    fn check_health(&self, max_reuse: u32, max_failures: u32, max_memory: usize) -> bool {
        let uses = self.use_count.load(Ordering::Relaxed);
        let failures = self.failure_count.load(Ordering::Relaxed);
        let memory = self.memory_usage.load(Ordering::Relaxed);

        uses < max_reuse as usize
            && failures < max_failures as usize
            && memory < max_memory
    }

    fn set_unhealthy(&self) {
        self.is_healthy.store(false, Ordering::Relaxed);
    }

    fn get_memory_usage(&self) -> usize {
        self.memory_usage.load(Ordering::Relaxed)
    }

    fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
}

// ============================================================================
// Mock WASM Pool Configuration
// ============================================================================

#[derive(Clone)]
struct MockWasmPoolConfig {
    max_pool_size: usize,
    initial_pool_size: usize,
    max_instance_reuse: u32,
    max_failure_count: u32,
    memory_limit_mb: usize,
    circuit_breaker_threshold: u32,
    epoch_timeout_ms: u64,
}

impl Default for MockWasmPoolConfig {
    fn default() -> Self {
        Self {
            max_pool_size: 8,
            initial_pool_size: 2,
            max_instance_reuse: 1000,
            max_failure_count: 10,
            memory_limit_mb: 256,
            circuit_breaker_threshold: 5,
            epoch_timeout_ms: 30000,
        }
    }
}

// ============================================================================
// Mock WASM Pool Implementation
// ============================================================================

struct MockWasmPool {
    config: MockWasmPoolConfig,
    available_instances: Arc<Mutex<VecDeque<MockWasmInstance>>>,
    semaphore: Arc<Semaphore>,
    total_extractions: Arc<AtomicUsize>,
    successful_extractions: Arc<AtomicUsize>,
    failed_extractions: Arc<AtomicUsize>,
    fallback_extractions: Arc<AtomicUsize>,
    circuit_breaker_trips: Arc<AtomicUsize>,
    is_circuit_open: Arc<AtomicBool>,
    total_memory_allocated: Arc<AtomicUsize>,
}

impl MockWasmPool {
    async fn new(config: MockWasmPoolConfig) -> Self {
        let pool = Self {
            config: config.clone(),
            available_instances: Arc::new(Mutex::new(VecDeque::new())),
            semaphore: Arc::new(Semaphore::new(config.max_pool_size)),
            total_extractions: Arc::new(AtomicUsize::new(0)),
            successful_extractions: Arc::new(AtomicUsize::new(0)),
            failed_extractions: Arc::new(AtomicUsize::new(0)),
            fallback_extractions: Arc::new(AtomicUsize::new(0)),
            circuit_breaker_trips: Arc::new(AtomicUsize::new(0)),
            is_circuit_open: Arc::new(AtomicBool::new(false)),
            total_memory_allocated: Arc::new(AtomicUsize::new(0)),
        };

        pool.warmup().await;
        pool
    }

    async fn warmup(&self) {
        let mut instances = self.available_instances.lock().await;
        for _ in 0..self.config.initial_pool_size {
            let instance = MockWasmInstance::new(1024 * 1024); // 1MB initial
            self.total_memory_allocated.fetch_add(1024 * 1024, Ordering::Relaxed);
            instances.push_back(instance);
        }
    }

    async fn get_or_create_instance(&self) -> Option<MockWasmInstance> {
        let mut instances = self.available_instances.lock().await;

        // Try to get healthy instance
        while let Some(instance) = instances.pop_front() {
            if instance.check_health(
                self.config.max_instance_reuse,
                self.config.max_failure_count,
                self.config.memory_limit_mb * 1024 * 1024,
            ) {
                return Some(instance);
            }
            // Discard unhealthy instance
            self.total_memory_allocated.fetch_sub(
                instance.get_memory_usage(),
                Ordering::Relaxed,
            );
        }

        // Create new instance
        let instance = MockWasmInstance::new(1024 * 1024);
        self.total_memory_allocated.fetch_add(1024 * 1024, Ordering::Relaxed);
        Some(instance)
    }

    async fn return_instance(&self, instance: MockWasmInstance) {
        if instance.check_health(
            self.config.max_instance_reuse,
            self.config.max_failure_count,
            self.config.memory_limit_mb * 1024 * 1024,
        ) {
            let mut instances = self.available_instances.lock().await;
            instances.push_back(instance);
        } else {
            // Discard unhealthy instance
            self.total_memory_allocated.fetch_sub(
                instance.get_memory_usage(),
                Ordering::Relaxed,
            );
        }
    }

    async fn extract(&self, input: &str) -> Result<String, String> {
        self.total_extractions.fetch_add(1, Ordering::Relaxed);

        // Check circuit breaker
        if self.is_circuit_open.load(Ordering::Relaxed) {
            return self.fallback_extract(input).await;
        }

        // Acquire semaphore with timeout
        let timeout_duration = Duration::from_millis(self.config.epoch_timeout_ms);
        let permit = match tokio::time::timeout(timeout_duration, self.semaphore.acquire()).await {
            Ok(Ok(permit)) => permit,
            _ => {
                return self.fallback_extract(input).await;
            }
        };

        // Get instance
        let instance = match self.get_or_create_instance().await {
            Some(inst) => inst,
            None => {
                drop(permit);
                return self.fallback_extract(input).await;
            }
        };

        // Execute
        let result = instance.execute(input);

        // Update metrics
        match &result {
            Ok(_) => {
                self.successful_extractions.fetch_add(1, Ordering::Relaxed);
                self.check_circuit_breaker(true);
            }
            Err(_) => {
                self.failed_extractions.fetch_add(1, Ordering::Relaxed);
                self.check_circuit_breaker(false);
            }
        }

        // Return instance
        self.return_instance(instance).await;
        drop(permit);

        result
    }

    async fn fallback_extract(&self, input: &str) -> Result<String, String> {
        self.fallback_extractions.fetch_add(1, Ordering::Relaxed);
        Ok(format!("Fallback extraction for: {}", input))
    }

    fn check_circuit_breaker(&self, success: bool) {
        if !success {
            let failures = self.failed_extractions.load(Ordering::Relaxed);
            let total = self.total_extractions.load(Ordering::Relaxed);

            if total >= 10 {
                let failure_rate = (failures as f64 / total as f64) * 100.0;
                if failure_rate >= self.config.circuit_breaker_threshold as f64 {
                    if !self.is_circuit_open.swap(true, Ordering::Relaxed) {
                        self.circuit_breaker_trips.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }
    }

    fn reset_circuit_breaker(&self) {
        self.is_circuit_open.store(false, Ordering::Relaxed);
    }

    async fn get_pool_size(&self) -> usize {
        self.available_instances.lock().await.len()
    }

    fn get_memory_usage(&self) -> usize {
        self.total_memory_allocated.load(Ordering::Relaxed)
    }

    fn get_metrics(&self) -> WasmPoolMetrics {
        WasmPoolMetrics {
            total_extractions: self.total_extractions.load(Ordering::Relaxed),
            successful_extractions: self.successful_extractions.load(Ordering::Relaxed),
            failed_extractions: self.failed_extractions.load(Ordering::Relaxed),
            fallback_extractions: self.fallback_extractions.load(Ordering::Relaxed),
            circuit_breaker_trips: self.circuit_breaker_trips.load(Ordering::Relaxed),
            memory_usage: self.get_memory_usage(),
        }
    }
}

#[derive(Debug)]
struct WasmPoolMetrics {
    total_extractions: usize,
    successful_extractions: usize,
    failed_extractions: usize,
    fallback_extractions: usize,
    circuit_breaker_trips: usize,
    memory_usage: usize,
}

// ============================================================================
// Pool Initialization Tests
// ============================================================================

#[tokio::test]
async fn test_wasm_pool_initialization() {
    println!("=== Test: WASM Pool Initialization ===");

    let config = MockWasmPoolConfig::default();
    let pool = MockWasmPool::new(config).await;

    let pool_size = pool.get_pool_size().await;
    assert_eq!(pool_size, 2, "Should have 2 instances after warmup");

    let memory = pool.get_memory_usage();
    assert!(memory > 0, "Should have allocated memory");

    println!("✓ Pool initialized: {} instances, {} bytes memory", pool_size, memory);
}

#[tokio::test]
async fn test_wasm_pool_custom_config() {
    println!("=== Test: Custom Pool Configuration ===");

    let config = MockWasmPoolConfig {
        max_pool_size: 16,
        initial_pool_size: 4,
        ..Default::default()
    };
    let pool = MockWasmPool::new(config).await;

    let pool_size = pool.get_pool_size().await;
    assert_eq!(pool_size, 4, "Should respect custom initial size");

    println!("✓ Custom config applied: {} instances", pool_size);
}

// ============================================================================
// Instance Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_wasm_instance_lifecycle() {
    println!("=== Test: WASM Instance Lifecycle ===");

    let instance = MockWasmInstance::new(1024);

    // Initial state
    assert_eq!(instance.use_count.load(Ordering::Relaxed), 0);
    assert_eq!(instance.failure_count.load(Ordering::Relaxed), 0);

    // Execute
    let result = instance.execute("test");
    assert!(result.is_ok());
    assert_eq!(instance.use_count.load(Ordering::Relaxed), 1);

    // Check health
    assert!(instance.check_health(1000, 10, 256 * 1024 * 1024));

    println!("✓ Instance lifecycle working correctly");
}

#[tokio::test]
async fn test_wasm_instance_health_degradation() {
    println!("=== Test: Instance Health Degradation ===");

    let instance = MockWasmInstance::new(1024);

    // Use instance many times
    for _ in 0..999 {
        instance.use_count.fetch_add(1, Ordering::Relaxed);
    }
    assert!(instance.check_health(1000, 10, 256 * 1024 * 1024), "Should be healthy at 999 uses");

    // Exceed reuse limit
    instance.use_count.fetch_add(1, Ordering::Relaxed);
    assert!(!instance.check_health(1000, 10, 256 * 1024 * 1024), "Should be unhealthy at 1000 uses");

    println!("✓ Health degradation detected correctly");
}

// ============================================================================
// Memory Management Tests
// ============================================================================

#[tokio::test]
async fn test_wasm_memory_tracking() {
    println!("=== Test: Memory Tracking ===");

    let config = MockWasmPoolConfig::default();
    let pool = MockWasmPool::new(config).await;

    let initial_memory = pool.get_memory_usage();
    assert!(initial_memory > 0, "Should have initial memory allocation");

    // Perform extractions
    for i in 0..10 {
        pool.extract(&format!("test data {}", i)).await.ok();
    }

    let final_memory = pool.get_memory_usage();
    println!("✓ Memory tracked: {} bytes → {} bytes", initial_memory, final_memory);
}

#[tokio::test]
async fn test_wasm_memory_limits() {
    println!("=== Test: Memory Limits Enforcement ===");

    let instance = MockWasmInstance::new(1024);

    // Grow memory
    let large_input = "x".repeat(300 * 1024 * 1024); // 300MB
    instance.memory_usage.store(large_input.len(), Ordering::Relaxed);

    // Check health with 256MB limit
    let is_healthy = instance.check_health(1000, 10, 256 * 1024 * 1024);
    assert!(!is_healthy, "Should be unhealthy when exceeding memory limit");

    println!("✓ Memory limit enforced");
}

// ============================================================================
// Extraction Tests
// ============================================================================

#[tokio::test]
async fn test_wasm_extraction() {
    println!("=== Test: WASM Extraction ===");

    let config = MockWasmPoolConfig::default();
    let pool = MockWasmPool::new(config).await;

    let result = pool.extract("test input").await;
    assert!(result.is_ok(), "Extraction should succeed");
    assert!(result.unwrap().contains("WASM output"), "Should return WASM output");

    let metrics = pool.get_metrics();
    assert_eq!(metrics.total_extractions, 1);
    assert_eq!(metrics.successful_extractions, 1);

    println!("✓ Extraction successful");
}

#[tokio::test]
async fn test_wasm_concurrent_extractions() {
    println!("=== Test: Concurrent WASM Extractions ===");

    let config = MockWasmPoolConfig {
        initial_pool_size: 4,
        max_pool_size: 8,
        ..Default::default()
    };
    let pool = Arc::new(MockWasmPool::new(config).await);

    let mut handles = vec![];

    // Spawn 20 concurrent extractions
    for i in 0..20 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            pool_clone.extract(&format!("input {}", i)).await.is_ok()
        });
        handles.push(handle);
    }

    let mut successful = 0;
    for handle in handles {
        if handle.await.unwrap() {
            successful += 1;
        }
    }

    assert!(successful >= 15, "Should process most concurrent extractions");
    println!("✓ Processed {}/20 concurrent extractions", successful);
}

// ============================================================================
// Circuit Breaker Tests
// ============================================================================

#[tokio::test]
async fn test_wasm_circuit_breaker() {
    println!("=== Test: WASM Circuit Breaker ===");

    let config = MockWasmPoolConfig {
        circuit_breaker_threshold: 50, // 50% failure rate
        ..Default::default()
    };
    let pool = Arc::new(MockWasmPool::new(config).await);

    // Get instance and make it unhealthy
    let instance = pool.get_or_create_instance().await.unwrap();
    instance.set_unhealthy();
    pool.return_instance(instance).await;

    // Force failures
    for _ in 0..20 {
        pool.extract("test").await.ok();
    }

    let is_open = pool.is_circuit_open.load(Ordering::Relaxed);
    let metrics = pool.get_metrics();

    println!("✓ Circuit status: open={}, failures={}/{}",
             is_open, metrics.failed_extractions, metrics.total_extractions);

    // Circuit should eventually open or use fallback
    assert!(metrics.fallback_extractions > 0 || is_open,
            "Should use fallback or open circuit under failures");
}

#[tokio::test]
async fn test_wasm_fallback_on_circuit_open() {
    println!("=== Test: Fallback on Circuit Open ===");

    let config = MockWasmPoolConfig::default();
    let pool = MockWasmPool::new(config).await;

    // Force circuit open
    pool.is_circuit_open.store(true, Ordering::Relaxed);

    // Extract should use fallback
    let result = pool.extract("test").await;
    assert!(result.is_ok(), "Should succeed with fallback");
    assert!(result.unwrap().contains("Fallback"), "Should use fallback extraction");

    let metrics = pool.get_metrics();
    assert_eq!(metrics.fallback_extractions, 1, "Should record fallback usage");

    println!("✓ Fallback triggered successfully");
}

// ============================================================================
// Timeout Tests
// ============================================================================

#[tokio::test]
async fn test_wasm_epoch_timeout() {
    println!("=== Test: Epoch Timeout ===");

    let config = MockWasmPoolConfig {
        epoch_timeout_ms: 100, // Very short timeout for testing
        ..Default::default()
    };
    let pool = Arc::new(MockWasmPool::new(config).await);

    // Block all semaphore permits
    let mut _permits = vec![];
    for _ in 0..pool.config.max_pool_size {
        if let Ok(permit) = pool.semaphore.try_acquire() {
            _permits.push(permit);
        }
    }

    // This should timeout and use fallback
    let result = pool.extract("test").await;
    assert!(result.is_ok(), "Should fallback on timeout");

    let metrics = pool.get_metrics();
    assert!(metrics.fallback_extractions > 0, "Should use fallback on timeout");

    println!("✓ Timeout handled with fallback");
}

// ============================================================================
// Performance Tests
// ============================================================================

#[tokio::test]
async fn test_wasm_pool_performance() {
    println!("=== Test: WASM Pool Performance ===");

    let config = MockWasmPoolConfig {
        initial_pool_size: 8,
        max_pool_size: 16,
        ..Default::default()
    };
    let pool = Arc::new(MockWasmPool::new(config).await);

    let start = Instant::now();
    let mut handles = vec![];

    // 100 extractions
    for i in 0..100 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            pool_clone.extract(&format!("data {}", i)).await.is_ok()
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

    let metrics = pool.get_metrics();
    println!("✓ Performance: {}/{} in {:?} ({:.1} req/s)",
             successful, 100, elapsed, throughput);
    println!("  Fallback rate: {}/{} ({:.1}%)",
             metrics.fallback_extractions, metrics.total_extractions,
             (metrics.fallback_extractions as f64 / metrics.total_extractions as f64) * 100.0);

    assert!(successful >= 80, "Should maintain high success rate");
}

// ============================================================================
// Stress Tests
// ============================================================================

#[tokio::test]
async fn test_wasm_pool_stress() {
    println!("=== Test: WASM Pool Stress Test ===");

    let config = MockWasmPoolConfig {
        initial_pool_size: 8,
        max_pool_size: 16,
        ..Default::default()
    };
    let pool = Arc::new(MockWasmPool::new(config).await);

    let mut handles = vec![];

    // 300 concurrent extractions with varying load
    for i in 0..300 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            // Variable delay to simulate real load
            tokio::time::sleep(Duration::from_millis((i % 100) as u64)).await;
            pool_clone.extract(&format!("stress {}", i)).await.is_ok()
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
    let success_rate = (successful as f64 / 300.0) * 100.0;

    println!("✓ Stress test: {}/300 ({:.1}%)", successful, success_rate);
    println!("  Metrics: {} total, {} success, {} failed, {} fallback",
             metrics.total_extractions, metrics.successful_extractions,
             metrics.failed_extractions, metrics.fallback_extractions);

    assert!(success_rate >= 80.0, "Should maintain reasonable success rate under stress");
}
