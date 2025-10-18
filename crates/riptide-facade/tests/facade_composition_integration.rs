//! Integration tests for facade composition and multi-facade workflows
//!
//! These tests verify that facades work together correctly in complex
//! scenarios requiring multiple facades to cooperate.

use riptide_facade::prelude::*;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn test_scraper_basic_workflow() -> Result<(), Box<dyn std::error::Error>> {
    // Test basic scraper workflow (currently the only fully implemented facade)
    let mock_server = MockServer::start().await;

    let html_content = r#"
        <!DOCTYPE html>
        <html>
            <head><title>Test Article</title></head>
            <body>
                <article>
                    <h1>Article Title</h1>
                    <p>Article content goes here.</p>
                </article>
            </body>
        </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/article"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html_content))
        .mount(&mock_server)
        .await;

    let scraper = Riptide::builder()
        .user_agent("CompositionBot/1.0")
        .build_scraper()
        .await?;

    let url = format!("{}/article", mock_server.uri());
    let html = scraper.fetch_html(&url).await?;

    assert!(html.contains("Article Title"));
    assert!(html.contains("Article content"));

    Ok(())
}

#[tokio::test]
#[ignore = "Facades not yet fully implemented"]
async fn test_browser_plus_extraction() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement when BrowserFacade and ExtractorFacade are ready
    // let config = RiptideConfig::default();
    // let runtime = RiptideRuntime::new()?;

    // Create facades
    // let browser = BrowserFacade::new(config.clone(), runtime.clone()).await?;
    // let extractor = ExtractorFacade::new(config, runtime).await?;

    // Launch browser and navigate
    // let session = browser.launch().await?;
    // browser.navigate(&session, "https://example.com").await?;

    // Get HTML content
    // let html = browser.get_content(&session).await?;

    // Extract structured data
    // let extracted = extractor.extract_html(&html).await?;

    // assert!(!extracted.content.is_empty());
    // assert!(extracted.title.is_some());

    // browser.close(session).await?;

    Ok(())
}

#[tokio::test]
#[ignore = "Facades not yet fully implemented"]
async fn test_scraper_extractor_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement when ExtractorFacade is ready
    // let config = RiptideConfig::default();
    // let runtime = RiptideRuntime::new()?;

    // Create facades
    // let scraper = ScraperFacade::new(config.clone()).await?;
    // let extractor = ExtractorFacade::new(config, runtime).await?;

    // Fetch HTML
    // let html = scraper.fetch_html("https://example.com").await?;

    // Extract content
    // let extracted = extractor.extract_html(&html).await?;

    // Verify pipeline worked
    // assert!(!extracted.content.is_empty());

    Ok(())
}

#[tokio::test]
#[ignore = "Facades not yet fully implemented"]
async fn test_spider_extraction_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement when SpiderFacade and ExtractorFacade are ready
    // let config = RiptideConfig::default();
    // let runtime = RiptideRuntime::new()?;

    // Create facades
    // let spider = SpiderFacade::new(config.clone(), runtime.clone()).await?;
    // let extractor = ExtractorFacade::new(config, runtime).await?;

    // Crawl website
    // let crawl_result = spider.crawl("https://example.com").await?;

    // Extract content from each page
    // let mut extracted_pages = vec![];
    // for page in crawl_result.pages {
    //     let extracted = extractor.extract_html(&page.html).await?;
    //     extracted_pages.push(extracted);
    // }

    // assert!(!extracted_pages.is_empty());

    Ok(())
}

#[tokio::test]
#[ignore = "Facades not yet fully implemented"]
async fn test_full_scraping_workflow() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement when all facades are ready
    // Complete workflow: Browser -> Spider -> Extraction -> Pipeline

    // let config = RiptideConfig::default();
    // let runtime = RiptideRuntime::new()?;

    // 1. Launch browser for JavaScript rendering
    // let browser = BrowserFacade::new(config.clone(), runtime.clone()).await?;
    // let session = browser.launch().await?;
    // browser.navigate(&session, "https://example.com").await?;

    // 2. Get initial HTML
    // let html = browser.get_content(&session).await?;

    // 3. Extract links for crawling
    // let extractor = ExtractorFacade::new(config.clone(), runtime.clone()).await?;
    // let links = extractor.extract_links(&html, "https://example.com").await?;

    // 4. Crawl discovered links
    // let spider = SpiderFacade::new(config.clone(), runtime.clone()).await?;
    // let crawl_result = spider.crawl_urls(&links.absolute_links).await?;

    // 5. Extract content from all pages
    // let mut all_content = vec![];
    // for page in crawl_result.pages {
    //     let content = extractor.extract_html(&page.html).await?;
    //     all_content.push(content);
    // }

    // browser.close(session).await?;

    // assert!(!all_content.is_empty());

    Ok(())
}

#[tokio::test]
#[ignore = "Facades not yet fully implemented"]
async fn test_resource_cleanup_multi_facade() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement when facades are ready
    // Test that resources are properly cleaned up when using multiple facades

    // let config = RiptideConfig::default();
    // let runtime = RiptideRuntime::new()?;

    // Create and use multiple facades
    // for _ in 0..5 {
    //     let browser = BrowserFacade::new(config.clone(), runtime.clone()).await?;
    //     let session = browser.launch().await?;
    //     browser.navigate(&session, "https://example.com").await?;
    //     browser.close(session).await?;

    //     let scraper = ScraperFacade::new(config.clone()).await?;
    //     let _ = scraper.fetch_html("https://example.com").await?;
    // }

    // Verify no resource leaks (would need monitoring)

    Ok(())
}

#[tokio::test]
#[ignore = "Facades not yet fully implemented"]
async fn test_concurrent_multi_facade_operations() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement when facades are ready
    // Test concurrent operations across multiple facades

    // let config = RiptideConfig::default();
    // let runtime = RiptideRuntime::new()?;

    // let scraper = ScraperFacade::new(config.clone()).await?;
    // let extractor = ExtractorFacade::new(config.clone(), runtime.clone()).await?;

    // let urls = vec![
    //     "https://example.com/page1",
    //     "https://example.com/page2",
    //     "https://example.com/page3",
    // ];

    // Fetch and extract concurrently
    // let handles: Vec<_> = urls.into_iter().map(|url| {
    //     let s = scraper.clone();
    //     let e = extractor.clone();
    //     tokio::spawn(async move {
    //         let html = s.fetch_html(url).await?;
    //         e.extract_html(&html).await
    //     })
    // }).collect();

    // let results = futures::future::join_all(handles).await;
    // assert_eq!(results.len(), 3);

    Ok(())
}

#[tokio::test]
#[ignore = "Facades not yet fully implemented"]
async fn test_error_propagation_across_facades() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement when facades are ready
    // Test that errors propagate correctly through facade chain

    // let config = RiptideConfig::default();
    // let runtime = RiptideRuntime::new()?;

    // let scraper = ScraperFacade::new(config.clone()).await?;
    // let extractor = ExtractorFacade::new(config, runtime).await?;

    // Try to fetch invalid URL
    // let result = scraper.fetch_html("not-a-url").await;
    // assert!(result.is_err());

    // Error should be proper RiptideError
    // assert!(matches!(result.unwrap_err(), RiptideError::InvalidUrl(_)));

    Ok(())
}

#[tokio::test]
#[ignore = "Facades not yet fully implemented"]
async fn test_shared_config_across_facades() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement when facades are ready
    // Test that configuration is shared correctly across facades

    // let config = RiptideConfig::default()
    //     .with_user_agent("SharedBot/1.0")
    //     .with_timeout_secs(60);
    // let runtime = RiptideRuntime::new()?;

    // let scraper = ScraperFacade::new(config.clone()).await?;
    // let browser = BrowserFacade::new(config.clone(), runtime.clone()).await?;
    // let extractor = ExtractorFacade::new(config.clone(), runtime).await?;

    // All should use same config
    // assert_eq!(scraper.config().user_agent, "SharedBot/1.0");
    // assert_eq!(browser.config().user_agent, "SharedBot/1.0");
    // assert_eq!(extractor.config().user_agent, "SharedBot/1.0");

    Ok(())
}

#[tokio::test]
async fn test_scraper_multiple_requests() -> Result<(), Box<dyn std::error::Error>> {
    // Test scraper with multiple sequential requests
    let mock_server = MockServer::start().await;

    for i in 1..=3 {
        Mock::given(method("GET"))
            .and(path(format!("/page{}", i)))
            .respond_with(ResponseTemplate::new(200).set_body_string(format!("Page {}", i)))
            .mount(&mock_server)
            .await;
    }

    let scraper = Riptide::builder().build_scraper().await?;

    // Fetch multiple pages
    for i in 1..=3 {
        let url = format!("{}/page{}", mock_server.uri(), i);
        let html = scraper.fetch_html(&url).await?;
        assert!(html.contains(&format!("Page {}", i)));
    }

    Ok(())
}
