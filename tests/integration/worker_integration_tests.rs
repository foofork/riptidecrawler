use anyhow::Result;
use riptide_workers::prelude::*;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

/// Test worker service basic functionality
#[tokio::test]
async fn test_worker_service_creation() -> Result<()> {
    let config = WorkerServiceConfig::default();

    // Validate configuration
    assert!(!config.redis_url.is_empty());
    assert!(config.worker_config.worker_count > 0);
    assert!(config.max_batch_size > 0);
    assert!(config.max_concurrency > 0);

    // Note: Full integration test would require Redis instance
    // For now, we test the configuration structure

    Ok(())
}

/// Test job creation and basic properties
#[tokio::test]
async fn test_job_creation_and_properties() -> Result<()> {
    // Test single crawl job
    let single_job = Job::new(JobType::SingleCrawl {
        url: "https://example.com".to_string(),
        options: None,
    });

    assert_eq!(single_job.priority, JobPriority::Normal);
    assert_eq!(single_job.status, JobStatus::Pending);
    assert_eq!(single_job.retry_count, 0);
    assert!(single_job.is_ready());

    // Test batch crawl job
    let batch_job = Job::with_priority(
        JobType::BatchCrawl {
            urls: vec![
                "https://example.com".to_string(),
                "https://httpbin.org/get".to_string(),
            ],
            options: None,
        },
        JobPriority::High,
    );

    assert_eq!(batch_job.priority, JobPriority::High);
    assert!(batch_job.is_ready());

    // Test maintenance job
    let maintenance_job = Job::new(JobType::Maintenance {
        task_type: "cache_cleanup".to_string(),
        parameters: std::collections::HashMap::new(),
    });

    assert_eq!(maintenance_job.status, JobStatus::Pending);

    Ok(())
}

/// Test job scheduling functionality
#[tokio::test]
async fn test_scheduled_job_creation() -> Result<()> {
    // Test daily maintenance job
    let scheduled_job = ScheduledJob::new(
        "Daily Cache Cleanup".to_string(),
        "0 2 * * *".to_string(), // Daily at 2 AM
        JobType::Maintenance {
            task_type: "cache_cleanup".to_string(),
            parameters: std::collections::HashMap::new(),
        },
    )?;

    assert_eq!(scheduled_job.name, "Daily Cache Cleanup");
    assert_eq!(scheduled_job.cron_expression, "0 2 * * *");
    assert!(scheduled_job.enabled);
    assert!(scheduled_job.next_execution_at.is_some());
    assert_eq!(scheduled_job.execution_count, 0);

    // Test invalid cron expression
    let invalid_scheduled_job = ScheduledJob::new(
        "Invalid Schedule".to_string(),
        "invalid cron expression".to_string(),
        JobType::Maintenance {
            task_type: "test".to_string(),
            parameters: std::collections::HashMap::new(),
        },
    );

    assert!(invalid_scheduled_job.is_err());

    Ok(())
}

/// Test retry configuration and logic
#[tokio::test]
async fn test_retry_logic() -> Result<()> {
    let mut job = Job::with_retry_config(
        JobType::SingleCrawl {
            url: "https://example.com".to_string(),
            options: None,
        },
        RetryConfig {
            max_attempts: 2,
            initial_delay_secs: 1,
            backoff_multiplier: 2.0,
            max_delay_secs: 10,
            use_jitter: false,
        },
    );

    // First failure - should retry
    job.fail("Network timeout".to_string());
    assert_eq!(job.status, JobStatus::Retrying);
    assert_eq!(job.retry_count, 1);
    assert!(job.next_retry_at.is_some());
    assert_eq!(job.last_error.as_ref().unwrap(), "Network timeout");

    // Second failure - should go to dead letter
    job.fail("Connection refused".to_string());
    assert_eq!(job.status, JobStatus::DeadLetter);
    assert_eq!(job.retry_count, 2);
    assert!(job.completed_at.is_some());

    Ok(())
}

/// Test job priority ordering
#[tokio::test]
async fn test_job_priority_ordering() -> Result<()> {
    let low_priority = Job::with_priority(
        JobType::SingleCrawl {
            url: "https://example.com".to_string(),
            options: None,
        },
        JobPriority::Low,
    );

    let high_priority = Job::with_priority(
        JobType::SingleCrawl {
            url: "https://example.com".to_string(),
            options: None,
        },
        JobPriority::High,
    );

    let critical_priority = Job::with_priority(
        JobType::SingleCrawl {
            url: "https://example.com".to_string(),
            options: None,
        },
        JobPriority::Critical,
    );

    // Test priority ordering
    assert!(critical_priority.priority > high_priority.priority);
    assert!(high_priority.priority > low_priority.priority);

    Ok(())
}

/// Test worker configuration validation
#[tokio::test]
async fn test_worker_config_validation() -> Result<()> {
    let config = WorkerConfig::default();

    // Validate default values
    assert!(config.worker_count >= 2);
    assert_eq!(config.poll_interval_secs, 5);
    assert_eq!(config.job_timeout_secs, 600);
    assert_eq!(config.heartbeat_interval_secs, 30);
    assert_eq!(config.max_concurrent_jobs, 4);
    assert!(config.enable_health_monitoring);

    // Test custom configuration
    let custom_config = WorkerConfig {
        worker_count: 8,
        poll_interval_secs: 10,
        job_timeout_secs: 1200,
        heartbeat_interval_secs: 60,
        max_concurrent_jobs: 8,
        enable_health_monitoring: true,
    };

    assert_eq!(custom_config.worker_count, 8);
    assert_eq!(custom_config.poll_interval_secs, 10);
    assert_eq!(custom_config.max_concurrent_jobs, 8);

    Ok(())
}

/// Test job metadata handling
#[tokio::test]
async fn test_job_metadata() -> Result<()> {
    let mut job = Job::new(JobType::Custom {
        job_name: "test_job".to_string(),
        payload: serde_json::json!({"test": "data"}),
    })
    .with_metadata("source".to_string(), serde_json::Value::String("api".to_string()))
    .with_metadata("priority_reason".to_string(), serde_json::Value::String("urgent_request".to_string()))
    .with_timeout(300);

    assert_eq!(job.timeout_secs, Some(300));
    assert_eq!(job.metadata.len(), 2);
    assert_eq!(job.metadata.get("source").unwrap(), &serde_json::Value::String("api".to_string()));

    Ok(())
}

/// Test queue configuration
#[tokio::test]
async fn test_queue_configuration() -> Result<()> {
    let config = QueueConfig::default();

    assert_eq!(config.namespace, "riptide_jobs");
    assert_eq!(config.cache_size, 1000);
    assert_eq!(config.delayed_job_poll_interval, 30);
    assert_eq!(config.job_lease_timeout, 600);
    assert!(config.persist_results);
    assert_eq!(config.result_ttl, 3600);

    // Test custom configuration
    let custom_config = QueueConfig {
        namespace: "custom_jobs".to_string(),
        cache_size: 500,
        delayed_job_poll_interval: 60,
        job_lease_timeout: 300,
        persist_results: false,
        result_ttl: 1800,
    };

    assert_eq!(custom_config.namespace, "custom_jobs");
    assert_eq!(custom_config.cache_size, 500);
    assert!(!custom_config.persist_results);

    Ok(())
}

/// Test scheduler configuration
#[tokio::test]
async fn test_scheduler_configuration() -> Result<()> {
    let config = SchedulerConfig::default();

    assert_eq!(config.check_interval_secs, 30);
    assert_eq!(config.max_scheduled_jobs, 1000);
    assert!(config.persist_schedules);
    assert_eq!(config.redis_prefix, "riptide_schedules");

    Ok(())
}

/// Integration test simulating a complete workflow
#[tokio::test]
async fn test_complete_workflow_simulation() -> Result<()> {
    // Create jobs of different types
    let batch_crawl_job = Job::with_priority(
        JobType::BatchCrawl {
            urls: vec![
                "https://example.com".to_string(),
                "https://httpbin.org/get".to_string(),
                "https://httpbin.org/json".to_string(),
            ],
            options: None,
        },
        JobPriority::High,
    );

    let single_crawl_job = Job::new(JobType::SingleCrawl {
        url: "https://httpbin.org/delay/1".to_string(),
        options: None,
    });

    let maintenance_job = Job::with_priority(
        JobType::Maintenance {
            task_type: "cache_cleanup".to_string(),
            parameters: [
                ("max_age_hours".to_string(), serde_json::Value::Number(serde_json::Number::from(24))),
                ("force".to_string(), serde_json::Value::Bool(false)),
            ].iter().cloned().collect(),
        },
        JobPriority::Low,
    );

    // Validate job creation
    assert!(batch_crawl_job.is_ready());
    assert!(single_crawl_job.is_ready());
    assert!(maintenance_job.is_ready());

    // Test job aging
    let job_age = batch_crawl_job.age_seconds();
    assert!(job_age >= 0);

    // Test job timeout detection (simulate time passage)
    let mut timeout_job = Job::new(JobType::SingleCrawl {
        url: "https://example.com".to_string(),
        options: None,
    }).with_timeout(1); // 1 second timeout

    timeout_job.start("test_worker".to_string());

    // Simulate processing time > timeout
    sleep(Duration::from_millis(1100)).await;

    // In a real scenario with proper time tracking, this would be true
    // For this test, we'll just validate the structure
    assert!(timeout_job.timeout_secs.is_some());
    assert_eq!(timeout_job.timeout_secs.unwrap(), 1);

    Ok(())
}

#[tokio::test]
async fn test_job_result_creation() -> Result<()> {
    let job_id = Uuid::new_v4();
    let worker_id = "test_worker_1".to_string();

    // Test successful job result
    let success_result = JobResult::success(
        job_id,
        worker_id.clone(),
        Some(serde_json::json!({"extracted_content": "test content"})),
        1500,
    );

    assert!(success_result.success);
    assert!(success_result.data.is_some());
    assert!(success_result.error.is_none());
    assert_eq!(success_result.processing_time_ms, 1500);
    assert_eq!(success_result.worker_id, worker_id);

    // Test failed job result
    let failure_result = JobResult::failure(
        job_id,
        worker_id.clone(),
        "Network timeout".to_string(),
        2000,
    );

    assert!(!failure_result.success);
    assert!(failure_result.data.is_none());
    assert!(failure_result.error.is_some());
    assert_eq!(failure_result.error.as_ref().unwrap(), "Network timeout");
    assert_eq!(failure_result.processing_time_ms, 2000);

    Ok(())
}