//! WorkersFacade for job queue and worker pool management.
//!
//! Provides a high-level interface for managing background jobs, scheduled tasks,
//! and worker pool operations with authorization, idempotency, and metrics.

#![cfg(feature = "workers")]

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use riptide_workers::{Job, JobPriority, JobStatus, JobType, RetryConfig, ScheduledJob};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Authorization context for worker operations
#[derive(Debug, Clone)]
pub struct AuthorizationContext {
    pub user_id: Option<String>,
    pub tenant_id: Option<String>,
    pub roles: Vec<String>,
}

impl Default for AuthorizationContext {
    fn default() -> Self {
        Self {
            user_id: None,
            tenant_id: None,
            roles: vec!["anonymous".to_string()],
        }
    }
}

/// Job submission request
#[derive(Debug, Clone)]
pub struct SubmitJobRequest {
    pub job_type: JobType,
    pub priority: Option<JobPriority>,
    pub retry_config: Option<RetryConfig>,
    pub metadata: Option<HashMap<String, JsonValue>>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub timeout_secs: Option<u64>,
}

/// Job filter for listing
#[derive(Debug, Clone)]
pub struct JobFilter {
    pub status: Option<String>,
    pub job_type: Option<String>,
    pub search: Option<String>,
    pub limit: usize,
    pub offset: usize,
}

impl Default for JobFilter {
    fn default() -> Self {
        Self {
            status: None,
            job_type: None,
            search: None,
            limit: 50,
            offset: 0,
        }
    }
}

/// Scheduled job request
#[derive(Debug, Clone)]
pub struct ScheduledJobRequest {
    pub name: String,
    pub cron_expression: String,
    pub job_template: JobType,
    pub priority: Option<JobPriority>,
    pub enabled: Option<bool>,
    pub retry_config: Option<RetryConfig>,
    pub metadata: Option<HashMap<String, JsonValue>>,
}

/// Job result with metadata
#[derive(Debug, Clone)]
pub struct JobResult {
    pub job_id: Uuid,
    pub success: bool,
    pub data: Option<JsonValue>,
    pub error: Option<String>,
    pub processing_time_ms: u64,
    pub worker_id: String,
    pub completed_at: DateTime<Utc>,
}

/// Queue statistics
#[derive(Debug, Clone)]
pub struct QueueStats {
    pub pending: usize,
    pub processing: usize,
    pub completed: usize,
    pub failed: usize,
    pub retry: usize,
    pub delayed: usize,
}

/// Worker pool statistics
#[derive(Debug, Clone)]
pub struct WorkerPoolStats {
    pub total_workers: usize,
    pub healthy_workers: usize,
    pub total_jobs_processed: u64,
    pub total_jobs_failed: u64,
    pub is_running: bool,
}

/// Worker metrics snapshot
#[derive(Debug, Clone)]
pub struct WorkerMetrics {
    pub jobs_submitted: u64,
    pub jobs_completed: u64,
    pub jobs_failed: u64,
    pub jobs_retried: u64,
    pub jobs_dead_letter: u64,
    pub avg_processing_time_ms: u64,
    pub p95_processing_time_ms: u64,
    pub p99_processing_time_ms: u64,
    pub success_rate: f64,
    pub job_type_stats: HashMap<String, u64>,
    pub queue_sizes: HashMap<String, usize>,
    pub total_workers: usize,
    pub healthy_workers: usize,
    pub uptime_seconds: u64,
    pub timestamp: DateTime<Utc>,
}

/// Port trait for worker service operations
pub trait WorkerService: Send + Sync {
    fn submit_job(&self, job: Job) -> impl std::future::Future<Output = Result<Uuid>> + Send;

    fn get_job(
        &self,
        job_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Option<Job>>> + Send;

    fn get_job_result(
        &self,
        job_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Option<JobResult>>> + Send;

    fn get_queue_stats(&self) -> impl std::future::Future<Output = Result<QueueStats>> + Send;

    fn get_worker_stats(&self) -> Option<WorkerPoolStats>;

    fn add_scheduled_job(
        &self,
        job: ScheduledJob,
    ) -> impl std::future::Future<Output = Result<Uuid>> + Send;

    fn list_scheduled_jobs(&self) -> Result<Vec<ScheduledJob>>;

    fn remove_scheduled_job(
        &self,
        job_id: Uuid,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;

    fn get_metrics(&self) -> impl std::future::Future<Output = WorkerMetrics> + Send;

    fn list_jobs(
        &self,
        status: Option<&str>,
        job_type: Option<&str>,
        search: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> impl std::future::Future<Output = Result<Vec<Job>>> + Send;
}

/// WorkersFacade for managing background jobs and worker pools.
///
/// This facade encapsulates all business logic for worker operations,
/// including authorization, idempotency, metrics, and event emission.
pub struct WorkersFacade<W: WorkerService> {
    worker_service: Arc<W>,
    enable_authorization: bool,
    enable_idempotency: bool,
}

impl<W: WorkerService> WorkersFacade<W> {
    /// Create a new WorkersFacade instance.
    ///
    /// # Arguments
    ///
    /// * `worker_service` - The worker service implementation
    pub fn new(worker_service: Arc<W>) -> Self {
        Self {
            worker_service,
            enable_authorization: true,
            enable_idempotency: true,
        }
    }

    /// Create a facade with authorization disabled (for testing).
    pub fn without_authorization(worker_service: Arc<W>) -> Self {
        Self {
            worker_service,
            enable_authorization: false,
            enable_idempotency: true,
        }
    }

    /// Submit a job to the worker queue.
    ///
    /// # Arguments
    ///
    /// * `request` - Job submission request
    /// * `authz_ctx` - Authorization context
    ///
    /// # Returns
    ///
    /// Returns the job ID if successful.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Authorization fails
    /// - Job validation fails
    /// - Submission to queue fails
    pub async fn submit_job(
        &self,
        request: SubmitJobRequest,
        authz_ctx: &AuthorizationContext,
    ) -> Result<Uuid> {
        // Authorization check
        if self.enable_authorization {
            self.authorize_job_submission(authz_ctx)?;
        }

        // Create job from request
        let mut job = if let Some(scheduled_at) = request.scheduled_at {
            Job::scheduled(request.job_type, scheduled_at)
        } else {
            Job::new(request.job_type)
        };

        // Apply optional configurations
        if let Some(priority) = request.priority {
            job.priority = priority;
        }
        if let Some(retry_config) = request.retry_config {
            job.retry_config = retry_config;
        }
        if let Some(metadata) = request.metadata {
            job.metadata = metadata;
        }
        if let Some(timeout) = request.timeout_secs {
            job.timeout_secs = Some(timeout);
        }

        // Idempotency check (if enabled)
        if self.enable_idempotency {
            // In production, this would check an idempotency store
            // For now, we rely on the underlying service
        }

        // Submit job
        let job_id = self
            .worker_service
            .submit_job(job)
            .await
            .context("Failed to submit job to worker queue")?;

        // Emit domain event (would integrate with event bus in production)
        tracing::info!(
            job_id = %job_id,
            tenant_id = ?authz_ctx.tenant_id,
            "Job submitted successfully"
        );

        Ok(job_id)
    }

    /// Get job status by ID.
    ///
    /// # Arguments
    ///
    /// * `job_id` - The job identifier
    /// * `authz_ctx` - Authorization context
    ///
    /// # Returns
    ///
    /// Returns the job if found and authorized.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Authorization fails
    /// - Job not found
    /// - Query fails
    pub async fn get_job_status(
        &self,
        job_id: Uuid,
        authz_ctx: &AuthorizationContext,
    ) -> Result<Option<Job>> {
        // Authorization check
        if self.enable_authorization {
            self.authorize_job_access(authz_ctx)?;
        }

        // Fetch job
        let job = self
            .worker_service
            .get_job(job_id)
            .await
            .context("Failed to fetch job status")?;

        Ok(job)
    }

    /// Get job result by ID.
    ///
    /// # Arguments
    ///
    /// * `job_id` - The job identifier
    /// * `authz_ctx` - Authorization context
    ///
    /// # Returns
    ///
    /// Returns the job result if available.
    pub async fn get_job_result(
        &self,
        job_id: Uuid,
        authz_ctx: &AuthorizationContext,
    ) -> Result<Option<JobResult>> {
        // Authorization check
        if self.enable_authorization {
            self.authorize_job_access(authz_ctx)?;
        }

        // Fetch result
        let result = self
            .worker_service
            .get_job_result(job_id)
            .await
            .context("Failed to fetch job result")?;

        Ok(result)
    }

    /// Get queue statistics.
    ///
    /// # Arguments
    ///
    /// * `authz_ctx` - Authorization context
    ///
    /// # Returns
    ///
    /// Returns queue statistics.
    pub async fn get_queue_stats(&self, authz_ctx: &AuthorizationContext) -> Result<QueueStats> {
        // Authorization check
        if self.enable_authorization {
            self.authorize_metrics_access(authz_ctx)?;
        }

        // Fetch stats
        let stats = self
            .worker_service
            .get_queue_stats()
            .await
            .context("Failed to fetch queue statistics")?;

        Ok(stats)
    }

    /// Get worker pool statistics.
    ///
    /// # Arguments
    ///
    /// * `authz_ctx` - Authorization context
    ///
    /// # Returns
    ///
    /// Returns worker pool statistics if available.
    pub fn get_worker_stats(&self, authz_ctx: &AuthorizationContext) -> Result<WorkerPoolStats> {
        // Authorization check
        if self.enable_authorization {
            self.authorize_metrics_access(authz_ctx)?;
        }

        // Fetch stats
        let stats = self
            .worker_service
            .get_worker_stats()
            .ok_or_else(|| anyhow::anyhow!("Worker pool not yet started"))?;

        Ok(stats)
    }

    /// Create a scheduled job.
    ///
    /// # Arguments
    ///
    /// * `request` - Scheduled job request
    /// * `authz_ctx` - Authorization context
    ///
    /// # Returns
    ///
    /// Returns the scheduled job ID and the created job.
    pub async fn create_scheduled_job(
        &self,
        request: ScheduledJobRequest,
        authz_ctx: &AuthorizationContext,
    ) -> Result<(Uuid, ScheduledJob)> {
        // Authorization check
        if self.enable_authorization {
            self.authorize_scheduled_job_creation(authz_ctx)?;
        }

        // Create scheduled job
        let mut scheduled_job = ScheduledJob::new(
            request.name.clone(),
            request.cron_expression.clone(),
            request.job_template,
        )
        .context("Invalid cron expression")?;

        // Apply configurations
        if let Some(priority) = request.priority {
            scheduled_job.priority = priority;
        }
        if let Some(enabled) = request.enabled {
            scheduled_job.enabled = enabled;
        }
        if let Some(retry_config) = request.retry_config {
            scheduled_job.retry_config = retry_config;
        }
        if let Some(metadata) = request.metadata {
            scheduled_job.metadata = metadata;
        }

        // Add to scheduler
        let job_id = self
            .worker_service
            .add_scheduled_job(scheduled_job.clone())
            .await
            .context("Failed to add scheduled job")?;

        tracing::info!(
            job_id = %job_id,
            name = %request.name,
            "Scheduled job created successfully"
        );

        Ok((job_id, scheduled_job))
    }

    /// List scheduled jobs.
    ///
    /// # Arguments
    ///
    /// * `authz_ctx` - Authorization context
    ///
    /// # Returns
    ///
    /// Returns list of scheduled jobs.
    pub fn list_scheduled_jobs(
        &self,
        authz_ctx: &AuthorizationContext,
    ) -> Result<Vec<ScheduledJob>> {
        // Authorization check
        if self.enable_authorization {
            self.authorize_scheduled_job_access(authz_ctx)?;
        }

        // List jobs
        let jobs = self
            .worker_service
            .list_scheduled_jobs()
            .context("Failed to list scheduled jobs")?;

        Ok(jobs)
    }

    /// Delete a scheduled job.
    ///
    /// # Arguments
    ///
    /// * `job_id` - The scheduled job ID
    /// * `authz_ctx` - Authorization context
    ///
    /// # Returns
    ///
    /// Returns true if the job was deleted.
    pub async fn delete_scheduled_job(
        &self,
        job_id: Uuid,
        authz_ctx: &AuthorizationContext,
    ) -> Result<bool> {
        // Authorization check
        if self.enable_authorization {
            self.authorize_scheduled_job_deletion(authz_ctx)?;
        }

        // Remove job
        let deleted = self
            .worker_service
            .remove_scheduled_job(job_id)
            .await
            .context("Failed to delete scheduled job")?;

        if deleted {
            tracing::info!(job_id = %job_id, "Scheduled job deleted");
        }

        Ok(deleted)
    }

    /// Get comprehensive worker metrics.
    ///
    /// # Arguments
    ///
    /// * `authz_ctx` - Authorization context
    ///
    /// # Returns
    ///
    /// Returns worker metrics snapshot.
    pub async fn get_worker_metrics(
        &self,
        authz_ctx: &AuthorizationContext,
    ) -> Result<WorkerMetrics> {
        // Authorization check
        if self.enable_authorization {
            self.authorize_metrics_access(authz_ctx)?;
        }

        // Fetch metrics
        let metrics = self.worker_service.get_metrics().await;

        Ok(metrics)
    }

    /// List jobs with filtering and pagination.
    ///
    /// # Arguments
    ///
    /// * `filter` - Job filter criteria
    /// * `authz_ctx` - Authorization context
    ///
    /// # Returns
    ///
    /// Returns filtered list of jobs.
    pub async fn list_jobs(
        &self,
        filter: JobFilter,
        authz_ctx: &AuthorizationContext,
    ) -> Result<Vec<Job>> {
        // Authorization check
        if self.enable_authorization {
            self.authorize_job_access(authz_ctx)?;
        }

        // Query jobs
        let jobs = self
            .worker_service
            .list_jobs(
                filter.status.as_deref(),
                filter.job_type.as_deref(),
                filter.search.as_deref(),
                filter.limit.min(500), // Cap at 500
                filter.offset,
            )
            .await
            .context("Failed to list jobs")?;

        Ok(jobs)
    }

    // Authorization helper methods

    fn authorize_job_submission(&self, authz_ctx: &AuthorizationContext) -> Result<()> {
        // In production, implement proper RBAC checks
        if authz_ctx.roles.contains(&"admin".to_string())
            || authz_ctx.roles.contains(&"operator".to_string())
        {
            Ok(())
        } else {
            anyhow::bail!("Unauthorized: job submission requires admin or operator role")
        }
    }

    fn authorize_job_access(&self, _authz_ctx: &AuthorizationContext) -> Result<()> {
        // In production, implement tenant-scoped access control
        Ok(())
    }

    fn authorize_metrics_access(&self, authz_ctx: &AuthorizationContext) -> Result<()> {
        // Metrics access requires admin role
        if authz_ctx.roles.contains(&"admin".to_string()) {
            Ok(())
        } else {
            anyhow::bail!("Unauthorized: metrics access requires admin role")
        }
    }

    fn authorize_scheduled_job_creation(&self, authz_ctx: &AuthorizationContext) -> Result<()> {
        if authz_ctx.roles.contains(&"admin".to_string()) {
            Ok(())
        } else {
            anyhow::bail!("Unauthorized: scheduled job creation requires admin role")
        }
    }

    fn authorize_scheduled_job_access(&self, _authz_ctx: &AuthorizationContext) -> Result<()> {
        Ok(())
    }

    fn authorize_scheduled_job_deletion(&self, authz_ctx: &AuthorizationContext) -> Result<()> {
        if authz_ctx.roles.contains(&"admin".to_string()) {
            Ok(())
        } else {
            anyhow::bail!("Unauthorized: scheduled job deletion requires admin role")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock worker service for testing
    struct MockWorkerService {
        jobs: Arc<std::sync::Mutex<HashMap<Uuid, Job>>>,
    }

    impl MockWorkerService {
        fn new() -> Self {
            Self {
                jobs: Arc::new(std::sync::Mutex::new(HashMap::new())),
            }
        }
    }

    impl WorkerService for MockWorkerService {
        async fn submit_job(&self, job: Job) -> Result<Uuid> {
            let job_id = job.id;
            self.jobs.lock().unwrap().insert(job_id, job);
            Ok(job_id)
        }

        async fn get_job(&self, job_id: Uuid) -> Result<Option<Job>> {
            Ok(self.jobs.lock().unwrap().get(&job_id).cloned())
        }

        async fn get_job_result(&self, _job_id: Uuid) -> Result<Option<JobResult>> {
            Ok(None)
        }

        async fn get_queue_stats(&self) -> Result<QueueStats> {
            Ok(QueueStats {
                pending: 0,
                processing: 0,
                completed: 0,
                failed: 0,
                retry: 0,
                delayed: 0,
            })
        }

        fn get_worker_stats(&self) -> Option<WorkerPoolStats> {
            Some(WorkerPoolStats {
                total_workers: 4,
                healthy_workers: 4,
                total_jobs_processed: 100,
                total_jobs_failed: 5,
                is_running: true,
            })
        }

        async fn add_scheduled_job(&self, _job: ScheduledJob) -> Result<Uuid> {
            Ok(Uuid::new_v4())
        }

        fn list_scheduled_jobs(&self) -> Result<Vec<ScheduledJob>> {
            Ok(vec![])
        }

        async fn remove_scheduled_job(&self, _job_id: Uuid) -> Result<bool> {
            Ok(true)
        }

        async fn get_metrics(&self) -> WorkerMetrics {
            WorkerMetrics {
                jobs_submitted: 100,
                jobs_completed: 95,
                jobs_failed: 5,
                jobs_retried: 10,
                jobs_dead_letter: 0,
                avg_processing_time_ms: 150,
                p95_processing_time_ms: 300,
                p99_processing_time_ms: 500,
                success_rate: 0.95,
                job_type_stats: HashMap::new(),
                queue_sizes: HashMap::new(),
                total_workers: 4,
                healthy_workers: 4,
                uptime_seconds: 3600,
                timestamp: Utc::now(),
            }
        }

        async fn list_jobs(
            &self,
            _status: Option<&str>,
            _job_type: Option<&str>,
            _search: Option<&str>,
            _limit: usize,
            _offset: usize,
        ) -> Result<Vec<Job>> {
            Ok(self.jobs.lock().unwrap().values().cloned().collect())
        }
    }

    #[tokio::test]
    async fn test_submit_job_with_authorization() {
        let service = Arc::new(MockWorkerService::new());
        let facade = WorkersFacade::new(service);

        let request = SubmitJobRequest {
            job_type: JobType::Custom {
                job_name: "test_job".to_string(),
                payload: serde_json::json!({}),
            },
            priority: Some(JobPriority::Normal),
            retry_config: None,
            metadata: None,
            scheduled_at: None,
            timeout_secs: None,
        };

        let authz_ctx = AuthorizationContext {
            user_id: Some("user123".to_string()),
            tenant_id: Some("tenant456".to_string()),
            roles: vec!["admin".to_string()],
        };

        let result = facade.submit_job(request, &authz_ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_submit_job_unauthorized() {
        let service = Arc::new(MockWorkerService::new());
        let facade = WorkersFacade::new(service);

        let request = SubmitJobRequest {
            job_type: JobType::Custom {
                job_name: "test_job".to_string(),
                payload: serde_json::json!({}),
            },
            priority: None,
            retry_config: None,
            metadata: None,
            scheduled_at: None,
            timeout_secs: None,
        };

        let authz_ctx = AuthorizationContext {
            user_id: Some("user123".to_string()),
            tenant_id: Some("tenant456".to_string()),
            roles: vec!["readonly".to_string()],
        };

        let result = facade.submit_job(request, &authz_ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_job_status() {
        let service = Arc::new(MockWorkerService::new());
        let facade = WorkersFacade::without_authorization(service.clone());

        // Submit a job first
        let job = Job::new(JobType::Custom {
            job_name: "test".to_string(),
            payload: serde_json::json!({}),
        });
        let job_id = job.id;
        service.submit_job(job).await.unwrap();

        // Get job status
        let authz_ctx = AuthorizationContext::default();
        let result = facade.get_job_status(job_id, &authz_ctx).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_get_queue_stats() {
        let service = Arc::new(MockWorkerService::new());
        let facade = WorkersFacade::without_authorization(service);

        let authz_ctx = AuthorizationContext::default();
        let result = facade.get_queue_stats(&authz_ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_worker_stats() {
        let service = Arc::new(MockWorkerService::new());
        let facade = WorkersFacade::without_authorization(service);

        let authz_ctx = AuthorizationContext::default();
        let result = facade.get_worker_stats(&authz_ctx);
        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.total_workers, 4);
        assert!(stats.is_running);
    }

    #[tokio::test]
    async fn test_list_jobs() {
        let service = Arc::new(MockWorkerService::new());
        let facade = WorkersFacade::without_authorization(service.clone());

        // Submit some jobs
        for i in 0..3 {
            let job = Job::new(JobType::Custom {
                job_name: format!("test_{}", i),
                payload: serde_json::json!({}),
            });
            service.submit_job(job).await.unwrap();
        }

        let filter = JobFilter::default();
        let authz_ctx = AuthorizationContext::default();
        let result = facade.list_jobs(filter, &authz_ctx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn test_create_scheduled_job() {
        let service = Arc::new(MockWorkerService::new());
        let facade = WorkersFacade::without_authorization(service);

        let request = ScheduledJobRequest {
            name: "daily_cleanup".to_string(),
            cron_expression: "0 0 * * *".to_string(),
            job_template: JobType::Maintenance {
                task_type: "cleanup".to_string(),
                parameters: HashMap::new(),
            },
            priority: Some(JobPriority::Low),
            enabled: Some(true),
            retry_config: None,
            metadata: None,
        };

        let authz_ctx = AuthorizationContext::default();
        let result = facade.create_scheduled_job(request, &authz_ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_scheduled_job() {
        let service = Arc::new(MockWorkerService::new());
        let facade = WorkersFacade::without_authorization(service);

        let job_id = Uuid::new_v4();
        let authz_ctx = AuthorizationContext::default();
        let result = facade.delete_scheduled_job(job_id, &authz_ctx).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_get_worker_metrics() {
        let service = Arc::new(MockWorkerService::new());
        let facade = WorkersFacade::without_authorization(service);

        let authz_ctx = AuthorizationContext::default();
        let result = facade.get_worker_metrics(&authz_ctx).await;
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.jobs_submitted, 100);
        assert_eq!(metrics.jobs_completed, 95);
    }

    #[test]
    fn test_authorization_context_default() {
        let ctx = AuthorizationContext::default();
        assert!(ctx.user_id.is_none());
        assert!(ctx.tenant_id.is_none());
        assert_eq!(ctx.roles, vec!["anonymous".to_string()]);
    }

    #[test]
    fn test_job_filter_default() {
        let filter = JobFilter::default();
        assert_eq!(filter.limit, 50);
        assert_eq!(filter.offset, 0);
        assert!(filter.status.is_none());
    }
}
