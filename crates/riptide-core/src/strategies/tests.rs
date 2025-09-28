//! Tests for the trait-based strategy management system

#[cfg(test)]
mod tests {
    use crate::strategies::{
        TrekExtractionStrategy, StrategyRegistry, PerformanceTier,
        traits::ExtractionStrategy,
    };
    use std::sync::Arc;

    const TEST_HTML: &str = r#"
    <html>
    <head>
        <title>Test Page</title>
        <meta name="description" content="Test description">
    </head>
    <body>
        <main>
            <article>
                <h1>Main Heading</h1>
                <p>This is the main content of the test page.</p>
                <p>It contains multiple paragraphs for testing.</p>
                <section>
                    <h2>Subsection</h2>
                    <p>Additional content in a subsection.</p>
                </section>
            </article>
        </main>
    </body>
    </html>
    "#;

    const TEST_URL: &str = "https://example.com/test";

    #[tokio::test]
    async fn test_trait_extraction_strategies() {
        // Test Trek strategy
        let trek_strategy = TrekExtractionStrategy;
        let result = trek_strategy.extract(TEST_HTML, TEST_URL).await;
        assert!(result.is_ok());
        let extraction_result = result.unwrap();
        assert!(!extraction_result.content.title.is_empty());
        assert!(!extraction_result.content.content.is_empty());
        assert_eq!(trek_strategy.name(), "trek");
    }

    #[tokio::test]
    async fn test_strategy_registry_basic() {
        let mut registry = StrategyRegistry::new();

        // Register Trek strategy
        registry.register_extraction(Arc::new(TrekExtractionStrategy));

        // Test retrieval
        let extraction_strategy = registry.get_extraction("trek");
        assert!(extraction_strategy.is_some());

        // Test listing
        let extraction_list = registry.list_extraction_strategies();
        assert!(!extraction_list.is_empty());
    }

    #[tokio::test]
    async fn test_strategy_registry_find_best() {
        let mut registry = StrategyRegistry::new();
        registry.register_extraction(Arc::new(TrekExtractionStrategy));

        // Test finding best strategy
        let best_strategy = registry.find_best_extraction(TEST_HTML);
        assert!(best_strategy.is_some());
        assert_eq!(best_strategy.unwrap().name(), "trek");
    }

    #[tokio::test]
    async fn test_strategy_capabilities() {
        let strategy = TrekExtractionStrategy;
        let capabilities = strategy.capabilities();

        assert_eq!(capabilities.strategy_type, "wasm_extraction");
        assert!(capabilities.supported_content_types.contains(&"text/html".to_string()));
        assert_eq!(capabilities.performance_tier, PerformanceTier::Fast);
        assert!(!capabilities.resource_requirements.requires_network);
    }

    #[tokio::test]
    async fn test_confidence_scoring() {
        let strategy = TrekExtractionStrategy;

        // Test with valid HTML
        let valid_html = "<html><head><title>Test</title></head><body><p>Content</p></body></html>";
        let score = strategy.confidence_score(valid_html);
        assert!(score > 0.5);

        // Test with less structured content
        let plain_text = "Just plain text without HTML tags";
        let score = strategy.confidence_score(plain_text);
        assert!(score < 0.5);
    }
}