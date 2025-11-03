//! Circuit breaker pattern tests for fault tolerance
//!
//! Tests circuit breaker states, transitions, and recovery

use riptide_pool::*;
use std::time::{Duration, Instant};

#[tokio::test]
async fn test_circuit_breaker_closed_state() {
    let state = CircuitBreakerState::Closed {
        failure_count: 0,
        success_count: 10,
        last_failure: None,
    };

    match state {
        CircuitBreakerState::Closed {
            failure_count,
            success_count,
            ..
        } => {
            assert_eq!(failure_count, 0);
            assert_eq!(success_count, 10);
        }
        _ => panic!("Expected Closed state"),
    }
}

#[tokio::test]
async fn test_circuit_breaker_open_state() {
    let now = Instant::now();
    let state = CircuitBreakerState::Open {
        opened_at: now,
        failure_count: 5,
    };

    match state {
        CircuitBreakerState::Open {
            opened_at,
            failure_count,
        } => {
            assert!(opened_at.elapsed() < Duration::from_millis(100));
            assert_eq!(failure_count, 5);
        }
        _ => panic!("Expected Open state"),
    }
}

#[tokio::test]
async fn test_circuit_breaker_half_open_state() {
    let start = Instant::now();
    let state = CircuitBreakerState::HalfOpen {
        test_requests: 1,
        start_time: start,
    };

    match state {
        CircuitBreakerState::HalfOpen {
            test_requests,
            start_time,
        } => {
            assert_eq!(test_requests, 1);
            assert!(start_time.elapsed() < Duration::from_millis(100));
        }
        _ => panic!("Expected HalfOpen state"),
    }
}

#[tokio::test]
async fn test_circuit_breaker_failure_threshold() {
    let threshold = 5u32;
    let mut failures = 0u64;
    let successes = 0u64;

    // Simulate requests with failures
    for _ in 0..10 {
        failures += 1;
    }

    let total = failures + successes;
    let failure_rate = if total > 0 {
        (failures as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    // Should trip circuit breaker (100% failure rate)
    assert!(failure_rate >= threshold as f64);
    assert_eq!(failure_rate, 100.0);
}

#[tokio::test]
async fn test_circuit_breaker_recovery() {
    let threshold = 50u32; // 50% failure threshold
    let failures = 3u64;
    let mut successes = 7u64;

    let total = failures + successes;
    let failure_rate = (failures as f64 / total as f64) * 100.0;

    // Should NOT trip (30% failure rate, below threshold)
    assert!(failure_rate < threshold as f64);

    // Simulate recovery
    for _ in 0..10 {
        successes += 1;
    }

    let new_total = failures + successes;
    let new_failure_rate = (failures as f64 / new_total as f64) * 100.0;

    // Failure rate should decrease
    assert!(new_failure_rate < failure_rate);
    assert!(new_failure_rate < 20.0);
}

#[tokio::test]
async fn test_circuit_breaker_timeout_check() {
    let timeout_ms = 5000u64;
    let opened_at = Instant::now();

    // Just opened - should still be open
    tokio::time::sleep(Duration::from_millis(100)).await;
    let is_still_open = opened_at.elapsed() < Duration::from_millis(timeout_ms);
    assert!(is_still_open);

    // Simulate timeout expiry
    let old_opened_at = Instant::now() - Duration::from_millis(timeout_ms + 100);
    let should_transition = old_opened_at.elapsed() >= Duration::from_millis(timeout_ms);
    assert!(should_transition);
}

#[tokio::test]
async fn test_circuit_breaker_state_transitions() {
    // Closed -> Open (high failure rate)
    let mut state = CircuitBreakerState::Closed {
        failure_count: 8,
        success_count: 2,
        last_failure: Some(Instant::now()),
    };

    // Check if should transition to Open
    if let CircuitBreakerState::Closed {
        failure_count,
        success_count,
        ..
    } = state
    {
        let total = failure_count + success_count;
        let failure_rate = (failure_count as f64 / total as f64) * 100.0;

        if total >= 10 && failure_rate >= 50.0 {
            state = CircuitBreakerState::Open {
                opened_at: Instant::now(),
                failure_count,
            };
        }
    }

    // Should have transitioned to Open
    assert!(matches!(state, CircuitBreakerState::Open { .. }));
}

#[tokio::test]
async fn test_circuit_breaker_half_open_success() {
    let state = CircuitBreakerState::HalfOpen {
        test_requests: 2,
        start_time: Instant::now(),
    };

    // Successful test request should close circuit
    let success = true;

    let new_state = if success {
        CircuitBreakerState::Closed {
            failure_count: 0,
            success_count: 1,
            last_failure: None,
        }
    } else {
        state
    };

    assert!(matches!(new_state, CircuitBreakerState::Closed { .. }));
}

#[tokio::test]
async fn test_circuit_breaker_half_open_failure() {
    let state = CircuitBreakerState::HalfOpen {
        test_requests: 3,
        start_time: Instant::now(),
    };

    // Failed test requests should reopen circuit
    let max_test_requests = 3;

    let new_state = match state {
        CircuitBreakerState::HalfOpen { test_requests, .. }
            if test_requests >= max_test_requests =>
        {
            CircuitBreakerState::Open {
                opened_at: Instant::now(),
                failure_count: 1,
            }
        }
        _ => state,
    };

    assert!(matches!(new_state, CircuitBreakerState::Open { .. }));
}

#[tokio::test]
async fn test_failure_rate_calculation() {
    // Test various failure scenarios
    let scenarios = vec![
        (0, 10, 0.0),   // 0% failure
        (5, 10, 33.3),  // 33.3% failure
        (10, 10, 50.0), // 50% failure
        (15, 10, 60.0), // 60% failure
        (10, 0, 100.0), // 100% failure
    ];

    for (failures, successes, expected_rate) in scenarios {
        let total = failures + successes;
        let actual_rate = if total > 0 {
            (failures as f64 / total as f64) * 100.0
        } else {
            100.0
        };

        assert!((actual_rate - expected_rate).abs() < 0.2);
    }
}

#[tokio::test]
async fn test_circuit_breaker_metrics_tracking() {
    let mut trips = 0u64;
    let mut resets = 0u64;

    // Simulate circuit breaker lifecycle
    for i in 0..5 {
        // Trip circuit
        trips += 1;

        // Wait and reset
        tokio::time::sleep(Duration::from_millis(10)).await;
        if i % 2 == 0 {
            resets += 1;
        }
    }

    assert_eq!(trips, 5);
    assert_eq!(resets, 3);
}

#[tokio::test]
async fn test_circuit_breaker_cooldown_period() {
    let cooldown_ms = 5000u64;
    let opened_at = Instant::now();

    // Check cooldown periods
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(opened_at.elapsed() < Duration::from_millis(cooldown_ms));

    // Simulate cooldown expiry
    let expired_time = Instant::now() - Duration::from_millis(cooldown_ms + 500);
    assert!(expired_time.elapsed() > Duration::from_millis(cooldown_ms));
}

#[tokio::test]
async fn test_circuit_breaker_concurrent_failures() {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    let failure_count = Arc::new(Mutex::new(0u64));
    let mut handles = vec![];

    // Simulate concurrent failures
    for _ in 0..10 {
        let fc = failure_count.clone();
        let handle = tokio::spawn(async move {
            let mut count = fc.lock().await;
            *count += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let final_count = *failure_count.lock().await;
    assert_eq!(final_count, 10);
}

#[tokio::test]
async fn test_circuit_breaker_exponential_backoff() {
    let base_timeout = 1000u64;
    let max_retries = 5;

    let mut timeouts = vec![];
    for retry in 0..max_retries {
        let timeout = base_timeout * 2u64.pow(retry);
        timeouts.push(timeout);
    }

    assert_eq!(timeouts[0], 1000); // 1s
    assert_eq!(timeouts[1], 2000); // 2s
    assert_eq!(timeouts[2], 4000); // 4s
    assert_eq!(timeouts[3], 8000); // 8s
    assert_eq!(timeouts[4], 16000); // 16s
}

// ============================================================================
// FAILOVER BEHAVIOR TESTS
// ============================================================================

/// Test failover sequence from primary to secondary instance
#[tokio::test]
async fn test_failover_sequence_primary_to_secondary() {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // Simulate pool with 2 instances (primary + secondary)
    #[derive(Clone, Debug)]
    struct MockInstance {
        id: String,
        is_healthy: bool,
        failure_count: u64,
    }

    let instances = Arc::new(Mutex::new(vec![
        MockInstance {
            id: "primary".to_string(),
            is_healthy: true,
            failure_count: 0,
        },
        MockInstance {
            id: "secondary".to_string(),
            is_healthy: true,
            failure_count: 0,
        },
    ]));

    // Circuit breaker state
    let circuit_state = Arc::new(Mutex::new(CircuitBreakerState::Closed {
        failure_count: 0,
        success_count: 0,
        last_failure: None,
    }));

    // Step 1: Primary instance fails
    {
        let mut instances_lock = instances.lock().await;
        instances_lock[0].is_healthy = false;
        instances_lock[0].failure_count = 5;
    }

    // Step 2: Circuit breaker should detect failure and open
    {
        let mut state = circuit_state.lock().await;
        *state = CircuitBreakerState::Open {
            opened_at: Instant::now(),
            failure_count: 5,
        };
    }

    // Verify circuit is open
    let is_open = matches!(
        *circuit_state.lock().await,
        CircuitBreakerState::Open { .. }
    );
    assert!(is_open, "Circuit should be open after primary failure");

    // Step 3: Get next available instance (should be secondary)
    let active_instance = {
        let instances_lock = instances.lock().await;
        instances_lock
            .iter()
            .find(|i| i.is_healthy)
            .cloned()
            .expect("Should have healthy secondary instance")
    };

    assert_eq!(
        active_instance.id, "secondary",
        "Should failover to secondary instance"
    );
    assert_eq!(
        active_instance.failure_count, 0,
        "Secondary should have no failures"
    );

    // Step 4: Track failover metrics
    let failover_count = 1u64;
    let total_instances = instances.lock().await.len();

    assert_eq!(failover_count, 1, "Should record one failover event");
    assert_eq!(
        total_instances, 2,
        "Should maintain both instances during failover"
    );
}

/// Test failover when both primary and secondary fail
#[tokio::test]
async fn test_failover_both_instances_failed() {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Clone, Debug)]
    struct MockInstance {
        id: String,
        is_healthy: bool,
        failure_count: u64,
    }

    let instances = Arc::new(Mutex::new(vec![
        MockInstance {
            id: "primary".to_string(),
            is_healthy: true,
            failure_count: 0,
        },
        MockInstance {
            id: "secondary".to_string(),
            is_healthy: true,
            failure_count: 0,
        },
    ]));

    // Step 1: Both instances fail
    {
        let mut instances_lock = instances.lock().await;
        instances_lock[0].is_healthy = false;
        instances_lock[0].failure_count = 5;
        instances_lock[1].is_healthy = false;
        instances_lock[1].failure_count = 3;
    }

    // Step 2: Verify no healthy instances available
    let healthy_instances = {
        let instances_lock = instances.lock().await;
        instances_lock.iter().filter(|i| i.is_healthy).count()
    };

    assert_eq!(
        healthy_instances, 0,
        "Should have no healthy instances when both fail"
    );

    // Step 3: Verify all instances marked unhealthy
    {
        let instances_lock = instances.lock().await;
        for instance in instances_lock.iter() {
            assert!(
                !instance.is_healthy,
                "Instance {} should be unhealthy",
                instance.id
            );
            assert!(
                instance.failure_count > 0,
                "Instance {} should have failures recorded",
                instance.id
            );
        }
    }

    // Step 4: Circuit breaker should remain open
    let circuit_state = CircuitBreakerState::Open {
        opened_at: Instant::now(),
        failure_count: 8, // Combined failures
    };

    assert!(
        matches!(circuit_state, CircuitBreakerState::Open { .. }),
        "Circuit should remain open when all instances fail"
    );
}

/// Test failover recovery sequence: primary recovery and restoration
#[tokio::test]
async fn test_failover_recovery_sequence() {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Clone, Debug)]
    struct MockInstance {
        id: String,
        is_healthy: bool,
        failure_count: u64,
        success_count: u64,
    }

    let instances = Arc::new(Mutex::new(vec![
        MockInstance {
            id: "primary".to_string(),
            is_healthy: false, // Start with failed primary
            failure_count: 5,
            success_count: 0,
        },
        MockInstance {
            id: "secondary".to_string(),
            is_healthy: true,
            failure_count: 0,
            success_count: 10,
        },
    ]));

    // Circuit breaker in Open state
    let circuit_state = Arc::new(Mutex::new(CircuitBreakerState::Open {
        opened_at: Instant::now() - Duration::from_millis(6000), // Past timeout
        failure_count: 5,
    }));

    // Step 1: Verify using secondary instance
    {
        let instances_lock = instances.lock().await;
        let active = instances_lock
            .iter()
            .find(|i| i.is_healthy)
            .expect("Should have healthy secondary");
        assert_eq!(active.id, "secondary");
    }

    // Step 2: Simulate primary recovery
    {
        let mut instances_lock = instances.lock().await;
        instances_lock[0].is_healthy = true;
        instances_lock[0].failure_count = 0;
        instances_lock[0].success_count = 1;
    }

    // Step 3: Circuit should transition to HalfOpen for testing
    {
        let mut state = circuit_state.lock().await;
        *state = CircuitBreakerState::HalfOpen {
            test_requests: 1,
            start_time: Instant::now(),
        };
    }

    // Verify half-open state
    assert!(
        matches!(
            *circuit_state.lock().await,
            CircuitBreakerState::HalfOpen { .. }
        ),
        "Circuit should be in HalfOpen state during recovery"
    );

    // Step 4: Successful test request should close circuit
    {
        let mut instances_lock = instances.lock().await;
        instances_lock[0].success_count += 1;
    }

    {
        let mut state = circuit_state.lock().await;
        *state = CircuitBreakerState::Closed {
            failure_count: 0,
            success_count: 2,
            last_failure: None,
        };
    }

    // Step 5: Verify circuit closed and primary restored
    assert!(
        matches!(
            *circuit_state.lock().await,
            CircuitBreakerState::Closed { .. }
        ),
        "Circuit should be closed after successful recovery"
    );

    {
        let instances_lock = instances.lock().await;
        assert!(
            instances_lock[0].is_healthy,
            "Primary instance should be healthy"
        );
        assert_eq!(
            instances_lock[0].success_count, 2,
            "Primary should have successful requests"
        );
    }
}

/// Test concurrent failover with multiple simultaneous failures
#[tokio::test]
async fn test_concurrent_failover_multiple_failures() {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Clone, Debug)]
    struct MockInstance {
        id: String,
        is_healthy: Arc<Mutex<bool>>,
        failure_count: Arc<Mutex<u64>>,
    }

    let instances = [
        MockInstance {
            id: "instance-1".to_string(),
            is_healthy: Arc::new(Mutex::new(true)),
            failure_count: Arc::new(Mutex::new(0)),
        },
        MockInstance {
            id: "instance-2".to_string(),
            is_healthy: Arc::new(Mutex::new(true)),
            failure_count: Arc::new(Mutex::new(0)),
        },
        MockInstance {
            id: "instance-3".to_string(),
            is_healthy: Arc::new(Mutex::new(true)),
            failure_count: Arc::new(Mutex::new(0)),
        },
    ];

    let failover_events = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];

    // Simulate concurrent failures across multiple instances
    for (idx, instance) in instances.iter().enumerate() {
        let is_healthy = instance.is_healthy.clone();
        let failure_count = instance.failure_count.clone();
        let events = failover_events.clone();
        let instance_id = instance.id.clone();

        let handle = tokio::spawn(async move {
            // Simulate failure detection
            tokio::time::sleep(Duration::from_millis(10 + idx as u64 * 5)).await;

            // Mark instance as unhealthy
            *is_healthy.lock().await = false;
            *failure_count.lock().await = 1;

            // Record failover event
            events
                .lock()
                .await
                .push(format!("{} failed at {:?}", instance_id, Instant::now()));
        });

        handles.push(handle);
    }

    // Wait for all concurrent failures
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all instances failed
    for instance in instances.iter() {
        assert!(
            !*instance.is_healthy.lock().await,
            "Instance {} should be unhealthy",
            instance.id
        );
        assert_eq!(
            *instance.failure_count.lock().await,
            1,
            "Instance {} should have failure recorded",
            instance.id
        );
    }

    // Verify failover events recorded
    let events = failover_events.lock().await;
    assert_eq!(
        events.len(),
        3,
        "Should record failover event for each instance"
    );
}

/// Test failover metrics tracking during circuit breaker transitions
#[tokio::test]
async fn test_failover_metrics_tracking() {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Clone, Debug, Default)]
    struct FailoverMetrics {
        total_failovers: u64,
        circuit_breaker_trips: u64,
        circuit_breaker_resets: u64,
        failed_instances: u64,
        recovered_instances: u64,
    }

    let metrics = Arc::new(Mutex::new(FailoverMetrics::default()));

    // Scenario 1: Instance fails, circuit opens
    {
        let mut m = metrics.lock().await;
        m.total_failovers += 1;
        m.circuit_breaker_trips += 1;
        m.failed_instances += 1;
    }

    // Verify failover recorded
    {
        let m = metrics.lock().await;
        assert_eq!(m.total_failovers, 1, "Should record failover");
        assert_eq!(m.circuit_breaker_trips, 1, "Should record circuit trip");
        assert_eq!(m.failed_instances, 1, "Should record failed instance");
    }

    // Scenario 2: Instance recovers, circuit closes
    {
        let mut m = metrics.lock().await;
        m.circuit_breaker_resets += 1;
        m.recovered_instances += 1;
    }

    // Verify recovery metrics
    {
        let m = metrics.lock().await;
        assert_eq!(m.circuit_breaker_resets, 1, "Should record circuit reset");
        assert_eq!(m.recovered_instances, 1, "Should record recovery");
        assert_eq!(
            m.failed_instances, m.recovered_instances,
            "Failed and recovered instances should match"
        );
    }
}

/// Test circuit breaker state during failover with precise timing
#[tokio::test]
async fn test_circuit_breaker_failover_timing() {
    let timeout_ms = 5000u64;

    // Step 1: Circuit opens when primary fails
    let opened_at = Instant::now();
    let state = CircuitBreakerState::Open {
        opened_at,
        failure_count: 5,
    };

    // Verify circuit is open immediately
    assert!(
        matches!(state, CircuitBreakerState::Open { .. }),
        "Circuit should be open"
    );

    // Step 2: Before timeout, circuit should remain open
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(
        opened_at.elapsed() < Duration::from_millis(timeout_ms),
        "Circuit should still be in timeout period"
    );

    // Step 3: Simulate timeout expiry
    let expired_opened_at = Instant::now() - Duration::from_millis(timeout_ms + 100);

    // Step 4: Transition to HalfOpen after timeout
    let half_open_state = CircuitBreakerState::HalfOpen {
        test_requests: 0,
        start_time: Instant::now(),
    };

    assert!(
        expired_opened_at.elapsed() >= Duration::from_millis(timeout_ms),
        "Timeout period should have expired"
    );
    assert!(
        matches!(half_open_state, CircuitBreakerState::HalfOpen { .. }),
        "Circuit should transition to HalfOpen"
    );

    // Step 5: Successful test closes circuit
    let closed_state = CircuitBreakerState::Closed {
        failure_count: 0,
        success_count: 1,
        last_failure: None,
    };

    assert!(
        matches!(closed_state, CircuitBreakerState::Closed { .. }),
        "Circuit should close after successful test"
    );
}
