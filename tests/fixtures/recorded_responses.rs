// Test fixtures with recorded HTTP responses for deterministic testing
// Uses wiremock for fast, reliable CI tests without Docker dependencies

use wiremock::{MockServer, Mock, ResponseTemplate, matchers};

/// Mock robots.txt server for spider tests
/// Recorded response simulates robots.txt disallow rules
pub async fn mock_robots_server() -> MockServer {
    let server = MockServer::start().await;

    // Simulate robots.txt with disallow rules
    Mock::given(matchers::path("/robots.txt"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string("User-agent: *\nDisallow: /admin\nDisallow: /private\n"))
        .mount(&server)
        .await;

    // Allowed pages
    Mock::given(matchers::path("/public"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string("<html><body><h1>Public Page</h1></body></html>"))
        .mount(&server)
        .await;

    // Disallowed page (should not be crawled if robots.txt respected)
    Mock::given(matchers::path("/admin"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string("<html><body><h1>Admin Page</h1></body></html>"))
        .mount(&server)
        .await;

    server
}

/// Mock HTML page server with event content
/// Simulates a page with structured event data
pub async fn mock_html_page_server() -> MockServer {
    let server = MockServer::start().await;

    let html_content = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Sample Event Page</title>
</head>
<body>
    <div class="event">
        <h1 class="title">Tech Conference 2025</h1>
        <p class="description">Annual technology conference</p>
        <time class="start-date">2025-03-15T09:00:00Z</time>
        <time class="end-date">2025-03-17T18:00:00Z</time>
        <div class="location">San Francisco, CA</div>
        <a href="https://example.com/register" class="url">Register</a>
    </div>
    <div class="event">
        <h1 class="title">Developer Meetup</h1>
        <p class="description">Monthly developer meetup</p>
        <time class="start-date">2025-04-01T18:00:00Z</time>
        <div class="location">Online</div>
    </div>
</body>
</html>
    "#;

    Mock::given(matchers::path("/events.html"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(html_content)
            .insert_header("content-type", "text/html; charset=utf-8"))
        .mount(&server)
        .await;

    server
}

/// Mock JSON-LD server with structured event data
/// Simulates schema.org Event markup
pub async fn mock_jsonld_server() -> MockServer {
    let server = MockServer::start().await;

    let html_with_jsonld = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Event with JSON-LD</title>
    <script type="application/ld+json">
    {
      "@context": "https://schema.org",
      "@type": "Event",
      "name": "AI Workshop 2025",
      "description": "Hands-on workshop on artificial intelligence",
      "startDate": "2025-05-10T10:00:00-07:00",
      "endDate": "2025-05-10T16:00:00-07:00",
      "location": {
        "@type": "Place",
        "name": "Tech Hub",
        "address": {
          "@type": "PostalAddress",
          "streetAddress": "123 Main St",
          "addressLocality": "Seattle",
          "addressRegion": "WA",
          "postalCode": "98101",
          "addressCountry": "US"
        }
      },
      "organizer": {
        "@type": "Organization",
        "name": "AI Community",
        "url": "https://example.com/ai-community"
      },
      "offers": {
        "@type": "Offer",
        "url": "https://example.com/tickets",
        "price": "50",
        "priceCurrency": "USD",
        "availability": "https://schema.org/InStock"
      }
    }
    </script>
</head>
<body>
    <h1>AI Workshop 2025</h1>
    <p>Hands-on workshop on artificial intelligence</p>
</body>
</html>
    "#;

    Mock::given(matchers::path("/event-jsonld.html"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(html_with_jsonld)
            .insert_header("content-type", "text/html; charset=utf-8"))
        .mount(&server)
        .await;

    server
}

/// Mock server with network errors for retry testing
pub async fn mock_flaky_server() -> MockServer {
    let server = MockServer::start().await;

    // Returns 500 errors for retry logic testing
    Mock::given(matchers::path("/flaky-endpoint"))
        .respond_with(ResponseTemplate::new(500)
            .set_body_string("Internal Server Error"))
        .mount(&server)
        .await;

    // Returns timeout for timeout testing (with delay)
    Mock::given(matchers::path("/slow-endpoint"))
        .respond_with(ResponseTemplate::new(200)
            .set_delay(std::time::Duration::from_secs(10))
            .set_body_string("Slow response"))
        .mount(&server)
        .await;

    server
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_robots_server_setup() {
        let server = mock_robots_server().await;
        let client = reqwest::Client::new();

        let response = client
            .get(format!("{}/robots.txt", server.uri()))
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let body = response.text().await.expect("Failed to read body");
        assert!(body.contains("User-agent: *"));
        assert!(body.contains("Disallow: /admin"));
    }

    #[tokio::test]
    async fn test_html_page_server_setup() {
        let server = mock_html_page_server().await;
        let client = reqwest::Client::new();

        let response = client
            .get(format!("{}/events.html", server.uri()))
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let body = response.text().await.expect("Failed to read body");
        assert!(body.contains("Tech Conference 2025"));
        assert!(body.contains("Developer Meetup"));
    }

    #[tokio::test]
    async fn test_jsonld_server_setup() {
        let server = mock_jsonld_server().await;
        let client = reqwest::Client::new();

        let response = client
            .get(format!("{}/event-jsonld.html", server.uri()))
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let body = response.text().await.expect("Failed to read body");
        assert!(body.contains("application/ld+json"));
        assert!(body.contains("AI Workshop 2025"));
    }
}
