# RipTide Test Utilities - Phase 2

Common test infrastructure for network-independent testing.

## Quick Start

```rust
use tests::common::mock_server::{setup_mock_server, MockResponse};

#[tokio::test]
async fn test_example() {
    let (server, url) = setup_mock_server(MockResponse::SimpleHtml).await;
    let response = reqwest::get(&url).await.unwrap();
    assert_eq!(response.status(), 200);
}
```

## Available Mock Types

- `SimpleHtml` - Basic HTML page
- `HtmlWithTables` - Table extraction testing
- `HtmlWithNavigation` - Link extraction
- `LargeHtml` - Performance testing
- `JsonSuccess` / `JsonError` - API responses
- `ServerError` - 500 errors
- `NotFound` - 404 errors
- `RateLimited` - 429 with retry-after
- `SlowResponse` - Timeout testing
- `RobotsTxt` - robots.txt testing

## Advanced Setup

```rust
// Multi-endpoint server
let (server, base_url) = setup_multi_endpoint_server().await;
// Endpoints: /html, /json, /error, /robots.txt

// Rate limiting (2 OK, then 429)
let (server, url) = setup_rate_limited_server(2).await;

// Slow responses (500ms delay)
let (server, url) = setup_slow_server(500).await;

// Authentication
let (server, url) = setup_auth_server("token").await;

// Redirect chains (3 redirects)
let (server, url) = setup_redirect_server(3).await;
```

## Phase 2 Status

✅ Wiremock utilities created
✅ Common test fixtures available
✅ Integration tests already using wiremock
✅ Zero external network dependencies

See `mock_server.rs` for full API documentation.
