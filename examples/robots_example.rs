use anyhow::Result;
use riptide_core::fetch::{ReliableHttpClient, RetryConfig, CircuitBreakerConfig};
use riptide_core::robots::RobotsConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create HTTP client with robots.txt compliance
    let robots_config = RobotsConfig {
        respect_robots: true,
        default_crawl_delay: 1.0,
        max_crawl_delay: 5.0,
        default_rps: 2.0,
        max_rps: 10.0,
        cache_ttl: 3600, // 1 hour
        user_agent: "RipTide/1.0".to_string(),
        jitter_factor: 0.2, // ±20% jitter
        development_mode: false,
        ..Default::default()
    };

    let client = ReliableHttpClient::new_with_robots(
        RetryConfig::default(),
        CircuitBreakerConfig::default(),
        robots_config,
    );

    // Example URLs to test
    let urls = vec![
        "https://httpbin.org/robots.txt",
        "https://httpbin.org/html",
        "https://httpbin.org/status/200",
    ];

    for url in urls {
        match client.get_with_retry(url).await {
            Ok(response) => {
                println!("✅ Successfully fetched {}: {}", url, response.status());
            }
            Err(e) => {
                println!("❌ Failed to fetch {}: {}", url, e);
            }
        }
    }

    // Demonstrate manual robots.txt checking
    if let Some(robots_manager) = client.get_robots_manager() {
        let test_urls = vec![
            "https://example.com/allowed",
            "https://example.com/robots.txt",
            "https://google.com/search",
        ];

        for url in test_urls {
            match robots_manager.is_allowed(url).await {
                Ok(allowed) => {
                    println!("🤖 {}: {}", url, if allowed { "ALLOWED" } else { "BLOCKED" });
                }
                Err(e) => {
                    println!("⚠️  Error checking {}: {}", url, e);
                }
            }
        }

        // Show cache stats
        let (robots_cached, rate_limiters) = robots_manager.get_cache_stats();
        println!("📊 Cache stats: {} robots.txt cached, {} rate limiters",
                 robots_cached, rate_limiters);
    }

    Ok(())
}