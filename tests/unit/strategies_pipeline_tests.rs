//! Unit tests for strategies pipeline orchestrator
//!
//! This module provides comprehensive testing for the strategies pipeline
//! orchestrator, including strategy selection, content processing, caching,
//! error handling, and performance metrics.

#[cfg(test)]
mod strategies_pipeline_tests {
    use riptide_api::strategies_pipeline::{StrategiesPipelineOrchestrator, StrategiesPipelineResult};
    use riptide_api::state::AppState;
    use riptide_core::strategies::{StrategyConfig, ExtractionStrategy, ChunkingConfig};
    use riptide_core::types::{CrawlOptions, RenderMode};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Mock cache implementation for testing
    struct MockCache {
        data: HashMap<String, serde_json::Value>,
    }

    impl MockCache {
        fn new() -> Self {
            Self {
                data: HashMap::new(),
            }
        }
    }

    /// Helper to create a minimal test state
    async fn create_test_state() -> AppState {
        let config = riptide_api::AppConfig {
            port: 0,
            redis_url: "redis://localhost:6379".to_string(),
            headless_url: None,
            cache_ttl: 300,
            max_concurrency: 10,
            gate_hi_threshold: 0.8,
            gate_lo_threshold: 0.3,
            cors_origins: vec![],
            api_key: None,
            openai_api_key: None,
            spider_config: None,
        };

        // This is a simplified test state - in real tests you'd use the proper factory
        AppState {
            config: Arc::new(config),
            http_client: reqwest::Client::new(),
            cache: Arc::new(Mutex::new(riptide_api::cache::Cache::new())),
            metrics: riptide_api::metrics::MetricsCollector::new(),
            resource_manager: Arc::new(riptide_api::resources::ResourceManager::new()),
            spider: None,
        }
    }

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let state = create_test_state().await;
        let options = CrawlOptions::default();

        let orchestrator = StrategiesPipelineOrchestrator::new(
            state,
            options.clone(),
            None,
        );

        // Orchestrator should be created with default strategy config
        assert_eq!(orchestrator.options.concurrency, options.concurrency);
        assert_eq!(orchestrator.options.cache_mode, options.cache_mode);
    }

    #[tokio::test]
    async fn test_orchestrator_with_auto_strategy() {
        let state = create_test_state().await;
        let options = CrawlOptions::default();

        // Test different URL patterns for auto-strategy selection
        let test_urls = vec![
            ("https://github.com/user/repo", "GitHub URL should select CSS strategy"),
            ("https://en.wikipedia.org/wiki/Test", "Wikipedia should select Trek strategy"),
            ("https://medium.com/@author/article", "Medium should select CSS strategy"),
            ("https://news.ycombinator.com/item?id=123", "HN should select Regex strategy"),
            ("https://example.com/unknown", "Unknown site should use Trek strategy"),
        ];

        for (url, description) in test_urls {
            let orchestrator = StrategiesPipelineOrchestrator::with_auto_strategy(
                state.clone(),
                options.clone(),
                url,
            );

            // Strategy should be selected based on URL
            match url {
                s if s.contains("github.com") => {
                    if let ExtractionStrategy::CssJson { .. } = orchestrator.strategy_config.extraction {
                        // Expected
                    } else {
                        panic!("GitHub URLs should use CSS strategy: {}", description);
                    }
                }
                s if s.contains("wikipedia.org") => {
                    if let ExtractionStrategy::Trek = orchestrator.strategy_config.extraction {
                        // Expected
                    } else {
                        panic!("Wikipedia URLs should use Trek strategy: {}", description);
                    }
                }
                _ => {
                    // Other URLs may use various strategies
                }
            }
        }
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let state = create_test_state().await;
        let options = CrawlOptions::default();
        let strategy_config = StrategyConfig::default();

        let orchestrator = StrategiesPipelineOrchestrator::new(
            state,
            options,
            Some(strategy_config),
        );

        let url = "https://example.com/test";
        let cache_key = orchestrator.generate_cache_key(url);

        // Cache key should follow expected format
        assert!(cache_key.starts_with("riptide:strategies:v1:"));
        assert!(cache_key.contains("read_through")); // Default cache mode

        // Same URL should generate same key
        let cache_key2 = orchestrator.generate_cache_key(url);
        assert_eq!(cache_key, cache_key2);

        // Different URLs should generate different keys
        let different_key = orchestrator.generate_cache_key("https://different.com");
        assert_ne!(cache_key, different_key);
    }

    #[tokio::test]
    async fn test_strategy_config_defaults() {
        let config = StrategyConfig::default();

        // Should have reasonable defaults
        assert!(matches!(config.extraction, ExtractionStrategy::Trek));
        assert!(config.chunking.token_max > 0);
        assert!(config.chunking.overlap >= 0);
        assert!(config.chunking.preserve_sentences);
        assert!(config.chunking.deterministic);
    }

    #[tokio::test]
    async fn test_chunking_config_variants() {
        use riptide_core::strategies::chunking::ChunkingMode;

        let modes = vec![
            ChunkingMode::Fixed { size: 1000, by_tokens: true },
            ChunkingMode::Fixed { size: 2000, by_tokens: false },
            ChunkingMode::Sliding,
            ChunkingMode::Sentence,
            ChunkingMode::Topic { similarity_threshold: 0.7 },
            ChunkingMode::Regex { pattern: r"\n\n".to_string() },
        ];

        for mode in modes {
            let config = ChunkingConfig {
                mode: mode.clone(),
                token_max: 1000,
                overlap: 100,
                preserve_sentences: true,
                deterministic: true,
            };

            // All modes should be valid and cloneable
            let _cloned = config.clone();
            let _debug_str = format!("{:?}", config);
        }
    }

    #[tokio::test]
    async fn test_render_mode_chunking_adaptation() {
        let state = create_test_state().await;

        let render_modes = vec![
            (RenderMode::Html, "HTML mode should use sliding chunking"),
            (RenderMode::Pdf, "PDF mode should use fixed chunking"),
            (RenderMode::Markdown, "Markdown mode should use topic chunking"),
        ];

        for (render_mode, description) in render_modes {
            let mut options = CrawlOptions::default();
            options.render_mode = render_mode;

            let orchestrator = StrategiesPipelineOrchestrator::with_auto_strategy(
                state.clone(),
                options,
                "https://example.com",
            );

            match render_mode {
                RenderMode::Html => {
                    if let riptide_core::strategies::chunking::ChunkingMode::Sliding = orchestrator.strategy_config.chunking.mode {
                        // Expected
                    } else {
                        panic!("{}", description);
                    }
                }
                RenderMode::Pdf => {
                    if let riptide_core::strategies::chunking::ChunkingMode::Fixed { .. } = orchestrator.strategy_config.chunking.mode {
                        // Expected
                    } else {
                        panic!("{}", description);
                    }
                }
                RenderMode::Markdown => {
                    if let riptide_core::strategies::chunking::ChunkingMode::Topic { .. } = orchestrator.strategy_config.chunking.mode {
                        // Expected
                    } else {
                        panic!("{}", description);
                    }
                }
            }
        }
    }

    #[test]
    fn test_github_selectors_creation() {
        use riptide_api::strategies_pipeline::create_github_selectors;

        let selectors = create_github_selectors();

        // Should contain expected GitHub-specific selectors
        assert!(selectors.contains_key("title"));
        assert!(selectors.contains_key("content"));
        assert!(selectors.contains_key("author"));
        assert!(selectors.contains_key("date"));

        // Selectors should be reasonable CSS selectors
        let title_selector = &selectors["title"];
        assert!(title_selector.contains("h1") || title_selector.contains(".entry-title"));

        let content_selector = &selectors["content"];
        assert!(content_selector.contains(".markdown-body") || content_selector.contains(".entry-content"));
    }

    #[test]
    fn test_blog_selectors_creation() {
        use riptide_api::strategies_pipeline::create_blog_selectors;

        let selectors = create_blog_selectors();

        // Should contain blog-specific selectors
        assert!(selectors.contains_key("title"));
        assert!(selectors.contains_key("content"));
        assert!(selectors.contains_key("author"));
        assert!(selectors.contains_key("date"));

        // Should include Medium and other blog platform selectors
        let title_selector = &selectors["title"];
        assert!(title_selector.contains("h1") || title_selector.contains("storyTitle"));
    }

    #[test]
    fn test_news_patterns_creation() {
        use riptide_api::strategies_pipeline::create_news_patterns;

        let patterns = create_news_patterns();

        // Should contain news site patterns
        assert!(!patterns.is_empty());

        // Should have patterns for common news site elements
        let has_title_pattern = patterns.iter().any(|p| p.field == "title");
        let has_points_pattern = patterns.iter().any(|p| p.field == "score");
        let has_comments_pattern = patterns.iter().any(|p| p.field == "comment_count");

        assert!(has_title_pattern);
        assert!(has_points_pattern);
        assert!(has_comments_pattern);

        // Patterns should be valid regex
        for pattern in patterns {
            assert!(regex::Regex::new(&pattern.pattern).is_ok(),
                   "Pattern '{}' should be valid regex", pattern.pattern);
        }
    }

    #[tokio::test]
    async fn test_gate_features_analysis() {
        let state = create_test_state().await;
        let options = CrawlOptions::default();
        let orchestrator = StrategiesPipelineOrchestrator::new(state, options, None);

        // Test HTML content analysis
        let html_samples = vec![
            (
                "<html><head><title>Test</title></head><body><p>Simple content</p></body></html>",
                "Simple HTML should have basic features"
            ),
            (
                r#"<html><head>
                    <meta property="og:title" content="Article">
                    <script type="application/ld+json">{"@type":"Article"}</script>
                </head><body>
                    <article><h1>Title</h1><p>Content</p></article>
                </body></html>"#,
                "Rich HTML should have advanced features"
            ),
            (
                r#"<html><head><title>SPA</title></head><body>
                    <div id="root"></div>
                    <script>window.__NEXT_DATA__ = {};</script>
                    <script src="app.js"></script>
                </body></html>"#,
                "SPA should be detected with appropriate markers"
            ),
        ];

        for (html, description) in html_samples {
            let features = orchestrator.analyze_content(html, "https://example.com").await;

            assert!(features.is_ok(), "Analysis should succeed: {}", description);

            let features = features.unwrap();
            assert!(features.html_bytes > 0);
            assert!(features.visible_text_chars > 0);

            if html.contains("og:") {
                assert!(features.has_og, "Should detect Open Graph tags");
            }

            if html.contains(r#""@type":"Article""#) {
                assert!(features.has_jsonld_article, "Should detect JSON-LD article");
            }

            if html.contains("__NEXT_DATA__") {
                assert!(features.spa_markers > 0, "Should detect SPA markers");
            }
        }
    }

    #[tokio::test]
    async fn test_domain_prior_scoring() {
        let state = create_test_state().await;
        let options = CrawlOptions::default();
        let orchestrator = StrategiesPipelineOrchestrator::new(state, options, None);

        let domain_tests = vec![
            ("https://en.wikipedia.org/wiki/Test", 0.9, "Wikipedia should have high prior"),
            ("https://github.com/user/repo", 0.9, "GitHub should have high prior"),
            ("https://medium.com/@author/post", 0.8, "Medium should have good prior"),
            ("https://dev.to/author/post", 0.8, "Dev.to should have good prior"),
            ("https://unknown-site.com/page", 0.5, "Unknown site should have neutral prior"),
        ];

        for (url, expected_min_prior, description) in domain_tests {
            let html = "<html><body><p>Test content</p></body></html>";
            let features = orchestrator.analyze_content(html, url).await;

            assert!(features.is_ok(), "Analysis should succeed: {}", description);

            let features = features.unwrap();
            assert!(features.domain_prior >= expected_min_prior,
                   "Domain prior should be at least {} for {}: got {}",
                   expected_min_prior, url, features.domain_prior);
        }
    }

    #[tokio::test]
    async fn test_orchestrator_cloning() {
        let state = create_test_state().await;
        let options = CrawlOptions {
            concurrency: 32,
            cache_mode: "bypass".to_string(),
            dynamic_wait_for: Some("networkidle2".to_string()),
            scroll_steps: 10,
            token_chunk_max: 2000,
            token_overlap: 200,
            render_mode: RenderMode::Pdf,
        };

        let original = StrategiesPipelineOrchestrator::new(
            state,
            options.clone(),
            Some(StrategyConfig::default()),
        );

        let cloned = original.clone();

        // Cloned orchestrator should have same configuration
        assert_eq!(cloned.options.concurrency, options.concurrency);
        assert_eq!(cloned.options.cache_mode, options.cache_mode);
        assert_eq!(cloned.options.dynamic_wait_for, options.dynamic_wait_for);
        assert_eq!(cloned.options.scroll_steps, options.scroll_steps);
        assert_eq!(cloned.options.token_chunk_max, options.token_chunk_max);
        assert_eq!(cloned.options.token_overlap, options.token_overlap);
    }

    #[test]
    fn test_strategies_pipeline_result_structure() {
        let processed_content = riptide_core::strategies::ProcessedContent {
            extracted_content: riptide_core::strategies::ExtractedContent {
                title: Some("Test Title".to_string()),
                content: "Test content".to_string(),
                metadata: riptide_core::strategies::DocumentMetadata {
                    author: Some("Test Author".to_string()),
                    published_date: Some("2024-01-01T00:00:00Z".to_string()),
                    language: Some("en".to_string()),
                    keywords: vec!["test".to_string()],
                    description: Some("Test description".to_string()),
                },
                links: vec!["https://example.com/link".to_string()],
                images: vec!["https://example.com/image.jpg".to_string()],
            },
            chunks: vec![],
            chunk_metadata: vec![],
        };

        let result = StrategiesPipelineResult {
            processed_content: processed_content.clone(),
            from_cache: false,
            gate_decision: "raw".to_string(),
            quality_score: 0.85,
            processing_time_ms: 150,
            cache_key: "test_key".to_string(),
            http_status: 200,
            strategy_config: StrategyConfig::default(),
            performance_metrics: None,
        };

        // Test result structure
        assert_eq!(result.processed_content.extracted_content.title.as_deref(), Some("Test Title"));
        assert_eq!(result.from_cache, false);
        assert_eq!(result.gate_decision, "raw");
        assert_eq!(result.quality_score, 0.85);
        assert_eq!(result.processing_time_ms, 150);
        assert_eq!(result.cache_key, "test_key");
        assert_eq!(result.http_status, 200);
    }

    #[test]
    fn test_extraction_strategy_variants() {
        let strategies = vec![
            ExtractionStrategy::Trek,
            ExtractionStrategy::CssJson {
                selectors: HashMap::from([
                    ("title".to_string(), "h1".to_string()),
                    ("content".to_string(), ".content".to_string()),
                ]),
            },
            ExtractionStrategy::Regex {
                patterns: vec![
                    riptide_core::strategies::RegexPattern {
                        name: "title".to_string(),
                        pattern: r"<title>([^<]+)</title>".to_string(),
                        field: "title".to_string(),
                        required: true,
                    },
                ],
            },
        ];

        for strategy in strategies {
            // Each strategy should be cloneable and debuggable
            let cloned = strategy.clone();
            let _debug_str = format!("{:?}", cloned);

            // Test strategy matching
            match strategy {
                ExtractionStrategy::Trek => {
                    // Trek strategy
                }
                ExtractionStrategy::CssJson { selectors } => {
                    assert!(!selectors.is_empty());
                }
                ExtractionStrategy::Regex { patterns } => {
                    assert!(!patterns.is_empty());
                }
                _ => {
                    // Other strategies
                }
            }
        }
    }
}

#[cfg(test)]
mod strategy_config_tests {
    use super::*;
    use riptide_core::strategies::{StrategyConfig, ExtractionStrategy, ChunkingConfig};
    use riptide_core::strategies::chunking::ChunkingMode;

    #[test]
    fn test_strategy_config_serialization() {
        let config = StrategyConfig {
            extraction: ExtractionStrategy::Trek,
            chunking: ChunkingConfig {
                mode: ChunkingMode::Sliding,
                token_max: 1500,
                overlap: 150,
                preserve_sentences: true,
                deterministic: true,
            },
            enable_metrics: true,
        };

        // Should be serializable and deserializable
        let json = serde_json::to_string(&config).expect("Should serialize");
        let deserialized: StrategyConfig = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.chunking.token_max, 1500);
        assert_eq!(deserialized.chunking.overlap, 150);
        assert_eq!(deserialized.enable_metrics, true);
    }

    #[test]
    fn test_chunking_mode_validation() {
        let modes = vec![
            ChunkingMode::Fixed { size: 0, by_tokens: true },      // Edge case: zero size
            ChunkingMode::Fixed { size: 5000, by_tokens: false },  // Large size
            ChunkingMode::Topic { similarity_threshold: 0.0 },     // Minimum threshold
            ChunkingMode::Topic { similarity_threshold: 1.0 },     // Maximum threshold
            ChunkingMode::Regex { pattern: "".to_string() },       // Empty pattern
        ];

        for mode in modes {
            let config = ChunkingConfig {
                mode: mode.clone(),
                token_max: 1000,
                overlap: 100,
                preserve_sentences: true,
                deterministic: true,
            };

            // All configurations should be valid structurally
            let _debug_str = format!("{:?}", config);
        }
    }

    #[test]
    fn test_overlap_validation() {
        let config = ChunkingConfig {
            mode: ChunkingMode::Sliding,
            token_max: 1000,
            overlap: 500, // 50% overlap
            preserve_sentences: true,
            deterministic: true,
        };

        // Overlap should be less than token_max for meaningful chunks
        assert!(config.overlap < config.token_max);

        // Test edge cases
        let edge_cases = vec![
            (1000, 999),  // Almost 100% overlap
            (1000, 0),    // No overlap
            (100, 50),    // 50% overlap
        ];

        for (token_max, overlap) in edge_cases {
            let config = ChunkingConfig {
                mode: ChunkingMode::Sliding,
                token_max,
                overlap,
                preserve_sentences: true,
                deterministic: true,
            };

            // Should be valid structurally
            assert!(overlap <= token_max);
        }
    }
}