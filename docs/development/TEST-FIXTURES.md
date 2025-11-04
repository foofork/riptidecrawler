# Test Fixtures Infrastructure

**Status:** Week 2-2.5 Foundation
**Purpose:** Deterministic testing with optional local fixtures
**CI Strategy:** Fast recorded responses (no Docker required)

## Overview

RipTide's test fixtures infrastructure provides:

1. **Fast CI tests** - Recorded HTTP responses via wiremock (no Docker)
2. **Optional local fixtures** - Docker Compose for manual testing
3. **Golden test files** - Static reference data for validation

## Quick Start

### For CI/Automated Testing (Default)

**No setup required!** Tests use recorded responses automatically:

```bash
# Run tests with recorded fixtures (fast)
cargo test --workspace

# Example: Test with mock servers
cargo test -p riptide-spider robots
```

All integration tests use wiremock mock servers by default - no Docker needed.

### For Local Development (Optional)

Want to test against real HTTP servers locally?

```bash
# Start local fixtures
cd test
make fixtures-up

# Run tests (optional - still uses recorded responses by default)
cargo test --workspace

# Stop fixtures
make fixtures-down
```

**Note:** As of Week 2-2.5, Docker Compose configuration is not yet implemented. CI and local tests both use recorded responses.

## Architecture

### 1. Recorded HTTP Fixtures (Primary)

Location: `/workspaces/eventmesh/tests/fixtures/recorded_responses.rs`

Fast, deterministic mock servers using wiremock:

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};

// Example: Mock robots.txt server
let server = mock_robots_server().await;
let client = reqwest::Client::new();

let response = client
    .get(format!("{}/robots.txt", server.uri()))
    .send()
    .await?;

assert_eq!(response.status(), 200);
```

**Available Mock Servers:**

- `mock_robots_server()` - robots.txt with disallow rules
- `mock_html_page_server()` - HTML pages with event content
- `mock_jsonld_server()` - JSON-LD schema.org Event markup
- `mock_flaky_server()` - Network errors for retry testing

### 2. Golden Test Files (Reference Data)

Location: `/workspaces/eventmesh/tests/fixtures/golden/`

Static reference files for validation:

- **sitemap.xml** - Sample sitemap for crawler tests
- **events.ics** - iCalendar events for ICS extraction tests
- **event_jsonld.html** - HTML with JSON-LD Event schema

**Usage:**

```rust
// Load golden file
let golden_ics = include_str!("../fixtures/golden/events.ics");

// Parse and validate
let events = parse_ics(golden_ics)?;
assert_eq!(events.len(), 3);
assert_eq!(events[0].summary, "Tech Conference 2025");
```

### 3. Local Docker Fixtures (Optional, Future)

Location: `/workspaces/eventmesh/test/Makefile`

**Status:** Not yet implemented (Week 2-2.5 foundation)

Future: Live HTTP servers via Docker Compose for manual testing.

## CI vs Local Testing Strategy

| Environment | Approach | Speed | Dependencies |
|------------|----------|-------|--------------|
| **CI (GitHub Actions)** | Recorded wiremock responses | ⚡ Fast (<10 min) | None (no Docker) |
| **Local Dev** | Recorded wiremock responses (same as CI) | ⚡ Fast | None |
| **Local Manual** | Optional Docker fixtures (future) | 🐢 Slower | Docker Compose |
| **Nightly E2E** | Optional live fixtures (future) | 🐢 Slower | Docker Compose |

**Key Principle:** CI uses fast mocks. Developers can optionally use live fixtures for debugging.

## How to Use Recorded Fixtures in Tests

### Example 1: Spider Robots.txt Test

```rust
use crate::fixtures::recorded_responses::mock_robots_server;

#[tokio::test]
async fn test_spider_respects_robots_txt() {
    // Fast: Uses recorded response, no Docker
    let server = mock_robots_server().await;

    let spider = Spider::new(SpiderOpts {
        respect_robots: true,
        ..Default::default()
    });

    // Should respect robots.txt and skip /admin
    let result = spider.crawl(&format!("{}/admin", server.uri())).await;
    assert!(result.urls.is_empty());
}
```

### Example 2: JSON-LD Extraction Test

```rust
use crate::fixtures::recorded_responses::mock_jsonld_server;

#[tokio::test]
async fn test_extract_event_from_jsonld() {
    let server = mock_jsonld_server().await;

    let extractor = Extractor::new();
    let result = extractor
        .extract_url(&format!("{}/event-jsonld.html", server.uri()))
        .await?;

    assert_eq!(result.events.len(), 1);
    assert_eq!(result.events[0].title, "AI Workshop 2025");
}
```

### Example 3: Golden File Test

```rust
#[test]
fn test_ics_parsing_with_golden_file() {
    // Recorded iCalendar data
    let golden_ics = include_str!("../fixtures/golden/events.ics");

    let events = parse_ics(golden_ics).expect("Failed to parse ICS");

    // Verify extraction works without live data
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].summary, "Tech Conference 2025");
    assert_eq!(events[1].summary, "Developer Meetup");
    assert_eq!(events[2].summary, "AI Workshop 2025");
}
```

## Why This Approach?

### ✅ Benefits

1. **Fast CI** - No Docker overhead, tests run in <10 minutes
2. **Deterministic** - Same recorded responses every time
3. **No dependencies** - Works in any environment
4. **Easy debugging** - Can inspect exact mock responses
5. **Flexible** - Optional live fixtures for manual testing

### ⚠️ Trade-offs

1. **Manual updates** - Need to update recorded responses if API changes
2. **Coverage gaps** - May miss edge cases not in recorded data
3. **Live debugging** - Must use optional Docker fixtures for real HTTP behavior

## Future Enhancements (v1.1+)

- [ ] Docker Compose configuration for live local fixtures
- [ ] Nightly E2E tests with live riptidecrawler-test-sites
- [ ] Automatic recording tool to capture live responses
- [ ] More mock servers (rate limiting, timeouts, redirects)
- [ ] Performance benchmarks with fixtures

## Related Documentation

- [TDD London School Guide](./TDD-LONDON-SCHOOL.md) - Testing methodology
- [Week 2-2.5 Roadmap](../roadmap/RIPTIDE-V1-DEFINITIVE-ROADMAP.md#w2-25-tdd-guide--test-fixtures-2-days) - Roadmap context

## Acceptance Criteria

Week 2-2.5 deliverables:

- [x] `/workspaces/eventmesh/test/Makefile` created
- [x] Recorded HTTP fixtures implemented (wiremock)
- [x] Golden test files created (sitemap.xml, events.ics, event_jsonld.html)
- [x] Documentation complete
- [x] `.gitignore` updated to exclude fixture artifacts
- [ ] Integration tests using fixtures (to be implemented in Week 14-16)

**Status:** ✅ Infrastructure complete, ready for testing integration
