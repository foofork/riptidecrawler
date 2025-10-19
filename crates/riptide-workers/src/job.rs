use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Options for PDF extraction jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfExtractionOptions {
    /// Extract text from PDF
    pub extract_text: bool,
    /// Extract images from PDF
    pub extract_images: bool,
    /// Extract metadata from PDF
    pub extract_metadata: bool,
    /// Maximum file size in bytes
    pub max_size_bytes: u64,
    /// Enable progress tracking
    pub enable_progress: bool,
    /// Custom extraction settings
    pub custom_settings: HashMap<String, serde_json::Value>,
}

impl Default for PdfExtractionOptions {
    fn default() -> Self {
        Self {
            extract_text: true,
            extract_images: false,
            extract_metadata: true,
            max_size_bytes: 100 * 1024 * 1024, // 100MB
            enable_progress: true,
            custom_settings: HashMap::new(),
        }
    }
}

/// Job priority levels for queue management
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum JobPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

impl Default for JobPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Job status tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Retrying,
    DeadLetter,
}

/// Job types that can be processed by workers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobType {
    /// Batch crawl multiple URLs
    BatchCrawl {
        urls: Vec<String>,
        options: Option<riptide_types::config::CrawlOptions>,
    },
    /// Single URL crawl
    SingleCrawl {
        url: String,
        options: Option<riptide_types::config::CrawlOptions>,
    },
    /// PDF extraction and processing
    PdfExtraction {
        pdf_data: Vec<u8>,
        url: Option<String>,
        options: Option<PdfExtractionOptions>,
    },
    /// Scheduled maintenance task
    Maintenance {
        task_type: String,
        parameters: HashMap<String, serde_json::Value>,
    },
    /// Custom job with arbitrary payload
    Custom {
        job_name: String,
        payload: serde_json::Value,
    },
}

/// Retry configuration for failed jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay in seconds before first retry
    pub initial_delay_secs: u64,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Maximum delay between retries in seconds
    pub max_delay_secs: u64,
    /// Whether to use jitter to avoid thundering herd
    pub use_jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_secs: 30,
            backoff_multiplier: 2.0,
            max_delay_secs: 300, // 5 minutes
            use_jitter: true,
        }
    }
}

/// A job in the worker queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique job identifier
    pub id: Uuid,
    /// Job type and payload
    pub job_type: JobType,
    /// Job priority for queue ordering
    pub priority: JobPriority,
    /// Current status
    pub status: JobStatus,
    /// Job created timestamp
    pub created_at: DateTime<Utc>,
    /// Scheduled execution time (for delayed jobs)
    pub scheduled_at: Option<DateTime<Utc>>,
    /// Job started processing timestamp
    pub started_at: Option<DateTime<Utc>>,
    /// Job completed timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Number of retry attempts made
    pub retry_count: u32,
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Next retry time if in retrying state
    pub next_retry_at: Option<DateTime<Utc>>,
    /// Error message from last failure
    pub last_error: Option<String>,
    /// Job metadata and tags
    pub metadata: HashMap<String, serde_json::Value>,
    /// Worker ID that processed/is processing this job
    pub worker_id: Option<String>,
    /// Maximum time job can run before being considered stuck
    pub timeout_secs: Option<u64>,
}

impl Job {
    /// Create a new job with default settings
    pub fn new(job_type: JobType) -> Self {
        Self {
            id: Uuid::new_v4(),
            job_type,
            priority: JobPriority::default(),
            status: JobStatus::Pending,
            created_at: Utc::now(),
            scheduled_at: None,
            started_at: None,
            completed_at: None,
            retry_count: 0,
            retry_config: RetryConfig::default(),
            next_retry_at: None,
            last_error: None,
            metadata: HashMap::new(),
            worker_id: None,
            timeout_secs: Some(600), // 10 minutes default
        }
    }

    /// Create a new job with custom priority
    pub fn with_priority(job_type: JobType, priority: JobPriority) -> Self {
        let mut job = Self::new(job_type);
        job.priority = priority;
        job
    }

    /// Create a delayed job that should be executed at a specific time
    pub fn scheduled(job_type: JobType, scheduled_at: DateTime<Utc>) -> Self {
        let mut job = Self::new(job_type);
        job.scheduled_at = Some(scheduled_at);
        job
    }

    /// Add metadata to the job
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set custom retry configuration
    pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = retry_config;
        self
    }

    /// Set custom timeout
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = Some(timeout_secs);
        self
    }

    /// Check if job is ready to be processed
    pub fn is_ready(&self) -> bool {
        match self.status {
            JobStatus::Pending => {
                if let Some(scheduled_at) = self.scheduled_at {
                    Utc::now() >= scheduled_at
                } else {
                    true
                }
            }
            JobStatus::Retrying => {
                if let Some(retry_at) = self.next_retry_at {
                    Utc::now() >= retry_at
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    /// Check if job has timed out
    pub fn is_timed_out(&self) -> bool {
        if let (Some(started_at), Some(timeout_secs)) = (self.started_at, self.timeout_secs) {
            let timeout_duration = chrono::Duration::seconds(timeout_secs as i64);
            Utc::now() - started_at > timeout_duration
        } else {
            false
        }
    }

    /// Calculate next retry time based on exponential backoff
    pub fn calculate_next_retry(&self) -> DateTime<Utc> {
        let base_delay = self.retry_config.initial_delay_secs as f64;
        let delay_secs = base_delay
            * self
                .retry_config
                .backoff_multiplier
                .powi(self.retry_count as i32);
        let max_delay = self.retry_config.max_delay_secs as f64;
        let actual_delay = delay_secs.min(max_delay);

        let final_delay = if self.retry_config.use_jitter {
            // Add up to 10% jitter to prevent thundering herd
            let jitter = actual_delay * 0.1 * rand::random::<f64>();
            actual_delay + jitter
        } else {
            actual_delay
        };

        Utc::now() + chrono::Duration::seconds(final_delay as i64)
    }

    /// Mark job as started
    pub fn start(&mut self, worker_id: String) {
        self.status = JobStatus::Processing;
        self.started_at = Some(Utc::now());
        self.worker_id = Some(worker_id);
    }

    /// Mark job as completed successfully
    pub fn complete(&mut self) {
        self.status = JobStatus::Completed;
        self.completed_at = Some(Utc::now());
    }

    /// Mark job as failed and calculate retry
    pub fn fail(&mut self, error: String) {
        self.last_error = Some(error);
        self.retry_count += 1;

        if self.retry_count >= self.retry_config.max_attempts {
            self.status = JobStatus::DeadLetter;
            self.completed_at = Some(Utc::now());
        } else {
            self.status = JobStatus::Retrying;
            self.next_retry_at = Some(self.calculate_next_retry());
        }
    }

    /// Get job age in seconds
    pub fn age_seconds(&self) -> i64 {
        (Utc::now() - self.created_at).num_seconds()
    }

    /// Get processing time if job has completed
    pub fn processing_time_ms(&self) -> Option<u64> {
        if let (Some(started), Some(completed)) = (self.started_at, self.completed_at) {
            Some((completed - started).num_milliseconds() as u64)
        } else {
            None
        }
    }
}

/// Job result after processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    /// Job ID
    pub job_id: Uuid,
    /// Whether job succeeded
    pub success: bool,
    /// Result data (varies by job type)
    pub data: Option<serde_json::Value>,
    /// Error message if failed
    pub error: Option<String>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Worker that processed the job
    pub worker_id: String,
    /// Timestamp when result was generated
    pub completed_at: DateTime<Utc>,
}

impl JobResult {
    /// Create successful job result
    pub fn success(
        job_id: Uuid,
        worker_id: String,
        data: Option<serde_json::Value>,
        processing_time_ms: u64,
    ) -> Self {
        Self {
            job_id,
            success: true,
            data,
            error: None,
            processing_time_ms,
            worker_id,
            completed_at: Utc::now(),
        }
    }

    /// Create failed job result
    pub fn failure(
        job_id: Uuid,
        worker_id: String,
        error: String,
        processing_time_ms: u64,
    ) -> Self {
        Self {
            job_id,
            success: false,
            data: None,
            error: Some(error),
            processing_time_ms,
            worker_id,
            completed_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation() {
        let job_type = JobType::SingleCrawl {
            url: "https://example.com".to_string(),
            options: None,
        };
        let job = Job::new(job_type);

        assert_eq!(job.priority, JobPriority::Normal);
        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.retry_count, 0);
        assert!(job.is_ready());
    }

    #[test]
    fn test_retry_calculation() {
        let job_type = JobType::SingleCrawl {
            url: "https://example.com".to_string(),
            options: None,
        };
        let mut job = Job::new(job_type);

        // Simulate failure
        job.fail("Test error".to_string());
        assert_eq!(job.status, JobStatus::Retrying);
        assert_eq!(job.retry_count, 1);
        assert!(job.next_retry_at.is_some());
    }

    #[test]
    fn test_dead_letter_after_max_retries() {
        let job_type = JobType::SingleCrawl {
            url: "https://example.com".to_string(),
            options: None,
        };
        let mut job = Job::new(job_type).with_retry_config(RetryConfig {
            max_attempts: 2,
            ..Default::default()
        });

        // First failure
        job.fail("Error 1".to_string());
        assert_eq!(job.status, JobStatus::Retrying);

        // Second failure - should go to dead letter
        job.fail("Error 2".to_string());
        assert_eq!(job.status, JobStatus::DeadLetter);
        assert!(job.completed_at.is_some());
    }
}
