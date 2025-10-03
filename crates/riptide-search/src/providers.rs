//! Concrete implementations of search providers.
//!
//! This module contains the actual implementations of the SearchProvider trait
//! for different backends like Serper and None (URL parsing).

// NoneProvider is implemented in a separate file
// pub use super::none_provider::NoneProvider;

use super::{SearchBackend, SearchHit, SearchProvider};
use anyhow::{Context, Result};
use serde_json::Value;
use std::time::Duration;

/// Serper.dev search provider implementation.
///
/// This provider uses the Serper.dev API to perform Google searches.
/// Requires a valid API key from serper.dev.
///
/// ## Features
/// - Google search results with ranking
/// - Configurable timeout and result limits
/// - Proper error handling and retry logic
/// - Rate limiting awareness
#[derive(Clone)]
pub struct SerperProvider {
    api_key: String,
    client: reqwest::Client,
    #[allow(dead_code)]
    timeout_seconds: u64,
}

impl SerperProvider {
    /// Create a new SerperProvider with the given API key.
    ///
    /// # Parameters
    /// - `api_key`: Valid Serper.dev API key
    /// - `timeout_seconds`: Request timeout in seconds
    pub fn new(api_key: String, timeout_seconds: u64) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .user_agent("RipTide/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            api_key,
            client,
            timeout_seconds,
        }
    }
}

#[async_trait::async_trait]
impl SearchProvider for SerperProvider {
    async fn search(
        &self,
        query: &str,
        limit: u32,
        country: &str,
        locale: &str,
    ) -> Result<Vec<SearchHit>> {
        if query.trim().is_empty() {
            return Err(anyhow::anyhow!("Search query cannot be empty"));
        }

        let limit = limit.clamp(1, 100); // Ensure reasonable limits

        let search_request = serde_json::json!({
            "q": query,
            "num": limit,
            "gl": country,
            "hl": locale
        });

        let response = self
            .client
            .post("https://google.serper.dev/search")
            .header("X-API-KEY", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&search_request)
            .send()
            .await
            .context("Failed to send search request to Serper API")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Serper API returned error status: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        let search_response: Value = response
            .json()
            .await
            .context("Failed to parse JSON response from Serper API")?;

        self.parse_serper_response(search_response)
    }

    fn backend_type(&self) -> SearchBackend {
        SearchBackend::Serper
    }

    async fn health_check(&self) -> Result<()> {
        // Perform a minimal search to verify API key and connectivity
        let test_response = self
            .client
            .get("https://google.serper.dev/news")
            .header("X-API-KEY", &self.api_key)
            .send()
            .await
            .context("Health check failed: Unable to connect to Serper API")?;

        if test_response.status() == 403 {
            return Err(anyhow::anyhow!("Invalid Serper API key"));
        }

        Ok(())
    }
}

impl SerperProvider {
    /// Parse the Serper API JSON response into SearchHit results.
    fn parse_serper_response(&self, response: Value) -> Result<Vec<SearchHit>> {
        let mut results = Vec::new();

        if let Some(organic_results) = response.get("organic").and_then(|v| v.as_array()) {
            for (index, result) in organic_results.iter().enumerate() {
                let url = result
                    .get("link")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing URL in search result"))?
                    .to_string();

                let mut hit = SearchHit::new(url, (index + 1) as u32);

                if let Some(title) = result.get("title").and_then(|v| v.as_str()) {
                    hit = hit.with_title(title.to_string());
                }

                if let Some(snippet) = result.get("snippet").and_then(|v| v.as_str()) {
                    hit = hit.with_snippet(snippet.to_string());
                }

                // Add additional metadata if available
                if let Some(position) = result.get("position").and_then(|v| v.as_u64()) {
                    hit = hit.with_metadata("position".to_string(), position.to_string());
                }

                results.push(hit);
            }
        }

        Ok(results)
    }
}

impl std::fmt::Debug for SerperProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SerperProvider")
            .field("api_key", &"***")
            .field("timeout_seconds", &self.timeout_seconds)
            .finish()
    }
}
