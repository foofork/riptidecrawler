# riptide-spider

🎯 **Domain Layer - Pure Business Logic**

Intelligent web crawling engine for the RipTide web scraping framework. This crate implements sophisticated spider algorithms with frontier-based URL management, adaptive stopping conditions, query-aware prioritization, and configurable crawling strategies—all without infrastructure dependencies.

## Quick Overview

`riptide-spider` provides the **crawling brain** for RipTide. It decides what to crawl, in what order, and when to stop. It handles URL management, depth tracking, link extraction, and intelligent crawl termination while remaining completely independent of HTTP clients, databases, or specific storage implementations.

**Why it exists:** Separates crawling logic (domain) from fetching infrastructure (HTTP clients, browsers). You can test crawl strategies without making network requests or swap HTTP implementations without touching crawl algorithms.

**Layer classification:** Pure domain layer—zero infrastructure dependencies ✅

## Key Concepts

### 1. Frontier-Based URL Management

The frontier is a priority queue of URLs to crawl, implementing various ordering strategies:

```rust
use riptide_spider::{FrontierManager, CrawlRequest, Priority};

let mut frontier = FrontierManager::new(1000);

// Add URLs with priority
let request = CrawlRequest::new(url)
    .with_priority(Priority::High)
    .with_depth(1)
    .with_parent(parent_url);

frontier.push(request).await?;

// URLs are automatically deduplicated
frontier.push(request).await?; // Ignored - already seen

// Pop by strategy (BFS, DFS, Best-First, etc.)
let next = frontier.pop().await?;
```

**Features:**
- **Automatic deduplication** - URLs are normalized and hashed
- **Priority-based ordering** - High-priority URLs crawled first
- **Depth tracking** - Enforce maximum crawl depth
- **Parent tracking** - Maintain crawl tree structure
- **Retry management** - Track failed requests with exponential backoff

### 2. Crawling Strategies

Multiple strategies for different crawling goals:

```rust
use riptide_spider::{Spider, CrawlingStrategy, SpiderConfig};

// Breadth-First Search - good for shallow, wide crawls
let spider = Spider::builder()
    .start_url("https://example.com")
    .strategy(CrawlingStrategy::BreadthFirst)
    .max_depth(3)
    .build();

// Depth-First Search - good for deep, focused crawls
let spider = Spider::builder()
    .start_url("https://example.com")
    .strategy(CrawlingStrategy::DepthFirst)
    .max_depth(10)
    .build();

// Best-First - prioritizes URLs by custom scores
let spider = Spider::builder()
    .start_url("https://example.com")
    .strategy(CrawlingStrategy::BestFirst)
    .max_pages(100)
    .build();

// Query-Aware - relevance-based crawling with BM25 scoring
let spider = Spider::builder()
    .start_url("https://example.com")
    .strategy(CrawlingStrategy::QueryAware {
        query: "rust web scraping".to_string(),
        config: QueryAwareConfig::default(),
    })
    .max_pages(50)
    .build();
```

**Strategy Comparison:**

| Strategy | Use Case | Order | Best For |
|----------|----------|-------|----------|
| **BreadthFirst** | Shallow site mapping | Level-by-level | Sitemaps, discovery |
| **DepthFirst** | Deep topic exploration | Follow links deeply | Topic research |
| **BestFirst** | Targeted crawling | By URL score | Quality over quantity |
| **QueryAware** | Search-driven crawling | By relevance to query | Research, content mining |

### 3. Query-Aware Crawling (Advanced)

BM25-based relevance scoring for intelligent crawling:

```rust
use riptide_spider::{Spider, CrawlingStrategy, QueryAwareConfig};

// Crawl for specific topic with relevance scoring
let config = QueryAwareConfig {
    bm25_k1: 1.2,                    // Term frequency saturation
    bm25_b: 0.75,                    // Length normalization
    url_signal_weight: 0.3,          // Weight for URL path/title signals
    content_similarity_weight: 0.4,  // Weight for content similarity
    domain_diversity_weight: 0.3,    // Weight for domain diversity
};

let spider = Spider::builder()
    .start_url("https://example.com")
    .strategy(CrawlingStrategy::QueryAware {
        query: "machine learning algorithms".to_string(),
        config,
    })
    .max_pages(100)
    .build();

// Spider prioritizes URLs most relevant to query
// Uses BM25 to score: URL text, page titles, content previews
```

**How it works:**
1. **URL Signal Analysis** - Score URLs based on query terms in path/title
2. **Content Similarity** - Compare content to query using TF-IDF
3. **BM25 Ranking** - Industry-standard relevance scoring algorithm
4. **Domain Diversity** - Encourage crawling multiple domains
5. **Dynamic Re-ranking** - Adjust scores as crawl progresses

**Performance:**
```
Benchmark: Query-aware scoring with 1000 URLs, 10-word query
  BM25 scoring:           24.3 µs per URL
  URL signal analysis:    8.7 µs per URL
  Content similarity:     15.2 µs per URL
  Total overhead:         48.2 µs per URL
```

### 4. Adaptive Stopping Conditions

Intelligent crawl termination based on content patterns:

```rust
use riptide_spider::{Spider, AdaptiveStopEngine};

let spider = Spider::builder()
    .start_url("https://example.com")
    .max_pages(1000)  // Hard limit
    .build();

// Stop early if:
// - Content diversity plateaus (same content repeating)
// - Quality degrades (low-value pages)
// - Relevance drops (moving away from seed topics)
// - Budget exhausted (time/page limits reached)

let result = spider.crawl().await?;
println!("Stopped after {} pages (reason: {})",
    result.pages_crawled,
    result.stop_reason
);
```

**Stop Conditions:**
- **Content plateau** - No new unique content in last N pages
- **Quality threshold** - Average quality below minimum
- **Time budget** - Maximum crawl duration exceeded
- **Page budget** - Maximum page count reached
- **Depth budget** - Maximum depth reached
- **Manual stop** - User cancellation

### 5. URL Management & Normalization

Sophisticated URL handling with normalization and filtering:

```rust
use riptide_spider::url_utils;

// Normalize URLs for deduplication
let url1 = url_utils::normalize_url("https://example.com/page?a=1&b=2")?;
let url2 = url_utils::normalize_url("https://example.com/page?b=2&a=1")?;
assert_eq!(url1, url2); // Query params sorted

// Extract domain for filtering
let domain = url_utils::extract_domain(&url)?;

// Check if URL should be followed
let should_follow = url_utils::should_follow(&url, &config)?;

// Validate URL before crawling
let is_valid = url_utils::is_valid_url(&url)?;
```

**Normalization Rules:**
- Lowercase scheme and domain
- Sort query parameters
- Remove default ports (80, 443)
- Remove fragments (#anchors)
- Decode percent-encoded characters
- Normalize path (resolve ../, ./)

### 6. Link Extraction

Extract and filter links from HTML content:

```rust
use riptide_spider::extractor::{ContentExtractor, BasicExtractor};

let extractor = BasicExtractor::new();

// Extract links with metadata
let extracted = extractor.extract(html, &url).await?;

println!("Found {} links", extracted.links.len());
for link in extracted.links {
    println!("  {} (depth: {})", link.url, link.depth);
}

// Links are automatically:
// - Resolved to absolute URLs
// - Filtered by domain rules
// - Deduplicated
// - Scored for priority
```

**Link Types Extracted:**
- `<a href>` tags
- `<link>` tags
- `<area>` tags
- JavaScript navigation (when rendered)
- Canonical URLs
- Sitemap references

## Design Principles

### Zero Infrastructure Dependencies ✅

**Why this matters:**
- **Testability**: Test crawl strategies with mock HTML, no network I/O needed
- **Portability**: Swap HTTP client (reqwest → ureq) without changing crawler logic
- **Evolution**: Crawl algorithms remain stable as fetch infrastructure changes
- **Performance**: No database queries in crawl decision logic—pure CPU-bound algorithms

**Dependencies:**
```toml
# Domain-level only
riptide-types   # Core domain types
riptide-utils   # Pure utility functions
riptide-config  # Configuration types

# Infrastructure dependencies (OK - used as tools)
url            # URL parsing
regex          # Pattern matching
chrono         # Timestamps
dashmap        # Concurrent hash maps
uuid           # IDs

# ❌ NOT included:
reqwest        # HTTP client - lives in riptide-fetch
sqlx           # Database - lives in riptide-persistence
redis          # Cache - lives in riptide-cache
```

### Hexagonal Architecture Role

```text
┌──────────────────────────────────┐
│  riptide-spider (Domain)         │
│  - Crawl strategies              │
│  - URL management                │
│  - Frontier queue                │
│  - NO HTTP, NO database          │
└──────────────────────────────────┘
         ↑ uses
         │
┌────────┴──────────┐
│ riptide-facade    │  ← Orchestrates crawling
│ (Application)     │
└────────┬──────────┘
         │ calls
         ↓
┌──────────────────────────────────┐
│ riptide-fetch (Infrastructure)   │
│ - HTTP client                    │
│ - Robots.txt parsing             │
│ - Rate limiting                  │
└──────────────────────────────────┘
```

**Clean separation:**
- **Spider** decides WHAT to crawl
- **Fetch** handles HOW to fetch
- **Facade** orchestrates WHEN and WHY

## Usage Examples

### Basic Crawling

```rust
use riptide_spider::{Spider, CrawlRequest};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Build spider with configuration
    let spider = Spider::builder()
        .start_url("https://example.com")
        .max_depth(3)
        .max_pages(100)
        .build();

    // Crawl with callback for each page
    spider.crawl(|page| async move {
        println!("Crawled: {} (depth: {})", page.url, page.depth);

        // Extract data from page
        // (actual fetching happens in riptide-fetch)

        Ok(())
    }).await?;

    Ok(())
}
```

### Advanced Configuration

```rust
use riptide_spider::{Spider, SpiderConfig, CrawlingStrategy, Priority};

let config = SpiderConfig {
    max_depth: 5,
    max_pages: 1000,
    max_concurrent: 10,
    respect_robots_txt: true,
    follow_external_links: false,
    allowed_domains: vec!["example.com".to_string()],
    excluded_patterns: vec!["/admin/".to_string(), "/private/".to_string()],
    strategy: CrawlingStrategy::BestFirst,
    adaptive_stop: true,
    ..Default::default()
};

let spider = Spider::with_config(config);

// Add seed URLs with priorities
spider.add_seed(CrawlRequest::new(url1).with_priority(Priority::High)).await?;
spider.add_seed(CrawlRequest::new(url2).with_priority(Priority::Medium)).await?;

let result = spider.crawl().await?;

println!("Crawl complete:");
println!("  Pages: {}", result.pages_crawled);
println!("  Duration: {:?}", result.duration);
println!("  Stop reason: {}", result.stop_reason);
```

### Query-Aware Research Crawling

```rust
use riptide_spider::{Spider, CrawlingStrategy, QueryAwareConfig};

// Research-focused crawling with relevance scoring
let spider = Spider::builder()
    .start_url("https://docs.rs")
    .strategy(CrawlingStrategy::QueryAware {
        query: "async rust tokio futures".to_string(),
        config: QueryAwareConfig {
            bm25_k1: 1.2,
            bm25_b: 0.75,
            url_signal_weight: 0.3,
            content_similarity_weight: 0.4,
            domain_diversity_weight: 0.3,
        },
    })
    .max_pages(200)
    .build();

let result = spider.crawl().await?;

// Crawled pages are ranked by relevance to query
for page in result.pages {
    println!("{} - relevance: {:.2}", page.url, page.relevance_score);
}
```

### Custom Link Extraction

```rust
use riptide_spider::extractor::{ContentExtractor, BasicExtractor};

let extractor = BasicExtractor::new();

// Extract links from HTML
let html = r#"<html><body>
    <a href="/page1">Page 1</a>
    <a href="https://external.com/page2">External</a>
    <link rel="canonical" href="https://example.com/canonical">
</body></html>"#;

let extracted = extractor.extract(html, "https://example.com").await?;

println!("Found {} links", extracted.links.len());
for link in extracted.links {
    println!("  {} (type: {:?}, depth: {})",
        link.url, link.link_type, link.depth
    );
}

// Output:
// Found 3 links
//   https://example.com/page1 (type: Internal, depth: 1)
//   https://external.com/page2 (type: External, depth: 1)
//   https://example.com/canonical (type: Canonical, depth: 0)
```

### Budget Management

```rust
use riptide_spider::{Spider, BudgetManager};
use std::time::Duration;

// Set crawl budget constraints
let spider = Spider::builder()
    .start_url("https://example.com")
    .max_pages(1000)
    .max_depth(5)
    .max_duration(Duration::from_secs(300)) // 5 minutes
    .max_memory_mb(512)
    .build();

let result = spider.crawl().await?;

// Check what stopped the crawl
match result.stop_reason.as_str() {
    "page_budget" => println!("Reached max pages"),
    "time_budget" => println!("Timeout"),
    "depth_budget" => println!("Max depth reached"),
    "memory_budget" => println!("Memory limit"),
    "adaptive_stop" => println!("Content plateau detected"),
    _ => println!("Other reason"),
}
```

### Sitemap Integration

```rust
use riptide_spider::{Spider, SitemapParser};

// Parse sitemap to seed crawler
let sitemap = SitemapParser::parse_from_url("https://example.com/sitemap.xml").await?;

let spider = Spider::builder()
    .start_url("https://example.com")
    .build();

// Add all sitemap URLs as seeds
for url in sitemap.urls {
    spider.add_seed(CrawlRequest::new(url.loc).with_priority(Priority::Medium)).await?;
}

let result = spider.crawl().await?;
```

### Session-Aware Crawling

```rust
use riptide_spider::{Spider, SessionManager};

// Maintain session state across requests
let session_manager = SessionManager::new();

// Login and get session cookies
let session = session_manager.create("user-session").await?;
session.set_cookie("auth_token", "abc123")?;

let spider = Spider::builder()
    .start_url("https://example.com/members")
    .session_manager(session_manager)
    .build();

// Crawler will reuse session for authenticated crawling
let result = spider.crawl().await?;
```

## Domain Models

### Core Types

**`CrawlRequest`** - URL to crawl with metadata
```rust
pub struct CrawlRequest {
    pub url: Url,
    pub priority: Priority,
    pub depth: u32,
    pub parent: Option<Url>,
    pub metadata: HashMap<String, String>,
    pub created_at: SystemTime,
    pub retry_count: u32,
    pub score: Option<f64>,
}
```

**`CrawlResult`** - Result of crawling a URL
```rust
pub struct CrawlResult {
    pub url: Url,
    pub status: CrawlStatus,
    pub content: Option<String>,
    pub links: Vec<Url>,
    pub depth: u32,
    pub duration: Duration,
}
```

**`CrawlingStrategy`** - Strategy enumeration
```rust
pub enum CrawlingStrategy {
    BreadthFirst,
    DepthFirst,
    BestFirst,
    QueryAware { query: String, config: QueryAwareConfig },
}
```

**`FrontierManager`** - URL queue manager
```rust
pub struct FrontierManager {
    // Priority queue of URLs to crawl
    // Deduplication via normalized URL hash
    // Depth tracking and parent relationships
}
```

**`AdaptiveStopEngine`** - Intelligent stopping
```rust
pub struct AdaptiveStopEngine {
    // Content similarity analysis
    // Quality tracking
    // Diversity measurement
    // Budget monitoring
}
```

## Testing

### Pure Domain Logic - No Network Needed

Because spider is pure domain logic, tests don't need network I/O:

```rust
use riptide_spider::{FrontierManager, CrawlRequest, Priority};

#[tokio::test]
async fn test_frontier_priority_ordering() {
    let mut frontier = FrontierManager::new(100);

    // Add URLs with different priorities
    frontier.push(CrawlRequest::new(url1).with_priority(Priority::Low)).await.unwrap();
    frontier.push(CrawlRequest::new(url2).with_priority(Priority::High)).await.unwrap();
    frontier.push(CrawlRequest::new(url3).with_priority(Priority::Medium)).await.unwrap();

    // Pop should return highest priority first
    let next = frontier.pop().await.unwrap();
    assert_eq!(next.priority, Priority::High);
    assert_eq!(next.url, url2);
}

#[tokio::test]
async fn test_url_deduplication() {
    let mut frontier = FrontierManager::new(100);

    // Same URL, different forms
    frontier.push(CrawlRequest::new("https://example.com/page?a=1&b=2".parse()?)).await?;
    frontier.push(CrawlRequest::new("https://example.com/page?b=2&a=1".parse()?)).await?;

    // Should only have one URL after normalization
    assert_eq!(frontier.len(), 1);
}

#[tokio::test]
async fn test_depth_tracking() {
    let mut frontier = FrontierManager::new(100);

    let parent = CrawlRequest::new(url).with_depth(1);
    let child = CrawlRequest::new(child_url)
        .with_depth(2)
        .with_parent(parent.url.clone());

    frontier.push(parent).await?;
    frontier.push(child).await?;

    // Verify parent-child relationship preserved
    let next = frontier.pop().await?;
    assert_eq!(next.depth, 1);
}
```

### Query-Aware Scoring Tests

```rust
use riptide_spider::{QueryAwareScorer, QueryAwareConfig};

#[test]
fn test_bm25_scoring() {
    let config = QueryAwareConfig::default();
    let scorer = QueryAwareScorer::new("rust async tokio", config);

    // Score URLs by relevance
    let score1 = scorer.score_url("https://tokio.rs/tokio/tutorial/async");
    let score2 = scorer.score_url("https://example.com/python/django");

    // Rust/async URL should score higher
    assert!(score1 > score2);
}

#[test]
fn test_content_similarity() {
    let scorer = QueryAwareScorer::new("machine learning", Default::default());

    let content1 = "Machine learning algorithms for classification and regression";
    let content2 = "Cooking recipes for Italian pasta dishes";

    let score1 = scorer.score_content(content1);
    let score2 = scorer.score_content(content2);

    assert!(score1 > score2);
}
```

### Integration Test Pattern

```rust
use riptide_spider::Spider;

#[tokio::test]
async fn test_spider_with_mock_fetcher() {
    // Mock fetcher (doesn't make real HTTP requests)
    let fetcher = MockFetcher::new()
        .with_response("https://example.com", r#"
            <html><body>
                <a href="/page1">Page 1</a>
                <a href="/page2">Page 2</a>
            </body></html>
        "#);

    let spider = Spider::builder()
        .start_url("https://example.com")
        .max_pages(3)
        .build();

    // Inject mock fetcher
    let result = spider.crawl_with_fetcher(fetcher).await?;

    assert_eq!(result.pages_crawled, 3);
    assert_eq!(result.links_found, 2);
}
```

## Common Patterns

### Idiomatic Usage

✅ **DO:** Separate crawl logic from fetch logic
```rust
// Spider decides WHAT to crawl
let spider = Spider::builder()
    .start_url("https://example.com")
    .strategy(CrawlingStrategy::BreadthFirst)
    .build();

// Fetch infrastructure handles HOW to fetch
let fetcher = HttpFetcher::new();

// Orchestrate together in facade layer
let result = crawl_with_fetcher(spider, fetcher).await?;
```

✅ **DO:** Use builder pattern for configuration
```rust
let spider = Spider::builder()
    .start_url("https://example.com")
    .max_depth(5)
    .max_pages(1000)
    .strategy(CrawlingStrategy::QueryAware {
        query: "rust web scraping".to_string(),
        config: Default::default(),
    })
    .respect_robots_txt(true)
    .follow_external_links(false)
    .build();
```

✅ **DO:** Track crawl progress with callbacks
```rust
let spider = Spider::builder()
    .start_url("https://example.com")
    .on_page(|page| async move {
        println!("Crawled: {}", page.url);
    })
    .on_error(|error| async move {
        eprintln!("Error: {}", error);
    })
    .on_complete(|stats| async move {
        println!("Done! {} pages", stats.pages_crawled);
    })
    .build();
```

### Anti-Patterns to Avoid

❌ **DON'T:** Mix HTTP logic with crawl logic
```rust
// ❌ Bad: HTTP client in spider domain
pub struct Spider {
    frontier: FrontierManager,
    http_client: reqwest::Client,  // ❌ Infrastructure in domain
}
```

✅ **DO:** Keep spider pure, inject fetcher
```rust
// ✅ Good: Spider is pure domain logic
pub struct Spider {
    frontier: FrontierManager,
    strategy: CrawlingStrategy,
    // No HTTP client!
}

// Fetcher is passed in at execution time
async fn crawl_with_fetcher(
    spider: Spider,
    fetcher: &dyn HttpFetcher,
) -> Result<CrawlResult> {
    // ...
}
```

❌ **DON'T:** Hardcode stopping conditions
```rust
// ❌ Bad: Magic numbers
if pages_crawled > 1000 {
    break;
}
```

✅ **DO:** Use configurable budget manager
```rust
// ✅ Good: Explicit budget configuration
let budget = BudgetManager::new()
    .with_max_pages(1000)
    .with_max_depth(5)
    .with_timeout(Duration::from_secs(300));
```

## Integration Points

### How Facades Use This Crate

**riptide-facade:**
```rust
use riptide_spider::{Spider, CrawlingStrategy};
use riptide_fetch::HttpFetcher;

pub struct CrawlFacade {
    fetcher: Arc<HttpFetcher>,
}

impl CrawlFacade {
    pub async fn crawl_website(&self, url: &str) -> Result<Vec<ExtractedContent>> {
        // Build spider (domain logic)
        let spider = Spider::builder()
            .start_url(url)
            .strategy(CrawlingStrategy::BreadthFirst)
            .max_pages(100)
            .build();

        // Execute with fetcher (infrastructure)
        let result = spider.crawl_with_fetcher(&*self.fetcher).await?;

        Ok(result.pages)
    }
}
```

### How Infrastructure Uses This Crate

**riptide-fetch:**
```rust
use riptide_spider::{CrawlRequest, CrawlResult};

pub struct HttpFetcher {
    client: reqwest::Client,
}

impl HttpFetcher {
    pub async fn fetch(&self, request: &CrawlRequest) -> Result<CrawlResult> {
        // Use domain types as interface
        let response = self.client.get(request.url.as_str()).send().await?;

        Ok(CrawlResult {
            url: request.url.clone(),
            content: response.text().await?,
            // ...
        })
    }
}
```

### Related Crates

- **Domain Layer:**
  - `riptide-types` - Core domain types and port traits
  - `riptide-extraction` - Content extraction strategies
  - `riptide-search` - Search domain logic

- **Application Layer:**
  - `riptide-facade` - Crawl workflows and orchestration

- **Infrastructure Layer:**
  - `riptide-fetch` - HTTP client, robots.txt, rate limiting
  - `riptide-browser` - Browser automation for dynamic content
  - `riptide-persistence` - Store crawl results

- **Composition Root:**
  - `riptide-api` - HTTP API handlers
  - `riptide-cli` - Command-line crawling

## License

Apache-2.0
