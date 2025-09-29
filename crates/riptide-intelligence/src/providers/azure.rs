//! Azure OpenAI provider implementation

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info};

use crate::{
    LlmProvider, CompletionRequest, CompletionResponse, LlmCapabilities, Cost, ModelInfo,
    IntelligenceError, Result, Message, Role, Usage,
};

// Reuse OpenAI structures since Azure OpenAI uses the same API format
use crate::providers::openai::{
    OpenAIResponse, OpenAIRequest, OpenAIMessageRequest, OpenAIEmbeddingRequest, OpenAIEmbeddingResponse
};

/// Azure OpenAI provider implementation
pub struct AzureOpenAIProvider {
    client: Client,
    api_key: String,
    endpoint: String,
    api_version: String,
    model_costs: HashMap<String, (f64, f64)>, // (prompt_cost_per_1k, completion_cost_per_1k)
}

impl AzureOpenAIProvider {
    /// Create a new Azure OpenAI provider
    pub fn new(api_key: String, endpoint: String, api_version: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| IntelligenceError::Configuration(format!("Failed to create HTTP client: {}", e)))?;

        let mut model_costs = HashMap::new();
        // Azure OpenAI pricing (similar to OpenAI but may vary by region)
        model_costs.insert("gpt-4".to_string(), (0.03, 0.06));
        model_costs.insert("gpt-4-32k".to_string(), (0.06, 0.12));
        model_costs.insert("gpt-4o".to_string(), (0.005, 0.015));
        model_costs.insert("gpt-4o-mini".to_string(), (0.00015, 0.0006));
        model_costs.insert("gpt-35-turbo".to_string(), (0.0015, 0.002));
        model_costs.insert("gpt-35-turbo-16k".to_string(), (0.003, 0.004));

        Ok(Self {
            client,
            api_key,
            endpoint,
            api_version,
            model_costs,
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
        let messages = request.messages.iter()
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

    fn build_completion_url(&self, deployment_name: &str) -> String {
        format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.endpoint.trim_end_matches('/'),
            deployment_name,
            self.api_version
        )
    }

    fn build_embedding_url(&self, deployment_name: &str) -> String {
        format!(
            "{}/openai/deployments/{}/embeddings?api-version={}",
            self.endpoint.trim_end_matches('/'),
            deployment_name,
            self.api_version
        )
    }

    async fn make_request<T>(&self, url: &str, payload: &impl Serialize) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self.client
            .post(url)
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(payload)
            .send()
            .await
            .map_err(|e| IntelligenceError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(IntelligenceError::Provider(format!("Azure OpenAI API error: {}", error_text)));
        }

        let result = response.json::<T>().await
            .map_err(|e| IntelligenceError::Provider(format!("Failed to parse response: {}", e)))?;

        Ok(result)
    }
}

#[async_trait]
impl LlmProvider for AzureOpenAIProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        debug!("Sending completion request to Azure OpenAI for model: {}", request.model);

        let openai_request = self.build_openai_request(&request);
        let url = self.build_completion_url(&request.model);
        let response: OpenAIResponse = self.make_request(&url, &openai_request).await?;

        let choice = response.choices.into_iter().next()
            .ok_or_else(|| IntelligenceError::Provider("No completion choices returned".to_string()))?;

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

        debug!("Azure OpenAI completion successful, tokens used: {}", total_tokens);
        Ok(completion_response)
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        debug!("Generating embeddings with Azure OpenAI");

        let request = OpenAIEmbeddingRequest {
            model: "text-embedding-ada-002".to_string(),
            input: text.to_string(),
        };

        let url = self.build_embedding_url("text-embedding-ada-002");
        let response: OpenAIEmbeddingResponse = self.make_request(&url, &request).await?;

        let embedding = response.data.into_iter().next()
            .ok_or_else(|| IntelligenceError::Provider("No embedding data returned".to_string()))?
            .embedding;

        debug!("Azure OpenAI embedding successful, dimensions: {}", embedding.len());
        Ok(embedding)
    }

    fn capabilities(&self) -> LlmCapabilities {
        let models = vec![
            ModelInfo {
                id: "gpt-4".to_string(),
                name: "GPT-4".to_string(),
                description: "Azure OpenAI GPT-4 model".to_string(),
                max_tokens: 8192,
                supports_functions: true,
                supports_streaming: true,
                cost_per_1k_prompt_tokens: 0.03,
                cost_per_1k_completion_tokens: 0.06,
            },
            ModelInfo {
                id: "gpt-4o".to_string(),
                name: "GPT-4o".to_string(),
                description: "Azure OpenAI GPT-4o model".to_string(),
                max_tokens: 4096,
                supports_functions: true,
                supports_streaming: true,
                cost_per_1k_prompt_tokens: 0.005,
                cost_per_1k_completion_tokens: 0.015,
            },
            ModelInfo {
                id: "gpt-35-turbo".to_string(),
                name: "GPT-3.5 Turbo".to_string(),
                description: "Azure OpenAI GPT-3.5 Turbo model".to_string(),
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
        rate_limits.insert("gpt-35-turbo".to_string(), 60);

        LlmCapabilities {
            provider_name: "Azure OpenAI".to_string(),
            models,
            supports_embeddings: true,
            supports_streaming: true,
            supports_functions: true,
            max_context_length: 32768,
            rate_limits,
        }
    }

    fn estimate_cost(&self, tokens: usize) -> Cost {
        // Default to GPT-4o pricing if model not found
        let (prompt_cost_per_1k, completion_cost_per_1k) = self.model_costs
            .get("gpt-4o")
            .copied()
            .unwrap_or((0.005, 0.015));

        // Assume even split between prompt and completion tokens
        let prompt_tokens = tokens / 2;
        let completion_tokens = tokens - prompt_tokens;

        let prompt_cost = (prompt_tokens as f64 / 1000.0) * prompt_cost_per_1k;
        let completion_cost = (completion_tokens as f64 / 1000.0) * completion_cost_per_1k;

        Cost::new(prompt_cost, completion_cost, "USD")
    }

    async fn health_check(&self) -> Result<()> {
        debug!("Performing Azure OpenAI health check");

        // Use a simple embeddings call for health check since it's less resource-intensive
        let request = OpenAIEmbeddingRequest {
            model: "text-embedding-ada-002".to_string(),
            input: "health check".to_string(),
        };

        let url = self.build_embedding_url("text-embedding-ada-002");
        self.make_request::<OpenAIEmbeddingResponse>(&url, &request).await?;

        info!("Azure OpenAI health check successful");
        Ok(())
    }

    fn name(&self) -> &str {
        "azure_openai"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = AzureOpenAIProvider::new(
            "test-key".to_string(),
            "https://test.openai.azure.com".to_string(),
            "2023-12-01-preview".to_string()
        ).unwrap();
        assert_eq!(provider.name(), "azure_openai");
    }

    #[test]
    fn test_capabilities() {
        let provider = AzureOpenAIProvider::new(
            "test-key".to_string(),
            "https://test.openai.azure.com".to_string(),
            "2023-12-01-preview".to_string()
        ).unwrap();
        let capabilities = provider.capabilities();
        assert_eq!(capabilities.provider_name, "Azure OpenAI");
        assert!(capabilities.supports_embeddings);
        assert!(!capabilities.models.is_empty());
    }

    #[test]
    fn test_url_building() {
        let provider = AzureOpenAIProvider::new(
            "test-key".to_string(),
            "https://test.openai.azure.com/".to_string(),
            "2023-12-01-preview".to_string()
        ).unwrap();

        let completion_url = provider.build_completion_url("gpt-4");
        assert!(completion_url.contains("gpt-4"));
        assert!(completion_url.contains("chat/completions"));
        assert!(completion_url.contains("2023-12-01-preview"));

        let embedding_url = provider.build_embedding_url("text-embedding-ada-002");
        assert!(embedding_url.contains("text-embedding-ada-002"));
        assert!(embedding_url.contains("embeddings"));
    }

    #[test]
    fn test_cost_estimation() {
        let provider = AzureOpenAIProvider::new(
            "test-key".to_string(),
            "https://test.openai.azure.com".to_string(),
            "2023-12-01-preview".to_string()
        ).unwrap();
        let cost = provider.estimate_cost(1000);
        assert!(cost.total_cost > 0.0);
        assert_eq!(cost.currency, "USD");
    }
}