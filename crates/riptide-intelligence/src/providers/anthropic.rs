//! Anthropic provider implementation

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

use crate::{
    LlmProvider, CompletionRequest, CompletionResponse, LlmCapabilities, Cost, ModelInfo,
    IntelligenceError, Result, Role, Usage,
};

/// Anthropic API response structure
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    #[allow(dead_code)]
    id: String,
    content: Vec<AnthropicContent>,
    usage: AnthropicUsage,
    model: String,
    stop_reason: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    text: String,
    #[serde(rename = "type")]
    content_type: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Anthropic API request structure
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessageRequest>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessageRequest {
    role: String,
    content: String,
}

/// Anthropic provider implementation
pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model_costs: HashMap<String, (f64, f64)>, // (prompt_cost_per_1k, completion_cost_per_1k)
}

impl AnthropicProvider {
    /// Create a new Anthropic provider
    pub fn new(api_key: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| IntelligenceError::Configuration(format!("Failed to create HTTP client: {}", e)))?;

        let base_url = "https://api.anthropic.com/v1".to_string();

        let mut model_costs = HashMap::new();
        // Anthropic pricing as of 2024 (per 1K tokens)
        model_costs.insert("claude-3-5-sonnet-20241022".to_string(), (0.003, 0.015));
        model_costs.insert("claude-3-5-haiku-20241022".to_string(), (0.0008, 0.004));
        model_costs.insert("claude-3-opus-20240229".to_string(), (0.015, 0.075));
        model_costs.insert("claude-3-sonnet-20240229".to_string(), (0.003, 0.015));
        model_costs.insert("claude-3-haiku-20240307".to_string(), (0.00025, 0.00125));

        Ok(Self {
            client,
            api_key,
            base_url,
            model_costs,
        })
    }

    fn convert_role_to_anthropic(role: &Role) -> String {
        match role {
            Role::User => "user".to_string(),
            Role::Assistant => "assistant".to_string(),
            Role::System => "user".to_string(), // Anthropic handles system differently
            Role::Function => "user".to_string(),
        }
    }

    fn build_anthropic_request(&self, request: &CompletionRequest) -> AnthropicRequest {
        let mut system_message = None;
        let mut messages = Vec::new();

        for msg in &request.messages {
            match msg.role {
                Role::System => {
                    system_message = Some(msg.content.clone());
                }
                _ => {
                    messages.push(AnthropicMessageRequest {
                        role: Self::convert_role_to_anthropic(&msg.role),
                        content: msg.content.clone(),
                    });
                }
            }
        }

        AnthropicRequest {
            model: request.model.clone(),
            messages,
            max_tokens: request.max_tokens.unwrap_or(4096),
            temperature: request.temperature,
            top_p: request.top_p,
            stop_sequences: request.stop.clone(),
            system: system_message,
        }
    }

    async fn make_request<T>(&self, endpoint: &str, payload: &impl Serialize) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/{}", self.base_url, endpoint);

        let response = self.client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(payload)
            .send()
            .await
            .map_err(|e| IntelligenceError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(IntelligenceError::Provider(format!("Anthropic API error: {}", error_text)));
        }

        let result = response.json::<T>().await
            .map_err(|e| IntelligenceError::Provider(format!("Failed to parse response: {}", e)))?;

        Ok(result)
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        debug!("Sending completion request to Anthropic for model: {}", request.model);

        let anthropic_request = self.build_anthropic_request(&request);
        let response: AnthropicResponse = self.make_request("messages", &anthropic_request).await?;

        let content = response.content.into_iter()
            .filter(|c| c.content_type == "text")
            .map(|c| c.text)
            .collect::<Vec<_>>()
            .join("");

        let usage = Usage {
            prompt_tokens: response.usage.input_tokens,
            completion_tokens: response.usage.output_tokens,
            total_tokens: response.usage.input_tokens + response.usage.output_tokens,
        };

        let total_tokens = usage.total_tokens;
        let completion_response = CompletionResponse {
            id: uuid::Uuid::new_v4(),
            request_id: request.id,
            content,
            model: response.model,
            usage,
            finish_reason: response.stop_reason,
            created_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        debug!("Anthropic completion successful, tokens used: {}", total_tokens);
        Ok(completion_response)
    }

    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        // Anthropic doesn't provide embeddings API directly
        Err(IntelligenceError::Provider("Embeddings not supported by Anthropic".to_string()))
    }

    fn capabilities(&self) -> LlmCapabilities {
        let models = vec![
            ModelInfo {
                id: "claude-3-5-sonnet-20241022".to_string(),
                name: "Claude 3.5 Sonnet".to_string(),
                description: "Most intelligent model, best performance on complex tasks".to_string(),
                max_tokens: 8192,
                supports_functions: true,
                supports_streaming: true,
                cost_per_1k_prompt_tokens: 0.003,
                cost_per_1k_completion_tokens: 0.015,
            },
            ModelInfo {
                id: "claude-3-5-haiku-20241022".to_string(),
                name: "Claude 3.5 Haiku".to_string(),
                description: "Fastest model, best for simple tasks and real-time applications".to_string(),
                max_tokens: 8192,
                supports_functions: true,
                supports_streaming: true,
                cost_per_1k_prompt_tokens: 0.0008,
                cost_per_1k_completion_tokens: 0.004,
            },
            ModelInfo {
                id: "claude-3-opus-20240229".to_string(),
                name: "Claude 3 Opus".to_string(),
                description: "Most powerful model for complex analysis and creative tasks".to_string(),
                max_tokens: 4096,
                supports_functions: true,
                supports_streaming: true,
                cost_per_1k_prompt_tokens: 0.015,
                cost_per_1k_completion_tokens: 0.075,
            },
            ModelInfo {
                id: "claude-3-sonnet-20240229".to_string(),
                name: "Claude 3 Sonnet".to_string(),
                description: "Balanced model for most use cases".to_string(),
                max_tokens: 4096,
                supports_functions: true,
                supports_streaming: true,
                cost_per_1k_prompt_tokens: 0.003,
                cost_per_1k_completion_tokens: 0.015,
            },
            ModelInfo {
                id: "claude-3-haiku-20240307".to_string(),
                name: "Claude 3 Haiku".to_string(),
                description: "Fastest and most affordable model".to_string(),
                max_tokens: 4096,
                supports_functions: true,
                supports_streaming: true,
                cost_per_1k_prompt_tokens: 0.00025,
                cost_per_1k_completion_tokens: 0.00125,
            },
        ];

        let mut rate_limits = HashMap::new();
        rate_limits.insert("claude-3-5-sonnet-20241022".to_string(), 50);
        rate_limits.insert("claude-3-5-haiku-20241022".to_string(), 100);
        rate_limits.insert("claude-3-opus-20240229".to_string(), 20);
        rate_limits.insert("claude-3-sonnet-20240229".to_string(), 50);
        rate_limits.insert("claude-3-haiku-20240307".to_string(), 100);

        LlmCapabilities {
            provider_name: "Anthropic".to_string(),
            models,
            supports_embeddings: false,
            supports_streaming: true,
            supports_functions: true,
            max_context_length: 200000, // Claude models support up to 200k context
            rate_limits,
        }
    }

    fn estimate_cost(&self, tokens: usize) -> Cost {
        // Default to Claude 3.5 Sonnet pricing if model not found
        let (prompt_cost_per_1k, completion_cost_per_1k) = self.model_costs
            .get("claude-3-5-sonnet-20241022")
            .copied()
            .unwrap_or((0.003, 0.015));

        // Assume even split between prompt and completion tokens
        let prompt_tokens = tokens / 2;
        let completion_tokens = tokens - prompt_tokens;

        let prompt_cost = (prompt_tokens as f64 / 1000.0) * prompt_cost_per_1k;
        let completion_cost = (completion_tokens as f64 / 1000.0) * completion_cost_per_1k;

        Cost::new(prompt_cost, completion_cost, "USD")
    }

    async fn health_check(&self) -> Result<()> {
        debug!("Performing Anthropic health check");

        let test_request = AnthropicRequest {
            model: "claude-3-haiku-20240307".to_string(),
            messages: vec![AnthropicMessageRequest {
                role: "user".to_string(),
                content: "ping".to_string(),
            }],
            max_tokens: 1,
            temperature: None,
            top_p: None,
            stop_sequences: None,
            system: None,
        };

        self.make_request::<AnthropicResponse>("messages", &test_request).await?;
        info!("Anthropic health check successful");
        Ok(())
    }

    fn name(&self) -> &str {
        "anthropic"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = AnthropicProvider::new("test-key".to_string()).unwrap();
        assert_eq!(provider.name(), "anthropic");
    }

    #[test]
    fn test_capabilities() {
        let provider = AnthropicProvider::new("test-key".to_string()).unwrap();
        let capabilities = provider.capabilities();
        assert_eq!(capabilities.provider_name, "Anthropic");
        assert!(!capabilities.supports_embeddings);
        assert!(!capabilities.models.is_empty());
    }

    #[test]
    fn test_cost_estimation() {
        let provider = AnthropicProvider::new("test-key".to_string()).unwrap();
        let cost = provider.estimate_cost(1000);
        assert!(cost.total_cost > 0.0);
        assert_eq!(cost.currency, "USD");
    }
}