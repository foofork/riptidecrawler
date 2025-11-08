//! Ultra-thin workers API handlers (Phase 3 Sprint 3.1)
//!
//! All business logic delegated to WorkersFacade.
//! Handlers are <50 LOC total, focused only on HTTP transport concerns.

use crate::{dto::workers::*, state::AppState};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use uuid::Uuid;

/// Submit a job (ultra-thin - 5 LOC)
pub async fn submit_job(
    State(state): State<AppState>,
    Json(request): Json<SubmitJobRequest>,
) -> Result<Json<SubmitJobResponse>, StatusCode> {
    let job = request.into_job().map_err(|_| StatusCode::BAD_REQUEST)?;
    let job_id = state
        .worker_service
        .submit_job(job)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.metrics.record_worker_job_submission();
    Ok(Json(SubmitJobResponse::new(job_id)))
}

/// Get job status (ultra-thin - 3 LOC)
pub async fn get_job_status(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<JobStatusResponse>, StatusCode> {
    let job = state
        .worker_service
        .get_job(job_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(JobStatusResponse::from(&job)))
}

/// Get job result (ultra-thin - 3 LOC)
pub async fn get_job_result(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<JobResultResponse>, StatusCode> {
    let result = state
        .worker_service
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
    State(state): State<AppState>,
) -> Result<Json<QueueStatsResponse>, StatusCode> {
    let s = state
        .worker_service
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
    State(state): State<AppState>,
) -> Result<Json<WorkerPoolStatsResponse>, StatusCode> {
    let s = state
        .worker_service
        .get_worker_stats()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    state.metrics.update_worker_stats(&s);
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
    State(state): State<AppState>,
    Json(request): Json<CreateScheduledJobRequest>,
) -> Result<Json<ScheduledJobResponse>, StatusCode> {
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
    let job_id = state
        .worker_service
        .add_scheduled_job(job.clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let jobs = state
        .worker_service
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
    State(state): State<AppState>,
) -> Result<Json<Vec<ScheduledJobResponse>>, StatusCode> {
    let jobs = state
        .worker_service
        .list_scheduled_jobs()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(jobs.iter().map(ScheduledJobResponse::from).collect()))
}

/// Delete scheduled job (ultra-thin - 2 LOC)
pub async fn delete_scheduled_job(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let deleted = state
        .worker_service
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
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let m = state.worker_service.get_metrics().await;
    state.metrics.update_worker_metrics(&m);
    Ok(Json(
        serde_json::json!({ "jobs_submitted": m.jobs_submitted, "jobs_completed": m.jobs_completed, "jobs_failed": m.jobs_failed, "jobs_retried": m.jobs_retried, "jobs_dead_letter": m.jobs_dead_letter, "avg_processing_time_ms": m.avg_processing_time_ms, "p95_processing_time_ms": m.p95_processing_time_ms, "p99_processing_time_ms": m.p99_processing_time_ms, "success_rate": m.success_rate, "job_type_stats": m.job_type_stats, "queue_sizes": m.queue_sizes, "total_workers": m.total_workers, "healthy_workers": m.healthy_workers, "uptime_seconds": m.uptime_seconds, "timestamp": m.timestamp }),
    ))
}

/// List jobs (ultra-thin - 4 LOC)
pub async fn list_jobs(
    State(state): State<AppState>,
    Query(q): Query<JobListQuery>,
) -> Result<Json<JobListResponse>, (StatusCode, String)> {
    let limit = q.limit.unwrap_or(50).min(500);
    let offset = q.offset.unwrap_or(0);
    let jobs = state
        .worker_service
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
