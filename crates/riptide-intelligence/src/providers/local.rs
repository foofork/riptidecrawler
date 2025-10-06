//! Local provider implementations (Ollama, LocalAI)

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

use crate::{
    CompletionRequest, CompletionResponse, Cost, IntelligenceError, LlmCapabilities, LlmProvider,
    ModelInfo, Result, Role, Usage,
};

/// Ollama API response structure
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: OllamaMessage,
    done: bool,
    #[serde(default)]
    eval_count: Option<u32>,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct OllamaMessage {
    #[allow(dead_code)]
    role: String,
    content: String,
}

/// Ollama API request structure
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessageRequest>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
struct OllamaMessageRequest {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct OllamaModelsResponse {
    models: Vec<OllamaModelInfo>,
}

#[derive(Debug, Deserialize)]
struct OllamaModelInfo {
    name: String,
    #[allow(dead_code)] // API response field, used for debugging
    size: u64,
    #[allow(dead_code)] // API response field, used for debugging
    digest: String,
    #[allow(dead_code)] // API response field, used for debugging
    details: OllamaModelDetails,
}

#[derive(Debug, Deserialize)]
struct OllamaModelDetails {
    #[allow(dead_code)] // API response field, used for debugging
    format: String,
    #[allow(dead_code)] // API response field, used for debugging
    family: String,
    #[allow(dead_code)] // API response field, used for debugging
    parameter_size: String,
}

/// Ollama provider implementation
pub struct OllamaProvider {
    client: Client,
    base_url: String,
    available_models: Vec<String>,
}

impl OllamaProvider {
    /// Create a new Ollama provider
    pub fn new(base_url: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120)) // Longer timeout for local models
            .build()
            .map_err(|e| {
                IntelligenceError::Configuration(format!("Failed to create HTTP client: {}", e))
            })?;

        Ok(Self {
            client,
            base_url,
            available_models: Vec::new(),
        })
    }

    /// Fetch available models from Ollama server
    ///
    /// This method queries the Ollama API to discover which models are currently
    /// installed and available for use. The discovered models are stored in
    /// `available_models` and can be used for model validation or dynamic
    /// model selection.
    ///
    /// # Example
    /// ```no_run
    /// # use riptide_intelligence::providers::OllamaProvider;
    /// # async fn example() -> anyhow::Result<()> {
    /// let mut provider = OllamaProvider::new("http://localhost:11434".to_string())?;
    /// provider.fetch_available_models().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_available_models(&mut self) -> Result<()> {
        let url = format!("{}/api/tags", self.base_url);

        let response =
            self.client.get(&url).send().await.map_err(|e| {
                IntelligenceError::Network(format!("Failed to fetch models: {}", e))
            })?;

        if response.status().is_success() {
            let models_response: OllamaModelsResponse = response.json().await.map_err(|e| {
                IntelligenceError::Provider(format!("Failed to parse models response: {}", e))
            })?;

            self.available_models = models_response
                .models
                .into_iter()
                .map(|model| model.name)
                .collect();

            debug!("Discovered {} available Ollama models", self.available_models.len());
        }

        Ok(())
    }

    /// Get list of available models
    ///
    /// Returns the list of models discovered by `fetch_available_models()`.
    /// If models haven't been fetched yet, returns an empty vector.
    pub fn available_models(&self) -> &[String] {
        &self.available_models
    }

    fn convert_role_to_ollama(role: &Role) -> String {
        match role {
            Role::System => "system".to_string(),
            Role::User => "user".to_string(),
            Role::Assistant => "assistant".to_string(),
            Role::Function => "user".to_string(), // Ollama doesn't have function role
        }
    }

    fn build_ollama_request(&self, request: &CompletionRequest) -> OllamaRequest {
        let messages = request
            .messages
            .iter()
            .map(|msg| OllamaMessageRequest {
                role: Self::convert_role_to_ollama(&msg.role),
                content: msg.content.clone(),
            })
            .collect();

        let options = if request.temperature.is_some()
            || request.top_p.is_some()
            || request.max_tokens.is_some()
            || request.stop.is_some()
        {
            Some(OllamaOptions {
                temperature: request.temperature,
                top_p: request.top_p,
                num_predict: request.max_tokens,
                stop: request.stop.clone(),
            })
        } else {
            None
        };

        OllamaRequest {
            model: request.model.clone(),
            messages,
            stream: false,
            options,
        }
    }

    async fn make_request<T>(&self, endpoint: &str, payload: &impl Serialize) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/{}", self.base_url, endpoint);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(payload)
            .send()
            .await
            .map_err(|e| IntelligenceError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(IntelligenceError::Provider(format!(
                "Ollama API error: {}",
                error_text
            )));
        }

        let result = response
            .json::<T>()
            .await
            .map_err(|e| IntelligenceError::Provider(format!("Failed to parse response: {}", e)))?;

        Ok(result)
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        debug!(
            "Sending completion request to Ollama for model: {}",
            request.model
        );

        let ollama_request = self.build_ollama_request(&request);
        let response: OllamaResponse = self.make_request("api/chat", &ollama_request).await?;

        let usage = Usage {
            prompt_tokens: response.prompt_eval_count.unwrap_or(0),
            completion_tokens: response.eval_count.unwrap_or(0),
            total_tokens: response.prompt_eval_count.unwrap_or(0)
                + response.eval_count.unwrap_or(0),
        };

        let total_tokens = usage.total_tokens;
        let completion_response = CompletionResponse {
            id: uuid::Uuid::new_v4(),
            request_id: request.id,
            content: response.message.content,
            model: request.model,
            usage,
            finish_reason: if response.done {
                "stop".to_string()
            } else {
                "length".to_string()
            },
            created_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        debug!(
            "Ollama completion successful, tokens used: {}",
            total_tokens
        );
        Ok(completion_response)
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        debug!("Generating embeddings with Ollama");

        #[derive(Serialize)]
        struct EmbedRequest {
            model: String,
            prompt: String,
        }

        #[derive(Deserialize)]
        struct EmbedResponse {
            embedding: Vec<f32>,
        }

        let request = EmbedRequest {
            model: "nomic-embed-text".to_string(), // Default embedding model
            prompt: text.to_string(),
        };

        let response: EmbedResponse = self.make_request("api/embeddings", &request).await?;

        debug!(
            "Ollama embedding successful, dimensions: {}",
            response.embedding.len()
        );
        Ok(response.embedding)
    }

    fn capabilities(&self) -> LlmCapabilities {
        let models = vec![
            ModelInfo {
                id: "llama3.2".to_string(),
                name: "Llama 3.2".to_string(),
                description: "Meta's latest language model".to_string(),
                max_tokens: 8192,
                supports_functions: false,
                supports_streaming: true,
                cost_per_1k_prompt_tokens: 0.0, // Free for local usage
                cost_per_1k_completion_tokens: 0.0,
            },
            ModelInfo {
                id: "llama3.1".to_string(),
                name: "Llama 3.1".to_string(),
                description: "High-performance language model".to_string(),
                max_tokens: 8192,
                supports_functions: false,
                supports_streaming: true,
                cost_per_1k_prompt_tokens: 0.0,
                cost_per_1k_completion_tokens: 0.0,
            },
            ModelInfo {
                id: "codellama".to_string(),
                name: "Code Llama".to_string(),
                description: "Specialized model for code generation".to_string(),
                max_tokens: 8192,
                supports_functions: false,
                supports_streaming: true,
                cost_per_1k_prompt_tokens: 0.0,
                cost_per_1k_completion_tokens: 0.0,
            },
        ];

        let rate_limits = HashMap::new(); // No rate limits for local models

        LlmCapabilities {
            provider_name: "Ollama".to_string(),
            models,
            supports_embeddings: true,
            supports_streaming: true,
            supports_functions: false,
            max_context_length: 8192,
            rate_limits,
        }
    }

    fn estimate_cost(&self, _tokens: usize) -> Cost {
        // Local models are free
        Cost::zero("USD")
    }

    async fn health_check(&self) -> Result<()> {
        debug!("Performing Ollama health check");

        let url = format!("{}/api/tags", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| IntelligenceError::Network(format!("Health check failed: {}", e)))?;

        if response.status().is_success() {
            info!("Ollama health check successful");
            Ok(())
        } else {
            Err(IntelligenceError::Provider(
                "Ollama server not responding".to_string(),
            ))
        }
    }

    fn name(&self) -> &str {
        "ollama"
    }
}

/// LocalAI provider implementation (similar to OpenAI API)
pub struct LocalAIProvider {
    client: Client,
    base_url: String,
}

impl LocalAIProvider {
    /// Create a new LocalAI provider
    pub fn new(base_url: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| {
                IntelligenceError::Configuration(format!("Failed to create HTTP client: {}", e))
            })?;

        Ok(Self { client, base_url })
    }

    fn convert_role_to_openai(role: &Role) -> String {
        match role {
            Role::System => "system".to_string(),
            Role::User => "user".to_string(),
            Role::Assistant => "assistant".to_string(),
            Role::Function => "function".to_string(),
        }
    }
}

#[async_trait]
impl LlmProvider for LocalAIProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        debug!(
            "Sending completion request to LocalAI for model: {}",
            request.model
        );

        // LocalAI uses OpenAI-compatible API
        use crate::providers::openai::{OpenAIMessageRequest, OpenAIRequest, OpenAIResponse};

        let messages = request
            .messages
            .iter()
            .map(|msg| OpenAIMessageRequest {
                role: Self::convert_role_to_openai(&msg.role),
                content: msg.content.clone(),
            })
            .collect();

        let localai_request = OpenAIRequest {
            model: request.model.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            frequency_penalty: request.frequency_penalty,
            presence_penalty: request.presence_penalty,
            stop: request.stop.clone(),
        };

        let url = format!("{}/v1/chat/completions", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&localai_request)
            .send()
            .await
            .map_err(|e| IntelligenceError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(IntelligenceError::Provider(format!(
                "LocalAI API error: {}",
                error_text
            )));
        }

        let api_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| IntelligenceError::Provider(format!("Failed to parse response: {}", e)))?;

        let choice = api_response.choices.into_iter().next().ok_or_else(|| {
            IntelligenceError::Provider("No completion choices returned".to_string())
        })?;

        let usage = Usage {
            prompt_tokens: api_response.usage.prompt_tokens,
            completion_tokens: api_response.usage.completion_tokens,
            total_tokens: api_response.usage.total_tokens,
        };

        let total_tokens = usage.total_tokens;
        let completion_response = CompletionResponse {
            id: uuid::Uuid::new_v4(),
            request_id: request.id,
            content: choice.message.content,
            model: api_response.model,
            usage,
            finish_reason: choice.finish_reason,
            created_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        debug!(
            "LocalAI completion successful, tokens used: {}",
            total_tokens
        );
        Ok(completion_response)
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        debug!("Generating embeddings with LocalAI");

        use crate::providers::openai::{OpenAIEmbeddingRequest, OpenAIEmbeddingResponse};

        let request = OpenAIEmbeddingRequest {
            model: "text-embedding-ada-002".to_string(),
            input: text.to_string(),
        };

        let url = format!("{}/v1/embeddings", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| IntelligenceError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(IntelligenceError::Provider(format!(
                "LocalAI API error: {}",
                error_text
            )));
        }

        let api_response: OpenAIEmbeddingResponse = response
            .json()
            .await
            .map_err(|e| IntelligenceError::Provider(format!("Failed to parse response: {}", e)))?;

        let embedding = api_response
            .data
            .into_iter()
            .next()
            .ok_or_else(|| IntelligenceError::Provider("No embedding data returned".to_string()))?
            .embedding;

        debug!(
            "LocalAI embedding successful, dimensions: {}",
            embedding.len()
        );
        Ok(embedding)
    }

    fn capabilities(&self) -> LlmCapabilities {
        let models = vec![ModelInfo {
            id: "gpt-3.5-turbo".to_string(),
            name: "Local GPT-3.5 Turbo".to_string(),
            description: "Local implementation of GPT-3.5 Turbo".to_string(),
            max_tokens: 4096,
            supports_functions: true,
            supports_streaming: true,
            cost_per_1k_prompt_tokens: 0.0, // Free for local usage
            cost_per_1k_completion_tokens: 0.0,
        }];

        let rate_limits = HashMap::new(); // No rate limits for local models

        LlmCapabilities {
            provider_name: "LocalAI".to_string(),
            models,
            supports_embeddings: true,
            supports_streaming: true,
            supports_functions: true,
            max_context_length: 4096,
            rate_limits,
        }
    }

    fn estimate_cost(&self, _tokens: usize) -> Cost {
        // Local models are free
        Cost::zero("USD")
    }

    async fn health_check(&self) -> Result<()> {
        debug!("Performing LocalAI health check");

        let url = format!("{}/v1/models", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| IntelligenceError::Network(format!("Health check failed: {}", e)))?;

        if response.status().is_success() {
            info!("LocalAI health check successful");
            Ok(())
        } else {
            Err(IntelligenceError::Provider(
                "LocalAI server not responding".to_string(),
            ))
        }
    }

    fn name(&self) -> &str {
        "localai"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_provider_creation() {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        assert_eq!(provider.name(), "ollama");
    }

    #[test]
    fn test_localai_provider_creation() {
        let provider = LocalAIProvider::new("http://localhost:8080".to_string()).unwrap();
        assert_eq!(provider.name(), "localai");
    }

    #[test]
    fn test_ollama_capabilities() {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        let capabilities = provider.capabilities();
        assert_eq!(capabilities.provider_name, "Ollama");
        assert!(capabilities.supports_embeddings);
    }

    #[test]
    fn test_cost_estimation() {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        let cost = provider.estimate_cost(1000);
        assert_eq!(cost.total_cost, 0.0); // Local models are free
    }
}
