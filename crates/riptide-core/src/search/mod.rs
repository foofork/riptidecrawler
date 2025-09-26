//! Search provider abstraction for flexible search backend implementations.
//!
//! This module provides a trait-based abstraction over different search providers,
//! allowing RipTide to work with multiple search backends without hard dependencies.
//!
//! ## Supported Backends
//!
//! - **Serper**: Google search via Serper.dev API (requires API key)
//! - **None**: URL parsing from query string, no external API required
//! - **SearXNG**: Self-hosted SearXNG instance (optional, future implementation)
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use riptide_core::search::{SearchProvider, SearchBackend, create_search_provider};
//!
//! # tokio_test::block_on(async {
//! let provider = create_search_provider(SearchBackend::Serper, Some("api-key".to_string())).await?;
//! let results = provider.search("rust programming", 10, "us", "en").await?;
//! # Ok::<(), anyhow::Error>(())
//! # });
//! ```

pub mod providers;
pub mod circuit_breaker;
pub mod none_provider;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a single search hit from a search provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchHit {
    /// The URL of the search result
    pub url: String,
    /// The rank/position in search results (1-based)
    pub rank: u32,
    /// The title from the search result
    pub title: Option<String>,
    /// The snippet/description from the search result
    pub snippet: Option<String>,
    /// Additional metadata specific to the search provider
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

impl SearchHit {
    /// Create a new SearchHit with minimal required fields
    pub fn new(url: String, rank: u32) -> Self {
        Self {
            url,
            rank,
            title: None,
            snippet: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Builder pattern for setting title
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    /// Builder pattern for setting snippet
    pub fn with_snippet(mut self, snippet: String) -> Self {
        self.snippet = Some(snippet);
        self
    }

    /// Builder pattern for adding metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Available search backends that can be used with the SearchProvider trait.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SearchBackend {
    /// Google search via Serper.dev API (requires API key)
    Serper,
    /// No external search - parse URLs from query string
    None,
    /// Self-hosted SearXNG instance (optional, future)
    #[serde(rename = "searxng")]
    SearXNG,
}

impl fmt::Display for SearchBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchBackend::Serper => write!(f, "serper"),
            SearchBackend::None => write!(f, "none"),
            SearchBackend::SearXNG => write!(f, "searxng"),
        }
    }
}

impl std::str::FromStr for SearchBackend {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "serper" => Ok(SearchBackend::Serper),
            "none" => Ok(SearchBackend::None),
            "searxng" => Ok(SearchBackend::SearXNG),
            _ => Err(anyhow::anyhow!(
                "Invalid search backend '{}'. Valid options: serper, none, searxng",
                s
            )),
        }
    }
}

/// Trait for search providers that can perform web searches.
///
/// This trait abstracts over different search backends, allowing for flexible
/// configuration and testing. Implementations should handle their own error
/// recovery, rate limiting, and API key management.
///
/// ## Error Handling
///
/// Implementations should return meaningful errors that can be used for:
/// - Circuit breaker decisions (temporary vs permanent failures)
/// - User feedback (API key missing, rate limit exceeded, etc.)
/// - Logging and monitoring
///
/// ## Threading and Safety
///
/// All implementations must be thread-safe (`Send + Sync`) to support
/// concurrent usage in the web server.
#[async_trait::async_trait]
pub trait SearchProvider: Send + Sync {
    /// Perform a web search with the given parameters.
    ///
    /// # Parameters
    ///
    /// - `query`: The search query string
    /// - `limit`: Maximum number of results to return (1-100)
    /// - `country`: ISO country code for localization (e.g., "us", "uk")
    /// - `locale`: Language locale (e.g., "en", "es", "fr")
    ///
    /// # Returns
    ///
    /// A vector of SearchHit results, ordered by relevance (rank).
    /// The vector may be shorter than the requested limit if fewer results
    /// are available.
    ///
    /// # Errors
    ///
    /// - API authentication failures (missing/invalid keys)
    /// - Network timeouts or connection errors
    /// - Rate limiting from the search provider
    /// - Invalid query parameters
    /// - Service unavailable errors
    async fn search(
        &self,
        query: &str,
        limit: u32,
        country: &str,
        locale: &str,
    ) -> Result<Vec<SearchHit>>;

    /// Get the backend type for this provider.
    /// Used for configuration, logging, and metrics.
    fn backend_type(&self) -> SearchBackend;

    /// Check if the provider is properly configured and ready to use.
    /// This is used for health checks and early error detection.
    async fn health_check(&self) -> Result<()>;
}

/// Configuration for search provider creation and behavior.
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// The search backend to use
    pub backend: SearchBackend,
    /// API key for external services (if required)
    pub api_key: Option<String>,
    /// Base URL for self-hosted services (e.g., SearXNG)
    pub base_url: Option<String>,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Whether to enable URL parsing from query for None backend
    pub enable_url_parsing: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            backend: SearchBackend::Serper,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: true,
        }
    }
}

/// Advanced configuration for search provider creation with circuit breaker settings.
#[derive(Debug, Clone)]
pub struct AdvancedSearchConfig {
    /// The search backend to use
    pub backend: SearchBackend,
    /// API key for external services (if required)
    pub api_key: Option<String>,
    /// Base URL for self-hosted services (e.g., SearXNG)
    pub base_url: Option<String>,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Whether to enable URL parsing from query for None backend
    pub enable_url_parsing: bool,
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfigOptions,
}

/// Circuit breaker configuration options.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfigOptions {
    /// Failure threshold percentage (0-100) to trigger circuit opening
    pub failure_threshold: u32,
    /// Minimum number of requests before circuit can open
    pub min_requests: u32,
    /// Time to wait before attempting to close an open circuit (in seconds)
    pub recovery_timeout_secs: u64,
}

impl Default for CircuitBreakerConfigOptions {
    fn default() -> Self {
        Self {
            failure_threshold: 50,
            min_requests: 5,
            recovery_timeout_secs: 60,
        }
    }
}

impl Default for AdvancedSearchConfig {
    fn default() -> Self {
        Self {
            backend: SearchBackend::Serper,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: true,
            circuit_breaker: CircuitBreakerConfigOptions::default(),
        }
    }
}

impl AdvancedSearchConfig {
    /// Create configuration from environment variables.
    pub fn from_env() -> Self {
        Self {
            backend: std::env::var("SEARCH_BACKEND")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(SearchBackend::Serper),
            api_key: match std::env::var("SEARCH_BACKEND").ok().as_deref() {
                Some("serper") | None => std::env::var("SERPER_API_KEY").ok(),
                _ => None,
            },
            base_url: match std::env::var("SEARCH_BACKEND").ok().as_deref() {
                Some("searxng") => std::env::var("SEARXNG_BASE_URL").ok(),
                _ => None,
            },
            timeout_seconds: std::env::var("SEARCH_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            enable_url_parsing: std::env::var("SEARCH_ENABLE_URL_PARSING")
                .map(|s| s.to_lowercase() == "true")
                .unwrap_or(true),
            circuit_breaker: CircuitBreakerConfigOptions {
                failure_threshold: std::env::var("CIRCUIT_BREAKER_FAILURE_THRESHOLD")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(50),
                min_requests: std::env::var("CIRCUIT_BREAKER_MIN_REQUESTS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(5),
                recovery_timeout_secs: std::env::var("CIRCUIT_BREAKER_RECOVERY_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(60),
            },
        }
    }

    /// Validate the configuration and return an error if invalid.
    pub fn validate(&self) -> Result<()> {
        // Validate backend-specific requirements
        match self.backend {
            SearchBackend::Serper => {
                if self.api_key.is_none() || self.api_key.as_ref().unwrap().trim().is_empty() {
                    return Err(anyhow::anyhow!("Serper backend requires a valid API key"));
                }
            }
            SearchBackend::SearXNG => {
                if self.base_url.is_none() || self.base_url.as_ref().unwrap().trim().is_empty() {
                    return Err(anyhow::anyhow!("SearXNG backend requires a valid base URL"));
                }
            }
            SearchBackend::None => {
                // None backend has no external dependencies
            }
        }

        // Validate timeout
        if self.timeout_seconds == 0 {
            return Err(anyhow::anyhow!("Timeout must be greater than 0"));
        }
        if self.timeout_seconds > 3600 {
            return Err(anyhow::anyhow!("Timeout cannot exceed 1 hour (3600 seconds)"));
        }

        // Validate circuit breaker settings
        if self.circuit_breaker.failure_threshold > 100 {
            return Err(anyhow::anyhow!("Failure threshold cannot exceed 100%"));
        }
        if self.circuit_breaker.min_requests == 0 {
            return Err(anyhow::anyhow!("Minimum requests must be greater than 0"));
        }
        if self.circuit_breaker.recovery_timeout_secs == 0 {
            return Err(anyhow::anyhow!("Recovery timeout must be greater than 0"));
        }

        Ok(())
    }
}

/// Factory function to create a search provider based on configuration.
///
/// This function handles the creation of the appropriate provider implementation
/// and wraps it with circuit breaker protection for reliability.
///
/// # Parameters
///
/// - `config`: Configuration specifying backend type and options
///
/// # Returns
///
/// A boxed SearchProvider implementation ready for use.
///
/// # Errors
///
/// - Invalid configuration (missing required fields)
/// - Unsupported backend type
/// - Provider initialization failures
pub async fn create_search_provider(config: SearchConfig) -> Result<Box<dyn SearchProvider>> {
    use providers::SerperProvider;
    use none_provider::NoneProvider;
    use circuit_breaker::CircuitBreakerWrapper;

    let provider: Box<dyn SearchProvider> = match config.backend {
        SearchBackend::Serper => {
            let api_key = config.api_key.ok_or_else(|| {
                anyhow::anyhow!("API key is required for Serper backend")
            })?;
            Box::new(SerperProvider::new(api_key, config.timeout_seconds))
        }
        SearchBackend::None => {
            Box::new(NoneProvider::new(config.enable_url_parsing))
        }
        SearchBackend::SearXNG => {
            let _base_url = config.base_url.ok_or_else(|| {
                anyhow::anyhow!("Base URL is required for SearXNG backend")
            })?;
            // Note: SearXNG provider is not yet implemented
            return Err(anyhow::anyhow!(
                "SearXNG backend is not yet implemented. Use 'serper' or 'none' instead."
            ));
        }
    };

    // Wrap with circuit breaker for reliability
    let circuit_breaker_provider = CircuitBreakerWrapper::new(provider);

    Ok(Box::new(circuit_breaker_provider))
}

/// Advanced SearchProvider factory with enhanced configuration support.
pub struct SearchProviderFactory;

impl SearchProviderFactory {
    /// Create a search provider with advanced configuration options.
    ///
    /// This factory method provides enhanced configuration support including:
    /// - Circuit breaker customization
    /// - Provider-specific timeouts
    /// - Health check configuration
    /// - Retry policies
    ///
    /// # Parameters
    ///
    /// - `config`: Advanced configuration for the search provider
    ///
    /// # Returns
    ///
    /// A configured SearchProvider with circuit breaker protection.
    pub async fn create_provider(config: AdvancedSearchConfig) -> Result<Box<dyn SearchProvider>> {
        use providers::SerperProvider;
        use none_provider::NoneProvider;
        use circuit_breaker::CircuitBreakerWrapper;

        // Validate configuration first
        config.validate()?;

        let provider: Box<dyn SearchProvider> = match config.backend {
            SearchBackend::Serper => {
                let api_key = config.api_key.ok_or_else(|| {
                    anyhow::anyhow!("API key is required for Serper backend")
                })?;
                Box::new(SerperProvider::new(api_key, config.timeout_seconds))
            }
            SearchBackend::None => {
                Box::new(NoneProvider::new(config.enable_url_parsing))
            }
            SearchBackend::SearXNG => {
                let _base_url = config.base_url.ok_or_else(|| {
                    anyhow::anyhow!("Base URL is required for SearXNG backend")
                })?;
                // Note: SearXNG provider is not yet implemented
                return Err(anyhow::anyhow!(
                    "SearXNG backend is not yet implemented. Use 'serper' or 'none' instead."
                ));
            }
        };

        // Create custom circuit breaker configuration
        let cb_config = circuit_breaker::CircuitBreakerConfig {
            failure_threshold_percentage: config.circuit_breaker.failure_threshold,
            minimum_request_threshold: config.circuit_breaker.min_requests,
            recovery_timeout: std::time::Duration::from_secs(config.circuit_breaker.recovery_timeout_secs),
            half_open_max_requests: 3, // Fixed value for now
        };

        // Wrap with configured circuit breaker
        let circuit_breaker_provider = CircuitBreakerWrapper::with_config(provider, cb_config);

        Ok(Box::new(circuit_breaker_provider))
    }

    /// Create a provider from environment variables with fallback to defaults.
    ///
    /// This method reads configuration from environment variables and provides
    /// sensible defaults for missing values.
    pub async fn create_from_env() -> Result<Box<dyn SearchProvider>> {
        let config = AdvancedSearchConfig::from_env();
        Self::create_provider(config).await
    }

    /// Create a provider with a specific backend, using environment for other settings.
    pub async fn create_with_backend(backend: SearchBackend) -> Result<Box<dyn SearchProvider>> {
        let mut config = AdvancedSearchConfig::from_env();
        config.backend = backend;
        Self::create_provider(config).await
    }
}

/// Convenience function to create a search provider from environment and backend type.
///
/// This function reads common environment variables and creates a search provider
/// with sensible defaults.
///
/// # Environment Variables
///
/// - `SERPER_API_KEY`: API key for Serper backend
/// - `SEARXNG_BASE_URL`: Base URL for SearXNG backend
/// - `SEARCH_TIMEOUT`: Request timeout in seconds (default: 30)
///
/// # Parameters
///
/// - `backend`: The search backend type to create
///
/// # Returns
///
/// A configured SearchProvider ready for use.
pub async fn create_search_provider_from_env(backend: SearchBackend) -> Result<Box<dyn SearchProvider>> {
    let config = SearchConfig {
        backend: backend.clone(),
        api_key: match backend {
            SearchBackend::Serper => std::env::var("SERPER_API_KEY").ok(),
            _ => None,
        },
        base_url: match backend {
            SearchBackend::SearXNG => std::env::var("SEARXNG_BASE_URL").ok(),
            _ => None,
        },
        timeout_seconds: std::env::var("SEARCH_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30),
        enable_url_parsing: std::env::var("SEARCH_ENABLE_URL_PARSING")
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(true),
    };

    create_search_provider(config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_backend_from_str() {
        assert_eq!(SearchBackend::Serper, "serper".parse().unwrap());
        assert_eq!(SearchBackend::None, "none".parse().unwrap());
        assert_eq!(SearchBackend::SearXNG, "searxng".parse().unwrap());

        // Case insensitive
        assert_eq!(SearchBackend::Serper, "SERPER".parse().unwrap());
        assert_eq!(SearchBackend::None, "None".parse().unwrap());

        // Invalid
        assert!("invalid".parse::<SearchBackend>().is_err());
    }

    #[test]
    fn test_search_backend_display() {
        assert_eq!("serper", SearchBackend::Serper.to_string());
        assert_eq!("none", SearchBackend::None.to_string());
        assert_eq!("searxng", SearchBackend::SearXNG.to_string());
    }

    #[test]
    fn test_search_hit_builder() {
        let hit = SearchHit::new("https://example.com".to_string(), 1)
            .with_title("Example Title".to_string())
            .with_snippet("Example snippet".to_string())
            .with_metadata("source".to_string(), "test".to_string());

        assert_eq!(hit.url, "https://example.com");
        assert_eq!(hit.rank, 1);
        assert_eq!(hit.title, Some("Example Title".to_string()));
        assert_eq!(hit.snippet, Some("Example snippet".to_string()));
        assert_eq!(hit.metadata.get("source"), Some(&"test".to_string()));
    }

    #[test]
    fn test_search_config_default() {
        let config = SearchConfig::default();
        assert_eq!(config.backend, SearchBackend::Serper);
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.enable_url_parsing);
    }
}