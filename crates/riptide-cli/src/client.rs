use anyhow::{Context, Result};
use reqwest::{Client, ClientBuilder, Method, Response};
use serde::Serialize;
use std::time::Duration;

pub struct RipTideClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl RipTideClient {
    pub fn new(base_url: String, api_key: Option<String>) -> Result<Self> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(300))
            .connect_timeout(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(90))
            .http2_prior_knowledge()
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
        })
    }

    pub async fn get(&self, path: &str) -> Result<Response> {
        self.request_raw(Method::GET, path, None).await
    }

    pub async fn post<T: Serialize>(&self, path: &str, body: &T) -> Result<Response> {
        self.request(Method::POST, path, body).await
    }

    async fn request<T: Serialize>(
        &self,
        method: Method,
        path: &str,
        body: &T,
    ) -> Result<Response> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.request(method, &url);

        // Add API key if provided
        if let Some(api_key) = &self.api_key {
            request = request.header("X-API-Key", api_key);
        }

        // Add body
        request = request.json(body);

        let response = request
            .send()
            .await
            .context(format!("Failed to send request to {}", url))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error response".to_string());
            anyhow::bail!("API request failed with status {}: {}", status, error_body);
        }

        Ok(response)
    }

    async fn request_raw(
        &self,
        method: Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<Response> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.request(method, &url);

        // Add API key if provided
        if let Some(api_key) = &self.api_key {
            request = request.header("X-API-Key", api_key);
        }

        // Add body if provided
        if let Some(body) = &body {
            request = request.json(body);
        }

        let response = request
            .send()
            .await
            .context(format!("Failed to send request to {}", url))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error response".to_string());
            anyhow::bail!("API request failed with status {}: {}", status, error_body);
        }

        Ok(response)
    }

    #[allow(dead_code)]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}
