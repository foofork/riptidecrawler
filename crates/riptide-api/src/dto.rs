// Import HTTP DTOs from riptide-types (Phase 2C.1 - breaking circular dependency)
// Re-export for backward compatibility
pub use riptide_types::{CrawledPage, ResultMode};

use serde::{Deserialize, Serialize};

// API-specific extensions for field filtering
#[allow(dead_code)]
pub trait CrawledPageExt {
    #[allow(dead_code)]
    fn apply_field_filter(&mut self, include: Option<&FieldFilter>, exclude: Option<&FieldFilter>);
}

impl CrawledPageExt for CrawledPage {
    /// Apply field filtering based on include/exclude parameters
    #[allow(dead_code)]
    fn apply_field_filter(&mut self, include: Option<&FieldFilter>, exclude: Option<&FieldFilter>) {
        // If include is specified, only keep those fields
        if let Some(filter) = include {
            if !filter.has_field("title") {
                self.title = None;
            }
            if !filter.has_field("content") {
                self.content = None;
            }
            if !filter.has_field("markdown") {
                self.markdown = None;
            }
            if !filter.has_field("links") {
                self.links.clear();
            }
            if !filter.has_field("final_url") {
                self.final_url = None;
            }
            if !filter.has_field("mime") {
                self.mime = None;
            }
            if !filter.has_field("fetch_time_ms") {
                self.fetch_time_ms = None;
            }
            if !filter.has_field("robots_obeyed") {
                self.robots_obeyed = None;
            }
        }

        // If exclude is specified, remove those fields
        if let Some(filter) = exclude {
            if filter.has_field("title") {
                self.title = None;
            }
            if filter.has_field("content") {
                self.content = None;
            }
            if filter.has_field("markdown") {
                self.markdown = None;
            }
            if filter.has_field("links") {
                self.links.clear();
            }
            if filter.has_field("final_url") {
                self.final_url = None;
            }
            if filter.has_field("mime") {
                self.mime = None;
            }
            if filter.has_field("fetch_time_ms") {
                self.fetch_time_ms = None;
            }
            if filter.has_field("robots_obeyed") {
                self.robots_obeyed = None;
            }
        }
    }
}

/// Field filter for selecting specific fields in responses
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct FieldFilter {
    fields: Vec<String>,
}

impl FieldFilter {
    /// Create a new field filter from comma-separated string
    #[allow(dead_code)]
    pub fn parse(s: &str) -> Self {
        Self {
            fields: s.split(',').map(|f| f.trim().to_string()).collect(),
        }
    }

    /// Check if a field is included in the filter
    pub fn has_field(&self, field: &str) -> bool {
        self.fields.iter().any(|f| f == field)
    }
}

/// Pages result for spider crawl operations (Phase 2)
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct SpiderResultPages {
    /// Total pages crawled
    pub pages_crawled: u64,

    /// Total pages failed
    pub pages_failed: u64,

    /// Crawl duration in seconds
    pub duration_seconds: f64,

    /// Reason for stopping
    pub stop_reason: String,

    /// Domains crawled
    pub domains: Vec<String>,

    /// All crawled pages with full details
    pub pages: Vec<CrawledPage>,

    /// API version for forward compatibility
    #[serde(default = "default_api_version")]
    pub api_version: String,
}

#[allow(dead_code)]
fn default_api_version() -> String {
    "v1".to_string()
}

impl SpiderResultPages {
    /// Apply field filtering to all pages
    #[allow(dead_code)]
    pub fn apply_field_filter(
        &mut self,
        include: Option<&FieldFilter>,
        exclude: Option<&FieldFilter>,
    ) {
        for page in &mut self.pages {
            page.apply_field_filter(include, exclude);
        }
    }

    /// Truncate content in all pages
    #[allow(dead_code)]
    pub fn truncate_content(&mut self, max_content_bytes: usize) {
        for page in &mut self.pages {
            page.truncate_content(max_content_bytes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riptide_types::{SpiderResultStats, SpiderResultUrls};

    #[test]
    fn test_result_mode_default_is_stats() {
        // Backward compatibility: default should be Stats
        assert_eq!(ResultMode::default(), ResultMode::Stats);
    }

    #[test]
    fn test_result_mode_serde() {
        // Test serialization
        assert_eq!(
            serde_json::to_string(&ResultMode::Stats).unwrap(),
            "\"stats\""
        );
        assert_eq!(
            serde_json::to_string(&ResultMode::Urls).unwrap(),
            "\"urls\""
        );
        assert_eq!(
            serde_json::to_string(&ResultMode::Pages).unwrap(),
            "\"pages\""
        );
        assert_eq!(
            serde_json::to_string(&ResultMode::Stream).unwrap(),
            "\"stream\""
        );
        assert_eq!(
            serde_json::to_string(&ResultMode::Store).unwrap(),
            "\"store\""
        );

        // Test deserialization
        assert_eq!(
            serde_json::from_str::<ResultMode>("\"stats\"").unwrap(),
            ResultMode::Stats
        );
        assert_eq!(
            serde_json::from_str::<ResultMode>("\"urls\"").unwrap(),
            ResultMode::Urls
        );
        assert_eq!(
            serde_json::from_str::<ResultMode>("\"pages\"").unwrap(),
            ResultMode::Pages
        );
    }

    #[test]
    fn test_crawled_page_creation() {
        let page = CrawledPage::new("https://example.com".to_string(), 1, 200);
        assert_eq!(page.url, "https://example.com");
        assert_eq!(page.depth, 1);
        assert_eq!(page.status_code, 200);
        assert!(page.title.is_none());
        assert!(page.content.is_none());
    }

    #[test]
    fn test_field_filter() {
        let filter = FieldFilter::parse("title,links,markdown");
        assert!(filter.has_field("title"));
        assert!(filter.has_field("links"));
        assert!(filter.has_field("markdown"));
        assert!(!filter.has_field("content"));
    }

    #[test]
    fn test_field_filter_with_whitespace() {
        let filter = FieldFilter::parse("title, links , markdown");
        assert!(filter.has_field("title"));
        assert!(filter.has_field("links"));
        assert!(filter.has_field("markdown"));
    }

    #[test]
    fn test_crawled_page_field_filtering_include() {
        let mut page = CrawledPage::new("https://example.com".to_string(), 1, 200);
        page.title = Some("Title".to_string());
        page.content = Some("Content".to_string());
        page.markdown = Some("Markdown".to_string());

        let filter = FieldFilter::parse("title");
        page.apply_field_filter(Some(&filter), None);

        assert!(page.title.is_some());
        assert!(page.content.is_none());
        assert!(page.markdown.is_none());
    }

    #[test]
    fn test_crawled_page_field_filtering_exclude() {
        let mut page = CrawledPage::new("https://example.com".to_string(), 1, 200);
        page.title = Some("Title".to_string());
        page.content = Some("Content".to_string());
        page.markdown = Some("Markdown".to_string());

        let filter = FieldFilter::parse("content");
        page.apply_field_filter(None, Some(&filter));

        assert!(page.title.is_some());
        assert!(page.content.is_none());
        assert!(page.markdown.is_some());
    }

    #[test]
    fn test_crawled_page_truncation() {
        let mut page = CrawledPage::new("https://example.com".to_string(), 1, 200);
        page.content = Some("x".repeat(2000));
        page.markdown = Some("y".repeat(2000));

        page.truncate_content(1000);

        assert_eq!(page.content.as_ref().unwrap().len(), 1000);
        assert_eq!(page.markdown.as_ref().unwrap().len(), 1000);
        assert_eq!(page.truncated, Some(true));
    }

    #[test]
    fn test_crawled_page_no_truncation_when_small() {
        let mut page = CrawledPage::new("https://example.com".to_string(), 1, 200);
        page.content = Some("small".to_string());

        page.truncate_content(1000);

        assert_eq!(page.content.as_ref().unwrap(), "small");
        assert!(page.truncated.is_none());
    }

    #[test]
    fn test_spider_result_pages_field_filtering() {
        let mut pages = vec![
            CrawledPage::new("https://example.com".to_string(), 0, 200),
            CrawledPage::new("https://example.com/page1".to_string(), 1, 200),
        ];
        for p in &mut pages {
            p.title = Some("Title".to_string());
            p.content = Some("Content".to_string());
        }

        let mut result = SpiderResultPages {
            pages_crawled: 2,
            pages_failed: 0,
            duration_seconds: 1.5,
            stop_reason: "max_pages".to_string(),
            domains: vec!["example.com".to_string()],
            pages,
            api_version: "v1".to_string(),
        };

        let filter = FieldFilter::parse("title");
        result.apply_field_filter(Some(&filter), None);

        for page in &result.pages {
            assert!(page.title.is_some());
            assert!(page.content.is_none());
        }
    }

    #[test]
    fn test_backward_compatibility_stats_struct() {
        // Ensure SpiderResultStats still exists and works
        let stats = SpiderResultStats {
            pages_crawled: 10,
            pages_failed: 1,
            duration_seconds: 2.5,
            stop_reason: "max_pages".to_string(),
            domains: vec!["example.com".to_string()],
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("pages_crawled"));
    }

    #[test]
    fn test_backward_compatibility_urls_struct() {
        // Ensure SpiderResultUrls still exists and works
        let urls = SpiderResultUrls {
            pages_crawled: 10,
            pages_failed: 1,
            duration_seconds: 2.5,
            stop_reason: "max_pages".to_string(),
            domains: vec!["example.com".to_string()],
            discovered_urls: vec!["https://example.com/page1".to_string()],
        };

        let json = serde_json::to_string(&urls).unwrap();
        assert!(json.contains("discovered_urls"));
    }
}
