# riptide-extraction

🎯 **Domain Layer - Pure Business Logic**

Intelligent content extraction engine for the RipTide web scraping framework. This crate provides sophisticated HTML parsing, content extraction strategies, and LLM-based extraction coordination—all without infrastructure dependencies.

## Quick Overview

`riptide-extraction` is the **content brain** for RipTide. It knows how to extract meaningful content from HTML: articles, tables, metadata, links, and structured data. It provides CSS selectors, regex patterns, DOM traversal, semantic chunking, and strategy composition for high-quality extraction.

**Why it exists:** Separates extraction logic (domain) from HTTP fetching and storage. You can test extraction strategies with HTML strings without needing browsers, databases, or network requests.

**Layer classification:** Pure domain layer—zero infrastructure dependencies ✅

## Key Concepts

### 1. Extraction Strategies

Multiple strategies for different content types and quality goals:

```rust
use riptide_extraction::{
    ExtractionStrategy,
    ExtractionStrategyType,
    StrategyManager,
};

// Built-in strategies
let strategy = ExtractionStrategyType::Article;      // News articles, blog posts
let strategy = ExtractionStrategyType::FullContent;  // Complete page content
let strategy = ExtractionStrategyType::Metadata;     // Meta tags, OpenGraph, Schema.org
let strategy = ExtractionStrategyType::Tables;       // Structured table data
let strategy = ExtractionStrategyType::Css {         // Custom CSS selectors
    selectors: HashMap::from([
        ("title".to_string(), "h1.title".to_string()),
        ("content".to_string(), "div.content".to_string()),
    ])
};
let strategy = ExtractionStrategyType::Regex {      // Pattern-based extraction
    patterns: vec![
        RegexPattern {
            name: "email".to_string(),
            pattern: r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b".to_string(),
            field: "emails".to_string(),
            required: false,
        }
    ]
};

// Execute strategy
let manager = StrategyManager::new();
let result = manager.execute(strategy, html, url).await?;

println!("Extracted: {}", result.content.title);
println!("Quality: {:.2}", result.quality.content_quality);
```

### 2. Strategy Composition (Phase 2D)

Combine multiple strategies for higher quality:

```rust
use riptide_extraction::{StrategyComposer, CompositionMode, ConfidenceScore};

// Parallel execution with voting
let composer = StrategyComposer::new(CompositionMode::Parallel {
    strategies: vec![
        ExtractionStrategyType::Article,
        ExtractionStrategyType::Css { selectors: custom_selectors },
        ExtractionStrategyType::Regex { patterns: patterns },
    ],
    aggregation: AggregationStrategy::Vote,  // or Max, Average, Weighted
});

let result = composer.extract(html, url).await?;

// Result is aggregated from all strategies
println!("Confidence: {:.2}", result.confidence.overall);
println!("Agreement: {}/{} strategies",
    result.confidence.agreements,
    result.confidence.total_strategies
);

// Fallback cascade
let composer = StrategyComposer::new(CompositionMode::Fallback {
    primary: ExtractionStrategyType::Article,
    fallbacks: vec![
        ExtractionStrategyType::Css { selectors: default_selectors },
        ExtractionStrategyType::FullContent,
    ],
});

// Uses first strategy that succeeds with high confidence
let result = composer.extract(html, url).await?;
```

**Aggregation Strategies:**
- **Vote** - Majority consensus among strategies
- **Max** - Highest confidence result
- **Average** - Averaged scores across strategies
- **Weighted** - Weighted combination by strategy reliability

### 3. CSS Selector Extraction

Extract content using CSS selectors with semantic defaults:

```rust
use riptide_extraction::css_extraction;

// Default selectors for common content
let result = css_extraction::extract_default(html, url).await?;
println!("Title: {}", result.title);
println!("Content: {}", result.content);
println!("Summary: {}", result.summary.unwrap_or_default());

// Custom selectors for specific site
let mut selectors = HashMap::new();
selectors.insert("title".to_string(), "h1.article-title".to_string());
selectors.insert("content".to_string(), "div.article-body".to_string());
selectors.insert("author".to_string(), "span.author-name".to_string());
selectors.insert("published".to_string(), "time[datetime]".to_string());

let result = css_extraction::extract(html, url, &selectors).await?;

// Specialized presets
let result = css_extraction::extract_article(html, url).await?;  // News articles
let result = css_extraction::extract_product(html, url).await?;  // E-commerce
let result = css_extraction::extract_recipe(html, url).await?;   // Recipes
```

**Default Selectors:**
```rust
{
    "title": "h1, h2, title, meta[property='og:title']",
    "content": "article, main, .content, .post-content, #content",
    "summary": "meta[name='description'], meta[property='og:description']",
    "author": ".author, [rel='author'], [itemprop='author']",
    "published": "time[datetime], .date, .published",
    "images": "img[src], picture source[srcset]",
}
```

### 4. Regex Pattern Extraction

Pattern-based extraction for emails, phones, URLs, dates, etc.:

```rust
use riptide_extraction::regex_extraction;

// Default patterns (email, phone, URL, dates)
let result = regex_extraction::extract_default(html, url).await?;
println!("Emails: {:?}", result.metadata.get("emails"));
println!("Phones: {:?}", result.metadata.get("phones"));

// Specialized pattern sets
let result = regex_extraction::extract_contacts(html, url).await?;   // Emails, phones
let result = regex_extraction::extract_financial(html, url).await?;  // Prices, amounts
let result = regex_extraction::extract_social(html, url).await?;     // Social handles
let result = regex_extraction::extract_dates(html, url).await?;      // Dates, times

// Custom patterns
let patterns = vec![
    RegexPattern {
        name: "order_id".to_string(),
        pattern: r"ORDER-\d{6}".to_string(),
        field: "order_ids".to_string(),
        required: false,
    },
    RegexPattern {
        name: "tracking".to_string(),
        pattern: r"[A-Z0-9]{12}".to_string(),
        field: "tracking_numbers".to_string(),
        required: false,
    },
];

let result = regex_extraction::extract(html, url, &patterns).await?;
```

**Built-in Patterns:**
```rust
// Email
r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b"

// Phone (US)
r"\b(\+?1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b"

// URL
r"https?://[^\s<>\"]+|www\.[^\s<>\"]+"

// Price
r"\$\d+(?:,\d{3})*(?:\.\d{2})?"

// Date (ISO 8601)
r"\d{4}-\d{2}-\d{2}"

// Social handles
r"@[A-Za-z0-9_]+"
```

### 5. Table Extraction

Extract structured data from HTML tables:

```rust
use riptide_extraction::table_extraction::{
    extract_tables_advanced,
    TableExtractionConfig,
    TableExtractor,
};

// Extract all tables with metadata
let tables = extract_tables_advanced(html, TableExtractionConfig::default()).await?;

for table in tables {
    println!("Table: {} rows × {} cols", table.rows.len(), table.headers.len());
    println!("Headers: {:?}", table.headers);

    for row in table.rows {
        println!("Row: {:?}", row.cells);
    }

    // Export as CSV
    let csv = table.to_csv()?;

    // Export as JSON
    let json = table.to_json()?;
}

// Filter by criteria
let config = TableExtractionConfig {
    min_rows: 3,
    min_cols: 2,
    require_headers: true,
    ignore_nested: true,
    css_selector: Some("table.data-table".to_string()),
};

let tables = extract_tables_advanced(html, config).await?;

// Semantic table understanding
let extractor = TableExtractor::new();
for table in tables {
    println!("Table type: {:?}", table.metadata.table_type);  // Data, Layout, Calendar
    println!("Has headers: {}", table.metadata.has_headers);
    println!("Row groups: {:?}", table.structure.row_groups);
}
```

**Table Features:**
- Automatic header detection
- Row/column grouping
- Cell type inference (string, number, date)
- Nested table handling
- Export to CSV/JSON/Markdown
- Semantic analysis (data vs layout tables)

### 6. Content Chunking

Split content into chunks for LLM processing:

```rust
use riptide_extraction::chunking::{ChunkingStrategy, ChunkingConfig, create_strategy};

// Sentence-based chunking
let strategy = create_strategy(ChunkingConfig {
    mode: ChunkingMode::Sentence { max_sentences: 3 },
    overlap: 100,
});

let chunks = strategy.chunk(content).await?;

for (i, chunk) in chunks.iter().enumerate() {
    println!("Chunk {}: {} chars", i, chunk.text.len());
    println!("  Metadata: {:?}", chunk.metadata);
}

// Token-based chunking (for LLMs)
let strategy = create_strategy(ChunkingConfig {
    mode: ChunkingMode::Token {
        max_tokens: 512,
        tokenizer: "gpt-3.5-turbo".to_string(),
    },
    overlap: 50,
});

let chunks = strategy.chunk(content).await?;

// Semantic chunking (topic-based)
let strategy = create_strategy(ChunkingConfig {
    mode: ChunkingMode::Semantic {
        similarity_threshold: 0.75,
        min_chunk_size: 100,
    },
    overlap: 0,  // No overlap for semantic boundaries
});

let chunks = strategy.chunk(content).await?;
```

**Chunking Modes:**
- **FixedSize** - Fixed character count with overlap
- **Sentence** - Natural sentence boundaries
- **Paragraph** - Paragraph breaks
- **Token** - Token count (for LLM context windows)
- **Semantic** - Topic-based segmentation using similarity

**Chunk Metadata:**
```rust
pub struct ChunkMetadata {
    pub index: usize,
    pub start_offset: usize,
    pub end_offset: usize,
    pub token_count: Option<usize>,
    pub sentence_count: Option<usize>,
    pub paragraph_count: Option<usize>,
    pub similarity_to_prev: Option<f64>,
}
```

### 7. DOM Traversal & Utilities

Low-level DOM manipulation for custom extraction:

```rust
use riptide_extraction::dom_utils::{DomTraverser, ElementInfo};

let traverser = DomTraverser::new();

// Find elements by selector
let elements = traverser.find_elements(html, "div.content > p")?;

for elem in elements {
    println!("Tag: {}", elem.tag);
    println!("Text: {}", elem.text);
    println!("Attributes: {:?}", elem.attributes);
}

// Extract text content (removes scripts, styles)
let text = dom_utils::extract_text_content(html)?;

// Find tables
let tables = dom_utils::find_tables(html)?;

// Traverse DOM tree
traverser.traverse(html, |element: ElementInfo| {
    println!("Visiting: <{}> depth={}", element.tag, element.depth);

    // Return false to stop traversal
    true
})?;
```

### 8. Enhanced Link Extraction

Extract links with context and classification:

```rust
use riptide_extraction::enhanced_link_extraction::{
    EnhancedLinkExtractor,
    LinkExtractionConfig,
    LinkType,
};

let config = LinkExtractionConfig {
    include_external: true,
    include_images: true,
    include_scripts: false,
    extract_text: true,
    extract_context: true,
    max_context_length: 200,
};

let extractor = EnhancedLinkExtractor::new(config);
let links = extractor.extract(html, base_url).await?;

for link in links {
    println!("URL: {}", link.url);
    println!("Type: {:?}", link.link_type);  // Navigation, Asset, Social, etc.
    println!("Text: {}", link.text);
    println!("Context: {}", link.context);
    println!("Score: {:.2}", link.relevance_score);
}
```

**Link Types:**
- **Navigation** - Internal site navigation
- **External** - External websites
- **Asset** - Images, CSS, JS
- **Social** - Social media links
- **Document** - PDFs, docs, downloads
- **Canonical** - Canonical URL
- **Alternate** - Alternate versions

### 9. Schema Extraction & Learning

Extract and learn structured schemas:

```rust
use riptide_extraction::schema::{SchemaExtractor, SchemaGenerator, SchemaRegistry};

// Extract Schema.org structured data
let extractor = SchemaExtractor::new();
let schemas = extractor.extract(html).await?;

for schema in schemas {
    println!("Type: {}", schema.schema_type);
    println!("Fields: {:?}", schema.fields);
}

// Learn schema from examples
let generator = SchemaGenerator::new();
let learned_schema = generator.learn_from_examples(vec![
    html1, html2, html3
]).await?;

println!("Learned schema:");
println!("  Confidence: {:.2}", learned_schema.confidence);
println!("  Fields: {:?}", learned_schema.fields);

// Use learned schema for extraction
let registry = SchemaRegistry::new();
registry.register("product", learned_schema)?;

let schema = registry.get("product")?;
let extracted = extractor.extract_with_schema(html, schema).await?;
```

### 10. Parallel Batch Extraction

High-performance parallel processing:

```rust
use riptide_extraction::parallel::{ParallelExtractor, ParallelConfig, DocumentTask};

let config = ParallelConfig {
    max_concurrent: 10,
    timeout_per_document: Duration::from_secs(30),
    strategy: ExtractionStrategyType::Article,
};

let extractor = ParallelExtractor::new(config);

// Batch extraction with progress tracking
let tasks = vec![
    DocumentTask { id: "1".to_string(), html: html1, url: url1 },
    DocumentTask { id: "2".to_string(), html: html2, url: url2 },
    DocumentTask { id: "3".to_string(), html: html3, url: url3 },
];

let results = extractor.extract_batch(tasks, |progress| {
    println!("Progress: {}/{} ({:.1}%)",
        progress.completed,
        progress.total,
        progress.percentage()
    );
}).await?;

for result in results {
    match result {
        Ok(extracted) => println!("Success: {}", extracted.title),
        Err(e) => println!("Failed: {}", e),
    }
}
```

## Design Principles

### Zero Infrastructure Dependencies ✅

**Why this matters:**
- **Testability**: Test extraction with HTML strings, no HTTP needed
- **Portability**: Swap rendering backend (headless Chrome → Firefox) without changing extractors
- **Evolution**: Extraction algorithms remain stable as infrastructure changes
- **Performance**: CPU-bound extraction logic with no I/O blocking

**Dependencies:**
```toml
# Domain-level only
riptide-types   # Core domain types

# Pure utilities (no I/O)
scraper         # HTML parsing (pure Rust)
lol_html        # Fast HTML rewriting
regex           # Pattern matching
tiktoken-rs     # Token counting
chrono          # Dates
uuid            # IDs

# ❌ NOT included:
reqwest         # HTTP client - lives in riptide-fetch
headless_chrome # Browser automation - lives in riptide-browser
sqlx            # Database - lives in riptide-persistence
```

### Hexagonal Architecture Role

```text
┌──────────────────────────────────┐
│  riptide-extraction (Domain)     │
│  - Extraction strategies         │
│  - CSS selectors, regex          │
│  - DOM parsing, chunking         │
│  - NO HTTP, NO browser           │
└──────────────────────────────────┘
         ↑ uses
         │
┌────────┴──────────┐
│ riptide-facade    │  ← Orchestrates extraction
│ (Application)     │
└────────┬──────────┘
         │ calls
         ↓
┌──────────────────────────────────┐
│ riptide-browser (Infrastructure) │
│ - Headless Chrome                │
│ - JavaScript rendering           │
│ - Dynamic content                │
└──────────────────────────────────┘
```

**Clean separation:**
- **Extraction** knows HOW to parse HTML
- **Browser** handles HOW to render pages
- **Facade** orchestrates WHEN and WHAT to extract

## Usage Examples

### Basic Article Extraction

```rust
use riptide_extraction::{ExtractionStrategyType, StrategyManager};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let html = r#"
        <html><body>
            <h1>Article Title</h1>
            <div class="content">
                <p>Article content goes here...</p>
            </div>
        </body></html>
    "#;

    let manager = StrategyManager::new();
    let result = manager.execute(
        ExtractionStrategyType::Article,
        html,
        "https://example.com/article"
    ).await?;

    println!("Title: {}", result.content.title);
    println!("Content: {}", result.content.content);
    println!("Quality: {:.2}", result.quality.content_quality);

    Ok(())
}
```

### Multi-Strategy Extraction with Fallback

```rust
use riptide_extraction::{StrategyComposer, CompositionMode};

let composer = StrategyComposer::new(CompositionMode::Fallback {
    primary: ExtractionStrategyType::Article,
    fallbacks: vec![
        ExtractionStrategyType::Css {
            selectors: default_selectors(),
        },
        ExtractionStrategyType::FullContent,
    ],
});

let result = composer.extract(html, url).await?;

println!("Extracted with: {:?}", result.strategy_used);
println!("Confidence: {:.2}", result.confidence.overall);
```

### Custom Extraction Pipeline

```rust
use riptide_extraction::*;

async fn extract_product(html: &str, url: &str) -> Result<Product> {
    // 1. Extract with CSS selectors
    let css_result = css_extraction::extract(html, url, &product_selectors()).await?;

    // 2. Extract pricing with regex
    let regex_result = regex_extraction::extract_financial(html, url).await?;

    // 3. Extract images from DOM
    let traverser = DomTraverser::new();
    let images = traverser.find_elements(html, "img.product-image")?;

    // 4. Combine results
    Ok(Product {
        title: css_result.title,
        description: css_result.content,
        price: regex_result.metadata.get("prices")
            .and_then(|p| p.first())
            .map(|s| s.to_string()),
        images: images.iter()
            .map(|e| e.attributes.get("src").cloned())
            .collect(),
    })
}
```

## Testing

### Pure Extraction Logic - No HTTP Needed

Test extraction with simple HTML strings:

```rust
use riptide_extraction::css_extraction;

#[tokio::test]
async fn test_article_extraction() {
    let html = r#"
        <html><body>
            <h1>Test Article</h1>
            <article>
                <p>This is test content.</p>
            </article>
        </body></html>
    "#;

    let result = css_extraction::extract_default(html, "https://test.com").await.unwrap();

    assert_eq!(result.title, "Test Article");
    assert!(result.content.contains("test content"));
}

#[tokio::test]
async fn test_regex_email_extraction() {
    let html = "Contact us at support@example.com or sales@example.com";

    let result = regex_extraction::extract_contacts(html, "https://test.com").await.unwrap();

    let emails = result.metadata.get("emails").unwrap();
    assert_eq!(emails.len(), 2);
    assert!(emails.contains(&"support@example.com".to_string()));
}

#[tokio::test]
async fn test_table_extraction() {
    let html = r#"
        <table>
            <thead><tr><th>Name</th><th>Age</th></tr></thead>
            <tbody>
                <tr><td>Alice</td><td>30</td></tr>
                <tr><td>Bob</td><td>25</td></tr>
            </tbody>
        </table>
    "#;

    let tables = table_extraction::extract_tables_advanced(
        html,
        TableExtractionConfig::default()
    ).await.unwrap();

    assert_eq!(tables.len(), 1);
    assert_eq!(tables[0].headers, vec!["Name", "Age"]);
    assert_eq!(tables[0].rows.len(), 2);
}
```

### Strategy Composition Tests

```rust
use riptide_extraction::{StrategyComposer, CompositionMode, AggregationStrategy};

#[tokio::test]
async fn test_parallel_voting() {
    let composer = StrategyComposer::new(CompositionMode::Parallel {
        strategies: vec![
            ExtractionStrategyType::Article,
            ExtractionStrategyType::FullContent,
        ],
        aggregation: AggregationStrategy::Vote,
    });

    let html = "<html><body><h1>Title</h1><p>Content</p></body></html>";
    let result = composer.extract(html, "https://test.com").await.unwrap();

    assert!(result.confidence.overall > 0.5);
    assert_eq!(result.confidence.total_strategies, 2);
}
```

## Common Patterns

### Idiomatic Usage

✅ **DO:** Use strategy composition for robustness
```rust
// Primary strategy with fallbacks
let composer = StrategyComposer::new(CompositionMode::Fallback {
    primary: ExtractionStrategyType::Article,
    fallbacks: vec![
        ExtractionStrategyType::Css { selectors },
        ExtractionStrategyType::FullContent,
    ],
});
```

✅ **DO:** Chunk content for LLM processing
```rust
let strategy = create_strategy(ChunkingConfig {
    mode: ChunkingMode::Token {
        max_tokens: 2048,
        tokenizer: "gpt-4".to_string(),
    },
    overlap: 100,
});

let chunks = strategy.chunk(long_content).await?;
for chunk in chunks {
    llm.process(chunk.text).await?;
}
```

✅ **DO:** Extract tables for structured data
```rust
let tables = extract_tables_advanced(html, TableExtractionConfig {
    min_rows: 2,
    require_headers: true,
    css_selector: Some("table.data".to_string()),
}).await?;

for table in tables {
    save_to_database(table.to_json()?).await?;
}
```

### Anti-Patterns to Avoid

❌ **DON'T:** Mix extraction with HTTP fetching
```rust
// ❌ Bad: HTTP client in extraction domain
pub async fn extract_article(url: &str) -> Result<ExtractedContent> {
    let html = reqwest::get(url).await?.text().await?;  // ❌
    extract_from_html(&html).await
}
```

✅ **DO:** Separate fetching from extraction
```rust
// ✅ Good: Extract from HTML string
pub async fn extract_article(html: &str, url: &str) -> Result<ExtractedContent> {
    css_extraction::extract_article(html, url).await
}

// Fetching happens elsewhere (riptide-fetch)
let html = fetcher.fetch(url).await?;
let content = extract_article(&html, url).await?;
```

## Integration Points

### How Facades Use This Crate

**riptide-facade:**
```rust
use riptide_extraction::{StrategyManager, ExtractionStrategyType};

pub struct ExtractionFacade {
    manager: StrategyManager,
}

impl ExtractionFacade {
    pub async fn extract_content(&self, html: &str, url: &str) -> Result<ExtractedContent> {
        // Domain extraction logic
        let result = self.manager.execute(
            ExtractionStrategyType::Article,
            html,
            url
        ).await?;

        Ok(result.content)
    }
}
```

### Related Crates

- **Domain Layer:**
  - `riptide-types` - Core domain types and port traits
  - `riptide-spider` - Crawling strategies
  - `riptide-search` - Search domain logic

- **Application Layer:**
  - `riptide-facade` - Extraction workflows

- **Infrastructure Layer:**
  - `riptide-browser` - JavaScript rendering for dynamic content
  - `riptide-fetch` - HTML fetching
  - `riptide-persistence` - Store extracted content

- **Composition Root:**
  - `riptide-api` - HTTP API handlers
  - `riptide-cli` - Command-line extraction

## Module Structure

```
src/
├── lib.rs                          # Public API and re-exports
├── processor.rs                    # HtmlProcessor trait
├── css_extraction.rs              # CSS selector-based extraction
├── regex_extraction.rs            # Regex pattern extraction
├── dom_utils.rs                   # DOM traversal utilities
├── extraction_strategies.rs       # Strategy implementations
├── strategies/                    # Strategy module
│   ├── mod.rs                    # ExtractionStrategy trait
│   └── manager.rs                # StrategyManager
├── composition.rs                 # Strategy composition (Phase 2D)
├── confidence.rs                  # Confidence scoring
├── confidence_integration.rs      # Confidence for strategies
├── enhanced_extractor.rs         # Enhanced extraction
├── enhanced_link_extraction.rs   # Link extraction with context
├── table_extraction.rs           # Advanced table extraction
├── tables.rs                      # Table conversion utilities
├── chunking/                      # Content chunking
│   ├── mod.rs                    # ChunkingStrategy trait
│   ├── fixed_size.rs             # Fixed-size chunking
│   ├── sentence.rs               # Sentence-based chunking
│   ├── token.rs                  # Token-based chunking
│   └── semantic.rs               # Semantic chunking
├── schema/                        # Schema extraction & learning
│   ├── mod.rs                    # Schema types
│   ├── extractor.rs              # Schema extractor
│   ├── generator.rs              # Schema learner
│   ├── validator.rs              # Schema validator
│   └── registry.rs               # Schema registry
├── parallel.rs                    # Parallel batch extraction
├── native_parser.rs              # Native HTML parser
├── html_parser.rs                # HTML parsing utilities
├── unified_extractor.rs          # Unified extraction interface
├── wasm_extraction.rs            # WASM-based extraction (optional)
└── validation.rs                  # WASM validation (optional)
```

## Features

### Default Features
```toml
default = ["css-extraction", "regex-extraction", "dom-utils", "chunking", "native-parser"]
```

### Optional Features
```toml
wasm-extractor = ["dep:wasmtime", "dep:wasmtime-wasi"]  # WASM-based extraction
strategy-traits = []                                     # Strategy trait implementations
jsonld-shortcircuit = []                                # Early return for complete JSON-LD
```

## License

Apache-2.0
