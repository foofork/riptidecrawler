//! Core type definitions for Riptide
//!
//! This module contains the primary data structures used throughout
//! the Riptide framework for web scraping and content extraction.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Re-export url::Url for convenience
pub use url::Url;

/// Configuration for browser instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    /// Whether to run browser in headless mode
    pub headless: bool,
    /// User agent string
    pub user_agent: Option<String>,
    /// Viewport width
    pub viewport_width: u32,
    /// Viewport height
    pub viewport_height: u32,
    /// Hard timeout for browser operations in milliseconds (default: 3000ms/3s)
    /// If operation exceeds timeout, falls back to static content
    pub timeout_ms: u64,
    /// Additional browser arguments
    pub args: Vec<String>,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            headless: true,
            user_agent: None,
            viewport_width: 1920,
            viewport_height: 1080,
            timeout_ms: 3000, // Hard timeout: 3 seconds
            args: Vec::new(),
        }
    }
}

/// Options for web scraping operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapingOptions {
    /// Whether to follow redirects
    pub follow_redirects: bool,
    /// Maximum number of redirects to follow
    pub max_redirects: u32,
    /// Whether to execute JavaScript
    pub execute_javascript: bool,
    /// Wait time after page load (milliseconds)
    pub wait_after_load_ms: u64,
    /// Custom headers to include
    pub headers: HashMap<String, String>,
    /// Whether to capture screenshots
    pub capture_screenshot: bool,
}

impl Default for ScrapingOptions {
    fn default() -> Self {
        Self {
            follow_redirects: true,
            max_redirects: 5,
            execute_javascript: true,
            wait_after_load_ms: 1000,
            headers: HashMap::new(),
            capture_screenshot: false,
        }
    }
}

/// Configuration for content extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionConfig {
    /// CSS selectors to extract
    pub selectors: Vec<String>,
    /// Whether to extract metadata
    pub extract_metadata: bool,
    /// Whether to extract links
    pub extract_links: bool,
    /// Whether to extract images
    pub extract_images: bool,
    /// Custom extraction rules
    pub custom_rules: HashMap<String, String>,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            selectors: Vec::new(),
            extract_metadata: true,
            extract_links: true,
            extract_images: true,
            custom_rules: HashMap::new(),
        }
    }
}

/// Request for content extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionRequest {
    /// Unique request identifier
    pub id: Uuid,
    /// URL to extract content from
    pub url: Url,
    /// Scraping options
    pub scraping_options: ScrapingOptions,
    /// Extraction configuration
    pub extraction_config: ExtractionConfig,
    /// Request timestamp
    pub created_at: DateTime<Utc>,
}

impl ExtractionRequest {
    /// Create a new extraction request
    pub fn new(url: Url) -> Self {
        Self {
            id: Uuid::new_v4(),
            url,
            scraping_options: ScrapingOptions::default(),
            extraction_config: ExtractionConfig::default(),
            created_at: Utc::now(),
        }
    }
}

/// Scraped content from a web page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedContent {
    /// HTML content
    pub html: String,
    /// Plain text content
    pub text: String,
    /// Page title
    pub title: Option<String>,
    /// Meta description
    pub description: Option<String>,
    /// Extracted links
    pub links: Vec<Url>,
    /// Extracted images
    pub images: Vec<Url>,
    /// Custom extracted data
    pub custom_data: HashMap<String, serde_json::Value>,
    /// Screenshot data (base64 encoded)
    pub screenshot: Option<String>,
}

/// Result of an extraction operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    /// Original request ID
    pub request_id: Uuid,
    /// URL that was extracted
    pub url: Url,
    /// Extracted content
    pub content: ScrapedContent,
    /// Extraction duration in milliseconds
    pub duration_ms: u64,
    /// Result timestamp
    pub completed_at: DateTime<Utc>,
    /// Whether extraction was successful
    pub success: bool,
    /// Error message if extraction failed
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_config_default() {
        let config = BrowserConfig::default();
        assert!(config.headless);
        assert_eq!(config.viewport_width, 1920);
        assert_eq!(config.viewport_height, 1080);
    }

    #[test]
    fn test_extraction_request_new() {
        let url = Url::parse("https://example.com").unwrap();
        let request = ExtractionRequest::new(url.clone());
        assert_eq!(request.url, url);
        assert!(request.scraping_options.follow_redirects);
    }

    #[test]
    fn test_scraping_options_default() {
        let options = ScrapingOptions::default();
        assert!(options.follow_redirects);
        assert_eq!(options.max_redirects, 5);
        assert!(options.execute_javascript);
    }
}
