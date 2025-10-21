#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique job identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(String);

impl JobId {
    /// Create a new unique job ID
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let random = rand::random::<u32>();
        Self(format!("job_{:x}_{:x}", timestamp, random))
    }

    /// Get the job ID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for JobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for JobId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for JobId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Job execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    /// Job is queued and waiting to start
    Pending,
    /// Job is currently running
    Running,
    /// Job completed successfully
    Completed,
    /// Job failed with an error
    Failed,
    /// Job was cancelled by user
    Cancelled,
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "pending"),
            JobStatus::Running => write!(f, "running"),
            JobStatus::Completed => write!(f, "completed"),
            JobStatus::Failed => write!(f, "failed"),
            JobStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Job priority level
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for JobPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobPriority::Low => write!(f, "low"),
            JobPriority::Medium => write!(f, "medium"),
            JobPriority::High => write!(f, "high"),
            JobPriority::Critical => write!(f, "critical"),
        }
    }
}

impl From<&str> for JobPriority {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => JobPriority::Low,
            "high" => JobPriority::High,
            "critical" => JobPriority::Critical,
            _ => JobPriority::Medium,
        }
    }
}

/// Job execution progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobProgress {
    /// Total number of items to process
    pub total: u32,
    /// Number of items completed
    pub completed: u32,
    /// Number of items failed
    pub failed: u32,
    /// Completion percentage (0-100)
    pub percentage: f32,
    /// Current item being processed
    pub current_item: Option<String>,
}

impl Default for JobProgress {
    fn default() -> Self {
        Self {
            total: 0,
            completed: 0,
            failed: 0,
            percentage: 0.0,
            current_item: None,
        }
    }
}

impl JobProgress {
    /// Create new progress tracker
    pub fn new(total: u32) -> Self {
        Self {
            total,
            completed: 0,
            failed: 0,
            percentage: 0.0,
            current_item: None,
        }
    }

    /// Update progress and recalculate percentage
    pub fn update(&mut self, completed: u32, failed: u32) {
        self.completed = completed;
        self.failed = failed;
        if self.total > 0 {
            self.percentage = ((completed + failed) as f32 / self.total as f32) * 100.0;
        }
    }

    /// Mark one item as completed
    pub fn increment_completed(&mut self) {
        self.completed += 1;
        self.update(self.completed, self.failed);
    }

    /// Mark one item as failed
    pub fn increment_failed(&mut self) {
        self.failed += 1;
        self.update(self.completed, self.failed);
    }

    /// Set the current item being processed
    pub fn set_current(&mut self, item: String) {
        self.current_item = Some(item);
    }

    /// Clear the current item
    pub fn clear_current(&mut self) {
        self.current_item = None;
    }
}

/// Main job structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique job identifier
    pub id: JobId,
    /// Optional job name
    pub name: Option<String>,
    /// Current job status
    pub status: JobStatus,
    /// Job priority
    pub priority: JobPriority,
    /// URLs to extract
    pub urls: Vec<String>,
    /// Extraction strategy (auto, wasm, css, llm, etc.)
    pub strategy: String,
    /// Job creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Job start timestamp
    pub started_at: Option<DateTime<Utc>>,
    /// Job completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Progress tracking
    pub progress: JobProgress,
    /// Job tags for categorization
    pub tags: Vec<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Path to results file
    pub results_path: Option<String>,
    /// Path to log file
    pub log_path: Option<String>,
    /// Enable streaming output
    pub stream: bool,
}

impl Job {
    /// Create a new job
    pub fn new(
        urls: Vec<String>,
        strategy: String,
        name: Option<String>,
        priority: JobPriority,
        tags: Vec<String>,
        stream: bool,
    ) -> Self {
        let total = urls.len() as u32;
        Self {
            id: JobId::new(),
            name,
            status: JobStatus::Pending,
            priority,
            urls,
            strategy,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            started_at: None,
            completed_at: None,
            progress: JobProgress::new(total),
            tags,
            error: None,
            results_path: None,
            log_path: None,
            stream,
        }
    }

    /// Mark job as running
    pub fn start(&mut self) {
        self.status = JobStatus::Running;
        self.started_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Mark job as completed
    pub fn complete(&mut self) {
        self.status = JobStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Mark job as failed with error message
    pub fn fail(&mut self, error: String) {
        self.status = JobStatus::Failed;
        self.error = Some(error);
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Mark job as cancelled
    pub fn cancel(&mut self) {
        self.status = JobStatus::Cancelled;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Update job progress
    pub fn update_progress(&mut self, completed: u32, failed: u32) {
        self.progress.update(completed, failed);
        self.updated_at = Utc::now();
    }

    /// Get job duration in seconds
    pub fn duration_secs(&self) -> Option<f64> {
        if let (Some(started), Some(completed)) = (self.started_at, self.completed_at) {
            Some(completed.signed_duration_since(started).num_milliseconds() as f64 / 1000.0)
        } else {
            None
        }
    }

    /// Check if job is terminal (completed, failed, or cancelled)
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
        )
    }

    /// Get short ID for display (first 8 characters)
    pub fn short_id(&self) -> &str {
        let id_str = self.id.as_str();
        if id_str.len() > 8 {
            &id_str[..8]
        } else {
            id_str
        }
    }
}

/// Log entry for job execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Log timestamp
    pub timestamp: DateTime<Utc>,
    /// Log level (DEBUG, INFO, WARN, ERROR)
    pub level: LogLevel,
    /// Log message
    pub message: String,
    /// Optional URL context
    pub url: Option<String>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(level: LogLevel, message: String) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            message,
            url: None,
        }
    }

    /// Create log entry with URL context
    pub fn with_url(level: LogLevel, message: String, url: String) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            message,
            url: Some(url),
        }
    }
}

/// Log level enumeration
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

impl From<&str> for LogLevel {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "DEBUG" => LogLevel::Debug,
            "WARN" => LogLevel::Warn,
            "ERROR" => LogLevel::Error,
            _ => LogLevel::Info,
        }
    }
}
