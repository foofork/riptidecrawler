//! Core LLM provider trait and related types

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{IntelligenceError, Result};

/// Role of a message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Function,
}

/// A message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<serde_json::Value>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
            name: None,
            function_call: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
            name: None,
            function_call: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
            name: None,
            function_call: None,
        }
    }
}

/// Request for text completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub id: Uuid,
    pub messages: Vec<Message>,
    pub model: String,
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
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl CompletionRequest {
    pub fn new(model: impl Into<String>, messages: Vec<Message>) -> Self {
        Self {
            id: Uuid::new_v4(),
            messages,
            model: model.into(),
            max_tokens: None,
            temperature: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Usage statistics for a completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Response from text completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub id: Uuid,
    pub request_id: Uuid,
    pub content: String,
    pub model: String,
    pub usage: Usage,
    pub finish_reason: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl CompletionResponse {
    pub fn new(
        request_id: Uuid,
        content: impl Into<String>,
        model: impl Into<String>,
        usage: Usage,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            request_id,
            content: content.into(),
            model: model.into(),
            usage,
            finish_reason: "stop".to_string(),
            created_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }
}

/// Cost information for LLM operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cost {
    pub prompt_cost: f64,
    pub completion_cost: f64,
    pub total_cost: f64,
    pub currency: String,
}

impl Cost {
    pub fn new(prompt_cost: f64, completion_cost: f64, currency: impl Into<String>) -> Self {
        Self {
            prompt_cost,
            completion_cost,
            total_cost: prompt_cost + completion_cost,
            currency: currency.into(),
        }
    }

    pub fn zero(currency: impl Into<String>) -> Self {
        Self::new(0.0, 0.0, currency)
    }
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub max_tokens: u32,
    pub supports_functions: bool,
    pub supports_streaming: bool,
    pub cost_per_1k_prompt_tokens: f64,
    pub cost_per_1k_completion_tokens: f64,
}

/// Capabilities of an LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCapabilities {
    pub provider_name: String,
    pub models: Vec<ModelInfo>,
    pub supports_embeddings: bool,
    pub supports_streaming: bool,
    pub supports_functions: bool,
    pub max_context_length: u32,
    pub rate_limits: HashMap<String, u32>, // model -> requests per minute
}

/// Core trait that all LLM providers must implement
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Generate a text completion
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;

    /// Generate embeddings for the given text
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Get provider capabilities
    fn capabilities(&self) -> LlmCapabilities;

    /// Estimate cost for a given number of tokens
    fn estimate_cost(&self, tokens: usize) -> Cost;

    /// Get the provider's health status
    async fn health_check(&self) -> Result<()> {
        // Default implementation - providers can override
        Ok(())
    }

    /// Get provider name
    fn name(&self) -> &str;

    /// Check if the provider is available
    async fn is_available(&self) -> bool {
        self.health_check().await.is_ok()
    }
}

/// Helper trait for creating provider-specific errors
pub trait ProviderError {
    fn into_intelligence_error(self) -> IntelligenceError;
}

impl ProviderError for String {
    fn into_intelligence_error(self) -> IntelligenceError {
        IntelligenceError::Provider(self)
    }
}

impl ProviderError for &str {
    fn into_intelligence_error(self) -> IntelligenceError {
        IntelligenceError::Provider(self.to_string())
    }
}