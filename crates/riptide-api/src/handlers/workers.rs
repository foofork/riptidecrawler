use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use riptide_workers::{Job, JobPriority, JobStatus, JobType, ScheduledJob};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::state::AppState;

// Note: All handlers are actively used via routes in main.rs
// No #[allow(dead_code)] needed as they are properly wired

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
    pub offset: Option<usize>,
    pub search: Option<String>,
}

/// Job listing response
#[derive(Serialize, Debug)]
pub struct JobListResponse {
    pub jobs: Vec<JobListItem>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

/// Individual job item in list response
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

    // Submit to WorkerService
    let job_id = state.worker_service.submit_job(job).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to submit job");
        state.metrics.record_error(crate::metrics::ErrorType::Redis);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Record job submission in Prometheus (Phase 4B Feature 5)
    state.metrics.record_worker_job_submission();

    tracing::info!(
        job_id = %job_id,
        "Job submitted successfully via API"
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
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<JobStatusResponse>, StatusCode> {
    tracing::info!(job_id = %job_id, "Getting job status");

    let job = state
        .worker_service
        .get_job(job_id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, job_id = %job_id, "Failed to get job status");
            state.metrics.record_error(crate::metrics::ErrorType::Redis);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!(job_id = %job_id, "Job not found");
            StatusCode::NOT_FOUND
        })?;

    // Calculate processing time if job has started
    let processing_time_ms =
        if let (Some(started), Some(completed)) = (job.started_at, job.completed_at) {
            Some((completed - started).num_milliseconds() as u64)
        } else {
            job.started_at
                .map(|started| (Utc::now() - started).num_milliseconds() as u64)
        };

    Ok(Json(JobStatusResponse {
        job_id: job.id,
        status: job.status,
        created_at: job.created_at,
        started_at: job.started_at,
        completed_at: job.completed_at,
        worker_id: job.worker_id,
        retry_count: job.retry_count,
        last_error: job.last_error,
        processing_time_ms,
        metadata: job.metadata,
    }))
}

/// Get job result by ID
pub async fn get_job_result(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<JobResultResponse>, StatusCode> {
    tracing::info!(job_id = %job_id, "Getting job result");

    let result = state
        .worker_service
        .get_job_result(job_id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, job_id = %job_id, "Failed to get job result");
            state.metrics.record_error(crate::metrics::ErrorType::Redis);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!(job_id = %job_id, "Job result not found");
            StatusCode::NOT_FOUND
        })?;

    Ok(Json(JobResultResponse {
        job_id: result.job_id,
        success: result.success,
        data: result.data,
        error: result.error,
        processing_time_ms: result.processing_time_ms,
        worker_id: result.worker_id,
        completed_at: result.completed_at,
    }))
}

/// Get queue statistics
pub async fn get_queue_stats(
    State(state): State<AppState>,
) -> Result<Json<QueueStatsResponse>, StatusCode> {
    tracing::info!("Getting queue statistics");

    let stats = state.worker_service.get_queue_stats().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get queue stats");
        state.metrics.record_error(crate::metrics::ErrorType::Redis);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(QueueStatsResponse {
        pending: stats.pending,
        processing: stats.processing,
        completed: stats.completed,
        failed: stats.failed,
        retry: stats.retry,
        delayed: stats.delayed,
        total: stats.pending
            + stats.processing
            + stats.completed
            + stats.failed
            + stats.retry
            + stats.delayed,
    }))
}

/// Get worker pool statistics
pub async fn get_worker_stats(
    State(state): State<AppState>,
) -> Result<Json<WorkerPoolStatsResponse>, StatusCode> {
    tracing::info!("Getting worker pool statistics");

    let stats = state.worker_service.get_worker_stats().ok_or_else(|| {
        tracing::warn!("Worker pool not yet started");
        StatusCode::SERVICE_UNAVAILABLE
    })?;

    // Update Prometheus metrics with current worker stats (Phase 4B Feature 5)
    state.metrics.update_worker_stats(&stats);

    Ok(Json(WorkerPoolStatsResponse {
        total_workers: stats.total_workers,
        healthy_workers: stats.healthy_workers,
        total_jobs_processed: stats.total_jobs_processed,
        total_jobs_failed: stats.total_jobs_failed,
        is_running: stats.is_running,
    }))
}

/// Create a scheduled job
pub async fn create_scheduled_job(
    State(state): State<AppState>,
    Json(request): Json<CreateScheduledJobRequest>,
) -> Result<Json<ScheduledJobResponse>, StatusCode> {
    tracing::info!(
        name = %request.name,
        cron_expression = %request.cron_expression,
        "Creating scheduled job"
    );

    // Convert request to scheduled job
    let job_type = JobType::from(request.job_template);
    let mut scheduled_job = match ScheduledJob::new(
        request.name.clone(),
        request.cron_expression.clone(),
        job_type,
    ) {
        Ok(job) => job,
        Err(e) => {
            tracing::error!(error = %e, "Failed to create scheduled job");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Apply configurations
    if let Some(priority) = request.priority {
        scheduled_job.priority = priority;
    }
    if let Some(enabled) = request.enabled {
        scheduled_job.enabled = enabled;
    }
    if let Some(retry_config) = request.retry_config {
        scheduled_job.retry_config = riptide_workers::RetryConfig::from(retry_config);
    }
    if let Some(metadata) = request.metadata {
        scheduled_job.metadata = metadata;
    }

    // Add to scheduler
    let job_id = state
        .worker_service
        .add_scheduled_job(scheduled_job)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to add scheduled job");
            state.metrics.record_error(crate::metrics::ErrorType::Redis);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Retrieve created job for response
    let jobs = state.worker_service.list_scheduled_jobs().map_err(|e| {
        tracing::error!(error = %e, "Failed to retrieve scheduled job");
        state.metrics.record_error(crate::metrics::ErrorType::Redis);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let scheduled_job = jobs
        .into_iter()
        .find(|j| j.id == job_id)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

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
    State(state): State<AppState>,
) -> Result<Json<Vec<ScheduledJobResponse>>, StatusCode> {
    tracing::info!("Listing scheduled jobs");

    let jobs = state.worker_service.list_scheduled_jobs().map_err(|e| {
        tracing::error!(error = %e, "Failed to list scheduled jobs");
        state.metrics.record_error(crate::metrics::ErrorType::Redis);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let responses: Vec<ScheduledJobResponse> = jobs
        .into_iter()
        .map(|job| ScheduledJobResponse {
            id: job.id,
            name: job.name,
            cron_expression: job.cron_expression,
            enabled: job.enabled,
            priority: job.priority,
            created_at: job.created_at,
            last_executed_at: job.last_executed_at,
            next_execution_at: job.next_execution_at,
            execution_count: job.execution_count,
        })
        .collect();

    Ok(Json(responses))
}

/// Delete a scheduled job
pub async fn delete_scheduled_job(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    tracing::info!(job_id = %job_id, "Deleting scheduled job");

    let deleted = state
        .worker_service
        .remove_scheduled_job(job_id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, job_id = %job_id, "Failed to delete scheduled job");
            state.metrics.record_error(crate::metrics::ErrorType::Redis);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        tracing::warn!(job_id = %job_id, "Scheduled job not found");
        Err(StatusCode::NOT_FOUND)
    }
}

/// Get comprehensive worker metrics
pub async fn get_worker_metrics(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    tracing::info!("Getting worker metrics");

    let metrics = state.worker_service.get_metrics().await;

    // Update Prometheus metrics with comprehensive worker data (Phase 4B Feature 5)
    state.metrics.update_worker_metrics(&metrics);

    let json = serde_json::json!({
        "jobs_submitted": metrics.jobs_submitted,
        "jobs_completed": metrics.jobs_completed,
        "jobs_failed": metrics.jobs_failed,
        "jobs_retried": metrics.jobs_retried,
        "jobs_dead_letter": metrics.jobs_dead_letter,
        "avg_processing_time_ms": metrics.avg_processing_time_ms,
        "p95_processing_time_ms": metrics.p95_processing_time_ms,
        "p99_processing_time_ms": metrics.p99_processing_time_ms,
        "success_rate": metrics.success_rate,
        "job_type_stats": metrics.job_type_stats,
        "queue_sizes": metrics.queue_sizes,
        "total_workers": metrics.total_workers,
        "healthy_workers": metrics.healthy_workers,
        "uptime_seconds": metrics.uptime_seconds,
        "timestamp": metrics.timestamp
    });

    Ok(Json(json))
}

/// List jobs with filtering and pagination
pub async fn list_jobs(
    State(state): State<AppState>,
    Query(query): Query<JobListQuery>,
) -> Result<Json<JobListResponse>, (StatusCode, String)> {
    tracing::info!(
        status = ?query.status,
        job_type = ?query.job_type,
        limit = ?query.limit,
        offset = ?query.offset,
        search = ?query.search,
        "Listing jobs"
    );

    // Set defaults for limit and offset
    let limit = query.limit.unwrap_or(50).min(500); // Cap at 500
    let offset = query.offset.unwrap_or(0);

    // List jobs from worker service
    let jobs = state
        .worker_service
        .list_jobs(
            query.status.as_deref(),
            query.job_type.as_deref(),
            query.search.as_deref(),
            limit,
            offset,
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to list jobs");
            state.metrics.record_error(crate::metrics::ErrorType::Redis);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to list jobs: {}", e),
            )
        })?;

    // Convert jobs to response items
    let job_items: Vec<JobListItem> = jobs
        .iter()
        .map(|job| {
            let job_type_str = match &job.job_type {
                JobType::BatchCrawl { .. } => "batch_crawl".to_string(),
                JobType::SingleCrawl { .. } => "single_crawl".to_string(),
                JobType::PdfExtraction { .. } => "pdf_extraction".to_string(),
                JobType::Maintenance { task_type, .. } => {
                    format!("maintenance:{}", task_type)
                }
                JobType::Custom { job_name, .. } => format!("custom:{}", job_name),
            };

            JobListItem {
                job_id: job.id,
                job_type: job_type_str,
                status: job.status.clone(),
                priority: job.priority,
                created_at: job.created_at,
                started_at: job.started_at,
                completed_at: job.completed_at,
                worker_id: job.worker_id.clone(),
                retry_count: job.retry_count,
            }
        })
        .collect();

    let total = job_items.len();

    Ok(Json(JobListResponse {
        jobs: job_items,
        total,
        limit,
        offset,
    }))
}
