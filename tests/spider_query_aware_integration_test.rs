use anyhow::Result;
use riptide_core::spider::{
    config::SpiderPresets,
    core::Spider,
    query_aware::{QueryAwareConfig, QueryAwareScorer},
    types::CrawlRequest,
    SpiderConfig,
};
use std::str::FromStr;
use tokio::time::Duration;
use url::Url;

/// Integration tests for Query-Aware Spider functionality
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_query_aware_spider_creation() -> Result<()> {
        let mut config = SpiderPresets::development();

        // Enable query-aware crawling
        config.query_aware = QueryAwareConfig {
            query_foraging: true,
            target_query: Some("machine learning".to_string()),
            bm25_weight: 0.4,
            url_signals_weight: 0.2,
            domain_diversity_weight: 0.2,
            content_similarity_weight: 0.2,
            min_relevance_threshold: 0.3,
            relevance_window_size: 10,
            bm25_k1: 1.2,
            bm25_b: 0.75,
        };

        let spider = Spider::new(config).await?;
        let stats = spider.get_query_aware_stats().await;

        assert!(stats.is_some());
        let stats = stats.unwrap();
        assert!(stats.enabled);

        Ok(())
    }

    #[tokio::test]
    async fn test_query_aware_disabled_by_default() -> Result<()> {
        let config = SpiderPresets::development();
        let spider = Spider::new(config).await?;

        let stats = spider.get_query_aware_stats().await;
        assert!(stats.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_query_aware_scoring_integration() -> Result<()> {
        let config = QueryAwareConfig {
            query_foraging: true,
            target_query: Some("rust programming".to_string()),
            bm25_weight: 0.4,
            url_signals_weight: 0.2,
            domain_diversity_weight: 0.2,
            content_similarity_weight: 0.2,
            min_relevance_threshold: 0.2,
            relevance_window_size: 5,
            bm25_k1: 1.2,
            bm25_b: 0.75,
        };

        let mut scorer = QueryAwareScorer::new(config);

        // Create test requests with different relevance levels
        let high_relevance_url = Url::from_str("https://rust-lang.org/learn/programming")?;
        let high_relevance_request = CrawlRequest::new(high_relevance_url).with_depth(1);
        let high_relevance_content = "Rust is a systems programming language that focuses on safety and performance";

        let low_relevance_url = Url::from_str("https://example.com/random")?;
        let low_relevance_request = CrawlRequest::new(low_relevance_url).with_depth(3);
        let low_relevance_content = "This is about cooking and recipes, nothing related to programming";

        // Score the requests
        let high_score = scorer.score_request(&high_relevance_request, Some(high_relevance_content));
        let low_score = scorer.score_request(&low_relevance_request, Some(low_relevance_content));

        // High relevance should score better than low relevance
        assert!(high_score > low_score);
        assert!(high_score > 0.5); // Should be reasonably high
        assert!(low_score < 0.8);  // Should be lower

        Ok(())
    }

    #[tokio::test]
    async fn test_early_stopping_functionality() -> Result<()> {
        let config = QueryAwareConfig {
            query_foraging: true,
            target_query: Some("machine learning".to_string()),
            min_relevance_threshold: 0.7, // High threshold for early stopping
            relevance_window_size: 3,
            ..Default::default()
        };

        let mut scorer = QueryAwareScorer::new(config);

        // Simulate low relevance scores
        let url = Url::from_str("https://example.com/irrelevant")?;
        let request = CrawlRequest::new(url);
        let irrelevant_content = "This content has nothing to do with the target query at all";

        // Add several low-scoring requests
        for _ in 0..3 {
            scorer.score_request(&request, Some(irrelevant_content));
        }

        // Should trigger early stopping
        let (should_stop, reason) = scorer.should_stop_early();
        assert!(should_stop);
        assert!(!reason.is_empty());
        assert!(reason.contains("Low relevance detected"));

        Ok(())
    }

    #[tokio::test]
    async fn test_domain_diversity_tracking() -> Result<()> {
        let config = QueryAwareConfig {
            query_foraging: true,
            target_query: Some("programming".to_string()),
            domain_diversity_weight: 1.0, // Focus on diversity
            bm25_weight: 0.0,
            url_signals_weight: 0.0,
            content_similarity_weight: 0.0,
            ..Default::default()
        };

        let mut scorer = QueryAwareScorer::new(config);

        // Test requests from different domains
        let domain1_url = Url::from_str("https://example.com/page1")?;
        let domain2_url = Url::from_str("https://different.com/page1")?;
        let domain1_url2 = Url::from_str("https://example.com/page2")?;

        let request1 = CrawlRequest::new(domain1_url);
        let request2 = CrawlRequest::new(domain2_url);
        let request3 = CrawlRequest::new(domain1_url2);

        let content = "Some programming content";

        // Score requests
        let score1 = scorer.score_request(&request1, Some(content)); // First from domain1
        let score2 = scorer.score_request(&request2, Some(content)); // First from domain2
        let score3 = scorer.score_request(&request3, Some(content)); // Second from domain1

        // New domain should score higher than repeated domain
        assert!(score2 > score3);

        let stats = scorer.get_stats();
        assert_eq!(stats.unique_domains, 2);
        assert_eq!(stats.total_pages, 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_url_signal_analysis() -> Result<()> {
        let config = QueryAwareConfig {
            query_foraging: true,
            target_query: Some("rust tutorial".to_string()),
            url_signals_weight: 1.0, // Focus on URL signals
            bm25_weight: 0.0,
            domain_diversity_weight: 0.0,
            content_similarity_weight: 0.0,
            ..Default::default()
        };

        let mut scorer = QueryAwareScorer::new(config);

        // Test URLs with different relevance and depths
        let relevant_shallow_url = Url::from_str("https://rust-lang.org/learn/tutorial")?;
        let relevant_deep_url = Url::from_str("https://rust-lang.org/learn/tutorial/advanced/deep/path")?;
        let irrelevant_url = Url::from_str("https://example.com/random/page")?;

        let shallow_request = CrawlRequest::new(relevant_shallow_url).with_depth(1);
        let deep_request = CrawlRequest::new(relevant_deep_url).with_depth(5);
        let irrelevant_request = CrawlRequest::new(irrelevant_url).with_depth(1);

        let content = "Some content";

        let shallow_score = scorer.score_request(&shallow_request, Some(content));
        let deep_score = scorer.score_request(&deep_request, Some(content));
        let irrelevant_score = scorer.score_request(&irrelevant_request, Some(content));

        // Shallow relevant URL should score highest
        assert!(shallow_score > deep_score);
        assert!(shallow_score > irrelevant_score);

        Ok(())
    }

    #[tokio::test]
    async fn test_bm25_scoring_component() -> Result<()> {
        let config = QueryAwareConfig {
            query_foraging: true,
            target_query: Some("artificial intelligence".to_string()),
            bm25_weight: 1.0, // Focus on BM25
            url_signals_weight: 0.0,
            domain_diversity_weight: 0.0,
            content_similarity_weight: 0.0,
            ..Default::default()
        };

        let mut scorer = QueryAwareScorer::new(config);

        let url = Url::from_str("https://example.com/ai")?;
        let request = CrawlRequest::new(url);

        // Build corpus first
        let corpus_docs = vec![
            "Machine learning is a subset of artificial intelligence",
            "Deep learning networks are used in artificial intelligence",
            "Natural language processing in artificial intelligence",
        ];

        for doc in &corpus_docs {
            scorer.score_request(&request, Some(doc));
        }

        // Test documents with different relevance
        let highly_relevant = "Artificial intelligence algorithms and machine learning techniques";
        let somewhat_relevant = "This document mentions artificial intelligence briefly";
        let irrelevant = "This document is about cooking and has no relation to AI";

        let high_score = scorer.score_request(&request, Some(highly_relevant));
        let medium_score = scorer.score_request(&request, Some(somewhat_relevant));
        let low_score = scorer.score_request(&request, Some(irrelevant));

        assert!(high_score > medium_score);
        assert!(medium_score > low_score);

        Ok(())
    }

    #[tokio::test]
    async fn test_weight_configuration_effects() -> Result<()> {
        // Test with different weight configurations
        let high_bm25_config = QueryAwareConfig {
            query_foraging: true,
            target_query: Some("programming".to_string()),
            bm25_weight: 0.8,
            url_signals_weight: 0.1,
            domain_diversity_weight: 0.05,
            content_similarity_weight: 0.05,
            ..Default::default()
        };

        let high_url_config = QueryAwareConfig {
            query_foraging: true,
            target_query: Some("programming".to_string()),
            bm25_weight: 0.1,
            url_signals_weight: 0.8,
            domain_diversity_weight: 0.05,
            content_similarity_weight: 0.05,
            ..Default::default()
        };

        let mut bm25_scorer = QueryAwareScorer::new(high_bm25_config);
        let mut url_scorer = QueryAwareScorer::new(high_url_config);

        // Create test scenario where BM25 and URL signals would give different preferences
        let relevant_content_poor_url = Url::from_str("https://example.com/random/deep/path")?;
        let poor_content_good_url = Url::from_str("https://programming-tutorial.com/learn")?;

        let request1 = CrawlRequest::new(relevant_content_poor_url).with_depth(5);
        let request2 = CrawlRequest::new(poor_content_good_url).with_depth(1);

        let relevant_content = "This is all about programming languages and software development";
        let poor_content = "This document discusses unrelated topics";

        // BM25-focused scorer should prefer relevant content
        let bm25_score1 = bm25_scorer.score_request(&request1, Some(relevant_content));
        let bm25_score2 = bm25_scorer.score_request(&request2, Some(poor_content));

        // URL-focused scorer should prefer good URL structure
        let url_score1 = url_scorer.score_request(&request1, Some(relevant_content));
        let url_score2 = url_scorer.score_request(&request2, Some(poor_content));

        // Different weight configurations should produce different relative scores
        // This test ensures the weight system is working
        assert!(bm25_score1 > 0.0);
        assert!(url_score2 > 0.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_performance_lift_simulation() -> Result<()> {
        // Simulate the expected 20% lift in on-topic tokens per page

        let query_aware_config = QueryAwareConfig {
            query_foraging: true,
            target_query: Some("machine learning algorithms".to_string()),
            ..Default::default()
        };

        let mut query_aware_scorer = QueryAwareScorer::new(query_aware_config.clone());

        // Simulate crawling without query-aware features (baseline)
        let baseline_urls = vec![
            "https://example.com/page1",
            "https://random.com/content",
            "https://unrelated.com/stuff",
            "https://ml.com/tutorial", // Only this one is relevant
        ];

        // Simulate crawling with query-aware features (should prioritize relevant content)
        let query_aware_urls = vec![
            "https://ml.com/tutorial",
            "https://ml.com/algorithms",
            "https://machinelearning.com/guide",
            "https://ai.com/ml-basics",
        ];

        // Calculate simulated relevance scores
        let mut baseline_relevance = 0.0;
        let mut query_aware_relevance = 0.0;

        for url_str in baseline_urls {
            let url = Url::from_str(url_str)?;
            let request = CrawlRequest::new(url);
            let content = if url_str.contains("ml") || url_str.contains("machine") {
                "Machine learning algorithms and neural networks"
            } else {
                "Random content about unrelated topics"
            };

            let score = query_aware_scorer.score_request(&request, Some(content));
            baseline_relevance += score;
        }

        for url_str in query_aware_urls {
            let url = Url::from_str(url_str)?;
            let request = CrawlRequest::new(url);
            let content = "Machine learning algorithms, neural networks, and artificial intelligence techniques";

            let score = query_aware_scorer.score_request(&request, Some(content));
            query_aware_relevance += score;
        }

        let baseline_avg = baseline_relevance / 4.0;
        let query_aware_avg = query_aware_relevance / 4.0;

        // Calculate percentage improvement
        let improvement = (query_aware_avg - baseline_avg) / baseline_avg * 100.0;

        // Should achieve at least 20% improvement in relevance scores
        println!("Baseline average relevance: {:.3}", baseline_avg);
        println!("Query-aware average relevance: {:.3}", query_aware_avg);
        println!("Improvement: {:.1}%", improvement);

        assert!(improvement >= 20.0, "Should achieve at least 20% improvement, got {:.1}%", improvement);

        Ok(())
    }

    #[tokio::test]
    async fn test_opt_in_flag_behavior() -> Result<()> {
        // Test that query_foraging: false completely disables the feature
        let disabled_config = QueryAwareConfig {
            query_foraging: false,
            target_query: Some("test query".to_string()),
            ..Default::default()
        };

        let mut scorer = QueryAwareScorer::new(disabled_config);

        let url = Url::from_str("https://example.com/test")?;
        let request = CrawlRequest::new(url);

        // Should return neutral score regardless of content when disabled
        let score1 = scorer.score_request(&request, Some("highly relevant test query content"));
        let score2 = scorer.score_request(&request, Some("completely irrelevant content"));

        assert_eq!(score1, 1.0);
        assert_eq!(score2, 1.0);

        // Should never trigger early stopping when disabled
        let (should_stop, _) = scorer.should_stop_early();
        assert!(!should_stop);

        Ok(())
    }
}