//! Example demonstrating the multi-provider support system
//!
//! This example shows how to:
//! - Configure providers using environment variables
//! - Set up automatic failover with health monitoring
//! - Implement tenant isolation with cost tracking
//! - Generate enhanced dashboards with insights
//! - Use hot-reload for runtime configuration updates

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use riptide_intelligence::{
    CompletionRequest,
    // Configuration system
    ConfigLoader,
    CostTrackingConfig,
    DashboardGenerator,
    // Failover
    FailoverConfig,
    FailoverManager,
    // Health monitoring
    HealthMonitorBuilder,
    HotReloadConfig,
    // Hot reload
    HotReloadManager,
    IntelligenceConfig,
    // Core types
    LlmRegistry,
    Message,
    // Metrics and dashboards
    MetricsCollector,
    // Mock provider for this example
    MockLlmProvider,
    ProviderConfig,
    ProviderPriority,
    TenantIsolationConfig,
    // Tenant isolation
    TenantIsolationManager,
    TenantLimits,
    TimeWindow,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt().init();

    println!("🚀 RipTide Intelligence Multi-Provider Support Demo");
    println!("==================================================");

    // 1. Configuration-Driven Provider Loading
    println!("\n📋 1. Configuration-Driven Provider Loading");
    let config = setup_configuration().await?;
    println!(
        "✅ Loaded configuration with {} providers",
        config.providers.len()
    );

    // 2. Set up provider registry with factories
    println!("\n🏭 2. Setting up Provider Registry");
    let registry = Arc::new(LlmRegistry::new());
    setup_provider_factories(&registry).await?;

    // Load providers from configuration
    registry.load_providers(config.providers.clone())?;
    println!(
        "✅ Registered {} providers",
        registry.list_providers().len()
    );

    // 3. Health Monitoring System
    println!("\n🩺 3. Setting up Health Monitoring");
    let health_monitor = Arc::new(
        HealthMonitorBuilder::new()
            .with_interval(Duration::from_secs(10))
            .with_timeout(Duration::from_secs(5))
            .with_failure_threshold(3)
            .build(),
    );

    // Add providers to health monitor
    for provider_name in registry.list_providers() {
        if let Some(provider) = registry.get_provider(&provider_name) {
            health_monitor
                .add_provider(provider_name.clone(), provider)
                .await;
        }
    }

    health_monitor.start().await?;
    println!("✅ Health monitoring started");

    // 4. Automatic Failover System
    println!("\n🔄 4. Setting up Automatic Failover");
    let failover_config = FailoverConfig {
        max_retries: 3,
        retry_delay: Duration::from_millis(500),
        failback_delay: Duration::from_secs(30),
        health_check_threshold: 2,
        circuit_breaker_enabled: true,
        load_balancing_enabled: true,
        ..Default::default()
    };

    let (failover_manager, mut failover_events) =
        FailoverManager::new(failover_config, health_monitor.clone());

    // Add providers with priorities
    setup_failover_priorities(&failover_manager, &registry).await?;
    println!("✅ Failover system configured");

    // 5. Tenant Isolation System
    println!("\n🏠 5. Setting up Tenant Isolation");
    let tenant_isolation = setup_tenant_isolation().await?;
    println!("✅ Tenant isolation configured");

    // 6. Metrics Collection and Dashboard
    println!("\n📊 6. Setting up Metrics and Dashboard");
    let metrics_collector = Arc::new(MetricsCollector::new(30));
    let dashboard_generator = setup_dashboard_generator(&metrics_collector).await?;
    println!("✅ Dashboard system ready");

    // 7. Hot-Reload System
    println!("\n🔥 7. Setting up Hot-Reload");
    let hot_reload_manager = setup_hot_reload(&registry).await?;
    println!("✅ Hot-reload system ready");

    // 8. Demonstrate the system in action
    println!("\n🎭 8. Demonstrating Multi-Provider System");

    // Initialize tenant
    tenant_isolation
        .initialize_tenant(
            "demo_tenant".to_string(),
            Some(TenantLimits {
                max_requests_per_minute: 60,
                max_tokens_per_minute: 10000,
                max_cost_per_hour: 5.0,
                max_concurrent_requests: 10,
                allowed_models: Some(vec!["gpt-4".to_string(), "claude-3".to_string()]),
                priority: 100,
            }),
        )
        .await?;

    // Make some requests with tenant isolation
    for i in 1..=5 {
        println!("\n  Request {} - Testing tenant isolation", i);

        let request = CompletionRequest::new(
            "gpt-4",
            vec![Message::user(format!(
                "Test request {} from demo tenant",
                i
            ))],
        );

        // Check if request is allowed for tenant
        match tenant_isolation
            .check_request_allowed("demo_tenant", &request, 0.02)
            .await
        {
            Ok(permit) => {
                println!("    ✅ Request approved for demo_tenant");

                // Record the request in metrics
                let request_id = metrics_collector
                    .start_request(&request, "primary", Some("demo_tenant".to_string()))
                    .await;

                // Execute request through failover system
                match failover_manager.complete_with_failover(request).await {
                    Ok(response) => {
                        println!("    ✅ Request completed successfully");

                        // Record success
                        metrics_collector
                            .complete_request_success(
                                request_id,
                                &response,
                                Some(riptide_intelligence::Cost::new(0.01, 0.01, "USD")),
                            )
                            .await;

                        // Record completion for tenant
                        tenant_isolation
                            .record_request_completion(permit, Some(&response), 0.02)
                            .await;
                    }
                    Err(e) => {
                        println!("    ❌ Request failed: {}", e);
                        metrics_collector
                            .complete_request_error(request_id, "failover_error", &e.to_string())
                            .await;
                    }
                }
            }
            Err(e) => {
                println!("    ❌ Request rejected: {}", e);
            }
        }

        sleep(Duration::from_millis(500)).await;
    }

    // 9. Generate Enhanced Dashboard
    println!("\n📈 9. Generating Enhanced Dashboard");
    let dashboard = dashboard_generator
        .generate_enhanced_dashboard(TimeWindow::LastHour)
        .await;

    println!("  Dashboard Summary:");
    println!(
        "    Total Requests: {}",
        dashboard.overall_metrics.request_count
    );
    println!(
        "    Success Rate: {:.1}%",
        dashboard.overall_metrics.success_rate
    );
    println!("    Total Cost: ${:.4}", dashboard.cost_analysis.total_cost);
    println!("    Providers: {}", dashboard.provider_metrics.len());
    println!("    Tenants: {}", dashboard.tenant_metrics.len());
    println!("    Recommendations: {}", dashboard.recommendations.len());
    println!("    Alerts: {}", dashboard.alerts.len());

    // Show tenant cost breakdown
    if let Some(tenant_cost) = dashboard.cost_analysis.cost_per_tenant.get("demo_tenant") {
        println!("\n  Demo Tenant Cost Breakdown:");
        println!("    Total Cost: ${:.4}", tenant_cost.total_cost);
        println!("    Requests: {}", tenant_cost.requests);
        println!("    Tokens: {}", tenant_cost.tokens);
        println!("    Cost per Token: ${:.6}", tenant_cost.cost_per_token);
        println!(
            "    Budget Utilization: {:.1}%",
            tenant_cost.budget_utilization_percent
        );
    }

    // 10. Monitor failover events
    println!("\n🔔 10. Monitoring System Events");
    tokio::select! {
        event = failover_events.recv() => {
            if let Some(event) = event {
                println!("    📢 Failover Event: {:?}", event);
            }
        }
        _ = sleep(Duration::from_secs(1)) => {
            println!("    ℹ️  No failover events in the last second");
        }
    }

    // 11. Test hot-reload
    println!("\n🔄 11. Testing Hot-Reload");
    match hot_reload_manager.trigger_reload(false).await {
        Ok(_) => println!("    ✅ Hot-reload triggered successfully"),
        Err(e) => println!("    ⚠️  Hot-reload error: {}", e),
    }

    // Clean up
    println!("\n🧹 Cleaning up resources");
    health_monitor.stop().await;
    println!("✅ Health monitor stopped");

    println!("\n🎉 Multi-Provider Support Demo Complete!");
    println!("=====================================");

    Ok(())
}

/// Set up configuration from environment variables
async fn setup_configuration() -> Result<IntelligenceConfig, Box<dyn std::error::Error>> {
    // Set up some example environment variables for demo
    std::env::set_var("RIPTIDE_PROVIDER_PRIMARY_ENABLED", "true");
    std::env::set_var("RIPTIDE_PROVIDER_PRIMARY_API_KEY", "demo-key");
    std::env::set_var("RIPTIDE_PROVIDER_PRIMARY_PRIORITY", "1");

    std::env::set_var("RIPTIDE_PROVIDER_SECONDARY_ENABLED", "true");
    std::env::set_var("RIPTIDE_PROVIDER_SECONDARY_API_KEY", "demo-key-2");
    std::env::set_var("RIPTIDE_PROVIDER_SECONDARY_PRIORITY", "2");

    let config_loader = ConfigLoader::new();
    let mut config = config_loader.load()?;

    // Add some demo providers if none were loaded from environment
    if config.providers.is_empty() {
        config.providers = vec![
            ProviderConfig::new("primary", "mock")
                .with_config("api_key", serde_json::Value::String("demo-key".to_string()))
                .with_fallback_order(1),
            ProviderConfig::new("secondary", "mock")
                .with_config(
                    "api_key",
                    serde_json::Value::String("demo-key-2".to_string()),
                )
                .with_fallback_order(2),
            ProviderConfig::new("tertiary", "mock")
                .with_config(
                    "api_key",
                    serde_json::Value::String("demo-key-3".to_string()),
                )
                .with_fallback_order(3),
        ];
    }

    Ok(config)
}

/// Set up provider factories
async fn setup_provider_factories(
    registry: &LlmRegistry,
) -> Result<(), Box<dyn std::error::Error>> {
    // Register mock factory for demonstration
    registry.register_factory("mock", |config| {
        let provider = MockLlmProvider::with_name(&config.name);
        Ok(Arc::new(provider) as Arc<dyn riptide_intelligence::LlmProvider>)
    })?;

    // In a real implementation, you would register actual provider factories:
    // registry.register_factory("openai", create_openai_provider)?;
    // registry.register_factory("anthropic", create_anthropic_provider)?;
    // etc.

    Ok(())
}

/// Set up failover priorities for providers
async fn setup_failover_priorities(
    failover_manager: &FailoverManager,
    registry: &LlmRegistry,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider_names = registry.list_providers();

    for (index, name) in provider_names.iter().enumerate() {
        if let Some(provider) = registry.get_provider(name) {
            let priority = ProviderPriority {
                name: name.clone(),
                priority: (index + 1) as u32,
                weight: 1.0,
                max_concurrent_requests: 10,
                enabled: true,
            };

            failover_manager.add_provider(provider, priority).await?;
        }
    }

    Ok(())
}

/// Set up tenant isolation configuration
async fn setup_tenant_isolation() -> Result<TenantIsolationManager, Box<dyn std::error::Error>> {
    let tenant_config = TenantIsolationConfig {
        enabled: true,
        strict_isolation: true,
        per_tenant_limits: [(
            "demo_tenant".to_string(),
            TenantLimits {
                max_requests_per_minute: 60,
                max_tokens_per_minute: 10000,
                max_cost_per_hour: 5.0,
                max_concurrent_requests: 10,
                allowed_models: Some(vec!["gpt-4".to_string(), "claude-3".to_string()]),
                priority: 100,
            },
        )]
        .iter()
        .cloned()
        .collect(),
        default_limits: TenantLimits::default(),
        tenant_provider_mapping: std::collections::HashMap::new(),
    };

    let manager = TenantIsolationManager::new(tenant_config);
    manager.start_cleanup_task().await;

    Ok(manager)
}

/// Set up dashboard generator with cost tracking
async fn setup_dashboard_generator(
    metrics_collector: &Arc<MetricsCollector>,
) -> Result<DashboardGenerator, Box<dyn std::error::Error>> {
    let cost_config = CostTrackingConfig {
        enabled: true,
        detailed_tracking: true,
        cost_alerts_enabled: true,
        per_tenant_budgets: [("demo_tenant".to_string(), 10.0)]
            .iter()
            .cloned()
            .collect(),
        billing_period_days: 30,
        currency: "USD".to_string(),
        cost_optimization_enabled: true,
        auto_switch_cheaper_provider: false,
    };

    let generator = DashboardGenerator::new(metrics_collector.clone(), cost_config);
    Ok(generator)
}

/// Set up hot-reload system
async fn setup_hot_reload(
    registry: &Arc<LlmRegistry>,
) -> Result<HotReloadManager, Box<dyn std::error::Error>> {
    let hot_reload_config = HotReloadConfig {
        enabled: true,
        watch_paths: vec![], // No file watching in demo
        reload_debounce_ms: 1000,
        validation_timeout_ms: 5000,
        rollback_on_failure: true,
        max_reload_attempts: 3,
        health_check_before_switch: false,
    };

    let config_loader = ConfigLoader::new();
    let initial_config = IntelligenceConfig::default();

    let (mut hot_reload_manager, _change_rx) = HotReloadManager::new(
        hot_reload_config,
        registry.clone(),
        config_loader,
        initial_config,
    )?;

    hot_reload_manager.start().await?;
    Ok(hot_reload_manager)
}
