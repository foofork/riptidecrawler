# Enhanced Link Extraction

This document describes the enhanced link extraction feature in `riptide-extraction` that provides context-aware link extraction with classification and rich metadata.

## Overview

The enhanced link extraction module (`enhanced_link_extraction`) extracts links from HTML documents with:

- **Anchor text**: The visible text inside the `<a>` tag
- **Surrounding context**: Text before and after the link (configurable, default 75 chars each side)
- **Link type classification**: Internal, External, Download, Anchor, Email, Phone
- **HTML attributes**: All link attributes (rel, target, data-*, aria-*, etc.)
- **Position tracking**: Character offset in the document

## Quick Start

```rust
use riptide_extraction::enhanced_link_extraction::EnhancedLinkExtractor;

// Create extractor with default configuration
let extractor = EnhancedLinkExtractor::new();

// Extract all links from HTML
let links = extractor.extract_links(html, Some("https://example.com"))?;

// Access link information
for link in links {
    println!("URL: {}", link.url);
    println!("Text: {}", link.anchor_text);
    println!("Type: {:?}", link.link_type);
    println!("Context: {}", link.surrounding_context);
}
```

## Data Structures

### ExtractedLink

The main data structure containing all link information:

```rust
pub struct ExtractedLink {
    /// The absolute or relative URL
    pub url: String,

    /// The anchor text (text inside <a> tag)
    pub anchor_text: String,

    /// Surrounding context (text before and after the link)
    pub surrounding_context: String,

    /// Classified link type
    pub link_type: LinkType,

    /// HTML attributes (rel, target, data-*, etc.)
    pub attributes: HashMap<String, String>,

    /// Position in the document (character offset)
    pub position: usize,
}
```

### LinkType

Link classification enum:

```rust
pub enum LinkType {
    Internal,  // Same domain as base URL
    External,  // Different domain
    Download,  // File downloads (.pdf, .zip, etc.)
    Anchor,    // Fragment links (#section)
    Email,     // mailto: links
    Phone,     // tel: links
    Other,     // Unknown or other types
}
```

## Configuration

Customize extraction behavior with `LinkExtractionConfig`:

```rust
use riptide_extraction::enhanced_link_extraction::{
    EnhancedLinkExtractor, LinkExtractionConfig
};

let config = LinkExtractionConfig {
    // Context extraction
    context_chars_before: 75,
    context_chars_after: 75,

    // Link type filtering
    extract_internal: true,
    extract_external: true,
    extract_downloads: true,
    extract_anchors: true,
    extract_special: true,  // Email/phone links

    // Download detection
    download_extensions: vec![
        "pdf".to_string(),
        "zip".to_string(),
        "doc".to_string(),
        // ... more extensions
    ],

    // Limits
    max_links: None,  // Or Some(100) to limit
};

let extractor = EnhancedLinkExtractor::with_config(config);
```

## Usage Examples

### Extract All Links

```rust
let extractor = EnhancedLinkExtractor::new();
let links = extractor.extract_links(html, Some("https://example.com"))?;

for link in links {
    println!("{} -> {}", link.anchor_text, link.url);
}
```

### Extract Links by Type

```rust
let grouped = extractor.extract_links_by_type(html, Some("https://example.com"))?;

// Access links by type
if let Some(internal_links) = grouped.get(&LinkType::Internal) {
    println!("Found {} internal links", internal_links.len());
}

if let Some(download_links) = grouped.get(&LinkType::Download) {
    println!("Found {} downloadable files", download_links.len());
}
```

### Extract Only Internal Links

```rust
let internal_links = extractor.extract_internal_links(
    html,
    Some("https://example.com")
)?;

for link in internal_links {
    println!("Internal: {} -> {}", link.anchor_text, link.url);
}
```

### Extract Only External Links

```rust
let external_links = extractor.extract_external_links(
    html,
    Some("https://example.com")
)?;

for link in external_links {
    println!("External: {} -> {}", link.anchor_text, link.url);

    // Check for security attributes
    if link.attributes.get("rel").map_or(false, |v| v.contains("nofollow")) {
        println!("  (nofollow)");
    }
}
```

### Access Link Attributes

```rust
let links = extractor.extract_links(html, Some("https://example.com"))?;

for link in links {
    // Check specific attributes
    if let Some(rel) = link.attributes.get("rel") {
        println!("rel attribute: {}", rel);
    }

    if let Some(target) = link.attributes.get("target") {
        println!("Opens in: {}", target);
    }

    // Access data-* attributes
    if let Some(tracking) = link.attributes.get("data-tracking") {
        println!("Tracking ID: {}", tracking);
    }

    // Check aria-* attributes for accessibility
    if let Some(label) = link.attributes.get("aria-label") {
        println!("Aria label: {}", label);
    }
}
```

### Filter by Configuration

```rust
let config = LinkExtractionConfig {
    extract_internal: false,  // Skip internal links
    extract_external: true,
    extract_special: false,   // Skip email/phone
    max_links: Some(50),      // Limit to 50 links
    ..Default::default()
};

let extractor = EnhancedLinkExtractor::with_config(config);
let links = extractor.extract_links(html, Some("https://example.com"))?;
// Only external links, max 50
```

### Custom Download Extensions

```rust
let mut config = LinkExtractionConfig::default();
config.download_extensions.push("sketch".to_string());
config.download_extensions.push("fig".to_string());

let extractor = EnhancedLinkExtractor::with_config(config);
let links = extractor.extract_links(html, None)?;

// Now .sketch and .fig files are classified as downloads
```

## JSON Output

All extracted links can be serialized to JSON:

```rust
use serde_json;

let links = extractor.extract_links(html, Some("https://example.com"))?;
let json = serde_json::to_string_pretty(&links)?;
println!("{}", json);
```

Example JSON output:

```json
{
  "url": "https://example.com/page",
  "anchor_text": "Click here",
  "surrounding_context": "For more information, click here to learn about our products and services.",
  "link_type": "internal",
  "attributes": {
    "rel": "nofollow",
    "target": "_blank",
    "data-tracking": "nav-link-001"
  },
  "position": 245
}
```

## Link Classification Logic

### Internal vs External

- **Internal**: Same `host` as base URL, or relative URLs
- **External**: Different `host` from base URL

```rust
// Base URL: https://example.com

// Internal:
"/page"                    // Relative
"page.html"                // Relative
"https://example.com/page" // Same host

// External:
"https://other.com/page"   // Different host
```

### Download Detection

Links are classified as downloads if they end with known file extensions:

- Documents: `.pdf`, `.doc`, `.docx`, `.xls`, `.xlsx`, `.ppt`, `.pptx`
- Archives: `.zip`, `.tar`, `.gz`, `.rar`
- Executables: `.exe`, `.dmg`, `.pkg`, `.deb`, `.rpm`

Extensions are matched case-insensitively.

### Special Link Types

- **Email**: `href` starts with `mailto:`
- **Phone**: `href` starts with `tel:`
- **Anchor**: `href` starts with `#`

## Context Extraction

The surrounding context is extracted from the full text content of the page:

1. Find the anchor text in the page's text content
2. Extract N characters before and after (configurable)
3. Clean and normalize whitespace
4. Return as a single string

Example:

```html
<p>This is some text before the link. Click <a href="/page">here</a> to continue reading more content.</p>
```

With default config (75 chars before/after):

```
surrounding_context: "This is some text before the link. Click here to continue reading more content."
```

## Performance Considerations

- **Parsing**: Uses the fast `scraper` crate for HTML parsing
- **Memory**: Links are collected into a Vec, suitable for typical web pages
- **Limits**: Use `max_links` to cap extraction for very large documents
- **Context**: Full text extraction is done once, then reused for all links

## Integration with Extraction Pipeline

The enhanced link extractor integrates with the existing extraction pipeline:

```rust
use riptide_extraction::{EnhancedLinkExtractor, css_extract_default};

// Extract content
let content = css_extract_default(html, url).await?;

// Extract enhanced links
let link_extractor = EnhancedLinkExtractor::new();
let links = link_extractor.extract_links(html, Some(url))?;

// Combine results
let result = ExtractionResult {
    content,
    links,
    // ... other fields
};
```

## Backward Compatibility

The enhanced link extraction is fully backward compatible:

- The existing `Link` struct in `html_parser.rs` remains unchanged
- Enhanced extraction is in a separate module
- Both can be used independently or together

## Testing

Comprehensive test coverage (25+ tests) including:

- Anchor text extraction
- Context extraction with configurable limits
- Link type classification (all types)
- Attribute extraction (rel, target, data-*, aria-*)
- Internal/external detection
- Download link detection
- URL resolution (relative to absolute)
- Configuration filtering
- Edge cases (empty hrefs, special characters, etc.)

Run tests:

```bash
cargo test -p riptide-extraction --test link_extraction_tests
```

## Examples

See the full example:

```bash
cargo run --example enhanced_link_extraction_example
```

This demonstrates:
1. Basic extraction
2. Grouping by type
3. Filtering (internal/external only)
4. Custom configuration
5. JSON output
6. Statistics calculation

## API Reference

### EnhancedLinkExtractor

```rust
impl EnhancedLinkExtractor {
    /// Create with default configuration
    pub fn new() -> Self

    /// Create with custom configuration
    pub fn with_config(config: LinkExtractionConfig) -> Self

    /// Extract all links with context and classification
    pub fn extract_links(&self, html: &str, base_url: Option<&str>) -> Result<Vec<ExtractedLink>>

    /// Extract links grouped by type
    pub fn extract_links_by_type(&self, html: &str, base_url: Option<&str>) -> Result<HashMap<LinkType, Vec<ExtractedLink>>>

    /// Extract only internal links
    pub fn extract_internal_links(&self, html: &str, base_url: Option<&str>) -> Result<Vec<ExtractedLink>>

    /// Extract only external links
    pub fn extract_external_links(&self, html: &str, base_url: Option<&str>) -> Result<Vec<ExtractedLink>>
}
```

## Use Cases

1. **SEO Analysis**: Identify internal vs external link ratios, nofollow links
2. **Content Migration**: Map all links before moving content
3. **Link Validation**: Check for broken links with context
4. **Navigation Analysis**: Understand site structure through link patterns
5. **Accessibility Auditing**: Check for proper aria labels and titles
6. **Security Analysis**: Identify external links without proper rel attributes
7. **Content Extraction**: Maintain link context for AI/ML training data

## Future Enhancements

Potential improvements for future versions:

- Link validation (check if URLs are reachable)
- Link depth tracking (how many clicks from root)
- Image link detection and classification
- Social media link detection
- Broken link detection
- Link text quality scoring
- Duplicate link detection
- Outbound link analysis
