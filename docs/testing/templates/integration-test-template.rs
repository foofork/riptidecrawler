/// Integration Test Template for Cross-Crate Testing
///
/// This template demonstrates testing interactions between multiple crates.
/// Place integration tests in `tests/` directory at crate root.

use riptide_monitoring::monitoring::collector::MetricsCollector;
use riptide_security::middleware::SecurityMiddleware;
use riptide_fetch::fetch::FetchClient;

/// Integration test: Multiple crates working together
#[tokio::test]
async fn test_cross_crate_integration() {
    // Arrange: Setup all required crates
    let metrics = MetricsCollector::new();
    let security = SecurityMiddleware::new(SecurityConfig::default());
    let fetch_client = FetchClient::builder()
        .with_metrics(metrics.clone())
        .with_security(security.clone())
        .build()
        .unwrap();

    // Act: Exercise integration
    let url = "https://example.com";
    let result = fetch_client.get(url).await;

    // Assert: Verify cross-crate behavior
    assert!(result.is_ok());

    // Verify metrics were recorded
    let request_count = metrics.get_counter("http_requests_total").unwrap();
    assert_eq!(request_count, 1);

    // Verify security audit logged
    let audit_entries = security.get_audit_log().await.unwrap();
    assert!(!audit_entries.is_empty());
    assert!(audit_entries[0].contains(url));
}

/// Integration test: API + Monitoring
#[tokio::test]
async fn test_api_monitoring_integration() {
    use riptide_api::AppState;

    // Arrange
    let metrics = MetricsCollector::new();
    let state = AppState::builder()
        .with_metrics(metrics.clone())
        .build()
        .unwrap();

    // Act: Process requests
    for i in 0..10 {
        let request = create_test_request(i);
        state.process_request(request).await.unwrap();
    }

    // Assert: Metrics collected correctly
    assert_eq!(metrics.get_counter("requests_processed").unwrap(), 10);
    assert!(metrics.get_gauge("active_requests").unwrap() == 0.0);

    let latency = metrics.get_histogram("request_latency_ms").unwrap();
    assert!(latency.mean() > 0.0);
    assert!(latency.p99() < 1000.0); // 99th percentile under 1s
}

/// Integration test: Security + API
#[tokio::test]
async fn test_security_middleware_integration() {
    use riptide_api::{AppState, Request};
    use riptide_security::middleware::RateLimiter;

    // Arrange
    let rate_limiter = RateLimiter::builder()
        .max_requests(5)
        .per_seconds(60)
        .build();

    let security = SecurityMiddleware::builder()
        .with_rate_limiter(rate_limiter)
        .build();

    let state = AppState::builder()
        .with_security(security.clone())
        .build()
        .unwrap();

    // Act: Send requests exceeding rate limit
    let mut results = vec![];
    for i in 0..10 {
        let request = Request::new(format!("request-{}", i));
        results.push(state.process_request(request).await);
    }

    // Assert: First 5 succeed, rest fail
    let successes = results.iter().filter(|r| r.is_ok()).count();
    let failures = results.iter().filter(|r| r.is_err()).count();

    assert_eq!(successes, 5, "Expected 5 successful requests");
    assert_eq!(failures, 5, "Expected 5 rate-limited requests");

    // Verify audit log
    let audit_log = security.get_audit_log().await.unwrap();
    assert!(audit_log.iter().any(|entry| entry.contains("rate_limit_exceeded")));
}

/// Integration test: Fetch + Circuit Breaker
#[tokio::test]
async fn test_fetch_circuit_breaker_integration() {
    use riptide_fetch::circuit::CircuitBreaker;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::method;

    // Arrange: Setup mock server that fails
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(500))
        .expect(5)
        .mount(&mock_server)
        .await;

    let circuit_breaker = CircuitBreaker::builder()
        .failure_threshold(3)
        .timeout_seconds(5)
        .build();

    let fetch_client = FetchClient::builder()
        .with_circuit_breaker(circuit_breaker.clone())
        .build()
        .unwrap();

    // Act: Make requests until circuit opens
    let mut results = vec![];
    for _ in 0..5 {
        let result = fetch_client.get(&mock_server.uri()).await;
        results.push(result);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // Assert: Circuit opens after threshold
    assert_eq!(circuit_breaker.state().await, CircuitState::Open);

    // First 3 requests hit server and fail
    assert!(results[0].is_err());
    assert!(results[1].is_err());
    assert!(results[2].is_err());

    // Subsequent requests fail fast (circuit open)
    assert!(results[3].is_err());
    assert!(results[4].is_err());

    // Verify circuit opens faster than hitting server
    let last_error = results[4].as_ref().unwrap_err();
    assert_eq!(last_error.kind(), ErrorKind::CircuitOpen);
}

/// Integration test: Full stack end-to-end
#[tokio::test]
async fn test_full_stack_e2e() {
    use riptide_api::{AppState, Server};
    use reqwest::Client;

    // Arrange: Start full application
    let metrics = MetricsCollector::new();
    let security = SecurityMiddleware::new(SecurityConfig::default());
    let fetch_client = FetchClient::builder()
        .with_metrics(metrics.clone())
        .with_security(security.clone())
        .build()
        .unwrap();

    let state = AppState::builder()
        .with_metrics(metrics.clone())
        .with_security(security.clone())
        .with_fetch_client(fetch_client)
        .build()
        .unwrap();

    let server = Server::new(state);
    let addr = server.bind("127.0.0.1:0").await.unwrap();
    let server_handle = tokio::spawn(server.run());

    // Act: Make HTTP request to running server
    let client = Client::new();
    let response = client
        .post(format!("http://{}/api/crawl", addr))
        .json(&serde_json::json!({
            "url": "https://example.com",
            "depth": 2
        }))
        .send()
        .await
        .unwrap();

    // Assert: Response and side effects
    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["results"].is_array());

    // Verify metrics were recorded
    let crawl_count = metrics.get_counter("crawls_total").unwrap();
    assert_eq!(crawl_count, 1);

    // Verify security audit
    let audit = security.get_audit_log().await.unwrap();
    assert!(audit.iter().any(|e| e.contains("crawl_request")));

    // Cleanup
    server_handle.abort();
}

/// Integration test: Database persistence
#[tokio::test]
async fn test_persistence_integration() {
    use riptide_persistence::{Repository, StateStore};
    use tempfile::TempDir;

    // Arrange: Setup temporary database
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let repository = Repository::new(&db_path).await.unwrap();
    let state_store = StateStore::new(repository.clone());

    // Act: Store and retrieve state
    let test_state = CrawlState {
        url: "https://example.com".to_string(),
        depth: 2,
        completed: false,
    };

    state_store.save("test-session", &test_state).await.unwrap();

    // Verify: State persisted and retrievable
    let retrieved = state_store.load::<CrawlState>("test-session").await.unwrap();
    assert_eq!(retrieved.url, test_state.url);
    assert_eq!(retrieved.depth, test_state.depth);

    // Verify: Survives application restart
    drop(state_store);
    drop(repository);

    let new_repository = Repository::new(&db_path).await.unwrap();
    let new_state_store = StateStore::new(new_repository);
    let reloaded = new_state_store.load::<CrawlState>("test-session").await.unwrap();
    assert_eq!(reloaded.url, test_state.url);

    // Cleanup
    temp_dir.close().unwrap();
}

/// Test helper: Create test request
fn create_test_request(id: usize) -> Request {
    Request {
        id: format!("test-{}", id),
        url: format!("https://example.com/page{}", id),
        method: Method::GET,
        headers: Default::default(),
    }
}

/// Test helper: Setup test database
async fn setup_test_database() -> (TempDir, Repository) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let repository = Repository::new(&db_path).await.unwrap();
    (temp_dir, repository)
}

/// Test helper: Assert metrics recorded
fn assert_metrics_recorded(metrics: &MetricsCollector, expected_requests: u64) {
    let actual = metrics.get_counter("requests_total").unwrap();
    assert_eq!(
        actual, expected_requests,
        "Expected {} requests, got {}",
        expected_requests, actual
    );
}
