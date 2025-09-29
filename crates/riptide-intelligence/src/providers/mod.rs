//! Built-in provider implementations
//!
//! This module contains concrete implementations of LLM providers for various services:
//! - OpenAI (GPT models)
//! - Anthropic (Claude models)
//! - Local models (Ollama, LocalAI)
//! - Azure OpenAI
//! - AWS Bedrock
//! - Google Vertex AI

pub mod openai;
pub mod anthropic;
pub mod local;
pub mod azure;
pub mod aws_bedrock;
pub mod google_vertex;

// Re-export provider implementations
pub use openai::OpenAIProvider;
pub use anthropic::AnthropicProvider;
pub use local::{OllamaProvider, LocalAIProvider};
pub use azure::AzureOpenAIProvider;
pub use aws_bedrock::BedrockProvider;
pub use google_vertex::VertexAIProvider;

use std::sync::Arc;

use crate::{LlmProvider, registry::ProviderConfig, IntelligenceError, Result};

/// Factory function to create providers from configuration
pub fn create_provider_from_config(config: &ProviderConfig) -> Result<Arc<dyn LlmProvider>> {
    match config.provider_type.as_str() {
        "openai" => {
            let api_key = get_config_string(config, "api_key")?;
            let base_url = get_config_string_optional(config, "base_url");
            Ok(Arc::new(OpenAIProvider::new(api_key, base_url)?))
        }
        "anthropic" => {
            let api_key = get_config_string(config, "api_key")?;
            Ok(Arc::new(AnthropicProvider::new(api_key)?))
        }
        "ollama" => {
            let base_url = get_config_string_optional(config, "base_url")
                .unwrap_or_else(|| "http://localhost:11434".to_string());
            Ok(Arc::new(OllamaProvider::new(base_url)?))
        }
        "localai" => {
            let base_url = get_config_string(config, "base_url")?;
            Ok(Arc::new(LocalAIProvider::new(base_url)?))
        }
        "azure_openai" => {
            let api_key = get_config_string(config, "api_key")?;
            let endpoint = get_config_string(config, "endpoint")?;
            let api_version = get_config_string_optional(config, "api_version")
                .unwrap_or_else(|| "2023-12-01-preview".to_string());
            Ok(Arc::new(AzureOpenAIProvider::new(api_key, endpoint, api_version)?))
        }
        "aws_bedrock" => {
            let region = get_config_string(config, "region")?;
            let access_key = get_config_string_optional(config, "access_key");
            let secret_key = get_config_string_optional(config, "secret_key");
            Ok(Arc::new(BedrockProvider::new(region, access_key, secret_key)?))
        }
        "google_vertex" => {
            let project_id = get_config_string(config, "project_id")?;
            let location = get_config_string_optional(config, "location")
                .unwrap_or_else(|| "us-central1".to_string());
            Ok(Arc::new(VertexAIProvider::new(project_id, location)?))
        }
        _ => Err(IntelligenceError::Configuration(
            format!("Unknown provider type: {}", config.provider_type)
        )),
    }
}

/// Helper to get string configuration value
fn get_config_string(config: &ProviderConfig, key: &str) -> Result<String> {
    config.config.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| IntelligenceError::Configuration(
            format!("Missing required configuration key: {}", key)
        ))
}

/// Helper to get optional string configuration value
fn get_config_string_optional(config: &ProviderConfig, key: &str) -> Option<String> {
    config.config.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Helper to get boolean configuration value
#[allow(dead_code)]
fn get_config_bool(config: &ProviderConfig, key: &str, default: bool) -> bool {
    config.config.get(key)
        .and_then(|v| v.as_bool())
        .unwrap_or(default)
}

/// Helper to get number configuration value
#[allow(dead_code)]
fn get_config_f64(config: &ProviderConfig, key: &str, default: f64) -> f64 {
    config.config.get(key)
        .and_then(|v| v.as_f64())
        .unwrap_or(default)
}

/// Register all built-in provider factories
pub fn register_builtin_providers(registry: &crate::registry::LlmRegistry) -> Result<()> {
    // OpenAI
    registry.register_factory("openai", |config| {
        create_provider_from_config(config)
    })?;

    // Anthropic
    registry.register_factory("anthropic", |config| {
        create_provider_from_config(config)
    })?;

    // Local providers
    registry.register_factory("ollama", |config| {
        create_provider_from_config(config)
    })?;

    registry.register_factory("localai", |config| {
        create_provider_from_config(config)
    })?;

    // Cloud providers
    registry.register_factory("azure_openai", |config| {
        create_provider_from_config(config)
    })?;

    registry.register_factory("aws_bedrock", |config| {
        create_provider_from_config(config)
    })?;

    registry.register_factory("google_vertex", |config| {
        create_provider_from_config(config)
    })?;

    Ok(())
}