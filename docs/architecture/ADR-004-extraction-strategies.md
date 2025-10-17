# ADR-004: Multi-Strategy Content Extraction

## Status
**Accepted** - Fully implemented with Phase 4 enhancements

## Context
Web scraping requires extracting structured data from diverse HTML sources. Different websites require different extraction approaches, and a single strategy often fails to handle all cases effectively. We needed a flexible extraction system that:

1. Supports multiple extraction strategies
2. Allows strategy combination and fallback
3. Handles dynamic content and complex layouts
4. Provides high accuracy and reliability
5. Scales to handle diverse content types

### Extraction Challenges
- **Static HTML**: Simple selector-based extraction
- **Dynamic Content**: JavaScript-rendered content
- **Complex Layouts**: Nested tables, lists, custom elements
- **Inconsistent Markup**: Varied HTML structure across sites
- **Anti-scraping**: Obfuscated content, dynamic class names
- **Performance**: Balance accuracy with speed

## Decision
**Implement a multi-strategy extraction system with 5 complementary approaches and intelligent fallback.**

## Architecture Overview

```rust
pub enum ExtractionMode {
    Css,        // CSS selector extraction
    Regex,      // Pattern-based extraction
    Dom,        // DOM tree traversal
    Spider,     // spider library integration
    Wasm,       // Custom WASM modules
    Auto,       // Automatic strategy selection
    Hybrid,     // Combine multiple strategies
}

pub struct Extractor {
    strategies: Vec<Box<dyn ExtractionStrategy>>,
    config: ExtractionConfig,
    fallback_chain: Vec<ExtractionMode>,
}

pub trait ExtractionStrategy {
    fn extract(&self, content: &[u8], url: &str) -> Result<ExtractionResult>;
    fn supports(&self, content_type: &ContentType) -> bool;
    fn priority(&self) -> u8;
}
```

## Extraction Strategies

### 1. CSS Selector Extraction
**Use Case**: Well-structured HTML with consistent classes/IDs

**Implementation**:
```rust
pub struct CssExtractor {
    selectors: Vec<String>,
    scraper: Scraper,
}

impl CssExtractor {
    pub fn extract(&self, html: &str, selector: &str) -> Result<Vec<String>> {
        let document = Html::parse_document(html);
        let selector = Selector::parse(selector)?;

        let results: Vec<String> = document
            .select(&selector)
            .map(|element| element.text().collect::<String>())
            .collect();

        Ok(results)
    }
}
```

**Strengths**:
- Fast and efficient
- Precise targeting
- Standard web development skill
- Rich selector syntax

**Weaknesses**:
- Requires consistent markup
- Breaks with dynamic class names
- Needs manual selector creation

**Best For**:
- Structured websites
- Known page layouts
- Static content

### 2. Regex Pattern Extraction
**Use Case**: Pattern-based content with consistent format

**Implementation**:
```rust
pub struct RegexExtractor {
    patterns: HashMap<String, Regex>,
}

impl RegexExtractor {
    pub fn extract(&self, text: &str, pattern: &str) -> Result<Vec<String>> {
        let regex = Regex::new(pattern)?;

        let results: Vec<String> = regex
            .captures_iter(text)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        Ok(results)
    }

    // Pre-defined patterns for common data types
    pub fn extract_emails(&self, text: &str) -> Result<Vec<String>> {
        self.extract(text, r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")
    }

    pub fn extract_phone_numbers(&self, text: &str) -> Result<Vec<String>> {
        self.extract(text, r"\+?1?\s*\(?[0-9]{3}\)?[\s.-]?[0-9]{3}[\s.-]?[0-9]{4}")
    }

    pub fn extract_urls(&self, text: &str) -> Result<Vec<String>> {
        self.extract(text, r"https?://[^\s<>\"]+")
    }
}
```

**Strengths**:
- Works without DOM parsing
- Fast on large text
- Good for specific patterns
- Independent of HTML structure

**Weaknesses**:
- Complex patterns hard to maintain
- May match unintended content
- Limited context awareness

**Best For**:
- Email extraction
- Phone number extraction
- URL extraction
- Structured text data

### 3. DOM Tree Traversal
**Use Case**: Complex nested structures, relative positioning

**Implementation**:
```rust
pub struct DomExtractor {
    parser: DomParser,
}

impl DomExtractor {
    pub fn extract_by_path(&self, html: &str, path: &[DomSelector]) -> Result<Vec<Node>> {
        let document = self.parser.parse(html)?;
        let mut current = vec![document.root()];

        for selector in path {
            current = current.into_iter()
                .flat_map(|node| node.select(selector))
                .collect();
        }

        Ok(current)
    }

    pub fn extract_table(&self, html: &str, table_selector: &str) -> Result<Vec<Vec<String>>> {
        let document = Html::parse_document(html);
        let table_sel = Selector::parse(table_selector)?;
        let row_sel = Selector::parse("tr")?;
        let cell_sel = Selector::parse("td, th")?;

        let tables: Vec<Vec<Vec<String>>> = document
            .select(&table_sel)
            .map(|table| {
                table.select(&row_sel)
                    .map(|row| {
                        row.select(&cell_sel)
                            .map(|cell| cell.text().collect::<String>().trim().to_string())
                            .collect()
                    })
                    .collect()
            })
            .collect();

        Ok(tables.into_iter().flatten().collect())
    }
}
```

**Strengths**:
- Handles complex structures
- Context-aware extraction
- Flexible navigation
- Good for tables and lists

**Weaknesses**:
- More complex logic
- Slower than CSS selectors
- Requires understanding of structure

**Best For**:
- Table extraction
- Nested list extraction
- Structured data with relationships
- Content based on position

### 4. Spider Integration
**Use Case**: Comprehensive site crawling and extraction

**Implementation**:
```rust
pub struct SpiderExtractor {
    spider: Spider,
    config: SpiderConfig,
}

impl SpiderExtractor {
    pub async fn extract(&self, url: &str) -> Result<SpiderResult> {
        let spider = Spider::builder()
            .url(url)
            .depth(self.config.max_depth)
            .concurrency(self.config.concurrency)
            .build()?;

        let pages = spider.crawl().await?;

        let results: Vec<PageData> = pages
            .into_iter()
            .map(|page| self.extract_page_data(&page))
            .collect::<Result<Vec<_>>>()?;

        Ok(SpiderResult { pages: results })
    }

    fn extract_page_data(&self, page: &Page) -> Result<PageData> {
        Ok(PageData {
            url: page.url().to_string(),
            title: page.title(),
            meta: page.meta_tags(),
            links: page.links().map(|l| l.to_string()).collect(),
            content: page.text_content(),
        })
    }
}
```

**Strengths**:
- Comprehensive extraction
- Follows links automatically
- Handles pagination
- JavaScript rendering support

**Weaknesses**:
- Slower for single pages
- May extract too much data
- Requires more resources

**Best For**:
- Multi-page extraction
- Site-wide crawling
- Discovery of related content
- Dynamic content

### 5. WASM Module Extraction
**Use Case**: Custom extraction logic, performance-critical operations

**Implementation**:
```rust
pub struct WasmExtractor {
    runtime: WasmRuntime,
    modules: HashMap<String, WasmModule>,
}

impl WasmExtractor {
    pub fn extract(&self, html: &[u8], module_name: &str) -> Result<ExtractionResult> {
        let module = self.modules.get(module_name)
            .ok_or_else(|| anyhow!("WASM module '{}' not found", module_name))?;

        // Call WASM function
        let result = module.call_function("extract", html)?;

        Ok(result)
    }

    pub fn load_module(&mut self, name: String, wasm_bytes: &[u8]) -> Result<()> {
        let module = self.runtime.compile(wasm_bytes)?;
        self.modules.insert(name, module);
        Ok(())
    }
}
```

**Example WASM Module** (Rust → WASM):
```rust
#[no_mangle]
pub extern "C" fn extract(html_ptr: *const u8, html_len: usize) -> *const u8 {
    let html = unsafe {
        std::slice::from_raw_parts(html_ptr, html_len)
    };

    // Custom extraction logic
    let result = perform_custom_extraction(html);

    // Return result pointer
    Box::into_raw(result.into_boxed_slice()) as *const u8
}
```

**Strengths**:
- Custom logic without recompilation
- Near-native performance
- Sandboxed execution
- Language-agnostic (Rust, C++, AssemblyScript)

**Weaknesses**:
- Complex to develop
- Requires WASM compilation
- Debugging difficulty

**Best For**:
- Site-specific extraction
- Performance-critical operations
- Custom parsing algorithms
- User-provided extraction logic

## Hybrid Strategy

### Auto Mode
Automatically selects the best strategy based on content:

```rust
impl Extractor {
    pub fn auto_extract(&self, content: &[u8], url: &str) -> Result<ExtractionResult> {
        // Analyze content
        let content_type = self.detect_content_type(content)?;

        // Select strategy
        let strategy = match content_type {
            ContentType::SimpleHtml => ExtractionMode::Css,
            ContentType::ComplexHtml => ExtractionMode::Dom,
            ContentType::DynamicHtml => ExtractionMode::Spider,
            ContentType::ObfuscatedHtml => ExtractionMode::Regex,
            ContentType::Custom => ExtractionMode::Wasm,
        };

        self.extract_with_strategy(content, url, strategy)
    }
}
```

### Fallback Chain
Try multiple strategies in sequence:

```rust
impl Extractor {
    pub fn extract_with_fallback(&self, content: &[u8], url: &str) -> Result<ExtractionResult> {
        let fallback_chain = vec![
            ExtractionMode::Css,      // Try CSS first (fastest)
            ExtractionMode::Dom,      // Then DOM traversal
            ExtractionMode::Regex,    // Then regex patterns
            ExtractionMode::Spider,   // Finally spider
        ];

        for mode in fallback_chain {
            match self.extract_with_strategy(content, url, mode) {
                Ok(result) if result.is_valid() => return Ok(result),
                _ => continue,
            }
        }

        Err(anyhow!("All extraction strategies failed"))
    }
}
```

### Combined Strategies
Use multiple strategies together:

```rust
impl Extractor {
    pub fn extract_hybrid(&self, content: &[u8], url: &str) -> Result<ExtractionResult> {
        // Extract with CSS
        let css_result = self.css_extractor.extract(content, url)?;

        // Enhance with regex
        let enhanced = self.regex_extractor.enhance(&css_result)?;

        // Validate with DOM
        let validated = self.dom_extractor.validate(&enhanced)?;

        Ok(validated)
    }
}
```

## Configuration

```rust
pub struct ExtractionConfig {
    pub mode: ExtractionMode,
    pub fallback_enabled: bool,
    pub fallback_chain: Vec<ExtractionMode>,
    pub timeout: Duration,
    pub max_retries: u32,
    pub validate_results: bool,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            mode: ExtractionMode::Auto,
            fallback_enabled: true,
            fallback_chain: vec![
                ExtractionMode::Css,
                ExtractionMode::Dom,
                ExtractionMode::Regex,
            ],
            timeout: Duration::from_secs(30),
            max_retries: 3,
            validate_results: true,
        }
    }
}
```

## Performance Comparison

| Strategy | Speed | Accuracy | Complexity | Resource Usage |
|----------|-------|----------|------------|----------------|
| CSS      | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐ | Low |
| Regex    | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | Low |
| DOM      | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | Medium |
| Spider   | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | High |
| WASM     | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | Medium |

## Consequences

### Positive
- **Flexibility**: Multiple strategies for different scenarios
- **Reliability**: Fallback chain ensures high success rate
- **Performance**: Fast strategies tried first
- **Extensibility**: Easy to add new strategies
- **Customization**: WASM modules for site-specific logic
- **Accuracy**: Combined strategies improve results

### Negative
- **Complexity**: More code to maintain
- **Learning Curve**: Users need to understand strategies
- **Resource Usage**: Multiple attempts increase overhead
- **Configuration**: More options to configure

### Mitigation
- **Auto Mode**: Simplifies strategy selection
- **Presets**: Common configurations provided
- **Documentation**: Clear guide for each strategy
- **Monitoring**: Track strategy success rates
- **Caching**: Cache successful strategy per domain

## Related ADRs
- ADR-001: Browser Automation Strategy
- ADR-002: Module Boundaries
- ADR-003: Stealth Architecture

## Future Enhancements
- [ ] Machine learning-based strategy selection
- [ ] Visual extraction (screenshot analysis)
- [ ] Natural language extraction ("get all prices")
- [ ] Schema.org / microdata extraction
- [ ] LLM-powered extraction

## References
- [scraper crate](https://docs.rs/scraper/)
- [regex crate](https://docs.rs/regex/)
- [spider library](https://docs.rs/spider/)
- [wasmer runtime](https://docs.rs/wasmer/)

---
**Last Updated**: 2025-10-17
**Approved By**: Architecture Team
**Review Date**: 2025-11-17
