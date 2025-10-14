use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

#[cfg(test)]
mod search_provider_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_search_provider_creation() {
        use riptide_search::{create_search_provider, SearchBackend, SearchConfig};

        // Test NoneProvider creation (no API key needed)
        let none_config = SearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: true,
        };

        let provider = create_search_provider(none_config).await;
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.backend_type(), SearchBackend::None);

        // Test health check
        let health_result = provider.health_check().await;
        assert!(health_result.is_ok());

        // Test SerperProvider creation with API key
        let serper_config = SearchConfig {
            backend: SearchBackend::Serper,
            api_key: Some("test_key".to_string()),
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: false,
        };

        let provider = create_search_provider(serper_config).await;
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().backend_type(), SearchBackend::Serper);

        // Test error case: SerperProvider without API key
        let invalid_config = SearchConfig {
            backend: SearchBackend::Serper,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: false,
        };

        let provider = create_search_provider(invalid_config).await;
        assert!(provider.is_err());
    }

    #[tokio::test]
    async fn test_none_provider_url_parsing() {
        use riptide_search::{NoneProvider, SearchProvider};

        let provider = NoneProvider::new(true);

        // Test URL parsing from query
        let results = provider.search("https://example.com", 10, "us", "en").await;
        assert!(results.is_ok());

        let search_results = results.unwrap();
        assert_eq!(search_results.len(), 1);
        assert_eq!(search_results[0].url, "https://example.com");
        assert_eq!(search_results[0].rank, 1);

        // Test multiple URLs
        let results = provider
            .search("https://example.com https://test.org", 10, "us", "en")
            .await;
        assert!(results.is_ok());

        let search_results = results.unwrap();
        assert_eq!(search_results.len(), 2);
        assert_eq!(search_results[0].url, "https://example.com");
        assert_eq!(search_results[1].url, "https://test.org");

        // Test non-URL query (should fail)
        let results = provider.search("not a url", 10, "us", "en").await;
        assert!(results.is_err());
    }

    #[tokio::test]
    async fn test_concurrent_search_requests() {
        use riptide_search::{NoneProvider, SearchProvider};

        let provider = Arc::new(NoneProvider::new(true));
        let queries = vec![
            "https://eventmesh.apache.org/",
            "https://github.com/apache/eventmesh",
            "https://example.com/api",
        ];

        let mut handles = vec![];

        for query in queries {
            let provider_clone = provider.clone();
            let query_owned = query.to_string();

            let handle =
                tokio::spawn(
                    async move { provider_clone.search(&query_owned, 10, "us", "en").await },
                );
            handles.push(handle);
        }

        let results = futures::future::join_all(handles).await;

        // All should succeed (URLs)
        for result in results {
            let task_result = result.expect("Task should complete");
            let search_result = task_result.expect("Search should succeed");
            assert_eq!(search_result.len(), 1); // Each query has one URL
        }
    }

    #[tokio::test]
    async fn test_search_provider_with_timeout() {
        use riptide_search::{NoneProvider, SearchProvider};

        // Test with reasonable timeout - NoneProvider should be very fast
        let provider = NoneProvider::new(true);
        let search_future = provider.search("https://example.com", 10, "us", "en");
        let result = timeout(Duration::from_secs(5), search_future).await;

        assert!(result.is_ok(), "Should complete within 5 seconds");
        let search_result = result.unwrap();
        assert!(search_result.is_ok(), "Search should succeed");
    }

    #[tokio::test]
    async fn test_search_result_consistency() {
        use riptide_search::{NoneProvider, SearchProvider};

        let provider = NoneProvider::new(true);
        let test_url = "https://eventmesh.apache.org/docs";

        let mut results = vec![];
        for _ in 0..3 {
            let result = provider.search(test_url, 10, "us", "en").await;
            assert!(result.is_ok());
            results.push(result.unwrap());
        }

        // All results should be identical
        for (i, result) in results.iter().enumerate() {
            assert_eq!(result.len(), 1, "Result {} has wrong length", i);
            assert_eq!(result[0].url, test_url, "Result {} has wrong URL", i);
            assert_eq!(result[0].rank, 1, "Result {} has wrong rank", i);
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_integration() {
        use riptide_search::{CircuitBreakerWrapper, NoneProvider, SearchProvider};

        let provider = Box::new(NoneProvider::new(true));
        let circuit_wrapped_provider = CircuitBreakerWrapper::new(provider);

        // Test that circuit breaker allows successful requests
        let result = circuit_wrapped_provider
            .search("https://example.com", 10, "us", "en")
            .await;
        assert!(result.is_ok());

        // Test backend type is preserved
        assert_eq!(
            circuit_wrapped_provider.backend_type(),
            riptide_search::SearchBackend::None
        );
    }
}
