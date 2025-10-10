//! Integration tests for all LLM providers
//!
//! These tests verify that all providers can be instantiated,
//! configured, and perform basic operations.

use riptide_intelligence::{
    providers::{
        register_builtin_providers, BedrockProvider, LocalAIProvider, OllamaProvider,
        VertexAIProvider,
    },
    CompletionRequest, LlmProvider, LlmRegistry, Message, ProviderConfig,
};
use serde_json::json;

#[test]
fn test_all_providers_registration() {
    let registry = LlmRegistry::new();

    // Register all built-in providers
    register_builtin_providers(&registry).expect("Failed to register providers");

    // Verify factories are registered
    let stats = registry.stats();
    assert!(
        stats.registered_factories >= 7,
        "Expected at least 7 provider factories"
    );
}

#[test]
fn test_vertex_ai_provider_creation() {
    let provider = VertexAIProvider::new("test-project".to_string(), "us-central1".to_string())
        .expect("Failed to create Vertex AI provider");

    assert_eq!(provider.name(), "google_vertex");

    let capabilities = provider.capabilities();
    assert_eq!(capabilities.provider_name, "Google Vertex AI");
    assert!(capabilities.supports_embeddings);
    assert!(!capabilities.models.is_empty());
}

#[test]
fn test_bedrock_provider_creation() {
    let provider = BedrockProvider::new(
        "us-east-1".to_string(),
        Some("test-key".to_string()),
        Some("test-secret".to_string()),
    )
    .expect("Failed to create Bedrock provider");

    assert_eq!(provider.name(), "aws_bedrock");
    assert!(provider.has_credentials());

    let capabilities = provider.capabilities();
    assert_eq!(capabilities.provider_name, "AWS Bedrock");
    assert!(!capabilities.models.is_empty());
}

#[test]
fn test_ollama_provider_creation() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string())
        .expect("Failed to create Ollama provider");

    assert_eq!(provider.name(), "ollama");

    let capabilities = provider.capabilities();
    assert_eq!(capabilities.provider_name, "Ollama");
    assert!(capabilities.supports_embeddings);
    assert_eq!(capabilities.rate_limits.len(), 0); // No rate limits for local
}

#[test]
fn test_localai_provider_creation() {
    let provider = LocalAIProvider::new("http://localhost:8080".to_string())
        .expect("Failed to create LocalAI provider");

    assert_eq!(provider.name(), "localai");

    let capabilities = provider.capabilities();
    assert_eq!(capabilities.provider_name, "LocalAI");
    assert!(capabilities.supports_embeddings);
    assert!(capabilities.supports_functions);
}

#[tokio::test]
async fn test_provider_health_checks() {
    // Test Vertex AI health check (should fail without credentials)
    let vertex_provider =
        VertexAIProvider::new("test-project".to_string(), "us-central1".to_string())
            .expect("Failed to create provider");

    let health_result = vertex_provider.health_check().await;
    assert!(
        health_result.is_err(),
        "Health check should fail without access token"
    );

    // Test Bedrock health check (should succeed with region)
    let bedrock_provider = BedrockProvider::new("us-east-1".to_string(), None, None)
        .expect("Failed to create provider");

    let health_result = bedrock_provider.health_check().await;
    assert!(
        health_result.is_ok(),
        "Health check should succeed with valid region"
    );

    // Test Ollama health check (will fail if server not running, which is expected)
    let ollama_provider = OllamaProvider::new("http://localhost:11434".to_string())
        .expect("Failed to create provider");

    let _health_result = ollama_provider.health_check().await;
    // Don't assert here since Ollama may not be running in CI
}

#[test]
fn test_cost_estimation() {
    // Test Vertex AI cost
    let vertex = VertexAIProvider::new("test-project".to_string(), "us-central1".to_string())
        .expect("Failed to create provider");
    let cost = vertex.estimate_cost(1000);
    assert!(cost.total_cost > 0.0);
    assert_eq!(cost.currency, "USD");

    // Test Bedrock cost
    let bedrock = BedrockProvider::new("us-east-1".to_string(), None, None)
        .expect("Failed to create provider");
    let cost = bedrock.estimate_cost(1000);
    assert!(cost.total_cost > 0.0);
    assert_eq!(cost.currency, "USD");

    // Test Ollama cost (free)
    let ollama = OllamaProvider::new("http://localhost:11434".to_string())
        .expect("Failed to create provider");
    let cost = ollama.estimate_cost(1000);
    assert_eq!(cost.total_cost, 0.0);
    assert_eq!(cost.currency, "USD");
}

#[test]
fn test_provider_config_loading() {
    let registry = LlmRegistry::new();
    register_builtin_providers(&registry).expect("Failed to register providers");

    // Test loading Vertex AI provider
    let vertex_config = ProviderConfig::new("vertex-test", "google_vertex")
        .with_config("project_id", json!("test-project"))
        .with_config("location", json!("us-central1"));

    let result = registry.load_provider(vertex_config);
    assert!(result.is_ok(), "Failed to load Vertex AI provider");
    assert!(registry.has_provider("vertex-test"));

    // Test loading Bedrock provider
    let bedrock_config = ProviderConfig::new("bedrock-test", "aws_bedrock")
        .with_config("region", json!("us-east-1"));

    let result = registry.load_provider(bedrock_config);
    assert!(result.is_ok(), "Failed to load Bedrock provider");
    assert!(registry.has_provider("bedrock-test"));

    // Test loading Ollama provider
    let ollama_config = ProviderConfig::new("ollama-test", "ollama")
        .with_config("base_url", json!("http://localhost:11434"));

    let result = registry.load_provider(ollama_config);
    assert!(result.is_ok(), "Failed to load Ollama provider");
    assert!(registry.has_provider("ollama-test"));
}

#[test]
fn test_fallback_provider_ordering() {
    let registry = LlmRegistry::new();
    register_builtin_providers(&registry).expect("Failed to register providers");

    // Load providers with different fallback orders
    let config1 = ProviderConfig::new("provider-1", "ollama")
        .with_config("base_url", json!("http://localhost:11434"))
        .with_fallback_order(2);

    let config2 = ProviderConfig::new("provider-2", "ollama")
        .with_config("base_url", json!("http://localhost:11435"))
        .with_fallback_order(1);

    let config3 = ProviderConfig::new("provider-3", "ollama")
        .with_config("base_url", json!("http://localhost:11436"))
        .with_fallback_order(3);

    registry
        .load_providers(vec![config1, config2, config3])
        .expect("Failed to load providers");

    let fallback_providers = registry.get_fallback_providers();
    assert_eq!(fallback_providers.len(), 3);

    // Verify ordering
    assert_eq!(fallback_providers[0].0, "provider-2"); // Order 1
    assert_eq!(fallback_providers[1].0, "provider-1"); // Order 2
    assert_eq!(fallback_providers[2].0, "provider-3"); // Order 3
}

#[test]
fn test_model_information() {
    // Test Vertex AI models
    let vertex = VertexAIProvider::new("test-project".to_string(), "us-central1".to_string())
        .expect("Failed to create provider");
    let caps = vertex.capabilities();

    let gemini_pro = caps.models.iter().find(|m| m.id == "gemini-1.5-pro");
    assert!(gemini_pro.is_some());

    let model = gemini_pro.unwrap();
    assert!(model.supports_functions);
    assert!(model.supports_streaming);
    assert!(model.cost_per_1k_prompt_tokens > 0.0);

    // Test Bedrock models
    let bedrock = BedrockProvider::new("us-east-1".to_string(), None, None)
        .expect("Failed to create provider");
    let caps = bedrock.capabilities();

    let claude = caps
        .models
        .iter()
        .find(|m| m.id.contains("claude-3-sonnet"));
    assert!(claude.is_some());

    // Test Ollama models
    let ollama = OllamaProvider::new("http://localhost:11434".to_string())
        .expect("Failed to create provider");
    let caps = ollama.capabilities();

    let llama = caps.models.iter().find(|m| m.id == "llama3.2");
    assert!(llama.is_some());

    let model = llama.unwrap();
    assert_eq!(model.cost_per_1k_prompt_tokens, 0.0); // Free
    assert_eq!(model.cost_per_1k_completion_tokens, 0.0); // Free
}

#[test]
fn test_completion_request_building() {
    let messages = vec![
        Message::system("You are a helpful assistant"),
        Message::user("Hello, how are you?"),
    ];

    let mut request = CompletionRequest::new("test-model".to_string(), messages)
        .with_temperature(0.7)
        .with_max_tokens(100);

    request.top_p = Some(0.9); // Direct field access since no builder method

    assert_eq!(request.model, "test-model");
    assert_eq!(request.messages.len(), 2);
    assert_eq!(request.temperature, Some(0.7));
    assert_eq!(request.max_tokens, Some(100));
    assert_eq!(request.top_p, Some(0.9));
}

#[test]
fn test_provider_capabilities_comparison() {
    let vertex = VertexAIProvider::new("test-project".to_string(), "us-central1".to_string())
        .expect("Failed to create provider");
    let bedrock = BedrockProvider::new("us-east-1".to_string(), None, None)
        .expect("Failed to create provider");
    let ollama = OllamaProvider::new("http://localhost:11434".to_string())
        .expect("Failed to create provider");

    let vertex_caps = vertex.capabilities();
    let bedrock_caps = bedrock.capabilities();
    let ollama_caps = ollama.capabilities();

    // All should support text generation
    assert!(!vertex_caps.models.is_empty());
    assert!(!bedrock_caps.models.is_empty());
    assert!(!ollama_caps.models.is_empty());

    // Vertex and Ollama support embeddings
    assert!(vertex_caps.supports_embeddings);
    assert!(ollama_caps.supports_embeddings);
    assert!(!bedrock_caps.supports_embeddings); // Mock implementation doesn't

    // Vertex supports functions
    assert!(vertex_caps.supports_functions);

    // Ollama and Vertex support streaming
    assert!(vertex_caps.supports_streaming);
    assert!(ollama_caps.supports_streaming);
}

#[test]
fn test_invalid_provider_configs() {
    let registry = LlmRegistry::new();
    register_builtin_providers(&registry).expect("Failed to register providers");

    // Test loading unknown provider type
    let invalid_config = ProviderConfig::new("invalid", "unknown_provider");

    let result = registry.load_provider(invalid_config);
    assert!(result.is_err(), "Should fail with unknown provider type");

    // Test Bedrock with empty region
    let bedrock_config =
        ProviderConfig::new("bedrock-invalid", "aws_bedrock").with_config("region", json!(""));

    let result = registry.load_provider(bedrock_config);
    assert!(result.is_err(), "Should fail with empty region");
}

#[test]
fn test_registry_stats() {
    let registry = LlmRegistry::new();
    register_builtin_providers(&registry).expect("Failed to register providers");

    // Load some providers
    let configs = vec![
        ProviderConfig::new("test-1", "ollama")
            .with_config("base_url", json!("http://localhost:11434")),
        ProviderConfig::new("test-2", "ollama")
            .with_config("base_url", json!("http://localhost:11435")),
        ProviderConfig::new("test-3", "ollama")
            .with_config("base_url", json!("http://localhost:11436"))
            .disabled(),
    ];

    registry
        .load_providers(configs)
        .expect("Failed to load providers");

    let stats = registry.stats();
    assert!(stats.total_providers >= 2); // At least 2 enabled
    assert!(stats.enabled_providers >= 2);
    assert!(stats.registered_factories >= 7);
}
