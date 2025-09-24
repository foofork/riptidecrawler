use riptide_workers::{
    Job, JobType, JobStatus, JobPriority, JobResult,
    WorkerPool, WorkerConfig, JobQueue, QueueConfig,
    WorkerMetrics
};
use chrono::Utc;
use uuid::Uuid;

#[tokio::test]
async fn test_job_creation() {
    let job_type = JobType::SingleCrawl {
        url: "https://example.com".to_string(),
        options: None,
    };
    let job = Job::new(job_type);

    assert!(!job.id.to_string().is_empty());
    assert_eq!(job.priority, JobPriority::Normal);
    assert_eq!(job.status, JobStatus::Pending);
    assert!(job.created_at <= Utc::now());
}

#[tokio::test]
async fn test_job_with_priority() {
    let job_type = JobType::BatchCrawl {
        urls: vec!["https://example.com".to_string(), "https://test.com".to_string()],
        options: None,
    };
    let job = Job::with_priority(job_type, JobPriority::High);

    assert!(!job.id.to_string().is_empty());
    assert_eq!(job.priority, JobPriority::High);
    assert_eq!(job.status, JobStatus::Pending);
    assert!(job.created_at <= Utc::now());
}

#[tokio::test]
async fn test_job_status_transitions() {
    let job_type = JobType::Custom {
        job_name: "test_extract".to_string(),
        payload: serde_json::json!({"html": "<html></html>"}),
    };
    let mut job = Job::new(job_type);

    // Test status transitions
    assert_eq!(job.status, JobStatus::Pending);

    job.start("worker-1".to_string());
    assert_eq!(job.status, JobStatus::Processing);
    assert!(job.started_at.is_some());
    assert_eq!(job.worker_id, Some("worker-1".to_string()));

    job.complete();
    assert_eq!(job.status, JobStatus::Completed);
    assert!(job.completed_at.is_some());
}

#[tokio::test]
async fn test_job_failure() {
    let job_type = JobType::SingleCrawl {
        url: "https://example.com".to_string(),
        options: None,
    };
    let mut job = Job::new(job_type);

    job.start("worker-1".to_string());
    job.fail("Connection timeout".to_string());

    // First failure should set status to Retrying
    assert_eq!(job.status, JobStatus::Retrying);
    assert_eq!(job.retry_count, 1);
    assert!(job.last_error.is_some());
    assert_eq!(job.last_error.unwrap(), "Connection timeout");
    assert!(job.next_retry_at.is_some());
}

#[tokio::test]
async fn test_job_retry_exhaustion() {
    let job_type = JobType::SingleCrawl {
        url: "https://example.com".to_string(),
        options: None,
    };
    let mut job = Job::new(job_type).with_retry_config(
        riptide_workers::RetryConfig {
            max_attempts: 2,
            ..Default::default()
        },
    );

    // First failure - should retry
    job.fail("Temporary error".to_string());
    assert_eq!(job.status, JobStatus::Retrying);
    assert_eq!(job.retry_count, 1);

    // Second failure - should go to dead letter
    job.fail("Another error".to_string());
    assert_eq!(job.status, JobStatus::DeadLetter);
    assert_eq!(job.retry_count, 2);
    assert!(job.completed_at.is_some());
}

#[tokio::test]
async fn test_worker_config_defaults() {
    let config = WorkerConfig::default();

    // Test that default config has reasonable values
    assert!(config.worker_count >= 2);
    assert_eq!(config.poll_interval_secs, 5);
    assert_eq!(config.job_timeout_secs, 600);
    assert_eq!(config.heartbeat_interval_secs, 30);
    assert_eq!(config.max_concurrent_jobs, 4);
    assert!(config.enable_health_monitoring);
}

#[tokio::test]
async fn test_job_ready_status() {
    // Test immediate job
    let job_type = JobType::SingleCrawl {
        url: "https://example.com".to_string(),
        options: None,
    };
    let job = Job::new(job_type);
    assert!(job.is_ready());

    // Test scheduled job in future
    let future_time = Utc::now() + chrono::Duration::hours(1);
    let scheduled_job_type = JobType::BatchCrawl {
        urls: vec!["https://test.com".to_string()],
        options: None,
    };
    let scheduled_job = Job::scheduled(scheduled_job_type, future_time);
    assert!(!scheduled_job.is_ready()); // Should not be ready yet
}

#[tokio::test]
async fn test_job_result_creation() {
    let job_id = Uuid::new_v4();
    let worker_id = "worker-test".to_string();
    let processing_time = 150;

    // Test successful result
    let success_result = JobResult::success(
        job_id,
        worker_id.clone(),
        Some(serde_json::json!({"status": "ok"})),
        processing_time,
    );
    assert!(success_result.success);
    assert!(success_result.data.is_some());
    assert!(success_result.error.is_none());
    assert_eq!(success_result.processing_time_ms, processing_time);

    // Test failure result
    let failure_result = JobResult::failure(
        job_id,
        worker_id,
        "Test error".to_string(),
        processing_time,
    );
    assert!(!failure_result.success);
    assert!(failure_result.data.is_none());
    assert!(failure_result.error.is_some());
    assert_eq!(failure_result.error.unwrap(), "Test error");
}

#[tokio::test]
async fn test_job_timeout_check() {
    let job_type = JobType::SingleCrawl {
        url: "https://slow-site.com".to_string(),
        options: None,
    };
    let mut job = Job::new(job_type).with_timeout(1); // 1 second timeout

    // Job without start time should not be timed out
    assert!(!job.is_timed_out());

    // Start the job
    job.start("worker-test".to_string());

    // Immediately check - should not be timed out yet
    assert!(!job.is_timed_out());

    // Simulate timeout by manually setting an old start time
    job.started_at = Some(Utc::now() - chrono::Duration::seconds(5));
    assert!(job.is_timed_out());
}

#[tokio::test]
async fn test_worker_metrics() {
    let metrics = WorkerMetrics::new();

    // Record some job operations
    for i in 0..5 {
        metrics.record_job_submitted("test_job");
        metrics.record_job_completed("test_job", 100 + i * 10);
    }

    // Record a failure
    metrics.record_job_submitted("test_job");
    metrics.record_job_failed("test_job");

    let snapshot = metrics.get_snapshot().await;
    assert_eq!(snapshot.jobs_submitted, 6);
    assert_eq!(snapshot.jobs_completed, 5);
    assert_eq!(snapshot.jobs_failed, 1);
    assert!(snapshot.success_rate > 80.0);

    // Test jobs per second calculation
    let jps = snapshot.jobs_per_second();
    assert!(jps >= 0.0);
}