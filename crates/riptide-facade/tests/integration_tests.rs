//! Integration tests for riptide-facade crate.

use riptide_facade::prelude::*;
use std::time::Duration;

#[tokio::test]
async fn test_builder_basic_creation() {
    let scraper = Riptide::builder()
        .user_agent("TestBot/1.0")
        .build_scraper()
        .await;

    assert!(scraper.is_ok());
    let scraper = scraper.unwrap();
    assert_eq!(scraper.config().user_agent, "TestBot/1.0");
}

#[tokio::test]
async fn test_builder_with_timeout() {
    let scraper = Riptide::builder()
        .timeout_secs(60)
        .build_scraper()
        .await
        .unwrap();

    assert_eq!(scraper.config().timeout, Duration::from_secs(60));
}

#[tokio::test]
async fn test_builder_with_custom_config() {
    let config = RiptideConfig::new()
        .with_user_agent("CustomBot/2.0")
        .with_timeout(Duration::from_secs(45))
        .with_max_redirects(10);

    let scraper = Riptide::builder()
        .config(config)
        .build_scraper()
        .await
        .unwrap();

    assert_eq!(scraper.config().user_agent, "CustomBot/2.0");
    assert_eq!(scraper.config().timeout, Duration::from_secs(45));
    assert_eq!(scraper.config().max_redirects, 10);
}

#[tokio::test]
async fn test_builder_with_headers() {
    let scraper = Riptide::builder()
        .header("X-API-Key", "secret123")
        .header("X-Custom", "value")
        .build_scraper()
        .await
        .unwrap();

    let headers = &scraper.config().headers;
    assert_eq!(headers.len(), 2);
    assert!(headers.contains(&("X-API-Key".to_string(), "secret123".to_string())));
}

#[tokio::test]
async fn test_builder_chained_configuration() {
    let scraper = Riptide::builder()
        .user_agent("ChainBot/1.0")
        .timeout_secs(30)
        .max_redirects(5)
        .verify_ssl(false)
        .max_body_size(1024 * 1024)
        .build_scraper()
        .await
        .unwrap();

    let config = scraper.config();
    assert_eq!(config.user_agent, "ChainBot/1.0");
    assert_eq!(config.timeout, Duration::from_secs(30));
    assert_eq!(config.max_redirects, 5);
    assert!(!config.verify_ssl);
    assert_eq!(config.max_body_size, 1024 * 1024);
}

#[tokio::test]
async fn test_scraper_facade_invalid_url() {
    let scraper = Riptide::builder().build_scraper().await.unwrap();

    let result = scraper.fetch_html("not a valid url").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RiptideError::InvalidUrl(_)));
}

#[tokio::test]
async fn test_scraper_facade_clone() {
    let scraper1 = Riptide::builder()
        .user_agent("CloneBot/1.0")
        .build_scraper()
        .await
        .unwrap();

    let scraper2 = scraper1.clone();
    assert_eq!(scraper1.config().user_agent, scraper2.config().user_agent);
}

#[tokio::test]
async fn test_config_validation_empty_user_agent() {
    let config = RiptideConfig {
        user_agent: String::new(),
        ..Default::default()
    };

    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("User agent"));
}

#[tokio::test]
async fn test_config_validation_zero_timeout() {
    let config = RiptideConfig {
        timeout: Duration::from_secs(0),
        ..Default::default()
    };

    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Timeout"));
}

#[tokio::test]
async fn test_error_types() {
    let config_err = RiptideError::config("test config error");
    assert!(matches!(config_err, RiptideError::Config(_)));

    let extraction_err = RiptideError::extraction("test extraction error");
    assert!(matches!(extraction_err, RiptideError::Extraction(_)));
}
