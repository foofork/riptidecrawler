//! Base provider utilities and shared functionality
//!
//! This module provides common functionality shared across all LLM providers,
//! reducing code duplication and ensuring consistent behavior.

use reqwest::Client;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info};

use crate::{Cost, IntelligenceError, Result};

/// Standard HTTP client timeout for all providers
pub const DEFAULT_TIMEOUT_SECS: u64 = 60;

/// HTTP authentication header types
#[derive(Debug, Clone)]
pub enum AuthHeader {
    /// Bearer token authorization (e.g., OpenAI)
    Bearer(String),
    /// API key in custom header (e.g., Anthropic uses "x-api-key")
    ApiKey { header_name: String, key: String },
    /// Multiple custom headers (e.g., Azure with multiple auth headers)
    Custom(Vec<(String, String)>),
}

/// Base HTTP client builder with standard configuration
pub struct HttpClientBuilder {
    timeout: Duration,
    additional_headers: Vec<(String, String)>,
}

impl HttpClientBuilder {
    /// Create a new HTTP client builder with default timeout
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            additional_headers: Vec::new(),
        }
    }

    /// Set custom timeout duration
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Add a custom header to all requests
    pub fn with_header(mut self, name: String, value: String) -> Self {
        self.additional_headers.push((name, value));
        self
    }

    /// Build the HTTP client
    pub fn build(self) -> Result<Client> {
        Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|e| {
                IntelligenceError::Configuration(format!("Failed to create HTTP client: {}", e))
            })
    }
}

impl Default for HttpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared HTTP request handler
pub struct HttpRequestHandler {
    client: Client,
    base_url: String,
    auth: AuthHeader,
}

impl HttpRequestHandler {
    /// Create a new HTTP request handler
    pub fn new(client: Client, base_url: String, auth: AuthHeader) -> Self {
        Self {
            client,
            base_url,
            auth,
        }
    }

    /// Make an HTTP POST request with JSON payload
    pub async fn post<T, P>(&self, endpoint: &str, payload: &P) -> Result<T>
    where
        T: DeserializeOwned,
        P: Serialize,
    {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            endpoint.trim_start_matches('/')
        );

        debug!("Making POST request to: {}", url);

        let mut request = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        // Add authentication headers
        request = match &self.auth {
            AuthHeader::Bearer(token) => {
                request.header("Authorization", format!("Bearer {}", token))
            }
            AuthHeader::ApiKey { header_name, key } => request.header(header_name, key),
            AuthHeader::Custom(headers) => {
                for (name, value) in headers {
                    request = request.header(name, value);
                }
                request
            }
        };

        let response = request
            .json(payload)
            .send()
            .await
            .map_err(|e| IntelligenceError::Network(format!("Request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(IntelligenceError::Provider(format!(
                "API error ({}): {}",
                status, error_text
            )));
        }

        response
            .json::<T>()
            .await
            .map_err(|e| IntelligenceError::Provider(format!("Failed to parse response: {}", e)))
    }

    /// Get a reference to the underlying HTTP client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

/// Shared cost calculator for token-based pricing
pub struct CostCalculator {
    model_costs: HashMap<String, ModelCost>,
    default_model: Option<String>,
}

/// Per-model cost structure
#[derive(Debug, Clone, Copy)]
pub struct ModelCost {
    /// Cost per 1,000 prompt tokens
    pub prompt_cost_per_1k: f64,
    /// Cost per 1,000 completion tokens
    pub completion_cost_per_1k: f64,
}

impl ModelCost {
    /// Create a new model cost entry
    pub fn new(prompt_cost_per_1k: f64, completion_cost_per_1k: f64) -> Self {
        Self {
            prompt_cost_per_1k,
            completion_cost_per_1k,
        }
    }
}

impl CostCalculator {
    /// Create a new cost calculator
    pub fn new() -> Self {
        Self {
            model_costs: HashMap::new(),
            default_model: None,
        }
    }

    /// Add a model's cost information
    pub fn add_model_cost(&mut self, model: String, cost: ModelCost) -> &mut Self {
        self.model_costs.insert(model, cost);
        self
    }

    /// Set the default model to use when costs aren't found
    pub fn with_default_model(mut self, model: String) -> Self {
        self.default_model = Some(model);
        self
    }

    /// Estimate cost based on total tokens (assumes 50/50 prompt/completion split)
    pub fn estimate_cost(&self, tokens: usize, model: Option<&str>) -> Cost {
        let cost_entry = self.get_model_cost(model);

        // Assume even split between prompt and completion tokens
        let prompt_tokens = tokens / 2;
        let completion_tokens = tokens - prompt_tokens;

        let prompt_cost = (prompt_tokens as f64 / 1000.0) * cost_entry.prompt_cost_per_1k;
        let completion_cost =
            (completion_tokens as f64 / 1000.0) * cost_entry.completion_cost_per_1k;

        Cost::new(prompt_cost, completion_cost, "USD")
    }

    /// Estimate cost with specific prompt and completion token counts
    pub fn estimate_cost_detailed(
        &self,
        prompt_tokens: usize,
        completion_tokens: usize,
        model: Option<&str>,
    ) -> Cost {
        let cost_entry = self.get_model_cost(model);

        let prompt_cost = (prompt_tokens as f64 / 1000.0) * cost_entry.prompt_cost_per_1k;
        let completion_cost =
            (completion_tokens as f64 / 1000.0) * cost_entry.completion_cost_per_1k;

        Cost::new(prompt_cost, completion_cost, "USD")
    }

    /// Get cost information for a specific model
    fn get_model_cost(&self, model: Option<&str>) -> ModelCost {
        // Try the specified model first
        if let Some(model_name) = model {
            if let Some(&cost) = self.model_costs.get(model_name) {
                return cost;
            }
        }

        // Try the default model
        if let Some(ref default) = self.default_model {
            if let Some(&cost) = self.model_costs.get(default) {
                return cost;
            }
        }

        // Fallback to a generic cost
        debug!("Model cost not found, using generic estimate");
        ModelCost::new(0.001, 0.002)
    }

    /// Get all model costs
    pub fn model_costs(&self) -> &HashMap<String, ModelCost> {
        &self.model_costs
    }
}

impl Default for CostCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared health check implementation
pub struct HealthChecker<'a> {
    provider_name: &'a str,
    test_endpoint: &'a str,
}

impl<'a> HealthChecker<'a> {
    /// Create a new health checker
    pub fn new(provider_name: &'a str, _test_model: &'a str, test_endpoint: &'a str) -> Self {
        Self {
            provider_name,
            test_endpoint,
        }
    }

    /// Perform health check by making a minimal request
    pub async fn check<T, P>(&self, handler: &HttpRequestHandler, test_request: P) -> Result<()>
    where
        T: DeserializeOwned,
        P: Serialize,
    {
        debug!("Performing {} health check", self.provider_name);

        handler
            .post::<T, P>(self.test_endpoint, &test_request)
            .await?;

        info!("{} health check successful", self.provider_name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_client_builder() {
        let _client = HttpClientBuilder::new()
            .with_timeout(Duration::from_secs(30))
            .build()
            .unwrap();
        // Client built successfully with timeout configuration
        // Note: reqwest::Client doesn't expose timeout() method to retrieve the value
    }

    #[test]
    fn test_cost_calculator() {
        let mut calculator = CostCalculator::new();
        calculator.add_model_cost("test-model".to_string(), ModelCost::new(0.001, 0.002));

        let cost = calculator.estimate_cost(1000, Some("test-model"));
        assert!(cost.total_cost > 0.0);
        assert_eq!(cost.currency, "USD");
    }

    #[test]
    fn test_cost_calculator_detailed() {
        let mut calculator = CostCalculator::new();
        calculator.add_model_cost("test-model".to_string(), ModelCost::new(0.001, 0.002));

        let cost = calculator.estimate_cost_detailed(500, 500, Some("test-model"));
        assert_eq!(cost.prompt_cost, 0.0005);
        assert_eq!(cost.completion_cost, 0.001);
        assert_eq!(cost.total_cost, 0.0015);
    }

    #[test]
    fn test_cost_calculator_fallback() {
        let calculator = CostCalculator::new();
        let cost = calculator.estimate_cost(1000, Some("unknown-model"));
        // Should use generic fallback
        assert!(cost.total_cost > 0.0);
    }

    #[test]
    fn test_auth_header_types() {
        let bearer = AuthHeader::Bearer("test-token".to_string());
        match bearer {
            AuthHeader::Bearer(token) => assert_eq!(token, "test-token"),
            _ => panic!("Wrong auth type"),
        }

        let api_key = AuthHeader::ApiKey {
            header_name: "x-api-key".to_string(),
            key: "test-key".to_string(),
        };
        match api_key {
            AuthHeader::ApiKey { header_name, key } => {
                assert_eq!(header_name, "x-api-key");
                assert_eq!(key, "test-key");
            }
            _ => panic!("Wrong auth type"),
        }
    }
}
