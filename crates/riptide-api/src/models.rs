use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CrawlBody {
    pub urls: Vec<String>,
}

#[derive(Serialize)]
pub struct CrawlResult {
    pub url: String,
    pub status: u16,
    pub from_cache: bool,
    pub markdown_path: Option<String>,
    pub json_path: Option<String>,
}

#[derive(Deserialize)]
pub struct DeepSearchBody {
    pub query: String,
    pub limit: Option<u32>,
    pub country: Option<String>,
    pub locale: Option<String>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}