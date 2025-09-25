use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use riptide_workers::{
    Job, JobType, JobPriority, JobStatus, ScheduledJob,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::state::AppState;

/// Request body for submitting a job
#[derive(Deserialize, Debug, Clone)]
pub struct SubmitJobRequest {
    /// Job type to submit
    pub job_type: JobTypeRequest,
    /// Job priority (optional, defaults to Normal)
    pub priority: Option<JobPriority>,
    /// Retry configuration (optional)
    pub retry_config: Option<RetryConfigRequest>,
    /// Job metadata (optional)
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Scheduled execution time (optional, for delayed jobs)
    pub scheduled_at: Option<DateTime<Utc>>,
    /// Job timeout in seconds (optional)
    pub timeout_secs: Option<u64>,
}

/// Job type request structure
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum JobTypeRequest {
    #[serde(rename = "batch_crawl")]
    BatchCrawl {
        urls: Vec<String>,
        options: Option<riptide_core::types::CrawlOptions>,
    },
    #[serde(rename = "single_crawl")]
    SingleCrawl {
        url: String,
        options: Option<riptide_core::types::CrawlOptions>,
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
            JobTypeRequest::Maintenance { task_type, parameters } => {
                JobType::Maintenance { task_type, parameters }
            }
            JobTypeRequest::Custom { job_name, payload } => JobType::Custom { job_name, payload },
        }
    }
}

/// Retry configuration request
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
        if let Some(max_attempts) = request.max_attempts {
            config.max_attempts = max_attempts;
        }
        if let Some(initial_delay) = request.initial_delay_secs {
            config.initial_delay_secs = initial_delay;
        }
        if let Some(multiplier) = request.backoff_multiplier {
            config.backoff_multiplier = multiplier;
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

/// Job submission response
#[derive(Serialize, Debug)]
pub struct SubmitJobResponse {
    pub job_id: Uuid,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
    pub message: String,
}

/// Job status response
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

/// Job result response
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

/// Queue statistics response
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

/// Worker pool statistics response
#[derive(Serialize, Debug)]
pub struct WorkerPoolStatsResponse {
    pub total_workers: usize,
    pub healthy_workers: usize,
    pub total_jobs_processed: u64,
    pub total_jobs_failed: u64,
    pub is_running: bool,
}

/// Request for scheduled job creation
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

/// Scheduled job response
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

/// Query parameters for job listing
#[derive(Deserialize, Debug)]
pub struct JobListQuery {
    pub status: Option<String>,
    pub job_type: Option<String>,
    pub limit: Option<usize>,
}

/// Submit a job to the worker queue
pub async fn submit_job(
    State(state): State<AppState>,
    Json(request): Json<SubmitJobRequest>,
) -> Result<Json<SubmitJobResponse>, StatusCode> {
    // Convert request to job
    let job_type = JobType::from(request.job_type);
    let mut job = if let Some(scheduled_at) = request.scheduled_at {
        Job::scheduled(job_type, scheduled_at)
    } else {
        Job::new(job_type)
    };

    // Set optional fields
    if let Some(priority) = request.priority {
        job.priority = priority;
    }
    if let Some(retry_config) = request.retry_config {
        job.retry_config = riptide_workers::RetryConfig::from(retry_config);
    }
    if let Some(metadata) = request.metadata {
        job.metadata = metadata;
    }
    if let Some(timeout) = request.timeout_secs {
        job.timeout_secs = Some(timeout);
    }

    // For now, we'll simulate job submission since WorkerService integration needs more work
    // In a complete implementation, you would have WorkerService as part of AppState
    let job_id = job.id;

    tracing::info!(
        job_id = %job_id,
        job_type = ?job.job_type,
        priority = ?job.priority,
        "Job submitted via API"
    );

    Ok(Json(SubmitJobResponse {
        job_id,
        status: "submitted".to_string(),
        submitted_at: Utc::now(),
        message: "Job submitted successfully".to_string(),
    }))
}

/// Get job status by ID
pub async fn get_job_status(
    State(_state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<JobStatusResponse>, StatusCode> {
    tracing::info!(job_id = %job_id, "Getting job status");

    // For now, return a mock response
    // In a complete implementation, you would query the WorkerService
    Ok(Json(JobStatusResponse {
        job_id,
        status: JobStatus::Pending,
        created_at: Utc::now(),
        started_at: None,
        completed_at: None,
        worker_id: None,
        retry_count: 0,
        last_error: None,
        processing_time_ms: None,
        metadata: HashMap::new(),
    }))
}

/// Get job result by ID
pub async fn get_job_result(
    State(_state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<JobResultResponse>, StatusCode> {
    tracing::info!(job_id = %job_id, "Getting job result");

    // For now, return a mock response
    Ok(Json(JobResultResponse {
        job_id,
        success: true,
        data: Some(serde_json::json!({"mock": "result"})),
        error: None,
        processing_time_ms: 1000,
        worker_id: "worker-0".to_string(),
        completed_at: Utc::now(),
    }))
}

/// Get queue statistics
pub async fn get_queue_stats(
    State(_state): State<AppState>,
) -> Result<Json<QueueStatsResponse>, StatusCode> {
    tracing::info!("Getting queue statistics");

    // For now, return mock statistics
    Ok(Json(QueueStatsResponse {
        pending: 5,
        processing: 2,
        completed: 100,
        failed: 3,
        retry: 1,
        delayed: 0,
        total: 111,
    }))
}

/// Get worker pool statistics
pub async fn get_worker_stats(
    State(_state): State<AppState>,
) -> Result<Json<WorkerPoolStatsResponse>, StatusCode> {
    tracing::info!("Getting worker pool statistics");

    // For now, return mock statistics
    Ok(Json(WorkerPoolStatsResponse {
        total_workers: 4,
        healthy_workers: 4,
        total_jobs_processed: 100,
        total_jobs_failed: 3,
        is_running: true,
    }))
}

/// Create a scheduled job
pub async fn create_scheduled_job(
    State(_state): State<AppState>,
    Json(request): Json<CreateScheduledJobRequest>,
) -> Result<Json<ScheduledJobResponse>, StatusCode> {
    tracing::info!(
        name = %request.name,
        cron_expression = %request.cron_expression,
        "Creating scheduled job"
    );

    // Convert request to scheduled job
    let job_type = JobType::from(request.job_template);
    let scheduled_job = match ScheduledJob::new(request.name.clone(), request.cron_expression.clone(), job_type) {
        Ok(mut job) => {
            if let Some(priority) = request.priority {
                job.priority = priority;
            }
            if let Some(enabled) = request.enabled {
                job.enabled = enabled;
            }
            if let Some(retry_config) = request.retry_config {
                job.retry_config = riptide_workers::RetryConfig::from(retry_config);
            }
            if let Some(metadata) = request.metadata {
                job.metadata = metadata;
            }
            job
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create scheduled job");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    Ok(Json(ScheduledJobResponse {
        id: scheduled_job.id,
        name: scheduled_job.name,
        cron_expression: scheduled_job.cron_expression,
        enabled: scheduled_job.enabled,
        priority: scheduled_job.priority,
        created_at: scheduled_job.created_at,
        last_executed_at: scheduled_job.last_executed_at,
        next_execution_at: scheduled_job.next_execution_at,
        execution_count: scheduled_job.execution_count,
    }))
}

/// List scheduled jobs
pub async fn list_scheduled_jobs(
    State(_state): State<AppState>,
) -> Result<Json<Vec<ScheduledJobResponse>>, StatusCode> {
    tracing::info!("Listing scheduled jobs");

    // For now, return empty list
    Ok(Json(vec![]))
}

/// Delete a scheduled job
pub async fn delete_scheduled_job(
    State(_state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    tracing::info!(job_id = %job_id, "Deleting scheduled job");

    // For now, always return success
    Ok(StatusCode::NO_CONTENT)
}

/// Get comprehensive worker metrics
pub async fn get_worker_metrics(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    tracing::info!("Getting worker metrics");

    // For now, return mock metrics
    let metrics = serde_json::json!({
        "jobs_submitted": 105,
        "jobs_completed": 100,
        "jobs_failed": 3,
        "jobs_retried": 2,
        "jobs_dead_letter": 1,
        "avg_processing_time_ms": 1500,
        "p95_processing_time_ms": 3000,
        "p99_processing_time_ms": 5000,
        "uptime_seconds": 7200,
        "success_rate": 95.2,
        "total_workers": 4,
        "healthy_workers": 4,
        "jobs_per_second": 0.014,
        "timestamp": Utc::now()
    });

    Ok(Json(metrics))
}