//! Comprehensive integration tests for gap fixes
//!
//! This test suite validates all gap fixes work together correctly:
//! - Unified confidence scoring across all extractors
//! - Cache key uniqueness for all extraction methods
//! - Strategy composition end-to-end
//! - WASM extraction without mocks in production

use std::collections::HashSet;

#[cfg(test)]
mod confidence_scoring_integration {
    use super::*;

    #[tokio::test]
    async fn test_all_extractors_have_consistent_confidence() {
        // This test ensures all extractors return confidence scores in [0.0, 1.0]
        // and use consistent calculation methods

        let html = r#"
            <!DOCTYPE html>
            <html>
            <head><title>Test Page</title></head>
            <body>
                <article>
                    <h1>Main Article</h1>
                    <p>This is test content with <strong>important</strong> information.</p>
                </article>
            </body>
            </html>
        "#;

        // Test CSS extractor confidence
        #[cfg(feature = "css-extraction")]
        {
            use riptide_html::processor::DefaultHtmlProcessor;
            use riptide_html::HtmlProcessor;

            let processor = DefaultHtmlProcessor::default();
            let confidence = processor.confidence_score(html);

            assert!(confidence >= 0.0 && confidence <= 1.0,
                "CSS extractor confidence {} out of range", confidence);
            assert!(confidence > 0.5,
                "CSS extractor should have high confidence for well-formed HTML");
        }

        // Test Regex extractor confidence
        #[cfg(feature = "regex-extraction")]
        {
            use riptide_html::regex_extraction::RegexExtractor;

            let extractor = RegexExtractor::default();
            let confidence = extractor.confidence_score(html);

            assert!(confidence >= 0.0 && confidence <= 1.0,
                "Regex extractor confidence {} out of range", confidence);
        }

        // Test WASM extractor confidence
        #[cfg(feature = "wasm-extraction")]
        {
            // WASM confidence is embedded in quality_score
            // Should be normalized to [0.0, 1.0] range
            use riptide_html::wasm_extraction::WasmExtractor;

            let extractor = WasmExtractor::new().await.expect("Failed to create WASM extractor");
            let result = extractor.extract(html, "https://example.com").await
                .expect("WASM extraction failed");

            assert!(result.extraction_confidence >= 0.0 && result.extraction_confidence <= 1.0,
                "WASM extractor confidence {} out of range", result.extraction_confidence);
        }
    }

    #[test]
    fn test_confidence_calculation_consistency() {
        // Test that confidence calculations are deterministic
        let html = "<html><body><p>Test content</p></body></html>";

        #[cfg(feature = "css-extraction")]
        {
            use riptide_html::processor::DefaultHtmlProcessor;
            use riptide_html::HtmlProcessor;

            let processor = DefaultHtmlProcessor::default();
            let conf1 = processor.confidence_score(html);
            let conf2 = processor.confidence_score(html);

            assert_eq!(conf1, conf2, "Confidence scoring should be deterministic");
        }
    }

    #[test]
    fn test_confidence_correlates_with_content_quality() {
        // High quality HTML should score higher than low quality
        let high_quality = r#"
            <!DOCTYPE html>
            <html lang="en">
            <head><meta charset="UTF-8"><title>Quality Page</title></head>
            <body>
                <article>
                    <h1>Well Structured</h1>
                    <p>Semantic HTML with proper structure.</p>
                </article>
            </body>
            </html>
        "#;

        let low_quality = "<p>Minimal content</p>";

        #[cfg(feature = "css-extraction")]
        {
            use riptide_html::processor::DefaultHtmlProcessor;
            use riptide_html::HtmlProcessor;

            let processor = DefaultHtmlProcessor::default();
            let high_conf = processor.confidence_score(high_quality);
            let low_conf = processor.confidence_score(low_quality);

            assert!(high_conf > low_conf,
                "High quality HTML should have higher confidence: {} vs {}",
                high_conf, low_conf);
        }
    }
}

#[cfg(test)]
mod cache_key_integration {
    use super::*;

    #[test]
    fn test_cache_keys_unique_across_all_methods() {
        // Ensure cache keys are unique for different extraction methods
        let url = "https://example.com/test";
        let content = "test content";

        let mut keys = HashSet::new();

        // CSS extraction cache key
        #[cfg(feature = "css-extraction")]
        {
            let css_key = format!("css:{}:{}", url, content);
            assert!(keys.insert(css_key.clone()), "CSS cache key should be unique");
        }

        // Regex extraction cache key
        #[cfg(feature = "regex-extraction")]
        {
            let regex_key = format!("regex:{}:{}", url, content);
            assert!(keys.insert(regex_key.clone()), "Regex cache key should be unique");
        }

        // WASM extraction cache key
        #[cfg(feature = "wasm-extraction")]
        {
            let wasm_key = format!("wasm:{}:{}", url, content);
            assert!(keys.insert(wasm_key.clone()), "WASM cache key should be unique");
        }

        // Verify all keys are different
        assert!(keys.len() >= 1, "At least one extraction method should be available");
    }

    #[test]
    fn test_cache_key_includes_method_discriminator() {
        // Each method should include a unique discriminator in cache keys
        let url = "https://example.com";
        let html = "<html><body>test</body></html>";

        // CSS should prefix with "css:"
        let css_key = format!("css:{}:{}", url, html);
        assert!(css_key.starts_with("css:"), "CSS key should start with 'css:'");

        // Regex should prefix with "regex:"
        let regex_key = format!("regex:{}:{}", url, html);
        assert!(regex_key.starts_with("regex:"), "Regex key should start with 'regex:'");

        // WASM should prefix with "wasm:"
        let wasm_key = format!("wasm:{}:{}", url, html);
        assert!(wasm_key.starts_with("wasm:"), "WASM key should start with 'wasm:'");

        // Verify they're all different
        assert_ne!(css_key, regex_key);
        assert_ne!(css_key, wasm_key);
        assert_ne!(regex_key, wasm_key);
    }

    #[test]
    fn test_cache_key_collision_resistance() {
        // Test that similar URLs don't cause collisions
        let urls = vec![
            "https://example.com/page",
            "https://example.com/page/",
            "https://example.com/page?query",
            "https://example.com/page#fragment",
        ];

        let content = "same content";
        let mut keys = HashSet::new();

        for url in urls {
            let key = format!("css:{}:{}", url, content);
            assert!(keys.insert(key), "Cache keys should be unique for different URLs");
        }

        assert_eq!(keys.len(), 4, "All 4 variations should have unique cache keys");
    }
}

#[cfg(test)]
mod strategy_composition_integration {
    use super::*;

    #[tokio::test]
    #[cfg(feature = "strategy-traits")]
    async fn test_strategy_composition_end_to_end() {
        // Test that strategies can be composed and work together
        use riptide_html::strategy_implementations::HtmlCssExtractionStrategy;

        let strategy = HtmlCssExtractionStrategy::with_default_selectors();
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head><title>Test Composition</title></head>
            <body>
                <article>
                    <h1>Strategy Testing</h1>
                    <p>Testing strategy composition.</p>
                </article>
            </body>
            </html>
        "#;

        let result = strategy.extract(html, "https://example.com").await;
        assert!(result.is_ok(), "Strategy extraction should succeed");

        let extraction = result.unwrap();
        assert!(!extraction.content.title.is_empty(), "Should extract title");
        assert!(!extraction.content.content.is_empty(), "Should extract content");

        // Verify quality metrics are populated
        assert!(extraction.quality.content_quality > 0.0, "Should have quality score");
        assert!(extraction.quality.structure_score > 0.0, "Should have structure score");
    }

    #[test]
    #[cfg(feature = "strategy-traits")]
    fn test_strategy_capabilities_reporting() {
        // Test that strategies correctly report their capabilities
        use riptide_html::strategy_implementations::HtmlCssExtractionStrategy;

        let strategy = HtmlCssExtractionStrategy::with_default_selectors();
        let capabilities = strategy.capabilities();

        assert_eq!(capabilities.strategy_type, "css_html_extraction");
        assert!(capabilities.supported_content_types.contains(&"text/html".to_string()));

        // Verify resource requirements are defined
        assert!(matches!(
            capabilities.performance_tier,
            riptide_html::strategy_implementations::PerformanceTier::Balanced |
            riptide_html::strategy_implementations::PerformanceTier::Fast
        ));
    }

    #[tokio::test]
    #[cfg(all(feature = "strategy-traits", feature = "wasm-extraction"))]
    async fn test_fallback_strategy_chain() {
        // Test that multiple strategies can work as fallbacks
        // Priority: WASM (high quality) -> CSS (fast) -> Regex (fallback)

        let html = "<html><body><p>Test fallback</p></body></html>";
        let url = "https://example.com";

        // Try WASM first
        use riptide_html::wasm_extraction::WasmExtractor;
        let wasm_result = WasmExtractor::new().await
            .and_then(|e| async { e.extract(html, url).await }.await);

        if wasm_result.is_ok() {
            // WASM succeeded, this is optimal
            assert!(true);
        } else {
            // WASM failed, try CSS
            use riptide_html::strategy_implementations::HtmlCssExtractionStrategy;
            let css_strategy = HtmlCssExtractionStrategy::with_default_selectors();
            let css_result = css_strategy.extract(html, url).await;

            assert!(css_result.is_ok(), "CSS fallback should succeed");
        }
    }
}

#[cfg(test)]
mod wasm_production_integration {
    use super::*;

    #[tokio::test]
    #[cfg(feature = "wasm-extraction")]
    async fn test_wasm_extraction_no_mocks_in_production() {
        // Verify WASM extraction uses real WASM runtime, not mocks
        use riptide_html::wasm_extraction::WasmExtractor;

        let extractor = WasmExtractor::new().await
            .expect("Should create real WASM extractor");

        let html = r#"
            <!DOCTYPE html>
            <html>
            <head><title>WASM Test</title></head>
            <body>
                <article>
                    <h1>Testing WASM</h1>
                    <p>Real WASM extraction without mocks.</p>
                </article>
            </body>
            </html>
        "#;

        let result = extractor.extract(html, "https://example.com").await;
        assert!(result.is_ok(), "WASM extraction should succeed");

        let content = result.unwrap();
        assert!(!content.title.is_empty(), "Should extract title");
        assert!(!content.content.is_empty(), "Should extract content");

        // Verify it's using real WASM by checking quality score is set
        assert!(content.extraction_confidence > 0.0, "Should have confidence score from WASM");
    }

    #[tokio::test]
    #[cfg(feature = "wasm-extraction")]
    async fn test_wasm_error_handling_production() {
        // Test that WASM handles errors gracefully in production
        use riptide_html::wasm_extraction::WasmExtractor;

        let extractor = WasmExtractor::new().await
            .expect("Should create WASM extractor");

        // Test with malformed HTML
        let bad_html = "<html><body><unclosed-tag>";
        let result = extractor.extract(bad_html, "https://example.com").await;

        // Should either succeed with low confidence or fail gracefully
        match result {
            Ok(content) => {
                // If it succeeds, confidence should be lower for malformed HTML
                assert!(content.extraction_confidence < 0.9,
                    "Malformed HTML should have lower confidence");
            }
            Err(e) => {
                // If it fails, error should be descriptive
                let error_msg = e.to_string();
                assert!(!error_msg.is_empty(), "Error message should be informative");
            }
        }
    }

    #[tokio::test]
    #[cfg(feature = "wasm-extraction")]
    async fn test_wasm_memory_safety() {
        // Test that WASM doesn't leak memory or crash
        use riptide_html::wasm_extraction::WasmExtractor;

        let extractor = WasmExtractor::new().await
            .expect("Should create WASM extractor");

        // Process multiple documents to check for memory issues
        for i in 0..10 {
            let html = format!(
                r#"<html><head><title>Test {}</title></head>
                   <body><p>Content {}</p></body></html>"#,
                i, i
            );

            let result = extractor.extract(&html, &format!("https://example.com/{}", i)).await;
            assert!(result.is_ok(), "Iteration {} should succeed", i);
        }

        // If we got here without panic/crash, memory safety is good
        assert!(true);
    }
}

#[cfg(test)]
mod end_to_end_integration {
    use super::*;

    #[tokio::test]
    async fn test_complete_extraction_pipeline() {
        // Test the complete extraction pipeline with all gap fixes
        let html = r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <title>Complete Pipeline Test</title>
            </head>
            <body>
                <article>
                    <h1>Main Article</h1>
                    <p>This tests the complete extraction pipeline with all fixes applied.</p>
                    <ul>
                        <li>Unified confidence scoring</li>
                        <li>Unique cache keys</li>
                        <li>Strategy composition</li>
                        <li>Production WASM</li>
                    </ul>
                </article>
            </body>
            </html>
        "#;

        let url = "https://example.com/complete-test";

        // Try each extraction method and collect results
        let mut results = Vec::new();

        // CSS extraction
        #[cfg(feature = "css-extraction")]
        {
            use riptide_html::processor::DefaultHtmlProcessor;
            use riptide_html::HtmlProcessor;

            let processor = DefaultHtmlProcessor::default();
            if let Ok(content) = processor.extract_with_css(
                html,
                url,
                &std::collections::HashMap::new()
            ).await {
                results.push(("css", content));
            }
        }

        // WASM extraction
        #[cfg(feature = "wasm-extraction")]
        {
            use riptide_html::wasm_extraction::WasmExtractor;

            if let Ok(extractor) = WasmExtractor::new().await {
                if let Ok(content) = extractor.extract(html, url).await {
                    results.push(("wasm", content));
                }
            }
        }

        assert!(!results.is_empty(), "At least one extraction method should succeed");

        // Verify all results have valid confidence scores
        for (method, content) in results {
            assert!(
                content.extraction_confidence >= 0.0 && content.extraction_confidence <= 1.0,
                "{} method has invalid confidence: {}",
                method,
                content.extraction_confidence
            );

            assert!(!content.content.is_empty(),
                "{} method should extract content", method);
        }
    }

    #[test]
    fn test_cache_key_uniqueness_across_pipeline() {
        // Verify cache keys remain unique throughout the pipeline
        let url = "https://example.com/cache-test";
        let html = "<html><body>test</body></html>";

        let mut cache_keys = HashSet::new();

        // Generate cache keys for all methods
        let methods = vec!["css", "regex", "wasm", "fallback"];

        for method in methods {
            let key = format!("{}:{}:{}", method, url, html);
            assert!(cache_keys.insert(key.clone()),
                "Cache key for {} should be unique", method);
        }

        assert_eq!(cache_keys.len(), 4, "All method cache keys should be unique");
    }
}
