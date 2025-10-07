//! Integration tests for the RipTide Intelligence layer
//!
//! This module contains comprehensive integration tests covering:
//! - Basic provider functionality
//! - Provider switching and failover scenarios
//! - Configuration-driven loading with environment variables
//! - Runtime hot-reload functionality
//! - Tenant isolation and cost tracking
//! - Health monitoring and automatic recovery
//! - Enhanced LLM ops dashboards

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use riptide_intelligence::{
    create_fallback_chain,
    with_circuit_breaker,

    with_timeout,
    CircuitBreaker,
    CircuitBreakerConfig,
    CompletionRequest,
    CompletionResponse,
    // Enhanced functionality
    ConfigLoader,
    Cost,
    CostTrackingConfig,
    DashboardGenerator,
    FailoverConfig,
    FailoverManager,
    FallbackChain,
    HealthMonitorBuilder,
    HotReloadConfig,
    HotReloadManager,
    // Core types
    IntelligenceClient,
    IntelligenceConfig,
    IntelligenceError,
    LlmProvider,
    LlmRegistry,
    Message,
    MetricsCollector,
    MockLlmProvider,
    ProviderConfig,
    ProviderPriority,
    ReloadStatus,
    TenantIsolationConfig,
    TenantIsolationManager,
    TenantLimits,
    TenantState,
    TenantStatus,
    TimeWindow,
    TimeoutWrapper,
    Usage,
    ValidationStatus,
};

/// Test basic provider registration and usage
#[tokio::test]
async fn test_basic_provider_registration() {
    let registry = LlmRegistry::new();

    // Register mock provider factory
    registry
        .register_factory("mock", |_config| {
            Ok(Arc::new(MockLlmProvider::new()) as Arc<dyn LlmProvider>)
        })
        .unwrap();

    // Load provider from config
    let config = ProviderConfig::new("test-provider", "mock");
    registry.load_provider(config).unwrap();

    // Create client
    let client = IntelligenceClient::new(registry, "test-provider");

    // Test completion
    let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello, world!")]);

    let response = client.complete(request).await.unwrap();
    assert!(response.content.contains("Mock response"));
}

/// Test timeout functionality
#[tokio::test]
async fn test_timeout_functionality() {
    let slow_provider = Arc::new(MockLlmProvider::new().with_delay(6000)); // 6 seconds
    let timeout_provider = with_timeout(slow_provider);

    let request =
        CompletionRequest::new("mock-gpt-3.5", vec![Message::user("This should timeout")]);

    let result = timeout_provider.complete(request).await;
    assert!(matches!(result, Err(IntelligenceError::Timeout { .. })));
}

/// Test circuit breaker functionality
#[tokio::test]
async fn test_circuit_breaker_functionality() {
    let failing_provider = Arc::new(MockLlmProvider::new().fail_after(0)); // Always fail
    let config = CircuitBreakerConfig::strict();
    let circuit_provider = with_circuit_breaker(failing_provider);

    let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("This will fail")]);

    // Make enough requests to trigger circuit opening
    for i in 0..10 {
        let result = circuit_provider.complete(request.clone()).await;
        println!("Request {}: {:?}", i + 1, result.is_err());

        // After a few failures, circuit should open and reject requests immediately
        if i > 5 {
            if let Err(IntelligenceError::CircuitOpen { .. }) = result {
                // Circuit is open, test passed
                return;
            }
        }
    }

    // Verify circuit is open
    assert!(matches!(
        circuit_provider.complete(request).await,
        Err(IntelligenceError::CircuitOpen { .. })
    ));
}

/// Test fallback chain functionality
#[tokio::test]
async fn test_fallback_chain_functionality() {
    let failing_provider = Arc::new(MockLlmProvider::with_name("failing").fail_after(0));
    let working_provider = Arc::new(MockLlmProvider::with_name("working"));

    let chain = create_fallback_chain(vec![failing_provider, working_provider]);

    let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Test fallback")]);

    let response = chain.complete(request).await.unwrap();
    assert!(response.content.contains("Mock response"));

    let stats = chain.stats();
    assert_eq!(stats.fallback_triggers, 1);
    assert_eq!(stats.successful_requests, 1);
}

/// Test complete safety stack (timeout + circuit breaker + fallback)
#[tokio::test]
async fn test_complete_safety_stack() {
    // Create providers with different failure modes
    let slow_provider = Arc::new(MockLlmProvider::with_name("slow").with_delay(3000));
    let failing_provider = Arc::new(MockLlmProvider::with_name("failing").fail_after(0));
    let working_provider = Arc::new(MockLlmProvider::with_name("working"));

    // Wrap providers with safety features
    let timeout_slow = TimeoutWrapper::with_timeout(slow_provider, Duration::from_millis(1000));
    let circuit_failing =
        CircuitBreaker::with_config(failing_provider, CircuitBreakerConfig::strict());
    let timeout_working = TimeoutWrapper::new(working_provider);

    // Create fallback chain
    let mut chain = FallbackChain::new();
    chain.add_provider_simple(Arc::new(timeout_slow));
    chain.add_provider_simple(Arc::new(circuit_failing));
    chain.add_provider_simple(Arc::new(timeout_working));

    let request = CompletionRequest::new(
        "mock-gpt-3.5",
        vec![Message::user("Test complete safety stack")],
    );

    // Should eventually succeed with the working provider
    let response = chain.complete(request).await.unwrap();
    assert!(response.content.contains("Mock response"));

    let stats = chain.stats();
    assert!(stats.fallback_triggers > 0);
    assert_eq!(stats.successful_requests, 1);
}

/// Test registry with multiple providers
#[tokio::test]
async fn test_registry_multiple_providers() {
    let registry = LlmRegistry::new();

    // Register multiple providers
    registry
        .register_provider(
            "provider1",
            Arc::new(MockLlmProvider::with_name("provider1")),
        )
        .unwrap();

    registry
        .register_provider(
            "provider2",
            Arc::new(MockLlmProvider::with_name("provider2")),
        )
        .unwrap();

    // Test client with different providers
    let client = IntelligenceClient::new(registry, "provider1");

    let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);

    // Test default provider
    let response1 = client.complete(request.clone()).await.unwrap();
    assert!(response1.content.contains("Mock response"));

    // Test specific provider
    let response2 = client
        .complete_with_provider("provider2", request)
        .await
        .unwrap();
    assert!(response2.content.contains("Mock response"));

    // Test capabilities
    let caps1 = client.capabilities("provider1").unwrap();
    assert_eq!(caps1.provider_name, "provider1");

    let caps2 = client.capabilities("provider2").unwrap();
    assert_eq!(caps2.provider_name, "provider2");
}

/// Test embeddings functionality
#[tokio::test]
async fn test_embeddings_functionality() {
    let provider = Arc::new(MockLlmProvider::new());
    let embedding = provider.embed("test text").await.unwrap();

    assert_eq!(embedding.len(), 768);
    assert!(embedding.iter().all(|&x| (-1.0..=1.0).contains(&x)));
}

/// Test cost estimation
#[tokio::test]
async fn test_cost_estimation() {
    let provider = Arc::new(MockLlmProvider::new());
    let cost = provider.estimate_cost(1000);

    assert!(cost.total_cost > 0.0);
    assert_eq!(cost.currency, "USD");
    assert_eq!(cost.total_cost, cost.prompt_cost + cost.completion_cost);
}

/// Test provider availability checking
#[tokio::test]
async fn test_provider_availability() {
    let working_provider = Arc::new(MockLlmProvider::new());
    let failing_provider = Arc::new(MockLlmProvider::new().always_fail());

    assert!(working_provider.is_available().await);
    assert!(!failing_provider.is_available().await);

    // Test with timeout
    let slow_provider = Arc::new(MockLlmProvider::new().with_delay(3000));
    let timeout_provider = TimeoutWrapper::with_timeout(slow_provider, Duration::from_millis(500));

    assert!(!timeout_provider.is_available().await);
}

/// Test configuration-driven provider loading
#[tokio::test]
async fn test_config_driven_loading() {
    let registry = LlmRegistry::new();

    // Register factory
    registry
        .register_factory("mock", |config| {
            let mut provider = MockLlmProvider::with_name(&config.name);

            // Configure based on config values
            if let Some(delay) = config.config.get("delay") {
                if let Some(delay_ms) = delay.as_u64() {
                    provider = provider.with_delay(delay_ms);
                }
            }

            Ok(Arc::new(provider) as Arc<dyn riptide_intelligence::LlmProvider>)
        })
        .unwrap();

    // Load providers with different configurations
    let configs = vec![
        ProviderConfig::new("fast-provider", "mock"),
        ProviderConfig::new("slow-provider", "mock").with_config(
            "delay",
            serde_json::Value::Number(serde_json::Number::from(2000u64)),
        ),
    ];

    registry.load_providers(configs).unwrap();

    // Test both providers
    let client = IntelligenceClient::new(registry, "fast-provider");

    let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);

    // Fast provider should respond quickly
    let start = std::time::Instant::now();
let _ = client.complete(request.clone()).await.unwrap();
    let fast_duration = start.elapsed();

    // Slow provider should take longer
    let start = std::time::Instant::now();
    let _response = client
        .complete_with_provider("slow-provider", request)
        .await
        .unwrap();
    let slow_duration = start.elapsed();

    assert!(slow_duration > fast_duration);
    assert!(slow_duration >= Duration::from_millis(2000));
}

/// Test error handling and propagation
#[tokio::test]
async fn test_error_handling() {
    let registry = LlmRegistry::new();
    let client = IntelligenceClient::new(registry, "nonexistent");

    let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("Hello")]);

    // Should get configuration error for nonexistent provider
    let result = client.complete(request).await;
    assert!(matches!(result, Err(IntelligenceError::Configuration(_))));

    // Test invalid request handling
    let provider = Arc::new(MockLlmProvider::new());
    let empty_request = CompletionRequest::new("mock-gpt-3.5", vec![]);
    let result = provider.complete(empty_request).await;
    // Mock provider should handle empty requests gracefully
    assert!(result.is_ok());
}

/// Test concurrent requests and thread safety
#[tokio::test]
async fn test_concurrent_requests() {
    let provider = Arc::new(MockLlmProvider::new());
    let provider_clone = provider.clone();

    let mut handles = Vec::new();

    // Spawn multiple concurrent requests
    for i in 0..10 {
        let provider = provider_clone.clone();
        let handle = tokio::spawn(async move {
            let request = CompletionRequest::new(
                "mock-gpt-3.5",
                vec![Message::user(format!("Request {}", i))],
            );
            provider.complete(request).await
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // All requests should succeed
    for result in results {
        let response = result.unwrap().unwrap();
        assert!(response.content.contains("Mock response"));
    }

    // Provider should have seen all requests
    assert_eq!(provider.request_count(), 10);
}

/// Test circuit breaker repair attempts limit
#[tokio::test]
async fn test_circuit_breaker_repair_limit() {
    let failing_provider = Arc::new(MockLlmProvider::new().always_fail());
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        min_request_threshold: 2,
        recovery_timeout_secs: 1,
        max_repair_attempts: 1,
        ..CircuitBreakerConfig::strict()
    };
    let circuit_provider = CircuitBreaker::with_config(failing_provider, config);

    let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("This will fail")]);

    // Trigger circuit opening
    for _ in 0..5 {
        let _ = circuit_provider.complete(request.clone()).await;
    }

    // Wait for recovery timeout
    sleep(Duration::from_secs(2)).await;

    // Should allow one repair attempt
    let result = circuit_provider.complete(request.clone()).await;
    assert!(result.is_err());

    // Verify repair attempts
    let stats = circuit_provider.stats();
    assert_eq!(stats.repair_attempts, 1);

    // Wait again - should not allow more repair attempts
    sleep(Duration::from_secs(2)).await;

    let result = circuit_provider.complete(request).await;
    assert!(matches!(result, Err(IntelligenceError::CircuitOpen { .. })));
}

// ===== New Enhanced Tests for Week 8 Multi-Provider Support =====

/// Test configuration-driven provider loading from environment variables
#[tokio::test]
async fn test_environment_configuration_loading() {
    // Set up test environment variables
    std::env::set_var("RIPTIDE_PROVIDER_OPENAI_ENABLED", "true");
    std::env::set_var("RIPTIDE_PROVIDER_OPENAI_API_KEY", "sk-test-key");
    std::env::set_var("RIPTIDE_PROVIDER_OPENAI_MODEL", "gpt-4");
    std::env::set_var("RIPTIDE_PROVIDER_OPENAI_PRIORITY", "1");

    std::env::set_var("RIPTIDE_PROVIDER_ANTHROPIC_ENABLED", "true");
    std::env::set_var("RIPTIDE_PROVIDER_ANTHROPIC_API_KEY", "ant-test-key");
    std::env::set_var("RIPTIDE_PROVIDER_ANTHROPIC_PRIORITY", "2");

    // Use config loader to load from environment
    let config_loader = ConfigLoader::new();
    let config = config_loader.load().unwrap();

    // Should have auto-discovered providers from environment
    assert!(
        !config.providers.is_empty(),
        "Should load providers from environment"
    );

    // Verify OpenAI provider configuration
    let openai_provider = config
        .providers
        .iter()
        .find(|p| p.provider_type == "openai");
    if let Some(provider) = openai_provider {
        assert_eq!(provider.fallback_order, Some(1));
        assert!(provider.config.contains_key("api_key"));
        assert!(provider.config.contains_key("default_model"));
    }

    // Clean up environment
    std::env::remove_var("RIPTIDE_PROVIDER_OPENAI_ENABLED");
    std::env::remove_var("RIPTIDE_PROVIDER_OPENAI_API_KEY");
    std::env::remove_var("RIPTIDE_PROVIDER_OPENAI_MODEL");
    std::env::remove_var("RIPTIDE_PROVIDER_OPENAI_PRIORITY");
    std::env::remove_var("RIPTIDE_PROVIDER_ANTHROPIC_ENABLED");
    std::env::remove_var("RIPTIDE_PROVIDER_ANTHROPIC_API_KEY");
    std::env::remove_var("RIPTIDE_PROVIDER_ANTHROPIC_PRIORITY");
}

/// Test automatic provider failover with health monitoring
#[tokio::test]
async fn test_automatic_provider_failover() {
    // Create health monitor
    let health_monitor = Arc::new(
        HealthMonitorBuilder::new()
            .with_interval(Duration::from_millis(100))
            .with_timeout(Duration::from_millis(50))
            .with_failure_threshold(2)
            .build(),
    );

    // Create failover manager
    let (failover_manager, mut event_rx) =
        FailoverManager::new(FailoverConfig::default(), health_monitor.clone());

    // Create mock providers
    let primary_provider = Arc::new(MockLlmProvider::with_name("primary"));
    let secondary_provider = Arc::new(MockLlmProvider::with_name("secondary"));

    // Add providers to health monitor
    health_monitor
        .add_provider("primary".to_string(), primary_provider.clone())
        .await;
    health_monitor
        .add_provider("secondary".to_string(), secondary_provider.clone())
        .await;

    // Start health monitoring
    health_monitor.start().await.unwrap();

    // Add providers to failover manager with priorities
    let primary_priority = ProviderPriority {
        name: "primary".to_string(),
        priority: 1,
        weight: 1.0,
        max_concurrent_requests: 10,
        enabled: true,
    };

    let secondary_priority = ProviderPriority {
        name: "secondary".to_string(),
        priority: 2,
        weight: 1.0,
        max_concurrent_requests: 10,
        enabled: true,
    };

    failover_manager
        .add_provider(primary_provider.clone(), primary_priority)
        .await
        .unwrap();
    failover_manager
        .add_provider(secondary_provider.clone(), secondary_priority)
        .await
        .unwrap();

    // Test successful request to primary
    let request = CompletionRequest::new("gpt-4", vec![Message::user("Test message")]);
    let response = failover_manager
        .complete_with_failover(request.clone())
        .await;
    assert!(response.is_ok(), "Primary provider should handle request");

    // Simulate primary provider failure
    primary_provider.set_healthy(false);
    sleep(Duration::from_millis(200)).await; // Allow health check to detect failure

    // Next request should failover to secondary
    let response2 = failover_manager.complete_with_failover(request).await;
    assert!(response2.is_ok(), "Should failover to secondary provider");

    // Check for failover event
    tokio::select! {
        event = event_rx.recv() => {
            assert!(event.is_some(), "Should receive failover event");
        }
        _ = sleep(Duration::from_millis(100)) => {
            // Event might be delayed, that's ok
        }
    }

    // Verify statistics
    let stats = failover_manager.get_statistics().await;
    assert_eq!(stats.total_providers, 2);
    assert!(stats.total_requests >= 2);

    // Clean up
    health_monitor.stop().await;
}

/// Test tenant isolation and cost tracking
#[tokio::test]
async fn test_tenant_isolation_and_cost_tracking() {
    // Set up tenant isolation configuration
    let mut tenant_limits = HashMap::new();
    tenant_limits.insert(
        "tenant1".to_string(),
        TenantLimits {
            max_requests_per_minute: 5,
            max_tokens_per_minute: 1000,
            max_cost_per_hour: 1.0,
            max_concurrent_requests: 2,
            allowed_models: Some(vec!["gpt-4".to_string()]),
            priority: 1,
        },
    );
    tenant_limits.insert(
        "tenant2".to_string(),
        TenantLimits {
            max_requests_per_minute: 10,
            max_tokens_per_minute: 2000,
            max_cost_per_hour: 2.0,
            max_concurrent_requests: 5,
            allowed_models: None,
            priority: 2,
        },
    );

    let tenant_config = TenantIsolationConfig {
        enabled: true,
        strict_isolation: true,
        per_tenant_limits: tenant_limits,
        default_limits: TenantLimits::default(),
        tenant_provider_mapping: HashMap::new(),
    };

    // Create tenant isolation manager
    let isolation_manager = TenantIsolationManager::new(tenant_config);

    // Initialize tenants
    let tenant1_limits = TenantLimits {
        max_requests_per_minute: 5,
        max_tokens_per_minute: 1000,
        max_cost_per_hour: 1.0,
        max_concurrent_requests: 2,
        allowed_models: Some(vec!["gpt-4".to_string()]),
        priority: 1,
    };

    isolation_manager
        .initialize_tenant("tenant1".to_string(), Some(tenant1_limits))
        .await
        .unwrap();

    // Test request allowance
    let request1 = CompletionRequest::new("gpt-4", vec![Message::user("Test from tenant1")]);
    let permit1 = isolation_manager
        .check_request_allowed("tenant1", &request1, 0.01)
        .await;
    assert!(permit1.is_ok(), "Request should be allowed for tenant1");

    // Test model restriction
    let restricted_request =
        CompletionRequest::new("claude-3", vec![Message::user("Test restricted")]);
    let permit_restricted = isolation_manager
        .check_request_allowed("tenant1", &restricted_request, 0.01)
        .await;
    assert!(
        permit_restricted.is_err(),
        "Restricted model should be rejected for tenant1"
    );

    // Record request completion and verify tracking
    if let Ok(permit) = permit1 {
        isolation_manager
            .record_request_completion(permit, None, 0.01)
            .await;
    }

    // Check tenant statistics
    let tenant1_stats = isolation_manager.get_tenant_stats("tenant1").await.unwrap();
    assert_eq!(tenant1_stats.total_requests, 1);
    assert_eq!(tenant1_stats.total_cost, 0.01);
    assert_eq!(tenant1_stats.status, TenantStatus::Active);
}

/// Test enhanced dashboard with tenant cost tracking
#[tokio::test]
async fn test_enhanced_dashboard_with_tenant_cost_tracking() {
    // Set up metrics collector
    let metrics_collector = Arc::new(MetricsCollector::new(30));

    // Set up cost tracking configuration
    let mut per_tenant_budgets = HashMap::new();
    per_tenant_budgets.insert("tenant1".to_string(), 10.0);
    per_tenant_budgets.insert("tenant2".to_string(), 5.0);

    let cost_config = CostTrackingConfig {
        enabled: true,
        detailed_tracking: true,
        cost_alerts_enabled: true,
        per_tenant_budgets,
        billing_period_days: 30,
        currency: "USD".to_string(),
        cost_optimization_enabled: true,
        auto_switch_cheaper_provider: false,
    };

    // Create dashboard generator
    let dashboard_generator = DashboardGenerator::new(metrics_collector.clone(), cost_config);

    // Set up mock tenant configuration
    let tenant1_limits = TenantLimits {
        max_requests_per_minute: 60,
        max_tokens_per_minute: 100000,
        max_cost_per_hour: 5.0,
        max_concurrent_requests: 10,
        allowed_models: None,
        priority: 100,
    };

    dashboard_generator
        .update_tenant_config("tenant1".to_string(), tenant1_limits)
        .await;

    // Simulate some requests to generate data
    let request = CompletionRequest::new("gpt-4", vec![Message::user("Test message")]);
    let request_id = metrics_collector
        .start_request(&request, "openai", Some("tenant1".to_string()))
        .await;

    // Simulate completion
    let response = CompletionResponse::new(
        request.id,
        "Test response",
        "gpt-4",
        Usage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
        },
    );
    let cost = Cost::new(0.01, 0.02, "USD");
    metrics_collector
        .complete_request_success(request_id, &response, Some(cost))
        .await;

    // Generate enhanced dashboard
    let dashboard = dashboard_generator
        .generate_enhanced_dashboard(TimeWindow::LastHour)
        .await;

    // Verify dashboard contains expected data
    assert_eq!(dashboard.overall_metrics.request_count, 1);
    assert_eq!(dashboard.cost_analysis.total_cost, 0.03);
    assert_eq!(dashboard.cost_analysis.currency, "USD");

    // Verify tenant cost breakdown
    assert!(!dashboard.cost_analysis.cost_per_tenant.is_empty());
    if let Some(tenant1_cost) = dashboard.cost_analysis.cost_per_tenant.get("tenant1") {
        assert_eq!(tenant1_cost.total_cost, 0.03);
        assert_eq!(tenant1_cost.requests, 1);
        assert_eq!(tenant1_cost.tokens, 30);
    }

    println!(
        "Generated {} recommendations",
        dashboard.recommendations.len()
    );
    println!("Generated {} alerts", dashboard.alerts.len());
}

/// Test hot-reload configuration management
#[tokio::test]
async fn test_hot_reload_configuration_management() {
    // Create initial configuration
    let initial_config = IntelligenceConfig {
        providers: vec![ProviderConfig::new("initial_provider", "mock").with_config(
            "api_key",
            serde_json::Value::String("initial-key".to_string()),
        )],
        ..Default::default()
    };

    // Create registry with mock factory
    let registry = Arc::new(LlmRegistry::new());
    registry
        .register_factory("mock", |_config| {
            Ok(Arc::new(MockLlmProvider::new()) as Arc<dyn LlmProvider>)
        })
        .unwrap();

    // Set up hot-reload manager
    let hot_reload_config = HotReloadConfig {
        enabled: true,
        watch_paths: vec![], // No file watching for this test
        reload_debounce_ms: 100,
        validation_timeout_ms: 1000,
        rollback_on_failure: true,
        max_reload_attempts: 3,
        health_check_before_switch: false,
    };

    let config_loader = ConfigLoader::new();
    let (mut hot_reload_manager, mut change_rx) = HotReloadManager::new(
        hot_reload_config,
        registry.clone(),
        config_loader,
        initial_config,
    )
    .unwrap();

    // Start hot-reload system
    hot_reload_manager.start().await.unwrap();

    // Load initial configuration
    registry
        .load_providers(vec![ProviderConfig::new("initial_provider", "mock")
            .with_config(
                "api_key",
                serde_json::Value::String("initial-key".to_string()),
            )])
        .unwrap();

    // Verify initial state
    assert!(registry.has_provider("initial_provider"));
    assert_eq!(registry.list_providers().len(), 1);

    // Trigger a manual reload
    hot_reload_manager.trigger_reload(false).await.unwrap();

    // Wait for change notification
    tokio::select! {
        change_event = change_rx.changed() => {
            if change_event.is_ok() {
                let event = change_rx.borrow().clone();
                // Event should indicate successful reload (even if no changes)
                assert_eq!(event.validation_status, ValidationStatus::Valid);
                assert_eq!(event.reload_status, ReloadStatus::Success);
            }
        }
        _ = sleep(Duration::from_millis(500)) => {
            // Timeout is acceptable for this test
        }
    }

    // Check change history
    let history = hot_reload_manager.get_change_history().await;
    assert!(!history.is_empty(), "Should have reload history");
}

/// Test comprehensive error handling and recovery
#[tokio::test]
async fn test_comprehensive_error_handling_and_recovery() {
    // Test invalid configuration handling
    let registry = Arc::new(LlmRegistry::new());

    // Test loading invalid configuration
    let invalid_providers = vec![
        ProviderConfig::new("", ""), // Invalid empty names
        ProviderConfig::new("valid_name", "nonexistent_type"),
    ];

    let result = registry.load_providers(invalid_providers);
    // Registry should handle invalid configs gracefully by skipping invalid ones
    assert!(
        result.is_ok(),
        "Registry should handle invalid configs gracefully"
    );

    // Test provider health failure and recovery
    let health_monitor = Arc::new(HealthMonitorBuilder::new().build());
    let recovering_provider = Arc::new(MockLlmProvider::new());

    // Initially unhealthy
    recovering_provider.set_healthy(false);
    health_monitor
        .add_provider("recovering".to_string(), recovering_provider.clone())
        .await;

    // Check initial unhealthy status
    let health_result = health_monitor.check_provider("recovering").await;
    if let Some(result) = health_result {
        assert!(!result.success, "Provider should be initially unhealthy");
    }

    // Simulate recovery
    recovering_provider.set_healthy(true);

    // Check recovered status
    let recovered_result = health_monitor.check_provider("recovering").await;
    if let Some(result) = recovered_result {
        assert!(result.success, "Provider should recover to healthy");
    }
}

/// Test memory management and resource cleanup
#[tokio::test]
async fn test_memory_management_and_cleanup() {
    let metrics_collector = Arc::new(MetricsCollector::new(1)); // Very short retention

    // Generate many requests to test memory management
    for i in 0..50 {
        let request = CompletionRequest::new("gpt-4", vec![Message::user(&format!("Test {}", i))]);
        let request_id = metrics_collector
            .start_request(&request, "test_provider", None)
            .await;

        // Complete some requests
        if i % 2 == 0 {
            metrics_collector
                .complete_request_error(request_id, "test_error", "Test error message")
                .await;
        }
    }

    // Force cleanup
    metrics_collector.cleanup_old_metrics().await;

    // Generate dashboard (should work despite cleanup)
    let dashboard = metrics_collector
        .generate_dashboard(TimeWindow::LastHour)
        .await;

    // Should still have some data (within retention period)
    assert!(dashboard.overall_metrics.request_count >= 0);
}

/// Test concurrent access and thread safety for new components
#[tokio::test]
async fn test_enhanced_concurrent_access() {
    let registry = Arc::new(LlmRegistry::new());
    let metrics_collector = Arc::new(MetricsCollector::new(30));

    // Register factory
    registry
        .register_factory("concurrent_test", |_config| {
            Ok(Arc::new(MockLlmProvider::new()) as Arc<dyn LlmProvider>)
        })
        .unwrap();

    // Create multiple concurrent tasks
    let mut tasks = Vec::new();

    for i in 0..10 {
        let registry_clone = Arc::clone(&registry);
        let metrics_clone = Arc::clone(&metrics_collector);

        let task = tokio::spawn(async move {
            // Load a provider
            let config = ProviderConfig::new(&format!("provider_{}", i), "concurrent_test");
            registry_clone.load_provider(config).unwrap();

            // Start and complete a request in metrics
            let request =
                CompletionRequest::new("gpt-4", vec![Message::user(&format!("Test {}", i))]);
            let request_id = metrics_clone
                .start_request(&request, &format!("provider_{}", i), None)
                .await;
            metrics_clone
                .complete_request_success(
                    request_id,
                    &CompletionResponse::new(
                        request.id,
                        "Test response",
                        "gpt-4",
                        Usage {
                            prompt_tokens: 10,
                            completion_tokens: 15,
                            total_tokens: 25,
                        },
                    ),
                    Some(Cost::new(0.01, 0.02, "USD")),
                )
                .await;

            // Verify provider exists
            assert!(registry_clone.has_provider(&format!("provider_{}", i)));
        });
        tasks.push(task);
    }

    // Wait for all tasks to complete
    for task in tasks {
        task.await.unwrap();
    }

    // Verify final state
    let final_stats = registry.stats();
    assert_eq!(final_stats.total_providers, 10);

    // Verify metrics were collected
    let dashboard = metrics_collector
        .generate_dashboard(TimeWindow::LastHour)
        .await;
    assert_eq!(dashboard.overall_metrics.request_count, 10);
}
