//! Adapter for riptide-monitoring validation module
//!
//! Provides the HTTP client implementation required by the validation checks.

use crate::client::RipTideClient;
use anyhow::Result;
use riptide_monitoring::validation::HttpClient;

/// Adapter to make RipTideClient compatible with validation checks
#[async_trait::async_trait]
impl HttpClient for RipTideClient {
    async fn get_json(&self, path: &str) -> Result<serde_json::Value> {
        let response = self.get(path).await?;
        Ok(response.json().await?)
    }

    async fn get_health(&self, path: &str) -> Result<()> {
        self.get(path).await?;
        Ok(())
    }
}
