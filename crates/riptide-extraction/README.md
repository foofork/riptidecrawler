# RipTide HTML - Week 2 Track A Implementation

![RipTide HTML](https://img.shields.io/badge/RipTide-HTML-blue.svg)
![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)
![Status](https://img.shields.io/badge/status-completed-green.svg)

HTML processing and extraction capabilities for the RipTide project, implementing **Week 2 Track A: HTML Crate Creation** requirements (HTML-001 to HTML-004).

## 🎯 Implementation Status

✅ **HTML-001**: Complete crate structure with proper organization
✅ **HTML-002**: HtmlProcessor trait with async processing methods
✅ **HTML-003**: CSS extraction code moved from riptide-core
✅ **HTML-004**: Regex extraction code moved from riptide-core
✅ **Bonus**: DOM utilities, table extraction, and content chunking
✅ **Zero Breaking Changes**: Full backward compatibility maintained

## 📁 Crate Structure

```
crates/riptide-html/
├── Cargo.toml              # Crate configuration with workspace dependencies
├── README.md               # This documentation
├── src/
│   ├── lib.rs             # Main library exports and documentation
│   ├── processor.rs       # HtmlProcessor trait and core interfaces
│   ├── css_extraction.rs  # CSS selector-based extraction (moved from core)
│   ├── regex_extraction.rs # Regex pattern extraction (moved from core)
│   └── dom_utils.rs       # DOM traversal and table extraction utilities
├── tests/
│   └── integration_tests.rs # Comprehensive integration tests
└── examples/
    └── basic_extraction.rs  # Complete usage demonstration
```

## 🚀 Features

### Core Extraction Capabilities

- **CSS Extraction**: Extract content using CSS selectors with JSON mapping
- **Regex Extraction**: Pattern-based content extraction with configurable rules
- **DOM Traversal**: Utilities for navigating and manipulating HTML structures
- **Table Extraction**: Structured data extraction from HTML tables
- **Content Chunking**: Split content for processing large documents

### HtmlProcessor Trait

```rust
#[async_trait]
pub trait HtmlProcessor: Send + Sync {
    async fn extract_with_css(&self, html: &str, url: &str, selectors: &HashMap<String, String>) -> Result<ExtractedContent>;
    async fn extract_with_regex(&self, html: &str, url: &str, patterns: &[RegexPattern]) -> Result<ExtractedContent>;
    async fn extract_tables(&self, html: &str, mode: TableExtractionMode) -> Result<Vec<TableData>>;
    async fn chunk_content(&self, content: &str, mode: ChunkingMode) -> Result<Vec<ContentChunk>>;
    fn confidence_score(&self, html: &str) -> f64;
    fn processor_name(&self) -> &'static str;
}
```

### Extraction Modes

- **CSS Selectors**: Default selectors for common content types
- **Custom Selectors**: News articles, blog posts, e-commerce products
- **Regex Patterns**: Contact info, financial data, social media, dates
- **Table Modes**: All tables, headers only, size filters, CSS selectors

### Chunking Strategies

- **Fixed Size**: Character-based chunks with overlap
- **Sentence**: Semantic sentence boundaries
- **Paragraph**: Natural paragraph breaks
- **Token**: Word-based tokenization
- **Semantic**: Topic-based segmentation

## 📖 Usage Examples

### Basic CSS Extraction

```rust
use riptide_extraction::*;

let html = r#"<html><head><title>Test</title></head><body><p>Content</p></body></html>"#;

// Default selectors
let result = css_extract_default(html, "https://example.com").await?;
println!("Title: {}", result.title);

// Custom selectors
let mut selectors = std::collections::HashMap::new();
selectors.insert("title".to_string(), "h1".to_string());
let result = css_extract(html, "https://example.com", &selectors).await?;
```

### Regex Pattern Extraction

```rust
// Default patterns (emails, phones, URLs, etc.)
let result = regex_extraction::extract_default(html, "https://example.com").await?;

// Specialized patterns
let contacts = regex_extraction::extract_contacts(html, "https://example.com").await?;
let financial = regex_extraction::extract_financial(html, "https://example.com").await?;
```

### Table Extraction

```rust
use riptide_extraction::processor::TableExtractionMode;

// Extract all tables
let tables = dom_utils::extract_tables(html, TableExtractionMode::All).await?;

// Extract tables with headers only
let header_tables = dom_utils::extract_tables(html, TableExtractionMode::WithHeaders).await?;

// Size-based filtering
let large_tables = dom_utils::extract_tables(
    html,
    TableExtractionMode::MinSize { min_rows: 3, min_cols: 2 }
).await?;
```

### Content Chunking

```rust
let processor = processor::DefaultHtmlProcessor::default();

// Sentence-based chunking
let chunks = processor.chunk_content(
    content,
    processor::ChunkingMode::Sentence { max_sentences: 2 }
).await?;

// Fixed-size chunking
let chunks = processor.chunk_content(
    content,
    processor::ChunkingMode::FixedSize { size: 1000, overlap: 100 }
).await?;
```

## 🔧 Migration from riptide-core

The extraction code has been moved from `riptide-core` to `riptide-html` with full backward compatibility:

### Before (riptide-core)
```rust
use riptide_core::strategies::{css_json, extraction_regex};
```

### After (riptide-html)
```rust
use riptide_extraction::{css_extraction, regex_extraction};
```

### Backward Compatibility
```rust
// Still works - re-exported from riptide-html
use riptide_core::strategies::{css_json, extraction_regex};
```

## 🧪 Testing

```bash
# Run all tests
cargo test -p riptide-html

# Run example
cargo run --example basic_extraction -p riptide-html

# Check compilation
cargo check -p riptide-html
```

## 📦 Dependencies

Core dependencies managed via workspace:
- `scraper` - HTML parsing and CSS selectors
- `regex` - Pattern matching
- `serde` - Serialization
- `async-trait` - Async trait support
- `anyhow` - Error handling

## 🔗 Integration

### Workspace Configuration

Added to `/workspaces/eventmesh/Cargo.toml`:
```toml
[workspace]
members = [
  "crates/riptide-core",
  "crates/riptide-html",  # ← New crate
  # ... other crates
]
```

### riptide-core Integration

Updated `riptide-core/Cargo.toml`:
```toml
[dependencies]
riptide-html = { path = "../riptide-html" }
```

Re-exports maintain compatibility in `riptide-core/src/strategies/mod.rs`:
```rust
// Re-export from riptide-html for backward compatibility
pub use riptide_extraction::{css_extraction as css_json, regex_extraction as extraction_regex};
pub use riptide_extraction::{ExtractedContent, RegexPattern};
```

## 🎯 Week 2 Track A Completion

| Requirement | Status | Implementation |
|-------------|--------|----------------|
| HTML-001 | ✅ | Complete crate structure with organized modules |
| HTML-002 | ✅ | HtmlProcessor trait with async methods |
| HTML-003 | ✅ | CSS extraction moved from core with enhancements |
| HTML-004 | ✅ | Regex extraction moved from core with patterns |
| Zero Breaking | ✅ | Full backward compatibility via re-exports |
| Testing | ✅ | Comprehensive integration tests |
| Documentation | ✅ | Complete API documentation and examples |

## 🚀 Next Steps

This implementation provides the foundation for:
- **Week 2 Track B**: LLM integration hooks
- **Advanced Features**: Machine learning extraction patterns
- **Performance**: WASM-based processing optimizations
- **Extensibility**: Plugin-based extraction strategies

## 📄 License

Part of the RipTide project - Apache 2.0 License