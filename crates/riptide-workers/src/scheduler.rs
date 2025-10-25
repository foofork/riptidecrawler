use crate::job::{Job, JobType};
use crate::queue::JobQueue;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use cron::Schedule;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Scheduled job definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    /// Unique identifier for the scheduled job
    pub id: Uuid,
    /// Human-readable name for the job
    pub name: String,
    /// Cron expression for scheduling
    pub cron_expression: String,
    /// Job type and payload to execute
    pub job_template: JobType,
    /// Whether this scheduled job is enabled
    pub enabled: bool,
    /// Job priority for queue ordering
    pub priority: crate::job::JobPriority,
    /// Retry configuration for spawned jobs
    pub retry_config: crate::job::RetryConfig,
    /// Maximum execution time for spawned jobs
    pub timeout_secs: Option<u64>,
    /// Metadata for spawned jobs
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last execution timestamp
    pub last_executed_at: Option<DateTime<Utc>>,
    /// Next execution timestamp
    pub next_execution_at: Option<DateTime<Utc>>,
    /// Number of jobs spawned by this schedule
    pub execution_count: u64,
}

impl ScheduledJob {
    /// Create a new scheduled job
    pub fn new(name: String, cron_expression: String, job_template: JobType) -> Result<Self> {
        // Validate cron expression
        Schedule::from_str(&cron_expression).context("Invalid cron expression")?;

        let mut scheduled_job = Self {
            id: Uuid::new_v4(),
            name,
            cron_expression,
            job_template,
            enabled: true,
            priority: crate::job::JobPriority::Normal,
            retry_config: crate::job::RetryConfig::default(),
            timeout_secs: Some(600), // 10 minutes default
            metadata: std::collections::HashMap::new(),
            created_at: Utc::now(),
            last_executed_at: None,
            next_execution_at: None,
            execution_count: 0,
        };

        // Calculate next execution time
        scheduled_job.update_next_execution()?;

        Ok(scheduled_job)
    }

    /// Update the next execution time based on cron expression
    pub fn update_next_execution(&mut self) -> Result<()> {
        let schedule =
            Schedule::from_str(&self.cron_expression).context("Failed to parse cron expression")?;

        // Get next execution time from now
        if let Some(next_time) = schedule.upcoming(Utc).next() {
            self.next_execution_at = Some(next_time);
        } else {
            warn!(
                schedule_id = %self.id,
                cron_expression = %self.cron_expression,
                "No upcoming execution time found for cron expression"
            );
            self.next_execution_at = None;
        }

        Ok(())
    }

    /// Check if the job should be executed now
    pub fn should_execute_now(&self) -> bool {
        if !self.enabled {
            return false;
        }

        if let Some(next_execution) = self.next_execution_at {
            Utc::now() >= next_execution
        } else {
            false
        }
    }

    /// Create a job instance from this schedule
    pub fn create_job_instance(&self) -> Job {
        let mut job = Job::with_priority(self.job_template.clone(), self.priority);
        job.retry_config = self.retry_config.clone();
        job.timeout_secs = self.timeout_secs;

        // Add schedule metadata
        job.metadata.insert(
            "scheduled_job_id".to_string(),
            serde_json::Value::String(self.id.to_string()),
        );
        job.metadata.insert(
            "scheduled_job_name".to_string(),
            serde_json::Value::String(self.name.clone()),
        );
        job.metadata.insert(
            "execution_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(self.execution_count + 1)),
        );

        // Add custom metadata
        for (key, value) in &self.metadata {
            job.metadata.insert(key.clone(), value.clone());
        }

        job
    }

    /// Mark as executed and update next execution time
    pub fn mark_executed(&mut self) -> Result<()> {
        self.last_executed_at = Some(Utc::now());
        self.execution_count += 1;
        self.update_next_execution()
    }
}

/// Job scheduler configuration
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// How often to check for scheduled jobs (in seconds)
    pub check_interval_secs: u64,
    /// Maximum number of scheduled jobs allowed
    pub max_scheduled_jobs: usize,
    /// Whether to persist scheduled jobs
    pub persist_schedules: bool,
    /// Redis key prefix for persisting schedules
    pub redis_prefix: String,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 30,
            max_scheduled_jobs: 1000,
            persist_schedules: true,
            redis_prefix: "riptide_schedules".to_string(),
        }
    }
}

/// Job scheduler for cron-like job scheduling
pub struct JobScheduler {
    /// Scheduler configuration
    config: SchedulerConfig,
    /// Job queue for submitting scheduled jobs
    queue: Arc<tokio::sync::Mutex<JobQueue>>,
    /// Scheduled jobs storage
    scheduled_jobs: Arc<DashMap<Uuid, ScheduledJob>>,
    /// Scheduler running state
    running: Arc<AtomicBool>,
    /// Redis client for persistence (optional)
    redis_client: Option<Arc<tokio::sync::Mutex<redis::aio::MultiplexedConnection>>>,
}

impl JobScheduler {
    /// Create a new job scheduler
    pub async fn new(
        config: SchedulerConfig,
        queue: Arc<tokio::sync::Mutex<JobQueue>>,
        redis_url: Option<&str>,
    ) -> Result<Self> {
        let redis_client = if config.persist_schedules && redis_url.is_some() {
            let url = redis_url
                .ok_or_else(|| anyhow::anyhow!("Redis URL required for persisted schedules"))?;
            let client =
                redis::Client::open(url).context("Failed to create Redis client for scheduler")?;
            let connection = client
                .get_multiplexed_async_connection()
                .await
                .context("Failed to connect to Redis for scheduler")?;
            Some(Arc::new(tokio::sync::Mutex::new(connection)))
        } else {
            None
        };

        let scheduler = Self {
            config,
            queue,
            scheduled_jobs: Arc::new(DashMap::new()),
            running: Arc::new(AtomicBool::new(false)),
            redis_client,
        };

        // Load persisted schedules if Redis is available
        if scheduler.redis_client.is_some() {
            if let Err(e) = scheduler.load_persisted_schedules().await {
                warn!("Failed to load persisted schedules: {}", e);
            }
        }

        Ok(scheduler)
    }

    /// Start the scheduler
    pub async fn start(&self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            warn!("Job scheduler is already running");
            return Ok(());
        }

        info!("Starting job scheduler");
        self.running.store(true, Ordering::Relaxed);

        // Start scheduler loop
        self.run_scheduler_loop().await
    }

    /// Stop the scheduler
    pub async fn stop(&self) {
        info!("Stopping job scheduler");
        self.running.store(false, Ordering::Relaxed);
    }

    /// Add a scheduled job
    pub async fn add_scheduled_job(&self, scheduled_job: ScheduledJob) -> Result<Uuid> {
        if self.scheduled_jobs.len() >= self.config.max_scheduled_jobs {
            return Err(anyhow::anyhow!(
                "Maximum number of scheduled jobs ({}) exceeded",
                self.config.max_scheduled_jobs
            ));
        }

        let job_id = scheduled_job.id;

        info!(
            schedule_id = %job_id,
            name = %scheduled_job.name,
            cron_expression = %scheduled_job.cron_expression,
            next_execution = ?scheduled_job.next_execution_at,
            "Adding scheduled job"
        );

        // Persist if Redis is available
        if let Some(ref redis_client) = self.redis_client {
            let mut conn = redis_client.lock().await;
            self.persist_scheduled_job(&scheduled_job, &mut conn)
                .await?;
        }

        self.scheduled_jobs.insert(job_id, scheduled_job);

        info!(schedule_id = %job_id, "Scheduled job added successfully");
        Ok(job_id)
    }

    /// Remove a scheduled job
    pub async fn remove_scheduled_job(&self, job_id: Uuid) -> Result<bool> {
        info!(schedule_id = %job_id, "Removing scheduled job");

        let removed = self.scheduled_jobs.remove(&job_id).is_some();

        if removed {
            // Remove from persistence if Redis is available
            if let Some(ref redis_client) = self.redis_client {
                let mut conn = redis_client.lock().await;
                self.remove_persisted_job(job_id, &mut conn).await?;
            }
            info!(schedule_id = %job_id, "Scheduled job removed successfully");
        } else {
            warn!(schedule_id = %job_id, "Scheduled job not found");
        }

        Ok(removed)
    }

    /// Update a scheduled job
    pub async fn update_scheduled_job(&self, mut scheduled_job: ScheduledJob) -> Result<()> {
        let job_id = scheduled_job.id;

        // Update next execution time
        scheduled_job.update_next_execution()?;

        info!(
            schedule_id = %job_id,
            name = %scheduled_job.name,
            next_execution = ?scheduled_job.next_execution_at,
            "Updating scheduled job"
        );

        // Persist if Redis is available
        if let Some(ref redis_client) = self.redis_client {
            let mut conn = redis_client.lock().await;
            self.persist_scheduled_job(&scheduled_job, &mut conn)
                .await?;
        }

        self.scheduled_jobs.insert(job_id, scheduled_job);

        Ok(())
    }

    /// Get a scheduled job by ID
    pub fn get_scheduled_job(&self, job_id: Uuid) -> Option<ScheduledJob> {
        self.scheduled_jobs
            .get(&job_id)
            .map(|entry| entry.value().clone())
    }

    /// List all scheduled jobs
    pub fn list_scheduled_jobs(&self) -> Vec<ScheduledJob> {
        self.scheduled_jobs
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get scheduler statistics
    pub fn get_scheduler_stats(&self) -> SchedulerStats {
        let total_schedules = self.scheduled_jobs.len();
        let mut enabled_schedules = 0;
        let mut next_execution: Option<DateTime<Utc>> = None;

        for entry in self.scheduled_jobs.iter() {
            let schedule = entry.value();
            if schedule.enabled {
                enabled_schedules += 1;
            }

            if let Some(next_exec) = schedule.next_execution_at {
                next_execution = match next_execution {
                    Some(current) if next_exec < current => Some(next_exec),
                    None => Some(next_exec),
                    _ => next_execution,
                };
            }
        }

        SchedulerStats {
            total_scheduled_jobs: total_schedules,
            enabled_scheduled_jobs: enabled_schedules,
            next_execution_at: next_execution,
            is_running: self.running.load(Ordering::Relaxed),
        }
    }

    /// Main scheduler loop
    async fn run_scheduler_loop(&self) -> Result<()> {
        info!("Starting scheduler loop");

        while self.running.load(Ordering::Relaxed) {
            match self.check_and_execute_scheduled_jobs().await {
                Ok(executed_count) => {
                    if executed_count > 0 {
                        info!(executed_jobs = executed_count, "Executed scheduled jobs");
                    }
                }
                Err(e) => {
                    error!(error = %e, "Error in scheduler loop");
                }
            }

            // Sleep for check interval
            sleep(Duration::from_secs(self.config.check_interval_secs)).await;
        }

        info!("Scheduler loop stopped");
        Ok(())
    }

    /// Check for and execute scheduled jobs that are due
    async fn check_and_execute_scheduled_jobs(&self) -> Result<usize> {
        let mut executed_count = 0;

        // Collect jobs that need to be executed
        let mut jobs_to_execute = Vec::new();

        for mut entry in self.scheduled_jobs.iter_mut() {
            let schedule = entry.value_mut();
            if schedule.should_execute_now() {
                jobs_to_execute.push(schedule.id);
            }
        }

        // Execute each job
        for schedule_id in jobs_to_execute {
            if let Some(mut entry) = self.scheduled_jobs.get_mut(&schedule_id) {
                let schedule = entry.value_mut();

                debug!(
                    schedule_id = %schedule_id,
                    name = %schedule.name,
                    "Executing scheduled job"
                );

                // Create job instance
                let job = schedule.create_job_instance();

                // Submit to queue
                let mut queue = self.queue.lock().await;
                match queue.submit_job(job).await {
                    Ok(job_id) => {
                        info!(
                            schedule_id = %schedule_id,
                            job_id = %job_id,
                            "Scheduled job submitted to queue"
                        );

                        // Update schedule
                        if let Err(e) = schedule.mark_executed() {
                            error!(
                                schedule_id = %schedule_id,
                                error = %e,
                                "Failed to update schedule after execution"
                            );
                        } else {
                            executed_count += 1;

                            // Persist updated schedule if Redis is available
                            if let Some(ref redis_client) = self.redis_client {
                                let mut conn = redis_client.lock().await;
                                if let Err(e) =
                                    self.persist_scheduled_job(schedule, &mut conn).await
                                {
                                    warn!(
                                        schedule_id = %schedule_id,
                                        error = %e,
                                        "Failed to persist updated schedule"
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!(
                            schedule_id = %schedule_id,
                            error = %e,
                            "Failed to submit scheduled job to queue"
                        );
                    }
                }
            }
        }

        Ok(executed_count)
    }

    /// Persist a scheduled job to Redis
    async fn persist_scheduled_job(
        &self,
        scheduled_job: &ScheduledJob,
        redis_client: &mut redis::aio::MultiplexedConnection,
    ) -> Result<()> {
        use redis::AsyncCommands;

        let key = format!("{}:schedule:{}", self.config.redis_prefix, scheduled_job.id);
        let data =
            serde_json::to_string(scheduled_job).context("Failed to serialize scheduled job")?;

        redis_client
            .hset::<_, _, _, ()>(&key, "data", data)
            .await
            .context("Failed to persist scheduled job to Redis")?;

        Ok(())
    }

    /// Remove a persisted job from Redis
    async fn remove_persisted_job(
        &self,
        job_id: Uuid,
        redis_client: &mut redis::aio::MultiplexedConnection,
    ) -> Result<()> {
        use redis::AsyncCommands;

        let key = format!("{}:schedule:{}", self.config.redis_prefix, job_id);
        redis_client
            .del::<_, ()>(&key)
            .await
            .context("Failed to remove scheduled job from Redis")?;

        Ok(())
    }

    /// Load persisted schedules from Redis
    async fn load_persisted_schedules(&self) -> Result<()> {
        if let Some(ref redis_client) = self.redis_client {
            let mut conn = redis_client.lock().await;
            use redis::AsyncCommands;

            let pattern = format!("{}:schedule:*", self.config.redis_prefix);
            let keys: Vec<String> = conn
                .keys::<_, Vec<String>>(pattern)
                .await
                .context("Failed to get scheduled job keys from Redis")?;

            info!(schedule_count = keys.len(), "Loading persisted schedules");

            for key in keys {
                let data: Option<String> = conn
                    .hget::<_, _, Option<String>>(&key, "data")
                    .await
                    .context("Failed to get scheduled job data from Redis")?;

                if let Some(job_data) = data {
                    match serde_json::from_str::<ScheduledJob>(&job_data) {
                        Ok(scheduled_job) => {
                            let job_id = scheduled_job.id;
                            self.scheduled_jobs.insert(job_id, scheduled_job);
                            debug!(schedule_id = %job_id, "Loaded persisted schedule");
                        }
                        Err(e) => {
                            warn!(
                                redis_key = %key,
                                error = %e,
                                "Failed to deserialize persisted schedule"
                            );
                        }
                    }
                }
            }

            info!("Finished loading persisted schedules");
        }

        Ok(())
    }
}

/// Scheduler statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStats {
    pub total_scheduled_jobs: usize,
    pub enabled_scheduled_jobs: usize,
    pub next_execution_at: Option<DateTime<Utc>>,
    pub is_running: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduled_job_creation() {
        let job_type = JobType::Maintenance {
            task_type: "cleanup".to_string(),
            parameters: std::collections::HashMap::new(),
        };

        let scheduled_job = ScheduledJob::new(
            "Daily Cleanup".to_string(),
            "0 2 * * *".to_string(), // Daily at 2 AM
            job_type,
        );

        assert!(scheduled_job.is_ok());
        let job = scheduled_job.unwrap();
        assert_eq!(job.name, "Daily Cleanup");
        assert!(job.enabled);
        assert!(job.next_execution_at.is_some());
    }

    #[test]
    fn test_invalid_cron_expression() {
        let job_type = JobType::Maintenance {
            task_type: "cleanup".to_string(),
            parameters: std::collections::HashMap::new(),
        };

        let result = ScheduledJob::new(
            "Invalid Schedule".to_string(),
            "invalid cron".to_string(),
            job_type,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_scheduler_config_default() {
        let config = SchedulerConfig::default();
        assert_eq!(config.check_interval_secs, 30);
        assert_eq!(config.max_scheduled_jobs, 1000);
        assert!(config.persist_schedules);
    }
}
