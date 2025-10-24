#![allow(clippy::all, dead_code, unused)]

//! Tests for job types and data structures
//!
//! Coverage includes:
//! - JobId generation and uniqueness
//! - Job status transitions
//! - Priority comparisons
//! - Progress calculations
//! - Log entry creation

use riptide_cli::job::types::{
    Job, JobId, JobPriority, JobProgress, JobStatus, LogEntry, LogLevel,
};

#[test]
fn test_job_id_generation() {
    let id1 = JobId::new();
    let id2 = JobId::new();

    // IDs should be unique
    assert_ne!(id1.as_str(), id2.as_str());

    // IDs should follow format
    assert!(id1.as_str().starts_with("job_"));
}

#[test]
fn test_job_id_from_string() {
    let id = JobId::from("custom_job_id");
    assert_eq!(id.as_str(), "custom_job_id");
}

#[test]
fn test_job_status_display() {
    assert_eq!(JobStatus::Pending.to_string(), "pending");
    assert_eq!(JobStatus::Running.to_string(), "running");
    assert_eq!(JobStatus::Completed.to_string(), "completed");
    assert_eq!(JobStatus::Failed.to_string(), "failed");
    assert_eq!(JobStatus::Cancelled.to_string(), "cancelled");
}

#[test]
fn test_job_priority_ordering() {
    assert!(JobPriority::Critical > JobPriority::High);
    assert!(JobPriority::High > JobPriority::Medium);
    assert!(JobPriority::Medium > JobPriority::Low);
}

#[test]
fn test_job_priority_from_string() {
    assert_eq!(JobPriority::from("low"), JobPriority::Low);
    assert_eq!(JobPriority::from("high"), JobPriority::High);
    assert_eq!(JobPriority::from("critical"), JobPriority::Critical);
    assert_eq!(JobPriority::from("unknown"), JobPriority::Medium); // default
}

#[test]
fn test_job_progress_initialization() {
    let progress = JobProgress::new(10);
    assert_eq!(progress.total, 10);
    assert_eq!(progress.completed, 0);
    assert_eq!(progress.failed, 0);
    assert_eq!(progress.percentage, 0.0);
    assert!(progress.current_item.is_none());
}

#[test]
fn test_job_progress_update() {
    let mut progress = JobProgress::new(100);
    progress.update(25, 5);

    assert_eq!(progress.completed, 25);
    assert_eq!(progress.failed, 5);
    assert!((progress.percentage - 30.0).abs() < 0.01); // (25 + 5) / 100 * 100
}

#[test]
fn test_job_progress_increment_completed() {
    let mut progress = JobProgress::new(10);
    progress.increment_completed();
    progress.increment_completed();

    assert_eq!(progress.completed, 2);
    assert_eq!(progress.percentage, 20.0);
}

#[test]
fn test_job_progress_increment_failed() {
    let mut progress = JobProgress::new(10);
    progress.increment_failed();

    assert_eq!(progress.failed, 1);
    assert_eq!(progress.percentage, 10.0);
}

#[test]
fn test_job_progress_current_item() {
    let mut progress = JobProgress::new(5);

    progress.set_current("https://example.com".to_string());
    assert_eq!(
        progress.current_item,
        Some("https://example.com".to_string())
    );

    progress.clear_current();
    assert!(progress.current_item.is_none());
}

#[test]
fn test_job_creation() {
    let job = Job::new(
        vec!["https://example.com".to_string()],
        "auto".to_string(),
        Some("Test Job".to_string()),
        JobPriority::High,
        vec!["test".to_string()],
        false,
    );

    assert_eq!(job.status, JobStatus::Pending);
    assert_eq!(job.name, Some("Test Job".to_string()));
    assert_eq!(job.priority, JobPriority::High);
    assert_eq!(job.urls.len(), 1);
    assert_eq!(job.progress.total, 1);
    assert!(!job.stream);
}

#[test]
fn test_job_start() {
    let mut job = Job::new(
        vec!["https://example.com".to_string()],
        "auto".to_string(),
        None,
        JobPriority::Medium,
        vec![],
        false,
    );

    job.start();
    assert_eq!(job.status, JobStatus::Running);
    assert!(job.started_at.is_some());
}

#[test]
fn test_job_complete() {
    let mut job = Job::new(
        vec!["https://example.com".to_string()],
        "auto".to_string(),
        None,
        JobPriority::Medium,
        vec![],
        false,
    );

    job.start();
    job.complete();

    assert_eq!(job.status, JobStatus::Completed);
    assert!(job.completed_at.is_some());
}

#[test]
fn test_job_fail() {
    let mut job = Job::new(
        vec!["https://example.com".to_string()],
        "auto".to_string(),
        None,
        JobPriority::Medium,
        vec![],
        false,
    );

    job.fail("Network error".to_string());

    assert_eq!(job.status, JobStatus::Failed);
    assert_eq!(job.error, Some("Network error".to_string()));
    assert!(job.completed_at.is_some());
}

#[test]
fn test_job_cancel() {
    let mut job = Job::new(
        vec!["https://example.com".to_string()],
        "auto".to_string(),
        None,
        JobPriority::Medium,
        vec![],
        false,
    );

    job.cancel();

    assert_eq!(job.status, JobStatus::Cancelled);
    assert!(job.completed_at.is_some());
}

#[test]
fn test_job_update_progress() {
    let mut job = Job::new(
        vec![
            "https://example.com/1".to_string(),
            "https://example.com/2".to_string(),
        ],
        "auto".to_string(),
        None,
        JobPriority::Medium,
        vec![],
        false,
    );

    job.update_progress(1, 0);
    assert_eq!(job.progress.completed, 1);
    assert_eq!(job.progress.percentage, 50.0);
}

#[test]
fn test_job_duration_calculation() {
    let mut job = Job::new(
        vec!["https://example.com".to_string()],
        "auto".to_string(),
        None,
        JobPriority::Medium,
        vec![],
        false,
    );

    // Before start, no duration
    assert!(job.duration_secs().is_none());

    job.start();
    std::thread::sleep(std::time::Duration::from_millis(100));
    job.complete();

    let duration = job.duration_secs().unwrap();
    assert!(duration >= 0.1 && duration < 1.0);
}

#[test]
fn test_job_is_terminal() {
    let mut job = Job::new(
        vec!["https://example.com".to_string()],
        "auto".to_string(),
        None,
        JobPriority::Medium,
        vec![],
        false,
    );

    assert!(!job.is_terminal()); // pending

    job.start();
    assert!(!job.is_terminal()); // running

    job.complete();
    assert!(job.is_terminal()); // completed
}

#[test]
fn test_job_short_id() {
    let job = Job::new(
        vec!["https://example.com".to_string()],
        "auto".to_string(),
        None,
        JobPriority::Medium,
        vec![],
        false,
    );

    let short_id = job.short_id();
    assert!(short_id.len() <= 8);
}

#[test]
fn test_log_level_display() {
    assert_eq!(LogLevel::Debug.to_string(), "DEBUG");
    assert_eq!(LogLevel::Info.to_string(), "INFO");
    assert_eq!(LogLevel::Warn.to_string(), "WARN");
    assert_eq!(LogLevel::Error.to_string(), "ERROR");
}

#[test]
fn test_log_level_from_string() {
    assert_eq!(LogLevel::from("debug"), LogLevel::Debug);
    assert_eq!(LogLevel::from("info"), LogLevel::Info);
    assert_eq!(LogLevel::from("warn"), LogLevel::Warn);
    assert_eq!(LogLevel::from("error"), LogLevel::Error);
    assert_eq!(LogLevel::from("unknown"), LogLevel::Info); // default
}

#[test]
fn test_log_level_ordering() {
    assert!(LogLevel::Error > LogLevel::Warn);
    assert!(LogLevel::Warn > LogLevel::Info);
    assert!(LogLevel::Info > LogLevel::Debug);
}

#[test]
fn test_log_entry_creation() {
    let entry = LogEntry::new(LogLevel::Info, "Test message".to_string());

    assert_eq!(entry.level, LogLevel::Info);
    assert_eq!(entry.message, "Test message");
    assert!(entry.url.is_none());
}

#[test]
fn test_log_entry_with_url() {
    let entry = LogEntry::with_url(
        LogLevel::Debug,
        "Processing URL".to_string(),
        "https://example.com".to_string(),
    );

    assert_eq!(entry.level, LogLevel::Debug);
    assert_eq!(entry.message, "Processing URL");
    assert_eq!(entry.url, Some("https://example.com".to_string()));
}

#[test]
fn test_job_serialization() {
    let job = Job::new(
        vec!["https://example.com".to_string()],
        "auto".to_string(),
        Some("Test".to_string()),
        JobPriority::High,
        vec!["tag1".to_string()],
        true,
    );

    let json = serde_json::to_string(&job).unwrap();
    let deserialized: Job = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name, job.name);
    assert_eq!(deserialized.priority, job.priority);
    assert_eq!(deserialized.urls, job.urls);
}

#[test]
fn test_log_entry_serialization() {
    let entry = LogEntry::new(LogLevel::Warn, "Warning message".to_string());

    let json = serde_json::to_string(&entry).unwrap();
    let deserialized: LogEntry = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.level, entry.level);
    assert_eq!(deserialized.message, entry.message);
}
