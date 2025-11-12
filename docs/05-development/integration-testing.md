# Integration Testing Guide

Complete guide for running Riptide's integration tests against real infrastructure and live websites.

## Overview

Riptide has **42 ignored tests** that require external infrastructure:

- **10 Browser Tests** - Require Chrome/Chromium
- **15 Redis Tests** - Require Redis server
- **8 PostgreSQL Tests** - Require PostgreSQL database
- **9 Other Tests** - PDF libraries, network access, etc.

These tests are ignored by default to allow CI/CD to run quickly without external dependencies.

## Quick Start

```bash
# Run all integration tests (auto-detects available infrastructure)
./scripts/run_integration_tests.sh

# Run only browser tests
./scripts/run_integration_tests.sh --browser-only

# Run only Redis tests
./scripts/run_integration_tests.sh --redis-only

# Run only PostgreSQL tests
./scripts/run_integration_tests.sh --postgres-only
```

## Infrastructure Setup

### 1. Chrome/Chromium (Required for Browser Tests)

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install chromium-browser
```

**macOS:**
```bash
brew install --cask google-chrome
```

**Verify Installation:**
```bash
google-chrome --version
# or
chromium --version
```

### 2. Redis (Required for Cache/Session Tests)

**Ubuntu/Debian:**
```bash
sudo apt install redis-server
sudo systemctl start redis-server
sudo systemctl enable redis-server
```

**macOS:**
```bash
brew install redis
brew services start redis
```

**Verify Installation:**
```bash
redis-cli ping
# Should output: PONG
```

**Configuration (Optional):**
```bash
# Edit /etc/redis/redis.conf or /usr/local/etc/redis.conf
# Set maxmemory policy for testing:
maxmemory 256mb
maxmemory-policy allkeys-lru
```

### 3. PostgreSQL (Required for Persistence Tests)

**Ubuntu/Debian:**
```bash
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

**macOS:**
```bash
brew install postgresql
brew services start postgresql
```

**Setup Test Database:**
```bash
# Create test user and database
sudo -u postgres psql << EOF
CREATE USER riptide_test WITH PASSWORD 'test_password';
CREATE DATABASE riptide_test OWNER riptide_test;
GRANT ALL PRIVILEGES ON DATABASE riptide_test TO riptide_test;
EOF
```

**Environment Variables:**
```bash
export DATABASE_URL="postgresql://riptide_test:test_password@localhost/riptide_test"
```

### 4. PDF Library (Optional - for PDF extraction tests)

**Ubuntu/Debian:**
```bash
# Install PDFium library
wget https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-linux-x64.tgz
sudo tar -xzf pdfium-linux-x64.tgz -C /usr/local/lib/
sudo ldconfig
```

**macOS:**
```bash
# Install via Homebrew (if available)
brew install pdfium
```

## Running Tests

### Browser Integration Tests

Tests browser launching, navigation, screenshots, and JavaScript execution.

```bash
# Run all browser tests
cargo test -p riptide-facade -- --ignored --nocapture

# Run specific browser test
cargo test -p riptide-facade test_browser_launch_and_close -- --ignored --nocapture

# Test against specific URL
CHROME_PATH=/usr/bin/google-chrome cargo test -p riptide-headless -- --ignored
```

**Browser Tests Include:**
- `test_browser_launch_and_close` - Basic browser lifecycle
- `test_browser_navigation` - Page navigation
- `test_browser_screenshot` - Screenshot capture
- `test_browser_content` - HTML content extraction
- `test_browser_multi_session` - Connection pool management

### Redis Integration Tests

Tests connection pooling, caching, session storage, and idempotency.

```bash
# Run all Redis tests
cargo test -p riptide-cache -- --ignored --nocapture
cargo test -p riptide-persistence redis -- --ignored --nocapture

# Test specific Redis feature
cargo test -p riptide-cache test_connection_pool -- --ignored --nocapture
```

**Redis Tests Include:**
- Connection pool creation and lifecycle
- Cache set/get/delete operations
- Session storage and retrieval
- Idempotency key management
- TTL and expiration handling

### PostgreSQL Integration Tests

Tests persistent storage, session management, and outbox pattern.

```bash
# Set database URL
export DATABASE_URL="postgresql://riptide_test:test_password@localhost/riptide_test"

# Run all PostgreSQL tests
cargo test -p riptide-persistence postgres -- --ignored --nocapture

# Test outbox publisher
cargo test -p riptide-persistence test_outbox_publisher -- --ignored --nocapture
```

**PostgreSQL Tests Include:**
- Session storage and retrieval
- Outbox pattern publisher
- Transaction handling
- Connection pooling

### Live Website Crawl Tests

Tests actual crawling against real websites.

```bash
# Test against safe, stable websites
export TEST_URLS="https://example.com,https://httpbin.org,https://quotes.toscrape.com"

# Run live crawl tests
cargo test test_live_crawl -- --ignored --nocapture
```

**Test Websites:**
1. **example.com** - Simple HTML, stable
2. **httpbin.org** - HTTP testing service
3. **quotes.toscrape.com** - Scraping practice site
4. **books.toscrape.com** - E-commerce practice site

## Manual Testing Strategy

### Phase 1: Unit Tests (No Infrastructure)
```bash
# Run all normal tests (no infrastructure required)
cargo test --workspace
```

### Phase 2: Browser Tests
```bash
# Test browser launching
./scripts/run_integration_tests.sh --browser-only

# Expected results:
# - Browser launches successfully
# - Navigation works
# - Screenshots captured
# - Content extracted
# - Multiple sessions managed
```

### Phase 3: Cache/Database Tests
```bash
# Test Redis integration
./scripts/run_integration_tests.sh --redis-only

# Test PostgreSQL integration
./scripts/run_integration_tests.sh --postgres-only
```

### Phase 4: Full Integration
```bash
# Run everything together
./scripts/run_integration_tests.sh

# Expected results:
# - All infrastructure detected
# - All tests pass
# - No resource leaks
# - Clean shutdown
```

## Test Site Selection

For complete testing against real websites, use these categories:

### Static Sites (Simple)
- `https://example.com` - Basic HTML
- `https://httpbin.org/html` - HTTP test service

### Dynamic Sites (JavaScript Heavy)
- `https://quotes.toscrape.com` - JS-rendered quotes
- `https://books.toscrape.com` - E-commerce with pagination

### Complex Sites (Production-Like)
- `https://news.ycombinator.com` - Real content, simple structure
- `https://lobste.rs` - Similar to HN, good for testing

### API Testing
- `https://httpbin.org/json` - JSON responses
- `https://jsonplaceholder.typicode.com` - REST API mock

## Creating New Integration Tests

### Example: Browser Test Against Real Site

```rust
#[tokio::test]
#[ignore] // Requires Chrome and network
async fn test_crawl_real_website() {
    // Setup
    let config = RiptideConfig::default();
    let facade = BrowserFacade::new(config).await.unwrap();

    // Launch browser
    let session = facade.launch().await.unwrap();

    // Navigate to real site
    facade.navigate(&session, "https://example.com").await.unwrap();

    // Extract content
    let content = facade.get_content(&session).await.unwrap();

    // Assertions
    assert!(content.contains("Example Domain"));
    assert!(!content.is_empty());

    // Cleanup
    facade.close(session).await.unwrap();
}
```

### Example: Multi-Site Crawl Test

```rust
#[tokio::test]
#[ignore] // Requires Chrome and network
async fn test_multi_site_crawl() {
    let sites = vec![
        "https://example.com",
        "https://httpbin.org/html",
        "https://quotes.toscrape.com",
    ];

    let facade = CrawlFacade::new().await.unwrap();

    for url in sites {
        let result = facade.crawl_single(url).await.unwrap();
        assert!(result.success);
        assert!(!result.content.is_empty());
    }
}
```

## Troubleshooting

### Chrome Not Found
```bash
# Set explicit Chrome path
export CHROME_PATH=/usr/bin/google-chrome
cargo test -- --ignored
```

### Redis Connection Failed
```bash
# Check Redis is running
redis-cli ping

# Check Redis port
netstat -an | grep 6379

# Restart Redis
sudo systemctl restart redis-server
```

### PostgreSQL Connection Failed
```bash
# Check PostgreSQL is running
pg_isready

# Check connection
psql -U riptide_test -d riptide_test -h localhost

# Reset test database
sudo -u postgres psql -c "DROP DATABASE IF EXISTS riptide_test;"
sudo -u postgres psql -c "CREATE DATABASE riptide_test;"
```

### Out of Disk Space
```bash
# Check space
df -h

# Clean build artifacts
cargo clean

# Free up space (need at least 5GB for Chrome tests)
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Integration Tests

on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly
  workflow_dispatch:     # Manual trigger

jobs:
  integration:
    runs-on: ubuntu-latest

    services:
      redis:
        image: redis:7
        ports:
          - 6379:6379
      postgres:
        image: postgres:15
        env:
          POSTGRES_USER: riptide_test
          POSTGRES_PASSWORD: test_password
          POSTGRES_DB: riptide_test
        ports:
          - 5432:5432

    steps:
      - uses: actions/checkout@v3

      - name: Install Chrome
        run: |
          sudo apt update
          sudo apt install -y chromium-browser

      - name: Run Integration Tests
        run: ./scripts/run_integration_tests.sh
        env:
          DATABASE_URL: postgresql://riptide_test:test_password@localhost/riptide_test
```

## Performance Benchmarks

Expected test execution times:

- **Browser Tests**: ~30 seconds (5 tests)
- **Redis Tests**: ~10 seconds (15 tests)
- **PostgreSQL Tests**: ~20 seconds (8 tests)
- **Full Suite**: ~60 seconds (all 42 tests)

## Best Practices

1. **Isolation**: Each test should be independent
2. **Cleanup**: Always close browsers/connections
3. **Timeouts**: Set reasonable timeouts (30s for network)
4. **Retries**: Network tests should retry once
5. **Logging**: Use `--nocapture` for debugging
6. **Parallelism**: Run tests in parallel when possible

## Next Steps

1. Run `./scripts/run_integration_tests.sh` to verify setup
2. Add new integration tests for your use cases
3. Set up CI/CD for weekly integration runs
4. Monitor test performance and adjust timeouts
5. Update test sites if they become unavailable

---

**Questions?** Check logs in `/tmp/` after test runs for detailed output.
