use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// API client for RipTide server operations
pub struct RiptideApiClient {
    base_url: String,
    api_key: Option<String>,
    client: Client,
}

/// Request for rendering a web page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderRequest {
    pub url: String,
    pub wait_condition: String,
    pub screenshot_mode: String,
    pub viewport: ViewportConfig,
    pub stealth_level: String,
    pub javascript_enabled: bool,
    pub extra_timeout: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// Viewport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportConfig {
    pub width: u32,
    pub height: u32,
}

/// Response from render operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dom: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub har: Option<String>,
    pub metadata: RenderMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Metadata from render operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderMetadata {
    pub final_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub render_time_ms: u64,
    pub resources_loaded: u32,
    pub cookies_set: u32,
}

/// Request for screenshot capture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotRequest {
    pub url: String,
    pub viewport: ViewportConfig,
    pub full_page: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_condition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
}

/// Request for content extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractRequest {
    pub url: String,
    pub selectors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wasm_module: Option<String>,
}

/// Result from extraction operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub success: bool,
    pub data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ExtractionMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Metadata from extraction operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionMetadata {
    pub url: String,
    pub extracted_fields: u32,
    pub extraction_time_ms: u64,
}

impl RiptideApiClient {
    /// Create a new API client
    pub fn new(base_url: String, api_key: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .connect_timeout(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(90))
            .http2_prior_knowledge()
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            client,
        })
    }

    /// Check if the API server is available
    pub async fn is_available(&self) -> bool {
        let url = format!("{}/health", self.base_url);

        match self
            .client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Render a web page via API
    pub async fn render(&self, request: RenderRequest) -> Result<RenderResponse> {
        let url = format!("{}/api/v1/render", self.base_url);

        let mut req = self.client.post(&url).json(&request);

        if let Some(api_key) = &self.api_key {
            req = req.header("X-API-Key", api_key);
        }

        let response = req
            .send()
            .await
            .context("Failed to send render request to API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error response".to_string());
            anyhow::bail!(
                "Render API request failed with status {}: {}",
                status,
                error_body
            );
        }

        let render_response = response
            .json::<RenderResponse>()
            .await
            .context("Failed to parse render response")?;

        Ok(render_response)
    }

    /// Capture screenshot via API
    pub async fn screenshot(&self, request: ScreenshotRequest) -> Result<Vec<u8>> {
        let url = format!("{}/api/v1/screenshot", self.base_url);

        let mut req = self.client.post(&url).json(&request);

        if let Some(api_key) = &self.api_key {
            req = req.header("X-API-Key", api_key);
        }

        let response = req
            .send()
            .await
            .context("Failed to send screenshot request to API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error response".to_string());
            anyhow::bail!(
                "Screenshot API request failed with status {}: {}",
                status,
                error_body
            );
        }

        let screenshot_bytes = response
            .bytes()
            .await
            .context("Failed to read screenshot bytes")?
            .to_vec();

        Ok(screenshot_bytes)
    }

    /// Extract content via API
    pub async fn extract(&self, request: ExtractRequest) -> Result<ExtractionResult> {
        let url = format!("{}/api/v1/extract", self.base_url);

        let mut req = self.client.post(&url).json(&request);

        if let Some(api_key) = &self.api_key {
            req = req.header("X-API-Key", api_key);
        }

        let response = req
            .send()
            .await
            .context("Failed to send extraction request to API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error response".to_string());
            anyhow::bail!(
                "Extract API request failed with status {}: {}",
                status,
                error_body
            );
        }

        let extraction_result = response
            .json::<ExtractionResult>()
            .await
            .context("Failed to parse extraction response")?;

        Ok(extraction_result)
    }

    /// Get base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_client_creation() {
        let client = RiptideApiClient::new(
            "http://localhost:8080".to_string(),
            Some("test-key".to_string()),
        );
        assert!(client.is_ok());
    }

    #[test]
    fn test_base_url_normalization() {
        let client = RiptideApiClient::new("http://localhost:8080/".to_string(), None).unwrap();
        assert_eq!(client.base_url(), "http://localhost:8080");
    }
}
