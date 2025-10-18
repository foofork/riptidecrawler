# Riptide Facade

Unified facade and composition layer for the Riptide web scraping framework.

## Overview

`riptide-facade` provides a simplified, cohesive API surface for accessing all Riptide functionality without requiring direct knowledge of the 24+ specialized crates. It serves as a composition layer that reduces coupling between `riptide-api` and individual crates while maintaining backward compatibility.

## Features

- **Simplified API**: Task-oriented facades for common operations
- **Reduced Coupling**: Abstract internal crate boundaries
- **Unified Error Handling**: Single error type with context preservation
- **Composition Patterns**: Pre-built workflows and pipelines
- **Type Safety**: Leverage Rust's type system for compile-time guarantees
- **Async-First**: Native async/await support throughout
- **Feature Flags**: Optional functionality behind gates

## Quick Start

```rust
use riptide_facade::{Riptide, ScrapeOptions};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Build Riptide instance with configuration
    let riptide = Riptide::builder()
        .with_default_config()
        .build()?;

    // Simple scraping
    let doc = riptide.scraper()
        .fetch("https://example.com")
        .await?;

    println!("Title: {}", doc.title);
    println!("Content length: {}", doc.text.len());

    Ok(())
}
```

## Architecture

The facade provides domain-specific entry points:

### Core Facades

- **ScraperFacade**: Web page fetching and extraction
- **SpiderFacade**: Web crawling and spidering
- **BrowserFacade**: Headless browser automation
- **ExtractorFacade**: Content extraction strategies

### Infrastructure Facades

- **IntelligenceFacade**: LLM-powered operations
- **SecurityFacade**: Authentication and rate limiting
- **MonitoringFacade**: Metrics and telemetry
- **CacheFacade**: Caching infrastructure

## Configuration

```rust
let riptide = Riptide::builder()
    // Fetch configuration
    .with_fetch(|fetch| {
        fetch
            .max_retries(3)
            .timeout(30)
            .user_agent("MyBot/1.0")
    })
    // Spider configuration
    .with_spider(|spider| {
        spider
            .max_depth(5)
            .max_pages(1000)
            .crawl_delay_ms(200)
    })
    // Browser configuration
    .with_browser(|browser| {
        browser
            .headless(true)
            .pool_size(5)
            .enable_stealth()
    })
    .build()?;
```

## Feature Flags

```toml
[dependencies]
riptide-facade = { version = "0.1", features = ["scraper", "spider"] }
```

### Available Features

- `default`: `["scraper", "spider"]` - Minimal but functional
- `scraper`: Web page scraping (riptide-fetch + riptide-extraction)
- `spider`: Web crawling (riptide-spider)
- `browser`: Headless browser automation (riptide-engine + riptide-headless)
- `extractor`: Content extraction strategies
- `intelligence`: LLM operations
- `security`: Authentication and rate limiting
- `monitoring`: Metrics and telemetry
- `cache`: Caching infrastructure
- `production`: All production-ready features
- `full`: All features including experimental

## Examples

### Simple Scraping

```rust
use riptide_facade::prelude::*;

let riptide = Riptide::with_defaults()?;
let doc = riptide.scraper().fetch("https://example.com").await?;
```

### Advanced Crawling

```rust
use riptide_facade::prelude::*;

let riptide = Riptide::with_defaults()?;
let result = riptide.spider()
    .crawl_with_budget("https://example.com", CrawlBudget {
        max_pages: Some(100),
        max_depth: Some(3),
        timeout_secs: Some(300),
    })
    .await?;
```

### Browser Automation

```rust
use riptide_facade::prelude::*;

let riptide = Riptide::with_defaults()?;
let screenshot = riptide.browser()
    .screenshot("https://example.com", ScreenshotOptions {
        full_page: true,
        width: Some(1920),
        height: Some(1080),
    })
    .await?;
```

### Workflow Composition

```rust
use riptide_facade::prelude::*;

let result = riptide.workflow()
    .scrape_and_extract()
    .with_cache()
    .with_retry(3)
    .execute(WorkflowInput::url("https://example.com"))
    .await?;
```

## Error Handling

All operations return `Result<T, RiptideError>` with rich error context:

```rust
use riptide_facade::error::RiptideError;

match riptide.scraper().fetch(url).await {
    Ok(doc) => println!("Success: {}", doc.title),
    Err(RiptideError::HttpError { status, message }) => {
        eprintln!("HTTP {}: {}", status, message);
    }
    Err(RiptideError::TimeoutError { timeout_ms }) => {
        eprintln!("Timed out after {}ms", timeout_ms);
    }
    Err(e) if e.is_retryable() => {
        eprintln!("Retryable error: {}", e);
    }
    Err(e) => eprintln!("Fatal error: {}", e),
}
```

## Design Principles

1. **Layered Abstraction**: High-level APIs delegate to specialized crates
2. **Trait-Based Composition**: Flexible and testable architecture
3. **Builder Pattern**: Fluent configuration APIs
4. **Error Context**: Rich errors without losing details
5. **Zero-Cost Abstractions**: Minimal overhead through inline delegation

## Architecture Documentation

See [docs/architecture/riptide-facade-design.md](../../docs/architecture/riptide-facade-design.md) for detailed architecture design, including:

- Problem statement and goals
- API surface design
- Module structure
- Trait unification strategy
- Composition patterns
- Error handling
- Migration path
- Implementation roadmap

## Development Status

**Current Status**: Phase 1 - Foundation

- [x] Crate structure created
- [x] Error types defined
- [x] Configuration system designed
- [x] Builder pattern implemented
- [ ] ScraperFacade implementation
- [ ] SpiderFacade implementation
- [ ] BrowserFacade implementation
- [ ] Workflow composition
- [ ] API integration

## Contributing

This crate is part of the Riptide/EventMesh project. Contributions are welcome!

## License

Apache-2.0
