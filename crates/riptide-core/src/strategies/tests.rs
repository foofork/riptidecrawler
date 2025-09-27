//! Tests for the trait-based strategy management system

#[cfg(test)]
mod tests {
    use super::*;
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

        // Test CSS strategy
        let css_strategy = CssJsonExtractionStrategy::with_default_selectors();
        let result = css_strategy.extract(TEST_HTML, TEST_URL).await;
        assert!(result.is_ok());
        assert_eq!(css_strategy.name(), "css_json");

        // Test Regex strategy
        let regex_strategy = RegexExtractionStrategy::with_default_patterns();
        let result = regex_strategy.extract(TEST_HTML, TEST_URL).await;
        assert!(result.is_ok());
        assert_eq!(regex_strategy.name(), "regex");
    }

    #[tokio::test]
    async fn test_trait_chunking_strategies() {
        let content = "This is a test content. It has multiple sentences. Each sentence should be processed correctly. The chunking strategy should handle this well.";

        // Test Fixed chunking
        let fixed_strategy = FixedChunkingStrategy::new(50, false);
        let config = fixed_strategy.optimal_config();
        let chunks = fixed_strategy.chunk(content, &config).await;
        assert!(chunks.is_ok());
        assert_eq!(fixed_strategy.name(), "fixed");

        // Test Sliding chunking
        let sliding_strategy = SlidingChunkingStrategy;
        let config = sliding_strategy.optimal_config();
        let chunks = sliding_strategy.chunk(content, &config).await;
        assert!(chunks.is_ok());
        assert_eq!(sliding_strategy.name(), "sliding");

        // Test Sentence chunking
        let sentence_strategy = SentenceChunkingStrategy::new(2);
        let config = sentence_strategy.optimal_config();
        let chunks = sentence_strategy.chunk(content, &config).await;
        assert!(chunks.is_ok());
        assert_eq!(sentence_strategy.name(), "sentence");
    }

    #[tokio::test]
    async fn test_strategy_registry() {
        let mut registry = StrategyRegistry::new();

        // Register strategies
        registry.register_extraction(Arc::new(TrekExtractionStrategy));
        registry.register_chunking(Arc::new(SlidingChunkingStrategy));

        // Test retrieval
        let extraction_strategy = registry.get_extraction("trek");
        assert!(extraction_strategy.is_some());

        let chunking_strategy = registry.get_chunking("sliding");
        assert!(chunking_strategy.is_some());

        // Test listing
        let extraction_list = registry.list_extraction_strategies();
        assert!(!extraction_list.is_empty());

        let chunking_list = registry.list_chunking_strategies();
        assert!(!chunking_list.is_empty());
    }

    #[tokio::test]
    async fn test_default_registry() {
        let registry = create_default_registry();

        // Check that default strategies are available
        assert!(registry.get_extraction("trek").is_some());
        assert!(registry.get_extraction("css_json").is_some());
        assert!(registry.get_extraction("regex").is_some());

        assert!(registry.get_chunking("sliding").is_some());
        assert!(registry.get_chunking("fixed").is_some());
        assert!(registry.get_chunking("sentence").is_some());
    }

    #[tokio::test]
    async fn test_enhanced_strategy_manager() {
        let config = StrategyManagerConfig::default();
        let manager = EnhancedStrategyManager::new(config).await;

        // Test extraction and processing
        let result = manager.extract_and_process(TEST_HTML, TEST_URL).await;
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert!(!processed.extracted.title.is_empty());
        assert!(!processed.extracted.content.is_empty());
        assert!(!processed.chunks.is_empty());
        assert!(processed.processing_time.as_millis() > 0);

        // Test strategy listing
        let strategies = manager.list_strategies().await;
        assert!(!strategies.extraction.is_empty());
        assert!(!strategies.chunking.is_empty());
    }

    #[tokio::test]
    async fn test_compatibility_layer() {
        use super::compatibility::*;

        let config = StrategyConfig::default();
        let mut manager = CompatibleStrategyManager::new(config).await;

        // Test backward compatibility
        let result = manager.extract_and_chunk(TEST_HTML, TEST_URL).await;
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert!(!processed.extracted.title.is_empty());
        assert!(!processed.extracted.content.is_empty());
        assert!(!processed.chunks.is_empty());
    }

    #[tokio::test]
    async fn test_strategy_factory() {
        use super::compatibility::*;

        // Test extraction strategy creation
        let trek_enum = ExtractionStrategy::Trek;
        let trek_trait = StrategyFactory::create_extraction(trek_enum);
        assert_eq!(trek_trait.name(), "trek");

        // Test chunking strategy creation
        let chunking_config = ChunkingConfig::default();
        let chunking_trait = StrategyFactory::create_chunking(&chunking_config);
        assert_eq!(chunking_trait.name(), "sliding");
    }

    #[tokio::test]
    async fn test_migration_utils() {
        use super::compatibility::*;

        let old_config = StrategyConfig::default();
        let new_config = MigrationUtils::convert_config(&old_config);

        assert_eq!(new_config.enable_metrics, old_config.enable_metrics);
        assert_eq!(new_config.validate_schema, old_config.validate_schema);

        let upgraded_manager = MigrationUtils::upgrade_manager(old_config).await;
        assert!(upgraded_manager.is_ok());
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

    #[tokio::test]
    async fn test_auto_strategy_selection() {
        let config = StrategyManagerConfig {
            auto_strategy_selection: true,
            ..Default::default()
        };
        let manager = EnhancedStrategyManager::new(config).await;

        // Test with structured HTML - should select a strategy
        let result = manager.extract_and_process(TEST_HTML, TEST_URL).await;
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert!(!processed.strategy_used.is_empty());
    }

    #[tokio::test]
    async fn test_chunking_estimation() {
        let content = "This is a test content for chunking estimation. It should provide reasonable estimates.";
        let config = ChunkingConfig::default();

        let strategy = SlidingChunkingStrategy;
        let estimate = strategy.estimate_chunks(content, &config);
        assert!(estimate > 0);

        let strategy = FixedChunkingStrategy::new(50, false);
        let estimate = strategy.estimate_chunks(content, &config);
        assert!(estimate > 0);
    }
}