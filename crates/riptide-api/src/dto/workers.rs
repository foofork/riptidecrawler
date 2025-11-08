//! Workers API DTOs - Request/Response types for job management
//!
//! Extracted from handlers/workers.rs (Phase 3 Sprint 3.1)
//! Contains all 11 DTOs + conversion traits

use chrono::{DateTime, Utc};
use riptide_workers::{Job, JobPriority, JobStatus, JobType, ScheduledJob};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Deserialize, Debug, Clone)]
pub struct SubmitJobRequest {
    pub job_type: JobTypeRequest,
    pub priority: Option<JobPriority>,
    pub retry_config: Option<RetryConfigRequest>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub timeout_secs: Option<u64>,
}

impl SubmitJobRequest {
    pub fn into_job(self) -> Result<Job, String> {
        let mut job = if let Some(scheduled_at) = self.scheduled_at {
            Job::scheduled(JobType::from(self.job_type), scheduled_at)
        } else {
            Job::new(JobType::from(self.job_type))
        };

        if let Some(p) = self.priority {
            job.priority = p;
        }
        if let Some(r) = self.retry_config {
            job.retry_config = r.into();
        }
        if let Some(m) = self.metadata {
            job.metadata = m;
        }
        if let Some(t) = self.timeout_secs {
            job.timeout_secs = Some(t);
        }

        Ok(job)
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum JobTypeRequest {
    #[serde(rename = "batch_crawl")]
    BatchCrawl {
        urls: Vec<String>,
        options: Option<riptide_types::config::CrawlOptions>,
    },
    #[serde(rename = "single_crawl")]
    SingleCrawl {
        url: String,
        options: Option<riptide_types::config::CrawlOptions>,
    },
    #[serde(rename = "maintenance")]
    Maintenance {
        task_type: String,
        parameters: HashMap<String, serde_json::Value>,
    },
    #[serde(rename = "custom")]
    Custom {
        job_name: String,
        payload: serde_json::Value,
    },
}

impl From<JobTypeRequest> for JobType {
    fn from(request: JobTypeRequest) -> Self {
        match request {
            JobTypeRequest::BatchCrawl { urls, options } => JobType::BatchCrawl { urls, options },
            JobTypeRequest::SingleCrawl { url, options } => JobType::SingleCrawl { url, options },
            JobTypeRequest::Maintenance {
                task_type,
                parameters,
            } => JobType::Maintenance {
                task_type,
                parameters,
            },
            JobTypeRequest::Custom { job_name, payload } => JobType::Custom { job_name, payload },
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct RetryConfigRequest {
    pub max_attempts: Option<u32>,
    pub initial_delay_secs: Option<u64>,
    pub backoff_multiplier: Option<f64>,
    pub max_delay_secs: Option<u64>,
    pub use_jitter: Option<bool>,
}

impl From<RetryConfigRequest> for riptide_workers::RetryConfig {
    fn from(request: RetryConfigRequest) -> Self {
        let mut config = riptide_workers::RetryConfig::default();
        if let Some(max) = request.max_attempts {
            config.max_attempts = max;
        }
        if let Some(delay) = request.initial_delay_secs {
            config.initial_delay_secs = delay;
        }
        if let Some(mult) = request.backoff_multiplier {
            config.backoff_multiplier = mult;
        }
        if let Some(max_delay) = request.max_delay_secs {
            config.max_delay_secs = max_delay;
        }
        if let Some(jitter) = request.use_jitter {
            config.use_jitter = jitter;
        }
        config
    }
}

#[derive(Serialize, Debug)]
pub struct SubmitJobResponse {
    pub job_id: Uuid,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
    pub message: String,
}

impl SubmitJobResponse {
    pub fn new(job_id: Uuid) -> Self {
        Self {
            job_id,
            status: "submitted".to_string(),
            submitted_at: Utc::now(),
            message: "Job submitted successfully".to_string(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct JobStatusResponse {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub worker_id: Option<String>,
    pub retry_count: u32,
    pub last_error: Option<String>,
    pub processing_time_ms: Option<u64>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl From<&Job> for JobStatusResponse {
    fn from(job: &Job) -> Self {
        let processing_time_ms = calculate_processing_time(job);
        Self {
            job_id: job.id,
            status: job.status.clone(),
            created_at: job.created_at,
            started_at: job.started_at,
            completed_at: job.completed_at,
            worker_id: job.worker_id.clone(),
            retry_count: job.retry_count,
            last_error: job.last_error.clone(),
            processing_time_ms,
            metadata: job.metadata.clone(),
        }
    }
}

fn calculate_processing_time(job: &Job) -> Option<u64> {
    if let (Some(s), Some(c)) = (job.started_at, job.completed_at) {
        Some((c - s).num_milliseconds() as u64)
    } else {
        job.started_at
            .map(|s| (Utc::now() - s).num_milliseconds() as u64)
    }
}

#[derive(Serialize, Debug)]
pub struct JobResultResponse {
    pub job_id: Uuid,
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub processing_time_ms: u64,
    pub worker_id: String,
    pub completed_at: DateTime<Utc>,
}

#[derive(Serialize, Debug)]
pub struct QueueStatsResponse {
    pub pending: usize,
    pub processing: usize,
    pub completed: usize,
    pub failed: usize,
    pub retry: usize,
    pub delayed: usize,
    pub total: usize,
}

#[derive(Serialize, Debug)]
pub struct WorkerPoolStatsResponse {
    pub total_workers: usize,
    pub healthy_workers: usize,
    pub total_jobs_processed: u64,
    pub total_jobs_failed: u64,
    pub is_running: bool,
}

#[derive(Deserialize, Debug)]
pub struct CreateScheduledJobRequest {
    pub name: String,
    pub cron_expression: String,
    pub job_template: JobTypeRequest,
    pub priority: Option<JobPriority>,
    pub enabled: Option<bool>,
    pub retry_config: Option<RetryConfigRequest>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Serialize, Debug)]
pub struct ScheduledJobResponse {
    pub id: Uuid,
    pub name: String,
    pub cron_expression: String,
    pub enabled: bool,
    pub priority: JobPriority,
    pub created_at: DateTime<Utc>,
    pub last_executed_at: Option<DateTime<Utc>>,
    pub next_execution_at: Option<DateTime<Utc>>,
    pub execution_count: u64,
}

impl From<&ScheduledJob> for ScheduledJobResponse {
    fn from(job: &ScheduledJob) -> Self {
        Self {
            id: job.id,
            name: job.name.clone(),
            cron_expression: job.cron_expression.clone(),
            enabled: job.enabled,
            priority: job.priority,
            created_at: job.created_at,
            last_executed_at: job.last_executed_at,
            next_execution_at: job.next_execution_at,
            execution_count: job.execution_count,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct JobListQuery {
    pub status: Option<String>,
    pub job_type: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub search: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct JobListResponse {
    pub jobs: Vec<JobListItem>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Serialize, Debug)]
pub struct JobListItem {
    pub job_id: Uuid,
    pub job_type: String,
    pub status: JobStatus,
    pub priority: JobPriority,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub worker_id: Option<String>,
    pub retry_count: u32,
}

impl JobListItem {
    pub fn from_job(job: &Job) -> Self {
        Self {
            job_id: job.id,
            job_type: format_job_type(&job.job_type),
            status: job.status.clone(),
            priority: job.priority,
            created_at: job.created_at,
            started_at: job.started_at,
            completed_at: job.completed_at,
            worker_id: job.worker_id.clone(),
            retry_count: job.retry_count,
        }
    }
}

pub fn format_job_type(job_type: &JobType) -> String {
    match job_type {
        JobType::BatchCrawl { .. } => "batch_crawl".to_string(),
        JobType::SingleCrawl { .. } => "single_crawl".to_string(),
        JobType::PdfExtraction { .. } => "pdf_extraction".to_string(),
        JobType::Maintenance { task_type, .. } => format!("maintenance:{}", task_type),
        JobType::Custom { job_name, .. } => format!("custom:{}", job_name),
    }
}
