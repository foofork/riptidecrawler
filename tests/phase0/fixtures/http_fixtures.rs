// Phase 0: HTTP Test Fixtures - Wiremock/HTTPMock Setup
// Recorded HTTP responses for deterministic testing without Docker

use std::time::Duration;

#[cfg(test)]
pub mod http_mocks {
    use super::*;

    /// Create mock server with robots.txt fixture
    /// Used for testing spider robots.txt respect
    pub async fn mock_robots_server() -> wiremock::MockServer {
        let server = wiremock::MockServer::start().await;

        // Recorded from riptidecrawler-test-sites :5003
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/robots.txt"))
            .respond_with(
                wiremock::ResponseTemplate::new(200)
                    .set_body_string(
                        "User-agent: *\n\
                         Disallow: /admin\n\
                         Disallow: /private\n\
                         Crawl-delay: 1\n"
                    )
                    .insert_header("Content-Type", "text/plain")
            )
            .mount(&server)
            .await;

        server
    }

    /// Create mock server with sitemap.xml fixture
    pub async fn mock_sitemap_server() -> wiremock::MockServer {
        let server = wiremock::MockServer::start().await;

        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/sitemap.xml"))
            .respond_with(
                wiremock::ResponseTemplate::new(200)
                    .set_body_string(include_str!("golden/sitemap.xml"))
                    .insert_header("Content-Type", "application/xml")
            )
            .mount(&server)
            .await;

        server
    }

    /// Create mock server that simulates timeouts
    /// Used for testing retry logic and timeout handling
    pub async fn mock_timeout_server(delay_ms: u64) -> wiremock::MockServer {
        let server = wiremock::MockServer::start().await;

        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .respond_with(
                wiremock::ResponseTemplate::new(200)
                    .set_delay(Duration::from_millis(delay_ms))
                    .set_body_string("Delayed response")
            )
            .mount(&server)
            .await;

        server
    }

    /// Create mock server that fails N times then succeeds
    /// Used for testing retry policy with exponential backoff
    pub async fn mock_flaky_server(fail_count: usize) -> wiremock::MockServer {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let server = wiremock::MockServer::start().await;
        let counter = Arc::new(AtomicUsize::new(0));

        let counter_clone = counter.clone();

        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .respond_with(move |_req: &wiremock::Request| {
                let count = counter_clone.fetch_add(1, Ordering::SeqCst);

                if count < fail_count {
                    wiremock::ResponseTemplate::new(503)
                        .set_body_string("Service Unavailable")
                } else {
                    wiremock::ResponseTemplate::new(200)
                        .set_body_string("Success after retries")
                }
            })
            .mount(&server)
            .await;

        server
    }

    /// Create mock server with rate limiting headers
    /// Used for testing rate limit handling
    pub async fn mock_rate_limited_server() -> wiremock::MockServer {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        let server = wiremock::MockServer::start().await;
        let request_count = Arc::new(AtomicU32::new(0));

        let count_clone = request_count.clone();

        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .respond_with(move |_req: &wiremock::Request| {
                let count = count_clone.fetch_add(1, Ordering::SeqCst);

                if count < 10 {
                    // Allow first 10 requests
                    wiremock::ResponseTemplate::new(200)
                        .set_body_string("OK")
                        .insert_header("X-RateLimit-Limit", "10")
                        .insert_header("X-RateLimit-Remaining", &(9 - count).to_string())
                } else {
                    // Rate limit subsequent requests
                    wiremock::ResponseTemplate::new(429)
                        .set_body_string("Too Many Requests")
                        .insert_header("X-RateLimit-Limit", "10")
                        .insert_header("X-RateLimit-Remaining", "0")
                        .insert_header("Retry-After", "60")
                }
            })
            .mount(&server)
            .await;

        server
    }

    /// Create mock server with redirect chain
    /// Used for testing redirect following
    pub async fn mock_redirect_server() -> wiremock::MockServer {
        let server = wiremock::MockServer::start().await;

        // First redirect
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/redirect1"))
            .respond_with(
                wiremock::ResponseTemplate::new(302)
                    .insert_header("Location", "/redirect2")
            )
            .mount(&server)
            .await;

        // Second redirect
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/redirect2"))
            .respond_with(
                wiremock::ResponseTemplate::new(302)
                    .insert_header("Location", "/final")
            )
            .mount(&server)
            .await;

        // Final destination
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/final"))
            .respond_with(
                wiremock::ResponseTemplate::new(200)
                    .set_body_string("Final destination")
            )
            .mount(&server)
            .await;

        server
    }

    /// Create mock server with various HTTP errors
    /// Used for testing error handling
    pub async fn mock_error_server() -> wiremock::MockServer {
        let server = wiremock::MockServer::start().await;

        // 404 Not Found
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/notfound"))
            .respond_with(
                wiremock::ResponseTemplate::new(404)
                    .set_body_string("Not Found")
            )
            .mount(&server)
            .await;

        // 500 Internal Server Error
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/error"))
            .respond_with(
                wiremock::ResponseTemplate::new(500)
                    .set_body_string("Internal Server Error")
            )
            .mount(&server)
            .await;

        // 401 Unauthorized
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/unauthorized"))
            .respond_with(
                wiremock::ResponseTemplate::new(401)
                    .set_body_string("Unauthorized")
                    .insert_header("WWW-Authenticate", "Bearer")
            )
            .mount(&server)
            .await;

        server
    }

    /// Create mock server with streaming response
    /// Used for testing streaming/chunked responses
    pub async fn mock_streaming_server() -> wiremock::MockServer {
        let server = wiremock::MockServer::start().await;

        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/stream"))
            .respond_with(
                wiremock::ResponseTemplate::new(200)
                    .set_body_string("chunk1\nchunk2\nchunk3\n")
                    .insert_header("Transfer-Encoding", "chunked")
            )
            .mount(&server)
            .await;

        server
    }

    /// Create mock server with event calendar (ICS) response
    /// Used for testing ICS extraction strategy
    pub async fn mock_calendar_server() -> wiremock::MockServer {
        let server = wiremock::MockServer::start().await;

        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/events.ics"))
            .respond_with(
                wiremock::ResponseTemplate::new(200)
                    .set_body_string(include_str!("golden/events.ics"))
                    .insert_header("Content-Type", "text/calendar")
            )
            .mount(&server)
            .await;

        server
    }

    /// Create mock server with JSON-LD structured data
    /// Used for testing JSON-LD extraction strategy
    pub async fn mock_jsonld_server() -> wiremock::MockServer {
        let server = wiremock::MockServer::start().await;

        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/event"))
            .respond_with(
                wiremock::ResponseTemplate::new(200)
                    .set_body_string(include_str!("golden/event_jsonld.html"))
                    .insert_header("Content-Type", "text/html")
            )
            .mount(&server)
            .await;

        server
    }
}

// Golden test fixtures (recorded responses)

/// Load golden fixture file
pub fn load_fixture(name: &str) -> String {
    let path = format!("tests/phase0/fixtures/golden/{}", name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to load fixture: {}", path))
}

/// Verify response matches golden fixture
pub fn assert_matches_fixture(actual: &str, fixture_name: &str) {
    let expected = load_fixture(fixture_name);
    assert_eq!(actual.trim(), expected.trim(),
        "Response should match golden fixture: {}", fixture_name);
}

// Test helper macros

/// Create a mock server and get its URI for testing
#[macro_export]
macro_rules! test_server {
    ($mock_fn:ident) => {{
        let server = $mock_fn().await;
        (server.uri(), server)
    }};
}

/// Assert HTTP request succeeds with expected status
#[macro_export]
macro_rules! assert_http_success {
    ($response:expr) => {{
        assert!($response.status().is_success(),
            "HTTP request should succeed, got: {}", $response.status());
    }};
    ($response:expr, $expected_status:expr) => {{
        assert_eq!($response.status(), $expected_status,
            "Expected status {}, got: {}", $expected_status, $response.status());
    }};
}

/// Assert HTTP request fails with expected status
#[macro_export]
macro_rules! assert_http_error {
    ($response:expr, $expected_status:expr) => {{
        assert_eq!($response.status(), $expected_status,
            "Expected error status {}, got: {}", $expected_status, $response.status());
    }};
}
