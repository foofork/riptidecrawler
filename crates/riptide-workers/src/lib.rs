pub mod job;
pub mod metrics;
pub mod processors;
pub mod queue;
pub mod scheduler;
pub mod service;
pub mod worker;

pub use job::{Job, JobPriority, JobResult, JobStatus, JobType, PdfExtractionOptions, RetryConfig};
pub use metrics::{WorkerMetrics, WorkerMetricsSnapshot};
pub use processors::{
    BatchCrawlProcessor, BatchCrawlResponse, CrawlResult, CustomJobProcessor, CustomJobResult,
    MaintenanceProcessor, MaintenanceResult, PdfExtractionResult, PdfExtractionStats, PdfProcessor,
    SingleCrawlProcessor,
};
pub use queue::{JobQueue, QueueConfig, QueueStats};
pub use scheduler::{JobScheduler, ScheduledJob, SchedulerConfig, SchedulerStats};
pub use service::{WorkerService, WorkerServiceConfig};
pub use worker::{
    JobProcessor, Worker, WorkerConfig, WorkerPool, WorkerPoolStats, WorkerStatsSnapshot,
};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::job::{Job, JobPriority, JobStatus, JobType, PdfExtractionOptions};
    pub use crate::processors::PdfProcessor;
    pub use crate::queue::{JobQueue, QueueConfig};
    pub use crate::scheduler::{JobScheduler, ScheduledJob};
    pub use crate::service::{WorkerService, WorkerServiceConfig};
    pub use crate::worker::{JobProcessor, WorkerConfig, WorkerPool};
    pub use async_trait::async_trait;
}
