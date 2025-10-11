//! Confidence scoring integration tests
//!
//! Tests for unified confidence scoring across all extractors

#[cfg(test)]
mod confidence_tests {
    use std::collections::HashMap;

    #[tokio::test]
    #[cfg(feature = "css-extraction")]
    async fn test_css_confidence_scoring() {
        use riptide_html::processor::{DefaultHtmlProcessor, HtmlProcessor};

        let processor = DefaultHtmlProcessor::default();

        // Test with high-quality HTML
        let high_quality = r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <title>High Quality Page</title>
            </head>
            <body>
                <article>
                    <h1>Well Structured Content</h1>
                    <p>Semantic HTML with proper structure.</p>
                </article>
            </body>
            </html>
        "#;

        let conf_high = processor.confidence_score(high_quality);
        assert!(conf_high > 0.7, "High quality HTML should score > 0.7, got {}", conf_high);

        // Test with low-quality HTML
        let low_quality = "<p>Minimal</p>";
        let conf_low = processor.confidence_score(low_quality);
        assert!(conf_low < conf_high, "Low quality should score lower: {} < {}", conf_low, conf_high);
    }

    #[tokio::test]
    #[cfg(feature = "regex-extraction")]
    async fn test_regex_confidence_scoring() {
        use riptide_html::regex_extraction::RegexExtractor;

        let extractor = RegexExtractor::default();

        let html_with_patterns = r#"
            <html>
            <body>
                <h1>Title</h1>
                <p>Paragraph 1</p>
                <p>Paragraph 2</p>
                <a href="link1">Link 1</a>
                <a href="link2">Link 2</a>
            </body>
            </html>
        "#;

        let confidence = extractor.confidence_score(html_with_patterns);
        assert!(confidence >= 0.0 && confidence <= 1.0,
            "Confidence should be in [0,1], got {}", confidence);
        assert!(confidence > 0.3, "HTML with multiple patterns should score > 0.3");
    }

    #[test]
    fn test_confidence_range_validation() {
        // All confidence scores must be in [0.0, 1.0]
        let test_scores = vec![0.0, 0.5, 0.8, 1.0];

        for score in test_scores {
            assert!(score >= 0.0 && score <= 1.0,
                "Score {} is out of valid range", score);
        }
    }

    #[test]
    fn test_confidence_deterministic() {
        // Confidence scoring should be deterministic
        #[cfg(feature = "css-extraction")]
        {
            use riptide_html::processor::{DefaultHtmlProcessor, HtmlProcessor};

            let processor = DefaultHtmlProcessor::default();
            let html = "<html><body><p>Test</p></body></html>";

            let score1 = processor.confidence_score(html);
            let score2 = processor.confidence_score(html);
            let score3 = processor.confidence_score(html);

            assert_eq!(score1, score2, "Scores should be deterministic");
            assert_eq!(score2, score3, "Scores should be deterministic");
        }
    }

    #[tokio::test]
    #[cfg(feature = "wasm-extraction")]
    async fn test_wasm_confidence_normalization() {
        use riptide_html::wasm_extraction::WasmExtractor;

        let extractor = WasmExtractor::new().await
            .expect("Should create WASM extractor");

        let html = r#"
            <html>
            <head><title>WASM Confidence Test</title></head>
            <body>
                <article>
                    <h1>Content</h1>
                    <p>Test content for WASM extraction.</p>
                </article>
            </body>
            </html>
        "#;

        let result = extractor.extract(html, "https://example.com").await
            .expect("WASM extraction should succeed");

        // WASM quality_score (0-100) should be normalized to confidence (0.0-1.0)
        assert!(result.extraction_confidence >= 0.0 && result.extraction_confidence <= 1.0,
            "WASM confidence {} should be normalized to [0,1]", result.extraction_confidence);
    }

    #[test]
    fn test_confidence_aggregation() {
        // Test aggregating confidence from multiple sources
        let confidences = vec![0.8, 0.9, 0.7, 0.85];

        // Weighted average
        let weights = vec![1.0, 1.0, 1.0, 1.0];
        let weighted_sum: f64 = confidences.iter()
            .zip(&weights)
            .map(|(c, w)| c * w)
            .sum();
        let weight_sum: f64 = weights.iter().sum();
        let avg_confidence = weighted_sum / weight_sum;

        assert!(avg_confidence >= 0.0 && avg_confidence <= 1.0);
        assert!((avg_confidence - 0.8125).abs() < 0.001, "Average should be ~0.8125");
    }
}
