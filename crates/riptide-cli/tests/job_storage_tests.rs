#![allow(clippy::all, dead_code, unused)]

//! Tests for job storage persistence
//!
//! Coverage includes:
//! - Storage initialization
//! - Job save/load operations
//! - Log persistence
//! - Results storage
//! - Job deletion
//! - Cleanup operations

use riptide_cli::job::storage::JobStorage;
use riptide_cli::job::types::{Job, JobId, JobPriority, LogEntry, LogLevel};
use tempfile::TempDir;

/// Helper to create test storage with temp directory
fn create_test_storage() -> JobStorage {
    let temp_dir = TempDir::new().unwrap();
    std::env::set_var("HOME", temp_dir.path());
    JobStorage::new().unwrap()
}

#[test]
fn test_storage_creation() {
    let storage = create_test_storage();
    assert!(storage.base_dir().exists());
}

#[test]
fn test_save_and_load_job() {
    let storage = create_test_storage();

    let job = Job::new(
        vec!["https://example.com".to_string()],
        "auto".to_string(),
        Some("Test Job".to_string()),
        JobPriority::High,
        vec!["test".to_string()],
        false,
    );

    storage.save_job(&job).unwrap();

    let loaded = storage.load_job(&job.id).unwrap();
    assert_eq!(loaded.id, job.id);
    assert_eq!(loaded.name, job.name);
    assert_eq!(loaded.urls, job.urls);
}

#[test]
fn test_load_nonexistent_job() {
    let storage = create_test_storage();

    let result = storage.load_job(&JobId::from("nonexistent"));
    assert!(result.is_err());
}

#[test]
fn test_list_empty_jobs() {
    let storage = create_test_storage();

    let jobs = storage.list_jobs().unwrap();
    assert_eq!(jobs.len(), 0);
}

#[test]
fn test_list_multiple_jobs() {
    let storage = create_test_storage();

    for i in 0..5 {
        let job = Job::new(
            vec![format!("https://example.com/{}", i)],
            "auto".to_string(),
            Some(format!("Job {}", i)),
            JobPriority::Medium,
            vec![],
            false,
        );
        storage.save_job(&job).unwrap();
    }

    let jobs = storage.list_jobs().unwrap();
    assert_eq!(jobs.len(), 5);
}

#[test]
fn test_delete_job() {
    let storage = create_test_storage();

    let job = Job::new(
        vec!["https://example.com".to_string()],
        "auto".to_string(),
        None,
        JobPriority::Medium,
        vec![],
        false,
    );

    storage.save_job(&job).unwrap();
    storage.delete_job(&job.id).unwrap();

    let result = storage.load_job(&job.id);
    assert!(result.is_err());
}

#[test]
fn test_delete_nonexistent_job() {
    let storage = create_test_storage();

    // Should not error when deleting nonexistent job
    let result = storage.delete_job(&JobId::from("nonexistent"));
    assert!(result.is_ok());
}

#[test]
fn test_append_log() {
    let storage = create_test_storage();

    let job = Job::new(
        vec!["https://example.com".to_string()],
        "auto".to_string(),
        None,
        JobPriority::Medium,
        vec![],
        false,
    );

    storage.save_job(&job).unwrap();

    let entry = LogEntry::new(LogLevel::Info, "Test log message".to_string());
    storage.append_log(&job.id, &entry).unwrap();

    let logs = storage.read_logs(&job.id, None, None).unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].message, "Test log message");
}

#[test]
fn test_append_multiple_logs() {
    let storage = create_test_storage();

    let job = Job::new(
        vec!["https://example.com".to_string()],
        "auto".to_string(),
        None,
        JobPriority::Medium,
        vec![],
        false,
    );

    storage.save_job(&job).unwrap();

    for i in 0..10 {
        let entry = LogEntry::new(LogLevel::Info, format!("Log message {}", i));
        storage.append_log(&job.id, &entry).unwrap();
    }

    let logs = storage.read_logs(&job.id, None, None).unwrap();
    assert_eq!(logs.len(), 10);
}

#[test]
fn test_read_logs_with_limit() {
    let storage = create_test_storage();

    let job = Job::new(
        vec!["https://example.com".to_string()],
        "auto".to_string(),
        None,
        JobPriority::Medium,
        vec![],
        false,
    );

    storage.save_job(&job).unwrap();

    for i in 0..20 {
        let entry = LogEntry::new(LogLevel::Info, format!("Log {}", i));
        storage.append_log(&job.id, &entry).unwrap();
    }

    let logs = storage.read_logs(&job.id, Some(5), None).unwrap();
    assert_eq!(logs.len(), 5);
    // Should get last 5 logs
    assert!(logs[0].message.contains("15") || logs[0].message.contains("16"));
}

#[test]
fn test_read_logs_with_level_filter() {
    let storage = create_test_storage();

    let job = Job::new(
        vec!["https://example.com".to_string()],
        "auto".to_string(),
        None,
        JobPriority::Medium,
        vec![],
        false,
    );

    storage.save_job(&job).unwrap();

    storage
        .append_log(
            &job.id,
            &LogEntry::new(LogLevel::Info, "Info message".to_string()),
        )
        .unwrap();
    storage
        .append_log(
            &job.id,
            &LogEntry::new(LogLevel::Warn, "Warning message".to_string()),
        )
        .unwrap();
    storage
        .append_log(
            &job.id,
            &LogEntry::new(LogLevel::Error, "Error message".to_string()),
        )
        .unwrap();

    let warnings = storage.read_logs(&job.id, None, Some("warn")).unwrap();
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].level, LogLevel::Warn);
}

#[test]
fn test_read_logs_nonexistent_job() {
    let storage = create_test_storage();

    let logs = storage
        .read_logs(&JobId::from("nonexistent"), None, None)
        .unwrap();
    assert_eq!(logs.len(), 0);
}

#[test]
fn test_save_and_load_results() {
    let storage = create_test_storage();

    let job = Job::new(
        vec!["https://example.com".to_string()],
        "auto".to_string(),
        None,
        JobPriority::Medium,
        vec![],
        false,
    );

    storage.save_job(&job).unwrap();

    let results = serde_json::json!({
        "status": "success",
        "extracted": 42,
        "data": ["item1", "item2", "item3"]
    });

    storage.save_results(&job.id, &results).unwrap();

    let loaded = storage.load_results(&job.id).unwrap();
    assert_eq!(loaded, results);
}

#[test]
fn test_load_results_nonexistent() {
    let storage = create_test_storage();

    let job = Job::new(
        vec!["https://example.com".to_string()],
        "auto".to_string(),
        None,
        JobPriority::Medium,
        vec![],
        false,
    );

    storage.save_job(&job).unwrap();

    let results = storage.load_results(&job.id).unwrap();
    // Should return default JSON when no results exist
    assert!(results.is_object());
}

#[test]
fn test_get_storage_stats() {
    let storage = create_test_storage();

    // Create some jobs
    for i in 0..3 {
        let job = Job::new(
            vec![format!("https://example.com/{}", i)],
            "auto".to_string(),
            None,
            JobPriority::Medium,
            vec![],
            false,
        );
        storage.save_job(&job).unwrap();
    }

    let stats = storage.get_stats().unwrap();
    assert_eq!(stats.total_jobs, 3);
    assert!(stats.total_size_bytes > 0);
}

#[test]
fn test_cleanup_old_jobs() {
    use chrono::{Duration, Utc};

    let storage = create_test_storage();

    let mut job = Job::new(
        vec!["https://example.com".to_string()],
        "auto".to_string(),
        None,
        JobPriority::Medium,
        vec![],
        false,
    );

    // Mark as completed in the past
    job.complete();
    job.completed_at = Some(Utc::now() - Duration::days(10));

    storage.save_job(&job).unwrap();

    // Cleanup jobs older than 5 days
    let deleted = storage.cleanup_old_jobs(5).unwrap();
    assert_eq!(deleted.len(), 1);

    let remaining = storage.list_jobs().unwrap();
    assert_eq!(remaining.len(), 0);
}

#[test]
fn test_cleanup_does_not_remove_recent_jobs() {
    let storage = create_test_storage();

    let mut job = Job::new(
        vec!["https://example.com".to_string()],
        "auto".to_string(),
        None,
        JobPriority::Medium,
        vec![],
        false,
    );

    job.complete();
    storage.save_job(&job).unwrap();

    // Cleanup jobs older than 30 days (recent job should remain)
    let deleted = storage.cleanup_old_jobs(30).unwrap();
    assert_eq!(deleted.len(), 0);

    let remaining = storage.list_jobs().unwrap();
    assert_eq!(remaining.len(), 1);
}

#[test]
fn test_cleanup_does_not_remove_running_jobs() {
    use chrono::{Duration, Utc};

    let storage = create_test_storage();

    let mut job = Job::new(
        vec!["https://example.com".to_string()],
        "auto".to_string(),
        None,
        JobPriority::Medium,
        vec![],
        false,
    );

    // Start but don't complete
    job.start();
    job.started_at = Some(Utc::now() - Duration::days(10));

    storage.save_job(&job).unwrap();

    let deleted = storage.cleanup_old_jobs(5).unwrap();
    assert_eq!(deleted.len(), 0); // Running job should not be deleted
}

#[test]
fn test_storage_persistence() {
    let temp_dir = TempDir::new().unwrap();
    std::env::set_var("HOME", temp_dir.path());

    let job_id = {
        let storage = JobStorage::new().unwrap();

        let job = Job::new(
            vec!["https://example.com".to_string()],
            "auto".to_string(),
            Some("Persistent Job".to_string()),
            JobPriority::High,
            vec![],
            false,
        );

        storage.save_job(&job).unwrap();
        job.id.clone()
    };

    // Create new storage instance and verify job exists
    {
        let storage = JobStorage::new().unwrap();
        let loaded = storage.load_job(&job_id).unwrap();
        assert_eq!(loaded.name, Some("Persistent Job".to_string()));
    }
}

#[test]
fn test_concurrent_storage_operations() {
    use std::sync::Arc;
    use std::thread;

    let temp_dir = TempDir::new().unwrap();
    std::env::set_var("HOME", temp_dir.path());

    let storage = Arc::new(JobStorage::new().unwrap());
    let mut handles = vec![];

    for i in 0..5 {
        let storage_clone = Arc::clone(&storage);
        let handle = thread::spawn(move || {
            let job = Job::new(
                vec![format!("https://example.com/{}", i)],
                "auto".to_string(),
                None,
                JobPriority::Medium,
                vec![],
                false,
            );
            storage_clone.save_job(&job).unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let jobs = storage.list_jobs().unwrap();
    assert_eq!(jobs.len(), 5);
}

#[test]
fn test_humanize_bytes_formatting() {
    let storage = create_test_storage();
    let stats = storage.get_stats().unwrap();

    let human = stats.size_human();
    assert!(human.contains("B") || human.contains("KB") || human.contains("MB"));
}
