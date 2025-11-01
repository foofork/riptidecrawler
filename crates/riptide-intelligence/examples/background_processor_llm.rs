//! Example: Background AI Processor with LLM Integration
//!
//! This example demonstrates how to use the Background AI Processor with a full
//! LLM client pool for production-ready AI content enhancement.
//!
//! Run with: cargo run --example background_processor_llm --features mock

#[cfg(feature = "mock")]
use riptide_intelligence::{
    AiProcessorConfig, AiTask, BackgroundAiProcessor, FailoverConfig, FailoverManager,
    FailoverStrategy, HealthMonitorBuilder, LlmRegistry, MockLlmProvider, ProviderConfig,
    ProviderPriority, TaskPriority,
};

#[cfg(feature = "mock")]
use std::sync::Arc;
#[cfg(feature = "mock")]
use std::time::Duration;

#[cfg(feature = "mock")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 Starting Background AI Processor with LLM Integration Example\n");

    // Step 1: Setup LLM Registry with providers
    println!("📋 Step 1: Setting up LLM Registry...");
    let registry = Arc::new(setup_llm_registry()?);
    println!(
        "   ✓ Registry configured with {} providers\n",
        registry.list_providers().len()
    );

    // Step 2: Setup Health Monitoring (optional but recommended)
    println!("🏥 Step 2: Setting up Health Monitoring...");
    let health_monitor = Arc::new(
        HealthMonitorBuilder::new()
            .check_interval(Duration::from_secs(30))
            .timeout(Duration::from_secs(5))
            .build(),
    );
    println!("   ✓ Health monitor configured\n");

    // Step 3: Setup Failover Manager (optional for high availability)
    println!("🔄 Step 3: Setting up Failover Manager...");
    let failover_mgr = setup_failover(health_monitor, registry.clone()).await?;
    println!("   ✓ Failover manager configured with automatic failover\n");

    // Step 4: Create and configure Background AI Processor
    println!("⚙️  Step 4: Configuring Background AI Processor...");
    let config = AiProcessorConfig {
        num_workers: 4,
        queue_size: 100,
        max_concurrent_requests: 10,
        worker_timeout: Duration::from_secs(30),
        stream_results: true,

        // LLM configuration
        llm_model: "gpt-3.5-turbo".to_string(),
        max_tokens: 1024,
        temperature: 0.7,

        // Rate limiting (10 requests/second)
        rate_limit_rps: 10.0,

        // Exponential backoff for retries
        initial_backoff: Duration::from_millis(100),
        max_backoff: Duration::from_secs(10),
        backoff_multiplier: 2.0,
    };

    let mut processor = BackgroundAiProcessor::new(config)
        .with_llm_registry(registry)
        .with_llm_failover(failover_mgr);

    println!("   ✓ Processor configured with {} workers\n", 4);

    // Step 5: Start the processor
    println!("▶️  Step 5: Starting Background AI Processor...");
    processor.start().await?;
    println!("   ✓ Processor started and ready for tasks\n");

    // Step 6: Queue some tasks
    println!("📝 Step 6: Queuing AI enhancement tasks...\n");

    let tasks = vec![
        (
            "https://example.com/article1",
            "Breaking news: AI transforms content processing...",
            TaskPriority::High,
        ),
        (
            "https://example.com/article2",
            "Tutorial: Getting started with Rust async programming...",
            TaskPriority::Normal,
        ),
        (
            "https://example.com/article3",
            "Analysis: The future of web crawling technology...",
            TaskPriority::Critical,
        ),
        (
            "https://example.com/article4",
            "Review: Top 10 developer tools for 2024...",
            TaskPriority::Low,
        ),
    ];

    for (url, content, priority) in tasks {
        let task = AiTask::new(url.to_string(), content.to_string())
            .with_priority(priority)
            .with_timeout(Duration::from_secs(20));

        processor.queue_task(task.clone()).await?;
        println!("   ✓ Queued task: {} (Priority: {:?})", url, priority);
    }

    println!("\n🔄 Step 7: Processing tasks...\n");

    // Step 7: Monitor progress
    let start = std::time::Instant::now();
    let mut results_count = 0;
    let expected_results = 4;

    while results_count < expected_results && start.elapsed() < Duration::from_secs(60) {
        // Check for results
        if let Some(result) = processor.try_recv_result().await {
            results_count += 1;

            if result.success {
                println!(
                    "   ✅ Task {} completed in {}ms",
                    result.task_id, result.processing_time_ms
                );
                println!("      URL: {}", result.url);
                if let Some(content) = &result.enhanced_content {
                    let preview = if content.len() > 100 {
                        format!("{}...", &content[..100])
                    } else {
                        content.clone()
                    };
                    println!("      Enhanced: {}", preview);
                }
            } else {
                println!(
                    "   ❌ Task {} failed: {}",
                    result.task_id,
                    result.error.unwrap_or_else(|| "Unknown error".to_string())
                );
            }
            println!();
        }

        // Show statistics periodically
        if start.elapsed().as_secs() % 5 == 0 {
            let stats = processor.stats().await;
            println!(
                "   📊 Stats: Queue={}, Active={}/{}, Running={}",
                stats.queue_size, stats.active_workers, stats.total_workers, stats.is_running
            );
        }

        // Small delay to avoid busy waiting
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Step 8: Collect any remaining results
    println!("📥 Step 8: Collecting remaining results...");
    let remaining_results = processor.recv_all_results().await;
    if !remaining_results.is_empty() {
        println!(
            "   ✓ Retrieved {} additional results",
            remaining_results.len()
        );
        results_count += remaining_results.len();
    }

    // Step 9: Display final statistics
    println!("\n📊 Step 9: Final Statistics:");
    let stats = processor.stats().await;
    println!(
        "   Total results processed: {}/{}",
        results_count, expected_results
    );
    println!("   Queue size: {}", stats.queue_size);
    println!(
        "   Active workers: {}/{}",
        stats.active_workers, stats.total_workers
    );
    println!("   Total time: {:.2}s", start.elapsed().as_secs_f64());

    // Step 10: Graceful shutdown
    println!("\n🛑 Step 10: Shutting down processor...");
    processor.stop().await?;
    println!("   ✓ Processor stopped gracefully\n");

    println!("✨ Example completed successfully!");

    Ok(())
}

#[cfg(feature = "mock")]
fn setup_llm_registry() -> anyhow::Result<LlmRegistry> {
    let registry = LlmRegistry::new();

    // Register mock provider factory
    registry.register_factory("mock", |_config| Ok(Arc::new(MockLlmProvider::new())))?;

    // Load provider configurations
    let configs = vec![
        ProviderConfig::new("mock-primary", "mock").with_fallback_order(1),
        ProviderConfig::new("mock-backup", "mock").with_fallback_order(2),
    ];

    registry.load_providers(configs)?;

    Ok(registry)
}

#[cfg(feature = "mock")]
async fn setup_failover(
    health_monitor: Arc<riptide_intelligence::HealthMonitor>,
    registry: Arc<LlmRegistry>,
) -> anyhow::Result<Arc<FailoverManager>> {
    let config = FailoverConfig {
        strategy: FailoverStrategy::RoundRobin,
        max_retries: 3,
        retry_delay: Duration::from_millis(500),
        failback_delay: Duration::from_secs(30),
        health_check_threshold: 3,
        circuit_breaker_enabled: true,
        load_balancing_enabled: true,
    };

    let (failover_mgr, _events) = FailoverManager::new(config, health_monitor);

    // Add providers from registry to failover manager
    for provider_name in registry.list_providers() {
        if let Some(provider) = registry.get_provider(&provider_name) {
            let priority = ProviderPriority {
                name: provider_name.clone(),
                priority: if provider_name.contains("primary") {
                    1
                } else {
                    2
                },
                weight: 1.0,
                max_concurrent_requests: 10,
                enabled: true,
            };

            failover_mgr.add_provider(provider, priority).await?;
        }
    }

    Ok(Arc::new(failover_mgr))
}

#[cfg(not(feature = "mock"))]
fn main() {
    eprintln!("This example requires the 'mock' feature to be enabled.");
    eprintln!("Run with: cargo run --example background_processor_llm --features mock");
    std::process::exit(1);
}
