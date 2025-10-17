# riptide-types

Shared types and traits for the RipTide extraction system.

## Purpose

This crate provides common type definitions and traits used across multiple RipTide crates. It was created to resolve circular dependencies between `riptide-core` and `riptide-extraction`.

## Architecture

The crate is organized into four main modules:

### `config`
Configuration types for extraction and crawling operations:
- `ExtractionMode` - Content extraction modes (Article, Full, Metadata, Custom)
- `RenderMode` - Rendering modes (Static, Dynamic, Adaptive, Pdf, Html, Markdown)
- `OutputFormat` - Output formats (Document, NdJson, Chunked, Text, Markdown)
- `ChunkingConfig` - Content chunking configuration
- `CrawlOptions` - Base crawl options (extended by riptide-core)

### `extracted`
Types representing extracted content and metadata:
- `ExtractedContent` - Common extraction result with title, content, summary, URL
- `BasicExtractedDoc` / `ExtractedDoc` - Basic extracted document for orchestration
- `ExtractionQuality` - Quality metrics (content_quality, title_quality, structure_score, etc.)
- `ExtractionStats` - Processing statistics (time, memory, nodes processed)
- `HealthStatus` - Component health information
- `ComponentInfo` - Component metadata
- `ContentChunk` - Chunked content with metadata

### `traits`
Core trait definitions for strategy implementations:
- `ExtractionStrategy` - Trait for content extraction strategies
- `SpiderStrategy` - Trait for crawler/spider strategies
- `ExtractionResult` - Result type with content, quality, performance
- `StrategyCapabilities` - Strategy capabilities descriptor
- `PerformanceMetrics` - Performance tracking (extraction time, content length, memory)
- `CrawlRequest` / `CrawlResult` - Spider crawling types
- `Priority` - Request priority levels

### `errors`
Error types for the RipTide system:
- `CoreError` - Main error enum with variants for WASM, memory, circuit breaker, HTTP, etc.
- `CoreResult<T>` - Type alias for `Result<T, CoreError>`
- Error recovery suggestions and telemetry support

## Usage

### In `riptide-core`:
```rust
use riptide_types::{
    ExtractedContent, ExtractionStrategy, ExtractionResult,
    CoreError, CoreResult
};
```

### In `riptide-extraction`:
```rust
use riptide_types::{
    ExtractedContent, ExtractionQuality, ExtractionResult,
    ExtractionStrategy, PerformanceMetrics
};
```

## Dependency Graph

```
riptide-types (no riptide dependencies)
    ├─> riptide-core
    └─> riptide-extraction
```

This structure ensures there are no circular dependencies between riptide crates.

## Re-exports

Both `riptide-core` and `riptide-extraction` re-export types from `riptide-types` for backward compatibility:

```rust
// In riptide-core/src/types.rs
pub use riptide_types::{
    BasicExtractedDoc, ExtractedContent, ExtractionQuality,
    ChunkingConfig, ExtractionMode, OutputFormat, RenderMode,
};

// In riptide-extraction/src/lib.rs
pub use riptide_types::{
    ExtractedContent, ExtractionQuality, ExtractionResult,
    ExtractionStrategy, PerformanceMetrics, StrategyCapabilities,
};
```

## Version

0.1.0

## License

Apache-2.0
