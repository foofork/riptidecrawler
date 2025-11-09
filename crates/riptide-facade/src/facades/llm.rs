//! LlmFacade - LLM Provider Management
//!
//! This facade provides a clean interface for LLM operations across multiple providers,
//! orchestrating authorization, metrics collection, caching, and event emission.
//!
//! ## Responsibilities
//!
//! - **Execute Prompts**: Run LLM prompts with provider selection
//! - **Stream Completions**: Stream LLM responses for real-time UX
//! - **Token Estimation**: Estimate token usage before execution
//! - **Provider Switching**: Runtime provider configuration
//! - **Metrics Collection**: Track usage, latency, and costs
//! - **Authorization**: Enforce usage quotas and permissions
//!
//! ## Architecture
//!
//! This facade depends ONLY on port traits (no concrete implementations):
//! - `LlmProvider`: LLM execution and streaming
//! - `CacheStorage`: Response caching for efficiency
//! - `EventBus`: Domain event publishing
//! - `AuthorizationPolicy`: Access control enforcement
//! - `MetricsCollector`: Usage and performance tracking
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use riptide_facade::facades::LlmFacade;
//!
//! let facade = LlmFacade::new(
//!     llm_provider,
//!     cache,
//!     event_bus,
//!     authz_policies,
//!     metrics_collector,
//! );
//!
//! // Execute prompt with caching
//! let response = facade.execute_prompt(request, &authz_ctx).await?;
//!
//! // Stream completion for real-time UX
//! let mut stream = facade.stream_completion(request, &authz_ctx).await?;
//! while let Some(chunk) = stream.next().await {
//!     println!("{}", chunk);
//! }
//! ```

use crate::authorization::{AuthorizationContext, AuthorizationPolicy, Resource};
use crate::error::{RiptideError, RiptideResult};
use riptide_types::ports::{CacheStorage, DomainEvent, EventBus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, info, instrument, warn};

/// LLM Provider port trait (defined here since it's domain-specific)
///
/// This trait abstracts LLM execution, allowing different providers
/// (OpenAI, Anthropic, Azure, etc.) to be injected at runtime.
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    /// Execute a prompt and return complete response
    async fn execute(&self, request: &LlmRequest) -> RiptideResult<LlmResponse>;

    /// Stream response chunks for real-time UX
    async fn stream(
        &self,
        request: &LlmRequest,
    ) -> RiptideResult<mpsc::Receiver<RiptideResult<String>>>;

    /// Estimate token count for a prompt
    async fn estimate_tokens(&self, text: &str) -> RiptideResult<usize>;

    /// Check provider availability
    async fn is_available(&self) -> bool;

    /// Get provider name
    fn name(&self) -> &str;

    /// Get provider capabilities
    fn capabilities(&self) -> LlmCapabilities;
}

/// LLM request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    /// Tenant ID for multi-tenancy
    pub tenant_id: String,

    /// User ID for quota tracking
    pub user_id: String,

    /// Prompt text
    pub prompt: String,

    /// Model identifier (e.g., "gpt-4", "claude-3-opus")
    pub model: String,

    /// Temperature (0.0 - 2.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Maximum tokens to generate
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,

    /// System prompt (optional)
    pub system_prompt: Option<String>,

    /// Additional parameters
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
}

fn default_temperature() -> f32 {
    0.7
}

fn default_max_tokens() -> usize {
    1000
}

/// LLM response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    /// Generated text
    pub text: String,

    /// Provider that generated the response
    pub provider: String,

    /// Model used
    pub model: String,

    /// Token usage statistics
    pub usage: TokenUsage,

    /// Response metadata
    pub metadata: HashMap<String, String>,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Prompt tokens
    pub prompt_tokens: usize,

    /// Completion tokens
    pub completion_tokens: usize,

    /// Total tokens
    pub total_tokens: usize,
}

/// LLM provider capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCapabilities {
    /// Supports streaming responses
    pub supports_streaming: bool,

    /// Supports function calling
    pub supports_functions: bool,

    /// Supports embeddings
    pub supports_embeddings: bool,

    /// Maximum context window size
    pub max_context_tokens: usize,

    /// Available models
    pub models: Vec<String>,
}

/// Metrics collector port trait
#[async_trait::async_trait]
pub trait MetricsCollector: Send + Sync {
    /// Record LLM execution metrics
    async fn record_llm_execution(
        &self,
        provider: &str,
        model: &str,
        prompt_tokens: usize,
        completion_tokens: usize,
        latency_ms: u64,
    );

    /// Record cache hit/miss
    async fn record_cache_hit(&self, hit: bool);

    /// Record error
    async fn record_error(&self, error_type: &str);
}

/// LlmFacade orchestrates LLM operations
///
/// This facade implements the application layer's LLM management use cases,
/// coordinating authorization, caching, metrics, and event emission.
pub struct LlmFacade {
    /// LLM provider for execution
    provider: Arc<dyn LlmProvider>,

    /// Cache for response caching
    cache: Arc<dyn CacheStorage>,

    /// Event bus for domain events
    event_bus: Arc<dyn EventBus>,

    /// Authorization policies
    authz_policies: Vec<Box<dyn AuthorizationPolicy>>,

    /// Business metrics collector
    metrics: Arc<dyn MetricsCollector>,

    /// Default cache TTL
    cache_ttl: Duration,
}

impl LlmFacade {
    /// Create new LlmFacade with injected dependencies
    ///
    /// # Arguments
    ///
    /// * `provider` - LLM provider implementation
    /// * `cache` - Cache storage for responses
    /// * `event_bus` - Event bus for domain events
    /// * `authz_policies` - Authorization policies to enforce
    /// * `metrics` - Metrics collector for tracking
    ///
    /// # Returns
    ///
    /// New `LlmFacade` instance ready for use
    pub fn new(
        provider: Arc<dyn LlmProvider>,
        cache: Arc<dyn CacheStorage>,
        event_bus: Arc<dyn EventBus>,
        authz_policies: Vec<Box<dyn AuthorizationPolicy>>,
        metrics: Arc<dyn MetricsCollector>,
    ) -> Self {
        Self {
            provider,
            cache,
            event_bus,
            authz_policies,
            metrics,
            cache_ttl: Duration::from_secs(3600), // 1 hour default
        }
    }

    /// Execute prompt with caching, authorization, and metrics
    ///
    /// This method orchestrates the complete LLM execution workflow:
    /// 1. Authorization check (quota enforcement)
    /// 2. Cache lookup (if cacheable)
    /// 3. Execute prompt via provider
    /// 4. Record metrics
    /// 5. Cache response
    /// 6. Emit domain event
    ///
    /// # Arguments
    ///
    /// * `request` - LLM request parameters
    /// * `authz_ctx` - Authorization context
    ///
    /// # Returns
    ///
    /// * `Ok(LlmResponse)` - Execution successful
    /// * `Err(_)` - Authorization failed or execution error
    #[instrument(skip(self, request, authz_ctx), fields(tenant_id = %request.tenant_id, model = %request.model))]
    pub async fn execute_prompt(
        &self,
        request: LlmRequest,
        authz_ctx: &AuthorizationContext,
    ) -> RiptideResult<LlmResponse> {
        let start_time = std::time::Instant::now();
        debug!("Executing LLM prompt");

        // Step 1: Authorization
        self.authorize(
            authz_ctx,
            &Resource::Custom {
                resource_type: "llm_execution".to_string(),
                resource_id: request.user_id.clone(),
            },
        )?;

        // Verify tenant ID matches
        if request.tenant_id != authz_ctx.tenant_id {
            return Err(RiptideError::PermissionDenied(
                "Request tenant_id does not match authorization context".to_string(),
            ));
        }

        // Check quota permission
        if !authz_ctx.has_permission("execute:llm") {
            return Err(RiptideError::PermissionDenied(
                "execute:llm permission required".to_string(),
            ));
        }

        // Step 2: Check cache
        let cache_key = self.generate_cache_key(&request);
        if let Ok(Some(cached)) = self.cache.get(&cache_key).await {
            debug!("Cache hit for LLM request");
            self.metrics.record_cache_hit(true).await;

            // Deserialize cached response
            if let Ok(response) = serde_json::from_slice::<LlmResponse>(&cached) {
                info!(cached = true, "LLM execution completed (cached)");
                return Ok(response);
            } else {
                warn!("Failed to deserialize cached response");
            }
        }
        self.metrics.record_cache_hit(false).await;

        // Step 3: Execute via provider
        debug!(provider = %self.provider.name(), "Executing via provider");
        let response = match self.provider.execute(&request).await {
            Ok(resp) => resp,
            Err(e) => {
                self.metrics.record_error("llm_execution_failed").await;
                return Err(e);
            }
        };

        let latency_ms = start_time.elapsed().as_millis() as u64;

        // Step 4: Record metrics
        self.metrics
            .record_llm_execution(
                &response.provider,
                &response.model,
                response.usage.prompt_tokens,
                response.usage.completion_tokens,
                latency_ms,
            )
            .await;

        // Step 5: Cache response
        if let Ok(serialized) = serde_json::to_vec(&response) {
            if let Err(e) = self
                .cache
                .set(&cache_key, &serialized, Some(self.cache_ttl))
                .await
            {
                warn!(error = %e, "Failed to cache LLM response");
            }
        }

        // Step 6: Emit domain event
        let event = DomainEvent::new(
            "llm.execution.completed",
            request.user_id.clone(),
            serde_json::json!({
                "tenant_id": request.tenant_id,
                "user_id": request.user_id,
                "model": response.model,
                "provider": response.provider,
                "prompt_tokens": response.usage.prompt_tokens,
                "completion_tokens": response.usage.completion_tokens,
                "latency_ms": latency_ms,
            }),
        );

        if let Err(e) = self.event_bus.publish(event).await {
            warn!(error = %e, "Failed to publish LLM execution event");
        }

        info!(
            provider = %response.provider,
            latency_ms,
            "LLM execution completed"
        );
        Ok(response)
    }

    /// Stream completion with authorization and metrics
    ///
    /// Streams LLM responses chunk-by-chunk for real-time user experience.
    ///
    /// # Arguments
    ///
    /// * `request` - LLM request parameters
    /// * `authz_ctx` - Authorization context
    ///
    /// # Returns
    ///
    /// * `Ok(Receiver<String>)` - Stream of response chunks
    /// * `Err(_)` - Authorization failed or streaming not supported
    #[instrument(skip(self, request, authz_ctx), fields(tenant_id = %request.tenant_id, model = %request.model))]
    pub async fn stream_completion(
        &self,
        request: LlmRequest,
        authz_ctx: &AuthorizationContext,
    ) -> RiptideResult<mpsc::Receiver<RiptideResult<String>>> {
        debug!("Streaming LLM completion");

        // Authorization
        self.authorize(
            authz_ctx,
            &Resource::Custom {
                resource_type: "llm_execution".to_string(),
                resource_id: request.user_id.clone(),
            },
        )?;

        // Verify tenant ID
        if request.tenant_id != authz_ctx.tenant_id {
            return Err(RiptideError::PermissionDenied(
                "Request tenant_id does not match authorization context".to_string(),
            ));
        }

        // Check permission
        if !authz_ctx.has_permission("execute:llm") {
            return Err(RiptideError::PermissionDenied(
                "execute:llm permission required".to_string(),
            ));
        }

        // Check provider supports streaming
        if !self.provider.capabilities().supports_streaming {
            return Err(RiptideError::Config(
                "Provider does not support streaming".to_string(),
            ));
        }

        // Start streaming
        let stream = self.provider.stream(&request).await?;

        info!(provider = %self.provider.name(), "LLM streaming started");
        Ok(stream)
    }

    /// Estimate token usage for a prompt
    ///
    /// Provides token count estimation without executing the prompt.
    ///
    /// # Arguments
    ///
    /// * `text` - Text to estimate tokens for
    /// * `authz_ctx` - Authorization context
    ///
    /// # Returns
    ///
    /// * `Ok(token_count)` - Estimated token count
    /// * `Err(_)` - Authorization failed or estimation error
    #[instrument(skip(self, text, authz_ctx))]
    pub async fn estimate_tokens(
        &self,
        text: &str,
        authz_ctx: &AuthorizationContext,
    ) -> RiptideResult<usize> {
        debug!("Estimating tokens");

        // Authorization (lightweight check)
        self.authorize(
            authz_ctx,
            &Resource::Custom {
                resource_type: "llm_estimation".to_string(),
                resource_id: authz_ctx.user_id.clone(),
            },
        )?;

        // Estimate tokens
        let token_count = self.provider.estimate_tokens(text).await?;

        info!(token_count, "Token estimation completed");
        Ok(token_count)
    }

    /// Check provider availability
    ///
    /// # Returns
    ///
    /// * `true` - Provider is available
    /// * `false` - Provider is unavailable
    pub async fn is_available(&self) -> bool {
        self.provider.is_available().await
    }

    /// Get provider name
    ///
    /// # Returns
    ///
    /// Provider name string
    pub fn provider_name(&self) -> &str {
        self.provider.name()
    }

    /// Get provider capabilities
    ///
    /// # Returns
    ///
    /// Provider capabilities structure
    pub fn capabilities(&self) -> LlmCapabilities {
        self.provider.capabilities()
    }

    // Private helper: Generate cache key for request
    fn generate_cache_key(&self, request: &LlmRequest) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        request.prompt.hash(&mut hasher);
        request.model.hash(&mut hasher);
        request.temperature.to_bits().hash(&mut hasher);
        request.max_tokens.hash(&mut hasher);
        if let Some(ref sys) = request.system_prompt {
            sys.hash(&mut hasher);
        }

        format!("llm:{}:{:x}", request.tenant_id, hasher.finish())
    }

    // Private helper: Run all authorization policies
    fn authorize(&self, ctx: &AuthorizationContext, resource: &Resource) -> RiptideResult<()> {
        for policy in &self.authz_policies {
            policy.authorize(ctx, resource)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authorization::policies::TenantScopingPolicy;
    use riptide_types::ports::InMemoryCache;
    use std::collections::HashSet;
    use tokio::sync::mpsc;

    // Mock implementations for testing

    struct MockLlmProvider;

    #[async_trait::async_trait]
    impl LlmProvider for MockLlmProvider {
        async fn execute(&self, request: &LlmRequest) -> RiptideResult<LlmResponse> {
            Ok(LlmResponse {
                text: format!("Response to: {}", request.prompt),
                provider: "mock".to_string(),
                model: request.model.clone(),
                usage: TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 20,
                    total_tokens: 30,
                },
                metadata: HashMap::new(),
            })
        }

        async fn stream(
            &self,
            _request: &LlmRequest,
        ) -> RiptideResult<mpsc::Receiver<RiptideResult<String>>> {
            let (tx, rx) = mpsc::channel(10);
            tokio::spawn(async move {
                let chunks = vec!["Hello", " ", "world", "!"];
                for chunk in chunks {
                    let _ = tx.send(Ok(chunk.to_string())).await;
                }
            });
            Ok(rx)
        }

        async fn estimate_tokens(&self, text: &str) -> RiptideResult<usize> {
            Ok(text.split_whitespace().count())
        }

        async fn is_available(&self) -> bool {
            true
        }

        fn name(&self) -> &str {
            "mock"
        }

        fn capabilities(&self) -> LlmCapabilities {
            LlmCapabilities {
                supports_streaming: true,
                supports_functions: false,
                supports_embeddings: false,
                max_context_tokens: 4096,
                models: vec!["mock-model".to_string()],
            }
        }
    }

    struct MockEventBus;

    #[async_trait::async_trait]
    impl EventBus for MockEventBus {
        async fn publish(&self, _event: DomainEvent) -> riptide_types::error::Result<()> {
            Ok(())
        }

        async fn subscribe(
            &self,
            _handler: Arc<dyn riptide_types::ports::EventHandler>,
        ) -> riptide_types::error::Result<riptide_types::ports::SubscriptionId> {
            Ok(uuid::Uuid::new_v4().to_string())
        }

        async fn unsubscribe(&self, _subscription_id: &str) -> riptide_types::error::Result<()> {
            Ok(())
        }
    }

    // Mock metrics collector for testing
    struct MockMetrics;
    #[async_trait::async_trait]
    impl MetricsCollector for MockMetrics {
        async fn record_llm_execution(
            &self,
            _provider: &str,
            _model: &str,
            _prompt_tokens: usize,
            _completion_tokens: usize,
            _latency_ms: u64,
        ) {
        }
        async fn record_cache_hit(&self, _hit: bool) {}
        async fn record_error(&self, _error_type: &str) {}
    }

    fn create_test_facade() -> LlmFacade {
        let provider = Arc::new(MockLlmProvider) as Arc<dyn LlmProvider>;
        let cache = Arc::new(InMemoryCache::new()) as Arc<dyn CacheStorage>;
        let event_bus = Arc::new(MockEventBus) as Arc<dyn EventBus>;
        let authz_policies: Vec<Box<dyn AuthorizationPolicy>> =
            vec![Box::new(TenantScopingPolicy::new())];
        let metrics = Arc::new(MockMetrics) as Arc<dyn MetricsCollector>;

        LlmFacade::new(provider, cache, event_bus, authz_policies, metrics)
    }

    fn create_test_request() -> LlmRequest {
        LlmRequest {
            tenant_id: "tenant1".to_string(),
            user_id: "user1".to_string(),
            prompt: "Hello, world!".to_string(),
            model: "mock-model".to_string(),
            temperature: 0.7,
            max_tokens: 100,
            system_prompt: None,
            parameters: HashMap::new(),
        }
    }

    fn create_test_authz_ctx() -> AuthorizationContext {
        AuthorizationContext::new(
            "user1",
            "tenant1",
            vec!["user"],
            HashSet::from(["execute:llm".to_string()]),
        )
    }

    #[tokio::test]
    async fn test_execute_prompt_success() {
        let facade = create_test_facade();
        let request = create_test_request();
        let authz_ctx = create_test_authz_ctx();

        let result = facade.execute_prompt(request, &authz_ctx).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.provider, "mock");
        assert_eq!(response.usage.total_tokens, 30);
    }

    #[tokio::test]
    async fn test_execute_prompt_tenant_mismatch() {
        let facade = create_test_facade();
        let mut request = create_test_request();
        request.tenant_id = "different-tenant".to_string();
        let authz_ctx = create_test_authz_ctx();

        let result = facade.execute_prompt(request, &authz_ctx).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RiptideError::PermissionDenied(_)
        ));
    }

    #[tokio::test]
    async fn test_execute_prompt_no_permission() {
        let facade = create_test_facade();
        let request = create_test_request();
        let authz_ctx = AuthorizationContext::new(
            "user1",
            "tenant1",
            vec!["viewer"],
            HashSet::new(), // No execute:llm permission
        );

        let result = facade.execute_prompt(request, &authz_ctx).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RiptideError::PermissionDenied(_)
        ));
    }

    #[tokio::test]
    async fn test_execute_prompt_caching() {
        let facade = create_test_facade();
        let request = create_test_request();
        let authz_ctx = create_test_authz_ctx();

        // First execution (cache miss)
        let result1 = facade.execute_prompt(request.clone(), &authz_ctx).await;
        assert!(result1.is_ok());

        // Second execution (cache hit)
        let result2 = facade.execute_prompt(request, &authz_ctx).await;
        assert!(result2.is_ok());

        // Both should have same response
        assert_eq!(result1.unwrap().text, result2.unwrap().text);
    }

    #[tokio::test]
    async fn test_stream_completion_success() {
        let facade = create_test_facade();
        let request = create_test_request();
        let authz_ctx = create_test_authz_ctx();

        let result = facade.stream_completion(request, &authz_ctx).await;
        assert!(result.is_ok());

        let mut rx = result.unwrap();
        let mut chunks = Vec::new();
        while let Some(chunk_result) = rx.recv().await {
            chunks.push(chunk_result.unwrap());
        }

        assert_eq!(chunks.len(), 4);
        assert_eq!(chunks.join(""), "Hello world!");
    }

    #[tokio::test]
    async fn test_estimate_tokens_success() {
        let facade = create_test_facade();
        let authz_ctx = create_test_authz_ctx();

        let result = facade.estimate_tokens("Hello world test", &authz_ctx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3); // 3 words
    }

    #[tokio::test]
    async fn test_is_available() {
        let facade = create_test_facade();
        assert!(facade.is_available().await);
    }

    #[tokio::test]
    async fn test_provider_name() {
        let facade = create_test_facade();
        assert_eq!(facade.provider_name(), "mock");
    }

    #[tokio::test]
    async fn test_capabilities() {
        let facade = create_test_facade();
        let caps = facade.capabilities();
        assert!(caps.supports_streaming);
        assert!(!caps.supports_functions);
        assert_eq!(caps.max_context_tokens, 4096);
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let facade = create_test_facade();
        let request1 = create_test_request();
        let mut request2 = create_test_request();
        request2.prompt = "Different prompt".to_string();

        let key1 = facade.generate_cache_key(&request1);
        let key2 = facade.generate_cache_key(&request2);

        // Different prompts should generate different cache keys
        assert_ne!(key1, key2);

        // Same request should generate same key
        let key3 = facade.generate_cache_key(&request1);
        assert_eq!(key1, key3);
    }
}
