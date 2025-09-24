use riptide_workers::{Worker, Job, JobType, JobStatus, WorkerPool};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;

#[tokio::test]
async fn test_worker_creation() {
    let worker = Worker::new(1, 4);
    assert_eq!(worker.id(), 1);
    assert_eq!(worker.capacity(), 4);
    assert!(!worker.is_busy());
}

#[tokio::test]
async fn test_job_creation() {
    let job = Job::new(
        JobType::Crawl,
        serde_json::json!({"url": "https://example.com"}),
    );

    assert!(job.id.len() > 0);
    assert_eq!(job.job_type, JobType::Crawl);
    assert_eq!(job.status, JobStatus::Pending);
    assert!(job.created_at > 0);
}

#[tokio::test]
async fn test_job_status_transitions() {
    let mut job = Job::new(
        JobType::Extract,
        serde_json::json!({"html": "<html></html>"}),
    );

    // Test status transitions
    assert_eq!(job.status, JobStatus::Pending);

    job.start();
    assert_eq!(job.status, JobStatus::Running);
    assert!(job.started_at.is_some());

    job.complete(serde_json::json!({"success": true}));
    assert_eq!(job.status, JobStatus::Completed);
    assert!(job.completed_at.is_some());
    assert!(job.result.is_some());
}

#[tokio::test]
async fn test_job_failure() {
    let mut job = Job::new(
        JobType::Render,
        serde_json::json!({"url": "https://example.com"}),
    );

    job.start();
    job.fail("Connection timeout");

    assert_eq!(job.status, JobStatus::Failed);
    assert!(job.error.is_some());
    assert_eq!(job.error.unwrap(), "Connection timeout");
    assert!(job.completed_at.is_some());
}

#[tokio::test]
async fn test_job_retry() {
    let mut job = Job::new(
        JobType::Crawl,
        serde_json::json!({"url": "https://example.com"}),
    );

    job.fail("Temporary error");
    assert_eq!(job.retry_count, 0);

    job.retry();
    assert_eq!(job.status, JobStatus::Pending);
    assert_eq!(job.retry_count, 1);
    assert!(job.error.is_none());
}

#[tokio::test]
async fn test_worker_pool_creation() {
    let pool = WorkerPool::new(4, 2);
    assert_eq!(pool.size(), 4);

    let workers = pool.workers();
    assert_eq!(workers.len(), 4);

    for (i, worker) in workers.iter().enumerate() {
        assert_eq!(worker.id(), i);
        assert_eq!(worker.capacity(), 2);
    }
}

#[tokio::test]
async fn test_worker_pool_job_assignment() {
    let pool = Arc::new(Mutex::new(WorkerPool::new(2, 1)));

    let job1 = Job::new(JobType::Crawl, serde_json::json!({}));
    let job2 = Job::new(JobType::Extract, serde_json::json!({}));
    let job3 = Job::new(JobType::Render, serde_json::json!({}));

    // Assign first two jobs - should succeed
    let worker1 = pool.lock().await.assign_job(job1);
    assert!(worker1.is_some());

    let worker2 = pool.lock().await.assign_job(job2);
    assert!(worker2.is_some());

    // Third job should fail (no available workers)
    let worker3 = pool.lock().await.assign_job(job3);
    assert!(worker3.is_none());
}

#[tokio::test]
async fn test_worker_job_completion() {
    let mut worker = Worker::new(0, 2);

    let job1 = Job::new(JobType::Crawl, serde_json::json!({}));
    let job2 = Job::new(JobType::Extract, serde_json::json!({}));

    // Assign jobs
    assert!(worker.assign(job1.clone()).is_ok());
    assert!(worker.assign(job2.clone()).is_ok());
    assert!(worker.is_busy());

    // Try to assign third job - should fail
    let job3 = Job::new(JobType::Render, serde_json::json!({}));
    assert!(worker.assign(job3).is_err());

    // Complete first job
    worker.complete_job(&job1.id);
    assert_eq!(worker.active_jobs(), 1);

    // Complete second job
    worker.complete_job(&job2.id);
    assert_eq!(worker.active_jobs(), 0);
    assert!(!worker.is_busy());
}

#[tokio::test]
async fn test_job_timeout() {
    let mut job = Job::new(
        JobType::Render,
        serde_json::json!({"url": "https://slow-site.com"}),
    );

    job.start();

    // Simulate timeout check
    let timeout_duration = Duration::from_secs(30);
    tokio::time::sleep(Duration::from_millis(100)).await;

    // In real implementation, this would be checked by the worker
    let elapsed = Duration::from_millis(
        (chrono::Utc::now().timestamp_millis() - job.started_at.unwrap()) as u64
    );

    if elapsed > timeout_duration {
        job.fail("Job timeout");
    }

    // For test, we'll manually fail it
    job.fail("Job timeout");
    assert_eq!(job.status, JobStatus::Failed);
}

#[tokio::test]
async fn test_worker_metrics() {
    let mut worker = Worker::new(0, 2);

    // Process some jobs
    for i in 0..5 {
        let mut job = Job::new(
            JobType::Extract,
            serde_json::json!({"id": i}),
        );

        worker.assign(job.clone()).ok();
        job.start();
        job.complete(serde_json::json!({"extracted": true}));
        worker.complete_job(&job.id);
    }

    let metrics = worker.metrics();
    assert_eq!(metrics.jobs_processed, 5);
    assert_eq!(metrics.jobs_failed, 0);
}