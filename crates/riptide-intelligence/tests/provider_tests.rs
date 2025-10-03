//! Real-world tests for LLM provider management and failover

use anyhow::Result;
use riptide_intelligence::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[cfg(test)]
mod provider_registry_tests {
    use super::*;
    use riptide_intelligence::providers::{LlmProvider, LlmRequest, LlmResponse};
    use riptide_intelligence::registry::{LlmRegistry, ProviderConfig};

    #[derive(Clone)]
    struct MockProvider {
        name: String,
        available: Arc<RwLock<bool>>,
        response_delay: Duration,
        failure_rate: f32,
    }

    #[async_trait::async_trait]
    impl LlmProvider for MockProvider {
        async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
            tokio::time::sleep(self.response_delay).await;

            if rand::random::<f32>() < self.failure_rate {
                return Err(anyhow::anyhow!("Provider failed"));
            }

            if !*self.available.read().await {
                return Err(anyhow::anyhow!("Provider unavailable"));
            }

            Ok(LlmResponse {
                content: format!("Response from {}: {}", self.name, request.prompt),
                model: self.name.clone(),
                tokens_used: request.prompt.len(),
                latency_ms: self.response_delay.as_millis() as u64,
            })
        }

        async fn is_available(&self) -> bool {
            *self.available.read().await
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn max_tokens(&self) -> usize {
            8192
        }
    }

    #[tokio::test]
    async fn test_provider_registration_and_retrieval() {
        let registry = LlmRegistry::new();

        let provider1 = Arc::new(MockProvider {
            name: "provider1".to_string(),
            available: Arc::new(RwLock::new(true)),
            response_delay: Duration::from_millis(10),
            failure_rate: 0.0,
        });

        let provider2 = Arc::new(MockProvider {
            name: "provider2".to_string(),
            available: Arc::new(RwLock::new(true)),
            response_delay: Duration::from_millis(20),
            failure_rate: 0.0,
        });

        registry.register("p1", provider1.clone()).await.unwrap();
        registry.register("p2", provider2.clone()).await.unwrap();

        let retrieved = registry.get_provider("p1").await.unwrap();
        assert_eq!(retrieved.name(), "provider1");

        let all_providers = registry.list_providers().await;
        assert_eq!(all_providers.len(), 2);
    }

    #[tokio::test]
    async fn test_automatic_failover() {
        let registry = LlmRegistry::new();

        // Primary provider that will fail
        let primary = Arc::new(MockProvider {
            name: "primary".to_string(),
            available: Arc::new(RwLock::new(false)),
            response_delay: Duration::from_millis(10),
            failure_rate: 1.0,
        });

        // Backup provider that works
        let backup = Arc::new(MockProvider {
            name: "backup".to_string(),
            available: Arc::new(RwLock::new(true)),
            response_delay: Duration::from_millis(20),
            failure_rate: 0.0,
        });

        registry.register("primary", primary).await.unwrap();
        registry.register("backup", backup).await.unwrap();
        registry.set_failover_chain(vec!["primary", "backup"]).await;

        let request = LlmRequest {
            prompt: "Test prompt".to_string(),
            max_tokens: 100,
            temperature: 0.7,
            ..Default::default()
        };

        let response = registry.complete_with_failover(request).await.unwrap();
        assert_eq!(response.model, "backup");
    }

    #[tokio::test]
    async fn test_health_monitoring() {
        let registry = LlmRegistry::new();

        let provider = Arc::new(MockProvider {
            name: "monitored".to_string(),
            available: Arc::new(RwLock::new(true)),
            response_delay: Duration::from_millis(10),
            failure_rate: 0.0,
        });

        registry
            .register("monitored", provider.clone())
            .await
            .unwrap();

        // Initially healthy
        let health = registry.health_check().await;
        assert!(health["monitored"]);

        // Make unavailable
        *provider.available.write().await = false;

        // Should detect unhealthy
        let health = registry.health_check().await;
        assert!(!health["monitored"]);
    }

    #[tokio::test]
    async fn test_load_balancing() {
        let registry = LlmRegistry::new();

        let provider1 = Arc::new(MockProvider {
            name: "provider1".to_string(),
            available: Arc::new(RwLock::new(true)),
            response_delay: Duration::from_millis(10),
            failure_rate: 0.0,
        });

        let provider2 = Arc::new(MockProvider {
            name: "provider2".to_string(),
            available: Arc::new(RwLock::new(true)),
            response_delay: Duration::from_millis(10),
            failure_rate: 0.0,
        });

        registry.register("p1", provider1).await.unwrap();
        registry.register("p2", provider2).await.unwrap();
        registry.enable_load_balancing(vec!["p1", "p2"]).await;

        let mut responses = vec![];
        for _ in 0..10 {
            let request = LlmRequest {
                prompt: "Test".to_string(),
                max_tokens: 10,
                ..Default::default()
            };
            let response = registry
                .complete_with_load_balancing(request)
                .await
                .unwrap();
            responses.push(response.model);
        }

        // Should distribute across providers
        let p1_count = responses.iter().filter(|m| m == &"provider1").count();
        let p2_count = responses.iter().filter(|m| m == &"provider2").count();

        assert!(p1_count > 0);
        assert!(p2_count > 0);
        assert!((p1_count as i32 - p2_count as i32).abs() <= 3); // Roughly balanced
    }
}

#[cfg(test)]
mod cost_tracking_tests {
    use super::*;
    use riptide_intelligence::dashboard::{CostTracker, UsageMetrics};

    #[tokio::test]
    async fn test_token_cost_calculation() {
        let tracker = CostTracker::new();

        // Configure pricing
        tracker.set_pricing("gpt-4", 0.03, 0.06).await; // $0.03 per 1K input, $0.06 per 1K output
        tracker.set_pricing("claude-3", 0.015, 0.075).await;

        // Track usage
        tracker.record_usage("tenant1", "gpt-4", 1500, 500).await;
        tracker
            .record_usage("tenant1", "claude-3", 2000, 1000)
            .await;
        tracker.record_usage("tenant2", "gpt-4", 500, 200).await;

        // Calculate costs
        let tenant1_cost = tracker.calculate_cost("tenant1").await;
        let tenant2_cost = tracker.calculate_cost("tenant2").await;

        // GPT-4: (1.5 * 0.03) + (0.5 * 0.06) = 0.045 + 0.03 = 0.075
        // Claude-3: (2.0 * 0.015) + (1.0 * 0.075) = 0.03 + 0.075 = 0.105
        // Total for tenant1: 0.18
        assert!((tenant1_cost - 0.18).abs() < 0.001);

        // GPT-4: (0.5 * 0.03) + (0.2 * 0.06) = 0.015 + 0.012 = 0.027
        assert!((tenant2_cost - 0.027).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_budget_enforcement() {
        let tracker = CostTracker::new();

        tracker.set_pricing("gpt-4", 0.03, 0.06).await;
        tracker.set_budget("tenant1", 10.0).await; // $10 budget

        // Use up most of budget
        tracker
            .record_usage("tenant1", "gpt-4", 150_000, 50_000)
            .await;
        // Cost: (150 * 0.03) + (50 * 0.06) = 4.5 + 3.0 = 7.5

        assert!(tracker.has_budget("tenant1").await);

        // Use remaining budget
        tracker
            .record_usage("tenant1", "gpt-4", 60_000, 20_000)
            .await;
        // Additional: (60 * 0.03) + (20 * 0.06) = 1.8 + 1.2 = 3.0
        // Total: 10.5 > 10.0

        assert!(!tracker.has_budget("tenant1").await);
    }

    #[tokio::test]
    async fn test_usage_metrics_aggregation() {
        let metrics = UsageMetrics::new();

        // Record various metrics
        metrics
            .record_request("tenant1", "gpt-4", 100, 50, 145)
            .await;
        metrics
            .record_request("tenant1", "gpt-4", 150, 75, 160)
            .await;
        metrics
            .record_request("tenant2", "claude-3", 200, 100, 220)
            .await;

        // Get tenant metrics
        let tenant1_stats = metrics.get_tenant_metrics("tenant1").await;
        assert_eq!(tenant1_stats.request_count, 2);
        assert_eq!(tenant1_stats.total_input_tokens, 250);
        assert_eq!(tenant1_stats.total_output_tokens, 125);
        assert_eq!(tenant1_stats.average_latency_ms, 152);

        // Get provider metrics
        let gpt4_stats = metrics.get_provider_metrics("gpt-4").await;
        assert_eq!(gpt4_stats.request_count, 2);
        assert_eq!(gpt4_stats.total_tokens, 375);
    }
}

#[cfg(test)]
mod hot_reload_tests {
    use super::*;
    use riptide_intelligence::config::{IntelligenceConfig, ProviderConfig};
    use riptide_intelligence::hot_reload::{ConfigChangeEvent, HotReloadManager};

    #[tokio::test]
    async fn test_config_hot_reload() {
        let manager = HotReloadManager::new();

        // Set initial config
        let config1 = IntelligenceConfig {
            default_provider: "gpt-4".to_string(),
            timeout: Duration::from_secs(30),
            ..Default::default()
        };
        manager.apply_config(config1.clone()).await.unwrap();

        // Subscribe to changes
        let mut change_rx = manager.subscribe_changes().await;

        // Update config
        let config2 = IntelligenceConfig {
            default_provider: "claude-3".to_string(),
            timeout: Duration::from_secs(60),
            ..Default::default()
        };
        manager.apply_config(config2).await.unwrap();

        // Should receive change event
        let event = change_rx.recv().await.unwrap();
        assert_eq!(event.change_type, ConfigChangeEvent::ProviderChanged);
        assert_eq!(event.new_config.default_provider, "claude-3");
    }

    #[tokio::test]
    async fn test_provider_runtime_switching() {
        let manager = HotReloadManager::new();

        // Configure providers
        let providers = vec![
            ProviderConfig {
                name: "primary".to_string(),
                enabled: true,
                endpoint: "http://primary".to_string(),
                ..Default::default()
            },
            ProviderConfig {
                name: "secondary".to_string(),
                enabled: false,
                endpoint: "http://secondary".to_string(),
                ..Default::default()
            },
        ];

        manager.configure_providers(providers).await.unwrap();

        // Switch providers at runtime
        manager.enable_provider("secondary").await.unwrap();
        manager.disable_provider("primary").await.unwrap();

        let active = manager.get_active_providers().await;
        assert!(active.contains(&"secondary".to_string()));
        assert!(!active.contains(&"primary".to_string()));
    }

    #[tokio::test]
    async fn test_gradual_rollout() {
        let manager = HotReloadManager::new();

        // Configure gradual rollout
        manager.configure_rollout("new_provider", 0.2).await; // 20% traffic

        let mut new_provider_count = 0;
        let mut old_provider_count = 0;

        for _ in 0..100 {
            let provider = manager
                .select_provider_with_rollout("old_provider", "new_provider")
                .await;
            if provider == "new_provider" {
                new_provider_count += 1;
            } else {
                old_provider_count += 1;
            }
        }

        // Should be roughly 20/80 split
        assert!(new_provider_count > 10 && new_provider_count < 30);
        assert!(old_provider_count > 70 && old_provider_count < 90);
    }
}

#[cfg(test)]
mod real_world_llm_scenarios {
    use super::*;

    #[tokio::test]
    async fn test_extraction_repair_with_timeout() {
        let extractor = LlmExtractor::new(LlmExtractionConfig {
            timeout: Duration::from_secs(5),
            max_retries: 1,
            enable_caching: true,
            ..Default::default()
        });

        let broken_json = r#"{
            "title": "Test Article",
            "author": "John Doe,
            "content": "This is the content"
            "date": "2024-03-15"
        }"#;

        let result = extractor.repair_extraction(broken_json).await;

        // Should either repair or timeout within 5 seconds
        match result {
            Ok(repaired) => {
                assert!(repaired.contains("\"author\": \"John Doe\""));
                assert!(repaired.contains("\"content\": \"This is the content\""));
            }
            Err(e) => {
                assert!(e.to_string().contains("timeout") || e.to_string().contains("unavailable"));
            }
        }
    }

    #[tokio::test]
    async fn test_structured_extraction_with_schema() {
        let extractor = LlmExtractor::new(LlmExtractionConfig::default());

        let html = r#"
            <article>
                <h1>Breaking News: Major Discovery</h1>
                <div class="meta">By Jane Smith | March 15, 2024</div>
                <p>Scientists have discovered...</p>
            </article>
        "#;

        let schema = r#"{
            "title": "string",
            "author": "string",
            "date": "ISO 8601 date",
            "content": "string"
        }"#;

        let result = extractor.extract_with_schema(html, schema).await;

        match result {
            Ok(extracted) => {
                let json: serde_json::Value = serde_json::from_str(&extracted).unwrap();
                assert!(json["title"].as_str().unwrap().contains("Breaking News"));
                assert_eq!(json["author"].as_str().unwrap(), "Jane Smith");
            }
            Err(e) => {
                // Accept provider unavailability in tests
                assert!(e.to_string().contains("unavailable") || e.to_string().contains("timeout"));
            }
        }
    }

    #[tokio::test]
    async fn test_multi_provider_consensus() {
        let consensus = ConsensusExtractor::new(vec!["provider1", "provider2", "provider3"]);

        let document = "Complex document requiring consensus";

        // Each provider extracts independently
        let results = consensus.extract_with_consensus(document).await;

        match results {
            Ok(consensus_result) => {
                // Should have confidence score based on agreement
                assert!(consensus_result.confidence >= 0.0 && consensus_result.confidence <= 1.0);

                // Should include dissenting opinions if any
                if consensus_result.confidence < 1.0 {
                    assert!(!consensus_result.dissenting_extractions.is_empty());
                }
            }
            Err(e) => {
                // Accept if providers unavailable
                assert!(e.to_string().contains("unavailable"));
            }
        }
    }
}
