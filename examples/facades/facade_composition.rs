//! Example: Composing multiple facades
//!
//! Demonstrates advanced workflows using multiple facades together:
//! - Search → Spider → Extract
//! - Browser → Extract
//! - Pipeline orchestration

use riptide_facade::{CrawlBudget, Riptide};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Facade Composition Examples ===\n");

    // Example 1: Search → Spider → Extract
    println!("1. Search → Spider → Extract Workflow:");
    println!("   a. Search for seed URLs...");

    let search = Riptide::builder().build_search().await?;
    let search_results = search.search("rust documentation", 3).await?;

    println!("      Found {} seed URLs", search_results.len());

    println!("   b. Crawl each seed URL...");
    let spider = Riptide::builder()
        .user_agent("CompositionBot/1.0")
        .build_spider()
        .await?;

    let mut total_crawled = 0;
    for hit in search_results.iter().take(2) {
        println!("      Crawling: {}", hit.url);

        let budget = CrawlBudget {
            max_pages: Some(5),
            max_depth: Some(1),
            timeout_secs: Some(30),
        };

        match spider.crawl(&hit.url, budget).await {
            Ok(result) => {
                total_crawled += result.total_pages;
                println!(
                    "      ✓ Crawled {} pages from {}",
                    result.total_pages, hit.url
                );
            }
            Err(e) => {
                println!("      ✗ Crawl failed: {}", e);
            }
        }
    }

    println!("   c. Total pages crawled: {}", total_crawled);

    // Example 2: Browser automation for dynamic content
    println!("\n2. Browser → Extract Workflow:");
    println!("   a. Launch browser...");

    let browser = Riptide::builder()
        .user_agent("BrowserBot/1.0")
        .build_browser()
        .await?;

    let session = browser.launch().await?;
    println!("      ✓ Browser launched");

    println!("   b. Navigate to page...");
    browser
        .navigate(&session, "https://example.com")
        .await?;
    println!("      ✓ Navigation complete");

    println!("   c. Extract content...");
    let content = browser.content(&session).await?;
    println!("      ✓ Extracted {} bytes", content.len());

    println!("   d. Clean up...");
    browser.close(session).await?;
    println!("      ✓ Browser closed");

    // Example 3: Query-aware crawl + extraction
    println!("\n3. Query-Aware Crawl + Extraction:");
    println!("   a. Query-aware crawl for 'async' content...");

    let result = spider
        .query_aware_crawl(
            "https://docs.rs",
            "async programming",
            CrawlBudget::pages(10),
        )
        .await?;

    println!("      ✓ Found {} relevant pages", result.total_pages);

    println!("   b. Extract data from crawled pages...");
    let extractor = Riptide::builder().build_extractor().await?;

    let mut extracted_count = 0;
    for page in result.pages.iter().take(5) {
        println!("      Extracting: {}", page.url);

        // Note: page.content would need to be HTML string
        // This is a simplified example
        extracted_count += 1;
    }

    println!("      ✓ Extracted data from {} pages", extracted_count);

    // Example 4: Multi-facade pipeline
    println!("\n4. Multi-Facade Pipeline:");
    println!("   Steps: Search → Scrape → Extract → Analyze");

    let scraper = Riptide::builder()
        .user_agent("PipelineBot/1.0")
        .build_scraper()
        .await?;

    println!("   a. Search for URLs...");
    let urls = search.search("rust best practices", 2).await?;
    println!("      ✓ Found {} URLs", urls.len());

    println!("   b. Scrape each URL...");
    let mut scraped_pages = Vec::new();
    for url in urls.iter().take(2) {
        match scraper.fetch_html(&url.url).await {
            Ok(html) => {
                scraped_pages.push((url.url.clone(), html));
                println!("      ✓ Scraped: {}", url.url);
            }
            Err(e) => {
                println!("      ✗ Failed: {}", e);
            }
        }
    }

    println!("   c. Extract structured data...");
    for (url, html) in &scraped_pages {
        let options = riptide_facade::facades::HtmlExtractionOptions {
            as_markdown: true,
            extract_links: true,
            ..Default::default()
        };

        match extractor.extract_html(html, url, options).await {
            Ok(_data) => {
                println!("      ✓ Extracted: {}", url);
            }
            Err(e) => {
                println!("      ✗ Extraction failed: {}", e);
            }
        }
    }

    println!("   d. Pipeline complete!");

    // Example 5: Frontier inspection
    println!("\n5. Frontier Inspection:");
    let frontier = spider.frontier();
    println!("   Queued URLs: {}", frontier.queued_count());

    println!("\n=== All Examples Complete ===");
    Ok(())
}
