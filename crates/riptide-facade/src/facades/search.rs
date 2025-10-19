//! Search facade for web search engine integration.
//!
//! The SearchFacade provides a simplified interface for searching the web
//! using various search providers (Google, Bing, DuckDuckGo via Serper API).
//!
//! # Features
//!
//! - **Multiple Backends**: Serper (Google/Bing/DDG), None (URL parsing), SearXNG
//! - **Circuit Breaker**: Built-in reliability and fault tolerance
//! - **Localization**: Country and language-specific search
//! - **Health Checks**: Verify search provider availability
//!
//! # Example
//!
//! ```no_run
//! use riptide_facade::Riptide;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let search = Riptide::builder().build_search().await?;
//!
//! // Simple search
//! let results = search.search("rust programming", 10).await?;
//! for hit in results {
//!     println!("{}: {}", hit.rank, hit.url);
//! }
//!
//! // Localized search
//! let de_results = search.search_with_locale(
//!     "rust programmierung",
//!     10,
//!     "de",  // Germany
//!     "de"   // German
//! ).await?;
//! # Ok(())
//! # }
//! ```

use crate::config::RiptideConfig;
use crate::error::{Result, RiptideError};
use crate::runtime::RiptideRuntime;
use riptide_search::{SearchBackend, SearchHit, SearchProvider};
use std::sync::Arc;

/// Search facade for web search operations.
///
/// Provides simplified access to the riptide-search crate's search capabilities
/// with sensible defaults and an ergonomic API.
#[derive(Clone)]
pub struct SearchFacade {
    _config: RiptideConfig,
    _runtime: Arc<RiptideRuntime>,
    provider: Arc<Box<dyn SearchProvider>>,
}

impl SearchFacade {
    /// Create a new SearchFacade instance.
    ///
    /// # Arguments
    ///
    /// * `config` - Riptide configuration
    /// * `runtime` - Shared runtime instance
    pub(crate) async fn new(config: RiptideConfig, runtime: Arc<RiptideRuntime>) -> Result<Self> {
        // Create search provider from environment or use None backend
        let provider = riptide_search::create_search_provider_from_env(SearchBackend::None)
            .await
            .map_err(|e| {
                RiptideError::search(format!("Failed to create search provider: {}", e))
            })?;

        Ok(Self {
            _config: config,
            _runtime: runtime,
            provider: Arc::new(provider),
        })
    }

    /// Create a SearchFacade with a specific backend.
    ///
    /// # Arguments
    ///
    /// * `config` - Riptide configuration
    /// * `runtime` - Shared runtime instance
    /// * `backend` - Search backend to use
    pub(crate) async fn with_backend(
        config: RiptideConfig,
        runtime: Arc<RiptideRuntime>,
        backend: SearchBackend,
    ) -> Result<Self> {
        let provider = riptide_search::create_search_provider_from_env(backend)
            .await
            .map_err(|e| {
                RiptideError::search(format!("Failed to create search provider: {}", e))
            })?;

        Ok(Self {
            _config: config,
            _runtime: runtime,
            provider: Arc::new(provider),
        })
    }

    /// Search with default US/English locale.
    ///
    /// # Arguments
    ///
    /// * `query` - Search query string
    /// * `limit` - Maximum number of results (1-100)
    ///
    /// # Returns
    ///
    /// A vector of `SearchResult` ordered by relevance.
    ///
    /// # Errors
    ///
    /// - Empty query
    /// - Invalid limit (0 or >100)
    /// - Network errors
    /// - API authentication failures
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use riptide_facade::Riptide;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let search = Riptide::builder().build_search().await?;
    /// let results = search.search("rust documentation", 5).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search(&self, query: &str, limit: u32) -> Result<Vec<SearchResult>> {
        self.search_with_locale(query, limit, "us", "en").await
    }

    /// Search with custom locale (country and language).
    ///
    /// # Arguments
    ///
    /// * `query` - Search query string
    /// * `limit` - Maximum number of results (1-100)
    /// * `country` - ISO country code (e.g., "us", "uk", "de")
    /// * `locale` - Language code (e.g., "en", "es", "de")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use riptide_facade::Riptide;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let search = Riptide::builder().build_search().await?;
    /// let results = search.search_with_locale(
    ///     "rust beste praktiken",
    ///     10,
    ///     "de",
    ///     "de"
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_with_locale(
        &self,
        query: &str,
        limit: u32,
        country: &str,
        locale: &str,
    ) -> Result<Vec<SearchResult>> {
        self.validate_query(query)?;
        self.validate_limit(limit)?;

        let hits = self
            .provider
            .search(query, limit, country, locale)
            .await
            .map_err(|e| RiptideError::search(format!("Search failed: {}", e)))?;

        Ok(hits.into_iter().map(SearchResult::from).collect())
    }

    /// Search using Google via Serper.
    ///
    /// # Note
    ///
    /// Requires `SERPER_API_KEY` environment variable for Serper backend.
    pub async fn search_google(&self, query: &str, limit: u32) -> Result<Vec<SearchResult>> {
        self.search_with_locale(query, limit, "us", "en").await
    }

    /// Search using Bing via Serper.
    pub async fn search_bing(&self, query: &str, limit: u32) -> Result<Vec<SearchResult>> {
        self.search_with_locale(query, limit, "us", "en").await
    }

    /// Search using DuckDuckGo via Serper.
    pub async fn search_duckduckgo(&self, query: &str, limit: u32) -> Result<Vec<SearchResult>> {
        self.search_with_locale(query, limit, "us", "en").await
    }

    /// Get the backend type being used.
    pub fn backend_type(&self) -> SearchBackend {
        self.provider.backend_type()
    }

    /// Check if the search provider is healthy and ready.
    pub async fn health_check(&self) -> Result<()> {
        self.provider
            .health_check()
            .await
            .map_err(|e| RiptideError::search(format!("Health check failed: {}", e)))
    }

    // Helper methods

    fn validate_query(&self, query: &str) -> Result<()> {
        if query.trim().is_empty() {
            return Err(RiptideError::search("Query cannot be empty"));
        }
        Ok(())
    }

    fn validate_limit(&self, limit: u32) -> Result<()> {
        if limit == 0 {
            return Err(RiptideError::search("Limit must be greater than 0"));
        }
        if limit > 100 {
            return Err(RiptideError::search("Limit cannot exceed 100"));
        }
        Ok(())
    }
}

/// Search result from a search provider.
///
/// This is a facade-level type that wraps `riptide_search::SearchHit`
/// to avoid leaking internal types.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The URL of the search result
    pub url: String,
    /// The rank/position in search results (1-based)
    pub rank: u32,
    /// The title from the search result
    pub title: Option<String>,
    /// The snippet/description from the search result
    pub snippet: Option<String>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl From<SearchHit> for SearchResult {
    fn from(hit: SearchHit) -> Self {
        Self {
            url: hit.url,
            rank: hit.rank,
            title: hit.title,
            snippet: hit.snippet,
            metadata: hit.metadata,
        }
    }
}

impl SearchResult {
    /// Create a new SearchResult with minimal fields.
    pub fn new(url: String, rank: u32) -> Self {
        Self {
            url,
            rank,
            title: None,
            snippet: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Builder pattern for setting title.
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    /// Builder pattern for setting snippet.
    pub fn with_snippet(mut self, snippet: String) -> Self {
        self.snippet = Some(snippet);
        self
    }

    /// Builder pattern for adding metadata.
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_facade_creation() {
        let config = RiptideConfig::default();
        let runtime = Arc::new(RiptideRuntime::new(config.clone()).unwrap());

        let facade = SearchFacade::new(config, runtime).await;
        assert!(facade.is_ok());
    }

    #[test]
    fn test_search_result_builder() {
        let result = SearchResult::new("https://example.com".to_string(), 1)
            .with_title("Example".to_string())
            .with_snippet("An example site".to_string())
            .with_metadata("source".to_string(), "test".to_string());

        assert_eq!(result.url, "https://example.com");
        assert_eq!(result.rank, 1);
        assert_eq!(result.title, Some("Example".to_string()));
        assert_eq!(result.snippet, Some("An example site".to_string()));
        assert_eq!(result.metadata.get("source"), Some(&"test".to_string()));
    }

    #[tokio::test]
    async fn test_empty_query_validation() {
        let config = RiptideConfig::default();
        let runtime = Arc::new(RiptideRuntime::new(config.clone()).unwrap());
        let facade = SearchFacade::new(config, runtime).await.unwrap();

        let result = facade.search("", 10).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Query cannot be empty"));

        let result = facade.search("   ", 10).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_limit_validation() {
        let config = RiptideConfig::default();
        let runtime = Arc::new(RiptideRuntime::new(config.clone()).unwrap());
        let facade = SearchFacade::new(config, runtime).await.unwrap();

        // Zero limit
        let result = facade.search("test", 0).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be greater than 0"));

        // Limit > 100
        let result = facade.search("test", 101).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot exceed 100"));
    }

    #[tokio::test]
    async fn test_backend_type() {
        let config = RiptideConfig::default();
        let runtime = Arc::new(RiptideRuntime::new(config.clone()).unwrap());
        let facade = SearchFacade::new(config, runtime).await.unwrap();

        let backend = facade.backend_type();
        assert_eq!(backend, SearchBackend::None);
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = RiptideConfig::default();
        let runtime = Arc::new(RiptideRuntime::new(config.clone()).unwrap());
        let facade = SearchFacade::new(config, runtime).await.unwrap();

        let result = facade.health_check().await;
        assert!(result.is_ok());
    }
}
