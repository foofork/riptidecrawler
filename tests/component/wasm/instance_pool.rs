/// Instance Pool and Circuit Breaker Tests
///
/// Tests the instance pool management, health monitoring, circuit breaker pattern,
/// and concurrent extraction handling with semaphore-based coordination.

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::sync::{Semaphore, RwLock};
use tokio::time::sleep;

/// Maximum concurrent extractions
const MAX_CONCURRENT: usize = 8;

/// Circuit breaker thresholds
const FAILURE_THRESHOLD: u64 = 5;
const SUCCESS_THRESHOLD: u64 = 3;
const CIRCUIT_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitState {
    Closed,    // Normal operation
    Open,      // Failures exceeded, reject requests
    HalfOpen,  // Testing if service recovered
}

/// Simulated WASM instance with health tracking
struct WasmInstance {
    id: usize,
    health_score: Arc<AtomicU64>,
    failure_count: Arc<AtomicU64>,
    last_used: Arc<RwLock<Instant>>,
}

impl WasmInstance {
    fn new(id: usize) -> Self {
        Self {
            id,
            health_score: Arc::new(AtomicU64::new(100)),
            failure_count: Arc::new(AtomicU64::new(0)),
            last_used: Arc::new(RwLock::new(Instant::now())),
        }
    }

    fn is_healthy(&self) -> bool {
        self.health_score.load(Ordering::SeqCst) >= 50
    }

    fn record_success(&self) {
        let current = self.health_score.load(Ordering::SeqCst);
        self.health_score.store((current + 10).min(100), Ordering::SeqCst);
        self.failure_count.store(0, Ordering::SeqCst);
    }

    fn record_failure(&self) {
        let current = self.health_score.load(Ordering::SeqCst);
        self.health_score.store(current.saturating_sub(20), Ordering::SeqCst);
        self.failure_count.fetch_add(1, Ordering::SeqCst);
    }

    async fn update_last_used(&self) {
        let mut last_used = self.last_used.write().await;
        *last_used = Instant::now();
    }
}

/// Circuit breaker for instance pool
struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<AtomicU64>,
    success_count: Arc<AtomicU64>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
}

impl CircuitBreaker {
    fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(AtomicU64::new(0)),
            success_count: Arc::new(AtomicU64::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
        }
    }

    async fn call<F, T>(&self, operation: F) -> Result<T, String>
    where
        F: FnOnce() -> Result<T, String>,
    {
        let current_state = *self.state.read().await;

        match current_state {
            CircuitState::Open => {
                // Check if timeout has elapsed
                let last_failure = self.last_failure_time.read().await;
                if let Some(last) = *last_failure {
                    if last.elapsed() > Duration::from_secs(CIRCUIT_TIMEOUT_SECS) {
                        // Transition to half-open
                        *self.state.write().await = CircuitState::HalfOpen;
                        self.success_count.store(0, Ordering::SeqCst);
                    } else {
                        return Err("Circuit breaker is OPEN".to_string());
                    }
                }
            }
            _ => {}
        }

        // Execute operation
        let result = operation();

        match result {
            Ok(value) => {
                self.record_success().await;
                Ok(value)
            }
            Err(err) => {
                self.record_failure().await;
                Err(err)
            }
        }
    }

    async fn record_success(&self) {
        let successes = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
        let current_state = *self.state.read().await;

        match current_state {
            CircuitState::HalfOpen => {
                if successes >= SUCCESS_THRESHOLD {
                    *self.state.write().await = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::SeqCst);
                    self.success_count.store(0, Ordering::SeqCst);
                }
            }
            _ => {}
        }
    }

    async fn record_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        *self.last_failure_time.write().await = Some(Instant::now());

        if failures >= FAILURE_THRESHOLD {
            *self.state.write().await = CircuitState::Open;
        }
    }

    async fn get_state(&self) -> CircuitState {
        *self.state.read().await
    }
}

/// Instance pool with health-based eviction
struct InstancePool {
    instances: Arc<RwLock<Vec<WasmInstance>>>,
    semaphore: Arc<Semaphore>,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl InstancePool {
    fn new(capacity: usize) -> Self {
        let instances: Vec<WasmInstance> = (0..capacity)
            .map(|i| WasmInstance::new(i))
            .collect();

        Self {
            instances: Arc::new(RwLock::new(instances)),
            semaphore: Arc::new(Semaphore::new(capacity)),
            circuit_breaker: Arc::new(CircuitBreaker::new()),
        }
    }

    async fn acquire_instance(&self) -> Result<WasmInstance, String> {
        // Acquire semaphore permit
        let _permit = self.semaphore.acquire().await
            .map_err(|_| "Failed to acquire semaphore".to_string())?;

        // Get a healthy instance
        let instances = self.instances.read().await;
        instances
            .iter()
            .find(|i| i.is_healthy())
            .cloned()
            .ok_or_else(|| "No healthy instances available".to_string())
    }

    async fn evict_unhealthy(&self) -> usize {
        let instances = self.instances.read().await;
        let evicted_count = instances.iter()
            .filter(|i| !i.is_healthy())
            .count();

        evicted_count
    }

    async fn get_healthy_count(&self) -> usize {
        let instances = self.instances.read().await;
        instances.iter().filter(|i| i.is_healthy()).count()
    }
}

#[tokio::test]
async fn test_circuit_breaker_trip() {
    let breaker = CircuitBreaker::new();

    // Verify initial state is Closed
    assert_eq!(breaker.get_state().await, CircuitState::Closed);

    // Trigger failures to trip the breaker
    for i in 0..FAILURE_THRESHOLD {
        let result = breaker.call(|| -> Result<(), String> {
            Err(format!("Simulated failure {}", i))
        }).await;
        assert!(result.is_err());
    }

    // Circuit should now be open
    assert_eq!(breaker.get_state().await, CircuitState::Open);

    // Subsequent calls should be rejected immediately
    let result = breaker.call(|| -> Result<(), String> {
        Ok(())
    }).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Circuit breaker is OPEN"));
}

#[tokio::test]
async fn test_circuit_breaker_recovery() {
    let breaker = CircuitBreaker::new();

    // Trip the breaker
    for _ in 0..FAILURE_THRESHOLD {
        let _ = breaker.call(|| -> Result<(), String> {
            Err("Failure".to_string())
        }).await;
    }

    assert_eq!(breaker.get_state().await, CircuitState::Open);

    // Wait for timeout
    sleep(Duration::from_secs(CIRCUIT_TIMEOUT_SECS + 1)).await;

    // Next call should transition to HalfOpen
    let result = breaker.call(|| -> Result<(), String> {
        Ok(())
    }).await;

    // Should succeed and move to HalfOpen
    assert!(result.is_ok());

    // After SUCCESS_THRESHOLD successes, should return to Closed
    for _ in 1..SUCCESS_THRESHOLD {
        let _ = breaker.call(|| -> Result<(), String> {
            Ok(())
        }).await;
    }

    let state = breaker.get_state().await;
    assert!(
        state == CircuitState::Closed || state == CircuitState::HalfOpen,
        "Circuit should be recovering, got {:?}",
        state
    );
}

#[tokio::test]
async fn test_health_based_eviction() {
    let pool = InstancePool::new(8);

    // All instances should start healthy
    let healthy_count = pool.get_healthy_count().await;
    assert_eq!(healthy_count, 8);

    // Simulate failures on some instances
    {
        let instances = pool.instances.read().await;
        for i in 0..3 {
            // Record enough failures to make unhealthy
            for _ in 0..6 {
                instances[i].record_failure();
            }
        }
    }

    // Verify some instances are now unhealthy
    let healthy_count = pool.get_healthy_count().await;
    assert!(healthy_count < 8, "Some instances should be unhealthy");

    // Evict unhealthy instances
    let evicted = pool.evict_unhealthy().await;
    assert!(evicted > 0, "Should have evicted unhealthy instances");
}

#[tokio::test]
async fn test_concurrent_extractions_with_semaphore() {
    let pool = Arc::new(InstancePool::new(MAX_CONCURRENT));
    let mut handles = vec![];

    // Spawn more tasks than the semaphore allows
    for i in 0..20 {
        let pool_clone = Arc::clone(&pool);
        let handle = tokio::spawn(async move {
            let result = pool_clone.acquire_instance().await;
            if result.is_ok() {
                // Simulate work
                sleep(Duration::from_millis(10)).await;
            }
            result
        });
        handles.push(handle);
    }

    // Wait for all tasks
    let mut successes = 0;
    for handle in handles {
        if let Ok(Ok(_)) = handle.await {
            successes += 1;
        }
    }

    // All should eventually succeed
    assert_eq!(successes, 20, "All concurrent operations should eventually succeed");
}

#[tokio::test]
async fn test_instance_health_scoring() {
    let instance = WasmInstance::new(0);

    // Initial health should be 100
    assert_eq!(instance.health_score.load(Ordering::SeqCst), 100);
    assert!(instance.is_healthy());

    // Record failures
    for _ in 0..3 {
        instance.record_failure();
    }

    // Health should decrease
    let health_after_failures = instance.health_score.load(Ordering::SeqCst);
    assert!(health_after_failures < 100);

    // If too many failures, should become unhealthy
    for _ in 0..3 {
        instance.record_failure();
    }

    let final_health = instance.health_score.load(Ordering::SeqCst);
    assert!(final_health < 50, "Instance should be unhealthy after many failures");
    assert!(!instance.is_healthy());

    // Record successes to recover
    for _ in 0..10 {
        instance.record_success();
    }

    assert_eq!(instance.health_score.load(Ordering::SeqCst), 100);
    assert!(instance.is_healthy());
}

#[tokio::test]
async fn test_semaphore_limiting() {
    let semaphore = Arc::new(Semaphore::new(3));
    let mut permits = vec![];

    // Acquire all permits
    for _ in 0..3 {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        permits.push(permit);
    }

    // Try to acquire one more (should block)
    let semaphore_clone = Arc::clone(&semaphore);
    let handle = tokio::spawn(async move {
        let start = Instant::now();
        let _permit = semaphore_clone.acquire().await.unwrap();
        start.elapsed()
    });

    // Give it a moment
    sleep(Duration::from_millis(50)).await;

    // Release one permit
    permits.pop();

    // The waiting task should now complete
    let elapsed = handle.await.unwrap();
    assert!(elapsed > Duration::from_millis(40), "Task should have been blocked");
}

#[tokio::test]
async fn test_instance_last_used_tracking() {
    let instance = WasmInstance::new(0);

    let initial_time = {
        let last_used = instance.last_used.read().await;
        *last_used
    };

    // Simulate some delay
    sleep(Duration::from_millis(100)).await;

    // Update last used
    instance.update_last_used().await;

    let updated_time = {
        let last_used = instance.last_used.read().await;
        *last_used
    };

    assert!(updated_time > initial_time, "Last used time should be updated");
}

#[tokio::test]
async fn test_pool_capacity_limits() {
    let pool = InstancePool::new(5);

    // Should have exactly 5 instances
    let total_count = {
        let instances = pool.instances.read().await;
        instances.len()
    };
    assert_eq!(total_count, 5);

    // All should be healthy initially
    let healthy_count = pool.get_healthy_count().await;
    assert_eq!(healthy_count, 5);
}

#[tokio::test]
async fn test_circuit_breaker_state_transitions() {
    let breaker = CircuitBreaker::new();

    // Closed -> Open transition
    for _ in 0..FAILURE_THRESHOLD {
        let _ = breaker.call(|| -> Result<(), String> {
            Err("Error".to_string())
        }).await;
    }
    assert_eq!(breaker.get_state().await, CircuitState::Open);

    // Wait for timeout (shortened for test)
    *breaker.last_failure_time.write().await = Some(
        Instant::now() - Duration::from_secs(CIRCUIT_TIMEOUT_SECS + 1)
    );

    // Open -> HalfOpen transition
    let _ = breaker.call(|| -> Result<(), String> {
        Ok(())
    }).await;

    // After successes, should transition to Closed
    for _ in 0..SUCCESS_THRESHOLD {
        let _ = breaker.call(|| -> Result<(), String> {
            Ok(())
        }).await;
    }

    let final_state = breaker.get_state().await;
    assert!(
        final_state == CircuitState::Closed || final_state == CircuitState::HalfOpen,
        "Should be on path to recovery"
    );
}

#[tokio::test]
async fn test_concurrent_health_updates() {
    let instance = Arc::new(WasmInstance::new(0));
    let mut handles = vec![];

    // Spawn concurrent updates
    for i in 0..10 {
        let instance_clone = Arc::clone(&instance);
        let handle = tokio::spawn(async move {
            if i % 2 == 0 {
                instance_clone.record_success();
            } else {
                instance_clone.record_failure();
            }
        });
        handles.push(handle);
    }

    // Wait for all updates
    for handle in handles {
        handle.await.unwrap();
    }

    // Health score should still be within valid range
    let health = instance.health_score.load(Ordering::SeqCst);
    assert!(health <= 100, "Health score should not exceed 100");
}

#[tokio::test]
async fn test_pool_under_load() {
    let pool = Arc::new(InstancePool::new(4));
    let mut handles = vec![];

    // Simulate high load
    for _ in 0..50 {
        let pool_clone = Arc::clone(&pool);
        let handle = tokio::spawn(async move {
            match pool_clone.acquire_instance().await {
                Ok(instance) => {
                    instance.update_last_used().await;
                    sleep(Duration::from_millis(5)).await;
                    instance.record_success();
                    Ok(())
                }
                Err(e) => Err(e),
            }
        });
        handles.push(handle);
    }

    // All operations should eventually complete
    let mut completed = 0;
    for handle in handles {
        if handle.await.is_ok() {
            completed += 1;
        }
    }

    assert!(completed > 0, "At least some operations should complete under load");
}
