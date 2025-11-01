# Spider Integration - Circular Dependency Fix Plan

## Problem Statement

The `spider` feature in `riptide-extraction` is disabled due to a circular dependency:
```
extraction → spider → reliability → pool → extraction (CYCLE)
```

This prevents the desired "extract while spidering" functionality where spider and extraction work together seamlessly.

## Current Status

- ✅ **riptide-spider**: Fully functional - can crawl/spider pages
- ✅ **riptide-extraction**: Fully functional - can extract from HTML
- ⚠️ **Integration**: Blocked by circular dependency

## Solution Strategy

Use **trait abstraction** (same pattern as CircuitBreaker fix):

### Phase 1: Move Types to Foundation Crate

Move these from `riptide-extraction` to `riptide-types`:

1. **CrawlRequest** - Request details for crawling
2. **CrawlResult** - Result of a crawl operation
3. **Priority** - Crawl priority enum
4. **SpiderStrategy trait** - Core spider strategy interface
5. **CrawlStats** - Statistics tracking

**File**: `/workspaces/eventmesh/crates/riptide-types/src/spider.rs` (NEW)

```rust
//! Spider types and traits for cross-crate integration
//!
//! This module provides spider-related types that can be used across
//! multiple crates without creating circular dependencies.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Priority level for crawl requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Request to crawl a URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlRequest {
    /// URL to crawl
    pub url: String,
    /// Crawl priority
    pub priority: Priority,
    /// Crawl depth (0 = seed URL)
    pub depth: usize,
    /// Referrer URL (if any)
    pub referrer: Option<String>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Result of a crawl operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlResult {
    /// Whether crawl succeeded
    pub success: bool,
    /// HTTP status code
    pub status_code: Option<u16>,
    /// Extracted content (if successful)
    pub extracted_data: Option<crate::ExtractedContent>,
    /// Discovered URLs to crawl
    pub discovered_urls: Vec<String>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Crawl duration in milliseconds
    pub duration_ms: u64,
}

/// Spider statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrawlStats {
    /// Total URLs discovered
    pub total_discovered: usize,
    /// URLs completed
    pub completed: usize,
    /// URLs pending
    pub pending: usize,
    /// URLs failed
    pub failed: usize,
    /// Average crawl time per URL (milliseconds)
    pub average_time_ms: u64,
    /// Total bytes downloaded
    pub total_bytes: u64,
    /// Crawl start time
    pub start_time: Option<std::time::SystemTime>,
}

/// Spider strategy trait for implementing crawl logic
///
/// This trait allows extraction and spider crates to integrate
/// without circular dependencies via dependency injection.
#[async_trait]
pub trait SpiderStrategy: Send + Sync {
    /// Get strategy name
    fn name(&self) -> &str;

    /// Initialize spider strategy
    async fn initialize(&mut self) -> Result<()>;

    /// Get next URL to crawl
    async fn next_url(&mut self) -> Option<CrawlRequest>;

    /// Add discovered URLs to the queue
    async fn add_urls(&mut self, requests: Vec<CrawlRequest>) -> Result<()>;

    /// Process a batch of crawl requests (prioritize, filter, etc.)
    async fn process_requests(&self, requests: Vec<CrawlRequest>) -> Result<Vec<CrawlRequest>> {
        // Default implementation: return requests as-is
        Ok(requests)
    }

    /// Mark URL as completed
    async fn mark_completed(&mut self, url: &str, result: CrawlResult) -> Result<()>;

    /// Check if crawling is complete
    fn is_complete(&self) -> bool;

    /// Get crawl statistics
    fn stats(&self) -> CrawlStats;

    /// Pause crawling
    async fn pause(&mut self) -> Result<()> {
        Ok(())
    }

    /// Resume crawling
    async fn resume(&mut self) -> Result<()> {
        Ok(())
    }

    /// Cancel crawling
    async fn cancel(&mut self) -> Result<()> {
        Ok(())
    }
}
```

### Phase 2: Update riptide-types/lib.rs

Add spider module:

```rust
// In riptide-types/src/lib.rs
pub mod spider;
pub use spider::{
    CrawlRequest, CrawlResult, CrawlStats, Priority, SpiderStrategy,
};
```

### Phase 3: Update riptide-spider

Remove type definitions, use from riptide-types:

```rust
// riptide-spider/src/types.rs
// Remove CrawlRequest, CrawlResult, Priority
// Re-export from riptide-types instead
pub use riptide_types::{CrawlRequest, CrawlResult, CrawlStats, Priority, SpiderStrategy};
```

Update spider implementations to use trait:

```rust
// riptide-spider/src/strategies/breadth_first.rs
use riptide_types::SpiderStrategy;

pub struct BreadthFirstStrategy {
    // ... fields
}

#[async_trait]
impl SpiderStrategy for BreadthFirstStrategy {
    // ... implementation
}
```

### Phase 4: Update riptide-extraction

Remove spider types, use from riptide-types:

```rust
// riptide-extraction/src/strategies/traits.rs
// Remove:
// #[cfg(feature = "spider")]
// pub trait SpiderStrategy { ... }
// pub struct CrawlStats { ... }

// Replace with:
pub use riptide_types::{SpiderStrategy, CrawlStats, CrawlRequest, CrawlResult, Priority};
```

### Phase 5: Update riptide-pool

If pool has spider-related code, update imports:

```rust
use riptide_types::{SpiderStrategy, CrawlRequest, CrawlResult};
```

### Phase 6: Re-enable spider Feature

Update Cargo.toml:

```toml
# riptide-extraction/Cargo.toml
[features]
default = ["css-extraction", "regex-extraction", "dom-utils", "chunking", "native-parser"]
spider = ["strategy-traits"]  # NO MORE riptide-spider dependency!
strategy-traits = []
```

Update lib.rs:

```rust
// Remove #[cfg(feature = "spider")] gates
// Spider types now always available from riptide-types
```

### Phase 7: Enable Extract-While-Spidering

Now you can do:

```rust
use riptide_spider::Spider;
use riptide_extraction::NativeHtmlParser;
use riptide_types::SpiderStrategy;

let spider = Spider::new(config);
let parser = Arc::new(NativeHtmlParser::new());

// Spider uses parser via trait injection - NO CIRCULAR DEPENDENCY!
spider.crawl_with_extraction(
    "https://example.com",
    parser as Arc<dyn SpiderStrategy>
).await?;
```

## Benefits

1. ✅ No circular dependencies
2. ✅ Spider and extraction can work together seamlessly
3. ✅ Trait-based design enables extensibility
4. ✅ Same proven pattern as CircuitBreaker fix
5. ✅ All features enabled by default

## Estimated Effort

- **Time**: 4-6 hours (similar to CircuitBreaker migration)
- **Risk**: Low (same proven pattern)
- **Breaking Changes**: Minimal (mostly internal refactoring)

## Testing Strategy

1. Unit tests for spider trait implementations
2. Integration tests for spider + extraction workflows
3. Performance benchmarks for crawl + extract pipelines
4. Verify no regressions in standalone spider functionality

## Rollout Plan

1. Create riptide-types/spider.rs with all types
2. Update riptide-spider to use new types (breaking change)
3. Update riptide-extraction to use new types
4. Re-enable spider feature
5. Add integration tests
6. Update documentation
7. Release as minor version bump

## Future Enhancements

Once integration is complete, you can add:

- Real-time extraction during crawling
- Adaptive crawl strategies based on extraction results
- Quality-based crawl prioritization
- Extract-and-link-follow patterns
- Multi-strategy spider orchestration

## Conclusion

**Will you have issues in the future?**

Only if you need "extract while spidering" functionality. When that time comes, follow this plan to enable full integration using the same trait abstraction pattern we successfully used for CircuitBreaker.

The circular dependency is **solvable** and **proven** - we just did it! 🚀
