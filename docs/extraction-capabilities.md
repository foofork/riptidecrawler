# RipTide Extraction Capabilities Analysis

## Currently Available Extraction Methods

### 1. **CSS Strategy** (`--method css`)
- Uses CSS selectors to extract specific content
- Can be customized with `--selector` parameter
- Good for structured HTML with predictable class/ID patterns

### 2. **Regex Strategy** (`--method regex`)
- Pattern-based extraction using regular expressions
- Can be customized with `--pattern` parameter
- Good for extracting specific data formats (emails, URLs, numbers)

### 3. **Trek Strategy** (`--method trek`)
- WASM-based extraction (may not be fully functional)
- Was designed for fast, lightweight extraction
- Current status: Unclear if working properly

### 4. **LLM Strategy** (`--method llm`)
- Uses language models for intelligent extraction
- Requires API configuration (OpenAI/Anthropic keys)
- Status: Not tested, likely requires additional setup

### 5. **Auto Strategy** (`--method auto`) [default]
- Automatically selects the best strategy
- Currently defaults to CSS strategy in most cases

## Strategy Composition Modes

### Chain Mode
```bash
riptide extract --url "URL" --strategy "chain:css,regex"
```
- Runs strategies sequentially
- Each strategy processes the output of the previous one

### Parallel Mode
```bash
riptide extract --url "URL" --strategy "parallel:css,regex"
```
- Runs multiple strategies simultaneously
- Combines results from all strategies

### Fallback Mode
```bash
riptide extract --url "URL" --strategy "fallback:css,regex,auto"
```
- Tries strategies in order until one succeeds
- Good for handling different site structures

## Current Issues

### 1. **Extraction Output Format**
- Content is extracted as plain text, not markdown
- HTML structure (headings, lists, links) is lost during extraction
- Tables are flattened to text instead of markdown tables

### 2. **Site-Specific Extraction**
- I incorrectly added hardcoded site-specific logic
- System should use intelligent strategies instead
- CSS selectors should adapt to common patterns

### 3. **Trek/WASM Issues**
- Trek strategy references WASM module that may not be properly configured
- WASM file path issues prevent proper execution

### 4. **Missing Capabilities**

#### Not Implemented in CLI:
- **PDF extraction** - Requires pdfium library (not installed)
- **Table extraction** - API endpoint exists but not exposed via CLI
- **Markdown preservation** - HTML→Markdown conversion not working
- **Media extraction** - Images/videos not properly handled
- **JavaScript rendering** - No headless browser for dynamic content
- **Authentication** - No cookie/session handling for protected content

#### Configuration Issues:
- LLM strategy requires API keys not documented
- Redis caching works but not well integrated with CLI
- Crawl command has API mismatch (expects `urls` array)
- Search command implementation unclear

## What Should Be Working (But Isn't)

1. **Markdown-Formatted Output**
   - The system should preserve HTML structure as markdown
   - Headers (h1-h6) → # Markdown headers
   - Lists (ul/ol) → - Markdown lists
   - Links → [text](url) format
   - Tables → | Markdown | tables |

2. **Intelligent Extraction**
   - CSS strategy should find common patterns:
     - Article content: `article, main, .content, #content`
     - Headings: `h1, h2, h3, .title, .headline`
     - Navigation: `nav, .nav, .menu`
   - No hardcoding for specific sites needed

3. **Strategy Composition**
   - Chain, parallel, and fallback modes should combine strategies
   - Results should merge intelligently

## Recommended Fixes

### Priority 1: Fix Markdown Output
- Modify extraction strategies to preserve HTML structure
- Convert HTML elements to markdown during extraction
- Don't just call `.text()` which strips formatting

### Priority 2: Remove Hardcoded Logic
- Delete site-specific extraction functions
- Use intelligent CSS selector discovery
- Let strategies adapt to content patterns

### Priority 3: Fix CLI Integration
- Ensure all extraction methods work via CLI
- Add table extraction as a CLI command
- Fix crawl command API mismatch

### Priority 4: Documentation
- Document LLM configuration requirements
- Provide examples for each strategy
- Create help for strategy composition

## Testing Commands

```bash
# Test CSS extraction with custom selector
riptide extract --url "https://example.com" --method css --selector "article"

# Test strategy composition
riptide extract --url "https://news.ycombinator.com" --strategy "chain:css,regex"

# Test with metadata
riptide extract --url "https://www.rust-lang.org" --metadata

# Test fallback for resilience
riptide extract --url "https://github.com" --strategy "fallback:css,regex,auto"
```

## Conclusion

The RipTide extraction pipeline has good architecture but implementation issues:
- **Good**: Multiple strategies, composition modes, caching, API structure
- **Bad**: Plain text output, hardcoded logic, incomplete CLI integration
- **Missing**: PDF support, table CLI, markdown preservation, JS rendering

The system needs fixes to fulfill its promise of intelligent, format-preserving extraction.