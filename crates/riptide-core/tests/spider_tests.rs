//! Query-aware spider tests with BM25 scoring

use riptide_core::spider::*;
use anyhow::Result;
use std::collections::HashMap;

#[cfg(test)]
mod bm25_scoring_tests {
    use super::*;

    #[test]
    fn test_bm25_calculation() {
        let scorer = BM25Scorer::new(1.2, 0.75);

        // Create document corpus
        let documents = vec![
            "The quick brown fox jumps over the lazy dog",
            "Machine learning is transforming artificial intelligence",
            "The fox is quick and clever",
            "Deep learning requires large datasets",
            "The brown fox hunts at night",
        ];

        // Build index
        scorer.index_documents(&documents);

        // Test scoring
        let query = "quick fox";
        let scores = scorer.score_documents(query, &documents);

        // Documents with "quick" and "fox" should score highest
        assert!(scores[0] > scores[1]); // Doc 0 has both terms
        assert!(scores[2] > scores[1]); // Doc 2 has both terms
        assert!(scores[4] > scores[3]); // Doc 4 has "fox"
    }

    #[test]
    fn test_term_frequency_saturation() {
        let scorer = BM25Scorer::new(1.2, 0.75);

        let documents = vec![
            "test test test test test",
            "test document with single occurrence",
            "another document without the term",
        ];

        scorer.index_documents(&documents);
        let scores = scorer.score_documents("test", &documents);

        // BM25 should saturate - doc with 5 occurrences shouldn't be 5x higher
        assert!(scores[0] < scores[1] * 3.0);
    }

    #[test]
    fn test_inverse_document_frequency() {
        let scorer = BM25Scorer::new(1.2, 0.75);

        let documents = vec![
            "common word appears everywhere",
            "common word appears here too",
            "unique specialized term appears once",
            "common word appears again",
        ];

        scorer.index_documents(&documents);

        // Rare terms should score higher
        let common_scores = scorer.score_documents("common", &documents);
        let unique_scores = scorer.score_documents("specialized", &documents);

        assert!(unique_scores[2] > common_scores[0]);
    }
}

#[cfg(test)]
mod query_aware_crawler_tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[derive(Clone)]
    struct MockFetcher {
        responses: Arc<RwLock<HashMap<String, String>>>,
    }

    impl MockFetcher {
        fn new() -> Self {
            Self {
                responses: Arc::new(RwLock::new(HashMap::new())),
            }
        }

        async fn add_response(&self, url: &str, content: &str) {
            self.responses.write().await.insert(url.to_string(), content.to_string());
        }

        async fn fetch(&self, url: &str) -> Result<String> {
            self.responses
                .read()
                .await
                .get(url)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("URL not found"))
        }
    }

    #[tokio::test]
    async fn test_query_aware_url_prioritization() {
        let config = QueryAwareConfig {
            enable_bm25: true,
            bm25_weight: 0.4,
            url_signal_weight: 0.3,
            domain_diversity_weight: 0.2,
            content_similarity_weight: 0.1,
            max_depth: 3,
            early_stop_threshold: 0.1,
            ..Default::default()
        };

        let crawler = QueryAwareCrawler::new(config);
        let query = "machine learning algorithms";

        // Test URL scoring
        let urls = vec![
            ("https://example.com/machine-learning-guide", 1),
            ("https://example.com/cooking-recipes", 2),
            ("https://example.com/ai/deep-learning", 2),
            ("https://blog.com/ml-algorithms", 1),
            ("https://news.com/sports", 1),
        ];

        let scores = crawler.score_urls(&urls, query).await;

        // URLs with query terms should score higher
        assert!(scores[0].1 > scores[1].1); // ML guide > cooking
        assert!(scores[2].1 > scores[1].1); // Deep learning > cooking
        assert!(scores[3].1 > scores[4].1); // ML algorithms > sports
    }

    #[tokio::test]
    async fn test_domain_diversity_scoring() {
        let crawler = QueryAwareCrawler::new(QueryAwareConfig::default());

        let mut visited_domains = HashMap::new();
        visited_domains.insert("example.com".to_string(), 5);
        visited_domains.insert("blog.com".to_string(), 1);

        // New domain should score higher
        let new_domain_score = crawler.calculate_domain_diversity("newsite.com", &visited_domains);
        let existing_domain_score = crawler.calculate_domain_diversity("example.com", &visited_domains);

        assert!(new_domain_score > existing_domain_score);
    }

    #[tokio::test]
    async fn test_early_stopping_on_low_relevance() {
        let config = QueryAwareConfig {
            early_stop_threshold: 0.3,
            min_crawl_count: 5,
            ..Default::default()
        };

        let crawler = QueryAwareCrawler::new(config);
        let fetcher = MockFetcher::new();

        // Add low-relevance content
        for i in 0..10 {
            fetcher.add_response(
                &format!("https://example.com/page{}", i),
                "This is unrelated content about cooking and recipes",
            ).await;
        }

        let query = "quantum computing research";
        let seeds = vec!["https://example.com/page0".to_string()];

        let results = crawler.crawl_with_query(seeds, query, fetcher).await.unwrap();

        // Should stop early due to low relevance
        assert!(results.pages_crawled <= 5);
        assert!(results.stopped_early);
    }

    #[tokio::test]
    async fn test_content_similarity_deduplication() {
        let crawler = QueryAwareCrawler::new(QueryAwareConfig::default());

        let content1 = "Machine learning is a subset of artificial intelligence";
        let content2 = "Machine learning is a subset of artificial intelligence";
        let content3 = "Deep learning uses neural networks for complex tasks";

        // Identical content should have high similarity
        let sim1 = crawler.calculate_content_similarity(content1, content2);
        assert!(sim1 > 0.95);

        // Different content should have lower similarity
        let sim2 = crawler.calculate_content_similarity(content1, content3);
        assert!(sim2 < 0.5);
    }
}

#[cfg(test)]
mod crawl_orchestration_tests {
    use super::*;

    #[tokio::test]
    async fn test_parallel_crawling_with_limits() {
        let config = CrawlConfig {
            max_concurrent: 3,
            max_pages: 10,
            timeout_per_page: Duration::from_secs(5),
            respect_robots_txt: true,
            ..Default::default()
        };

        let orchestrator = CrawlOrchestrator::new(config);

        // Create test URLs
        let mut urls = vec![];
        for i in 0..20 {
            urls.push(format!("https://example.com/page{}", i));
        }

        let start = std::time::Instant::now();
        let results = orchestrator.crawl_batch(urls).await;
        let duration = start.elapsed();

        // Should respect max_pages limit
        assert!(results.successful <= 10);

        // Should run concurrently (faster than sequential)
        assert!(duration.as_secs() < 20);
    }

    #[tokio::test]
    async fn test_crawl_with_robots_txt_compliance() {
        let orchestrator = CrawlOrchestrator::new(CrawlConfig {
            respect_robots_txt: true,
            ..Default::default()
        });

        // Mock robots.txt that disallows /private/
        orchestrator.set_robots_rules("example.com", vec![
            "User-agent: *",
            "Disallow: /private/",
            "Allow: /public/",
            "Crawl-delay: 1",
        ]).await;

        let allowed = orchestrator.can_crawl("https://example.com/public/page").await;
        let disallowed = orchestrator.can_crawl("https://example.com/private/data").await;

        assert!(allowed);
        assert!(!disallowed);
    }

    #[tokio::test]
    async fn test_crawl_rate_limiting() {
        let config = CrawlConfig {
            rate_limit_per_domain: Some(2), // 2 requests per second
            ..Default::default()
        };

        let orchestrator = CrawlOrchestrator::new(config);

        let urls = vec![
            "https://example.com/page1",
            "https://example.com/page2",
            "https://example.com/page3",
            "https://example.com/page4",
        ];

        let start = std::time::Instant::now();
        orchestrator.crawl_batch(urls.into_iter().map(String::from).collect()).await;
        let duration = start.elapsed();

        // Should take at least 2 seconds for 4 requests at 2/sec
        assert!(duration.as_secs() >= 2);
    }
}

#[cfg(test)]
mod url_frontier_tests {
    use super::*;

    #[test]
    fn test_url_frontier_prioritization() {
        let mut frontier = UrlFrontier::new(FrontierConfig {
            max_urls: 1000,
            priority_buckets: 5,
            ..Default::default()
        });

        // Add URLs with different priorities
        frontier.add("https://high-priority.com", 0.9);
        frontier.add("https://medium-priority.com", 0.5);
        frontier.add("https://low-priority.com", 0.1);

        // Should return highest priority first
        let next = frontier.pop().unwrap();
        assert_eq!(next, "https://high-priority.com");
    }

    #[test]
    fn test_url_deduplication() {
        let mut frontier = UrlFrontier::new(FrontierConfig::default());

        frontier.add("https://example.com/page", 0.5);
        frontier.add("https://example.com/page", 0.8); // Duplicate
        frontier.add("https://example.com/page/", 0.7); // Normalized duplicate

        assert_eq!(frontier.size(), 1);
    }

    #[test]
    fn test_url_normalization() {
        let frontier = UrlFrontier::new(FrontierConfig::default());

        let url1 = frontier.normalize("https://Example.COM/Page");
        let url2 = frontier.normalize("https://example.com/Page");
        let url3 = frontier.normalize("https://example.com/Page/");

        assert_eq!(url1, url2);
        assert_eq!(url2, url3);
    }
}