//! Google Vertex AI provider implementation (refactored with base utilities)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

use super::base::{CostCalculator, HttpClientBuilder, ModelCost};
use crate::{
    CompletionRequest, CompletionResponse, Cost, IntelligenceError, LlmCapabilities, LlmProvider,
    ModelInfo, Result, Role, Usage,
};

/// Vertex AI API response structure
#[derive(Debug, Deserialize)]
struct VertexResponse {
    candidates: Vec<VertexCandidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<VertexUsage>,
}

#[derive(Debug, Deserialize)]
struct VertexCandidate {
    content: VertexContent,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VertexContent {
    parts: Vec<VertexPart>,
    #[allow(dead_code)] // API response field, not used internally
    role: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VertexPart {
    text: String,
}

#[derive(Debug, Deserialize)]
struct VertexUsage {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: Option<u32>,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: Option<u32>,
    #[serde(rename = "totalTokenCount")]
    total_token_count: Option<u32>,
}

/// Vertex AI API request structure
#[derive(Debug, Serialize)]
struct VertexRequest {
    contents: Vec<VertexContentRequest>,
    #[serde(rename = "generationConfig")]
    generation_config: Option<VertexGenerationConfig>,
    #[serde(rename = "safetySettings")]
    safety_settings: Option<Vec<VertexSafetySetting>>,
}

#[derive(Debug, Serialize)]
struct VertexContentRequest {
    role: String,
    parts: Vec<VertexPartRequest>,
}

#[derive(Debug, Serialize)]
struct VertexPartRequest {
    text: String,
}

#[derive(Debug, Serialize)]
struct VertexGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "topP")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "stopSequences")]
    stop_sequences: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct VertexSafetySetting {
    category: String,
    threshold: String,
}

/// Google Vertex AI provider implementation (refactored)
pub struct VertexAIProvider {
    client: reqwest::Client,
    project_id: String,
    location: String,
    access_token: Option<String>,
    cost_calculator: CostCalculator,
}

impl VertexAIProvider {
    /// Create a new Vertex AI provider
    pub fn new(project_id: String, location: String) -> Result<Self> {
        // Use shared HTTP client builder
        let client = HttpClientBuilder::new().build()?;

        // Initialize cost calculator with Vertex AI pricing
        let mut cost_calculator =
            CostCalculator::new().with_default_model("gemini-1.5-pro".to_string());

        // Vertex AI pricing (approximate, varies by region)
        cost_calculator
            .add_model_cost(
                "gemini-1.5-pro".to_string(),
                ModelCost::new(0.00125, 0.00375),
            )
            .add_model_cost(
                "gemini-1.5-flash".to_string(),
                ModelCost::new(0.000075, 0.0003),
            )
            .add_model_cost("gemini-1.0-pro".to_string(), ModelCost::new(0.0005, 0.0015))
            .add_model_cost("text-bison".to_string(), ModelCost::new(0.001, 0.001))
            .add_model_cost("chat-bison".to_string(), ModelCost::new(0.0005, 0.0005));

        Ok(Self {
            client,
            project_id,
            location,
            access_token: None,
            cost_calculator,
        })
    }

    /// Set access token (would typically be obtained through Google Cloud authentication)
    pub fn with_access_token(mut self, token: String) -> Self {
        self.access_token = Some(token);
        self
    }

    fn convert_role_to_vertex(role: &Role) -> String {
        match role {
            Role::User => "user".to_string(),
            Role::Assistant => "model".to_string(),
            Role::System => "user".to_string(), // Vertex AI handles system messages differently
            Role::Function => "user".to_string(),
        }
    }

    fn build_vertex_request(&self, request: &CompletionRequest) -> VertexRequest {
        let contents = request
            .messages
            .iter()
            .map(|msg| VertexContentRequest {
                role: Self::convert_role_to_vertex(&msg.role),
                parts: vec![VertexPartRequest {
                    text: msg.content.clone(),
                }],
            })
            .collect();

        let generation_config = if request.temperature.is_some()
            || request.top_p.is_some()
            || request.max_tokens.is_some()
            || request.stop.is_some()
        {
            Some(VertexGenerationConfig {
                temperature: request.temperature,
                top_p: request.top_p,
                max_output_tokens: request.max_tokens,
                stop_sequences: request.stop.clone(),
            })
        } else {
            None
        };

        // Default safety settings (permissive for development)
        let safety_settings = Some(vec![
            VertexSafetySetting {
                category: "HARM_CATEGORY_HARASSMENT".to_string(),
                threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
            },
            VertexSafetySetting {
                category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
                threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
            },
            VertexSafetySetting {
                category: "HARM_CATEGORY_SEXUALLY_EXPLICIT".to_string(),
                threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
            },
            VertexSafetySetting {
                category: "HARM_CATEGORY_DANGEROUS_CONTENT".to_string(),
                threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
            },
        ]);

        VertexRequest {
            contents,
            generation_config,
            safety_settings,
        }
    }

    fn build_endpoint_url(&self, model: &str) -> String {
        format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
            self.location,
            self.project_id,
            self.location,
            model
        )
    }

    /// Vertex-specific request method (uses Bearer token authentication)
    async fn make_request<T>(&self, url: &str, payload: &impl Serialize) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        debug!("Making POST request to: {}", url);

        let mut request_builder = self
            .client
            .post(url)
            .header("Content-Type", "application/json");

        if let Some(ref token) = self.access_token {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", token));
        }

        let response = request_builder
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
                "Vertex AI API error ({}): {}",
                status, error_text
            )));
        }

        response
            .json::<T>()
            .await
            .map_err(|e| IntelligenceError::Provider(format!("Failed to parse response: {}", e)))
    }
}

#[async_trait]
impl LlmProvider for VertexAIProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        debug!(
            "Sending completion request to Vertex AI for model: {}",
            request.model
        );

        let vertex_request = self.build_vertex_request(&request);
        let url = self.build_endpoint_url(&request.model);
        let response: VertexResponse = self.make_request(&url, &vertex_request).await?;

        let candidate = response.candidates.into_iter().next().ok_or_else(|| {
            IntelligenceError::Provider("No completion candidates returned".to_string())
        })?;

        let content = candidate
            .content
            .parts
            .into_iter()
            .map(|part| part.text)
            .collect::<Vec<_>>()
            .join("");

        let usage = if let Some(usage_metadata) = response.usage_metadata {
            Usage {
                prompt_tokens: usage_metadata.prompt_token_count.unwrap_or(0),
                completion_tokens: usage_metadata.candidates_token_count.unwrap_or(0),
                total_tokens: usage_metadata.total_token_count.unwrap_or(0),
            }
        } else {
            Usage {
                prompt_tokens: 0,
                completion_tokens: content.len() as u32 / 4, // Rough estimate
                total_tokens: content.len() as u32 / 4,
            }
        };

        let total_tokens = usage.total_tokens;
        let completion_response = CompletionResponse {
            id: uuid::Uuid::new_v4(),
            request_id: request.id,
            content,
            model: request.model,
            usage,
            finish_reason: candidate
                .finish_reason
                .unwrap_or_else(|| "stop".to_string()),
            created_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        debug!(
            "Vertex AI completion successful, tokens used: {}",
            total_tokens
        );
        Ok(completion_response)
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        debug!("Generating embeddings with Vertex AI");

        // Vertex AI has specific embedding models
        #[derive(Serialize)]
        struct EmbeddingRequest {
            instances: Vec<EmbeddingInstance>,
        }

        #[derive(Serialize)]
        struct EmbeddingInstance {
            content: String,
        }

        #[derive(Deserialize)]
        struct EmbeddingResponse {
            predictions: Vec<EmbeddingPrediction>,
        }

        #[derive(Deserialize)]
        struct EmbeddingPrediction {
            embeddings: EmbeddingValues,
        }

        #[derive(Deserialize)]
        struct EmbeddingValues {
            values: Vec<f32>,
        }

        let request = EmbeddingRequest {
            instances: vec![EmbeddingInstance {
                content: text.to_string(),
            }],
        };

        // Use text-embedding-004 model for embeddings
        let url = format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/text-embedding-004:predict",
            self.location,
            self.project_id,
            self.location
        );

        let response: EmbeddingResponse = self.make_request(&url, &request).await?;

        let prediction = response.predictions.into_iter().next().ok_or_else(|| {
            IntelligenceError::Provider("No embedding predictions returned".to_string())
        })?;

        debug!(
            "Vertex AI embedding successful, dimensions: {}",
            prediction.embeddings.values.len()
        );
        Ok(prediction.embeddings.values)
    }

    fn capabilities(&self) -> LlmCapabilities {
        let models = vec![
            ModelInfo {
                id: "gemini-1.5-pro".to_string(),
                name: "Gemini 1.5 Pro".to_string(),
                description: "Google's most capable multimodal model".to_string(),
                max_tokens: 8192,
                supports_functions: true,
                supports_streaming: true,
                cost_per_1k_prompt_tokens: 0.00125,
                cost_per_1k_completion_tokens: 0.00375,
            },
            ModelInfo {
                id: "gemini-1.5-flash".to_string(),
                name: "Gemini 1.5 Flash".to_string(),
                description: "Fast and efficient multimodal model".to_string(),
                max_tokens: 8192,
                supports_functions: true,
                supports_streaming: true,
                cost_per_1k_prompt_tokens: 0.000075,
                cost_per_1k_completion_tokens: 0.0003,
            },
            ModelInfo {
                id: "gemini-1.0-pro".to_string(),
                name: "Gemini 1.0 Pro".to_string(),
                description: "Powerful text and reasoning model".to_string(),
                max_tokens: 32768,
                supports_functions: true,
                supports_streaming: true,
                cost_per_1k_prompt_tokens: 0.0005,
                cost_per_1k_completion_tokens: 0.0015,
            },
            ModelInfo {
                id: "text-bison".to_string(),
                name: "Text Bison".to_string(),
                description: "Text generation model".to_string(),
                max_tokens: 1024,
                supports_functions: false,
                supports_streaming: false,
                cost_per_1k_prompt_tokens: 0.001,
                cost_per_1k_completion_tokens: 0.001,
            },
        ];

        let mut rate_limits = HashMap::new();
        rate_limits.insert("gemini-1.5-pro".to_string(), 60);
        rate_limits.insert("gemini-1.5-flash".to_string(), 100);
        rate_limits.insert("gemini-1.0-pro".to_string(), 60);
        rate_limits.insert("text-bison".to_string(), 100);

        LlmCapabilities {
            provider_name: "Google Vertex AI".to_string(),
            models,
            supports_embeddings: true,
            supports_streaming: true,
            supports_functions: true,
            max_context_length: 32768, // Varies by model
            rate_limits,
        }
    }

    fn estimate_cost(&self, tokens: usize) -> Cost {
        // Use shared cost calculator
        self.cost_calculator.estimate_cost(tokens, None)
    }

    async fn health_check(&self) -> Result<()> {
        debug!("Performing Vertex AI health check");

        // Check if we have the required configuration
        if self.project_id.is_empty() {
            return Err(IntelligenceError::Configuration(
                "Google Cloud project ID not configured".to_string(),
            ));
        }

        // In a real implementation, you would make a simple API call to verify connectivity
        // For now, just verify we have authentication
        if self.access_token.is_none() {
            return Err(IntelligenceError::Configuration(
                "Vertex AI access token not configured".to_string(),
            ));
        }

        info!("Vertex AI health check successful");
        Ok(())
    }

    fn name(&self) -> &str {
        "google_vertex"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider =
            VertexAIProvider::new("test-project".to_string(), "us-central1".to_string()).unwrap();
        assert_eq!(provider.name(), "google_vertex");
    }

    #[test]
    fn test_capabilities() {
        let provider =
            VertexAIProvider::new("test-project".to_string(), "us-central1".to_string()).unwrap();
        let capabilities = provider.capabilities();
        assert_eq!(capabilities.provider_name, "Google Vertex AI");
        assert!(capabilities.supports_embeddings);
        assert!(!capabilities.models.is_empty());
    }

    #[test]
    fn test_endpoint_url_building() {
        let provider =
            VertexAIProvider::new("test-project".to_string(), "us-central1".to_string()).unwrap();

        let url = provider.build_endpoint_url("gemini-1.5-pro");
        assert!(url.contains("test-project"));
        assert!(url.contains("us-central1"));
        assert!(url.contains("gemini-1.5-pro"));
        assert!(url.contains("generateContent"));
    }

    #[test]
    fn test_cost_estimation() {
        let provider =
            VertexAIProvider::new("test-project".to_string(), "us-central1".to_string()).unwrap();
        let cost = provider.estimate_cost(1000);
        assert!(cost.total_cost > 0.0);
        assert_eq!(cost.currency, "USD");
    }
}
