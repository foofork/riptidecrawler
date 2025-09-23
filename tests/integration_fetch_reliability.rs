use riptide_core::fetch::{
    ReliableHttpClient, RetryConfig, CircuitBreakerConfig, CircuitState, CircuitBreakerError
};
use riptide_core::robots::{RobotsConfig, RobotsManager};
use std::time::Duration;
use tokio::time::sleep;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

/// Integration tests for HTTP fetch reliability patterns
/// These tests cover circuit breaker functionality, retry logic, and robots.txt compliance
/// to improve coverage in critical production paths.

#[tokio::test]
async fn test_circuit_breaker_full_lifecycle() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        recovery_timeout: Duration::from_millis(100),
        success_threshold: 2,
        failure_window: Duration::from_secs(60),
    };

    let retry_config = RetryConfig {
        max_attempts: 1, // No retries for this test
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 1.5,
        jitter: false,
    };

    let client = ReliableHttpClient::new(retry_config, config).unwrap();

    // Start mock server that returns 500 errors
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(500))
        .expect(2)
        .mount(&mock_server)
        .await;

    let url = format!("{}/test", mock_server.uri());

    // First request should fail
    assert!(client.get_with_retry(&url).await.is_err());
    assert_eq!(client.get_circuit_breaker_failure_count(), 1);
    assert_eq!(client.get_circuit_breaker_state().await, CircuitState::Closed);

    // Second request should fail and open circuit
    assert!(client.get_with_retry(&url).await.is_err());
    assert_eq!(client.get_circuit_breaker_failure_count(), 2);
    assert_eq!(client.get_circuit_breaker_state().await, CircuitState::Open);

    // Third request should immediately fail due to open circuit
    let start = std::time::Instant::now();
    assert!(client.get_with_retry(&url).await.is_err());
    let duration = start.elapsed();
    assert!(duration < Duration::from_millis(50)); // Should fail fast

    // Wait for recovery timeout
    sleep(Duration::from_millis(150)).await;

    // Setup successful response for recovery
    Mock::given(method("GET"))
        .and(path("/success"))
        .respond_with(ResponseTemplate::new(200).set_body_string("success"))
        .expect(2)
        .mount(&mock_server)
        .await;

    let success_url = format!("{}/success", mock_server.uri());

    // First success should transition to half-open
    assert!(client.get_with_retry(&success_url).await.is_ok());
    assert_eq!(client.get_circuit_breaker_state().await, CircuitState::HalfOpen);

    // Second success should close the circuit
    assert!(client.get_with_retry(&success_url).await.is_ok());
    assert_eq!(client.get_circuit_breaker_state().await, CircuitState::Closed);
    assert_eq!(client.get_circuit_breaker_failure_count(), 0);
}

#[tokio::test]
async fn test_retry_logic_with_exponential_backoff() {
    let retry_config = RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        backoff_multiplier: 2.0,
        jitter: false,
    };

    let client = ReliableHttpClient::new(retry_config, CircuitBreakerConfig::default()).unwrap();

    let mock_server = MockServer::start().await;

    // First two calls return 503 (retryable), third succeeds
    Mock::given(method("GET"))
        .and(path("/retry-test"))
        .respond_with(ResponseTemplate::new(503))
        .expect(2)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/retry-test"))
        .respond_with(ResponseTemplate::new(200).set_body_string("success"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let url = format!("{}/retry-test", mock_server.uri());
    let start = std::time::Instant::now();

    let response = client.get_with_retry(&url).await.unwrap();
    let duration = start.elapsed();

    assert_eq!(response.status(), 200);
    // Should have taken at least 10ms + 20ms = 30ms for the two retries
    assert!(duration >= Duration::from_millis(25));
}

#[tokio::test]
async fn test_non_retryable_errors_fail_fast() {
    let retry_config = RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 2.0,
        jitter: false,
    };

    let client = ReliableHttpClient::new(retry_config, CircuitBreakerConfig::default()).unwrap();

    let mock_server = MockServer::start().await;

    // Return 404 (non-retryable client error)
    Mock::given(method("GET"))
        .and(path("/not-found"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1) // Should only be called once
        .mount(&mock_server)
        .await;

    let url = format!("{}/not-found", mock_server.uri());
    let start = std::time::Instant::now();

    let result = client.get_with_retry(&url).await;
    let duration = start.elapsed();

    assert!(result.is_err());
    // Should fail fast without retries
    assert!(duration < Duration::from_millis(50));
}

#[tokio::test]
async fn test_timeout_and_429_are_retryable() {
    let retry_config = RetryConfig {
        max_attempts: 2,
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(50),
        backoff_multiplier: 2.0,
        jitter: false,
    };

    let client = ReliableHttpClient::new(retry_config, CircuitBreakerConfig::default()).unwrap();

    let mock_server = MockServer::start().await;

    // First call returns 429 (rate limited), second succeeds
    Mock::given(method("GET"))
        .and(path("/rate-limited"))
        .respond_with(ResponseTemplate::new(429))
        .expect(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/rate-limited"))
        .respond_with(ResponseTemplate::new(200).set_body_string("success"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let url = format!("{}/rate-limited", mock_server.uri());

    let response = client.get_with_retry(&url).await.unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_robots_txt_compliance_blocking() {
    let robots_config = RobotsConfig {
        user_agent: "TestBot".to_string(),
        respect_robots_txt: true,
        crawl_delay: Some(Duration::from_millis(10)),
        request_rate: None,
    };

    let client = ReliableHttpClient::new_with_robots(
        RetryConfig::default(),
        CircuitBreakerConfig::default(),
        robots_config,
    ).unwrap();

    let mock_server = MockServer::start().await;

    // Mock robots.txt that disallows everything for TestBot
    Mock::given(method("GET"))
        .and(path("/robots.txt"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            "User-agent: TestBot\nDisallow: /"
        ))
        .expect(1)
        .mount(&mock_server)
        .await;

    let url = format!("{}/blocked-path", mock_server.uri());

    let result = client.get_with_retry(&url).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("robots.txt"));
}

#[tokio::test]
async fn test_robots_txt_compliance_allowed() {
    let robots_config = RobotsConfig {
        user_agent: "TestBot".to_string(),
        respect_robots_txt: true,
        crawl_delay: Some(Duration::from_millis(10)),
        request_rate: None,
    };

    let client = ReliableHttpClient::new_with_robots(
        RetryConfig::default(),
        CircuitBreakerConfig::default(),
        robots_config,
    ).unwrap();

    let mock_server = MockServer::start().await;

    // Mock robots.txt that allows TestBot
    Mock::given(method("GET"))
        .and(path("/robots.txt"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            "User-agent: TestBot\nAllow: /allowed\nDisallow: /blocked"
        ))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock allowed endpoint
    Mock::given(method("GET"))
        .and(path("/allowed"))
        .respond_with(ResponseTemplate::new(200).set_body_string("allowed content"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let url = format!("{}/allowed", mock_server.uri());

    let response = client.get_with_retry(&url).await.unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_client_configuration_with_robots() {
    let robots_config = RobotsConfig::default();
    let client = ReliableHttpClient::new(
        RetryConfig::default(),
        CircuitBreakerConfig::default(),
    ).unwrap()
    .with_robots_manager(robots_config);

    assert!(client.is_robots_enabled());
    assert!(client.get_robots_manager().is_some());
}

#[tokio::test]
async fn test_circuit_breaker_half_open_failure() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        recovery_timeout: Duration::from_millis(50),
        success_threshold: 2,
        failure_window: Duration::from_secs(60),
    };

    let client = ReliableHttpClient::new(
        RetryConfig { max_attempts: 1, ..RetryConfig::default() },
        config,
    ).unwrap();

    let mock_server = MockServer::start().await;

    // First call fails to open circuit
    Mock::given(method("GET"))
        .and(path("/fail"))
        .respond_with(ResponseTemplate::new(500))
        .expect(2)
        .mount(&mock_server)
        .await;

    let url = format!("{}/fail", mock_server.uri());

    // Open the circuit
    assert!(client.get_with_retry(&url).await.is_err());
    assert_eq!(client.get_circuit_breaker_state().await, CircuitState::Open);

    // Wait for recovery
    sleep(Duration::from_millis(75)).await;

    // Next call should transition to half-open, then fail and reopen
    assert!(client.get_with_retry(&url).await.is_err());
    assert_eq!(client.get_circuit_breaker_state().await, CircuitState::Open);
}

#[tokio::test]
async fn test_jitter_calculation() {
    let retry_config = RetryConfig {
        max_attempts: 2,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 2.0,
        jitter: true,
    };

    let client = ReliableHttpClient::new(retry_config, CircuitBreakerConfig::default()).unwrap();

    // Test delay calculation (private method tested through public behavior)
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/jitter-test"))
        .respond_with(ResponseTemplate::new(503))
        .expect(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/jitter-test"))
        .respond_with(ResponseTemplate::new(200).set_body_string("success"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let url = format!("{}/jitter-test", mock_server.uri());
    let start = std::time::Instant::now();

    let response = client.get_with_retry(&url).await.unwrap();
    let duration = start.elapsed();

    assert_eq!(response.status(), 200);
    // With jitter, delay should be at least base delay but may vary
    assert!(duration >= Duration::from_millis(90));
}