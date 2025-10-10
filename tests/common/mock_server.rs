//! Wiremock Test Utilities for RipTide v1.0
//!
//! This module provides reusable mock server utilities for integration tests,
//! eliminating external network dependencies and improving test reliability.
//!
//! # Phase 2 Goals
//! - Replace all example.com and httpbin.org calls with local mocks
//! - Provide common response fixtures (HTML, JSON, errors)
//! - Enable fast, deterministic, offline testing
//!
//! # Usage
//! ```rust
//! use crate::common::mock_server::{setup_mock_server, MockResponse};
//!
//! #[tokio::test]
//! async fn test_html_extraction() {
//!     let (server, url) = setup_mock_server(MockResponse::SimpleHtml).await;
//!     // Use url in your test
//! }
//! ```

use wiremock::{
    matchers::{method, path, header},
    Mock, MockServer, ResponseTemplate,
};

/// Common mock response types for testing
#[derive(Debug, Clone, Copy)]
pub enum MockResponse {
    /// Simple HTML page for basic extraction tests
    SimpleHtml,
    /// HTML with complex table structures
    HtmlWithTables,
    /// HTML with navigation and links
    HtmlWithNavigation,
    /// Large HTML document for performance testing
    LargeHtml,
    /// JSON API response
    JsonSuccess,
    /// JSON error response
    JsonError,
    /// Server error (500)
    ServerError,
    /// Not found (404)
    NotFound,
    /// Rate limit error (429)
    RateLimited,
    /// Timeout simulation (slow response)
    SlowResponse,
    /// Empty response
    Empty,
    /// Redirect (301)
    PermanentRedirect,
    /// Temporary redirect (302)
    TemporaryRedirect,
    /// Robots.txt file
    RobotsTxt,
}

impl MockResponse {
    /// Get the HTTP status code for this response type
    pub fn status_code(&self) -> u16 {
        match self {
            Self::SimpleHtml | Self::HtmlWithTables | Self::HtmlWithNavigation
                | Self::LargeHtml | Self::JsonSuccess | Self::Empty
                | Self::RobotsTxt | Self::SlowResponse => 200,
            Self::PermanentRedirect => 301,
            Self::TemporaryRedirect => 302,
            Self::NotFound => 404,
            Self::RateLimited => 429,
            Self::ServerError | Self::JsonError => 500,
        }
    }

    /// Get the content type for this response
    pub fn content_type(&self) -> &'static str {
        match self {
            Self::SimpleHtml | Self::HtmlWithTables | Self::HtmlWithNavigation
                | Self::LargeHtml => "text/html; charset=utf-8",
            Self::JsonSuccess | Self::JsonError => "application/json",
            Self::RobotsTxt => "text/plain",
            _ => "text/plain",
        }
    }

    /// Get the response body for this type
    pub fn body(&self) -> String {
        match self {
            Self::SimpleHtml => SIMPLE_HTML_FIXTURE.to_string(),
            Self::HtmlWithTables => HTML_WITH_TABLES_FIXTURE.to_string(),
            Self::HtmlWithNavigation => HTML_WITH_NAVIGATION_FIXTURE.to_string(),
            Self::LargeHtml => generate_large_html(),
            Self::JsonSuccess => r#"{"status":"success","data":{"message":"Test data"}}"#.to_string(),
            Self::JsonError => r#"{"status":"error","message":"Internal server error"}"#.to_string(),
            Self::ServerError => "Internal Server Error".to_string(),
            Self::NotFound => "Not Found".to_string(),
            Self::RateLimited => "Rate limit exceeded".to_string(),
            Self::Empty => String::new(),
            Self::PermanentRedirect => "Moved Permanently".to_string(),
            Self::TemporaryRedirect => "Found".to_string(),
            Self::RobotsTxt => ROBOTS_TXT_FIXTURE.to_string(),
            Self::SlowResponse => "Slow response".to_string(),
        }
    }
}

/// Setup a basic mock server with a single endpoint
///
/// Returns: (MockServer, URL) - The server and the full URL to call
pub async fn setup_mock_server(response_type: MockResponse) -> (MockServer, String) {
    let server = MockServer::start().await;
    let url = format!("{}/test", server.uri());

    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(
            ResponseTemplate::new(response_type.status_code())
                .set_body_string(response_type.body())
                .insert_header("content-type", response_type.content_type())
        )
        .mount(&server)
        .await;

    (server, url)
}

/// Setup a mock server with multiple endpoints
pub async fn setup_multi_endpoint_server() -> (MockServer, String) {
    let server = MockServer::start().await;
    let base_url = server.uri();

    // HTML endpoint
    Mock::given(method("GET"))
        .and(path("/html"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(SIMPLE_HTML_FIXTURE)
                .insert_header("content-type", "text/html; charset=utf-8")
        )
        .mount(&server)
        .await;

    // JSON endpoint
    Mock::given(method("GET"))
        .and(path("/json"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"status":"success"}"#)
                .insert_header("content-type", "application/json")
        )
        .mount(&server)
        .await;

    // Error endpoint
    Mock::given(method("GET"))
        .and(path("/error"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    // Robots.txt endpoint
    Mock::given(method("GET"))
        .and(path("/robots.txt"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(ROBOTS_TXT_FIXTURE)
                .insert_header("content-type", "text/plain")
        )
        .mount(&server)
        .await;

    (server, base_url)
}

/// Setup a mock server that simulates rate limiting after N requests
pub async fn setup_rate_limited_server(limit: usize) -> (MockServer, String) {
    let server = MockServer::start().await;
    let url = format!("{}/api", server.uri());

    // First N requests succeed
    Mock::given(method("GET"))
        .and(path("/api"))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .expect(limit)
        .mount(&server)
        .await;

    // Subsequent requests return 429
    Mock::given(method("GET"))
        .and(path("/api"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_string("Rate limit exceeded")
                .insert_header("retry-after", "60")
        )
        .mount(&server)
        .await;

    (server, url)
}

/// Setup a mock server that simulates intermittent failures
pub async fn setup_flaky_server(failure_rate: f32) -> (MockServer, String) {
    let server = MockServer::start().await;
    let url = format!("{}/flaky", server.uri());

    // Simulate failures based on rate
    let failures = (10.0 * failure_rate) as usize;
    let successes = 10 - failures;

    if failures > 0 {
        Mock::given(method("GET"))
            .and(path("/flaky"))
            .respond_with(ResponseTemplate::new(500))
            .expect(failures)
            .mount(&server)
            .await;
    }

    if successes > 0 {
        Mock::given(method("GET"))
            .and(path("/flaky"))
            .respond_with(ResponseTemplate::new(200).set_body_string("Success"))
            .expect(successes)
            .mount(&server)
            .await;
    }

    (server, url)
}

/// Setup a mock server with custom response delay
pub async fn setup_slow_server(delay_ms: u64) -> (MockServer, String) {
    let server = MockServer::start().await;
    let url = format!("{}/slow", server.uri());

    Mock::given(method("GET"))
        .and(path("/slow"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("Slow response")
                .set_delay(std::time::Duration::from_millis(delay_ms))
        )
        .mount(&server)
        .await;

    (server, url)
}

/// Setup a mock server that requires authentication
pub async fn setup_auth_server(valid_token: &str) -> (MockServer, String) {
    let server = MockServer::start().await;
    let url = format!("{}/protected", server.uri());
    let valid_token = valid_token.to_string();

    // Valid auth
    Mock::given(method("GET"))
        .and(path("/protected"))
        .and(header("authorization", format!("Bearer {}", valid_token)))
        .respond_with(ResponseTemplate::new(200).set_body_string("Authorized"))
        .mount(&server)
        .await;

    // Invalid auth
    Mock::given(method("GET"))
        .and(path("/protected"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    (server, url)
}

/// Setup a mock server with redirect chain
pub async fn setup_redirect_server(redirect_count: usize) -> (MockServer, String) {
    let server = MockServer::start().await;
    let base = server.uri();

    // Setup redirect chain
    for i in 0..redirect_count {
        let from = format!("/redirect/{}", i);
        let to = if i == redirect_count - 1 {
            format!("{}/final", base)
        } else {
            format!("{}/redirect/{}", base, i + 1)
        };

        Mock::given(method("GET"))
            .and(path(from.as_str()))
            .respond_with(
                ResponseTemplate::new(302)
                    .insert_header("location", to.as_str())
            )
            .mount(&server)
            .await;
    }

    // Final destination
    Mock::given(method("GET"))
        .and(path("/final"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Redirect completed"))
        .mount(&server)
        .await;

    let start_url = format!("{}/redirect/0", base);
    (server, start_url)
}

// ============================================================================
// Response Fixtures
// ============================================================================

const SIMPLE_HTML_FIXTURE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Simple Test Page</title>
    <meta name="description" content="A simple test page for extraction">
</head>
<body>
    <header>
        <h1>Test Page Title</h1>
        <nav>
            <a href="/page1">Page 1</a>
            <a href="/page2">Page 2</a>
        </nav>
    </header>
    <main>
        <article>
            <h2>Article Heading</h2>
            <p>This is a paragraph of test content. It contains some meaningful text for extraction testing.</p>
            <p>Another paragraph with more content to extract.</p>
        </article>
    </main>
    <footer>
        <p>&copy; 2025 Test Site</p>
    </footer>
</body>
</html>"#;

const HTML_WITH_TABLES_FIXTURE: &str = r#"<!DOCTYPE html>
<html>
<head><title>Tables Test Page</title></head>
<body>
    <h1>Product Catalog</h1>
    <table id="products" class="data-table">
        <thead>
            <tr>
                <th>Product ID</th>
                <th>Name</th>
                <th>Price</th>
                <th>Category</th>
            </tr>
        </thead>
        <tbody>
            <tr>
                <td>001</td>
                <td>Laptop</td>
                <td>$999.99</td>
                <td>Electronics</td>
            </tr>
            <tr>
                <td>002</td>
                <td>Mouse</td>
                <td>$24.99</td>
                <td>Accessories</td>
            </tr>
            <tr>
                <td>003</td>
                <td>Keyboard</td>
                <td>$79.99</td>
                <td>Accessories</td>
            </tr>
        </tbody>
    </table>

    <h2>Complex Table with Spans</h2>
    <table id="complex" class="complex-table">
        <tr>
            <th colspan="2">Header Span</th>
            <th>Single</th>
        </tr>
        <tr>
            <td rowspan="2">Row Span</td>
            <td>Data 1</td>
            <td>Data 2</td>
        </tr>
        <tr>
            <td>Data 3</td>
            <td>Data 4</td>
        </tr>
    </table>
</body>
</html>"#;

const HTML_WITH_NAVIGATION_FIXTURE: &str = r#"<!DOCTYPE html>
<html>
<head><title>Navigation Test</title></head>
<body>
    <nav class="main-nav">
        <ul>
            <li><a href="/home">Home</a></li>
            <li><a href="/products">Products</a></li>
            <li><a href="/about">About</a></li>
            <li><a href="/contact">Contact</a></li>
        </ul>
    </nav>
    <main>
        <h1>Welcome</h1>
        <section class="content">
            <p>Main content with <a href="/internal-link">internal links</a>.</p>
            <p>External link: <a href="https://external.example.com">External Site</a></p>
        </section>
        <aside>
            <h2>Related Links</h2>
            <ul>
                <li><a href="/related1">Related Page 1</a></li>
                <li><a href="/related2">Related Page 2</a></li>
            </ul>
        </aside>
    </main>
    <footer>
        <a href="/privacy">Privacy Policy</a>
        <a href="/terms">Terms of Service</a>
    </footer>
</body>
</html>"#;

const ROBOTS_TXT_FIXTURE: &str = r#"User-agent: *
Disallow: /admin/
Disallow: /private/
Allow: /public/

User-agent: Googlebot
Allow: /

Sitemap: https://example.com/sitemap.xml
"#;

/// Generate a large HTML document for performance testing
fn generate_large_html() -> String {
    let mut html = String::from(r#"<!DOCTYPE html>
<html>
<head><title>Large Test Document</title></head>
<body>
<h1>Large Document for Performance Testing</h1>
"#);

    // Generate 100 sections with content
    for i in 0..100 {
        html.push_str(&format!(
            r#"<section id="section-{}">
    <h2>Section {}</h2>
    <p>This is paragraph 1 of section {}. It contains some test content for extraction.</p>
    <p>This is paragraph 2 of section {}. More content here for testing purposes.</p>
    <p>This is paragraph 3 of section {}. Additional content for comprehensive testing.</p>
    <ul>
        <li>List item 1</li>
        <li>List item 2</li>
        <li>List item 3</li>
    </ul>
</section>
"#,
            i, i, i, i, i
        ));
    }

    html.push_str("</body></html>");
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_mock_server() {
        let (server, url) = setup_mock_server(MockResponse::SimpleHtml).await;

        let response = reqwest::get(&url).await.unwrap();
        assert_eq!(response.status(), 200);

        let body = response.text().await.unwrap();
        assert!(body.contains("Test Page Title"));

        drop(server); // Cleanup
    }

    #[tokio::test]
    async fn test_multi_endpoint_server() {
        let (server, base_url) = setup_multi_endpoint_server().await;

        // Test HTML endpoint
        let html_response = reqwest::get(format!("{}/html", base_url)).await.unwrap();
        assert_eq!(html_response.status(), 200);
        assert!(html_response.text().await.unwrap().contains("<html"));

        // Test JSON endpoint
        let json_response = reqwest::get(format!("{}/json", base_url)).await.unwrap();
        assert_eq!(json_response.status(), 200);

        // Test error endpoint
        let error_response = reqwest::get(format!("{}/error", base_url)).await.unwrap();
        assert_eq!(error_response.status(), 500);

        drop(server);
    }

    #[tokio::test]
    async fn test_rate_limited_server() {
        let (server, url) = setup_rate_limited_server(2).await;

        // First two requests should succeed
        assert_eq!(reqwest::get(&url).await.unwrap().status(), 200);
        assert_eq!(reqwest::get(&url).await.unwrap().status(), 200);

        // Third request should be rate limited
        let response = reqwest::get(&url).await.unwrap();
        assert_eq!(response.status(), 429);
        assert!(response.headers().contains_key("retry-after"));

        drop(server);
    }

    #[tokio::test]
    async fn test_slow_server() {
        let (server, url) = setup_slow_server(100).await;

        let start = std::time::Instant::now();
        let response = reqwest::get(&url).await.unwrap();
        let elapsed = start.elapsed();

        assert_eq!(response.status(), 200);
        assert!(elapsed >= std::time::Duration::from_millis(90));

        drop(server);
    }

    #[tokio::test]
    async fn test_auth_server() {
        let (server, url) = setup_auth_server("secret-token-123").await;
        let client = reqwest::Client::new();

        // Request without auth should fail
        let response = client.get(&url).send().await.unwrap();
        assert_eq!(response.status(), 401);

        // Request with valid auth should succeed
        let response = client
            .get(&url)
            .header("authorization", "Bearer secret-token-123")
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200);

        drop(server);
    }

    #[tokio::test]
    async fn test_redirect_server() {
        let (server, url) = setup_redirect_server(3).await;

        let response = reqwest::get(&url).await.unwrap();
        assert_eq!(response.status(), 200);
        assert!(response.text().await.unwrap().contains("Redirect completed"));

        drop(server);
    }
}
