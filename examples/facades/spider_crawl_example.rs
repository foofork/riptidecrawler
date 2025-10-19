//! Example: Web crawling with SpiderFacade
//!
//! Demonstrates multi-page crawling with budget controls and different strategies.

use riptide_facade::{CrawlBudget, Riptide};
use riptide_spider::CrawlingStrategy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Spider Crawl Examples ===\n");

    // Example 1: Basic crawl with page budget
    println!("1. Basic Crawl (max 10 pages):");
    let spider = Riptide::builder()
        .user_agent("ExampleBot/1.0")
        .build_spider()
        .await?;

    let budget = CrawlBudget::pages(10);
    let result = spider.crawl("https://example.com", budget).await?;

    println!(
        "   Crawled {} pages, {} URLs queued, {} visited",
        result.total_pages, result.frontier_stats.queued_urls, result.frontier_stats.visited_urls
    );

    // Example 2: Depth-limited crawl
    println!("\n2. Depth-Limited Crawl (max depth 2):");
    let budget = CrawlBudget::depth(2);
    let result = spider.crawl("https://example.com", budget).await?;

    println!("   Crawled {} pages at depth <= 2", result.total_pages);

    // Example 3: Time-limited crawl
    println!("\n3. Time-Limited Crawl (60 seconds):");
    let budget = CrawlBudget::timeout(60);
    let result = spider.crawl("https://example.com", budget).await?;

    println!("   Crawled {} pages within 60 seconds", result.total_pages);

    // Example 4: Combined budget constraints
    println!("\n4. Combined Budget (50 pages, depth 3, 5 minutes):");
    let budget = CrawlBudget {
        max_pages: Some(50),
        max_depth: Some(3),
        timeout_secs: Some(300),
    };
    let result = spider.crawl("https://example.com", budget).await?;

    println!(
        "   Crawled {} pages (budget: 50 pages, depth 3, 300s)",
        result.total_pages
    );

    // Example 5: Depth-first crawl strategy
    println!("\n5. Depth-First Strategy:");
    let result = spider
        .crawl_with_strategy(
            "https://example.com",
            CrawlBudget::pages(20),
            CrawlingStrategy::DepthFirst,
        )
        .await?;

    println!("   Crawled {} pages using DFS", result.total_pages);

    // Example 6: Query-aware crawl (relevance-based)
    println!("\n6. Query-Aware Crawl (relevant to 'documentation'):");
    let result = spider
        .query_aware_crawl(
            "https://docs.rs",
            "async programming",
            CrawlBudget::pages(30),
        )
        .await?;

    println!(
        "   Crawled {} pages relevant to 'async programming'",
        result.total_pages
    );

    // Example 7: Access frontier statistics
    println!("\n7. Frontier Statistics:");
    let frontier = spider.frontier();
    println!("   Queued URLs: {}", frontier.queued_count());

    println!("\n=== Examples Complete ===");
    Ok(())
}
