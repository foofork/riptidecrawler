//! Basic usage example of the RipTide Intelligence layer

use riptide_intelligence::{
    create_fallback_chain, CircuitBreaker, CircuitBreakerConfig, CompletionRequest,
    IntelligenceClient, LlmProvider, LlmRegistry, Message, MockLlmProvider, ProviderConfig,
    TimeoutWrapper,
};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🚀 RipTide Intelligence Layer Demo");
    println!("===================================\n");

    // 1. Basic Provider Registration
    println!("1. Setting up provider registry...");
    let registry = LlmRegistry::new();

    // Register mock provider factory
    registry.register_factory("mock", |config| {
        let mut provider = MockLlmProvider::with_name(&config.name);

        // Configure based on config values
        if let Some(delay) = config.config.get("delay") {
            if let Some(delay_ms) = delay.as_u64() {
                provider = provider.with_delay(delay_ms);
            }
        }

        Ok(Arc::new(provider) as Arc<dyn LlmProvider>)
    })?;

    // Load providers
    let configs = vec![
        ProviderConfig::new("primary", "mock"),
        ProviderConfig::new("backup", "mock").with_config(
            "delay",
            serde_json::Value::Number(serde_json::Number::from(100u64)),
        ),
    ];

    registry.load_providers(configs)?;
    println!("✅ Providers registered successfully\n");

    // 2. Basic Client Usage
    println!("2. Testing basic completion...");
    let client = IntelligenceClient::new(registry, "primary");

    let request = CompletionRequest::new(
        "mock-gpt-3.5",
        vec![Message::user("Hello, what can you tell me about Rust?")],
    )
    .with_max_tokens(100);

    let response = client.complete(request).await?;
    println!("✅ Response: {}", response.content);
    println!("   Tokens used: {}", response.usage.total_tokens);
    println!("   Model: {}", response.model);
    println!();

    // 3. Timeout Demonstration
    println!("3. Testing timeout wrapper...");
    let slow_provider = Arc::new(MockLlmProvider::with_name("slow").with_delay(3000));
    let timeout_provider = TimeoutWrapper::with_timeout(slow_provider, Duration::from_millis(1000));

    let request =
        CompletionRequest::new("mock-gpt-3.5", vec![Message::user("This should timeout")]);

    match timeout_provider.complete(request).await {
        Ok(_) => println!("❌ Expected timeout but request succeeded"),
        Err(e) => println!("✅ Timeout triggered as expected: {}", e),
    }
    println!();

    // 4. Circuit Breaker Demonstration
    println!("4. Testing circuit breaker...");
    let failing_provider = Arc::new(MockLlmProvider::with_name("failing").fail_after(0));
    let config = CircuitBreakerConfig::strict();
    let circuit_provider = CircuitBreaker::with_config(failing_provider, config);

    let request = CompletionRequest::new("mock-gpt-3.5", vec![Message::user("This will fail")]);

    // Trigger circuit opening
    for i in 0..6 {
        match circuit_provider.complete(request.clone()).await {
            Ok(_) => println!("   Request {} succeeded", i + 1),
            Err(e) => println!("   Request {} failed: {}", i + 1, e),
        }
    }

    let stats = circuit_provider.stats();
    println!("✅ Circuit breaker stats:");
    println!("   State: {:?}", stats.state);
    println!("   Total requests: {}", stats.total_requests);
    println!("   Failed requests: {}", stats.failed_requests);
    println!();

    // 5. Fallback Chain Demonstration
    println!("5. Testing fallback chain...");
    let failing_provider = Arc::new(MockLlmProvider::with_name("primary").fail_after(0));
    let working_provider = Arc::new(MockLlmProvider::with_name("backup"));

    let chain = create_fallback_chain(vec![failing_provider, working_provider]);

    let request = CompletionRequest::new(
        "mock-gpt-3.5",
        vec![Message::user("Test fallback functionality")],
    );

    match chain.complete(request).await {
        Ok(response) => {
            println!("✅ Fallback succeeded: {}", response.content);
            let stats = chain.stats();
            println!("   Providers tried: {}", stats.average_providers_tried);
            println!("   Fallback triggered: {}", stats.fallback_triggers > 0);
        }
        Err(e) => println!("❌ Fallback failed: {}", e),
    }
    println!();

    // 6. Cost Estimation
    println!("6. Testing cost estimation...");
    let provider = Arc::new(MockLlmProvider::new());
    let cost = provider.estimate_cost(1000);
    println!("✅ Estimated cost for 1000 tokens:");
    println!("   Prompt cost: ${:.4}", cost.prompt_cost);
    println!("   Completion cost: ${:.4}", cost.completion_cost);
    println!("   Total cost: ${:.4}", cost.total_cost);
    println!();

    // 7. Embeddings
    println!("7. Testing embeddings...");
    let embeddings = provider
        .embed("This is a test sentence for embeddings")
        .await?;
    println!("✅ Generated embeddings:");
    println!("   Dimensions: {}", embeddings.len());
    println!("   Sample values: {:?}", &embeddings[0..5]);
    println!();

    // 8. Provider Capabilities
    println!("8. Checking provider capabilities...");
    let capabilities = provider.capabilities();
    println!("✅ Provider capabilities:");
    println!("   Name: {}", capabilities.provider_name);
    println!("   Models: {}", capabilities.models.len());
    println!(
        "   Supports embeddings: {}",
        capabilities.supports_embeddings
    );
    println!("   Max context length: {}", capabilities.max_context_length);
    println!();

    println!("🎉 Demo completed successfully!");
    println!("The RipTide Intelligence layer provides:");
    println!("   • Vendor-agnostic LLM abstraction");
    println!("   • 5-second hard timeouts");
    println!("   • Circuit breaker with max 1 repair attempt");
    println!("   • Deterministic fallback chains");
    println!("   • Cost estimation and monitoring");
    println!("   • Comprehensive safety guarantees");

    Ok(())
}
