/*!
# Extraction Strategies

Modular extraction strategies for different content types and formats.

## Overview

Riptide supports 8 extraction strategies for maximum flexibility:
- **ICS**: iCalendar parsing for events
- **JsonLd**: JSON-LD structured data extraction
- **CSS**: CSS selector-based extraction
- **Regex**: Regular expression pattern matching
- **Rules**: Rule-based extraction with custom rules
- **LLM**: Large Language Model extraction (OpenAI for v1.0)
- **Browser**: Headless browser extraction
- **WASM**: Custom WebAssembly extractors

## Auto-Selection

The framework can automatically select the best strategy based on content:

```rust
use riptide_schemas::extraction::select_strategy;

let html = r#"<script type="application/ld+json">...</script>"#;
let strategy = select_strategy(html, "text/html");
// Returns ExtractionStrategy::JsonLd
```

## Example

```rust
use riptide_schemas::extraction::ExtractionStrategy;

// Explicit strategy selection
let strategy = ExtractionStrategy::CSS(".event-title".to_string());

// iCalendar parsing
let ics_strategy = ExtractionStrategy::ICS;

// LLM extraction with schema
let llm_strategy = ExtractionStrategy::LLM("openai".to_string());
```
*/

use serde::{Deserialize, Serialize};

/// Extraction strategy for content processing
///
/// Each strategy targets specific content types and formats:
///
/// - **ICS**: Best for iCalendar (.ics) files and VCALENDAR content
/// - **JsonLd**: Best for structured JSON-LD data in HTML
/// - **CSS**: Best for well-structured HTML with semantic classes
/// - **Regex**: Best for pattern-based extraction from text
/// - **Rules**: Best for complex multi-step extraction logic
/// - **LLM**: Best for unstructured content requiring inference
/// - **Browser**: Best for JavaScript-rendered content
/// - **WASM**: Best for custom high-performance extractors
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtractionStrategy {
    /// iCalendar (.ics) parsing strategy
    ///
    /// Extracts events from iCalendar format (RFC 5545).
    /// Handles VEVENT, VTODO, VJOURNAL components.
    ///
    /// # Example Input
    /// ```text
    /// BEGIN:VCALENDAR
    /// VERSION:2.0
    /// BEGIN:VEVENT
    /// DTSTART:20250115T100000Z
    /// SUMMARY:Team Meeting
    /// END:VEVENT
    /// END:VCALENDAR
    /// ```
    ICS,

    /// JSON-LD structured data extraction
    ///
    /// Extracts events from JSON-LD markup embedded in HTML.
    /// Supports schema.org Event types.
    ///
    /// # Example Input
    /// ```html
    /// <script type="application/ld+json">
    /// {
    ///   "@context": "https://schema.org",
    ///   "@type": "Event",
    ///   "name": "Concert",
    ///   "startDate": "2025-01-15T19:00"
    /// }
    /// </script>
    /// ```
    JsonLd,

    /// CSS selector-based extraction
    ///
    /// Uses CSS selectors to extract content from HTML.
    /// Flexible and works with most structured HTML.
    ///
    /// # Example
    /// ```rust
    /// use riptide_schemas::extraction::ExtractionStrategy;
    /// let strategy = ExtractionStrategy::CSS(".event-title".to_string());
    /// ```
    CSS(String),

    /// Regular expression pattern matching
    ///
    /// Extracts content using regex patterns.
    /// Useful for unstructured or semi-structured text.
    ///
    /// # Example
    /// ```rust
    /// use riptide_schemas::extraction::ExtractionStrategy;
    /// let strategy = ExtractionStrategy::Regex(r"\d{4}-\d{2}-\d{2}".to_string());
    /// ```
    Regex(String),

    /// Rule-based extraction with custom rules
    ///
    /// Applies custom extraction rules defined in configuration.
    /// Supports complex multi-step extraction logic.
    ///
    /// # Example
    /// ```rust
    /// use riptide_schemas::extraction::ExtractionStrategy;
    /// let strategy = ExtractionStrategy::Rules("event_rules".to_string());
    /// ```
    Rules(String),

    /// LLM-powered extraction
    ///
    /// Uses Large Language Models for intelligent extraction.
    /// v1.0 supports OpenAI only (Azure, Bedrock deferred to v1.1).
    ///
    /// # Example
    /// ```rust
    /// use riptide_schemas::extraction::ExtractionStrategy;
    /// let strategy = ExtractionStrategy::LLM("openai".to_string());
    /// ```
    LLM(String),

    /// Headless browser extraction
    ///
    /// Renders JavaScript-heavy pages using headless browser.
    /// Useful for SPAs and dynamic content.
    Browser,

    /// Custom WebAssembly extractor
    ///
    /// Runs custom WASM modules for high-performance extraction.
    /// Allows user-defined extraction logic.
    ///
    /// # Example
    /// ```rust
    /// use riptide_schemas::extraction::ExtractionStrategy;
    /// let strategy = ExtractionStrategy::WASM("custom_extractor.wasm".to_string());
    /// ```
    WASM(String),
}

impl std::fmt::Display for ExtractionStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractionStrategy::ICS => write!(f, "ics"),
            ExtractionStrategy::JsonLd => write!(f, "json_ld"),
            ExtractionStrategy::CSS(selector) => write!(f, "css({})", selector),
            ExtractionStrategy::Regex(pattern) => write!(f, "regex({})", pattern),
            ExtractionStrategy::Rules(name) => write!(f, "rules({})", name),
            ExtractionStrategy::LLM(provider) => write!(f, "llm({})", provider),
            ExtractionStrategy::Browser => write!(f, "browser"),
            ExtractionStrategy::WASM(module) => write!(f, "wasm({})", module),
        }
    }
}

/// Auto-select extraction strategy based on content analysis
///
/// Analyzes the content and content type to determine the most appropriate
/// extraction strategy.
///
/// # Selection Logic
///
/// 1. **iCalendar**: If content contains `BEGIN:VCALENDAR`
/// 2. **JSON-LD**: If content contains `application/ld+json`
/// 3. **CSS**: Default fallback for HTML content
///
/// # Arguments
///
/// * `content` - The content to analyze (HTML, text, etc.)
/// * `content_type` - MIME type (e.g., "text/html", "text/calendar")
///
/// # Example
///
/// ```rust
/// use riptide_schemas::extraction::{select_strategy, ExtractionStrategy};
///
/// let html = r#"<script type="application/ld+json">{"@type": "Event"}</script>"#;
/// let strategy = select_strategy(html, "text/html");
/// assert_eq!(strategy, ExtractionStrategy::JsonLd);
/// ```
pub fn select_strategy(content: &str, content_type: &str) -> ExtractionStrategy {
    // Check for iCalendar format
    if content.contains("BEGIN:VCALENDAR") || content_type.contains("text/calendar") {
        return ExtractionStrategy::ICS;
    }

    // Check for JSON-LD structured data
    if content.contains("application/ld+json") || content.contains("@context") {
        return ExtractionStrategy::JsonLd;
    }

    // Check for common event microformats
    if content.contains("class=\"h-event\"") || content.contains("class=\"vevent\"") {
        return ExtractionStrategy::CSS(".h-event, .vevent".to_string());
    }

    // Default to CSS selector for HTML
    if content_type.contains("html") {
        return ExtractionStrategy::CSS(".content, article, main".to_string());
    }

    // Fallback to regex for plain text
    ExtractionStrategy::Regex(r".*".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_display() {
        assert_eq!(ExtractionStrategy::ICS.to_string(), "ics");
        assert_eq!(ExtractionStrategy::JsonLd.to_string(), "json_ld");
        assert_eq!(
            ExtractionStrategy::CSS(".event".to_string()).to_string(),
            "css(.event)"
        );
        assert_eq!(ExtractionStrategy::Browser.to_string(), "browser");
    }

    #[test]
    fn test_select_strategy_icalendar() {
        let content = "BEGIN:VCALENDAR\nVERSION:2.0\nEND:VCALENDAR";
        let strategy = select_strategy(content, "text/calendar");
        assert_eq!(strategy, ExtractionStrategy::ICS);
    }

    #[test]
    fn test_select_strategy_jsonld() {
        let content = r#"<script type="application/ld+json">{"@type": "Event"}</script>"#;
        let strategy = select_strategy(content, "text/html");
        assert_eq!(strategy, ExtractionStrategy::JsonLd);
    }

    #[test]
    fn test_select_strategy_microformat() {
        let content = r#"<div class="h-event">Event Title</div>"#;
        let strategy = select_strategy(content, "text/html");
        assert!(matches!(strategy, ExtractionStrategy::CSS(_)));
    }

    #[test]
    fn test_select_strategy_html_fallback() {
        let content = "<html><body>Regular content</body></html>";
        let strategy = select_strategy(content, "text/html");
        assert!(matches!(strategy, ExtractionStrategy::CSS(_)));
    }

    #[test]
    fn test_strategy_serialization() {
        let strategy = ExtractionStrategy::JsonLd;
        let json = serde_json::to_string(&strategy).unwrap();
        let deserialized: ExtractionStrategy = serde_json::from_str(&json).unwrap();
        assert_eq!(strategy, deserialized);
    }
}
