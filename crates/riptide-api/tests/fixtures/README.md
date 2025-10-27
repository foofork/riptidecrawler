# Test Fixtures for Integration Testing

This directory contains test fixtures that provide sample data for API integration testing. These fixtures ensure that all API endpoints have valid test data available, preventing 404 errors during schema validation tests.

## Overview

The fixtures module provides:

1. **Table Extraction Fixtures** - Sample HTML tables with various structures
2. **Session Management Fixtures** - Browser session data with cookies
3. **Test Data Store** - Centralized, thread-safe test data management

## Structure

```
fixtures/
├── mod.rs              # Main module with FixtureManager
├── tables.rs           # Table extraction test data
├── sessions.rs         # Session management test data
├── test_data.rs        # Centralized test data store
└── README.md           # This file
```

## Usage

### Basic Usage

```rust
use crate::fixtures::FixtureManager;

#[tokio::test]
async fn test_table_export() {
    let manager = FixtureManager::new();

    // Get a table fixture
    let table = manager.get_table("table_12345").unwrap();

    // Test CSV export
    let csv = crate::fixtures::tables::table_to_csv(&table);
    assert!(csv.contains("Product ID,Name,Price,Category"));
}
```

### Available Fixtures

#### Table Fixtures

The following table IDs are available by default:

- **`table_12345`** - Simple products table (4 columns, 3 data rows)
  - Headers: Product ID, Name, Price, Category
  - No colspan/rowspan
  - Good for basic export testing

- **`table_67890`** - Complex quarterly report (4 columns, 3 data rows)
  - Headers: Quarter, Sales, Profit, Notes
  - Has colspan/rowspan attributes
  - Good for testing complex table handling

- **`table_financial_001`** - Financial metrics table (4 columns, 3 data rows)
  - Headers: Metric, 2023, 2024, Change %
  - No colspan/rowspan
  - Good for currency and percentage data types

#### Session Fixtures

The following session IDs are available by default:

- **`test-session-123`** - Active session with cookies
  - URL: https://example.com
  - 3 cookies (session_token, user_preferences, analytics_id)
  - Status: Active
  - Good for basic session testing

- **`test-session-456`** - Idle session
  - URL: https://test.example.com/dashboard
  - 1 cookie (auth_token)
  - Status: Idle
  - Good for testing session lifecycle

- **`test-session-789`** - Multi-domain cookie session
  - URL: https://app.example.com/api/v1/data
  - 3 cookies across different domains
  - Status: Active
  - Good for testing cookie management across domains

### Cookie Lookup

```rust
use crate::fixtures::sessions;

// Get all cookies for a session and domain
let cookies = sessions::get_session_cookies_by_domain(
    "test-session-123",
    "example.com"
);

// Get a specific cookie
let cookie = sessions::get_session_cookie(
    "test-session-123",
    "example.com",
    "session_token"
);
```

### Export Functions

```rust
use crate::fixtures::tables;

let table = /* get table fixture */;

// Convert to CSV
let csv = tables::table_to_csv(&table);

// Convert to Markdown
let markdown = tables::table_to_markdown(&table);
```

### Global Test Data Store

For tests that need shared, thread-safe access to test data:

```rust
use crate::fixtures::test_data::GLOBAL_TEST_DATA;

#[tokio::test]
async fn test_with_global_store() {
    // Reset to defaults
    crate::fixtures::test_data::seed_test_data();

    // Get data
    let table = GLOBAL_TEST_DATA.get_table("table_12345").unwrap();

    // Add custom data
    GLOBAL_TEST_DATA.add_table(custom_table);

    // Clean up after test
    crate::fixtures::test_data::cleanup_test_data();
}
```

## Adding Custom Fixtures

### Add a Custom Table

```rust
use crate::fixtures::{FixtureManager, tables::TableFixture};

let mut manager = FixtureManager::new();

let custom_table = TableFixture {
    id: "custom_001".to_string(),
    source_url: Some("https://example.com/custom".to_string()),
    html_content: Some("<table>...</table>".to_string()),
    headers: vec!["Col1".to_string(), "Col2".to_string()],
    data: vec![vec!["A".to_string(), "B".to_string()]],
    rows: 2,
    columns: 2,
    has_spans: false,
    metadata: tables::TableMetadata {
        element_id: Some("custom".to_string()),
        classes: vec![],
        extracted_at: "2025-10-27T00:00:00Z".to_string(),
        data_types: vec!["string".to_string(), "string".to_string()],
    },
};

manager.add_table(custom_table);
```

### Add a Custom Session

```rust
use crate::fixtures::{FixtureManager, sessions::*};

let mut manager = FixtureManager::new();

let custom_session = SessionFixture {
    session_id: "custom-session-001".to_string(),
    url: "https://custom.example.com".to_string(),
    status: SessionStatus::Active,
    created_at: "2025-10-27T00:00:00Z".to_string(),
    expires_at: "2025-10-27T01:00:00Z".to_string(),
    last_activity: "2025-10-27T00:30:00Z".to_string(),
    cookies: vec![],
    metadata: SessionMetadata {
        browser: "chromium".to_string(),
        browser_version: "120.0.0".to_string(),
        user_agent: "Custom User Agent".to_string(),
        viewport: Some(Viewport { width: 1920, height: 1080 }),
        javascript_enabled: true,
        request_count: 0,
    },
};

manager.add_session(custom_session);
```

## Integration with API Tests

The fixtures are designed to work with the existing integration test infrastructure:

```rust
// In your integration test
mod test_utils {
    use super::*;
    use crate::fixtures::FixtureManager;

    pub async fn create_test_app_with_fixtures() -> (axum::Router, FixtureManager) {
        let app = create_test_app().await;
        let fixtures = FixtureManager::new();
        (app, fixtures)
    }
}

#[tokio::test]
async fn test_table_export_endpoint() {
    let (app, fixtures) = test_utils::create_test_app_with_fixtures().await;

    // Get a known table ID
    let table = fixtures.get_table("table_12345").unwrap();

    // Test the export endpoint
    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/tables/{}/export?format=csv", table.id))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

## Addressing Schemathesis 404 Warnings

These fixtures specifically address the Schemathesis warning:
> ⚠️ Missing valid test data: 3 operations repeatedly returned 404 responses

The affected operations were:

1. **`/api/v1/tables/{id}/export`** - Fixed with `table_12345`, `table_67890`, `table_financial_001`
2. **`/sessions/{session_id}`** - Fixed with `test-session-123`, `test-session-456`, `test-session-789`
3. **`/sessions/{session_id}/cookies/{domain}/{name}`** - Fixed with cookie fixtures in sessions

### Testing with Schemathesis

To verify fixtures work with Schemathesis:

```bash
# Start the API server with fixtures enabled
cargo run -p riptide-api

# Run Schemathesis tests
schemathesis run docs/02-api-reference/openapi.yaml \
  --base-url http://localhost:8080 \
  --checks all
```

## Dependencies

Add to `Cargo.toml` dev-dependencies:

```toml
[dev-dependencies]
lazy_static = "1.4"
```

## Best Practices

1. **Always use fixtures for deterministic tests** - Don't rely on external data
2. **Reset fixtures between tests** - Use `seed_test_data()` at test start
3. **Clean up after tests** - Use `cleanup_test_data()` in test teardown
4. **Use meaningful IDs** - Follow naming convention: `{type}_{identifier}`
5. **Document custom fixtures** - Add comments explaining special test cases

## Troubleshooting

### Fixtures not found

If fixtures are not being loaded:

1. Verify the module is included in test scope:
   ```rust
   #[cfg(test)]
   mod fixtures;
   ```

2. Check that lazy_static is in dev-dependencies

3. Ensure fixtures are loaded before use:
   ```rust
   seed_test_data();
   ```

### 404 errors persist

If 404 errors persist after adding fixtures:

1. Verify the OpenAPI spec uses the correct fixture IDs
2. Check that the API implementation actually uses the fixtures
3. Ensure test data is seeded before Schemathesis runs
4. Verify URL paths match exactly (including `/api/v1/` prefix)

## Future Enhancements

Potential improvements:

- [ ] Add profiling fixtures for profiling endpoints
- [ ] Add database seeding for Redis/PostgreSQL fixtures
- [ ] Add fixture factories for generating random test data
- [ ] Add fixture validation against OpenAPI schema
- [ ] Add fixture versioning for API compatibility testing
