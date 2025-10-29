# Workers API Quick Reference

## Installation

```python
from riptide_sdk import RipTideClient, JobConfig, JobType, JobPriority
```

## Quick Start

```python
async with RipTideClient(base_url="http://localhost:8080") as client:
    # Submit job
    config = JobConfig(
        job_type=JobType.batch_crawl(["https://example.com"]),
        priority=JobPriority.HIGH,
    )
    job_id = await client.workers.submit_job(config)

    # Wait for result
    result = await client.workers.wait_for_job(job_id)
    print(f"Success: {result.success}")
```

## API Methods

| Method | Purpose |
|--------|---------|
| `submit_job(config)` | Submit a job to the queue |
| `list_jobs(status, job_type, limit, offset)` | List jobs with filtering |
| `get_job_status(job_id)` | Get current job status |
| `get_job_result(job_id)` | Get completed job result |
| `get_queue_stats()` | Get queue statistics |
| `get_worker_stats()` | Get worker pool statistics |
| `create_scheduled_job(config)` | Create scheduled job (cron) |
| `wait_for_job(job_id, poll_interval, timeout)` | Wait for job completion |

## Job Types

```python
# Batch crawl
JobType.batch_crawl(["url1", "url2"], options=CrawlOptions(...))

# Single crawl
JobType.single_crawl("url", options=CrawlOptions(...))

# Maintenance
JobType.maintenance("task_type", {"param": "value"})

# Custom
JobType.custom("job_name", {"data": ...})
```

## Priority Levels

```python
JobPriority.LOW       # Background tasks
JobPriority.NORMAL    # Default
JobPriority.HIGH      # Important
JobPriority.CRITICAL  # Use sparingly
```

## Job Status

```python
JobStatus.PENDING      # Waiting in queue
JobStatus.DELAYED      # Scheduled for later
JobStatus.PROCESSING   # Currently running
JobStatus.COMPLETED    # Finished successfully
JobStatus.FAILED       # Finished with error
JobStatus.RETRY        # Retrying after failure
JobStatus.DEAD_LETTER  # Max retries exceeded
```

## Examples

### Submit and Monitor

```python
job_id = await client.workers.submit_job(config)

while True:
    job = await client.workers.get_job_status(job_id)
    if job.status == JobStatus.COMPLETED:
        result = await client.workers.get_job_result(job_id)
        break
    await asyncio.sleep(2)
```

### List Pending Jobs

```python
jobs = await client.workers.list_jobs(
    status="pending",
    limit=20
)
print(f"Found {jobs.total} pending jobs")
```

### Queue Statistics

```python
stats = await client.workers.get_queue_stats()
print(f"Pending: {stats.pending}")
print(f"Processing: {stats.processing}")
```

### Scheduled Job (Daily at 2 AM)

```python
config = ScheduledJobConfig(
    name="daily_crawl",
    cron_expression="0 2 * * *",
    job_template=JobType.batch_crawl(["https://example.com"]),
)
scheduled = await client.workers.create_scheduled_job(config)
```

### With Retry Configuration

```python
from riptide_sdk.models import RetryConfig

config = JobConfig(
    job_type=JobType.batch_crawl(urls),
    retry_config=RetryConfig(
        max_attempts=3,
        initial_delay_secs=1,
        backoff_multiplier=2.0,
        max_delay_secs=300,
        use_jitter=True,
    )
)
```

### With Metadata

```python
config = JobConfig(
    job_type=job_type,
    metadata={
        "user_id": "123",
        "source": "api",
        "request_id": "abc-def",
    }
)
```

## Common Patterns

### Bulk Submission

```python
job_ids = []
for batch in url_batches:
    config = JobConfig(
        job_type=JobType.batch_crawl(batch),
        priority=JobPriority.NORMAL,
    )
    job_id = await client.workers.submit_job(config)
    job_ids.append(job_id)
```

### Wait with Timeout

```python
try:
    result = await client.workers.wait_for_job(
        job_id,
        poll_interval=2.0,
        timeout=300.0  # 5 minutes
    )
except TimeoutError:
    print("Job did not complete in time")
```

### Error Handling

```python
from riptide_sdk import APIError

try:
    job = await client.workers.get_job_status(job_id)
except APIError as e:
    if e.status_code == 404:
        print("Job not found")
    else:
        print(f"API error: {e.message}")
```

## Cron Expressions

| Expression | Meaning |
|-----------|---------|
| `* * * * *` | Every minute |
| `0 * * * *` | Every hour |
| `0 0 * * *` | Daily at midnight |
| `0 2 * * *` | Daily at 2 AM |
| `0 0 * * 0` | Weekly (Sunday) |
| `0 0 1 * *` | Monthly (1st) |
| `*/5 * * * *` | Every 5 minutes |
| `0 9-17 * * 1-5` | Weekdays 9 AM-5 PM |

## Files

- **API Implementation**: `riptide_sdk/endpoints/workers.py`
- **Data Models**: `riptide_sdk/models.py` (Worker section)
- **Documentation**: `docs/WORKERS_API.md`
- **Examples**: `examples/workers_example.py`

## See Also

- [Full Documentation](docs/WORKERS_API.md)
- [Complete Examples](examples/workers_example.py)
- [Implementation Summary](WORKERS_API_IMPLEMENTATION_SUMMARY.md)
