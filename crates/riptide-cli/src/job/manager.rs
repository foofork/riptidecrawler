#![allow(dead_code)]

use super::storage::JobStorage;
use super::types::{Job, JobId, JobPriority, JobStatus, LogEntry, LogLevel};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Job manager for orchestrating job lifecycle
pub struct JobManager {
    /// Job storage backend
    storage: JobStorage,
    /// Active jobs cache (job_id -> job)
    active_jobs: Arc<RwLock<HashMap<JobId, Job>>>,
}

impl JobManager {
    /// Create a new job manager
    pub fn new() -> Result<Self> {
        let storage = JobStorage::new()?;
        let active_jobs = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            storage,
            active_jobs,
        })
    }

    /// Submit a new job
    pub async fn submit_job(
        &self,
        urls: Vec<String>,
        strategy: String,
        name: Option<String>,
        priority: JobPriority,
        tags: Vec<String>,
        stream: bool,
    ) -> Result<JobId> {
        let job = Job::new(urls, strategy, name, priority, tags, stream);
        let job_id = job.id.clone();

        // Save job to storage
        self.storage.save_job(&job).context("Failed to save job")?;

        // Add to active jobs cache
        let mut active = self.active_jobs.write().await;
        active.insert(job_id.clone(), job.clone());

        // Log job submission
        self.log_job(
            &job_id,
            LogLevel::Info,
            format!("Job submitted with {} URLs", job.urls.len()),
        )
        .await?;

        Ok(job_id)
    }

    /// Get a job by ID
    pub async fn get_job(&self, job_id: &JobId) -> Result<Job> {
        // Try cache first
        let active = self.active_jobs.read().await;
        if let Some(job) = active.get(job_id) {
            return Ok(job.clone());
        }
        drop(active);

        // Load from storage
        let job = self
            .storage
            .load_job(job_id)
            .context(format!("Job not found: {}", job_id))?;

        // Update cache
        let mut active = self.active_jobs.write().await;
        active.insert(job_id.clone(), job.clone());

        Ok(job)
    }

    /// List all jobs with optional filters
    pub async fn list_jobs(
        &self,
        status_filter: Option<JobStatus>,
        priority_filter: Option<JobPriority>,
        tag_filter: Option<String>,
        limit: Option<usize>,
    ) -> Result<Vec<Job>> {
        let job_ids = self.storage.list_jobs()?;
        let mut jobs = Vec::new();

        for job_id in job_ids {
            if let Ok(job) = self.storage.load_job(&job_id) {
                // Apply filters
                if let Some(ref status) = status_filter {
                    if &job.status != status {
                        continue;
                    }
                }

                if let Some(ref priority) = priority_filter {
                    if &job.priority != priority {
                        continue;
                    }
                }

                if let Some(ref tag) = tag_filter {
                    if !job.tags.contains(tag) {
                        continue;
                    }
                }

                jobs.push(job);
            }
        }

        // Sort by creation time (newest first)
        jobs.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Apply limit
        if let Some(limit) = limit {
            jobs.truncate(limit);
        }

        Ok(jobs)
    }

    /// Start job execution
    pub async fn start_job(&self, job_id: &JobId) -> Result<()> {
        let mut job = self.get_job(job_id).await?;
        job.start();
        self.update_job(&job).await?;

        self.log_job(job_id, LogLevel::Info, "Job started".to_string())
            .await?;

        Ok(())
    }

    /// Update job progress
    pub async fn update_progress(
        &self,
        job_id: &JobId,
        completed: u32,
        failed: u32,
        current_url: Option<String>,
    ) -> Result<()> {
        let mut job = self.get_job(job_id).await?;
        job.update_progress(completed, failed);

        if let Some(url) = current_url {
            job.progress.set_current(url);
        } else {
            job.progress.clear_current();
        }

        self.update_job(&job).await?;
        Ok(())
    }

    /// Complete a job successfully
    pub async fn complete_job(&self, job_id: &JobId) -> Result<()> {
        let mut job = self.get_job(job_id).await?;
        job.complete();
        self.update_job(&job).await?;

        self.log_job(
            job_id,
            LogLevel::Info,
            format!(
                "Job completed successfully in {:.2}s",
                job.duration_secs().unwrap_or(0.0)
            ),
        )
        .await?;

        // Remove from active jobs cache
        let mut active = self.active_jobs.write().await;
        active.remove(job_id);

        Ok(())
    }

    /// Fail a job with error message
    pub async fn fail_job(&self, job_id: &JobId, error: String) -> Result<()> {
        let mut job = self.get_job(job_id).await?;
        job.fail(error.clone());
        self.update_job(&job).await?;

        self.log_job(job_id, LogLevel::Error, format!("Job failed: {}", error))
            .await?;

        // Remove from active jobs cache
        let mut active = self.active_jobs.write().await;
        active.remove(job_id);

        Ok(())
    }

    /// Cancel a running job
    pub async fn cancel_job(&self, job_id: &JobId) -> Result<()> {
        let mut job = self.get_job(job_id).await?;

        if job.is_terminal() {
            anyhow::bail!("Cannot cancel a job that is already completed/failed/cancelled");
        }

        job.cancel();
        self.update_job(&job).await?;

        self.log_job(job_id, LogLevel::Warn, "Job cancelled by user".to_string())
            .await?;

        // Remove from active jobs cache
        let mut active = self.active_jobs.write().await;
        active.remove(job_id);

        Ok(())
    }

    /// Delete a job from storage
    pub async fn delete_job(&self, job_id: &JobId) -> Result<()> {
        self.storage.delete_job(job_id)?;

        let mut active = self.active_jobs.write().await;
        active.remove(job_id);

        Ok(())
    }

    /// Update job in storage and cache
    async fn update_job(&self, job: &Job) -> Result<()> {
        self.storage.save_job(job)?;

        let mut active = self.active_jobs.write().await;
        active.insert(job.id.clone(), job.clone());

        Ok(())
    }

    /// Log a message for a job
    pub async fn log_job(&self, job_id: &JobId, level: LogLevel, message: String) -> Result<()> {
        let entry = LogEntry::new(level, message);
        self.storage.append_log(job_id, &entry)?;
        Ok(())
    }

    /// Log a message with URL context
    pub async fn log_job_url(
        &self,
        job_id: &JobId,
        level: LogLevel,
        message: String,
        url: String,
    ) -> Result<()> {
        let entry = LogEntry::with_url(level, message, url);
        self.storage.append_log(job_id, &entry)?;
        Ok(())
    }

    /// Read job logs
    pub async fn read_logs(
        &self,
        job_id: &JobId,
        lines: Option<usize>,
        level_filter: Option<&str>,
    ) -> Result<Vec<LogEntry>> {
        self.storage.read_logs(job_id, lines, level_filter)
    }

    /// Save job results
    pub async fn save_results(&self, job_id: &JobId, results: &serde_json::Value) -> Result<()> {
        self.storage.save_results(job_id, results)?;

        // Update job results path
        let mut job = self.get_job(job_id).await?;
        job.results_path = Some(
            self.storage
                .base_dir()
                .join(job_id.as_str())
                .join("results.json")
                .to_string_lossy()
                .to_string(),
        );
        self.update_job(&job).await?;

        Ok(())
    }

    /// Load job results
    pub async fn load_results(&self, job_id: &JobId) -> Result<serde_json::Value> {
        self.storage.load_results(job_id)
    }

    /// Get job statistics
    pub async fn get_stats(&self) -> Result<JobStats> {
        let jobs = self.list_jobs(None, None, None, None).await?;

        let total = jobs.len();
        let mut by_status = HashMap::new();
        let mut by_priority = HashMap::new();
        let mut total_duration = 0.0;
        let mut completed_count = 0;

        for job in &jobs {
            *by_status.entry(job.status.to_string()).or_insert(0) += 1;
            *by_priority.entry(job.priority.to_string()).or_insert(0) += 1;

            if let Some(duration) = job.duration_secs() {
                total_duration += duration;
                completed_count += 1;
            }
        }

        let avg_duration = if completed_count > 0 {
            total_duration / completed_count as f64
        } else {
            0.0
        };

        let completed = by_status.get("completed").copied().unwrap_or(0);
        let failed = by_status.get("failed").copied().unwrap_or(0);
        let success_rate = if completed + failed > 0 {
            completed as f64 / (completed + failed) as f64
        } else {
            0.0
        };

        Ok(JobStats {
            total_jobs: total,
            by_status,
            by_priority,
            avg_duration_secs: avg_duration,
            success_rate,
        })
    }

    /// Clean up old completed jobs
    pub async fn cleanup_old_jobs(&self, days: u32) -> Result<Vec<JobId>> {
        let deleted = self.storage.cleanup_old_jobs(days)?;

        // Remove from cache
        let mut active = self.active_jobs.write().await;
        for job_id in &deleted {
            active.remove(job_id);
        }

        Ok(deleted)
    }

    /// Get storage statistics
    pub fn get_storage_stats(&self) -> Result<super::storage::StorageStats> {
        self.storage.get_stats()
    }
}

impl Default for JobManager {
    fn default() -> Self {
        Self::new().expect("Failed to create job manager")
    }
}

/// Job statistics
#[derive(Debug)]
pub struct JobStats {
    pub total_jobs: usize,
    pub by_status: HashMap<String, usize>,
    pub by_priority: HashMap<String, usize>,
    pub avg_duration_secs: f64,
    pub success_rate: f64,
}
