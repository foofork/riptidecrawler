use crate::job::{Job, JobResult, JobStatus};
use anyhow::{Context, Result};
use chrono::Utc;
use redis::{aio::MultiplexedConnection, AsyncCommands};
use serde_json;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Redis-backed job queue with priority support
#[derive(Clone)]
pub struct JobQueue {
    /// Redis connection manager
    redis: MultiplexedConnection,
    /// Queue configuration
    config: QueueConfig,
    /// In-memory job tracking for fast access
    job_cache: std::sync::Arc<RwLock<HashMap<Uuid, Job>>>,
}

/// Configuration for job queue behavior
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// Namespace prefix for Redis keys
    pub namespace: String,
    /// Maximum number of jobs to keep in memory cache
    pub cache_size: usize,
    /// Polling interval for delayed jobs in seconds
    pub delayed_job_poll_interval: u64,
    /// Maximum time to hold a job lease in seconds
    pub job_lease_timeout: u64,
    /// Enable job result persistence
    pub persist_results: bool,
    /// Result TTL in seconds
    pub result_ttl: u64,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            namespace: "riptide_jobs".to_string(),
            cache_size: 1000,
            delayed_job_poll_interval: 30,
            job_lease_timeout: 600, // 10 minutes
            persist_results: true,
            result_ttl: 3600, // 1 hour
        }
    }
}

impl JobQueue {
    /// Create a new job queue instance
    pub async fn new(redis_url: &str, config: QueueConfig) -> Result<Self> {
        info!("Connecting to Redis at {}", redis_url);
        let client = redis::Client::open(redis_url)
            .context("Failed to create Redis client")?;

        let redis = client.get_multiplexed_async_connection()
            .await
            .context("Failed to create Redis connection manager")?;

        info!("Successfully connected to Redis for job queue");

        Ok(Self {
            redis,
            config,
            job_cache: std::sync::Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Submit a job to the queue
    pub async fn submit_job(&mut self, job: Job) -> Result<Uuid> {
        let job_id = job.id;
        let priority_score = job.priority as i64;

        info!(
            job_id = %job_id,
            job_type = ?job.job_type,
            priority = ?job.priority,
            "Submitting job to queue"
        );

        // Store job data in Redis hash
        let job_key = format!("{}:job:{}", self.config.namespace, job_id);
        let job_json = serde_json::to_string(&job)
            .context("Failed to serialize job")?;

        // Add to priority queue
        let queue_key = if job.scheduled_at.is_some() {
            format!("{}:delayed", self.config.namespace)
        } else {
            format!("{}:pending", self.config.namespace)
        };

        // Use Redis transaction to ensure atomicity
        let mut pipe = redis::pipe();
        pipe.atomic()
            .hset(&job_key, "data", &job_json)
            .hset(&job_key, "status", "pending")
            .hset(&job_key, "created_at", job.created_at.timestamp())
            .zadd(&queue_key, job_id.to_string(), priority_score);

        if let Some(scheduled_at) = job.scheduled_at {
            // Use timestamp as score for delayed jobs
            pipe.zadd(
                format!("{}:scheduled", self.config.namespace),
                job_id.to_string(),
                scheduled_at.timestamp()
            );
        }

        pipe.query_async(&mut self.redis)
            .await
            .context("Failed to submit job to Redis")?;

        // Update memory cache
        {
            let mut cache = self.job_cache.write().await;
            if cache.len() >= self.config.cache_size {
                // Remove oldest entry if cache is full
                if let Some(oldest_id) = cache.keys().next().copied() {
                    cache.remove(&oldest_id);
                }
            }
            cache.insert(job_id, job);
        }

        info!(job_id = %job_id, "Job submitted successfully");
        Ok(job_id)
    }

    /// Get the next available job for processing
    pub async fn next_job(&mut self, worker_id: &str) -> Result<Option<Job>, anyhow::Error> {
        debug!(worker_id = worker_id, "Looking for next job");

        // First, check for ready delayed jobs and move them to pending
        self.process_delayed_jobs().await?;

        // Get highest priority job from pending queue
        let pending_key = format!("{}:pending", self.config.namespace);
        let job_ids: Vec<String> = self.redis
            .zrevrange(&pending_key, 0, 0)
            .await
            .context("Failed to get job from pending queue")?;

        if let Some(job_id_str) = job_ids.first() {
            let job_id = Uuid::parse_str(job_id_str)
                .context("Failed to parse job ID")?;

            // Try to acquire lease on the job
            if let Some(mut job) = self.acquire_job_lease(job_id, worker_id).await? {
                // Remove from pending queue
                self.redis
                    .zrem(&pending_key, job_id.to_string())
                    .await
                    .context("Failed to remove job from pending queue")?;

                // Mark as processing
                job.start(worker_id.to_string());
                self.update_job_status(&job).await?;

                info!(
                    job_id = %job_id,
                    worker_id = worker_id,
                    "Acquired job for processing"
                );

                return Ok(Some(job));
            }
        }

        debug!(worker_id = worker_id, "No jobs available");
        Ok(None)
    }

    /// Complete a job successfully
    pub async fn complete_job(&mut self, job_id: Uuid, result: JobResult) -> Result<(), anyhow::Error> {
        info!(job_id = %job_id, worker_id = %result.worker_id, "Completing job");

        if let Some(mut job) = self.get_job(job_id).await? {
            job.complete();
            self.update_job_status(&job).await?;

            // Store result if configured
            if self.config.persist_results {
                let result_key = format!("{}:result:{}", self.config.namespace, job_id);
                let result_json = serde_json::to_string(&result)
                    .context("Failed to serialize job result")?;

                // Use SET EX command instead of deprecated setex
                redis::cmd("SET")
                    .arg(&result_key)
                    .arg(&result_json)
                    .arg("EX")
                    .arg(self.config.result_ttl)
                    .query_async(&mut self.redis)
                    .await
                    .context("Failed to store job result")?;
            }

            // Remove from processing and add to completed
            self.move_job_to_completed(job_id).await?;

            info!(job_id = %job_id, "Job completed successfully");
        } else {
            warn!(job_id = %job_id, "Attempted to complete non-existent job");
        }

        Ok(())
    }

    /// Fail a job and handle retry logic
    pub async fn fail_job(&mut self, job_id: Uuid, error: String) -> Result<(), anyhow::Error> {
        info!(job_id = %job_id, error = %error, "Failing job");

        if let Some(mut job) = self.get_job(job_id).await? {
            job.fail(error);

            match job.status {
                JobStatus::Retrying => {
                    // Move to retry queue with scheduled time
                    let retry_key = format!("{}:retry", self.config.namespace);
                    let retry_score = job.next_retry_at.unwrap().timestamp();

                    self.redis
                        .zadd(&retry_key, job_id.to_string(), retry_score)
                        .await
                        .context("Failed to add job to retry queue")?;

                    info!(
                        job_id = %job_id,
                        retry_count = job.retry_count,
                        next_retry = ?job.next_retry_at,
                        "Job scheduled for retry"
                    );
                }
                JobStatus::DeadLetter => {
                    // Move to dead letter queue
                    let dead_letter_key = format!("{}:dead_letter", self.config.namespace);
                    self.redis
                        .zadd(&dead_letter_key, job_id.to_string(), Utc::now().timestamp())
                        .await
                        .context("Failed to add job to dead letter queue")?;

                    error!(
                        job_id = %job_id,
                        retry_count = job.retry_count,
                        "Job moved to dead letter queue after max retries"
                    );
                }
                _ => {}
            }

            self.update_job_status(&job).await?;
        }

        Ok(())
    }

    /// Get job by ID
    pub async fn get_job(&mut self, job_id: Uuid) -> Result<Option<Job>, anyhow::Error> {
        // Check cache first
        {
            let cache = self.job_cache.read().await;
            if let Some(job) = cache.get(&job_id) {
                return Ok(Some(job.clone()));
            }
        }

        // Load from Redis
        let job_key = format!("{}:job:{}", self.config.namespace, job_id);
        let job_data: Option<String> = self.redis
            .hget(&job_key, "data")
            .await
            .context("Failed to get job from Redis")?;

        if let Some(job_json) = job_data {
            let job: Job = serde_json::from_str(&job_json)
                .context("Failed to deserialize job")?;

            // Update cache
            {
                let mut cache = self.job_cache.write().await;
                cache.insert(job_id, job.clone());
            }

            Ok(Some(job))
        } else {
            Ok(None)
        }
    }

    /// Get job result
    pub async fn get_job_result(&mut self, job_id: Uuid) -> Result<Option<JobResult>, anyhow::Error> {
        let result_key = format!("{}:result:{}", self.config.namespace, job_id);
        let result_data: Option<String> = self.redis
            .get(&result_key)
            .await
            .context("Failed to get job result from Redis")?;

        if let Some(result_json) = result_data {
            let result: JobResult = serde_json::from_str(&result_json)
                .context("Failed to deserialize job result")?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    /// Get queue statistics
    pub async fn get_stats(&mut self) -> Result<QueueStats, anyhow::Error> {
        let pending_count = self.get_queue_size("pending").await?;
        let processing_count = self.get_queue_size("processing").await?;
        let completed_count = self.get_queue_size("completed").await?;
        let failed_count = self.get_queue_size("dead_letter").await?;
        let retry_count = self.get_queue_size("retry").await?;
        let delayed_count = self.get_queue_size("scheduled").await?;

        Ok(QueueStats {
            pending: pending_count,
            processing: processing_count,
            completed: completed_count,
            failed: failed_count,
            retry: retry_count,
            delayed: delayed_count,
            total: pending_count + processing_count + completed_count + failed_count + retry_count + delayed_count,
        })
    }

    /// Process delayed jobs and move ready ones to pending
    async fn process_delayed_jobs(&mut self) -> Result<(), anyhow::Error> {
        let now = Utc::now().timestamp();
        let scheduled_key = format!("{}:scheduled", self.config.namespace);
        let retry_key = format!("{}:retry", self.config.namespace);
        let pending_key = format!("{}:pending", self.config.namespace);

        // Check scheduled jobs
        let ready_scheduled: Vec<String> = self.redis
            .zrangebyscore(&scheduled_key, 0, now)
            .await
            .context("Failed to get ready scheduled jobs")?;

        for job_id_str in ready_scheduled {
            if let Ok(job_id) = Uuid::parse_str(&job_id_str) {
                if let Some(job) = self.get_job(job_id).await? {
                    // Move to pending with priority
                    let priority_score = job.priority as i64;

                    let mut pipe = redis::pipe();
                    pipe.atomic()
                        .zrem(&scheduled_key, &job_id_str)
                        .zadd(&pending_key, &job_id_str, priority_score);

                    pipe.query_async(&mut self.redis).await?;

                    debug!(job_id = %job_id, "Moved scheduled job to pending queue");
                }
            }
        }

        // Check retry jobs
        let ready_retries: Vec<String> = self.redis
            .zrangebyscore(&retry_key, 0, now)
            .await
            .context("Failed to get ready retry jobs")?;

        for job_id_str in ready_retries {
            if let Ok(job_id) = Uuid::parse_str(&job_id_str) {
                if let Some(job) = self.get_job(job_id).await? {
                    let priority_score = job.priority as i64;

                    let mut pipe = redis::pipe();
                    pipe.atomic()
                        .zrem(&retry_key, &job_id_str)
                        .zadd(&pending_key, &job_id_str, priority_score);

                    pipe.query_async(&mut self.redis).await?;

                    debug!(job_id = %job_id, "Moved retry job to pending queue");
                }
            }
        }

        Ok(())
    }

    /// Acquire a lease on a job for exclusive processing
    async fn acquire_job_lease(&mut self, job_id: Uuid, worker_id: &str) -> Result<Option<Job>, anyhow::Error> {
        let lease_key = format!("{}:lease:{}", self.config.namespace, job_id);
        let lease_acquired: bool = self.redis
            .set_nx(&lease_key, worker_id)
            .await
            .context("Failed to acquire job lease")?;

        if lease_acquired {
            // Set lease expiration
            self.redis
                .expire(&lease_key, self.config.job_lease_timeout as i64)
                .await
                .context("Failed to set lease expiration")?;

            // Get the job
            self.get_job(job_id).await
        } else {
            // Job is already being processed by another worker
            Ok(None)
        }
    }

    /// Update job status in Redis
    async fn update_job_status(&mut self, job: &Job) -> Result<(), anyhow::Error> {
        let job_key = format!("{}:job:{}", self.config.namespace, job.id);
        let job_json = serde_json::to_string(job)
            .context("Failed to serialize job")?;

        self.redis
            .hset(&job_key, "data", job_json)
            .await
            .context("Failed to update job in Redis")?;

        // Update cache
        {
            let mut cache = self.job_cache.write().await;
            cache.insert(job.id, job.clone());
        }

        Ok(())
    }

    /// Move job to completed queue
    async fn move_job_to_completed(&mut self, job_id: Uuid) -> Result<(), anyhow::Error> {
        let processing_key = format!("{}:processing", self.config.namespace);
        let completed_key = format!("{}:completed", self.config.namespace);
        let lease_key = format!("{}:lease:{}", self.config.namespace, job_id);

        let mut pipe = redis::pipe();
        pipe.atomic()
            .zrem(&processing_key, job_id.to_string())
            .zadd(&completed_key, job_id.to_string(), Utc::now().timestamp())
            .del(&lease_key);

        pipe.query_async(&mut self.redis)
            .await
            .context("Failed to move job to completed queue")?;

        Ok(())
    }

    /// Get queue size for a specific queue
    async fn get_queue_size(&mut self, queue_name: &str) -> Result<usize, anyhow::Error> {
        let queue_key = format!("{}:{}", self.config.namespace, queue_name);
        let size: usize = self.redis
            .zcard(&queue_key)
            .await
            .context("Failed to get queue size")?;
        Ok(size)
    }
}

/// Queue statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueueStats {
    pub pending: usize,
    pub processing: usize,
    pub completed: usize,
    pub failed: usize,
    pub retry: usize,
    pub delayed: usize,
    pub total: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::JobType;

    #[tokio::test]
    async fn test_queue_config_default() {
        let config = QueueConfig::default();
        assert_eq!(config.namespace, "riptide_jobs");
        assert_eq!(config.cache_size, 1000);
    }

    #[tokio::test]
    async fn test_job_creation_and_submission() {
        // This test would require a Redis instance for full integration testing
        // For now, we test the basic structure
        let job_type = JobType::SingleCrawl {
            url: "https://example.com".to_string(),
            options: None,
        };
        let job = Job::new(job_type);
        assert_eq!(job.status, JobStatus::Pending);
    }
}