use crate::job::{Job, JobResult, JobType};
use crate::queue::JobQueue;
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use futures::future::join_all;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Trait for processing different types of jobs
#[async_trait]
pub trait JobProcessor: Send + Sync {
    /// Process a specific job and return the result
    async fn process_job(&self, job: &Job) -> Result<serde_json::Value>;

    /// Get the job types this processor can handle
    fn supported_job_types(&self) -> Vec<String>;

    /// Get processor name for identification
    fn processor_name(&self) -> String;
}

/// Worker configuration
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Number of worker threads
    pub worker_count: usize,
    /// Worker polling interval in seconds
    pub poll_interval_secs: u64,
    /// Maximum job processing time before timeout
    pub job_timeout_secs: u64,
    /// Worker heartbeat interval
    pub heartbeat_interval_secs: u64,
    /// Maximum concurrent jobs per worker
    pub max_concurrent_jobs: usize,
    /// Enable worker health monitoring
    pub enable_health_monitoring: bool,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            worker_count: num_cpus::get().max(2),
            poll_interval_secs: 5,
            job_timeout_secs: 600, // 10 minutes
            heartbeat_interval_secs: 30,
            max_concurrent_jobs: 4,
            enable_health_monitoring: true,
        }
    }
}

/// Individual worker instance
pub struct Worker {
    /// Worker unique identifier
    pub id: String,
    /// Worker configuration
    config: WorkerConfig,
    /// Job queue reference
    queue: Arc<tokio::sync::Mutex<JobQueue>>,
    /// Job processors
    processors: Vec<Arc<dyn JobProcessor>>,
    /// Worker running state
    running: Arc<AtomicBool>,
    /// Worker statistics
    stats: Arc<WorkerStats>,
    /// Concurrency semaphore
    semaphore: Arc<Semaphore>,
}

/// Worker statistics
#[derive(Debug, Default)]
pub struct WorkerStats {
    /// Total jobs processed
    pub jobs_processed: AtomicU64,
    /// Total jobs failed
    pub jobs_failed: AtomicU64,
    /// Last heartbeat timestamp
    pub last_heartbeat: parking_lot::RwLock<Option<chrono::DateTime<chrono::Utc>>>,
    /// Worker start time
    pub started_at: parking_lot::RwLock<Option<chrono::DateTime<chrono::Utc>>>,
    /// Current processing job ID
    pub current_job: parking_lot::RwLock<Option<Uuid>>,
    /// Processing time histogram
    pub processing_times: parking_lot::RwLock<Vec<u64>>,
}

impl Worker {
    /// Create a new worker instance
    pub fn new(
        id: String,
        config: WorkerConfig,
        queue: Arc<tokio::sync::Mutex<JobQueue>>,
        processors: Vec<Arc<dyn JobProcessor>>,
    ) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_jobs));

        Self {
            id,
            config,
            queue,
            processors,
            running: Arc::new(AtomicBool::new(false)),
            stats: Arc::new(WorkerStats::default()),
            semaphore,
        }
    }

    /// Start the worker
    pub async fn start(&self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            warn!(worker_id = %self.id, "Worker is already running");
            return Ok(());
        }

        info!(worker_id = %self.id, "Starting worker");
        self.running.store(true, Ordering::Relaxed);

        // Update start time
        {
            let mut started_at = self.stats.started_at.write();
            *started_at = Some(Utc::now());
        }

        // Start heartbeat task
        let heartbeat_task = self.start_heartbeat_task();

        // Start main worker loop
        let worker_task = self.run_worker_loop();

        // Wait for both tasks
        tokio::select! {
            result = heartbeat_task => {
                error!(worker_id = %self.id, "Heartbeat task failed: {:?}", result);
            }
            result = worker_task => {
                info!(worker_id = %self.id, "Worker task completed: {:?}", result);
            }
        }

        Ok(())
    }

    /// Stop the worker
    pub async fn stop(&self) {
        info!(worker_id = %self.id, "Stopping worker");
        self.running.store(false, Ordering::Relaxed);
    }

    /// Get worker statistics
    pub fn get_stats(&self) -> WorkerStatsSnapshot {
        let jobs_processed = self.stats.jobs_processed.load(Ordering::Relaxed);
        let jobs_failed = self.stats.jobs_failed.load(Ordering::Relaxed);
        let last_heartbeat = *self.stats.last_heartbeat.read();
        let started_at = *self.stats.started_at.read();
        let current_job = *self.stats.current_job.read();

        // Calculate average processing time
        let processing_times = self.stats.processing_times.read();
        let avg_processing_time = if !processing_times.is_empty() {
            processing_times.iter().sum::<u64>() / processing_times.len() as u64
        } else {
            0
        };

        WorkerStatsSnapshot {
            worker_id: self.id.clone(),
            jobs_processed,
            jobs_failed,
            last_heartbeat,
            started_at,
            current_job,
            avg_processing_time_ms: avg_processing_time,
            success_rate: if jobs_processed > 0 {
                ((jobs_processed - jobs_failed) as f64 / jobs_processed as f64) * 100.0
            } else {
                0.0
            },
            is_healthy: self.is_healthy(),
        }
    }

    /// Check if worker is healthy
    pub fn is_healthy(&self) -> bool {
        if !self.running.load(Ordering::Relaxed) {
            return false;
        }

        // Check last heartbeat
        if let Some(last_heartbeat) = *self.stats.last_heartbeat.read() {
            let heartbeat_age = (Utc::now() - last_heartbeat).num_seconds();
            heartbeat_age < (self.config.heartbeat_interval_secs * 3) as i64
        } else {
            true // Just started, no heartbeat yet
        }
    }

    /// Main worker processing loop
    async fn run_worker_loop(&self) -> Result<()> {
        info!(worker_id = %self.id, "Starting worker processing loop");

        while self.running.load(Ordering::Relaxed) {
            match self.process_next_job().await {
                Ok(processed) => {
                    if !processed {
                        // No job available, sleep for polling interval
                        sleep(Duration::from_secs(self.config.poll_interval_secs)).await;
                    }
                }
                Err(e) => {
                    error!(
                        worker_id = %self.id,
                        error = %e,
                        "Error processing job, continuing"
                    );
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }

        info!(worker_id = %self.id, "Worker processing loop stopped");
        Ok(())
    }

    /// Process the next available job
    async fn process_next_job(&self) -> Result<bool> {
        // Acquire semaphore permit for concurrency control
let _ = self.semaphore.acquire().await?;
        let mut queue = self.queue.lock().await;
        if let Some(job) = queue.next_job(&self.id).await? {
            drop(queue); // Release queue lock early

            debug!(
                worker_id = %self.id,
                job_id = %job.id,
                job_type = ?job.job_type,
                "Processing job"
            );

            // Update current job tracking
            {
                let mut current_job = self.stats.current_job.write();
                *current_job = Some(job.id);
            }

            let start_time = Instant::now();
            let result = self.execute_job(&job).await;
            let processing_time_ms = start_time.elapsed().as_millis() as u64;

            // Update processing time stats
            {
                let mut processing_times = self.stats.processing_times.write();
                processing_times.push(processing_time_ms);
                // Keep only last 100 measurements
                if processing_times.len() > 100 {
                    processing_times.remove(0);
                }
            }

            // Clear current job tracking
            {
                let mut current_job = self.stats.current_job.write();
                *current_job = None;
            }

            // Handle job result
            let mut queue = self.queue.lock().await;
            match result {
                Ok(job_result) => {
                    queue.complete_job(job.id, job_result).await?;
                    self.stats.jobs_processed.fetch_add(1, Ordering::Relaxed);
                    info!(
                        worker_id = %self.id,
                        job_id = %job.id,
                        processing_time_ms = processing_time_ms,
                        "Job completed successfully"
                    );
                }
                Err(e) => {
                    queue.fail_job(job.id, e.to_string()).await?;
                    self.stats.jobs_failed.fetch_add(1, Ordering::Relaxed);
                    error!(
                        worker_id = %self.id,
                        job_id = %job.id,
                        error = %e,
                        processing_time_ms = processing_time_ms,
                        "Job failed"
                    );
                }
            }

            Ok(true) // Job was processed
        } else {
            Ok(false) // No job available
        }
    }

    /// Execute a job using appropriate processor
    async fn execute_job(&self, job: &Job) -> Result<JobResult> {
        let job_type_name = match &job.job_type {
            JobType::BatchCrawl { .. } => "BatchCrawl",
            JobType::SingleCrawl { .. } => "SingleCrawl",
            JobType::PdfExtraction { .. } => "PdfExtraction",
            JobType::Maintenance { .. } => "Maintenance",
            JobType::Custom { job_name, .. } => job_name,
        };

        // Find appropriate processor
        let processor = self
            .processors
            .iter()
            .find(|p| p.supported_job_types().contains(&job_type_name.to_string()))
            .context("No processor found for job type")?;

        info!(
            worker_id = %self.id,
            job_id = %job.id,
            processor = %processor.processor_name(),
            "Executing job"
        );

        // Execute with timeout
        let result = tokio::time::timeout(
            Duration::from_secs(self.config.job_timeout_secs),
            processor.process_job(job),
        )
        .await;

        match result {
            Ok(Ok(data)) => {
                let processing_time = job.processing_time_ms().unwrap_or(0);
                Ok(JobResult::success(
                    job.id,
                    self.id.clone(),
                    Some(data),
                    processing_time,
                ))
            }
            Ok(Err(e)) => {
                let processing_time = job.processing_time_ms().unwrap_or(0);
                Ok(JobResult::failure(
                    job.id,
                    self.id.clone(),
                    e.to_string(),
                    processing_time,
                ))
            }
            Err(_) => {
                let error = format!(
                    "Job timed out after {} seconds",
                    self.config.job_timeout_secs
                );
                let processing_time = job
                    .processing_time_ms()
                    .unwrap_or(self.config.job_timeout_secs * 1000);
                Ok(JobResult::failure(
                    job.id,
                    self.id.clone(),
                    error,
                    processing_time,
                ))
            }
        }
    }

    /// Start heartbeat task
    async fn start_heartbeat_task(&self) -> Result<()> {
        let worker_id = self.id.clone();
        let running = self.running.clone();
        let stats = self.stats.clone();
        let interval = Duration::from_secs(self.config.heartbeat_interval_secs);

        tokio::spawn(async move {
            while running.load(Ordering::Relaxed) {
                {
                    let mut last_heartbeat = stats.last_heartbeat.write();
                    *last_heartbeat = Some(Utc::now());
                }

                debug!(worker_id = %worker_id, "Heartbeat");
                sleep(interval).await;
            }
        });

        Ok(())
    }
}

/// Snapshot of worker statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkerStatsSnapshot {
    pub worker_id: String,
    pub jobs_processed: u64,
    pub jobs_failed: u64,
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub current_job: Option<Uuid>,
    pub avg_processing_time_ms: u64,
    pub success_rate: f64,
    pub is_healthy: bool,
}

/// Worker pool manager
pub struct WorkerPool {
    /// Pool configuration
    config: WorkerConfig,
    /// Job queue
    queue: Arc<tokio::sync::Mutex<JobQueue>>,
    /// Active workers
    workers: Arc<DashMap<String, Arc<Worker>>>,
    /// Job processors
    processors: Vec<Arc<dyn JobProcessor>>,
    /// Pool running state
    running: Arc<AtomicBool>,
}

impl WorkerPool {
    /// Create a new worker pool
    pub fn new(config: WorkerConfig, queue: JobQueue) -> Self {
        Self {
            config,
            queue: Arc::new(tokio::sync::Mutex::new(queue)),
            workers: Arc::new(DashMap::new()),
            processors: Vec::new(),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Add a job processor to the pool
    pub fn add_processor(&mut self, processor: Arc<dyn JobProcessor>) {
        info!(processor_name = %processor.processor_name(), "Adding job processor");
        self.processors.push(processor);
    }

    /// Start the worker pool
    pub async fn start(&self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            warn!("Worker pool is already running");
            return Ok(());
        }

        info!(
            "Starting worker pool with {} workers",
            self.config.worker_count
        );
        self.running.store(true, Ordering::Relaxed);

        // Start workers
        let mut worker_handles = Vec::new();

        for i in 0..self.config.worker_count {
            let worker_id = format!("worker-{}", i);
            let worker = Arc::new(Worker::new(
                worker_id.clone(),
                self.config.clone(),
                self.queue.clone(),
                self.processors.clone(),
            ));

            self.workers.insert(worker_id.clone(), worker.clone());

            let worker_handle = {
                let worker = worker.clone();
                tokio::spawn(async move {
                    if let Err(e) = worker.start().await {
                        error!(worker_id = %worker.id, error = %e, "Worker failed");
                    }
                })
            };

            worker_handles.push(worker_handle);
        }

        // Wait for all workers to complete
        join_all(worker_handles).await;

        info!("Worker pool stopped");
        Ok(())
    }

    /// Stop the worker pool
    pub async fn stop(&self) {
        info!("Stopping worker pool");
        self.running.store(false, Ordering::Relaxed);

        // Stop all workers
        for worker_ref in self.workers.iter() {
            worker_ref.value().stop().await;
        }

        self.workers.clear();
    }

    /// Get pool statistics
    pub fn get_pool_stats(&self) -> WorkerPoolStats {
        let mut worker_stats = Vec::new();
        let mut total_processed = 0;
        let mut total_failed = 0;
        let mut healthy_workers = 0;

        for worker_ref in self.workers.iter() {
            let stats = worker_ref.value().get_stats();
            total_processed += stats.jobs_processed;
            total_failed += stats.jobs_failed;
            if stats.is_healthy {
                healthy_workers += 1;
            }
            worker_stats.push(stats);
        }

        WorkerPoolStats {
            total_workers: self.workers.len(),
            healthy_workers,
            total_jobs_processed: total_processed,
            total_jobs_failed: total_failed,
            worker_stats,
            is_running: self.running.load(Ordering::Relaxed),
        }
    }
}

/// Worker pool statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkerPoolStats {
    pub total_workers: usize,
    pub healthy_workers: usize,
    pub total_jobs_processed: u64,
    pub total_jobs_failed: u64,
    pub worker_stats: Vec<WorkerStatsSnapshot>,
    pub is_running: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_config_default() {
        let config = WorkerConfig::default();
        assert!(config.worker_count >= 2);
        assert_eq!(config.poll_interval_secs, 5);
    }

    #[test]
    fn test_worker_stats_creation() {
        let stats = WorkerStats::default();
        assert_eq!(stats.jobs_processed.load(Ordering::Relaxed), 0);
        assert_eq!(stats.jobs_failed.load(Ordering::Relaxed), 0);
    }
}
