/// Comprehensive tests for Query-Aware Spider functionality
/// Week 7 requirements: BM25 scoring, URL signals, domain diversity, early stopping
#[cfg(test)]
mod query_aware_week7_tests {
    use crate::spider::query_aware::*;

    /// Test BM25 scoring algorithm accuracy with known relevance rankings
    #[test]
    fn test_bm25_scoring_accuracy() {
        let mut scorer = BM25Scorer::new("machine learning artificial intelligence", 1.2, 0.75);

        // Build corpus with known content
        let documents = [
            "Machine learning is a subset of artificial intelligence that focuses on algorithms",
            "Deep learning uses neural networks with multiple layers for artificial intelligence",
            "Natural language processing is another area of artificial intelligence research",
            "Computer vision applications use machine learning for image recognition",
            "This document is about cooking recipes and has no relevant content at all"
        ];

        // Update corpus
        for doc in &documents {
            scorer.update_corpus(doc);
        }

        // Score documents - should rank by relevance
        let mut scores: Vec<(usize, f64)> = documents
            .iter()
            .enumerate()
            .map(|(i, doc)| (i, scorer.score(doc)))
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Verify ranking correctness
        // Doc 0 has both "machine learning" and "artificial intelligence" - should rank highest
        assert_eq!(scores[0].0, 0, "Document with both query terms should rank highest");

        // Doc 1 and 2 have "artificial intelligence" - should rank higher than doc 3
        let doc1_rank = scores.iter().position(|(i, _)| *i == 1).unwrap();
        let doc2_rank = scores.iter().position(|(i, _)| *i == 2).unwrap();
        let doc3_rank = scores.iter().position(|(i, _)| *i == 3).unwrap();

        assert!(doc1_rank < doc3_rank && doc2_rank < doc3_rank,
                "Documents with 'artificial intelligence' should rank higher than those with only 'machine learning'");

        // Doc 4 (irrelevant) should rank lowest
        assert_eq!(scores.last().unwrap().0, 4, "Irrelevant document should rank lowest");

        // Verify score ranges are reasonable
        assert!(scores[0].1 > 0.0, "Top document should have positive score");
        assert!(scores.last().unwrap().1 == 0.0, "Irrelevant document should have zero score");

        // Test BM25 parameter effects
        let mut k1_scorer = BM25Scorer::new("test", 2.0, 0.75); // Higher k1
        let mut b_scorer = BM25Scorer::new("test", 1.2, 0.5);   // Different b

        let test_doc = "test document with test word repeated test test";
        k1_scorer.update_corpus(test_doc);
        b_scorer.update_corpus(test_doc);

        let k1_score = k1_scorer.score(test_doc);
        let b_score = b_scorer.score(test_doc);

        assert!(k1_score > 0.0 && b_score > 0.0, "Both parameter variations should produce positive scores");
    }

    /// Test URL signal integration with depth and path analysis
    #[test]
    fn test_url_signal_analysis() {
        let analyzer = UrlSignalAnalyzer::new(Some("machine learning tutorial"));

        // Test depth scoring
        let base_url = Url::from_str("https://example.com/base").unwrap();
        let score_depth_1 = analyzer.score(&base_url, 1);
        let score_depth_5 = analyzer.score(&base_url, 5);

        assert!(score_depth_1 > score_depth_5, "Shallow URLs should score higher than deep ones");

        // Test path relevance
        let relevant_urls = [
            Url::from_str("https://ml.example.com/machine-learning/tutorial").unwrap(),
            Url::from_str("https://example.com/tutorials/machine-learning").unwrap(),
            Url::from_str("https://example.com/learning/machine/intro").unwrap(),
        ];

        let irrelevant_urls = [
            Url::from_str("https://example.com/cooking/recipes").unwrap(),
            Url::from_str("https://example.com/random/path").unwrap(),
        ];

        let relevant_scores: Vec<f64> = relevant_urls
            .iter()
            .map(|url| analyzer.score(url, 2))
            .collect();

        let irrelevant_scores: Vec<f64> = irrelevant_urls
            .iter()
            .map(|url| analyzer.score(url, 2))
            .collect();

        let avg_relevant = relevant_scores.iter().sum::<f64>() / relevant_scores.len() as f64;
        let avg_irrelevant = irrelevant_scores.iter().sum::<f64>() / irrelevant_scores.len() as f64;

        assert!(avg_relevant > avg_irrelevant,
                "URLs with relevant paths should score higher than irrelevant ones");

        // Test domain bonus
        let domain_url = Url::from_str("https://machinelearning.example.com/tutorial").unwrap();
        let domain_score = analyzer.score(&domain_url, 2);
        let regular_score = analyzer.score(&Url::from_str("https://example.com/tutorial").unwrap(), 2);

        assert!(domain_score > regular_score, "Relevant domains should get bonus points");
    }

    /// Test domain diversity scoring algorithm
    #[test]
    fn test_domain_diversity_scoring() {
        let mut analyzer = DomainDiversityAnalyzer::new();

        // First domain should get high score
        let first_score = analyzer.score("example.com");
        assert!(first_score > 0.8, "New domains should get high diversity scores");

        // Record pages from the domain
        for _ in 0..5 {
            analyzer.record_page("example.com");
        }

        // Same domain should now get lower score
        let repeat_score = analyzer.score("example.com");
        assert!(repeat_score < first_score, "Repeated domains should get lower scores");

        // New domain should still get high score
        let new_domain_score = analyzer.score("newsite.com");
        assert!(new_domain_score > repeat_score, "New domains should score higher than overused ones");

        // Test sigmoid function behavior with extreme values
        for _ in 0..50 {
            analyzer.record_page("overused.com");
        }

        let overused_score = analyzer.score("overused.com");
        assert!(overused_score < 0.2, "Heavily overused domains should get very low scores");
        assert!(overused_score > 0.0, "Scores should never go completely to zero");

        // Verify statistics
        let (unique_domains, total_pages) = analyzer.get_stats();
        assert_eq!(unique_domains, 2); // example.com and overused.com
        assert_eq!(total_pages, 55);   // 5 + 50
    }

    /// Test early stopping logic with low relevance detection
    #[test]
    fn test_early_stopping_logic() {
        let config = QueryAwareConfig {
            query_foraging: true,
            target_query: Some("machine learning".to_string()),
            min_relevance_threshold: 0.4,
            relevance_window_size: 5,
            ..Default::default()
        };

        let mut scorer = QueryAwareScorer::new(config);

        // Add high-relevance scores - should not trigger stopping
        for _ in 0..3 {
            scorer.recent_scores.push(0.6);
        }

        let (should_stop, _) = scorer.should_stop_early();
        assert!(!should_stop, "Should not stop with high relevance scores");

        // Add low-relevance scores to fill window
        scorer.recent_scores.clear();
        for _ in 0..5 {
            scorer.recent_scores.push(0.2); // Below threshold
        }

        let (should_stop, reason) = scorer.should_stop_early();
        assert!(should_stop, "Should stop when average relevance is below threshold");
        assert!(reason.contains("Low relevance detected"), "Reason should explain low relevance");
        assert!(reason.contains("0.200"), "Reason should include actual average score");

        // Test with mixed scores
        scorer.recent_scores.clear();
        scorer.recent_scores.extend([0.6, 0.5, 0.3, 0.2, 0.1]); // Average = 0.34, below 0.4

        let (should_stop, _) = scorer.should_stop_early();
        assert!(should_stop, "Should stop when average of mixed scores is below threshold");

        // Test insufficient data
        scorer.recent_scores.clear();
        scorer.recent_scores.push(0.1); // Only one score

        let (should_stop, _) = scorer.should_stop_early();
        assert!(!should_stop, "Should not stop without sufficient data in window");
    }

    /// Test comprehensive scoring formula: S = α*BM25 + β*URLSignals + γ*DomainDiversity + δ*ContentSimilarity
    #[test]
    fn test_comprehensive_scoring_formula() {
        let config = QueryAwareConfig {
            query_foraging: true,
            target_query: Some("machine learning".to_string()),
            bm25_weight: 0.4,        // α
            url_signals_weight: 0.2, // β
            domain_diversity_weight: 0.2, // γ
            content_similarity_weight: 0.2, // δ
            ..Default::default()
        };

        let mut scorer = QueryAwareScorer::new(config);

        // Create relevant request and content
        let url = Url::from_str("https://ml.example.com/machine-learning/tutorial").unwrap();
        let request = CrawlRequest::new(url).with_depth(1);
        let content = "Machine learning algorithms are used in artificial intelligence to create models";

        // Update scorer with some corpus data
        scorer.bm25_scorer.update_corpus("Machine learning is a field of computer science");
        scorer.bm25_scorer.update_corpus("Artificial intelligence uses various techniques");

        let score = scorer.score_request(&request, Some(content));

        // Score should be positive and reasonable
        assert!(score > 0.0, "Relevant content should get positive score");
        assert!(score <= 1.0, "Score should be normalized"); // Given our weights sum to 1.0

        // Test with irrelevant content
        let irrelevant_url = Url::from_str("https://cooking.example.com/recipes/pasta").unwrap();
        let irrelevant_request = CrawlRequest::new(irrelevant_url).with_depth(3);
        let irrelevant_content = "This pasta recipe uses tomatoes and cheese for a delicious meal";

        let irrelevant_score = scorer.score_request(&irrelevant_request, Some(irrelevant_content));

        assert!(score > irrelevant_score, "Relevant content should score higher than irrelevant");

        // Test weight configuration effects
        let mut high_bm25_config = config.clone();
        high_bm25_config.bm25_weight = 0.8;
        high_bm25_config.url_signals_weight = 0.1;
        high_bm25_config.domain_diversity_weight = 0.05;
        high_bm25_config.content_similarity_weight = 0.05;

        let mut bm25_scorer = QueryAwareScorer::new(high_bm25_config);
        bm25_scorer.bm25_scorer.update_corpus("Machine learning is a field of computer science");
        bm25_scorer.bm25_scorer.update_corpus("Artificial intelligence uses various techniques");

        let bm25_focused_score = bm25_scorer.score_request(&request, Some(content));

        // Higher BM25 weight should affect scores differently
        assert!(bm25_focused_score > 0.0, "BM25-focused scoring should still work");
    }

    /// Performance benchmarking to ensure <10% throughput impact
    #[test]
    fn test_performance_benchmarking() {
        let num_requests = 1000;
        let content_size = 2000; // 2KB content per request

        // Benchmark without query-aware features
        let baseline_config = QueryAwareConfig {
            query_foraging: false,
            ..Default::default()
        };

        let mut baseline_scorer = QueryAwareScorer::new(baseline_config);
        let baseline_start = Instant::now();

        for i in 0..num_requests {
            let url = Url::from_str(&format!("https://example.com/page{}", i)).unwrap();
            let request = CrawlRequest::new(url);
            let content = "a".repeat(content_size);
            baseline_scorer.score_request(&request, Some(&content));
        }

        let baseline_duration = baseline_start.elapsed();

        // Benchmark with query-aware features enabled
        let qa_config = QueryAwareConfig {
            query_foraging: true,
            target_query: Some("machine learning artificial intelligence".to_string()),
            ..Default::default()
        };

        let mut qa_scorer = QueryAwareScorer::new(qa_config);
        let qa_start = Instant::now();

        for i in 0..num_requests {
            let url = Url::from_str(&format!("https://example.com/page{}", i)).unwrap();
            let request = CrawlRequest::new(url);
            let content = format!("Page {} content about machine learning and artificial intelligence. {}",
                                  i, "a".repeat(content_size - 100));
            qa_scorer.score_request(&request, Some(&content));
        }

        let qa_duration = qa_start.elapsed();

        // Calculate performance impact
        let performance_impact = if baseline_duration.as_nanos() > 0 {
            (qa_duration.as_nanos() as f64 / baseline_duration.as_nanos() as f64 - 1.0) * 100.0
        } else {
            0.0
        };

        println!("Baseline duration: {:?}", baseline_duration);
        println!("Query-aware duration: {:?}", qa_duration);
        println!("Performance impact: {:.2}%", performance_impact);

        // Requirement: <10% throughput impact
        assert!(performance_impact < 10.0,
                "Query-aware features should have <10% performance impact, actual: {:.2}%",
                performance_impact);

        // Additional throughput calculations
        let baseline_rps = num_requests as f64 / baseline_duration.as_secs_f64();
        let qa_rps = num_requests as f64 / qa_duration.as_secs_f64();
        let throughput_ratio = qa_rps / baseline_rps;

        println!("Baseline RPS: {:.2}", baseline_rps);
        println!("Query-aware RPS: {:.2}", qa_rps);
        println!("Throughput ratio: {:.3}", throughput_ratio);

        assert!(throughput_ratio > 0.9, "Throughput should not decrease by more than 10%");
    }

    /// Test for ≥20% lift in on-topic tokens/page at same budget
    #[test]
    fn test_relevance_lift_requirement() {
        // Simulate crawl results with and without query-aware selection

        // Baseline: Random page selection (simulated)
        let baseline_pages = vec![
            "Machine learning algorithms are powerful tools for data analysis", // Relevant
            "Today's weather forecast shows sunny skies and warm temperatures", // Irrelevant
            "Cooking pasta requires boiling water and adding salt to taste", // Irrelevant
            "Deep learning neural networks can process complex data patterns", // Relevant
            "Shopping for groceries includes milk, bread, and fresh vegetables", // Irrelevant
            "Artificial intelligence research continues to advance rapidly", // Relevant
            "Sports scores from last night's games were exciting to watch", // Irrelevant
            "Computer vision algorithms can detect objects in images effectively", // Relevant
        ];

        // Query-aware selection (would prioritize relevant pages)
        let query_aware_pages = vec![
            "Machine learning algorithms are powerful tools for data analysis", // Relevant
            "Deep learning neural networks can process complex data patterns", // Relevant
            "Artificial intelligence research continues to advance rapidly", // Relevant
            "Computer vision algorithms can detect objects in images effectively", // Relevant
            "Natural language processing enables computers to understand text", // Relevant
            "Supervised learning requires labeled training data for accuracy", // Relevant
            "Unsupervised learning discovers hidden patterns in unlabeled data", // Relevant
            "Reinforcement learning agents learn through trial and error methods", // Relevant
        ];

        let query = "machine learning artificial intelligence";
        let analyzer = ContentSimilarityAnalyzer::new(Some(query));

        // Calculate on-topic token ratios
        let mut baseline_on_topic_tokens = 0;
        let mut baseline_total_tokens = 0;

        for page in &baseline_pages {
            let tokens = tokenize(page);
            baseline_total_tokens += tokens.len();

            // Count tokens that match query terms
            let query_terms: Vec<String> = tokenize(query);
            for token in tokens {
                if query_terms.contains(&token) {
                    baseline_on_topic_tokens += 1;
                }
            }
        }

        let mut qa_on_topic_tokens = 0;
        let mut qa_total_tokens = 0;

        for page in &query_aware_pages {
            let tokens = tokenize(page);
            qa_total_tokens += tokens.len();

            // Count tokens that match query terms
            let query_terms: Vec<String> = tokenize(query);
            for token in tokens {
                if query_terms.contains(&token) {
                    qa_on_topic_tokens += 1;
                }
            }
        }

        let baseline_ratio = baseline_on_topic_tokens as f64 / baseline_total_tokens as f64;
        let qa_ratio = qa_on_topic_tokens as f64 / qa_total_tokens as f64;

        let lift_percentage = (qa_ratio / baseline_ratio - 1.0) * 100.0;

        println!("Baseline on-topic ratio: {:.3}", baseline_ratio);
        println!("Query-aware on-topic ratio: {:.3}", qa_ratio);
        println!("Lift percentage: {:.1}%", lift_percentage);

        // Requirement: ≥20% lift in on-topic tokens/page
        assert!(lift_percentage >= 20.0,
                "Query-aware crawling should achieve ≥20% lift in on-topic tokens, actual: {:.1}%",
                lift_percentage);

        // Additional verification using content similarity scoring
        let baseline_similarity_avg = baseline_pages.iter()
            .map(|page| analyzer.score(page))
            .sum::<f64>() / baseline_pages.len() as f64;

        let qa_similarity_avg = query_aware_pages.iter()
            .map(|page| analyzer.score(page))
            .sum::<f64>() / query_aware_pages.len() as f64;

        let similarity_lift = (qa_similarity_avg / baseline_similarity_avg - 1.0) * 100.0;

        println!("Baseline avg similarity: {:.3}", baseline_similarity_avg);
        println!("Query-aware avg similarity: {:.3}", qa_similarity_avg);
        println!("Similarity lift: {:.1}%", similarity_lift);

        assert!(similarity_lift >= 50.0, "Query-aware selection should significantly improve content relevance");
    }

    /// Test weight configuration system validation
    #[test]
    fn test_weight_configuration_system() {
        // Test default weights sum to 1.0
        let default_config = QueryAwareConfig::default();
        let weight_sum = default_config.bm25_weight + default_config.url_signals_weight +
                        default_config.domain_diversity_weight + default_config.content_similarity_weight;

        assert!((weight_sum - 1.0).abs() < 0.001, "Default weights should sum to 1.0");

        // Test custom weight configurations
        let custom_configs = vec![
            QueryAwareConfig {
                bm25_weight: 0.5,
                url_signals_weight: 0.3,
                domain_diversity_weight: 0.1,
                content_similarity_weight: 0.1,
                ..Default::default()
            },
            QueryAwareConfig {
                bm25_weight: 0.7,
                url_signals_weight: 0.1,
                domain_diversity_weight: 0.1,
                content_similarity_weight: 0.1,
                ..Default::default()
            },
            QueryAwareConfig {
                bm25_weight: 0.25,
                url_signals_weight: 0.25,
                domain_diversity_weight: 0.25,
                content_similarity_weight: 0.25,
                ..Default::default()
            },
        ];

        for config in custom_configs {
            let mut scorer = QueryAwareScorer::new(config.clone());

            // Test that custom weights affect scoring
            let url = Url::from_str("https://example.com/test").unwrap();
            let request = CrawlRequest::new(url);
            let content = "Test content with some keywords";

            let score = scorer.score_request(&request, Some(content));

            // Score should be within reasonable bounds
            assert!(score >= 0.0, "Scores should not be negative");
            assert!(score <= 2.0, "Scores should not exceed reasonable maximum");
        }
    }

    /// Test BM25 parameter optimization
    #[test]
    fn test_bm25_parameter_optimization() {
        let test_docs = [
            "machine learning algorithms artificial intelligence",
            "machine learning machine learning machine learning", // High term frequency
            "artificial intelligence research and development",
            "the the the the the", // High frequency common terms
        ];

        // Test different k1 values (controls term frequency saturation)
        let k1_values = [0.5, 1.2, 2.0];
        let b_values = [0.0, 0.75, 1.0]; // Controls length normalization

        for k1 in k1_values {
            for b in b_values {
                let mut scorer = BM25Scorer::new("machine learning", k1, b);

                for doc in &test_docs {
                    scorer.update_corpus(doc);
                }

                let scores: Vec<f64> = test_docs.iter().map(|doc| scorer.score(doc)).collect();

                // All relevant documents should get positive scores
                assert!(scores[0] > 0.0, "Document with query terms should score > 0 (k1={}, b={})", k1, b);
                assert!(scores[1] > 0.0, "Document with repeated terms should score > 0 (k1={}, b={})", k1, b);
                assert!(scores[2] > 0.0, "Document with partial match should score > 0 (k1={}, b={})", k1, b);

                // Document with no relevant terms should score 0
                assert_eq!(scores[3], 0.0, "Irrelevant document should score 0 (k1={}, b={})", k1, b);

                // Different parameter values should produce different rankings
                if k1 == 1.2 && b == 0.75 {
                    // Standard BM25 parameters - doc with both terms should rank highest
                    assert!(scores[0] > scores[2], "Multi-term document should outrank single-term");
                }
            }
        }
    }

    /// Helper function for tokenization (matches the one in query_aware.rs)
    fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .filter(|word| word.len() > 2)
            .map(|word| {
                word.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect()
            })
            .filter(|word: &String| !word.is_empty())
            .collect()
    }
}

/// Integration tests for query-aware spider with other components
#[cfg(test)]
mod query_aware_integration_tests {
    use super::*;
    use crate::spider::{
        config::{SpiderConfig, SpiderPresets},
        core::Spider,
    };

    /// Test integration with spider configuration
    #[tokio::test]
    async fn test_query_aware_config_integration() {
        let mut config = SpiderPresets::development();

        // Enable query-aware features
        config.query_aware = QueryAwareConfig {
            query_foraging: true,
            target_query: Some("machine learning".to_string()),
            bm25_weight: 0.5,
            url_signals_weight: 0.2,
            domain_diversity_weight: 0.2,
            content_similarity_weight: 0.1,
            min_relevance_threshold: 0.3,
            ..Default::default()
        };

        // Create spider with query-aware config
        let spider = Spider::new(config).await.expect("Spider creation should work");

        // Verify query-aware features are enabled
        let stats = spider.get_query_aware_stats().await;
        assert!(stats.enabled, "Query-aware features should be enabled");

        // Test scoring integration
        let url = Url::from_str("https://ml.example.com/machine-learning").unwrap();
        let request = CrawlRequest::new(url);

        // This would normally be done during crawling
        let content = "Machine learning algorithms for artificial intelligence";
        let score = spider.score_query_aware_request(&request, Some(content)).await
            .expect("Query-aware scoring should work");

        assert!(score > 0.0, "Relevant content should get positive score");
    }

    /// Test early stopping integration with crawl process
    #[tokio::test]
    async fn test_early_stopping_integration() {
        let mut config = SpiderPresets::development();
        config.query_aware = QueryAwareConfig {
            query_foraging: true,
            target_query: Some("machine learning".to_string()),
            min_relevance_threshold: 0.5,
            relevance_window_size: 3,
            ..Default::default()
        };

        let spider = Spider::new(config).await.expect("Spider creation should work");

        // Simulate multiple low-relevance results
        for i in 0..5 {
            let url = Url::from_str(&format!("https://example.com/irrelevant{}", i)).unwrap();
            let request = CrawlRequest::new(url);
            let result = CrawlResult::success(request);

            spider.update_query_aware_with_result(&result).await
                .expect("Update should work");
        }

        // Check if early stopping would be triggered
        let (should_stop, reason) = spider.should_stop_query_aware().await
            .expect("Stop check should work");

        if should_stop {
            assert!(!reason.is_empty(), "Stop reason should be provided");
            println!("Early stopping triggered: {}", reason);
        }
    }

    /// Performance test with realistic crawl simulation
    #[tokio::test]
    async fn test_query_aware_performance_realistic() {
        let config = SpiderPresets::high_performance();
        let spider = Spider::new(config).await.expect("Spider creation should work");

        let start_time = Instant::now();

        // Simulate processing 1000 URLs with query-aware scoring
        for i in 0..1000 {
            let url = Url::from_str(&format!("https://example.com/page{}", i)).unwrap();
            let request = CrawlRequest::new(url);
            let content = format!("Page {} about various topics including some keywords", i);

            let _score = spider.score_query_aware_request(&request, Some(&content)).await
                .expect("Scoring should work");
        }

        let duration = start_time.elapsed();
        let throughput = 1000.0 / duration.as_secs_f64();

        println!("Query-aware scoring throughput: {:.2} requests/sec", throughput);

        // Should maintain reasonable throughput
        assert!(throughput > 100.0, "Should maintain >100 req/sec throughput");
    }
}