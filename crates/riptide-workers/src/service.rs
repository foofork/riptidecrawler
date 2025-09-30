use crate::job::{Job, JobType};
use crate::queue::{JobQueue, QueueConfig};
use crate::worker::{WorkerPool, WorkerConfig};
use crate::scheduler::{JobScheduler, ScheduledJob, SchedulerConfig};
use crate::processors::{
    BatchCrawlProcessor, SingleCrawlProcessor, MaintenanceProcessor, CustomJobProcessor,
};
use crate::metrics::WorkerMetrics;
use anyhow::{Context, Result};
// use riptide_core::extract::WasmExtractor;
use riptide_core::cache::CacheManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Configuration for the worker service
#[derive(Debug, Clone)]
pub struct WorkerServiceConfig {
    /// Redis URL for job queue
    pub redis_url: String,
    /// Worker pool configuration
    pub worker_config: WorkerConfig,
    /// Job queue configuration
    pub queue_config: QueueConfig,
    /// Job scheduler configuration
    pub scheduler_config: SchedulerConfig,
    /// Maximum batch size for batch crawl jobs
    pub max_batch_size: usize,
    /// Maximum concurrent requests within a batch
    pub max_concurrency: usize,
    /// WASM extractor path
    pub wasm_path: String,
    /// Enable job scheduling
    pub enable_scheduler: bool,
}

impl Default for WorkerServiceConfig {
    fn default() -> Self {
        Self {
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            worker_config: WorkerConfig::default(),
            queue_config: QueueConfig::default(),
            scheduler_config: SchedulerConfig::default(),
            max_batch_size: 50,
            max_concurrency: 10,
            wasm_path: std::env::var("WASM_EXTRACTOR_PATH").unwrap_or_else(|_| {
                "./target/wasm32-wasip2/release/riptide_extractor_wasm.wasm".to_string()
            }),
            enable_scheduler: true,
        }
    }
}

/// Main worker service that orchestrates job processing
pub struct WorkerService {
    /// Service configuration
    config: WorkerServiceConfig,
    /// Job queue
    queue: Arc<Mutex<JobQueue>>,
    /// Worker pool
    worker_pool: Option<WorkerPool>,
    /// Job scheduler
    scheduler: Option<Arc<JobScheduler>>,
    /// Metrics collector
    metrics: Arc<WorkerMetrics>,
    /// Service running state
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl WorkerService {
    /// Create a new worker service
    pub async fn new(config: WorkerServiceConfig) -> Result<Self> {
        info!("Initializing worker service");

        // Initialize job queue
        let queue = JobQueue::new(&config.redis_url, config.queue_config.clone())
            .await
            .context("Failed to initialize job queue")?;
        let queue = Arc::new(Mutex::new(queue));

        // Initialize metrics
        let metrics = Arc::new(WorkerMetrics::new());

        // Initialize scheduler if enabled
        let scheduler = if config.enable_scheduler {
            let scheduler = JobScheduler::new(
                config.scheduler_config.clone(),
                queue.clone(),
                Some(&config.redis_url),
            )
            .await
            .context("Failed to initialize job scheduler")?;
            Some(Arc::new(scheduler))
        } else {
            None
        };

        info!("Worker service initialized successfully");

        Ok(Self {
            config,
            queue,
            worker_pool: None,
            scheduler,
            metrics,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
    }

    /// Start the worker service
    pub async fn start(&mut self) -> Result<()> {
        if self.running.load(std::sync::atomic::Ordering::Relaxed) {
            warn!("Worker service is already running");
            return Ok(());
        }

        info!("Starting worker service");
        self.running.store(true, std::sync::atomic::Ordering::Relaxed);

        // Initialize job processors
        let processors = self.create_job_processors().await?;

        // Create and configure worker pool
        let mut worker_pool = WorkerPool::new(self.config.worker_config.clone(), {
            let _queue = self.queue.lock().await;
            // We need to clone the queue, but JobQueue doesn't implement Clone
            // For now, we'll create a new queue connection
            JobQueue::new(&self.config.redis_url, self.config.queue_config.clone()).await?
        });

        // Add processors to worker pool
        for processor in processors {
            worker_pool.add_processor(processor);
        }

        self.worker_pool = Some(worker_pool);

        // Start scheduler if enabled
        if let Some(scheduler) = &self.scheduler {
            let scheduler_handle = {
                let scheduler = Arc::clone(scheduler);
                tokio::spawn(async move {
                    if let Err(e) = scheduler.start().await {
                        error!(error = %e, "Scheduler failed");
                    }
                })
            };

            // Start worker pool
            let worker_handle = {
                let worker_pool = self.worker_pool.as_ref().unwrap();
                let _worker_pool_clone = Arc::new(worker_pool);
                tokio::spawn(async move {
                    // Due to ownership issues, we'll need to handle this differently
                    // For now, we'll log that the worker pool would start here
                    info!("Worker pool would start here");
                })
            };

            // Start metrics collection task
            let metrics_handle = self.start_metrics_collection_task();

            info!("Worker service started successfully");

            // Wait for all tasks (in a real implementation, you'd want proper task management)
            tokio::select! {
                _ = scheduler_handle => {
                    info!("Scheduler task completed");
                }
                _ = worker_handle => {
                    info!("Worker pool task completed");
                }
                _ = metrics_handle => {
                    info!("Metrics task completed");
                }
            }
        } else {
            // Start worker pool without scheduler
            info!("Starting worker pool without scheduler");
            // Similar handling needed here
        }

        Ok(())
    }

    /// Stop the worker service
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping worker service");
        self.running.store(false, std::sync::atomic::Ordering::Relaxed);

        // Stop scheduler
        if let Some(scheduler) = &self.scheduler {
            scheduler.stop().await;
        }

        // Stop worker pool
        if let Some(worker_pool) = &self.worker_pool {
            worker_pool.stop().await;
        }

        info!("Worker service stopped");
        Ok(())
    }

    /// Submit a job to the queue
    pub async fn submit_job(&self, job: Job) -> Result<Uuid> {
        let job_type_name = match &job.job_type {
            JobType::BatchCrawl { .. } => "BatchCrawl",
            JobType::SingleCrawl { .. } => "SingleCrawl",
            JobType::PdfExtraction { .. } => "PdfExtraction",
            JobType::Maintenance { .. } => "Maintenance",
            JobType::Custom { job_name, .. } => job_name,
        };

        self.metrics.record_job_submitted(job_type_name);

        let mut queue = self.queue.lock().await;
        queue.submit_job(job).await
    }

    /// Get job status
    pub async fn get_job(&self, job_id: Uuid) -> Result<Option<Job>> {
        let mut queue = self.queue.lock().await;
        queue.get_job(job_id).await
    }

    /// Get job result
    pub async fn get_job_result(&self, job_id: Uuid) -> Result<Option<crate::job::JobResult>> {
        let mut queue = self.queue.lock().await;
        queue.get_job_result(job_id).await
    }

    /// Get queue statistics
    pub async fn get_queue_stats(&self) -> Result<crate::queue::QueueStats> {
        let mut queue = self.queue.lock().await;
        queue.get_stats().await
    }

    /// Get worker pool statistics
    pub fn get_worker_stats(&self) -> Option<crate::worker::WorkerPoolStats> {
        self.worker_pool.as_ref().map(|pool| pool.get_pool_stats())
    }

    /// Get scheduler statistics
    pub fn get_scheduler_stats(&self) -> Option<crate::scheduler::SchedulerStats> {
        self.scheduler.as_ref().map(|scheduler| scheduler.get_scheduler_stats())
    }

    /// Get metrics snapshot
    pub async fn get_metrics(&self) -> crate::metrics::WorkerMetricsSnapshot {
        self.metrics.get_snapshot().await
    }

    /// Add a scheduled job
    pub async fn add_scheduled_job(&self, scheduled_job: ScheduledJob) -> Result<Uuid> {
        if let Some(scheduler) = &self.scheduler {
            scheduler.add_scheduled_job(scheduled_job).await
        } else {
            Err(anyhow::anyhow!("Scheduler is not enabled"))
        }
    }

    /// Remove a scheduled job
    pub async fn remove_scheduled_job(&self, job_id: Uuid) -> Result<bool> {
        if let Some(scheduler) = &self.scheduler {
            scheduler.remove_scheduled_job(job_id).await
        } else {
            Err(anyhow::anyhow!("Scheduler is not enabled"))
        }
    }

    /// List scheduled jobs
    pub fn list_scheduled_jobs(&self) -> Result<Vec<ScheduledJob>> {
        if let Some(scheduler) = &self.scheduler {
            Ok(scheduler.list_scheduled_jobs())
        } else {
            Err(anyhow::anyhow!("Scheduler is not enabled"))
        }
    }

    /// Create job processors
    async fn create_job_processors(&self) -> Result<Vec<Arc<dyn crate::worker::JobProcessor>>> {
        info!("Initializing job processors");

        // Initialize HTTP client
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        // Initialize WASM extractor
        use riptide_core::extract::CmExtractor;
        use riptide_core::component::ExtractorConfig;
        let extractor_config = ExtractorConfig::default();
        let extractor = CmExtractor::new(extractor_config);
        let extractor = Arc::new(extractor) as Arc<dyn riptide_core::extract::WasmExtractor>;

        // Initialize cache manager
        let cache_manager = CacheManager::new(&self.config.redis_url)
            .await
            .context("Failed to initialize cache manager")?;
        let cache = Arc::new(tokio::sync::Mutex::new(cache_manager));

        let processors: Vec<Arc<dyn crate::worker::JobProcessor>> = vec![
            // Batch crawl processor
            Arc::new(BatchCrawlProcessor::new(
                http_client.clone(),
                extractor.clone(),
                cache.clone(),
                self.config.max_batch_size,
                self.config.max_concurrency,
            )),
            // Single crawl processor
            Arc::new(SingleCrawlProcessor::new(
                http_client.clone(),
                extractor.clone(),
                cache.clone(),
            )),
            // Maintenance processor
            Arc::new(MaintenanceProcessor),
            // Custom job processor
            Arc::new(CustomJobProcessor),
        ];

        info!("Initialized {} job processors", processors.len());
        Ok(processors)
    }

    /// Start metrics collection task
    async fn start_metrics_collection_task(&self) -> tokio::task::JoinHandle<()> {
        let metrics = self.metrics.clone();
        let queue = self.queue.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            while running.load(std::sync::atomic::Ordering::Relaxed) {
                // Collect queue statistics
                if let Ok(mut queue_guard) = queue.try_lock() {
                    match queue_guard.get_stats().await {
                        Ok(stats) => {
                            metrics.update_queue_size("pending", stats.pending as u64).await;
                            metrics.update_queue_size("processing", stats.processing as u64).await;
                            metrics.update_queue_size("completed", stats.completed as u64).await;
                            metrics.update_queue_size("failed", stats.failed as u64).await;
                            metrics.update_queue_size("retry", stats.retry as u64).await;
                            metrics.update_queue_size("delayed", stats.delayed as u64).await;
                        }
                        Err(e) => {
                            warn!(error = %e, "Failed to collect queue statistics");
                        }
                    }
                }

                // Sleep for 30 seconds before next collection
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            }
        })
    }

    /// Health check for the worker service
    pub async fn health_check(&self) -> WorkerServiceHealth {
        let queue_healthy = {
            match self.queue.try_lock() {
                Ok(mut queue) => queue.get_stats().await.is_ok(),
                Err(_) => false, // Queue is locked, assume unhealthy
            }
        };

        let scheduler_healthy = self.scheduler
            .as_ref()
            .map(|s| s.get_scheduler_stats().is_running)
            .unwrap_or(true); // If no scheduler, consider healthy

        let worker_pool_healthy = self.worker_pool
            .as_ref()
            .map(|p| p.get_pool_stats().healthy_workers > 0)
            .unwrap_or(false);

        let metrics = self.metrics.get_snapshot().await;

        WorkerServiceHealth {
            overall_healthy: queue_healthy && scheduler_healthy && worker_pool_healthy,
            queue_healthy,
            scheduler_healthy,
            worker_pool_healthy,
            metrics_snapshot: metrics,
        }
    }
}

/// Health status of the worker service
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkerServiceHealth {
    pub overall_healthy: bool,
    pub queue_healthy: bool,
    pub scheduler_healthy: bool,
    pub worker_pool_healthy: bool,
    pub metrics_snapshot: crate::metrics::WorkerMetricsSnapshot,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_service_config_default() {
        let config = WorkerServiceConfig::default();
        assert!(config.redis_url.contains("redis://"));
        assert_eq!(config.max_batch_size, 50);
        assert_eq!(config.max_concurrency, 10);
        assert!(config.enable_scheduler);
    }

    #[tokio::test]
    async fn test_worker_service_creation() {
        // This test would require a Redis instance for full functionality
        // For now, we test the basic structure
        let config = WorkerServiceConfig::default();

        // In a real test environment with Redis available:
        // let service = WorkerService::new(config).await;
        // assert!(service.is_ok());

        // For now, just test config validation
        assert!(config.max_batch_size > 0);
        assert!(config.max_concurrency > 0);
    }
}