//! TDD tests for SearchProvider implementation
//! Following London School TDD with mock collaborations

use anyhow::Result;
use mockall::predicate::*;
use riptide_search::{
    create_search_provider, SearchBackend, SearchConfig, SearchHit, SearchProvider,
};

#[tokio::test]
async fn test_serper_provider_requires_api_key() {
    let config = SearchConfig {
        backend: SearchBackend::Serper,
        api_key: None,
        ..Default::default()
    };

    let result = create_search_provider(config).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("API key is required"));
}

#[tokio::test]
async fn test_none_provider_parses_urls_from_query() {
    let config = SearchConfig {
        backend: SearchBackend::None,
        enable_url_parsing: true,
        ..Default::default()
    };

    let provider = create_search_provider(config)
        .await
        .expect("Should create None provider without API key");

    // Test URL parsing from space-separated query
    let query = "https://example.com https://test.org http://localhost:8080";
    let results = provider
        .search(query, 10, "us", "en")
        .await
        .expect("Should parse URLs from query");

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].url, "https://example.com");
    assert_eq!(results[1].url, "https://test.org");
    assert_eq!(results[2].url, "http://localhost:8080");
}

#[tokio::test]
async fn test_none_provider_handles_comma_separated_urls() {
    let config = SearchConfig {
        backend: SearchBackend::None,
        enable_url_parsing: true,
        ..Default::default()
    };

    let provider = create_search_provider(config)
        .await
        .expect("Should create None provider");

    let query = "https://example.com,https://test.org";
    let results = provider
        .search(query, 10, "us", "en")
        .await
        .expect("Should parse comma-separated URLs");

    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_none_provider_returns_empty_for_no_urls() {
    let config = SearchConfig {
        backend: SearchBackend::None,
        enable_url_parsing: true,
        ..Default::default()
    };

    let provider = create_search_provider(config)
        .await
        .expect("Should create None provider");

    let query = "this is just text without any urls";
    let results = provider.search(query, 10, "us", "en").await;

    // Should return error with helpful message
    assert!(results.is_err());
    assert!(results.unwrap_err().to_string().contains("No URLs found"));
}

#[tokio::test]
async fn test_circuit_breaker_wraps_provider() {
    let config = SearchConfig {
        backend: SearchBackend::None,
        enable_url_parsing: true,
        ..Default::default()
    };

    let provider = create_search_provider(config)
        .await
        .expect("Should create wrapped provider");

    // Provider should be wrapped with circuit breaker
    // First call should work
    let query = "https://example.com";
    let results = provider.search(query, 10, "us", "en").await;
    assert!(results.is_ok());
}

#[tokio::test]
async fn test_search_hit_builder_pattern() {
    let hit = SearchHit::new("https://example.com".to_string(), 1)
        .with_title("Example Title".to_string())
        .with_snippet("Example snippet".to_string())
        .with_metadata("source".to_string(), "test".to_string());

    assert_eq!(hit.url, "https://example.com");
    assert_eq!(hit.rank, 1);
    assert_eq!(hit.title.unwrap(), "Example Title");
    assert_eq!(hit.snippet.unwrap(), "Example snippet");
    assert_eq!(hit.metadata.get("source").unwrap(), "test");
}

#[tokio::test]
async fn test_provider_health_check() {
    let config = SearchConfig {
        backend: SearchBackend::None,
        enable_url_parsing: true,
        ..Default::default()
    };

    let provider = create_search_provider(config)
        .await
        .expect("Should create provider");

    let health = provider.health_check().await;
    assert!(health.is_ok(), "Health check should pass for None provider");
}
