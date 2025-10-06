//! Anthropic provider implementation (refactored with base utilities)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

use super::base::{AuthHeader, CostCalculator, HealthChecker, HttpClientBuilder, HttpRequestHandler, ModelCost};
use crate::{
    CompletionRequest, CompletionResponse, Cost, IntelligenceError, LlmCapabilities, LlmProvider,
    ModelInfo, Result, Role, Usage,
};

/// Anthropic API response structure
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
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

/// Anthropic provider implementation (refactored)
pub struct AnthropicProvider {
    http_handler: HttpRequestHandler,
    cost_calculator: CostCalculator,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider
    pub fn new(api_key: String) -> Result<Self> {
        // Use shared HTTP client builder
        let client = HttpClientBuilder::new().build()?;

        // Create HTTP request handler with Anthropic-specific auth
        let auth_headers = vec![
            ("x-api-key".to_string(), api_key),
            ("anthropic-version".to_string(), "2023-06-01".to_string()),
        ];
        let auth = AuthHeader::Custom(auth_headers);

        let http_handler = HttpRequestHandler::new(
            client,
            "https://api.anthropic.com/v1".to_string(),
            auth,
        );

        // Initialize cost calculator with Anthropic pricing
        let mut cost_calculator = CostCalculator::new()
            .with_default_model("claude-3-5-sonnet-20241022".to_string());

        // Anthropic pricing as of 2024 (per 1K tokens)
        cost_calculator
            .add_model_cost("claude-3-5-sonnet-20241022".to_string(), ModelCost::new(0.003, 0.015))
            .add_model_cost("claude-3-5-haiku-20241022".to_string(), ModelCost::new(0.0008, 0.004))
            .add_model_cost("claude-3-opus-20240229".to_string(), ModelCost::new(0.015, 0.075))
            .add_model_cost("claude-3-sonnet-20240229".to_string(), ModelCost::new(0.003, 0.015))
            .add_model_cost("claude-3-haiku-20240307".to_string(), ModelCost::new(0.00025, 0.00125));

        Ok(Self {
            http_handler,
            cost_calculator,
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
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        debug!(
            "Sending completion request to Anthropic for model: {}",
            request.model
        );

        let anthropic_request = self.build_anthropic_request(&request);

        // Use shared HTTP handler
        let response: AnthropicResponse = self
            .http_handler
            .post("messages", &anthropic_request)
            .await?;

        let content = response
            .content
            .into_iter()
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

        debug!(
            "Anthropic completion successful, tokens used: {}",
            total_tokens
        );
        Ok(completion_response)
    }

    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        // Anthropic doesn't provide embeddings API directly
        Err(IntelligenceError::Provider(
            "Embeddings not supported by Anthropic".to_string(),
        ))
    }

    fn capabilities(&self) -> LlmCapabilities {
        let models = vec![
            ModelInfo {
                id: "claude-3-5-sonnet-20241022".to_string(),
                name: "Claude 3.5 Sonnet".to_string(),
                description: "Most intelligent model, best performance on complex tasks"
                    .to_string(),
                max_tokens: 8192,
                supports_functions: true,
                supports_streaming: true,
                cost_per_1k_prompt_tokens: 0.003,
                cost_per_1k_completion_tokens: 0.015,
            },
            ModelInfo {
                id: "claude-3-5-haiku-20241022".to_string(),
                name: "Claude 3.5 Haiku".to_string(),
                description: "Fastest model, best for simple tasks and real-time applications"
                    .to_string(),
                max_tokens: 8192,
                supports_functions: true,
                supports_streaming: true,
                cost_per_1k_prompt_tokens: 0.0008,
                cost_per_1k_completion_tokens: 0.004,
            },
            ModelInfo {
                id: "claude-3-opus-20240229".to_string(),
                name: "Claude 3 Opus".to_string(),
                description: "Most powerful model for complex analysis and creative tasks"
                    .to_string(),
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
        // Use shared cost calculator
        self.cost_calculator.estimate_cost(tokens, None)
    }

    async fn health_check(&self) -> Result<()> {
        // Use shared health checker
        let checker = HealthChecker::new("Anthropic", "claude-3-haiku-20240307", "messages");

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

        checker.check::<AnthropicResponse, _>(&self.http_handler, test_request).await
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
