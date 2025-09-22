use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedDoc {
    pub url: String,
    pub title: Option<String>,
    pub byline: Option<String>,
    pub published_iso: Option<String>,
    pub markdown: String,
    pub text: String,
    pub links: Vec<String>,
    pub media: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlOptions {
    pub concurrency: usize,
    pub cache_mode: String, // "enabled" | "bypass" | "read_through"
    pub dynamic_wait_for: Option<String>,
    pub scroll_steps: u32,
    pub token_chunk_max: usize,
    pub token_overlap: usize,
}

impl Default for CrawlOptions {
    fn default() -> Self {
        Self {
            concurrency: 16,
            cache_mode: "read_through".to_string(),
            dynamic_wait_for: None,
            scroll_steps: 8,
            token_chunk_max: 1200,
            token_overlap: 120,
        }
    }
}
