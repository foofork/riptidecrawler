# riptide-workers

Background job queue and scheduling system with Redis backend for async task processing in the RipTide event mesh.

## Overview

`riptide-workers` provides a robust, production-ready worker system for processing asynchronous jobs with features like:

- **Redis-backed job queue** with priority support and persistence
- **Cron-based scheduling** for recurring tasks
- **Job priorities and retry logic** with exponential backoff
- **Worker pool management** with health monitoring and graceful shutdown
- **Job status tracking** through pending, processing, completed, failed, retrying, and dead letter states
- **Dead letter queue** for permanently failed jobs
- **Metrics and monitoring** with real-time statistics
- **Multiple job processors** for different task types (crawling, PDF extraction, maintenance, custom)

### Architecture

The system is built on several core components:

```
┌─────────────────┐
│   Job Queue     │  ← Redis-backed priority queue
│   (Redis)       │     with delayed job support
└────────┬────────┘
         │
┌────────▼────────┐
│  Job Scheduler  │  ← Cron-based recurring jobs
└────────┬────────┘
         │
┌────────▼────────┐
│  Worker Pool    │  ← Multiple concurrent workers
│  (2-N workers)  │     with health monitoring
└────────┬────────┘
         │
┌────────▼────────┐
│  Job Processors │  ← Pluggable job handlers
│  - BatchCrawl   │     for different job types
│  - SingleCrawl  │
│  - PDF Extract  │
│  - Maintenance  │
│  - Custom       │
└─────────────────┘
```

## Job Types and Queues

### Supported Job Types

1. **BatchCrawl**: Process multiple URLs concurrently
2. **SingleCrawl**: Process a single URL
3. **PdfExtraction**: Extract text, images, and metadata from PDFs
4. **Maintenance**: System maintenance tasks (cache cleanup, health checks, log rotation)
5. **Custom**: Arbitrary user-defined jobs with JSON payloads

### Queue States

Jobs flow through different queues based on their state:

- **Pending**: Jobs waiting to be processed
- **Scheduled**: Jobs with delayed execution times
- **Processing**: Jobs currently being worked on
- **Retry**: Failed jobs waiting for retry
- **Completed**: Successfully completed jobs
- **Dead Letter**: Jobs that exceeded max retry attempts

### Job Priorities

Jobs support four priority levels that determine queue ordering:

```rust
pub enum JobPriority {
    Low = 1,
    Normal = 2,    // Default
    High = 3,
    Critical = 4,
}
```

## Worker Management

### Worker Pool

The worker pool manages multiple worker threads with automatic job distribution:

```rust
pub struct WorkerConfig {
    /// Number of worker threads (default: CPU count)
    pub worker_count: usize,
    /// Worker polling interval in seconds (default: 5)
    pub poll_interval_secs: u64,
    /// Maximum job processing time before timeout (default: 600)
    pub job_timeout_secs: u64,
    /// Worker heartbeat interval (default: 30)
    pub heartbeat_interval_secs: u64,
    /// Maximum concurrent jobs per worker (default: 4)
    pub max_concurrent_jobs: usize,
    /// Enable worker health monitoring (default: true)
    pub enable_health_monitoring: bool,
}
```

### Worker Health Monitoring

Workers automatically send heartbeats and track:
- Jobs processed/failed count
- Current processing job
- Average processing time
- Last heartbeat timestamp
- Health status

## Scheduling with Cron

### Creating Scheduled Jobs

Use standard cron expressions to schedule recurring tasks:

```rust
use riptide_workers::{ScheduledJob, JobType};

// Daily cleanup at 2 AM
let scheduled_job = ScheduledJob::new(
    "Daily Cleanup".to_string(),
    "0 2 * * *".to_string(),
    JobType::Maintenance {
        task_type: "cache_cleanup".to_string(),
        parameters: Default::default(),
    },
)?;

scheduler.add_scheduled_job(scheduled_job).await?;
```

### Cron Expression Format

Standard 5-field cron format:
```
┌───────────── minute (0-59)
│ ┌───────────── hour (0-23)
│ │ ┌───────────── day of month (1-31)
│ │ │ ┌───────────── month (1-12)
│ │ │ │ ┌───────────── day of week (0-6, Sunday = 0)
│ │ │ │ │
* * * * *
```

Examples:
- `0 2 * * *` - Daily at 2:00 AM
- `*/15 * * * *` - Every 15 minutes
- `0 0 * * 0` - Weekly on Sunday at midnight
- `0 9-17 * * 1-5` - Weekdays between 9 AM and 5 PM

## Configuration

### Redis Configuration

The system uses Redis for:
- Job queue storage
- Scheduled job persistence
- Job result caching
- Worker coordination

```rust
pub struct QueueConfig {
    /// Namespace prefix for Redis keys (default: "riptide_jobs")
    pub namespace: String,
    /// Maximum jobs in memory cache (default: 1000)
    pub cache_size: usize,
    /// Polling interval for delayed jobs (default: 30s)
    pub delayed_job_poll_interval: u64,
    /// Maximum time to hold a job lease (default: 600s)
    pub job_lease_timeout: u64,
    /// Enable job result persistence (default: true)
    pub persist_results: bool,
    /// Result TTL in seconds (default: 3600)
    pub result_ttl: u64,
}
```

### Complete Service Configuration

```rust
use riptide_workers::{WorkerServiceConfig, WorkerConfig, QueueConfig, SchedulerConfig};

let config = WorkerServiceConfig {
    redis_url: "redis://localhost:6379".to_string(),
    worker_config: WorkerConfig {
        worker_count: 4,
        poll_interval_secs: 5,
        job_timeout_secs: 600,
        max_concurrent_jobs: 4,
        heartbeat_interval_secs: 30,
        enable_health_monitoring: true,
    },
    queue_config: QueueConfig::default(),
    scheduler_config: SchedulerConfig::default(),
    max_batch_size: 50,
    max_concurrency: 10,
    wasm_path: "./wasm/riptide-extractor.wasm".to_string(),
    enable_scheduler: true,
};
```

## Usage Examples

### Basic Job Enqueue and Process

```rust
use riptide_workers::prelude::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create worker service
    let config = WorkerServiceConfig::default();
    let mut service = WorkerService::new(config).await?;

    // Submit a crawl job
    let job = Job::new(JobType::SingleCrawl {
        url: "https://example.com".to_string(),
        options: None,
    });

    let job_id = service.submit_job(job).await?;
    println!("Job submitted: {}", job_id);

    // Start processing
    service.start().await?;

    Ok(())
}
```

### Batch Crawl with Custom Priority

```rust
use riptide_workers::{Job, JobType, JobPriority};

let urls = vec![
    "https://example.com/page1".to_string(),
    "https://example.com/page2".to_string(),
    "https://example.com/page3".to_string(),
];

let job = Job::with_priority(
    JobType::BatchCrawl {
        urls,
        options: None,
    },
    JobPriority::High,
);

let job_id = service.submit_job(job).await?;
```

### PDF Extraction Job

```rust
use riptide_workers::{Job, JobType, PdfExtractionOptions};
use std::fs;

// Read PDF file
let pdf_data = fs::read("document.pdf")?;

// Create extraction job with options
let job = Job::new(JobType::PdfExtraction {
    pdf_data,
    url: Some("document.pdf".to_string()),
    options: Some(PdfExtractionOptions {
        extract_text: true,
        extract_images: false,
        extract_metadata: true,
        max_size_bytes: 100 * 1024 * 1024, // 100MB
        enable_progress: true,
        custom_settings: Default::default(),
    }),
});

let job_id = service.submit_job(job).await?;
```

### Schedule Recurring Maintenance

```rust
use riptide_workers::{ScheduledJob, JobType};
use std::collections::HashMap;

// Schedule cache cleanup every hour
let scheduled_job = ScheduledJob::new(
    "Hourly Cache Cleanup".to_string(),
    "0 * * * *".to_string(),
    JobType::Maintenance {
        task_type: "cache_cleanup".to_string(),
        parameters: HashMap::new(),
    },
)?;

service.add_scheduled_job(scheduled_job).await?;
```

### Custom Job with Metadata

```rust
use riptide_workers::{Job, JobType};
use serde_json::json;

let job = Job::new(JobType::Custom {
    job_name: "data_processing".to_string(),
    payload: json!({
        "source": "user_uploads",
        "batch_size": 100,
        "filters": ["active", "verified"],
    }),
})
.with_metadata(
    "tenant_id".to_string(),
    json!("tenant-123"),
)
.with_timeout(300) // 5 minutes
.with_retry_config(RetryConfig {
    max_attempts: 5,
    initial_delay_secs: 60,
    backoff_multiplier: 2.0,
    max_delay_secs: 600,
    use_jitter: true,
});

let job_id = service.submit_job(job).await?;
```

## Monitoring and Metrics

### Real-time Statistics

Get comprehensive metrics about the worker system:

```rust
use riptide_workers::WorkerService;

// Get queue statistics
let queue_stats = service.get_queue_stats().await?;
println!("Pending jobs: {}", queue_stats.pending);
println!("Processing jobs: {}", queue_stats.processing);
println!("Completed jobs: {}", queue_stats.completed);
println!("Failed jobs: {}", queue_stats.failed);

// Get worker pool statistics
let pool_stats = service.get_worker_stats();
println!("Total workers: {}", pool_stats.total_workers);
println!("Healthy workers: {}", pool_stats.healthy_workers);
println!("Total jobs processed: {}", pool_stats.total_jobs_processed);

// Get scheduler statistics
let scheduler_stats = service.get_scheduler_stats()?;
println!("Scheduled jobs: {}", scheduler_stats.total_scheduled_jobs);
println!("Next execution: {:?}", scheduler_stats.next_execution_at);
```

### Worker Metrics

The metrics system tracks:

```rust
pub struct WorkerMetricsSnapshot {
    pub jobs_submitted: u64,
    pub jobs_completed: u64,
    pub jobs_failed: u64,
    pub jobs_retried: u64,
    pub jobs_dead_letter: u64,

    // Performance metrics
    pub avg_processing_time_ms: u64,
    pub p95_processing_time_ms: u64,
    pub p99_processing_time_ms: u64,

    // System health
    pub success_rate: f64,
    pub jobs_per_second: f64,
    pub total_queue_depth: u64,
    pub uptime_seconds: u64,
}
```

### Health Checks

```rust
// Check service health
let health = service.get_health().await;
println!("Service healthy: {}", health.is_healthy);
println!("Queue status: {:?}", health.queue_status);
println!("Worker status: {:?}", health.worker_status);
println!("Scheduler status: {:?}", health.scheduler_status);
```

## Error Handling and Retries

### Retry Configuration

Jobs support automatic retries with exponential backoff:

```rust
use riptide_workers::RetryConfig;

let retry_config = RetryConfig {
    max_attempts: 3,              // Retry up to 3 times
    initial_delay_secs: 30,       // Wait 30 seconds before first retry
    backoff_multiplier: 2.0,      // Double delay each retry
    max_delay_secs: 300,          // Cap at 5 minutes
    use_jitter: true,             // Add randomness to prevent thundering herd
};

let job = Job::new(job_type).with_retry_config(retry_config);
```

### Retry Flow

```
Job Failed
    ↓
Retry Count < Max?
    ↓ Yes
Calculate Next Retry Time
    ↓
Move to Retry Queue
    ↓
Wait for Retry Time
    ↓
Move to Pending Queue
    ↓
Process Again

Retry Count >= Max?
    ↓ Yes
Move to Dead Letter Queue
```

### Dead Letter Queue

Jobs that exceed max retry attempts are moved to the dead letter queue for manual inspection:

```rust
// List jobs in dead letter queue
let dead_jobs = service.list_jobs(
    Some("dead_letter"),
    None,
    None,
    50,
    0,
).await?;

for job in dead_jobs {
    println!("Failed job: {}", job.id);
    println!("Last error: {:?}", job.last_error);
    println!("Retry count: {}", job.retry_count);
}
```

## Testing

### Unit Tests

The crate includes comprehensive unit tests:

```bash
cargo test -p riptide-workers
```

### Integration Tests

Run integration tests (requires Redis):

```bash
# Start Redis
docker run -d -p 6379:6379 redis:7

# Run tests
cargo test -p riptide-workers --test worker_tests
```

### Example Test

```rust
use riptide_workers::{JobQueue, QueueConfig, Job, JobType};

#[tokio::test]
async fn test_job_submission() {
    let config = QueueConfig::default();
    let mut queue = JobQueue::new("redis://localhost:6379", config)
        .await
        .unwrap();

    let job = Job::new(JobType::SingleCrawl {
        url: "https://example.com".to_string(),
        options: None,
    });

    let job_id = queue.submit_job(job).await.unwrap();
    assert!(!job_id.is_nil());

    let retrieved = queue.get_job(job_id).await.unwrap();
    assert!(retrieved.is_some());
}
```

## Binary Usage (riptide-workers daemon)

The crate includes a standalone binary for running workers as a service:

### Installation

```bash
cargo install --path crates/riptide-workers
```

### Running the Daemon

```bash
# Start with defaults
riptide-workers

# Customize configuration
riptide-workers \
    --redis-url redis://localhost:6379 \
    --worker-count 8 \
    --max-batch-size 100 \
    --max-concurrency 20 \
    --wasm-path ./wasm/riptide-extractor.wasm \
    --enable-scheduler true
```

### Command-line Options

```
Options:
  --redis-url <URL>              Redis connection URL [default: redis://localhost:6379]
  --worker-count <N>             Number of worker threads [default: 4]
  --max-batch-size <N>           Maximum batch crawl size [default: 50]
  --max-concurrency <N>          Maximum concurrent operations [default: 10]
  --wasm-path <PATH>             Path to WASM extractor module
  --enable-scheduler <BOOL>      Enable job scheduler [default: true]
  -h, --help                     Print help
```

### Docker Deployment

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p riptide-workers

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/riptide-workers /usr/local/bin/
CMD ["riptide-workers"]
```

### Systemd Service

```ini
[Unit]
Description=RipTide Background Workers
After=network.target redis.service

[Service]
Type=simple
User=riptide
ExecStart=/usr/local/bin/riptide-workers \
    --redis-url redis://localhost:6379 \
    --worker-count 4
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
```

### Graceful Shutdown

The daemon supports graceful shutdown via SIGINT (Ctrl+C) or SIGTERM:

1. Stops accepting new jobs
2. Waits for current jobs to complete
3. Persists queue state to Redis
4. Cleans up worker resources
5. Exits cleanly

### Logging

Structured JSON logging to stdout:

```bash
# Set log level
RUST_LOG=info riptide-workers

# More verbose
RUST_LOG=debug riptide-workers

# Filter by module
RUST_LOG=riptide_workers=debug,riptide_core=info riptide-workers
```

## Performance Characteristics

- **Throughput**: 100-1000 jobs/second depending on job complexity
- **Latency**: Jobs typically start processing within polling interval (default 5s)
- **Memory**: ~50-200MB per worker depending on job types
- **Redis Load**: Minimal, uses pipelining and connection pooling
- **Concurrency**: Configurable per-worker job concurrency (default 4)

## Dependencies

- `tokio` - Async runtime
- `redis` - Redis client
- `serde` - Serialization
- `chrono` - Date/time handling
- `uuid` - Unique identifiers
- `cron` - Cron expression parsing
- `dashmap` - Concurrent hash maps
- `riptide-core` - Core RipTide functionality

## License

Apache-2.0

## Related Crates

- `riptide-core` - Core functionality
- `riptide-api` - HTTP API
- `riptide-streaming` - Real-time streaming
- `riptide-search` - Search functionality
