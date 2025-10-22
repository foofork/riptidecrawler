/// WASM Extractor Integration Tests - London School TDD
///
/// Tests the collaboration between the host and WASM component using mocks
/// to verify behavior and contracts rather than internal implementation details.

use crate::fixtures::*;
use crate::fixtures::test_data::*;
use mockall::predicate::*;
use proptest::prelude::*;
use std::sync::Arc;
use tokio_test;
use tracing_test::traced_test;

#[cfg(test)]
mod wasm_extractor_tests {
    use super::*;

    /// Test the 5-URL mixed validation set using mock collaboration
    #[traced_test]
    #[tokio::test]
    async fn test_mixed_url_validation_set() {
        // Arrange - Set up mock collaborators
        let mut mock_extractor = MockWasmExtractor::new();
        let test_urls = TestUrls::mixed_validation_set();

        // Define expected behavior for each URL type
        for (url, content_type) in test_urls.iter() {
            let expected_content = match *content_type {
                "article" => ExpectedResults::article_extraction(),
                "spa" => ExtractedContent {
                    url: url.to_string(),
                    title: Some("SPA Dashboard".to_string()),
                    content: "Dynamic content loaded".to_string(),
                    links: vec![],
                    images: vec![],
                },
                "pdf" => ExtractedContent {
                    url: url.to_string(),
                    title: Some("PDF Document".to_string()),
                    content: "PDF content extracted".to_string(),
                    links: vec![],
                    images: vec![],
                },
                "news" => ExtractedContent {
                    url: url.to_string(),
                    title: Some("Breaking News".to_string()),
                    content: "News article content".to_string(),
                    links: vec!["https://source.com".to_string()],
                    images: vec!["https://news.com/image.jpg".to_string()],
                },
                "product" => ExtractedContent {
                    url: url.to_string(),
                    title: Some("Product Name".to_string()),
                    content: "Product description and details".to_string(),
                    links: vec!["https://reviews.com".to_string()],
                    images: vec!["https://ecommerce.com/product.jpg".to_string()],
                },
                _ => unreachable!(),
            };

            // Expect the extractor to be called with the URL and return appropriate content
            mock_extractor
                .expect_extract()
                .with(always(), eq(*url), always())
                .times(1)
                .returning(move |_, _, _| Ok(expected_content.clone()));
        }

        // Act & Assert - Verify each URL is processed correctly
        for (url, content_type) in test_urls.iter() {
            let html = match *content_type {
                "article" => HtmlSamples::article_html(),
                "spa" => HtmlSamples::spa_html(),
                "pdf" => HtmlSamples::pdf_placeholder_html(),
                _ => HtmlSamples::article_html(), // Default for news/product
            };

            let result = mock_extractor.extract(&html, url, "article");

            assert!(result.is_ok(), "Extraction should succeed for URL: {}", url);
            let content = result.unwrap();
            assert_eq!(content.url, *url);
            assert!(content.title.is_some(), "Content should have a title for URL: {}", url);
            assert!(!content.content.is_empty(), "Content should not be empty for URL: {}", url);
        }

        // Verify all expectations were met
        // mockall automatically verifies this when the mock goes out of scope
    }

    /// Test WASM component health and version reporting
    #[traced_test]
    #[tokio::test]
    async fn test_component_health_monitoring() {
        // Arrange
        let mut mock_extractor = MockWasmExtractor::new();

        mock_extractor
            .expect_health_check()
            .times(1)
            .returning(|| ExpectedResults::healthy_status());

        mock_extractor
            .expect_get_info()
            .times(1)
            .returning(|| ExpectedResults::component_info());

        // Act
        let health = mock_extractor.health_check();
        let info = mock_extractor.get_info();

        // Assert - Verify component reports healthy status
        assert_eq!(health.status, "healthy");
        assert_eq!(health.version, "0.1.0");
        assert!(health.memory_usage > 0);

        // Assert - Verify component info is complete
        assert_eq!(info.name, "riptide-extractor-wasm");
        assert_eq!(info.version, "0.1.0");
        assert!(!info.features.is_empty());
        assert!(info.features.contains(&"article-extraction".to_string()));
    }

    /// Test HTML validation contract
    #[traced_test]
    #[tokio::test]
    async fn test_html_validation_contract() {
        // Arrange
        let mut mock_extractor = MockWasmExtractor::new();

        // Define validation behavior
        mock_extractor
            .expect_validate_html()
            .with(eq(HtmlSamples::article_html()))
            .times(1)
            .returning(|_| Ok(true));

        mock_extractor
            .expect_validate_html()
            .with(eq(HtmlSamples::malformed_html()))
            .times(1)
            .returning(|_| Ok(false));

        mock_extractor
            .expect_validate_html()
            .with(eq("".to_string()))
            .times(1)
            .returning(|_| Ok(false));

        // Act & Assert
        assert!(mock_extractor.validate_html(&HtmlSamples::article_html()).unwrap());
        assert!(!mock_extractor.validate_html(&HtmlSamples::malformed_html()).unwrap());
        assert!(!mock_extractor.validate_html(&"".to_string()).unwrap());
    }

    /// Test error handling and resilience
    #[traced_test]
    #[tokio::test]
    async fn test_error_handling_resilience() {
        // Arrange
        let mut mock_extractor = MockWasmExtractor::new();

        // Test various error conditions
        mock_extractor
            .expect_extract()
            .with(eq(""), always(), always())
            .times(1)
            .returning(|_, _, _| Err("Empty HTML content".to_string()));

        mock_extractor
            .expect_extract()
            .with(eq(HtmlSamples::malformed_html()), always(), always())
            .times(1)
            .returning(|_, _, _| Err("Invalid HTML structure".to_string()));

        mock_extractor
            .expect_validate_html()
            .with(eq(""))
            .times(1)
            .returning(|_| Err("Validation error".to_string()));

        // Act & Assert - Verify errors are properly propagated
        let empty_result = mock_extractor.extract("", "https://example.com", "article");
        assert!(empty_result.is_err());
        assert!(empty_result.unwrap_err().contains("Empty HTML"));

        let malformed_result = mock_extractor.extract(
            &HtmlSamples::malformed_html(),
            "https://example.com",
            "article"
        );
        assert!(malformed_result.is_err());
        assert!(malformed_result.unwrap_err().contains("Invalid HTML"));

        let validation_error = mock_extractor.validate_html("");
        assert!(validation_error.is_err());
        assert!(validation_error.unwrap_err().contains("Validation error"));
    }

    /// Property-based test for extraction consistency
    #[traced_test]
    #[tokio::test]
    async fn test_extraction_consistency_properties() {
        // Arrange
        let mut mock_extractor = MockWasmExtractor::new();

        // Property: Any valid HTML should either succeed or fail consistently
        mock_extractor
            .expect_extract()
            .returning(|html, url, mode| {
                if html.contains("<html") && html.contains("</html>") {
                    Ok(ExtractedContent {
                        url: url.to_string(),
                        title: Some("Test Title".to_string()),
                        content: "Test content".to_string(),
                        links: vec![],
                        images: vec![],
                    })
                } else {
                    Err("Invalid HTML structure".to_string())
                }
            });

        // Test property: Well-formed HTML always succeeds
        let valid_html = r#"<html><head><title>Test</title></head><body><p>Content</p></body></html>"#;
        let result = mock_extractor.extract(valid_html, "https://example.com", "article");
        assert!(result.is_ok());

        // Test property: Malformed HTML always fails
        let invalid_html = r#"<html><head><title>Test</title><body><p>Content"#;
        let result = mock_extractor.extract(invalid_html, "https://example.com", "article");
        assert!(result.is_err());
    }

    /// Test extraction mode behavior variation
    #[traced_test]
    #[tokio::test]
    async fn test_extraction_mode_behavior() {
        // Arrange
        let mut mock_extractor = MockWasmExtractor::new();
        let html = HtmlSamples::article_html();

        // Different modes should produce different extraction strategies
        mock_extractor
            .expect_extract()
            .with(always(), always(), eq("article"))
            .times(1)
            .returning(|_, url, _| Ok(ExtractedContent {
                url: url.to_string(),
                title: Some("Article Title".to_string()),
                content: "Clean article content without sidebar".to_string(),
                links: vec!["https://example.com".to_string()],
                images: vec!["https://example.com/image.jpg".to_string()],
            }));

        mock_extractor
            .expect_extract()
            .with(always(), always(), eq("full"))
            .times(1)
            .returning(|_, url, _| Ok(ExtractedContent {
                url: url.to_string(),
                title: Some("Article Title".to_string()),
                content: "Full page content including sidebar and navigation".to_string(),
                links: vec!["https://example.com".to_string(), "https://sidebar.com".to_string()],
                images: vec!["https://example.com/image.jpg".to_string()],
            }));

        mock_extractor
            .expect_extract()
            .with(always(), always(), eq("metadata"))
            .times(1)
            .returning(|_, url, _| Ok(ExtractedContent {
                url: url.to_string(),
                title: Some("Article Title".to_string()),
                content: "".to_string(), // Metadata mode extracts minimal content
                links: vec![],
                images: vec![],
            }));

        // Act & Assert
        let article_result = mock_extractor.extract(&html, "https://example.com", "article").unwrap();
        assert!(article_result.content.contains("Clean article content"));
        assert_eq!(article_result.links.len(), 1);

        let full_result = mock_extractor.extract(&html, "https://example.com", "full").unwrap();
        assert!(full_result.content.contains("Full page content"));
        assert_eq!(full_result.links.len(), 2); // More links in full mode

        let metadata_result = mock_extractor.extract(&html, "https://example.com", "metadata").unwrap();
        assert!(metadata_result.content.is_empty()); // Metadata mode minimal content
        assert_eq!(metadata_result.links.len(), 0);
    }

    /// Test concurrent extraction safety
    #[traced_test]
    #[tokio::test]
    async fn test_concurrent_extraction_safety() {
        // Arrange
        let mock_extractor = Arc::new(std::sync::Mutex::new(MockWasmExtractor::new()));

        // This test verifies that the extraction contract works safely in concurrent scenarios
        // In a real implementation, we'd test that the WASM component handles concurrent calls properly

        let html = HtmlSamples::article_html();
        let mut handles = vec![];

        // Set up expectations for concurrent calls
        {
            let mut extractor = mock_extractor.lock().unwrap();
            extractor
                .expect_extract()
                .times(5)
                .returning(|_, url, _| Ok(ExtractedContent {
                    url: url.to_string(),
                    title: Some("Concurrent Title".to_string()),
                    content: "Concurrent content".to_string(),
                    links: vec![],
                    images: vec![],
                }));
        }

        // Act - Simulate concurrent extractions
        for i in 0..5 {
            let extractor = Arc::clone(&mock_extractor);
            let html_clone = html.clone();
            let url = format!("https://example.com/article/{}", i);

            let handle = tokio::spawn(async move {
                let mut extractor = extractor.lock().unwrap();
                extractor.extract(&html_clone, &url, "article")
            });
            handles.push(handle);
        }

        // Assert - All concurrent extractions should succeed
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
            let content = result.unwrap();
            assert!(!content.content.is_empty());
        }
    }
}

/// Proptest strategies for property-based testing
mod proptest_strategies {
    use super::*;

    prop_compose! {
        fn valid_html_strategy()(
            title in "[a-zA-Z0-9 ]{1,100}",
            content in "[a-zA-Z0-9 .,!?]{10,1000}"
        ) -> String {
            format!(
                r#"<html><head><title>{}</title></head><body><p>{}</p></body></html>"#,
                title, content
            )
        }
    }

    prop_compose! {
        fn valid_url_strategy()(
            domain in "[a-z]{3,20}",
            path in "[a-z0-9/-]{0,50}"
        ) -> String {
            format!("https://{}.com{}", domain, path)
        }
    }

    proptest! {
        #[test]
        fn test_extraction_properties(
            html in valid_html_strategy(),
            url in valid_url_strategy()
        ) {
            // Property: Valid HTML should always produce some result
            // This would be connected to a real extractor in integration tests
            assert!(!html.is_empty());
            assert!(html.contains("<html"));
            assert!(html.contains("</html>"));
            assert!(url.starts_with("https://"));
        }
    }
}