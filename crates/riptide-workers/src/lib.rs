pub mod job;
pub mod queue;
pub mod worker;
pub mod scheduler;
pub mod processors;
pub mod metrics;
pub mod service;

pub use job::{Job, JobType, JobPriority, JobStatus, JobResult, RetryConfig};
pub use queue::{JobQueue, QueueConfig, QueueStats};
pub use worker::{Worker, WorkerPool, WorkerConfig, WorkerStatsSnapshot, WorkerPoolStats, JobProcessor};
pub use scheduler::{JobScheduler, ScheduledJob, SchedulerConfig, SchedulerStats};
pub use processors::{
    BatchCrawlProcessor, SingleCrawlProcessor, MaintenanceProcessor, CustomJobProcessor,
    BatchCrawlResponse, CrawlResult, MaintenanceResult, CustomJobResult,
};
pub use metrics::{WorkerMetrics, WorkerMetricsSnapshot};
pub use service::{WorkerService, WorkerServiceConfig};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::job::{Job, JobType, JobPriority, JobStatus};
    pub use crate::queue::{JobQueue, QueueConfig};
    pub use crate::worker::{WorkerPool, WorkerConfig, JobProcessor};
    pub use crate::scheduler::{JobScheduler, ScheduledJob};
    pub use crate::service::{WorkerService, WorkerServiceConfig};
    pub use async_trait::async_trait;
}