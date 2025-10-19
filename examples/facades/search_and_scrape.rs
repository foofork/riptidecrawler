//! Example: Search then scrape workflow
//!
//! Demonstrates combining SearchFacade and ScraperFacade for
//! a complete "search → scrape" pipeline.

use riptide_facade::Riptide;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Search → Scrape Workflow ===\n");

    // Step 1: Initialize facades
    println!("1. Initializing facades...");
    let search = Riptide::builder().build_search().await?;
    let scraper = Riptide::builder()
        .user_agent("ExampleBot/1.0")
        .build_scraper()
        .await?;

    println!("   Search backend: {}", search.backend_type());

    // Step 2: Perform search
    println!("\n2. Searching for 'rust web scraping'...");
    let search_results = search.search("rust web scraping", 5).await?;

    println!("   Found {} results", search_results.len());
    for hit in &search_results {
        println!(
            "   [{}] {} - {}",
            hit.rank,
            hit.title.as_ref().unwrap_or(&"(no title)".to_string()),
            hit.url
        );
    }

    // Step 3: Scrape search results
    println!("\n3. Scraping top search results...");
    for hit in search_results.iter().take(3) {
        match scraper.fetch_html(&hit.url).await {
            Ok(html) => {
                println!(
                    "   ✓ Scraped {} ({} bytes)",
                    hit.url,
                    html.len()
                );
            }
            Err(e) => {
                println!("   ✗ Failed to scrape {}: {}", hit.url, e);
            }
        }
    }

    // Step 4: Search with custom locale
    println!("\n4. Searching with German locale...");
    let de_results = search
        .search_with_locale("rust programmierung", 3, "de", "de")
        .await?;

    println!("   Found {} German results", de_results.len());
    for hit in &de_results {
        println!("   [{}] {}", hit.rank, hit.url);
    }

    // Step 5: Health check
    println!("\n5. Checking search provider health...");
    match search.health_check().await {
        Ok(_) => println!("   ✓ Search provider is healthy"),
        Err(e) => println!("   ✗ Health check failed: {}", e),
    }

    println!("\n=== Workflow Complete ===");
    Ok(())
}
