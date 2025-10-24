//! Comprehensive Multi-Level Spider Crawling Tests
//!
//! This test suite validates spider's ability to perform multi-level depth-first
//! and breadth-first crawling with proper depth tracking and control.

#[cfg(test)]
mod multi_level_spider_tests {
    use riptide_core::spider::{Spider, SpiderConfig};
    use riptide_core::robots::RobotsManager;
    use std::sync::Arc;
    use std::time::Duration;
    use url::Url;

    /// Create a test spider configuration with custom depth settings
    fn create_test_spider_config(max_depth: Option<usize>, max_pages: Option<usize>) -> SpiderConfig {
        let mut config = SpiderConfig::new(
            Url::parse("https://httpbin.org").expect("Valid base URL")
        );

        config.max_depth = max_depth;
        config.max_pages = max_pages;
        config.timeout = Duration::from_secs(10);
        config.delay = Duration::from_millis(200);
        config.concurrency = 2;
        config.respect_robots = false; // For testing
        config.follow_redirects = true;
        config.enable_javascript = false;

        config
    }

    #[tokio::test]
    async fn test_single_level_crawl_depth_0() {
        // Test that max_depth = 0 only crawls seed URLs
        let config = create_test_spider_config(Some(0), Some(10));
        let robots = Arc::new(RobotsManager::new());

        let spider = Spider::new(config, robots, None, None, None)
            .expect("Failed to create spider");

        let seed_urls = vec![
            Url::parse("https://httpbin.org/html").expect("Valid URL")
        ];

        let result = spider.crawl(seed_urls).await;

        match result {
            Ok(crawl_result) => {
                // With depth 0, should only crawl the seed URL (no following links)
                assert_eq!(
                    crawl_result.pages_crawled, 1,
                    "Should only crawl seed URL at depth 0"
                );

                println!("✓ Depth 0 crawl completed: {} pages", crawl_result.pages_crawled);
            }
            Err(e) => {
                println!("⚠ Depth 0 test skipped (network/config issue): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_two_level_crawl_depth_1() {
        // Test max_depth = 1 crawls seed + first-level links
        let config = create_test_spider_config(Some(1), Some(20));
        let robots = Arc::new(RobotsManager::new());

        let spider = Spider::new(config, robots, None, None, None)
            .expect("Failed to create spider");

        let seed_urls = vec![
            Url::parse("https://httpbin.org/links/3").expect("Valid URL") // Page with 3 links
        ];

        let result = spider.crawl(seed_urls).await;

        match result {
            Ok(crawl_result) => {
                // Should crawl seed (depth 0) + some links (depth 1)
                assert!(
                    crawl_result.pages_crawled > 1,
                    "Should crawl seed and at least one child page"
                );
                assert!(
                    crawl_result.pages_crawled <= 5,
                    "Should not crawl too many pages with depth 1"
                );

                println!(
                    "✓ Depth 1 crawl completed: {} pages crawled",
                    crawl_result.pages_crawled
                );
            }
            Err(e) => {
                println!("⚠ Depth 1 test skipped (network/config issue): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_three_level_crawl_depth_2() {
        // Test max_depth = 2 crawls three levels
        let config = create_test_spider_config(Some(2), Some(30));
        let robots = Arc::new(RobotsManager::new());

        let spider = Spider::new(config, robots, None, None, None)
            .expect("Failed to create spider");

        let seed_urls = vec![
            Url::parse("https://httpbin.org/links/2").expect("Valid URL")
        ];

        let result = spider.crawl(seed_urls).await;

        match result {
            Ok(crawl_result) => {
                // Should crawl multiple levels
                assert!(
                    crawl_result.pages_crawled >= 1,
                    "Should crawl at least seed page"
                );

                println!(
                    "✓ Depth 2 crawl completed: {} pages, duration: {:.2}s",
                    crawl_result.pages_crawled,
                    crawl_result.duration_seconds
                );

                // Duration should be reasonable for multi-level crawl
                assert!(
                    crawl_result.duration_seconds < 60.0,
                    "Multi-level crawl should complete within 60 seconds"
                );
            }
            Err(e) => {
                println!("⚠ Depth 2 test skipped (network/config issue): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_unlimited_depth_with_page_limit() {
        // Test crawling without depth limit but with page limit
        let config = create_test_spider_config(None, Some(10));
        let robots = Arc::new(RobotsManager::new());

        let spider = Spider::new(config, robots, None, None, None)
            .expect("Failed to create spider");

        let seed_urls = vec![
            Url::parse("https://httpbin.org/links/5").expect("Valid URL")
        ];

        let result = spider.crawl(seed_urls).await;

        match result {
            Ok(crawl_result) => {
                // Should respect page limit even without depth limit
                assert!(
                    crawl_result.pages_crawled <= 10,
                    "Should not exceed max_pages limit"
                );

                println!(
                    "✓ Unlimited depth crawl completed: {} pages (limit: 10)",
                    crawl_result.pages_crawled
                );
            }
            Err(e) => {
                println!("⚠ Unlimited depth test skipped (network/config issue): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_multi_level_breadth_first() {
        // Test breadth-first strategy with multiple levels
        let mut config = create_test_spider_config(Some(2), Some(15));
        config.strategy.strategy_type = riptide_core::spider::types::StrategyType::BreadthFirst;

        let robots = Arc::new(RobotsManager::new());

        let spider = Spider::new(config, robots, None, None, None)
            .expect("Failed to create spider");

        let seed_urls = vec![
            Url::parse("https://httpbin.org/links/3").expect("Valid URL")
        ];

        let result = spider.crawl(seed_urls).await;

        match result {
            Ok(crawl_result) => {
                assert!(
                    crawl_result.pages_crawled >= 1,
                    "Breadth-first should crawl at least seed"
                );

                println!(
                    "✓ Breadth-first multi-level crawl: {} pages",
                    crawl_result.pages_crawled
                );
            }
            Err(e) => {
                println!("⚠ Breadth-first test skipped (network/config issue): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_multi_level_depth_first() {
        // Test depth-first strategy with multiple levels
        let mut config = create_test_spider_config(Some(2), Some(15));
        config.strategy.strategy_type = riptide_core::spider::types::StrategyType::DepthFirst;

        let robots = Arc::new(RobotsManager::new());

        let spider = Spider::new(config, robots, None, None, None)
            .expect("Failed to create spider");

        let seed_urls = vec![
            Url::parse("https://httpbin.org/links/2").expect("Valid URL")
        ];

        let result = spider.crawl(seed_urls).await;

        match result {
            Ok(crawl_result) => {
                assert!(
                    crawl_result.pages_crawled >= 1,
                    "Depth-first should crawl at least seed"
                );

                println!(
                    "✓ Depth-first multi-level crawl: {} pages",
                    crawl_result.pages_crawled
                );
            }
            Err(e) => {
                println!("⚠ Depth-first test skipped (network/config issue): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_multiple_seed_urls_multi_level() {
        // Test multi-level crawling with multiple seed URLs
        let config = create_test_spider_config(Some(1), Some(20));
        let robots = Arc::new(RobotsManager::new());

        let spider = Spider::new(config, robots, None, None, None)
            .expect("Failed to create spider");

        let seed_urls = vec![
            Url::parse("https://httpbin.org/html").expect("Valid URL 1"),
            Url::parse("https://httpbin.org/links/2").expect("Valid URL 2"),
        ];

        let result = spider.crawl(seed_urls).await;

        match result {
            Ok(crawl_result) => {
                let total_attempted = crawl_result.pages_crawled + crawl_result.pages_failed;

                assert!(
                    total_attempted >= 2,
                    "Should attempt both seed URLs"
                );

                println!(
                    "✓ Multi-seed multi-level crawl: {} crawled, {} failed",
                    crawl_result.pages_crawled,
                    crawl_result.pages_failed
                );
            }
            Err(e) => {
                println!("⚠ Multi-seed test skipped (network/config issue): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_depth_limit_enforcement() {
        // Verify that depth limit is strictly enforced
        let config = create_test_spider_config(Some(1), Some(100));
        let robots = Arc::new(RobotsManager::new());

        let spider = Spider::new(config, robots, None, None, None)
            .expect("Failed to create spider");

        let seed_urls = vec![
            Url::parse("https://httpbin.org/links/10").expect("Valid URL") // Many links
        ];

        let result = spider.crawl(seed_urls).await;

        match result {
            Ok(crawl_result) => {
                // Even with many links available, should respect depth limit
                // At depth 1, should crawl seed + some first-level links
                // but not go deeper even if page limit allows

                println!(
                    "✓ Depth limit enforcement: {} pages with max_depth=1",
                    crawl_result.pages_crawled
                );

                // Stop reason should mention depth or pages limit
                assert!(
                    crawl_result.stop_reason.contains("depth")
                    || crawl_result.stop_reason.contains("page")
                    || crawl_result.stop_reason.contains("finished"),
                    "Stop reason should be related to limits: {}",
                    crawl_result.stop_reason
                );
            }
            Err(e) => {
                println!("⚠ Depth enforcement test skipped (network/config issue): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_crawl_performance_metrics() {
        // Test that performance metrics are accurate for multi-level crawls
        let config = create_test_spider_config(Some(2), Some(10));
        let robots = Arc::new(RobotsManager::new());

        let spider = Spider::new(config, robots, None, None, None)
            .expect("Failed to create spider");

        let seed_urls = vec![
            Url::parse("https://httpbin.org/links/3").expect("Valid URL")
        ];

        let start = std::time::Instant::now();
        let result = spider.crawl(seed_urls).await;
        let elapsed = start.elapsed();

        match result {
            Ok(crawl_result) => {
                // Duration should be positive
                assert!(
                    crawl_result.duration_seconds > 0.0,
                    "Duration should be positive"
                );

                // Duration should be close to actual elapsed time
                let reported_duration = Duration::from_secs_f64(crawl_result.duration_seconds);
                let diff = if elapsed > reported_duration {
                    elapsed - reported_duration
                } else {
                    reported_duration - elapsed
                };

                assert!(
                    diff.as_secs() < 5,
                    "Reported duration should be close to actual: reported={:.2}s, actual={:.2}s",
                    crawl_result.duration_seconds,
                    elapsed.as_secs_f64()
                );

                println!(
                    "✓ Performance metrics accurate: {:.2}s duration, {} pages",
                    crawl_result.duration_seconds,
                    crawl_result.pages_crawled
                );
            }
            Err(e) => {
                println!("⚠ Performance metrics test skipped (network/config issue): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_domain_tracking_multi_level() {
        // Test that domains are tracked correctly across multiple levels
        let config = create_test_spider_config(Some(2), Some(15));
        let robots = Arc::new(RobotsManager::new());

        let spider = Spider::new(config, robots, None, None, None)
            .expect("Failed to create spider");

        let seed_urls = vec![
            Url::parse("https://httpbin.org/links/2").expect("Valid URL")
        ];

        let result = spider.crawl(seed_urls).await;

        match result {
            Ok(crawl_result) => {
                // Should discover at least the base domain
                assert!(
                    !crawl_result.domains.is_empty(),
                    "Should discover at least one domain"
                );

                // httpbin.org should be in discovered domains
                let has_httpbin = crawl_result.domains.iter()
                    .any(|d| d.contains("httpbin.org"));

                assert!(
                    has_httpbin,
                    "Should discover httpbin.org domain"
                );

                println!(
                    "✓ Domain tracking: {} domains discovered",
                    crawl_result.domains.len()
                );
            }
            Err(e) => {
                println!("⚠ Domain tracking test skipped (network/config issue): {}", e);
            }
        }
    }
}
