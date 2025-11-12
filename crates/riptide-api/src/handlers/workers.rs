//! Ultra-thin workers API handlers (Phase 3 Sprint 3.1)
//!
//! All business logic delegated to WorkersFacade.
//! Handlers are <50 LOC total, focused only on HTTP transport concerns.
//! Phase 1: Workers are optional - handlers return SERVICE_UNAVAILABLE if workers disabled.

use crate::{context::ApplicationContext, dto::workers::*};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use uuid::Uuid;

/// Check if workers are enabled, return 503 if not
fn check_workers_enabled(
    worker_service: &Option<std::sync::Arc<dyn riptide_types::ports::WorkerService>>,
) -> Result<&std::sync::Arc<dyn riptide_types::ports::WorkerService>, StatusCode> {
    worker_service
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)
}

/// Submit a job (ultra-thin - 5 LOC)
pub async fn submit_job(
    State(state): State<ApplicationContext>,
    Json(request): Json<SubmitJobRequest>,
) -> Result<Json<SubmitJobResponse>, StatusCode> {
    let worker_service = check_workers_enabled(&state.worker_service)?;
    let job = request.into_job().map_err(|_| StatusCode::BAD_REQUEST)?;
    let job_type = format_job_type(&job.job_type);
    let job_id = worker_service
        .submit_job(job)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    // Phase D: Worker metrics now tracked via business_metrics
    state
        .business_metrics
        .record_worker_job_submitted(&job_type);
    Ok(Json(SubmitJobResponse::new(job_id)))
}

/// Get job status (ultra-thin - 3 LOC)
pub async fn get_job_status(
    State(state): State<ApplicationContext>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<JobStatusResponse>, StatusCode> {
    let worker_service = check_workers_enabled(&state.worker_service)?;
    let job = worker_service
        .get_job(job_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(JobStatusResponse::from(&job)))
}

/// Get job result (ultra-thin - 3 LOC)
pub async fn get_job_result(
    State(state): State<ApplicationContext>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<JobResultResponse>, StatusCode> {
    let worker_service = check_workers_enabled(&state.worker_service)?;
    let result = worker_service
        .get_job_result(job_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
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

/// Get queue stats (ultra-thin - 3 LOC)
pub async fn get_queue_stats(
    State(state): State<ApplicationContext>,
) -> Result<Json<QueueStatsResponse>, StatusCode> {
    let worker_service = check_workers_enabled(&state.worker_service)?;
    let s = worker_service
        .get_queue_stats()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(QueueStatsResponse {
        pending: s.pending,
        processing: s.processing,
        completed: s.completed,
        failed: s.failed,
        retry: s.retry,
        delayed: s.delayed,
        total: s.pending + s.processing + s.completed + s.failed + s.retry + s.delayed,
    }))
}

/// Get worker stats (ultra-thin - 3 LOC)
pub async fn get_worker_stats(
    State(state): State<ApplicationContext>,
) -> Result<Json<WorkerPoolStatsResponse>, StatusCode> {
    let worker_service = check_workers_enabled(&state.worker_service)?;
    let s = worker_service
        .get_worker_stats()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    // Phase D: Worker stats now tracked via business_metrics
    // record_worker_pool_stats takes: active, idle, queue_depth
    // We map: total_workers=active+idle, healthy_workers=active, queue_depth=0 (not tracked here)
    let active = s.healthy_workers;
    let idle = s.total_workers.saturating_sub(s.healthy_workers);
    state.business_metrics.record_worker_pool_stats(
        active, idle, 0, // queue_depth not available in WorkerPoolStats
    );
    Ok(Json(WorkerPoolStatsResponse {
        total_workers: s.total_workers,
        healthy_workers: s.healthy_workers,
        total_jobs_processed: s.total_jobs_processed,
        total_jobs_failed: s.total_jobs_failed,
        is_running: s.is_running,
    }))
}

/// Create scheduled job (ultra-thin - 6 LOC)
pub async fn create_scheduled_job(
    State(state): State<ApplicationContext>,
    Json(request): Json<CreateScheduledJobRequest>,
) -> Result<Json<ScheduledJobResponse>, StatusCode> {
    let worker_service = check_workers_enabled(&state.worker_service)?;
    let mut job = riptide_workers::ScheduledJob::new(
        request.name.clone(),
        request.cron_expression.clone(),
        request.job_template.into(),
    )
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    if let Some(p) = request.priority {
        job.priority = p;
    }
    if let Some(e) = request.enabled {
        job.enabled = e;
    }
    if let Some(r) = request.retry_config {
        job.retry_config = r.into();
    }
    if let Some(m) = request.metadata {
        job.metadata = m;
    }
    let job_id = worker_service
        .add_scheduled_job(job.clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let jobs = worker_service
        .list_scheduled_jobs()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let job = jobs
        .into_iter()
        .find(|j| j.id == job_id)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ScheduledJobResponse::from(&job)))
}

/// List scheduled jobs (ultra-thin - 2 LOC)
pub async fn list_scheduled_jobs(
    State(state): State<ApplicationContext>,
) -> Result<Json<Vec<ScheduledJobResponse>>, StatusCode> {
    let worker_service = check_workers_enabled(&state.worker_service)?;
    let jobs = worker_service
        .list_scheduled_jobs()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(jobs.iter().map(ScheduledJobResponse::from).collect()))
}

/// Delete scheduled job (ultra-thin - 2 LOC)
pub async fn delete_scheduled_job(
    State(state): State<ApplicationContext>,
    Path(job_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let worker_service = check_workers_enabled(&state.worker_service)?;
    let deleted = worker_service
        .remove_scheduled_job(job_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Get worker metrics (ultra-thin - 3 LOC)
pub async fn get_worker_metrics(
    State(state): State<ApplicationContext>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let worker_service = check_workers_enabled(&state.worker_service)?;
    let m = worker_service.get_metrics().await;
    // Phase D: Worker metrics tracked via business_metrics (detailed breakdown)
    // record_worker_metrics takes: worker_id, jobs_completed, avg_duration_ms
    // We use "aggregate" as worker_id since we're tracking all workers
    state.business_metrics.record_worker_metrics(
        "aggregate",
        m.jobs_completed,
        m.avg_processing_time_ms as f64,
    );
    Ok(Json(
        serde_json::json!({ "jobs_submitted": m.jobs_submitted, "jobs_completed": m.jobs_completed, "jobs_failed": m.jobs_failed, "jobs_retried": m.jobs_retried, "jobs_dead_letter": m.jobs_dead_letter, "avg_processing_time_ms": m.avg_processing_time_ms, "p95_processing_time_ms": m.p95_processing_time_ms, "p99_processing_time_ms": m.p99_processing_time_ms, "success_rate": m.success_rate, "job_type_stats": m.job_type_stats, "queue_sizes": m.queue_sizes, "total_workers": m.total_workers, "healthy_workers": m.healthy_workers, "uptime_seconds": m.uptime_seconds, "timestamp": m.timestamp }),
    ))
}

/// List jobs (ultra-thin - 4 LOC)
pub async fn list_jobs(
    State(state): State<ApplicationContext>,
    Query(q): Query<JobListQuery>,
) -> Result<Json<JobListResponse>, (StatusCode, String)> {
    let worker_service = check_workers_enabled(&state.worker_service)
        .map_err(|e| (e, "Worker service not available".to_string()))?;
    let limit = q.limit.unwrap_or(50).min(500);
    let offset = q.offset.unwrap_or(0);
    let jobs = worker_service
        .list_jobs(
            q.status.as_deref(),
            q.job_type.as_deref(),
            q.search.as_deref(),
            limit,
            offset,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed: {}", e)))?;
    let items: Vec<JobListItem> = jobs.iter().map(JobListItem::from_job).collect();
    Ok(Json(JobListResponse {
        jobs: items.clone(),
        total: items.len(),
        limit,
        offset,
    }))
}

/// Helper function to format job type for metrics
fn format_job_type(job_type: &riptide_workers::JobType) -> String {
    match job_type {
        riptide_workers::JobType::SingleCrawl => "single_crawl".to_string(),
        riptide_workers::JobType::BatchCrawl => "batch_crawl".to_string(),
        riptide_workers::JobType::Maintenance => "maintenance".to_string(),
        riptide_workers::JobType::Custom(name) => format!("custom_{}", name),
    }
}
