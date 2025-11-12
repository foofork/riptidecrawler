# Domain Type Specifications for Transport Abstraction

**Date:** 2025-11-12
**Component:** Domain Layer Types
**Priority:** CRITICAL
**Estimated Implementation Time:** 4 hours

---

## Overview

This document specifies the **concrete domain types** that replace transport-specific types (HTTP, JSON) in the facade layer, enabling protocol-agnostic business logic.

---

## 1. FetchOperation (Replaces HttpMethod)

### Location
`crates/riptide-types/src/domain/fetch_operation.rs` (new file)

### Purpose
Replace HTTP-specific `HttpMethod` enum with domain-level operation intent.

### Full Implementation

```rust
//! Domain-level fetch operation types
//!
//! This module defines protocol-agnostic operation types that express
//! business intent rather than transport details.

use std::fmt;

/// Domain-level fetch operation
///
/// Represents the intent of a fetch operation without coupling to
/// HTTP or any specific transport protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchOperation {
    /// Retrieve resource (read-only, idempotent, cacheable)
    ///
    /// Equivalent to HTTP GET, but protocol-agnostic.
    /// Should not modify state on the server.
    Retrieve,

    /// Submit new data (write, non-idempotent, not cacheable)
    ///
    /// Equivalent to HTTP POST, but protocol-agnostic.
    /// Creates new resources or submits forms.
    Submit {
        /// Binary data to submit
        data: Vec<u8>,
        /// Content type hint (e.g., "application/json", "multipart/form-data")
        content_type: Option<String>,
    },

    /// Update existing resource (write, idempotent, not cacheable)
    ///
    /// Equivalent to HTTP PUT, but protocol-agnostic.
    /// Replaces entire resource with new representation.
    Update {
        /// Binary data for update
        data: Vec<u8>,
        /// Content type hint
        content_type: Option<String>,
    },

    /// Partial update (write, idempotent, not cacheable)
    ///
    /// Equivalent to HTTP PATCH, but protocol-agnostic.
    /// Modifies part of a resource.
    Patch {
        /// Binary patch data
        data: Vec<u8>,
        /// Content type hint (e.g., "application/json-patch+json")
        content_type: Option<String>,
    },

    /// Remove resource (write, idempotent, not cacheable)
    ///
    /// Equivalent to HTTP DELETE, but protocol-agnostic.
    /// Removes a resource from the system.
    Remove,

    /// Retrieve metadata only (read-only, idempotent, cacheable)
    ///
    /// Equivalent to HTTP HEAD, but protocol-agnostic.
    /// Returns metadata without content body.
    Inspect,
}

impl FetchOperation {
    /// Check if operation is read-only
    pub fn is_readonly(&self) -> bool {
        matches!(self, Self::Retrieve | Self::Inspect)
    }

    /// Check if operation modifies state
    pub fn is_mutating(&self) -> bool {
        !self.is_readonly()
    }

    /// Check if operation is idempotent
    ///
    /// Idempotent operations can be safely retried without side effects.
    pub fn is_idempotent(&self) -> bool {
        matches!(
            self,
            Self::Retrieve | Self::Update { .. } | Self::Patch { .. } | Self::Remove | Self::Inspect
        )
    }

    /// Check if operation supports caching
    pub fn is_cacheable(&self) -> bool {
        matches!(self, Self::Retrieve | Self::Inspect)
    }

    /// Get operation body data if present
    pub fn body_data(&self) -> Option<&[u8]> {
        match self {
            Self::Submit { data, .. }
            | Self::Update { data, .. }
            | Self::Patch { data, .. } => Some(data),
            _ => None,
        }
    }

    /// Get content type hint if present
    pub fn content_type(&self) -> Option<&str> {
        match self {
            Self::Submit { content_type, .. }
            | Self::Update { content_type, .. }
            | Self::Patch { content_type, .. } => content_type.as_deref(),
            _ => None,
        }
    }

    /// Convert to HTTP method string (for adapter layer)
    ///
    /// This should only be called in HTTP adapter implementations,
    /// not in domain or facade layers.
    pub fn to_http_method(&self) -> &'static str {
        match self {
            Self::Retrieve => "GET",
            Self::Submit { .. } => "POST",
            Self::Update { .. } => "PUT",
            Self::Patch { .. } => "PATCH",
            Self::Remove => "DELETE",
            Self::Inspect => "HEAD",
        }
    }

    /// Create Retrieve operation
    pub fn retrieve() -> Self {
        Self::Retrieve
    }

    /// Create Submit operation with data
    pub fn submit(data: Vec<u8>) -> Self {
        Self::Submit {
            data,
            content_type: None,
        }
    }

    /// Create Submit operation with typed data
    pub fn submit_with_type(data: Vec<u8>, content_type: impl Into<String>) -> Self {
        Self::Submit {
            data,
            content_type: Some(content_type.into()),
        }
    }

    /// Create Update operation with data
    pub fn update(data: Vec<u8>) -> Self {
        Self::Update {
            data,
            content_type: None,
        }
    }

    /// Create Remove operation
    pub fn remove() -> Self {
        Self::Remove
    }
}

impl fmt::Display for FetchOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Retrieve => write!(f, "Retrieve"),
            Self::Submit { .. } => write!(f, "Submit"),
            Self::Update { .. } => write!(f, "Update"),
            Self::Patch { .. } => write!(f, "Patch"),
            Self::Remove => write!(f, "Remove"),
            Self::Inspect => write!(f, "Inspect"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_readonly_operations() {
        assert!(FetchOperation::retrieve().is_readonly());
        assert!(FetchOperation::Inspect.is_readonly());
        assert!(!FetchOperation::submit(vec![]).is_readonly());
        assert!(!FetchOperation::remove().is_readonly());
    }

    #[test]
    fn test_idempotent_operations() {
        assert!(FetchOperation::retrieve().is_idempotent());
        assert!(FetchOperation::update(vec![]).is_idempotent());
        assert!(FetchOperation::remove().is_idempotent());
        assert!(!FetchOperation::submit(vec![]).is_idempotent());
    }

    #[test]
    fn test_cacheable_operations() {
        assert!(FetchOperation::retrieve().is_cacheable());
        assert!(FetchOperation::Inspect.is_cacheable());
        assert!(!FetchOperation::submit(vec![]).is_cacheable());
    }

    #[test]
    fn test_body_data() {
        let data = vec![1, 2, 3];
        let op = FetchOperation::submit(data.clone());
        assert_eq!(op.body_data(), Some(data.as_slice()));

        let retrieve_op = FetchOperation::retrieve();
        assert_eq!(retrieve_op.body_data(), None);
    }

    #[test]
    fn test_http_method_conversion() {
        assert_eq!(FetchOperation::retrieve().to_http_method(), "GET");
        assert_eq!(FetchOperation::submit(vec![]).to_http_method(), "POST");
        assert_eq!(FetchOperation::update(vec![]).to_http_method(), "PUT");
        assert_eq!(FetchOperation::remove().to_http_method(), "DELETE");
        assert_eq!(FetchOperation::Inspect.to_http_method(), "HEAD");
    }

    #[test]
    fn test_builder_methods() {
        let op = FetchOperation::submit_with_type(vec![1, 2, 3], "application/json");
        assert_eq!(op.content_type(), Some("application/json"));
        assert_eq!(op.body_data(), Some(&[1, 2, 3][..]));
    }
}
```

### Integration Points

#### In `FetchOptions` (facade layer)
```rust
// ❌ BEFORE
pub struct FetchOptions {
    pub method: HttpMethod,
    pub headers: Vec<(String, String)>,
    pub timeout: Duration,
}

// ✅ AFTER
pub struct FetchOptions {
    pub operation: FetchOperation,
    pub metadata: OperationMetadata,
    pub timeout: Duration,
}
```

#### In HTTP Adapter
```rust
// HTTP adapter converts to HTTP request
impl HttpClient for ReqwestAdapter {
    async fn request(&self, req: HttpRequest) -> Result<HttpResponse> {
        let method = req.operation.to_http_method();
        let mut builder = self.client.request(
            reqwest::Method::from_bytes(method.as_bytes())?,
            &req.url
        );

        if let Some(data) = req.operation.body_data() {
            builder = builder.body(data.to_vec());
        }

        // ... rest of implementation
    }
}
```

---

## 2. OperationMetadata (Replaces HTTP Headers)

### Location
`crates/riptide-types/src/domain/operation_metadata.rs` (new file)

### Purpose
Provide protocol-agnostic metadata container that can be adapted to any transport.

### Full Implementation

```rust
//! Protocol-agnostic operation metadata
//!
//! This module provides a generic metadata container that can represent
//! headers, tags, labels, or any key-value metadata for operations.

use std::collections::HashMap;
use std::fmt;

/// Generic operation metadata container
///
/// Provides a protocol-agnostic way to attach metadata to operations.
/// Can be converted to HTTP headers, gRPC metadata, or any other format.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OperationMetadata {
    /// Key-value entries
    entries: HashMap<String, String>,
}

impl OperationMetadata {
    /// Create new empty metadata
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(capacity),
        }
    }

    /// Add metadata entry (builder pattern)
    pub fn with_entry(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.entries.insert(key.into(), value.into());
        self
    }

    /// Insert metadata entry
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.entries.insert(key.into(), value.into());
    }

    /// Get metadata value by key
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(|s| s.as_str())
    }

    /// Get metadata value by key (case-insensitive)
    pub fn get_case_insensitive(&self, key: &str) -> Option<&str> {
        let key_lower = key.to_lowercase();
        self.entries
            .iter()
            .find(|(k, _)| k.to_lowercase() == key_lower)
            .map(|(_, v)| v.as_str())
    }

    /// Check if key exists
    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// Remove metadata entry
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.entries.remove(key)
    }

    /// Iterate over entries
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if metadata is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Convert to HTTP headers (for HTTP adapter)
    pub fn to_http_headers(&self) -> HashMap<String, String> {
        self.entries.clone()
    }

    /// Create from HTTP headers (for HTTP adapter)
    pub fn from_http_headers(headers: HashMap<String, String>) -> Self {
        Self { entries: headers }
    }

    /// Common metadata: User-Agent
    pub fn with_user_agent(self, user_agent: impl Into<String>) -> Self {
        self.with_entry("User-Agent", user_agent)
    }

    /// Common metadata: Accept
    pub fn with_accept(self, accept: impl Into<String>) -> Self {
        self.with_entry("Accept", accept)
    }

    /// Common metadata: Content-Type
    pub fn with_content_type(self, content_type: impl Into<String>) -> Self {
        self.with_entry("Content-Type", content_type)
    }

    /// Common metadata: Authorization
    pub fn with_authorization(self, auth: impl Into<String>) -> Self {
        self.with_entry("Authorization", auth)
    }

    /// Merge with another metadata (other overwrites self)
    pub fn merge(&mut self, other: &OperationMetadata) {
        for (key, value) in &other.entries {
            self.entries.insert(key.clone(), value.clone());
        }
    }
}

impl From<HashMap<String, String>> for OperationMetadata {
    fn from(entries: HashMap<String, String>) -> Self {
        Self { entries }
    }
}

impl From<OperationMetadata> for HashMap<String, String> {
    fn from(metadata: OperationMetadata) -> Self {
        metadata.entries
    }
}

impl FromIterator<(String, String)> for OperationMetadata {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        Self {
            entries: iter.into_iter().collect(),
        }
    }
}

impl fmt::Display for OperationMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OperationMetadata {{ ")?;
        for (i, (key, value)) in self.entries.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", key, value)?;
        }
        write!(f, " }}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_pattern() {
        let metadata = OperationMetadata::new()
            .with_entry("key1", "value1")
            .with_entry("key2", "value2");

        assert_eq!(metadata.get("key1"), Some("value1"));
        assert_eq!(metadata.get("key2"), Some("value2"));
        assert_eq!(metadata.len(), 2);
    }

    #[test]
    fn test_case_insensitive_get() {
        let metadata = OperationMetadata::new()
            .with_entry("Content-Type", "application/json");

        assert_eq!(metadata.get_case_insensitive("content-type"), Some("application/json"));
        assert_eq!(metadata.get_case_insensitive("CONTENT-TYPE"), Some("application/json"));
    }

    #[test]
    fn test_common_metadata() {
        let metadata = OperationMetadata::new()
            .with_user_agent("RiptideCrawler/1.0")
            .with_accept("application/json")
            .with_content_type("text/html");

        assert_eq!(metadata.get("User-Agent"), Some("RiptideCrawler/1.0"));
        assert_eq!(metadata.get("Accept"), Some("application/json"));
        assert_eq!(metadata.get("Content-Type"), Some("text/html"));
    }

    #[test]
    fn test_merge() {
        let mut base = OperationMetadata::new()
            .with_entry("key1", "value1");

        let override_metadata = OperationMetadata::new()
            .with_entry("key1", "new_value1")
            .with_entry("key2", "value2");

        base.merge(&override_metadata);

        assert_eq!(base.get("key1"), Some("new_value1"));
        assert_eq!(base.get("key2"), Some("value2"));
    }

    #[test]
    fn test_from_iterator() {
        let entries = vec![
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
        ];

        let metadata: OperationMetadata = entries.into_iter().collect();
        assert_eq!(metadata.len(), 2);
    }
}
```

### Integration Example

```rust
// In facade layer
let metadata = OperationMetadata::new()
    .with_user_agent("RiptideCrawler/1.0")
    .with_accept("text/html")
    .with_entry("X-Request-ID", request_id);

let operation = FetchOperation::retrieve();

let result = http_client.fetch_with_metadata(url, operation, metadata).await?;
```

---

## 3. Module Structure Update

### Update `crates/riptide-types/src/lib.rs`

```rust
// Add new domain module
pub mod domain;

// In domain/mod.rs
pub mod fetch_operation;
pub mod operation_metadata;

pub use fetch_operation::FetchOperation;
pub use operation_metadata::OperationMetadata;
```

---

## 4. Migration Checklist

### Phase 1: Add Domain Types (2 hours)
- [ ] Create `src/domain/` directory in riptide-types
- [ ] Implement `FetchOperation` with tests
- [ ] Implement `OperationMetadata` with tests
- [ ] Export from `riptide-types::domain`
- [ ] Run `cargo test -p riptide-types`

### Phase 2: Update HttpClient Trait (1 hour)
- [ ] Add `fetch_with_metadata()` method to `HttpClient` trait
- [ ] Update `HttpRequest` to include `FetchOperation`
- [ ] Update tests

### Phase 3: Update Facades (1 hour)
- [ ] Replace `HttpMethod` with `FetchOperation` in `FetchOptions`
- [ ] Replace `Vec<(String, String)>` with `OperationMetadata`
- [ ] Update facade method signatures
- [ ] Update tests

### Phase 4: Update Adapters (1 hour)
- [ ] Update reqwest adapter to convert `FetchOperation` to HTTP method
- [ ] Update adapter to convert `OperationMetadata` to headers
- [ ] Test adapter conversion logic

---

## 5. Benefits Realized

✅ **Transport Independence**
- Can support gRPC, GraphQL, WebSockets without facade changes

✅ **Type Safety**
- Compile-time guarantees for operation properties (idempotency, caching)

✅ **Self-Documenting**
- Operation intent is clear from type

✅ **Testability**
- Easy to mock without HTTP stack
- Property-based testing for invariants

✅ **Performance**
- No runtime overhead (zero-cost abstractions)
- Efficient conversion to transport types

---

## 6. Testing Strategy

### Unit Tests
```rust
#[test]
fn test_retrieve_operation_is_cacheable() {
    let op = FetchOperation::retrieve();
    assert!(op.is_readonly());
    assert!(op.is_idempotent());
    assert!(op.is_cacheable());
}

#[test]
fn test_submit_operation_has_body() {
    let data = vec![1, 2, 3];
    let op = FetchOperation::submit(data.clone());
    assert_eq!(op.body_data(), Some(data.as_slice()));
    assert!(!op.is_idempotent());
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_fetch_with_domain_types() {
    let http_client = create_test_client();
    let operation = FetchOperation::retrieve();
    let metadata = OperationMetadata::new()
        .with_user_agent("Test/1.0");

    let response = http_client
        .fetch_with_metadata("http://example.com", operation, metadata)
        .await
        .unwrap();

    assert!(response.is_success());
}
```

---

## Conclusion

These domain types form the foundation for transport-agnostic business logic, enabling:

- **Clean Architecture**: Domain logic independent of transport
- **Flexibility**: Easy to support new protocols
- **Maintainability**: Clear, self-documenting types
- **Testability**: Mock-friendly, property-testable

Implementation time: **4 hours** with comprehensive tests.
