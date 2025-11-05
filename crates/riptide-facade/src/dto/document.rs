//! Document DTO for extracted content

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::dto::StructuredData;

/// Public API document type
///
/// This DTO is decoupled from internal extraction models to allow
/// internal implementation changes without breaking the public API.
///
/// # Examples
///
/// ```
/// use riptide_facade::dto::Document;
/// use chrono::Utc;
///
/// let doc = Document {
///     url: "https://example.com".to_string(),
///     title: "Example".to_string(),
///     content: "Content here".to_string(),
///     metadata: serde_json::json!({"author": "John"}),
///     extracted_at: Utc::now(),
///     structured_data: None,
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document {
    /// Source URL of the document
    pub url: String,

    /// Extracted title
    pub title: String,

    /// Main content body
    pub content: String,

    /// Generic metadata for forward compatibility
    pub metadata: serde_json::Value,

    /// Timestamp when extraction occurred
    pub extracted_at: DateTime<Utc>,

    /// Format-specific structured data (events, products, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structured_data: Option<StructuredData>,
}

impl Document {
    /// Create a new document with default values
    pub fn new(url: String, title: String, content: String) -> Self {
        Self {
            url,
            title,
            content,
            metadata: serde_json::json!({}),
            extracted_at: Utc::now(),
            structured_data: None,
        }
    }

    /// Add metadata to the document
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Add structured data to the document
    pub fn with_structured_data(mut self, data: StructuredData) -> Self {
        self.structured_data = Some(data);
        self
    }

    /// Convert document to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Convert document to markdown format
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str(&format!("# {}\n\n", self.title));
        md.push_str(&format!("**Source:** {}\n\n", self.url));
        md.push_str(&format!("**Extracted:** {}\n\n", self.extracted_at));
        md.push_str("## Content\n\n");
        md.push_str(&self.content);
        md.push_str("\n");
        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = Document::new(
            "https://example.com".to_string(),
            "Test Title".to_string(),
            "Test content".to_string(),
        );

        assert_eq!(doc.url, "https://example.com");
        assert_eq!(doc.title, "Test Title");
        assert_eq!(doc.content, "Test content");
    }

    #[test]
    fn test_document_to_json() {
        let doc = Document::new(
            "https://example.com".to_string(),
            "Test".to_string(),
            "Content".to_string(),
        );

        let json = doc.to_json().unwrap();
        assert!(json.contains("example.com"));
        assert!(json.contains("Test"));
    }

    #[test]
    fn test_document_to_markdown() {
        let doc = Document::new(
            "https://example.com".to_string(),
            "Test Title".to_string(),
            "Test content".to_string(),
        );

        let md = doc.to_markdown();
        assert!(md.contains("# Test Title"));
        assert!(md.contains("**Source:** https://example.com"));
        assert!(md.contains("Test content"));
    }
}
