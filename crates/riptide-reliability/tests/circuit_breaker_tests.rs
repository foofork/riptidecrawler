//! Circuit Breaker Tests
//!
//! Tests for both atomic and state-based circuit breaker implementations.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use riptide_monitoring::PerformanceMetrics;
use riptide_reliability::circuit_breaker::{
    record_extraction_result, CircuitBreakerState, ExtractionResult,
};
use riptide_utils::circuit_breaker::{CircuitBreaker, Clock, Config as CircuitConfig, State};

// Mock clock for testing
#[derive(Debug)]
struct MockClock {
    current_ms: Arc<Mutex<u64>>,
}

impl MockClock {
    fn new() -> Self {
        Self {
            current_ms: Arc::new(Mutex::new(0)),
        }
    }

    async fn advance(&self, ms: u64) {
        let mut current = self.current_ms.lock().await;
        *current += ms;
    }

    #[allow(dead_code)]
    async fn set(&self, ms: u64) {
        let mut current = self.current_ms.lock().await;
        *current = ms;
    }
}

impl Clock for MockClock {
    fn now_ms(&self) -> u64 {
        // Use try_lock since Clock trait is sync
        // This works for tests since we control the timing
        self.current_ms.try_lock().map(|guard| *guard).unwrap_or(0)
    }
}

// Atomic Circuit Breaker Tests

#[tokio::test]
async fn test_atomic_circuit_breaker_starts_closed() {
    let config = CircuitConfig::default();
    let clock = Arc::new(MockClock::new());
    let cb = CircuitBreaker::new(config, clock);

    assert_eq!(cb.state(), State::Closed, "Circuit should start closed");
}

#[tokio::test]
async fn test_atomic_circuit_breaker_trips_after_threshold() {
    let config = CircuitConfig {
        failure_threshold: 3,
        open_cooldown_ms: 30_000,
        half_open_max_in_flight: 2,
    };
    let clock = Arc::new(MockClock::new());
    let cb = CircuitBreaker::new(config, clock);

    // Report failures up to threshold
    for i in 0..3 {
        assert_eq!(
            cb.state(),
            State::Closed,
            "Should be closed at failure {}",
            i
        );
        cb.on_failure();
    }

    assert_eq!(
        cb.state(),
        State::Open,
        "Circuit should be open after threshold"
    );
}

#[tokio::test]
async fn test_atomic_circuit_breaker_allows_acquire_when_closed() {
    let config = CircuitConfig::default();
    let clock = Arc::new(MockClock::new());
    let cb = CircuitBreaker::new(config, clock);

    let result = cb.try_acquire();
    assert!(result.is_ok(), "Should allow acquire when closed");
}

#[tokio::test]
async fn test_atomic_circuit_breaker_rejects_when_open() {
    let config = CircuitConfig {
        failure_threshold: 2,
        open_cooldown_ms: 30_000,
        half_open_max_in_flight: 2,
    };
    let clock = Arc::new(MockClock::new());
    let cb = CircuitBreaker::new(config, clock);

    // Trip the circuit
    cb.on_failure();
    cb.on_failure();
    assert_eq!(cb.state(), State::Open);

    // Try to acquire
    let result = cb.try_acquire();
    assert!(result.is_err(), "Should reject when open");
    assert_eq!(result.unwrap_err(), "circuit open");
}

#[tokio::test]
async fn test_atomic_circuit_breaker_transitions_to_half_open() {
    let config = CircuitConfig {
        failure_threshold: 2,
        open_cooldown_ms: 1000, // 1 second
        half_open_max_in_flight: 2,
    };
    let clock = Arc::new(MockClock::new());
    let cb = CircuitBreaker::new(config, clock.clone());

    // Trip the circuit
    cb.on_failure();
    cb.on_failure();
    assert_eq!(cb.state(), State::Open);

    // Advance time past cooldown
    clock.advance(1001).await;

    // Next acquire should transition to half-open
    let result = cb.try_acquire();
    assert!(
        result.is_ok(),
        "Should allow trial when transitioning to half-open"
    );
    assert_eq!(cb.state(), State::HalfOpen);
}

#[tokio::test]
async fn test_atomic_circuit_breaker_half_open_limits_concurrency() {
    let config = CircuitConfig {
        failure_threshold: 2,
        open_cooldown_ms: 1000,
        half_open_max_in_flight: 2,
    };
    let clock = Arc::new(MockClock::new());
    let cb = CircuitBreaker::new(config, clock.clone());

    // Trip and transition to half-open
    cb.on_failure();
    cb.on_failure();
    clock.advance(1001).await;
    let _ = cb.try_acquire();
    assert_eq!(cb.state(), State::HalfOpen);

    // Acquire permits
    let permit1 = cb.try_acquire().unwrap();
    let permit2 = cb.try_acquire().unwrap();

    // Third should fail (max 2 in flight)
    let result = cb.try_acquire();
    assert!(result.is_err(), "Should reject when half-open saturated");
    assert_eq!(result.unwrap_err(), "half-open saturated");

    // Drop permits
    drop(permit1);
    drop(permit2);
}

#[tokio::test]
async fn test_atomic_circuit_breaker_closes_on_success() {
    let config = CircuitConfig {
        failure_threshold: 2,
        open_cooldown_ms: 1000,
        half_open_max_in_flight: 2,
    };
    let clock = Arc::new(MockClock::new());
    let cb = CircuitBreaker::new(config, clock.clone());

    // Trip and transition to half-open
    cb.on_failure();
    cb.on_failure();
    clock.advance(1001).await;
    let _ = cb.try_acquire();

    // Success in half-open should close
    cb.on_success();
    assert_eq!(
        cb.state(),
        State::Closed,
        "Should close on success in half-open"
    );
}

#[tokio::test]
async fn test_atomic_circuit_breaker_reopens_on_half_open_failure() {
    let config = CircuitConfig {
        failure_threshold: 2,
        open_cooldown_ms: 1000,
        half_open_max_in_flight: 2,
    };
    let clock = Arc::new(MockClock::new());
    let cb = CircuitBreaker::new(config, clock.clone());

    // Trip and transition to half-open
    cb.on_failure();
    cb.on_failure();
    clock.advance(1001).await;
    let _ = cb.try_acquire();
    assert_eq!(cb.state(), State::HalfOpen);

    // Failure in half-open should reopen
    cb.on_failure();
    assert_eq!(
        cb.state(),
        State::Open,
        "Should reopen on failure in half-open"
    );
}

// State-Based Circuit Breaker Tests

#[test]
fn test_circuit_breaker_state_checks() {
    let closed = CircuitBreakerState::Closed {
        failure_count: 0,
        success_count: 0,
        last_failure: None,
    };
    assert!(closed.is_closed());
    assert!(!closed.is_open());
    assert!(!closed.is_half_open());

    let open = CircuitBreakerState::Open {
        opened_at: Instant::now(),
        failure_count: 5,
    };
    assert!(!open.is_closed());
    assert!(open.is_open());
    assert!(!open.is_half_open());

    let half_open = CircuitBreakerState::HalfOpen {
        test_requests: 0,
        start_time: Instant::now(),
    };
    assert!(!half_open.is_closed());
    assert!(!half_open.is_open());
    assert!(half_open.is_half_open());
}

#[tokio::test]
async fn test_record_extraction_success() {
    let metrics = Arc::new(Mutex::new(PerformanceMetrics::default()));
    let state = Arc::new(Mutex::new(CircuitBreakerState::default()));

    record_extraction_result(
        &metrics,
        &state,
        &None,
        ExtractionResult {
            pool_id: "test-pool".to_string(),
            failure_threshold: 50,
            timeout_duration: 30_000,
            success: true,
            duration: Duration::from_millis(100),
        },
    )
    .await;

    let m = metrics.lock().await;
    assert_eq!(m.total_extractions, 1);
    assert_eq!(m.successful_extractions, 1);
    assert_eq!(m.failed_extractions, 0);
    assert!(m.avg_extraction_time_ms > 0.0);

    let s = state.lock().await;
    assert!(s.is_closed());
}

#[tokio::test]
async fn test_record_extraction_failure() {
    let metrics = Arc::new(Mutex::new(PerformanceMetrics::default()));
    let state = Arc::new(Mutex::new(CircuitBreakerState::default()));

    record_extraction_result(
        &metrics,
        &state,
        &None,
        ExtractionResult {
            pool_id: "test-pool".to_string(),
            failure_threshold: 50,
            timeout_duration: 30_000,
            success: false,
            duration: Duration::from_millis(200),
        },
    )
    .await;

    let m = metrics.lock().await;
    assert_eq!(m.total_extractions, 1);
    assert_eq!(m.successful_extractions, 0);
    assert_eq!(m.failed_extractions, 1);
}

#[tokio::test]
async fn test_circuit_breaker_trips_on_threshold() {
    let metrics = Arc::new(Mutex::new(PerformanceMetrics::default()));
    let state = Arc::new(Mutex::new(CircuitBreakerState::default()));

    // Record failures up to threshold (50%)
    for i in 0..10 {
        let success = i < 4; // 4 success, 6 failures = 60% failure rate
        record_extraction_result(
            &metrics,
            &state,
            &None,
            ExtractionResult {
                pool_id: "test-pool".to_string(),
                failure_threshold: 50,
                timeout_duration: 30_000,
                success,
                duration: Duration::from_millis(100),
            },
        )
        .await;
    }

    let s = state.lock().await;
    // Circuit should be open due to high failure rate
    assert!(s.is_open(), "Circuit should trip at 50% failure threshold");
}

#[tokio::test]
async fn test_average_extraction_time_calculation() {
    let metrics = Arc::new(Mutex::new(PerformanceMetrics::default()));
    let state = Arc::new(Mutex::new(CircuitBreakerState::default()));

    // Record multiple extractions with different durations
    let durations = vec![100, 200, 150, 250];
    for duration_ms in durations {
        record_extraction_result(
            &metrics,
            &state,
            &None,
            ExtractionResult {
                pool_id: "test-pool".to_string(),
                failure_threshold: 50,
                timeout_duration: 30_000,
                success: true,
                duration: Duration::from_millis(duration_ms),
            },
        )
        .await;
    }

    let m = metrics.lock().await;
    assert!(m.avg_extraction_time_ms > 0.0);
    assert!(m.avg_extraction_time_ms < 300.0);
}
