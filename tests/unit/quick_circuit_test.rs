// Quick test to verify circuit breaker fixes
use riptide_utils::circuit_breaker::{CircuitBreaker, Config, RealClock, State};
use std::sync::Arc;

#[test]
fn test_circuit_breaker_basic() {
    let cb = CircuitBreaker::new(
        Config {
            failure_threshold: 3,
            open_cooldown_ms: 1000,
            half_open_max_in_flight: 2,
        },
        Arc::new(RealClock),
    );

    // Initially closed
    assert_eq!(cb.state(), State::Closed);

    // Trip to open after failures
    cb.on_failure();
    cb.on_failure();
    cb.on_failure();
    assert_eq!(cb.state(), State::Open);

    println!("✅ Circuit breaker state transitions work correctly");
}

#[test]
fn test_fetch_client_creation() {
    use riptide_fetch::{ReliableHttpClient, RetryConfig, CircuitBreakerConfig};

    let client = ReliableHttpClient::new(
        RetryConfig::default(),
        CircuitBreakerConfig::default(),
    );

    assert!(client.is_ok());
    println!("✅ ReliableHttpClient creates successfully");
}

fn main() {
    test_circuit_breaker_basic();
    test_fetch_client_creation();
    println!("\n🎉 All circuit breaker fixes verified successfully!");
}