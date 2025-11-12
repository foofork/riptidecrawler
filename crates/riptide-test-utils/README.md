# 🛠️ RipTide Test Utils - Testing Infrastructure

**Category:** Testing & Development
**Purpose:** Shared test utilities, fixtures, and helpers for testing RipTide crates

## Quick Overview

`riptide-test-utils` provides a comprehensive testing toolkit for the RipTide ecosystem. It includes test fixtures, data factories, assertion helpers, and utilities for testing async code - everything you need to write thorough, maintainable tests.

## Why This Exists

Testing infrastructure was being duplicated across crates, with each crate implementing its own:
- Test fixtures
- Mock data generators
- Assertion helpers
- Async test utilities

This crate centralizes all testing utilities, ensuring consistency and reducing duplication.

## Key Features

- **Test Fixtures**: Pre-defined test data and HTML samples
- **Data Factories**: Builders for creating test objects
- **Assertion Helpers**: Custom assertions for common test patterns
- **Async Utilities**: Helpers for testing async code
- **HTTP Mock Server**: Optional mock server for integration tests (with `http-mock` feature)
- **Temporary Files**: Easy management of temporary test files

## Quick Start

```rust
use riptide_test_utils::{fixtures, factories, assertions};

#[tokio::test]
async fn test_extraction() {
    // Use pre-made HTML fixture
    let html = fixtures::html_with_article();

    // Create test data
    let config = factories::spider_config().build();

    // Run extraction
    let result = extract_content(&html, config).await?;

    // Use custom assertions
    assertions::assert_not_empty(&result.text);
    assertions::assert_contains(&result.text, "article content");
}
```

## Fixtures Module

Pre-defined test data for common scenarios:

```rust
use riptide_test_utils::fixtures;

// HTML fixtures
let simple_html = fixtures::html_simple();
let complex_html = fixtures::html_with_nested_divs();
let article_html = fixtures::html_with_article();
let list_html = fixtures::html_with_lists();
let table_html = fixtures::html_with_table();
let form_html = fixtures::html_with_form();

// URL fixtures
let test_url = fixtures::url_simple();
let complex_url = fixtures::url_with_params();
let redirect_url = fixtures::url_redirect();

// JSON fixtures
let json_data = fixtures::json_article();
let json_array = fixtures::json_list();

// Response fixtures
let success_response = fixtures::response_200_ok();
let not_found_response = fixtures::response_404();
let server_error = fixtures::response_500();
```

### Custom HTML Fixtures

```rust
use riptide_test_utils::fixtures;

#[test]
fn test_with_custom_html() {
    let html = fixtures::html_builder()
        .with_title("Test Page")
        .with_meta_description("Test description")
        .with_article("Article content")
        .with_links(vec!["https://example.com", "https://test.com"])
        .build();

    // Test with generated HTML
}
```

## Factories Module

Builders for creating test objects:

```rust
use riptide_test_utils::factories;

// Spider configuration factory
let config = factories::spider_config()
    .with_concurrency(5)
    .with_timeout(30)
    .with_max_depth(3)
    .build();

// URL factory
let url = factories::url()
    .with_scheme("https")
    .with_host("example.com")
    .with_path("/test")
    .with_query("page=1")
    .build();

// Extraction result factory
let result = factories::extraction_result()
    .with_url("https://example.com")
    .with_title("Test Title")
    .with_content("Test content")
    .with_links(5)
    .with_images(3)
    .build();

// Job factory
let job = factories::job()
    .with_url("https://example.com")
    .with_priority_high()
    .with_retry_count(3)
    .build();
```

### Batch Data Generation

```rust
use riptide_test_utils::factories;

// Generate multiple test URLs
let urls = factories::urls_batch(10, |i| {
    format!("https://example.com/page/{}", i)
});

// Generate test extraction results
let results = factories::extraction_results_batch(5, |i| {
    factories::extraction_result()
        .with_url(&format!("https://example.com/{}", i))
        .with_title(&format!("Page {}", i))
        .build()
});
```

## Assertions Module

Custom assertions for common test patterns:

```rust
use riptide_test_utils::assertions::*;

#[test]
fn test_custom_assertions() {
    let text = "Hello, world!";
    let numbers = vec![1, 2, 3, 4, 5];
    let url = "https://example.com";

    // String assertions
    assert_not_empty(text);
    assert_contains(text, "world");
    assert_starts_with(text, "Hello");
    assert_ends_with(text, "!");
    assert_length_between(text, 10, 20);

    // Collection assertions
    assert_length(&numbers, 5);
    assert_contains_item(&numbers, &3);
    assert_all_unique(&numbers);

    // URL assertions
    assert_valid_url(url);
    assert_https_url(url);
    assert_domain_matches(url, "example.com");

    // Numeric assertions
    assert_in_range(42, 0, 100);
    assert_positive(42);

    // Error assertions
    let result: Result<i32, String> = Err("failed".to_string());
    assert_error(&result);
    assert_error_contains(&result, "failed");
}
```

### Async Assertions

```rust
use riptide_test_utils::assertions::*;

#[tokio::test]
async fn test_async_assertions() {
    // Assert timeout
    assert_completes_within(
        Duration::from_secs(5),
        async { slow_operation().await }
    ).await;

    // Assert future resolves
    assert_resolves(async {
        Ok::<_, Error>(42)
    }).await;

    // Assert future rejects
    assert_rejects(async {
        Err::<(), _>("error")
    }).await;
}
```

## Async Test Utilities

```rust
use riptide_test_utils::{async_test, tokio};

// Use provided tokio runtime
#[tokio::test]
async fn test_async_operation() {
    let result = async_operation().await;
    assert!(result.is_ok());
}

// Test with timeout
#[tokio::test(flavor = "multi_thread")]
async fn test_with_timeout() {
    tokio::time::timeout(
        Duration::from_secs(5),
        long_operation()
    ).await.expect("Operation timed out");
}

// Test channels
#[tokio::test]
async fn test_channels() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    tokio::spawn(async move {
        tx.send(42).await.unwrap();
    });

    let value = rx.recv().await.unwrap();
    assert_eq!(value, 42);
}
```

## HTTP Mock Server (Optional Feature)

With the `http-mock` feature enabled:

```rust
use riptide_test_utils::mock_server::{MockServer, MockResponse};

#[tokio::test]
async fn test_with_mock_server() {
    // Start mock server
    let server = MockServer::start().await;

    // Configure mock endpoints
    server.mock(
        "/api/data",
        MockResponse::ok(r#"{"status": "success"}"#)
            .with_header("Content-Type", "application/json")
    );

    server.mock(
        "/api/error",
        MockResponse::server_error("Internal error")
    );

    // Make requests to mock server
    let response = reqwest::get(&server.url("/api/data")).await?;
    assert!(response.status().is_success());

    let body = response.text().await?;
    assert_contains(&body, "success");
}
```

### Mock Response Builder

```rust
use riptide_test_utils::mock_server::MockResponse;

// Success responses
let ok = MockResponse::ok("response body");
let json = MockResponse::json(r#"{"key": "value"}"#);
let html = MockResponse::html("<html><body>Test</body></html>");

// Error responses
let not_found = MockResponse::not_found();
let server_error = MockResponse::server_error("Error message");
let bad_request = MockResponse::bad_request("Invalid input");

// Custom responses
let custom = MockResponse::new(201)
    .with_header("Location", "/resource/123")
    .with_header("X-Custom", "value")
    .with_body("Created");

// Delayed responses (for timeout testing)
let slow = MockResponse::ok("data")
    .with_delay(Duration::from_secs(2));

// Redirect responses
let redirect = MockResponse::redirect("/new-location");
```

## Temporary Files

Manage temporary files for tests:

```rust
use riptide_test_utils::tempfile;
use std::fs;

#[test]
fn test_with_temp_file() {
    // Create temporary file
    let temp = tempfile::NamedTempFile::new()?;

    // Write test data
    fs::write(temp.path(), b"test data")?;

    // Use in test
    let content = fs::read(temp.path())?;
    assert_eq!(content, b"test data");

    // File automatically deleted when temp goes out of scope
}

#[test]
fn test_with_temp_dir() {
    let temp_dir = tempfile::TempDir::new()?;

    // Create files in temp directory
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, b"content")?;

    // Test with files
    assert!(file_path.exists());

    // Directory and contents automatically cleaned up
}
```

## Re-exported Dependencies

Commonly used test dependencies are re-exported for convenience:

```rust
use riptide_test_utils::{
    anyhow,      // Error handling
    tokio,       // Async runtime
    tempfile,    // Temporary files
};
```

## Integration with Other Crates

### Example: Testing Extraction

```rust
use riptide_test_utils::{fixtures, factories, assertions};
use riptide_extraction::extract;

#[tokio::test]
async fn test_article_extraction() {
    // Use fixture
    let html = fixtures::html_with_article();

    // Create config
    let config = factories::extraction_config()
        .with_css_selectors()
        .build();

    // Run extraction
    let result = extract(&html, config).await?;

    // Assert results
    assertions::assert_not_empty(&result.title);
    assertions::assert_contains(&result.content, "article");
    assertions::assert_length_at_least(&result.links, 1);
}
```

### Example: Testing API

```rust
use riptide_test_utils::mock_server::MockServer;

#[tokio::test]
async fn test_api_client() {
    let server = MockServer::start().await;

    server.mock("/api/items", MockResponse::json(r#"
        [{"id": 1, "name": "Item 1"}, {"id": 2, "name": "Item 2"}]
    "#));

    let client = ApiClient::new(&server.url(""));
    let items = client.get_items().await?;

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].name, "Item 1");
}
```

## Testing Patterns

### Parameterized Tests

```rust
use riptide_test_utils::fixtures;

#[test]
fn test_multiple_html_types() {
    let test_cases = vec![
        fixtures::html_simple(),
        fixtures::html_with_article(),
        fixtures::html_with_lists(),
    ];

    for html in test_cases {
        let result = parse_html(&html);
        assert!(result.is_ok());
    }
}
```

### Property-Based Testing

```rust
use riptide_test_utils::factories;

#[test]
fn test_url_normalization_properties() {
    for _ in 0..100 {
        let url = factories::url_random();
        let normalized = normalize_url(&url);

        // Properties that should always hold
        assert!(normalized.starts_with("http"));
        assert!(!normalized.contains("//example")); // No double slashes
        assert_eq!(normalize_url(&normalized), normalized); // Idempotent
    }
}
```

## Best Practices

1. **Use Fixtures**: Leverage pre-made fixtures for consistency
2. **Use Factories**: Build test data with factories, not manual construction
3. **Use Assertions**: Custom assertions provide better error messages
4. **Clean Up**: Use `tempfile` for automatic cleanup
5. **Mock External Services**: Use mock server instead of real HTTP calls
6. **Test Async Properly**: Use `#[tokio::test]` for async tests

## Testing

```bash
# Run test-utils tests
cargo test -p riptide-test-utils

# With HTTP mock feature
cargo test -p riptide-test-utils --features http-mock

# Test in other crates using test-utils
cargo test -p riptide-extraction
cargo test -p riptide-spider
```

## Feature Flags

```toml
[features]
default = []
http-mock = ["axum", "tower"]  # Enable HTTP mock server
```

## Dependencies

- `anyhow` - Error handling
- `tokio` - Async runtime
- `tempfile` - Temporary files
- `serde` - Serialization (for fixtures)
- `serde_json` - JSON fixtures

Optional:
- `axum` - HTTP server (with `http-mock` feature)
- `tower` - HTTP middleware (with `http-mock` feature)

## License

Apache-2.0

## Related Crates

This crate is used by all RipTide crates for testing:
- `riptide-extraction`
- `riptide-spider`
- `riptide-fetch`
- `riptide-api`
- `riptide-workers`
- And more...
