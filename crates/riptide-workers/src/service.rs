use crate::job::{Job, JobType};
use crate::metrics::WorkerMetrics;
use crate::processors::{
    BatchCrawlProcessor, CustomJobProcessor, MaintenanceProcessor, SingleCrawlProcessor,
};
use crate::queue::{JobQueue, QueueConfig};
use crate::scheduler::{JobScheduler, ScheduledJob, SchedulerConfig};
use crate::worker::{WorkerConfig, WorkerPool};
use anyhow::{Context, Result};
// use riptide_reliability::WasmExtractor;
use riptide_cache::redis::CacheManager;
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
    /// Create a new worker service and automatically start it
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

        // Initialize job processors
        info!("Initializing job processors");
        let processors = Self::create_job_processors_static(&config).await?;

        // Create worker pool immediately (not deferred to start())
        info!("Creating worker pool");
        let queue_for_pool = JobQueue::new(&config.redis_url, config.queue_config.clone()).await?;
        let mut worker_pool = WorkerPool::new(config.worker_config.clone(), queue_for_pool);

        // Add processors to worker pool
        for processor in processors {
            worker_pool.add_processor(processor);
        }

        let mut service = Self {
            config,
            queue,
            worker_pool: Some(worker_pool),
            scheduler,
            metrics,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        };

        // Auto-start the service
        service.start_internal().await?;

        info!("Worker service initialized and started successfully");

        Ok(service)
    }

    /// Start the worker service (public API for backward compatibility, calls internal start)
    pub async fn start(&mut self) -> Result<()> {
        self.start_internal().await
    }

    /// Internal start method that actually starts the worker pool and scheduler
    async fn start_internal(&mut self) -> Result<()> {
        if self.running.load(std::sync::atomic::Ordering::Relaxed) {
            warn!("Worker service is already running");
            return Ok(());
        }

        info!("Starting worker service");
        self.running
            .store(true, std::sync::atomic::Ordering::Relaxed);

        // Start scheduler if enabled
        if let Some(scheduler) = &self.scheduler {
            let scheduler = Arc::clone(scheduler);
            tokio::spawn(async move {
                if let Err(e) = scheduler.start().await {
                    error!(error = %e, "Scheduler failed");
                }
            });
        }

        // Start worker pool
        if let Some(worker_pool) = &self.worker_pool {
            // Clone the worker pool stats reference for monitoring
            let worker_config = self.config.worker_config.clone();
            info!(
                "Worker pool ready with {} workers",
                worker_config.worker_count
            );

            // Start worker pool in background task
            // Note: The worker pool's start() method should be called here
            // For now, workers will poll automatically based on their configuration
            let _ = worker_pool.start().await;
        } else {
            return Err(anyhow::anyhow!("Worker pool not initialized"));
        }

        // Start metrics collection task
        let _metrics_handle = self.start_metrics_collection_task().await;

        info!("Worker service started successfully");

        Ok(())
    }

    /// Stop the worker service
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping worker service");
        self.running
            .store(false, std::sync::atomic::Ordering::Relaxed);

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
        self.scheduler
            .as_ref()
            .map(|scheduler| scheduler.get_scheduler_stats())
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

    /// List jobs with filtering and pagination
    pub async fn list_jobs(
        &self,
        status: Option<&str>,
        job_type: Option<&str>,
        search: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Job>> {
        let mut queue = self.queue.lock().await;
        queue
            .list_jobs(status, job_type, search, limit, offset)
            .await
    }

    /// Create job processors (static version for use in new())
    async fn create_job_processors_static(
        config: &WorkerServiceConfig,
    ) -> Result<Vec<Arc<dyn crate::worker::JobProcessor>>> {
        info!("Initializing job processors with native-first extraction strategy");

        // Initialize HTTP client
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        // Initialize UnifiedExtractor with native-first strategy
        use riptide_extraction::UnifiedExtractor;
        use riptide_reliability::WasmExtractor;

        // UnifiedExtractor uses native extraction by default
        // WASM is only used if:
        // 1. wasm-extractor feature is enabled
        // 2. WASM file exists at the specified path
        let wasm_path = if cfg!(feature = "wasm-extractor")
            && std::path::Path::new(&config.wasm_path).exists()
        {
            tracing::info!(
                wasm_path = %config.wasm_path,
                "WASM extractor available, will be used as enhancement over native"
            );
            Some(config.wasm_path.as_str())
        } else {
            if !std::path::Path::new(&config.wasm_path).exists() {
                tracing::info!(
                    wasm_path = %config.wasm_path,
                    "WASM extractor path not found, using native extraction"
                );
            } else {
                tracing::info!("wasm-extractor feature not enabled, using native extraction");
            }
            None
        };

        let unified_extractor = UnifiedExtractor::new(wasm_path)
            .await
            .context("Failed to initialize unified extractor")?;

        tracing::info!(
            strategy = unified_extractor.extractor_type(),
            "Initialized content extractor"
        );

        // Wrap UnifiedExtractor to implement WasmExtractor trait
        struct UnifiedExtractorAdapter {
            inner: UnifiedExtractor,
        }

        impl WasmExtractor for UnifiedExtractorAdapter {
            fn extract(
                &self,
                html: &[u8],
                url: &str,
                _mode: &str,
            ) -> anyhow::Result<riptide_types::ExtractedDoc> {
                // Convert bytes to string
                let html_str = String::from_utf8_lossy(html);

                // Use tokio block_in_place for async extraction in sync context
                let result = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current()
                        .block_on(async { self.inner.extract(&html_str, url).await })
                })?;

                // Convert ExtractedContent to ExtractedDoc
                Ok(riptide_types::ExtractedDoc {
                    url: result.url,
                    title: Some(result.title),
                    text: result.content,
                    description: result.summary,
                    quality_score: Some((result.extraction_confidence * 100.0).min(100.0) as u8),
                    ..Default::default()
                })
            }
        }

        let extractor = Arc::new(UnifiedExtractorAdapter {
            inner: unified_extractor,
        }) as Arc<dyn WasmExtractor>;

        // Initialize cache manager
        let cache_manager = CacheManager::new(&config.redis_url)
            .await
            .context("Failed to initialize cache manager")?;
        let cache = Arc::new(tokio::sync::Mutex::new(cache_manager));

        let processors: Vec<Arc<dyn crate::worker::JobProcessor>> = vec![
            // Batch crawl processor
            Arc::new(BatchCrawlProcessor::new(
                http_client.clone(),
                extractor.clone(),
                cache.clone(),
                config.max_batch_size,
                config.max_concurrency,
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

    /// Create job processors (instance method for backward compatibility)
    #[allow(dead_code)]
    async fn create_job_processors(&self) -> Result<Vec<Arc<dyn crate::worker::JobProcessor>>> {
        Self::create_job_processors_static(&self.config).await
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
                            metrics
                                .update_queue_size("pending", stats.pending as u64)
                                .await;
                            metrics
                                .update_queue_size("processing", stats.processing as u64)
                                .await;
                            metrics
                                .update_queue_size("completed", stats.completed as u64)
                                .await;
                            metrics
                                .update_queue_size("failed", stats.failed as u64)
                                .await;
                            metrics.update_queue_size("retry", stats.retry as u64).await;
                            metrics
                                .update_queue_size("delayed", stats.delayed as u64)
                                .await;
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

        let scheduler_healthy = self
            .scheduler
            .as_ref()
            .map(|s| s.get_scheduler_stats().is_running)
            .unwrap_or(true); // If no scheduler, consider healthy

        let worker_pool_healthy = self
            .worker_pool
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

// Implement the WorkerService trait from riptide-types
#[async_trait::async_trait]
impl riptide_types::ports::WorkerService for WorkerService {
    async fn health_check(&self) -> riptide_types::ports::WorkerHealth {
        let service_health = self.health_check().await;

        // Convert WorkerServiceHealth to WorkerHealth
        riptide_types::ports::WorkerHealth {
            overall_healthy: service_health.overall_healthy,
            queue_healthy: service_health.queue_healthy,
            worker_pool_healthy: service_health.worker_pool_healthy,
            scheduler_healthy: service_health.scheduler_healthy,
            active_workers: service_health.metrics_snapshot.worker_health.len(),
            pending_jobs: service_health.metrics_snapshot.queue_sizes.values().sum::<u64>() as usize,
        }
    }

    async fn active_worker_count(&self) -> usize {
        self.get_metrics().await.worker_health.len()
    }

    async fn pending_jobs_count(&self) -> usize {
        self.get_metrics().await.queue_sizes.values().sum::<u64>() as usize
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
