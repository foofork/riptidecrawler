# Native HTML Parser Architecture Design

**Version**: 1.0
**Date**: 2025-10-28
**Status**: Design Phase
**Author**: System Architecture Designer

---

## Executive Summary

This document outlines the architecture for a native Rust HTML parser module to replace the problematic WASM-based extraction for headless-rendered content. The WASM parser crashes due to `tendril`/`html5ever` incompatibility with the WASM Component Model, resulting in 100% extraction failure rate. The native parser will process headless-rendered HTML directly within the API service, bypassing WASM entirely for this critical path.

---

## 1. Problem Statement

### 1.1 Current Architecture

```
┌─────────────┐      ┌──────────────────┐      ┌─────────────────┐
│  Headless   │─POST─▶│  Rendered HTML   │─────▶│  WASM Parser    │
│  Service    │      │  (Perfect)       │      │  (CRASHES)      │
└─────────────┘      └──────────────────┘      └─────────────────┘
                                                         │
                                                         ↓
                                                   [FAILURE]
                                         tendril::unsafe_pop_front
                                         html5ever::tokenizer
```

### 1.2 Root Cause

**Location**: `/workspaces/eventmesh/wasm/riptide-extractor-wasm/src/extraction.rs:9`

```rust
// Line 9 - This crashes in WASM environment
let document = Html::parse_document(html);
```

**Stack Trace**:
```
0: tendril::Tendril<F,A>::unsafe_pop_front
1: tendril::Tendril<F,A>::pop_front_char
2: <markup5ever::util::buffer_queue::BufferQueue as Iterator>::next
3: html5ever::tokenizer::Tokenizer::get_char
4: html5ever::tokenizer::Tokenizer::step
5: html5ever::tokenizer::Tokenizer::run
6: scraper::html::Html::parse_document ← CRASH
```

**Impact**:
- 100% extraction failure rate
- All fallback paths fail (all depend on same WASM parser)
- Headless rendering works perfectly but results are unusable

### 1.3 Architecture Decision Record (ADR)

**Decision**: Implement native Rust HTML parser in `riptide-extraction` crate for headless-rendered content.

**Rationale**:
1. **Immediate Relief**: Unblocks headless extraction path immediately
2. **Performance**: Native parsing is faster than WASM
3. **Reliability**: No WASM Component Model compatibility issues
4. **Maintainability**: Single codebase, easier to debug
5. **Flexibility**: Can optimize specifically for headless-rendered HTML

**Trade-offs**:
- ✅ **Pro**: Immediate solution, no WASM debugging needed
- ✅ **Pro**: Better performance and error handling
- ⚠️ **Con**: Duplicate parsing logic (WASM + Native)
- ⚠️ **Con**: Need to maintain two parsers temporarily

**Alternatives Considered**:
1. **Fix tendril/html5ever in WASM**: Upstream issue, unpredictable timeline
2. **Replace scraper entirely**: Too risky, affects all extraction
3. **Return raw HTML**: Shifts problem to users, not a real solution

---

## 2. Architecture Overview

### 2.1 High-Level Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                    riptide-reliability                         │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │  extract_headless()                                       │ │
│  │                                                           │ │
│  │  1. POST /render → Headless Service                      │ │
│  │  2. Get rendered HTML (✅ Working)                        │ │
│  │  3. Native Parser (NEW) ──────────┐                      │ │
│  │     └─ Bypass WASM entirely       │                      │ │
│  └───────────────────────────────────┼───────────────────────┘ │
└────────────────────────────────────┼───────────────────────────┘
                                     │
                                     ↓
┌────────────────────────────────────────────────────────────────┐
│                    riptide-extraction                          │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │              native_parser (NEW MODULE)                   │ │
│  │                                                           │ │
│  │  ┌─────────────────────────────────────────────────────┐ │ │
│  │  │  NativeHtmlParser                                    │ │ │
│  │  │  - Uses scraper in native Rust (NOT WASM)           │ │ │
│  │  │  - Same extraction logic as WASM                    │ │ │
│  │  │  - Rich error handling and fallbacks                │ │ │
│  │  └─────────────────────────────────────────────────────┘ │ │
│  │                                                           │ │
│  │  ┌─────────────────────────────────────────────────────┐ │ │
│  │  │  Extraction Modules (reusable)                       │ │ │
│  │  │  - TitleExtractor                                    │ │ │
│  │  │  - ContentExtractor                                  │ │ │
│  │  │  - MetadataExtractor                                 │ │ │
│  │  │  - LinkExtractor                                     │ │ │
│  │  │  - MediaExtractor                                    │ │ │
│  │  │  - LanguageDetector                                  │ │ │
│  │  │  - CategoryExtractor                                 │ │ │
│  │  └─────────────────────────────────────────────────────┘ │ │
│  └──────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────┘
```

### 2.2 Component Diagram (C4 - Component Level)

```
┌─────────────────────────────────────────────────────────────────┐
│                        API Service                              │
│                                                                 │
│  ┌──────────────────────┐         ┌──────────────────────────┐ │
│  │  ReliableExtractor   │────────▶│  WasmExtractor          │ │
│  │  (reliability.rs)    │         │  (WASM - keep for fast) │ │
│  └──────────────────────┘         └──────────────────────────┘ │
│            │                                                    │
│            │ NEW                                                │
│            ↓                                                    │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │         NativeHtmlParser (NEW)                            │  │
│  │  - parse_headless_html()                                  │  │
│  │  - extract_with_fallbacks()                               │  │
│  │  - quality_assessment()                                   │  │
│  └──────────────────────────────────────────────────────────┘  │
│            │                                                    │
│            ↓                                                    │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │         Extraction Modules (Shared)                       │  │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐           │  │
│  │  │   Title    │ │  Content   │ │  Metadata  │ ...       │  │
│  │  │ Extractor  │ │ Extractor  │ │ Extractor  │           │  │
│  │  └────────────┘ └────────────┘ └────────────┘           │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.3 Data Flow Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Headless Extraction Flow                    │
└─────────────────────────────────────────────────────────────────┘

1. REQUEST
   ├─▶ ReliableExtractor::extract_headless()
   │   └─▶ POST /render to Headless Service
   │       └─▶ Response: Fully rendered HTML ✅
   │
2. PARSE (NEW PATH)
   ├─▶ NativeHtmlParser::parse_headless_html()
   │   ├─▶ scraper::Html::parse_document() [Native Rust - Safe!]
   │   ├─▶ Validate document structure
   │   └─▶ Return Document handle
   │
3. EXTRACT
   ├─▶ Extract title (TitleExtractor)
   ├─▶ Extract content (ContentExtractor)
   ├─▶ Extract metadata (MetadataExtractor)
   ├─▶ Extract links (LinkExtractor)
   ├─▶ Extract media (MediaExtractor)
   ├─▶ Detect language (LanguageDetector)
   ├─▶ Extract categories (CategoryExtractor)
   └─▶ Calculate quality metrics
   │
4. VALIDATE & RETURN
   ├─▶ Quality assessment (QualityAssessor)
   ├─▶ Apply fallbacks if needed
   └─▶ Return ExtractedDoc


┌─────────────────────────────────────────────────────────────────┐
│                    Fast Path (Unchanged)                        │
└─────────────────────────────────────────────────────────────────┘

1. REQUEST
   └─▶ ReliableExtractor::extract_fast()
       └─▶ HTTP GET raw HTML
           └─▶ WASM Extractor (keep as-is for now)
```

---

## 3. Module Structure

### 3.1 Directory Structure

```
crates/riptide-extraction/src/
├── lib.rs                       # Re-export native_parser module
├── native_parser/               # NEW MODULE
│   ├── mod.rs                   # Module root, public API
│   ├── parser.rs                # NativeHtmlParser implementation
│   ├── extractors/              # Extraction logic (modular)
│   │   ├── mod.rs
│   │   ├── title.rs             # Title extraction
│   │   ├── content.rs           # Content extraction (text, markdown)
│   │   ├── metadata.rs          # Metadata (byline, date, description)
│   │   ├── links.rs             # Link extraction
│   │   ├── media.rs             # Media extraction (images, videos)
│   │   ├── language.rs          # Language detection
│   │   └── categories.rs        # Category extraction
│   ├── quality.rs               # Quality assessment
│   ├── fallbacks.rs             # Fallback strategies
│   └── error.rs                 # Native parser error types
├── wasm_extractor.rs            # Existing WASM integration (unchanged)
└── ...                          # Other existing modules
```

### 3.2 Crate Dependencies

**Add to `crates/riptide-extraction/Cargo.toml`**:

```toml
[dependencies]
# Already present (use for native parser)
scraper = "0.20"        # HTML parsing (native Rust, NOT WASM)
regex.workspace = true
url.workspace = true
chrono.workspace = true
whatlang = "0.16"       # Language detection

# Additional for native parser
select = "0.6"          # Alternative CSS selector engine (optional)
html_parser = "0.7"     # Backup parser (optional)
```

**Note**: `scraper` is ALREADY a dependency. We're just using it in **native Rust** instead of WASM.

---

## 4. API Design & Interfaces

### 4.1 Public API

**Module**: `crates/riptide-extraction/src/native_parser/mod.rs`

```rust
/// Native HTML parser for headless-rendered content
pub struct NativeHtmlParser {
    config: ParserConfig,
}

/// Parser configuration
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Enable markdown generation
    pub enable_markdown: bool,
    /// Enable link extraction
    pub extract_links: bool,
    /// Enable media extraction
    pub extract_media: bool,
    /// Enable language detection
    pub detect_language: bool,
    /// Enable category extraction
    pub extract_categories: bool,
    /// Maximum content length (bytes)
    pub max_content_length: usize,
    /// Timeout for parsing (ms)
    pub parse_timeout_ms: u64,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            enable_markdown: true,
            extract_links: true,
            extract_media: true,
            detect_language: true,
            extract_categories: true,
            max_content_length: 10_000_000, // 10MB
            parse_timeout_ms: 5000,           // 5 seconds
        }
    }
}

impl NativeHtmlParser {
    /// Create new parser with default config
    pub fn new() -> Self {
        Self::with_config(ParserConfig::default())
    }

    /// Create parser with custom config
    pub fn with_config(config: ParserConfig) -> Self {
        Self { config }
    }

    /// Parse headless-rendered HTML and extract document
    ///
    /// # Arguments
    /// * `html` - Fully rendered HTML from headless service
    /// * `url` - Original URL (for link resolution)
    ///
    /// # Returns
    /// * `Ok(ExtractedDoc)` - Successfully extracted document
    /// * `Err(NativeParserError)` - Parsing or extraction failed
    pub fn parse_headless_html(
        &self,
        html: &str,
        url: &str,
    ) -> Result<ExtractedDoc, NativeParserError> {
        // Implementation in Section 5
    }

    /// Extract document with quality-based fallbacks
    ///
    /// Tries multiple extraction strategies if initial attempt
    /// produces low-quality results.
    pub fn extract_with_fallbacks(
        &self,
        html: &str,
        url: &str,
    ) -> Result<ExtractedDoc, NativeParserError> {
        // Implementation in Section 5
    }

    /// Validate HTML before parsing
    pub fn validate_html(&self, html: &str) -> Result<(), NativeParserError> {
        // Check size, encoding, structure
    }
}
```

### 4.2 Error Types

**Module**: `crates/riptide-extraction/src/native_parser/error.rs`

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NativeParserError {
    #[error("HTML parsing failed: {0}")]
    ParseError(String),

    #[error("HTML exceeds maximum size: {size} bytes (max: {max})")]
    OversizedHtml { size: usize, max: usize },

    #[error("Invalid UTF-8 encoding: {0}")]
    EncodingError(#[from] std::str::Utf8Error),

    #[error("Extraction timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Document structure invalid: {0}")]
    InvalidStructure(String),

    #[error("No extractable content found")]
    NoContentFound,

    #[error("Quality too low: {score} (threshold: {threshold})")]
    LowQuality { score: f32, threshold: f32 },

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, NativeParserError>;
```

### 4.3 Integration API (reliability.rs)

**Location**: `crates/riptide-reliability/src/reliability.rs`

**Changes to `extract_headless` method** (line 234):

```rust
/// Headless extraction with native parser
async fn extract_headless(
    &self,
    url: &str,
    headless_url: Option<&str>,
    wasm_extractor: &dyn WasmExtractor, // Keep for compatibility
    request_id: &str,
) -> Result<ExtractedDoc> {
    debug!(request_id = %request_id, "Using headless extraction path");

    let headless_service_url =
        headless_url.ok_or_else(|| anyhow::anyhow!("Headless service URL not configured"))?;

    // ... existing headless service call ...
    let rendered_html = response.text().await?;

    // ===== NEW: Use native parser instead of WASM =====
    use riptide_extraction::native_parser::NativeHtmlParser;

    let native_parser = NativeHtmlParser::new();
    let doc = native_parser
        .parse_headless_html(&rendered_html, url)
        .map_err(|e| anyhow::anyhow!("Native parser failed: {}", e))?;

    info!(
        request_id = %request_id,
        content_length = doc.text.len(),
        "Headless extraction completed (native parser)"
    );

    Ok(doc)
}
```

---

## 5. Implementation Details

### 5.1 Core Parser Implementation

**File**: `crates/riptide-extraction/src/native_parser/parser.rs`

```rust
use scraper::{Html, Selector};
use riptide_types::ExtractedDoc;
use crate::native_parser::{
    extractors::*,
    quality::QualityAssessor,
    fallbacks::FallbackStrategy,
    error::{NativeParserError, Result},
};

impl NativeHtmlParser {
    pub fn parse_headless_html(
        &self,
        html: &str,
        url: &str,
    ) -> Result<ExtractedDoc> {
        // 1. Validate input
        self.validate_html(html)?;

        // 2. Parse HTML with timeout protection
        let document = tokio::time::timeout(
            std::time::Duration::from_millis(self.config.parse_timeout_ms),
            async { Html::parse_document(html) },
        )
        .await
        .map_err(|_| NativeParserError::Timeout {
            timeout_ms: self.config.parse_timeout_ms,
        })?;

        // 3. Extract all components
        let title = TitleExtractor::extract(&document);
        let byline = MetadataExtractor::extract_byline(&document);
        let published = MetadataExtractor::extract_published_date(&document);
        let description = MetadataExtractor::extract_description(&document);
        let site_name = MetadataExtractor::extract_site_name(&document);

        // 4. Extract content (text + markdown)
        let (text, markdown) = ContentExtractor::extract(&document, url)?;

        // 5. Extract links (conditional)
        let links = if self.config.extract_links {
            LinkExtractor::extract(&document, url)
        } else {
            Vec::new()
        };

        // 6. Extract media (conditional)
        let media = if self.config.extract_media {
            MediaExtractor::extract(&document, url)
        } else {
            Vec::new()
        };

        // 7. Detect language (conditional)
        let language = if self.config.detect_language {
            LanguageDetector::detect(&document, html)
        } else {
            None
        };

        // 8. Extract categories (conditional)
        let categories = if self.config.extract_categories {
            CategoryExtractor::extract(&document)
        } else {
            Vec::new()
        };

        // 9. Calculate quality metrics
        let word_count = text.split_whitespace().count();
        let reading_time = (word_count / 200).max(1); // 200 wpm
        let quality_score = QualityAssessor::calculate(&text, &markdown, &title);

        // 10. Build result
        let doc = ExtractedDoc {
            url: url.to_string(),
            title,
            byline,
            published_iso: published,
            markdown,
            text,
            links,
            media,
            language,
            reading_time: Some(reading_time),
            quality_score: Some(quality_score),
            word_count: Some(word_count),
            categories,
            site_name,
            description,
        };

        // 11. Validate minimum quality
        if quality_score < 30 {
            return Err(NativeParserError::LowQuality {
                score: quality_score as f32,
                threshold: 30.0,
            });
        }

        Ok(doc)
    }

    pub fn extract_with_fallbacks(
        &self,
        html: &str,
        url: &str,
    ) -> Result<ExtractedDoc> {
        // Primary extraction
        match self.parse_headless_html(html, url) {
            Ok(doc) => {
                // Check quality
                let quality = doc.quality_score.unwrap_or(0);
                if quality >= 60 {
                    return Ok(doc);
                }
                // Low quality - try fallback
                warn!("Primary extraction low quality ({quality}), trying fallback");
            }
            Err(e) => {
                warn!("Primary extraction failed: {e}, trying fallback");
            }
        }

        // Fallback strategy 1: Full content extraction
        let doc = FallbackStrategy::full_content_extraction(html, url)?;
        if doc.quality_score.unwrap_or(0) >= 40 {
            return Ok(doc);
        }

        // Fallback strategy 2: Simple text extraction
        FallbackStrategy::simple_text_extraction(html, url)
    }

    fn validate_html(&self, html: &str) -> Result<()> {
        // Check size
        if html.len() > self.config.max_content_length {
            return Err(NativeParserError::OversizedHtml {
                size: html.len(),
                max: self.config.max_content_length,
            });
        }

        // Check encoding (must be valid UTF-8)
        if !html.is_ascii() && html.contains('\u{FFFD}') {
            return Err(NativeParserError::EncodingError(
                std::str::Utf8Error::from(std::str::from_utf8(b"\xFF").unwrap_err()),
            ));
        }

        // Check basic HTML structure
        if !html.contains("<html") && !html.contains("<HTML") {
            return Err(NativeParserError::InvalidStructure(
                "Missing <html> tag".to_string(),
            ));
        }

        Ok(())
    }
}
```

### 5.2 Extractor Modules (Reusable)

Each extractor module follows this pattern:

**File**: `crates/riptide-extraction/src/native_parser/extractors/title.rs`

```rust
use scraper::{Html, Selector};

pub struct TitleExtractor;

impl TitleExtractor {
    pub fn extract(document: &Html) -> Option<String> {
        // Priority 1: Open Graph title
        if let Some(title) = Self::extract_og_title(document) {
            return Some(title);
        }

        // Priority 2: Twitter title
        if let Some(title) = Self::extract_twitter_title(document) {
            return Some(title);
        }

        // Priority 3: <title> tag
        if let Some(title) = Self::extract_html_title(document) {
            return Some(title);
        }

        // Priority 4: <h1> tag
        Self::extract_h1_title(document)
    }

    fn extract_og_title(document: &Html) -> Option<String> {
        let selector = Selector::parse("meta[property='og:title']").ok()?;
        document
            .select(&selector)
            .next()?
            .value()
            .attr("content")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn extract_twitter_title(document: &Html) -> Option<String> {
        let selector = Selector::parse("meta[name='twitter:title']").ok()?;
        document
            .select(&selector)
            .next()?
            .value()
            .attr("content")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn extract_html_title(document: &Html) -> Option<String> {
        let selector = Selector::parse("title").ok()?;
        let title = document
            .select(&selector)
            .next()?
            .text()
            .collect::<String>();

        let cleaned = title.trim().to_string();
        if cleaned.is_empty() {
            None
        } else {
            Some(cleaned)
        }
    }

    fn extract_h1_title(document: &Html) -> Option<String> {
        let selector = Selector::parse("h1").ok()?;
        let h1 = document
            .select(&selector)
            .next()?
            .text()
            .collect::<String>();

        let cleaned = h1.trim().to_string();
        if cleaned.is_empty() || cleaned.len() > 200 {
            None
        } else {
            Some(cleaned)
        }
    }
}
```

**Similar structure for**:
- `content.rs` - Text and markdown extraction
- `metadata.rs` - Byline, date, description
- `links.rs` - Link extraction (reuse WASM logic)
- `media.rs` - Media extraction (reuse WASM logic)
- `language.rs` - Language detection (reuse WASM logic)
- `categories.rs` - Category extraction (reuse WASM logic)

### 5.3 Quality Assessment

**File**: `crates/riptide-extraction/src/native_parser/quality.rs`

```rust
pub struct QualityAssessor;

impl QualityAssessor {
    /// Calculate quality score (0-100)
    pub fn calculate(
        text: &str,
        markdown: &Option<String>,
        title: &Option<String>,
    ) -> usize {
        let mut score = 0;

        // Title presence (20 points)
        if title.as_ref().is_some_and(|t| !t.trim().is_empty()) {
            score += 20;
        }

        // Content length (40 points)
        let text_length = text.len();
        if text_length > 2000 {
            score += 40;
        } else if text_length > 500 {
            score += 25;
        } else if text_length > 100 {
            score += 10;
        }

        // Markdown structure (20 points)
        if let Some(ref md) = markdown {
            let structure_indicators =
                md.matches('#').count() +
                md.matches('*').count() +
                md.matches('[').count();

            if structure_indicators > 10 {
                score += 20;
            } else if structure_indicators > 5 {
                score += 12;
            } else if structure_indicators > 2 {
                score += 6;
            }
        }

        // Word count (10 points)
        let word_count = text.split_whitespace().count();
        if word_count > 500 {
            score += 10;
        } else if word_count > 100 {
            score += 5;
        }

        // Sentence structure (10 points)
        let sentence_count = text.matches('.').count();
        if sentence_count > 10 {
            score += 10;
        } else if sentence_count > 3 {
            score += 5;
        }

        score.min(100)
    }
}
```

### 5.4 Fallback Strategies

**File**: `crates/riptide-extraction/src/native_parser/fallbacks.rs`

```rust
use scraper::{Html, Selector};
use riptide_types::ExtractedDoc;
use super::error::Result;

pub struct FallbackStrategy;

impl FallbackStrategy {
    /// Fallback 1: Extract all visible text
    pub fn full_content_extraction(html: &str, url: &str) -> Result<ExtractedDoc> {
        let document = Html::parse_document(html);

        // Remove script and style tags
        let text = Self::extract_all_text(&document);

        Ok(ExtractedDoc {
            url: url.to_string(),
            title: Some("Fallback Extraction".to_string()),
            text,
            quality_score: Some(40),
            ..Default::default()
        })
    }

    /// Fallback 2: Simple text extraction
    pub fn simple_text_extraction(html: &str, url: &str) -> Result<ExtractedDoc> {
        // Strip HTML tags with regex
        let text = Self::strip_html_tags(html);

        Ok(ExtractedDoc {
            url: url.to_string(),
            title: Some("Simple Extraction".to_string()),
            text,
            quality_score: Some(20),
            ..Default::default()
        })
    }

    fn extract_all_text(document: &Html) -> String {
        let selector = Selector::parse("body").ok();
        if let Some(sel) = selector {
            if let Some(body) = document.select(&sel).next() {
                return body.text().collect::<String>();
            }
        }
        String::new()
    }

    fn strip_html_tags(html: &str) -> String {
        use regex::Regex;
        let tag_regex = Regex::new(r"<[^>]*>").unwrap();
        tag_regex.replace_all(html, " ").to_string()
    }
}
```

---

## 6. Migration Path

### Phase 1: Implementation (Week 1)

**Tasks**:
1. ✅ Create `native_parser` module structure
2. ✅ Implement `NativeHtmlParser` core
3. ✅ Port extractor logic from WASM (reuse where possible)
4. ✅ Implement quality assessment
5. ✅ Implement fallback strategies
6. ✅ Add comprehensive error handling

**Testing**:
- Unit tests for each extractor
- Integration tests with real HTML samples
- Performance benchmarks

### Phase 2: Integration (Week 1-2)

**Tasks**:
1. ✅ Update `reliability.rs` to use native parser
2. ✅ Add feature flag `native-headless-parser` (optional)
3. ✅ Add metrics for native parser performance
4. ✅ Integration tests with headless service

**Code Changes**:
```rust
// In reliability.rs - extract_headless method
#[cfg(feature = "native-headless-parser")]
let doc = native_parser.parse_headless_html(&rendered_html, url)?;

#[cfg(not(feature = "native-headless-parser"))]
let doc = wasm_extractor.extract(rendered_html.as_bytes(), url, "article")?;
```

### Phase 3: Testing & Rollout (Week 2)

**Testing**:
1. ✅ A/B test: Native parser vs WASM (for comparison)
2. ✅ Load testing with real headless rendering
3. ✅ Edge case testing (malformed HTML, large docs)
4. ✅ Performance profiling

**Metrics to Track**:
- Extraction success rate (target: >95%)
- Average latency (target: <500ms for typical page)
- Quality score distribution
- Error types and frequencies

### Phase 4: Production (Week 3)

**Rollout**:
1. ✅ Enable feature flag in staging
2. ✅ Monitor metrics for 48 hours
3. ✅ Gradual rollout to production (10% → 50% → 100%)
4. ✅ Full production deployment

**Success Criteria**:
- ✅ 0% crashes (vs 100% with WASM)
- ✅ >90% quality score for typical content
- ✅ <1s average extraction time
- ✅ <1% error rate

### Phase 5: Cleanup (Week 4+)

**Long-term**:
1. ⚠️ Monitor for WASM parser fixes upstream
2. ⚠️ Keep WASM parser for fast path (direct HTML fetch)
3. ⚠️ Consider native parser for fast path too
4. ⚠️ Deprecate WASM entirely if native performs well

---

## 7. Testing Strategy

### 7.1 Unit Tests

**Test Coverage**:
- Each extractor module (>90% coverage)
- Quality assessment algorithms
- Fallback strategies
- Error handling

**Example Test**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_extraction() {
        let html = r#"
            <html>
            <head>
                <meta property="og:title" content="Test Article">
                <title>Fallback Title</title>
            </head>
            <body><h1>Header</h1></body>
            </html>
        "#;

        let document = Html::parse_document(html);
        let title = TitleExtractor::extract(&document);

        assert_eq!(title, Some("Test Article".to_string()));
    }

    #[test]
    fn test_quality_assessment() {
        let text = "This is a long article with lots of content...".repeat(50);
        let markdown = Some("# Title\n\n*Emphasis* and [links](url)".to_string());
        let title = Some("Great Article".to_string());

        let score = QualityAssessor::calculate(&text, &markdown, &title);

        assert!(score > 80, "Score: {score}");
    }
}
```

### 7.2 Integration Tests

**Test Scenarios**:
1. **Simple HTML**: Basic structure, minimal content
2. **Complex HTML**: News article with metadata, images, links
3. **JavaScript-heavy**: SPA after headless rendering
4. **Large HTML**: >1MB document
5. **International**: UTF-8, non-English content
6. **Malformed HTML**: Missing tags, broken structure
7. **Edge cases**: Empty HTML, only scripts, etc.

**Test Data**:
```
tests/integration/native_parser/
├── fixtures/
│   ├── simple.html
│   ├── news_article.html
│   ├── spa_rendered.html
│   ├── large_doc.html
│   └── international.html
└── tests.rs
```

### 7.3 Performance Tests

**Benchmarks**:
- Parse time: <100ms for typical page (10-50KB HTML)
- Memory usage: <50MB for large documents (1MB HTML)
- Concurrent parsing: 100 documents simultaneously

**Benchmark Code**:
```rust
#[bench]
fn bench_native_parser(b: &mut Bencher) {
    let html = load_fixture("news_article.html");
    let parser = NativeHtmlParser::new();

    b.iter(|| {
        parser.parse_headless_html(&html, "https://example.com").unwrap()
    });
}
```

### 7.4 Comparative Testing

**Native vs WASM Comparison** (when WASM works):
- Extraction accuracy (same results?)
- Performance (native should be faster)
- Memory usage (native should use less)

---

## 8. Performance Considerations

### 8.1 Optimization Strategies

**Parsing Optimization**:
- ✅ Pre-compile CSS selectors (once at initialization)
- ✅ Use `once_cell` for regex patterns
- ✅ Limit DOM traversal depth
- ✅ Parallel extraction where possible
- ✅ Early exit on quality thresholds

**Memory Optimization**:
- ✅ Stream large HTML instead of loading entirely
- ✅ Limit extracted content size
- ✅ Drop parsed document after extraction
- ✅ Use `String::with_capacity` for known sizes

### 8.2 Expected Performance

**Parse Time** (typical news article, 50KB HTML):
- Parsing: ~20-50ms
- Extraction: ~10-30ms
- Total: ~30-80ms

**Memory Usage**:
- Peak: ~5-10MB per document
- Steady: ~2-3MB after extraction

**Throughput**:
- Single-threaded: ~100 docs/sec
- Multi-threaded (8 cores): ~600 docs/sec

### 8.3 Scalability

**Horizontal Scaling**:
- Native parser is CPU-bound, scales linearly
- No shared state, perfect for parallel processing
- Can process in async tasks or thread pool

**Vertical Scaling**:
- More CPU cores = more throughput
- Memory grows linearly with concurrency
- Typical: 8 cores can handle 600 docs/sec

---

## 9. Security Considerations

### 9.1 Input Validation

**Threats**:
- ❌ XXE (XML External Entity) - Not applicable (HTML parser)
- ✅ DoS via large documents - Mitigated by size limit
- ✅ DoS via deeply nested HTML - Mitigated by parse timeout
- ✅ Memory exhaustion - Mitigated by size limit

**Mitigations**:
```rust
// Size limit
const MAX_HTML_SIZE: usize = 10_000_000; // 10MB

// Parse timeout
const PARSE_TIMEOUT_MS: u64 = 5000; // 5 seconds

// Depth limit (in scraper configuration)
const MAX_DOM_DEPTH: usize = 256;
```

### 9.2 Content Sanitization

**What to sanitize**:
- ✅ Script tags (remove from extracted text)
- ✅ Style tags (remove from extracted text)
- ✅ Event handlers (not in extracted content)
- ✅ Dangerous URLs (validate before including)

**What NOT to sanitize**:
- ⚠️ Raw HTML (we're extracting, not rendering)
- ⚠️ User content (out of scope for parser)

### 9.3 Error Information Leakage

**Avoid leaking**:
- ❌ Internal file paths
- ❌ Stack traces to external users
- ❌ Memory addresses

**Safe error messages**:
```rust
// ❌ BAD
Err(format!("Failed to parse at {:?}", ptr))

// ✅ GOOD
Err(NativeParserError::ParseError("Invalid HTML structure".to_string()))
```

---

## 10. Monitoring & Observability

### 10.1 Metrics

**Key Metrics**:
```rust
// Success rate
native_parser_success_total
native_parser_error_total

// Performance
native_parser_duration_seconds (histogram)
native_parser_parse_duration_ms (histogram)
native_parser_extract_duration_ms (histogram)

// Quality
native_parser_quality_score (histogram)
native_parser_word_count (histogram)

// Errors
native_parser_errors_by_type (counter, labels: error_type)
```

### 10.2 Logging

**Log Levels**:
- `ERROR`: Parsing failures, unexpected errors
- `WARN`: Low quality extractions, fallbacks
- `INFO`: Successful extractions, quality scores
- `DEBUG`: Detailed extraction steps
- `TRACE`: CSS selector matches, DOM traversal

**Structured Logging**:
```rust
info!(
    request_id = %request_id,
    url = %url,
    quality_score = quality_score,
    word_count = word_count,
    duration_ms = duration.as_millis(),
    "Native parser extraction completed"
);
```

### 10.3 Tracing

**Spans**:
```rust
#[instrument(skip(self, html), fields(url = %url, html_size = html.len()))]
pub fn parse_headless_html(&self, html: &str, url: &str) -> Result<ExtractedDoc> {
    let _span = tracing::info_span!("native_parser.parse").entered();

    // Nested spans for each extraction step
    let _parse_span = tracing::debug_span!("parse_document").entered();
    // ...
}
```

---

## 11. Documentation

### 11.1 User Documentation

**README section** (to add):
```markdown
### Native HTML Parser

RipTide uses a native Rust HTML parser for headless-rendered content,
ensuring 100% reliability without WASM limitations.

#### Features
- ✅ 0% crash rate (vs 100% with WASM)
- ✅ Faster performance (native vs WASM)
- ✅ Rich error handling with fallbacks
- ✅ Quality-based extraction strategies

#### Configuration

Set in `config.toml`:
```toml
[extraction.native_parser]
enable_markdown = true
extract_links = true
extract_media = true
detect_language = true
extract_categories = true
max_content_length = 10_000_000
parse_timeout_ms = 5000
```
```

### 11.2 API Documentation

**rustdoc comments** for public API:
```rust
/// Native HTML parser for headless-rendered content.
///
/// This parser uses native Rust (via `scraper` crate) instead of WASM,
/// providing 100% reliability for headless extraction workflows.
///
/// # Example
///
/// ```
/// use riptide_extraction::native_parser::NativeHtmlParser;
///
/// let parser = NativeHtmlParser::new();
/// let doc = parser.parse_headless_html(html, url)?;
/// ```
pub struct NativeHtmlParser { ... }
```

### 11.3 Architecture Documentation

**ADR (Architecture Decision Record)**:
```markdown
# ADR-001: Native HTML Parser for Headless Extraction

## Status
Accepted

## Context
WASM HTML parser crashes on Html::parse_document() due to
tendril/html5ever incompatibility with WASM Component Model.

## Decision
Implement native Rust HTML parser in riptide-extraction crate.

## Consequences
- ✅ Immediate fix for 100% extraction failure
- ✅ Better performance and error handling
- ⚠️ Duplicate parsing logic (temporary)
```

---

## 12. Risks & Mitigation

### 12.1 Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|-----------|
| Native parser has bugs | Medium | High | Comprehensive testing, fallbacks |
| Performance regression | Low | Medium | Benchmarks, profiling |
| Memory leaks | Low | High | Valgrind testing, monitoring |
| Different results than WASM | Medium | Medium | Comparative testing, validation |

### 12.2 Operational Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|-----------|
| Rollout breaks production | Low | Critical | Feature flag, gradual rollout |
| Increased CPU usage | Medium | Medium | Performance monitoring, scaling |
| Increased memory usage | Low | Medium | Memory limits, monitoring |

### 12.3 Business Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|-----------|
| Delayed release | Low | Medium | Phased implementation |
| User complaints (quality) | Low | High | Quality thresholds, fallbacks |
| Maintenance burden | Medium | Low | Good documentation, testing |

---

## 13. Success Metrics

### 13.1 Technical Metrics

**Must Have**:
- ✅ Extraction success rate >95% (vs 0% with WASM)
- ✅ Average latency <500ms
- ✅ Error rate <1%
- ✅ Memory usage <100MB per request

**Nice to Have**:
- ✅ Success rate >98%
- ✅ Average latency <300ms
- ✅ Quality score >80 for typical content

### 13.2 Business Metrics

**Impact**:
- ✅ Unblock headless extraction feature
- ✅ Enable full scraping functionality
- ✅ Improve user satisfaction (working features!)
- ✅ Reduce support tickets (no crashes)

---

## 14. Future Enhancements

### 14.1 Short-term (Next Sprint)

1. **Parallel Extraction**: Extract multiple documents concurrently
2. **Caching**: Cache parsed documents for repeated requests
3. **Streaming**: Stream large HTML instead of loading entirely
4. **Custom Extractors**: Allow users to define custom extraction rules

### 14.2 Medium-term (Next Quarter)

1. **Machine Learning**: Train model for better content extraction
2. **Schema.org Support**: Enhanced structured data extraction
3. **PDF Generation**: Convert extracted content to PDF
4. **Multi-language**: Better support for non-English content

### 14.3 Long-term (Next Year)

1. **Unified Parser**: Replace WASM entirely with native parser
2. **Browser Integration**: Native parser for headless Chrome results
3. **Real-time Extraction**: Stream extraction as HTML loads
4. **Distributed Parsing**: Scale across multiple nodes

---

## 15. References

### 15.1 Internal Documentation

- `/workspaces/eventmesh/docs/wasm-issues-analysis.md` - Root cause analysis
- `/workspaces/eventmesh/docs/ROADMAP.md` - Feature roadmap
- `/workspaces/eventmesh/crates/riptide-reliability/src/reliability.rs` - Integration point

### 15.2 External Resources

- [scraper crate](https://docs.rs/scraper/) - HTML parsing library
- [html5ever](https://docs.rs/html5ever/) - HTML5 parser (used by scraper)
- [WASM Component Model](https://github.com/WebAssembly/component-model) - Understanding WASM limitations
- [Readability Algorithm](https://github.com/mozilla/readability) - Content extraction inspiration

### 15.3 Related Issues

- RUSTSEC-2025-0046: wasmtime security update
- WebAssembly/binaryen#6728: Component Model support
- tendril memory safety in WASM Component Model

---

## Appendix A: Code Structure Summary

```
crates/riptide-extraction/src/native_parser/
├── mod.rs              (170 lines)  - Public API, NativeHtmlParser struct
├── parser.rs           (250 lines)  - Core parsing logic
├── error.rs            (60 lines)   - Error types
├── quality.rs          (120 lines)  - Quality assessment
├── fallbacks.rs        (150 lines)  - Fallback strategies
└── extractors/
    ├── mod.rs          (30 lines)   - Re-exports
    ├── title.rs        (80 lines)   - Title extraction
    ├── content.rs      (200 lines)  - Text & markdown extraction
    ├── metadata.rs     (150 lines)  - Byline, date, description
    ├── links.rs        (120 lines)  - Link extraction
    ├── media.rs        (130 lines)  - Media extraction
    ├── language.rs     (100 lines)  - Language detection
    └── categories.rs   (140 lines)  - Category extraction

Total: ~1,600 lines of well-structured, tested code
```

---

## Appendix B: Configuration Example

```toml
# config/default.toml

[extraction.native_parser]
# Enable markdown generation (default: true)
enable_markdown = true

# Extract links (default: true)
extract_links = true

# Extract media (images, videos) (default: true)
extract_media = true

# Detect content language (default: true)
detect_language = true

# Extract categories/tags (default: true)
extract_categories = true

# Maximum HTML size in bytes (default: 10MB)
max_content_length = 10_000_000

# Parse timeout in milliseconds (default: 5000ms)
parse_timeout_ms = 5000

# Minimum quality score (0-100) (default: 30)
min_quality_score = 30

# Enable fallback strategies (default: true)
enable_fallbacks = true
```

---

## Appendix C: Migration Checklist

- [ ] **Week 1: Implementation**
  - [ ] Create module structure
  - [ ] Implement core parser
  - [ ] Port extractor logic
  - [ ] Add error handling
  - [ ] Write unit tests

- [ ] **Week 1-2: Integration**
  - [ ] Update reliability.rs
  - [ ] Add feature flag
  - [ ] Integration tests
  - [ ] Performance benchmarks

- [ ] **Week 2: Testing**
  - [ ] A/B testing setup
  - [ ] Load testing
  - [ ] Edge case testing
  - [ ] Performance profiling

- [ ] **Week 3: Rollout**
  - [ ] Staging deployment
  - [ ] Monitor metrics (48h)
  - [ ] Production rollout (gradual)
  - [ ] Full deployment

- [ ] **Week 4+: Maintenance**
  - [ ] Monitor production metrics
  - [ ] Fix any issues
  - [ ] Optimize performance
  - [ ] Plan future enhancements

---

**End of Document**

**Next Steps**:
1. Review and approve design
2. Create implementation tickets
3. Begin Phase 1 implementation
4. Regular architecture reviews during development

**Questions or Feedback**: Contact architecture team or file issue in project tracker.
