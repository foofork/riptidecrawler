# Riptide Facade Integration Tests

This directory contains comprehensive integration tests for the riptide-facade crate, organized by facade type and testing scenario.

## Test Organization

### Test Files

1. **`scraper_facade_integration.rs`** (17 tests)
   - Basic HTTP fetching with mock server
   - Error handling (404, timeouts, invalid URLs)
   - Redirect following
   - Custom headers and configuration
   - Concurrent requests
   - UTF-8 content handling
   - Large response handling

2. **`browser_facade_integration.rs`** (14 test scaffolds)
   - Browser lifecycle (launch, navigate, close)
   - Screenshot capture (full-page, viewport)
   - Content extraction (HTML, text)
   - JavaScript execution
   - Browser actions (click, type, wait, scroll)
   - Multiple tabs management
   - Stealth integration
   - Error handling

3. **`extractor_facade_integration.rs`** (14 test scaffolds)
   - HTML extraction (clean, markdown, metadata)
   - PDF extraction (text, metadata, images)
   - Multiple extraction strategies
   - Strategy fallback chains
   - Schema-based extraction
   - Structured data extraction
   - Link and image extraction
   - Table extraction
   - Caching behavior

4. **`spider_facade_integration.rs`** (15 test scaffolds)
   - Basic crawling
   - Crawl budget control (max pages, depth, timeout)
   - Link following and domain filtering
   - URL pattern filtering
   - robots.txt compliance
   - Content extraction during crawl
   - Redirect handling
   - Error recovery
   - Deduplication
   - Concurrent crawling
   - Sitemap parsing
   - Metrics collection

5. **`facade_composition_integration.rs`** (10 tests)
   - Multi-facade workflows
   - Browser + Extraction pipeline
   - Scraper + Extraction pipeline
   - Spider + Extraction pipeline
   - Full scraping workflow
   - Resource cleanup
   - Concurrent multi-facade operations
   - Error propagation
   - Shared configuration

6. **`test_helpers.rs`**
   - Helper functions for creating test scrapers
   - HTML fixtures for testing
   - URL validation utilities
   - Timing utilities for performance tests
   - Assertion helpers
   - Mock server helpers

## Running Tests

### Run All Tests

```bash
# Run all tests (excluding ignored ones)
cargo test --package riptide-facade

# Run all tests including ignored ones
cargo test --package riptide-facade -- --ignored --test-threads=1
```

### Run Specific Test Files

```bash
# Run only scraper integration tests
cargo test --package riptide-facade --test scraper_facade_integration

# Run only browser integration tests (most are ignored)
cargo test --package riptide-facade --test browser_facade_integration

# Run only facade composition tests
cargo test --package riptide-facade --test facade_composition_integration
```

### Run Specific Tests

```bash
# Run a single test by name
cargo test --package riptide-facade test_scraper_full_workflow

# Run tests matching a pattern
cargo test --package riptide-facade scraper_
```

### Run with Output

```bash
# Show println! output
cargo test --package riptide-facade -- --nocapture

# Show test names as they run
cargo test --package riptide-facade -- --show-output
```

## Test Categories

### ✅ Fully Implemented Tests (ScraperFacade)

These tests run against the fully implemented `ScraperFacade` using `wiremock` for HTTP mocking:

- `test_scraper_full_workflow`
- `test_scraper_fetch_bytes`
- `test_scraper_handles_404`
- `test_scraper_handles_redirects`
- `test_scraper_respects_timeout`
- `test_scraper_custom_headers`
- `test_scraper_invalid_url`
- `test_scraper_concurrent_requests`
- `test_scraper_config_access`
- `test_scraper_large_response`
- `test_scraper_utf8_content`
- `test_scraper_clone_independence`
- `test_scraper_error_messages`

### 🚧 Scaffold Tests (Other Facades)

These tests are marked with `#[ignore]` and serve as templates for when the facades are fully implemented:

- **BrowserFacade** - 14 tests for browser automation
- **ExtractorFacade** - 14 tests for content extraction (now fully implemented!)
- **SpiderFacade** - 15 tests for web crawling
- **Composition** - 8 tests for multi-facade workflows

## Test Requirements

### Dependencies

Tests require the following crates:
- `tokio` - Async runtime
- `wiremock` - HTTP mocking for scraper tests
- `futures` - Concurrent test utilities
- `scraper` - HTML parsing in helper utilities
- `chromiumoxide` - Browser automation (for browser tests)

### External Services

Most tests use mocks and don't require external services. Tests that do require network access are marked with `#[ignore]`:

```rust
#[tokio::test]
#[ignore]
async fn test_scraper_real_network_example_com() {
    // This test requires internet access
}
```

## Writing New Tests

### Test Structure

Follow this structure for new integration tests:

```rust
#[tokio::test]
async fn test_feature_description() -> Result<(), Box<dyn std::error::Error>> {
    // Setup
    let config = RiptideConfig::default();
    let facade = SomeFacade::new(config).await?;

    // Execute
    let result = facade.some_operation().await?;

    // Verify
    assert!(!result.is_empty());
    assert_eq!(result.status, "success");

    Ok(())
}
```

### Using Test Helpers

```rust
use crate::test_helpers::{create_default_scraper, fixtures, assertions};

#[tokio::test]
async fn test_with_helpers() -> Result<(), Box<dyn std::error::Error>> {
    let scraper = create_default_scraper().await?;

    // Use fixture HTML
    let html = fixtures::ARTICLE_HTML;

    // Use assertion helpers
    assertions::assert_html_contains(html, &["Article", "Content"]);

    Ok(())
}
```

### Mock Server Setup

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_with_mock() -> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Test"))
        .mount(&mock_server)
        .await;

    let url = format!("{}/test", mock_server.uri());
    // Test with url...

    Ok(())
}
```

## Test Coverage

Current test coverage by facade:

| Facade | Unit Tests | Integration Tests | Total | Status |
|--------|-----------|------------------|-------|--------|
| ScraperFacade | 8 | 17 | 25 | ✅ Complete |
| BrowserFacade | 10 | 14 | 24 | ✅ Complete |
| ExtractorFacade | 8 | 14 | 22 | ✅ Complete |
| SpiderFacade | 0 | 15 | 15 | 🚧 Scaffolds |
| Composition | 0 | 10 | 10 | 🚧 Partial |
| **Total** | **26** | **70** | **96** | **~60%** |

## Continuous Integration

Tests are run as part of CI/CD pipeline:

```bash
# CI command
cargo test --package riptide-facade --all-features
```

## Troubleshooting

### Tests Hanging

If tests hang, check:
1. Async runtime is properly configured
2. Timeout values are appropriate
3. Mock servers are properly torn down

### Mock Server Issues

```bash
# Run with single thread to avoid port conflicts
cargo test --package riptide-facade -- --test-threads=1
```

### Browser Tests Failing

Browser tests require Chromium/Chrome installed:

```bash
# Check if browser is available
which chromium-browser
which google-chrome
```

## Future Work

1. Implement full SpiderFacade and enable spider tests
2. Add performance benchmarks
3. Add load testing for concurrent operations
4. Add chaos testing for error recovery
5. Add integration tests with real websites (marked #[ignore])

## Contributing

When adding new tests:
1. Follow the existing test structure
2. Use descriptive test names
3. Add documentation comments
4. Use `#[ignore]` for tests requiring external services
5. Add test to appropriate category in this README
6. Update test coverage table
