//! Extraction method types for facade-level extraction strategies
//!
//! This module defines the implementation-level extraction methods used by
//! the facade layer, separate from the high-level domain strategies in
//! `riptide-schemas`.

use serde::{Deserialize, Serialize};

/// Extraction method for facade-level content processing
///
/// Defines the concrete extraction approach to use, distinct from the
/// high-level `ExtractionStrategy` in riptide-schemas which defines
/// domain-level strategies (ICS, JsonLd, LLM, etc).
///
/// These methods represent the actual implementation techniques:
/// - **HtmlCss**: Use CSS selectors for HTML content
/// - **HtmlRegex**: Use regex patterns for HTML content
/// - **Wasm**: Use WebAssembly-based extraction
/// - **Fallback**: Basic text extraction fallback
/// - **PdfText**: Extract text from PDF documents
/// - **Schema**: Schema-guided extraction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtractionMethod {
    /// HTML extraction with CSS selectors (native parser)
    ///
    /// Uses the native HTML parser with CSS selector-based content extraction.
    /// Fast and efficient for well-structured HTML.
    HtmlCss,

    /// HTML extraction with regex patterns
    ///
    /// Uses regular expression patterns to extract content.
    /// Useful for semi-structured content or specific patterns.
    HtmlRegex,

    /// WASM-based extraction
    ///
    /// High-quality extraction using WebAssembly modules.
    /// Provides advanced content extraction with better quality scoring.
    Wasm,

    /// Fallback extraction
    ///
    /// Basic text extraction when other methods fail or aren't applicable.
    /// Provides minimal but reliable content extraction.
    Fallback,

    /// PDF text extraction
    ///
    /// Extracts text content from PDF documents.
    PdfText,

    /// Schema-based extraction
    ///
    /// Uses predefined schemas to extract structured data.
    Schema,
}

impl ExtractionMethod {
    /// Get method name as string
    pub fn name(&self) -> &'static str {
        match self {
            Self::HtmlCss => "html_css",
            Self::HtmlRegex => "html_regex",
            Self::Wasm => "wasm",
            Self::Fallback => "fallback",
            Self::PdfText => "pdf_text",
            Self::Schema => "schema",
        }
    }

    /// Check if this is a WASM-based method
    pub fn is_wasm(&self) -> bool {
        matches!(self, Self::Wasm)
    }

    /// Check if this is an HTML extraction method
    pub fn is_html(&self) -> bool {
        matches!(self, Self::HtmlCss | Self::HtmlRegex | Self::Fallback)
    }

    /// Check if this is a PDF extraction method
    pub fn is_pdf(&self) -> bool {
        matches!(self, Self::PdfText)
    }
}

impl std::fmt::Display for ExtractionMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_name() {
        assert_eq!(ExtractionMethod::HtmlCss.name(), "html_css");
        assert_eq!(ExtractionMethod::Wasm.name(), "wasm");
        assert_eq!(ExtractionMethod::Fallback.name(), "fallback");
    }

    #[test]
    fn test_method_predicates() {
        assert!(ExtractionMethod::HtmlCss.is_html());
        assert!(ExtractionMethod::Wasm.is_wasm());
        assert!(ExtractionMethod::PdfText.is_pdf());
        assert!(!ExtractionMethod::Wasm.is_html());
    }

    #[test]
    fn test_display() {
        assert_eq!(ExtractionMethod::HtmlCss.to_string(), "html_css");
        assert_eq!(ExtractionMethod::Wasm.to_string(), "wasm");
    }
}
