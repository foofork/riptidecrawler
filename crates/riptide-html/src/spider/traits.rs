//! Core traits for DOM-specific spider functionality

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

/// Main trait for DOM-specific spider operations
pub trait DomSpider: Send + Sync {
    /// Extract all links from HTML content
    async fn extract_links(&self, html: &str, base_url: &Url) -> Result<Vec<Url>>;

    /// Extract forms from HTML content
    async fn extract_forms(&self, html: &str, base_url: &Url) -> Result<Vec<FormData>>;

    /// Extract metadata from HTML content
    async fn extract_metadata(&self, html: &str) -> Result<MetaData>;

    /// Extract plain text content from HTML
    async fn extract_text_content(&self, html: &str) -> Result<Option<String>>;

    /// Analyze HTML content for spider optimization hints
    async fn analyze_content(&self, html: &str) -> Result<ContentAnalysis>;
}

/// Result of DOM crawling operations
#[derive(Debug, Clone)]
pub struct DomCrawlerResult {
    /// Successfully extracted links
    pub links: Vec<Url>,
    /// Extracted forms
    pub forms: Vec<FormData>,
    /// Page metadata
    pub metadata: MetaData,
    /// Extracted text content
    pub text_content: Option<String>,
    /// Content analysis for spider optimization
    pub analysis: ContentAnalysis,
}

/// HTML form data extracted from pages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormData {
    /// Form action URL
    pub action: Option<Url>,
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Form encoding type
    pub enctype: Option<String>,
    /// Form input fields
    pub fields: Vec<FormField>,
    /// Form name or id if available
    pub name: Option<String>,
}

/// Individual form field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    /// Field type (text, password, email, etc.)
    pub field_type: String,
    /// Field name attribute
    pub name: String,
    /// Field value if present
    pub value: Option<String>,
    /// Whether field is required
    pub required: bool,
    /// Field placeholder text
    pub placeholder: Option<String>,
}

/// HTML metadata extracted from pages
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetaData {
    /// Page title
    pub title: Option<String>,
    /// Meta description
    pub description: Option<String>,
    /// Meta keywords
    pub keywords: Vec<String>,
    /// Author information
    pub author: Option<String>,
    /// Language information
    pub language: Option<String>,
    /// Canonical URL
    pub canonical: Option<Url>,
    /// Open Graph metadata
    pub og_data: HashMap<String, String>,
    /// Twitter Card metadata
    pub twitter_data: HashMap<String, String>,
    /// Custom meta tags
    pub custom_meta: HashMap<String, String>,
    /// Schema.org structured data
    pub structured_data: Vec<String>,
}

/// Content analysis for spider optimization
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContentAnalysis {
    /// Estimated content type (article, product, navigation, etc.)
    pub content_type: ContentType,
    /// Link density (links per content size)
    pub link_density: f64,
    /// Content quality score (0.0 to 1.0)
    pub quality_score: f64,
    /// Unique text character count
    pub unique_text_chars: usize,
    /// Internal vs external link ratio
    pub internal_link_ratio: f64,
    /// Navigation hints (breadcrumbs, pagination, etc.)
    pub navigation_hints: Vec<NavigationHint>,
    /// Content freshness indicators
    pub freshness_indicators: Vec<String>,
}

/// Content type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    /// Article or blog post
    Article,
    /// Product page
    Product,
    /// Category or listing page
    Category,
    /// Navigation page
    Navigation,
    /// Form page
    Form,
    /// Media page (images, videos)
    Media,
    /// Documentation page
    Documentation,
    /// Unknown content type
    Unknown,
}

impl Default for ContentType {
    fn default() -> Self {
        ContentType::Unknown
    }
}

/// Navigation hints found in content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NavigationHint {
    /// Breadcrumb navigation
    Breadcrumb { path: Vec<String> },
    /// Pagination controls
    Pagination { current: u32, total: Option<u32> },
    /// Site navigation menu
    SiteNavigation { items: Vec<String> },
    /// Related content links
    RelatedContent { count: usize },
    /// Category navigation
    CategoryNav { categories: Vec<String> },
}

/// Configuration for DOM spider operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomSpiderConfig {
    /// Maximum links to extract per page
    pub max_links_per_page: Option<usize>,
    /// Extract external links
    pub extract_external_links: bool,
    /// Extract form data
    pub extract_forms: bool,
    /// Extract metadata
    pub extract_metadata: bool,
    /// Analyze content for optimization
    pub analyze_content: bool,
    /// Link extraction patterns to ignore
    pub ignore_link_patterns: Vec<String>,
    /// Form field types to extract
    pub form_field_types: Vec<String>,
    /// Meta tags to extract
    pub meta_tags_to_extract: Vec<String>,
}

impl Default for DomSpiderConfig {
    fn default() -> Self {
        Self {
            max_links_per_page: Some(1000),
            extract_external_links: true,
            extract_forms: true,
            extract_metadata: true,
            analyze_content: true,
            ignore_link_patterns: vec![
                r"^#".to_string(),           // Fragment links
                r"^javascript:".to_string(), // JavaScript links
                r"^mailto:".to_string(),     // Email links
                r"^tel:".to_string(),        // Phone links
            ],
            form_field_types: vec![
                "text".to_string(),
                "email".to_string(),
                "password".to_string(),
                "hidden".to_string(),
                "submit".to_string(),
                "button".to_string(),
            ],
            meta_tags_to_extract: vec![
                "description".to_string(),
                "keywords".to_string(),
                "author".to_string(),
                "robots".to_string(),
                "viewport".to_string(),
            ],
        }
    }
}