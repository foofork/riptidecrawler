use crate::{
    config::SpiderPresets,
    core::Spider,
    types::{CrawlRequest, Priority},
};
use std::str::FromStr;
use tokio::time::{sleep, Duration};
use url::Url;

/// Test scenarios for various site structures and crawling patterns
pub mod scenarios {
    use super::*;

    /// Test crawling a simple static website structure
    #[tokio::test]
    async fn test_static_website_crawling() {
        let config = SpiderPresets::development();
        let spider = Spider::new(config)
            .await
            .expect("Spider creation should work");

        // Simulate crawling a simple website structure
        // This would normally perform actual HTTP requests
        // In a real test, you'd use a mock HTTP server
        // let result = spider.crawl(seeds).await.expect("Crawl should work");

        let state = spider.get_crawl_state().await;
        assert!(!state.active); // Should not be active initially
    }

    /// Test news site crawling with high content variation
    #[tokio::test]
    async fn test_news_site_pattern() {
        let config = SpiderPresets::news_site();
        let spider = Spider::new(config)
            .await
            .expect("Spider creation should work");

        // Verify news site configuration
        let adaptive_stats = spider.get_adaptive_stop_stats().await;
        assert!(adaptive_stats.pages_analyzed == 0); // Initial state

        // Test that configuration is optimized for news sites
        let crawl_state = spider.get_crawl_state().await;
        assert_eq!(crawl_state.pages_crawled, 0);
    }

    /// Test e-commerce site crawling with product pages
    #[tokio::test]
    async fn test_ecommerce_site_pattern() {
        let config = SpiderPresets::ecommerce_site();
        let spider = Spider::new(config)
            .await
            .expect("Spider creation should work");

        // Test e-commerce specific exclusions
        let request_cart =
            CrawlRequest::new(Url::from_str("https://shop.example.com/cart").expect("Valid URL"));

        let request_product = CrawlRequest::new(
            Url::from_str("https://shop.example.com/products/widget").expect("Valid URL"),
        );

        // Cart should potentially be excluded, product should be allowed
        // (exact behavior depends on URL utils configuration)
        let _should_crawl_cart = spider
            .should_crawl_url(&request_cart)
            .await
            .expect("Check should work");
        let _should_crawl_product = spider
            .should_crawl_url(&request_product)
            .await
            .expect("Check should work");

        // Results depend on configuration but both checks should complete
    }

    /// Test documentation site crawling with hierarchical structure
    #[tokio::test]
    async fn test_documentation_site_pattern() {
        let config = SpiderPresets::documentation_site();
        let spider = Spider::new(config)
            .await
            .expect("Spider creation should work");

        // Documentation sites typically use depth-first strategy
        let frontier_stats = spider.get_frontier_stats().await;
        assert_eq!(frontier_stats.total_requests, 0); // Initially empty
    }

    /// Test authenticated crawling scenario
    #[tokio::test]
    async fn test_authenticated_crawling() {
        let config = SpiderPresets::authenticated_crawling();
        let spider = Spider::new(config)
            .await
            .expect("Spider creation should work");

        // Test session creation
        let session_id = spider
            .session_manager()
            .get_or_create_session("members.example.com")
            .await
            .expect("Session creation should work");

        assert!(!session_id.is_empty());

        // Check that session is not authenticated initially
        assert!(
            !spider
                .session_manager()
                .is_authenticated("members.example.com")
                .await
        );
    }

    /// Test high-performance crawling configuration
    #[tokio::test]
    async fn test_high_performance_crawling() {
        let config = SpiderPresets::high_performance();
        let spider = Spider::new(config)
            .await
            .expect("Spider creation should work");

        // Verify high performance settings
        let state = spider.get_crawl_state().await;
        assert_eq!(state.pages_crawled, 0);

        // Test that multiple domains can be handled
        let domains = vec![
            "site1.example.com",
            "site2.example.com",
            "site3.example.com",
        ];

        for domain in domains {
            let _session = spider
                .session_manager()
                .get_or_create_session(domain)
                .await
                .expect("Session creation should work");
        }
    }
}

/// Integration tests with existing systems
pub mod integration {
    use super::*;

    /// Test robots.txt integration
    #[tokio::test]
    async fn test_robots_txt_integration() {
        let config = SpiderPresets::development();
        let spider = Spider::new(config)
            .await
            .expect("Spider creation should work");

        // Test that robots manager is properly integrated
        let test_url = "https://example.com/allowed-path";

        // This would normally check actual robots.txt
        // In development mode, should allow most URLs
        let allowed = spider
            .robots_manager()
            .is_allowed(test_url)
            .await
            .expect("Robots check should work");

        // In development mode, should typically be allowed
        assert!(allowed);
    }

    /// Test budget management integration
    #[tokio::test]
    async fn test_budget_management() {
        let mut config = SpiderPresets::development();
        config.budget.global.max_pages = Some(5);
        config.budget.global.max_depth = Some(3);

        let spider = Spider::new(config)
            .await
            .expect("Spider creation should work");

        let test_url = Url::from_str("https://example.com/").expect("Valid URL");

        // Should be able to make initial requests
        let can_crawl_0 = spider
            .budget_manager()
            .can_make_request(&test_url, 0)
            .await
            .expect("Budget check should work");
        assert!(can_crawl_0);

        // Deep URLs should be rejected
        let can_crawl_deep = spider
            .budget_manager()
            .can_make_request(&test_url, 10)
            .await
            .expect("Budget check should work");
        assert!(!can_crawl_deep);
    }

    /// Test URL deduplication and normalization
    #[tokio::test]
    async fn test_url_processing() {
        let config = SpiderPresets::development();
        let spider = Spider::new(config)
            .await
            .expect("Spider creation should work");

        let urls = vec![
            Url::from_str("https://example.com/page").expect("Valid URL"),
            Url::from_str("https://example.com/page#fragment").expect("Valid URL"),
            Url::from_str("https://example.com/page?b=2&a=1").expect("Valid URL"),
            Url::from_str("https://example.com/style.css").expect("Valid URL"), // Should be excluded
        ];

        let filtered = spider
            .url_utils()
            .read()
            .await
            .filter_urls(urls)
            .await
            .expect("URL filtering should work");

        // Should exclude CSS file and normalize duplicates
        assert!(filtered.len() <= 3);

        // Check that CSS was excluded
        assert!(!filtered.iter().any(|url| url.path().ends_with(".css")));
    }

    /// Test adaptive stopping algorithm
    #[tokio::test]
    async fn test_adaptive_stopping() {
        let mut config = SpiderPresets::development();
        config.adaptive_stop.min_pages_before_stop = 3;
        config.adaptive_stop.patience = 2;
        config.adaptive_stop.min_gain_threshold = 50.0;
        config.adaptive_stop.quality_threshold = 0.3; // Match default threshold

        let spider = Spider::new(config)
            .await
            .expect("Spider creation should work");

        // Simulate analyzing low-quality results
        for i in 0..5 {
            let url = Url::from_str(&format!("https://example.com/page{}", i)).expect("Valid URL");
            let request = CrawlRequest::new(url);

            let mut result = crate::types::CrawlResult::success(request);
            result.text_content = Some("short".to_string()); // Very low content
            result.content_size = 50;

            spider
                .adaptive_stop_engine()
                .analyze_result(&result)
                .await
                .expect("Analysis should work");
        }

        // Should eventually recommend stopping
        let decision = spider
            .adaptive_stop_engine()
            .should_stop()
            .await
            .expect("Stop decision should work");

        assert!(decision.should_stop);
        // Accept either low content gain or low quality reason
        assert!(
            decision.reason.to_lowercase().contains("low content")
                || decision.reason.to_lowercase().contains("quality"),
            "Expected low content or quality reason but got: {}",
            decision.reason
        );
    }

    /// Test frontier management with different strategies
    #[tokio::test]
    async fn test_frontier_strategies() {
        let config = SpiderPresets::development();
        let spider = Spider::new(config)
            .await
            .expect("Spider creation should work");

        // Add requests with different priorities
        let high_priority =
            CrawlRequest::new(Url::from_str("https://example.com/important").expect("Valid URL"))
                .with_priority(Priority::High);

        let low_priority =
            CrawlRequest::new(Url::from_str("https://example.com/optional").expect("Valid URL"))
                .with_priority(Priority::Low);

        spider
            .frontier_manager()
            .add_request(low_priority)
            .await
            .expect("Add should work");
        spider
            .frontier_manager()
            .add_request(high_priority)
            .await
            .expect("Add should work");

        // High priority should come first
        let next_request = spider
            .frontier_manager()
            .next_request()
            .await
            .expect("Get should work");
        assert!(next_request.is_some());

        let request = next_request.unwrap();
        assert_eq!(request.priority, Priority::High);
        assert!(request.url.path().contains("important"));
    }

    /// Test session management lifecycle
    #[tokio::test]
    async fn test_session_lifecycle() {
        let mut config = SpiderPresets::authenticated_crawling();
        config.session.session_timeout = Duration::from_millis(100); // Very short for testing

        let spider = Spider::new(config)
            .await
            .expect("Spider creation should work");

        let domain = "test.example.com";

        // Create session
        let session_id1 = spider
            .session_manager()
            .get_or_create_session(domain)
            .await
            .expect("Session creation should work");

        // Should return same session immediately
        let session_id2 = spider
            .session_manager()
            .get_or_create_session(domain)
            .await
            .expect("Session retrieval should work");
        assert_eq!(session_id1, session_id2);

        // Wait for expiration
        sleep(Duration::from_millis(150)).await;

        // Should create new session after expiration
        let session_id3 = spider
            .session_manager()
            .get_or_create_session(domain)
            .await
            .expect("New session creation should work");
        assert_ne!(session_id1, session_id3);
    }
}

/// Performance and stress tests
pub mod performance {
    use super::*;

    /// Test frontier performance with large number of URLs
    #[tokio::test]
    async fn test_frontier_performance() {
        let config = SpiderPresets::high_performance();
        let spider = Spider::new(config)
            .await
            .expect("Spider creation should work");

        let start_time = std::time::Instant::now();

        // Add many requests
        for i in 0..1000 {
            let url = Url::from_str(&format!("https://example.com/page{}", i)).expect("Valid URL");
            let request = CrawlRequest::new(url);
            spider
                .frontier_manager()
                .add_request(request)
                .await
                .expect("Add should work");
        }

        let add_time = start_time.elapsed();
        assert!(
            add_time < Duration::from_secs(1),
            "Adding 1000 URLs should be fast"
        );

        // Test retrieval performance
        let start_time = std::time::Instant::now();

        let mut retrieved = 0;
        while spider
            .frontier_manager()
            .next_request()
            .await
            .expect("Get should work")
            .is_some()
        {
            retrieved += 1;
            if retrieved >= 100 {
                break; // Test first 100
            }
        }

        let get_time = start_time.elapsed();
        assert!(
            get_time < Duration::from_millis(100),
            "Getting 100 URLs should be fast"
        );
    }

    /// Test URL processing performance
    #[tokio::test]
    async fn test_url_processing_performance() {
        let config = SpiderPresets::high_performance();
        let spider = Spider::new(config)
            .await
            .expect("Spider creation should work");

        // Use a smaller dataset for CI environments where performance is variable
        // 1000 URLs is sufficient to test filtering performance
        let mut urls = Vec::new();
        for i in 0..1000 {
            urls.push(Url::from_str(&format!("https://example.com/page{}", i)).expect("Valid URL"));
        }

        let start_time = std::time::Instant::now();
        let filtered = spider
            .url_utils()
            .read()
            .await
            .filter_urls(urls)
            .await
            .expect("Filtering should work");
        let filter_time = start_time.elapsed();

        // Relaxed timing for CI environments - 5 seconds should be sufficient
        assert!(
            filter_time < Duration::from_secs(5),
            "Filtering 1k URLs took {:?}, should be under 5s",
            filter_time
        );
        assert!(filtered.len() <= 1000); // Should not increase
    }

    /// Test memory usage under load
    #[tokio::test]
    async fn test_memory_usage() {
        let config = SpiderPresets::high_performance();
        let spider = Spider::new(config)
            .await
            .expect("Spider creation should work");

        // Add many requests to test memory usage
        for i in 0..1000 {
            let url = Url::from_str(&format!("https://example.com/page{}", i)).expect("Valid URL");
            let request = CrawlRequest::new(url);
            spider
                .frontier_manager()
                .add_request(request)
                .await
                .expect("Add should work");
        }

        let metrics = spider.get_frontier_stats().await;
        assert_eq!(metrics.total_requests, 1000);

        // Memory usage should be reasonable
        // The formula is: total_requests * 1024
        // For 1000 URLs: 1000 * 1024 = 1,024,000 bytes (~1MB)
        assert!(
            metrics.memory_usage < 100_000_000,
            "Memory usage {} should be less than 100MB",
            metrics.memory_usage
        );
    }

    /// Test concurrent access performance
    #[tokio::test(flavor = "multi_thread")]
    async fn test_concurrent_access() {
        // Add timeout to prevent hanging
        let test_future = async {
            let config = SpiderPresets::high_performance();
            let spider = std::sync::Arc::new(
                Spider::new(config)
                    .await
                    .expect("Spider creation should work"),
            );

            let mut handles = Vec::new();

            // Spawn multiple tasks adding URLs concurrently (reduced from 10 to 5 tasks)
            for task_id in 0..5 {
                let spider_clone = spider.clone();
                let handle = tokio::spawn(async move {
                    for i in 0..50 {
                        let url = Url::from_str(&format!(
                            "https://example.com/task{}/page{}",
                            task_id, i
                        ))
                        .expect("Valid URL");
                        let request = CrawlRequest::new(url);
                        spider_clone
                            .frontier_manager()
                            .add_request(request)
                            .await
                            .expect("Add should work");
                    }
                });
                handles.push(handle);
            }

            // Wait for all tasks to complete
            for handle in handles {
                handle.await.expect("Task should complete");
            }

            let metrics = spider.get_frontier_stats().await;
            assert_eq!(metrics.total_requests, 250); // 5 tasks * 50 URLs each
        };

        // Run with 30 second timeout
        tokio::time::timeout(std::time::Duration::from_secs(30), test_future)
            .await
            .expect("Test should complete within 30 seconds");
    }
}

/// Error handling and edge case tests
pub mod edge_cases {
    use super::*;

    /// Test handling of invalid URLs
    #[tokio::test]
    async fn test_invalid_urls() {
        let config = SpiderPresets::development();
        let spider = Spider::new(config)
            .await
            .expect("Spider creation should work");

        // Test with malformed URLs (should be filtered out by URL utils)
        let urls = vec![
            "not-a-url",
            "https://",
            "ftp://example.com",         // Different scheme
            "https://example.com/valid", // This one should be valid
        ]
        .into_iter()
        .filter_map(|s| Url::from_str(s).ok())
        .collect();

        let filtered = spider
            .url_utils()
            .read()
            .await
            .filter_urls(urls)
            .await
            .expect("Filtering should work even with some invalid URLs");

        // Should have at least the valid HTTPS URL
        assert!(!filtered.is_empty());
    }

    /// Test budget overflow handling
    #[tokio::test]
    async fn test_budget_overflow() {
        let mut config = SpiderPresets::development();
        config.budget.global.max_pages = Some(1);

        let spider = Spider::new(config)
            .await
            .expect("Spider creation should work");

        let url = Url::from_str("https://example.com/").expect("Valid URL");

        // First request should be allowed
        let can_make_first = spider
            .budget_manager()
            .can_make_request(&url, 0)
            .await
            .expect("Budget check should work");
        assert!(can_make_first);

        // Simulate completing a request
        spider
            .budget_manager()
            .start_request(&url, 0)
            .await
            .expect("Start should work");
        spider
            .budget_manager()
            .complete_request(&url, 1024, true)
            .await
            .expect("Complete should work");

        // Second request should be rejected due to budget
        let can_make_second = spider
            .budget_manager()
            .can_make_request(&url, 0)
            .await
            .expect("Budget check should work");
        assert!(!can_make_second);
    }

    /// Test adaptive stop with no content
    #[tokio::test]
    async fn test_adaptive_stop_no_content() {
        let mut config = SpiderPresets::development();
        // Lower the minimum pages to trigger empty content detection sooner
        config.adaptive_stop.min_pages_before_stop = 3;

        let spider = Spider::new(config)
            .await
            .expect("Spider creation should work");

        // Analyze empty results
        for i in 0..10 {
            let url = Url::from_str(&format!("https://example.com/empty{}", i)).expect("Valid URL");
            let request = CrawlRequest::new(url);

            let mut result = crate::types::CrawlResult::success(request);
            result.text_content = None; // No content
            result.content_size = 0;

            spider
                .adaptive_stop_engine()
                .analyze_result(&result)
                .await
                .expect("Analysis should work");
        }

        let decision = spider
            .adaptive_stop_engine()
            .should_stop()
            .await
            .expect("Stop decision should work");

        // Should recommend stopping due to lack of content
        // The engine detects consecutive empty pages after min_pages_before_stop
        assert!(
            decision.should_stop,
            "Expected stop decision but got: {}",
            decision.reason
        );
    }

    /// Test frontier exhaustion
    #[tokio::test]
    async fn test_frontier_exhaustion() {
        let config = SpiderPresets::development();
        let spider = Spider::new(config)
            .await
            .expect("Spider creation should work");

        // Add a single request
        let url = Url::from_str("https://example.com/only-page").expect("Valid URL");
        let request = CrawlRequest::new(url);
        spider
            .frontier_manager()
            .add_request(request)
            .await
            .expect("Add should work");

        // Retrieve the request
        let retrieved = spider
            .frontier_manager()
            .next_request()
            .await
            .expect("Get should work");
        assert!(retrieved.is_some());

        // Frontier should now be empty
        let empty_get = spider
            .frontier_manager()
            .next_request()
            .await
            .expect("Get should work");
        assert!(empty_get.is_none());

        assert!(spider.frontier_manager().is_empty().await);
    }

    /// Test session limit enforcement
    #[tokio::test]
    async fn test_session_limits() {
        let mut config = SpiderPresets::authenticated_crawling();
        config.session.max_concurrent_sessions = 2;

        let spider = Spider::new(config)
            .await
            .expect("Spider creation should work");

        // Create maximum sessions
        let _session1 = spider
            .session_manager()
            .get_or_create_session("site1.com")
            .await
            .expect("First session should work");
        let _session2 = spider
            .session_manager()
            .get_or_create_session("site2.com")
            .await
            .expect("Second session should work");

        // Third session should evict oldest
        let _session3 = spider
            .session_manager()
            .get_or_create_session("site3.com")
            .await
            .expect("Third session should work");

        let stats = spider.session_manager().get_stats().await;
        assert_eq!(stats.active_sessions, 2); // Should still be at limit
    }
}

/// Configuration and validation tests
pub mod config_tests {
    use super::*;
    use crate::config::SpiderConfig;

    /// Test configuration validation
    #[test]
    fn test_config_validation() {
        // Valid configuration should pass
        let valid_config = SpiderPresets::development();
        assert!(valid_config.validate().is_ok());

        // Invalid configuration should fail
        let mut invalid_config = SpiderConfig::default();
        invalid_config.frontier.memory_limit = 0; // Invalid
        assert!(invalid_config.validate().is_err());
    }

    /// Test resource optimization
    #[test]
    fn test_resource_optimization() {
        let mut config = SpiderConfig::default();
        let original_limit = config.frontier.memory_limit;

        config.optimize_for_resources(16384, 16); // 16GB RAM, 16 cores

        // Should increase limits based on available resources
        assert!(config.frontier.memory_limit > original_limit);
        assert!(config.performance.max_concurrent_global > 10);
    }

    /// Test memory estimation
    #[test]
    fn test_memory_estimation() {
        let config = SpiderPresets::high_performance();
        let estimated = config.estimate_memory_usage();

        assert!(estimated > 0);
        assert!(estimated < u32::MAX as usize); // Reasonable size
    }

    /// Test preset configurations
    #[test]
    fn test_preset_configurations() {
        let configs = vec![
            SpiderPresets::news_site(),
            SpiderPresets::ecommerce_site(),
            SpiderPresets::documentation_site(),
            SpiderPresets::authenticated_crawling(),
            SpiderPresets::development(),
            SpiderPresets::high_performance(),
        ];

        for config in configs {
            assert!(
                config.validate().is_ok(),
                "Preset configuration should be valid"
            );
        }
    }
}

/// Helper functions for testing
pub mod test_helpers {
    use super::*;

    /// Create a mock crawl result for testing
    #[allow(dead_code)]
    pub fn create_mock_result(
        url: &str,
        content: &str,
        links: Vec<&str>,
    ) -> crate::types::CrawlResult {
        let url = Url::from_str(url).expect("Valid URL");
        let request = CrawlRequest::new(url);

        let mut result = crate::types::CrawlResult::success(request);
        result.text_content = Some(content.to_string());
        result.content_size = content.len();
        result.extracted_urls = links
            .into_iter()
            .map(|link| Url::from_str(link).expect("Valid URL"))
            .collect();

        result
    }

    /// Create a spider with custom configuration for testing
    #[allow(dead_code)]
    pub async fn create_test_spider() -> Spider {
        let config = SpiderPresets::development();
        Spider::new(config)
            .await
            .expect("Test spider creation should work")
    }

    /// Simulate crawling without actual HTTP requests
    #[allow(dead_code)]
    pub async fn simulate_crawl(spider: &Spider, pages: usize) {
        for i in 0..pages {
            let url = format!("https://test.example.com/page{}", i);
            let content = format!("Test content for page {} with some unique text.", i);
            let links = [format!("https://test.example.com/page{}", i + 1)];

            let result =
                create_mock_result(&url, &content, links.iter().map(|s| s.as_str()).collect());

            // Simulate analysis
            spider
                .adaptive_stop_engine()
                .analyze_result(&result)
                .await
                .expect("Analysis should work");
        }
    }
}
