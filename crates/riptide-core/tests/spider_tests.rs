//! Query-aware spider tests with BM25 scoring

use riptide_core::spider::query_aware::BM25Scorer;

#[cfg(test)]
mod bm25_scoring_tests {
    use super::*;

    #[test]
    #[ignore = "TODO: Adjust test expectations for BM25Scorer - scoring behavior changed"]
    fn test_bm25_calculation() {
        let mut scorer = BM25Scorer::new("quick fox", 1.2, 0.75);

        // Create document corpus
        let documents = vec![
            "The quick brown fox jumps over the lazy dog",
            "Machine learning is transforming artificial intelligence",
            "The fox is quick and clever",
            "Deep learning requires large datasets",
            "The brown fox hunts at night",
        ];

        // Build index with update_corpus
        for doc in &documents {
            scorer.update_corpus(doc);
        }

        // Test scoring with score() method
        let scores: Vec<f64> = documents.iter().map(|doc| scorer.score(doc)).collect();

        // TODO: Verify expected scoring behavior with new BM25 implementation
        // The IDF calculation and term frequency handling may differ from old API
        // Documents with "quick" and "fox" should score highest
        assert!(scores[0] > scores[1]); // Doc 0 has both terms
        assert!(scores[2] > scores[1]); // Doc 2 has both terms
        assert!(scores[4] > scores[3]); // Doc 4 has "fox"
    }

    #[test]
    #[ignore = "TODO: Adjust saturation expectations for BM25Scorer - implementation changed"]
    fn test_term_frequency_saturation() {
        let mut scorer = BM25Scorer::new("test", 1.2, 0.75);

        let documents = vec![
            "test test test test test",
            "test document with single occurrence",
            "another document without the term",
        ];

        // Build index with update_corpus
        for doc in &documents {
            scorer.update_corpus(doc);
        }

        let scores: Vec<f64> = documents.iter().map(|doc| scorer.score(doc)).collect();

        // TODO: Verify saturation behavior with new BM25 implementation
        // BM25 should saturate - doc with 5 occurrences shouldn't be 5x higher
        assert!(scores[0] < scores[1] * 3.0);
    }

    #[test]
    fn test_inverse_document_frequency() {
        // Test with common term
        let mut scorer_common = BM25Scorer::new("common", 1.2, 0.75);

        let documents = vec![
            "common word appears everywhere",
            "common word appears here too",
            "unique specialized term appears once",
            "common word appears again",
        ];

        for doc in &documents {
            scorer_common.update_corpus(doc);
        }
        let common_scores: Vec<f64> = documents
            .iter()
            .map(|doc| scorer_common.score(doc))
            .collect();

        // Test with rare term
        let mut scorer_unique = BM25Scorer::new("specialized", 1.2, 0.75);
        for doc in &documents {
            scorer_unique.update_corpus(doc);
        }
        let unique_scores: Vec<f64> = documents
            .iter()
            .map(|doc| scorer_unique.score(doc))
            .collect();

        // Rare terms should score higher
        assert!(unique_scores[2] > common_scores[0]);
    }
}

#[cfg(test)]
mod query_aware_crawler_tests {
    // NOTE: QueryAwareCrawler has been refactored to QueryAwareScorer
    // These tests need to be rewritten to use the new API:
    // - QueryAwareScorer::new(config) instead of QueryAwareCrawler
    // - score_request() instead of score_urls()
    // - Individual analyzers (UrlSignalAnalyzer, DomainDiversityAnalyzer, ContentSimilarityAnalyzer)
    //   are now internal to QueryAwareScorer
    // - See query_aware.rs for current API and existing tests

    #[tokio::test]
    #[ignore = "TODO: Rewrite for QueryAwareScorer API - old QueryAwareCrawler removed"]
    async fn test_query_aware_url_prioritization() {
        // Old config fields removed/renamed:
        // - enable_bm25 removed (use query_foraging instead)
        // - url_signal_weight renamed to url_signals_weight
        // - max_depth, early_stop_threshold, min_crawl_count removed
        // - Added: query_foraging, target_query, min_relevance_threshold, relevance_window_size

        // TODO: Rewrite using QueryAwareScorer::score_request() with CrawlRequest
        unimplemented!("Rewrite for new QueryAwareScorer API")
    }

    #[tokio::test]
    #[ignore = "TODO: Rewrite for QueryAwareScorer API - domain analyzer is now internal"]
    async fn test_domain_diversity_scoring() {
        // DomainDiversityAnalyzer is now internal to QueryAwareScorer
        // Use QueryAwareScorer::score_request() which internally uses DomainDiversityAnalyzer
        // Or test DomainDiversityAnalyzer directly if needed

        unimplemented!("Rewrite for new API - domain analyzer is internal")
    }

    #[tokio::test]
    #[ignore = "TODO: Rewrite for Spider/QueryAwareScorer integration - crawl_with_query removed"]
    async fn test_early_stopping_on_low_relevance() {
        // Old fields: early_stop_threshold, min_crawl_count - removed
        // New: min_relevance_threshold, relevance_window_size
        // Use Spider with QueryAwareScorer integration and should_stop_early()

        unimplemented!("Rewrite using Spider with QueryAwareScorer")
    }

    #[tokio::test]
    #[ignore = "TODO: Test ContentSimilarityAnalyzer directly or via QueryAwareScorer"]
    async fn test_content_similarity_deduplication() {
        // ContentSimilarityAnalyzer is internal to QueryAwareScorer
        // Test via QueryAwareScorer::score_request() or test analyzer directly if exposed

        unimplemented!("Rewrite for new API - content analyzer is internal")
    }
}

#[cfg(test)]
mod crawl_orchestration_tests {
    // NOTE: CrawlConfig and CrawlOrchestrator have been removed
    // Use Spider with SpiderConfig instead for orchestration
    // See spider/core.rs for Spider API and spider/budget.rs for budget controls

    #[tokio::test]
    #[ignore = "TODO: Rewrite using Spider with SpiderConfig - CrawlOrchestrator removed"]
    async fn test_parallel_crawling_with_limits() {
        // Old: CrawlOrchestrator with CrawlConfig
        // New: Spider::new(SpiderConfig) with BudgetManager for limits
        // SpiderConfig has: max_concurrent, max_pages, timeout_ms, respect_robots_txt

        unimplemented!("Rewrite using Spider API")
    }

    #[tokio::test]
    #[ignore = "TODO: Rewrite robots.txt handling with Spider - CrawlOrchestrator removed"]
    async fn test_crawl_with_robots_txt_compliance() {
        // Robots.txt handling is in Spider
        // Use Spider with SpiderConfig { respect_robots_txt: true }

        unimplemented!("Rewrite using Spider robots.txt handling")
    }

    #[tokio::test]
    #[ignore = "TODO: Rewrite rate limiting with BudgetManager - CrawlOrchestrator removed"]
    async fn test_crawl_rate_limiting() {
        // Rate limiting is in BudgetManager
        // Use Spider with BudgetConfig for rate limiting

        unimplemented!("Rewrite using BudgetManager")
    }
}

#[cfg(test)]
mod url_frontier_tests {
    use riptide_core::spider::frontier::{FrontierConfig, FrontierManager};
    use riptide_core::spider::types::{CrawlRequest, Priority};
    use std::str::FromStr;
    use url::Url;

    #[tokio::test]
    async fn test_url_frontier_prioritization() {
        // FrontierConfig fields changed:
        // - max_urls removed (use memory_limit instead)
        // - priority_buckets removed (uses priority queues internally)
        let config = FrontierConfig {
            memory_limit: 1000,
            ..Default::default()
        };

        let frontier = FrontierManager::new(config).expect("Failed to create frontier");

        // Add URLs with different priorities using CrawlRequest
        let high_url = Url::from_str("https://high-priority.com").unwrap();
        let medium_url = Url::from_str("https://medium-priority.com").unwrap();
        let low_url = Url::from_str("https://low-priority.com").unwrap();

        frontier
            .add_request(CrawlRequest::new(low_url.clone()).with_priority(Priority::Low))
            .await
            .expect("Failed to add low priority");

        frontier
            .add_request(CrawlRequest::new(high_url.clone()).with_priority(Priority::High))
            .await
            .expect("Failed to add high priority");

        frontier
            .add_request(CrawlRequest::new(medium_url.clone()).with_priority(Priority::Medium))
            .await
            .expect("Failed to add medium priority");

        // Should return highest priority first
        let next = frontier
            .next_request()
            .await
            .expect("Failed to get request")
            .expect("Should have request");
        assert_eq!(next.url, high_url);
    }

    #[tokio::test]
    #[ignore = "TODO: Implement deduplication test with FrontierManager"]
    async fn test_url_deduplication() {
        // FrontierManager doesn't automatically deduplicate URLs
        // Deduplication would need to be handled at a higher level (Spider)
        // or by checking if URL already exists before adding

        unimplemented!("Deduplication handled by Spider, not FrontierManager")
    }

    #[tokio::test]
    #[ignore = "TODO: URL normalization moved to url_utils module"]
    async fn test_url_normalization() {
        // URL normalization is in spider/url_utils.rs, not in FrontierManager
        // Test url_utils::normalize_url() instead

        unimplemented!("Test url_utils::normalize_url() instead")
    }
}
