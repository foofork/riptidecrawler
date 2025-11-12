# SearchProvider Port Trait Design

**Port Trait**: `SearchProvider`
**Facade**: SearchFacade
**Action**: Create abstraction
**Location**: `/workspaces/riptidecrawler/crates/riptide-types/src/ports/search.rs`
**Risk Level**: Low
**Estimated Time**: 3-4 hours

---

## Rationale

The `SearchFacade` wraps search functionality from `riptide-search` crate, but the `SearchProvider` trait currently lives in the infrastructure layer. We need to move it to the domain layer (`riptide-types/ports`) for proper hexagonal architecture.

---

## Current SearchFacade Interface

**File**: `/workspaces/riptidecrawler/crates/riptide-facade/src/facades/search.rs`

```rust
pub struct SearchFacade {
    provider: Arc<Box<dyn SearchProvider>>, // ← Uses SearchProvider from riptide-search
}

impl SearchFacade {
    pub async fn search(&self, query: &str) -> Result<Vec<SearchHit>>;
    pub async fn search_with_options(
        &self,
        query: &str,
        limit: usize,
        country: &str,
        locale: &str
    ) -> Result<Vec<SearchHit>>;
}
```

**Key Operations**:
1. Simple text search
2. Search with pagination/filtering options
3. Backend abstraction (Serper, SearXNG, None)

---

## Port Trait Design

### File: `/workspaces/riptidecrawler/crates/riptide-types/src/ports/search.rs`

```rust
//! Search Provider Port - Hexagonal abstraction for web search operations
//!
//! This port trait provides a backend-agnostic interface for search operations,
//! enabling dependency inversion and facilitating testing with mock implementations.
//!
//! # Architecture
//!
//! ```text
//! Domain Layer (riptide-types)
//!     ↓ defines SearchProvider trait
//! Infrastructure Layer (riptide-search)
//!     ↓ implements Serper, SearXNG, None backends
//! Application Layer (riptide-facade)
//!     ↓ adapts SearchFacade
//! Composition Root (riptide-api)
//!     ↓ wires Arc<dyn SearchProvider>
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_types::ports::{SearchProvider, SearchQuery};
//!
//! async fn find_results(provider: &dyn SearchProvider, q: &str) -> Result<Vec<SearchResult>> {
//!     let query = SearchQuery::new(q);
//!     let results = provider.search(query).await?;
//!     Ok(results.hits)
//! }
//! ```

use async_trait::async_trait;
use crate::error::Result as RiptideResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Search provider port trait
///
/// Defines the interface for web search operations.
/// Implementations handle API requests, result parsing, and error handling.
#[async_trait]
pub trait SearchProvider: Send + Sync {
    /// Perform a search query
    ///
    /// # Arguments
    ///
    /// * `query` - The search query with all parameters
    ///
    /// # Returns
    ///
    /// * `Ok(SearchResults)` - Search results with metadata
    /// * `Err(_)` - API error, network error, or invalid query
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let query = SearchQuery::new("rust programming")
    ///     .with_limit(20)
    ///     .with_country("us");
    /// let results = provider.search(query).await?;
    /// println!("Found {} results", results.hits.len());
    /// ```
    async fn search(&self, query: SearchQuery) -> RiptideResult<SearchResults>;

    /// Index a document for search
    ///
    /// (Optional operation - not all backends support indexing)
    ///
    /// # Arguments
    ///
    /// * `doc` - The document to index
    ///
    /// # Returns
    ///
    /// * `Ok(DocumentId)` - The ID of the indexed document
    /// * `Err(_)` - Indexing error or unsupported operation
    async fn index_document(&self, doc: SearchDocument) -> RiptideResult<DocumentId>;

    /// Delete a document from the index
    ///
    /// (Optional operation - not all backends support deletion)
    ///
    /// # Arguments
    ///
    /// * `id` - The document ID to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Document deleted successfully
    /// * `Err(_)` - Deletion error or unsupported operation
    async fn delete_document(&self, id: DocumentId) -> RiptideResult<()>;

    /// Get search provider capabilities
    ///
    /// # Returns
    ///
    /// Provider capabilities (indexing, deletion, etc.)
    fn capabilities(&self) -> SearchCapabilities;

    /// Get provider name/type
    ///
    /// # Returns
    ///
    /// String identifying the provider (e.g., "serper", "searxng", "none")
    fn provider_name(&self) -> &str;

    /// Check if provider is available/healthy
    ///
    /// # Returns
    ///
    /// `true` if provider is ready to process requests
    async fn is_available(&self) -> bool;
}

/// Search query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Search query text
    pub query: String,

    /// Maximum results to return
    #[serde(default = "default_limit")]
    pub limit: usize,

    /// Offset for pagination
    #[serde(default)]
    pub offset: usize,

    /// Country code (e.g., "us", "uk")
    #[serde(default)]
    pub country: Option<String>,

    /// Locale code (e.g., "en", "es")
    #[serde(default)]
    pub locale: Option<String>,

    /// Search type/domain filter
    #[serde(default)]
    pub search_type: SearchType,

    /// Time range filter
    #[serde(default)]
    pub time_range: Option<TimeRange>,

    /// Custom parameters for backend-specific features
    #[serde(default)]
    pub custom_params: HashMap<String, String>,
}

impl SearchQuery {
    /// Create a new search query
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            limit: 10,
            offset: 0,
            country: None,
            locale: None,
            search_type: SearchType::Web,
            time_range: None,
            custom_params: HashMap::new(),
        }
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set offset for pagination
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// Set country filter
    pub fn with_country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }

    /// Set locale filter
    pub fn with_locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = Some(locale.into());
        self
    }

    /// Set search type
    pub fn with_type(mut self, search_type: SearchType) -> Self {
        self.search_type = search_type;
        self
    }

    /// Set time range filter
    pub fn with_time_range(mut self, time_range: TimeRange) -> Self {
        self.time_range = Some(time_range);
        self
    }
}

/// Search type/domain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SearchType {
    /// General web search
    #[default]
    Web,
    /// Image search
    Image,
    /// News search
    News,
    /// Video search
    Video,
    /// Academic/scholarly search
    Scholar,
}

/// Time range filter for search results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeRange {
    /// Past hour
    Hour,
    /// Past 24 hours
    Day,
    /// Past week
    Week,
    /// Past month
    Month,
    /// Past year
    Year,
    /// All time
    All,
}

/// Search results container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    /// Search hits/results
    pub hits: Vec<SearchHit>,

    /// Total number of results available
    pub total_results: u64,

    /// Query that was executed
    pub query: String,

    /// Search duration in milliseconds
    pub search_duration_ms: u64,

    /// Additional metadata from provider
    pub metadata: HashMap<String, String>,
}

/// Individual search result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchHit {
    /// Result rank/position
    pub rank: usize,

    /// Page title
    pub title: Option<String>,

    /// Page URL
    pub url: String,

    /// Description/snippet
    pub description: Option<String>,

    /// Domain name
    pub domain: Option<String>,

    /// Published/updated date (ISO 8601)
    pub date: Option<String>,

    /// Result type (web, news, image, etc.)
    pub result_type: SearchType,

    /// Result score/relevance (0.0-1.0)
    pub score: Option<f64>,
}

/// Document to be indexed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocument {
    /// Document ID (optional - generated if not provided)
    pub id: Option<DocumentId>,

    /// Document title
    pub title: String,

    /// Document content
    pub content: String,

    /// Document URL
    pub url: String,

    /// Document metadata
    pub metadata: HashMap<String, String>,

    /// Timestamp (ISO 8601)
    pub timestamp: Option<String>,
}

/// Document identifier
pub type DocumentId = String;

/// Provider capabilities
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SearchCapabilities {
    /// Supports document indexing
    pub supports_indexing: bool,

    /// Supports document deletion
    pub supports_deletion: bool,

    /// Supports image search
    pub supports_image_search: bool,

    /// Supports news search
    pub supports_news_search: bool,

    /// Supports time range filtering
    pub supports_time_range: bool,

    /// Supports custom parameters
    pub supports_custom_params: bool,
}

impl Default for SearchCapabilities {
    fn default() -> Self {
        Self {
            supports_indexing: false,
            supports_deletion: false,
            supports_image_search: false,
            supports_news_search: false,
            supports_time_range: false,
            supports_custom_params: false,
        }
    }
}

// Default value functions for serde
fn default_limit() -> usize {
    10
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_query_builder() {
        let query = SearchQuery::new("test query")
            .with_limit(20)
            .with_country("us")
            .with_locale("en");

        assert_eq!(query.query, "test query");
        assert_eq!(query.limit, 20);
        assert_eq!(query.country, Some("us".to_string()));
    }

    #[test]
    fn test_search_hit_creation() {
        let hit = SearchHit {
            rank: 1,
            title: Some("Example".to_string()),
            url: "https://example.com".to_string(),
            description: Some("An example site".to_string()),
            domain: Some("example.com".to_string()),
            date: None,
            result_type: SearchType::Web,
            score: Some(0.95),
        };

        assert_eq!(hit.rank, 1);
        assert_eq!(hit.score, Some(0.95));
    }

    #[test]
    fn test_capabilities_default() {
        let caps = SearchCapabilities::default();
        assert!(!caps.supports_indexing);
        assert!(!caps.supports_deletion);
    }
}
```

---

## Integration with ApplicationContext

### Update context.rs

**File**: `/workspaces/riptidecrawler/crates/riptide-api/src/context.rs`

```diff
pub struct ApplicationContext {
    // ... other fields ...

-   #[cfg(feature = "search")]
-   /// SearchFacade for web search operations
-   pub search_facade: Option<Arc<riptide_facade::facades::SearchFacade>>,
+   #[cfg(feature = "search")]
+   /// Search provider for web search operations
+   pub search_provider: Option<Arc<dyn SearchProvider>>,

    // ... other fields ...
}
```

---

## Migration Path for riptide-search Crate

The `riptide-search` crate currently has its own `SearchProvider` trait. We need to:

1. Move the trait definition to `riptide-types/src/ports/search.rs`
2. Update `riptide-search` to implement the domain-layer trait
3. Keep backend-specific types in `riptide-search`

This is a **refactoring**, not a breaking change - the trait signature can remain the same.

---

## Benefits of Port Trait in Domain Layer

1. ✅ **Hexagonal compliance** - Trait in domain, implementations in infrastructure
2. ✅ **Testability** - Easy to create mock implementations
3. ✅ **Swappability** - Can swap search backends without changing API
4. ✅ **Type safety** - Strongly typed queries and results
5. ✅ **Documentation** - Clear contract for all search providers

---

## Mock Implementation for Testing

```rust
// File: crates/riptide-types/src/ports/search/mock.rs

#[cfg(test)]
pub struct MockSearchProvider {
    results: HashMap<String, Vec<SearchHit>>,
}

#[cfg(test)]
impl MockSearchProvider {
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
        }
    }

    pub fn with_results(mut self, query: &str, hits: Vec<SearchHit>) -> Self {
        self.results.insert(query.to_string(), hits);
        self
    }
}

#[cfg(test)]
#[async_trait]
impl SearchProvider for MockSearchProvider {
    async fn search(&self, query: SearchQuery) -> RiptideResult<SearchResults> {
        let hits = self
            .results
            .get(&query.query)
            .cloned()
            .unwrap_or_default();

        Ok(SearchResults {
            hits,
            total_results: hits.len() as u64,
            query: query.query.clone(),
            search_duration_ms: 10,
            metadata: HashMap::new(),
        })
    }

    async fn index_document(&self, _doc: SearchDocument) -> RiptideResult<DocumentId> {
        Err(RiptideError::unsupported("Indexing not supported in mock"))
    }

    async fn delete_document(&self, _id: DocumentId) -> RiptideResult<()> {
        Err(RiptideError::unsupported("Deletion not supported in mock"))
    }

    fn capabilities(&self) -> SearchCapabilities {
        SearchCapabilities::default()
    }

    fn provider_name(&self) -> &str {
        "mock"
    }

    async fn is_available(&self) -> bool {
        true
    }
}
```

---

## Next Steps

1. Create the port trait file
2. Update `riptide-search` to implement domain trait
3. Implement adapter (see `07-search-facade-adapter.md`)
4. Update ApplicationContext
5. Run tests

---

**Status**: ✅ Design Complete - Ready for Implementation
**Dependencies**: Requires refactoring `riptide-search` crate
**Blockers**: None
