//! Integration tests for LLM client pool

#[cfg(test)]
mod tests {
    use riptide_intelligence::{
        AiProcessorConfig, AiTask, BackgroundAiProcessor, LlmClientPool, LlmClientPoolConfig,
        LlmRegistry, TaskPriority,
    };
    use std::sync::Arc;
    use std::time::Duration;

    #[tokio::test]
    async fn test_background_processor_with_client_pool() {
        // Create registry
        let registry = Arc::new(LlmRegistry::new());

        // Create client pool
        let pool_config = LlmClientPoolConfig {
            max_concurrent_requests: 5,
            max_connections_per_provider: 3,
            request_timeout: Duration::from_secs(10),
            enable_circuit_breaker: true,
            ..Default::default()
        };

        let client_pool = Arc::new(LlmClientPool::new(pool_config, registry.clone()));

        // Create background processor
        let processor_config = AiProcessorConfig {
            num_workers: 2,
            max_concurrent_requests: 5,
            queue_size: 100,
            ..Default::default()
        };

        let mut processor = BackgroundAiProcessor::new(processor_config)
            .with_llm_registry(registry)
            .with_llm_client_pool(client_pool);

        // Start processor
        processor.start().await.expect("Failed to start processor");

        // Queue a task
        let task = AiTask::new(
            "https://example.com".to_string(),
            "Test content for enhancement".to_string(),
        )
        .with_priority(TaskPriority::Normal);

        processor
            .queue_task(task)
            .await
            .expect("Failed to queue task");

        // Wait for processing
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Check stats
        let stats = processor.stats().await;
        assert_eq!(stats.queue_size, 0); // Task should be processed

        // Stop processor
        processor.stop().await.expect("Failed to stop processor");
    }

    #[tokio::test]
    async fn test_client_pool_concurrency_control() {
        let registry = Arc::new(LlmRegistry::new());

        let pool_config = LlmClientPoolConfig {
            max_concurrent_requests: 2,
            request_timeout: Duration::from_secs(5),
            ..Default::default()
        };

        let client_pool = Arc::new(LlmClientPool::new(pool_config, registry));
        client_pool.start().await.expect("Failed to start pool");

        let stats = client_pool.stats().await;
        assert_eq!(stats.max_permits, 2);
        assert_eq!(stats.available_permits, 2);

        client_pool.stop().await.expect("Failed to stop pool");
    }

    #[tokio::test]
    async fn test_client_pool_stats_tracking() {
        let registry = Arc::new(LlmRegistry::new());
        let client_pool = Arc::new(LlmClientPool::new(LlmClientPoolConfig::default(), registry));

        let stats = client_pool.stats().await;
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.successful_requests, 0);
        assert_eq!(stats.failed_requests, 0);
        assert_eq!(stats.circuit_breaker_trips, 0);
    }
}
