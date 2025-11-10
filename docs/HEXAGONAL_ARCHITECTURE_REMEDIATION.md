# Hexagonal Architecture Remediation Guide

**Project**: Riptide Web Scraping Framework
**Date**: 2025-11-10
**Priority**: HIGH
**Estimated Effort**: 4-5 weeks

---

## Executive Summary

This document provides **step-by-step technical guidance** for fixing hexagonal architecture violations in the Riptide codebase. The primary issue is that **riptide-facade (Application Layer) directly depends on 13 infrastructure crates**, violating the Dependency Inversion Principle.

**Goal**: Achieve 95%+ hexagonal architecture compliance by:
1. Removing all infrastructure dependencies from riptide-facade
2. Using only port traits (Arc<dyn Trait>) in application layer
3. Wiring all concrete adapters in composition root (riptide-api)

---

## Phase 1: Critical Path - HttpClient Port (Week 1)

### Step 1.1: Define HttpClient Port Trait

**File**: `/workspaces/eventmesh/crates/riptide-types/src/ports/http.rs`

**Current State**: Port trait exists but may not be complete.

**Action**: Verify and enhance the trait:

```rust
use async_trait::async_trait;
use bytes::Bytes;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Bytes>,
    pub timeout: Option<Duration>,
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Bytes,
}

#[async_trait]
pub trait HttpClient: Send + Sync {
    /// Send an HTTP request and return response
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse, RiptideError>;

    /// GET request convenience method
    async fn get(&self, url: &str) -> Result<HttpResponse, RiptideError> {
        self.send(HttpRequest {
            url: url.to_string(),
            method: "GET".to_string(),
            headers: vec![],
            body: None,
            timeout: None,
        })
        .await
    }

    /// POST request convenience method
    async fn post(
        &self,
        url: &str,
        body: Bytes,
    ) -> Result<HttpResponse, RiptideError> {
        self.send(HttpRequest {
            url: url.to_string(),
            method: "POST".to_string(),
            headers: vec![],
            body: Some(body),
            timeout: None,
        })
        .await
    }
}
```

**Verification**:
```bash
cargo check -p riptide-types
cargo test -p riptide-types -- http
```

---

### Step 1.2: Implement HttpClient Adapter in riptide-fetch

**File**: `/workspaces/eventmesh/crates/riptide-fetch/src/adapters/reqwest_client.rs`

**Create new file**:

```rust
//! Reqwest adapter implementing HttpClient port

use async_trait::async_trait;
use riptide_types::ports::{HttpClient, HttpRequest, HttpResponse};
use riptide_types::RiptideError;
use std::sync::Arc;

/// Reqwest-based HTTP client adapter
pub struct ReqwestAdapter {
    client: reqwest::Client,
}

impl ReqwestAdapter {
    /// Create new adapter with default reqwest client
    pub fn new() -> Result<Self, RiptideError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| RiptideError::Network(e.to_string()))?;

        Ok(Self { client })
    }

    /// Create adapter with custom reqwest client
    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl HttpClient for ReqwestAdapter {
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse, RiptideError> {
        // Build reqwest request
        let mut req = self.client.request(
            reqwest::Method::from_bytes(request.method.as_bytes())
                .map_err(|e| RiptideError::InvalidInput(e.to_string()))?,
            &request.url,
        );

        // Add headers
        for (key, value) in request.headers {
            req = req.header(key, value);
        }

        // Add body if present
        if let Some(body) = request.body {
            req = req.body(body);
        }

        // Add timeout if specified
        if let Some(timeout) = request.timeout {
            req = req.timeout(timeout);
        }

        // Execute request
        let response = req
            .send()
            .await
            .map_err(|e| RiptideError::Network(e.to_string()))?;

        // Extract response data
        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| {
                (
                    k.as_str().to_string(),
                    v.to_str().unwrap_or_default().to_string(),
                )
            })
            .collect();

        let body = response
            .bytes()
            .await
            .map_err(|e| RiptideError::Network(e.to_string()))?;

        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_reqwest_adapter_get() {
        let adapter = ReqwestAdapter::new().unwrap();
        let response = adapter.get("https://httpbin.org/get").await;
        assert!(response.is_ok());
    }
}
```

**Update**: `/workspaces/eventmesh/crates/riptide-fetch/src/lib.rs`

```rust
pub mod adapters;

// Re-export adapter
pub use adapters::reqwest_client::ReqwestAdapter;
```

**Update**: `/workspaces/eventmesh/crates/riptide-fetch/Cargo.toml`

```toml
[dependencies]
riptide-types = { path = "../riptide-types" }
reqwest = { workspace = true }
async-trait = { workspace = true }
bytes = { workspace = true }
```

**Verification**:
```bash
cargo test -p riptide-fetch
cargo clippy -p riptide-fetch -- -D warnings
```

---

### Step 1.3: Refactor ExtractionFacade to Use HttpClient Port

**File**: `/workspaces/eventmesh/crates/riptide-facade/src/facades/extraction.rs`

**Before**:
```rust
use std::sync::Arc;

pub struct ExtractionFacade {
    http_client: Arc<reqwest::Client>,  // ❌ Concrete type
}

impl ExtractionFacade {
    pub fn new(http_client: Arc<reqwest::Client>) -> Self {
        Self { http_client }
    }
}
```

**After**:
```rust
use riptide_types::ports::HttpClient;
use std::sync::Arc;

pub struct ExtractionFacade {
    http_client: Arc<dyn HttpClient>,  // ✅ Port trait
}

impl ExtractionFacade {
    pub fn new(http_client: Arc<dyn HttpClient>) -> Self {
        Self { http_client }
    }

    pub async fn fetch_html(&self, url: &str) -> Result<String, RiptideError> {
        let response = self.http_client.get(url).await?;

        if response.status >= 400 {
            return Err(RiptideError::HttpError(response.status));
        }

        let html = String::from_utf8(response.body.to_vec())
            .map_err(|e| RiptideError::InvalidEncoding(e.to_string()))?;

        Ok(html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riptide_types::ports::{HttpClient, HttpRequest, HttpResponse};
    use async_trait::async_trait;
    use bytes::Bytes;

    struct MockHttpClient;

    #[async_trait]
    impl HttpClient for MockHttpClient {
        async fn send(&self, _request: HttpRequest) -> Result<HttpResponse, RiptideError> {
            Ok(HttpResponse {
                status: 200,
                headers: vec![],
                body: Bytes::from("<html>Test</html>"),
            })
        }
    }

    #[tokio::test]
    async fn test_extraction_with_mock() {
        let facade = ExtractionFacade::new(Arc::new(MockHttpClient));
        let html = facade.fetch_html("https://example.com").await.unwrap();
        assert_eq!(html, "<html>Test</html>");
    }
}
```

**Update**: `/workspaces/eventmesh/crates/riptide-facade/Cargo.toml`

```toml
[dependencies]
# ✅ ONLY domain dependencies
riptide-types = { path = "../riptide-types" }

# ❌ REMOVE:
# reqwest = { workspace = true }
```

**Verification**:
```bash
cargo test -p riptide-facade -- extraction
cargo clippy -p riptide-facade -- -D warnings
```

---

### Step 1.4: Wire Adapter in Composition Root

**File**: `/workspaces/eventmesh/crates/riptide-api/src/composition/mod.rs`

**Update ApplicationContext**:

```rust
use riptide_fetch::ReqwestAdapter;
use riptide_types::ports::HttpClient;

pub struct ApplicationContext {
    // Add HTTP client port
    pub http_client: Arc<dyn HttpClient>,

    // ... existing fields
}

impl ApplicationContext {
    pub async fn new(config: &DiConfig) -> Result<Self> {
        // ✅ Wire concrete adapter to port
        let http_client: Arc<dyn HttpClient> = Arc::new(
            ReqwestAdapter::new()
                .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?
        );

        // Create facade with port
        let extraction_facade = ExtractionFacade::new(http_client.clone());

        Ok(Self {
            http_client,
            extraction_facade,
            // ... rest
        })
    }

    pub fn for_testing() -> Self {
        // ✅ Wire mock adapter for tests
        let http_client: Arc<dyn HttpClient> = Arc::new(MockHttpClient::new());

        let extraction_facade = ExtractionFacade::new(http_client.clone());

        Self {
            http_client,
            extraction_facade,
            // ... rest
        }
    }
}
```

**Update**: `/workspaces/eventmesh/crates/riptide-api/Cargo.toml`

```toml
[dependencies]
riptide-fetch = { path = "../riptide-fetch" }
```

**Verification**:
```bash
cargo test -p riptide-api
cargo clippy -p riptide-api -- -D warnings
cargo check --workspace
```

---

## Phase 2: BrowserDriver Port (Week 2)

### Step 2.1: Verify BrowserDriver Port Trait

**File**: `/workspaces/eventmesh/crates/riptide-types/src/ports/features.rs`

**Verify trait exists and is complete**:

```rust
#[async_trait]
pub trait BrowserDriver: Send + Sync {
    /// Create a new browser session
    async fn create_session(&self) -> Result<Box<dyn BrowserSession>, RiptideError>;

    /// Get browser health status
    async fn health_check(&self) -> Result<(), RiptideError>;

    /// Close all browser instances
    async fn shutdown(&self) -> Result<(), RiptideError>;
}

#[async_trait]
pub trait BrowserSession: Send + Sync {
    /// Navigate to URL
    async fn navigate(&mut self, url: &str) -> Result<(), RiptideError>;

    /// Get page HTML content
    async fn get_content(&self) -> Result<String, RiptideError>;

    /// Execute JavaScript
    async fn execute_script(&self, script: &str) -> Result<ScriptResult, RiptideError>;

    /// Take screenshot
    async fn screenshot(&self, options: ScreenshotOptions) -> Result<Vec<u8>, RiptideError>;

    /// Get current URL
    fn current_url(&self) -> Option<String>;

    /// Close session
    async fn close(self: Box<Self>) -> Result<(), RiptideError>;
}
```

---

### Step 2.2: Implement BrowserDriver Adapter

**File**: `/workspaces/eventmesh/crates/riptide-browser/src/port_adapter.rs`

**Create new file**:

```rust
//! Browser port adapter implementing BrowserDriver trait

use async_trait::async_trait;
use riptide_types::ports::{BrowserDriver, BrowserSession, ScriptResult, ScreenshotOptions};
use riptide_types::RiptideError;
use std::sync::Arc;

use crate::pool::BrowserPool;  // Your existing pool

/// Chrome-based browser adapter
pub struct ChromeBrowserAdapter {
    pool: Arc<BrowserPool>,
}

impl ChromeBrowserAdapter {
    pub async fn new(config: BrowserPoolConfig) -> Result<Self, RiptideError> {
        let pool = BrowserPool::new(config)
            .await
            .map_err(|e| RiptideError::BrowserError(e.to_string()))?;

        Ok(Self {
            pool: Arc::new(pool),
        })
    }
}

#[async_trait]
impl BrowserDriver for ChromeBrowserAdapter {
    async fn create_session(&self) -> Result<Box<dyn BrowserSession>, RiptideError> {
        let checkout = self.pool.checkout()
            .await
            .map_err(|e| RiptideError::BrowserError(e.to_string()))?;

        Ok(Box::new(ChromeSession {
            checkout,
        }))
    }

    async fn health_check(&self) -> Result<(), RiptideError> {
        self.pool.health_check()
            .await
            .map_err(|e| RiptideError::BrowserError(e.to_string()))
    }

    async fn shutdown(&self) -> Result<(), RiptideError> {
        self.pool.shutdown()
            .await
            .map_err(|e| RiptideError::BrowserError(e.to_string()))
    }
}

struct ChromeSession {
    checkout: BrowserCheckout,  // Your existing checkout type
}

#[async_trait]
impl BrowserSession for ChromeSession {
    async fn navigate(&mut self, url: &str) -> Result<(), RiptideError> {
        self.checkout.navigate(url)
            .await
            .map_err(|e| RiptideError::BrowserError(e.to_string()))
    }

    async fn get_content(&self) -> Result<String, RiptideError> {
        self.checkout.get_html()
            .await
            .map_err(|e| RiptideError::BrowserError(e.to_string()))
    }

    async fn execute_script(&self, script: &str) -> Result<ScriptResult, RiptideError> {
        let result = self.checkout.execute_script(script)
            .await
            .map_err(|e| RiptideError::BrowserError(e.to_string()))?;

        Ok(ScriptResult {
            value: result,
        })
    }

    async fn screenshot(&self, options: ScreenshotOptions) -> Result<Vec<u8>, RiptideError> {
        self.checkout.take_screenshot(options)
            .await
            .map_err(|e| RiptideError::BrowserError(e.to_string()))
    }

    fn current_url(&self) -> Option<String> {
        self.checkout.current_url()
    }

    async fn close(self: Box<Self>) -> Result<(), RiptideError> {
        // Checkout automatically returns to pool on drop
        Ok(())
    }
}
```

**Update**: `/workspaces/eventmesh/crates/riptide-browser/src/lib.rs`

```rust
pub mod port_adapter;

pub use port_adapter::ChromeBrowserAdapter;
```

**Verification**:
```bash
cargo test -p riptide-browser
cargo clippy -p riptide-browser -- -D warnings
```

---

### Step 2.3: Refactor BrowserFacade

**File**: `/workspaces/eventmesh/crates/riptide-facade/src/facades/browser.rs`

**Remove direct chromiumoxide_cdp usage**:

```rust
// REMOVE these imports:
// use chromiumoxide_cdp::cdp::browser_protocol::page::{...};

// REPLACE with:
use riptide_types::ports::{BrowserDriver, BrowserSession};

pub struct BrowserFacade {
    driver: Arc<dyn BrowserDriver>,  // ✅ Port trait
}

impl BrowserFacade {
    pub fn new(driver: Arc<dyn BrowserDriver>) -> Self {
        Self { driver }
    }

    pub async fn scrape_page(&self, url: &str) -> Result<String, RiptideError> {
        let mut session = self.driver.create_session().await?;
        session.navigate(url).await?;
        let content = session.get_content().await?;
        session.close().await?;
        Ok(content)
    }
}
```

**Update Cargo.toml**:
```toml
[dependencies]
riptide-types = { path = "../riptide-types" }

# ❌ REMOVE:
# spider_chromiumoxide_cdp = { workspace = true }
```

---

## Phase 3: Remove All Infrastructure Dependencies from Facade (Week 3)

### Step 3.1: Audit Remaining Dependencies

**Run**:
```bash
cd /workspaces/eventmesh/crates/riptide-facade
grep -E "riptide-(cache|browser|fetch|spider|extraction|pdf|search|stealth|monitoring|reliability|intelligence|headless|workers)" Cargo.toml
```

### Step 3.2: Create Port Adapters for Each

For each infrastructure dependency found:

1. **riptide-cache** → Already has `CacheStorage` port ✅
2. **riptide-extraction** → Create `Extractor` port adapter
3. **riptide-spider** → Create `Spider` port adapter
4. **riptide-pdf** → Already has `PdfProcessor` port ✅
5. **riptide-search** → Already has `SearchEngine` port ✅

**Example for riptide-extraction**:

```rust
// crates/riptide-extraction/src/port_adapter.rs

use riptide_types::ports::Extractor;

pub struct NativeExtractorAdapter {
    // Your existing extractor
}

impl Extractor for NativeExtractorAdapter {
    // Implement trait
}
```

### Step 3.3: Update Facade Cargo.toml

**Target state**:

```toml
[package]
name = "riptide-facade"
version = "0.9.0"

[dependencies]
# ✅ Domain layer only
riptide-types = { path = "../riptide-types" }

# ✅ Shared utilities (acceptable)
riptide-utils = { path = "../riptide-utils" }
riptide-config = { path = "../riptide-config" }
riptide-monitoring = { path = "../riptide-monitoring" }  # Observability

# ❌ REMOVE ALL THESE:
# riptide-cache = { path = "../riptide-cache" }
# riptide-browser = { path = "../riptide-browser" }
# riptide-fetch = { path = "../riptide-fetch" }
# riptide-spider = { path = "../riptide-spider" }
# riptide-extraction = { path = "../riptide-extraction" }
# riptide-pdf = { path = "../riptide-pdf" }
# riptide-search = { path = "../riptide-search" }
# riptide-stealth = { path = "../riptide-stealth" }
# riptide-headless = { path = "../riptide-headless" }
# riptide-reliability = { path = "../riptide-reliability" }
# riptide-intelligence = { path = "../riptide-intelligence" }
# riptide-workers = { path = "../riptide-workers" }

[dev-dependencies]
# ✅ Test utilities
riptide-test-fixtures = { path = "../riptide-test-fixtures" }

# ❌ REMOVE:
# riptide-api = { path = "../riptide-api" }
```

---

## Phase 4: Break Circular Dependency (Week 4)

### Step 4.1: Create riptide-test-fixtures Crate

```bash
cd /workspaces/eventmesh/crates
cargo new --lib riptide-test-fixtures
```

**File**: `/workspaces/eventmesh/crates/riptide-test-fixtures/Cargo.toml`

```toml
[package]
name = "riptide-test-fixtures"
version = "0.9.0"
edition = "2021"

[dependencies]
riptide-types = { path = "../riptide-types" }
async-trait = { workspace = true }
tokio = { workspace = true, features = ["sync"] }
```

**File**: `/workspaces/eventmesh/crates/riptide-test-fixtures/src/lib.rs`

```rust
//! Test fixtures and mocks for Riptide testing

use async_trait::async_trait;
use riptide_types::ports::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock HTTP client for testing
pub struct MockHttpClient {
    responses: Arc<Mutex<HashMap<String, HttpResponse>>>,
}

impl MockHttpClient {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_response(&self, url: &str, response: HttpResponse) {
        self.responses.lock().unwrap().insert(url.to_string(), response);
    }
}

#[async_trait]
impl HttpClient for MockHttpClient {
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse, RiptideError> {
        self.responses
            .lock()
            .unwrap()
            .get(&request.url)
            .cloned()
            .ok_or_else(|| RiptideError::NotFound)
    }
}

/// Mock browser driver for testing
pub struct MockBrowserDriver;

#[async_trait]
impl BrowserDriver for MockBrowserDriver {
    async fn create_session(&self) -> Result<Box<dyn BrowserSession>, RiptideError> {
        Ok(Box::new(MockBrowserSession))
    }

    async fn health_check(&self) -> Result<(), RiptideError> {
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), RiptideError> {
        Ok(())
    }
}

struct MockBrowserSession;

#[async_trait]
impl BrowserSession for MockBrowserSession {
    async fn navigate(&mut self, _url: &str) -> Result<(), RiptideError> {
        Ok(())
    }

    async fn get_content(&self) -> Result<String, RiptideError> {
        Ok("<html>Mock</html>".to_string())
    }

    // ... implement other methods
}

// Add more mocks for other ports
```

### Step 4.2: Update Workspace Members

**File**: `/workspaces/eventmesh/Cargo.toml`

```toml
[workspace]
members = [
    "crates/riptide-test-fixtures",  # Add this
    # ... rest
]
```

### Step 4.3: Update Test Dependencies

**In riptide-facade**:
```toml
[dev-dependencies]
riptide-test-fixtures = { path = "../riptide-test-fixtures" }
# Remove: riptide-api
```

**In riptide-api**:
```toml
[dev-dependencies]
riptide-test-fixtures = { path = "../riptide-test-fixtures" }
```

---

## Phase 5: Validation & Documentation (Week 5)

### Step 5.1: Architectural Fitness Functions

**Create**: `/workspaces/eventmesh/scripts/check-architecture.sh`

```bash
#!/bin/bash
set -e

echo "🔍 Checking Hexagonal Architecture Compliance..."

# Check that riptide-facade only depends on riptide-types
echo "✅ Checking facade dependencies..."
FACADE_DEPS=$(cargo metadata --format-version 1 | jq -r '
  .packages[] |
  select(.name == "riptide-facade") |
  .dependencies[] |
  select(.name | startswith("riptide-")) |
  select(.name != "riptide-types" and .name != "riptide-utils" and .name != "riptide-config" and .name != "riptide-monitoring") |
  .name
')

if [ -n "$FACADE_DEPS" ]; then
  echo "❌ VIOLATION: riptide-facade has forbidden dependencies:"
  echo "$FACADE_DEPS"
  exit 1
fi

echo "✅ Facade layer is pure - no infrastructure dependencies"

# Check for circular dependencies
echo "✅ Checking for circular dependencies..."
cargo tree -p riptide-facade -e normal | grep "riptide-api" && {
  echo "❌ VIOLATION: Circular dependency detected (facade → api)"
  exit 1
}

echo "✅ No circular dependencies found"

echo ""
echo "🎉 All hexagonal architecture checks passed!"
```

**Make executable**:
```bash
chmod +x /workspaces/eventmesh/scripts/check-architecture.sh
```

**Add to CI** (`.github/workflows/ci.yml`):
```yaml
- name: Architecture Compliance
  run: ./scripts/check-architecture.sh
```

---

### Step 5.2: Update Documentation

**Update**: `/workspaces/eventmesh/crates/riptide-facade/README.md`

```markdown
# riptide-facade - Application Layer

**Architectural Role**: Pure application/use-case orchestration

## Dependency Rules

This crate follows hexagonal architecture principles:

✅ **ALLOWED Dependencies**:
- `riptide-types` - Domain models and port traits
- `riptide-utils` - Shared utilities
- `riptide-config` - Configuration types
- `riptide-monitoring` - Observability

❌ **FORBIDDEN Dependencies**:
- NO infrastructure crates (riptide-cache, riptide-browser, etc.)
- NO database drivers (sqlx, postgres)
- NO HTTP clients (reqwest, hyper)
- NO browser automation libraries (chromiumoxide_cdp)

## Design Principle

All infrastructure is accessed via **port traits** defined in `riptide-types`:

```rust
use riptide_types::ports::{HttpClient, CacheStorage, BrowserDriver};

pub struct ExtractionFacade {
    http_client: Arc<dyn HttpClient>,      // ✅ Port trait
    cache: Arc<dyn CacheStorage>,          // ✅ Port trait
    browser: Arc<dyn BrowserDriver>,       // ✅ Port trait
}
```

Concrete implementations are wired in `riptide-api` composition root.
```

---

### Step 5.3: Create ADR

**File**: `/workspaces/eventmesh/docs/adr/ADR-001-hexagonal-architecture.md`

```markdown
# ADR-001: Hexagonal Architecture (Ports & Adapters)

**Status**: Accepted
**Date**: 2025-11-10
**Deciders**: Architecture Team

## Context

We need a maintainable, testable architecture that allows:
- Swapping infrastructure implementations
- Fast unit tests without external dependencies
- Clear separation of concerns
- Parallel development of layers

## Decision

We adopt **Hexagonal Architecture** (Ports & Adapters pattern):

1. **Domain Layer (riptide-types)**: Pure business logic, port traits
2. **Application Layer (riptide-facade)**: Use-case orchestration via ports
3. **Infrastructure Layer (riptide-*)**: Concrete adapter implementations
4. **Composition Root (riptide-api)**: Dependency injection

### Dependency Rules

- Application layer depends ONLY on domain layer
- Infrastructure implements domain ports
- Composition root wires everything together

## Consequences

**Positive**:
- Testable without Docker/external services
- Infrastructure can be swapped without changing business logic
- Clear architectural boundaries

**Negative**:
- More upfront design (defining port traits)
- Additional indirection via trait objects
- Small runtime overhead for dynamic dispatch

## Compliance

Enforced via:
- CI checks (`scripts/check-architecture.sh`)
- Code review checklist
- Cargo.toml audits
```

---

## Testing Strategy

### Unit Tests (Fast, No External Dependencies)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use riptide_test_fixtures::*;

    #[tokio::test]
    async fn test_extraction_facade() {
        // ✅ Uses mock adapters - no Docker, no Redis, no Chrome
        let http_client = Arc::new(MockHttpClient::new());
        let cache = Arc::new(InMemoryCache::new());
        let browser = Arc::new(MockBrowserDriver);

        let facade = ExtractionFacade::new(http_client, cache, browser);

        let result = facade.extract("https://example.com").await;
        assert!(result.is_ok());
    }
}
```

**Benefits**:
- Run in milliseconds
- No flaky network calls
- Deterministic results
- Can run in parallel

### Integration Tests (With Real Infrastructure)

```rust
#[tokio::test]
#[ignore]  // Run with --ignored flag
async fn test_extraction_with_real_browser() {
    let config = DiConfig::from_env().unwrap();
    let ctx = ApplicationContext::new(&config).await.unwrap();

    let result = ctx.extraction_facade.extract("https://example.com").await;
    assert!(result.is_ok());
}
```

**Run integration tests**:
```bash
cargo test -p riptide-facade -- --ignored
```

---

## Migration Checklist

### Week 1: HttpClient Port
- [ ] Verify `HttpClient` port trait in riptide-types
- [ ] Implement `ReqwestAdapter` in riptide-fetch
- [ ] Refactor `ExtractionFacade` to use port
- [ ] Wire adapter in composition root
- [ ] Remove reqwest from facade Cargo.toml
- [ ] Update tests to use mock adapter
- [ ] Verify: `cargo test -p riptide-facade`

### Week 2: BrowserDriver Port
- [ ] Verify `BrowserDriver` port trait in riptide-types
- [ ] Implement `ChromeBrowserAdapter` in riptide-browser
- [ ] Refactor `BrowserFacade` to use port
- [ ] Remove chromiumoxide_cdp from facade
- [ ] Wire adapter in composition root
- [ ] Update tests
- [ ] Verify: `cargo test -p riptide-facade`

### Week 3: Remaining Infrastructure
- [ ] Create `Extractor` port adapter (riptide-extraction)
- [ ] Create `Spider` port adapter (riptide-spider)
- [ ] Refactor all remaining facades
- [ ] Remove ALL infrastructure deps from facade Cargo.toml
- [ ] Verify: `cargo tree -p riptide-facade`

### Week 4: Break Circular Dependency
- [ ] Create `riptide-test-fixtures` crate
- [ ] Move test mocks to test-fixtures
- [ ] Update facade dev-dependencies
- [ ] Remove facade → api dev-dependency
- [ ] Verify: No cycles in dependency graph

### Week 5: Validation
- [ ] Create architecture fitness function script
- [ ] Add CI check for architecture compliance
- [ ] Update all crate READMEs
- [ ] Write ADR-001
- [ ] Run full test suite
- [ ] Final audit

---

## Success Metrics

| Metric | Target | Verification |
|--------|--------|--------------|
| Facade infrastructure dependencies | 0 | `cargo tree -p riptide-facade` |
| Circular dependencies | 0 | `cargo tree -e normal` |
| Port trait coverage | 100% | All infra has adapters |
| Unit test speed | <1s | `cargo test -p riptide-facade` |
| CI architecture check | Pass | GitHub Actions |

---

## Rollback Plan

If issues arise:

1. **Keep changes in feature branch** until fully validated
2. **Incremental merge**: One port at a time
3. **Feature flags**: Can temporarily enable old code paths
4. **Monitoring**: Watch for performance regressions

---

## Support & Questions

- Architecture questions → Create GitHub Discussion
- Implementation help → Tag @architecture-team in PR
- CI failures → Check `scripts/check-architecture.sh` output

---

**End of Remediation Guide**
