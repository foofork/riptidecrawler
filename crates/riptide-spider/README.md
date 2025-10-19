# RipTide Spider

Spider/crawler engine for the RipTide web scraping framework.

## Overview

`riptide-spider` implements intelligent web crawling with frontier management, adaptive strategies, URL normalization, and politeness policies for ethical web scraping at scale.

## Features

- **Frontier Management**: Priority queue-based URL management
- **Adaptive Strategies**: Depth-first, breadth-first, and intelligent crawling
- **URL Normalization**: Consistent URL handling and deduplication
- **Politeness Policies**: Respect robots.txt and crawl delays
- **Link Extraction**: Automatic link discovery and filtering
- **Crawl State**: Persistent crawl state and resume capability
- **Domain Filtering**: Include/exclude domain patterns
- **Max Depth Control**: Configurable crawl depth limits

## Usage

### Basic Crawling

```rust
use riptide_spider::*;

#[tokio::main]
async fn main() -> Result<()> {
    let spider = Spider::builder()
        .start_url("https://example.com")
        .max_depth(3)
        .max_pages(100)
        .build();

    spider.crawl(|page| async move {
        println!("Crawled: {}", page.url);
        // Process page
        Ok(())
    }).await?;

    Ok(())
}
```

### Advanced Configuration

```rust
use riptide_spider::*;

let spider = Spider::builder()
    .start_urls(vec![
        "https://example.com",
        "https://example.org",
    ])
    .max_depth(5)
    .max_pages(1000)
    .respect_robots_txt(true)
    .follow_external(false)
    .strategy(CrawlStrategy::BreadthFirst)
    .concurrency(10)
    .build();
```

## License

Apache-2.0
