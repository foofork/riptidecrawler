# RipTide Test Utils

Testing utilities and helpers for the RipTide framework.

## Overview

`riptide-test-utils` provides shared testing infrastructure, mock servers, test fixtures, and helper functions for testing RipTide crates.

## Features

- **Mock HTTP Server**: Test HTTP server with configurable responses
- **Test Fixtures**: Pre-defined test data and HTML samples
- **Helper Functions**: Common testing utilities
- **Async Test Helpers**: Utilities for testing async code
- **Mock Browsers**: Simulated browser instances for testing
- **Test Harness**: Integration test setup helpers

## Usage

### Mock HTTP Server

```rust
use riptide_test_utils::*;

#[tokio::test]
async fn test_http_fetch() {
    let server = MockServer::start().await;
    server.mock("/test", MockResponse::ok("test content"));

    let response = fetch(&server.url("/test")).await?;
    assert_eq!(response.text(), "test content");
}
```

### Test Fixtures

```rust
use riptide_test_utils::fixtures::*;

#[test]
fn test_extraction() {
    let html = html_with_article();
    let extracted = extract_content(&html)?;
    assert!(!extracted.is_empty());
}
```

## License

Apache-2.0
