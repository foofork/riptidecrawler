#![allow(clippy::all, dead_code, unused)]

//! Comprehensive tests for job management system
//!
//! Test coverage includes:
//! - Job lifecycle (create, start, complete, fail, cancel)
//! - Job queue operations (submit, retrieve, list, filter)
//! - Progress tracking and updates
//! - Job statistics and analytics
//! - Error handling and edge cases
//! - Concurrent job operations

use riptide_cli::job::manager::JobManager;
use riptide_cli::job::types::{Job, JobId, JobPriority, JobStatus, LogLevel};
use tempfile::TempDir;

/// Helper to create a test job manager with temp storage
async fn create_test_manager() -> JobManager {
    // Set temp directory for testing
    let temp_dir = TempDir::new().unwrap();
    std::env::set_var("HOME", temp_dir.path());
    JobManager::new().unwrap()
}

#[tokio::test]
async fn test_job_manager_creation() {
    let manager = create_test_manager().await;
    let stats = manager.get_stats().await.unwrap();
    assert_eq!(stats.total_jobs, 0);
}

#[tokio::test]
async fn test_submit_single_job() {
    let manager = create_test_manager().await;

    let job_id = manager
        .submit_job(
            vec!["https://example.com".to_string()],
            "auto".to_string(),
            Some("Test Job".to_string()),
            JobPriority::Medium,
            vec!["test".to_string()],
            false,
        )
        .await
        .unwrap();

    assert!(!job_id.as_str().is_empty());

    let job = manager.get_job(&job_id).await.unwrap();
    assert_eq!(job.status, JobStatus::Pending);
    assert_eq!(job.name, Some("Test Job".to_string()));
    assert_eq!(job.urls.len(), 1);
}

#[tokio::test]
async fn test_submit_multiple_jobs() {
    let manager = create_test_manager().await;

    let mut job_ids = Vec::new();
    for i in 0..5 {
        let job_id = manager
            .submit_job(
                vec![format!("https://example.com/{}", i)],
                "auto".to_string(),
                Some(format!("Job {}", i)),
                JobPriority::Medium,
                vec![],
                false,
            )
            .await
            .unwrap();
        job_ids.push(job_id);
    }

    assert_eq!(job_ids.len(), 5);

    let stats = manager.get_stats().await.unwrap();
    assert_eq!(stats.total_jobs, 5);
}

#[tokio::test]
async fn test_job_lifecycle_complete() {
    let manager = create_test_manager().await;

    // Submit job
    let job_id = manager
        .submit_job(
            vec!["https://example.com".to_string()],
            "auto".to_string(),
            None,
            JobPriority::High,
            vec![],
            false,
        )
        .await
        .unwrap();

    // Start job
    manager.start_job(&job_id).await.unwrap();
    let job = manager.get_job(&job_id).await.unwrap();
    assert_eq!(job.status, JobStatus::Running);
    assert!(job.started_at.is_some());

    // Update progress
    manager
        .update_progress(&job_id, 1, 0, Some("https://example.com".to_string()))
        .await
        .unwrap();
    let job = manager.get_job(&job_id).await.unwrap();
    assert_eq!(job.progress.completed, 1);
    assert_eq!(job.progress.percentage, 100.0);

    // Complete job
    manager.complete_job(&job_id).await.unwrap();
    let job = manager.get_job(&job_id).await.unwrap();
    assert_eq!(job.status, JobStatus::Completed);
    assert!(job.completed_at.is_some());
    assert!(job.duration_secs().is_some());
}

#[tokio::test]
async fn test_job_lifecycle_failed() {
    let manager = create_test_manager().await;

    let job_id = manager
        .submit_job(
            vec!["https://example.com".to_string()],
            "auto".to_string(),
            None,
            JobPriority::Medium,
            vec![],
            false,
        )
        .await
        .unwrap();

    manager.start_job(&job_id).await.unwrap();

    // Fail job with error
    manager
        .fail_job(&job_id, "Network timeout".to_string())
        .await
        .unwrap();

    let job = manager.get_job(&job_id).await.unwrap();
    assert_eq!(job.status, JobStatus::Failed);
    assert_eq!(job.error, Some("Network timeout".to_string()));
    assert!(job.completed_at.is_some());
}

#[tokio::test]
async fn test_job_cancellation() {
    let manager = create_test_manager().await;

    let job_id = manager
        .submit_job(
            vec!["https://example.com".to_string()],
            "auto".to_string(),
            None,
            JobPriority::Low,
            vec![],
            false,
        )
        .await
        .unwrap();

    manager.start_job(&job_id).await.unwrap();

    // Cancel running job
    manager.cancel_job(&job_id).await.unwrap();

    let job = manager.get_job(&job_id).await.unwrap();
    assert_eq!(job.status, JobStatus::Cancelled);
    assert!(job.is_terminal());
}

#[tokio::test]
async fn test_cannot_cancel_completed_job() {
    let manager = create_test_manager().await;

    let job_id = manager
        .submit_job(
            vec!["https://example.com".to_string()],
            "auto".to_string(),
            None,
            JobPriority::Medium,
            vec![],
            false,
        )
        .await
        .unwrap();

    manager.start_job(&job_id).await.unwrap();
    manager.complete_job(&job_id).await.unwrap();

    // Attempt to cancel completed job should fail
    let result = manager.cancel_job(&job_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_jobs_no_filter() {
    let manager = create_test_manager().await;

    for i in 0..3 {
        manager
            .submit_job(
                vec![format!("https://example.com/{}", i)],
                "auto".to_string(),
                None,
                JobPriority::Medium,
                vec![],
                false,
            )
            .await
            .unwrap();
    }

    let jobs = manager.list_jobs(None, None, None, None).await.unwrap();
    assert_eq!(jobs.len(), 3);
}

#[tokio::test]
async fn test_list_jobs_filter_by_status() {
    let manager = create_test_manager().await;

    // Create jobs with different statuses
    let job1 = manager
        .submit_job(
            vec!["https://example.com/1".to_string()],
            "auto".to_string(),
            None,
            JobPriority::Medium,
            vec![],
            false,
        )
        .await
        .unwrap();

    let job2 = manager
        .submit_job(
            vec!["https://example.com/2".to_string()],
            "auto".to_string(),
            None,
            JobPriority::Medium,
            vec![],
            false,
        )
        .await
        .unwrap();

    manager.start_job(&job1).await.unwrap();
    manager.complete_job(&job1).await.unwrap();

    // Filter by completed
    let completed = manager
        .list_jobs(Some(JobStatus::Completed), None, None, None)
        .await
        .unwrap();
    assert_eq!(completed.len(), 1);

    // Filter by pending
    let pending = manager
        .list_jobs(Some(JobStatus::Pending), None, None, None)
        .await
        .unwrap();
    assert_eq!(pending.len(), 1);
}

#[tokio::test]
async fn test_list_jobs_filter_by_priority() {
    let manager = create_test_manager().await;

    manager
        .submit_job(
            vec!["https://example.com/1".to_string()],
            "auto".to_string(),
            None,
            JobPriority::High,
            vec![],
            false,
        )
        .await
        .unwrap();

    manager
        .submit_job(
            vec!["https://example.com/2".to_string()],
            "auto".to_string(),
            None,
            JobPriority::Low,
            vec![],
            false,
        )
        .await
        .unwrap();

    let high_priority = manager
        .list_jobs(None, Some(JobPriority::High), None, None)
        .await
        .unwrap();
    assert_eq!(high_priority.len(), 1);
    assert_eq!(high_priority[0].priority, JobPriority::High);
}

#[tokio::test]
async fn test_list_jobs_filter_by_tag() {
    let manager = create_test_manager().await;

    manager
        .submit_job(
            vec!["https://example.com/1".to_string()],
            "auto".to_string(),
            None,
            JobPriority::Medium,
            vec!["urgent".to_string()],
            false,
        )
        .await
        .unwrap();

    manager
        .submit_job(
            vec!["https://example.com/2".to_string()],
            "auto".to_string(),
            None,
            JobPriority::Medium,
            vec!["normal".to_string()],
            false,
        )
        .await
        .unwrap();

    let urgent_jobs = manager
        .list_jobs(None, None, Some("urgent".to_string()), None)
        .await
        .unwrap();
    assert_eq!(urgent_jobs.len(), 1);
    assert!(urgent_jobs[0].tags.contains(&"urgent".to_string()));
}

#[tokio::test]
async fn test_list_jobs_with_limit() {
    let manager = create_test_manager().await;

    for i in 0..10 {
        manager
            .submit_job(
                vec![format!("https://example.com/{}", i)],
                "auto".to_string(),
                None,
                JobPriority::Medium,
                vec![],
                false,
            )
            .await
            .unwrap();
    }

    let limited = manager.list_jobs(None, None, None, Some(5)).await.unwrap();
    assert_eq!(limited.len(), 5);
}

#[tokio::test]
async fn test_progress_tracking() {
    let manager = create_test_manager().await;

    let job_id = manager
        .submit_job(
            vec![
                "https://example.com/1".to_string(),
                "https://example.com/2".to_string(),
                "https://example.com/3".to_string(),
            ],
            "auto".to_string(),
            None,
            JobPriority::Medium,
            vec![],
            false,
        )
        .await
        .unwrap();

    manager.start_job(&job_id).await.unwrap();

    // Update progress incrementally
    manager.update_progress(&job_id, 1, 0, None).await.unwrap();
    let job = manager.get_job(&job_id).await.unwrap();
    assert_eq!(job.progress.completed, 1);
    assert_eq!(job.progress.percentage, 33.333_336);

    manager.update_progress(&job_id, 2, 1, None).await.unwrap();
    let job = manager.get_job(&job_id).await.unwrap();
    assert_eq!(job.progress.completed, 2);
    assert_eq!(job.progress.failed, 1);
    assert_eq!(job.progress.percentage, 100.0);
}

#[tokio::test]
async fn test_job_logging() {
    let manager = create_test_manager().await;

    let job_id = manager
        .submit_job(
            vec!["https://example.com".to_string()],
            "auto".to_string(),
            None,
            JobPriority::Medium,
            vec![],
            false,
        )
        .await
        .unwrap();

    // Write different log levels
    manager
        .log_job(&job_id, LogLevel::Info, "Job started".to_string())
        .await
        .unwrap();

    manager
        .log_job(&job_id, LogLevel::Debug, "Debug message".to_string())
        .await
        .unwrap();

    manager
        .log_job(&job_id, LogLevel::Warn, "Warning message".to_string())
        .await
        .unwrap();

    // Read all logs
    let logs = manager.read_logs(&job_id, None, None).await.unwrap();
    assert_eq!(logs.len(), 4); // Including submission log

    // Read with level filter
    let warnings = manager
        .read_logs(&job_id, None, Some("warn"))
        .await
        .unwrap();
    assert_eq!(warnings.len(), 1);
}

#[tokio::test]
async fn test_job_logging_with_url() {
    let manager = create_test_manager().await;

    let job_id = manager
        .submit_job(
            vec!["https://example.com".to_string()],
            "auto".to_string(),
            None,
            JobPriority::Medium,
            vec![],
            false,
        )
        .await
        .unwrap();

    manager
        .log_job_url(
            &job_id,
            LogLevel::Info,
            "Processing URL".to_string(),
            "https://example.com".to_string(),
        )
        .await
        .unwrap();

    let logs = manager.read_logs(&job_id, None, None).await.unwrap();
    let url_log = logs.iter().find(|l| l.url.is_some());
    assert!(url_log.is_some());
}

#[tokio::test]
async fn test_save_and_load_results() {
    let manager = create_test_manager().await;

    let job_id = manager
        .submit_job(
            vec!["https://example.com".to_string()],
            "auto".to_string(),
            None,
            JobPriority::Medium,
            vec![],
            false,
        )
        .await
        .unwrap();

    let results = serde_json::json!({
        "extracted": 10,
        "data": ["item1", "item2"]
    });

    manager.save_results(&job_id, &results).await.unwrap();

    let loaded = manager.load_results(&job_id).await.unwrap();
    assert_eq!(loaded, results);
}

#[tokio::test]
async fn test_delete_job() {
    let manager = create_test_manager().await;

    let job_id = manager
        .submit_job(
            vec!["https://example.com".to_string()],
            "auto".to_string(),
            None,
            JobPriority::Medium,
            vec![],
            false,
        )
        .await
        .unwrap();

    manager.delete_job(&job_id).await.unwrap();

    let result = manager.get_job(&job_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_job_statistics() {
    let manager = create_test_manager().await;

    // Create mix of jobs
    let job1 = manager
        .submit_job(
            vec!["https://example.com/1".to_string()],
            "auto".to_string(),
            None,
            JobPriority::High,
            vec![],
            false,
        )
        .await
        .unwrap();

    let job2 = manager
        .submit_job(
            vec!["https://example.com/2".to_string()],
            "auto".to_string(),
            None,
            JobPriority::Low,
            vec![],
            false,
        )
        .await
        .unwrap();

    manager.start_job(&job1).await.unwrap();
    manager.complete_job(&job1).await.unwrap();

    manager.start_job(&job2).await.unwrap();
    manager
        .fail_job(&job2, "Test error".to_string())
        .await
        .unwrap();

    let stats = manager.get_stats().await.unwrap();
    assert_eq!(stats.total_jobs, 2);
    assert_eq!(stats.by_status.get("completed").copied().unwrap_or(0), 1);
    assert_eq!(stats.by_status.get("failed").copied().unwrap_or(0), 1);
    assert!(stats.success_rate > 0.0 && stats.success_rate < 1.0);
}

#[tokio::test]
async fn test_concurrent_job_operations() {
    let manager = create_test_manager().await;

    let mut handles = vec![];

    // Submit 10 jobs concurrently
    for i in 0..10 {
        let manager_clone = create_test_manager().await;
        let handle = tokio::spawn(async move {
            manager_clone
                .submit_job(
                    vec![format!("https://example.com/{}", i)],
                    "auto".to_string(),
                    None,
                    JobPriority::Medium,
                    vec![],
                    false,
                )
                .await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
}

#[tokio::test]
async fn test_job_priority_ordering() {
    let manager = create_test_manager().await;

    manager
        .submit_job(
            vec!["https://example.com/1".to_string()],
            "auto".to_string(),
            None,
            JobPriority::Low,
            vec![],
            false,
        )
        .await
        .unwrap();

    manager
        .submit_job(
            vec!["https://example.com/2".to_string()],
            "auto".to_string(),
            None,
            JobPriority::Critical,
            vec![],
            false,
        )
        .await
        .unwrap();

    manager
        .submit_job(
            vec!["https://example.com/3".to_string()],
            "auto".to_string(),
            None,
            JobPriority::High,
            vec![],
            false,
        )
        .await
        .unwrap();

    let jobs = manager.list_jobs(None, None, None, None).await.unwrap();

    // Verify priorities
    let critical_count = jobs
        .iter()
        .filter(|j| j.priority == JobPriority::Critical)
        .count();
    let high_count = jobs
        .iter()
        .filter(|j| j.priority == JobPriority::High)
        .count();
    let low_count = jobs
        .iter()
        .filter(|j| j.priority == JobPriority::Low)
        .count();

    assert_eq!(critical_count, 1);
    assert_eq!(high_count, 1);
    assert_eq!(low_count, 1);
}

#[tokio::test]
async fn test_empty_url_list() {
    let manager = create_test_manager().await;

    let job_id = manager
        .submit_job(
            vec![],
            "auto".to_string(),
            None,
            JobPriority::Medium,
            vec![],
            false,
        )
        .await
        .unwrap();

    let job = manager.get_job(&job_id).await.unwrap();
    assert_eq!(job.urls.len(), 0);
    assert_eq!(job.progress.total, 0);
}

#[tokio::test]
async fn test_job_with_multiple_tags() {
    let manager = create_test_manager().await;

    let job_id = manager
        .submit_job(
            vec!["https://example.com".to_string()],
            "auto".to_string(),
            None,
            JobPriority::Medium,
            vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()],
            false,
        )
        .await
        .unwrap();

    let job = manager.get_job(&job_id).await.unwrap();
    assert_eq!(job.tags.len(), 3);
    assert!(job.tags.contains(&"tag1".to_string()));
    assert!(job.tags.contains(&"tag2".to_string()));
    assert!(job.tags.contains(&"tag3".to_string()));
}

#[tokio::test]
async fn test_job_streaming_flag() {
    let manager = create_test_manager().await;

    let job_id = manager
        .submit_job(
            vec!["https://example.com".to_string()],
            "auto".to_string(),
            None,
            JobPriority::Medium,
            vec![],
            true, // streaming enabled
        )
        .await
        .unwrap();

    let job = manager.get_job(&job_id).await.unwrap();
    assert!(job.stream);
}
