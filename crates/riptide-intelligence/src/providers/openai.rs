//! OpenAI provider implementation (refactored with base utilities)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

use super::base::{AuthHeader, CostCalculator, HealthChecker, HttpClientBuilder, HttpRequestHandler, ModelCost};
use crate::{
    CompletionRequest, CompletionResponse, Cost, IntelligenceError, LlmCapabilities, LlmProvider,
    ModelInfo, Result, Role, Usage,
};

/// OpenAI API response structure
#[derive(Debug, Deserialize)]
pub struct OpenAIResponse {
    pub id: String,
    pub choices: Vec<OpenAIChoice>,
    pub usage: OpenAIUsage,
    pub model: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIChoice {
    pub message: OpenAIMessage,
    pub finish_reason: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIMessage {
    pub content: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// OpenAI API request structure
#[derive(Debug, Serialize)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessageRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct OpenAIMessageRequest {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIEmbeddingResponse {
    pub data: Vec<OpenAIEmbeddingData>,
    pub usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIEmbeddingData {
    pub embedding: Vec<f32>,
}

#[derive(Debug, Serialize)]
pub struct OpenAIEmbeddingRequest {
    pub model: String,
    pub input: String,
}

/// OpenAI provider implementation (refactored)
pub struct OpenAIProvider {
    http_handler: HttpRequestHandler,
    cost_calculator: CostCalculator,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider
    pub fn new(api_key: String, base_url: Option<String>) -> Result<Self> {
        // Use shared HTTP client builder
        let client = HttpClientBuilder::new().build()?;

        let base_url = base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string());

        // Create HTTP request handler with Bearer token auth
        let auth = AuthHeader::Bearer(api_key);
        let http_handler = HttpRequestHandler::new(client, base_url, auth);

        // Initialize cost calculator with OpenAI pricing
        let mut cost_calculator = CostCalculator::new()
            .with_default_model("gpt-4o".to_string());

        // OpenAI pricing as of 2024 (per 1K tokens)
        cost_calculator
            .add_model_cost("gpt-4".to_string(), ModelCost::new(0.03, 0.06))
            .add_model_cost("gpt-4-32k".to_string(), ModelCost::new(0.06, 0.12))
            .add_model_cost("gpt-4o".to_string(), ModelCost::new(0.005, 0.015))
            .add_model_cost("gpt-4o-mini".to_string(), ModelCost::new(0.00015, 0.0006))
            .add_model_cost("gpt-3.5-turbo".to_string(), ModelCost::new(0.0015, 0.002))
            .add_model_cost("gpt-3.5-turbo-16k".to_string(), ModelCost::new(0.003, 0.004));

        Ok(Self {
            http_handler,
            cost_calculator,
        })
    }

    fn convert_role_to_openai(role: &Role) -> String {
        match role {
            Role::System => "system".to_string(),
            Role::User => "user".to_string(),
            Role::Assistant => "assistant".to_string(),
            Role::Function => "function".to_string(),
        }
    }

    fn build_openai_request(&self, request: &CompletionRequest) -> OpenAIRequest {
        let messages = request
            .messages
            .iter()
            .map(|msg| OpenAIMessageRequest {
                role: Self::convert_role_to_openai(&msg.role),
                content: msg.content.clone(),
            })
            .collect();

        OpenAIRequest {
            model: request.model.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            frequency_penalty: request.frequency_penalty,
            presence_penalty: request.presence_penalty,
            stop: request.stop.clone(),
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        debug!(
            "Sending completion request to OpenAI for model: {}",
            request.model
        );

        let openai_request = self.build_openai_request(&request);

        // Use shared HTTP handler
        let response: OpenAIResponse = self
            .http_handler
            .post("chat/completions", &openai_request)
            .await?;

        let choice = response.choices.into_iter().next().ok_or_else(|| {
            IntelligenceError::Provider("No completion choices returned".to_string())
        })?;

        let usage = Usage {
            prompt_tokens: response.usage.prompt_tokens,
            completion_tokens: response.usage.completion_tokens,
            total_tokens: response.usage.total_tokens,
        };

        let total_tokens = usage.total_tokens;
        let completion_response = CompletionResponse {
            id: uuid::Uuid::new_v4(),
            request_id: request.id,
            content: choice.message.content,
            model: response.model,
            usage,
            finish_reason: choice.finish_reason,
            created_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        debug!(
            "OpenAI completion successful, tokens used: {}",
            total_tokens
        );
        Ok(completion_response)
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        debug!("Generating embeddings with OpenAI");

        let request = OpenAIEmbeddingRequest {
            model: "text-embedding-ada-002".to_string(),
            input: text.to_string(),
        };

        let response: OpenAIEmbeddingResponse = self
            .http_handler
            .post("embeddings", &request)
            .await?;

        let embedding = response
            .data
            .into_iter()
            .next()
            .ok_or_else(|| IntelligenceError::Provider("No embedding data returned".to_string()))?
            .embedding;

        debug!(
            "OpenAI embedding successful, dimensions: {}",
            embedding.len()
        );
        Ok(embedding)
    }

    fn capabilities(&self) -> LlmCapabilities {
        let models = vec![
            ModelInfo {
                id: "gpt-4o".to_string(),
                name: "GPT-4o".to_string(),
                description: "Most capable GPT-4 model, optimized for chat and code".to_string(),
                max_tokens: 8192,
                supports_functions: true,
                supports_streaming: true,
                cost_per_1k_prompt_tokens: 0.005,
                cost_per_1k_completion_tokens: 0.015,
            },
            ModelInfo {
                id: "gpt-4o-mini".to_string(),
                name: "GPT-4o Mini".to_string(),
                description: "Affordable and intelligent small model for fast, lightweight tasks"
                    .to_string(),
                max_tokens: 16384,
                supports_functions: true,
                supports_streaming: true,
                cost_per_1k_prompt_tokens: 0.00015,
                cost_per_1k_completion_tokens: 0.0006,
            },
            ModelInfo {
                id: "gpt-3.5-turbo".to_string(),
                name: "GPT-3.5 Turbo".to_string(),
                description: "Fast and capable model for most tasks".to_string(),
                max_tokens: 4096,
                supports_functions: true,
                supports_streaming: true,
                cost_per_1k_prompt_tokens: 0.0015,
                cost_per_1k_completion_tokens: 0.002,
            },
        ];

        let mut rate_limits = HashMap::new();
        rate_limits.insert("gpt-4".to_string(), 40);
        rate_limits.insert("gpt-4o".to_string(), 60);
        rate_limits.insert("gpt-4o-mini".to_string(), 100);
        rate_limits.insert("gpt-3.5-turbo".to_string(), 60);

        LlmCapabilities {
            provider_name: "OpenAI".to_string(),
            models,
            supports_embeddings: true,
            supports_streaming: true,
            supports_functions: true,
            max_context_length: 32768,
            rate_limits,
        }
    }

    fn estimate_cost(&self, tokens: usize) -> Cost {
        // Use shared cost calculator
        self.cost_calculator.estimate_cost(tokens, None)
    }

    async fn health_check(&self) -> Result<()> {
        // Use shared health checker
        let checker = HealthChecker::new("OpenAI", "gpt-3.5-turbo", "chat/completions");

        let test_request = OpenAIRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![OpenAIMessageRequest {
                role: "user".to_string(),
                content: "ping".to_string(),
            }],
            max_tokens: Some(1),
            temperature: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
        };

        checker.check::<OpenAIResponse, _>(&self.http_handler, test_request).await
    }

    fn name(&self) -> &str {
        "openai"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = OpenAIProvider::new(
            "test-key".to_string(),
            Some("https://api.openai.com/v1".to_string()),
        )
        .unwrap();
        assert_eq!(provider.name(), "openai");
    }

    #[test]
    fn test_capabilities() {
        let provider = OpenAIProvider::new("test-key".to_string(), None).unwrap();
        let capabilities = provider.capabilities();
        assert_eq!(capabilities.provider_name, "OpenAI");
        assert!(capabilities.supports_embeddings);
        assert!(!capabilities.models.is_empty());
    }

    #[test]
    fn test_cost_estimation() {
        let provider = OpenAIProvider::new("test-key".to_string(), None).unwrap();
        let cost = provider.estimate_cost(1000);
        assert!(cost.total_cost > 0.0);
        assert_eq!(cost.currency, "USD");
    }
}
