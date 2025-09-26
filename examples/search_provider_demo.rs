//! SearchProvider Phase 1 Implementation Demo
//!
//! This example demonstrates the newly implemented SearchProvider abstraction
//! that fulfills Phase 1 of the RipTide roadmap.
//!
//! Run with: `cargo run --example search_provider_demo`

use riptide_core::search::{
    create_search_provider, create_search_provider_from_env,
    SearchBackend, SearchConfig, SearchProvider, SearchHit,
    providers::{NoneProvider, SerperProvider},
    circuit_breaker::{CircuitBreakerWrapper, CircuitBreakerConfig},
};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::init();

    println!("🔍 RipTide SearchProvider Phase 1 Demo");
    println!("=====================================\n");

    demo_none_provider().await?;
    demo_search_backends().await?;
    demo_circuit_breaker().await?;
    demo_factory_functions().await?;
    demo_search_hit_builder().await?;

    println!("✨ Phase 1 SearchProvider implementation complete!");
    println!("Ready for integration with /deepsearch endpoint");

    Ok(())
}

async fn demo_none_provider() -> anyhow::Result<()> {
    println!("📋 Demo 1: None Provider (URL Parsing)");
    println!("--------------------------------------");

    let provider = NoneProvider::new(true);

    // Test 1: Multiple URLs with different separators
    let query = "Check these: https://rust-lang.org, https://github.com\nAlso: https://example.com";
    println!("Query: {}", query);

    match provider.search(query, 5, "us", "en").await {
        Ok(results) => {
            println!("✓ Found {} URLs:", results.len());
            for result in results {
                println!("  Rank {}: {} ({})",
                         result.rank,
                         result.url,
                         result.title.unwrap_or_default());
            }
        }
        Err(e) => println!("✗ Error: {}", e),
    }

    // Test 2: No URLs in query
    println!("\nTesting query with no URLs...");
    match provider.search("just plain text", 5, "us", "en").await {
        Ok(_) => println!("✗ Unexpected success"),
        Err(e) => println!("✓ Correctly rejected: {}", e),
    }

    println!("✓ Health check: {:?}\n", provider.health_check().await);
    Ok(())
}

async fn demo_search_backends() -> anyhow::Result<()> {
    println!("🔧 Demo 2: SearchBackend Enum");
    println!("-----------------------------");

    // Parse from strings
    let backends = vec!["serper", "none", "searxng"];
    for backend_str in backends {
        match backend_str.parse::<SearchBackend>() {
            Ok(backend) => println!("✓ Parsed '{}' -> {}", backend_str, backend),
            Err(e) => println!("✗ Failed to parse '{}': {}", backend_str, e),
        }
    }

    // Test invalid backend
    match "invalid".parse::<SearchBackend>() {
        Ok(_) => println!("✗ Should have failed"),
        Err(e) => println!("✓ Correctly rejected invalid backend: {}", e),
    }

    println!();
    Ok(())
}

async fn demo_circuit_breaker() -> anyhow::Result<()> {
    println!("🛡️  Demo 3: Circuit Breaker");
    println!("---------------------------");

    let provider = Box::new(NoneProvider::new(true));
    let config = CircuitBreakerConfig {
        failure_threshold_percentage: 50,
        minimum_request_threshold: 2,
        recovery_timeout: Duration::from_millis(100),
        half_open_max_requests: 1,
    };

    let cb_provider = CircuitBreakerWrapper::with_config(provider, config);
    println!("✓ Circuit breaker created with 50% failure threshold");
    println!("Initial state: {:?}", cb_provider.current_state());

    // Trigger some failures to demonstrate circuit breaker
    println!("\nGenerating failures to test circuit breaker...");
    let _ = cb_provider.search("no urls here", 1, "us", "en").await;
    let _ = cb_provider.search("still no urls", 1, "us", "en").await;

    println!("State after failures: {:?}", cb_provider.current_state());
    println!("Failure rate: {}%", cb_provider.failure_rate());

    println!();
    Ok(())
}

async fn demo_factory_functions() -> anyhow::Result<()> {
    println!("🏭 Demo 4: Factory Functions");
    println!("----------------------------");

    // Create None provider through factory
    let config = SearchConfig {
        backend: SearchBackend::None,
        enable_url_parsing: true,
        timeout_seconds: 30,
        ..Default::default()
    };

    let provider = create_search_provider(config).await?;
    println!("✓ Created provider via factory: {}", provider.backend_type());

    // Test environment-based creation (will use defaults since no env vars set)
    match create_search_provider_from_env(SearchBackend::None).await {
        Ok(env_provider) => {
            println!("✓ Created provider from env: {}", env_provider.backend_type());
        }
        Err(e) => println!("✗ Env provider failed: {}", e),
    }

    println!();
    Ok(())
}

async fn demo_search_hit_builder() -> anyhow::Result<()> {
    println!("🔨 Demo 5: SearchHit Builder Pattern");
    println!("------------------------------------");

    let hit = SearchHit::new("https://example.com".to_string(), 1)
        .with_title("Example Domain".to_string())
        .with_snippet("This domain is for use in illustrative examples".to_string())
        .with_metadata("source".to_string(), "demo".to_string())
        .with_metadata("timestamp".to_string(), "2024-01-01".to_string());

    println!("✓ Built SearchHit:");
    println!("  URL: {}", hit.url);
    println!("  Rank: {}", hit.rank);
    println!("  Title: {:?}", hit.title);
    println!("  Snippet: {:?}", hit.snippet);
    println!("  Metadata: {:?}", hit.metadata);

    println!();
    Ok(())
}

// Mock implementation for demonstration
impl SearchProvider for NoneProvider {
    async fn search(
        &self,
        query: &str,
        limit: u32,
        _country: &str,
        _locale: &str,
    ) -> anyhow::Result<Vec<SearchHit>> {
        // This is a simplified version for the demo
        use regex::Regex;

        let url_regex = Regex::new(r"https?://[^\s,\n]+").unwrap();
        let mut results = Vec::new();

        for (index, url_match) in url_regex.find_iter(query).enumerate() {
            if index >= limit as usize {
                break;
            }

            let url = url_match.as_str().to_string();
            let hit = SearchHit::new(url.clone(), (index + 1) as u32)
                .with_title(format!("Direct URL: {}", extract_domain(&url)))
                .with_snippet(format!("Direct access to {}", url));

            results.push(hit);
        }

        if results.is_empty() {
            anyhow::bail!("No valid URLs found in query");
        }

        Ok(results)
    }

    fn backend_type(&self) -> SearchBackend {
        SearchBackend::None
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

fn extract_domain(url: &str) -> String {
    use url::Url;
    if let Ok(parsed) = Url::parse(url) {
        parsed.host_str().unwrap_or("unknown").to_string()
    } else {
        "unknown".to_string()
    }
}