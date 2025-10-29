# Workers API Documentation

The Workers API provides comprehensive job submission, monitoring, and management capabilities for the RipTide worker queue system.

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [API Reference](#api-reference)
- [Job Types](#job-types)
- [Examples](#examples)
- [Error Handling](#error-handling)
- [Best Practices](#best-practices)

## Overview

The Workers API enables you to:

- ✅ Submit jobs to the worker queue
- ✅ Monitor job status and progress
- ✅ Retrieve job results
- ✅ List and filter jobs
- ✅ Get queue and worker statistics
- ✅ Create scheduled jobs with cron expressions
- ✅ Wait for job completion with polling

## Installation

```bash
pip install riptide-sdk
```

## Quick Start

```python
import asyncio
from riptide_sdk import (
    RipTideClient,
    JobConfig,
    JobType,
    JobPriority,
)

async def main():
    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Submit a batch crawl job
        config = JobConfig(
            job_type=JobType.batch_crawl(["https://example.com"]),
            priority=JobPriority.HIGH,
        )

        job_id = await client.workers.submit_job(config)
        print(f"Job submitted: {job_id}")

        # Wait for completion
        result = await client.workers.wait_for_job(job_id)
        print(f"Success: {result.success}")

asyncio.run(main())
```

## API Reference

### `client.workers.submit_job(config: JobConfig) -> str`

Submit a job to the worker queue.

**Parameters:**
- `config` (JobConfig): Job configuration including type, priority, and options

**Returns:**
- `str`: Job ID (UUID)

**Example:**
```python
config = JobConfig(
    job_type=JobType.batch_crawl(["https://example.com"]),
    priority=JobPriority.HIGH,
    retry_config=RetryConfig(
        max_attempts=3,
        backoff_multiplier=2.0
    ),
    metadata={"source": "api", "user_id": "123"}
)

job_id = await client.workers.submit_job(config)
```

---

### `client.workers.list_jobs(...) -> JobListResponse`

List jobs with filtering and pagination.

**Parameters:**
- `status` (Optional[str]): Filter by status (pending, processing, completed, failed)
- `job_type` (Optional[str]): Filter by job type (batch_crawl, single_crawl, etc.)
- `limit` (int): Maximum results (default: 50, max: 500)
- `offset` (int): Skip results (default: 0)
- `search` (Optional[str]): Search term

**Returns:**
- `JobListResponse`: Jobs list with pagination info

**Example:**
```python
# List pending jobs
result = await client.workers.list_jobs(status="pending", limit=20)
print(f"Found {result.total} pending jobs")
for job in result.jobs:
    print(f"  - {job.job_id}: {job.job_type}")
```

---

### `client.workers.get_job_status(job_id: str) -> Job`

Get status of a specific job.

**Parameters:**
- `job_id` (str): Job ID (UUID)

**Returns:**
- `Job`: Job object with current status

**Example:**
```python
job = await client.workers.get_job_status(job_id)
print(f"Status: {job.status}")
print(f"Created: {job.created_at}")
print(f"Retry count: {job.retry_count}")
if job.last_error:
    print(f"Last error: {job.last_error}")
```

---

### `client.workers.get_job_result(job_id: str) -> JobResult`

Get result of a completed job.

**Parameters:**
- `job_id` (str): Job ID (UUID)

**Returns:**
- `JobResult`: Job execution result

**Example:**
```python
result = await client.workers.get_job_result(job_id)
if result.success:
    print(f"Completed in {result.processing_time_ms}ms")
    print(f"Data: {result.data}")
else:
    print(f"Failed: {result.error}")
```

---

### `client.workers.get_queue_stats() -> QueueStats`

Get queue statistics.

**Returns:**
- `QueueStats`: Queue statistics

**Example:**
```python
stats = await client.workers.get_queue_stats()
print(f"Pending: {stats.pending}")
print(f"Processing: {stats.processing}")
print(f"Completed: {stats.completed}")
print(f"Failed: {stats.failed}")
print(f"Total: {stats.total}")
```

---

### `client.workers.get_worker_stats() -> WorkerStats`

Get worker pool statistics.

**Returns:**
- `WorkerStats`: Worker pool statistics

**Example:**
```python
stats = await client.workers.get_worker_stats()
print(f"Total workers: {stats.total_workers}")
print(f"Healthy workers: {stats.healthy_workers}")
print(f"Jobs processed: {stats.total_jobs_processed}")
print(f"Jobs failed: {stats.total_jobs_failed}")
```

---

### `client.workers.create_scheduled_job(config: ScheduledJobConfig) -> ScheduledJob`

Create a scheduled job with cron expression.

**Parameters:**
- `config` (ScheduledJobConfig): Scheduled job configuration

**Returns:**
- `ScheduledJob`: Created scheduled job

**Example:**
```python
config = ScheduledJobConfig(
    name="daily_crawl",
    cron_expression="0 2 * * *",  # Daily at 2 AM
    job_template=JobType.batch_crawl(["https://example.com"]),
    priority=JobPriority.NORMAL,
    enabled=True,
)

scheduled = await client.workers.create_scheduled_job(config)
print(f"Next execution: {scheduled.next_execution_at}")
```

---

### `client.workers.wait_for_job(...) -> JobResult`

Wait for a job to complete (convenience method).

**Parameters:**
- `job_id` (str): Job ID to wait for
- `poll_interval` (float): Seconds between status checks (default: 1.0)
- `timeout` (Optional[float]): Maximum seconds to wait (default: None = forever)

**Returns:**
- `JobResult`: Job result when completed

**Raises:**
- `TimeoutError`: If timeout is reached
- `APIError`: If job fails

**Example:**
```python
# Submit and wait
job_id = await client.workers.submit_job(config)
result = await client.workers.wait_for_job(
    job_id,
    poll_interval=2.0,
    timeout=300.0  # 5 minutes
)
```

## Job Types

### Batch Crawl

```python
job_type = JobType.batch_crawl(
    urls=["https://example.com", "https://example.org"],
    options=CrawlOptions(
        cache_mode=CacheMode.READ_WRITE,
        concurrency=10
    )
)
```

### Single Crawl

```python
job_type = JobType.single_crawl(
    url="https://example.com",
    options=CrawlOptions(concurrency=1)
)
```

### Maintenance

```python
job_type = JobType.maintenance(
    task_type="cache_cleanup",
    parameters={
        "max_age_days": 30,
        "dry_run": False
    }
)
```

### Custom

```python
job_type = JobType.custom(
    job_name="data_export",
    payload={
        "format": "json",
        "date_range": "2024-01-01/2024-12-31"
    }
)
```

## Examples

### Submit and Monitor Job

```python
async def submit_and_monitor():
    async with RipTideClient() as client:
        # Submit job
        config = JobConfig(
            job_type=JobType.batch_crawl(["https://example.com"]),
            priority=JobPriority.HIGH,
        )
        job_id = await client.workers.submit_job(config)

        # Poll for status
        while True:
            job = await client.workers.get_job_status(job_id)

            if job.status == JobStatus.COMPLETED:
                result = await client.workers.get_job_result(job_id)
                print(f"Success: {result.data}")
                break
            elif job.status == JobStatus.FAILED:
                print(f"Failed: {job.last_error}")
                break

            await asyncio.sleep(2)
```

### Bulk Job Submission

```python
async def bulk_submit():
    async with RipTideClient() as client:
        urls = [f"https://example.com/page{i}" for i in range(100)]

        # Split into batches
        batch_size = 10
        batches = [urls[i:i+batch_size] for i in range(0, len(urls), batch_size)]

        # Submit all batches
        job_ids = []
        for batch in batches:
            config = JobConfig(
                job_type=JobType.batch_crawl(batch),
                priority=JobPriority.NORMAL,
            )
            job_id = await client.workers.submit_job(config)
            job_ids.append(job_id)

        print(f"Submitted {len(job_ids)} jobs")
```

### Scheduled Job

```python
async def create_scheduled():
    async with RipTideClient() as client:
        # Daily crawl at 2 AM
        config = ScheduledJobConfig(
            name="daily_news",
            cron_expression="0 2 * * *",
            job_template=JobType.batch_crawl([
                "https://news.example.com",
            ]),
            priority=JobPriority.NORMAL,
        )

        scheduled = await client.workers.create_scheduled_job(config)
        print(f"Scheduled: {scheduled.id}")
        print(f"Next run: {scheduled.next_execution_at}")
```

### Monitor Queue

```python
async def monitor_queue():
    async with RipTideClient() as client:
        while True:
            stats = await client.workers.get_queue_stats()

            print(f"\rPending: {stats.pending} | "
                  f"Processing: {stats.processing} | "
                  f"Completed: {stats.completed} | "
                  f"Failed: {stats.failed}", end="")

            await asyncio.sleep(5)
```

## Error Handling

```python
from riptide_sdk import APIError, ValidationError, TimeoutError

async def handle_errors():
    async with RipTideClient() as client:
        try:
            result = await client.workers.wait_for_job(
                job_id,
                timeout=60.0
            )
        except TimeoutError:
            print("Job did not complete in time")
        except APIError as e:
            if e.status_code == 404:
                print("Job not found")
            else:
                print(f"API error: {e.message}")
        except ValidationError as e:
            print(f"Invalid configuration: {e}")
```

## Best Practices

### 1. Use Priorities Appropriately

```python
# Critical jobs
JobPriority.CRITICAL  # Use sparingly

# Important but not urgent
JobPriority.HIGH

# Normal operations
JobPriority.NORMAL  # Default

# Background tasks
JobPriority.LOW
```

### 2. Configure Retries

```python
config = JobConfig(
    job_type=job_type,
    retry_config=RetryConfig(
        max_attempts=3,
        initial_delay_secs=1,
        backoff_multiplier=2.0,
        max_delay_secs=300,
        use_jitter=True,
    )
)
```

### 3. Add Metadata

```python
config = JobConfig(
    job_type=job_type,
    metadata={
        "user_id": "123",
        "source": "api",
        "request_id": "abc-def",
        "environment": "production",
    }
)
```

### 4. Set Timeouts

```python
config = JobConfig(
    job_type=job_type,
    timeout_secs=300,  # 5 minutes
)
```

### 5. Monitor Queue Health

```python
async def check_queue_health():
    stats = await client.workers.get_queue_stats()

    # Alert if too many failed jobs
    if stats.failed > 100:
        alert("High failure rate")

    # Alert if queue is backing up
    if stats.pending > 1000:
        alert("Queue backlog")
```

### 6. Use Scheduled Jobs for Recurring Tasks

```python
# Instead of manually submitting daily
config = ScheduledJobConfig(
    name="daily_task",
    cron_expression="0 0 * * *",  # Daily at midnight
    job_template=job_type,
)
```

### 7. Batch Related URLs

```python
# Good: Batch related URLs together
JobType.batch_crawl([
    "https://example.com/page1",
    "https://example.com/page2",
    "https://example.com/page3",
])

# Less efficient: Individual jobs
# for url in urls:
#     JobType.single_crawl(url)
```

## Cron Expression Reference

| Expression | Description |
|-----------|-------------|
| `* * * * *` | Every minute |
| `0 * * * *` | Every hour |
| `0 0 * * *` | Daily at midnight |
| `0 2 * * *` | Daily at 2 AM |
| `0 0 * * 0` | Weekly on Sunday |
| `0 0 1 * *` | Monthly on 1st |
| `*/5 * * * *` | Every 5 minutes |
| `0 9-17 * * 1-5` | Weekdays 9 AM-5 PM |

## Job Status Flow

```
PENDING → DELAYED (if scheduled_at set)
   ↓
PROCESSING → COMPLETED (success)
   ↓
FAILED → RETRY (if retries remaining)
   ↓
DEAD_LETTER (max retries exceeded)
```

## Performance Tuning

### Concurrency

```python
# For I/O-bound tasks, increase concurrency
options = CrawlOptions(concurrency=20)

# For CPU-bound tasks, keep it low
options = CrawlOptions(concurrency=5)
```

### Batch Size

```python
# Good balance: 10-50 URLs per batch
urls_batch = urls[0:25]
JobType.batch_crawl(urls_batch)
```

### Polling Interval

```python
# For time-critical jobs
wait_for_job(job_id, poll_interval=0.5)

# For background jobs
wait_for_job(job_id, poll_interval=5.0)
```

## See Also

- [Main SDK Documentation](../README.md)
- [Examples](../examples/workers_example.py)
- [API Reference](./API_REFERENCE.md)
