//! Integration tests for riptide-search crate
//!
//! These tests verify that the search providers work correctly when integrated together.

use riptide_search::{
    create_search_provider, create_search_provider_from_env, NoneProvider, SearchBackend,
    SearchConfig, SearchHit, SearchProvider, SearchProviderFactory,
};
use std::env;

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test basic SearchConfig creation and defaults
    #[test]
    fn test_search_config_defaults() {
        let config = SearchConfig::default();

        assert_eq!(config.backend, SearchBackend::Serper);
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.enable_url_parsing);
        assert!(config.api_key.is_none());
        assert!(config.base_url.is_none());
    }

    /// Test SearchConfig with custom values
    #[test]
    fn test_search_config_custom() {
        let config = SearchConfig {
            backend: SearchBackend::None,
            api_key: Some("test-key".to_string()),
            base_url: Some("http://localhost:8080".to_string()),
            timeout_seconds: 60,
            enable_url_parsing: false,
        };

        assert_eq!(config.backend, SearchBackend::None);
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.base_url, Some("http://localhost:8080".to_string()));
        assert_eq!(config.timeout_seconds, 60);
        assert!(!config.enable_url_parsing);
    }

    /// Test SearchBackend string parsing and display
    #[test]
    fn test_search_backend_parsing() {
        // Test valid parsing
        assert_eq!(
            "serper".parse::<SearchBackend>().unwrap(),
            SearchBackend::Serper
        );
        assert_eq!(
            "none".parse::<SearchBackend>().unwrap(),
            SearchBackend::None
        );
        assert_eq!(
            "searxng".parse::<SearchBackend>().unwrap(),
            SearchBackend::SearXNG
        );

        // Test case insensitive
        assert_eq!(
            "SERPER".parse::<SearchBackend>().unwrap(),
            SearchBackend::Serper
        );
        assert_eq!(
            "None".parse::<SearchBackend>().unwrap(),
            SearchBackend::None
        );
        assert_eq!(
            "SearXNG".parse::<SearchBackend>().unwrap(),
            SearchBackend::SearXNG
        );

        // Test invalid parsing
        assert!("invalid_backend".parse::<SearchBackend>().is_err());
        assert!("".parse::<SearchBackend>().is_err());
        assert!("google".parse::<SearchBackend>().is_err());

        // Test display
        assert_eq!(SearchBackend::Serper.to_string(), "serper");
        assert_eq!(SearchBackend::None.to_string(), "none");
        assert_eq!(SearchBackend::SearXNG.to_string(), "searxng");
    }

    /// Test provider creation with None backend
    #[tokio::test]
    async fn test_create_none_provider() {
        let config = SearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: true,
        };

        let provider = create_search_provider(config).await;
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.backend_type(), SearchBackend::None);

        // Test health check
        assert!(provider.health_check().await.is_ok());
    }

    /// Test provider creation with Serper backend and API key
    #[tokio::test]
    async fn test_create_serper_provider_with_api_key() {
        let config = SearchConfig {
            backend: SearchBackend::Serper,
            api_key: Some("test-api-key".to_string()),
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: false,
        };

        let provider = create_search_provider(config).await;
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.backend_type(), SearchBackend::Serper);
    }

    /// Test provider creation with Serper backend but missing API key
    #[tokio::test]
    async fn test_create_serper_provider_missing_api_key() {
        let config = SearchConfig {
            backend: SearchBackend::Serper,
            api_key: None, // Missing API key should cause error
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: false,
        };

        let provider = create_search_provider(config).await;
        assert!(provider.is_err());

        let error = provider.err().unwrap();
        assert!(error.to_string().contains("API key is required"));
    }

    /// Test provider creation with SearXNG backend (not implemented)
    #[tokio::test]
    async fn test_create_searxng_provider_not_implemented() {
        let config = SearchConfig {
            backend: SearchBackend::SearXNG,
            api_key: None,
            base_url: Some("http://localhost:8080".to_string()),
            timeout_seconds: 30,
            enable_url_parsing: false,
        };

        let provider = create_search_provider(config).await;
        assert!(provider.is_err());

        let error = provider.err().unwrap();
        assert!(error.to_string().contains("not yet implemented"));
    }

    /// Test environment-based provider creation
    #[tokio::test]
    async fn test_create_provider_from_env() {
        // Test with None backend (no env vars needed)
        let provider = create_search_provider_from_env(SearchBackend::None).await;
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().backend_type(), SearchBackend::None);

        // Test with Serper backend and env var
        env::set_var("SERPER_API_KEY", "test-key-from-env");
        let provider = create_search_provider_from_env(SearchBackend::Serper).await;
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().backend_type(), SearchBackend::Serper);

        // Clean up
        env::remove_var("SERPER_API_KEY");
    }

    /// Test None provider URL extraction functionality
    #[tokio::test]
    async fn test_none_provider_url_extraction() {
        let provider = NoneProvider::new(true);

        // Test successful URL extraction
        let result = provider
            .search("https://example.com https://test.org", 10, "us", "en")
            .await;
        assert!(result.is_ok());

        let hits = result.unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].url, "https://example.com");
        assert_eq!(hits[1].url, "https://test.org");
        assert_eq!(hits[0].rank, 1);
        assert_eq!(hits[1].rank, 2);

        // Test no URLs found
        let result = provider.search("no urls here", 10, "us", "en").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No URLs found"));
    }

    /// Test SearchHit builder pattern
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

    /// Test SearchProviderFactory
    #[tokio::test]
    async fn test_factory_create_with_backend() {
        // Test None backend creation
        let provider = SearchProviderFactory::create_with_backend(SearchBackend::None).await;
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().backend_type(), SearchBackend::None);
    }
}
