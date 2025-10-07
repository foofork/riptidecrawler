//! LLM provider management API handlers
//!
//! This module implements LLM provider management endpoints that utilize riptide-intelligence's
//! multi-provider system for runtime configuration and provider switching.

use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use riptide_intelligence::{
    metrics::MetricsCollector,
    providers::register_builtin_providers,
    runtime_switch::{RuntimeSwitchConfig, RuntimeSwitchManager},
    LlmRegistry, ProviderConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn};

/// Response for listing available LLM providers
#[derive(Serialize, Debug)]
pub struct ProvidersResponse {
    /// List of available providers
    pub providers: Vec<ProviderInfo>,
    /// Currently active provider
    pub current_provider: Option<String>,
    /// Total number of providers
    pub total_providers: usize,
}

/// Information about an LLM provider
#[derive(Serialize, Debug)]
pub struct ProviderInfo {
    /// Provider name/identifier
    pub name: String,
    /// Provider type (openai, anthropic, etc.)
    pub provider_type: String,
    /// Current status
    pub status: String,
    /// Provider capabilities
    pub capabilities: Vec<String>,
    /// Required configuration fields
    pub config_required: Vec<String>,
    /// Whether provider is currently available
    pub available: bool,
    /// Cost information (if available)
    pub cost_info: Option<CostInfo>,
    /// Model information
    pub models: Vec<ModelInfo>,
}

/// Cost information for a provider
#[derive(Serialize, Debug)]
pub struct CostInfo {
    /// Cost per 1K input tokens
    pub input_token_cost: Option<f64>,
    /// Cost per 1K output tokens
    pub output_token_cost: Option<f64>,
    /// Currency (e.g., "USD")
    pub currency: String,
}

/// Model information
#[derive(Serialize, Debug)]
pub struct ModelInfo {
    /// Model name
    pub name: String,
    /// Model context window
    pub context_window: Option<usize>,
    /// Maximum output tokens
    pub max_output_tokens: Option<usize>,
    /// Whether model supports function calling
    pub supports_functions: bool,
}

/// Request to switch active LLM provider
#[derive(Deserialize, Debug)]
pub struct SwitchProviderRequest {
    /// Name of the provider to switch to
    pub provider_name: String,
    /// Optional configuration updates to apply during switch
    pub config_updates: Option<HashMap<String, String>>,
    /// Whether to perform gradual rollout
    #[serde(default)]
    pub gradual_rollout: bool,
    /// Rollout percentage (0-100) if gradual_rollout is true
    #[serde(default = "default_rollout_percentage")]
    pub rollout_percentage: u32,
}

fn default_rollout_percentage() -> u32 {
    100
}

/// Response for provider switch operation
#[derive(Serialize, Debug)]
pub struct SwitchProviderResponse {
    /// Whether switch was successful
    pub success: bool,
    /// Previous active provider
    pub previous_provider: Option<String>,
    /// New active provider
    pub new_provider: String,
    /// Switch timestamp
    pub switched_at: String,
    /// Additional information
    pub message: String,
}

/// Request to update LLM configuration
#[derive(Deserialize, Debug)]
pub struct ConfigUpdateRequest {
    /// Provider-specific configuration
    pub provider_configs: HashMap<String, HashMap<String, String>>,
    /// Global configuration settings
    pub global_config: Option<HashMap<String, String>>,
    /// Whether to validate configuration before applying
    #[serde(default = "default_true")]
    pub validate: bool,
}

fn default_true() -> bool {
    true
}

/// Response for configuration update
#[derive(Serialize, Debug)]
pub struct ConfigUpdateResponse {
    /// Whether update was successful
    pub success: bool,
    /// Validation results
    pub validation_results: HashMap<String, String>,
    /// Updated configuration summary
    pub updated_config: ConfigSummary,
    /// Update timestamp
    pub updated_at: String,
}

/// Configuration summary
#[derive(Serialize, Debug)]
pub struct ConfigSummary {
    /// Number of configured providers
    pub configured_providers: usize,
    /// Active provider
    pub active_provider: Option<String>,
    /// Global settings
    pub global_settings: HashMap<String, String>,
}

/// Query parameters for provider listing
#[derive(Deserialize, Debug)]
pub struct ProviderQuery {
    /// Filter by provider type
    pub provider_type: Option<String>,
    /// Include only available providers
    #[serde(default)]
    pub available_only: bool,
    /// Include cost information
    #[serde(default)]
    pub include_cost: bool,
    /// Include model details
    #[serde(default)]
    pub include_models: bool,
}

/// Static registry for LLM management (in production, this would be persistent)
static LLM_REGISTRY: std::sync::OnceLock<Arc<tokio::sync::Mutex<LlmRegistry>>> =
    std::sync::OnceLock::new();

static RUNTIME_SWITCH_MANAGER: std::sync::OnceLock<Arc<tokio::sync::Mutex<RuntimeSwitchManager>>> =
    std::sync::OnceLock::new();

static CURRENT_PROVIDER: std::sync::OnceLock<Arc<tokio::sync::Mutex<Option<String>>>> =
    std::sync::OnceLock::new();

fn get_llm_registry() -> Arc<tokio::sync::Mutex<LlmRegistry>> {
    LLM_REGISTRY
        .get_or_init(|| {
            let registry = LlmRegistry::new();

            // Register built-in providers with default configurations
            if let Err(e) = register_builtin_providers(&registry) {
                warn!("Failed to register built-in providers: {}", e);
            }

            Arc::new(tokio::sync::Mutex::new(registry))
        })
        .clone()
}

fn get_runtime_switch_manager() -> Arc<tokio::sync::Mutex<RuntimeSwitchManager>> {
    RUNTIME_SWITCH_MANAGER
        .get_or_init(|| {
            // Create a new registry instance for the RuntimeSwitchManager
            // We can't share the mutex-wrapped registry, so we create a new one
            let registry = LlmRegistry::new();
            if let Err(e) = register_builtin_providers(&registry) {
                warn!(
                    "Failed to register built-in providers in runtime switch manager: {}",
                    e
                );
            }
            let registry_arc = Arc::new(registry);

            let metrics_collector = Arc::new(MetricsCollector::new(7)); // 7 days retention
            let config = RuntimeSwitchConfig::default();

            let (manager, _event_rx) =
                RuntimeSwitchManager::new(registry_arc, None, metrics_collector, config);
            Arc::new(tokio::sync::Mutex::new(manager))
        })
        .clone()
}

fn get_current_provider() -> Arc<tokio::sync::Mutex<Option<String>>> {
    CURRENT_PROVIDER
        .get_or_init(|| Arc::new(tokio::sync::Mutex::new(Some("openai".to_string()))))
        .clone()
}

/// List available LLM providers
///
/// This endpoint returns information about all available LLM providers,
/// their capabilities, status, and configuration requirements.
///
/// ## Query Parameters
/// - `provider_type`: Filter by provider type (openai, anthropic, etc.)
/// - `available_only`: Include only currently available providers
/// - `include_cost`: Include cost information in response
/// - `include_models`: Include detailed model information
pub async fn list_providers(
    State(state): State<AppState>,
    Query(params): Query<ProviderQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();

    info!(
        provider_type = ?params.provider_type,
        available_only = params.available_only,
        "Received list providers request"
    );

    let registry = get_llm_registry();
    let registry_guard = registry.lock().await;
    let current_provider_ref = get_current_provider();
    let current_provider = current_provider_ref.lock().await.clone();

    // Get all registered providers
    let provider_names = registry_guard.list_providers();
    let mut providers = Vec::new();

    for provider_name in provider_names {
        // Skip if filtering by type
        if let Some(ref filter_type) = params.provider_type {
            if !provider_name.contains(filter_type) {
                continue;
            }
        }

        let provider_info = create_provider_info(&registry_guard, &provider_name, &params).await?;

        // Skip if filtering by availability
        if params.available_only && !provider_info.available {
            continue;
        }

        providers.push(provider_info);
    }

    drop(registry_guard);

    let total_providers = providers.len();
    let response = ProvidersResponse {
        providers,
        current_provider,
        total_providers,
    };

    info!(
        total_providers = response.total_providers,
        current_provider = ?response.current_provider,
        processing_time_ms = start_time.elapsed().as_millis(),
        "List providers request completed"
    );

    // Record metrics
    state.metrics.record_http_request(
        "GET",
        "/api/v1/llm/providers",
        200,
        start_time.elapsed().as_secs_f64(),
    );

    Ok((StatusCode::OK, Json(response)))
}

/// Switch the active LLM provider
///
/// This endpoint allows switching between configured LLM providers at runtime,
/// with optional gradual rollout for production safety.
///
/// ## Request Body
/// - `provider_name`: Name of the provider to switch to
/// - `config_updates`: Optional configuration updates for the provider
/// - `gradual_rollout`: Whether to perform gradual rollout (default: false)
/// - `rollout_percentage`: Percentage for gradual rollout (default: 100)
pub async fn switch_provider(
    State(state): State<AppState>,
    Json(request): Json<SwitchProviderRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();

    info!(
        provider_name = %request.provider_name,
        gradual_rollout = request.gradual_rollout,
        rollout_percentage = request.rollout_percentage,
        "Received switch provider request"
    );

    // Validate provider exists
    let registry = get_llm_registry();
    let registry_guard = registry.lock().await;

    if !registry_guard
        .list_providers()
        .contains(&request.provider_name)
    {
        return Err(ApiError::validation(format!(
            "Provider '{}' not found",
            request.provider_name
        )));
    }

    // Validate rollout percentage
    if request.rollout_percentage > 100 {
        return Err(ApiError::validation(
            "Rollout percentage must be between 0 and 100".to_string(),
        ));
    }

    // Apply configuration updates if provided
    if let Some(config_updates) = &request.config_updates {
        info!(
            provider_name = %request.provider_name,
            config_keys = ?config_updates.keys().collect::<Vec<_>>(),
            "Applying configuration updates during provider switch"
        );

        // Validate configuration before applying
        if let Err(e) = validate_provider_config(&request.provider_name, config_updates).await {
            return Err(ApiError::validation(format!(
                "Configuration validation failed: {}",
                e
            )));
        }

        // Convert HashMap<String, String> to HashMap<String, serde_json::Value>
        let config_values: HashMap<String, serde_json::Value> = config_updates
            .clone()
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();

        // Create provider config using the builder pattern
        let provider_config =
            ProviderConfig::new(request.provider_name.clone(), request.provider_name.clone())
                .with_config(
                    "config",
                    serde_json::Value::Object(config_values.into_iter().collect()),
                );

        // Apply the configuration by loading the provider
        if let Err(e) = registry_guard.load_provider(provider_config) {
            return Err(ApiError::internal(format!(
                "Failed to apply configuration updates: {}",
                e
            )));
        }

        info!(
            provider_name = %request.provider_name,
            "Configuration updates applied successfully"
        );
    }

    drop(registry_guard);

    // Get current provider
    let current_provider_ref = get_current_provider();
    let mut current_provider_guard = current_provider_ref.lock().await;
    let previous_provider = current_provider_guard.clone();

    // Perform the switch
    let switch_result = if request.gradual_rollout {
        // Use runtime switch manager for gradual rollout
        let switch_manager = get_runtime_switch_manager();

        // Drop the current_provider_guard before acquiring manager_guard to avoid holding multiple locks
        drop(current_provider_guard);

        let manager_guard = switch_manager.lock().await;
        match manager_guard
            .switch_to_provider(request.provider_name.clone())
            .await
        {
            Ok(_) => {
                // Re-acquire the current_provider_guard to update the current provider
                let mut current_provider_guard = current_provider_ref.lock().await;
                *current_provider_guard = Some(request.provider_name.clone());
                true
            }
            Err(e) => {
                warn!("Gradual rollout switch failed: {}", e);
                false
            }
        }
    } else {
        // Direct switch
        *current_provider_guard = Some(request.provider_name.clone());
        drop(current_provider_guard);
        true
    };

    let response = if switch_result {
        SwitchProviderResponse {
            success: true,
            previous_provider,
            new_provider: request.provider_name.clone(),
            switched_at: chrono::Utc::now().to_rfc3339(),
            message: if request.gradual_rollout {
                format!(
                    "Gradual rollout initiated to {} ({}%)",
                    request.provider_name, request.rollout_percentage
                )
            } else {
                format!("Switched to provider '{}'", request.provider_name)
            },
        }
    } else {
        SwitchProviderResponse {
            success: false,
            previous_provider,
            new_provider: request.provider_name.clone(),
            switched_at: chrono::Utc::now().to_rfc3339(),
            message: "Provider switch failed".to_string(),
        }
    };

    let status_code = if response.success {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    info!(
        success = response.success,
        previous_provider = ?response.previous_provider,
        new_provider = %response.new_provider,
        processing_time_ms = start_time.elapsed().as_millis(),
        "Switch provider request completed"
    );

    // Record metrics
    state.metrics.record_http_request(
        "POST",
        "/api/v1/llm/providers/switch",
        status_code.as_u16(),
        start_time.elapsed().as_secs_f64(),
    );

    Ok((status_code, Json(response)))
}

/// Update LLM configuration
///
/// This endpoint allows updating configuration for LLM providers,
/// including API keys, endpoints, and other provider-specific settings.
///
/// ## Request Body
/// - `provider_configs`: Provider-specific configuration updates
/// - `global_config`: Global configuration settings
/// - `validate`: Whether to validate configuration before applying
pub async fn update_config(
    State(state): State<AppState>,
    Json(request): Json<ConfigUpdateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();

    info!(
        provider_count = request.provider_configs.len(),
        validate = request.validate,
        "Received config update request"
    );

    let mut validation_results = HashMap::new();
    let mut success = true;

    // Validate configurations if requested
    if request.validate {
        for (provider_name, config) in &request.provider_configs {
            match validate_provider_config(provider_name, config).await {
                Ok(msg) => {
                    validation_results.insert(provider_name.clone(), msg);
                }
                Err(e) => {
                    validation_results
                        .insert(provider_name.clone(), format!("Validation failed: {}", e));
                    success = false;
                }
            }
        }
    }

    // Apply configuration updates if validation passed
    if success {
        let registry = get_llm_registry();
        let registry_guard = registry.lock().await;

        for (provider_name, config) in request.provider_configs {
            // Convert HashMap<String, String> to HashMap<String, serde_json::Value>
            let config_values: HashMap<String, serde_json::Value> = config
                .into_iter()
                .map(|(k, v)| (k, serde_json::Value::String(v)))
                .collect();

            // Create provider config using the builder pattern
            let provider_config = ProviderConfig::new(provider_name.clone(), provider_name.clone())
                .with_config(
                    "config",
                    serde_json::Value::Object(config_values.into_iter().collect()),
                );

            // Load provider using the config (this will register it internally)
            if let Err(e) = registry_guard.load_provider(provider_config) {
                warn!("Failed to update provider {}: {}", provider_name, e);
                validation_results.insert(provider_name, format!("Update failed: {}", e));
                success = false;
            } else if !validation_results.contains_key(&provider_name) {
                validation_results
                    .entry(provider_name)
                    .or_insert_with(|| "Configuration updated successfully".to_string());
            }
        }

        drop(registry_guard);
    }

    // Create config summary
    let registry = get_llm_registry();
    let registry_guard = registry.lock().await;
    let provider_names = registry_guard.list_providers();
    let current_provider_ref = get_current_provider();
    let current_provider = current_provider_ref.lock().await.clone();

    let updated_config = ConfigSummary {
        configured_providers: provider_names.len(),
        active_provider: current_provider,
        global_settings: request.global_config.unwrap_or_default(),
    };

    drop(registry_guard);

    let response = ConfigUpdateResponse {
        success,
        validation_results,
        updated_config,
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    let status_code = if success {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    };

    info!(
        success = response.success,
        configured_providers = response.updated_config.configured_providers,
        processing_time_ms = start_time.elapsed().as_millis(),
        "Config update request completed"
    );

    // Record metrics
    state.metrics.record_http_request(
        "POST",
        "/api/v1/llm/config",
        status_code.as_u16(),
        start_time.elapsed().as_secs_f64(),
    );

    Ok((status_code, Json(response)))
}

/// Get current LLM configuration
///
/// This endpoint returns the current LLM configuration including
/// active provider and configuration summary.
pub async fn get_config(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();

    info!("Received get config request");

    let registry = get_llm_registry();
    let registry_guard = registry.lock().await;
    let provider_names = registry_guard.list_providers();
    let current_provider_ref = get_current_provider();
    let current_provider = current_provider_ref.lock().await.clone();

    let config_summary = ConfigSummary {
        configured_providers: provider_names.len(),
        active_provider: current_provider,
        global_settings: HashMap::new(), // Would load from persistent storage in production
    };

    drop(registry_guard);

    info!(
        configured_providers = config_summary.configured_providers,
        active_provider = ?config_summary.active_provider,
        processing_time_ms = start_time.elapsed().as_millis(),
        "Get config request completed"
    );

    // Record metrics
    state.metrics.record_http_request(
        "GET",
        "/api/v1/llm/config",
        200,
        start_time.elapsed().as_secs_f64(),
    );

    Ok((StatusCode::OK, Json(config_summary)))
}

/// Create provider info from registry data
async fn create_provider_info(
    _registry: &LlmRegistry,
    provider_name: &str,
    params: &ProviderQuery,
) -> ApiResult<ProviderInfo> {
    // Get provider (simplified - in real implementation would use registry methods)
    let provider_type = if provider_name.contains("openai") {
        "openai"
    } else if provider_name.contains("anthropic") {
        "anthropic"
    } else if provider_name.contains("ollama") {
        "ollama"
    } else {
        "unknown"
    };

    // Create capabilities list
    let capabilities = match provider_type {
        "openai" => vec!["text-generation", "embedding", "chat", "function-calling"],
        "anthropic" => vec!["text-generation", "chat"],
        "ollama" => vec!["text-generation", "embedding", "chat"],
        _ => vec!["text-generation"],
    }
    .into_iter()
    .map(|s| s.to_string())
    .collect();

    // Create config requirements
    let config_required = match provider_type {
        "openai" => vec!["api_key", "model"],
        "anthropic" => vec!["api_key", "model"],
        "ollama" => vec!["base_url", "model"],
        _ => vec!["api_key"],
    }
    .into_iter()
    .map(|s| s.to_string())
    .collect();

    // Create cost info if requested
    let cost_info = if params.include_cost {
        match provider_type {
            "openai" => Some(CostInfo {
                input_token_cost: Some(0.001),
                output_token_cost: Some(0.002),
                currency: "USD".to_string(),
            }),
            "anthropic" => Some(CostInfo {
                input_token_cost: Some(0.008),
                output_token_cost: Some(0.024),
                currency: "USD".to_string(),
            }),
            _ => None,
        }
    } else {
        None
    };

    // Create model info if requested
    let models = if params.include_models {
        match provider_type {
            "openai" => vec![
                ModelInfo {
                    name: "gpt-4".to_string(),
                    context_window: Some(8192),
                    max_output_tokens: Some(4096),
                    supports_functions: true,
                },
                ModelInfo {
                    name: "gpt-3.5-turbo".to_string(),
                    context_window: Some(4096),
                    max_output_tokens: Some(4096),
                    supports_functions: true,
                },
            ],
            "anthropic" => vec![ModelInfo {
                name: "claude-3-opus".to_string(),
                context_window: Some(200000),
                max_output_tokens: Some(4096),
                supports_functions: false,
            }],
            _ => vec![],
        }
    } else {
        vec![]
    };

    Ok(ProviderInfo {
        name: provider_name.to_string(),
        provider_type: provider_type.to_string(),
        status: "available".to_string(),
        capabilities,
        config_required,
        available: true, // Would check actual availability in production
        cost_info,
        models,
    })
}

/// Validate provider configuration
async fn validate_provider_config(
    provider_name: &str,
    config: &HashMap<String, String>,
) -> ApiResult<String> {
    // Basic validation - in production would actually test provider connectivity
    match provider_name {
        name if name.contains("openai") => {
            if !config.contains_key("api_key") {
                return Err(ApiError::validation("OpenAI requires api_key".to_string()));
            }
            if !config.contains_key("model") {
                return Err(ApiError::validation(
                    "OpenAI requires model specification".to_string(),
                ));
            }
        }
        name if name.contains("anthropic") => {
            if !config.contains_key("api_key") {
                return Err(ApiError::validation(
                    "Anthropic requires api_key".to_string(),
                ));
            }
        }
        name if name.contains("ollama") => {
            if !config.contains_key("base_url") {
                return Err(ApiError::validation("Ollama requires base_url".to_string()));
            }
        }
        _ => {
            // Unknown provider - basic validation
            if config.is_empty() {
                return Err(ApiError::validation(
                    "Provider configuration cannot be empty".to_string(),
                ));
            }
        }
    }

    Ok("Configuration is valid".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_provider_config() {
        let mut openai_config = HashMap::new();
        openai_config.insert("api_key".to_string(), "sk-test".to_string());
        openai_config.insert("model".to_string(), "gpt-4".to_string());

        let result = validate_provider_config("openai", &openai_config).await;
        assert!(result.is_ok());

        // Test missing api_key
        let mut invalid_config = HashMap::new();
        invalid_config.insert("model".to_string(), "gpt-4".to_string());

        let result = validate_provider_config("openai", &invalid_config).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_default_values() {
        assert_eq!(default_true(), true);
        assert_eq!(default_rollout_percentage(), 100);
    }
}
