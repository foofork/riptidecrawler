//! Search facade for simplified web search operations.
//!
//! This module provides a high-level interface for performing web searches
//! using various backend providers (Serper, None, SearXNG).
//!
//! # Examples
//!
//! ```rust,no_run
//! use riptide_facade::SearchFacade;
//! use riptide_search::SearchBackend;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create facade with Serper backend
//! let facade = SearchFacade::with_api_key(
//!     SearchBackend::Serper,
//!     Some("your-api-key".to_string())
//! ).await?;
//!
//! // Perform a simple search
//! let results = facade.search("rust programming").await?;
//! for hit in results {
//!     println!("{}: {} - {}", hit.rank, hit.title.unwrap_or_default(), hit.url);
//! }
//!
//! // Search with custom options
//! let results = facade.search_with_options(
//!     "web scraping",
//!     20,  // limit
//!     "us", // country
//!     "en"  // locale
//! ).await?;
//! # Ok(())
//! # }
//! ```

use anyhow::Result;
use riptide_search::{
    create_search_provider, SearchBackend, SearchConfig, SearchHit, SearchProvider,
};
use std::sync::Arc;

/// A high-level facade for web search operations.
///
/// This facade simplifies the use of the riptide-search crate by providing
/// convenient constructors and sensible defaults for common search scenarios.
///
/// # Thread Safety
///
/// SearchFacade is `Clone` and can be safely shared across threads. The underlying
/// provider is wrapped in an `Arc` for efficient cloning.
#[derive(Clone)]
pub struct SearchFacade {
    provider: Arc<Box<dyn SearchProvider>>,
}

impl SearchFacade {
    /// Create a new SearchFacade with the specified backend.
    ///
    /// This constructor uses environment variables for configuration:
    /// - `SERPER_API_KEY` for Serper backend
    /// - `SEARXNG_BASE_URL` for SearXNG backend
    ///
    /// # Parameters
    ///
    /// * `backend` - The search backend to use
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Required environment variables are missing
    /// - The backend is not properly configured
    /// - Provider initialization fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use riptide_facade::SearchFacade;
    /// use riptide_search::SearchBackend;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// // Uses SERPER_API_KEY from environment
    /// let facade = SearchFacade::new(SearchBackend::Serper).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(backend: SearchBackend) -> Result<Self> {
        Self::with_api_key(backend, None).await
    }

    /// Create a new SearchFacade with an explicit API key.
    ///
    /// This constructor allows you to provide an API key directly instead of
    /// relying on environment variables. If `api_key` is `None`, it falls back
    /// to environment variable lookup.
    ///
    /// # Parameters
    ///
    /// * `backend` - The search backend to use
    /// * `api_key` - Optional API key (falls back to environment if None)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Required API key is missing for backends that need it
    /// - Provider initialization fails
    /// - Invalid configuration
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use riptide_facade::SearchFacade;
    /// use riptide_search::SearchBackend;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// // Explicit API key
    /// let facade = SearchFacade::with_api_key(
    ///     SearchBackend::Serper,
    ///     Some("your-api-key".to_string())
    /// ).await?;
    ///
    /// // Fallback to environment
    /// let facade = SearchFacade::with_api_key(
    ///     SearchBackend::Serper,
    ///     None
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn with_api_key(backend: SearchBackend, api_key: Option<String>) -> Result<Self> {
        let config = SearchConfig {
            backend: backend.clone(),
            api_key: api_key.or_else(|| match backend {
                SearchBackend::Serper => std::env::var("SERPER_API_KEY").ok(),
                SearchBackend::SearXNG => None,
                SearchBackend::None => None,
            }),
            base_url: match backend {
                SearchBackend::SearXNG => std::env::var("SEARXNG_BASE_URL").ok(),
                _ => None,
            },
            timeout_seconds: 30,
            enable_url_parsing: true,
        };

        let provider = create_search_provider(config).await?;
        Ok(Self {
            provider: Arc::new(provider),
        })
    }

    /// Create a SearchFacade with custom configuration.
    ///
    /// This provides full control over all search provider settings.
    ///
    /// # Parameters
    ///
    /// * `config` - Custom search configuration
    ///
    /// # Errors
    ///
    /// Returns an error if provider initialization fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use riptide_facade::SearchFacade;
    /// use riptide_search::{SearchBackend, SearchConfig};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let config = SearchConfig {
    ///     backend: SearchBackend::Serper,
    ///     api_key: Some("your-key".to_string()),
    ///     timeout_seconds: 60,
    ///     enable_url_parsing: true,
    ///     ..Default::default()
    /// };
    ///
    /// let facade = SearchFacade::with_config(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn with_config(config: SearchConfig) -> Result<Self> {
        let provider = create_search_provider(config).await?;
        Ok(Self {
            provider: Arc::new(provider),
        })
    }

    /// Perform a web search with default options.
    ///
    /// This is a convenience method that uses sensible defaults:
    /// - Limit: 10 results
    /// - Country: "us"
    /// - Locale: "en"
    ///
    /// # Parameters
    ///
    /// * `query` - The search query string
    ///
    /// # Returns
    ///
    /// A vector of search results ordered by relevance (rank).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The search request fails
    /// - Network issues occur
    /// - API rate limits are exceeded
    /// - The circuit breaker is open
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use riptide_facade::SearchFacade;
    /// use riptide_search::SearchBackend;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let facade = SearchFacade::new(SearchBackend::Serper).await?;
    /// let results = facade.search("rust async programming").await?;
    ///
    /// for hit in results {
    ///     println!("#{}: {}", hit.rank, hit.url);
    ///     if let Some(title) = hit.title {
    ///         println!("  Title: {}", title);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search(&self, query: &str) -> Result<Vec<SearchHit>> {
        self.search_with_options(query, 10, "us", "en").await
    }

    /// Perform a web search with custom options.
    ///
    /// This method provides full control over search parameters.
    ///
    /// # Parameters
    ///
    /// * `query` - The search query string
    /// * `limit` - Maximum number of results to return (1-100)
    /// * `country` - ISO country code for localization (e.g., "us", "uk", "de")
    /// * `locale` - Language locale (e.g., "en", "es", "fr", "de")
    ///
    /// # Returns
    ///
    /// A vector of search results ordered by relevance. The vector may contain
    /// fewer results than requested if the search provider returns fewer matches.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The search request fails
    /// - Invalid parameters are provided
    /// - Network issues occur
    /// - API rate limits are exceeded
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use riptide_facade::SearchFacade;
    /// use riptide_search::SearchBackend;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let facade = SearchFacade::new(SearchBackend::Serper).await?;
    ///
    /// // Search with custom options
    /// let results = facade.search_with_options(
    ///     "machine learning",
    ///     20,   // Get more results
    ///     "de", // German region
    ///     "de"  // German language
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_with_options(
        &self,
        query: &str,
        limit: u32,
        country: &str,
        locale: &str,
    ) -> Result<Vec<SearchHit>> {
        self.provider.search(query, limit, country, locale).await
    }

    /// Get the backend type of this search facade.
    ///
    /// This is useful for logging, debugging, and conditional logic based on
    /// the active search backend.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use riptide_facade::SearchFacade;
    /// use riptide_search::SearchBackend;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let facade = SearchFacade::new(SearchBackend::Serper).await?;
    /// assert_eq!(facade.backend_type(), SearchBackend::Serper);
    /// # Ok(())
    /// # }
    /// ```
    pub fn backend_type(&self) -> SearchBackend {
        self.provider.backend_type()
    }

    /// Check if the search provider is healthy and properly configured.
    ///
    /// This performs a health check on the underlying provider, verifying that:
    /// - API keys are valid (if required)
    /// - The service is reachable
    /// - Configuration is correct
    ///
    /// # Errors
    ///
    /// Returns an error if the health check fails, with details about the failure.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use riptide_facade::SearchFacade;
    /// use riptide_search::SearchBackend;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let facade = SearchFacade::new(SearchBackend::Serper).await?;
    ///
    /// match facade.health_check().await {
    ///     Ok(_) => println!("Search provider is healthy"),
    ///     Err(e) => eprintln!("Health check failed: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn health_check(&self) -> Result<()> {
        self.provider.health_check().await
    }
}

impl std::fmt::Debug for SearchFacade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SearchFacade")
            .field("backend", &self.backend_type())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_facade_none_backend() {
        let facade = SearchFacade::new(SearchBackend::None).await.unwrap();
        assert_eq!(facade.backend_type(), SearchBackend::None);
    }

    #[tokio::test]
    async fn test_search_facade_with_config() {
        let config = SearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 60,
            enable_url_parsing: true,
        };

        let facade = SearchFacade::with_config(config).await.unwrap();
        assert_eq!(facade.backend_type(), SearchBackend::None);
    }

    #[tokio::test]
    async fn test_search_facade_clone() {
        let facade = SearchFacade::new(SearchBackend::None).await.unwrap();
        let cloned = facade.clone();

        assert_eq!(facade.backend_type(), cloned.backend_type());
    }

    #[tokio::test]
    async fn test_search_facade_debug() {
        let facade = SearchFacade::new(SearchBackend::None).await.unwrap();
        let debug_str = format!("{:?}", facade);
        assert!(debug_str.contains("SearchFacade"));
        // SearchBackend::None uses Debug trait which shows "None", not "none"
        assert!(debug_str.contains("None"));
    }

    #[tokio::test]
    async fn test_health_check_none_backend() {
        let facade = SearchFacade::new(SearchBackend::None).await.unwrap();
        assert!(facade.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_search_with_none_backend() {
        let facade = SearchFacade::new(SearchBackend::None).await.unwrap();

        // None backend parses URLs from query
        let results = facade
            .search("https://example.com https://test.com")
            .await
            .unwrap();

        // Should extract URLs from the query
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_search_with_options() {
        let facade = SearchFacade::new(SearchBackend::None).await.unwrap();

        let results = facade
            .search_with_options("https://example.com", 5, "us", "en")
            .await
            .unwrap();

        assert!(!results.is_empty());
    }
}
