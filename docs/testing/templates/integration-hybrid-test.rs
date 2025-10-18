/// Spider-Chrome Hybrid Integration Test Template
///
/// Tests for riptide-headless-hybrid crate validating seamless switching
/// between spider_rs (fast) and chrome_headless (JavaScript support).

use riptide_headless_hybrid::{HybridCrawler, CrawlMode, BrowserPool};
use wiremock::{MockServer, Mock, ResponseTemplate};

#[tokio::test]
async fn test_spider_only_mode() {
    // Arrange: Simple HTML without JavaScript
    let mock_server = MockServer::start().await;
    Mock::given(wiremock::matchers::method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"<html><body><h1>Hello World</h1></body></html>"#
        ))
        .mount(&mock_server)
        .await;

    // Act: Crawl with spider-only mode
    let crawler = HybridCrawler::builder()
        .mode(CrawlMode::SpiderOnly)
        .build()
        .unwrap();

    let result = crawler.crawl(&mock_server.uri()).await;

    // Assert: Should succeed without Chrome
    assert!(result.is_ok());
    let page = result.unwrap();
    assert!(page.html.contains("Hello World"));
    assert_eq!(page.engine_used, "spider");
}

#[tokio::test]
async fn test_chrome_only_mode() {
    // Arrange: JavaScript-heavy page
    let mock_server = MockServer::start().await;
    Mock::given(wiremock::matchers::method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"<html><body><script>document.body.innerHTML = '<h1>JS Rendered</h1>';</script></body></html>"#
        ))
        .mount(&mock_server)
        .await;

    // Act: Crawl with chrome-only mode
    let crawler = HybridCrawler::builder()
        .mode(CrawlMode::ChromeOnly)
        .build()
        .unwrap();

    let result = crawler.crawl(&mock_server.uri()).await;

    // Assert: Should execute JavaScript
    assert!(result.is_ok());
    let page = result.unwrap();
    assert!(page.html.contains("JS Rendered"));
    assert_eq!(page.engine_used, "chrome");
}

#[tokio::test]
async fn test_hybrid_auto_switching() {
    // Arrange: Mixed content
    let mock_server = MockServer::start().await;

    // Page 1: Static HTML (spider should handle)
    Mock::given(wiremock::matchers::path("/static"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"<html><body><h1>Static</h1></body></html>"#
        ))
        .mount(&mock_server)
        .await;

    // Page 2: JavaScript required (should fallback to chrome)
    Mock::given(wiremock::matchers::path("/dynamic"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"<html><body><script>document.write('<h1>Dynamic</h1>');</script></body></html>"#
        ))
        .mount(&mock_server)
        .await;

    // Act: Crawl with hybrid mode
    let crawler = HybridCrawler::builder()
        .mode(CrawlMode::HybridAuto)
        .build()
        .unwrap();

    let static_result = crawler.crawl(&format!("{}/static", mock_server.uri())).await.unwrap();
    let dynamic_result = crawler.crawl(&format!("{}/dynamic", mock_server.uri())).await.unwrap();

    // Assert: Correct engine selected
    assert_eq!(static_result.engine_used, "spider", "Static page uses spider");
    assert_eq!(dynamic_result.engine_used, "chrome", "Dynamic page uses chrome");
}

#[tokio::test]
async fn test_javascript_detection() {
    let crawler = HybridCrawler::new();

    // Test cases
    let test_cases = vec![
        (r#"<html><body>No JS</body></html>"#, false),
        (r#"<html><body><script>alert('hi');</script></body></html>"#, true),
        (r#"<html><body onclick="doSomething()">JS Event</body></html>"#, true),
        (r#"<html><head><script src="app.js"></script></head></html>"#, true),
        (r#"<html><body><div id="react-root"></div></body></html>"#, true),
        (r#"<html><body>{{vue_template}}</body></html>"#, true),
    ];

    for (html, expected) in test_cases {
        let detected = crawler.detect_javascript(html);
        assert_eq!(
            detected, expected,
            "JS detection failed for: {}",
            &html[..50.min(html.len())]
        );
    }
}

#[tokio::test]
async fn test_browser_pool_lifecycle() {
    // Arrange: Create browser pool
    let pool = BrowserPool::builder()
        .max_browsers(3)
        .idle_timeout_seconds(10)
        .build()
        .unwrap();

    // Act: Acquire browsers
    let browser1 = pool.acquire().await.unwrap();
    let browser2 = pool.acquire().await.unwrap();
    let browser3 = pool.acquire().await.unwrap();

    // Assert: Pool at capacity
    assert_eq!(pool.active_count(), 3);
    assert_eq!(pool.idle_count(), 0);

    // Release browsers
    drop(browser1);
    drop(browser2);

    // Wait for browsers to return to pool
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Assert: Browsers returned to pool
    assert_eq!(pool.active_count(), 1);
    assert_eq!(pool.idle_count(), 2);

    // Cleanup
    drop(browser3);
    pool.shutdown().await.unwrap();
    assert_eq!(pool.active_count(), 0);
}

#[tokio::test]
async fn test_browser_pool_timeout() {
    // Arrange: Pool with 1 browser
    let pool = BrowserPool::builder()
        .max_browsers(1)
        .acquire_timeout_seconds(2)
        .build()
        .unwrap();

    // Act: Acquire browser and hold it
    let _browser = pool.acquire().await.unwrap();

    // Try to acquire second browser (should timeout)
    let start = tokio::time::Instant::now();
    let result = pool.acquire().await;
    let elapsed = start.elapsed();

    // Assert: Timed out correctly
    assert!(result.is_err());
    assert!(elapsed.as_secs() >= 2 && elapsed.as_secs() <= 3);
    assert_eq!(result.unwrap_err().kind(), ErrorKind::PoolTimeout);
}

#[tokio::test]
async fn test_cdp_protocol_integration() {
    use chromiumoxide::cdp::browser_protocol::network::EventResponseReceived;

    // Arrange: Create chrome browser
    let browser = Browser::launch().await.unwrap();
    let page = browser.new_page("https://example.com").await.unwrap();

    // Act: Listen for CDP events
    let mut events = page.event_listener::<EventResponseReceived>().await.unwrap();

    page.goto("https://example.com").await.unwrap();

    // Assert: Received network events
    let event = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        events.recv()
    ).await;

    assert!(event.is_ok());
    let response = event.unwrap().unwrap();
    assert_eq!(response.response.status, 200);
}

#[tokio::test]
async fn test_hybrid_performance_vs_chrome_only() {
    // Arrange: 10 static pages
    let mock_server = MockServer::start().await;
    for i in 0..10 {
        Mock::given(wiremock::matchers::path(&format!("/page{}", i)))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                format!(r#"<html><body><h1>Page {}</h1></body></html>"#, i)
            ))
            .mount(&mock_server)
            .await;
    }

    let urls: Vec<_> = (0..10)
        .map(|i| format!("{}/page{}", mock_server.uri(), i))
        .collect();

    // Act: Hybrid mode
    let hybrid_crawler = HybridCrawler::builder()
        .mode(CrawlMode::HybridAuto)
        .build()
        .unwrap();

    let hybrid_start = tokio::time::Instant::now();
    for url in &urls {
        hybrid_crawler.crawl(url).await.unwrap();
    }
    let hybrid_duration = hybrid_start.elapsed();

    // Act: Chrome-only mode
    let chrome_crawler = HybridCrawler::builder()
        .mode(CrawlMode::ChromeOnly)
        .build()
        .unwrap();

    let chrome_start = tokio::time::Instant::now();
    for url in &urls {
        chrome_crawler.crawl(url).await.unwrap();
    }
    let chrome_duration = chrome_start.elapsed();

    // Assert: Hybrid should be faster for static pages
    assert!(
        hybrid_duration < chrome_duration,
        "Hybrid ({:?}) should be faster than Chrome-only ({:?})",
        hybrid_duration,
        chrome_duration
    );

    // Expect at least 2x speedup
    let speedup = chrome_duration.as_millis() as f64 / hybrid_duration.as_millis() as f64;
    assert!(speedup >= 2.0, "Expected 2x+ speedup, got {:.2}x", speedup);
}

#[tokio::test]
async fn test_resource_usage_spider_vs_chrome() {
    use sysinfo::{System, SystemExt, ProcessExt};

    // Baseline memory
    let mut sys = System::new_all();
    sys.refresh_all();
    let baseline_memory = sys.used_memory();

    // Spider crawl (low memory)
    let spider_crawler = HybridCrawler::builder()
        .mode(CrawlMode::SpiderOnly)
        .build()
        .unwrap();

    spider_crawler.crawl("https://example.com").await.unwrap();

    sys.refresh_all();
    let spider_memory = sys.used_memory() - baseline_memory;

    // Chrome crawl (high memory)
    let chrome_crawler = HybridCrawler::builder()
        .mode(CrawlMode::ChromeOnly)
        .build()
        .unwrap();

    chrome_crawler.crawl("https://example.com").await.unwrap();

    sys.refresh_all();
    let chrome_memory = sys.used_memory() - baseline_memory;

    // Assert: Chrome uses significantly more memory
    assert!(
        chrome_memory > spider_memory * 5,
        "Chrome ({} MB) should use 5x+ memory vs Spider ({} MB)",
        chrome_memory / 1024 / 1024,
        spider_memory / 1024 / 1024
    );
}

#[tokio::test]
async fn test_fallback_on_spider_failure() {
    // Arrange: Page that spider can't handle
    let mock_server = MockServer::start().await;
    Mock::given(wiremock::matchers::method("GET"))
        .respond_with(ResponseTemplate::new(403)) // Spider blocked
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    Mock::given(wiremock::matchers::method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"<html><body>Success with Chrome</body></html>"#
        ))
        .mount(&mock_server)
        .await;

    // Act: Hybrid crawler should fallback to chrome
    let crawler = HybridCrawler::builder()
        .mode(CrawlMode::HybridAuto)
        .fallback_on_error(true)
        .build()
        .unwrap();

    let result = crawler.crawl(&mock_server.uri()).await;

    // Assert: Succeeded with chrome fallback
    assert!(result.is_ok());
    let page = result.unwrap();
    assert_eq!(page.engine_used, "chrome");
    assert!(page.html.contains("Success with Chrome"));
}

#[tokio::test]
async fn test_migration_compatibility() {
    // Old spider_rs API
    let old_result = {
        use spider_chrome::Spider;
        let spider = Spider::new();
        spider.crawl("https://example.com").await.unwrap()
    };

    // New hybrid API (spider mode)
    let new_result = {
        let crawler = HybridCrawler::builder()
            .mode(CrawlMode::SpiderOnly)
            .build()
            .unwrap();
        crawler.crawl("https://example.com").await.unwrap()
    };

    // Assert: Results should be equivalent
    assert_eq!(old_result.status, new_result.status);
    assert_eq!(old_result.url, new_result.url);
    // HTML may differ slightly due to processing, but structure should match
}

/// Test helper: Generate test HTML
fn generate_html(has_js: bool) -> String {
    if has_js {
        r#"<html>
            <body>
                <div id="content">Loading...</div>
                <script>
                    document.getElementById('content').innerHTML = 'JS Content';
                </script>
            </body>
        </html>"#.to_string()
    } else {
        r#"<html>
            <body>
                <div id="content">Static Content</div>
            </body>
        </html>"#.to_string()
    }
}
