//! AWS Bedrock provider implementation (refactored with base utilities)
//! Note: This is a simplified implementation. In practice, you would use the AWS SDK
//! and proper authentication mechanisms.

use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{debug, info};

use super::base::{CostCalculator, ModelCost};
use crate::{
    CompletionRequest, CompletionResponse, Cost, IntelligenceError, LlmCapabilities, LlmProvider,
    ModelInfo, Result, Role, Usage,
};

/// AWS Bedrock provider implementation (simplified/mock)
pub struct BedrockProvider {
    region: String,
    access_key: Option<String>,
    secret_key: Option<String>,
    cost_calculator: CostCalculator,
}

impl BedrockProvider {
    /// Create a new AWS Bedrock provider
    pub fn new(
        region: String,
        access_key: Option<String>,
        secret_key: Option<String>,
    ) -> Result<Self> {
        // Validate region is not empty
        if region.is_empty() {
            return Err(IntelligenceError::Configuration(
                "AWS region cannot be empty".to_string(),
            ));
        }

        // Initialize cost calculator with Bedrock pricing
        let mut cost_calculator = CostCalculator::new()
            .with_default_model("anthropic.claude-3-sonnet-20240229-v1:0".to_string());

        // AWS Bedrock pricing (approximate, varies by region)
        cost_calculator
            .add_model_cost(
                "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
                ModelCost::new(0.003, 0.015),
            )
            .add_model_cost(
                "anthropic.claude-3-haiku-20240307-v1:0".to_string(),
                ModelCost::new(0.00025, 0.00125),
            )
            .add_model_cost(
                "anthropic.claude-3-opus-20240229-v1:0".to_string(),
                ModelCost::new(0.015, 0.075),
            )
            .add_model_cost(
                "amazon.titan-text-express-v1".to_string(),
                ModelCost::new(0.0008, 0.0016),
            )
            .add_model_cost(
                "amazon.titan-text-lite-v1".to_string(),
                ModelCost::new(0.0003, 0.0004),
            )
            .add_model_cost(
                "meta.llama2-70b-chat-v1".to_string(),
                ModelCost::new(0.00195, 0.00256),
            )
            .add_model_cost(
                "meta.llama2-13b-chat-v1".to_string(),
                ModelCost::new(0.00075, 0.001),
            );

        Ok(Self {
            region,
            access_key,
            secret_key,
            cost_calculator,
        })
    }

    /// Get AWS credentials for authentication
    /// Returns (access_key, secret_key) tuple
    pub fn credentials(&self) -> (Option<&String>, Option<&String>) {
        (self.access_key.as_ref(), self.secret_key.as_ref())
    }

    /// Check if credentials are configured
    pub fn has_credentials(&self) -> bool {
        self.access_key.is_some() && self.secret_key.is_some()
    }

    /// Convert internal message format to Bedrock format
    /// Note: Different models in Bedrock have different input formats
    fn build_bedrock_payload(&self, request: &CompletionRequest) -> Result<serde_json::Value> {
        if request.model.starts_with("anthropic.claude") {
            self.build_claude_payload(request)
        } else if request.model.starts_with("amazon.titan") {
            self.build_titan_payload(request)
        } else if request.model.starts_with("meta.llama") {
            self.build_llama_payload(request)
        } else {
            Err(IntelligenceError::Configuration(format!(
                "Unsupported Bedrock model: {}",
                request.model
            )))
        }
    }

    fn build_claude_payload(&self, request: &CompletionRequest) -> Result<serde_json::Value> {
        let mut prompt = String::new();
        let mut _system_message = None;

        for message in &request.messages {
            match message.role {
                Role::System => {
                    _system_message = Some(message.content.clone());
                }
                Role::User => {
                    prompt.push_str(&format!("\n\nHuman: {}", message.content));
                }
                Role::Assistant => {
                    prompt.push_str(&format!("\n\nAssistant: {}", message.content));
                }
                Role::Function => {
                    prompt.push_str(&format!("\n\nHuman: {}", message.content));
                }
            }
        }

        if prompt.is_empty() {
            return Err(IntelligenceError::InvalidRequest(
                "No messages provided".to_string(),
            ));
        }

        // Ensure prompt ends with Assistant prompt
        if !prompt.ends_with("\n\nAssistant:") {
            prompt.push_str("\n\nAssistant:");
        }

        let mut payload = serde_json::json!({
            "prompt": prompt,
            "max_tokens_to_sample": request.max_tokens.unwrap_or(4096),
        });

        if let Some(temperature) = request.temperature {
            if let Some(num) = serde_json::Number::from_f64(temperature as f64) {
                payload["temperature"] = serde_json::Value::Number(num);
            }
        }

        if let Some(top_p) = request.top_p {
            if let Some(num) = serde_json::Number::from_f64(top_p as f64) {
                payload["top_p"] = serde_json::Value::Number(num);
            }
        }

        if let Some(stop_sequences) = &request.stop {
            payload["stop_sequences"] = serde_json::Value::Array(
                stop_sequences
                    .iter()
                    .map(|s| serde_json::Value::String(s.clone()))
                    .collect(),
            );
        }

        Ok(payload)
    }

    fn build_titan_payload(&self, request: &CompletionRequest) -> Result<serde_json::Value> {
        let prompt = request
            .messages
            .iter()
            .map(|msg| {
                format!(
                    "{}: {}",
                    match msg.role {
                        Role::System => "System",
                        Role::User => "User",
                        Role::Assistant => "Assistant",
                        Role::Function => "User",
                    },
                    msg.content
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let mut text_generation_config = serde_json::json!({
            "maxTokenCount": request.max_tokens.unwrap_or(4096),
        });

        if let Some(temperature) = request.temperature {
            text_generation_config["temperature"] = serde_json::Value::Number(
                serde_json::Number::from_f64(temperature as f64).unwrap(),
            );
        }

        if let Some(top_p) = request.top_p {
            text_generation_config["topP"] =
                serde_json::Value::Number(serde_json::Number::from_f64(top_p as f64).unwrap());
        }

        if let Some(stop_sequences) = &request.stop {
            text_generation_config["stopSequences"] = serde_json::Value::Array(
                stop_sequences
                    .iter()
                    .map(|s| serde_json::Value::String(s.clone()))
                    .collect(),
            );
        }

        Ok(serde_json::json!({
            "inputText": prompt,
            "textGenerationConfig": text_generation_config
        }))
    }

    fn build_llama_payload(&self, request: &CompletionRequest) -> Result<serde_json::Value> {
        let prompt = request
            .messages
            .iter()
            .map(|msg| {
                format!(
                    "{}: {}",
                    match msg.role {
                        Role::System => "System",
                        Role::User => "User",
                        Role::Assistant => "Assistant",
                        Role::Function => "User",
                    },
                    msg.content
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let mut payload = serde_json::json!({
            "prompt": prompt,
            "max_gen_len": request.max_tokens.unwrap_or(4096),
        });

        if let Some(temperature) = request.temperature {
            if let Some(num) = serde_json::Number::from_f64(temperature as f64) {
                payload["temperature"] = serde_json::Value::Number(num);
            }
        }

        if let Some(top_p) = request.top_p {
            if let Some(num) = serde_json::Number::from_f64(top_p as f64) {
                payload["top_p"] = serde_json::Value::Number(num);
            }
        }

        Ok(payload)
    }

    /// Parse response based on model type
    /// Used when actual AWS SDK integration is enabled
    #[allow(dead_code)]
    fn parse_bedrock_response(&self, model: &str, response_body: &str) -> Result<(String, Usage)> {
        if model.starts_with("anthropic.claude") {
            self.parse_claude_response(response_body)
        } else if model.starts_with("amazon.titan") {
            self.parse_titan_response(response_body)
        } else if model.starts_with("meta.llama") {
            self.parse_llama_response(response_body)
        } else {
            Err(IntelligenceError::Provider(format!(
                "Unsupported model for response parsing: {}",
                model
            )))
        }
    }

    fn parse_claude_response(&self, response_body: &str) -> Result<(String, Usage)> {
        #[derive(Deserialize)]
        struct ClaudeResponse {
            completion: String,
            #[serde(default)]
            usage: Option<ClaudeUsage>,
        }

        #[derive(Deserialize)]
        struct ClaudeUsage {
            input_tokens: Option<u32>,
            output_tokens: Option<u32>,
        }

        let response: ClaudeResponse = serde_json::from_str(response_body).map_err(|e| {
            IntelligenceError::Provider(format!("Failed to parse Claude response: {}", e))
        })?;

        let usage = if let Some(usage) = response.usage {
            Usage {
                prompt_tokens: usage.input_tokens.unwrap_or(0),
                completion_tokens: usage.output_tokens.unwrap_or(0),
                total_tokens: usage.input_tokens.unwrap_or(0) + usage.output_tokens.unwrap_or(0),
            }
        } else {
            Usage {
                prompt_tokens: 0,
                completion_tokens: response.completion.len() as u32 / 4, // Rough estimate
                total_tokens: response.completion.len() as u32 / 4,
            }
        };

        Ok((response.completion, usage))
    }

    fn parse_titan_response(&self, response_body: &str) -> Result<(String, Usage)> {
        #[derive(Deserialize)]
        struct TitanResponse {
            results: Vec<TitanResult>,
            #[serde(rename = "inputTextTokenCount")]
            input_text_token_count: Option<u32>,
        }

        #[derive(Deserialize)]
        struct TitanResult {
            #[serde(rename = "outputText")]
            output_text: String,
            #[serde(rename = "tokenCount")]
            token_count: Option<u32>,
        }

        let response: TitanResponse = serde_json::from_str(response_body).map_err(|e| {
            IntelligenceError::Provider(format!("Failed to parse Titan response: {}", e))
        })?;

        let result = response.results.into_iter().next().ok_or_else(|| {
            IntelligenceError::Provider("No results in Titan response".to_string())
        })?;

        let usage = Usage {
            prompt_tokens: response.input_text_token_count.unwrap_or(0),
            completion_tokens: result
                .token_count
                .unwrap_or(result.output_text.len() as u32 / 4),
            total_tokens: response.input_text_token_count.unwrap_or(0)
                + result
                    .token_count
                    .unwrap_or(result.output_text.len() as u32 / 4),
        };

        Ok((result.output_text, usage))
    }

    fn parse_llama_response(&self, response_body: &str) -> Result<(String, Usage)> {
        #[derive(Deserialize)]
        struct LlamaResponse {
            generation: String,
            #[serde(rename = "prompt_token_count")]
            prompt_token_count: Option<u32>,
            #[serde(rename = "generation_token_count")]
            generation_token_count: Option<u32>,
        }

        let response: LlamaResponse = serde_json::from_str(response_body).map_err(|e| {
            IntelligenceError::Provider(format!("Failed to parse Llama response: {}", e))
        })?;

        let usage = Usage {
            prompt_tokens: response.prompt_token_count.unwrap_or(0),
            completion_tokens: response
                .generation_token_count
                .unwrap_or(response.generation.len() as u32 / 4),
            total_tokens: response.prompt_token_count.unwrap_or(0)
                + response
                    .generation_token_count
                    .unwrap_or(response.generation.len() as u32 / 4),
        };

        Ok((response.generation, usage))
    }
}

#[async_trait]
impl LlmProvider for BedrockProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        debug!(
            "Sending completion request to AWS Bedrock for model: {}",
            request.model
        );

        // In a real implementation, you would use the AWS SDK to invoke the model
        // For now, this is a placeholder that shows the structure

        // Build the payload for the specific model
        let _payload = self.build_bedrock_payload(&request)?;
        // This is where you would make the actual AWS Bedrock API call
        // Example structure:
        /*
        use aws_sdk_bedrockruntime::Client;
        let client = Client::new(&aws_config);
        let response = client
            .invoke_model()
            .model_id(&request.model)
            .content_type("application/json")
            .body(serde_json::to_vec(&_payload)?)
            .send()
            .await?;
        */

        // For now, return a mock response
        let mock_response = CompletionResponse {
            id: uuid::Uuid::new_v4(),
            request_id: request.id,
            content: "Mock response from Bedrock".to_string(),
            model: request.model.clone(),
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            },
            finish_reason: "stop".to_string(),
            created_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        debug!("Bedrock completion successful (mock)");
        Ok(mock_response)
    }

    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        // AWS Bedrock supports embeddings through specific models like Titan Embeddings
        // This would require a separate model invocation
        Err(IntelligenceError::Provider(
            "Embeddings not implemented for Bedrock provider".to_string(),
        ))
    }

    fn capabilities(&self) -> LlmCapabilities {
        let models = vec![
            ModelInfo {
                id: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
                name: "Claude 3 Sonnet".to_string(),
                description: "Anthropic Claude 3 Sonnet on AWS Bedrock".to_string(),
                max_tokens: 4096,
                supports_functions: false,
                supports_streaming: true,
                cost_per_1k_prompt_tokens: 0.003,
                cost_per_1k_completion_tokens: 0.015,
            },
            ModelInfo {
                id: "anthropic.claude-3-haiku-20240307-v1:0".to_string(),
                name: "Claude 3 Haiku".to_string(),
                description: "Anthropic Claude 3 Haiku on AWS Bedrock".to_string(),
                max_tokens: 4096,
                supports_functions: false,
                supports_streaming: true,
                cost_per_1k_prompt_tokens: 0.00025,
                cost_per_1k_completion_tokens: 0.00125,
            },
            ModelInfo {
                id: "amazon.titan-text-express-v1".to_string(),
                name: "Titan Text Express".to_string(),
                description: "Amazon Titan Text Express model".to_string(),
                max_tokens: 8192,
                supports_functions: false,
                supports_streaming: false,
                cost_per_1k_prompt_tokens: 0.0008,
                cost_per_1k_completion_tokens: 0.0016,
            },
            ModelInfo {
                id: "meta.llama2-70b-chat-v1".to_string(),
                name: "Llama 2 70B Chat".to_string(),
                description: "Meta Llama 2 70B Chat model".to_string(),
                max_tokens: 4096,
                supports_functions: false,
                supports_streaming: false,
                cost_per_1k_prompt_tokens: 0.00195,
                cost_per_1k_completion_tokens: 0.00256,
            },
        ];

        let mut rate_limits = HashMap::new();
        // Bedrock rate limits vary by model and region
        rate_limits.insert("anthropic.claude-3-sonnet-20240229-v1:0".to_string(), 100);
        rate_limits.insert("anthropic.claude-3-haiku-20240307-v1:0".to_string(), 200);
        rate_limits.insert("amazon.titan-text-express-v1".to_string(), 100);
        rate_limits.insert("meta.llama2-70b-chat-v1".to_string(), 50);

        LlmCapabilities {
            provider_name: "AWS Bedrock".to_string(),
            models,
            supports_embeddings: false, // Would need separate implementation
            supports_streaming: true,
            supports_functions: false,
            max_context_length: 100000, // Varies by model
            rate_limits,
        }
    }

    fn estimate_cost(&self, tokens: usize) -> Cost {
        // Use shared cost calculator
        self.cost_calculator.estimate_cost(tokens, None)
    }

    async fn health_check(&self) -> Result<()> {
        debug!("Performing AWS Bedrock health check");

        // In a real implementation, you would check AWS credentials and region accessibility
        // For now, just check that we have the required configuration
        if self.region.is_empty() {
            return Err(IntelligenceError::Configuration(
                "AWS region not configured".to_string(),
            ));
        }

        info!("AWS Bedrock health check successful (mock)");
        Ok(())
    }

    fn name(&self) -> &str {
        "aws_bedrock"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Message;

    #[test]
    fn test_provider_creation() {
        let provider = BedrockProvider::new(
            "us-east-1".to_string(),
            Some("access-key".to_string()),
            Some("secret-key".to_string()),
        )
        .unwrap();
        assert_eq!(provider.name(), "aws_bedrock");
    }

    #[test]
    fn test_capabilities() {
        let provider = BedrockProvider::new("us-east-1".to_string(), None, None).unwrap();
        let capabilities = provider.capabilities();
        assert_eq!(capabilities.provider_name, "AWS Bedrock");
        assert!(!capabilities.models.is_empty());
    }

    #[test]
    fn test_claude_payload_building() {
        let provider = BedrockProvider::new("us-east-1".to_string(), None, None).unwrap();
        let request = CompletionRequest::new(
            "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
            vec![
                Message::system("You are a helpful assistant"),
                Message::user("Hello"),
            ],
        );

        let payload = provider.build_claude_payload(&request).unwrap();
        assert!(payload["prompt"].as_str().unwrap().contains("Human: Hello"));
        assert!(payload["prompt"].as_str().unwrap().contains("Assistant:"));
    }

    #[test]
    fn test_cost_estimation() {
        let provider = BedrockProvider::new("us-east-1".to_string(), None, None).unwrap();
        let cost = provider.estimate_cost(1000);
        assert!(cost.total_cost > 0.0);
        assert_eq!(cost.currency, "USD");
    }

    #[test]
    fn test_credentials() {
        let provider = BedrockProvider::new(
            "us-east-1".to_string(),
            Some("access".to_string()),
            Some("secret".to_string()),
        )
        .unwrap();

        assert!(provider.has_credentials());
        let (access, secret) = provider.credentials();
        assert_eq!(access.unwrap(), "access");
        assert_eq!(secret.unwrap(), "secret");

        // Test without credentials
        let provider_no_creds = BedrockProvider::new("us-east-1".to_string(), None, None).unwrap();
        assert!(!provider_no_creds.has_credentials());
    }

    #[test]
    fn test_parse_responses() {
        let provider = BedrockProvider::new("us-east-1".to_string(), None, None).unwrap();

        // Test Claude response parsing
        let claude_response =
            r#"{"completion": "Hello, world!", "usage": {"input_tokens": 5, "output_tokens": 3}}"#;
        let (content, usage) = provider.parse_claude_response(claude_response).unwrap();
        assert_eq!(content, "Hello, world!");
        assert_eq!(usage.prompt_tokens, 5);
        assert_eq!(usage.completion_tokens, 3);

        // Test Titan response parsing
        let titan_response = r#"{"results": [{"outputText": "Response text", "tokenCount": 10}], "inputTextTokenCount": 20}"#;
        let (content, usage) = provider.parse_titan_response(titan_response).unwrap();
        assert_eq!(content, "Response text");
        assert_eq!(usage.prompt_tokens, 20);
        assert_eq!(usage.completion_tokens, 10);

        // Test Llama response parsing
        let llama_response = r#"{"generation": "Generated text", "prompt_token_count": 15, "generation_token_count": 25}"#;
        let (content, usage) = provider.parse_llama_response(llama_response).unwrap();
        assert_eq!(content, "Generated text");
        assert_eq!(usage.prompt_tokens, 15);
        assert_eq!(usage.completion_tokens, 25);

        // Test parse_bedrock_response routing
        let (content, _) = provider
            .parse_bedrock_response("anthropic.claude-3-sonnet-20240229-v1:0", claude_response)
            .unwrap();
        assert_eq!(content, "Hello, world!");
    }

    #[test]
    fn test_empty_region_validation() {
        let result = BedrockProvider::new("".to_string(), None, None);
        assert!(result.is_err());
    }
}
